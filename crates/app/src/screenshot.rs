//! Developer aid, compiled only with `--features screenshot` (CLAUDE.md rule 5).
//!
//! With `CRW_SCREENSHOT=<file.png>` set, the app runs on a fake portfolio, saves its own window
//! contents as a PNG once the projects are read, and closes. Nothing outside the app's window is
//! captured. The run's home is `CRW_HOME` when set, else a `crw-screenshot` folder beside the
//! picture: the registry (written there from `fixtures/registry.toml` on the first run, so a
//! second run starts from what the first one changed), eframe's state and the backups. The fake
//! projects are the tracked folders of `fixtures/portfolio`, which a run never writes into: ticks
//! are refused with a notice.
//!
//! `CRW_SCREENSHOT_ROOT=<folder>`, or the setup flag `root=<folder>`, which wins, names another
//! data root (EP-36). Its use is a scratch copy of the real roadmap files, made with
//! `scripts\copy-roadmap.ps1` one project at a time, so that a session checks a layout on real
//! content. A root that holds every folder of the fake portfolio (a copy of it, or another path
//! to it) reads with the fake registry. Any other root reads with one written from its folders,
//! each active with a roadmap and unformalised without ([`registry_for`]). The guards are the
//! same, and the run writes nothing into the root. A relative root is resolved against the run's
//! working folder, since a registry's data root is a full path. The README's pictures name a
//! neutral root, a copy of the fixtures in a folder whose path names nothing of the machine
//! (`C:\Users\Public\Projects`, removed after the run), in place of EP-8's `subst` drive and
//! junction. A home keeps the root its registry was first written for, and a run that names
//! another is refused.
//!
//! `CRW_SCREENSHOT_SETUP` (comma-separated) arranges the view:
//!
//! - `root=<folder>` (the data root, as above);
//! - `light` / `dark` (the theme); `compact` / `comfortable` (the density); `legend` (the key
//!   to the row's marks open under the list);
//! - `select=<project or folder>` (the selection); `expand=<project>` (its row expanded inline);
//! - `tile=<pane>` (bring `projects`, `detail`, `decisions`, `sessions`, `chat` or `usage` to
//!   the front of its group);
//! - `keys=<chord>;<chord>…` (key presses such as `Down`, `Enter` or `ctrl+F`, key names as
//!   egui spells them, fed one per frame through eframe's input hook as if typed);
//! - `adopt=<folder>`, `ignore=<folder>`, `pin=<project>`, `tick=<project>:<OT-n>` (the action a
//!   click queues, through the same path: the first three write the run's registry, the tick is
//!   refused);
//! - `sort=<key>` (`priority`, `completion`, `remaining`, `last-activity`, `hosting`,
//!   `usage-share`, `name`);
//!   `stack=<stack>` (`rust`, `python`, … or the chip's letters) and `hosting=<class>` (`public`,
//!   `private`, `remote`, `local`), each adding to its filter;
//! - `launch=<project>` (the launch dialog open on the project's next brief, once it is read;
//!   with `usage`, lantern's EP-3, an S launched visually, shows the Fable downshift, EP-18);
//! - `sessions` (canned sessions that Claude Code reports: lantern's visual one waiting on a
//!   permission prompt, orchard's manual one busy with its status line's figures though every
//!   row of orchard's is checked, beacon's tab open with no prompt yet, one outside the registry,
//!   and the signals listening with their entries installed; from EP-13 also a headless harbor
//!   EP-4 of the window's with a transcript, and ledger rows of every mode and outcome);
//!   `sessions=<project>` also filters the All tab to that project, so that its ledger cards and
//!   their Revert… show in the narrow pane (EP-15c);
//! - `transcript` (the canned sessions, the Sessions pane in front with harbor's tab open);
//!   `transcript=<project>` opens that project's canned tab instead (quarry's run has ended,
//!   deferred, with its Revert…, EP-15c);
//! - `decisions` (the canned sessions, the Decisions pane in front: from EP-14 harbor's session
//!   waits on a pair of questions and a `git push`, under a 30-minute limit; a quarry EP-2 run
//!   was deferred at its limit with a question open; lantern's visual session waits in VS Code;
//!   harbor's refused edit of a neighbour);
//! - `quit` (the canned sessions, and the question Quit asks before it stops harbor's run, from
//!   EP-15); `restart` (the canned sessions, and File > Install and restart refused with its
//!   reasons in the status bar, from EP-37);
//!   `popup` (the notifications' popup with canned toasts, drawn inside the window: a
//!   real one is a window of its own at the screen's lower right);
//! - `revert=<project>` (the preview of undoing the project's latest session with a canned plan,
//!   from EP-15c: its session from the canned ledger rows when `sessions` comes first; no git is
//!   asked, and its Revert reverts nothing); `revert=<project>:refused` (the same plan refused,
//!   with Close alone) and `revert=<project>:done` (what the revert did);
//! - `usage` (the usage meter's strip and pane with a canned reading, from EP-17: weekly 41 %,
//!   Fable past its 80 % mark, a five-hour window open, the week by surface, a status line and a
//!   week of samples across the weekly reset; from EP-18b also the week's shares of harbor,
//!   lantern, quarry and orchard, orchard's `+0 %`, in the list's Usage column and the Detail
//!   pane); `usage=degraded` (the same with the endpoint's last poll failed and a newer status
//!   line); `tile=usage` brings its pane forward; `sort=usage-share` orders by the shares;
//! - `autopilot` (from EP-19: the canned sessions, the Sessions pane's All tab in front with the
//!   Autopilot section: harbor's autopilot turned on in the run's registry, a chain held by the
//!   Fable figure, and a canned log of a chained run, a cap, a re-plan and a project whose runs
//!   would wait); `autopilot=<project>` turns that project's on instead of harbor's; harbor's
//!   canned run is a chained one with its cap, and a beacon EP-2 run of the window's has
//!   completed under the rule that stops before committing: `transcript=beacon` after it shows
//!   that tab's Commit button; from EP-19b the section's line per project: harbor turned on as
//!   it is (its runs would wait, so the gate's refusal shows) and quarry with permission mode
//!   `auto`, each with its limits;
//! - `offer=<project>` (from EP-19b: the project's autopilot box ticked, which opens the offer of
//!   `auto`, since a fixture's runs would wait: the run reads no user settings);
//!   `keep=<project>:<mode>` (the permission mode kept for the project, as the launch dialog's
//!   Keep menu writes it, before `launch=` opens the dialog);
//!   `keeprule=<project>:<model|effort>[@<phase>]=<value>` (from EP-19c: a model or an effort
//!   kept as the Keep menu beside Model or Effort writes it, for every brief or for one phase,
//!   into the run's registry, before `launch=` opens the dialog);
//! - `size=<width>x<height>` (the window's inner size in points; 1600x1000 by default).
//!
//! The fake projects are not repositories and have no remote, so their git, GitHub and backup
//! readings come from [`FAKES`], set against the run's own clock ([`readings`]); their stack chips
//! and licenses are read from the fixtures' own files. The projects of a copied portfolio take the
//! fakes by their place in the list, under their own names. A run never starts the GitHub pass. The
//! launch dialog reads the synthetic model catalog in `fixtures/model-catalog`, a canned
//! preflight and a canned VS Code with no window open, never the real CLI, VS Code or its lock
//! files, and a run starts no session and opens nothing. In the fixture registry harbor launches
//! headless and lantern visually. A run binds no port, polls no live session and reads no Claude
//! Code settings (EP-12); its usage meter is off, or canned with `usage`, and never reads the
//! credentials or asks the endpoint (EP-17).

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use crw_core::badges::activity::Activity;
use crw_core::badges::backup::{Archive, Backup};
use crw_core::badges::health;
use crw_core::badges::hosting::{self, Check, Class, Listed, Repo, Verdict, Visibility};
use crw_core::badges::stack::Kind;
use crw_core::inbox::Recipient;
use crw_core::inbox::mailman::{Mark, State as MailState};
use crw_core::ledger::{RowUsage, SessionRow};
use crw_core::registry::{DEFAULT_PRIORITY, Defaults, State, Store};
use crw_core::revert::{
    Action as FileAction, Change, Commit, Done, Fate, FileState, Plan, Start, StartFile, Target,
    Tick,
};
use crw_core::session::{
    Choice, Denied, Ended, Event, Init, Launch, Outcome, Question, Request, ToolResult, ToolUse,
};
use crw_core::sessions::{Mode, Seen, Status};
use crw_core::signals::{self, StatusLine};
use crw_core::{discovery, paths};
use eframe::egui;

use crate::board::{Action, Board, Dashboard};
use crate::layout::{self, Pane};
use crate::portfolio::{Fetched, Portfolio, Selection, SortKey};
use crate::server::Listening;
use crate::sessions::Live;
use crate::style::Density;

pub const PICTURE_VAR: &str = "CRW_SCREENSHOT";
pub const SETUP_VAR: &str = "CRW_SCREENSHOT_SETUP";
/// The data root a run reads in place of the fake portfolio (EP-36); the setup flag `root=` wins.
pub const ROOT_VAR: &str = "CRW_SCREENSHOT_ROOT";

/// The window of a screenshot run, unless `size=` says otherwise.
const SIZE: [f32; 2] = [1600.0, 1000.0];
/// The capture waits for this many frames and this much time, so that layout and fonts have
/// settled on a fast display too, and for the projects to be read.
const CAPTURE_AT_FRAME: u32 = 10;
const CAPTURE_AFTER_SECONDS: f64 = 1.0;
/// Reads that take longer than this are not waited for.
const WAIT_AT_MOST_SECONDS: f64 = 30.0;
/// Injected keys start here, one chord per frame.
const KEYS_FROM_FRAME: u32 = 4;
static FRAMES: AtomicU32 = AtomicU32::new(0);
static CAPTURING: AtomicBool = AtomicBool::new(false);
static BUSY: AtomicBool = AtomicBool::new(true);

/// A screenshot run: its home folder, its setup and the data root it reads; [`tick`] reads the
/// picture's path.
pub struct Run {
    pub home: PathBuf,
    pub flags: Vec<String>,
    /// The root named by `root=` or [`ROOT_VAR`] (EP-36); the fake portfolio when `None`.
    pub root: Option<PathBuf>,
}

impl Run {
    /// The run `CRW_SCREENSHOT` asks for; `None` outside one.
    pub fn from_env() -> Option<Run> {
        let picture = PathBuf::from(std::env::var_os(PICTURE_VAR)?);
        let home = paths::home().unwrap_or_else(|| {
            picture
                .parent()
                .unwrap_or(Path::new("."))
                .join("crw-screenshot")
        });
        let flags = flags(&std::env::var(SETUP_VAR).unwrap_or_default());
        let root = root_of(&flags, std::env::var_os(ROOT_VAR));
        Some(Run { home, flags, root })
    }

    /// The data root the run reads: the one named, a relative one resolved against the run's
    /// working folder (a registry's data root is a full path), else the tracked fake portfolio.
    pub fn root(&self) -> PathBuf {
        match &self.root {
            Some(root) => std::path::absolute(root).unwrap_or_else(|_| root.clone()),
            None => portfolio_root(),
        }
    }

    /// The run's registry in the home folder, written for its root ([`registry_for`]) when the
    /// home has none yet. Refused when the root is not a folder, and when the home's registry
    /// reads another root, which would show a portfolio the run did not name.
    pub fn store(&self) -> io::Result<Store> {
        let root = self.root();
        if !root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("the data root {} is not a folder", root.display()),
            ));
        }
        let path = self.home.join("registry.toml");
        if !path.exists() {
            let text = registry_for(&root)?;
            fs::create_dir_all(&self.home)?;
            fs::write(&path, text)?;
        }
        let store = Store::new(
            path,
            self.home.join("backups"),
            Defaults {
                repository: root.join("harbor"),
                data_root: root.clone(),
            },
        );
        if let Ok(loaded) = store.load() {
            let read = loaded.registry.data_root();
            if !paths::same(&read, &root) {
                return Err(io::Error::other(format!(
                    "its registry reads the data root {}, not {}: start from a fresh home",
                    read.display(),
                    root.display()
                )));
            }
        }
        Ok(store)
    }

    /// eframe's state file of the run, in its home.
    pub fn state_file(&self) -> PathBuf {
        self.home.join("app.ron")
    }

    pub fn size(&self) -> [f32; 2] {
        self.value("size")
            .and_then(|value| {
                let (w, h) = value.split_once('x')?;
                Some([w.trim().parse().ok()?, h.trim().parse().ok()?])
            })
            .unwrap_or(SIZE)
    }

    fn value(&self, name: &str) -> Option<&str> {
        self.flags.iter().find_map(|flag| {
            flag.strip_prefix(name)
                .and_then(|rest| rest.strip_prefix('='))
        })
    }

    /// Applies the setup to the window just made.
    pub fn arrange(&self, dashboard: &mut Dashboard, ctx: &egui::Context) {
        let board = dashboard.board_mut();
        board.fixtures = true;
        board.home_note = Some("screenshot run".to_owned());
        board.sessions.canned = Some(Arc::new(crate::launch::canned_preflight()));
        board.visual.canned = Some((
            PathBuf::from(r"C:\Tools\VS Code\Code.exe"),
            crw_core::version::Version::new(2, 1, 314),
        ));
        board.catalog_folder = Some(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("fixtures")
                .join("model-catalog"),
        );
        for flag in &self.flags {
            let (name, value) = flag.split_once('=').unwrap_or((flag.as_str(), ""));
            let board = dashboard.board_mut();
            match name {
                "light" => {
                    board.theme = egui::ThemePreference::Light;
                    ctx.set_theme(board.theme);
                }
                "dark" => {
                    board.theme = egui::ThemePreference::Dark;
                    ctx.set_theme(board.theme);
                }
                "compact" | "comfortable" => {
                    board.view.density = if name == "compact" {
                        Density::Compact
                    } else {
                        Density::Comfortable
                    };
                    board.view.density.apply(ctx);
                }
                "legend" => board.view.legend = true,
                "select" => {
                    // A name the registry lists is a project; anything else is taken for a
                    // folder of the data root (the portfolio is not read yet).
                    let registered = board
                        .store()
                        .load()
                        .ok()
                        .is_some_and(|loaded| loaded.registry.project(value).is_some());
                    board.view.selected = Some(if registered {
                        Selection::Project(value.to_owned())
                    } else {
                        Selection::Folder(value.to_owned())
                    });
                    board.scroll_to_selected = true;
                }
                "expand" => {
                    board.view.expanded.insert(value.to_owned());
                }
                "launch" => {
                    // `launch=harbor:headless` opens the dialog on Headless, where the chain
                    // plan shows (EP-19a).
                    let (project, mode) = value.split_once(':').unwrap_or((value, ""));
                    board.open_when_read = Some(project.to_owned());
                    board.open_headless = mode == "headless";
                }
                "sessions" => {
                    canned_sessions(board);
                    if !value.is_empty() {
                        board.view.sessions.project = Some(value.to_owned());
                    }
                }
                "transcript" => {
                    if board.sessions.live.is_empty() {
                        canned_sessions(board);
                    }
                    let project = if value.is_empty() { "harbor" } else { value };
                    board.sessions.tab = board
                        .sessions
                        .live
                        .iter()
                        .find(|live| live.project == project)
                        .map(|live| live.key);
                    layout::show(dashboard.layout_mut(), Pane::Sessions);
                }
                "decisions" => {
                    if board.sessions.live.is_empty() {
                        canned_sessions(board);
                    }
                    layout::show(dashboard.layout_mut(), Pane::Decisions);
                }
                "quit" => {
                    if board.sessions.live.is_empty() {
                        canned_sessions(board);
                    }
                    board.quit_asked = true;
                }
                // File > Install and restart over the canned sessions (EP-37): refused, its
                // reasons in the status bar.
                "restart" => {
                    if board.sessions.live.is_empty() {
                        canned_sessions(board);
                    }
                    board.actions.push(Action::InstallAndRestart);
                }
                "popup" => {
                    ctx.set_embed_viewports(true);
                    for toast in crate::toasts::canned() {
                        board.toasts.popup.push(toast);
                    }
                }
                "usage" => {
                    board.meter = crate::meter::canned(chrono::Utc::now(), value == "degraded");
                }
                "autopilot" => {
                    if board.sessions.live.is_empty() {
                        canned_sessions(board);
                    }
                    let project = if value.is_empty() { "harbor" } else { value };
                    canned_autopilot(board, project);
                    board.sessions.tab = None;
                    layout::show(dashboard.layout_mut(), Pane::Sessions);
                }
                "offer" => board
                    .actions
                    .push(Action::SetAutopilot(value.to_owned(), true)),
                "keep" => match value.split_once(':') {
                    Some((project, mode)) => board.actions.push(Action::KeepPermissionMode(
                        project.to_owned(),
                        Some(mode.to_owned()),
                    )),
                    None => eprintln!("screenshot: keep takes <project>:<mode>, not {value}"),
                },
                "keeprule" => match keeprule_action(value) {
                    Some(action) => board.actions.push(action),
                    None => eprintln!(
                        "screenshot: keeprule takes <project>:<model|effort>[@<phase>]=<value>, \
                         not {value}"
                    ),
                },
                "revert" => {
                    let (project, form) = value.split_once(':').unwrap_or((value, ""));
                    let rows = &board.sessions.history.rows;
                    let plan = canned_plan(project, None, rows, form == "refused");
                    let done = (form == "done").then(|| canned_done(&plan));
                    let mut dialog = crate::revert::Dialog::ready(plan);
                    if let Some(done) = done {
                        dialog.stage = crate::revert::Stage::Done(done);
                    }
                    board.revert = Some(dialog);
                }
                "tile" => match Pane::from_name(value) {
                    Some(pane) => layout::show(dashboard.layout_mut(), pane),
                    None => eprintln!("screenshot: no pane called {value}"),
                },
                // The same actions a click queues, run after the first frame.
                "adopt" => board.actions.push(Action::Adopt(value.to_owned())),
                "ignore" => board.actions.push(Action::Ignore(value.to_owned())),
                "pin" => board
                    .actions
                    .push(Action::SetPinned(value.to_owned(), true)),
                "tick" => match value.split_once(':') {
                    Some((project, id)) => board.actions.push(Action::Tick {
                        project: project.to_owned(),
                        id: id.to_owned(),
                    }),
                    None => eprintln!("screenshot: tick takes <project>:<id>, not {value}"),
                },
                "sort" => {
                    let wanted = value.replace('-', " ");
                    match SortKey::ALL
                        .into_iter()
                        .find(|key| key.label().eq_ignore_ascii_case(&wanted))
                    {
                        Some(key) => board.view.sort = key,
                        None => eprintln!("screenshot: no sort key called {value}"),
                    }
                }
                "stack" => match Kind::ALL.into_iter().find(|kind| {
                    kind.label().eq_ignore_ascii_case(value)
                        || kind.short().eq_ignore_ascii_case(value)
                }) {
                    Some(kind) => {
                        board.view.stacks.insert(kind);
                    }
                    None => eprintln!("screenshot: no stack called {value}"),
                },
                "hosting" => match Class::ALL
                    .into_iter()
                    .find(|class| format!("{class:?}").eq_ignore_ascii_case(value))
                {
                    Some(class) => {
                        board.view.hosting.insert(class);
                    }
                    None => eprintln!("screenshot: no hosting class called {value}"),
                },
                "keys" | "size" | "root" => {} // read by `inject_keys`, `main` and `from_env`
                _ => eprintln!("screenshot: unknown setup flag {flag}"),
            }
        }
    }
}

/// The sessions a picture shows Claude Code reporting (EP-12), set against the run's own clock.
/// The autopilot as a screenshot shows it (EP-19): `project`'s turned on in the run's own
/// registry, a chain held by the Fable figure, a canned log, and harbor's canned tab with its cap,
/// ended so that its Commit shows. Nothing is read from or written to a ledger.
/// A chain plan after `project`'s EP-4 (EP-19a): EP-5 at the owner's choice, Opus 5.5 at max,
/// and the chain ending at EP-6, a re-plan; the rules' resolution over an M as its default.
fn canned_chain_plan(project: &str) -> crw_core::autopilot::Plan {
    use crw_core::autopilot::{Plan, Planned, Stop};
    use crw_core::registry::{Project, Registry, State};
    use crw_core::roadmap::Size;
    use crw_core::rules::{self, Picked};

    let entry = Project::new(project, Path::new(project), State::Active);
    let registry = Registry {
        format: 1,
        ignore: Vec::new(),
        settings: Default::default(),
        rules: Vec::new(),
        projects: vec![entry.clone()],
    };
    let brief = rules::Brief {
        size: Some(Size::M),
        phase: Some("P1"),
        title: "Season fees",
    };
    let resolution = rules::resolve(
        &registry,
        &entry,
        &brief,
        &[],
        &rules::Usage::unread(Utc::now()),
    );
    Plan {
        project: project.to_owned(),
        after: "EP-4".parse().expect("an EP id"),
        briefs: vec![Planned {
            ep: "EP-5".parse().expect("an EP id"),
            title: "Season fees: rates by length band, invoices as PDF".to_owned(),
            size: Some(Size::M),
            resolution,
            due: Vec::new(),
            chain: true,
            picked: Some(Picked {
                model: "claude-opus-5-5".to_owned(),
                effort: Some("max".to_owned()),
                ultracode: false,
            }),
        }],
        stop: Some(Stop {
            ep: "EP-6".parse().expect("an EP id"),
            why: "EP-6 is a re-plan, the owner's to run".to_owned(),
        }),
        daily: 2,
    }
}

fn canned_autopilot(board: &mut Board, project: &str) {
    use crw_core::autopilot::{self, Cap, EVENT_KIND};
    use crw_core::ledger::EventRow;
    use crw_core::roadmap::Size;

    board.autopilot.canned = true;
    // Turned on as it is, past the offer of `auto` (EP-19b), so that the section shows the
    // gate's refusal of a project whose runs would wait; quarry beside it with `auto`.
    board.actions.push(Action::AutopilotOn {
        project: project.to_owned(),
        auto: false,
    });
    if !project.eq_ignore_ascii_case("quarry") {
        board.actions.push(Action::AutopilotOn {
            project: "quarry".to_owned(),
            auto: true,
        });
    }
    let now = Utc::now();
    let row = |minutes: i64, project: &str, words: &str| EventRow {
        id: 0,
        at: (now - TimeDelta::minutes(minutes)).to_rfc3339(),
        kind: EVENT_KIND.to_owned(),
        project: (!project.is_empty()).then(|| project.to_owned()),
        detail: Some(words.to_owned()),
    };
    let held = "held after EP-2 until the pause lifts: Fable is at 91 %, at or over the pause at \
                90 % until its reset, Sat 07 Nov 18:00";
    board.autopilot.log = vec![
        row(3, "quarry", held),
        row(
            21,
            "orchard",
            "the chain ends after EP-6: its runs would wait for you at each shell command: \
             permission mode acceptEdits and no allow rule (keep auto with the pin beside the \
             launch dialog's Permissions, crw set <project> permission_mode auto, or allow rules \
             in its .claude settings)",
        ),
        row(
            48,
            project,
            "the chain ends after EP-5: EP-6 is a re-plan, the owner's to run",
        ),
        row(
            102,
            project,
            "chained EP-5 after EP-4: claude-opus-5-5 at max, your choice, M's cap of 1 h 30 min",
        ),
        row(
            140,
            "lantern",
            "capped EP-3: stopped after 45 min of work, S's cap of 45 min; the chain ends there",
        ),
        row(160, "", autopilot::RESUMED_BY_OWNER),
    ];
    board.autopilot.held = vec![crate::autopilot::Held {
        project: "quarry".to_owned(),
        after: "EP-2".parse().ok(),
        words: held.to_owned(),
    }];
    // The plan the chain follows (EP-19a): the owner's choice for EP-5, the chain ending at the
    // re-plan.
    board.autopilot.keep_plan(canned_chain_plan(project));
    if let Some(live) = board
        .sessions
        .live
        .iter_mut()
        .find(|live| live.project == "harbor" && live.ended.is_none())
    {
        let mut cap = Cap::new(Some(Size::M), Instant::now() - Duration::from_secs(52 * 60));
        cap.clock.tick(false, Instant::now());
        live.cap = Some(cap);
        live.chained = true;
    }
    if !board
        .sessions
        .live
        .iter()
        .any(|live| live.project == "beacon")
    {
        use crw_core::session::{Ended, Event, Launch, Outcome};
        let mut launch = Launch::new(
            "beacon",
            Path::new("beacon"),
            "fable",
            "Run the next brief of beacon: EP-2.",
        );
        launch.ep = Some("EP-2".to_owned());
        let text = "EP-2's brief has its completion note; the commit waits for your yes.";
        let mut done =
            crate::sessions::Live::canned(90, launch, "high", vec![Event::Text(text.to_owned())]);
        done.end_canned(Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        });
        board.sessions.live.push(done);
    }
}

/// What `keeprule=<project>:<model|effort>[@<phase>]=<value>` asks for (EP-19c): a model or an
/// effort kept for every brief of the project, or for one phase alone, as the launch dialog's
/// Keep menu keeps it; `None` for any other form.
fn keeprule_action(value: &str) -> Option<Action> {
    use crw_core::rules::{Key, Scope};

    let (project, rest) = value.split_once(':')?;
    let (key, kept) = rest.split_once('=')?;
    let (key, scope) = match key.split_once('@') {
        Some((key, phase)) => (key, Scope::Phase(phase.to_owned())),
        None => (key, Scope::Project),
    };
    let key = match key {
        "model" => Key::Model,
        "effort" => Key::Effort,
        _ => return None,
    };
    Some(Action::KeepRule {
        project: project.to_owned(),
        scope,
        key,
        value: Some(kept.to_owned()),
    })
}

fn canned_sessions(board: &mut Board) {
    let now = Utc::now();
    let ago = |minutes| now - TimeDelta::minutes(minutes);
    let mut lantern = Seen::canned(
        "5f1c0a2e-lantern",
        Some("lantern"),
        Mode::Visual,
        Status::Waiting,
        Some("a permission prompt"),
        ago(12),
    );
    lantern.name = Some("lantern EP-6".to_owned());
    lantern.entrypoint = Some("claude-vscode".to_owned());
    // Every row of orchard's is checked: its Detail pane still says the session (EP-13).
    let mut orchard = Seen::canned(
        "9a3e7d41-orchard",
        Some("orchard"),
        Mode::Manual,
        Status::Busy,
        None,
        ago(41),
    );
    orchard.name = Some("Tidy the fixtures".to_owned());
    orchard.entrypoint = Some("cli".to_owned());
    orchard.line = Some(StatusLine {
        model: signals::Model {
            id: "claude-opus-5-5".to_owned(),
            display_name: "Opus".to_owned(),
        },
        cost: signals::Cost {
            total_cost_usd: Some(1.84),
            total_duration_ms: None,
        },
        context_window: signals::ContextWindow {
            used_percentage: Some(37.0),
            context_window_size: None,
        },
        ..StatusLine::default()
    });
    // A tab open with no prompt yet: new, no row and no session in progress (EP-13).
    let mut fresh = Seen::canned(
        "41d8e2c0-beacon",
        Some("beacon"),
        Mode::Manual,
        Status::Idle,
        None,
        ago(1),
    );
    fresh.worked = false;
    fresh.entrypoint = Some("claude-vscode".to_owned());
    let outside = Seen::canned("e7b0c3d9", None, Mode::Manual, Status::Idle, None, ago(5));
    for mut seen in [lantern, orchard, fresh, outside] {
        seen.heard = now;
        board.watch.model.insert(seen);
    }
    // The list of live sessions has answered, so a ledger card's Resume in VS Code shows as it
    // would (EP-35).
    board.watch.model.polled = Some(now);
    board.watch.listening = Some(Listening::On(signals::DEFAULT_PORT));
    board.watch.entries = Some(Ok("hooks installed, status line installed".to_owned()));
    board.sessions.live.insert(0, canned_deferred());
    board.sessions.live.insert(0, canned_transcript());
    board.sessions.history.rows = canned_rows(now);
    board.sessions.history.canned = true;
}

/// A question pair as harbor's canned session asks it (EP-14): one answer of two, then several
/// of three. Synthetic text only.
fn canned_questions() -> Vec<Question> {
    let choice = |label: &str, description: &str| Choice {
        label: label.to_owned(),
        description: Some(description.to_owned()),
    };
    vec![
        Question {
            text: "Which length band should the berth filter open on?".to_owned(),
            header: "Band".to_owned(),
            options: vec![
                choice("Under 12 m", "The small craft, most of the register"),
                choice("Any length", "Every berth, the filter off at first"),
            ],
            multi_select: false,
        },
        Question {
            text: "Which columns should the berth list keep?".to_owned(),
            header: "Columns".to_owned(),
            options: vec![
                choice("Length", "Overall length in metres"),
                choice("Draught", "The deepest point"),
                choice("Owner", "The holder of the berth"),
            ],
            multi_select: true,
        },
    ]
}

/// A quarry EP-2 run of the window's, deferred at its 30-minute limit with its question open
/// (EP-14). Synthetic text only.
fn canned_deferred() -> Live {
    let folder = portfolio_root().join("quarry");
    let mut launch = Launch::new(
        "quarry",
        &folder,
        "claude-opus-5-5",
        "You are working in quarry, launched from clauderoadmapwrangler: run EP-2, Seam map.",
    );
    launch.ep = Some("EP-2".to_owned());
    launch.answer_timeout = Some(Duration::from_secs(30 * 60));
    let question = Request {
        id: "req-q1".to_owned(),
        tool: "AskUserQuestion".to_owned(),
        summary: "Which seam should the map show first?".to_owned(),
        reason: None,
        questions: vec![Question {
            text: "Which seam should the map show first?".to_owned(),
            header: "Seam".to_owned(),
            options: vec![
                Choice {
                    label: "The upper seam".to_owned(),
                    description: Some("Surveyed last spring".to_owned()),
                },
                Choice {
                    label: "The lower seam".to_owned(),
                    description: Some("The older survey, in feet".to_owned()),
                },
            ],
            multi_select: false,
        }],
        input: String::new(),
    };
    let events = vec![
        Event::Text("The seam map needs one choice before the layout.".to_owned()),
        Event::Request(question),
        Event::TimedOut("req-q1".to_owned()),
    ];
    let mut live = Live::canned(901, launch, "medium", events);
    live.end_canned(Ended {
        outcome: Outcome::Deferred,
        exit_code: Some(0),
        result: None,
        stderr: Vec::new(),
    });
    live
}

/// A headless harbor EP-4 of the window's as it runs (EP-13): text, tool calls with their results,
/// a request withdrawn, a refused edit of a neighbour, and text streaming in. Synthetic text only.
fn canned_transcript() -> Live {
    let folder = portfolio_root().join("harbor");
    let mut launch = Launch::new(
        "harbor",
        &folder,
        "claude-opus-5-5",
        "You are working in harbor, launched from clauderoadmapwrangler: run EP-4, Ledger view.",
    );
    launch.ep = Some("EP-4".to_owned());
    launch.answer_timeout = Some(Duration::from_secs(30 * 60));
    let tool = |id: &str, name: &str, summary: &str| {
        Event::ToolUse(ToolUse {
            id: id.to_owned(),
            tool: name.to_owned(),
            summary: summary.to_owned(),
        })
    };
    let result = |id: &str, lines: usize, excerpt: &str| {
        Event::ToolResult(ToolResult {
            tool_use_id: id.to_owned(),
            is_error: false,
            lines,
            excerpt: excerpt.to_owned(),
        })
    };
    let events = vec![
        Event::Init(Init {
            version: Some("2.1.316".to_owned()),
            ..Init::default()
        }),
        Event::Text(
            "Reading the roadmap first: EP-4 is the next row, an L with its brief notes."
                .to_owned(),
        ),
        tool("t1", "Read", "Read roadmap/README.md"),
        result(
            "t1",
            41,
            "# harbor — Roadmap\n\nMaster roadmap for the berth register.",
        ),
        tool("t2", "Bash", "Bash cargo test"),
        result(
            "t2",
            31,
            "running 24 tests\ntest berths::reads_the_register ... ok\ntest result: ok. 24 passed; 0 failed",
        ),
        Event::Text(
            "The berth list needs a filter by length band; its test comes first.".to_owned(),
        ),
        tool("t3", "Edit", "Edit src/view/list.rs"),
        result("t3", 1, "The file src/view/list.rs has been updated."),
        Event::Denied(Denied {
            tool: "Edit".to_owned(),
            message: "Edit to the neighbour lantern is denied".to_owned(),
            reason: Some("deny rule".to_owned()),
        }),
        Event::Text("Two choices for the filter first, then the push.".to_owned()),
        Event::Request(Request {
            id: "req-2".to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which length band should the berth filter open on?".to_owned(),
            reason: None,
            questions: canned_questions(),
            input: String::new(),
        }),
        tool("t4", "Bash", "Bash git push origin main"),
        Event::Request(Request {
            id: "req-3".to_owned(),
            tool: "Bash".to_owned(),
            summary: "Bash git push origin main".to_owned(),
            reason: Some("Not in the allow list".to_owned()),
            questions: Vec::new(),
            input: "{\n  \"command\": \"git push origin main\",\n  \"description\": \"Push the \
                    filter's branch\"\n}"
                .to_owned(),
        }),
    ];
    let mut live = Live::canned(900, launch, "high", events);
    // Asked two minutes ago, under the project's 30-minute limit.
    let asked = Instant::now()
        .checked_sub(Duration::from_secs(120))
        .unwrap_or_else(Instant::now);
    for waiting in &mut live.waiting {
        waiting.since = asked;
    }
    live
}

/// The ledger's rows a picture shows (EP-13), one of each mode and outcome, the latest first.
fn canned_rows(now: DateTime<Utc>) -> Vec<SessionRow> {
    let time = |at: DateTime<Utc>| at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let row = |id: i64, project: &str, ep: Option<&str>, mode: &str, start: i64, minutes: i64| {
        let started = now - TimeDelta::minutes(start);
        SessionRow {
            id,
            session_id: Some(format!("{id:08x}-0000-4000-8000-00000000000{id}")),
            project: project.to_owned(),
            ep: ep.map(str::to_owned),
            mode: mode.to_owned(),
            model: None,
            effort: None,
            ultracode: false,
            started_at: time(started),
            ended_at: Some(time(started + TimeDelta::minutes(minutes))),
            outcome: Some("completed".to_owned()),
            cost_usd: None,
            questions: 0,
            result: None,
            context_percent: None,
            answer_latency_s: None,
            start: None,
            usage: RowUsage::default(),
        }
    };
    // Each session's share of the usage (EP-18a): a manual one counts too.
    let share = |weekly: f64, before: f64, after: f64, fable: Option<f64>| RowUsage {
        weekly_share: Some(weekly),
        weekly_before: Some(before),
        weekly_after: Some(after),
        fable_share: fable,
        fable_before: fable.map(|_| 19.0),
        fable_after: fable.map(|share| 19.0 + share.ceil()),
    };
    let mut quarry = row(5, "quarry", None, "manual", 95, 36);
    quarry.cost_usd = Some(0.87);
    quarry.context_percent = Some(41.0);
    quarry.usage = share(0.5, 27.0, 28.0, None);
    let mut lapsed = row(4, "lantern", Some("EP-6"), "visual", 200, 30);
    lapsed.session_id = None;
    lapsed.outcome = None;
    let mut lantern = row(3, "lantern", Some("EP-5"), "visual", 1500, 64);
    lantern.usage = share(1.5, 24.0, 26.0, Some(1.5));
    let mut stopped = row(2, "harbor", Some("EP-3"), "headless", 1700, 12);
    stopped.outcome = Some("stopped".to_owned());
    stopped.model = Some("claude-opus-5-5".to_owned());
    stopped.effort = Some("high".to_owned());
    stopped.cost_usd = Some(0.95);
    let mut done = row(1, "harbor", Some("EP-3"), "headless", 1650, 52);
    done.model = Some("claude-opus-5-5".to_owned());
    done.effort = Some("high".to_owned());
    done.cost_usd = Some(3.10);
    done.questions = 2;
    done.answer_latency_s = Some(95.0);
    done.result =
        Some("EP-3 is done: the tide tables are read and cached, and the tests pass.".to_owned());
    done.usage = share(2.0, 21.0, 23.0, Some(0.0));
    vec![quarry, lapsed, lantern, done, stopped]
}

/// The plan a picture's preview shows (EP-15c): `project`'s latest session among the canned
/// ledger `rows`, or `session`; its work two commits and six files, one of them the start's own
/// file put back, and a tick to redo. `refused` adds what the window and the engine would refuse
/// it for. Synthetic names and a neutral copy path; no git is asked.
pub fn canned_plan(
    project: &str,
    session: Option<&str>,
    rows: &[SessionRow],
    refused: bool,
) -> Plan {
    let now = Utc::now();
    let row = rows.iter().find(|row| {
        row.project.eq_ignore_ascii_case(project)
            && row.session_id.is_some()
            && session.is_none_or(|id| row.session_id.as_deref() == Some(id))
    });
    let id = row
        .and_then(|row| row.session_id.clone())
        .or_else(|| session.map(str::to_owned))
        .unwrap_or_else(|| "3b9d2f10-5e8c-4a71-9b06-7c4e1a2d8f53".to_owned());
    let ep = row
        .and_then(|row| row.ep.clone())
        .unwrap_or_else(|| "EP-3".to_owned());
    let started = row
        .and_then(SessionRow::started)
        .unwrap_or(now - TimeDelta::minutes(52));
    // Where the window's copies go, without a path of the machine (D-3).
    let backups = format!(r"%LOCALAPPDATA%\clauderoadmapwrangler\backups\{project}");
    let copy = PathBuf::from(format!(
        r"{backups}\reverts\{}",
        now.format("%Y%m%d-%H%M%S")
    ));
    let mut plan = Plan::new(project, &portfolio_root().join(project), copy);
    let head = "3c1e9b7a5d2f4e6081a9c3b5d7e9f1a2b4c6d8e0";
    plan.target = Some(Target {
        commit: head.to_owned(),
        recorded: true,
        subject: None,
    });
    plan.branch = Some("main".to_owned());
    plan.commits = vec![
        Commit {
            oid: "e41d7c2098b3a6f5c1d0e9f8a7b6c5d4e3f2a1b0".to_owned(),
            subject: format!(
                "Tide tables: read and cached, the berths filtered by length ({ep}, D-4)"
            ),
            pushed: false,
        },
        Commit {
            oid: "9b2f6a15c3d4e5f60718293a4b5c6d7e8f901a2b".to_owned(),
            subject: format!("docs(roadmap): record {ep} commit hash"),
            pushed: refused,
        },
    ];
    let fate = |path: &str, change, action| Fate {
        path: path.to_owned(),
        change,
        action,
    };
    plan.files = vec![
        fate("drafts/register.md", Change::Changed, FileAction::PutBack),
        fate("notes/berths.md", Change::Created, FileAction::Remove),
        fate("roadmap/README.md", Change::Changed, FileAction::Undo),
        fate(
            &format!("roadmap/{ep}-tide-tables.md"),
            Change::Created,
            FileAction::Undo,
        ),
        fate("src/tides.rs", Change::Created, FileAction::Undo),
        fate("src/view/list.rs", Change::Changed, FileAction::Undo),
    ];
    plan.start = Some(Start {
        at: started,
        head: Some(head.to_owned()),
        branch: Some("main".to_owned()),
        files: vec![
            StartFile {
                path: "drafts/register.md".to_owned(),
                state: FileState::Changed,
            },
            StartFile {
                path: "drafts/moorings.md".to_owned(),
                state: FileState::Untracked,
            },
        ],
        copy: Some(PathBuf::from(format!(r"{backups}\starts\{id}"))),
        copied: true,
    });
    plan.ticks = vec![Tick {
        at: started + TimeDelta::minutes(20),
        id: "OT-2".to_owned(),
        checked: true,
        disposition: None,
        delivery: false,
    }];
    plan.session_id = Some(id);
    plan.ep = Some(ep);
    plan.started = Some(started);
    if refused {
        plan.refusals = vec![
            "the window's headless EP-4 is running: revert once it has ended".to_owned(),
            "the remote has 1 of its commits (9b2f6a15): the tool never force-pushes; undo them \
             by hand with new commits (git revert)"
                .to_owned(),
        ];
    }
    plan
}

/// What the canned plan's revert did (EP-15c): every part of it, nothing left to do by hand.
fn canned_done(plan: &Plan) -> Done {
    let count = |action| {
        plan.files
            .iter()
            .filter(|fate| fate.action == action)
            .count()
    };
    Done {
        copy: plan.copy.clone(),
        target: plan
            .target
            .as_ref()
            .map(|target| target.commit.clone())
            .unwrap_or_default(),
        commits: plan.commits.len(),
        undone: count(FileAction::Undo),
        removed: count(FileAction::Remove),
        put_back: count(FileAction::PutBack),
        kept: count(FileAction::Keep),
        redone: plan.ticks.iter().map(Tick::detail).collect(),
        not_redone: Vec::new(),
        problems: Vec::new(),
    }
}

/// The comma-separated flags of a setup, trimmed, empty ones left out.
fn flags(setup: &str) -> Vec<String> {
    setup
        .split(',')
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The tracked fake projects.
pub fn portfolio_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("portfolio")
}

/// The root a run names (EP-36): the setup flag `root=`, else the variable; an empty one names
/// none.
fn root_of(flags: &[String], var: Option<OsString>) -> Option<PathBuf> {
    flags
        .iter()
        .find_map(|flag| flag.strip_prefix("root="))
        .map(OsString::from)
        .or(var)
        .filter(|root| !root.is_empty())
        .map(PathBuf::from)
}

/// The registry a run over `root` starts from (EP-36): the fake portfolio's, the root written in,
/// when the root holds every folder of the fake portfolio; else one project per folder of the
/// root, in the fake registry's shape. A folder is active with a house-style roadmap and
/// unformalised without, as Adopt would start it with the copy counted as a repository.
fn registry_for(root: &Path) -> io::Result<String> {
    if holds_the_fakes(root) {
        return Ok(registry_text(root));
    }
    let mut text = format!(
        "# The registry of a screenshot run over a copied portfolio (CLAUDE.md rule 5, EP-36): one\n\
         # project per folder of the data root, active with a roadmap and unformalised without.\n\
         format = 1\n\
         ignore = []\n\
         \n\
         [settings]\n\
         data_root = {}\n",
        quoted(&root.display().to_string())
    );
    for folder in discovery::folders(root)? {
        let state = if folder.roadmap {
            State::Active
        } else {
            State::Unformalised
        };
        let name = quoted(&folder.name);
        text.push_str(&format!(
            "\n[[project]]\nname = {name}\npath = {name}\nstate = \"{state}\"\n\
             priority = {DEFAULT_PRIORITY}\npinned = false\n"
        ));
    }
    Ok(text)
}

/// Whether `root` holds every folder of the fake portfolio: the tracked folder, a copy of it, or
/// another path to either.
fn holds_the_fakes(root: &Path) -> bool {
    let names = |dir: &Path| -> Option<BTreeSet<String>> {
        let folders = discovery::folders(dir).ok()?;
        Some(
            folders
                .into_iter()
                .map(|folder| folder.name.to_lowercase())
                .collect(),
        )
    };
    match (names(&portfolio_root()), names(root)) {
        (Some(fakes), Some(here)) => !fakes.is_empty() && fakes.is_subset(&here),
        _ => false,
    }
}

/// The template with the data root written in as a TOML string.
fn registry_text(root: &Path) -> String {
    include_str!("../fixtures/registry.toml")
        .replace("@DATA_ROOT@", &quoted(&root.display().to_string()))
}

/// A TOML basic string.
fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The fake account's owner.
const OWNER: &str = "example";

/// A fake project's readings: ages in hours before the run's clock.
struct Fake {
    name: &'static str,
    /// A repository: it has a last commit and a working tree.
    git: bool,
    commit_hours: i64,
    changed: usize,
    /// The origin on GitHub: `ahead`, `behind` as git counts them.
    origin: Option<(u32, u32)>,
    /// A repository of the project's name in the fake account.
    listed: Option<(Visibility, bool)>,
    head: Option<Verdict>,
    /// The latest run's conclusion, or `in_progress`.
    ci: Option<&'static str>,
    backup_hours: Option<i64>,
    backup_problem: Option<&'static str>,
}

static FAKES: [Fake; 8] = [
    Fake {
        name: "harbor",
        git: true,
        commit_hours: 3,
        changed: 2,
        origin: Some((1, 0)),
        listed: Some((Visibility::Public, false)),
        head: Some(Verdict::Ahead(1)),
        ci: Some("success"),
        backup_hours: Some(5),
        backup_problem: None,
    },
    Fake {
        name: "lantern",
        git: true,
        commit_hours: 26,
        changed: 0,
        origin: Some((0, 0)),
        listed: Some((Visibility::Private, false)),
        head: Some(Verdict::InSync),
        ci: Some("failure"),
        backup_hours: Some(5),
        backup_problem: None,
    },
    Fake {
        name: "meadow",
        git: true,
        commit_hours: 9 * 24,
        changed: 0,
        origin: Some((0, 0)),
        listed: Some((Visibility::Public, false)),
        head: Some(Verdict::NotFetched),
        ci: Some("in_progress"),
        backup_hours: Some(30),
        backup_problem: None,
    },
    Fake {
        name: "quarry",
        git: true,
        commit_hours: 50,
        changed: 5,
        origin: None,
        listed: None,
        head: None,
        ci: None,
        backup_hours: Some(70),
        backup_problem: None,
    },
    Fake {
        name: "compass",
        git: true,
        commit_hours: 75 * 24,
        changed: 0,
        origin: None,
        listed: Some((Visibility::Private, false)),
        head: None,
        ci: None,
        backup_hours: Some(5),
        backup_problem: Some("could not read notes.db: it is in use"),
    },
    Fake {
        name: "orchard",
        git: true,
        commit_hours: 40 * 24,
        changed: 1,
        origin: Some((0, 2)),
        listed: Some((Visibility::Private, false)),
        head: Some(Verdict::Behind(2)),
        ci: Some("success"),
        backup_hours: Some(5),
        backup_problem: None,
    },
    Fake {
        name: "beacon",
        git: true,
        commit_hours: 120 * 24,
        changed: 0,
        origin: Some((0, 0)),
        listed: Some((Visibility::Public, true)),
        head: Some(Verdict::InSync),
        ci: Some("success"),
        backup_hours: Some(5),
        backup_problem: None,
    },
    Fake {
        name: "atlas",
        git: false,
        commit_hours: 0,
        changed: 0,
        origin: None,
        listed: None,
        head: None,
        ci: None,
        backup_hours: None,
        backup_problem: None,
    },
];

/// Puts the fake readings into the portfolio, as if the git pass, the backup job and a GitHub
/// pass 40 minutes ago had read them. Each project takes the fake of its name. In a copied
/// portfolio (EP-36), where no project is named as a fake, each project with a folder takes the
/// fake at its place in the list, under its own name, so that the badge columns hold what a real
/// window's would.
pub fn readings(portfolio: &mut Portfolio, now: DateTime<Utc>) {
    let hours = TimeDelta::hours;
    let by_name = |name: &str| FAKES.iter().find(|fake| fake.name == name);
    let copied = !portfolio
        .projects
        .iter()
        .any(|row| by_name(row.name()).is_some());
    let mut place = 0;
    let fakes: Vec<Option<&Fake>> = portfolio
        .projects
        .iter()
        .map(|row| {
            if !copied {
                return by_name(row.name());
            }
            row.folder.as_ref()?;
            place += 1;
            Some(&FAKES[(place - 1) % FAKES.len()])
        })
        .collect();
    let account: Vec<Listed> = portfolio
        .projects
        .iter()
        .zip(&fakes)
        .filter_map(|(row, fake)| {
            let fake = (*fake)?;
            let (visibility, archived) = fake.listed?;
            Some(Listed {
                repo: Repo {
                    owner: OWNER.to_owned(),
                    name: row.name().to_owned(),
                },
                visibility,
                archived,
                pushed_at: Some(now - hours(fake.commit_hours)),
                default_branch: Some("main".to_owned()),
            })
        })
        .collect();
    let at = now - TimeDelta::minutes(40);
    for (row, fake) in portfolio.projects.iter_mut().zip(fakes) {
        let Some(fake) = fake else {
            continue;
        };
        let name = row.name().to_owned();
        let oid = format!("{:0<40}", format!("f00d{}", name.len()));
        let last = now - hours(fake.commit_hours);
        let badges = &mut row.badges;
        if fake.git {
            row.facts.last_commit = Some(last);
            badges.activity = Some(Activity {
                oid: Some(oid.clone()),
                last_commit: Some(last),
                changed: fake.changed,
                branch: Some("main".to_owned()),
                upstream: fake.origin.map(|_| "origin/main".to_owned()),
                ahead_behind: fake.origin,
            });
        }
        badges.remote = fake
            .origin
            .map(|_| format!("https://github.com/{OWNER}/{name}.git"));
        badges.hosting = hosting::classify(
            &[name.as_str()],
            badges.remote.as_deref(),
            Some(account.as_slice()),
        );
        badges.head = fake.head.clone().map(|verdict| Fetched {
            value: Check {
                remote_ref: "refs/heads/main".to_owned(),
                remote: Some("0".repeat(40)),
                local: Some(oid.clone()),
                verdict,
            },
            at,
        });
        badges.workflows = usize::from(fake.ci.is_some());
        badges.run = fake.ci.map(|conclusion| {
            let running = conclusion == "in_progress";
            Fetched {
                value: Ok(Some(health::Run {
                    workflow: "ci".to_owned(),
                    status: if running { "in_progress" } else { "completed" }.to_owned(),
                    conclusion: (!running).then(|| conclusion.to_owned()),
                    created_at: Some(last + TimeDelta::minutes(1)),
                    head_sha: Some(oid.clone()),
                })),
                at,
            }
        });
        badges.backup = Some(Backup {
            unit: Some(format!("WORK\\{name}")),
            archive: fake.backup_hours.map(|h| Archive {
                name: format!(
                    "{} {name}.zip",
                    (now - hours(h))
                        .with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H%M%S"),
                ),
                at: Some(now - hours(h)),
            }),
            checked: fake.backup_hours.map(|h| now - hours(h)),
            problem: fake.backup_problem.map(str::to_owned),
            excluded: Vec::new(),
        });
    }
    portfolio.account = Some(Fetched {
        value: Ok(account),
        at,
    });
    // The mailman's states, canned, since a screenshot run delivers nothing (EP-20): delivered,
    // waiting and counted in turn over each project's blocks to a registered project.
    for row in &mut portfolio.projects {
        let sender = row.name().to_owned();
        row.mail = row
            .outgoing
            .iter()
            .filter_map(|block| match &block.recipient {
                Recipient::Project(receiver) => Some((block, receiver)),
                Recipient::Unknown => None,
            })
            .enumerate()
            .map(|(n, (block, receiver))| Mark {
                sender: sender.clone(),
                path: block.path.clone(),
                line: block.line,
                state: match n % 3 {
                    0 => MailState::Delivered {
                        receiver: receiver.clone(),
                        id: "IN-3".to_owned(),
                    },
                    1 => MailState::Waits("a session works there".to_owned()),
                    _ => MailState::Counted,
                },
            })
            .collect();
    }
}

/// Called once per frame: whether the projects are still being read.
pub fn set_busy(busy: bool) {
    BUSY.store(busy, Ordering::Relaxed);
}

fn keys_flag() -> Vec<String> {
    let setup = std::env::var(SETUP_VAR).unwrap_or_default();
    flags(&setup)
        .iter()
        .find_map(|flag| flag.strip_prefix("keys="))
        .map(|keys| keys.split(';').map(str::to_owned).collect())
        .unwrap_or_default()
}

/// Called once per frame: takes the picture once the frame has settled, the keys are typed and
/// the projects are read, then closes the window.
pub fn tick(ctx: &egui::Context) {
    let Some(path) = std::env::var_os(PICTURE_VAR) else {
        return;
    };
    let frame = FRAMES.fetch_add(1, Ordering::Relaxed);
    let time = ctx.input(|i| i.time);
    let keys = keys_flag().len() as u32;
    let busy = BUSY.load(Ordering::Relaxed) && time < WAIT_AT_MOST_SECONDS;
    if frame < CAPTURE_AT_FRAME.max(KEYS_FROM_FRAME + keys + 3)
        || time < CAPTURE_AFTER_SECONDS
        || busy
    {
        ctx.request_repaint();
    } else if !CAPTURING.swap(true, Ordering::Relaxed) {
        let ctx = ctx.clone();
        ctx.clone().request_screenshot(move |image| {
            match write_png(Path::new(&path), &image) {
                Ok(()) => eprintln!("screenshot: saved {}", Path::new(&path).display()),
                Err(e) => eprintln!("screenshot: {e}"),
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            ctx.request_repaint();
        });
    }
}

/// Feeds the `keys=` chords into the frame's input, one per frame.
pub fn inject_keys(raw: &mut egui::RawInput) {
    let Some(n) = FRAMES.load(Ordering::Relaxed).checked_sub(KEYS_FROM_FRAME) else {
        return;
    };
    let keys = keys_flag();
    let Some(text) = keys.get(n as usize) else {
        return;
    };
    match chord(text) {
        Some((modifiers, key)) => {
            for pressed in [true, false] {
                raw.events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers,
                });
            }
        }
        None => eprintln!("screenshot: cannot read the key chord {text}"),
    }
}

/// `ctrl+Down` → Ctrl (which egui also reports as the command key on Windows) and ArrowDown;
/// `Down`, `Up`, `Left` and `Right` stand for the arrows.
fn chord(text: &str) -> Option<(egui::Modifiers, egui::Key)> {
    let mut modifiers = egui::Modifiers::NONE;
    let mut key = None;
    for part in text.split('+').map(str::trim) {
        match part {
            "ctrl" => modifiers |= egui::Modifiers::CTRL | egui::Modifiers::COMMAND,
            "shift" => modifiers |= egui::Modifiers::SHIFT,
            "alt" => modifiers |= egui::Modifiers::ALT,
            "Down" => key = Some(egui::Key::ArrowDown),
            "Up" => key = Some(egui::Key::ArrowUp),
            "Left" => key = Some(egui::Key::ArrowLeft),
            "Right" => key = Some(egui::Key::ArrowRight),
            name => key = Some(egui::Key::from_name(name)?),
        }
    }
    Some((modifiers, key?))
}

/// The frame as an RGB PNG.
fn write_png(path: &Path, image: &egui::ColorImage) -> image::ImageResult<()> {
    let [width, height] = image.size;
    let rgb: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|pixel| {
            let [r, g, b, _] = pixel.to_srgba_unmultiplied();
            [r, g, b]
        })
        .collect();
    image::save_buffer_with_format(
        path,
        &rgb,
        width as u32,
        height as u32,
        image::ExtendedColorType::Rgb8,
        image::ImageFormat::Png,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_setup_flags_and_the_key_chords() {
        assert_eq!(
            flags(" dark, select=harbor ,,keys=Down;Enter "),
            ["dark", "select=harbor", "keys=Down;Enter"]
        );
        let run = Run {
            home: PathBuf::from("home"),
            flags: flags("size=1200x800,expand=harbor"),
            root: None,
        };
        assert_eq!(run.size(), [1200.0, 800.0]);
        assert_eq!(run.value("expand"), Some("harbor"));
        let default = Run {
            flags: flags("size=wide"),
            ..run
        };
        assert_eq!(default.size(), SIZE);
        assert_eq!(
            chord("Down"),
            Some((egui::Modifiers::NONE, egui::Key::ArrowDown))
        );
        assert_eq!(chord("ctrl+F").map(|(_, key)| key), Some(egui::Key::F));
        assert_eq!(chord("Enter").map(|(_, key)| key), Some(egui::Key::Enter));
        assert_eq!(chord("nonsense"), None);
    }

    #[test]
    fn keeprule_reads_the_project_the_key_the_phase_and_the_value() {
        use crw_core::rules::{Key, Scope};

        assert_eq!(
            keeprule_action("harbor:model=claude-opus-5-5"),
            Some(Action::KeepRule {
                project: "harbor".to_owned(),
                scope: Scope::Project,
                key: Key::Model,
                value: Some("claude-opus-5-5".to_owned()),
            })
        );
        assert_eq!(
            keeprule_action("harbor:effort@P1=high"),
            Some(Action::KeepRule {
                project: "harbor".to_owned(),
                scope: Scope::Phase("P1".to_owned()),
                key: Key::Effort,
                value: Some("high".to_owned()),
            })
        );
        for wrong in ["harbor", "harbor:model", "harbor:ultracode=yes"] {
            assert_eq!(keeprule_action(wrong), None, "{wrong}");
        }
    }

    #[test]
    fn the_registry_template_reads_with_the_data_root_written_in() {
        let text = registry_text(Path::new(r"C:\Some folder\fixtures\portfolio"));
        assert!(text.contains(r#"data_root = "C:\\Some folder\\fixtures\\portfolio""#));
        assert!(!text.contains("@DATA_ROOT@"));
    }

    use crate::portfolio::read_files;
    use crate::portfolio::tests::Scratch;

    /// A copy of `from` at `to`, folder by folder.
    fn copy_tree(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap().flatten() {
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    /// A run over `root` with its home in `home`, the root named through the setup flag.
    fn run_over(home: &Path, root: &Path) -> Run {
        let flags = flags(&format!("light,root={}", root.display()));
        let root = root_of(&flags, None);
        Run {
            home: home.to_owned(),
            flags,
            root,
        }
    }

    /// What a run shows of each project, the data root's place aside (a brief's path names
    /// it): the registry's entry, the files' readings and the canned ones; then the folders left
    /// out.
    fn shown(portfolio: &Portfolio) -> Vec<String> {
        let root = portfolio.data_root.display().to_string();
        let escaped = root.replace('\\', "\\\\");
        let mut lines: Vec<String> = portfolio
            .projects
            .iter()
            .map(|row| {
                let (project, badges) = (&row.project, &row.badges);
                let line = format!(
                    "{} {} {} {} {:?} | {:?} {:?} {} {:?} {:?} {:?} {:?} {:?} | {:?} {:?} {:?} {:?} \
                     {:?} {:?} {:?} {:?} {:?}",
                    project.name,
                    project.state,
                    project.priority,
                    project.pinned,
                    project.launch,
                    row.folder.as_ref().map(|folder| (folder.git, folder.roadmap)),
                    row.next(),
                    row.open_items(),
                    row.open_messages(),
                    row.in_progress(),
                    row.snapshot.decisions.log,
                    row.facts,
                    row.outgoing,
                    badges.stack,
                    badges.license,
                    badges.workflows,
                    badges.remote,
                    badges.hosting,
                    badges.activity,
                    badges.head,
                    badges.run,
                    badges.backup,
                );
                line.replace(&escaped, "<root>").replace(&root, "<root>")
            })
            .collect();
        lines.extend(
            portfolio
                .unadopted
                .iter()
                .map(|folder| format!("unadopted {}", folder.name)),
        );
        lines
    }

    #[test]
    fn names_the_root_by_the_flag_before_the_variable() {
        let var = Some(OsString::from(r"C:\From the variable"));
        assert_eq!(
            root_of(&flags(r"dark,root=C:\From the flag"), var.clone()),
            Some(PathBuf::from(r"C:\From the flag"))
        );
        assert_eq!(
            root_of(&flags("dark"), var),
            Some(PathBuf::from(r"C:\From the variable"))
        );
        assert_eq!(root_of(&flags("dark"), Some(OsString::new())), None);
        assert_eq!(root_of(&flags("root="), None), None);
        assert_eq!(root_of(&[], None), None);
    }

    #[test]
    fn the_fixtures_copied_elsewhere_read_as_the_tracked_ones() {
        let scratch = Scratch::new("screenshot-copied-fixtures");
        let copy = scratch.0.join("Projects");
        copy_tree(&portfolio_root(), &copy);
        let tracked = Run {
            home: scratch.0.join("tracked-home"),
            flags: Vec::new(),
            root: None,
        };
        let copied = run_over(&scratch.0.join("copy-home"), &copy);
        assert_eq!(copied.root(), copy);

        let now = Utc::now();
        let read = |run: &Run| {
            let store = run.store().unwrap();
            let mut portfolio = read_files(&store);
            assert_eq!(portfolio.problem, None);
            readings(&mut portfolio, now);
            portfolio
        };
        let (tracked, copied) = (read(&tracked), read(&copied));
        assert_eq!(copied.data_root, copy);
        assert_eq!(shown(&copied), shown(&tracked));
        assert!(
            shown(&tracked)
                .iter()
                .any(|line| line.starts_with("harbor active 1 true"))
        );
        assert!(shown(&tracked).contains(&"unadopted sketchbook".to_owned()));
        let harbor = copied.project("harbor").unwrap();
        assert_eq!(harbor.path, copy.join("harbor"));
        assert!(harbor.next().is_some());
        let names = |portfolio: &Portfolio| -> Vec<String> {
            let account = portfolio.account.as_ref().unwrap().value.as_ref().unwrap();
            account
                .iter()
                .map(|listed| listed.repo.name.clone())
                .collect()
        };
        assert_eq!(names(&copied), names(&tracked));
        assert_eq!(
            names(&tracked),
            [
                "harbor", "lantern", "meadow", "compass", "orchard", "beacon"
            ]
        );
    }

    #[test]
    fn another_root_reads_its_own_folders_with_the_fakes_by_place() {
        let scratch = Scratch::new("screenshot-other-root");
        let root = scratch.0.join("copy");
        let alpha = root.join("alpha").join("roadmap");
        fs::create_dir_all(&alpha).unwrap();
        fs::write(
            alpha.join("README.md"),
            "# alpha — Roadmap\n\n## Phase P1 — Start\n\n\
             | # | Brief | Size | Depends on | Core | Done |\n|---|---|---|---|---|---|\n\
             | EP-1 | First brief | S | — | core | ☐ |\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("beta")).unwrap();
        fs::create_dir_all(root.join("gamma")).unwrap();
        fs::create_dir_all(root.join(".hidden")).unwrap();
        let run = run_over(&scratch.0.join("home"), &root);
        let store = run.store().unwrap();
        let registry = store.load().unwrap().registry;
        let entries: Vec<(&str, State, u8, bool)> = registry
            .projects
            .iter()
            .map(|p| (p.name.as_str(), p.state, p.priority, p.pinned))
            .collect();
        assert_eq!(
            entries,
            [
                ("alpha", State::Active, DEFAULT_PRIORITY, false),
                ("beta", State::Unformalised, DEFAULT_PRIORITY, false),
                ("gamma", State::Unformalised, DEFAULT_PRIORITY, false),
            ]
        );
        assert!(registry.ignore.is_empty());
        assert!(paths::same(&registry.data_root(), &root));

        // A project whose folder has gone takes no fake, and the next one takes its place.
        fs::remove_dir_all(root.join("beta")).unwrap();
        let mut portfolio = read_files(&store);
        assert_eq!(portfolio.problem, None);
        readings(&mut portfolio, Utc::now());
        let alpha = portfolio.project("alpha").unwrap();
        assert_eq!(
            alpha.next().map(|row| row.id.to_string()),
            Some("EP-1".to_owned())
        );
        assert_eq!(
            alpha.badges.remote.as_deref(),
            Some("https://github.com/example/alpha.git")
        );
        assert_eq!(alpha.badges.hosting.class(), Class::Public);
        assert!(alpha.badges.activity.is_some());
        let beta = portfolio.project("beta").unwrap();
        assert!(beta.folder.is_none());
        assert_eq!(beta.badges.remote, None);
        assert!(beta.badges.backup.is_none());
        let gamma = portfolio.project("gamma").unwrap();
        assert_eq!(gamma.badges.hosting.class(), Class::Private);
        assert_eq!(
            gamma.badges.backup.as_ref().and_then(|b| b.unit.as_deref()),
            Some(r"WORK\gamma")
        );
        let account = portfolio.account.as_ref().unwrap().value.as_ref().unwrap();
        let names: Vec<&str> = account.iter().map(|l| l.repo.name.as_str()).collect();
        assert_eq!(names, ["alpha", "gamma"]);
    }

    #[test]
    fn a_relative_root_is_resolved_against_the_working_folder() {
        // Tests run in the crate's folder, which holds `fixtures`.
        let scratch = Scratch::new("screenshot-relative-root");
        let run = run_over(
            &scratch.0.join("home"),
            &Path::new("fixtures").join("portfolio"),
        );
        let root = run.root();
        assert!(root.is_absolute());
        assert!(paths::same(&root, &portfolio_root()));
        let store = run.store().unwrap();
        let portfolio = read_files(&store);
        assert_eq!(portfolio.problem, None);
        let harbor = portfolio.project("harbor").unwrap();
        assert_eq!(harbor.path, root.join("harbor"));
        assert!(
            harbor.project.pinned,
            "the fake registry, as the root holds the fakes"
        );
    }

    #[test]
    fn refuses_a_root_that_is_no_folder_and_a_home_written_for_another() {
        let scratch = Scratch::new("screenshot-refusals");
        let missing = run_over(&scratch.0.join("home-a"), &scratch.0.join("missing"));
        let refused = missing.store().err().unwrap();
        assert!(refused.to_string().contains("is not a folder"), "{refused}");
        assert!(!scratch.0.join("home-a").exists());

        let copy = scratch.0.join("copy");
        fs::create_dir_all(copy.join("alpha")).unwrap();
        let home = scratch.0.join("home-b");
        run_over(&home, &copy).store().unwrap();
        run_over(&home, &copy).store().unwrap();
        let fakes = Run {
            home: home.clone(),
            flags: Vec::new(),
            root: None,
        };
        let refused = fakes.store().err().unwrap();
        assert!(
            refused.to_string().contains("start from a fresh home"),
            "{refused}"
        );
    }
}
