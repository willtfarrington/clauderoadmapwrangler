//! What the window publishes for the phone's pages (D-14, D-28; EP-30): the project list as plain
//! data, read by the server's handlers on their own thread, since nothing the window holds can be
//! read there. The window builds it in each pass, shown or hidden (`Dashboard::logic`), at most
//! once a second and at once when a page asks for a fresh one, and sends it through a `watch`
//! channel when its content changed or a page asked; the version rises only with the content,
//! which is the change EP-30b's event stream names. A build that finds the same content renews
//! the snapshot's time without a word to the receivers. A page asks when the snapshot it holds is
//! older than [`FRESH_FOR`], and waits at most [`WAIT`] for the window's answer: a window too
//! busy to answer leaves it the older snapshot, whose time the page shows.
//!
//! EP-30b adds the list's usage strip and autopilot block, a project's runs with their Stop, and
//! the start page's preview, which the window drafts when the page asks (`Start`). The version
//! now rises only with what the clock alone does not change: the list is compared without its
//! badges' sentences, the last commit's age word and the figures' read times (`List::steady`),
//! which a page still reads as of each build. The event stream (`Reader::changes`) names each
//! part that changed by the path of the page that shows it.
//!
//! EP-31 adds the sessions' pages: each window session's page (its transcript's latest entries
//! with their place in it, the text streaming in to its last whole line, what it waits on), the
//! sessions list (the window's, those started elsewhere, the ledger's latest rows), the latest
//! notifications and the doctor's report the Diagnostics pane last gathered. Their times are kept
//! as instants, drawn when a page is drawn, so an idle window publishes no new version for them.

use std::collections::{BTreeSet, VecDeque};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use crw_core::autopilot::{self as chains, Refusal};
use crw_core::badges::hosting::Class;
use crw_core::badges::stack::Kind;
use crw_core::doctor::Report;
use crw_core::ledger::{self, EventRow, Ledger, RowState, SessionRow};
use crw_core::registry::State;
use crw_core::roadmap::table::Counts;
use crw_core::session::Request;
use crw_core::sessions::{Seen, VISUAL_CLAIM};
use crw_core::tasks::OwnerItem;
use futures_util::Stream;
use tokio::sync::watch;

use crate::badges::{self, Tone};
use crate::board::Board;
use crate::portfolio::{Filter, GroupKind, Member, ProjectRow, SortKey};
use crate::sessions::Live;
use crate::transcript::{Entry, Transcript};

/// The window builds the snapshot at most this often, unless a page asks.
const EVERY: Duration = Duration::from_secs(1);

/// A page asks the window for a fresh snapshot when the one it holds is older than this.
pub const FRESH_FOR: Duration = Duration::from_secs(2);

/// How long a page waits for the window's fresh snapshot.
pub const WAIT: Duration = Duration::from_secs(2);

/// What the event stream names when the window has moved on since the page was drawn (EP-30b):
/// every part, so the page fetches itself again.
pub const EVERYTHING: &str = "*";

/// The most entries a session's page carries (EP-31): the latest, the earlier ones counted. The
/// window keeps 2,000, which the desk's tab shows.
pub const PAGE_ENTRIES: usize = 300;

/// The ledger's rows the sessions page lists, as `crw sessions` prints them (EP-31).
pub const LEDGER_ROWS: usize = 20;

/// The notifications the page lists, as `crw notifications` reads them (EP-31), and how often the
/// window reads them again: they are written by the toasts and by the pushes' thread.
pub const NOTIFICATIONS: usize = 20;
const NOTIFICATIONS_EVERY: Duration = Duration::from_secs(15);

static NO_STACKS: BTreeSet<Kind> = BTreeSet::new();
static NO_CLASSES: BTreeSet<Class> = BTreeSet::new();

/// One snapshot: when the window built it, its version (raised only when the content changes)
/// and the content: the list, each project's page and the decisions (EP-30a); the sessions list,
/// each window session's page, the notifications and the doctor's report (EP-31).
#[derive(Debug, Clone, PartialEq)]
pub struct Published {
    pub at: DateTime<Utc>,
    pub version: u64,
    pub list: List,
    pub projects: Vec<Project>,
    pub decisions: Decisions,
    pub sessions: SessionList,
    pub runs: Vec<SessionPage>,
    pub notifications: Notifications,
    pub doctor: Doctor,
}

impl Published {
    /// What the server holds before the window's first pass: nothing read yet. Its version is
    /// the process's start in milliseconds (EP-30b's review), so that a page drawn by an earlier
    /// window, whose version the event stream's `since` names, cannot match this one's by chance.
    fn empty() -> Self {
        Self {
            at: DateTime::<Utc>::MIN_UTC,
            version: u64::try_from(Utc::now().timestamp_millis()).unwrap_or_default(),
            list: List {
                reading: true,
                ..List::default()
            },
            projects: Vec::new(),
            decisions: Decisions::default(),
            sessions: SessionList::default(),
            runs: Vec::new(),
            notifications: Notifications::default(),
            doctor: Doctor::default(),
        }
    }

    /// The page of the project called `name`, its case aside.
    pub fn project(&self, name: &str) -> Option<&Project> {
        self.projects
            .iter()
            .find(|project| project.name.eq_ignore_ascii_case(name))
    }

    /// The page of the window's session `id` (EP-31), its case aside.
    pub fn session(&self, id: &str) -> Option<&SessionPage> {
        self.runs
            .iter()
            .find(|page| page.id.eq_ignore_ascii_case(id))
    }

    /// A snapshot of `parts` built at `at`, at `version`: the screenshot feature's pages and the
    /// tests'.
    #[cfg(any(test, feature = "screenshot"))]
    pub fn of(parts: Parts, at: DateTime<Utc>, version: u64) -> Self {
        let Parts {
            list,
            projects,
            decisions,
            sessions,
            runs,
            notifications,
            doctor,
        } = parts;
        Self {
            at,
            version,
            list,
            projects,
            decisions,
            sessions,
            runs,
            notifications,
            doctor,
        }
    }

    /// Its content as parts again, for a test to change one and offer them.
    #[cfg(test)]
    pub fn parts(&self) -> Parts {
        Parts {
            list: self.list.clone(),
            projects: self.projects.clone(),
            decisions: self.decisions.clone(),
            sessions: self.sessions.clone(),
            runs: self.runs.clone(),
            notifications: self.notifications.clone(),
            doctor: self.doctor.clone(),
        }
    }

    /// The parts that differ from `before`, each by the path of the page that shows it (EP-30b):
    /// the list's (its two blocks with it), each project's page (one added or gone too), the
    /// decisions'. The list is compared steady, as the version is.
    pub fn changed_paths(&self, before: &Published) -> Vec<String> {
        let mut paths = Vec::new();
        if self.list.steady() != before.list.steady() {
            paths.push(crate::server::PAGES_ROOT.to_owned());
        }
        let mut seen = BTreeSet::new();
        for project in self.projects.iter().chain(&before.projects) {
            if !seen.insert(project.name.to_lowercase()) {
                continue;
            }
            if self.project(&project.name) != before.project(&project.name) {
                paths.push(crate::server::project_path(&project.name));
            }
        }
        if self.decisions != before.decisions {
            paths.push(crate::server::DECISIONS_ROUTE.to_owned());
        }
        // The sessions' pages (EP-31): the list, each session's page (one added or gone too),
        // the notifications, the doctor's report.
        if self.sessions != before.sessions {
            paths.push(crate::server::SESSIONS_ROUTE.to_owned());
        }
        let mut seen = BTreeSet::new();
        for page in self.runs.iter().chain(&before.runs) {
            if !seen.insert(page.id.to_lowercase()) {
                continue;
            }
            if self.session(&page.id) != before.session(&page.id) {
                paths.push(crate::server::session_path(&page.id));
            }
        }
        if self.notifications != before.notifications {
            paths.push(crate::server::NOTIFICATIONS_ROUTE.to_owned());
        }
        if self.doctor != before.doctor {
            paths.push(crate::server::DIAGNOSTICS_ROUTE.to_owned());
        }
        paths
    }
}

/// What one build of the snapshot holds: the content without its time and version.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Parts {
    pub list: List,
    pub projects: Vec<Project>,
    pub decisions: Decisions,
    pub sessions: SessionList,
    pub runs: Vec<SessionPage>,
    pub notifications: Notifications,
    pub doctor: Doctor,
}

impl Parts {
    /// Everything the pages read, as the window's board holds it at `now`.
    pub fn of(board: &Board, now: DateTime<Utc>) -> Self {
        let busy = board.busy_projects();
        let projects = board
            .portfolio
            .projects
            .iter()
            .map(|row| {
                let live = board.session_summary(row.name());
                let busy = busy.get(&row.name().to_lowercase()).cloned();
                let mut page = Project::of(row, live.as_deref(), busy, now);
                page.revert = board.revert_open_in(row.name());
                page.runs = Run::of(board, row.name());
                page
            })
            .collect();
        Self {
            list: List::of(board, now),
            projects,
            decisions: Decisions::of(board),
            sessions: SessionList::of(board, now),
            runs: board
                .sessions
                .live
                .iter()
                .filter(|live| !live.is_chat())
                .map(SessionPage::of)
                .collect(),
            notifications: board.pages.notifications().clone(),
            doctor: Doctor::of(&board.diagnostics),
        }
    }
}

/// A window session's page (EP-31): its head, the latest entries of its transcript with their
/// place in it, the text streaming in to its last whole line, and what it waits on. A chat's run
/// has none: it is the Chat pane's.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionPage {
    /// The run's session id, pre-assigned at its launch (D-16): the page's path, which outlives
    /// the window's numbers, since those start again at 1 at each restart.
    pub id: String,
    /// The window's number for it, which the forms name.
    pub key: u64,
    pub project: String,
    pub ep: String,
    pub title: String,
    /// Its state in words: running, waiting for an answer, stopping, how it ended.
    pub state: String,
    pub running: bool,
    /// It works or waits and is not being stopped: Stop is offered.
    pub stoppable: bool,
    pub model: String,
    pub effort: String,
    pub started: DateTime<Utc>,
    /// One, and one more for each resume.
    pub runs: u32,
    /// The autopilot's part in words: chained or on, its cap, where its cap stopped it.
    pub autopilot: Option<String>,
    /// What the launch asked for and the CLI could not take.
    pub dropped: Vec<String>,
    /// How it ended, in one line.
    pub ended: Option<String>,
    /// The transcript's latest entries, the first of them its `first`-th (from 0).
    pub entries: Vec<Entry>,
    pub first: usize,
    /// How many of the `first` earlier ones the window let go, past its 2,000 (the review's):
    /// the rest are in its tab.
    pub let_go: usize,
    /// The text streaming in, to its last whole line: the version rises with a line, not a word.
    pub streaming: String,
    /// The requests it waits on, and an ended run's deferred one.
    pub waiting: Vec<Asking>,
    pub deferred: Option<Asking>,
}

impl SessionPage {
    /// The window's session `live` as its page shows it.
    pub fn of(live: &Live) -> Self {
        let transcript = &live.transcript;
        let skip = transcript.entries.len().saturating_sub(PAGE_ENTRIES);
        let (waiting, deferred) = Asking::of_live(live);
        // The cap's words and where it stopped the run, never the work so far, which the clock
        // alone moves.
        let autopilot = live.cap.as_ref().map(|cap| {
            let how = if live.chained {
                "chained by the autopilot"
            } else {
                "autopilot on"
            };
            match live.capped {
                Some(worked) => format!(
                    "{how} · {} · stopped at its cap after {}",
                    cap.words(),
                    chains::length(worked)
                ),
                None => format!("{how} · {}", cap.words()),
            }
        });
        Self {
            id: live.session_id.clone(),
            key: live.key,
            project: live.project.clone(),
            ep: live.ep.clone(),
            title: live.title(),
            state: live.state(),
            running: live.is_running(),
            stoppable: live.is_running() && !live.stopping,
            model: live.model.clone(),
            effort: live.effort.clone(),
            started: live.started.with_timezone(&Utc),
            runs: live.runs,
            autopilot,
            dropped: live.dropped.clone(),
            ended: live.ended.as_ref().map(crate::sessions::ending),
            entries: transcript.entries[skip..].to_vec(),
            first: transcript.dropped + skip,
            let_go: transcript.dropped,
            streaming: whole_lines(&transcript.streaming).to_owned(),
            waiting,
            deferred,
        }
    }
}

/// `text` to its last whole line: what streams in, said a line at a time (EP-31).
pub fn whole_lines(text: &str) -> &str {
    text.rfind('\n').map_or("", |end| &text[..end])
}

/// The sessions page (EP-31): the window's sessions, those started elsewhere, then the ledger's
/// latest rows as `crw sessions` prints them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SessionList {
    /// A start waits its turn in the window.
    pub starting: bool,
    /// The window's sessions, a chat's aside, the latest first.
    pub window: Vec<WindowSession>,
    /// Those the hooks and the poll report in registered projects.
    pub elsewhere: Vec<SeenSession>,
    /// How many run outside the registered projects.
    pub outside: usize,
    /// Why the list of live sessions did not read.
    pub poll_problem: Option<String>,
    pub rows: Vec<LedgerRow>,
    /// Why the ledger's rows did not read.
    pub problem: Option<String>,
}

/// One of the window's sessions on the sessions page.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowSession {
    /// Its page's id.
    pub id: String,
    pub title: String,
    pub state: String,
    pub model: String,
    pub effort: String,
    pub started: DateTime<Utc>,
    pub runs: u32,
    /// Its latest whole entry in one line: a text's last line, a tool call, a request.
    pub last: String,
    pub chained: bool,
}

/// A session started elsewhere on the sessions page (EP-12's `Seen`).
#[derive(Debug, Clone, PartialEq)]
pub struct SeenSession {
    pub project: String,
    /// Its mode, or `new` for a tab open with no prompt yet.
    pub mode: String,
    pub status: &'static str,
    pub waiting: bool,
    /// What it does, after its subject: `waits for you in VS Code: a permission prompt`.
    pub doing: String,
    pub started: Option<DateTime<Utc>>,
    pub name: Option<String>,
    /// The status line's figures, its surface and its session.
    pub figures: Vec<String>,
}

/// A ledger row on the sessions page, in `crw sessions`' words.
#[derive(Debug, Clone, PartialEq)]
pub struct LedgerRow {
    pub started: Option<DateTime<Utc>>,
    /// As written, for a start that does not read as a time.
    pub started_at: String,
    /// The session id's short form, which `crw revert` takes.
    pub short: String,
    pub title: String,
    pub mode: String,
    pub model: String,
    pub took: String,
    pub state: String,
    pub cost: String,
    pub figures: Vec<String>,
    /// The result's gist in one line.
    pub result: Option<String>,
    /// The window holds its session: its page's id.
    pub page: Option<String>,
}

impl SessionList {
    /// The window's sessions, those seen elsewhere and the ledger's rows as of `now`.
    pub fn of(board: &Board, now: DateTime<Utc>) -> Self {
        let lives: Vec<&Live> = board
            .sessions
            .live
            .iter()
            .filter(|live| !live.is_chat())
            .collect();
        let model = &board.watch.model;
        let history = &board.sessions.history;
        Self {
            starting: board.sessions.is_starting(),
            window: lives
                .iter()
                .map(|live| WindowSession {
                    id: live.session_id.clone(),
                    title: live.title(),
                    state: live.state(),
                    model: live.model.clone(),
                    effort: live.effort.clone(),
                    started: live.started.with_timezone(&Utc),
                    runs: live.runs,
                    last: last_words(&live.transcript),
                    chained: live.chained,
                })
                .collect(),
            elsewhere: board
                .seen_sessions()
                .into_iter()
                .map(SeenSession::of)
                .collect(),
            outside: model.outside(),
            poll_problem: model.poll_problem.clone(),
            rows: history
                .rows
                .iter()
                .take(LEDGER_ROWS)
                .map(|row| {
                    let page = row.session_id.as_deref().and_then(|id| {
                        lives
                            .iter()
                            .find(|live| live.session_id.eq_ignore_ascii_case(id))
                            .map(|live| live.session_id.clone())
                    });
                    LedgerRow::of(row, now, page)
                })
                .collect(),
            problem: history.problem.clone(),
        }
    }
}

/// A transcript's latest whole entry in one line, as the All tab's card says what a run does.
fn last_words(transcript: &Transcript) -> String {
    transcript
        .entries
        .iter()
        .rev()
        .find_map(|entry| match entry {
            Entry::Text(text) => crate::sessions::last_line(text),
            Entry::Tool { call, .. } => Some(crate::sessions::cut_line(&call.summary)),
            Entry::Denied(denied) => Some(crate::sessions::cut_line(&format!(
                "refused: {} ({})",
                denied.tool, denied.message
            ))),
            Entry::Asked { request, .. } => Some(crate::sessions::cut_line(&format!(
                "asked: {}",
                request.summary
            ))),
            _ => None,
        })
        .unwrap_or_default()
}

impl SeenSession {
    /// A session the hooks or the poll report, as the All tab's card reads it.
    fn of(seen: &Seen) -> Self {
        let mode = if seen.is_new() {
            "new".to_owned()
        } else {
            seen.mode.as_str().to_owned()
        };
        let waiting = seen.status == crw_core::sessions::Status::Waiting;
        let doing = if waiting {
            let (place, _) = crate::decisions::place(seen);
            seen.waits_for_you(place)
        } else {
            seen.doing()
        };
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
        Self {
            project: seen.project.clone().unwrap_or_default(),
            mode,
            status: seen.status.as_str(),
            waiting,
            doing,
            started: seen.started,
            name: seen.name.clone(),
            figures,
        }
    }
}

impl LedgerRow {
    /// `row` at `now` in `crw sessions`' words, `page` the window's session it is, if any.
    fn of(row: &SessionRow, now: DateTime<Utc>, page: Option<String>) -> Self {
        let state = match row.state(now, VISUAL_CLAIM) {
            RowState::Ended(outcome) => outcome.to_owned(),
            RowState::Lapsed => "lapsed: no session took it up".to_owned(),
            RowState::Launched => "handed to VS Code, not taken up yet".to_owned(),
            RowState::Open if page.is_some() => "open: the window's own run".to_owned(),
            RowState::Open => "open: no end recorded".to_owned(),
        };
        let title = match &row.ep {
            Some(ep) => format!("{} {ep}", row.project),
            None => row.project.clone(),
        };
        let model = match (&row.model, &row.effort) {
            (Some(model), Some(effort)) => format!("{model}/{effort}"),
            (Some(model), None) => model.clone(),
            _ => String::new(),
        };
        Self {
            started: row.started(),
            started_at: row.started_at.clone(),
            short: row
                .session_id
                .as_deref()
                .map(|id| crw_core::revert::short(id).to_owned())
                .unwrap_or_default(),
            title,
            mode: row.mode.clone(),
            model,
            took: row.took().map(ledger::length).unwrap_or_default(),
            state,
            cost: row
                .cost_usd
                .map(|cost| format!("${cost:.2}"))
                .unwrap_or_default(),
            figures: ledger::asked(row)
                .into_iter()
                .chain(ledger::shares(row))
                .collect(),
            result: row.result.as_deref().and_then(crate::sessions::result_line),
            page,
        }
    }
}

/// The notifications page (EP-31): the ledger's latest rows of kind `notification`, as `crw
/// notifications` reads them, the newest first.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Notifications {
    /// The window keeps a ledger, which it has read.
    pub read: bool,
    pub rows: Vec<LogLine>,
    /// Why the ledger did not read.
    pub problem: Option<String>,
}

/// The doctor's page (EP-31): the report the Diagnostics pane last gathered with when, and
/// whether a check runs; the real window's alone gathers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Doctor {
    pub live: bool,
    pub checking: bool,
    pub report: Option<(Report, DateTime<Utc>)>,
}

impl Doctor {
    /// The Diagnostics pane's report as it stands.
    pub fn of(diagnostics: &crate::diagnostics::Diagnostics) -> Self {
        Self {
            live: diagnostics.is_live(),
            checking: diagnostics.checking(),
            report: diagnostics.last().map(|(report, at)| (report.clone(), at)),
        }
    }
}

/// A project's page (EP-30a): what the Detail pane shows of it, as plain data.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub name: String,
    pub state: State,
    /// The folder is there, so the tool may write into it.
    pub folder: bool,
    pub next: Next,
    /// The next brief's note as written, its own `**EP-n.**` label left out.
    pub note: Option<String>,
    /// What waits on the owner (D-10): the open lines of `owner-tasks.md`, and how many are
    /// resolved.
    pub waiting: Vec<Item>,
    pub resolved: usize,
    /// The project keeps `owner-tasks.md` (EP-50): the page's words for the older forms follow.
    pub keeps_tasks: bool,
    /// The older forms, open, counted nowhere, with or without the file (EP-50).
    pub uncounted: Vec<Item>,
    /// The inbox; none when the project keeps no `roadmap/inbox.md`.
    pub inbox: Option<Mail>,
    /// What says a session is in progress, as the list's line says it.
    pub sessions: Vec<String>,
    /// A session at work there or a revert open, in the compose dialog's words.
    pub busy: Option<String>,
    /// A revert is open or running there: the page offers no tick and no message until it is
    /// done (the review's).
    pub revert: bool,
    /// The list's marks after the state (EP-30b, moved here from the list's line so that the
    /// page reads its own part alone): pinned, a state change proposed, the autopilot on.
    pub pinned: bool,
    pub proposes: bool,
    pub autopilot: bool,
    /// The window's own headless runs there that work, wait or are being stopped (EP-30b).
    pub runs: Vec<Run>,
}

/// One of the window's own headless runs in a project (EP-30b), with its Stop.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// The window's number for it, which a Stop names with its EP and its session (EP-31).
    pub key: u64,
    pub ep: String,
    pub session: String,
    /// Its state in words: running, waiting for an answer, stopping.
    pub state: String,
    pub started: DateTime<Utc>,
    /// It works or waits and is not being stopped: Stop is offered.
    pub stoppable: bool,
    /// The autopilot's chain started it.
    pub chained: bool,
}

impl Run {
    /// The window's runs in `project` that are not over, a chat's aside (it is the Chat pane's).
    pub fn of(board: &Board, project: &str) -> Vec<Self> {
        board
            .sessions
            .live
            .iter()
            .filter(|live| {
                !live.is_chat() && live.is_running() && live.project.eq_ignore_ascii_case(project)
            })
            .map(|live| Run {
                key: live.key,
                ep: live.ep.clone(),
                session: live.session_id.clone(),
                state: live.state(),
                started: live.started.with_timezone(&Utc),
                stoppable: !live.stopping,
                chained: live.chained,
            })
            .collect()
    }
}

/// The start page's preview (D-28, D-22; EP-30b): what a start of the project's next brief from
/// the phone would run, drafted by the window when the page asks, and what refuses it now.
#[derive(Debug, Clone, PartialEq)]
pub struct Start {
    pub project: String,
    /// The next brief, or why there is nothing to launch.
    pub next: Next,
    /// The rules' resolution, each value with its reason; empty without a draft.
    pub model: (String, String),
    pub effort: (String, String),
    pub ultracode: bool,
    pub permission_mode: (String, String),
    /// The downshift's words when it chose the model.
    pub downshift: Option<String>,
    /// What the run is, in sentences: its cap, its commits, the chain that may follow.
    pub run: Vec<String>,
    /// What the launch dialog would note first: owner tasks due, a session in progress, …
    pub notes: Vec<String>,
    /// What refuses the start now; none when it may go.
    pub refusals: Vec<String>,
}

/// One open item waiting on the owner.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: Option<String>,
    pub gate: Option<String>,
    /// As written, without the gate its chip says.
    pub text: String,
    /// Its form, file and line, state and origin.
    pub meta: String,
    /// A line of `owner-tasks.md`, which the tool ticks; the older forms are read-only.
    pub tickable: bool,
}

/// A project's inbox: its open messages, and how many it holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Mail {
    pub open: Vec<Message>,
    pub total: usize,
}

/// One open message.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: String,
    /// `from <sender>, <date>` as far as the line says.
    pub from: String,
    pub urgent: bool,
    /// Its words, without a trailing `[urgent]` the chip says.
    pub text: String,
}

impl Project {
    /// `row`'s page at `now`, with the window's sessions' words `live` and a session's words
    /// `busy`; its runs are the board's to add.
    pub fn of(
        row: &ProjectRow,
        live: Option<&str>,
        busy: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        let folder = row.folder.as_ref().map(|folder| folder.path.clone());
        let base = folder.clone().unwrap_or_default();
        let note = row.snapshot.current().and_then(|roadmap| {
            let next = roadmap.next()?;
            let note = roadmap.note(next.id)?;
            let label = format!("**{}.**", next.id);
            let text = note.text.strip_prefix(&label).unwrap_or(&note.text);
            Some(text.trim().to_owned())
        });
        let item = |item: &OwnerItem| Item {
            id: item.id.clone(),
            gate: item.gate.as_ref().map(ToString::to_string),
            text: crate::detail::without_gate(&item.text, item.gate.as_ref()).to_owned(),
            meta: crate::detail::meta(item, &base),
            tickable: item.is_tickable(),
        };
        let waiting = row.snapshot.waiting();
        let open: Vec<Item> = waiting
            .iter()
            .filter(|entry| entry.is_open())
            .map(item)
            .collect();
        let resolved = waiting.len() - open.len();
        let uncounted = row
            .snapshot
            .uncounted()
            .iter()
            .filter(|entry| entry.is_open())
            .map(item)
            .collect();
        let inbox = row.snapshot.inbox.as_ref().map(|inbox| Mail {
            open: inbox
                .open()
                .map(|message| Message {
                    id: message.id.clone(),
                    from: [message.from.clone(), message.date.clone()]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(", "),
                    urgent: message.urgent,
                    text: if message.urgent {
                        crw_core::inbox::without_urgent(&message.text).to_owned()
                    } else {
                        message.text.clone()
                    },
                })
                .collect(),
            total: inbox.messages.len(),
        });
        Self {
            name: row.name().to_owned(),
            state: row.project.state,
            folder: folder.is_some(),
            next: Next::of(row),
            note,
            waiting: open,
            resolved,
            keeps_tasks: row.snapshot.keeps_tasks(),
            uncounted,
            inbox,
            sessions: crate::list::session_words(row, live),
            busy,
            revert: false,
            pinned: row.project.pinned,
            proposes: row.proposes(now),
            autopilot: row.project.autopilot.enabled,
            runs: Vec::new(),
        }
    }
}

/// The decisions page (EP-30a): what the Decisions tile holds, as plain data.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Decisions {
    /// The requests the window's sessions wait on, a chat's left to the Chat pane.
    pub waiting: Vec<Asking>,
    /// The ended runs deferred with a request open, which an answer resumes.
    pub deferred: Vec<Asking>,
    /// The sessions Claude Code reports waiting outside the window, never answered here.
    pub elsewhere: Vec<Elsewhere>,
}

impl Decisions {
    /// What waits for the owner's answer, counted as the tile's title counts it.
    pub fn count(&self) -> usize {
        self.waiting.len() + self.deferred.len() + self.elsewhere.len()
    }

    /// The board's sessions as the Decisions tile reads them.
    pub fn of(board: &Board) -> Self {
        let mut decisions = Decisions::default();
        for live in board.sessions.live.iter().filter(|live| !live.is_chat()) {
            let (waiting, deferred) = Asking::of_live(live);
            decisions.waiting.extend(waiting);
            decisions.deferred.extend(deferred);
        }
        decisions.elsewhere = board
            .waiting_elsewhere()
            .into_iter()
            .map(|seen| {
                let (place, _) = crate::decisions::place(seen);
                Elsewhere {
                    project: seen.project.clone().unwrap_or_default(),
                    mode: seen.mode.as_str(),
                    words: seen.waits_for_you(place),
                    name: seen.name.clone(),
                }
            })
            .collect();
        decisions
    }
}

/// A request waiting for the owner's answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Asking {
    /// The window's session, by its key.
    pub key: u64,
    pub title: String,
    pub project: String,
    pub request: Request,
    /// When it came, or when the run was deferred.
    pub at: DateTime<Utc>,
    pub how: How,
}

impl Asking {
    /// What the window's session `live` waits on, as the Decisions tile reads it: its requests,
    /// and an ended run's deferred one, which an answer resumes (EP-31: the decisions page's and
    /// the session's page's).
    fn of_live(live: &Live) -> (Vec<Asking>, Option<Asking>) {
        let limit = live.launch_timeout().map(|limit| limit.as_secs() / 60);
        let waiting = live
            .waiting
            .iter()
            .map(|waiting| Asking {
                key: live.key,
                title: live.title(),
                project: live.project.clone(),
                request: waiting.request.clone(),
                at: waiting.at,
                how: How::Live { limit },
            })
            .collect();
        let deferred = live
            .open
            .as_ref()
            .filter(|_| live.ended.is_some())
            .map(|open| Asking {
                key: live.key,
                title: live.title(),
                project: live.project.clone(),
                request: open.request.clone(),
                at: open.at.with_timezone(&Utc),
                how: How::Deferred {
                    hook: open.deferred.is_some(),
                },
            });
        (waiting, deferred)
    }
}

/// How a request waits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How {
    /// A running session's, with the project's time limit in minutes.
    Live { limit: Option<u64> },
    /// An ended run's, deferred by the defer route's hook or else at the time limit.
    Deferred { hook: bool },
}

/// A session waiting outside the window.
#[derive(Debug, Clone, PartialEq)]
pub struct Elsewhere {
    pub project: String,
    pub mode: &'static str,
    /// It "waits for you in VS Code", in the tile's words.
    pub words: String,
    pub name: Option<String>,
}

/// The project list as the window's list shows it (D-25): its state groups in the list's order
/// and sort, no filter; the folders not adopted, by name.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct List {
    /// The files are being read and no project is listed yet.
    pub reading: bool,
    /// Why the registry or the data root did not read.
    pub problem: Option<String>,
    pub groups: Vec<Group>,
    pub unadopted: Vec<String>,
    /// The usage strip and the autopilot block (EP-30b, the D-28 addendum's).
    pub usage: Usage,
    pub autopilot: Autopilot,
}

/// The meter's figures for the list's strip (D-23; EP-30b): each with its reset and when it was
/// read, the ages drawn when the page is.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Usage {
    /// The meter runs: off in a test, canned in a screenshot run.
    pub on: bool,
    pub figures: Vec<Figure>,
    /// The reading gives the session (five-hour) limit with no window open.
    pub session_closed: bool,
    /// Why the endpoint's figures do not stand, when they do not.
    pub degraded: Option<String>,
}

/// One figure of the strip.
#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    /// `Week`, `Fable`, `Session (five-hour window)`.
    pub name: &'static str,
    pub percent: f64,
    pub resets_at: Option<DateTime<Utc>>,
    /// When it was read, which the steady comparison leaves out.
    pub at: DateTime<Utc>,
    /// The usage endpoint, or a session's status line.
    pub from: &'static str,
}

impl Usage {
    /// The meter's figures as of `now`, as the status bar's strip shows them.
    pub fn of(board: &Board, now: DateTime<Utc>) -> Self {
        let meter = &board.meter;
        if !meter.is_on() {
            return Self::default();
        }
        let figures = meter.gauge.figures(now);
        let shown = [
            ("Week", figures.weekly.as_ref()),
            ("Fable", figures.fable.as_ref()),
            ("Session (five-hour window)", figures.five_hour.as_ref()),
        ];
        Self {
            on: true,
            figures: shown
                .into_iter()
                .filter_map(|(name, figure)| {
                    let figure = figure?;
                    Some(Figure {
                        name,
                        percent: figure.limit.percent,
                        resets_at: figure.limit.resets_at,
                        at: figure.at,
                        from: match figure.source {
                            crw_core::usage::Source::Endpoint => "the usage endpoint",
                            crw_core::usage::Source::StatusLine => "a session's status line",
                        },
                    })
                })
                .collect(),
            session_closed: figures.five_hour.is_none()
                && meter
                    .gauge
                    .fetched
                    .as_ref()
                    .is_some_and(|fetched| fetched.reading.session_closed(now)),
            degraded: figures.degraded,
        }
    }
}

/// The list's autopilot block (D-22; EP-30b): what the window's Autopilot pane says (EP-52).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Autopilot {
    /// The owner's Pause, as the registry last said it.
    pub paused: bool,
    /// The automatic pause as it stands, each limit at or over its mark, in the pane's words.
    pub held_by_usage: Vec<String>,
    /// The projects whose autopilot is on.
    pub on: Vec<String>,
    /// The chained runs a day in all projects, and the day's chained starts in all projects as
    /// the gate counts them, none while the chain log does not read (EP-31b): the pane's line,
    /// read-only here (D-28).
    pub daily_total: u32,
    pub chained_today: Option<u32>,
    /// The held chains: the project and the log's words for the hold.
    pub held: Vec<(String, String)>,
    /// The chain plans: the project, the plan in words, and the mark the chain holds at.
    pub plans: Vec<(String, String, String)>,
    /// The chain log's latest lines, the newest first.
    pub log: Vec<LogLine>,
    /// Why the chain log did not read or was not kept.
    pub problem: Option<String>,
}

/// One line of the chain log, or of the notifications' (EP-31).
#[derive(Debug, Clone, PartialEq)]
pub struct LogLine {
    pub at: Option<DateTime<Utc>>,
    pub project: Option<String>,
    pub words: String,
}

impl LogLine {
    /// A ledger's event row.
    pub fn of(row: &EventRow) -> Self {
        Self {
            at: row.at(),
            project: row.project.clone(),
            words: row.detail.clone().unwrap_or_default(),
        }
    }
}

impl Autopilot {
    /// The window's autopilot as of `now`.
    pub fn of(board: &Board, now: DateTime<Utc>) -> Self {
        let pilot = &board.autopilot;
        let setting = pilot.five_hour_mark.max(1);
        let held_by_usage = chains::usage_pause(
            &board.meter.gauge.figures(now),
            pilot.mark.max(1),
            setting,
            now,
        )
        .iter()
        .map(|refusal| {
            // The session window's at the setting: a chain's own mark may let it go (EP-41).
            let who = match refusal {
                Refusal::Usage { which, .. } if *which == chains::FIVE_HOUR => {
                    "Chains without a mark of their own hold"
                }
                _ => "The chains hold",
            };
            format!("{who}: {}.", refusal.words())
        })
        .collect();
        Self {
            paused: pilot.paused,
            held_by_usage,
            on: board
                .portfolio
                .projects
                .iter()
                .filter(|row| row.project.autopilot.enabled)
                .map(|row| row.name().to_owned())
                .collect(),
            daily_total: pilot.daily_total,
            chained_today: pilot.chained_today,
            held: pilot
                .held
                .iter()
                .map(|held| (held.project.clone(), held.words.clone()))
                .collect(),
            plans: pilot
                .plans
                .values()
                .map(|plan| {
                    let mark = match plan.five_hour_mark {
                        Some(mark) => format!("{mark} % of {}", chains::FIVE_HOUR),
                        None => format!("the setting's {setting} % of {}", chains::FIVE_HOUR),
                    };
                    (plan.project.clone(), plan.words(), mark)
                })
                .collect(),
            log: pilot.log.iter().map(LogLine::of).collect(),
            problem: pilot.problem.clone(),
        }
    }
}

/// A state's group.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub state: State,
    pub projects: Vec<Line>,
}

impl Group {
    /// The group's heading, as the window's list names it.
    pub fn heading(&self) -> &'static str {
        crate::list::heading(GroupKind::State(self.state))
    }
}

/// One project as its row says it.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub name: String,
    pub state: State,
    pub pinned: bool,
    /// A state change is proposed and not dismissed (D-6).
    pub proposes: bool,
    /// The project's autopilot is on (D-22).
    pub autopilot: bool,
    pub next: Next,
    pub counts: Counts,
    /// The open owner tasks, the one form counted as waiting (D-10 addendum of 2026-11-22).
    pub owner_items: usize,
    /// Open inbox messages, and how many are urgent (D-13).
    pub messages: usize,
    pub urgent: usize,
    /// What says a session is in progress: the sessions' own words, then the files' evidence.
    pub sessions: Vec<String>,
    /// The week's share in tenths of a point (EP-18b), for a sort and a test.
    pub usage_tenths: Option<i64>,
    pub badges: Vec<Badge>,
}

/// The next brief, or why there is none.
#[derive(Debug, Clone, PartialEq)]
pub enum Next {
    Brief {
        id: String,
        size: Option<String>,
        /// The row's one-liner as written, its markup kept for the page to draw.
        title: String,
    },
    /// No next brief, in the row's words; `missing` when the folder is gone.
    None { words: String, missing: bool },
}

impl Next {
    /// `row`'s next brief, or why there is none in the row's words.
    fn of(row: &ProjectRow) -> Self {
        match (&row.folder, row.next()) {
            (None, _) => Next::None {
                words: crate::list::missing_folder(row),
                missing: true,
            },
            (Some(_), Some(next)) => Next::Brief {
                id: next.id.to_string(),
                size: next.size.map(|size| size.as_str().to_owned()),
                title: next.brief.title.clone(),
            },
            (Some(_), None) => Next::None {
                words: crate::list::no_next(row).to_owned(),
                missing: false,
            },
        }
    }
}

/// One badge in words (`badges::words`), its tone as the page's class.
#[derive(Debug, Clone, PartialEq)]
pub struct Badge {
    pub column: &'static str,
    pub word: String,
    pub sentence: String,
    pub tone: &'static str,
    /// The word is an age the clock alone changes (EP-30b): left out of the steady comparison.
    pub clocked: bool,
}

impl List {
    /// The list as the window's board holds it at `now`.
    pub fn of(board: &Board, now: DateTime<Utc>) -> Self {
        let portfolio = &board.portfolio;
        let filter = Filter {
            name: "",
            stacks: &NO_STACKS,
            hosting: &NO_CLASSES,
        };
        let sort: SortKey = board.view.sort;
        let groups = portfolio
            .groups(sort, filter)
            .into_iter()
            .filter_map(|group| match group.kind {
                GroupKind::State(state) => Some(Group {
                    state,
                    projects: group
                        .members
                        .iter()
                        .filter_map(|member| match member {
                            Member::Project(i) => {
                                let row = &portfolio.projects[*i];
                                let live = board.session_summary(row.name());
                                Some(Line::of(row, live.as_deref(), now))
                            }
                            Member::Folder(_) => None,
                        })
                        .collect(),
                }),
                GroupKind::Unadopted => None,
            })
            .collect();
        Self {
            reading: board.loader.reading_files && portfolio.projects.is_empty(),
            problem: portfolio.problem.clone(),
            groups,
            unadopted: portfolio
                .unadopted
                .iter()
                .map(|folder| folder.name.clone())
                .collect(),
            usage: Usage::of(board, now),
            autopilot: Autopilot::of(board, now),
        }
    }

    /// Every project listed.
    pub fn projects(&self) -> impl Iterator<Item = &Line> {
        self.groups.iter().flat_map(|group| group.projects.iter())
    }

    /// The list without what the clock alone changes (EP-30b): each badge's sentence (`pushed
    /// 3 h ago`, `checked 5 min ago`), the last commit's age word and the figures' read times.
    /// The version and the event stream compare this, so an idle window publishes no new
    /// version, while the snapshot itself keeps the words of each build for a page to draw.
    pub fn steady(&self) -> List {
        let mut list = self.clone();
        for line in list
            .groups
            .iter_mut()
            .flat_map(|group| group.projects.iter_mut())
        {
            for badge in &mut line.badges {
                badge.sentence.clear();
                if badge.clocked {
                    badge.word.clear();
                }
            }
        }
        for figure in &mut list.usage.figures {
            figure.at = DateTime::<Utc>::MIN_UTC;
        }
        list
    }
}

impl Line {
    /// `row` with the window's sessions' words `live`, at `now`.
    pub fn of(row: &ProjectRow, live: Option<&str>, now: DateTime<Utc>) -> Self {
        let (messages, urgent) = row.open_messages();
        Self {
            name: row.name().to_owned(),
            state: row.project.state,
            pinned: row.project.pinned,
            proposes: row.proposes(now),
            autopilot: row.project.autopilot.enabled,
            next: Next::of(row),
            counts: row.counts(),
            owner_items: row.open_items(),
            messages,
            urgent,
            sessions: crate::list::session_words(row, live),
            usage_tenths: row.usage_tenths(),
            badges: badges::words(row, now)
                .into_iter()
                .map(|word| Badge {
                    column: word.column,
                    word: word.word,
                    sentence: word.sentence,
                    tone: Tone::class(word.tone),
                    clocked: word.clocked,
                })
                .collect(),
        }
    }
}

/// The window's half: the channel's sender, the pages' ask and when it last built; the ledger's
/// latest notifications, when they were read (EP-31), and when a read that found the ledger
/// locked tries again (EP-43).
pub struct Publisher {
    sender: watch::Sender<Arc<Published>>,
    asked: Arc<AtomicBool>,
    built: Option<Instant>,
    notifications: Notifications,
    notified: Option<Instant>,
    retry_at: Option<Instant>,
    /// A screenshot run's or a test's notifications, never read from a ledger.
    canned: bool,
}

impl Default for Publisher {
    fn default() -> Self {
        let (sender, _) = watch::channel(Arc::new(Published::empty()));
        Self {
            sender,
            asked: Arc::new(AtomicBool::new(false)),
            built: None,
            notifications: Notifications::default(),
            notified: None,
            retry_at: None,
            canned: false,
        }
    }
}

impl Publisher {
    /// The ledger's latest notifications as last read (EP-31).
    pub fn notifications(&self) -> &Notifications {
        &self.notifications
    }

    /// Reads the ledger's latest notifications again once [`NOTIFICATIONS_EVERY`] has passed
    /// (EP-31), without writing it: the toasts and the pushes' thread add them. A ledger that does
    /// not read keeps the rows last read, with why; one locked by another connection past a
    /// moment's wait keeps them without a word and is read again a second later, so that the
    /// window's pass never waits out the ledger's five seconds here (EP-43).
    pub fn read_notifications(&mut self, ledger: Option<&Path>) {
        if self.canned
            || self.retry_at.is_some_and(|at| Instant::now() < at)
            || self
                .notified
                .is_some_and(|at| at.elapsed() < NOTIFICATIONS_EVERY)
        {
            return;
        }
        self.notified = Some(Instant::now());
        self.retry_at = None;
        let Some(path) = ledger else {
            self.notifications = Notifications::default();
            return;
        };
        if !path.exists() {
            self.notifications = Notifications {
                read: true,
                ..Notifications::default()
            };
            return;
        }
        let read = Ledger::open_read_only_waiting(path, crate::pace::LEDGER_WAIT)
            .and_then(|ledger| ledger.events(crw_core::notify::EVENT_KIND, Some(NOTIFICATIONS)));
        match read {
            Ok(rows) => {
                self.notifications = Notifications {
                    read: true,
                    rows: rows.iter().map(LogLine::of).collect(),
                    problem: None,
                };
            }
            Err(e) if e.is_busy() => {
                self.notified = None;
                self.retry_at = Some(crate::pace::retry_after(Instant::now()));
            }
            Err(e) => self.notifications.problem = Some(format!("The ledger did not read: {e}")),
        }
    }

    /// A screenshot run's or a test's notifications, shown as they are (EP-31).
    #[cfg(any(test, feature = "screenshot"))]
    pub fn can_notifications(&mut self, rows: Vec<LogLine>) {
        self.canned = true;
        self.notifications = Notifications {
            read: true,
            rows,
            problem: None,
        };
    }

    /// The server's half, for each page to read.
    pub fn reader(&self) -> Reader {
        Reader {
            receiver: self.sender.subscribe(),
            asked: Arc::clone(&self.asked),
        }
    }

    /// Asks for a build at the next pass, as a page's ask does (EP-30a): the files landed, or a
    /// form ran.
    pub fn refresh(&self) {
        self.asked.store(true, Ordering::Release);
    }

    /// Whether this pass builds the snapshot: a page asked, or a second has passed.
    pub fn due(&self) -> bool {
        self.asked.load(Ordering::Acquire) || self.built.is_none_or(|at| at.elapsed() >= EVERY)
    }

    /// Takes the parts built at `now`: sent with a new version when they differ from the ones
    /// published, sent with their time renewed when a page asked; else their time is renewed
    /// without a word to the receivers, so that a page reads them as fresh while the window passes
    /// (the review's: `at` is the last build, not the last send). The list is compared steady
    /// (EP-30b): a badge's age moving with the clock is no change, though the snapshot takes it.
    pub fn offer(&mut self, parts: Parts, now: DateTime<Utc>) {
        self.built = Some(Instant::now());
        let asked = self.asked.swap(false, Ordering::AcqRel);
        let Parts {
            list,
            projects,
            decisions,
            sessions,
            runs,
            notifications,
            doctor,
        } = parts;
        self.sender.send_if_modified(|current| {
            let changed = current.list.steady() != list.steady()
                || current.projects != projects
                || current.decisions != decisions
                || current.sessions != sessions
                || current.runs != runs
                || current.notifications != notifications
                || current.doctor != doctor;
            let version = current.version + u64::from(changed);
            *current = Arc::new(Published {
                at: now,
                version,
                list,
                projects,
                decisions,
                sessions,
                runs,
                notifications,
                doctor,
            });
            changed || asked
        });
    }

    /// The snapshot published last.
    #[cfg(test)]
    pub fn current(&self) -> Arc<Published> {
        Arc::clone(&self.sender.borrow())
    }
}

/// The server's half: the snapshot as last published, and the ask for a fresh one.
#[derive(Clone)]
pub struct Reader {
    receiver: watch::Receiver<Arc<Published>>,
    asked: Arc<AtomicBool>,
}

impl Reader {
    /// The version of the snapshot published last.
    pub fn version(&self) -> u64 {
        self.receiver.borrow().version
    }

    /// Waits at most `wait` for a snapshot whose version is above `version` (EP-30a): after a
    /// form's write, so that the page its redirect shows holds what the write changed (the
    /// window reads the files again after it, off its thread, and builds at most once a second).
    pub async fn newer_than(&self, version: u64, wait: Duration) {
        let mut receiver = self.receiver.clone();
        let _ = tokio::time::timeout(
            wait,
            receiver.wait_for(|published| published.version > version),
        )
        .await;
    }

    /// The parts that change from now on, each by the path of the page that shows it (EP-30b):
    /// the event stream's. It holds the snapshot it last saw and awaits the channel's change;
    /// when the version moved it names each part that differs (`Published::changed_paths`),
    /// and an ask that renewed the time alone names nothing. `since` is the version the page
    /// was drawn from: when the window has moved on since, the first item is [`EVERYTHING`],
    /// since a change made between the page's drawing and the stream's opening would otherwise
    /// go unnamed (a run's start just after the page that shows it starting). It ends when the
    /// window's publisher goes, as at the window's quit.
    pub fn changes(&self, since: Option<u64>) -> impl Stream<Item = String> + Send + 'static {
        let mut receiver = self.receiver.clone();
        let last = Arc::clone(&receiver.borrow_and_update());
        let mut pending = VecDeque::new();
        if since.is_some_and(|since| since != last.version) {
            pending.push_back(EVERYTHING.to_owned());
        }
        futures_util::stream::unfold(
            (receiver, last, pending),
            |(mut receiver, mut last, mut pending)| async move {
                loop {
                    if let Some(path) = pending.pop_front() {
                        return Some((path, (receiver, last, pending)));
                    }
                    receiver.changed().await.ok()?;
                    let now = Arc::clone(&receiver.borrow_and_update());
                    if now.version != last.version {
                        pending.extend(now.changed_paths(&last));
                        last = now;
                    }
                }
            },
        )
    }

    /// The snapshot for a page: the one held when it is fresh, else the window's next, asked for
    /// through `wake` and waited for at most [`WAIT`], else the one held.
    pub async fn fresh(&self, wake: impl Fn()) -> Arc<Published> {
        let mut receiver = self.receiver.clone();
        let held = Arc::clone(&receiver.borrow_and_update());
        let age = Utc::now().signed_duration_since(held.at);
        if age.to_std().is_ok_and(|age| age < FRESH_FOR) {
            return held;
        }
        self.asked.store(true, Ordering::Release);
        wake();
        match tokio::time::timeout(WAIT, receiver.changed()).await {
            Ok(Ok(())) => Arc::clone(&receiver.borrow_and_update()),
            _ => held,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Places, View};
    use crate::portfolio::tests::{Scratch, fake_store};
    use eframe::egui;

    /// The latest notifications give up on a ledger another connection holds within a moment,
    /// keep the rows last read without a word, and are read again a second later rather than
    /// at the next 15 s (EP-43).
    #[test]
    fn the_notifications_wait_a_moment_at_most_on_a_held_lock() {
        let scratch = Scratch::new("published-busy");
        let path = scratch.0.join("ledger.sqlite");
        let note = |line: &str| {
            Ledger::open(&path)
                .unwrap()
                .add_event(
                    chrono::Utc::now(),
                    crw_core::notify::EVENT_KIND,
                    Some("harbor"),
                    line,
                )
                .unwrap();
        };
        note("popup · finished · harbor: a session's turn is over");
        let mut publisher = Publisher::default();
        publisher.read_notifications(Some(&path));
        assert_eq!(publisher.notifications().rows.len(), 1);
        note("popup · waiting · harbor waits for you");
        let holder = rusqlite::Connection::open(&path).unwrap();
        holder.execute_batch("BEGIN EXCLUSIVE").unwrap();
        publisher.notified = None;
        let began = Instant::now();
        publisher.read_notifications(Some(&path));
        assert!(
            began.elapsed() < Duration::from_secs(2),
            "{:?}",
            began.elapsed()
        );
        assert_eq!(
            publisher.notifications().rows.len(),
            1,
            "the rows last read"
        );
        assert_eq!(publisher.notifications().problem, None);
        holder.execute_batch("ROLLBACK").unwrap();
        // Not within its second; after it, read without waiting out the 15 s.
        publisher.read_notifications(Some(&path));
        assert_eq!(publisher.notifications().rows.len(), 1);
        publisher.retry_at = Some(Instant::now());
        publisher.read_notifications(Some(&path));
        assert_eq!(publisher.notifications().rows.len(), 2);
        assert_eq!(publisher.notifications().problem, None);
    }

    /// A board over the fake portfolio, its files read.
    fn board(scratch: &Scratch) -> Board {
        let places = Places {
            store: fake_store(scratch),
            ledger: None,
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        let mut board = Board::new(places, &ctx, View::default(), egui::ThemePreference::Light);
        let deadline = Instant::now() + Duration::from_secs(10);
        while board.loader.reading_files && Instant::now() < deadline {
            board.receive(&ctx);
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!board.loader.reading_files, "the files were read");
        board
    }

    #[test]
    fn the_list_says_what_the_rows_say() {
        let scratch = Scratch::new("published-list");
        let board = board(&scratch);
        let now = Utc::now();
        let list = List::of(&board, now);
        assert!(!list.reading);
        assert_eq!(list.problem, None);
        // The groups in the window's order, each project once.
        let states: Vec<State> = list.groups.iter().map(|group| group.state).collect();
        let order: Vec<State> = GroupKind::ORDER
            .iter()
            .filter_map(|kind| match kind {
                GroupKind::State(state) if states.contains(state) => Some(*state),
                _ => None,
            })
            .collect();
        assert_eq!(states, order);
        assert_eq!(list.projects().count(), board.portfolio.projects.len());
        assert_eq!(list.groups[0].heading(), "Active");
        // Each line as its row reads.
        for row in &board.portfolio.projects {
            let line = list
                .projects()
                .find(|line| line.name == row.name())
                .unwrap();
            assert_eq!(line.counts, row.counts());
            assert_eq!(line.owner_items, row.open_items());
            assert_eq!((line.messages, line.urgent), row.open_messages());
            match (row.next(), &line.next) {
                (Some(next), Next::Brief { id, title, .. }) => {
                    assert_eq!(id, &next.id.to_string());
                    assert_eq!(title, &next.brief.title);
                }
                (None, Next::None { words, .. }) => assert!(!words.is_empty()),
                (next, line) => panic!("{}: {next:?} against {line:?}", row.name()),
            }
            assert_eq!(
                line.badges.iter().any(|badge| badge.column == "Hosting"),
                row.folder.is_some(),
                "{}: a folder has its hosting",
                row.name()
            );
        }
        assert_eq!(
            list.unadopted,
            board
                .portfolio
                .unadopted
                .iter()
                .map(|folder| folder.name.clone())
                .collect::<Vec<_>>()
        );
    }

    /// Each project's page says what its Detail pane says, the decisions what the tile holds, a
    /// chat's question left to the Chat pane; a second build with nothing new is the same
    /// content, whatever the clock says (EP-30a).
    #[test]
    fn the_pages_parts_say_what_the_detail_pane_and_the_tile_say() {
        use crw_core::session::{Ended, Event, Launch, Outcome, Question};

        use crate::sessions::Live;

        let scratch = Scratch::new("published-parts");
        let mut board = board(&scratch);
        let now = Utc::now();
        let parts = Parts::of(&board, now);
        assert_eq!(parts.projects.len(), board.portfolio.projects.len());
        for row in &board.portfolio.projects {
            let page = parts
                .projects
                .iter()
                .find(|page| page.name == row.name())
                .unwrap();
            assert_eq!(
                page.waiting.len(),
                row.snapshot.open_waiting(),
                "{}",
                row.name()
            );
            assert_eq!(
                page.waiting.len() + page.resolved,
                row.snapshot.waiting().len()
            );
            assert_eq!(
                page.inbox.as_ref().map(|inbox| inbox.total),
                row.snapshot
                    .inbox
                    .as_ref()
                    .map(|inbox| inbox.messages.len())
            );
            if let (Some(next), Some(note)) = (row.next(), &page.note) {
                assert!(
                    !note.starts_with(&format!("**{}.**", next.id)),
                    "the bullet's own label is left out: {note}"
                );
            }
            assert_eq!(page.folder, row.folder.is_some());
        }
        assert!(parts.projects.iter().any(|page| page.note.is_some()));
        assert!(
            parts
                .projects
                .iter()
                .any(|page| page.waiting.iter().any(|item| item.tickable))
        );
        assert!(parts.projects.iter().any(|page| {
            page.inbox
                .as_ref()
                .is_some_and(|inbox| !inbox.open.is_empty())
        }));
        // The projects' pages and the decisions hold no age: later, they are the same.
        let later = Parts::of(&board, now + chrono::TimeDelta::minutes(3));
        assert_eq!(later.projects, parts.projects);
        assert_eq!(later.decisions, parts.decisions);
        // The window's sessions: a request waiting, a deferred run's, a chat's left out.
        let question = Question {
            text: "Which shelf?".to_owned(),
            header: "Shelf".to_owned(),
            options: Vec::new(),
            multi_select: false,
        };
        let request = |id: &str| crw_core::session::Request {
            id: id.to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which shelf?".to_owned(),
            reason: None,
            questions: vec![question.clone()],
            input: String::new(),
        };
        let launch = |project: &str| {
            let mut launch = Launch::new(project, std::path::Path::new(project), "fable", "go");
            launch.ep = Some("EP-4".to_owned());
            launch
        };
        let mut chat = launch("chat");
        chat.conversation = Some("What is next?".to_owned());
        board.sessions.live.extend([
            Live::canned(
                1,
                launch("harbor"),
                "high",
                vec![Event::Request(request("a"))],
            ),
            Live::canned(
                2,
                launch("meadow"),
                "high",
                vec![
                    Event::Request(request("b")),
                    Event::TimedOut("b".to_owned()),
                    Event::Exited(Ended {
                        outcome: Outcome::Deferred,
                        exit_code: Some(0),
                        result: None,
                        stderr: Vec::new(),
                    }),
                ],
            ),
            Live::canned(3, chat, "high", vec![Event::Request(request("c"))]),
        ]);
        let decisions = Decisions::of(&board);
        assert_eq!(decisions.waiting.len(), 1, "{decisions:?}");
        assert_eq!(
            (
                decisions.waiting[0].key,
                decisions.waiting[0].request.id.as_str()
            ),
            (1, "a")
        );
        assert_eq!(
            decisions.waiting[0].at,
            board.sessions.live[0].waiting[0].at
        );
        assert_eq!(decisions.deferred.len(), 1);
        assert_eq!(decisions.deferred[0].request.id, "b");
        assert_eq!(decisions.deferred[0].how, How::Deferred { hook: false });
        assert_eq!(decisions.count(), 2);
        // An age read from the window's clock would show here (the review's): the requests
        // backdated ten minutes, the decisions are the same.
        let old = Instant::now()
            .checked_sub(Duration::from_secs(600))
            .unwrap();
        board.sessions.live[0].waiting[0].since = old;
        if let Some(open) = board.sessions.live[1].open.as_mut() {
            open.since = old;
        }
        assert_eq!(Decisions::of(&board), decisions, "no age in it");
    }

    /// The window's own session joins the line's words, before the files' evidence.
    #[test]
    fn a_line_carries_the_sessions_words() {
        let scratch = Scratch::new("published-session");
        let board = board(&scratch);
        let row = &board.portfolio.projects[0];
        let line = Line::of(
            row,
            Some("a headless EP-4 started here at 10:00 is working"),
            Utc::now(),
        );
        assert_eq!(
            line.sessions.first().map(String::as_str),
            Some("a headless EP-4 started here at 10:00 is working")
        );
    }

    #[test]
    fn a_change_raises_the_version_and_an_ask_renews_the_time_alone() {
        let mut publisher = Publisher::default();
        let reader = publisher.reader();
        let mut listener = reader.receiver.clone();
        assert!(publisher.due(), "never built");
        // The first version is the process's start in milliseconds (EP-30b's review), so that a
        // page from an earlier window cannot match a later one's by chance.
        let v = publisher.current().version;
        assert!(v > 1_700_000_000_000, "{v}");
        let first = Utc::now();
        let list = parts(List {
            problem: Some("a problem".to_owned()),
            ..List::default()
        });
        publisher.offer(list.clone(), first);
        assert_eq!(publisher.current().version, v + 1);
        assert!(listener.has_changed().unwrap(), "a change is heard");
        listener.borrow_and_update();
        assert!(!publisher.due(), "built a moment ago");
        // The same content again: the time renewed, the version kept, nothing heard.
        let again = first + chrono::TimeDelta::seconds(5);
        publisher.offer(list.clone(), again);
        assert_eq!(
            (publisher.current().version, publisher.current().at),
            (v + 1, again)
        );
        assert!(!listener.has_changed().unwrap(), "no word of a time alone");
        // A page's ask: the same version, the time renewed, heard, and due at once.
        reader.asked.store(true, Ordering::Release);
        assert!(publisher.due());
        let later = first + chrono::TimeDelta::seconds(9);
        publisher.offer(list, later);
        assert_eq!(
            (publisher.current().version, publisher.current().at),
            (v + 1, later)
        );
        assert!(
            listener.has_changed().unwrap(),
            "the answer to an ask is heard"
        );
        listener.borrow_and_update();
        assert!(!reader.asked.load(Ordering::Acquire), "the ask was taken");
        // A change: a new version.
        publisher.offer(Parts::default(), later);
        assert_eq!(publisher.current().version, v + 2);
        assert!(listener.has_changed().unwrap());
        listener.borrow_and_update();
        // A change to a project's page or to the decisions alone is a change too (EP-30a).
        let mut with_decision = Parts::default();
        with_decision.decisions.elsewhere.push(Elsewhere {
            project: "harbor".to_owned(),
            mode: "visual",
            words: "waits for you in VS Code".to_owned(),
            name: None,
        });
        publisher.offer(with_decision.clone(), later);
        assert_eq!(publisher.current().version, v + 3);
        assert!(listener.has_changed().unwrap());
        listener.borrow_and_update();
        publisher.offer(with_decision, later);
        assert_eq!(publisher.current().version, v + 3, "the same parts again");
        assert!(!listener.has_changed().unwrap());
    }

    fn parts(list: List) -> Parts {
        Parts {
            list,
            ..Parts::default()
        }
    }

    /// A project's page with nothing in it, for the paths' and the stream's tests.
    fn page(name: &str) -> Project {
        Project {
            name: name.to_owned(),
            state: State::Active,
            folder: true,
            next: Next::None {
                words: "Every row is checked".to_owned(),
                missing: false,
            },
            note: None,
            waiting: Vec::new(),
            resolved: 0,
            keeps_tasks: true,
            uncounted: Vec::new(),
            inbox: None,
            sessions: Vec::new(),
            busy: None,
            revert: false,
            pinned: false,
            proposes: false,
            autopilot: false,
            runs: Vec::new(),
        }
    }

    /// A list of one project with a sync badge and the last commit's age, and a weekly figure
    /// read at `read`.
    fn aged_list(sync: &str, age: &str, read: DateTime<Utc>) -> List {
        let badge = |column: &'static str, word: &str, clocked: bool| Badge {
            column,
            word: word.to_owned(),
            sentence: format!("{word}: checked {age}"),
            tone: "plain",
            clocked,
        };
        List {
            groups: vec![Group {
                state: State::Active,
                projects: vec![Line {
                    name: "harbor".to_owned(),
                    state: State::Active,
                    pinned: false,
                    proposes: false,
                    autopilot: false,
                    next: Next::None {
                        words: "Every row is checked".to_owned(),
                        missing: false,
                    },
                    counts: Counts::default(),
                    owner_items: 0,
                    messages: 0,
                    urgent: 0,
                    sessions: Vec::new(),
                    usage_tenths: None,
                    badges: vec![
                        badge("Sync", sync, false),
                        badge("Age", &format!("last commit {age}"), true),
                    ],
                }],
            }],
            usage: Usage {
                on: true,
                figures: vec![Figure {
                    name: "Week",
                    percent: 41.0,
                    resets_at: None,
                    at: read,
                    from: "the usage endpoint",
                }],
                ..Usage::default()
            },
            ..List::default()
        }
    }

    /// The clock alone raises no version (EP-30b): the badges' sentences, the last commit's age
    /// and a figure's read time change in the snapshot, which a page reads, and not its version;
    /// a real change does, and each part that changed is named by its page's path.
    #[test]
    fn the_clock_alone_raises_no_version_and_a_change_names_its_part() {
        let now = Utc::now();
        let minute = chrono::TimeDelta::minutes(1);
        let first = aged_list("↑1 not pushed", "3 min ago", now);
        let later = aged_list("↑1 not pushed", "4 min ago", now + minute);
        assert_ne!(first, later, "each build keeps its words");
        assert_eq!(first.steady(), later.steady());
        let mut publisher = Publisher::default();
        publisher.offer(parts(first.clone()), now);
        let version = publisher.current().version;
        publisher.offer(parts(later.clone()), now + minute);
        assert_eq!(publisher.current().version, version, "the clock alone");
        assert!(
            publisher.current().list.groups[0].projects[0].badges[0]
                .sentence
                .ends_with("4 min ago"),
            "a page reads the newest words"
        );
        // A word that is no age is a change.
        let pushed = aged_list("↑2 not pushed", "4 min ago", now + minute);
        publisher.offer(parts(pushed.clone()), now + minute);
        assert_eq!(publisher.current().version, version + 1);
        // The paths: the list, a project changed, one added, the decisions; none for the clock.
        let base = Published::of(
            Parts {
                list: first,
                projects: vec![page("harbor & co")],
                ..Parts::default()
            },
            now,
            1,
        );
        let mut aged = base.clone();
        aged.list = later;
        assert!(aged.changed_paths(&base).is_empty());
        let mut changed = base.clone();
        changed.list = pushed;
        changed.projects[0].resolved = 1;
        changed.projects.push(page("meadow"));
        changed.decisions.elsewhere.push(Elsewhere {
            project: "meadow".to_owned(),
            mode: "visual",
            words: "waits for you in VS Code".to_owned(),
            name: None,
        });
        assert_eq!(
            changed.changed_paths(&base),
            [
                "/m/",
                "/m/p/harbor%20%26%20co",
                "/m/p/meadow",
                "/m/decisions"
            ]
        );
        // A project gone is named too.
        assert_eq!(
            base.changed_paths(&changed),
            [
                "/m/",
                "/m/p/harbor%20%26%20co",
                "/m/p/meadow",
                "/m/decisions"
            ]
        );
    }

    /// The event stream's source (EP-30b): each change named by its part's path, nothing for an
    /// ask that renewed the time alone, and the end with the window's publisher.
    #[test]
    fn the_stream_names_each_change_and_ends_with_the_window() {
        use futures_util::StreamExt;

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut publisher = Publisher::default();
        let reader = publisher.reader();
        let mut stream = Box::pin(reader.changes(None));
        let mut next = |wait: u64| {
            runtime.block_on(async {
                tokio::time::timeout(Duration::from_millis(wait), stream.next())
                    .await
                    .ok()
                    .flatten()
            })
        };
        // The list as the empty snapshot holds it, so that only the project changes.
        let reading = || List {
            reading: true,
            ..List::default()
        };
        let mut harbor = Parts {
            list: reading(),
            projects: vec![page("harbor")],
            ..Parts::default()
        };
        publisher.offer(harbor.clone(), Utc::now());
        assert_eq!(next(2000).as_deref(), Some("/m/p/harbor"));
        // An ask answered with the same content: heard by the channel, named by no event.
        reader.asked.store(true, Ordering::Release);
        publisher.offer(harbor.clone(), Utc::now());
        assert_eq!(next(300), None);
        // Two parts at once: both named, in the pages' order.
        harbor.projects[0].resolved = 2;
        harbor.decisions.elsewhere.push(Elsewhere {
            project: "harbor".to_owned(),
            mode: "visual",
            words: "waits for you in VS Code".to_owned(),
            name: None,
        });
        publisher.offer(harbor, Utc::now());
        assert_eq!(next(2000).as_deref(), Some("/m/p/harbor"));
        assert_eq!(next(2000).as_deref(), Some("/m/decisions"));
        // A page drawn from an older version than the window's now: everything, first; one drawn
        // from the current version: nothing until a change.
        let current = publisher.current().version;
        let catch_up = |since: u64| {
            let mut stream = Box::pin(reader.changes(Some(since)));
            runtime.block_on(async {
                tokio::time::timeout(Duration::from_millis(300), stream.next())
                    .await
                    .ok()
                    .flatten()
            })
        };
        assert_eq!(catch_up(current - 1).as_deref(), Some(EVERYTHING));
        assert_eq!(catch_up(current), None);
        // The window's quit: the stream ends, rather than waits (the review's: a timeout is no
        // end).
        drop(publisher);
        let ended = runtime.block_on(async {
            tokio::time::timeout(Duration::from_millis(2000), stream.next()).await
        });
        assert!(matches!(ended, Ok(None)), "{ended:?}");
    }

    /// The autopilot block says what the Autopilot pane says (EP-30b, EP-52): the owner's Pause, the
    /// held chains, the plans with their marks and the log; the usage strip is off with the
    /// meter.
    #[test]
    fn the_autopilot_block_says_what_the_section_says() {
        let scratch = Scratch::new("published-autopilot");
        let mut board = board(&scratch);
        board.autopilot.paused = true;
        board.autopilot.held.push(crate::autopilot::Held {
            project: "harbor".to_owned(),
            after: "EP-3".parse().ok(),
            words: "held after EP-3 until the pause lifts: the autopilot is paused (your Pause)"
                .to_owned(),
        });
        board.autopilot.keep_plan(chains::Plan {
            project: "harbor".to_owned(),
            after: "EP-3".parse().unwrap(),
            briefs: Vec::new(),
            stop: None,
            daily: 2,
            five_hour_mark: Some(70),
        });
        board.autopilot.log.push(crw_core::ledger::EventRow {
            id: 1,
            at: "2026-11-16T20:48:00+00:00".to_owned(),
            kind: chains::EVENT_KIND.to_owned(),
            project: Some("harbor".to_owned()),
            detail: Some("chained EP-4 after EP-3".to_owned()),
        });
        // The day's count and the total as the window read them (EP-31b).
        board.autopilot.daily_total = 8;
        board.autopilot.chained_today = Some(3);
        let now = Utc::now();
        let block = Autopilot::of(&board, now);
        assert_eq!((block.chained_today, block.daily_total), (Some(3), 8));
        assert!(block.paused);
        assert_eq!(block.held.len(), 1);
        assert!(block.held[0].1.starts_with("held after EP-3"));
        assert_eq!(block.plans.len(), 1);
        assert_eq!(block.plans[0].0, "harbor");
        assert!(
            block.plans[0]
                .1
                .starts_with("after EP-3: no brief to chain")
        );
        assert_eq!(block.plans[0].2, "70 % of the session (five-hour) window");
        assert_eq!(block.log[0].words, "chained EP-4 after EP-3");
        assert!(block.log[0].at.is_some());
        assert!(block.held_by_usage.is_empty(), "no figure is read");
        assert_eq!(
            block.on,
            board
                .portfolio
                .projects
                .iter()
                .filter(|row| row.project.autopilot.enabled)
                .map(|row| row.name().to_owned())
                .collect::<Vec<_>>()
        );
        // The meter is off in a test: the strip says so.
        assert_eq!(Usage::of(&board, now), Usage::default());
        // The list carries both, and an idle second build is the same steady content.
        let list = List::of(&board, now);
        assert_eq!(list.autopilot, block);
        assert_eq!(
            list.steady(),
            List::of(&board, now + chrono::TimeDelta::minutes(5)).steady()
        );
        // A new chained start moves the list: the stream names `/m/` (EP-31b).
        board.autopilot.chained_today = Some(4);
        assert_ne!(list.steady(), List::of(&board, now).steady());
    }

    /// A page with an old snapshot asks the window and takes its answer; a fresh one is taken
    /// as it is; a window that never answers leaves the page the one held.
    #[test]
    fn a_page_asks_for_a_fresh_snapshot_only_when_its_own_is_old() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut publisher = Publisher::default();
        let reader = publisher.reader();
        // Nothing published: the window is asked, and here never answers.
        let woken = std::sync::atomic::AtomicUsize::new(0);
        let started = Instant::now();
        let held = runtime.block_on(reader.fresh(|| {
            woken.fetch_add(1, Ordering::SeqCst);
        }));
        assert_eq!(woken.load(Ordering::SeqCst), 1);
        assert_eq!(
            held.version,
            publisher.current().version,
            "the empty snapshot"
        );
        assert!(held.list.reading);
        assert!(started.elapsed() >= WAIT, "it waited for the answer");
        // Published just now: taken as it is, no ask.
        publisher.offer(Parts::default(), Utc::now());
        let fresh = runtime.block_on(reader.fresh(|| panic!("a fresh snapshot needs no ask")));
        assert_eq!(fresh.version, publisher.current().version);
        // An old snapshot: the ask is answered by the next offer.
        publisher.offer(
            parts(List {
                reading: true,
                ..List::default()
            }),
            Utc::now() - chrono::TimeDelta::seconds(30),
        );
        let reader_for_page = reader.clone();
        let page = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(reader_for_page.fresh(|| {}))
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        while !reader.asked.load(Ordering::Acquire) {
            assert!(Instant::now() < deadline, "the page never asked");
            std::thread::sleep(Duration::from_millis(5));
        }
        let answered_at = Utc::now();
        publisher.offer(
            parts(List {
                reading: true,
                ..List::default()
            }),
            answered_at,
        );
        let answer = page.join().unwrap();
        assert_eq!(
            answer.at, answered_at,
            "the window's answer, not the old one"
        );
    }

    /// A window session of `project` with `events` taken in, canned.
    fn live(key: u64, project: &str, events: Vec<crw_core::session::Event>) -> Live {
        let mut launch =
            crw_core::session::Launch::new(project, std::path::Path::new(project), "fable", "Go.");
        launch.ep = Some("EP-4".to_owned());
        Live::canned(key, launch, "high", events)
    }

    fn question(id: &str) -> crw_core::session::Event {
        crw_core::session::Event::Request(Request {
            id: id.to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which shelf?".to_owned(),
            reason: None,
            questions: Vec::new(),
            input: String::new(),
        })
    }

    /// A session's page (EP-31): the transcript's latest entries with their place in it, the
    /// earlier ones counted; the streaming text to its last whole line, so a word more is the same
    /// content and no new version, a line more a new one; what it waits on; its path named when
    /// it changes, the sessions list's, the notifications' and the report's too.
    #[test]
    fn a_sessions_page_carries_its_latest_entries_and_whole_lines() {
        use crw_core::session::Event;

        let events: Vec<Event> = (0..PAGE_ENTRIES + 10)
            .map(|n| Event::Text(format!("line {n}")))
            .chain([question("req-1")])
            .collect();
        let mut run = live(3, "harbor", events);
        let page = SessionPage::of(&run);
        // The prompt, the texts and the request: the latest PAGE_ENTRIES of them.
        assert_eq!(page.entries.len(), PAGE_ENTRIES);
        assert_eq!(page.first, 12, "the prompt and eleven texts before");
        assert_eq!(page.entries.last(), run.transcript.entries.last());
        assert_eq!(page.waiting.len(), 1);
        assert_eq!(page.waiting[0].request.id, "req-1");
        assert_eq!(page.deferred, None);
        assert_eq!((page.key, page.id.as_str()), (3, run.session_id.as_str()));
        assert_eq!(page.title, "harbor EP-4");
        assert!(page.stoppable && page.running);
        assert_eq!(page.state, "waiting for an answer");
        // The streaming text, a line at a time.
        run.take(Event::TextDelta("The first line\nand the sec".to_owned()));
        let streaming = SessionPage::of(&run);
        assert_eq!(streaming.streaming, "The first line");
        run.take(Event::TextDelta("ond".to_owned()));
        assert_eq!(SessionPage::of(&run), streaming, "a word more is no change");
        run.take(Event::TextDelta(" line\nthe third".to_owned()));
        assert_eq!(
            SessionPage::of(&run).streaming,
            "The first line\nand the second line"
        );
        assert_eq!(whole_lines("no line yet"), "");
        // Through the publisher: a word more keeps the version, a line more raises it.
        let mut publisher = Publisher::default();
        let mut parts = Parts {
            runs: vec![SessionPage::of(&run)],
            ..Parts::default()
        };
        publisher.offer(parts.clone(), Utc::now());
        let version = publisher.current().version;
        run.take(Event::TextDelta(" line".to_owned()));
        parts.runs = vec![SessionPage::of(&run)];
        publisher.offer(parts.clone(), Utc::now());
        assert_eq!(publisher.current().version, version);
        run.take(Event::TextDelta("\n".to_owned()));
        parts.runs = vec![SessionPage::of(&run)];
        publisher.offer(parts, Utc::now());
        assert_eq!(publisher.current().version, version + 1);
        // The paths: a session's page changed and one added, the list, the notifications and
        // the report; none for a page the same.
        let base = Published::of(
            Parts {
                runs: vec![SessionPage::of(&live(1, "harbor", Vec::new()))],
                ..Parts::default()
            },
            Utc::now(),
            1,
        );
        let mut changed = base.clone();
        changed.runs[0]
            .entries
            .push(Entry::Notice("a notice".to_owned()));
        let mut other = SessionPage::of(&live(2, "meadow", Vec::new()));
        other.id = "0f1e2d3c-0000-4000-8000-000000000002".to_owned();
        changed.runs.push(other);
        changed.sessions.outside = 1;
        changed.notifications.read = true;
        changed.doctor.checking = true;
        let id = base.runs[0].id.clone();
        assert_eq!(
            changed.changed_paths(&base),
            [
                "/m/sessions".to_owned(),
                format!("/m/s/{id}"),
                "/m/s/0f1e2d3c-0000-4000-8000-000000000002".to_owned(),
                "/m/notifications".to_owned(),
                "/m/diagnostics".to_owned(),
            ]
        );
        assert!(base.changed_paths(&base.clone()).is_empty());
        assert!(base.session(&id.to_uppercase()).is_some(), "its case aside");
    }

    /// The sessions list (EP-31): the window's sessions, a chat's aside, with their latest whole
    /// entry; the ledger's rows, the window's own linked to its page.
    #[test]
    fn the_sessions_list_says_what_the_all_tab_says() {
        use crw_core::session::{Event, ToolUse};

        let scratch = Scratch::new("published-sessions");
        let mut board = board(&scratch);
        let mut chat = crw_core::session::Launch::new(
            "chat",
            std::path::Path::new("chat"),
            "fable",
            "What is next?",
        );
        chat.conversation = Some("What is next?".to_owned());
        // The chat's own session id, so that a row wrongly linked to it would show (the
        // review's: the canned runs share one).
        let mut chat = Live::canned(2, chat, "high", Vec::new());
        chat.session_id = "c4a7e2f0-0000-4000-8000-0000000000c4".to_owned();
        board.sessions.live.extend([
            live(
                1,
                "harbor",
                vec![
                    Event::Text("Reading the brief.\nThen the tests.".to_owned()),
                    Event::ToolUse(ToolUse {
                        id: "t1".to_owned(),
                        tool: "Bash".to_owned(),
                        summary: "Bash cargo test".to_owned(),
                    }),
                ],
            ),
            chat,
        ]);
        let id = board.sessions.live[0].session_id.clone();
        let started = Utc::now() - chrono::TimeDelta::minutes(5);
        let row = |session: &str, project: &str| SessionRow {
            id: 1,
            session_id: Some(session.to_owned()),
            project: project.to_owned(),
            ep: Some("EP-4".to_owned()),
            mode: "headless".to_owned(),
            model: Some("fable".to_owned()),
            effort: Some("high".to_owned()),
            ultracode: false,
            started_at: started.to_rfc3339(),
            ended_at: None,
            outcome: None,
            cost_usd: Some(1.5),
            questions: 0,
            result: None,
            context_percent: None,
            answer_latency_s: None,
            start: None,
            usage: crw_core::ledger::RowUsage::default(),
        };
        board.sessions.history.rows = vec![
            row(&id, "harbor"),
            row("aaaaaaaa-0000-4000-8000-000000000009", "meadow"),
            row("c4a7e2f0-0000-4000-8000-0000000000c4", "chat"),
        ];
        board.sessions.history.canned = true;
        let list = SessionList::of(&board, Utc::now());
        assert_eq!(list.window.len(), 1, "the chat's run is the Chat pane's");
        assert_eq!(list.window[0].id, id);
        assert_eq!(list.window[0].last, "Bash cargo test");
        assert_eq!(list.rows.len(), 3);
        assert_eq!(list.rows[2].page, None, "a chat's row links no page");
        assert_eq!(list.rows[0].page.as_deref(), Some(id.as_str()));
        assert_eq!(list.rows[0].state, "open: the window's own run");
        assert_eq!(list.rows[0].short, crw_core::revert::short(&id));
        assert_eq!(list.rows[0].model, "fable/high");
        assert_eq!(list.rows[0].cost, "$1.50");
        assert_eq!(list.rows[1].page, None);
        assert_eq!(list.rows[1].state, "open: no end recorded");
        // The chat has no page; the harbor run has one.
        let parts = Parts::of(&board, Utc::now());
        assert_eq!(parts.runs.len(), 1);
        assert_eq!(parts.runs[0].id, id);
        // A test's board gathers no report and keeps no ledger: the parts say so.
        assert!(!parts.doctor.live && parts.doctor.report.is_none());
    }

    /// The notifications (EP-31): the ledger's latest rows of kind `notification`, read again
    /// after their 15 s, the rows kept with why when the ledger does not read; canned ones stay.
    #[test]
    fn the_notifications_are_read_from_the_ledger() {
        let scratch = Scratch::new("published-notifications");
        let path = scratch.0.join("ledger.sqlite");
        let mut publisher = Publisher::default();
        publisher.read_notifications(Some(&path));
        assert!(publisher.notifications().read && publisher.notifications().rows.is_empty());
        let ledger = Ledger::open(&path).unwrap();
        ledger
            .add_event(
                Utc::now(),
                crw_core::notify::EVENT_KIND,
                Some("harbor"),
                "popup · finished · harbor EP-4 has finished",
            )
            .unwrap();
        ledger
            .add_event(Utc::now(), "mail", None, "not a notification")
            .unwrap();
        // Within its 15 s it is not read again; afterwards it is.
        publisher.read_notifications(Some(&path));
        assert!(publisher.notifications().rows.is_empty());
        publisher.notified = None;
        publisher.read_notifications(Some(&path));
        let rows = &publisher.notifications().rows;
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0].project.as_deref(), Some("harbor"));
        assert!(rows[0].words.ends_with("has finished") && rows[0].at.is_some());
        // No ledger: none read.
        publisher.notified = None;
        publisher.read_notifications(None);
        assert!(!publisher.notifications().read);
        // Canned rows stay as they are.
        publisher.can_notifications(vec![LogLine {
            at: None,
            project: None,
            words: "canned".to_owned(),
        }]);
        publisher.notified = None;
        publisher.read_notifications(Some(&path));
        assert_eq!(publisher.notifications().rows[0].words, "canned");
    }
}
