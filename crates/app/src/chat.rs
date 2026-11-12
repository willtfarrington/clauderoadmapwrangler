//! The Chat tile (D-27; EP-24): one conversation at a time with a read-only Claude Code session
//! over the portfolio, started through the window's one start funnel (`Sessions::begin`) with
//! the launch `crw_core::chat` builds, each of the owner's messages a turn that resumes the
//! session by its id. The window builds the digest the session is given from its own readings
//! (the portfolio's rows, the sessions it sees, the usage meter), writes it to the chat folder
//! for the record and passes it inline. The exchange is drawn from the run's transcript as plain
//! text while the reply streams (EP-24a draws it as Markdown); a permission request is refused by
//! the window at once (`sessions::Live::take`), and a question waits in the Decisions tile as
//! any run's. A screenshot run cans a conversation (`chat`), and starts nothing.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Local, Utc};
use crw_core::chat::{self, Digest, Next, ProjectDigest, Turn};
use crw_core::files;
use crw_core::ledger;
use crw_core::session::{Appended, Outcome};
use crw_core::usage;
use eframe::egui::{self, RichText};

use crate::board::{Action, Board, crw_exe};
use crate::sessions::Live;
use crate::style::{self, icons};
use crate::transcript::{self, Entry, Transcript};

/// The room the footer keeps under the exchange: the notes, the input and its buttons.
const FOOTER: f32 = 132.0;

/// The tile's state: one conversation at a time.
#[derive(Default)]
pub struct Chat {
    /// The conversation's live in `Sessions`, once its first turn started.
    pub key: Option<u64>,
    /// The turn on its way: its id and title, taken up once the start comes back.
    pub pending: Option<Pending>,
    /// The session id chosen for the conversation's first turn, kept for its later turns.
    pub session_id: Option<String>,
    /// The conversation's title, from its first prompt.
    pub title: Option<String>,
    /// The message being typed.
    pub draft: String,
    /// Why the last turn did not start.
    pub error: Option<String>,
    /// A turn is starting: the worker thread waits its turn and the spacing.
    pub starting: bool,
    /// What the last digest told the session it reads, and what it left out with why.
    pub reads: Vec<String>,
    pub left_out: Vec<(String, String)>,
    /// Where the last digest was written, for the record.
    pub digest_file: Option<PathBuf>,
}

/// A turn handed to the start thread: the session id it starts or resumes under, and the
/// conversation's title.
pub struct Pending {
    pub session_id: String,
    pub title: String,
}

impl Board {
    /// The conversation's live, when it has one.
    fn conversation(&self) -> Option<&Live> {
        self.chat
            .key
            .and_then(|key| self.sessions.conversation(key))
    }

    /// The folders the session reads (`chat::folders`): each registered project not marked a
    /// data repository whose folder is there, and the plugin copy's.
    fn chat_folders(&self) -> (Vec<PathBuf>, Vec<(String, String)>) {
        let plugin = chat::plugin_dir();
        chat::folders(
            self.portfolio.projects.iter().map(|row| {
                (
                    row.name(),
                    row.folder.as_ref().map(|folder| folder.path.as_path()),
                    row.project.data,
                )
            }),
            plugin.as_deref(),
        )
    }

    /// The digest of the portfolio as the window reads it now (D-27; EP-24), from its own
    /// readings and never from a `crw status` over the registry (EP-7's note).
    pub fn chat_digest(&self) -> Digest {
        let now = Utc::now();
        let own = |id: &str| self.sessions.owns(id);
        let (_, left_out) = self.chat_folders();
        let projects = self
            .portfolio
            .projects
            .iter()
            .filter(|row| !row.project.data && row.folder.is_some())
            .map(|row| {
                let name = row.name();
                let mut sessions: Vec<String> = self.sessions.summary(name).into_iter().collect();
                sessions.extend(self.watch.running(name, own).map(|seen| seen.summary()));
                let counts = row.counts();
                let (open_messages, urgent) = row.open_messages();
                ProjectDigest {
                    name: name.to_owned(),
                    state: row.project.state.as_str().to_owned(),
                    folder: row
                        .folder
                        .as_ref()
                        .map_or_else(|| row.path.clone(), |folder| folder.path.clone()),
                    counts: (counts.total > 0).then_some((counts.done, counts.total)),
                    next: row.next().map(|next| Next {
                        ep: next.id.to_string(),
                        size: next.size.map(|size| size.as_str().to_owned()),
                        title: next.brief.title.clone(),
                    }),
                    open_tasks: row.open_items(),
                    open_messages,
                    urgent,
                    sessions,
                    last_commit: row
                        .facts
                        .last_commit
                        .map(|at| at.with_timezone(&Local).format("%Y-%m-%d").to_string()),
                    hosting: Some(format!("{:?}", row.badges.hosting.class()).to_lowercase()),
                    usage: row.badges.usage.map(usage_words),
                }
            })
            .collect();
        let figures = self.meter.gauge.figures(now);
        let mut figure_words = Vec::new();
        for (name, figure) in [("weekly", figures.weekly), ("Fable", figures.fable)] {
            if let Some(figure) = figure {
                let reset = figure.limit.resets_at.map_or(String::new(), |at| {
                    format!(
                        ", resets {}",
                        at.with_timezone(&Local).format("%a %d %b %H:%M")
                    )
                });
                figure_words.push(format!(
                    "{name} {} of the limit{reset}",
                    usage::percent(figure.limit.percent)
                ));
            }
        }
        Digest {
            date: files::today(),
            projects,
            left_out,
            usage: figure_words,
            reference: chat::reference(chat::plugin_dir().as_deref()),
        }
    }

    /// Send (EP-24): one turn of the conversation, a new session or the conversation's resumed
    /// with the message, off the UI thread as every start is; a screenshot run starts nothing.
    pub(crate) fn chat_send(&mut self) {
        if self.fixtures {
            self.warn("A screenshot run starts no session: its projects are fixtures or a copy.");
            return;
        }
        let message = self.chat.draft.trim().to_owned();
        if message.is_empty() {
            return;
        }
        if self.sessions.is_starting() || self.chat.starting {
            self.chat.error = Some("A session is starting: try again in a moment.".to_owned());
            return;
        }
        if self.conversation().is_some_and(Live::is_running) {
            return;
        }
        let registry = match self.store().load() {
            Ok(loaded) => loaded.registry,
            Err(e) => {
                self.chat.error = Some(format!("The registry could not be read: {e}"));
                return;
            }
        };
        // The conversations' rows carry the project name `chat` (D-27 addendum): a project of
        // that name would mix with them.
        if registry.project(chat::PROJECT).is_some() {
            self.chat.error = Some(format!(
                "A registered project is called {}, the name the conversations' ledger rows \
                 carry: rename it before the chat.",
                chat::PROJECT
            ));
            return;
        }
        let (program, _, route) = self.launch_cli(&registry);
        let ctx = self.ctx().clone();
        let Some(preflight) = self.sessions.preflight(&program, &ctx) else {
            self.chat.error =
                Some("claude has not answered yet: try again in a moment.".to_owned());
            return;
        };
        let refusals = preflight.refusals();
        if !refusals.is_empty() {
            self.chat.error = Some(refusals.join("; "));
            return;
        }
        let Some(dir) = chat::dir() else {
            self.chat.error = Some(
                "no local folder for the chat: neither LOCALAPPDATA nor CRW_HOME is set".to_owned(),
            );
            return;
        };
        if let Err(e) = fs::create_dir_all(&dir) {
            self.chat.error = Some(format!(
                "The chat folder {} was not made: {e}",
                dir.display()
            ));
            return;
        }
        // The turn: the conversation's session goes on, or a new one starts under a chosen id.
        let (turn, title, resumes) = match self.conversation() {
            Some(live) => (
                Turn::Resume {
                    session_id: live.session_id.clone(),
                },
                live.conversation().unwrap_or_default().to_owned(),
                Some(live.key),
            ),
            None => (Turn::new(), chat::title_of(&message), None),
        };
        // The plugin's copy written now, so that the first digest of a fresh profile names
        // the reference (the start writes it again, on its worker).
        let _ = crw_core::plugin::prepare();
        let digest = self.chat_digest();
        let text = chat::digest_text(&digest);
        let file = chat::digest_file(&dir, turn.session_id());
        if let Err(e) = fs::write(&file, &text) {
            self.warn(format!(
                "The digest was not written to {}: {e}; it goes to the session inline.",
                file.display()
            ));
        }
        let (folders, left_out) = self.chat_folders();
        self.chat.reads = digest
            .projects
            .iter()
            .map(|project| project.name.clone())
            .collect();
        self.chat.left_out = left_out;
        self.chat.digest_file = Some(file.clone());
        let places = chat::Places {
            cwd: dir,
            folders,
            digest: Appended { file, text },
            ledger: self.ledger().map(Path::to_owned),
        };
        let mut launch = chat::launch(&title, &message, &turn, places);
        launch.route = route.0;
        launch.hooks = crw_exe();
        // The message, the id and the title stay pending until the start comes back: a
        // refused start keeps the message typed (the review's finding).
        self.chat.pending = Some(Pending {
            session_id: turn.session_id().to_owned(),
            title,
        });
        self.chat.error = None;
        self.sessions.chat(preflight, launch, resumes, &ctx);
        self.chat.starting = self.sessions.is_starting();
        if !self.chat.starting {
            self.chat.pending = None;
            self.chat.error =
                Some("The session's start could not be handed to a thread.".to_owned());
        }
    }

    /// A turn that started, or did not (EP-24): the conversation's live found by its id.
    pub(crate) fn chat_started(&mut self, started: Result<String, String>) {
        self.chat.starting = false;
        match started {
            Ok(text) => {
                if let Some(pending) = self.chat.pending.take() {
                    self.chat.session_id = Some(pending.session_id);
                    self.chat.title = Some(pending.title);
                    self.chat.draft.clear();
                }
                let wanted = self.chat.session_id.clone();
                self.chat.key = self
                    .sessions
                    .live
                    .iter()
                    .find(|live| live.is_chat() && Some(&live.session_id) == wanted.as_ref())
                    .map(|live| live.key)
                    .or(self.chat.key);
                self.notify(text);
            }
            Err(why) => {
                self.chat.pending = None;
                self.sessions.forget_preflight();
                self.chat.error = Some(format!("The session did not start: {why}"));
            }
        }
    }

    /// Stop: the reply's turn interrupted.
    pub(crate) fn chat_stop(&mut self) {
        if let Some(key) = self.chat.key {
            self.sessions.stop(key);
        }
    }

    /// New conversation: the ended one let go (the ledger keeps its rows), the next Send a new
    /// session; refused while a reply streams.
    pub(crate) fn chat_new(&mut self) {
        if self.conversation().is_some_and(Live::is_running) || self.chat.starting {
            self.warn("The conversation is still replying: Stop it first.");
            return;
        }
        if let Some(key) = self.chat.key.take() {
            self.sessions.dismiss(key);
        }
        self.chat.session_id = None;
        self.chat.title = None;
        self.chat.error = None;
        self.chat.reads.clear();
        self.chat.left_out.clear();
        self.chat.digest_file = None;
    }

    /// The Chat tile.
    pub fn chat_pane(&mut self, ui: &mut egui::Ui) {
        let mut actions = Vec::new();
        // A turn of this conversation on its way; another start's only holds Send back.
        let starting = self.chat.starting;
        let busy = self.sessions.is_starting();
        let (running, stopping, state, colour, dropped, ended, init) = match self.conversation() {
            Some(live) => (
                live.is_running(),
                live.stopping,
                Some(live.state()),
                Some(live_colour(live, ui)),
                live.dropped.clone(),
                live.ended.as_ref().map(|ended| ended.outcome),
                live.init.as_ref().map(init_words),
            ),
            None => (false, false, None, None, Vec::new(), None, None),
        };
        // The title, the chip and the buttons on rows of their own, so that a narrow pane
        // neither squeezes a chip nor gets widened past its edge (EP-13's lesson).
        let title = match &self.chat.title {
            Some(title) => format!("chat: {title}"),
            None => "Chat".to_owned(),
        };
        ui.label(RichText::new(title).strong());
        if let (Some(state), Some(colour)) = (&state, colour) {
            ui.horizontal(|ui| {
                // The CLI's own account of the session on hover: its version, mode and tools,
                // which the first real conversation checks (OT-26).
                let chip = style::chip(ui, state, colour);
                match &init {
                    Some(words) => chip.on_hover_text(words),
                    None => chip.on_hover_text("The session's init has not come yet"),
                };
            });
        } else if starting {
            ui.horizontal(|ui| style::chip(ui, "starting", ui.visuals().weak_text_color()));
        }
        ui.label(RichText::new(self.chat_reads_line()).weak())
            .on_hover_text(self.chat_reads_hover());
        // Wrapped: a narrow pane puts Stop on the next line rather than past its edge.
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(!running && !starting, egui::Button::new("New conversation"))
                .on_hover_text(
                    "Let this conversation go (the ledger keeps its rows) and start the next \
                     Send as a new session",
                )
                .clicked()
            {
                actions.push(Action::ChatNew);
            }
            if running
                && ui
                    .add_enabled(!stopping, egui::Button::new("Stop"))
                    .on_hover_text("Interrupt the reply; the conversation can go on afterwards")
                    .clicked()
            {
                actions.push(Action::ChatStop);
            }
        });
        for note in &dropped {
            ui.label(RichText::new(note).small().color(style::amber(ui)));
        }
        ui.separator();

        let height = (ui.available_height() - FOOTER).max(80.0);
        egui::ScrollArea::vertical()
            .id_salt("chat-exchange")
            .auto_shrink(false)
            .stick_to_bottom(true)
            .max_height(height)
            .show(ui, |ui| match self.conversation() {
                Some(live) => {
                    exchange(ui, &live.transcript);
                    if live.is_running() && live.transcript.streaming.is_empty() {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(RichText::new("Replying…").weak());
                        });
                    }
                }
                None => {
                    ui.add_space(24.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(
                                "Ask about the portfolio: what a brief plans, what waits on you, \
                                 how a project stands. The session reads the projects' files and \
                                 writes nothing; each message is a turn of one Claude Code \
                                 session, and the ledger keeps the conversation.",
                            )
                            .weak(),
                        );
                    });
                }
            });
        if let Some(Outcome::Failed) = ended
            && let Some(live) = self.conversation()
            && let Some(ended) = &live.ended
            && let Some(line) = ended.stderr.last()
        {
            ui.label(RichText::new(line).small().color(style::red(ui)));
        }
        ui.separator();
        if let Some(error) = &self.chat.error {
            ui.label(RichText::new(error).color(style::red(ui)));
        }
        let response = ui.add(
            egui::TextEdit::multiline(&mut self.chat.draft)
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .hint_text("Ask about the portfolio (Ctrl+Enter sends)"),
        );
        let ready = !running && !starting && !busy && !self.chat.draft.trim().is_empty();
        let sent_by_key = response.has_focus()
            && ready
            && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter));
        let send = ui
            .add_enabled(ready, egui::Button::new("Send"))
            .on_hover_text(
                "One turn of the conversation: a new read-only session, or the conversation's \
                 resumed with this message",
            );
        if send.clicked() || sent_by_key {
            actions.push(Action::ChatSend);
        }
        // The session's short id on a line of its own, which wraps; its full id on hover.
        let short = self.chat.session_id.as_deref().map(crw_core::revert::short);
        let words = match (short, ended) {
            (_, _) if starting => "starting the session".to_owned(),
            (Some(id), Some(outcome)) => format!("session {id} · ended: {}", outcome.as_str()),
            (Some(id), None) if running => format!("session {id} · replying"),
            (Some(id), None) => format!("session {id}"),
            (None, _) => "a new session on Send".to_owned(),
        };
        let line = ui.label(RichText::new(words).small().weak());
        if let Some(id) = &self.chat.session_id {
            line.on_hover_text(format!("session {id}"));
        }
        self.actions.extend(actions);
    }

    /// The header's line: the model and effort, what the session reads.
    fn chat_reads_line(&self) -> String {
        let reads = if self.chat.reads.is_empty() {
            let readable = self
                .portfolio
                .projects
                .iter()
                .filter(|row| !row.project.data && row.folder.is_some())
                .count();
            format!("reads {} on Send", count(readable, "project"))
        } else {
            format!("reads {}", count(self.chat.reads.len(), "project"))
        };
        let left_out = match self.chat.left_out.len() {
            0 => String::new(),
            n => format!(", {n} left out"),
        };
        format!(
            "{} at {} · {reads}{left_out} · read-only",
            chat::MODEL,
            chat::EFFORT
        )
    }

    /// What the header's line says on hover: the projects read, those left out with why, and
    /// the digest's file.
    fn chat_reads_hover(&self) -> String {
        let mut lines = Vec::new();
        if !self.chat.reads.is_empty() {
            lines.push(format!("Reads: {}.", self.chat.reads.join(", ")));
        }
        for (name, why) in &self.chat.left_out {
            lines.push(format!("Left out: {name} ({why})."));
        }
        if let Some(file) = &self.chat.digest_file {
            lines.push(format!("The digest it was given: {}.", file.display()));
        }
        if lines.is_empty() {
            lines.push(
                "Every registered project not marked data = true, read-only, with the \
                 house-style reference; the digest is written at each Send."
                    .to_owned(),
            );
        }
        lines.join("\n")
    }
}

/// A project's share of the week, in words (EP-18b's figures).
fn usage_words(usage: crate::portfolio::Usage) -> String {
    let mut parts = Vec::new();
    if let Some(weekly) = usage.weekly {
        parts.push(format!("week +{} %", ledger::points(weekly)));
    }
    if let Some(fable) = usage.fable {
        parts.push(format!("Fable +{} %", ledger::points(fable)));
    }
    format!(
        "{} over {}",
        parts.join(" · "),
        count(usage.sessions as usize, "session")
    )
}

fn count(n: usize, noun: &str) -> String {
    match n {
        1 => format!("1 {noun}"),
        n => format!("{n} {noun}s"),
    }
}

/// The session's init in words: `claude 2.1.316 · claude-opus-5-5 · manual · tools: Read, Glob`.
fn init_words(init: &crw_core::session::Init) -> String {
    let mut parts = Vec::new();
    if let Some(version) = &init.version {
        parts.push(format!("claude {version}"));
    }
    if let Some(model) = &init.model {
        parts.push(model.clone());
    }
    if let Some(mode) = &init.permission_mode {
        parts.push(mode.clone());
    }
    parts.push(if init.tools.is_empty() {
        "tools: none listed".to_owned()
    } else {
        format!("tools: {}", init.tools.join(", "))
    });
    parts.join(" · ")
}

fn live_colour(live: &Live, ui: &egui::Ui) -> egui::Color32 {
    match &live.ended {
        Some(ended) if ended.outcome == Outcome::Completed => style::green(ui),
        Some(ended) if ended.outcome == Outcome::Deferred => style::amber(ui),
        Some(_) => style::red(ui),
        None if !live.is_running() || live.stopping => ui.visuals().weak_text_color(),
        None if !live.waiting.is_empty() => style::amber(ui),
        None => style::blue(ui),
    }
}

/// The exchange, from the run's transcript: the owner's messages, the replies as plain text,
/// each tool call in one small line, the refusals, the questions and the notices; the result's
/// errors and its cost, not its text, which the reply holds already.
fn exchange(ui: &mut egui::Ui, transcript: &Transcript) {
    let weak = ui.visuals().weak_text_color();
    for (index, entry) in transcript.entries.iter().enumerate() {
        match entry {
            Entry::Prompt { text, at, .. } => {
                if index > 0 {
                    ui.add_space(6.0);
                }
                ui.label(
                    RichText::new(format!("You · {}", at.format("%H:%M")))
                        .small()
                        .strong()
                        .color(style::reference(ui)),
                );
                ui.label(text);
                ui.add_space(4.0);
            }
            Entry::Text(text) => {
                ui.label(text);
                ui.add_space(4.0);
            }
            Entry::Tool { call, result } => {
                transcript::tool_line(ui, &call.tool, &call.summary);
                if let Some(result) = result
                    && result.is_error
                {
                    ui.label(
                        RichText::new(format!("failed: {}", first_line(&result.excerpt)))
                            .small()
                            .color(style::red(ui)),
                    );
                }
            }
            Entry::Denied(denied) => {
                ui.label(
                    RichText::new(format!("Refused: {} ({})", denied.tool, denied.message))
                        .small()
                        .color(style::red(ui)),
                );
            }
            Entry::Asked { request, answer } => {
                let words = match answer {
                    Some(words) => words.clone(),
                    None => "waiting: the Decisions pane answers it".to_owned(),
                };
                ui.horizontal_wrapped(|ui| {
                    ui.label(style::icon(icons::ICON_CHAT, Some(style::amber(ui))));
                    ui.label(
                        RichText::new(format!("{} → {words}", request.summary))
                            .small()
                            .color(style::amber(ui)),
                    );
                });
            }
            Entry::Notice(text) => {
                ui.label(RichText::new(text).small().color(style::amber(ui)));
            }
            Entry::Result(finish) => {
                for error in &finish.errors {
                    ui.label(RichText::new(error).small().color(style::red(ui)));
                }
                ui.label(
                    RichText::new(format!(
                        "{} · {} s · an estimated ${:.3}",
                        finish.subtype.replace('_', " "),
                        finish.duration_ms / 1000,
                        finish.cost_usd
                    ))
                    .small()
                    .color(weak),
                );
            }
        }
        ui.add_space(2.0);
    }
    if !transcript.streaming.is_empty() {
        ui.label(format!("{}…", transcript.streaming));
    }
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crw_core::registry::{Defaults, Store};
    use crw_core::session::{Event, Launch, ToolResult, ToolUse};
    use eframe::egui;

    use super::*;
    use crate::board::{Places, View};
    use crate::portfolio::tests::Scratch;
    use crate::sessions::Live;
    use crate::testing::{self, Screen};

    fn board(scratch: &Scratch) -> Board {
        let root = scratch.0.join("data");
        fs::create_dir_all(root.join("alpha").join("roadmap")).unwrap();
        fs::write(
            root.join("alpha").join("roadmap").join("README.md"),
            "# alpha\n\n## Phase P1\n\n| # | Brief | Size | Depends on | Core | Done |\n\
             |---|---|---|---|---|---|\n| EP-1 | Boot | S | — | core | ☑ `abc1234` |\n\
             | EP-2 | Lists | M | EP-1 | core | ☐ |\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("beta")).unwrap();
        let registry = scratch.0.join("registry.toml");
        fs::write(
            &registry,
            format!(
                "format = 1\n\n[settings]\ndata_root = {:?}\n\n[[project]]\nname = \"alpha\"\n\
                 path = \"alpha\"\nstate = \"active\"\n\n[[project]]\nname = \"beta\"\n\
                 path = \"beta\"\nstate = \"paused\"\ndata = true\n",
                root.display()
            ),
        )
        .unwrap();
        let store = Store::new(
            registry,
            scratch.0.join("backups"),
            Defaults {
                repository: root.join("alpha"),
                data_root: root,
            },
        );
        let places = Places {
            store,
            ledger: None,
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        let mut board = Board::new(places, &ctx, View::default(), egui::ThemePreference::Light);
        // The files read, as a frame would take them in.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while board.portfolio.projects.is_empty() && std::time::Instant::now() < deadline {
            board.loader.receive(&mut board.portfolio);
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        board
    }

    /// The digest from a board's readings (EP-24): the readable project with its roadmap and
    /// counts, the data repository left out with why, the folders the session is given.
    #[test]
    fn the_digest_comes_from_the_windows_readings() {
        let scratch = Scratch::new("chat-digest-board");
        let board = board(&scratch);
        assert_eq!(board.portfolio.projects.len(), 2, "both projects read");
        let digest = board.chat_digest();
        assert_eq!(digest.projects.len(), 1);
        let alpha = &digest.projects[0];
        assert_eq!(alpha.name, "alpha");
        assert_eq!(alpha.state, "active");
        assert_eq!(alpha.counts, Some((1, 2)));
        assert_eq!(
            alpha
                .next
                .as_ref()
                .map(|next| (next.ep.as_str(), next.title.as_str())),
            Some(("EP-2", "Lists"))
        );
        assert_eq!(alpha.next.as_ref().unwrap().size.as_deref(), Some("M"));
        assert_eq!(digest.left_out.len(), 1);
        assert_eq!(digest.left_out[0].0, "beta");
        assert!(digest.left_out[0].1.contains("data = true"));
        let text = chat::digest_text(&digest);
        assert!(text.contains("- alpha (active)"));
        assert!(text.contains("Not readable by this session: beta"));
        let (folders, _) = board.chat_folders();
        // The plugin copy's folder may join (its place depends on the environment).
        assert!(folders[0].ends_with("alpha"));
        assert!(folders.iter().all(|folder| !folder.ends_with("beta")));
        assert!(board.chat_reads_line().contains("reads 1 project on Send"));
    }

    /// The tile draws a canned conversation (EP-24): the exchange, Send disabled while the reply
    /// streams and enabled once it has ended with a draft typed, New conversation the other way.
    #[test]
    fn the_tile_draws_the_exchange_and_gates_send() {
        let scratch = Scratch::new("chat-tile");
        let mut board = board(&scratch);
        board.fixtures = true;
        let mut live = Live::canned(
            950,
            canned_launch(),
            chat::EFFORT,
            vec![
                Event::ToolUse(ToolUse {
                    id: "t1".to_owned(),
                    tool: "Read".to_owned(),
                    summary: "Read alpha/roadmap/README.md".to_owned(),
                }),
                Event::ToolResult(ToolResult {
                    tool_use_id: "t1".to_owned(),
                    is_error: false,
                    lines: 9,
                    excerpt: "# alpha".to_owned(),
                }),
                Event::Text("EP-2, Lists, is next: an M.".to_owned()),
                Event::TextDelta("It plans".to_owned()),
            ],
        );
        live.project = chat::PROJECT.to_owned();
        board.sessions.live.insert(0, live);
        board.chat.key = Some(950);
        board.chat.title = Some("What is next in alpha?".to_owned());
        board.chat.session_id = Some("3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f".to_owned());
        board.chat.draft = "And after that?".to_owned();
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        assert!(testing::text_at(&output, "chat: What is next in alpha?").is_some());
        assert!(testing::text_at(&output, "EP-2, Lists, is next: an M.").is_some());
        assert!(testing::text_at(&output, "It plans…").is_some());
        // The tool line's galley starts with its icon, so it is not found by its words alone.
        assert!(testing::text_at(&output, "Stop").is_some());
        assert!(testing::text_at(&output, "New conversation").is_some());
        // Send is drawn disabled while the reply streams: a click reaches nothing.
        let send = testing::text_at(&output, "Send").expect("the Send button");
        screen.pass(testing::moved(send), |ui| board.chat_pane(ui));
        screen.pass(
            testing::button(send, egui::PointerButton::Primary, true),
            |ui| board.chat_pane(ui),
        );
        screen.pass(
            testing::button(send, egui::PointerButton::Primary, false),
            |ui| board.chat_pane(ui),
        );
        assert!(board.actions.is_empty(), "{:?}", board.actions);
        // Ended: Send goes through (a screenshot run's warns rather than starts).
        board.sessions.live[0].end_canned(crw_core::session::Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        });
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        assert!(testing::text_at(&output, "ended: completed").is_some());
        let send = testing::text_at(&output, "Send").expect("the Send button");
        screen.pass(testing::moved(send), |ui| board.chat_pane(ui));
        screen.pass(
            testing::button(send, egui::PointerButton::Primary, true),
            |ui| board.chat_pane(ui),
        );
        screen.pass(
            testing::button(send, egui::PointerButton::Primary, false),
            |ui| board.chat_pane(ui),
        );
        assert_eq!(board.actions, [Action::ChatSend]);
        board.actions.clear();
        board.chat_send();
        assert_eq!(
            board.chat.draft, "And after that?",
            "a screenshot run starts nothing"
        );
        // New conversation lets the ended one go.
        board.chat_new();
        assert!(board.chat.key.is_none());
        assert!(board.chat.title.is_none());
        assert!(
            board.sessions.live.is_empty(),
            "the ended live was dismissed"
        );
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        assert!(testing::text_at(&output, "Chat").is_some());
    }

    fn canned_launch() -> Launch {
        let mut launch = Launch::new(
            chat::PROJECT,
            Path::new("chat"),
            chat::MODEL,
            "What is next in alpha?",
        );
        launch.effort = Some(chat::EFFORT.to_owned());
        launch.conversation = Some("What is next in alpha?".to_owned());
        launch
    }
}
