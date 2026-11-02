//! The window (D-25): a menu bar, the docking layout of panes over one shared [`Board`], and a
//! status bar. The panes queue what the owner asks for as [`Action`]s, which run after the frame
//! is drawn: the writes the tool owns (D-7, D-8, D-10, D-11, D-13), each followed by a new read of
//! the files, the moves of the selection, and from EP-10 the launch dialog and the headless
//! sessions it starts (D-16, D-17, D-22), from EP-11 its visual launches; from EP-12 the sessions
//! the hooks and the poll of live sessions report join the window's own (D-20).

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crw_core::badges::hosting::Class;
use crw_core::badges::short_age;
use crw_core::badges::stack::Kind;
use crw_core::catalog::Catalog;
use crw_core::discovery::{self, Listing, OwnFile, OwnFiles};
use crw_core::proposal::{self, Reason};
use crw_core::registry::{State, Store};
use crw_core::roadmap::snapshot::InProgress;
use crw_core::session::cli;
use crw_core::sessions::Registered;
use crw_core::{files, inbox, prompt, tasks};
use eframe::egui::{self, Key, KeyboardShortcut, Modifiers, RichText, ThemePreference};
use egui_tiles::Tree;
use serde::{Deserialize, Serialize};

use crate::launch::{Dialog, Opening};
use crate::layout::{self, Pane};
use crate::portfolio::{Filter, GroupKind, Loader, Portfolio, Selection, SortKey, step};
use crate::sessions::{Sessions, Started};
use crate::style::{self, Density, icons};
use crate::visual::{self, Visual};
use crate::watch::Watch;

const THEME_KEY: &str = "theme";
const VIEW_KEY: &str = "view";
/// A notice stays this long in the status bar; an error stays until the next notice.
const NOTICE_FOR: Duration = Duration::from_secs(8);
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
/// cached in, and whether the GitHub pass may ask the network (a screenshot run's never does).
pub struct Places {
    pub store: Store,
    pub ledger: Option<PathBuf>,
    pub online: bool,
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
    /// A key or a button of a brief not built yet: a notice that says which.
    NotYet(&'static str),
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
    /// Ticks are refused: the projects are a screenshot run's tracked fixtures.
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
    /// Where Claude Code caches its model catalog; a screenshot run's is a fixture.
    pub catalog_folder: Option<PathBuf>,
    /// A launch dialog to open once the files are read (a screenshot run's `launch=`).
    pub open_when_read: Option<String>,
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
            catalog_folder: Catalog::folder(),
            open_when_read: None,
            ledger,
            ctx: ctx.clone(),
            notice: None,
            error: None,
            scroll_to_selected: false,
            bring: None,
            settle_until: None,
        }
    }

    pub fn store(&self) -> &Store {
        self.loader.store()
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

    pub fn notify(&mut self, text: impl Into<String>) {
        self.notice = Some((text.into(), Tone::Plain, Instant::now()));
        self.error = None;
    }

    fn warn(&mut self, text: impl Into<String>) {
        self.notice = Some((text.into(), Tone::Warning, Instant::now()));
    }

    fn fail(&mut self, text: impl Into<String>) {
        self.error = Some(text.into());
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
            Some(Started::Running(text)) => {
                self.launch = None;
                self.notify(text);
                self.bring = Some(Pane::Sessions);
            }
            Some(Started::Refused(why)) => {
                // The CLI may have changed under the tool: the next launch asks it again.
                self.sessions.forget_preflight();
                let why = format!("The session did not start: {why}");
                match self.launch.as_mut() {
                    Some(dialog) => dialog.error = Some(why),
                    None => self.fail(why),
                }
            }
            None => {}
        }
        match self.visual.receive(ctx) {
            Some(visual::Outcome::Opened(text)) => {
                self.launch = None;
                self.notify(text);
            }
            Some(visual::Outcome::Refused(why)) => match self.launch.as_mut() {
                Some(dialog) => dialog.error = Some(why),
                None => self.fail(why),
            },
            None => {}
        }
        let portfolio = &self.portfolio;
        let registered = || {
            portfolio
                .projects
                .iter()
                .filter_map(|row| {
                    Some(Registered {
                        name: row.name().to_owned(),
                        folder: row.folder.as_ref()?.path.clone(),
                    })
                })
                .collect()
        };
        let focused = ctx.input(|i| i.focused);
        self.watch
            .receive(registered, self.ledger.as_deref(), focused);
        if let Some(name) = &self.open_when_read
            && !self.loader.reading_files
            && self.portfolio.project(name).is_some()
        {
            let name = name.clone();
            self.open_when_read = None;
            self.actions.push(Action::OpenLaunch(name));
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

    /// The window's keys (D-25), unless a text field has the keyboard or the launch dialog is
    /// open: ↑ ↓ select, → ← expand and fold the row, Enter opens the Detail pane, S opens the
    /// launch dialog, A says what is not built yet, F5 refreshes.
    pub fn keys(&mut self, ctx: &egui::Context) {
        if self.launch.is_some() {
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
            self.actions
                .push(Action::NotYet("The autopilot arrives at EP-19."));
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
            Action::NotYet(text) => self.warn(text),
            Action::Adopt(name) => {
                // A screenshot run's data root is the tracked fixtures: no file beside them.
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
                        self.notify(format!(
                            "Ticked {id} in {project} with its disposition; the file as it was \
                             is kept as {}.",
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
                "A screenshot run writes nothing into its fake projects: {project} is a tracked \
                 fixture."
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
            Ok(copy) => self.notify(format!(
                "{} {id} in {project}; the file as it was is kept as {}.",
                if checked { "Ticked" } else { "Cleared" },
                copy.display()
            )),
            Err(e) => self.fail(format!("{project} {id}: {e}")),
        }
        true
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
        let program = cli::locate(registry.settings.claude_cli.as_deref())
            .unwrap_or_else(|| PathBuf::from("claude.exe"));
        let levels: Vec<String> = self
            .sessions
            .preflight(&program, &self.ctx)
            .and_then(|preflight| preflight.flags.as_ref().ok().map(|f| f.efforts.clone()))
            .unwrap_or_default();
        let claude_md = prompt::claude_md(&folder);
        let today = files::today();
        let draft = match prompt::draft(
            &registry,
            &project,
            &snapshot,
            claude_md.as_deref(),
            &levels,
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
            },
        ));
    }

    /// Hands the dialog's prompt to VS Code (D-16, EP-11), off the UI thread, since the project's
    /// window may take seconds to start; a screenshot run opens nothing (rule 5).
    fn start_visual(&mut self) {
        if self.fixtures {
            self.warn("A screenshot run opens nothing: its projects are tracked fixtures.");
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
            self.warn("A screenshot run starts no session: its projects are tracked fixtures.");
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
        let launch = dialog.launch(self.ledger.clone());
        let effort = dialog.effort_words().to_owned();
        self.sessions.start(preflight, launch, effort, &self.ctx);
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
                && at.elapsed() < NOTICE_FOR
            {
                let text = RichText::new(text);
                ui.label(match tone {
                    Tone::Plain => text,
                    Tone::Warning => text.color(style::amber(ui)),
                });
                ui.ctx().request_repaint_after(NOTICE_FOR - at.elapsed());
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
                ui.label(RichText::new(reading).weak()).on_hover_text(format!(
                    "Registry {}\nData root {}\n{github}\n{signals}",
                    self.portfolio.registry.display(),
                    self.portfolio.data_root.display()
                ));
                if let Some(note) = &self.home_note {
                    style::chip(ui, note, style::amber(ui));
                }
            });
        });
        ui.add_space(2.0);
    }
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
        Self {
            layout: layout::restore(cc.storage),
            board: Board::new(places, &cc.egui_ctx, view, theme),
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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        #[cfg(feature = "screenshot")]
        crate::screenshot::tick(&ctx);
        self.board.receive(&ctx);
        self.board.keys(&ctx);
        #[cfg(feature = "screenshot")]
        crate::screenshot::set_busy(self.board.busy());

        egui::Panel::top("menu").show(ui, |ui| self.menu(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.board.status_bar(ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(0.0))
            .show(ui, |ui| self.layout.ui(&mut self.board, ui));
        self.board.launch_dialog(&ctx);
        self.board.run_actions(&mut self.layout);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, layout::LAYOUT_KEY, &self.layout);
        eframe::set_value(storage, THEME_KEY, &self.board.theme);
        eframe::set_value(storage, VIEW_KEY, &self.board.view);
    }

    #[cfg(feature = "screenshot")]
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        crate::screenshot::inject_keys(raw_input);
    }
}

#[cfg(test)]
mod tests {
    use eframe::Storage as _;

    use super::*;
    use crate::layout::tests::MemoryStorage;

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
