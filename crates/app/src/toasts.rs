//! The window's notifications (D-21; EP-15): a toast for each moment `crw_core::notify`'s rule
//! table sends at once, shown in the window's own popup (`popup`; Windows' toasts are off on the
//! owner's machine), and the badge events the tray icon marks (`tray`). The moments come from the
//! window's own runs (`sessions::News`: a request, a run's end), from the sessions Claude Code
//! reports elsewhere, told apart from one look to the next (a wait, the end of a long turn, a
//! background session's end), and from the usage meter's marks (EP-17). A toast is held back in the quiet hours and while the window is in
//! front (its status bar says it then); a session elsewhere's second wait within two minutes is a
//! repeat; one session's moments of one pass make one toast. The popup's Show brings the window or
//! the session's VS Code window forward. Each toast, toast held back, badge arrival and Show is a
//! line of the ledger's `event` table. Nothing is shown or logged before `main` turns them on:
//! never in a test or a screenshot run.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, NaiveTime, TimeDelta, Utc};
use crw_core::badges::backup::State as BackupState;
use crw_core::badges::hosting::{Hosting, Verdict};
use crw_core::ledger::{self, Ledger};
use crw_core::notify::{self, Fate, Kind, QuietHours};
use crw_core::session::Outcome;
use crw_core::sessions::{Mode, Seen, Status};
use crw_core::usage::{self, Crossing, Which};
use eframe::egui;

use crate::popup::Popup;
use crate::portfolio::Portfolio;
use crate::sessions::News;

/// A session elsewhere toasts a second wait only after this rest.
const REST: TimeDelta = TimeDelta::minutes(2);

/// The end of a session elsewhere's turn toasts when the turn worked at least this long.
const LONG_TURN: TimeDelta = TimeDelta::minutes(2);

/// The sessions Claude Code reports when the window starts are taken as they stand, until its
/// first poll answers or this long has passed.
const SETTLE: Duration = Duration::from_secs(30);

/// The badge events are read again at most this often, between reads of the portfolio.
const MARKS_EVERY: Duration = Duration::from_secs(1);

/// The longest title and line of a toast, in characters.
const TITLE: usize = 80;
const TEXT: usize = 200;

/// Where a toast's Show leads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// The window, its Decisions pane in front.
    Decisions,
    /// The window, the Sessions pane in front with a window's session's tab.
    Session(u64),
    /// The VS Code window of the project a session waits in (Focus).
    VsCode(String),
    /// The window, the Usage pane in front (EP-17).
    Usage,
    /// The window, the Sessions pane's All tab in front, where the chain log is (EP-19a).
    ChainLog,
    /// The window as it stands.
    Window,
}

/// A notification the rules send at once, or the tool's own word.
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    /// None for the tool's own words: a test, the first close to the tray.
    pub kind: Option<Kind>,
    /// What its log line calls it: the kind's word, `test` or `tray`.
    pub what: &'static str,
    pub project: Option<String>,
    pub title: String,
    pub text: String,
    pub target: Target,
    /// Shown whatever the quiet hours and the focus: the owner asked for it.
    pub forced: bool,
    /// Said in amber beyond what its kind says: a chain held or ended (EP-19a).
    pub warning: bool,
    /// Held back already, as a repeat.
    held: Option<Fate>,
}

impl Toast {
    fn new(kind: Kind, project: &str, title: String, text: String, target: Target) -> Self {
        Self {
            kind: Some(kind),
            what: kind.as_str(),
            project: Some(project.to_owned()),
            title: cut(&title, TITLE),
            text: cut(&text, TEXT),
            target,
            forced: false,
            warning: false,
            held: None,
        }
    }

    /// The owner's test of the toasts: shown whatever the hours and the focus.
    pub fn test() -> Self {
        Self {
            kind: None,
            what: "test",
            project: None,
            title: "Roadmap Wrangler".to_owned(),
            text: "A test notification: Show brings the window forward.".to_owned(),
            target: Target::Window,
            forced: true,
            warning: false,
            held: None,
        }
    }

    /// A chain's moment (D-22; EP-19a): a brief chained, the chain held or ended, from the chain
    /// log's words, which begin with what happened: the title up to the first colon, the rest
    /// the text. Its Show brings the Sessions pane's All tab forward, where the log is; a hold
    /// or an end is a `warning`.
    pub fn chain(project: &str, words: &str, warning: bool) -> Self {
        let (what, rest) = words.split_once(": ").unwrap_or((words, ""));
        let mut toast = Self::new(
            Kind::Chain,
            project,
            format!("{project}: {what}"),
            rest.to_owned(),
            Target::ChainLog,
        );
        toast.warning = warning;
        toast
    }

    /// The first close to the tray, said once (D-29).
    pub fn tray(running: usize) -> Self {
        let runs = match running {
            0 => String::new(),
            1 => " Its headless session goes on.".to_owned(),
            n => format!(" Its {n} headless sessions go on."),
        };
        Self {
            kind: None,
            what: "tray",
            project: None,
            title: "Roadmap Wrangler is still running".to_owned(),
            text: format!(
                "It is in the notification area: its icon shows the window, and its menu \
                 quits.{runs}"
            ),
            target: Target::Window,
            forced: false,
            warning: false,
            held: None,
        }
    }

    /// A usage mark reached (D-21, D-23; EP-17): the account's, so of no project; its Show brings
    /// the Usage pane forward.
    pub fn usage(crossing: &Crossing, now: DateTime<Utc>) -> Self {
        let limit = match crossing.which {
            Which::Weekly => "weekly limit",
            Which::Fable => "Fable-scoped weekly limit",
        };
        let resets = crossing.resets_at.map_or_else(
            || "Its reset time is not known.".to_owned(),
            |at| {
                format!(
                    "It resets in {}, {}.",
                    usage::countdown(at, now),
                    at.with_timezone(&Local).format("%A %H:%M")
                )
            },
        );
        Self {
            kind: Some(Kind::Usage),
            what: Kind::Usage.as_str(),
            project: None,
            title: cut(
                &format!(
                    "{} usage at {}",
                    crossing.which.name(),
                    usage::percent(crossing.percent)
                ),
                TITLE,
            ),
            text: cut(
                &format!(
                    "The {limit} is past its {} mark. {resets}",
                    usage::percent(crossing.mark)
                ),
                TEXT,
            ),
            target: Target::Usage,
            forced: false,
            warning: false,
            held: None,
        }
    }

    /// The toast in one line, for the status bar.
    pub fn words(&self) -> String {
        format!("{}: {}", self.title, self.text)
    }

    /// Whether the status bar says it in amber: a wait, a failure, a usage mark, a chain held or
    /// ended.
    pub fn is_warning(&self) -> bool {
        self.warning || matches!(self.kind, Some(Kind::Waiting | Kind::Failed | Kind::Usage))
    }

    fn line(&self, fate: &Fate) -> String {
        notify::line(fate, self.what, &self.title, &self.text)
    }
}

/// The toast a window's run's news calls for (D-21): its request, its end; none for a stop.
/// `title` is the session's, `<project> <EP>`.
pub fn from_news(key: u64, project: &str, title: &str, news: &News) -> Option<Toast> {
    let toast = match news {
        News::Asked(summary) => Toast::new(
            Kind::Waiting,
            project,
            format!("{title} waits for you"),
            summary.clone(),
            Target::Decisions,
        ),
        News::Ended(Outcome::Completed, line) => Toast::new(
            Kind::Finished,
            project,
            format!("{title} has finished"),
            if line.is_empty() {
                "Its headless run is complete.".to_owned()
            } else {
                line.clone()
            },
            Target::Session(key),
        ),
        News::Ended(Outcome::Failed, why) => Toast::new(
            Kind::Failed,
            project,
            format!("{title} has failed"),
            why.clone(),
            Target::Session(key),
        ),
        News::Ended(Outcome::Deferred, summary) => Toast::new(
            Kind::Waiting,
            project,
            format!("{title} is deferred"),
            format!("Your answer resumes it: {summary}"),
            Target::Decisions,
        ),
        News::Ended(Outcome::Stopped, _) => return None,
    };
    Some(toast)
}

/// The toasts a screenshot run's popup shows (EP-15), the oldest first: a test, a failure, an end,
/// two waits, one of them in VS Code, and a chain's end (EP-19a).
#[cfg(feature = "screenshot")]
pub fn canned() -> Vec<Toast> {
    let failed = News::Ended(Outcome::Failed, "error max budget usd".to_owned());
    let finished = News::Ended(
        Outcome::Completed,
        "EP-2 is done: the archive layout reads its folders by decade.".to_owned(),
    );
    let asked = News::Asked("Which length band should the berth filter open on?".to_owned());
    let in_vs_code = Toast::new(
        Kind::Waiting,
        "lantern",
        "lantern waits for you in VS Code".to_owned(),
        "A visual session waits for you: a permission prompt.".to_owned(),
        Target::VsCode("lantern".to_owned()),
    );
    let chain = Toast::chain(
        "orchard",
        "the chain ends after EP-6: EP-7 is a re-plan, the owner's to run",
        true,
    );
    [
        Some(Toast::test()),
        from_news(902, "compass", "compass EP-3", &failed),
        from_news(901, "quarry", "quarry EP-2", &finished),
        from_news(900, "harbor", "harbor EP-4", &asked),
        Some(in_vs_code),
        Some(chain),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// A session elsewhere as the notifications last saw it.
#[derive(Debug, Clone, PartialEq)]
struct Track {
    status: Status,
    /// Since when it has worked without a rest: busy or waiting since it was last idle.
    working_since: Option<DateTime<Utc>>,
    /// When its last wait was toasted, or held back as a repeat.
    waited_at: Option<DateTime<Utc>>,
}

/// What the sessions elsewhere did since the last look, as toasts (D-21): a wait (a repeat within
/// [`REST`] of the last), the end of a turn that worked [`LONG_TURN`] or more, a background
/// session done or failed. Left out: the window's own sessions (`own`), which their runs' news
/// cover; sessions of no registered project; another program's `-p` runs; sessions that never
/// worked. Before `settled`, each session is taken as it stands.
fn elsewhere(
    tracks: &mut HashMap<String, Track>,
    seen: &[&Seen],
    own: impl Fn(&str) -> bool,
    settled: bool,
    now: DateTime<Utc>,
) -> Vec<Toast> {
    let mut toasts = Vec::new();
    let mut present = BTreeSet::new();
    for seen in seen {
        let Some(project) = seen.project.as_deref() else {
            continue;
        };
        if own(&seen.session_id) || seen.mode == Mode::Headless || seen.is_new() {
            continue;
        }
        present.insert(seen.session_id.clone());
        let working = matches!(seen.status, Status::Busy | Status::Waiting);
        let first = !tracks.contains_key(&seen.session_id);
        // A session that appears once the window has settled starts from a rest.
        let track = tracks
            .entry(seen.session_id.clone())
            .or_insert_with(|| Track {
                status: if settled { Status::Idle } else { seen.status },
                working_since: working.then_some(now),
                waited_at: None,
            });
        if first && !settled {
            continue;
        }
        toasts.extend(change(track, seen, project, now));
    }
    tracks.retain(|id, _| present.contains(id));
    toasts
}

/// The toast a session elsewhere's new status calls for, and its track brought up to date.
fn change(track: &mut Track, seen: &Seen, project: &str, now: DateTime<Utc>) -> Option<Toast> {
    let was = std::mem::replace(&mut track.status, seen.status);
    let in_vs_code = seen
        .entrypoint
        .as_deref()
        .is_some_and(|surface| surface.contains("vscode"));
    let mode = seen.mode.as_str();
    match seen.status {
        Status::Busy | Status::Waiting if track.working_since.is_none() => {
            track.working_since = Some(now);
        }
        _ => {}
    }
    match (was, seen.status) {
        (Status::Waiting, Status::Waiting) | (_, Status::Busy) => None,
        (_, Status::Waiting) => {
            let repeat = track.waited_at.is_some_and(|at| now - at < REST);
            track.waited_at = Some(now);
            let (title, target) = if in_vs_code {
                (
                    format!("{project} waits for you in VS Code"),
                    Target::VsCode(project.to_owned()),
                )
            } else {
                (format!("{project} waits for you"), Target::Decisions)
            };
            let mut toast = Toast::new(
                Kind::Waiting,
                project,
                title,
                // The poll says what a session waits for in Claude Code's own words ("input
                // needed"), the hooks as a thing ("a permission prompt"): both read after a colon.
                match &seen.waiting_for {
                    Some(what) => format!("A {mode} session waits for you: {what}."),
                    None => format!("A {mode} session waits for you."),
                },
                target,
            );
            toast.held = repeat.then_some(Fate::Repeat);
            Some(toast)
        }
        (Status::Busy | Status::Waiting, Status::Idle) => {
            let since = track.working_since.take()?;
            (now - since >= LONG_TURN).then(|| {
                Toast::new(
                    Kind::Finished,
                    project,
                    format!("{project}: a session's turn is over"),
                    format!(
                        "A {mode} session worked {} and waits for a prompt.",
                        ledger::length(now - since)
                    ),
                    if in_vs_code {
                        Target::VsCode(project.to_owned())
                    } else {
                        Target::Window
                    },
                )
            })
        }
        (_, Status::Idle) => {
            track.working_since = None;
            None
        }
        (was, Status::Failed) if was != Status::Failed => Some(Toast::new(
            Kind::Failed,
            project,
            format!("{project}: a session has failed"),
            format!("A {} session failed.", seen.kind.as_deref().unwrap_or(mode)),
            Target::Window,
        )),
        // A background session's end is its work done; an interactive one was closed by hand.
        (was, Status::Done) if !was.is_over() && seen.kind.as_deref() == Some("background") => {
            Some(Toast::new(
                Kind::Finished,
                project,
                format!("{project}: a background session is done"),
                seen.name.clone().unwrap_or_default(),
                Target::Window,
            ))
        }
        (_, Status::Failed | Status::Done) => None,
    }
}

/// One toast for the moments of one pass that share a kind and a target: the first, saying how
/// many more there were. A toast held back stays apart, and so does a usage mark.
fn coalesce(toasts: Vec<Toast>) -> Vec<Toast> {
    let mut merged: Vec<(Toast, usize)> = Vec::new();
    for toast in toasts {
        // Each usage mark names its own limit (EP-17).
        let same = merged.iter_mut().find(|(first, _)| {
            first.held.is_none()
                && toast.held.is_none()
                && toast.kind != Some(Kind::Usage)
                && first.kind == toast.kind
                && first.target == toast.target
                && first.project == toast.project
        });
        match same {
            Some((_, more)) => *more += 1,
            None => merged.push((toast, 0)),
        }
    }
    merged
        .into_iter()
        .map(|(mut toast, more)| {
            if more > 0 {
                toast.text = cut(&format!("{} (and {more} more)", toast.text), TEXT);
            }
            toast
        })
        .collect()
}

/// Why a toast is held back, if it is (D-21): a repeat, the quiet hours at `at` in local time,
/// the window in front. One the owner asked for is never held back.
fn held(toast: &Toast, quiet: Option<QuietHours>, at: NaiveTime, in_front: bool) -> Option<Fate> {
    if toast.forced {
        return None;
    }
    if let Some(fate) = &toast.held {
        return Some(fate.clone());
    }
    if quiet.is_some_and(|hours| hours.contains(at)) {
        return Some(Fate::Quiet);
    }
    (in_front && toast.kind.is_some()).then_some(Fate::InFront)
}

/// A badge event (D-21): what only wants a look, by its kind and what it concerns.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mark {
    pub kind: Kind,
    pub project: Option<String>,
    /// The folder, the message's id, or the state in words.
    pub what: String,
}

impl Mark {
    fn new(kind: Kind, project: Option<&str>, what: impl Into<String>) -> Self {
        Self {
            kind,
            project: project.map(str::to_owned),
            what: what.into(),
        }
    }

    /// The mark in words: `harbor IN-3`, `harbor: backup stale`, `a new folder notes`.
    fn words(&self) -> String {
        match (self.kind, &self.project) {
            (Kind::Inbox, Some(project)) => format!("{project} {}", self.what),
            (_, Some(project)) => format!("{project}: {} {}", self.kind.as_str(), self.what),
            (_, None) => format!("a new folder {}", self.what),
        }
    }
}

/// The badge events the portfolio holds as of `now` (D-21): each unadopted folder; a project's
/// orphan or missing repository, or an origin a current head check finds diverged or ahead with
/// commits never fetched; a stale, failing or uncovered backup; each open inbox message.
pub fn marks(portfolio: &Portfolio, now: DateTime<Utc>) -> BTreeSet<Mark> {
    let mut marks = BTreeSet::new();
    for folder in &portfolio.unadopted {
        marks.insert(Mark::new(Kind::Unadopted, None, folder.name.clone()));
    }
    for row in &portfolio.projects {
        let project = Some(row.name());
        match &row.badges.hosting {
            Hosting::Orphan(_) => {
                marks.insert(Mark::new(Kind::Remote, project, "orphan"));
            }
            Hosting::Missing(_) => {
                marks.insert(Mark::new(Kind::Remote, project, "missing"));
            }
            _ => {}
        }
        if row.badges.head_is_current()
            && let Some(head) = &row.badges.head
        {
            let verdict = match head.value.verdict {
                Verdict::Diverged { .. } => Some("diverged"),
                Verdict::NotFetched => Some("not fetched"),
                _ => None,
            };
            if let Some(verdict) = verdict {
                marks.insert(Mark::new(Kind::Remote, project, verdict));
            }
        }
        let backup = row.badges.backup.as_ref().map(|backup| backup.state(now));
        let state = match backup {
            Some(BackupState::Stale) => Some("stale"),
            Some(BackupState::Failing) => Some("failing"),
            Some(BackupState::NotCovered) => Some("not covered"),
            _ => None,
        };
        if let Some(state) = state {
            marks.insert(Mark::new(Kind::Backup, project, state));
        }
        if let Some(inbox) = &row.snapshot.inbox {
            for message in inbox.open() {
                marks.insert(Mark::new(Kind::Inbox, project, message.id.clone()));
            }
        }
    }
    marks
}

/// The badge events as the owner last saw them: those not seen are new, and mark the tray icon.
#[derive(Debug, Default)]
pub struct Attention {
    /// What the window held the last time it was in front, of what still stands.
    seen: BTreeSet<Mark>,
    /// The badge events as the portfolio stands.
    current: BTreeSet<Mark>,
    /// The first reading is taken as seen.
    settled: bool,
}

impl Attention {
    /// Takes in the badge events as they stand; `in_front` means the owner sees them. Returns
    /// those that have just arrived, for the log.
    fn update(&mut self, current: BTreeSet<Mark>, in_front: bool) -> Vec<Mark> {
        if !std::mem::replace(&mut self.settled, true) {
            self.seen = current.clone();
            self.current = current;
            return Vec::new();
        }
        let arrived: Vec<Mark> = current
            .iter()
            .filter(|mark| !self.current.contains(*mark) && !self.seen.contains(*mark))
            .cloned()
            .collect();
        self.current = current;
        if in_front {
            self.seen = self.current.clone();
        } else {
            let current = &self.current;
            self.seen.retain(|mark| current.contains(mark));
        }
        arrived
    }

    /// The badge events new since the owner looked.
    pub fn unseen(&self) -> Vec<&Mark> {
        self.current
            .iter()
            .filter(|mark| !self.seen.contains(*mark))
            .collect()
    }

    /// The new ones counted by kind, in words: `1 inbox message, 2 unadopted folders`.
    pub fn counted(&self) -> String {
        let mut counts: BTreeMap<Kind, usize> = BTreeMap::new();
        for mark in self.unseen() {
            *counts.entry(mark.kind).or_default() += 1;
        }
        counts
            .into_iter()
            .map(|(kind, n)| {
                let (one, many) = match kind {
                    Kind::Unadopted => ("unadopted folder", "unadopted folders"),
                    Kind::Remote => ("remote to look at", "remotes to look at"),
                    Kind::Backup => ("backup to look at", "backups to look at"),
                    Kind::Inbox => ("inbox message", "inbox messages"),
                    Kind::Waiting | Kind::Finished | Kind::Failed | Kind::Usage | Kind::Chain => {
                        ("moment", "moments")
                    }
                };
                format!("{n} {}", if n == 1 { one } else { many })
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The window's notifications.
pub struct Toasts {
    /// Shown and logged: turned on by `main` only.
    on: bool,
    ledger: Option<PathBuf>,
    started: Instant,
    tracks: HashMap<String, Track>,
    pub attention: Attention,
    marks_at: Option<Instant>,
    /// The toasts shown, until the owner dismisses them or acts on them.
    pub popup: Popup,
    /// A log line the ledger did not take, until the next one goes well.
    pub problem: Option<String>,
    /// How many toasts the popup has shown since the window started.
    pub shown: usize,
}

impl Default for Toasts {
    fn default() -> Self {
        Self {
            on: false,
            ledger: None,
            started: Instant::now(),
            tracks: HashMap::new(),
            attention: Attention::default(),
            marks_at: None,
            popup: Popup::default(),
            problem: None,
            shown: 0,
        }
    }
}

impl Toasts {
    /// Turns the toasts and their log on, the log in `ledger`.
    pub fn start(&mut self, ledger: Option<PathBuf>) {
        self.on = true;
        self.ledger = ledger;
        self.started = Instant::now();
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    /// What the sessions elsewhere did since the last look (`seen`, the state model's), as
    /// toasts; settled once the poll has answered (`polled`) or the window has run a while.
    pub fn elsewhere(
        &mut self,
        seen: &[&Seen],
        own: impl Fn(&str) -> bool,
        polled: bool,
    ) -> Vec<Toast> {
        let settled = polled || self.started.elapsed() >= SETTLE;
        elsewhere(&mut self.tracks, seen, own, settled, Utc::now())
    }

    /// Whether the badge events are due to be read again.
    pub fn marks_due(&self) -> bool {
        self.on && self.marks_at.is_none_or(|at| at.elapsed() >= MARKS_EVERY)
    }

    /// Takes in the badge events as they stand, the arrivals logged.
    pub fn mark(&mut self, marks: BTreeSet<Mark>, in_front: bool) {
        self.marks_at = Some(Instant::now());
        for mark in self.attention.update(marks, in_front) {
            let line = notify::line(&Fate::Badge, mark.kind.as_str(), &mark.words(), "");
            self.log(mark.project.as_deref(), &line);
        }
    }

    /// Shows this pass's toasts in the popup, or holds them back (`quiet`, `in_front`), each
    /// logged. Returns those held back because the window is in front, for its status bar to say.
    pub fn deliver(
        &mut self,
        toasts: Vec<Toast>,
        quiet: Option<QuietHours>,
        in_front: bool,
        ctx: &egui::Context,
    ) -> Vec<Toast> {
        if !self.on {
            return Vec::new();
        }
        let at = Local::now().time();
        let mut in_view = Vec::new();
        for toast in coalesce(toasts) {
            let fate = held(&toast, quiet, at, in_front);
            self.log(
                toast.project.as_deref(),
                &toast.line(fate.as_ref().unwrap_or(&Fate::Shown)),
            );
            match fate {
                None => {
                    self.shown += 1;
                    self.popup.push(toast);
                    ctx.request_repaint();
                }
                Some(Fate::InFront) => in_view.push(toast),
                Some(_) => {}
            }
        }
        in_view
    }

    /// The toasts the owner asked the popup to show since the last frame, each logged.
    pub fn chosen(&mut self, ctx: &egui::Context) -> Vec<Target> {
        let chosen = self.popup.ui(ctx);
        chosen
            .into_iter()
            .map(|toast| {
                self.log(toast.project.as_deref(), &toast.line(&Fate::Clicked));
                toast.target
            })
            .collect()
    }

    fn log(&mut self, project: Option<&str>, line: &str) {
        let Some(path) = self.ledger.as_deref().filter(|_| self.on) else {
            return;
        };
        let added = Ledger::open(path)
            .and_then(|ledger| ledger.add_event(Utc::now(), notify::EVENT_KIND, project, line));
        if let Err(e) = added {
            self.problem = Some(format!("The ledger did not take a notification: {e}"));
        }
    }

    /// The notifications in words, for the status bar's hover.
    pub fn line(&self, quiet: Option<QuietHours>) -> String {
        if !self.on {
            return "Notifications: off (a screenshot run, or not started)".to_owned();
        }
        let hours = match quiet {
            Some(hours) if hours.contains(Local::now().time()) => {
                format!("quiet hours {hours}, quiet now")
            }
            Some(hours) => format!("quiet hours {hours}"),
            None => "no quiet hours".to_owned(),
        };
        let mut line = format!(
            "Notifications: the popup, {hours}; {} shown, {} in the popup",
            self.shown,
            self.popup.len()
        );
        if let Some(problem) = &self.problem {
            line.push_str(&format!("; {problem}"));
        }
        line
    }
}

/// `text` on one line, cut to `max` characters with an ellipsis.
fn cut(text: &str, max: usize) -> String {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.chars().count() <= max {
        return line;
    }
    let kept: String = line.chars().take(max - 1).collect();
    format!("{}…", kept.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-03T20:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn seen(id: &str, project: Option<&str>, mode: Mode, status: Status) -> Seen {
        let mut seen = Seen::canned(id, project, mode, status, None, now());
        seen.entrypoint = Some("claude-vscode".to_owned());
        seen
    }

    fn look(
        tracks: &mut HashMap<String, Track>,
        sessions: &[Seen],
        at: DateTime<Utc>,
    ) -> Vec<Toast> {
        let refs: Vec<&Seen> = sessions.iter().collect();
        elsewhere(tracks, &refs, |id| id == "own", true, at)
    }

    #[test]
    fn a_runs_news_makes_its_toast() {
        let asked = from_news(
            7,
            "harbor",
            "harbor EP-4",
            &News::Asked("Which colour?".into()),
        );
        let asked = asked.unwrap();
        assert_eq!(
            (asked.kind, asked.title.as_str(), asked.text.as_str()),
            (
                Some(Kind::Waiting),
                "harbor EP-4 waits for you",
                "Which colour?"
            )
        );
        assert_eq!(asked.target, Target::Decisions);
        let done = from_news(
            7,
            "harbor",
            "harbor EP-4",
            &News::Ended(Outcome::Completed, String::new()),
        );
        let done = done.unwrap();
        assert_eq!(done.kind, Some(Kind::Finished));
        assert_eq!(done.text, "Its headless run is complete.");
        assert_eq!(done.target, Target::Session(7));
        let failed = from_news(
            7,
            "harbor",
            "harbor EP-4",
            &News::Ended(Outcome::Failed, "exit 1".into()),
        );
        assert_eq!(failed.unwrap().title, "harbor EP-4 has failed");
        let deferred = from_news(
            7,
            "harbor",
            "harbor EP-4",
            &News::Ended(Outcome::Deferred, "Ship?".into()),
        );
        let deferred = deferred.unwrap();
        assert_eq!(deferred.kind, Some(Kind::Waiting));
        assert_eq!(deferred.text, "Your answer resumes it: Ship?");
        assert_eq!(
            from_news(
                7,
                "harbor",
                "harbor EP-4",
                &News::Ended(Outcome::Stopped, String::new())
            ),
            None
        );
    }

    #[test]
    fn a_wait_elsewhere_toasts_once_then_rests() {
        let mut tracks = HashMap::new();
        let mut lantern = seen("a1", Some("lantern"), Mode::Visual, Status::Busy);
        assert!(look(&mut tracks, &[lantern.clone()], now()).is_empty());
        lantern.status = Status::Waiting;
        lantern.waiting_for = Some("a permission prompt".to_owned());
        let toasts = look(&mut tracks, &[lantern.clone()], now());
        assert_eq!(toasts.len(), 1);
        assert_eq!(toasts[0].title, "lantern waits for you in VS Code");
        assert_eq!(
            toasts[0].text,
            "A visual session waits for you: a permission prompt."
        );
        assert_eq!(toasts[0].target, Target::VsCode("lantern".to_owned()));
        assert_eq!(toasts[0].held, None);
        // Still waiting: nothing more. Busy, then waiting again within the rest: a repeat.
        assert!(look(&mut tracks, &[lantern.clone()], now()).is_empty());
        lantern.status = Status::Busy;
        look(&mut tracks, &[lantern.clone()], now());
        lantern.status = Status::Waiting;
        let again = look(
            &mut tracks,
            &[lantern.clone()],
            now() + TimeDelta::seconds(30),
        );
        assert_eq!(again[0].held, Some(Fate::Repeat));
        // After the rest, a wait toasts again.
        lantern.status = Status::Busy;
        look(
            &mut tracks,
            &[lantern.clone()],
            now() + TimeDelta::minutes(5),
        );
        lantern.status = Status::Waiting;
        let later = look(&mut tracks, &[lantern], now() + TimeDelta::minutes(5));
        assert_eq!(later[0].held, None);
    }

    #[test]
    fn a_long_turn_elsewhere_toasts_its_end_and_a_short_one_does_not() {
        let mut tracks = HashMap::new();
        let mut orchard = seen("b1", Some("orchard"), Mode::Manual, Status::Idle);
        orchard.entrypoint = Some("cli".to_owned());
        look(&mut tracks, &[orchard.clone()], now());
        orchard.status = Status::Busy;
        look(&mut tracks, &[orchard.clone()], now());
        orchard.status = Status::Idle;
        assert!(
            look(
                &mut tracks,
                &[orchard.clone()],
                now() + TimeDelta::seconds(50)
            )
            .is_empty()
        );
        orchard.status = Status::Busy;
        look(
            &mut tracks,
            &[orchard.clone()],
            now() + TimeDelta::minutes(1),
        );
        orchard.status = Status::Waiting;
        look(
            &mut tracks,
            &[orchard.clone()],
            now() + TimeDelta::minutes(3),
        );
        orchard.status = Status::Idle;
        let ended = look(&mut tracks, &[orchard], now() + TimeDelta::minutes(43));
        assert_eq!(ended.len(), 1);
        assert_eq!(ended[0].kind, Some(Kind::Finished));
        assert_eq!(ended[0].title, "orchard: a session's turn is over");
        assert_eq!(
            ended[0].text,
            "A manual session worked 42 min and waits for a prompt."
        );
        assert_eq!(ended[0].target, Target::Window);
    }

    #[test]
    fn ends_by_hand_and_the_windows_own_sessions_toast_nothing() {
        let mut tracks = HashMap::new();
        let closed = seen("c1", Some("lantern"), Mode::Visual, Status::Idle);
        let own = seen("own", Some("harbor"), Mode::Visual, Status::Idle);
        // A tab the hooks saw open that never worked, one outside the registry, another
        // program's `-p` run.
        let mut fresh = seen("c2", Some("beacon"), Mode::Manual, Status::Idle);
        fresh.worked = false;
        let outside = seen("c3", None, Mode::Manual, Status::Idle);
        let other = seen("c4", Some("quarry"), Mode::Headless, Status::Idle);
        let mut sessions = vec![closed, own, fresh, outside, other];
        look(&mut tracks, &sessions, now());
        for session in &mut sessions {
            session.status = Status::Waiting;
        }
        let toasts = look(&mut tracks, &sessions, now());
        let projects: Vec<_> = toasts.iter().filter_map(|t| t.project.as_deref()).collect();
        assert_eq!(projects, ["lantern"]);
        // Closed by hand: no toast.
        sessions[0].status = Status::Done;
        assert!(look(&mut tracks, &sessions[..1], now()).is_empty());
        // A background session's end and failure toast.
        let mut background = seen("d1", Some("quarry"), Mode::Manual, Status::Busy);
        background.kind = Some("background".to_owned());
        background.name = Some("nightly checks".to_owned());
        look(&mut tracks, &[background.clone()], now());
        background.status = Status::Done;
        let done = look(&mut tracks, &[background.clone()], now());
        assert_eq!(done[0].title, "quarry: a background session is done");
        assert_eq!(done[0].text, "nightly checks");
        background.status = Status::Failed;
        let failed = look(&mut tracks, &[background], now());
        assert_eq!(failed[0].kind, Some(Kind::Failed));
    }

    #[test]
    fn the_sessions_at_the_start_are_taken_as_they_stand() {
        let mut tracks = HashMap::new();
        let waiting = seen("e1", Some("lantern"), Mode::Visual, Status::Waiting);
        let refs = [&waiting];
        assert!(elsewhere(&mut tracks, &refs, |_| false, false, now()).is_empty());
        assert!(elsewhere(&mut tracks, &refs, |_| false, true, now()).is_empty());
        // A session first seen once settled starts from a rest: its wait toasts.
        let late = seen("e2", Some("orchard"), Mode::Manual, Status::Waiting);
        let toasts = elsewhere(&mut tracks, &[&waiting, &late], |_| false, true, now());
        assert_eq!(toasts.len(), 1);
        assert_eq!(toasts[0].project.as_deref(), Some("orchard"));
        // A session gone is forgotten.
        elsewhere(&mut tracks, &[&late], |_| false, true, now());
        assert_eq!(tracks.len(), 1);
    }

    #[test]
    fn a_toast_is_held_back_by_the_hours_and_the_focus_but_not_when_asked_for() {
        let night = QuietHours::parse("22:00-07:00").ok();
        let toast = from_news(1, "harbor", "harbor EP-4", &News::Asked("Q?".into())).unwrap();
        let at = |text| NaiveTime::parse_from_str(text, "%H:%M").unwrap();
        assert_eq!(held(&toast, night, at("23:00"), false), Some(Fate::Quiet));
        assert_eq!(held(&toast, night, at("12:00"), false), None);
        assert_eq!(held(&toast, night, at("12:00"), true), Some(Fate::InFront));
        assert_eq!(held(&Toast::test(), night, at("23:00"), true), None);
        // The tool's own word ignores the focus (the window has just gone) but keeps the hours.
        assert_eq!(held(&Toast::tray(0), None, at("12:00"), true), None);
        assert_eq!(
            held(&Toast::tray(0), night, at("23:00"), false),
            Some(Fate::Quiet)
        );
        let mut repeat = toast.clone();
        repeat.held = Some(Fate::Repeat);
        assert_eq!(held(&repeat, None, at("12:00"), false), Some(Fate::Repeat));
    }

    #[test]
    fn one_sessions_moments_of_one_pass_make_one_toast() {
        let one = from_news(1, "harbor", "harbor EP-4", &News::Asked("First?".into())).unwrap();
        let two = from_news(1, "harbor", "harbor EP-4", &News::Asked("Second?".into())).unwrap();
        let other = from_news(2, "quarry", "quarry EP-2", &News::Asked("Third?".into())).unwrap();
        let merged = coalesce(vec![one, two, other]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].text, "First? (and 1 more)");
        assert_eq!(merged[1].text, "Third?");
    }

    #[test]
    fn badge_events_are_new_until_the_window_is_in_front() {
        let mark = |kind, project: Option<&str>, what: &str| Mark::new(kind, project, what);
        let mut attention = Attention::default();
        let first: BTreeSet<Mark> = [mark(Kind::Inbox, Some("harbor"), "IN-1")].into();
        // The first reading is the baseline.
        assert!(attention.update(first.clone(), false).is_empty());
        assert!(attention.unseen().is_empty());
        let mut second = first.clone();
        second.insert(mark(Kind::Inbox, Some("harbor"), "IN-2"));
        second.insert(mark(Kind::Unadopted, None, "notes"));
        second.insert(mark(Kind::Backup, Some("quarry"), "stale"));
        let arrived = attention.update(second.clone(), false);
        assert_eq!(arrived.len(), 3);
        assert_eq!(
            attention.counted(),
            "1 unadopted folder, 1 backup to look at, 1 inbox message"
        );
        // Read again while away: no new arrival, still new.
        assert!(attention.update(second.clone(), false).is_empty());
        assert_eq!(attention.unseen().len(), 3);
        // The owner looks: seen.
        attention.update(second.clone(), true);
        assert!(attention.unseen().is_empty() && attention.counted().is_empty());
        // One goes and comes back while away: it arrives again.
        let mut gone = second.clone();
        gone.remove(&mark(Kind::Backup, Some("quarry"), "stale"));
        attention.update(gone, false);
        assert_eq!(attention.update(second, false).len(), 1);
        assert_eq!(
            mark(Kind::Inbox, Some("harbor"), "IN-2").words(),
            "harbor IN-2"
        );
        assert_eq!(
            mark(Kind::Backup, Some("quarry"), "stale").words(),
            "quarry: backup stale"
        );
        assert_eq!(
            mark(Kind::Unadopted, None, "notes").words(),
            "a new folder notes"
        );
    }

    #[test]
    fn a_toasts_words_keep_one_line() {
        assert_eq!(cut("two\nlines  here", 40), "two lines here");
        let long = cut(&"word ".repeat(80), TEXT);
        assert_eq!(long.chars().count(), TEXT);
        assert!(long.ends_with('…'));
        let toast = Toast::tray(2);
        assert!(
            toast.text.ends_with("Its 2 headless sessions go on."),
            "{}",
            toast.text
        );
    }

    #[test]
    fn a_usage_mark_toasts_the_account_and_leads_to_the_usage_pane() {
        let now = Utc::now();
        let crossing = Crossing {
            which: Which::Fable,
            mark: usage::ALARM,
            percent: 96.4,
            resets_at: Some(now + TimeDelta::hours(30)),
        };
        let toast = Toast::usage(&crossing, now);
        assert_eq!(toast.title, "Fable usage at 96 %");
        assert!(
            toast.text.starts_with(
                "The Fable-scoped weekly limit is past its 95 % mark. It resets in 1 d 6 h,"
            ),
            "{}",
            toast.text
        );
        assert_eq!(
            (toast.kind, toast.project.as_deref()),
            (Some(Kind::Usage), None)
        );
        assert_eq!(toast.target, Target::Usage);
        assert!(toast.is_warning());
        // A chain's moment (EP-19a): the log's words split at their first colon, the Show to the
        // chain log; a chained start plain, a hold or an end in amber.
        let chained = Toast::chain(
            "harbor",
            "chained EP-5 after EP-4: claude-opus-5-5 at max, your choice, M's cap of 1 h 30 min",
            false,
        );
        assert_eq!(chained.title, "harbor: chained EP-5 after EP-4");
        assert_eq!(
            chained.text,
            "claude-opus-5-5 at max, your choice, M's cap of 1 h 30 min"
        );
        assert_eq!(
            (chained.kind, chained.target.clone(), chained.what),
            (Some(Kind::Chain), Target::ChainLog, "chain")
        );
        assert!(!chained.is_warning());
        let ended = Toast::chain(
            "harbor",
            "the chain ends after EP-5: EP-6 is a re-plan",
            true,
        );
        assert_eq!(ended.title, "harbor: the chain ends after EP-5");
        assert!(ended.is_warning());
        assert_eq!(
            ended.line(&Fate::Shown),
            "popup · chain · harbor: the chain ends after EP-5 · EP-6 is a re-plan"
        );
        let bare = Toast::chain("harbor", "turned on by the owner", false);
        assert_eq!(
            (bare.title.as_str(), bare.text.as_str()),
            ("harbor: turned on by the owner", "")
        );
        // Weekly and Fable reached in one pass make two toasts, not one with "and 1 more".
        let weekly = Toast::usage(
            &Crossing {
                which: Which::Weekly,
                ..crossing.clone()
            },
            now,
        );
        assert_eq!(coalesce(vec![weekly, toast]).len(), 2);
    }
}
