//! The Sessions tile (D-16, D-25; EP-13): one tab per headless session the window started, with
//! its transcript as it streams (`transcript`), Stop and Kill while it runs and Resume with a
//! message once it has ended; and an All tab that lists every session, the window's, those Claude
//! Code reports elsewhere (EP-12) and the ledger's recent rows, with filters by project, mode and
//! running only. An ended session's tab and a ledger row with a session id have Revert…, which
//! opens the preview of undoing it (`revert`, EP-15c), and such a row has Resume in VS Code too,
//! which opens its session in a tab of the project's VS Code window (`visual`, EP-35). The CLI's
//! preflight is asked on a worker thread and kept an hour (D-15), and a
//! start waits its turn and its spacing on one too (D-29), so the frame never blocks. Quitting the
//! window drops the handles, which kills whatever still runs with its process tree (D-16
//! addendum); closing it only hides it in the tray (D-29; EP-15). What a session waits on is answered in the Decisions tile (`decisions`, EP-14):
//! the answer goes through the session's handle, and a run deferred at its time limit or by the
//! defer route's hook resumes with the owner's answer (D-19). A run's requests and its end are
//! news for the notifications (`toasts`, EP-15).

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Utc};
use crw_core::autopilot::{self, Cap};
use crw_core::chat;
use crw_core::ledger::{self, Ledger, RowState, SessionRow};
use crw_core::roadmap::Size;
use crw_core::session::{
    Answered, Deferred, Denied, Ended, Event, Init, Launch, Outcome, Preflight, Request, Response,
    Route, Session,
};
use crw_core::sessions::{Seen, Status as SeenStatus, VISUAL_CLAIM};
use eframe::egui::{self, Color32, RichText};
use serde::{Deserialize, Serialize};

use crate::board::{Action, Board};
use crate::decisions::{self, Draft};
use crate::portfolio::ProjectRow;
use crate::style::{self, icons};
use crate::transcript::Transcript;

/// The refusals a session keeps for the Decisions tile; the count goes on.
const REFUSALS: usize = 20;

/// How often the window looks at a running session's events.
const POLL: Duration = Duration::from_millis(250);

/// The longest line of a session's text shown.
const LINE: usize = 220;

/// The ledger's rows the All tab reads, and how often while the pane shows.
const HISTORY_ROWS: usize = 200;
const HISTORY_EVERY: Duration = Duration::from_secs(30);

/// The modes a session may have, in the order the filters show them (D-16, D-26).
const MODES: [&str; 3] = ["headless", "visual", "manual"];

/// A headless session the window started.
pub struct Live {
    /// The window's own number for it, for the buttons and its tab.
    pub key: u64,
    pub project: String,
    pub ep: String,
    pub model: String,
    /// The effort in words, as the prompt says it.
    pub effort: String,
    pub started: DateTime<Local>,
    pub session_id: String,
    /// The handle; `None` once the run has exited.
    session: Option<Session>,
    /// What the launch asked for and the CLI could not take.
    pub dropped: Vec<String>,
    pub init: Option<Init>,
    /// The last line of text, or the last tool call, in one line: the All tab's.
    pub last: String,
    pub transcript: Transcript,
    /// The requests the session waits on, the Decisions tile's to answer (D-19).
    pub waiting: Vec<Waiting>,
    /// The request an ended run left open: deferred at the time limit or by the defer route's
    /// hook, answered by resuming the session (EP-14).
    pub open: Option<Open>,
    pub denied: usize,
    /// The latest of the tool uses refused without asking, for the Decisions tile.
    pub refusals: Vec<Denied>,
    pub unreadable: usize,
    pub stopping: bool,
    pub ended: Option<Ended>,
    /// How many runs it has had: one, and one more for each resume.
    pub runs: u32,
    /// The message the owner types to resume it.
    pub draft: String,
    /// The answer forms of its requests, by request id.
    pub drafts: HashMap<String, Draft>,
    /// The control protocol's handshake was refused in a run (D-19's fallback).
    pub handshake_refused: bool,
    /// The run's cap while its project's autopilot is on (D-22; EP-19): it stops there.
    pub cap: Option<Cap>,
    /// The working time after which its cap stopped it.
    pub capped: Option<Duration>,
    /// The autopilot's chain started it (D-22).
    pub chained: bool,
    /// Its prompt or a resume told it to commit, so the Commit button is not offered.
    pub commits: bool,
    /// A migration of the project's roadmap (D-9; EP-21a): no gate and no chain follow it, and
    /// it offers no Commit, whose message records a brief's hash; the owner commits it.
    pub migration: bool,
    /// What it was launched with: a resume runs the same with `--resume` (EP-13).
    launch: Launch,
    /// The CLI it ran, whose preflight a resume asks, and that CLI's version.
    program: PathBuf,
    version: Option<String>,
    /// A screenshot run's stand-in: shown running, with no process behind it.
    canned: bool,
    /// What the notifications have yet to hear of (EP-15).
    news: Vec<News>,
}

/// What a window's run did that may call the owner (D-21; EP-15), for the notifications.
#[derive(Debug, Clone, PartialEq)]
pub enum News {
    /// It asks: a question's text, or the call a permission is asked for, in one line.
    Asked(String),
    /// Its run ended: how, and in one line what it came to, why it failed, or the request a
    /// deferred run left open.
    Ended(Outcome, String),
}

/// A request a window's session waits on (D-19), and since when.
#[derive(Debug, Clone, PartialEq)]
pub struct Waiting {
    pub request: Request,
    pub since: Instant,
}

/// A request an ended run left open (D-19; EP-14).
#[derive(Debug, Clone, PartialEq)]
pub struct Open {
    pub request: Request,
    /// The call the defer route's hook deferred, which the answer's hook hands back; none for a
    /// request the time limit deferred, whose answer is the resume's message.
    pub deferred: Option<Deferred>,
    /// When the run was deferred.
    pub at: DateTime<Local>,
    pub since: Instant,
}

impl Open {
    fn now(request: Request, deferred: Option<Deferred>) -> Self {
        Self {
            request,
            deferred,
            at: Local::now(),
            since: Instant::now(),
        }
    }
}

impl Live {
    fn new(key: u64, starting: &Starting, session: Session) -> Self {
        let mut transcript = Transcript::default();
        transcript.prompt(&starting.message, false);
        Self {
            key,
            project: starting.launch.project.clone(),
            ep: starting.launch.ep.clone().unwrap_or_default(),
            model: starting.launch.model.clone(),
            effort: starting.effort.clone(),
            started: Local::now(),
            session_id: session.session_id.clone(),
            dropped: session.dropped.clone(),
            session: Some(session),
            init: None,
            last: String::new(),
            transcript,
            waiting: Vec::new(),
            open: None,
            denied: 0,
            refusals: Vec::new(),
            unreadable: 0,
            stopping: false,
            ended: None,
            runs: 1,
            draft: String::new(),
            drafts: HashMap::new(),
            handshake_refused: false,
            cap: starting
                .run
                .capped
                .then(|| Cap::new(starting.run.size, Instant::now())),
            capped: None,
            chained: starting.run.chained,
            commits: starting.run.commits,
            migration: starting.run.migration,
            launch: starting.launch.clone(),
            program: starting.program.clone(),
            version: starting.version.clone(),
            canned: false,
            news: Vec::new(),
        }
    }

    /// A session as a screenshot run shows it: running, its transcript made of `events`; a
    /// test's stand-in too (EP-24).
    #[cfg(any(test, feature = "screenshot"))]
    pub fn canned(key: u64, launch: Launch, effort: &str, events: Vec<Event>) -> Self {
        let mut live = Self {
            key,
            project: launch.project.clone(),
            ep: launch.ep.clone().unwrap_or_default(),
            model: launch.model.clone(),
            effort: effort.to_owned(),
            started: Local::now() - chrono::TimeDelta::minutes(18),
            session_id: "3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f".to_owned(),
            session: None,
            dropped: Vec::new(),
            init: None,
            last: String::new(),
            transcript: Transcript::default(),
            waiting: Vec::new(),
            open: None,
            denied: 0,
            refusals: Vec::new(),
            unreadable: 0,
            stopping: false,
            ended: None,
            runs: 1,
            draft: String::new(),
            drafts: HashMap::new(),
            handshake_refused: false,
            cap: None,
            capped: None,
            chained: false,
            commits: false,
            migration: false,
            program: PathBuf::from("claude.exe"),
            version: None,
            canned: true,
            news: Vec::new(),
            launch,
        };
        live.transcript.prompt(&live.launch.prompt, false);
        for event in events {
            live.take(event);
        }
        live
    }

    /// A screenshot run's canned session ended: its run deferred at the time limit.
    #[cfg(any(test, feature = "screenshot"))]
    pub fn end_canned(&mut self, ended: Ended) {
        self.take(Event::Exited(ended));
    }

    /// A chat conversation's run (D-27; EP-24): the Chat tile's, not a tab of the Sessions pane.
    pub fn is_chat(&self) -> bool {
        self.launch.conversation.is_some()
    }

    /// The conversation's title, for a chat run.
    pub fn conversation(&self) -> Option<&str> {
        self.launch.conversation.as_deref()
    }

    /// Takes in one of the runner's events.
    pub(crate) fn take(&mut self, event: Event) {
        self.transcript.take(&event);
        match event {
            Event::Init(init) => self.init = Some(init),
            // A chat session reads and nothing else (D-27): its permission requests are refused
            // here at once, and only its questions reach the owner (EP-24).
            Event::Request(request) if self.is_chat() && request.questions.is_empty() => {
                let sent = self.session.as_ref().is_some_and(|session| {
                    session.respond(&request.id, Response::Deny(chat::REFUSAL.to_owned()))
                });
                let words = if sent {
                    "refused by the window: the chat session is read-only"
                } else {
                    "refused by the window: the chat session is read-only (the session had ended)"
                };
                self.transcript.answered(&request.id, words);
                self.denied += 1;
                self.last = format!("refused: {}", request.summary);
            }
            Event::TextDelta(_) => {
                if let Some(line) = last_line(&self.transcript.streaming) {
                    self.last = line;
                }
            }
            Event::Text(text) => {
                if let Some(line) = last_line(&text) {
                    self.last = line;
                }
            }
            Event::ToolUse(tool) => self.last = tool.summary,
            Event::Request(request) => {
                self.news.push(News::Asked(request.summary.clone()));
                self.waiting.push(Waiting {
                    request,
                    since: Instant::now(),
                });
            }
            Event::Withdrawn(id) => {
                self.waiting.retain(|waiting| waiting.request.id != id);
                self.drafts.remove(&id);
            }
            // The time limit refused it: the run is deferred with it open (EP-14).
            Event::TimedOut(id) => {
                if let Some(at) = self.waiting.iter().position(|w| w.request.id == id) {
                    let waiting = self.waiting.remove(at);
                    self.open = Some(Open::now(waiting.request, None));
                }
            }
            Event::Result(finish) => {
                if let Some(deferred) = finish.deferred {
                    self.open = Some(Open::now(deferred.request(), Some(deferred)));
                }
            }
            Event::Denied(denied) => {
                self.denied += 1;
                self.last = format!("refused: {} ({})", denied.tool, denied.message);
                if self.refusals.len() == REFUSALS {
                    self.refusals.remove(0);
                }
                self.refusals.push(denied);
            }
            Event::Handshake(Err(_)) => self.handshake_refused = true,
            Event::Unreadable(_) => self.unreadable += 1,
            Event::Exited(ended) => {
                // A chat's reply is read in its tile: its end raises no toast (EP-24).
                if ended.outcome != Outcome::Stopped && !self.is_chat() {
                    let line = ended_line(&ended, self.open.as_ref());
                    self.news.push(News::Ended(ended.outcome, line));
                }
                self.waiting.clear();
                self.ended = Some(ended);
                self.session = None;
                self.stopping = false;
            }
            Event::Handshake(Ok(())) | Event::ToolResult(_) | Event::Notice(_) => {}
        }
    }

    /// What the autopilot asked of its runs, for the next one (EP-19).
    fn run(&self) -> Run {
        Run {
            capped: self.cap.is_some(),
            size: self.cap.and_then(|cap| cap.size),
            chained: self.chained,
            commits: self.commits,
            // A migration resumed stays one (EP-21a).
            migration: self.migration,
        }
    }

    /// A migration that completed (EP-21a): its tab holds the session's report and its commit is
    /// the owner's, so a restart nobody asked for keeps the tab, as it keeps one offering Commit.
    pub fn finished_migration(&self) -> bool {
        self.migration
            && self
                .ended
                .as_ref()
                .is_some_and(|ended| ended.outcome == Outcome::Completed)
    }

    /// Whether its tab offers Commit (D-22): the run completed under the rule that stops before
    /// committing, and nothing told it to commit since.
    pub fn offers_commit(&self) -> bool {
        !self.commits
            && !self.migration
            && self
                .ended
                .as_ref()
                .is_some_and(|ended| ended.outcome == Outcome::Completed)
    }

    /// `<project> <EP>`, as its tab says it; a conversation's `chat: <title>`.
    pub fn title(&self) -> String {
        match self.conversation() {
            Some(title) => format!("chat: {title}"),
            None => format!("{} {}", self.project, self.ep),
        }
    }

    /// How long its requests may wait before the run is deferred (D-19).
    pub fn launch_timeout(&self) -> Option<Duration> {
        self.launch.answer_timeout
    }

    /// The CLI it ran, whose preflight a resume asks.
    pub fn program(&self) -> &Path {
        &self.program
    }

    /// Takes in what the session said since the last frame; `true` when its run ended.
    fn drain(&mut self) -> bool {
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
        let mut ended = false;
        for event in events {
            ended |= matches!(event, Event::Exited(_));
            self.take(event);
        }
        if gone && self.ended.is_none() {
            self.take(Event::Notice(
                "the session's thread ended without saying how".to_owned(),
            ));
            self.session = None;
            ended = true;
        }
        ended
    }

    pub fn is_running(&self) -> bool {
        self.ended.is_none() && (self.session.is_some() || self.canned)
    }

    /// The state in words: running, waiting for an answer, stopping, or how it ended.
    pub fn state(&self) -> String {
        match &self.ended {
            Some(ended) => format!("ended: {}", ended.outcome.as_str()),
            None if !self.is_running() => "gone".to_owned(),
            None if self.stopping => "stopping".to_owned(),
            None if !self.waiting.is_empty() => "waiting for an answer".to_owned(),
            None => "running".to_owned(),
        }
    }

    /// The colour of its state: at work in blue, waiting in amber, ended well in green.
    fn colour(&self, ui: &egui::Ui) -> Color32 {
        match &self.ended {
            Some(ended) if ended.outcome == Outcome::Completed => style::green(ui),
            Some(ended) if ended.outcome == Outcome::Deferred => style::amber(ui),
            Some(_) => style::red(ui),
            None if !self.is_running() || self.stopping => ui.visuals().weak_text_color(),
            None if !self.waiting.is_empty() => style::amber(ui),
            None => style::blue(ui),
        }
    }
}

/// A run's end in one line, for its toast (EP-15): the result's gist when it completed (EP-20a),
/// the request a deferred run left open, or why it failed.
fn ended_line(ended: &Ended, open: Option<&Open>) -> String {
    match ended.outcome {
        Outcome::Completed => ended
            .result
            .as_ref()
            .and_then(|finish| finish.text.as_deref())
            .and_then(result_line)
            .unwrap_or_default(),
        Outcome::Deferred => open
            .map(|open| open.request.summary.clone())
            .unwrap_or_default(),
        Outcome::Failed | Outcome::Stopped => {
            let result = ended.result.as_ref();
            result
                .and_then(|finish| finish.errors.first().cloned())
                .or_else(|| {
                    result
                        .filter(|finish| finish.subtype != "success")
                        .map(|finish| finish.subtype.replace('_', " "))
                })
                .or_else(|| {
                    ended
                        .stderr
                        .iter()
                        .rev()
                        .find(|line| !line.trim().is_empty())
                        .cloned()
                })
                .or_else(|| ended.exit_code.map(|code| format!("exit code {code}")))
                .unwrap_or_else(|| "it ended without a result".to_owned())
        }
    }
}

/// The last line of `text` that holds anything, cut to [`LINE`] characters: what a run says as it
/// works.
fn last_line(text: &str) -> Option<String> {
    let line = text
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    Some(cut_line(line))
}

/// What a run's result says it did, cut to [`LINE`] characters: its gist, the first sentence of
/// its first prose paragraph without markdown (EP-20a, the owner's choice; the last line before,
/// often a list item or a link).
fn result_line(text: &str) -> Option<String> {
    crate::markup::gist(text).map(|gist| cut_line(&gist))
}

/// `line` cut to [`LINE`] characters with an ellipsis.
fn cut_line(line: &str) -> String {
    if line.chars().count() > LINE {
        let cut: String = line.chars().take(LINE - 1).collect();
        format!("{cut}…")
    } else {
        line.to_owned()
    }
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
    launch: Launch,
    program: PathBuf,
    /// The CLI's version, from its preflight.
    version: Option<String>,
    effort: String,
    /// The session of the window's this run resumes.
    resumes: Option<u64>,
    /// What its transcript shows it started with: the prompt, or a deferred call's answer.
    message: String,
    /// What the autopilot asks of the run.
    run: Run,
}

/// What the autopilot asks of a start (D-22; EP-19).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Run {
    /// The run has a cap by its brief's `size`: its project's autopilot is on.
    pub capped: bool,
    pub size: Option<Size>,
    /// The chain starts it.
    pub chained: bool,
    /// Its prompt or its message tells it to commit.
    pub commits: bool,
    /// A migration of the project's roadmap, launched by Migrate… (EP-21a).
    pub migration: bool,
}

/// What became of a start.
pub enum Started {
    Running(String),
    Refused(String),
}

/// What the All tab lists, saved with the view (EP-13).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionFilter {
    /// One project's sessions, or every project's.
    pub project: Option<String>,
    /// The modes left out: `headless`, `visual`, `manual`.
    pub hidden_modes: BTreeSet<String>,
    /// Only the sessions that run now: the ledger's rows stay out.
    pub running_only: bool,
}

impl SessionFilter {
    fn keeps(&self, project: &str, mode: &str) -> bool {
        self.project
            .as_deref()
            .is_none_or(|wanted| wanted.eq_ignore_ascii_case(project))
            && !self.hidden_modes.contains(mode)
    }
}

/// The ledger's recent rows for the All tab, read without writing (EP-13): again when the ledger
/// changed, and every half minute while the pane shows.
#[derive(Default)]
pub struct History {
    pub rows: Vec<SessionRow>,
    pub problem: Option<String>,
    read_at: Option<Instant>,
    /// The ledger changed since the last read.
    pub stale: bool,
    /// A screenshot run's rows: the ledger is not read.
    pub canned: bool,
}

impl History {
    fn refresh(&mut self, ledger: Option<&Path>) {
        let due = self.stale || self.read_at.is_none_or(|at| at.elapsed() >= HISTORY_EVERY);
        let Some(path) = ledger.filter(|_| due && !self.canned) else {
            return;
        };
        self.read_at = Some(Instant::now());
        self.stale = false;
        if !path.exists() {
            self.rows.clear();
            return;
        }
        match Ledger::open_read_only(path).and_then(|ledger| ledger.recent_sessions(HISTORY_ROWS)) {
            Ok(rows) => {
                self.rows = rows;
                self.problem = None;
            }
            Err(e) => self.problem = Some(format!("The ledger did not read: {e}")),
        }
    }
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
    /// The tab in front: All, or one of the window's sessions by its key.
    pub tab: Option<u64>,
    pub history: History,
    /// The CLI, and its version, whose control-protocol handshake a session refused: its later
    /// runs take the defer route (D-19's fallback; EP-14).
    refused: Option<(PathBuf, Option<String>)>,
    /// A notice for the status bar: the switch to the defer route, once.
    pub switched: Option<String>,
    /// The runs' news the notifications have not taken yet, by session (EP-15).
    news: Vec<(u64, News)>,
    /// The runs that ended since the autopilot last looked, by key (D-22; EP-19).
    ended: Vec<u64>,
    /// The start that last came back was a chat conversation's turn (EP-24), whatever asked
    /// for it: the Chat tile takes its outcome.
    pub last_chat: bool,
    /// The ledger's conversations may have changed (a start came back, a run ended): the Chat
    /// tile's list reads them again (EP-24a).
    pub conversations_stale: bool,
}

impl Default for Sessions {
    fn default() -> Self {
        Self {
            live: Vec::new(),
            starting: None,
            asked: Asked::Not,
            canned: None,
            next_key: 1,
            tab: None,
            history: History::default(),
            refused: None,
            switched: None,
            news: Vec::new(),
            ended: Vec::new(),
            last_chat: false,
            conversations_stale: false,
        }
    }
}

/// The version a preflight read, in words.
fn version_of(preflight: &Preflight) -> Option<String> {
    preflight.version.as_ref().ok().map(ToString::to_string)
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

    /// Starts `launch` on a worker thread, which waits for the turn and the spacing (D-29); `run`
    /// is what the autopilot asks of it (EP-19).
    pub fn start(
        &mut self,
        preflight: Arc<Preflight>,
        launch: Launch,
        effort: String,
        run: Run,
        ctx: &egui::Context,
    ) {
        let message = launch.prompt.clone();
        self.begin(preflight, launch, effort, None, message, run, ctx);
    }

    /// Starts one turn of a chat conversation (D-27; EP-24): `launch` as `chat::launch` built
    /// it, a new session or the resume of the conversation's, whose live is `resumes`; the
    /// autopilot asks nothing of it.
    pub fn chat(
        &mut self,
        preflight: Arc<Preflight>,
        launch: Launch,
        resumes: Option<u64>,
        ctx: &egui::Context,
    ) {
        let message = launch.prompt.clone();
        let effort = crw_core::prompt::effort_words(launch.effort.as_deref(), false);
        self.begin(
            preflight,
            launch,
            effort,
            resumes,
            message,
            Run::default(),
            ctx,
        );
    }

    /// The conversation's live, by its key, when it is a chat's.
    pub fn conversation(&self, key: u64) -> Option<&Live> {
        self.live
            .iter()
            .find(|live| live.key == key && live.is_chat())
    }

    /// The same, to answer its questions in the Chat tile (EP-24a).
    pub fn conversation_mut(&mut self, key: u64) -> Option<&mut Live> {
        self.live
            .iter_mut()
            .find(|live| live.key == key && live.is_chat())
    }

    /// The conversation's model and effort for its next turn, from the Chat tile's picker
    /// (EP-24a): a resume that carries a deferred question's answer builds on the live's
    /// launch, so the choice is written into it first.
    pub fn set_chat_pick(&mut self, key: u64, model: &str, effort: Option<&str>) {
        if let Some(live) = self.conversation_mut(key) {
            live.launch.model = model.to_owned();
            live.launch.effort = effort.map(str::to_owned);
            live.model = model.to_owned();
            live.effort = crw_core::prompt::effort_words(effort, false);
        }
    }

    /// Resumes the window's ended session `key` with `message` as its next prompt (EP-13): a new
    /// run of the same launch under the session's id, in the same tab, its questions taking
    /// `route` (D-19).
    pub fn resume(
        &mut self,
        key: u64,
        preflight: Arc<Preflight>,
        message: &str,
        route: Route,
        ctx: &egui::Context,
    ) {
        self.resume_with(key, preflight, message, route, false, ctx);
    }

    /// Resumes the window's ended session `key` with the owner's yes to its commit (D-22;
    /// EP-19): the Commit button's message, after which the button is not offered again.
    pub fn commit(
        &mut self,
        key: u64,
        preflight: Arc<Preflight>,
        route: Route,
        ctx: &egui::Context,
    ) {
        let Some(ep) = self
            .live
            .iter()
            .find(|live| live.key == key)
            .map(|live| live.ep.clone())
        else {
            return;
        };
        let message = crw_core::prompt::commit_message(&ep);
        self.resume_with(key, preflight, &message, route, true, ctx);
    }

    fn resume_with(
        &mut self,
        key: u64,
        preflight: Arc<Preflight>,
        message: &str,
        route: Route,
        commits: bool,
        ctx: &egui::Context,
    ) {
        let Some(live) = self.live.iter().find(|live| live.key == key) else {
            return;
        };
        let mut launch = live.launch.clone();
        launch.resume = Some(live.session_id.clone());
        launch.prompt = message.to_owned();
        launch.route = route;
        launch.answer = None;
        launch.answered_after = None;
        let effort = live.effort.clone();
        let run = Run {
            commits,
            ..live.run()
        };
        self.begin(
            preflight,
            launch,
            effort,
            Some(key),
            message.to_owned(),
            run,
            ctx,
        );
    }

    /// Answers the request a deferred run of `key` left open with `response`, by resuming the
    /// session (D-19; EP-14): a call the defer route's hook deferred resumes without a prompt, the
    /// answer handed to it through `answers/<session>.json` in `answers` by a one-shot hook that
    /// `hooks` (`crw.exe`) runs; a request the time limit deferred resumes with the answer as its
    /// message, its questions taking `route`. `Err` says why it cannot.
    pub fn answer_open(
        &mut self,
        key: u64,
        preflight: Arc<Preflight>,
        response: Response,
        places: (Route, Option<PathBuf>, &Path),
        ctx: &egui::Context,
    ) -> Result<(), String> {
        let (route, hooks, answers) = places;
        let live = self
            .live
            .iter()
            .find(|live| live.key == key)
            .ok_or("no such session")?;
        let open = live
            .open
            .as_ref()
            .filter(|_| live.ended.is_some())
            .ok_or("the session has no deferred request to answer")?;
        let words = decisions::answer_words(&open.request, &response);
        let mut launch = live.launch.clone();
        launch.resume = Some(live.session_id.clone());
        launch.answered_after = Some(open.since.elapsed());
        launch.hooks = hooks;
        match &open.deferred {
            Some(deferred) => {
                launch.prompt = String::new();
                launch.route = Route::Defer;
                launch.answer = Some(Answered {
                    deferred: deferred.clone(),
                    response,
                    file: answers.join(format!("{}.json", live.session_id)),
                });
            }
            None => {
                launch.prompt = decisions::resume_message(&open.request, &response);
                launch.route = route;
                launch.answer = None;
            }
        }
        let effort = live.effort.clone();
        let message = format!("Your answer to {}: {words}", open.request.summary);
        let run = Run {
            commits: false,
            ..live.run()
        };
        self.begin(preflight, launch, effort, Some(key), message, run, ctx);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn begin(
        &mut self,
        preflight: Arc<Preflight>,
        launch: Launch,
        effort: String,
        resumes: Option<u64>,
        message: String,
        run: Run,
        ctx: &egui::Context,
    ) {
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        let program = preflight.program.clone();
        let version = version_of(&preflight);
        let mut started = launch.clone();
        let spawned = thread::Builder::new()
            .name("session start".to_owned())
            .spawn(move || {
                // The build's copy of the plugin, written before every start, resumes included,
                // so the run gets the plugin of the build that starts it (D-27; EP-22a); a copy
                // that cannot be written is said under the run, which starts without it.
                let unwritten = match crw_core::plugin::prepare() {
                    Ok(dir) => {
                        started.plugin_dir = Some(dir);
                        None
                    }
                    Err(why) => Some(format!("{why}; the run goes without the plugin")),
                };
                let result = Session::start(&preflight, started).map(|mut session| {
                    session.dropped.extend(unwritten);
                    session
                });
                let _ = sender.send(result);
                ctx.request_repaint();
            });
        if spawned.is_ok() {
            self.starting = Some(Starting {
                receiver,
                launch,
                program,
                version,
                effort,
                resumes,
                message,
                run,
            });
        }
    }

    /// The runs that ended since the last call, by key: the autopilot's to weigh (D-22).
    pub fn take_ended(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.ended)
    }

    /// Ticks each capped run's working clock and stops gracefully each run that has worked its
    /// cap (D-22; EP-19): the time a request waits for the owner is left out. Returns each run
    /// stopped, by key, with the log's words.
    pub fn check_caps(&mut self, now: Instant) -> Vec<(u64, String)> {
        let mut stopped = Vec::new();
        for live in &mut self.live {
            let waiting = !live.waiting.is_empty();
            let Some(cap) = live.cap.as_mut().filter(|_| live.ended.is_none()) else {
                continue;
            };
            cap.clock.tick(waiting, now);
            if live.stopping || live.capped.is_some() || !cap.reached(now) {
                continue;
            }
            let worked = cap.clock.worked(now);
            let words = crw_core::autopilot::capped_words(&live.ep, cap, worked);
            if let Some(session) = &live.session
                && session.stop()
            {
                live.stopping = true;
                live.capped = Some(worked);
                live.transcript.take(&Event::Notice(format!(
                    "The autopilot stopped the run: {words}"
                )));
                stopped.push((live.key, words));
            }
        }
        stopped
    }

    /// [`Sessions::route`] for the CLI `preflight` read.
    pub fn route_for(
        &self,
        preflight: &Preflight,
        setting: Option<Route>,
    ) -> (Route, Option<String>) {
        self.route(
            &preflight.program,
            version_of(preflight).as_deref(),
            setting,
        )
    }

    /// How a headless run of `program` (at `version`, when its preflight has read it) takes its
    /// questions (D-19): the defer route when the registry's `settings.decisions` says so or when
    /// a session of this CLI refused the protocol's handshake in this window, with the reason in
    /// words; else the control protocol.
    pub fn route(
        &self,
        program: &Path,
        version: Option<&str>,
        setting: Option<Route>,
    ) -> (Route, Option<String>) {
        if setting == Some(Route::Defer) {
            return (
                Route::Defer,
                Some("the registry's settings.decisions".to_owned()),
            );
        }
        match &self.refused {
            Some((refused, refused_version))
                if refused == program
                    && version.is_none_or(|v| refused_version.as_deref() == Some(v)) =>
            {
                let which = refused_version
                    .as_deref()
                    .map_or("this claude".to_owned(), |v| format!("claude {v}"));
                (
                    Route::Defer,
                    Some(format!(
                        "{which} refused the control protocol's handshake in this window"
                    )),
                )
            }
            _ => (Route::Protocol, None),
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
                    self.last_chat = starting.launch.conversation.is_some();
                    started = Some(match result {
                        Ok(session) => self.started(&starting, session),
                        Err(why) => Started::Refused(why),
                    });
                    self.history.stale = true;
                    self.conversations_stale = true;
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
            if live.drain() {
                self.history.stale = true;
                self.conversations_stale = true;
                self.ended.push(live.key);
            }
            let key = live.key;
            self.news
                .extend(live.news.drain(..).map(|news| (key, news)));
        }
        // A refused handshake: this CLI's later runs take the defer route (D-19's fallback).
        let refusing = self
            .live
            .iter()
            .find(|live| live.handshake_refused && !live.canned)
            .map(|live| (live.program.clone(), live.version.clone()));
        if let Some(refused) = refusing
            && self.refused.as_ref() != Some(&refused)
        {
            let which = refused
                .1
                .as_deref()
                .map_or("This claude".to_owned(), |v| format!("claude {v}"));
            self.switched = Some(format!(
                "{which} refused the control protocol's handshake: the window's later headless \
                 runs take the defer route, a question ending the run until its answer resumes \
                 it (D-19)."
            ));
            self.refused = Some(refused);
        }
        if self.is_starting() || self.live.iter().any(Live::is_running) {
            ctx.request_repaint_after(POLL);
        }
        started
    }

    /// A session that started: a new tab, or the resumed session's own going on.
    fn started(&mut self, starting: &Starting, session: Session) -> Started {
        let (project, ep) = (
            &starting.launch.project,
            starting.launch.ep.as_deref().unwrap_or_default(),
        );
        let resumed = starting
            .resumes
            .and_then(|key| self.live.iter_mut().find(|live| live.key == key));
        if let Some(live) = resumed {
            live.dropped = session.dropped.clone();
            live.session = Some(session);
            live.ended = None;
            live.stopping = false;
            live.runs += 1;
            live.draft.clear();
            live.waiting.clear();
            live.drafts.clear();
            live.handshake_refused = false;
            live.launch.route = starting.launch.route;
            live.version = starting.version.clone();
            // Each run has its own cap (D-22), and a Commit's message is the owner's yes.
            live.cap = live.cap.map(|cap| Cap::new(cap.size, Instant::now()));
            live.capped = None;
            live.commits |= starting.run.commits;
            let answered = live.open.take().is_some() && starting.launch.answered_after.is_some();
            live.transcript.prompt(&starting.message, true);
            // A conversation's later turn: the tile goes on, no tab (EP-24). Each turn
            // rebuilds the whole launch (the digest, and from EP-24a the model and effort the
            // picker holds), so the live keeps the latest, which a deferred question's resume
            // builds on (the review's finding).
            if let Some(title) = &starting.launch.conversation {
                live.launch = starting.launch.clone();
                live.model = starting.launch.model.clone();
                live.effort = starting.effort.clone();
                return Started::Running(format!(
                    "The conversation goes on: chat: {title}, its reply in the Chat pane."
                ));
            }
            self.tab = Some(live.key);
            return Started::Running(if answered {
                format!(
                    "Resumed {project} {ep} with your answer: its tab in the Sessions pane goes on, \
                     and quitting the window stops it."
                )
            } else {
                format!(
                    "Resumed {project} {ep} under its session id: its tab in the Sessions pane \
                     goes on, and quitting the window stops it."
                )
            });
        }
        let key = self.next_key;
        self.next_key += 1;
        let chat = starting.launch.conversation.clone();
        self.live.insert(0, Live::new(key, starting, session));
        if let Some(title) = chat {
            return Started::Running(format!(
                "Started the conversation chat: {title} with {} at {} effort: its reply streams \
                 in the Chat pane, and quitting the window stops it.",
                starting.launch.model, starting.effort
            ));
        }
        self.tab = Some(key);
        Started::Running(format!(
            "Started {project} {ep} headless with {} at {} effort: the Sessions pane shows it, \
             and quitting the window stops it.",
            starting.launch.model, starting.effort
        ))
    }

    /// What the row of `project` says of the window's sessions; `None` when none runs.
    pub fn summary(&self, project: &str) -> Option<String> {
        let running: Vec<String> = self
            .live
            .iter()
            .filter(|live| {
                live.is_running() && !live.is_chat() && live.project.eq_ignore_ascii_case(project)
            })
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
            .filter(|starting| starting.launch.project.eq_ignore_ascii_case(project))
            .map(|starting| match starting.resumes {
                Some(_) => format!(
                    "a headless {} is resuming",
                    starting.launch.ep.as_deref().unwrap_or_default()
                ),
                None => format!(
                    "a headless {} is starting",
                    starting.launch.ep.as_deref().unwrap_or_default()
                ),
            });
        let all: Vec<String> = starting.into_iter().chain(running).collect();
        (!all.is_empty()).then(|| all.join("; "))
    }

    /// The window's own work in `project`, which refuses a revert there (EP-15c): a start or a
    /// resume waiting its turn and each run that goes on, in words that the plan's "revert once
    /// it has ended" follows; and, said as they are, its ended runs whose deferred request waits
    /// for the owner's answer, which a resume would carry onto the reverted project.
    pub fn at_work(&self, project: &str) -> (Vec<String>, Vec<String>) {
        let here = |name: &str| name.eq_ignore_ascii_case(project);
        let mut working = Vec::new();
        if let Some(starting) = self
            .starting
            .as_ref()
            .filter(|starting| here(&starting.launch.project))
        {
            let ep = starting.launch.ep.as_deref().unwrap_or("session");
            let how = match starting.resumes {
                Some(_) => "resuming",
                None => "starting",
            };
            working.push(format!("the window's headless {ep} is {how}"));
        }
        let mut refusals = Vec::new();
        for live in self
            .live
            .iter()
            .filter(|live| !live.is_chat() && here(&live.project))
        {
            if live.is_running() {
                working.push(format!(
                    "the window's headless {} is {}",
                    live.ep,
                    live.state()
                ));
            } else if live.open.is_some() {
                refusals.push(format!(
                    "the window's headless {} was deferred with a request open: answer it in the \
                     Decisions pane, or dismiss its tab, first",
                    live.ep
                ));
            }
        }
        (working, refusals)
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

    /// Forgets an ended session (the ledger keeps its rows).
    pub fn dismiss(&mut self, key: u64) {
        self.live
            .retain(|live| live.key != key || live.is_running());
        if self.tab == Some(key) && self.find(key).is_none() {
            self.tab = None;
        }
    }

    /// The message typed to resume `key`, when it can resume: it has ended, and no start waits.
    pub fn resumable(&self, key: u64) -> Option<(String, PathBuf)> {
        let live = self.live.iter().find(|live| live.key == key)?;
        let message = live.draft.trim();
        (live.ended.is_some() && !live.canned && !message.is_empty() && !self.is_starting())
            .then(|| (message.to_owned(), live.program.clone()))
    }

    /// How many requests the window's runs wait on, for the Decisions tile; a conversation's
    /// are the Chat tile's (EP-24a) and are not counted.
    pub fn waiting(&self) -> usize {
        self.live
            .iter()
            .filter(|live| !live.is_chat())
            .map(|live| live.waiting.len())
            .sum()
    }

    /// How many ended runs wait for the owner's answer to resume (EP-14), a conversation's
    /// aside (EP-24a).
    pub fn deferred(&self) -> usize {
        self.live
            .iter()
            .filter(|live| !live.is_chat() && live.open.is_some() && live.ended.is_some())
            .count()
    }

    /// The CLI's answers when they are in hand, without asking: a screenshot run's canned ones,
    /// or the last preflight that came back, fresh or not (the Chat tile's effort picker reads
    /// the levels from them, EP-24a).
    pub fn preflight_ready(&self) -> Option<Arc<Preflight>> {
        if let Some(canned) = &self.canned {
            return Some(Arc::clone(canned));
        }
        match &self.asked {
            Asked::Ready(preflight) => Some(Arc::clone(preflight)),
            _ => None,
        }
    }

    /// Sends the owner's `response` to the request `request` the session `key` waits on (D-19);
    /// the answer in words, or why it was not sent.
    pub fn respond(
        &mut self,
        key: u64,
        request: &str,
        response: Response,
    ) -> Result<String, String> {
        let live = self.find(key).ok_or("no such session")?;
        let at = live
            .waiting
            .iter()
            .position(|waiting| waiting.request.id == request)
            .ok_or("the session no longer waits on it")?;
        let session = live.session.as_ref().ok_or("the session has ended")?;
        let words = decisions::answer_words(&live.waiting[at].request, &response);
        if !session.respond(request, response) {
            return Err("the session has ended".to_owned());
        }
        live.waiting.remove(at);
        live.drafts.remove(request);
        live.transcript.answered(request, &words);
        Ok(words)
    }

    /// Whether the window started the session `session_id`.
    pub fn owns(&self, session_id: &str) -> bool {
        self.live.iter().any(|live| live.session_id == session_id)
    }

    /// The runs' news since the last call, each with its session's key (D-21; EP-15).
    pub fn take_news(&mut self) -> Vec<(u64, News)> {
        std::mem::take(&mut self.news)
    }

    /// What quitting the window would stop, in words: each headless run that goes on, started
    /// here, and a start on its way (D-16's EP-10 addendum; EP-15).
    pub fn stoppable(&self) -> Vec<String> {
        let running = self
            .live
            .iter()
            .filter(|live| live.is_running() && !live.stopping)
            .map(|live| {
                format!(
                    "{} ({} since {})",
                    live.title(),
                    live.state(),
                    live.started.format("%H:%M")
                )
            });
        let starting = self.starting.as_ref().map(|starting| {
            format!(
                "{} {} (starting)",
                starting.launch.project,
                starting.launch.ep.as_deref().unwrap_or_default()
            )
        });
        running.chain(starting).collect()
    }

    /// The headless runs being stopped, in words: their end is still to be recorded and the gate
    /// still to read it, which a restart waits for (EP-37).
    pub fn stopping(&self) -> Vec<String> {
        self.live
            .iter()
            .filter(|live| live.is_running() && live.stopping)
            .map(Live::title)
            .collect()
    }
}

impl Board {
    /// The sessions the hooks and the poll report in registered projects, the window's own left
    /// out (EP-12), the latest start first.
    pub fn seen_sessions(&self) -> Vec<&Seen> {
        self.watch
            .model
            .all()
            .into_iter()
            .filter(|seen| seen.project.is_some() && !self.sessions.owns(&seen.session_id))
            .collect()
    }

    /// The Sessions tile (EP-13): the tabs, then All or a session's transcript.
    pub fn sessions_pane(&mut self, ui: &mut egui::Ui) {
        let ledger = self.ledger().map(Path::to_owned);
        self.sessions.history.refresh(ledger.as_deref());
        if self.sessions.tab.is_some_and(|key| {
            !self
                .sessions
                .live
                .iter()
                .any(|live| live.key == key && !live.is_chat())
        }) {
            self.sessions.tab = None;
        }
        self.tab_strip(ui);
        ui.separator();
        match self.sessions.tab {
            Some(key) => self.session_tab(ui, key),
            None => self.all_tab(ui),
        }
    }

    fn tab_strip(&mut self, ui: &mut egui::Ui) {
        let mut tab = self.sessions.tab;
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut tab, None, "All");
            // A conversation's run is the Chat pane's (EP-24).
            for live in self.sessions.live.iter().filter(|live| !live.is_chat()) {
                let mut job = egui::text::LayoutJob::default();
                let font = egui::TextStyle::Button.resolve(ui.style());
                job.append(
                    "● ",
                    0.0,
                    egui::TextFormat::simple(font.clone(), live.colour(ui)),
                );
                job.append(
                    &format!("{} {}", live.project, live.ep),
                    0.0,
                    egui::TextFormat::simple(font, ui.visuals().text_color()),
                );
                ui.selectable_value(&mut tab, Some(live.key), job)
                    .on_hover_text(live.state());
            }
        });
        self.sessions.tab = tab;
    }

    /// Every session: the window's, those started elsewhere, the ledger's recent rows.
    fn all_tab(&mut self, ui: &mut egui::Ui) {
        self.autopilot_section(ui);
        let names: Vec<String> = self
            .portfolio
            .projects
            .iter()
            .map(|row| row.name().to_owned())
            .collect();
        filters(ui, &mut self.view.sessions, &names);
        let filter = self.view.sessions.clone();
        let seen: Vec<&Seen> = self
            .seen_sessions()
            .into_iter()
            .filter(|seen| {
                filter.keeps(
                    seen.project.as_deref().unwrap_or_default(),
                    seen.mode.as_str(),
                ) && !(filter.running_only && seen.status.is_over())
            })
            .collect();
        let outside = self.watch.model.outside();
        let lives: Vec<&Live> = self
            .sessions
            .live
            .iter()
            .filter(|live| {
                !live.is_chat()
                    && filter.keeps(&live.project, "headless")
                    && !(filter.running_only && !live.is_running())
            })
            .collect();
        // A row whose session shows above is not listed again.
        let shown: BTreeSet<&str> = self
            .sessions
            .live
            .iter()
            .map(|live| live.session_id.as_str())
            .chain(
                self.watch
                    .model
                    .all()
                    .into_iter()
                    .filter(|seen| !seen.status.is_over())
                    .map(|seen| seen.session_id.as_str()),
            )
            .collect();
        // What a ledger card's Resume in VS Code must not cross (EP-35).
        let model = &self.watch.model;
        let resumes = Resumes {
            window: self
                .sessions
                .live
                .iter()
                .map(|live| live.session_id.as_str())
                .collect(),
            open: model
                .all()
                .into_iter()
                .filter(|seen| !seen.status.is_over())
                .map(|seen| seen.session_id.as_str())
                .collect(),
            unknown: match (&model.poll_problem, model.polled) {
                (Some(why), _) => Some(format!(
                    "the list of live sessions did not read ({why}), so whether the session runs \
                     elsewhere is not known"
                )),
                (None, None) => Some(
                    "the list of live sessions has not answered yet, so whether the session runs \
                     elsewhere is not known: try again in a moment"
                        .to_owned(),
                ),
                (None, Some(_)) => None,
            },
            opening: self.visual.is_opening(),
        };
        let history = &self.sessions.history;
        let rows: Vec<&SessionRow> = if filter.running_only {
            Vec::new()
        } else {
            history
                .rows
                .iter()
                .filter(|row| {
                    filter.keeps(&row.project, &row.mode)
                        && !row
                            .session_id
                            .as_deref()
                            .is_some_and(|id| shown.contains(id))
                })
                .collect()
        };
        let (mut open, mut focus, mut revert, mut resume) = (None, None, None, None);
        let portfolio = &self.portfolio;
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
                if lives.is_empty() && seen.is_empty() && rows.is_empty() {
                    placeholder(
                        ui,
                        "No session to list. The headless sessions started from the launch dialog \
                         (S, or a row's Start button) get a tab here with their transcript; the \
                         sessions Claude Code reports elsewhere and the ledger's rows join them.",
                    );
                }
                if !lives.is_empty() {
                    heading(ui, "Started here: headless, with a tab each");
                    for live in &lives {
                        if live_line(ui, live) {
                            open = Some(live.key);
                        }
                        ui.add_space(4.0);
                    }
                }
                if !seen.is_empty() {
                    heading(ui, "Started elsewhere: in VS Code, a terminal or another program");
                    for one in &seen {
                        if seen_card(ui, one)
                            && let Some(project) = &one.project
                        {
                            focus = Some(project.clone());
                        }
                        ui.add_space(6.0);
                    }
                }
                if outside > 0 {
                    ui.label(RichText::new(outside_words(outside)).weak());
                }
                if let Some(problem) = &self.watch.model.poll_problem {
                    ui.label(
                        RichText::new(format!("The list of live sessions did not read: {problem}"))
                            .small()
                            .color(style::amber(ui)),
                    );
                }
                if !rows.is_empty() {
                    heading(ui, "From the ledger: the latest first");
                    let now = Utc::now();
                    for row in &rows {
                        // A row with a session id keeps its Revert, its tab gone or not (EP-15c),
                        // and can be resumed in VS Code (EP-35).
                        let buttons = row.session_id.as_ref().map(|_| {
                            let project = portfolio.project(&row.project);
                            CardButtons {
                                resume: resume_availability(row, project, &resumes),
                                revert: crate::revert::availability(project, &row.project),
                            }
                        });
                        match ledger_line(ui, row, now, buttons.as_ref()) {
                            Some(Pressed::Revert) => {
                                revert = Some((row.project.clone(), row.session_id.clone()));
                            }
                            Some(Pressed::Resume) => {
                                resume = row.session_id.clone().map(|session| {
                                    (row.project.clone(), session, row.ep.clone())
                                });
                            }
                            None => {}
                        }
                        ui.add_space(4.0);
                    }
                }
                if let Some(problem) = &history.problem {
                    ui.label(RichText::new(problem).small().color(style::amber(ui)));
                }
            });
        if open.is_some() {
            self.sessions.tab = open;
        }
        if let Some(project) = focus {
            self.actions.push(Action::Focus(project));
        }
        if let Some((project, session)) = revert {
            self.actions.push(Action::OpenRevert { project, session });
        }
        if let Some((project, session, ep)) = resume {
            self.actions.push(Action::ResumeInCode {
                project,
                session,
                ep,
            });
        }
    }

    /// One session of the window's: its header and buttons, its transcript, and Resume and Revert
    /// once it has ended.
    fn session_tab(&mut self, ui: &mut egui::Ui, key: u64) {
        let starting = self.sessions.is_starting();
        let Some(live) = self.sessions.find(key) else {
            return;
        };
        let available =
            crate::revert::availability(self.portfolio.project(&live.project), &live.project);
        let mut actions = Vec::new();
        // The title, the chip and the buttons on rows of their own, so that a narrow pane neither
        // squeezes a chip nor gets widened past its edge by a row too long for it.
        ui.label(RichText::new(format!("{} {}", live.project, live.ep)).strong());
        ui.horizontal(|ui| style::chip(ui, &live.state(), live.colour(ui)));
        let runs = match live.runs {
            1 => String::new(),
            n => format!(" · {n} runs"),
        };
        ui.label(
            RichText::new(format!(
                "{} · {} · started {}{runs}",
                live.model,
                live.effort,
                live.started.format("%H:%M")
            ))
            .weak(),
        );
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
                    actions.push(Action::StopSession(key));
                }
                if ui
                    .button(style::icon(icons::ICON_CLOSE, Some(style::red(ui))))
                    .on_hover_text("Kill the session and its whole process tree at once")
                    .clicked()
                {
                    actions.push(Action::KillSession(key));
                }
            } else if ui
                .button("Dismiss")
                .on_hover_text("Close its tab; the ledger keeps its rows")
                .clicked()
            {
                actions.push(Action::DismissSession(key));
            }
        });
        for note in &live.dropped {
            ui.label(RichText::new(note).small().color(style::amber(ui)));
        }
        let mut counts = Vec::new();
        match live.denied {
            0 => {}
            1 => counts.push("1 tool use refused".to_owned()),
            n => counts.push(format!("{n} tool uses refused")),
        }
        match live.unreadable {
            0 => {}
            1 => counts.push("1 line unread".to_owned()),
            n => counts.push(format!("{n} lines unread")),
        }
        if let Some(init) = &live.init
            && let Some(version) = &init.version
        {
            counts.push(format!("claude {version}"));
        }
        counts.push(format!("session {}", live.session_id));
        ui.label(RichText::new(counts.join(" · ")).small().weak());
        // The autopilot's part (D-22): the chain started it, its cap and the work so far.
        if let Some(cap) = &live.cap {
            let chained = if live.chained {
                "chained by the autopilot"
            } else {
                "autopilot on"
            };
            let worked = match live.capped {
                Some(worked) => format!("stopped at its cap after {}", autopilot::length(worked)),
                None if live.is_running() => format!(
                    "{} worked",
                    autopilot::length(cap.clock.worked(Instant::now()))
                ),
                None => String::new(),
            };
            let line = [chained.to_owned(), cap.words(), worked]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" · ");
            ui.label(RichText::new(line).small().color(style::reference(ui)));
        }
        ui.separator();

        // The footer keeps its room below the transcript once the run has ended.
        let footer = match &live.ended {
            Some(_) if live.offers_commit() => 128.0,
            Some(_) => 96.0,
            None => 0.0,
        };
        egui::ScrollArea::vertical()
            .id_salt(("transcript", key))
            .auto_shrink(false)
            .stick_to_bottom(true)
            .max_height((ui.available_height() - footer).max(80.0))
            .show(ui, |ui| live.transcript.ui(ui, key));
        if let Some(ended) = &live.ended {
            ui.separator();
            ui.label(RichText::new(ending(ended)).weak());
            if ended.outcome == Outcome::Failed
                && let Some(line) = ended.stderr.last()
            {
                ui.label(RichText::new(line).color(style::red(ui)));
            }
            ui.horizontal(|ui| {
                // A screenshot run's canned session cannot resume; its Revert opens a canned plan.
                let resume = (!live.canned).then(|| {
                    let ready = !live.draft.trim().is_empty() && !starting;
                    ui.add_enabled(ready, egui::Button::new("Resume"))
                        .on_hover_text(
                            "Run the session again under its id (--resume), with this message as \
                             its next prompt; the ledger keeps a row for each run",
                        )
                });
                // Revert at the line's end, the message's field between (EP-15c).
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if crate::revert::revert_button(ui, &available, false) {
                        actions.push(Action::OpenRevert {
                            project: live.project.clone(),
                            session: Some(live.session_id.clone()),
                        });
                    }
                    if !live.canned {
                        ui.add(
                            egui::TextEdit::singleline(&mut live.draft)
                                .hint_text("A message to resume it with")
                                .desired_width(f32::INFINITY),
                        );
                    }
                });
                if resume.is_some_and(|resume| resume.clicked()) {
                    actions.push(Action::ResumeSession(key));
                }
            });
            // With the autopilot off the run stopped before committing (D-22).
            if live.offers_commit()
                && ui
                    .add_enabled(!starting, egui::Button::new("Commit"))
                    .on_hover_text(
                        "Resume the session with your yes to its commit: the house pattern's \
                         two commits, the hash recorded in the roadmap, never a push",
                    )
                    .clicked()
            {
                actions.push(Action::CommitSession(key));
            }
        }
        self.actions.extend(actions);
    }
}

/// The All tab's filters: one project or every one, the modes shown, running only.
fn filters(ui: &mut egui::Ui, filter: &mut SessionFilter, names: &[String]) {
    ui.horizontal_wrapped(|ui| {
        ui.label(style::icon(icons::ICON_FILTER_LIST, None));
        egui::ComboBox::from_id_salt("sessions-project")
            .selected_text(filter.project.as_deref().unwrap_or("Every project"))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut filter.project, None, "Every project");
                for name in names {
                    ui.selectable_value(&mut filter.project, Some(name.clone()), name);
                }
            });
        for mode in MODES {
            let mut shown = !filter.hidden_modes.contains(mode);
            if ui.toggle_value(&mut shown, mode).changed() {
                if shown {
                    filter.hidden_modes.remove(mode);
                } else {
                    filter.hidden_modes.insert(mode.to_owned());
                }
            }
        }
        ui.checkbox(&mut filter.running_only, "Running only")
            .on_hover_text("Leave out the sessions that have ended and the ledger's rows");
    });
    ui.add_space(4.0);
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).weak());
    ui.add_space(2.0);
}

fn placeholder(ui: &mut egui::Ui, text: &str) {
    ui.add_space(24.0);
    ui.vertical_centered(|ui| ui.label(RichText::new(text).weak()));
}

fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        })
        .response
}

/// A session of the window's in the All tab; `true` when it was clicked, to open its tab.
fn live_line(ui: &mut egui::Ui, live: &Live) -> bool {
    let response = card(ui, |ui| {
        // The card takes the click over its text, not a selection (EP-15a).
        style::unselectable(ui);
        // The title and the chip on rows of their own: a row wider than a narrow pane would widen
        // everything after it past the pane's edge, and a wrapped row would squeeze the chip.
        ui.label(RichText::new(format!("{} {}", live.project, live.ep)).strong());
        ui.horizontal(|ui| style::chip(ui, &live.state(), live.colour(ui)));
        ui.label(
            RichText::new(format!(
                "{} · {} · started {}",
                live.model,
                live.effort,
                live.started.format("%H:%M")
            ))
            .weak(),
        );
        if !live.last.is_empty() {
            ui.add(egui::Label::new(RichText::new(&live.last).weak()).truncate());
        }
    });
    response
        .interact(egui::Sense::click())
        .on_hover_text("Open its tab: the transcript")
        .clicked()
}

fn outside_words(count: usize) -> String {
    match count {
        1 => "One more session runs outside the registered projects.".to_owned(),
        n => format!("{n} more sessions run outside the registered projects."),
    }
}

/// The colour of a seen session's status: at work in blue, waiting in amber (EP-13).
fn seen_colour(ui: &egui::Ui, seen: &Seen) -> Color32 {
    match seen.status {
        SeenStatus::Busy => style::blue(ui),
        SeenStatus::Waiting => style::amber(ui),
        SeenStatus::Done => style::green(ui),
        SeenStatus::Failed => style::red(ui),
        SeenStatus::Idle => ui.visuals().weak_text_color(),
    }
}

/// A session Claude Code reports (EP-12): its project, mode and status, its name, since when, and
/// the status line's figures when a terminal session sends them (D-23). The tool reads no
/// transcript of it (D-26). One open with no prompt yet is marked new (EP-13); one that waits in
/// VS Code has Focus, which brings its window forward (D-19; EP-14). `true` when Focus was
/// pressed.
fn seen_card(ui: &mut egui::Ui, seen: &Seen) -> bool {
    let mut focus = false;
    card(ui, |ui| {
        // Two rows, so that a narrow pane never squeezes a chip into a column of letters.
        ui.horizontal(|ui| {
            ui.label(RichText::new(seen.project.as_deref().unwrap_or_default()).strong());
            let mode = if seen.is_new() {
                "new"
            } else {
                seen.mode.as_str()
            };
            style::chip(ui, mode, ui.visuals().weak_text_color());
        });
        ui.horizontal_wrapped(|ui| {
            style::chip(ui, seen.status.as_str(), seen_colour(ui, seen));
            if let Some(started) = seen.started {
                ui.label(
                    RichText::new(format!(
                        "since {}",
                        started.with_timezone(&Local).format("%H:%M")
                    ))
                    .weak(),
                );
            }
        });
        if seen.status == SeenStatus::Waiting {
            let in_code = seen.entrypoint.as_deref() == Some("claude-vscode")
                || seen.mode == crw_core::sessions::Mode::Visual;
            let place = if in_code { "in VS Code" } else { "" };
            ui.add(
                egui::Label::new(
                    RichText::new(format!("It {}.", seen.waits_for_you(place)))
                        .color(style::amber(ui)),
                )
                .wrap(),
            );
            if in_code
                && ui
                    .button("Focus")
                    .on_hover_text("Bring its VS Code window forward")
                    .clicked()
            {
                focus = true;
            }
        } else if seen.is_new() {
            ui.label(
                RichText::new(format!(
                    "It {}: no row in the ledger, and no session in progress, until it works.",
                    seen.doing()
                ))
                .weak(),
            );
        }
        if let Some(name) = &seen.name {
            ui.add(egui::Label::new(RichText::new(name).weak()).truncate());
        }
        let mut figures = Vec::new();
        if let Some(line) = &seen.line {
            if !line.model.display_name.is_empty() {
                figures.push(line.model.display_name.clone());
            }
            if let Some(cost) = line.cost.total_cost_usd {
                figures.push(format!("an estimated ${cost:.2}"));
            }
            if let Some(context) = line.context_window.used_percentage {
                figures.push(format!("context {context:.0} %"));
            }
        }
        if let Some(surface) = &seen.entrypoint {
            figures.push(surface.clone());
        }
        figures.push(format!("session {}", seen.session_id));
        ui.label(RichText::new(figures.join(" · ")).small().weak());
    });
    focus
}

/// What a ledger row's session came to: a word for its chip, the sentence its hover gives, and its
/// colour (D-26; EP-13). A row listed here is no live session's, so an open one's end was never
/// heard.
fn row_state(
    ui: &egui::Ui,
    row: &SessionRow,
    now: DateTime<Utc>,
) -> (String, &'static str, Color32) {
    let weak = ui.visuals().weak_text_color();
    match row.state(now, VISUAL_CLAIM) {
        RowState::Ended("completed") => (
            "completed".to_owned(),
            "It ended as it should",
            style::green(ui),
        ),
        RowState::Ended("deferred") => (
            "deferred".to_owned(),
            "It ended waiting on a deferred tool use, to be resumed",
            style::amber(ui),
        ),
        RowState::Ended(other) => (
            other.to_owned(),
            "It ended before its result",
            style::red(ui),
        ),
        RowState::Lapsed => (
            "lapsed".to_owned(),
            "No session took up this visual launch within its 30 minutes",
            weak,
        ),
        RowState::Launched => (
            "launched".to_owned(),
            "Handed to VS Code: no session has taken it up yet",
            style::amber(ui),
        ),
        RowState::Open => (
            "no end seen".to_owned(),
            "Its end was never heard: it ended while no window listened, or before the hooks",
            weak,
        ),
    }
}

/// How long a row's run took: `36 s`, `52 min`, `1 h 04 min`.
fn duration_words(row: &SessionRow) -> Option<String> {
    row.took().map(ledger::length)
}

/// What a resume in VS Code must not cross (EP-35).
#[derive(Debug, Default)]
pub struct Resumes<'a> {
    /// The window's own sessions, which their tabs here resume.
    pub window: BTreeSet<&'a str>,
    /// The sessions the hooks or the poll show open.
    pub open: BTreeSet<&'a str>,
    /// Why it is not known whether a session runs elsewhere, while it is not.
    pub unknown: Option<String>,
    /// A launch to VS Code is on its way.
    pub opening: bool,
}

/// Whether a ledger row's session may be resumed in a tab of its project's VS Code window (D-16;
/// EP-35), `project` being its row in the portfolio: the reason when not, which the disabled
/// button gives on hover. A session that runs must not be opened twice.
pub fn resume_availability(
    row: &SessionRow,
    project: Option<&ProjectRow>,
    resumes: &Resumes,
) -> Result<(), String> {
    let name = &row.project;
    let project = project.ok_or_else(|| format!("{name} is not a registered project"))?;
    if project.folder.is_none() {
        return Err(format!("{name}: its folder is missing"));
    }
    let id = row
        .session_id
        .as_deref()
        .ok_or("no session id was recorded for this row")?;
    if !crw_core::visual::is_uuid(id) {
        return Err(format!(
            "its session id {id} is not one the extension's link takes"
        ));
    }
    // EP-35's check: the extension opens a `-p` run's id as an empty tab, as its own list leaves
    // such runs out.
    if row.mode == "headless" {
        return Err(format!(
            "VS Code opens a headless run as an empty tab, without its conversation (EP-35's \
             check): claude --resume {id} in a terminal in the project's folder takes it up"
        ));
    }
    if resumes.window.contains(id) {
        return Err("it is this window's own session: its tab resumes it".to_owned());
    }
    if resumes.open.contains(id) {
        return Err("it runs now: a tab or a terminal has it open".to_owned());
    }
    if let Some(why) = &resumes.unknown {
        return Err(why.clone());
    }
    if resumes.opening {
        return Err(
            "a launch to VS Code is on its way: try again once it has gone through".to_owned(),
        );
    }
    Ok(())
}

/// What a ledger card's buttons may do (EP-15c, EP-35): each available, or disabled with the
/// reason.
pub struct CardButtons {
    pub resume: Result<(), String>,
    pub revert: Result<(), String>,
}

/// Which of a ledger card's buttons was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pressed {
    Resume,
    Revert,
}

/// A ledger card's Resume in VS Code (EP-35): it opens the row's session in a Claude Code tab of
/// the project's window, and sends nothing; disabled, with its reason on hover. `true` when
/// pressed.
fn resume_button(ui: &mut egui::Ui, available: &Result<(), String>) -> bool {
    let text = RichText::new("Resume in VS Code").small();
    let button = ui.add_enabled(available.is_ok(), egui::Button::new(text));
    match available {
        Ok(()) => button
            .on_hover_text(
                "Open this session in a Claude Code tab of the project's VS Code window, to go on \
                 with it there; nothing is sent, and a tab that holds it comes to the front",
            )
            .clicked(),
        Err(why) => {
            button.on_disabled_hover_text(why);
            false
        }
    }
}

/// A ledger card's two buttons, Resume in VS Code then Revert…, in the order they read: in a
/// right-to-left layout Revert… goes first, so that it keeps the line's end.
fn card_buttons(ui: &mut egui::Ui, buttons: &CardButtons, right_to_left: bool) -> Option<Pressed> {
    let mut pressed = None;
    let mut resume = |ui: &mut egui::Ui| {
        if resume_button(ui, &buttons.resume) {
            pressed = Some(Pressed::Resume);
        }
    };
    if right_to_left {
        let revert = crate::revert::revert_button(ui, &buttons.revert, true);
        resume(ui);
        if revert {
            pressed = Some(Pressed::Revert);
        }
    } else {
        resume(ui);
        if crate::revert::revert_button(ui, &buttons.revert, true) {
            pressed = Some(Pressed::Revert);
        }
    }
    pressed
}

/// A ledger row in the All tab: when, which project and EP, its mode, model and effort, how long,
/// how it ended and the cost estimate, then its result's first line. A row with a session id has
/// its Resume in VS Code (EP-35) and its Revert… (EP-15c), each available or disabled with the
/// reason; which one was pressed.
fn ledger_line(
    ui: &mut egui::Ui,
    row: &SessionRow,
    now: DateTime<Utc>,
    buttons: Option<&CardButtons>,
) -> Option<Pressed> {
    let mut pressed = None;
    card(ui, |ui| {
        // Rows of their own, so that a narrow pane never squeezes a chip into a column of letters.
        ui.horizontal_wrapped(|ui| {
            let started = row.started().map(|at| at.with_timezone(&Local));
            let when = match started {
                Some(at) if at.date_naive() == Local::now().date_naive() => {
                    at.format("%H:%M").to_string()
                }
                Some(at) => at.format("%Y-%m-%d %H:%M").to_string(),
                None => row.started_at.clone(),
            };
            ui.label(RichText::new(when).monospace().weak());
            let title = match &row.ep {
                Some(ep) => format!("{} {ep}", row.project),
                None => row.project.clone(),
            };
            ui.label(RichText::new(title).strong());
        });
        // The buttons at the chips' line's end when they fit there, else on a line of their own: a
        // line wider than a narrow pane would widen everything after it (EP-13).
        let mut below = false;
        ui.horizontal(|ui| {
            style::chip(ui, &row.mode, ui.visuals().weak_text_color());
            let (state, why, colour) = row_state(ui, row, now);
            style::chip(ui, &state, colour).on_hover_text(why);
            if let Some(buttons) = buttons {
                let width = style::width_of(ui, |ui| {
                    card_buttons(ui, buttons, false);
                });
                if ui.available_width() >= width {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        pressed = card_buttons(ui, buttons, true);
                    });
                } else {
                    below = true;
                }
            }
        });
        if let (true, Some(buttons)) = (below, buttons) {
            ui.horizontal_wrapped(|ui| pressed = card_buttons(ui, buttons, false));
        }
        let mut figures = Vec::new();
        match (&row.model, &row.effort) {
            (Some(model), Some(effort)) => figures.push(format!("{model} · {effort}")),
            (Some(model), None) => figures.push(model.clone()),
            _ => {}
        }
        if let Some(took) = duration_words(row) {
            figures.push(took);
        }
        if let Some(cost) = row.cost_usd {
            figures.push(format!("an estimated ${cost:.2}"));
        }
        if let Some(context) = row.context_percent {
            figures.push(format!("context {context:.0} %"));
        }
        figures.extend(ledger::asked(row));
        figures.extend(ledger::shares(row));
        // Whole, to copy for `claude --resume` or `crw revert` (EP-35).
        if let Some(id) = &row.session_id {
            figures.push(format!("session {id}"));
        }
        if !figures.is_empty() {
            ui.label(RichText::new(figures.join(" · ")).weak());
        }
        if let Some(line) = row.result.as_deref().and_then(result_line) {
            ui.add(egui::Label::new(RichText::new(line).weak()).truncate());
        }
    });
    pressed
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
        if let Some(text) = result.text.as_deref().and_then(result_line) {
            parts.push(text);
        }
    }
    parts.join(" · ")
}

#[cfg(test)]
pub(crate) mod tests {
    use crw_core::session::{Finish, ToolUse};

    use super::*;
    use crate::transcript::Entry;

    /// A run of the window's, alpha EP-3, working.
    pub(crate) fn live() -> Live {
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
            transcript: Transcript::default(),
            waiting: Vec::new(),
            open: None,
            denied: 0,
            refusals: Vec::new(),
            unreadable: 0,
            stopping: false,
            ended: None,
            runs: 1,
            draft: String::new(),
            drafts: HashMap::new(),
            handshake_refused: false,
            cap: None,
            capped: None,
            chained: false,
            commits: false,
            migration: false,
            launch: Launch::new("alpha", Path::new("alpha"), "fable", "Run EP-3."),
            program: PathBuf::from("claude.exe"),
            version: Some("2.1.316".to_owned()),
            canned: false,
            news: Vec::new(),
        }
    }

    /// A chat conversation's run (EP-24): a permission request is refused by the window at
    /// once and said so in the transcript, a question waits for the owner as any run's, and
    /// its end raises no toast; its title is the conversation's.
    #[test]
    fn a_chat_run_refuses_permissions_itself_and_ends_quietly() {
        let mut live = live();
        live.launch.conversation = Some("What does EP-24 plan?".to_owned());
        live.project = chat::PROJECT.to_owned();
        live.ep.clear();
        assert!(live.is_chat());
        assert_eq!(live.title(), "chat: What does EP-24 plan?");
        let request = |id: &str, questions: Vec<crw_core::session::Question>| Request {
            id: id.to_owned(),
            tool: "Bash".to_owned(),
            summary: "Bash git log".to_owned(),
            reason: None,
            questions,
            input: String::new(),
        };
        live.take(Event::Request(request("req-1", Vec::new())));
        assert!(live.waiting.is_empty(), "refused, not waiting");
        assert_eq!(live.denied, 1);
        assert!(live.last.starts_with("refused: Bash git log"));
        assert!(live.news.is_empty(), "no toast for a refusal");
        let answer = live
            .transcript
            .entries
            .iter()
            .find_map(|entry| match entry {
                Entry::Asked { request, answer } if request.id == "req-1" => Some(answer.clone()),
                _ => None,
            });
        assert!(
            answer
                .flatten()
                .is_some_and(|words| words.contains("read-only") && words.contains("had ended")),
            "the session handle is gone in a test, and the transcript says so"
        );
        // A question reaches the owner as any run's.
        let question = crw_core::session::Question {
            text: "Which project?".to_owned(),
            header: "Project".to_owned(),
            options: Vec::new(),
            multi_select: false,
        };
        live.take(Event::Request(request("req-2", vec![question])));
        assert_eq!(live.waiting.len(), 1);
        assert_eq!(live.news, [News::Asked("Bash git log".to_owned())]);
        live.news.clear();
        live.take(Event::Exited(Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        }));
        assert!(live.news.is_empty(), "a chat's end is read in its tile");
        assert!(live.ended.is_some());
        // A brief's run still raises its end.
        let mut run = self::live();
        run.take(Event::Exited(Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        }));
        assert_eq!(run.news.len(), 1);
    }

    /// A migration's run (EP-21a) offers no Commit, whose message records a brief's hash, and a
    /// resume of it, or the answer to a request it left open, stays a migration.
    #[test]
    fn a_migration_offers_no_commit_and_resumes_as_one() {
        let mut live = live();
        live.ended = Some(Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        });
        assert!(live.offers_commit(), "a brief's completed run offers it");
        live.migration = true;
        assert!(!live.offers_commit());
        assert!(live.run().migration);
        assert!(!live.run().capped && !live.run().chained);
    }

    /// A run being stopped is no longer one Quit would stop, but its end is still to come, which
    /// a restart waits for (EP-37).
    #[test]
    fn a_run_being_stopped_is_named_apart() {
        let mut sessions = Sessions::default();
        let mut run = live();
        run.canned = true;
        sessions.live.push(run);
        assert_eq!(sessions.stoppable().len(), 1);
        assert!(sessions.stopping().is_empty());
        sessions.live[0].stopping = true;
        assert!(sessions.stoppable().is_empty());
        assert_eq!(sessions.stopping(), vec![sessions.live[0].title()]);
    }

    /// The window's own work in a project refuses a revert there (EP-15c): a start waiting its
    /// turn and a run that goes on in words the plan's "revert once it has ended" follows, a
    /// deferred run with a request open said as it is; an ended run, another project's, none.
    #[test]
    fn the_windows_runs_and_starts_refuse_a_revert_there() {
        let mut sessions = Sessions::default();
        assert_eq!(sessions.at_work("alpha"), (Vec::new(), Vec::new()));
        let mut running = live();
        running.canned = true;
        let mut deferred = live();
        deferred.key = 2;
        deferred.ep = "EP-2".to_owned();
        deferred.open = Some(Open::now(asked("req-1"), None));
        deferred.ended = Some(Ended {
            outcome: Outcome::Deferred,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        });
        let mut done = live();
        done.key = 3;
        done.ended = deferred.ended.clone();
        done.ended.as_mut().unwrap().outcome = Outcome::Completed;
        let mut other = live();
        other.key = 4;
        other.project = "beta".to_owned();
        other.canned = true;
        sessions.live = vec![running, deferred, done, other];
        let mut launch = Launch::new("Alpha", Path::new("alpha"), "fable", "Run EP-4.");
        launch.ep = Some("EP-4".to_owned());
        sessions.starting = Some(Starting {
            receiver: mpsc::channel().1,
            launch,
            program: PathBuf::from("claude.exe"),
            version: None,
            effort: "high".to_owned(),
            resumes: None,
            message: String::new(),
            run: Run::default(),
        });
        let (working, refusals) = sessions.at_work("alpha");
        assert_eq!(
            working,
            [
                "the window's headless EP-4 is starting",
                "the window's headless EP-3 is running",
            ]
        );
        assert_eq!(refusals.len(), 1);
        assert!(refusals[0].starts_with("the window's headless EP-2 was deferred"));
        assert_eq!(
            sessions.at_work("beta").0,
            ["the window's headless EP-3 is running"]
        );
    }

    /// A headless run of alpha's that the window no longer holds.
    fn ledger_row() -> SessionRow {
        SessionRow {
            id: 1,
            session_id: Some("0e1d2c3b-4a59-4687-9a0b-1c2d3e4f5a6b".to_owned()),
            project: "alpha".to_owned(),
            ep: Some("EP-3".to_owned()),
            mode: "headless".to_owned(),
            model: None,
            effort: None,
            ultracode: false,
            started_at: (Utc::now() - chrono::TimeDelta::minutes(30)).to_rfc3339(),
            ended_at: Some(Utc::now().to_rfc3339()),
            outcome: Some("completed".to_owned()),
            cost_usd: None,
            questions: 0,
            result: None,
            context_percent: None,
            answer_latency_s: None,
            start: None,
            usage: Default::default(),
        }
    }

    /// A ledger card's buttons answer a bare context's press (EP-15c, EP-35): Revert… opens the
    /// preview and Resume in VS Code the session; disabled, neither does; a row with no session id
    /// has neither.
    #[test]
    fn a_ledger_cards_buttons_answer_a_press() {
        use crate::testing::{self, Screen};
        let row = ledger_row();
        let press = |label: &str, buttons: Option<CardButtons>| {
            let mut screen = Screen::new();
            let now = Utc::now();
            let mut pressed = None;
            let output = screen.pass(Vec::new(), |ui| {
                pressed = pressed.or(ledger_line(ui, &row, now, buttons.as_ref()));
            });
            let at = testing::text_at(&output, label)?;
            for events in [
                testing::moved(at),
                testing::button(at, egui::PointerButton::Primary, true),
                testing::button(at, egui::PointerButton::Primary, false),
            ] {
                screen.pass(events, |ui| {
                    pressed = pressed.or(ledger_line(ui, &row, now, buttons.as_ref()));
                });
            }
            Some(pressed)
        };
        let both = |resume: Result<(), String>, revert: Result<(), String>| {
            Some(CardButtons { resume, revert })
        };
        assert_eq!(
            press("Revert…", both(Ok(()), Ok(()))),
            Some(Some(Pressed::Revert))
        );
        assert_eq!(
            press("Resume in VS Code", both(Ok(()), Ok(()))),
            Some(Some(Pressed::Resume))
        );
        // Each is disabled on its own.
        let off = |why: &str| Err(why.to_owned());
        assert_eq!(
            press("Revert…", both(Ok(()), off("Revert is off for alpha"))),
            Some(None)
        );
        assert_eq!(
            press("Resume in VS Code", both(off("it runs now"), Ok(()))),
            Some(None)
        );
        assert_eq!(
            press("Resume in VS Code", None),
            None,
            "no buttons without an id"
        );
        assert_eq!(press("Revert…", None), None);
    }

    /// Resume in VS Code is refused for a project or folder it cannot open, an id the link would
    /// drop, a headless run, which the extension opens empty, and a session that may run: the
    /// window's, one open elsewhere, or one the list of live sessions has not answered for; and
    /// while another launch is on its way (EP-35).
    #[test]
    fn a_resume_in_vs_code_is_refused_for_what_it_cannot_open_or_may_run() {
        use crw_core::discovery::Folder;
        use crw_core::proposal::Facts;
        use crw_core::registry::{Project, State};
        use crw_core::roadmap::snapshot::Snapshot;
        let project = ProjectRow {
            project: Project::new("alpha", Path::new("alpha"), State::Active),
            path: PathBuf::from("alpha"),
            folder: Some(Folder {
                name: "alpha".to_owned(),
                path: PathBuf::from("alpha"),
                git: true,
                roadmap: true,
                own_workspace: true,
            }),
            snapshot: Snapshot::default(),
            facts: Facts::default(),
            outgoing: Vec::new(),
            mail: Vec::new(),
            git: None,
            badges: Default::default(),
            house: Default::default(),
        };
        let mut row = ledger_row();
        row.mode = "visual".to_owned();
        let id = row.session_id.clone().unwrap();
        let free = Resumes::default();
        assert_eq!(resume_availability(&row, Some(&project), &free), Ok(()));
        let mut manual = row.clone();
        manual.mode = "manual".to_owned();
        assert_eq!(resume_availability(&manual, Some(&project), &free), Ok(()));
        let headless = resume_availability(&ledger_row(), Some(&project), &free).unwrap_err();
        assert!(headless.starts_with("VS Code opens a headless run as an empty tab"));
        assert!(headless.contains(&format!("claude --resume {id}")));
        let why = |row: &SessionRow, project: Option<&ProjectRow>, resumes: &Resumes| {
            resume_availability(row, project, resumes).unwrap_err()
        };
        assert_eq!(why(&row, None, &free), "alpha is not a registered project");
        let mut gone = project.clone();
        gone.folder = None;
        assert_eq!(
            why(&row, Some(&gone), &free),
            "alpha: its folder is missing"
        );
        let mut odd = row.clone();
        odd.session_id = Some("s1".to_owned());
        assert!(why(&odd, Some(&project), &free).contains("not one the extension's link takes"));
        let mut window = Resumes::default();
        window.window.insert(&id);
        assert!(why(&row, Some(&project), &window).contains("this window's own session"));
        let mut open = Resumes::default();
        open.open.insert(&id);
        assert!(why(&row, Some(&project), &open).starts_with("it runs now"));
        let unknown = Resumes {
            unknown: Some("the list of live sessions has not answered yet".to_owned()),
            ..Resumes::default()
        };
        assert!(why(&row, Some(&project), &unknown).starts_with("the list of live sessions"));
        let opening = Resumes {
            opening: true,
            ..Resumes::default()
        };
        assert!(why(&row, Some(&project), &opening).starts_with("a launch to VS Code"));
        // Another session's being open changes nothing for this one.
        let mut other = Resumes::default();
        other.open.insert("7c4e1a2d-8f53-4b06-9a0b-3b9d2f105e8c");
        assert_eq!(resume_availability(&row, Some(&project), &other), Ok(()));
    }

    #[test]
    fn a_completed_run_that_stopped_before_committing_offers_commit() {
        let mut done = live();
        assert!(!done.offers_commit(), "not while it runs");
        done.take(exit(Outcome::Completed, None));
        assert!(done.offers_commit());
        done.commits = true;
        assert!(!done.offers_commit(), "a run told to commit offers none");
        let mut failed = live();
        failed.take(exit(Outcome::Failed, None));
        assert!(!failed.offers_commit());
        // What the autopilot asked of a run carries to the next: its cap, its chain.
        let mut chained = live();
        chained.cap = Some(Cap::new(Some(Size::M), Instant::now()));
        chained.chained = true;
        assert_eq!(
            chained.run(),
            Run {
                capped: true,
                size: Some(Size::M),
                chained: true,
                commits: false,
                migration: false,
            }
        );
        assert_eq!(live().run(), Run::default());
    }

    #[test]
    fn a_window_sessions_card_opens_its_tab_from_over_its_text() {
        use crate::testing::{self, Screen};
        let live = live();
        let mut screen = Screen::new();
        let mut pass = |events| {
            let mut clicked = false;
            screen.pass(events, |ui| clicked = live_line(ui, &live));
            clicked
        };
        // Over the title's first letters, inside the card's margin.
        let at = egui::pos2(22.0, 16.0);
        let press = |pressed| testing::button(at, egui::PointerButton::Primary, pressed);
        assert!(!pass(Vec::new()));
        assert!(!pass(testing::moved(at)));
        assert!(!pass(press(true)));
        assert!(pass(press(false)), "the card is clicked over its text");
    }

    fn asked(id: &str) -> Request {
        Request {
            id: id.to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which shelf?".to_owned(),
            reason: None,
            questions: vec![crw_core::session::Question {
                text: "Which shelf?".to_owned(),
                header: "Shelf".to_owned(),
                options: Vec::new(),
                multi_select: false,
            }],
            input: String::new(),
        }
    }

    /// The runner's word that a run ended with `outcome`.
    pub(crate) fn exit(outcome: Outcome, result: Option<Finish>) -> Event {
        Event::Exited(Ended {
            outcome,
            exit_code: Some(0),
            result,
            stderr: Vec::new(),
        })
    }

    #[test]
    fn a_request_past_its_limit_or_deferred_by_the_hook_stays_open() {
        // The time limit: the request moves from waiting to open, the run deferred.
        let mut limited = live();
        limited.take(Event::Request(asked("req-1")));
        limited.take(Event::TimedOut("req-1".to_owned()));
        assert!(limited.waiting.is_empty());
        assert_eq!(limited.open.as_ref().unwrap().request.id, "req-1");
        assert!(limited.open.as_ref().unwrap().deferred.is_none());
        limited.take(exit(Outcome::Deferred, None));
        assert_eq!(limited.state(), "ended: deferred");
        let mut sessions = Sessions::default();
        sessions.live.push(limited);
        assert_eq!((sessions.waiting(), sessions.deferred()), (0, 1));
        // The hook: the result's deferred call is the open request.
        let mut live = live();
        let deferred = crw_core::session::Deferred {
            id: "toolu_d".to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which shelf?".to_owned(),
            questions: asked("x").questions,
            input: Default::default(),
        };
        let finish = Finish {
            subtype: "success".to_owned(),
            is_error: false,
            text: None,
            turns: 1,
            cost_usd: 0.01,
            duration_ms: 900,
            stop_reason: Some("tool_deferred".to_owned()),
            deferred: Some(deferred),
            denials: 0,
            errors: Vec::new(),
        };
        live.take(Event::Result(finish.clone()));
        live.take(exit(Outcome::Deferred, Some(finish)));
        let open = live.open.as_ref().unwrap();
        assert_eq!(open.request.id, "toolu_d");
        assert_eq!(open.deferred.as_ref().unwrap().id, "toolu_d");
        // A refusal is kept for the tile, the latest few.
        for n in 0..REFUSALS + 3 {
            live.take(Event::Denied(Denied {
                tool: "Edit".to_owned(),
                message: format!("edit {n} denied"),
                reason: Some("deny rule".to_owned()),
            }));
        }
        assert_eq!((live.denied, live.refusals.len()), (REFUSALS + 3, REFUSALS));
        assert_eq!(live.refusals[0].message, "edit 3 denied");
    }

    #[test]
    fn a_runs_requests_and_end_are_news_for_the_notifications() {
        let finish = |subtype: &str, text: Option<&str>, errors: Vec<String>| Finish {
            subtype: subtype.to_owned(),
            is_error: subtype != "success",
            text: text.map(str::to_owned),
            turns: 3,
            cost_usd: 0.2,
            duration_ms: 60_000,
            stop_reason: None,
            deferred: None,
            denials: 0,
            errors,
        };
        let mut done = live();
        done.take(Event::Request(asked("req-1")));
        done.take(exit(
            Outcome::Completed,
            Some(finish(
                "success",
                Some(
                    "EP-3 is done and committed as `0a1b2c3`. Nothing is pushed.\n\n\
                     **Checks.** All green.\n\n- **Push:** one commit waits for you.",
                ),
                Vec::new(),
            )),
        ));
        // The result's gist, not its last line (EP-20a).
        assert_eq!(
            done.news,
            [
                News::Asked("Which shelf?".to_owned()),
                News::Ended(
                    Outcome::Completed,
                    "EP-3 is done and committed as 0a1b2c3.".to_owned()
                ),
            ]
        );
        // A stop is the owner's own doing: no news.
        let mut stopped = live();
        stopped.take(exit(Outcome::Stopped, None));
        assert!(stopped.news.is_empty());
        // A failure says why: the result's error, its subtype, stderr, the exit code.
        let mut failed = live();
        failed.take(exit(
            Outcome::Failed,
            Some(finish("error_max_budget_usd", None, Vec::new())),
        ));
        assert_eq!(
            failed.news,
            [News::Ended(
                Outcome::Failed,
                "error max budget usd".to_owned()
            )]
        );
        let mut bare = live();
        bare.take(Event::Exited(Ended {
            outcome: Outcome::Failed,
            exit_code: Some(3),
            result: None,
            stderr: vec![
                "first".to_owned(),
                "the CLI's last word".to_owned(),
                " ".to_owned(),
            ],
        }));
        assert_eq!(
            bare.news,
            [News::Ended(
                Outcome::Failed,
                "the CLI's last word".to_owned()
            )]
        );
        // A deferred run's news names the request it left open.
        let mut limited = live();
        limited.take(Event::Request(asked("req-2")));
        limited.take(Event::TimedOut("req-2".to_owned()));
        limited.take(exit(Outcome::Deferred, None));
        assert_eq!(
            limited.news.last(),
            Some(&News::Ended(Outcome::Deferred, "Which shelf?".to_owned()))
        );
        // The window takes them in with each session's key, once.
        let mut sessions = Sessions::default();
        sessions.live.push(done);
        sessions.receive(&egui::Context::default());
        let news = sessions.take_news();
        assert_eq!(news.len(), 2);
        assert!(news.iter().all(|(key, _)| *key == 1));
        assert!(sessions.take_news().is_empty());
    }

    #[test]
    fn a_refused_handshake_sends_the_clis_later_runs_the_defer_route() {
        let mut sessions = Sessions::default();
        let program = PathBuf::from("claude.exe");
        assert_eq!(
            sessions.route(&program, Some("2.1.316"), None),
            (Route::Protocol, None)
        );
        let (route, why) = sessions.route(&program, None, Some(Route::Defer));
        assert_eq!(route, Route::Defer);
        assert!(why.unwrap().contains("settings.decisions"));
        let mut live = live();
        live.take(Event::Handshake(Err("not supported".to_owned())));
        sessions.live.push(live);
        sessions.receive(&egui::Context::default());
        assert!(sessions.switched.take().unwrap().contains("claude 2.1.316"));
        let (route, why) = sessions.route(&program, Some("2.1.316"), None);
        assert_eq!(route, Route::Defer);
        assert!(why.unwrap().contains("refused"));
        // Another version of the CLI, or another CLI, tries the protocol again.
        assert_eq!(
            sessions.route(&program, Some("2.1.320"), None).0,
            Route::Protocol
        );
        let other = PathBuf::from("other.exe");
        assert_eq!(sessions.route(&other, None, None).0, Route::Protocol);
        // Said once.
        sessions.receive(&egui::Context::default());
        assert!(sessions.switched.is_none());
    }

    #[test]
    fn a_sessions_events_make_its_lines_and_transcript() {
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
            input: "{\"command\": \"git push\"}".to_owned(),
        };
        live.take(Event::Request(request.clone()));
        assert_eq!(live.waiting.len(), 1);
        live.take(Event::Withdrawn("req-1".to_owned()));
        assert!(live.waiting.is_empty());
        live.take(Event::Request(request));
        live.take(Event::Notice("a notice".to_owned()));
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
        live.take(Event::Result(result.clone()));
        live.take(Event::Exited(Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: Some(result),
            stderr: Vec::new(),
        }));
        // An exit takes the requests with it; the transcript keeps what was asked.
        assert!(live.waiting.is_empty());
        assert_eq!(live.state(), "ended: completed");
        assert!(!live.is_running());
        assert_eq!(
            ending(live.ended.as_ref().unwrap()),
            "ended completed · exit 0 · 4 turns · 4 s · an estimated $0.420 · EP-3 is done."
        );
        let entries = &live.transcript.entries;
        assert_eq!(entries.len(), 6, "{entries:?}");
        assert!(matches!(&entries[0], Entry::Text(text) if text.starts_with("Reading")));
        // The request withdrawn says so; the one asked again went with the run's end.
        assert!(
            matches!(&entries[2], Entry::Asked { answer: Some(words), .. }
            if words == "withdrawn by the session")
        );
        assert!(matches!(&entries[3], Entry::Asked { answer: None, .. }));
        assert!(matches!(&entries[5], Entry::Result(_)));
    }

    #[test]
    fn a_session_resumes_only_once_ended_with_a_message() {
        let mut sessions = Sessions::default();
        let mut one = live();
        one.draft = "  Go on from the tests. ".to_owned();
        sessions.live.push(one);
        assert_eq!(sessions.resumable(1), None, "it has not ended");
        sessions.live[0].ended = Some(Ended {
            outcome: Outcome::Stopped,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        });
        assert_eq!(
            sessions.resumable(1),
            Some((
                "Go on from the tests.".to_owned(),
                PathBuf::from("claude.exe")
            ))
        );
        sessions.live[0].draft.clear();
        assert_eq!(sessions.resumable(1), None, "no message");
        assert_eq!(sessions.resumable(2), None, "no such session");
        // Dismissing its tab brings All forward.
        sessions.tab = Some(1);
        sessions.dismiss(1);
        assert!(sessions.live.is_empty());
        assert_eq!(sessions.tab, None);
    }

    #[test]
    fn the_filter_keeps_a_project_and_the_modes_shown() {
        let mut filter = SessionFilter::default();
        assert!(filter.keeps("alpha", "visual"));
        filter.project = Some("ALPHA".to_owned());
        filter.hidden_modes.insert("manual".to_owned());
        assert!(filter.keeps("alpha", "visual"));
        assert!(!filter.keeps("beta", "visual"));
        assert!(!filter.keeps("alpha", "manual"));
    }

    #[test]
    fn durations_read_in_words() {
        let row = |started: &str, ended: Option<&str>| SessionRow {
            id: 1,
            session_id: None,
            project: "alpha".to_owned(),
            ep: None,
            mode: "manual".to_owned(),
            model: None,
            effort: None,
            ultracode: false,
            started_at: started.to_owned(),
            ended_at: ended.map(str::to_owned),
            outcome: None,
            cost_usd: None,
            questions: 0,
            result: None,
            context_percent: None,
            answer_latency_s: None,
            start: None,
            usage: Default::default(),
        };
        let start = "2026-11-02T10:00:00Z";
        assert_eq!(
            duration_words(&row(start, Some("2026-11-02T10:00:36Z"))).as_deref(),
            Some("36 s")
        );
        assert_eq!(
            duration_words(&row(start, Some("2026-11-02T10:52:10Z"))).as_deref(),
            Some("52 min")
        );
        assert_eq!(
            duration_words(&row(start, Some("2026-11-02T11:04:00Z"))).as_deref(),
            Some("1 h 04 min")
        );
        assert_eq!(duration_words(&row(start, None)), None);
    }

    #[test]
    fn the_last_line_is_the_last_with_text() {
        assert_eq!(last_line("one\ntwo\n\n  "), Some("two".to_owned()));
        assert_eq!(last_line("  \n"), None);
        let long = "x".repeat(LINE + 10);
        assert_eq!(last_line(&long).unwrap().chars().count(), LINE);
    }
}
