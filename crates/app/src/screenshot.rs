//! Developer aid, compiled only with `--features screenshot` (CLAUDE.md rule 5).
//!
//! With `CRW_SCREENSHOT=<file.png>` set, the app runs on a fake portfolio, saves its own window
//! contents as a PNG once the projects are read, and closes. Nothing outside the app's window is
//! captured. The run's home is `CRW_HOME` when set, else a `crw-screenshot` folder beside the
//! picture: the registry (written there from `fixtures/registry.toml` on the first run, so a
//! second run starts from what the first one changed), eframe's state and the backups. The fake
//! projects are the tracked folders of `fixtures/portfolio`, which a run never writes into: ticks
//! are refused with a notice. `CRW_SCREENSHOT_SETUP` (comma-separated) arranges the view:
//!
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
//! - `sort=<key>` (`priority`, `completion`, `remaining`, `last-activity`, `hosting`, `name`);
//!   `stack=<stack>` (`rust`, `python`, … or the chip's letters) and `hosting=<class>` (`public`,
//!   `private`, `remote`, `local`), each adding to its filter;
//! - `launch=<project>` (the launch dialog open on the project's next brief, once it is read);
//! - `sessions` (canned sessions that Claude Code reports: lantern's visual one waiting on a
//!   permission prompt, orchard's manual one busy with its status line's figures though every
//!   row of orchard's is checked, beacon's tab open with no prompt yet, one outside the registry,
//!   and the signals listening with their entries installed; from EP-13 also a headless harbor
//!   EP-4 of the window's with a transcript, and ledger rows of every mode and outcome);
//! - `transcript` (the canned sessions, the Sessions pane in front with harbor's tab open);
//! - `decisions` (the canned sessions, the Decisions pane in front: from EP-14 harbor's session
//!   waits on a pair of questions and a `git push`, under a 30-minute limit; a quarry EP-2 run
//!   was deferred at its limit with a question open; lantern's visual session waits in VS Code;
//!   harbor's refused edit of a neighbour);
//! - `quit` (the canned sessions, and the question Quit asks before it stops harbor's run, from
//!   EP-15); `popup` (the notifications' popup with canned toasts, drawn inside the window: a
//!   real one is a window of its own at the screen's lower right);
//! - `size=<width>x<height>` (the window's inner size in points; 1600x1000 by default).
//!
//! The fake projects are not repositories and have no remote, so their git, GitHub and backup
//! readings come from [`FAKES`], set against the run's own clock ([`readings`]); their stack chips
//! and licenses are read from the fixtures' own files. A run never starts the GitHub pass. The
//! launch dialog reads the synthetic model catalog in `fixtures/model-catalog`, a canned
//! preflight and a canned VS Code with no window open, never the real CLI, VS Code or its lock
//! files, and a run starts no session and opens nothing. In the fixture registry harbor launches
//! headless and lantern visually. A run binds no port, polls no live session and reads no Claude
//! Code settings (EP-12).

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
use crw_core::ledger::SessionRow;
use crw_core::paths;
use crw_core::registry::{Defaults, Store};
use crw_core::session::{
    Choice, Denied, Ended, Event, Init, Launch, Outcome, Question, Request, ToolResult, ToolUse,
};
use crw_core::sessions::{Mode, Seen, Status};
use crw_core::signals::{self, StatusLine};
use eframe::egui;

use crate::board::{Action, Board, Dashboard};
use crate::layout::{self, Pane};
use crate::portfolio::{Fetched, Portfolio, Selection, SortKey};
use crate::server::Listening;
use crate::sessions::Live;
use crate::style::Density;

pub const PICTURE_VAR: &str = "CRW_SCREENSHOT";
pub const SETUP_VAR: &str = "CRW_SCREENSHOT_SETUP";

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

/// A screenshot run: its home folder and its setup; [`tick`] reads the picture's path.
pub struct Run {
    pub home: PathBuf,
    pub flags: Vec<String>,
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
        Some(Run { home, flags })
    }

    /// The fake portfolio's registry in the home folder, written from the template when the
    /// home has none yet.
    pub fn store(&self) -> io::Result<Store> {
        let root = portfolio_root();
        let path = self.home.join("registry.toml");
        if !path.exists() {
            fs::create_dir_all(&self.home)?;
            fs::write(&path, registry_text(&root))?;
        }
        Ok(Store::new(
            path,
            self.home.join("backups"),
            Defaults {
                repository: root.join("harbor"),
                data_root: root,
            },
        ))
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
                "launch" => board.open_when_read = Some(value.to_owned()),
                "sessions" => canned_sessions(board),
                "transcript" => {
                    if board.sessions.live.is_empty() {
                        canned_sessions(board);
                    }
                    board.sessions.tab = board
                        .sessions
                        .live
                        .iter()
                        .find(|live| live.project == "harbor")
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
                "popup" => {
                    ctx.set_embed_viewports(true);
                    for toast in crate::toasts::canned() {
                        board.toasts.popup.push(toast);
                    }
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
                "keys" | "size" => {} // read by `inject_keys` and `main`
                _ => eprintln!("screenshot: unknown setup flag {flag}"),
            }
        }
    }
}

/// The sessions a picture shows Claude Code reporting (EP-12), set against the run's own clock.
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
        }
    };
    let mut quarry = row(5, "quarry", None, "manual", 95, 36);
    quarry.cost_usd = Some(0.87);
    quarry.context_percent = Some(41.0);
    let mut lapsed = row(4, "lantern", Some("EP-6"), "visual", 200, 30);
    lapsed.session_id = None;
    lapsed.outcome = None;
    let lantern = row(3, "lantern", Some("EP-5"), "visual", 1500, 64);
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
    vec![quarry, lapsed, lantern, done, stopped]
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

/// The template with the data root written in as a TOML string.
fn registry_text(root: &Path) -> String {
    let quoted = format!(
        "\"{}\"",
        root.display()
            .to_string()
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
    );
    include_str!("../fixtures/registry.toml").replace("@DATA_ROOT@", &quoted)
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
/// pass 40 minutes ago had read them.
pub fn readings(portfolio: &mut Portfolio, now: DateTime<Utc>) {
    let hours = TimeDelta::hours;
    let account: Vec<Listed> = FAKES
        .iter()
        .filter_map(|fake| {
            let (visibility, archived) = fake.listed?;
            Some(Listed {
                repo: Repo {
                    owner: OWNER.to_owned(),
                    name: fake.name.to_owned(),
                },
                visibility,
                archived,
                pushed_at: Some(now - hours(fake.commit_hours)),
                default_branch: Some("main".to_owned()),
            })
        })
        .collect();
    let at = now - TimeDelta::minutes(40);
    for row in &mut portfolio.projects {
        let Some(fake) = FAKES.iter().find(|fake| fake.name == row.name()) else {
            continue;
        };
        let oid = format!("{:0<40}", format!("f00d{}", fake.name.len()));
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
            .map(|_| format!("https://github.com/{OWNER}/{}.git", fake.name));
        badges.hosting = hosting::classify(
            &[fake.name],
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
            unit: Some(format!("WORK\\{}", fake.name)),
            archive: fake.backup_hours.map(|h| Archive {
                name: format!(
                    "{} {}.zip",
                    (now - hours(h))
                        .with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H%M%S"),
                    fake.name
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
    fn the_registry_template_reads_with_the_data_root_written_in() {
        let text = registry_text(Path::new(r"C:\Some folder\fixtures\portfolio"));
        assert!(text.contains(r#"data_root = "C:\\Some folder\\fixtures\\portfolio""#));
        assert!(!text.contains("@DATA_ROOT@"));
    }
}
