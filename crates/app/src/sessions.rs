//! The headless sessions the window starts (D-16), held until EP-13's tile shows their transcripts
//! and EP-14's answers their requests (the owner's choice at EP-10): each session's state, its last
//! line and what it waits for, in plain lines in the Sessions pane with Stop and Kill; a request
//! waits, listed read-only in the Decisions pane. The CLI's preflight is asked on a worker thread
//! and kept an hour (D-15), and a start waits its turn and its spacing on one too (D-29), so the
//! frame never blocks. Closing the window drops the handles, which kills whatever still runs with
//! its process tree (D-16 addendum).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Local};
use crw_core::session::{Ended, Event, Init, Launch, Outcome, Preflight, Request, Session};
use eframe::egui::{self, RichText};

use crate::board::{Action, Board};
use crate::style::{self, icons};

/// How often the window looks at a running session's events.
const POLL: Duration = Duration::from_millis(250);

/// The notices a session keeps, the latest last.
const NOTICES: usize = 5;

/// The longest line of a session's text shown.
const LINE: usize = 220;

/// A headless session the window started.
pub struct Live {
    /// The window's own number for it, for the buttons.
    pub key: u64,
    pub project: String,
    pub ep: String,
    pub model: String,
    /// The effort in words, as the prompt says it.
    pub effort: String,
    pub started: DateTime<Local>,
    pub session_id: String,
    /// The handle; `None` once the session has exited.
    session: Option<Session>,
    /// What the launch asked for and the CLI could not take.
    pub dropped: Vec<String>,
    pub init: Option<Init>,
    /// The last line of text, or the last tool call, in one line.
    pub last: String,
    streaming: String,
    /// The requests the session waits on (D-19); answered from EP-14.
    pub waiting: Vec<Request>,
    pub notices: Vec<String>,
    pub denied: usize,
    pub unreadable: usize,
    pub stopping: bool,
    pub ended: Option<Ended>,
}

impl Live {
    fn take(&mut self, event: Event) {
        match event {
            Event::Init(init) => self.init = Some(init),
            Event::Handshake(Ok(())) => {}
            Event::Handshake(Err(e)) => {
                self.notice(format!("the control protocol's handshake was refused: {e}"));
            }
            Event::TextDelta(text) => {
                self.streaming.push_str(&text);
                if let Some(line) = last_line(&self.streaming) {
                    self.last = line;
                }
            }
            Event::Text(text) => {
                self.streaming.clear();
                if let Some(line) = last_line(&text) {
                    self.last = line;
                }
            }
            Event::ToolUse(tool) => self.last = tool.summary,
            Event::Request(request) => self.waiting.push(request),
            Event::Withdrawn(id) => self.waiting.retain(|request| request.id != id),
            Event::Denied(denied) => {
                self.denied += 1;
                self.last = format!("refused: {} ({})", denied.tool, denied.message);
            }
            Event::Result(_) => {}
            Event::Notice(text) => self.notice(text),
            Event::Unreadable(_) => self.unreadable += 1,
            Event::Exited(ended) => {
                self.waiting.clear();
                self.ended = Some(ended);
                self.session = None;
            }
        }
    }

    fn notice(&mut self, text: String) {
        if self.notices.len() == NOTICES {
            self.notices.remove(0);
        }
        self.notices.push(text);
    }

    /// Takes in what the session said since the last frame.
    fn drain(&mut self) {
        let mut events = Vec::new();
        let mut gone = false;
        if let Some(session) = &self.session {
            loop {
                match session.events().try_recv() {
                    Ok(event) => events.push(event),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        gone = true;
                        break;
                    }
                }
            }
        }
        for event in events {
            self.take(event);
        }
        if gone && self.ended.is_none() {
            self.notice("the session's thread ended without saying how".to_owned());
            self.session = None;
        }
    }

    pub fn is_running(&self) -> bool {
        self.ended.is_none() && self.session.is_some()
    }

    /// The state in words: running, waiting for an answer, stopping, or how it ended.
    pub fn state(&self) -> String {
        match &self.ended {
            Some(ended) => format!("ended: {}", ended.outcome.as_str()),
            None if self.session.is_none() => "gone".to_owned(),
            None if self.stopping => "stopping".to_owned(),
            None if !self.waiting.is_empty() => "waiting for an answer".to_owned(),
            None => "running".to_owned(),
        }
    }
}

/// The last line of `text` that holds anything, cut to [`LINE`] characters.
fn last_line(text: &str) -> Option<String> {
    let line = text
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    Some(if line.chars().count() > LINE {
        let cut: String = line.chars().take(LINE - 1).collect();
        format!("{cut}…")
    } else {
        line.to_owned()
    })
}

/// The CLI's answers before a launch, asked off the UI thread (D-15).
enum Asked {
    Not,
    Asking {
        program: PathBuf,
        receiver: Receiver<Preflight>,
    },
    Ready(Arc<Preflight>),
}

/// A start on its way: the runner waits for its turn and the spacing on a worker thread.
struct Starting {
    receiver: Receiver<Result<Session, String>>,
    project: String,
    ep: String,
    model: String,
    effort: String,
}

/// What became of a start.
pub enum Started {
    Running(String),
    Refused(String),
}

/// The window's sessions and what their launches need from the CLI.
pub struct Sessions {
    /// The latest first.
    pub live: Vec<Live>,
    starting: Option<Starting>,
    asked: Asked,
    /// A screenshot run's stand-in for the CLI's answers: the real CLI is never asked there.
    pub canned: Option<Arc<Preflight>>,
    next_key: u64,
}

impl Default for Sessions {
    fn default() -> Self {
        Self {
            live: Vec::new(),
            starting: None,
            asked: Asked::Not,
            canned: None,
            next_key: 1,
        }
    }
}

impl Sessions {
    /// The CLI's answers about `program`: at once when they are fresh, else asked on a worker
    /// thread (three children, one at a time) and `None` until they come.
    pub fn preflight(&mut self, program: &Path, ctx: &egui::Context) -> Option<Arc<Preflight>> {
        if let Some(canned) = &self.canned {
            return Some(Arc::clone(canned));
        }
        self.poll_preflight();
        match &self.asked {
            Asked::Ready(preflight) if preflight.program == program && preflight.is_fresh() => {
                return Some(Arc::clone(preflight));
            }
            Asked::Asking { program: asked, .. } if asked == program => return None,
            _ => {}
        }
        let (sender, receiver) = mpsc::channel();
        let (asked, ctx) = (program.to_owned(), ctx.clone());
        let spawned = thread::Builder::new()
            .name("preflight".to_owned())
            .spawn(move || {
                let _ = sender.send(Preflight::run(&asked));
                ctx.request_repaint();
            });
        self.asked = match spawned {
            Ok(_) => Asked::Asking {
                program: program.to_owned(),
                receiver,
            },
            Err(_) => Asked::Not,
        };
        None
    }

    fn poll_preflight(&mut self) {
        if let Asked::Asking { receiver, .. } = &self.asked {
            match receiver.try_recv() {
                Ok(preflight) => self.asked = Asked::Ready(Arc::new(preflight)),
                Err(TryRecvError::Disconnected) => self.asked = Asked::Not,
                Err(TryRecvError::Empty) => {}
            }
        }
    }

    pub fn is_asking(&self) -> bool {
        self.canned.is_none() && matches!(self.asked, Asked::Asking { .. })
    }

    /// Drops the answers, so that the next launch asks again (after a refused start).
    pub fn forget_preflight(&mut self) {
        if !self.is_asking() {
            self.asked = Asked::Not;
        }
    }

    pub fn is_starting(&self) -> bool {
        self.starting.is_some()
    }

    /// Starts `launch` on a worker thread, which waits for the turn and the spacing (D-29).
    pub fn start(
        &mut self,
        preflight: Arc<Preflight>,
        launch: Launch,
        effort: String,
        ctx: &egui::Context,
    ) {
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        let (project, ep, model) = (
            launch.project.clone(),
            launch.ep.clone().unwrap_or_default(),
            launch.model.clone(),
        );
        let spawned = thread::Builder::new()
            .name("session start".to_owned())
            .spawn(move || {
                let _ = sender.send(Session::start(&preflight, launch));
                ctx.request_repaint();
            });
        if spawned.is_ok() {
            self.starting = Some(Starting {
                receiver,
                project,
                ep,
                model,
                effort,
            });
        }
    }

    /// Takes in a start that has come back and every session's events; asks for the next frame
    /// while one runs. Returns what became of a start, when one came back.
    pub fn receive(&mut self, ctx: &egui::Context) -> Option<Started> {
        self.poll_preflight();
        let mut started = None;
        if let Some(starting) = &self.starting {
            match starting.receiver.try_recv() {
                Ok(result) => {
                    let starting = self.starting.take().expect("a start was waited on");
                    started = Some(match result {
                        Ok(session) => {
                            let text = format!(
                                "Started {} {} headless with {} at {} effort: the Sessions pane \
                                 lists it, and closing the window stops it.",
                                starting.project, starting.ep, starting.model, starting.effort
                            );
                            self.live.insert(
                                0,
                                Live {
                                    key: self.next_key,
                                    project: starting.project,
                                    ep: starting.ep,
                                    model: starting.model,
                                    effort: starting.effort,
                                    started: Local::now(),
                                    session_id: session.session_id.clone(),
                                    dropped: session.dropped.clone(),
                                    session: Some(session),
                                    init: None,
                                    last: String::new(),
                                    streaming: String::new(),
                                    waiting: Vec::new(),
                                    notices: Vec::new(),
                                    denied: 0,
                                    unreadable: 0,
                                    stopping: false,
                                    ended: None,
                                },
                            );
                            self.next_key += 1;
                            Started::Running(text)
                        }
                        Err(why) => Started::Refused(why),
                    });
                }
                Err(TryRecvError::Disconnected) => {
                    self.starting = None;
                    started = Some(Started::Refused(
                        "the start's thread ended without an answer".to_owned(),
                    ));
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        for live in &mut self.live {
            live.drain();
        }
        if self.is_starting() || self.live.iter().any(Live::is_running) {
            ctx.request_repaint_after(POLL);
        }
        started
    }

    /// What the row of `project` says of the window's sessions; `None` when none runs.
    pub fn summary(&self, project: &str) -> Option<String> {
        let running: Vec<String> = self
            .live
            .iter()
            .filter(|live| live.is_running() && live.project.eq_ignore_ascii_case(project))
            .map(|live| {
                format!(
                    "a headless {} started here at {} is {}",
                    live.ep,
                    live.started.format("%H:%M"),
                    live.state()
                )
            })
            .collect();
        let starting = self
            .starting
            .as_ref()
            .filter(|starting| starting.project.eq_ignore_ascii_case(project))
            .map(|starting| format!("a headless {} is starting", starting.ep));
        let all: Vec<String> = starting.into_iter().chain(running).collect();
        (!all.is_empty()).then(|| all.join("; "))
    }

    fn find(&mut self, key: u64) -> Option<&mut Live> {
        self.live.iter_mut().find(|live| live.key == key)
    }

    pub fn stop(&mut self, key: u64) {
        if let Some(live) = self.find(key)
            && let Some(session) = &live.session
            && session.stop()
        {
            live.stopping = true;
        }
    }

    pub fn kill(&mut self, key: u64) {
        if let Some(live) = self.find(key)
            && let Some(session) = &live.session
        {
            session.kill();
            live.stopping = true;
        }
    }

    /// Forgets an ended session (the ledger keeps its row).
    pub fn dismiss(&mut self, key: u64) {
        self.live
            .retain(|live| live.key != key || live.is_running());
    }

    pub fn waiting(&self) -> usize {
        self.live.iter().map(|live| live.waiting.len()).sum()
    }
}

impl Board {
    /// The Sessions pane until EP-13's tile: the window's headless sessions in plain lines.
    pub fn sessions_pane(&mut self, ui: &mut egui::Ui) {
        if self.sessions.live.is_empty() && !self.sessions.is_starting() {
            placeholder(
                ui,
                "The headless sessions started from the launch dialog (S, or a row's Start \
                 button) are listed here, with Stop and Kill; their transcripts arrive at EP-13.",
            );
            return;
        }
        let mut actions = Vec::new();
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if self.sessions.is_starting() {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Starting a session: it waits its turn, two seconds after the last start.");
                    });
                    ui.add_space(6.0);
                }
                for live in &self.sessions.live {
                    session_card(ui, live, &mut actions);
                    ui.add_space(6.0);
                }
            });
        self.actions.extend(actions);
    }

    /// The Decisions pane until EP-14's tile: what the window's sessions wait on, read-only.
    pub fn decisions_pane(&mut self, ui: &mut egui::Ui) {
        if self.sessions.waiting() == 0 {
            placeholder(
                ui,
                "A headless session's questions and permission requests are listed here while it \
                 waits; answering them arrives at EP-14, and until then Stop in the Sessions pane \
                 ends a session that waits.",
            );
            return;
        }
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(
                        "Answering arrives at EP-14: until then a session waits here, and Stop in \
                         the Sessions pane ends it.",
                    )
                    .color(style::amber(ui)),
                );
                ui.add_space(6.0);
                for live in &self.sessions.live {
                    for request in &live.waiting {
                        request_card(ui, live, request);
                        ui.add_space(6.0);
                    }
                }
            });
    }
}

fn placeholder(ui: &mut egui::Ui, text: &str) {
    ui.add_space(24.0);
    ui.vertical_centered(|ui| ui.label(RichText::new(text).weak()));
}

fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

fn session_card(ui: &mut egui::Ui, live: &Live, actions: &mut Vec<Action>) {
    card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("{} {}", live.project, live.ep)).strong());
            let state = live.state();
            let colour = match &live.ended {
                Some(ended) if ended.outcome == Outcome::Completed => style::green(ui),
                Some(_) => style::red(ui),
                None if !live.waiting.is_empty() => style::amber(ui),
                None => ui.visuals().weak_text_color(),
            };
            style::chip(ui, &state, colour);
            ui.label(
                RichText::new(format!(
                    "{} · {} · started {}",
                    live.model,
                    live.effort,
                    live.started.format("%H:%M")
                ))
                .weak(),
            );
        });
        if !live.last.is_empty() {
            ui.add(egui::Label::new(RichText::new(&live.last).weak()).truncate());
        }
        if let Some(ended) = &live.ended {
            ui.label(RichText::new(ending(ended)).weak());
            if ended.outcome == Outcome::Failed
                && let Some(line) = ended.stderr.last()
            {
                ui.label(RichText::new(line).color(style::red(ui)));
            }
        }
        for note in live.dropped.iter().chain(&live.notices) {
            ui.label(RichText::new(note).small().color(style::amber(ui)));
        }
        let mut counts = Vec::new();
        if live.denied > 0 {
            counts.push(format!("{} tool uses refused", live.denied));
        }
        if live.unreadable > 0 {
            counts.push(format!("{} lines unread", live.unreadable));
        }
        if let Some(init) = &live.init
            && let Some(version) = &init.version
        {
            counts.push(format!("claude {version}"));
        }
        counts.push(format!("session {}", live.session_id));
        ui.label(RichText::new(counts.join(" · ")).small().weak());
        ui.horizontal(|ui| {
            if live.is_running() {
                if ui
                    .add_enabled(!live.stopping, egui::Button::new("Stop"))
                    .on_hover_text(
                        "Interrupt the turn and close the session's input; the job is killed \
                         if it has not exited within 30 seconds",
                    )
                    .clicked()
                {
                    actions.push(Action::StopSession(live.key));
                }
                if ui
                    .button(style::icon(icons::ICON_CLOSE, Some(style::red(ui))))
                    .on_hover_text("Kill the session and its whole process tree at once")
                    .clicked()
                {
                    actions.push(Action::KillSession(live.key));
                }
            } else if ui
                .button("Dismiss")
                .on_hover_text("Take it off this list; the ledger keeps its row")
                .clicked()
            {
                actions.push(Action::DismissSession(live.key));
            }
        });
    });
}

/// How a session ended, in one line: the outcome, the exit code, the turns, the time and the
/// CLI's cost estimate (a subscription is not billed in dollars, D-23).
fn ending(ended: &Ended) -> String {
    let mut parts = vec![format!("ended {}", ended.outcome.as_str())];
    if let Some(code) = ended.exit_code {
        parts.push(format!("exit {code}"));
    }
    if let Some(result) = &ended.result {
        parts.push(format!("{} turns", result.turns));
        parts.push(format!("{} s", result.duration_ms / 1000));
        parts.push(format!("an estimated ${:.3}", result.cost_usd));
        if let Some(text) = result.text.as_deref().and_then(last_line) {
            parts.push(text);
        }
    }
    parts.join(" · ")
}

fn request_card(ui: &mut egui::Ui, live: &Live, request: &Request) {
    card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("{} {}", live.project, live.ep)).strong());
            style::chip(ui, &request.tool, style::amber(ui));
        });
        if request.questions.is_empty() {
            ui.label(&request.summary);
        }
        if let Some(reason) = &request.reason {
            ui.label(RichText::new(reason).small().weak());
        }
        for question in &request.questions {
            ui.add_space(4.0);
            ui.label(RichText::new(&question.text).strong());
            for choice in &question.options {
                let text = match &choice.description {
                    Some(description) => format!("○ {} — {description}", choice.label),
                    None => format!("○ {}", choice.label),
                };
                ui.label(RichText::new(text).weak());
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crw_core::session::{Finish, ToolUse};

    use super::*;

    fn live() -> Live {
        Live {
            key: 1,
            project: "alpha".to_owned(),
            ep: "EP-3".to_owned(),
            model: "fable".to_owned(),
            effort: "xhigh".to_owned(),
            started: Local::now(),
            session_id: "0e1d2c3b".to_owned(),
            session: None,
            dropped: Vec::new(),
            init: None,
            last: String::new(),
            streaming: String::new(),
            waiting: Vec::new(),
            notices: Vec::new(),
            denied: 0,
            unreadable: 0,
            stopping: false,
            ended: None,
        }
    }

    #[test]
    fn a_sessions_events_make_its_lines() {
        let mut live = live();
        live.take(Event::TextDelta("Reading ".to_owned()));
        live.take(Event::TextDelta("the brief.\nThen".to_owned()));
        assert_eq!(live.last, "Then");
        live.take(Event::Text("Reading the brief.\nThen the code.".to_owned()));
        assert_eq!(live.last, "Then the code.");
        live.take(Event::ToolUse(ToolUse {
            id: "toolu_01".to_owned(),
            tool: "Bash".to_owned(),
            summary: "Bash cargo test".to_owned(),
        }));
        assert_eq!(live.last, "Bash cargo test");
        let request = Request {
            id: "req-1".to_owned(),
            tool: "Bash".to_owned(),
            summary: "Bash git push".to_owned(),
            reason: None,
            questions: Vec::new(),
        };
        live.take(Event::Request(request.clone()));
        assert_eq!(live.waiting.len(), 1);
        live.take(Event::Withdrawn("req-1".to_owned()));
        assert!(live.waiting.is_empty());
        live.take(Event::Request(request));
        for n in 0..7 {
            live.take(Event::Notice(format!("notice {n}")));
        }
        assert_eq!(live.notices.len(), NOTICES);
        assert_eq!(live.notices.last().map(String::as_str), Some("notice 6"));
        let result = Finish {
            subtype: "success".to_owned(),
            is_error: false,
            text: Some("EP-3 is done.".to_owned()),
            turns: 4,
            cost_usd: 0.42,
            duration_ms: 4200,
            stop_reason: Some("end_turn".to_owned()),
            deferred: None,
            denials: 0,
            errors: Vec::new(),
        };
        live.take(Event::Exited(Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: Some(result),
            stderr: Vec::new(),
        }));
        // An exit takes the requests with it.
        assert!(live.waiting.is_empty());
        assert_eq!(live.state(), "ended: completed");
        assert!(!live.is_running());
        assert_eq!(
            ending(live.ended.as_ref().unwrap()),
            "ended completed · exit 0 · 4 turns · 4 s · an estimated $0.420 · EP-3 is done."
        );
    }

    #[test]
    fn the_last_line_is_the_last_with_text() {
        assert_eq!(last_line("one\ntwo\n\n  "), Some("two".to_owned()));
        assert_eq!(last_line("  \n"), None);
        let long = "x".repeat(LINE + 10);
        assert_eq!(last_line(&long).unwrap().chars().count(), LINE);
    }
}
