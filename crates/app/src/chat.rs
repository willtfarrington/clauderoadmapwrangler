//! The Chat tile (D-27; EP-24): one conversation at a time with a read-only Claude Code session
//! over the portfolio, started through the window's one start funnel (`Sessions::begin`) with
//! the launch `crw_core::chat` builds, each of the owner's messages a turn that resumes the
//! session by its id. The window builds the digest the session is given from its own readings
//! (the portfolio's rows, the sessions it sees, the usage meter), writes it to the chat folder
//! for the record and passes it inline. The exchange is drawn from the run's transcript, the
//! replies as Markdown through `egui_commonmark` while they stream (EP-24a), a link's click
//! opening nothing (the target goes to the status bar); a permission request is refused by the
//! window at once (`sessions::Live::take`), and the session's questions are answered here with
//! the Decisions form (EP-24a). The ledger's conversations are listed, the latest activity
//! first, and one is picked up again with `--resume`; the model and effort are the
//! conversation's, picked in the header (EP-24a). The actions a reply proposes in its
//! `actions` block are drawn as chips under it, each opening the window's own dialog with the
//! project filled in (the formalise preview, the launch dialog with the migration prompt, the
//! compose dialog with the words typed, an Adopt confirmation that asks for the data mark
//! first), nothing written before the owner's click there; the session runs the tool's own
//! `crw` reading commands, and an `@<project>` in the owner's message gives the digest that
//! project's context, the prompt hook's own (EP-24b). A screenshot run cans a conversation
//! (`chat`, `chat=asks`, `chat=actions`), and starts nothing.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Utc};
use crw_core::catalog::Catalog;
use crw_core::chat::{self, Digest, Next, Pick, ProjectDigest, Proposed, Turn};
use crw_core::files;
use crw_core::ledger::{self, ConversationRow, Ledger};
use crw_core::session::{Appended, Outcome, Request};
use crw_core::sessions::Registered;
use crw_core::signals::mentions;
use crw_core::usage;
use eframe::egui::{self, OutputCommand, RichText};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::board::{Action, Board, crw_exe};
use crate::decisions::{self, Draft, Wait};
use crate::launch;
use crate::sessions::Live;
use crate::style::{self, icons};
use crate::transcript::{self, Entry, Transcript};

/// The room the footer keeps under the exchange: the notes, the input and its buttons.
const FOOTER: f32 = 132.0;

/// The most conversations the list shows, the latest activity first (the ledger keeps them all;
/// `crw chats --all` prints every one).
pub const CONVERSATIONS_SHOWN: usize = 20;

/// How often the list reads the ledger again while the pane shows and nothing marked it stale.
const LIST_EVERY: Duration = Duration::from_secs(30);

/// The pickers' widths, in points: each takes the rest of its row, between these, so that a
/// narrow pane is never widened past its edge by one (EP-13's lesson, seen at 1200 points).
const PICKER_MIN: f32 = 80.0;
const PICKER_MAX: f32 = 220.0;

/// A picker's width on its row: what is left beside its label, within the bounds.
fn picker_width(ui: &egui::Ui) -> f32 {
    (ui.available_width() - ui.spacing().item_spacing.x).clamp(PICKER_MIN, PICKER_MAX)
}

/// The tile's state: one conversation at a time.
#[derive(Default)]
pub struct Chat {
    /// The conversation's live in `Sessions`, once its first turn started.
    pub key: Option<u64>,
    /// The turn on its way: its id and title, taken up once the start comes back.
    pub pending: Option<Pending>,
    /// The session id chosen for the conversation's first turn, kept for its later turns; a
    /// conversation picked up from the list carries its row's (EP-24a).
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
    /// The model and effort of the conversation's next turn (EP-24a): the defaults for a new
    /// one, a picked-up conversation's row's, the header's pickers after.
    pub pick: Pick,
    /// The conversation was picked up from the ledger's list: its earlier turns are the
    /// session's own record, and the exchange holds the turns since (EP-24a).
    pub picked_up: bool,
    /// The ledger's conversations, for the list (EP-24a).
    pub list: Conversations,
    /// Open the list's fold at the next draw (a screenshot run's `chat=list`).
    pub unfold: bool,
    /// An Adopt chip taken (EP-24b): the confirmation under the exchange, which asks whether
    /// the folder is a data repository before anything is written.
    pub adopting: Option<Adopting>,
    /// The projects the last message mentioned with `@`, whose context the digest carried
    /// (EP-24b), for the header's hover.
    pub mentioned: Vec<String>,
    /// Claude Code's model catalog, read once for the picker.
    catalog: Option<Catalog>,
    /// The Markdown viewer's cache, one per conversation.
    cache: CommonMarkCache,
}

/// The Adopt chip's confirmation (EP-24b): the folder, and whether the owner marks it a data
/// repository with the adoption (`data = true`, which keeps it out of every chat session).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adopting {
    pub folder: String,
    pub data: bool,
}

/// What the window knows for the chips (EP-24b): the registered projects' names and the
/// unadopted folders', so that a chip opens a dialog only on a target the window has.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Known {
    pub projects: Vec<String>,
    pub unadopted: Vec<String>,
}

impl Known {
    fn project(&self, name: &str) -> bool {
        self.projects.iter().any(|p| p.eq_ignore_ascii_case(name))
    }

    fn unadopted(&self, name: &str) -> bool {
        self.unadopted.iter().any(|f| f.eq_ignore_ascii_case(name))
    }
}

/// A turn handed to the start thread: the session id it starts or resumes under, the
/// conversation's title, and the projects its message mentioned, whose context went with
/// the digest (EP-24b; taken up only once the start came back, the review's finding).
pub struct Pending {
    pub session_id: String,
    pub title: String,
    pub mentioned: Vec<String>,
}

/// The ledger's conversations as the list shows them (EP-24a), read as the Sessions pane reads
/// its rows: when the pane first shows, again when a start came back or a run ended, and
/// every half minute while it shows.
#[derive(Default)]
pub struct Conversations {
    /// The latest activity first.
    pub rows: Vec<ConversationRow>,
    pub problem: Option<String>,
    read_at: Option<Instant>,
    /// The ledger changed since the last read.
    pub stale: bool,
    /// A screenshot run's or a test's rows: the ledger is not read.
    pub canned: bool,
}

impl Conversations {
    fn refresh(&mut self, ledger: Option<&Path>) {
        let due = self.stale || self.read_at.is_none_or(|at| at.elapsed() >= LIST_EVERY);
        let Some(path) = ledger.filter(|_| due && !self.canned) else {
            return;
        };
        self.read_at = Some(Instant::now());
        self.stale = false;
        if !path.exists() {
            self.rows.clear();
            return;
        }
        match Ledger::open_read_only(path)
            .and_then(|ledger| ledger.conversations(CONVERSATIONS_SHOWN))
        {
            Ok(rows) => {
                self.rows = rows;
                self.problem = None;
            }
            Err(e) => self.problem = Some(format!("The ledger did not read: {e}")),
        }
    }
}

impl Board {
    /// The conversation's live, when it has one.
    fn conversation(&self) -> Option<&Live> {
        self.chat
            .key
            .and_then(|key| self.sessions.conversation(key))
    }

    /// How many questions the conversation waits on, answered in this tile (EP-24a): the Chat
    /// tab's count.
    pub fn chat_asks(&self) -> usize {
        self.conversation().map_or(0, |live| {
            live.waiting.len() + usize::from(live.open.is_some() && live.ended.is_some())
        })
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
                    older_forms: row.snapshot.older_forms_wait(),
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
            unadopted: self
                .portfolio
                .unadopted
                .iter()
                .map(|folder| folder.name.clone())
                .collect(),
        }
    }

    /// The context of the projects `message` mentions with `@` (EP-24b): the prompt hook's
    /// own (`mentions::context`, EP-39's order), over the readable projects alone, so that a
    /// data repository's files are never read into the chat; the chat folder is no project's,
    /// so none is left out as the session's own. With it, the names given, for the header.
    pub(crate) fn chat_mentions(&self, message: &str, cwd: &Path) -> (Option<String>, Vec<String>) {
        if !message.contains('@') {
            return (None, Vec::new());
        }
        let registered: Vec<Registered> = self
            .portfolio
            .projects
            .iter()
            .filter(|row| !row.project.data)
            .filter_map(|row| {
                Some(Registered {
                    name: row.name().to_owned(),
                    folder: row.folder.as_ref()?.path.clone(),
                    revert: row.project.revert,
                })
            })
            .collect();
        let context = mentions::context(message, cwd, &registered);
        let mentioned = if context.is_some() {
            mentions::names(message)
                .iter()
                .filter_map(|name| {
                    registered
                        .iter()
                        .find(|project| project.name.eq_ignore_ascii_case(name))
                        .map(|project| project.name.clone())
                })
                .take(mentions::LIMIT)
                .collect()
        } else {
            Vec::new()
        };
        (context, mentioned)
    }

    /// What the chips can open (EP-24b).
    fn chat_known(&self) -> Known {
        Known {
            projects: self
                .portfolio
                .projects
                .iter()
                .map(|row| row.name().to_owned())
                .collect(),
            unadopted: self
                .portfolio
                .unadopted
                .iter()
                .map(|folder| folder.name.clone())
                .collect(),
        }
    }

    /// A chip taken (EP-24b): the window's own dialog opened on the target, or the Adopt
    /// confirmation shown; nothing is written here.
    pub(crate) fn chat_act(&mut self, proposed: Proposed) {
        match proposed {
            // The preview opens one at a time, silently otherwise (the review's nit).
            Proposed::Formalise { .. } if self.formalise.is_some() => {
                self.warn("The formalise preview is open already: close it first.");
            }
            Proposed::Formalise { project } => self.open_formalise(&project),
            Proposed::Migrate { project } => self.open_migration(&project),
            // A message being written would be replaced (the review's nit).
            Proposed::Compose { .. }
                if self
                    .compose
                    .as_ref()
                    .is_some_and(|dialog| !dialog.text.trim().is_empty()) =>
            {
                self.warn(
                    "A message is being written in the compose dialog: send or cancel it first.",
                );
            }
            Proposed::Compose { project, words } => self.open_compose_with(&project, &words),
            Proposed::Adopt { folder } => {
                match self
                    .portfolio
                    .unadopted
                    .iter()
                    .find(|one| one.name.eq_ignore_ascii_case(&folder))
                {
                    Some(one) => {
                        self.chat.adopting = Some(Adopting {
                            folder: one.name.clone(),
                            data: false,
                        });
                    }
                    None => self.warn(format!(
                        "{folder} is not an unadopted folder of the data root: nothing to adopt."
                    )),
                }
            }
            Proposed::Unknown { line } => {
                self.warn(format!("Not an action the window knows: {line}"));
            }
        }
    }

    /// The confirmation's Adopt (EP-24b): the folder adopted as the list's Adopt does, then
    /// marked a data repository when the box was ticked, so that the next Send leaves it out.
    pub(crate) fn chat_adopt(&mut self, folder: &str, data: bool) {
        self.chat.adopting = None;
        let today = files::today();
        let Some(adopted) = self.adopt_folder(folder, &today) else {
            return;
        };
        if !data {
            self.notify(adopted);
            return;
        }
        match self.store().change(&today, |c| c.set_data(folder, true)) {
            Ok(_) => self.notify(format!(
                "{adopted} Marked a data repository (data = true): the chat never reads it."
            )),
            Err(e) => self.fail(format!(
                "{adopted} Its data mark was not written: {e}. Run crw set {folder} data yes \
                 before the next Send, or the chat is given its folder."
            )),
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
        // The turn: the conversation's session goes on, a conversation picked up from the
        // list is resumed under a fresh live (EP-24a), or a new one starts under a chosen id.
        let (turn, title, resumes) = match (self.conversation(), &self.chat.session_id) {
            (Some(live), _) => (
                Turn::Resume {
                    session_id: live.session_id.clone(),
                },
                live.conversation().unwrap_or_default().to_owned(),
                Some(live.key),
            ),
            (None, Some(session_id)) => (
                Turn::Resume {
                    session_id: session_id.clone(),
                },
                self.chat
                    .title
                    .clone()
                    .unwrap_or_else(|| chat::title_of(&message)),
                None,
            ),
            (None, None) => (Turn::new(), chat::title_of(&message), None),
        };
        // The plugin's copy written now, so that the first digest of a fresh profile names
        // the reference (the start writes it again, on its worker).
        let _ = crw_core::plugin::prepare();
        let digest = self.chat_digest();
        // A mentioned project's context goes with the digest for this turn (EP-24b).
        let (context, mentioned) = self.chat_mentions(&message, &dir);
        let text = chat::appended(&chat::digest_text(&digest), context.as_deref());
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
            readable: self.chat.reads.clone(),
            // The rules name `crw` bare: the child's PATH starts with the folder the window
            // runs beside, where crw.exe is (EP-24b).
            bin: crw_exe().and_then(|exe| exe.parent().map(Path::to_owned)),
            digest: Appended { file, text },
            ledger: self.ledger().map(Path::to_owned),
        };
        let mut launch = chat::launch(&title, &message, &turn, places, &self.chat.pick);
        launch.route = route.0;
        launch.hooks = crw_exe();
        // The message, the id and the title stay pending until the start comes back: a
        // refused start keeps the message typed (the review's finding).
        self.chat.pending = Some(Pending {
            session_id: turn.session_id().to_owned(),
            title,
            mentioned,
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
                    self.chat.mentioned = pending.mentioned;
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

    /// Whether the conversation can be let go now: not while its reply streams, nor while any
    /// start is on its way (a deferred question's resume among them, the review's finding),
    /// which would leave a running chat no pane shows.
    fn chat_can_let_go(&mut self) -> bool {
        if self.conversation().is_some_and(Live::is_running)
            || self.chat.starting
            || self.sessions.is_starting()
        {
            self.warn(
                "The conversation is still replying, or a session is starting: wait for it, or \
                 Stop it.",
            );
            return false;
        }
        true
    }

    /// What going on from an ended conversation that still holds a deferred question says.
    fn chat_let_go_words(&self) -> &'static str {
        if self.conversation().is_some_and(|live| live.open.is_some()) {
            " Its open question went with it: picking the conversation up again resumes the \
             session, which asks again."
        } else {
            ""
        }
    }

    /// New conversation: the ended one let go (the ledger keeps its rows), the next Send a new
    /// session at the default model and effort; refused while a reply streams or a session
    /// starts.
    pub(crate) fn chat_new(&mut self) {
        if !self.chat_can_let_go() {
            return;
        }
        let went = self.chat_let_go_words();
        if let Some(key) = self.chat.key.take() {
            self.sessions.dismiss(key);
        }
        self.chat.session_id = None;
        self.chat.title = None;
        self.chat.error = None;
        self.chat.reads.clear();
        self.chat.left_out.clear();
        self.chat.digest_file = None;
        self.chat.pick = Pick::default();
        self.chat.picked_up = false;
        self.chat.adopting = None;
        self.chat.mentioned.clear();
        self.chat.cache = CommonMarkCache::default();
        if !went.is_empty() {
            self.notify(format!("The conversation was let go.{went}"));
        }
    }

    /// A conversation picked up from the list (EP-24a): the open one let go, the tile taking
    /// the row's title, session id, model and effort; the next Send resumes its session with
    /// the message. Refused while a reply streams or a session starts.
    pub(crate) fn chat_pick_up(&mut self, session_id: &str) {
        if self.chat.session_id.as_deref() == Some(session_id) {
            return;
        }
        if !self.chat_can_let_go() {
            return;
        }
        let went = self.chat_let_go_words();
        let Some(row) = self
            .chat
            .list
            .rows
            .iter()
            .find(|row| row.session_id == session_id)
            .cloned()
        else {
            self.warn("That conversation is no longer in the list.");
            return;
        };
        if let Some(key) = self.chat.key.take() {
            self.sessions.dismiss(key);
        }
        self.chat.session_id = Some(row.session_id);
        self.chat.title = Some(row.title.clone());
        self.chat.pick = Pick {
            model: row.model,
            effort: row.effort,
        };
        self.chat.picked_up = true;
        self.chat.error = None;
        self.chat.reads.clear();
        self.chat.left_out.clear();
        self.chat.digest_file = None;
        self.chat.adopting = None;
        self.chat.mentioned.clear();
        self.chat.cache = CommonMarkCache::default();
        self.notify(format!(
            "Picked up chat: {}: your next message resumes its session.{went}",
            row.title
        ));
    }

    /// The Chat tile.
    pub fn chat_pane(&mut self, ui: &mut egui::Ui) {
        if self.chat.catalog.is_none() {
            self.chat.catalog = Some(self.read_catalog());
        }
        if std::mem::take(&mut self.sessions.conversations_stale) {
            self.chat.list.stale = true;
        }
        let ledger = self.ledger().map(Path::to_owned);
        self.chat.list.refresh(ledger.as_deref());
        let preflight = self.sessions.preflight_ready();
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
        let open = self.chat.session_id.is_some();
        // The title, the chip, the pickers and the buttons on rows of their own, so that a
        // narrow pane neither squeezes a chip nor gets widened past its edge (EP-13's lesson).
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
        // The conversation's model and effort (EP-24a): greyed while a turn runs, since the
        // choice goes with the next message.
        {
            let Chat { catalog, pick, .. } = &mut self.chat;
            let catalog = catalog.as_ref().expect("the catalog is read above");
            // Each picker on a row of its own beside its label, sized to the row's rest.
            ui.add_enabled_ui(!running && !starting, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Model").small().weak());
                    let width = picker_width(ui);
                    if let Some(model) =
                        launch::model_picker(ui, "chat-model", catalog, &pick.model, width)
                    {
                        pick.model = model;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Effort").small().weak());
                    let width = picker_width(ui);
                    let efforts = launch::efforts_of(
                        catalog,
                        &launch::cli_efforts(preflight.as_deref()),
                        &pick.model,
                    );
                    if let Some(effort) = launch::effort_picker(
                        ui,
                        "chat-effort",
                        &efforts,
                        pick.effort.as_deref(),
                        width,
                    ) {
                        pick.effort = effort;
                    }
                });
            })
            .response
            .on_hover_text(
                "The conversation's model and effort: they go with your next message, and the \
                 ledger's row says the latest turn's",
            );
        }
        ui.label(RichText::new(self.chat_reads_line()).weak())
            .on_hover_text(self.chat_reads_hover());
        // Wrapped: a narrow pane puts Stop on the next line rather than past its edge.
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    !running && !starting && !busy,
                    egui::Button::new("New conversation"),
                )
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
        // The ledger's conversations fold under the header while one is open (EP-24a); with
        // none open they stand in the exchange's place below.
        if open {
            let count = self.chat.list.rows.len();
            let current = self.chat.session_id.clone();
            let unfold = std::mem::take(&mut self.chat.unfold);
            egui::CollapsingHeader::new(
                RichText::new(format!("Conversations · {count}"))
                    .small()
                    .weak(),
            )
            .id_salt("chat-list")
            .default_open(false)
            .open(unfold.then_some(true))
            .show(ui, |ui| {
                if let Some(id) = conversation_rows(
                    ui,
                    &self.chat.list,
                    current.as_deref(),
                    !running && !starting && !busy,
                ) {
                    actions.push(Action::ChatPickUp(id));
                }
            });
        }
        ui.separator();

        // The Adopt confirmation is drawn at the exchange's end, inside its scroll area, so
        // that its wrapped lines never push the footer off the pane (the review's finding).
        let adopt_state = self.chat.adopting.as_ref().and_then(|adopting| {
            self.portfolio
                .unadopted
                .iter()
                .find(|one| one.name.eq_ignore_ascii_case(&adopting.folder))
                .map(|one| one.first_state().to_string())
        });
        let height = (ui.available_height() - FOOTER).max(80.0);
        let known = self.chat_known();
        let mut opened = Vec::new();
        let mut dismiss = false;
        {
            let chat = &mut self.chat;
            let sessions = &mut self.sessions;
            egui::ScrollArea::vertical()
                .id_salt("chat-exchange")
                .auto_shrink(false)
                .stick_to_bottom(true)
                .max_height(height)
                .show(ui, |ui| {
                    let live = chat.key.and_then(|key| sessions.conversation_mut(key));
                    match live {
                        Some(live) => {
                            if chat.picked_up {
                                picked_up_note(ui);
                            }
                            exchange(
                                ui,
                                &live.transcript,
                                &mut chat.cache,
                                live.key,
                                &known,
                                &mut actions,
                            );
                            if live.is_running()
                                && live.transcript.streaming.is_empty()
                                && live.waiting.is_empty()
                            {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(RichText::new("Replying…").weak());
                                });
                            }
                            questions(ui, live, busy, &mut actions);
                        }
                        None if chat.picked_up => picked_up_note(ui),
                        None => {
                            ui.add_space(12.0);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(
                                        "Ask about the portfolio: what a brief plans, what waits \
                                         on you, how a project stands. The session reads the \
                                         projects' files and writes nothing; each message is a \
                                         turn of one Claude Code session, and the ledger keeps \
                                         the conversation.",
                                    )
                                    .weak(),
                                )
                                .wrap(),
                            );
                            ui.add_space(10.0);
                            ui.label(RichText::new("Conversations").weak());
                            ui.add_space(2.0);
                            if let Some(id) =
                                conversation_rows(ui, &chat.list, None, !starting && !busy)
                            {
                                actions.push(Action::ChatPickUp(id));
                            }
                        }
                    }
                    // The Adopt confirmation (EP-24b): the data box, Adopt and Not now.
                    if let Some(adopting) = &mut chat.adopting {
                        ui.add_space(4.0);
                        dismiss =
                            adopt_confirmation(ui, adopting, adopt_state.as_deref(), &mut actions);
                    }
                    opened = take_opened(ui);
                });
        }
        if dismiss {
            self.chat.adopting = None;
        }
        // A link's click opens nothing: the window opens no URL, and a reply is a session's
        // text; the status bar says the target instead.
        for url in opened {
            self.notify(format!("The chat opens no link: {url}"));
        }
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
            (Some(id), None) if self.chat.key.is_none() => {
                format!("session {id} · resumed on Send")
            }
            (Some(id), None) => format!("session {id}"),
            (None, _) => "a new session on Send".to_owned(),
        };
        let line = ui.label(RichText::new(words).small().weak());
        if let Some(id) = &self.chat.session_id {
            line.on_hover_text(format!("session {id}"));
        }
        self.actions.extend(actions);
    }

    /// The header's line: what the session reads.
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
        format!("{reads}{left_out} · read-only")
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
        if !self.chat.mentioned.is_empty() {
            lines.push(format!(
                "Mentioned in the last message, their context given with the digest: {}.",
                self.chat
                    .mentioned
                    .iter()
                    .map(|name| format!("@{name}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if let Some(file) = &self.chat.digest_file {
            lines.push(format!("The digest it was given: {}.", file.display()));
        }
        if lines.is_empty() {
            lines.push(
                "Every registered project not marked data = true, read-only, with the \
                 house-style reference; the digest is written at each Send, and an @<project> \
                 in your message adds that project's context to it."
                    .to_owned(),
            );
        }
        lines.join("\n")
    }
}

/// The Adopt chip's confirmation (EP-24b): what the adoption writes, the data box, Adopt and
/// Not now; `true` when Not now was clicked.
fn adopt_confirmation(
    ui: &mut egui::Ui,
    adopting: &mut Adopting,
    state: Option<&str>,
    actions: &mut Vec<Action>,
) -> bool {
    let mut dismiss = false;
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(format!("Adopt {}", adopting.folder)).strong());
            ui.add(
                egui::Label::new(
                    RichText::new(format!(
                        "Registers it as {}, writes {}.code-workspace beside it unless one is \
                         there, and adds it to the workspace file (D-7); nothing else is \
                         written.",
                        state.unwrap_or("a project"),
                        adopting.folder
                    ))
                    .small()
                    .weak(),
                )
                .wrap(),
            );
            ui.checkbox(
                &mut adopting.data,
                "A data repository: the chat never reads it",
            )
            .on_hover_text(
                "Writes data = true with the adoption (crw set <project> data yes), which keeps \
                 the folder out of every chat session; crw set <project> data no takes it off",
            );
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button("Adopt")
                    .on_hover_text("Adopt it now, as the list's Adopt does")
                    .clicked()
                {
                    actions.push(Action::ChatAdopt {
                        folder: adopting.folder.clone(),
                        data: adopting.data,
                    });
                }
                if ui
                    .button("Not now")
                    .on_hover_text("Nothing is written; the chip stays under the reply")
                    .clicked()
                {
                    dismiss = true;
                }
            });
        });
    dismiss
}

/// The chips under a reply (EP-24b): one button per action proposed, enabled when the window
/// has its target, its hover saying what the click opens; an unknown line is drawn disabled.
fn chips(ui: &mut egui::Ui, proposed: &[Proposed], known: &Known, actions: &mut Vec<Action>) {
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        for action in proposed {
            let (enabled, hover) = match action {
                Proposed::Adopt { folder } if known.unadopted(folder) => (
                    true,
                    "Opens the confirmation below, which asks first whether it is a data \
                     repository; nothing is written before its Adopt"
                        .to_owned(),
                ),
                Proposed::Adopt { folder } if known.project(folder) => {
                    (false, format!("{folder} is registered already"))
                }
                Proposed::Adopt { folder } => (
                    false,
                    format!("{folder} is not an unadopted folder of the data root"),
                ),
                Proposed::Formalise { project } if known.project(project) => (
                    true,
                    "Opens the formalise preview on it: the files it would write, and its Write"
                        .to_owned(),
                ),
                Proposed::Migrate { project } if known.project(project) => (
                    true,
                    "Opens the launch dialog with the migration prompt for it; nothing starts \
                     before its Start"
                        .to_owned(),
                ),
                Proposed::Compose { project, .. } if known.project(project) => (
                    true,
                    "Opens the compose dialog on its inbox with the words typed in; nothing is \
                     written before its Send"
                        .to_owned(),
                ),
                Proposed::Unknown { .. } => (
                    false,
                    "Not an action the window knows: the reply's own line".to_owned(),
                ),
                other => (
                    false,
                    format!(
                        "{} is not a registered project",
                        other.target().unwrap_or_default()
                    ),
                ),
            };
            let button = ui
                .add_enabled(enabled, egui::Button::new(action.label()))
                .on_hover_text(hover.clone())
                .on_disabled_hover_text(hover);
            if button.clicked() {
                actions.push(Action::ChatAct(action.clone()));
            }
        }
    });
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

/// The note over a picked-up conversation's exchange (EP-24a).
fn picked_up_note(ui: &mut egui::Ui) {
    ui.add(
        egui::Label::new(
            RichText::new(
                "Picked up from the ledger: its earlier turns are in the session's own record, \
                 of which the tool keeps no copy; the turns since are below, and your next \
                 message goes on from them.",
            )
            .small()
            .color(style::amber(ui)),
        )
        .wrap(),
    );
    ui.add_space(4.0);
}

/// The ledger's conversations as the list shows them (EP-24a), the latest activity first: the
/// title, when it was last active, its turns, its model and effort, its cost and its session's
/// short id; the open one marked. The session id of the row clicked, when `enabled`.
fn conversation_rows(
    ui: &mut egui::Ui,
    list: &Conversations,
    open: Option<&str>,
    enabled: bool,
) -> Option<String> {
    let mut picked = None;
    if let Some(problem) = &list.problem {
        ui.label(RichText::new(problem).small().color(style::red(ui)));
    }
    if list.rows.is_empty() {
        ui.label(
            RichText::new(
                "No conversation yet: the first Send starts one, and the ledger keeps it.",
            )
            .small()
            .weak(),
        );
        return None;
    }
    for row in &list.rows {
        let is_open = open == Some(row.session_id.as_str());
        let response = egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                // The row takes the click over its text, not a selection (EP-15a).
                style::unselectable(ui);
                // A title up to the ledger's cap stays on one line, cut; the chip on a row of
                // its own (EP-13).
                ui.add(egui::Label::new(RichText::new(&row.title).strong()).truncate());
                if is_open {
                    ui.horizontal(|ui| style::chip(ui, "open", style::blue(ui)));
                }
                ui.add(egui::Label::new(RichText::new(row_words(row)).small().weak()).wrap());
            })
            .response
            .interact(egui::Sense::click())
            .on_hover_text(if is_open {
                "This conversation"
            } else {
                "Pick it up: your next message resumes its session"
            });
        if enabled && !is_open && response.clicked() {
            picked = Some(row.session_id.clone());
        }
        ui.add_space(2.0);
    }
    picked
}

/// A list row's second line: `last today 19:48 · 3 turns · claude-opus-5-5 at medium · $0.41 ·
/// 3c9d2e71`.
fn row_words(row: &ConversationRow) -> String {
    let when = row
        .last()
        .map_or_else(|| row.last_at.clone(), activity_words);
    let pick = Pick {
        model: row.model.clone(),
        effort: row.effort.clone(),
    };
    let cost = row
        .cost_usd
        .map_or(String::new(), |cost| format!(" · ${cost:.2}"));
    format!(
        "last {when} · {} · {}{cost} · {}",
        count(row.turns as usize, "turn"),
        pick.words(),
        crw_core::revert::short(&row.session_id)
    )
}

/// When a conversation was last active, in words: `today 19:48`, else `Tue 11 Nov 19:48`.
fn activity_words(at: DateTime<Utc>) -> String {
    let local = at.with_timezone(&Local);
    if local.date_naive() == Local::now().date_naive() {
        local.format("today %H:%M").to_string()
    } else {
        local.format("%a %d %b %H:%M").to_string()
    }
}

/// The conversation's questions, answered here with the Decisions form (EP-24a): each request
/// the running session waits on, and the question an ended run left open (the defer route),
/// whose answer resumes it. Permission requests never wait: the window refuses them (EP-24).
fn questions(ui: &mut egui::Ui, live: &mut Live, busy: bool, actions: &mut Vec<Action>) {
    let (title, key, limit) = (live.title(), live.key, live.launch_timeout());
    let waiting: Vec<(Request, Instant)> = live
        .waiting
        .iter()
        .map(|waiting| (waiting.request.clone(), waiting.since))
        .collect();
    for (request, since) in &waiting {
        ui.add_space(4.0);
        let draft = live
            .drafts
            .entry(request.id.clone())
            .or_insert_with(|| Draft::new(request));
        let wait = Wait::Live {
            since: *since,
            limit,
        };
        if let Some(response) = decisions::request_card(ui, &title, request, draft, &wait, true) {
            actions.push(Action::Respond {
                key,
                request: request.id.clone(),
                response,
            });
        }
    }
    if live.ended.is_some()
        && let Some(open) = live.open.clone()
    {
        ui.add_space(4.0);
        let draft = live
            .drafts
            .entry(open.request.id.clone())
            .or_insert_with(|| Draft::new(&open.request));
        let wait = Wait::Deferred {
            at: open.at,
            hook: open.deferred.is_some(),
        };
        // Greyed while any start is on its way, as the Decisions tile greys it.
        if let Some(response) =
            decisions::request_card(ui, &title, &open.request, draft, &wait, !busy)
        {
            actions.push(Action::AnswerDeferred { key, response });
        }
    }
}

/// The exchange, from the run's transcript: the owner's messages, the replies as Markdown with
/// the chips of the actions they propose under them (EP-24b), each tool call in one small line,
/// the refusals, the questions and the notices; the result's errors and its cost, not its
/// text, which the reply holds already.
fn exchange(
    ui: &mut egui::Ui,
    transcript: &Transcript,
    cache: &mut CommonMarkCache,
    salt: u64,
    known: &Known,
    actions: &mut Vec<Action>,
) {
    let weak = ui.visuals().weak_text_color();
    if transcript.dropped > 0 {
        ui.label(
            RichText::new(format!(
                "{} earlier entries were let go to keep the exchange short.",
                transcript.dropped
            ))
            .small()
            .weak(),
        );
    }
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
                // The actions block is read out of the reply and drawn as chips, not as code.
                let (text, proposed) = chat::proposals(text);
                markdown(
                    ui,
                    cache,
                    &text,
                    egui::Id::unique(("chat-reply", salt, index)),
                );
                if !proposed.is_empty() {
                    ui.push_id(("chat-chips", salt, index), |ui| {
                        chips(ui, &proposed, known, actions);
                    });
                }
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
                    None => "waiting: the form below answers it".to_owned(),
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
        markdown(
            ui,
            cache,
            &transcript.streaming,
            egui::Id::unique(("chat-streaming", salt)),
        );
        ui.label(RichText::new("…").weak());
    }
}

/// A reply's Markdown drawn through `egui_commonmark` (EP-24a): headings, lists, fenced code,
/// tables, quotes and links in egui's visuals, a link's text in the violet the notes give a
/// reference and a link (EP-15a), its target shown on hover. `id` keeps each block's state
/// apart, in the cache and in egui's own ids: the crate numbers a reply's tables from zero,
/// so two replies with a table would share a grid under one scope (the review's finding).
fn markdown(ui: &mut egui::Ui, cache: &mut CommonMarkCache, text: &str, id: egui::Id) {
    ui.push_id(id, |ui| {
        let violet = style::reference(ui);
        ui.visuals_mut().hyperlink_color = violet;
        ui.style_mut().url_in_tooltip = true;
        CommonMarkViewer::new().show_with_id(id, ui, cache, text);
    });
}

/// The link targets clicked this pass, taken out of egui's output so that nothing opens: the
/// window opens no URL, and the status bar says the target instead.
fn take_opened(ui: &egui::Ui) -> Vec<String> {
    ui.ctx().output_mut(|output| {
        let mut urls = Vec::new();
        output.commands.retain(|command| match command {
            OutputCommand::OpenUrl(open) => {
                urls.push(open.url.clone());
                false
            }
            _ => true,
        });
        urls
    })
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crw_core::registry::{Defaults, Store};
    use crw_core::session::{Choice, Event, Launch, Question, Response, ToolResult, ToolUse};
    use eframe::egui;
    use egui_tiles::Behavior;

    use super::*;
    use crate::board::{Places, View};
    use crate::layout::Pane;
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
        // An unadopted folder of the data root, for the Adopt chip (EP-24b).
        fs::create_dir_all(root.join("gamma")).unwrap();
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
        // The built-in catalog, never the profile's cache; the list canned, never the ledger.
        board.catalog_folder = None;
        board.chat.list.canned = true;
        // The files read, as a frame would take them in.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while board.portfolio.projects.is_empty() && std::time::Instant::now() < deadline {
            board.loader.receive(&mut board.portfolio);
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        board
    }

    /// A press and a release at `at`, after a pass that moves the pointer there; the release
    /// pass's output, where a click's commands would be.
    fn click(screen: &mut Screen, board: &mut Board, at: egui::Pos2) -> egui::FullOutput {
        screen.pass(testing::moved(at), |ui| board.chat_pane(ui));
        screen.pass(
            testing::button(at, egui::PointerButton::Primary, true),
            |ui| board.chat_pane(ui),
        );
        screen.pass(
            testing::button(at, egui::PointerButton::Primary, false),
            |ui| board.chat_pane(ui),
        )
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
        assert_eq!(digest.unadopted, ["gamma"]);
        let text = chat::digest_text(&digest);
        assert!(text.contains("- alpha (active)"));
        assert!(text.contains("Not readable by this session: beta"));
        let (folders, _) = board.chat_folders();
        // The plugin copy's folder may join (its place depends on the environment).
        assert!(folders[0].ends_with("alpha"));
        assert!(folders.iter().all(|folder| !folder.ends_with("beta")));
        assert_eq!(
            board.chat_reads_line(),
            "reads 1 project on Send · read-only"
        );
    }

    /// The tile draws a canned conversation (EP-24): the exchange, Send disabled while the reply
    /// streams and enabled once it has ended with a draft typed, New conversation the other way;
    /// the pickers say the defaults (EP-24a).
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
        assert!(testing::text_at(&output, "It plans").is_some());
        assert!(testing::text_at(&output, "…").is_some());
        // The tool line's galley starts with its icon, so it is not found by its words alone.
        assert!(testing::text_at(&output, "Stop").is_some());
        assert!(testing::text_at(&output, "New conversation").is_some());
        // The pickers, from the built-in catalog: the defaults, greyed while the reply streams.
        assert!(testing::text_at(&output, "Opus 5.5").is_some());
        assert!(testing::text_at(&output, "medium").is_some());
        assert!(testing::text_at(&output, "Conversations · 0").is_some());
        // Send is drawn disabled while the reply streams: a click reaches nothing.
        let send = testing::text_at(&output, "Send").expect("the Send button");
        click(&mut screen, &mut board, send);
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
        click(&mut screen, &mut board, send);
        assert_eq!(board.actions, [Action::ChatSend]);
        board.actions.clear();
        board.chat_send();
        assert_eq!(
            board.chat.draft, "And after that?",
            "a screenshot run starts nothing"
        );
        // New conversation lets the ended one go, the pickers back at the defaults.
        board.chat.pick.model = "fable".to_owned();
        board.chat_new();
        assert!(board.chat.key.is_none());
        assert!(board.chat.title.is_none());
        assert_eq!(board.chat.pick, Pick::default());
        assert!(
            board.sessions.live.is_empty(),
            "the ended live was dismissed"
        );
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        assert!(testing::text_at(&output, "Chat").is_some());
        assert!(testing::text_at(&output, "Conversations").is_some());
    }

    /// A reply draws as Markdown (EP-24a): its heading, list items, code, table cells and link
    /// are painted as their words, and a link's click opens nothing, the status bar saying the
    /// target.
    #[test]
    fn a_reply_draws_as_markdown_and_a_link_opens_nothing() {
        let scratch = Scratch::new("chat-markdown");
        let mut board = board(&scratch);
        board.fixtures = true;
        let reply = "## What EP-2 plans\n\nLists, an **M**: two parts.\n\n\
                     - the first part\n- the second part\n\n\
                     ```toml\nkeep = true\n```\n\n\
                     | Column | Value |\n|---|---|\n| rows | seven |\n\n\
                     > A quote from the brief.\n\n\
                     The brief is [EP-2-lists.md](https://example.org/EP-2-lists.md).";
        let mut live = Live::canned(
            951,
            canned_launch(),
            chat::EFFORT,
            vec![Event::Text(reply.to_owned())],
        );
        live.project = chat::PROJECT.to_owned();
        live.end_canned(crw_core::session::Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        });
        board.sessions.live.insert(0, live);
        board.chat.key = Some(951);
        board.chat.title = Some("What does EP-2 plan?".to_owned());
        board.chat.session_id = Some("3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f".to_owned());
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        for words in [
            "What EP-2 plans",
            "the first part",
            "the second part",
            "Column",
            "seven",
            "A quote from the brief.",
            "EP-2-lists.md",
        ] {
            assert!(testing::text_at(&output, words).is_some(), "{words}");
        }
        assert!(testing::text_containing(&output, "keep = true").is_some());
        // The source's marks are not painted as written.
        assert!(testing::text_containing(&output, "**M**").is_none());
        assert!(testing::text_containing(&output, "## ").is_none());
        // The link's click: no URL leaves the release's frame, the status bar names the target.
        let link = testing::text_at(&output, "EP-2-lists.md").unwrap();
        let released = click(&mut screen, &mut board, link);
        assert!(
            !released
                .platform_output
                .commands
                .iter()
                .any(|command| matches!(command, OutputCommand::OpenUrl(_)))
        );
        assert_eq!(
            board.notice_text(),
            Some("The chat opens no link: https://example.org/EP-2-lists.md")
        );
    }

    fn row(
        session_id: &str,
        title: &str,
        model: &str,
        effort: Option<&str>,
        hour: u32,
    ) -> ConversationRow {
        ConversationRow {
            id: 1,
            session_id: session_id.to_owned(),
            title: title.to_owned(),
            model: model.to_owned(),
            effort: effort.map(str::to_owned),
            started_at: format!("2026-11-10T{hour:02}:00:00Z"),
            last_at: format!("2026-11-10T{hour:02}:30:00Z"),
            turns: 3,
            cost_usd: Some(0.41),
        }
    }

    /// The list (EP-24a): the ledger's conversations drawn with no conversation open, a row's
    /// click picking one up with its title, id, model and effort, the next Send its resume; the
    /// fold while one is open; New conversation letting it go.
    #[test]
    fn the_list_picks_a_conversation_up() {
        let scratch = Scratch::new("chat-list");
        let mut board = board(&scratch);
        board.fixtures = true;
        board.chat.list.rows = vec![
            row(
                "c-1-aaaa",
                "What waits on me this week?",
                "claude-fable-5-1",
                Some("high"),
                9,
            ),
            row(
                "c-2-bbbb",
                "Which project is furthest behind?",
                "claude-opus-5-5",
                None,
                8,
            ),
        ];
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        let first = testing::text_at(&output, "What waits on me this week?").expect("the row");
        assert!(testing::text_at(&output, "Which project is furthest behind?").is_some());
        let words = row_words(&board.chat.list.rows[0]);
        assert!(words.contains("3 turns · claude-fable-5-1 at high · $0.41 · c-1-aaaa"));
        assert!(
            row_words(&board.chat.list.rows[1]).contains("at the CLI's default effort"),
            "{}",
            row_words(&board.chat.list.rows[1])
        );
        assert!(testing::text_containing(&output, "3 turns").is_some());
        click(&mut screen, &mut board, first);
        assert_eq!(board.actions, [Action::ChatPickUp("c-1-aaaa".to_owned())]);
        board.actions.clear();
        board.chat_pick_up("c-1-aaaa");
        assert_eq!(board.chat.session_id.as_deref(), Some("c-1-aaaa"));
        assert_eq!(
            board.chat.title.as_deref(),
            Some("What waits on me this week?")
        );
        assert_eq!(
            board.chat.pick,
            Pick {
                model: "claude-fable-5-1".to_owned(),
                effort: Some("high".to_owned())
            }
        );
        assert!(board.chat.picked_up);
        assert!(board.chat.key.is_none());
        assert_eq!(
            board.notice_text(),
            Some(
                "Picked up chat: What waits on me this week?: your next message resumes its \
                 session."
            )
        );
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        assert!(testing::text_at(&output, "chat: What waits on me this week?").is_some());
        assert!(testing::text_at(&output, "Conversations · 2").is_some());
        assert!(testing::text_containing(&output, "Picked up from the ledger").is_some());
        assert!(testing::text_at(&output, "session c-1-aaaa · resumed on Send").is_some());
        assert!(testing::text_at(&output, "Fable 5.1").is_some());
        assert!(testing::text_at(&output, "high").is_some());
        // The same row again changes nothing; a row that went is said.
        board.chat_pick_up("c-1-aaaa");
        board.chat_pick_up("c-9-gone");
        assert_eq!(
            board.notice_text(),
            Some("That conversation is no longer in the list.")
        );
        // Send would resume it (a screenshot run warns instead), the draft kept.
        board.chat.draft = "And next week?".to_owned();
        board.chat_send();
        assert_eq!(board.chat.draft, "And next week?");
        board.chat_new();
        assert!(board.chat.session_id.is_none());
        assert!(!board.chat.picked_up);
        assert_eq!(board.chat.pick, Pick::default());
    }

    fn asked() -> Request {
        Request {
            id: "req-1".to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which project first?".to_owned(),
            reason: None,
            questions: vec![Question {
                text: "Which project first?".to_owned(),
                header: "Project".to_owned(),
                options: vec![
                    Choice {
                        label: "alpha".to_owned(),
                        description: Some("The one with the open task".to_owned()),
                    },
                    Choice {
                        label: "beta".to_owned(),
                        description: None,
                    },
                ],
                multi_select: false,
            }],
            input: String::new(),
        }
    }

    /// A conversation's question is answered in the tile (EP-24a): the form drawn under the
    /// exchange, a pick and Send answers reaching the session's respond action; the Decisions
    /// tile leaves it out, and the Chat tab counts it.
    #[test]
    fn a_conversations_question_is_answered_in_the_tile() {
        let scratch = Scratch::new("chat-question");
        let mut board = board(&scratch);
        board.fixtures = true;
        let mut live = Live::canned(
            952,
            canned_launch(),
            chat::EFFORT,
            vec![
                Event::Text("Two projects wait; one first.".to_owned()),
                Event::Request(asked()),
            ],
        );
        live.project = chat::PROJECT.to_owned();
        board.sessions.live.insert(0, live);
        board.chat.key = Some(952);
        board.chat.title = Some("What waits?".to_owned());
        board.chat.session_id = Some("3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f".to_owned());
        assert_eq!(board.chat_asks(), 1);
        assert_eq!(
            board.decisions_count(),
            0,
            "the Decisions tile leaves it out"
        );
        assert_eq!(board.sessions.waiting(), 0);
        assert_eq!(board.tab_title_for_pane(&Pane::Chat).text(), "Chat · 1");
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        assert!(testing::text_at(&output, "Which project first?").is_some());
        assert!(testing::text_at(&output, "Send answers").is_some());
        assert!(testing::text_at(&output, "Decline").is_some());
        assert!(testing::text_containing(&output, "waiting: the form below answers it").is_some());
        // A pick, then Send answers.
        let alpha = testing::text_at(&output, "alpha").expect("the option");
        click(&mut screen, &mut board, alpha);
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        let send = testing::text_at(&output, "Send answers").unwrap();
        click(&mut screen, &mut board, send);
        assert_eq!(
            board.actions,
            [Action::Respond {
                key: 952,
                request: "req-1".to_owned(),
                response: Response::Answers(vec!["alpha".to_owned()]),
            }]
        );
        // The Decisions pane draws its placeholder: nothing of the chat's.
        let output = screen.pass(Vec::new(), |ui| board.decisions_pane(ui));
        assert!(testing::text_at(&output, "Which project first?").is_none());
    }

    /// A conversation deferred with a question open (the defer route) shows the question with
    /// Answer and resume and counts it; letting it go is refused while a start is on its way
    /// and, once it goes, the notice says the question went with it (the review's findings).
    #[test]
    fn a_deferred_question_waits_in_the_tile_and_goes_with_the_conversation() {
        let scratch = Scratch::new("chat-deferred");
        let mut board = board(&scratch);
        board.fixtures = true;
        let mut live = Live::canned(
            953,
            canned_launch(),
            chat::EFFORT,
            vec![Event::Request(asked()), Event::TimedOut("req-1".to_owned())],
        );
        live.project = chat::PROJECT.to_owned();
        live.end_canned(crw_core::session::Ended {
            outcome: Outcome::Deferred,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        });
        assert!(live.open.is_some());
        board.sessions.live.insert(0, live);
        board.chat.key = Some(953);
        board.chat.title = Some("What waits?".to_owned());
        board.chat.session_id = Some("3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f".to_owned());
        assert_eq!(board.chat_asks(), 1);
        assert_eq!(
            board.sessions.deferred(),
            0,
            "the Decisions tile leaves it out"
        );
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        assert!(testing::text_at(&output, "Answer and resume").is_some());
        assert!(testing::text_at(&output, "Decline and resume").is_some());
        assert!(testing::text_at(&output, "ended: deferred").is_some());
        // While a turn starts, the conversation cannot be let go.
        board.chat.starting = true;
        board.chat_new();
        assert_eq!(board.chat.key, Some(953));
        assert!(
            board
                .notice_text()
                .is_some_and(|words| words.contains("a session is starting"))
        );
        board.chat.starting = false;
        board.chat_new();
        assert!(board.chat.key.is_none());
        assert!(board.sessions.live.is_empty());
        assert_eq!(
            board.notice_text(),
            Some(
                "The conversation was let go. Its open question went with it: picking the \
                 conversation up again resumes the session, which asks again."
            )
        );
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

    /// An ended conversation whose reply is `text`, in the tile.
    fn ended_with(board: &mut Board, key: u64, text: &str) {
        let mut live = Live::canned(
            key,
            canned_launch(),
            chat::EFFORT,
            vec![Event::Text(text.to_owned())],
        );
        live.project = chat::PROJECT.to_owned();
        live.end_canned(crw_core::session::Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        });
        board.sessions.live.insert(0, live);
        board.chat.key = Some(key);
        board.chat.title = Some("What is next in alpha?".to_owned());
        board.chat.session_id = Some("3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f".to_owned());
    }

    /// A reply's actions draw as chips under it, not as a code block (EP-24b): a chip on a
    /// registered project or an unadopted folder is enabled and its click queues the action,
    /// which opens the window's own dialog; one on an unknown name, or an unknown line, is
    /// drawn disabled and queues nothing.
    #[test]
    fn a_replys_actions_draw_as_chips_that_open_the_windows_dialogs() {
        let scratch = Scratch::new("chat-chips");
        let mut board = board(&scratch);
        board.fixtures = true;
        ended_with(
            &mut board,
            954,
            "alpha's EP-2 is next; gamma is not registered.\n\n\
             ```actions\nformalise alpha\nmigrate alpha\ncompose alpha: the season names\n\
             adopt gamma\nformalise delta\nadopt beta\nstart alpha\n```",
        );
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        for label in [
            "Formalise alpha…",
            "Migrate alpha…",
            "Compose to alpha…",
            "Adopt gamma…",
            "Formalise delta…",
            "Adopt beta…",
            "start alpha",
        ] {
            assert!(testing::text_at(&output, label).is_some(), "{label}");
        }
        assert!(testing::text_containing(&output, "```").is_none());
        assert!(testing::text_containing(&output, "formalise alpha").is_none());
        // A disabled chip's click reaches nothing.
        let delta = testing::text_at(&output, "Formalise delta…").unwrap();
        click(&mut screen, &mut board, delta);
        let unknown = testing::text_at(&output, "start alpha").unwrap();
        click(&mut screen, &mut board, unknown);
        assert!(board.actions.is_empty(), "{:?}", board.actions);
        // Compose: the dialog opens on alpha with the words typed in.
        let compose = testing::text_at(&output, "Compose to alpha…").unwrap();
        click(&mut screen, &mut board, compose);
        let proposed = Proposed::Compose {
            project: "alpha".to_owned(),
            words: "the season names".to_owned(),
        };
        assert_eq!(board.actions, [Action::ChatAct(proposed.clone())]);
        board.actions.clear();
        board.chat_act(proposed);
        let dialog = board.compose.as_ref().expect("the compose dialog");
        assert_eq!(dialog.project, "alpha");
        assert_eq!(dialog.text, "the season names");
        board.compose = None;
        // Formalise: the preview opens on alpha (a screenshot run reads the plan at once).
        board.chat_act(Proposed::Formalise {
            project: "alpha".to_owned(),
        });
        assert!(board.formalise.is_some(), "{:?}", board.notice_text());
        board.formalise = None;
        // An unknown line, taken by hand, is said.
        board.chat_act(Proposed::Unknown {
            line: "start alpha".to_owned(),
        });
        assert_eq!(
            board.notice_text(),
            Some("Not an action the window knows: start alpha")
        );
    }

    /// The Adopt chip asks for the data mark before anything is written (EP-24b): the
    /// confirmation under the exchange with its box, Not now putting it away, Adopt queuing the
    /// adoption, which registers the folder and writes `data = true` when the box was ticked.
    #[test]
    fn the_adopt_chip_asks_for_the_data_mark_before_writing() {
        let scratch = Scratch::new("chat-adopt");
        let mut board = board(&scratch);
        board.fixtures = true;
        ended_with(
            &mut board,
            955,
            "gamma is not registered.\n\n```actions\nadopt gamma\n```",
        );
        board.chat_act(Proposed::Adopt {
            folder: "nowhere".to_owned(),
        });
        assert!(board.chat.adopting.is_none());
        assert!(
            board
                .notice_text()
                .is_some_and(|words| words.contains("not an unadopted folder"))
        );
        board.chat_act(Proposed::Adopt {
            folder: "Gamma".to_owned(),
        });
        assert_eq!(
            board.chat.adopting,
            Some(Adopting {
                folder: "gamma".to_owned(),
                data: false
            })
        );
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        assert!(testing::text_at(&output, "Adopt gamma").is_some());
        assert!(testing::text_containing(&output, "writes gamma.code-workspace").is_some());
        let not_now = testing::text_at(&output, "Not now").unwrap();
        click(&mut screen, &mut board, not_now);
        assert!(board.chat.adopting.is_none());
        assert!(board.actions.is_empty());
        // Again, the box ticked, then Adopt.
        board.chat_act(Proposed::Adopt {
            folder: "gamma".to_owned(),
        });
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        let data = testing::text_at(&output, "A data repository: the chat never reads it").unwrap();
        click(&mut screen, &mut board, data);
        assert!(board.chat.adopting.as_ref().is_some_and(|a| a.data));
        let output = screen.pass(Vec::new(), |ui| board.chat_pane(ui));
        let adopt = testing::text_at(&output, "Adopt").unwrap();
        click(&mut screen, &mut board, adopt);
        assert_eq!(
            board.actions,
            [Action::ChatAdopt {
                folder: "gamma".to_owned(),
                data: true
            }]
        );
        board.actions.clear();
        board.chat_adopt("gamma", true);
        assert!(board.chat.adopting.is_none());
        // The status bar cuts a long notice with an ellipsis: the start and the mark's words.
        assert!(
            board
                .notice_text()
                .is_some_and(|words| words.starts_with("Adopted gamma as unformalised")
                    && words.contains("Marked a data repository (data = true)")),
            "{:?}",
            board.notice_text()
        );
        let registry = fs::read_to_string(scratch.0.join("registry.toml")).unwrap();
        let gamma = registry.find("name = \"gamma\"").expect("gamma registered");
        assert!(registry[gamma..].contains("data = true"));
    }

    /// An `@<project>` in the message gives the digest that project's context (EP-24b): the
    /// prompt hook's own, for a readable project; nothing for a data repository, nothing
    /// without an `@`.
    #[test]
    fn a_mention_gives_the_digest_a_readable_projects_context() {
        let scratch = Scratch::new("chat-mentions");
        let board = board(&scratch);
        let cwd = Path::new("C:\\chat");
        let (context, mentioned) = board.chat_mentions("@alpha what is next, and @beta too?", cwd);
        let context = context.expect("alpha's context");
        assert!(context.contains("The prompt mentions @alpha"), "{context}");
        assert!(context.contains("alpha's next brief"), "{context}");
        assert!(context.contains("EP-2"), "{context}");
        assert!(!context.contains("@beta"), "{context}");
        assert_eq!(mentioned, ["alpha"]);
        assert_eq!(board.chat_mentions("@beta alone", cwd), (None, Vec::new()));
        assert_eq!(board.chat_mentions("no mention", cwd), (None, Vec::new()));
        let text = chat::appended("The digest.", Some(&context));
        assert!(text.starts_with("The digest.\n\nThe owner's message names a project with @."));
        assert!(text.contains("alpha's next brief"));
    }

    /// A running chat refuses the window's restart as a headless run does (EP-24b).
    #[test]
    fn a_running_chat_refuses_a_restart() {
        let scratch = Scratch::new("chat-restart");
        let mut board = board(&scratch);
        let mut live = Live::canned(
            956,
            canned_launch(),
            chat::EFFORT,
            vec![Event::TextDelta("It plans".to_owned())],
        );
        live.project = chat::PROJECT.to_owned();
        board.sessions.live.insert(0, live);
        board.chat.key = Some(956);
        let refusals = board.restart_refusals();
        assert!(
            refusals
                .iter()
                .any(|why| why.contains("chat: What is next in alpha?")),
            "{refusals:?}"
        );
        board.sessions.live[0].end_canned(crw_core::session::Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        });
        let refusals = board.restart_refusals();
        assert!(
            !refusals.iter().any(|why| why.contains("chat:")),
            "{refusals:?}"
        );
    }
}
