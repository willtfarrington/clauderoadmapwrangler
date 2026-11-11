//! The window (D-25): a menu bar, the docking layout of panes over one shared [`Board`], and a
//! status bar. The panes queue what the owner asks for as [`Action`]s, which run after the frame
//! is drawn: the writes the tool owns (D-7, D-8, D-10, D-11, D-13), each followed by a new read of
//! the files, the moves of the selection, and from EP-10 the launch dialog and the headless
//! sessions it starts (D-16, D-17, D-22), from EP-11 its visual launches; from EP-12 the sessions
//! the hooks and the poll of live sessions report join the window's own (D-20); from EP-13 an
//! ended headless session resumes in its tab (D-16); from EP-14 the owner answers what a session
//! asks, a deferred run resumes with the answer, and Focus brings a waiting session's VS Code
//! window forward (D-19); from EP-15 the notifications (D-21): the popup, the tray icon with its
//! menu, closing to the tray and Quit, whose share of each pass runs while the window is hidden
//! too (`attend`, from eframe's `App::logic`, D-29); from EP-15c a session's revert, previewed
//! in a dialog and run on a worker thread (`revert`, D-11), which Quit waits for; from EP-17 the
//! usage meter (`meter`, D-23): its strip in the status bar, its pane and its notifications; from
//! EP-37 File's Install and restart (`restart`, D-29), the restart into the installed build.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crw_core::badges::hosting::Class;
use crw_core::badges::short_age;
use crw_core::badges::stack::Kind;
use crw_core::catalog::Catalog;
use crw_core::discovery::{self, Listing, OwnFile, OwnFiles};
use crw_core::ledger::{Ledger, Part};
use crw_core::notify::QuietHours;
use crw_core::proposal::{self, Reason};
use crw_core::registry::{LaunchMode, Limit, State, Store};
use crw_core::roadmap::snapshot::InProgress;
use crw_core::session::{Response, Route, cli};
use crw_core::sessions::Registered;
use crw_core::usage::share;
use crw_core::{files, inbox, paths, prompt, revert, rules, tasks};
use eframe::egui::{self, Key, KeyboardShortcut, Modifiers, RichText, ThemePreference};
use egui_tiles::Tree;
use serde::{Deserialize, Serialize};

use crate::crash;
use crate::launch::{Dialog, Opening};
use crate::layout::{self, Pane};
use crate::meter::Meter;
use crate::portfolio::{Filter, GroupKind, Loader, Portfolio, Selection, SortKey, step};
use crate::restart::Standing;
use crate::sessions::{News, SessionFilter, Sessions, Started};
use crate::style::{self, Density, icons};
use crate::toasts::{self, Target, Toast, Toasts};
use crate::tray::{self, Order, Tray};
use crate::visual::{self, Visual};
use crate::watch::Watch;

const THEME_KEY: &str = "theme";
const VIEW_KEY: &str = "view";
/// Whether the first close to the tray has been said (EP-15).
const TRAY_KEY: &str = "tray_told";
/// A notice stays this long in the status bar; an error stays until the next notice.
const NOTICE_FOR: Duration = Duration::from_secs(8);
/// The most a notice or an error says in the status bar, on one line (EP-14a).
const NOTICE_CHARS: usize = 160;
/// The most of an answer's words a notice repeats, so that the line's end still shows.
const ANSWER_CHARS: usize = 80;
/// The most of a brief's title a Start item in the tray's menu shows (EP-15).
const START_TITLE: usize = 48;
/// How long the delayed test notification waits (EP-15d).
const TEST_DELAY: Duration = Duration::from_secs(10);
/// What quitting says while a revert runs (EP-15c).
const REVERTING: &str = "A revert is running: quit once it has finished, in a few seconds.";
/// Folders that appear or go in a burst (a clone, a move) are read once, after the burst.
const SETTLE: Duration = Duration::from_millis(600);
/// An idle window wakes this often to see whether the hourly look at the GitHub cache is due.
const ONLINE_WAKE: Duration = Duration::from_secs(600);
const REFRESH: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F5);

/// What the list shows and how, saved with the layout (D-25).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct View {
    pub sort: SortKey,
    pub density: Density,
    pub filter: String,
    /// Folded groups, by [`GroupKind::name`].
    pub folded: BTreeSet<String>,
    /// Projects whose rows are expanded inline, by name.
    pub expanded: BTreeSet<String>,
    pub selected: Option<Selection>,
    /// The stack filter: projects with any of these stacks (EP-7).
    pub stacks: BTreeSet<Kind>,
    /// The hosting filter: projects in any of these classes (EP-7).
    pub hosting: BTreeSet<Class>,
    /// The key to the row's marks, open at the foot of the Projects pane (EP-7a).
    pub legend: bool,
    /// What the Sessions pane's All tab lists (EP-13).
    pub sessions: SessionFilter,
}

impl Default for View {
    /// Finished work folded away at first.
    fn default() -> Self {
        Self {
            sort: SortKey::default(),
            density: Density::default(),
            filter: String::new(),
            folded: ["done", "archived"].map(str::to_owned).into(),
            expanded: BTreeSet::new(),
            selected: None,
            stacks: BTreeSet::new(),
            hosting: BTreeSet::new(),
            legend: false,
            sessions: SessionFilter::default(),
        }
    }
}

impl View {
    pub fn is_folded(&self, kind: GroupKind) -> bool {
        self.folded.contains(kind.name())
    }

    /// The name, stack and hosting filters as the list applies them.
    pub fn list_filter(&self) -> Filter<'_> {
        Filter {
            name: &self.filter,
            stacks: &self.stacks,
            hosting: &self.hosting,
        }
    }
}

/// Where the window's data comes from: the registry's store, the ledger the GitHub readings are
/// cached in, whether the GitHub pass may ask the network (a screenshot run's never does), and the
/// crash log whose last report the status bar names once (EP-14a).
pub struct Places {
    pub store: Store,
    pub ledger: Option<PathBuf>,
    pub online: bool,
    pub crash_log: Option<PathBuf>,
}

/// What the owner asked for in a pane, run after the frame.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Select(Selection),
    /// Selects and brings the Detail pane forward (Enter, a double click).
    Open(Selection),
    ToggleExpanded(String),
    ToggleGroup(GroupKind),
    Adopt(String),
    Ignore(String),
    SetState(String, State),
    /// A proposal left out until the fact it rests on changes (D-6 addendum).
    Dismiss(String, Reason),
    /// A dismissal taken back.
    Restore(String, proposal::Kind),
    SetPinned(String, bool),
    SetPriority(String, u8),
    Tick {
        project: String,
        id: String,
    },
    Untick {
        project: String,
        id: String,
    },
    /// An inbox message, with the disposition typed for it (D-13).
    TickMessage {
        project: String,
        id: String,
    },
    /// The compose dialog on a project (D-13; EP-20c): the Detail pane's Compose…, a row's
    /// Send a message….
    OpenCompose(String),
    /// The dialog's Send: the message written into the project's inbox.
    Send,
    CloseCompose,
    Refresh,
    /// The launch dialog on the project's next brief (S, a row's Start button).
    OpenLaunch(String),
    CloseLaunch,
    /// The dialog's launch, started headless (D-16).
    StartHeadless,
    /// The dialog's launch, its prompt filled into a Claude Code tab in VS Code (D-16, EP-11).
    StartVisual,
    /// The window's sessions, by their number.
    StopSession(u64),
    KillSession(u64),
    DismissSession(u64),
    /// An ended session run again under its id with the message typed in its tab (EP-13).
    ResumeSession(u64),
    /// The owner's answer to a request a window's session waits on (D-19; EP-14).
    Respond {
        key: u64,
        request: String,
        response: Response,
    },
    /// A deferred run's open request answered: the session resumes with the answer (EP-14).
    AnswerDeferred {
        key: u64,
        response: Response,
    },
    /// The VS Code window of the project a session waits in, brought forward (D-19).
    Focus(String),
    /// A ledger row's session opened in a Claude Code tab of its project's VS Code window, through
    /// the extension's `session=<id>` link (D-16; EP-35).
    ResumeInCode {
        project: String,
        session: String,
        ep: Option<String>,
    },
    /// A toast shown whatever the quiet hours and the focus, so that the owner sees toasts reach
    /// the screen (EP-15).
    TestToast,
    /// The same in ten seconds, time to close the window to the tray first: the path of a toast
    /// that comes while the window is hidden (EP-15d).
    TestToastLater,
    /// The window hidden in the tray, as closing it does (D-29).
    HideWindow,
    /// A newer build of the window's repository installed, and the installed window started in
    /// place of this one, unless something would be lost (EP-37): File's item, the tray's.
    InstallAndRestart,
    /// Quit: at once, or after a question when headless runs would stop (EP-15).
    Quit,
    /// The question's yes: quit, and the runs stop.
    QuitAnyway,
    /// The question put away: nothing stops.
    KeepRunning,
    /// The preview of undoing a session (D-11; EP-15c): the session by its id, else the
    /// project's latest in the ledger.
    OpenRevert {
        project: String,
        session: Option<String>,
    },
    /// The preview's Revert: the plan read again, and the revert run if it is the one shown.
    ConfirmRevert,
    /// The preview put away, or its result read.
    CloseRevert,
    /// A project's autopilot turned on or off (D-22; EP-19): the list's box, the A key. Turned
    /// on where its runs would wait at each shell command, it opens the offer of `auto` first
    /// (EP-19b).
    SetAutopilot(String, bool),
    /// The offer's answer (D-18; EP-19b): the autopilot turned on, with permission mode `auto`
    /// set in the same write when `auto` holds.
    AutopilotOn {
        project: String,
        auto: bool,
    },
    /// The offer put away: nothing is written.
    CloseOffer,
    /// A project's permission mode kept from the launch dialog (D-18; EP-19b): written into its
    /// `[project.launch]`, or taken out with `None` (back to the default).
    KeepPermissionMode(String, Option<String>),
    /// A project's model or effort kept from the launch dialog (D-22; EP-19c): written into its
    /// rule for every brief or for one phase, or taken out with `None` (back to the rules).
    KeepRule {
        project: String,
        scope: rules::Scope,
        key: rules::Key,
        value: Option<String>,
    },
    /// One of a project's autopilot limits set in the Autopilot section (D-22; EP-19b).
    SetAutopilotLimit(String, Limit),
    /// Every chain paused by the owner, or resumed (D-22): the Sessions pane, the tray.
    PauseAutopilot(bool),
    /// An ended session resumed with the owner's yes to its commit (D-22): its tab's Commit.
    CommitSession(u64),
}

/// What closing the window does (D-29; EP-15).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Closing {
    /// It goes to the tray, and its runs go on.
    Hide,
    /// The question first: headless runs would stop.
    Ask,
    /// Not yet: a revert runs, which a closing process would cut short (EP-15c).
    Wait,
    /// It closes, and the process ends.
    Close,
}

/// What closing does: a Quit asked for, or a screenshot run's own close, closes; with a tray the
/// window hides; without one it waits for a revert that runs (`reverting`), and closes after the
/// question when runs would stop (`stoppable`).
fn closing(
    quitting: bool,
    fixtures: bool,
    tray: bool,
    stoppable: usize,
    reverting: bool,
) -> Closing {
    if quitting || fixtures {
        Closing::Close
    } else if tray {
        Closing::Hide
    } else if reverting {
        Closing::Wait
    } else if stoppable > 0 {
        Closing::Ask
    } else {
        Closing::Close
    }
}

/// How a status-bar line reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Warning,
}

/// Everything the panes show and change.
pub struct Board {
    pub loader: Loader,
    pub portfolio: Portfolio,
    pub view: View,
    pub theme: ThemePreference,
    /// Ticks and every other write into a project are refused: the projects are a screenshot
    /// run's, the tracked fixtures or a copy (EP-36).
    pub fixtures: bool,
    /// What the window runs on when it is not the owner's registry: `CRW_HOME`, or a
    /// screenshot run.
    pub home_note: Option<String>,
    /// Dispositions being typed, by project and message id.
    pub drafts: HashMap<(String, String), String>,
    pub actions: Vec<Action>,
    /// The launch dialog, while it is open.
    pub launch: Option<Dialog>,
    /// The headless sessions the window started, and the CLI's preflight.
    pub sessions: Sessions,
    /// The visual launch on its way, if one is.
    pub visual: Visual,
    /// The sessions the hooks, the status lines and the poll report (EP-12); started by `main`,
    /// never by a screenshot run or a test.
    pub watch: Watch,
    /// The toasts, the badge events and their log (D-21; EP-15); turned on by `main`, never in a
    /// screenshot run or a test.
    pub toasts: Toasts,
    /// The tray icon while it runs; started by `main`, never by a screenshot run or a test.
    pub tray: Option<Tray>,
    /// The usage meter (D-23; EP-17); started by `main`, canned in a screenshot run, off in a
    /// test.
    pub meter: Meter,
    /// The autopilot's chains, held chains and log (D-22; EP-19).
    pub autopilot: crate::autopilot::Pilot,
    /// The offer of `auto` while it is open (EP-19b): a project whose autopilot is being turned
    /// on and whose runs would wait at each shell command.
    pub offer: Option<crate::autopilot::Offer>,
    /// The window is hidden in the tray (D-29).
    pub hidden: bool,
    /// The first close to the tray has been said, once for good (kept with the layout).
    pub told_tray: bool,
    /// The question before Quit stops the window's headless runs is open.
    pub quit_asked: bool,
    /// The preview of a session's revert, while it is open (EP-15c).
    pub revert: Option<crate::revert::Dialog>,
    /// The restart into the installed build (EP-37); set by `main`, inert in a screenshot run
    /// or a test.
    pub restart: crate::restart::Restart,
    /// The mailman's rounds and what they made of each block (D-13; EP-20).
    pub mail: crate::mail::Mail,
    /// The compose dialog, while it is open (D-13; EP-20c).
    pub compose: Option<crate::compose::Dialog>,
    /// A compose dialog to open once the files are read, with its text canned (a screenshot
    /// run's `compose=<project>[:urgent]`).
    pub compose_when_read: Option<(String, bool)>,
    /// Quit goes ahead: the close is let through.
    quitting: bool,
    /// When the delayed test notification is due (EP-15d).
    test_due: Option<Instant>,
    /// Where Claude Code caches its model catalog; a screenshot run's is a fixture.
    pub catalog_folder: Option<PathBuf>,
    /// A launch dialog to open once the files are read (a screenshot run's `launch=`).
    pub open_when_read: Option<String>,
    /// That dialog opens on Headless (a screenshot run's `launch=<project>:headless`), where
    /// the chain plan shows (EP-19a).
    pub open_headless: bool,
    /// The ledger a session is recorded in.
    ledger: Option<PathBuf>,
    ctx: egui::Context,
    notice: Option<(String, Tone, Instant)>,
    error: Option<String>,
    /// The selection moved by a key: the list scrolls it into view once.
    pub scroll_to_selected: bool,
    /// A pane to bring to the front after the frame.
    bring: Option<Pane>,
    settle_until: Option<Instant>,
}

impl Board {
    pub fn new(places: Places, ctx: &egui::Context, view: View, theme: ThemePreference) -> Self {
        let ledger = places.ledger.clone();
        let crashed = places
            .crash_log
            .as_deref()
            .and_then(crash::unseen)
            .map(|text| one_line(&text, NOTICE_CHARS));
        let mut loader = Loader::new(
            Arc::new(places.store),
            places.ledger,
            places.online,
            ctx.clone(),
        );
        loader.read_files();
        loader.read_git();
        loader.read_online(false);
        Self {
            loader,
            portfolio: Portfolio::default(),
            view,
            theme,
            fixtures: false,
            home_note: None,
            drafts: HashMap::new(),
            actions: Vec::new(),
            launch: None,
            sessions: Sessions::default(),
            visual: Visual::default(),
            watch: Watch::default(),
            toasts: Toasts::default(),
            tray: None,
            meter: Meter::default(),
            autopilot: crate::autopilot::Pilot::default(),
            offer: None,
            hidden: false,
            told_tray: false,
            quit_asked: false,
            revert: None,
            restart: crate::restart::Restart::default(),
            mail: crate::mail::Mail::default(),
            compose: None,
            compose_when_read: None,
            quitting: false,
            test_due: None,
            catalog_folder: Catalog::folder(),
            open_when_read: None,
            open_headless: false,
            ledger,
            ctx: ctx.clone(),
            notice: None,
            error: crashed,
            scroll_to_selected: false,
            bring: None,
            settle_until: None,
        }
    }

    pub fn store(&self) -> &Store {
        self.loader.store()
    }

    /// The window's context, which a worker thread wakes.
    pub fn ctx(&self) -> &egui::Context {
        &self.ctx
    }

    /// The ledger the window's sessions are recorded in, when it has one.
    pub fn ledger(&self) -> Option<&Path> {
        self.ledger.as_deref()
    }

    /// The name of the selected project or folder.
    pub fn selected_name(&self) -> Option<&str> {
        match self.view.selected.as_ref()? {
            Selection::Project(name) | Selection::Folder(name) => Some(name),
        }
    }

    pub fn busy(&self) -> bool {
        self.loader.busy() || self.settle_until.is_some()
    }

    /// The sessions that run in the project called `name`, as in-progress reasons (D-9's running
    /// session, EP-12): the window's own headless runs, then those the hooks and the poll report.
    pub fn session_reasons(&self, name: &str) -> Vec<InProgress> {
        let own = |id: &str| self.sessions.owns(id);
        self.sessions
            .summary(name)
            .into_iter()
            .chain(self.watch.running(name, own).map(|seen| seen.summary()))
            .map(InProgress::Session)
            .collect()
    }

    /// The same in one line for a row's Session cell; `None` when nothing runs there.
    pub fn session_summary(&self, name: &str) -> Option<String> {
        let words: Vec<String> = self
            .session_reasons(name)
            .iter()
            .map(ToString::to_string)
            .collect();
        (!words.is_empty()).then(|| words.join("; "))
    }

    /// The status bar's notice, for the tests (EP-19c).
    #[cfg(test)]
    pub fn notice_text(&self) -> Option<&str> {
        self.notice.as_ref().map(|(text, _, _)| text.as_str())
    }

    /// The status bar's error, while one stands.
    #[cfg(test)]
    pub fn error_text(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn notify(&mut self, text: impl Into<String>) {
        let text = one_line(&text.into(), NOTICE_CHARS);
        self.notice = Some((text, Tone::Plain, Instant::now()));
        self.error = None;
    }

    pub fn warn(&mut self, text: impl Into<String>) {
        let text = one_line(&text.into(), NOTICE_CHARS);
        self.notice = Some((text, Tone::Warning, Instant::now()));
    }

    /// What a toast held back while the window is in front says instead; an error stands.
    fn mention(&mut self, toast: &Toast) {
        let tone = if toast.is_warning() {
            Tone::Warning
        } else {
            Tone::Plain
        };
        let text = one_line(&toast.words(), NOTICE_CHARS);
        self.notice = Some((text, tone, Instant::now()));
    }

    pub fn fail(&mut self, text: impl Into<String>) {
        self.error = Some(one_line(&text.into(), NOTICE_CHARS));
        self.notice = None;
    }

    /// Takes in the worker threads' results and the watcher's news; after a burst of folder
    /// changes settles, reads the files again; once an hour, looks at the GitHub cache's age.
    /// The sessions' events and a start that came back are taken in too.
    pub fn receive(&mut self, ctx: &egui::Context) {
        if self.loader.receive(&mut self.portfolio) {
            self.settle_until = Some(Instant::now() + SETTLE);
            ctx.request_repaint_after(SETTLE);
        }
        match self.sessions.receive(ctx) {
            // The chain's own start neither closes the dialog nor takes the pane (EP-19).
            Some(started) if self.autopilot.is_starting() => self.chain_started(started),
            Some(Started::Running(text)) => {
                self.launch = None;
                self.notify(text);
                self.bring = Some(Pane::Sessions);
            }
            Some(Started::Refused(why)) => {
                // The CLI may have changed under the tool: the next launch asks it again.
                self.sessions.forget_preflight();
                let why = format!("The session did not start: {why}");
                // No run, so the chain plan kept at the start goes (EP-19a).
                if let Some(project) = self.launch.as_ref().map(|dialog| dialog.project.clone()) {
                    self.autopilot.drop_plan(&project);
                }
                match self.launch.as_mut() {
                    Some(dialog) => dialog.error = Some(why),
                    None => self.fail(why),
                }
            }
            None => {}
        }
        if let Some(text) = self.sessions.switched.take() {
            self.warn(text);
        }
        match self.visual.receive(ctx) {
            Some(visual::Outcome::Opened(text)) => {
                self.launch = None;
                self.notify(text);
            }
            Some(visual::Outcome::Resumed(text) | visual::Outcome::Focused(text)) => {
                self.notify(text);
            }
            Some(visual::Outcome::Refused(why)) => match self.launch.as_mut() {
                Some(dialog) => dialog.error = Some(why),
                None => self.fail(why),
            },
            None => {}
        }
        self.receive_revert(ctx);
        let portfolio = &self.portfolio;
        let registered = || {
            portfolio
                .projects
                .iter()
                .filter_map(|row| {
                    Some(Registered {
                        name: row.name().to_owned(),
                        folder: row.folder.as_ref()?.path.clone(),
                        revert: row.project.revert,
                    })
                })
                .collect()
        };
        let focused = ctx.input(|i| i.focused);
        self.watch
            .receive(registered, self.ledger.as_deref(), focused);
        if std::mem::take(&mut self.watch.recorded) {
            self.sessions.history.stale = true;
            // A session started or ended: its brief may hand mail on, or its receiver be free.
            self.mail.due = true;
        }
        // The usage meter's polls and its second series, the status lines (D-23; EP-17); a
        // visual or headless session's end the hooks report brings the next poll forward.
        let now = chrono::Utc::now();
        self.meter.receive(now);
        self.split_usage();
        // Each project's share of the week, into its row again after a files read (EP-18b).
        self.meter.read_week(now);
        self.meter.week.apply(&mut self.portfolio);
        for line in self.watch.take_lines() {
            self.meter.status_line(&line, now);
        }
        if std::mem::take(&mut self.watch.launched_ended) {
            self.meter.session_ended();
        }
        // The autopilot weighs the runs that ended against the meter's figures (D-22; EP-19).
        self.autopilot_pass();
        // The mailman delivers what the briefs hand on (D-13; EP-20).
        self.mail_pass();
        self.compose_pass();
        if let Some(name) = &self.open_when_read
            && !self.loader.reading_files
            && self.portfolio.project(name).is_some()
        {
            let name = name.clone();
            self.open_when_read = None;
            self.actions.push(Action::OpenLaunch(name));
        }
        if let Some((name, urgent)) = &self.compose_when_read
            && !self.loader.reading_files
            && self.portfolio.project(name).is_some()
        {
            let (name, urgent) = (name.clone(), *urgent);
            self.compose_when_read = None;
            self.open_compose(&name);
            if let Some(dialog) = self.compose.as_mut() {
                dialog.text = "The export's shared fixture names changed on 2026-11-10: read \
                               `fixtures/README.md` before your EP-4, and keep the old names \
                               as aliases until the index is rebuilt."
                    .to_owned();
                dialog.urgent = urgent;
                if dialog.busy.is_none() {
                    dialog.busy = Some("a session works there".to_owned());
                }
            }
        }
        #[cfg(feature = "screenshot")]
        if self.fixtures {
            crate::screenshot::readings(&mut self.portfolio, chrono::Utc::now());
        }
        if self.loader.online_due() {
            self.loader.read_online(false);
        }
        ctx.request_repaint_after(ONLINE_WAKE);
        if let Some(at) = self.settle_until {
            if Instant::now() >= at {
                self.settle_until = None;
                self.loader.read_files();
            } else {
                ctx.request_repaint_after(at - Instant::now());
            }
        }
    }

    /// Splits each of the meter's readings' rise among the sessions that worked since this run's
    /// reading before it (D-23; EP-18a): those the watch saw work, and the window's own headless
    /// runs while they run. Each part goes to its session's latest ledger row; a session with no
    /// row keeps its part unattributed.
    fn split_usage(&mut self) {
        let readings = self.meter.take_readings();
        if readings.is_empty() {
            return;
        }
        let mut workers = self.watch.take_worked();
        workers.extend(
            self.sessions
                .live
                .iter()
                .filter(|live| live.is_running())
                .map(|live| live.session_id.clone()),
        );
        let parts: Vec<Part> = readings
            .iter()
            .filter_map(|(before, after)| share::part(before.as_ref()?, after, workers.len()))
            .collect();
        let Some(path) = self.ledger.as_deref().filter(|_| !parts.is_empty()) else {
            return;
        };
        let written = Ledger::open(path).and_then(|ledger| {
            let mut any = false;
            for part in &parts {
                for id in &workers {
                    any |= ledger.add_share(id, part)?;
                }
            }
            Ok(any)
        });
        match written {
            Ok(true) => {
                self.sessions.history.stale = true;
                self.meter.week.stale = true;
            }
            Ok(false) => {}
            Err(e) => {
                self.meter.problem =
                    Some(format!("The ledger did not keep the sessions' usage: {e}"));
            }
        }
    }

    /// The notifications' share of each pass, the window hidden or shown (D-21, D-29; EP-15): a
    /// close turned into a hide or a question, the tray's orders and the toasts' clicks carried
    /// out, this pass's moments toasted or held back, the badge events read, the tray brought up
    /// to date.
    pub fn attend(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            let stoppable = self.sessions.stoppable().len();
            match closing(
                self.quitting,
                self.fixtures,
                self.tray.is_some(),
                stoppable,
                self.reverting(),
            ) {
                Closing::Hide => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    self.hide();
                }
                Closing::Ask => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    self.quit_asked = true;
                }
                Closing::Wait => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    self.warn(REVERTING);
                }
                Closing::Close => {}
            }
        }
        // The delayed test notification, sent whether the window shows or hides (EP-15d).
        if let Some(due) = self.test_due {
            let now = Instant::now();
            if now >= due {
                self.test_due = None;
                self.test_toast();
            } else {
                ctx.request_repaint_after(due - now);
            }
        }
        let orders = self.tray.as_ref().map(Tray::orders).unwrap_or_default();
        for order in orders {
            match order {
                Order::Show => self.show(),
                Order::Start(name) => {
                    self.show();
                    self.actions.push(Action::OpenLaunch(name));
                }
                Order::Quit => self.ask_quit(),
                Order::Pause => {
                    let paused = !self.autopilot.paused;
                    self.actions.push(Action::PauseAutopilot(paused));
                }
                Order::Restart => self.restart_from_window(),
            }
        }
        // The restart's share (EP-37), the window shown or hidden.
        self.restart_pass(ctx);
        // The popup is drawn by the window's pass, which a minimized window skips unless asked.
        if !self.toasts.popup.is_empty() {
            ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
        }
        // Taken whether or not toasts are on, so that a window without them keeps none.
        let news = self.sessions.take_news();
        if news.iter().any(|(_, news)| matches!(news, News::Ended(..))) {
            self.meter.session_ended();
        }
        let crossings = self.meter.take_crossings();
        // The chain's moments (EP-19a), taken whether or not toasts are on.
        let chain = self.autopilot.take_moments();
        if !self.toasts.is_on() {
            return;
        }
        let in_front = self.in_front(ctx);
        let mut moments = Vec::new();
        for (key, news) in news {
            if let Some(live) = self.sessions.live.iter().find(|live| live.key == key) {
                moments.extend(toasts::from_news(key, &live.project, &live.title(), &news));
            }
        }
        let (sessions, model) = (&self.sessions, &self.watch.model);
        let polled = model.polled.is_some();
        moments.extend(
            self.toasts
                .elsewhere(&model.all(), |id| sessions.owns(id), polled),
        );
        let now = chrono::Utc::now();
        moments.extend(crossings.iter().map(|crossing| Toast::usage(crossing, now)));
        moments.extend(chain);
        if !self.loader.busy() && self.toasts.marks_due() {
            let marks = toasts::marks(&self.portfolio, chrono::Utc::now());
            self.toasts.mark(marks, in_front);
        }
        // A message flagged urgent that arrived since the last look (EP-20a). The inboxes come
        // from the files read alone, so the look does not wait for the git and GitHub passes, and
        // the mailman's first round waits for its first look (EP-20a's review).
        if !self.loader.reading_files && !self.portfolio.projects.is_empty() {
            moments.extend(self.toasts.urgent(&self.portfolio));
        }
        if !moments.is_empty() {
            let (quiet, ntfy) = self.notify_settings();
            for toast in self
                .toasts
                .deliver(moments, quiet, in_front, ntfy.as_ref(), ctx)
            {
                self.mention(&toast);
            }
        }
        self.update_tray(ctx);
    }

    /// Whether the owner has the window in front: shown, not minimized, with the focus.
    fn in_front(&self, ctx: &egui::Context) -> bool {
        !self.hidden && ctx.input(|i| i.focused && i.viewport().minimized != Some(true))
    }

    /// The registry's quiet hours, read when a toast is due, so that `crw setting` counts at
    /// once.
    fn quiet_hours(&self) -> Option<QuietHours> {
        self.store()
            .load()
            .ok()
            .and_then(|loaded| loaded.registry.settings.quiet_hours)
    }

    /// The registry's quiet hours and its ntfy publisher (EP-20a), read in one go when a toast is
    /// due, so that `crw setting` counts at once.
    fn notify_settings(
        &self,
    ) -> (
        Option<QuietHours>,
        Option<Result<crw_core::notify::ntfy::Ntfy, String>>,
    ) {
        match self.store().load() {
            Ok(loaded) => {
                let settings = &loaded.registry.settings;
                (settings.quiet_hours, settings.ntfy())
            }
            Err(_) => (None, None),
        }
    }

    /// The tray as the window stands (D-21): what waits, what is new, the pinned projects'
    /// next briefs.
    fn update_tray(&mut self, ctx: &egui::Context) {
        if self.tray.is_none() {
            return;
        }
        let starts = self
            .portfolio
            .projects
            .iter()
            .filter(|row| row.project.pinned)
            .map(|row| {
                let next = row.next().map(|next| {
                    let size = next
                        .size
                        .map(|size| format!(" ({size})"))
                        .unwrap_or_default();
                    format!(
                        "{}{size}: {}",
                        next.id,
                        one_line(&next.brief.title, START_TITLE)
                    )
                });
                (row.name().to_owned(), next)
            })
            .collect();
        let scale = ctx.input(|i| i.viewport().native_pixels_per_point.unwrap_or(2.0));
        let now = chrono::Utc::now();
        let held = crw_core::autopilot::usage_pause(
            &self.meter.gauge.figures(now),
            self.autopilot.mark.max(1),
            now,
        )
        .iter()
        .filter_map(|refusal| match refusal {
            crw_core::autopilot::Refusal::Usage { which, percent, .. } => {
                Some(format!("{which} at {percent:.0} %"))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
        let state = tray::State {
            waiting: self.decisions_count(),
            new: self.toasts.attention.counted(),
            starts,
            paused: self.autopilot.paused,
            held: (!held.is_empty()).then(|| held.join(", ")),
            restart: self.restart.standing != Standing::Newest,
            size: (16.0 * scale).round() as u32,
        };
        if let Some(tray) = &mut self.tray {
            tray.update(&state);
        }
    }

    /// Shows the window from the tray, restored and in front (D-29).
    pub(crate) fn show(&mut self) {
        self.hidden = false;
        self.ctx
            .send_viewport_cmd(egui::ViewportCommand::Visible(true));
        self.ctx
            .send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        self.ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        self.ctx.request_repaint();
    }

    /// Hides the window in the tray, where its runs, the loopback server and the poll go on
    /// (D-29); the first time, outside the quiet hours, a toast says where it went.
    fn hide(&mut self) {
        if self.tray.is_none() {
            return;
        }
        self.hidden = true;
        self.quit_asked = false;
        self.ctx
            .send_viewport_cmd(egui::ViewportCommand::Visible(false));
        let quiet = self.quiet_hours();
        let quiet_now = quiet.is_some_and(|hours| hours.contains(chrono::Local::now().time()));
        if !self.told_tray && !quiet_now && self.toasts.is_on() {
            self.told_tray = true;
            let toast = Toast::tray(self.sessions.stoppable().len());
            let ctx = self.ctx.clone();
            // The tool's own word stays in the popup: it is no moment for the phone.
            self.toasts.deliver(vec![toast], quiet, false, None, &ctx);
        }
    }

    /// Hides the window in the tray at its start, without the toast that says where it went: a
    /// restart at the gate's moment from a window hidden there (EP-37a).
    pub(crate) fn hide_at_start(&mut self) {
        self.told_tray = true;
        self.hide();
    }

    /// Quits at once when no headless run would stop; else shows the window with the question
    /// (D-16's EP-10 addendum).
    fn ask_quit(&mut self) {
        if self.sessions.stoppable().is_empty() {
            self.quit();
        } else {
            self.show();
            self.quit_asked = true;
        }
    }

    /// Whether a revert runs, which quitting waits for (EP-15c): the process's end would cut it
    /// short, the project half put back.
    pub fn reverting(&self) -> bool {
        self.revert
            .as_ref()
            .is_some_and(crate::revert::Dialog::is_reverting)
    }

    /// Lets the window close for a restart whose new window has started (EP-37): nothing runs,
    /// since a restart is refused while anything would stop.
    pub(crate) fn close_for_restart(&mut self) {
        self.quitting = true;
        self.quit_asked = false;
        self.ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        self.ctx.request_repaint();
    }

    /// Lets the window close: the process ends, and the runs it holds with it; not while a revert
    /// runs (EP-15c).
    fn quit(&mut self) {
        if self.reverting() {
            self.quit_asked = false;
            self.show();
            self.warn(REVERTING);
            return;
        }
        self.quitting = true;
        self.quit_asked = false;
        self.ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        self.ctx.request_repaint();
    }

    /// A toast shown at once whatever the hours and the focus (EP-15).
    fn test_toast(&mut self) {
        if !self.toasts.is_on() {
            self.warn("Notifications are off in this window: it is a screenshot run.");
            return;
        }
        let ctx = self.ctx.clone();
        // Pushed to ntfy too when the registry names a server (EP-20a): the owner's check of it.
        let (_, ntfy) = self.notify_settings();
        self.toasts
            .deliver(vec![Toast::test()], None, false, ntfy.as_ref(), &ctx);
        let phone = match &ntfy {
            Some(Ok(ntfy)) => format!(
                " It goes to ntfy at {} too; the status bar's hover says how the push went.",
                ntfy.server()
            ),
            Some(Err(why)) => format!(" ntfy's settings do not read: {why}."),
            None => String::new(),
        };
        self.notify(format!(
            "A test notification is in the popup at the screen's lower right: its Show brings \
             the window forward.{phone}"
        ));
    }

    /// A test notification in ten seconds (EP-15d): time to close the window to the tray, so that
    /// it comes while the window is hidden, the path that panicked an immediate popup.
    fn test_later(&mut self) {
        if !self.toasts.is_on() {
            self.warn("Notifications are off in this window: it is a screenshot run.");
            return;
        }
        self.test_due = Some(Instant::now() + TEST_DELAY);
        self.notify(
            "A test notification comes in 10 seconds: close the window to the tray now to see \
             it arrive while the window is hidden.",
        );
    }

    /// The popup, the window shown or hidden (EP-15): a toast's Show leads where it says.
    pub fn popup(&mut self, ctx: &egui::Context) {
        for target in self.toasts.chosen(ctx) {
            match target {
                Target::VsCode(name) => self.focus(&name),
                Target::Decisions => {
                    self.show();
                    self.bring = Some(Pane::Decisions);
                }
                Target::Session(key) => {
                    self.show();
                    self.sessions.tab = Some(key);
                    self.bring = Some(Pane::Sessions);
                }
                Target::Usage => {
                    self.show();
                    self.bring = Some(Pane::Usage);
                }
                Target::ChainLog => {
                    self.show();
                    self.sessions.tab = None;
                    self.bring = Some(Pane::Sessions);
                }
                Target::Project(name) => {
                    self.show();
                    self.view.selected = Some(Selection::Project(name));
                    self.bring = Some(Pane::Detail);
                }
                Target::Window => self.show(),
            }
        }
    }

    /// The window's keys (D-25), unless a text field has the keyboard or the launch dialog or a
    /// revert's preview is open: ↑ ↓ select, → ← expand and fold the row, Enter opens the Detail
    /// pane, S opens the launch dialog, A says what is not built yet, F5 refreshes.
    pub fn keys(&mut self, ctx: &egui::Context) {
        // A dialog over the window takes the keys: none reaches the list behind it.
        if self.launch.is_some()
            || self.revert.is_some()
            || self.offer.is_some()
            || self.compose.is_some()
        {
            return;
        }
        if ctx.input_mut(|i| i.consume_shortcut(&REFRESH)) {
            self.actions.push(Action::Refresh);
        }
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let pressed = |key| ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key));
        for (key, down) in [(Key::ArrowDown, true), (Key::ArrowUp, false)] {
            if pressed(key) {
                let order = self
                    .portfolio
                    .order(self.view.sort, self.view.list_filter(), |kind| {
                        self.view.is_folded(kind)
                    });
                if let Some(next) = step(&order, self.view.selected.as_ref(), down) {
                    self.view.selected = Some(next);
                    self.scroll_to_selected = true;
                }
            }
        }
        if let Some(Selection::Project(name)) = self.view.selected.clone() {
            if pressed(Key::ArrowRight) {
                self.view.expanded.insert(name.clone());
            }
            if pressed(Key::ArrowLeft) {
                self.view.expanded.remove(&name);
            }
        }
        if pressed(Key::Enter)
            && let Some(selected) = self.view.selected.clone()
        {
            self.actions.push(Action::Open(selected));
        }
        if pressed(Key::S) {
            match self.view.selected.clone() {
                Some(Selection::Project(name)) => self.actions.push(Action::OpenLaunch(name)),
                _ => self
                    .warn("Select a project first: S opens the launch dialog on its next brief."),
            }
        }
        if pressed(Key::A) {
            let selected = match self.view.selected.clone() {
                Some(Selection::Project(name)) => self
                    .portfolio
                    .project(&name)
                    .map(|row| (name, row.project.autopilot.enabled)),
                _ => None,
            };
            match selected {
                Some((name, on)) => self.actions.push(Action::SetAutopilot(name, !on)),
                None => self.warn("Select a project first: A turns its autopilot on or off."),
            }
        }
    }

    /// Runs the queued actions; the writes read the files again after them.
    pub fn run_actions(&mut self, tree: &mut Tree<Pane>) {
        let actions = std::mem::take(&mut self.actions);
        let mut wrote = false;
        for action in actions {
            wrote |= self.run(action);
        }
        if let Some(pane) = self.bring.take() {
            layout::show(tree, pane);
        }
        if wrote {
            self.loader.read_files();
        }
    }

    /// Runs one action; `true` when it wrote a file.
    fn run(&mut self, action: Action) -> bool {
        let today = files::today();
        match action {
            Action::Select(selection) => self.view.selected = Some(selection),
            Action::Open(selection) => {
                self.view.selected = Some(selection);
                self.bring = Some(Pane::Detail);
            }
            Action::OpenLaunch(name) => self.open_launch(&name),
            Action::OpenCompose(name) => self.open_compose(&name),
            Action::Send => self.send_message(),
            Action::CloseCompose => {
                if self.compose.as_ref().is_none_or(|d| !d.is_sending()) {
                    self.compose = None;
                }
            }
            Action::CloseLaunch => {
                if self.visual.is_opening() {
                    self.visual.cancel();
                    self.notify("The visual launch is called off: nothing was handed to VS Code.");
                }
                if !self.sessions.is_starting() {
                    self.launch = None;
                }
            }
            Action::StartHeadless => self.start_headless(),
            Action::StartVisual => self.start_visual(),
            Action::StopSession(key) => self.sessions.stop(key),
            Action::KillSession(key) => self.sessions.kill(key),
            Action::DismissSession(key) => self.sessions.dismiss(key),
            Action::ResumeSession(key) => self.resume_session(key),
            Action::Respond {
                key,
                request,
                response,
            } => self.respond(key, &request, response),
            Action::AnswerDeferred { key, response } => self.answer_deferred(key, response),
            Action::Focus(name) => self.focus(&name),
            Action::ResumeInCode {
                project,
                session,
                ep,
            } => self.resume_in_code(&project, &session, ep),
            Action::TestToast => self.test_toast(),
            Action::TestToastLater => self.test_later(),
            Action::HideWindow => self.hide(),
            Action::InstallAndRestart => self.restart_from_window(),
            Action::Quit => self.ask_quit(),
            Action::QuitAnyway => self.quit(),
            Action::KeepRunning => self.quit_asked = false,
            Action::OpenRevert { project, session } => self.open_revert(&project, session),
            Action::ConfirmRevert => self.confirm_revert(),
            Action::CloseRevert => self.close_revert(),
            Action::SetAutopilot(name, on) => return self.set_autopilot(&name, on),
            Action::AutopilotOn { project, auto } => return self.autopilot_on(&project, auto),
            Action::CloseOffer => self.offer = None,
            Action::KeepPermissionMode(project, mode) => {
                return self.keep_permission_mode(&project, mode.as_deref());
            }
            Action::KeepRule {
                project,
                scope,
                key,
                value,
            } => return self.keep_rule(&project, &scope, key, value.as_deref()),
            Action::SetAutopilotLimit(project, limit) => {
                return self.set_autopilot_limit(&project, &limit);
            }
            Action::PauseAutopilot(paused) => return self.pause_autopilot(paused),
            Action::CommitSession(key) => self.commit_session(key),
            Action::ToggleExpanded(name) => {
                if !self.view.expanded.remove(&name) {
                    self.view.expanded.insert(name);
                }
            }
            Action::ToggleGroup(kind) => {
                let name = kind.name().to_owned();
                if !self.view.folded.remove(&name) {
                    self.view.folded.insert(name);
                }
            }
            Action::Refresh => {
                self.loader.read_files();
                self.loader.read_git();
                self.loader.read_online(true);
                if !self.fixtures {
                    self.watch.read_entries();
                }
                self.notify(
                    "Reading every project again: the files, the backup job, git, then GitHub.",
                );
            }
            Action::Adopt(name) => {
                // A screenshot run's data root is the tracked fixtures or a copy: no file beside
                // them.
                let own_files = if self.fixtures {
                    OwnFiles::Skip
                } else {
                    OwnFiles::Write
                };
                match discovery::adopt(self.store(), &[name.as_str()], &today, own_files) {
                    Ok(adopted) => {
                        let text = adopted
                            .iter()
                            .map(|one| {
                                let own = match &one.own_file {
                                    OwnFile::Written(path) => format!("wrote {}", path.display()),
                                    OwnFile::Kept(path) => {
                                        format!("kept its own {}", path.display())
                                    }
                                    OwnFile::Skipped => {
                                        "no workspace file of its own in a screenshot run"
                                            .to_owned()
                                    }
                                };
                                let listing = match &one.workspace {
                                    Listing::Appended { backup } => format!(
                                        "added to the workspace file (the old one is kept as {})",
                                        backup.display()
                                    ),
                                    Listing::AlreadyListed => {
                                        "already in the workspace file".to_owned()
                                    }
                                    Listing::NoWorkspaceFile => {
                                        "no workspace file is set".to_owned()
                                    }
                                };
                                format!("Adopted {} as {}; {own}; {listing}.", one.name, one.state)
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        self.view.selected = Some(Selection::Project(name));
                        self.notify(text);
                    }
                    Err(e) => self.fail(format!("Adopt {name}: {e}")),
                }
                return true;
            }
            Action::Ignore(name) => {
                match discovery::ignore(self.store(), &[name.as_str()], &today) {
                    Ok(_) => self.notify(format!(
                        "Ignoring {name}: it stays out of the unadopted group (the registry's \
                         ignore list)."
                    )),
                    Err(e) => self.fail(format!("Ignore {name}: {e}")),
                }
                return true;
            }
            Action::SetState(name, state) => {
                let changed = self.store().change(&today, |c| c.set_state(&name, state));
                self.report_change(changed.map(|_| format!("{name} is {state} now.")));
                return true;
            }
            Action::Dismiss(name, reason) => {
                let changed = self.store().change(&today, |c| c.dismiss(&name, &reason));
                self.report_change(changed.map(|_| {
                    format!(
                        "{name}: dismissed ({reason}); it returns when {}.",
                        reason.returns_when()
                    )
                }));
                return true;
            }
            Action::Restore(name, kind) => {
                let changed = self.store().change(&today, |c| c.undismiss(&name, kind));
                self.report_change(
                    changed.map(|_| format!("{name}: the dismissed proposal shows again.")),
                );
                return true;
            }
            Action::SetPinned(name, pinned) => {
                let changed = self.store().change(&today, |c| c.set_pinned(&name, pinned));
                let text = if pinned {
                    format!("{name} is pinned: it sorts first in its group.")
                } else {
                    format!("{name} is no longer pinned.")
                };
                self.report_change(changed.map(|_| text));
                return true;
            }
            Action::SetPriority(name, priority) => {
                let changed = self
                    .store()
                    .change(&today, |c| c.set_priority(&name, priority));
                self.report_change(changed.map(|_| format!("{name} has priority {priority}.")));
                return true;
            }
            Action::Tick { project, id } => return self.tick(&project, &id, true, &today),
            Action::Untick { project, id } => return self.tick(&project, &id, false, &today),
            Action::TickMessage { project, id } => {
                let key = (project.clone(), id.clone());
                let disposition = self.drafts.get(&key).cloned().unwrap_or_default();
                let Some(folder) = self.folder_for_write(&project) else {
                    return false;
                };
                match inbox::tick(&folder, &id, &disposition, &self.store().backups, &today) {
                    Ok(copy) => {
                        self.drafts.remove(&key);
                        let logged = self.log_tick(&project, &id, true, Some(&disposition));
                        self.notify(format!(
                            "Ticked {id} in {project} with its disposition; the file as it was \
                             is kept as {}.{logged}",
                            copy.display()
                        ));
                    }
                    Err(e) => self.fail(format!("{project} {id}: {e}")),
                }
                return true;
            }
        }
        false
    }

    fn report_change<T>(&mut self, changed: Result<String, T>)
    where
        T: std::fmt::Display,
    {
        match changed {
            Ok(text) => self.notify(text),
            Err(e) => self.fail(e.to_string()),
        }
    }

    /// The folder of `project` when the tool may write into it; a screenshot run's fixtures
    /// are never written.
    fn folder_for_write(&mut self, project: &str) -> Option<PathBuf> {
        if self.fixtures {
            self.warn(format!(
                "A screenshot run writes nothing into its projects: {project} is a fixture or a \
                 copy."
            ));
            return None;
        }
        let folder = self
            .portfolio
            .project(project)
            .and_then(|row| row.folder.as_ref())
            .map(|folder| folder.path.clone());
        if folder.is_none() {
            self.fail(format!("{project}: its folder is missing."));
        }
        folder
    }

    fn tick(&mut self, project: &str, id: &str, checked: bool, today: &str) -> bool {
        let Some(folder) = self.folder_for_write(project) else {
            return false;
        };
        let backups = self.store().backups.clone();
        let done = if checked {
            tasks::tick(&folder, id, &backups, today)
        } else {
            tasks::untick(&folder, id, &backups, today)
        };
        match done {
            Ok(copy) => {
                let logged = self.log_tick(project, id, checked, None);
                self.notify(format!(
                    "{} {id} in {project}; the file as it was is kept as {}.{logged}",
                    if checked { "Ticked" } else { "Cleared" },
                    copy.display()
                ));
            }
            Err(e) => self.fail(format!("{project} {id}: {e}")),
        }
        true
    }

    /// Logs a tick the window made in the ledger, for a revert to redo (EP-15b): nothing to add
    /// to the notice when the ledger took it, else a sentence that says so.
    fn log_tick(
        &self,
        project: &str,
        id: &str,
        checked: bool,
        disposition: Option<&str>,
    ) -> String {
        match &self.ledger {
            Some(path) => match revert::log_tick(path, project, id, checked, disposition) {
                Ok(()) => String::new(),
                Err(why) => format!(" A revert would not redo it: {why}."),
            },
            None => String::new(),
        }
    }

    /// Opens the launch dialog on `name`'s next brief (D-17, D-22): the rule table's resolution
    /// and the prompt, from the registry as it stands and the project's snapshot; or says why
    /// there is nothing to launch. The CLI's preflight is asked meanwhile.
    fn open_launch(&mut self, name: &str) {
        let Some(row) = self.portfolio.project(name) else {
            self.warn(format!("{name}: no such project to start."));
            return;
        };
        let Some(folder) = row.folder.as_ref().map(|folder| folder.path.clone()) else {
            self.fail(format!(
                "{name}: its folder is missing, so nothing can start there."
            ));
            return;
        };
        let (project, snapshot, reasons) =
            (row.project.clone(), row.snapshot.clone(), row.in_progress());
        let registry = match self.store().load() {
            Ok(loaded) => loaded.registry,
            Err(e) => {
                self.fail(format!("The registry could not be read: {e}"));
                return;
            }
        };
        // The project as the registry says it now, not as the files were last read: a keep just
        // written is read again only after this (EP-19c's review).
        let project = registry.project(name).cloned().unwrap_or(project);
        let program = cli::locate(registry.settings.claude_cli.as_deref())
            .unwrap_or_else(|| PathBuf::from("claude.exe"));
        let preflight = self.sessions.preflight(&program, &self.ctx);
        let levels: Vec<String> = preflight
            .as_ref()
            .and_then(|preflight| preflight.flags.as_ref().ok().map(|f| f.efforts.clone()))
            .unwrap_or_default();
        let version = preflight
            .as_ref()
            .and_then(|preflight| preflight.version.as_ref().ok().map(ToString::to_string));
        let route = self
            .sessions
            .route(&program, version.as_deref(), registry.settings.decisions);
        let claude_md = prompt::claude_md(&folder);
        let today = files::today();
        // The downshift weighs the meter's Fable figure as it stands (D-23; EP-18).
        let now = chrono::Utc::now();
        let usage = rules::Usage::of(&self.meter.gauge.figures(now), now);
        let draft = match prompt::draft(
            &registry,
            &project,
            &snapshot,
            claude_md.as_deref(),
            &levels,
            &usage,
            &today,
        ) {
            Ok(draft) => draft,
            Err(why) => {
                self.warn(format!("{name}: nothing to launch: {why}."));
                return;
            }
        };
        let mut warnings = Vec::new();
        if !draft.parts.due.is_empty() {
            warnings.push(draft.parts.due.clone());
        }
        if !reasons.is_empty() {
            let texts: Vec<String> = reasons.iter().map(ToString::to_string).collect();
            warnings.push(format!(
                "A session may be in progress already ({}): a second one would edit the same \
                 files.",
                texts.join("; ")
            ));
        }
        if let Some(running) = self.sessions.summary(name) {
            warnings.push(format!("This window already has {running}."));
        }
        let own = |id: &str| self.sessions.owns(id);
        let seen: Vec<String> = self
            .watch
            .running(name, own)
            .map(|seen| seen.summary())
            .collect();
        if !seen.is_empty() {
            warnings.push(format!(
                "Claude Code reports {}: a second session would edit the same files.",
                seen.join("; ")
            ));
        }
        let catalog = self.catalog_folder.as_deref().map_or_else(
            || Catalog::built_in("no profile folder".to_owned()),
            Catalog::read,
        );
        let vscode_setting = registry.settings.vscode.clone();
        // The chain plan (EP-19a): the briefs a chain may run after this one, each as the rules
        // resolve it, for a project whose autopilot is on.
        let plan = project.autopilot.enabled.then(|| {
            crw_core::autopilot::plan(&registry, &project, &snapshot, draft.ep, &levels, &usage)
        });
        // What the project keeps for the model and the effort, which the Keep menus beside them
        // can take back (EP-19c).
        let phase = draft.phase.as_deref();
        let kept_model = rules::Kept::of(&project, phase, rules::Key::Model);
        let kept_effort = rules::Kept::of(&project, phase, rules::Key::Effort);
        self.launch = Some(Dialog::new(
            draft,
            Opening {
                readiness: self.visual.readiness(&folder, vscode_setting.as_deref()),
                folder,
                program,
                neighbours: project.neighbour_folders(&registry.data_root()),
                warnings,
                template: project.launch.template.clone(),
                catalog,
                date: today,
                vscode_setting,
                answer_timeout: project.answer_timeout(),
                route,
                plan,
                kept_mode: project.launch.permission_mode.clone(),
                kept_model,
                kept_effort,
                levels,
            },
        ));
        // A screenshot run's launch opens on Headless, where the plan shows.
        if self.open_headless
            && let Some(dialog) = self.launch.as_mut()
        {
            dialog.set_mode(LaunchMode::Headless);
        }
    }

    /// Hands the dialog's prompt to VS Code (D-16, EP-11), off the UI thread, since the project's
    /// window may take seconds to start; a screenshot run opens nothing (rule 5).
    fn start_visual(&mut self) {
        if self.fixtures {
            self.warn("A screenshot run opens nothing: its projects are fixtures or a copy.");
            return;
        }
        if self.visual.is_opening() || self.sessions.is_starting() {
            return;
        }
        let Some(dialog) = self.launch.as_mut() else {
            return;
        };
        let Some(code) = dialog.readiness.code.clone() else {
            dialog.error =
                Some("VS Code was not found: set its Code.exe with crw setting vscode.".to_owned());
            return;
        };
        let Some(locks) = crw_core::visual::lock_folder() else {
            dialog.error = Some("No profile folder: USERPROFILE is not set.".to_owned());
            return;
        };
        dialog.error = None;
        let launch = dialog.visual_launch(&code, &locks, self.ledger.clone());
        self.visual.start(launch, &self.ctx);
    }

    /// Starts the dialog's launch headless (D-16), off the UI thread; a screenshot run starts
    /// nothing (rule 5).
    fn start_headless(&mut self) {
        if self.fixtures {
            self.warn("A screenshot run starts no session: its projects are fixtures or a copy.");
            return;
        }
        if self.sessions.is_starting() {
            return;
        }
        let Some(dialog) = self.launch.as_mut() else {
            return;
        };
        let Some(preflight) = self.sessions.preflight(&dialog.program, &self.ctx) else {
            dialog.error = Some("claude has not answered yet: try again in a moment.".to_owned());
            return;
        };
        let refusals = preflight.refusals();
        if !refusals.is_empty() {
            dialog.error = Some(refusals.join("; "));
            return;
        }
        dialog.error = None;
        let mut launch = dialog.launch(self.ledger.clone());
        let effort = dialog.effort_words().to_owned();
        let (size, commits) = (dialog.size, dialog.commits());
        let plan = dialog.plan.clone();
        // The project as the registry says it now, so that an autopilot turned on a moment ago
        // counts before the files are read again (EP-19b's review): its cap, budget, plan and
        // push guard.
        let project = self
            .store()
            .load()
            .ok()
            .and_then(|loaded| loaded.registry.project(&launch.project).cloned())
            .or_else(|| {
                self.portfolio
                    .project(&launch.project)
                    .map(|row| row.project.clone())
            });
        // The project's start, read just before the run for a revert, unless its switch is off
        // (EP-15b).
        if project.as_ref().is_some_and(|project| project.revert) {
            launch.starts = Some(self.store().backups.clone());
        }
        // With the autopilot on, the run has its cap and the project's budget, and its end is
        // weighed by the gate (D-22; EP-19).
        let run = match project.filter(|project| project.autopilot.enabled) {
            Some(project) => {
                launch.max_budget_usd = project.autopilot.max_budget_usd;
                // Its commits stay local until the owner pushes them (D-22; EP-19b).
                launch.deny_push = true;
                // The chain plan as the owner left it, kept in the window until the chain
                // ends (EP-19a); a start the runner refuses lets it go again.
                if let Some(plan) = plan {
                    self.autopilot.keep_plan(plan);
                }
                crate::sessions::Run {
                    capped: true,
                    size,
                    chained: false,
                    commits,
                }
            }
            None => crate::sessions::Run {
                commits,
                ..crate::sessions::Run::default()
            },
        };
        // The route as it stands now: a refused handshake since the dialog opened counts (D-19).
        launch.route = self
            .sessions
            .route_for(&preflight, self.decisions_setting())
            .0;
        launch.hooks = crw_exe();
        self.sessions
            .start(preflight, launch, effort, run, &self.ctx);
    }

    /// The registry's `settings.decisions`, when it reads.
    fn decisions_setting(&self) -> Option<Route> {
        self.store()
            .load()
            .ok()
            .and_then(|loaded| loaded.registry.settings.decisions)
    }

    /// Sends the owner's answer to the window's session `key` that waits on `request` (D-19,
    /// EP-14); a screenshot run sends nothing.
    fn respond(&mut self, key: u64, request: &str, response: Response) {
        if self.fixtures {
            self.warn("A screenshot run answers nothing: its sessions are canned.");
            return;
        }
        let title = self
            .sessions
            .live
            .iter()
            .find(|live| live.key == key)
            .map(|live| live.title())
            .unwrap_or_default();
        match self.sessions.respond(key, request, response) {
            Ok(words) => self.notify(format!(
                "{title}: {}; the session goes on.",
                one_line(&words, ANSWER_CHARS)
            )),
            Err(why) => self.fail(format!("{title}: the answer was not sent: {why}.")),
        }
    }

    /// Resumes the window's deferred run `key` with the owner's answer to its open request (D-19;
    /// EP-14), off the UI thread as a start is; a screenshot run starts nothing.
    fn answer_deferred(&mut self, key: u64, response: Response) {
        if self.fixtures {
            self.warn("A screenshot run starts no session: its projects are fixtures or a copy.");
            return;
        }
        if self.sessions.is_starting() {
            return;
        }
        let Some(program) = self
            .sessions
            .live
            .iter()
            .find(|live| live.key == key)
            .map(|live| live.program().to_owned())
        else {
            return;
        };
        let Some(preflight) = self.sessions.preflight(&program, &self.ctx) else {
            self.warn("claude has not answered yet: press the button again in a moment.");
            return;
        };
        let refusals = preflight.refusals();
        if !refusals.is_empty() {
            self.fail(format!(
                "The session cannot resume: {}",
                refusals.join("; ")
            ));
            return;
        }
        let Some(answers) = paths::local_dir().map(|dir| dir.join("answers")) else {
            self.fail("No profile folder for the answer: LOCALAPPDATA is not set.");
            return;
        };
        let route = self
            .sessions
            .route_for(&preflight, self.decisions_setting())
            .0;
        let answered = self.sessions.answer_open(
            key,
            preflight,
            response,
            (route, crw_exe(), &answers),
            &self.ctx,
        );
        if let Err(why) = answered {
            self.fail(format!("The session cannot resume: {why}."));
        }
    }

    /// Brings forward the VS Code window of `name`, where one of its sessions waits (D-19; EP-14),
    /// off the UI thread; a screenshot run opens nothing.
    fn focus(&mut self, name: &str) {
        if self.fixtures {
            self.warn("A screenshot run opens nothing: its projects are fixtures or a copy.");
            return;
        }
        let Some(folder) = self
            .portfolio
            .project(name)
            .and_then(|row| row.folder.as_ref())
            .map(|folder| folder.path.clone())
        else {
            self.fail(format!("{name}: its folder is missing."));
            return;
        };
        let setting = self
            .store()
            .load()
            .ok()
            .and_then(|loaded| loaded.registry.settings.vscode);
        let Some(code) = crw_core::visual::locate(setting.as_deref()) else {
            self.fail("VS Code was not found: set its Code.exe with crw setting vscode.");
            return;
        };
        self.visual.focus(name, &folder, &code, &self.ctx);
    }

    /// Opens the session `session` of the project called `name` in a Claude Code tab of the
    /// project's VS Code window (D-16; EP-35), off the UI thread as a visual launch is, since the
    /// window may take seconds to start; the ledger keeps the handover as an event, and the run
    /// earns its row at its first work. A screenshot run opens nothing.
    fn resume_in_code(&mut self, name: &str, session: &str, ep: Option<String>) {
        if self.fixtures {
            self.warn("A screenshot run opens nothing: its projects are fixtures or a copy.");
            return;
        }
        if self.visual.is_opening() {
            self.warn("A launch to VS Code is on its way: resume once it has gone through.");
            return;
        }
        let Some(folder) = self
            .portfolio
            .project(name)
            .and_then(|row| row.folder.as_ref())
            .map(|folder| folder.path.clone())
        else {
            self.fail(format!("{name}: its folder is missing."));
            return;
        };
        let setting = self
            .store()
            .load()
            .ok()
            .and_then(|loaded| loaded.registry.settings.vscode);
        let Some(code) = crw_core::visual::locate(setting.as_deref()) else {
            self.fail("VS Code was not found: set its Code.exe with crw setting vscode.");
            return;
        };
        let Some(locks) = crw_core::visual::lock_folder() else {
            self.fail("No profile folder: USERPROFILE is not set.");
            return;
        };
        let mut launch = crw_core::visual::Launch::resume(name, &folder, session, &code, &locks);
        launch.ep = ep;
        launch.ledger = self.ledger.clone();
        self.notify(format!(
            "{name}: opening its session {} in VS Code…",
            crw_core::revert::short(session)
        ));
        self.visual.start(launch, &self.ctx);
    }

    /// Resumes the window's ended session `key` with the message typed in its tab (D-16's EP-9
    /// addendum, EP-13), off the UI thread as a start is; a screenshot run starts nothing.
    fn resume_session(&mut self, key: u64) {
        if self.fixtures {
            self.warn("A screenshot run starts no session: its projects are fixtures or a copy.");
            return;
        }
        let Some((message, program)) = self.sessions.resumable(key) else {
            return;
        };
        let Some(preflight) = self.sessions.preflight(&program, &self.ctx) else {
            self.warn("claude has not answered yet: press Resume again in a moment.");
            return;
        };
        let refusals = preflight.refusals();
        if !refusals.is_empty() {
            self.fail(format!(
                "The session cannot resume: {}",
                refusals.join("; ")
            ));
            return;
        }
        let route = self
            .sessions
            .route_for(&preflight, self.decisions_setting())
            .0;
        self.sessions
            .resume(key, preflight, &message, route, &self.ctx);
    }

    /// The launch dialog over the window, while it is open. Escape and Cancel close it; a click
    /// beside it does not, so the prompt's edits are not lost by a stray click.
    pub fn launch_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.launch.as_mut() else {
            return;
        };
        let preflight = self.sessions.preflight(&dialog.program, ctx);
        if let Some(preflight) = &preflight {
            dialog.fit(preflight);
        }
        if dialog.readiness_due() {
            let readiness = self
                .visual
                .readiness(&dialog.folder, dialog.vscode_setting.as_deref());
            dialog.set_readiness(readiness);
        }
        if dialog.visual() {
            // So that a window opened or closed meanwhile shows without a mouse move.
            ctx.request_repaint_after(Duration::from_secs(2));
        }
        let (asking, starting) = (self.sessions.is_asking(), self.sessions.is_starting());
        let opening = self.visual.is_opening();
        let modal = egui::Modal::new(egui::Id::unique("launch-dialog")).show(ctx, |ui| {
            dialog.ui(ui, preflight.as_deref(), asking, starting, opening)
        });
        let close = !starting && !modal.backdrop_response.clicked() && modal.should_close();
        self.actions.extend(modal.inner);
        if close {
            self.actions.push(Action::CloseLaunch);
        }
    }

    /// The question before Quit stops the window's headless runs (D-16's EP-10 addendum; EP-15):
    /// what stops, how a stopped run is picked up again, and the ways out. Escape keeps running.
    pub fn quit_dialog(&mut self, ctx: &egui::Context) {
        if !self.quit_asked {
            return;
        }
        let runs = self.sessions.stoppable();
        if runs.is_empty() && !self.fixtures {
            // The runs ended meanwhile: nothing would stop.
            self.quit();
            return;
        }
        let tray = self.tray.is_some() || self.fixtures;
        let width = 560.0_f32.min(ctx.content_rect().width() - 80.0);
        let modal = egui::Modal::new(egui::Id::unique("quit-question")).show(ctx, |ui| {
            ui.set_width(width);
            ui.heading("Quit Roadmap Wrangler?");
            ui.add_space(4.0);
            let what = match runs.len() {
                1 => "the window's headless session".to_owned(),
                n => format!("the window's {n} headless sessions"),
            };
            ui.label(format!(
                "Quitting stops {what} with its process tree, and the ledger records the run as \
                 stopped:"
            ));
            for run in &runs {
                ui.label(RichText::new(format!("    {run}")).strong());
            }
            ui.add_space(4.0);
            ui.add(
                egui::Label::new(
                    RichText::new(
                        "Claude Code's own list of the project's sessions (claude --resume there) \
                         picks a stopped run up again. The sessions in VS Code go on.",
                    )
                    .small()
                    .weak(),
                )
                .wrap(),
            );
            ui.add_space(8.0);
            let mut chosen = None;
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Keep running").clicked() {
                        chosen = Some(Action::KeepRunning);
                    }
                    if tray
                        && ui
                            .button("Close to the tray")
                            .on_hover_text("Hide the window: its runs go on")
                            .clicked()
                    {
                        chosen = Some(Action::HideWindow);
                    }
                    let stop = if runs.len() == 1 {
                        "Quit and stop it"
                    } else {
                        "Quit and stop them"
                    };
                    if ui
                        .button(RichText::new(stop).color(style::red(ui)))
                        .clicked()
                    {
                        chosen = Some(Action::QuitAnyway);
                    }
                });
            });
            chosen
        });
        match modal.inner {
            Some(action) => self.actions.push(action),
            None if modal.should_close() => self.actions.push(Action::KeepRunning),
            None => {}
        }
    }

    /// The notifications and the tray in words, for the status bar's hover (EP-15).
    fn notifications_line(&self) -> String {
        let tray = match &self.tray {
            Some(tray) => tray.line(),
            None if self.toasts.is_on() => "no tray icon: closing the window quits".to_owned(),
            None => "no tray icon".to_owned(),
        };
        let (quiet, ntfy) = self.notify_settings();
        format!("{}; {tray}", self.toasts.line(quiet, ntfy.as_ref()))
    }

    /// Errors stay until the next notice; a notice fades. With nothing to report, the keys.
    pub fn status_bar(&self, ui: &mut egui::Ui) {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let mut quiet = true;
            if let Some(problem) = &self.portfolio.problem {
                ui.label(RichText::new(problem).color(style::red(ui)));
                quiet = false;
            }
            if let Some(error) = &self.error {
                ui.label(RichText::new(error).color(style::red(ui)));
                quiet = false;
            } else if let Some((text, tone, at)) = &self.notice
                && let Some(left) = notice_left(at.elapsed())
            {
                let text = RichText::new(text);
                ui.label(match tone {
                    Tone::Plain => text,
                    Tone::Warning => text.color(style::amber(ui)),
                });
                ui.ctx().request_repaint_after(left);
                quiet = false;
            }
            if let Some(problem) = &self.loader.watch_problem {
                ui.label(RichText::new(problem).color(style::amber(ui)));
                quiet = false;
            }
            if let Some(problem) = &self.loader.online_problem {
                ui.label(RichText::new(problem).color(style::amber(ui)));
                quiet = false;
            }
            if let Some(problem) = &self.watch.problem {
                ui.label(RichText::new(problem).color(style::amber(ui)));
                quiet = false;
            }
            if let Some((line, true)) = self.watch.line() {
                ui.label(RichText::new(line).color(style::amber(ui)));
                quiet = false;
            }
            if quiet {
                ui.label(
                    RichText::new(
                        "↑ ↓ select · → ← expand · Enter detail · S start · A autopilot · F5 refresh",
                    )
                    .weak(),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let reading = match (self.loader.git_progress, self.loader.online_progress) {
                    _ if self.loader.reading_files => "Reading the files…".to_owned(),
                    (Some((done, of)), _) if of > 0 => format!("Asking git: {done} of {of}"),
                    (Some(_), _) => "Asking git…".to_owned(),
                    (None, Some((done, of))) if of > 0 => {
                        format!("Asking GitHub: {done} of {of}")
                    }
                    (None, Some(_)) => "Asking GitHub…".to_owned(),
                    (None, None) => "Up to date".to_owned(),
                };
                let github = match &self.portfolio.account {
                    Some(fetched) => {
                        let age = short_age(chrono::Utc::now() - fetched.at);
                        match &fetched.value {
                            Ok(list) => format!(
                                "GitHub: {} repositories of the account, read {age} ago",
                                list.len()
                            ),
                            Err(e) => format!("GitHub could not be asked ({age} ago): {e}"),
                        }
                    }
                    None => "GitHub: not read yet".to_owned(),
                };
                let signals = self.watch.line().map_or_else(
                    || "Signals: not started".to_owned(),
                    |(line, _)| line,
                );
                // Built only while hovered: the notifications' line reads the registry.
                ui.label(RichText::new(reading).weak()).on_hover_ui(|ui| {
                    ui.label(format!(
                        "Registry {}\nData root {}\n{github}\n{signals}\n{}",
                        self.portfolio.registry.display(),
                        self.portfolio.data_root.display(),
                        self.notifications_line()
                    ));
                });
                self.meter.strip(ui, chrono::Utc::now());
                if let Some(note) = &self.home_note {
                    style::chip(ui, note, style::amber(ui));
                }
                // A newer build built or installed (EP-37).
                if let Some((chip, hover)) = self.restart.chip() {
                    style::chip(ui, chip, style::amber(ui)).on_hover_text(hover);
                }
            });
        });
        ui.add_space(2.0);
    }
}

/// The `crw.exe` beside the window's own exe, which the defer route's hooks run (D-19; EP-14).
pub(crate) fn crw_exe() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .map(|exe| exe.with_file_name("crw.exe"))
        .filter(|path| path.is_file())
}

fn stored<T: serde::de::DeserializeOwned>(
    storage: Option<&dyn eframe::Storage>,
    key: &str,
) -> Option<T> {
    storage.and_then(|s| eframe::get_value(s, key))
}

/// The window: the layout over the board.
pub struct Dashboard {
    layout: Tree<Pane>,
    board: Board,
}

impl Dashboard {
    pub fn new(cc: &eframe::CreationContext<'_>, places: Places) -> Self {
        egui_material_icons::initialize(&cc.egui_ctx);
        let theme: ThemePreference = stored(cc.storage, THEME_KEY).unwrap_or_default();
        let view: View = stored(cc.storage, VIEW_KEY).unwrap_or_default();
        cc.egui_ctx.set_theme(theme);
        view.density.apply(&cc.egui_ctx);
        let mut board = Board::new(places, &cc.egui_ctx, view, theme);
        board.told_tray = stored(cc.storage, TRAY_KEY).unwrap_or(false);
        Self {
            layout: layout::restore(cc.storage),
            board,
        }
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    #[cfg(feature = "screenshot")]
    pub fn layout_mut(&mut self) -> &mut Tree<Pane> {
        &mut self.layout
    }

    fn menu(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui
                    .button("Send a test notification")
                    .on_hover_text(
                        "A notification in the popup at the screen's lower right, whatever the \
                         quiet hours: its Show brings the window forward",
                    )
                    .clicked()
                {
                    self.board.actions.push(Action::TestToast);
                }
                if ui
                    .button("Send a test notification in 10 seconds")
                    .on_hover_text(
                        "Time to close the window to the tray first: the notification then comes \
                         while the window is hidden",
                    )
                    .clicked()
                {
                    self.board.actions.push(Action::TestToastLater);
                }
                ui.separator();
                // The restart into the installed build (EP-37): enabled when a newer build waits
                // and nothing would be lost, else its hover says why not.
                self.board.restart.look(crate::restart::LOOK_OPEN);
                let refusals = self.board.restart_refusals();
                let standing = self.board.restart.standing;
                let hover = match standing {
                    Standing::Installed => {
                        "Open the newer installed build in place of this window: the layout is \
                         kept, and the window is back in a moment"
                    }
                    Standing::Built | Standing::Newest => {
                        "Install the newer build of the window's repository (its target\\release) \
                         into the install folder, then open it in place of this window"
                    }
                };
                let why_not = if refusals.is_empty() {
                    "The window runs the newest build: nothing to install or restart into"
                        .to_owned()
                } else {
                    format!("Not now: {}", refusals.join("; "))
                };
                if ui
                    .add_enabled(
                        refusals.is_empty() && standing != Standing::Newest,
                        egui::Button::new("Install and restart"),
                    )
                    .on_hover_text(hover)
                    .on_disabled_hover_text(why_not)
                    .clicked()
                {
                    self.board.actions.push(Action::InstallAndRestart);
                }
                if ui
                    .add_enabled(
                        self.board.tray.is_some(),
                        egui::Button::new("Close to the tray"),
                    )
                    .on_hover_text("Hide the window: its sessions go on, and its icon shows it")
                    .on_disabled_hover_text("No tray icon runs in this window: closing it quits")
                    .clicked()
                {
                    self.board.actions.push(Action::HideWindow);
                }
                if ui
                    .button("Quit")
                    .on_hover_text("End the window, and with it the headless sessions it holds")
                    .clicked()
                {
                    self.board.actions.push(Action::Quit);
                }
            });
            ui.menu_button("View", |ui| {
                ui.label(RichText::new("Theme").weak());
                let before = self.board.theme;
                for (theme, label) in [
                    (ThemePreference::System, "System"),
                    (ThemePreference::Light, "Light"),
                    (ThemePreference::Dark, "Dark"),
                ] {
                    ui.radio_value(&mut self.board.theme, theme, label);
                }
                if self.board.theme != before {
                    ui.ctx().set_theme(self.board.theme);
                }
                ui.separator();
                ui.label(RichText::new("Density").weak());
                let before = self.board.view.density;
                for density in [Density::Compact, Density::Comfortable] {
                    ui.radio_value(&mut self.board.view.density, density, density.label());
                }
                if self.board.view.density != before {
                    self.board.view.density.apply(ui.ctx());
                }
                ui.separator();
                if ui.button("Expand every row").clicked() {
                    let names = self
                        .board
                        .portfolio
                        .projects
                        .iter()
                        .map(|row| row.name().to_owned());
                    self.board.view.expanded.extend(names);
                }
                if ui.button("Fold every row").clicked() {
                    self.board.view.expanded.clear();
                }
                ui.separator();
                if ui
                    .button("Reset layout")
                    .on_hover_text("Put every pane back where it started")
                    .clicked()
                {
                    self.layout = layout::default_layout();
                }
            });
            if ui
                .button(style::icon(icons::ICON_REFRESH, None))
                .on_hover_text("Read every project again: the files, then git (F5)")
                .clicked()
            {
                self.board.actions.push(Action::Refresh);
            }
            if self.board.busy() {
                ui.spinner()
                    .on_hover_text("Reading the projects: the files, then git");
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let portfolio = &self.board.portfolio;
                let waiting: usize = portfolio.projects.iter().map(|r| r.open_items()).sum();
                let mail: usize = portfolio.projects.iter().map(|r| r.open_messages().0).sum();
                ui.label(
                    RichText::new(format!(
                        "{} projects · {} unadopted · {waiting} owner items · {mail} inbox",
                        portfolio.projects.len(),
                        portfolio.unadopted.len(),
                    ))
                    .weak(),
                );
            });
        });
    }
}

impl eframe::App for Dashboard {
    /// Runs before each frame and, while the window is hidden in the tray, without one (eframe
    /// 0.37): the window's work goes on in the tray (D-29; EP-15).
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.board.receive(ctx);
        self.board.attend(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        #[cfg(feature = "screenshot")]
        crate::screenshot::tick(&ctx);
        // The popup is a window of its own: drawn whether this one shows or not.
        self.board.popup(&ctx);
        if self.board.hidden {
            // Hidden in the tray, whose pass Windows still runs: nothing more to draw.
            return;
        }
        self.board.keys(&ctx);
        #[cfg(feature = "screenshot")]
        crate::screenshot::set_busy(self.board.busy());

        egui::Panel::top("menu").show(ui, |ui| self.menu(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.board.status_bar(ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(0.0))
            .show(ui, |ui| self.layout.ui(&mut self.board, ui));
        self.board.launch_dialog(&ctx);
        self.board.offer_dialog(&ctx);
        self.board.compose_dialog(&ctx);
        self.board.quit_dialog(&ctx);
        self.board.revert_dialog(&ctx);
        self.board.run_actions(&mut self.layout);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, layout::LAYOUT_KEY, &self.layout);
        eframe::set_value(storage, THEME_KEY, &self.board.theme);
        eframe::set_value(storage, VIEW_KEY, &self.board.view);
        eframe::set_value(storage, TRAY_KEY, &self.board.told_tray);
    }

    #[cfg(feature = "screenshot")]
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        crate::screenshot::inject_keys(raw_input);
    }
}

/// How long a notice shown for `shown` has left; none once [`NOTICE_FOR`] has passed. The age is
/// read once: a second reading after the check could pass the mark, and `Duration` subtraction
/// panics on underflow (EP-14a).
fn notice_left(shown: Duration) -> Option<Duration> {
    NOTICE_FOR.checked_sub(shown).filter(|left| !left.is_zero())
}

/// `text`'s first line, at most `limit` characters: a longer line is cut with an ellipsis, and
/// further lines are marked by one.
fn one_line(text: &str, limit: usize) -> String {
    let mut lines = text.lines();
    let first = lines.next().unwrap_or_default().trim_end();
    if first.chars().count() > limit {
        let kept: String = first.chars().take(limit.saturating_sub(1)).collect();
        format!("{}…", kept.trim_end())
    } else if lines.next().is_some() {
        format!("{first} …")
    } else {
        first.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use eframe::Storage as _;

    use super::*;
    use crate::layout::tests::MemoryStorage;

    #[test]
    fn a_notice_has_time_left_only_before_its_mark() {
        assert_eq!(
            notice_left(Duration::from_secs(3)),
            Some(Duration::from_secs(5))
        );
        assert_eq!(notice_left(NOTICE_FOR), None);
        // Past the mark: the reading that used to underflow.
        assert_eq!(notice_left(NOTICE_FOR + Duration::from_millis(1)), None);
    }

    #[test]
    fn a_notice_keeps_one_line_cut_by_characters() {
        assert_eq!(one_line("short", 10), "short");
        assert_eq!(one_line("first\nsecond", 10), "first …");
        // Multibyte characters near the cut: counted, never split.
        let long = "é—…".repeat(40);
        let cut = one_line(&long, 10);
        assert_eq!(cut.chars().count(), 10, "{cut}");
        assert!(cut.ends_with('…') && cut.starts_with("é—…é"), "{cut}");
        assert_eq!(one_line("", 10), "");
    }

    #[test]
    fn closing_hides_asks_or_closes() {
        // With a tray the window hides, whatever runs: a revert goes on in the tray.
        assert_eq!(closing(false, false, true, 2, false), Closing::Hide);
        assert_eq!(closing(false, false, true, 0, true), Closing::Hide);
        // Without one, the question comes first when runs would stop.
        assert_eq!(closing(false, false, false, 1, false), Closing::Ask);
        assert_eq!(closing(false, false, false, 0, false), Closing::Close);
        // A revert that runs is waited for (EP-15c).
        assert_eq!(closing(false, false, false, 1, true), Closing::Wait);
        // A Quit asked for, and a screenshot run's own close, close.
        assert_eq!(closing(true, false, true, 2, false), Closing::Close);
        assert_eq!(closing(false, true, false, 3, false), Closing::Close);
    }

    #[test]
    fn the_view_keeps_finished_work_folded_at_first() {
        let view = View::default();
        assert!(view.is_folded(GroupKind::State(State::Done)));
        assert!(view.is_folded(GroupKind::State(State::Archived)));
        assert!(!view.is_folded(GroupKind::State(State::Active)));
        assert!(!view.is_folded(GroupKind::Unadopted));
        assert!(!view.legend);
    }

    #[test]
    fn a_view_saved_before_the_legend_reads_with_it_closed() {
        let mut storage = MemoryStorage::default();
        let open = View {
            legend: true,
            ..View::default()
        };
        eframe::set_value(&mut storage, VIEW_KEY, &open);
        let saved = storage.get_string(VIEW_KEY).unwrap();
        assert!(saved.contains("legend:true"), "{saved}");
        assert_eq!(stored::<View>(Some(&storage), VIEW_KEY), Some(open));
        // What a build before EP-7a wrote: no such field.
        storage.set_string(VIEW_KEY, saved.replace(",legend:true", ""));
        let older: View = stored(Some(&storage), VIEW_KEY).unwrap();
        assert_eq!(older, View::default());
    }
}
