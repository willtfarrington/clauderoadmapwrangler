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
//! - `size=<width>x<height>` (the window's inner size in points; 1600x1000 by default).
//!
//! The fake projects are not repositories and have no remote, so their git, GitHub and backup
//! readings come from [`FAKES`], set against the run's own clock ([`readings`]); their stack chips
//! and licenses are read from the fixtures' own files. A run never starts the GitHub pass. The
//! launch dialog reads the synthetic model catalog in `fixtures/model-catalog` and a canned
//! preflight, never the real CLI, and a run starts no session.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use chrono::{DateTime, TimeDelta, Utc};
use crw_core::badges::activity::Activity;
use crw_core::badges::backup::{Archive, Backup};
use crw_core::badges::health;
use crw_core::badges::hosting::{self, Check, Class, Listed, Repo, Verdict, Visibility};
use crw_core::badges::stack::Kind;
use crw_core::paths;
use crw_core::registry::{Defaults, Store};
use eframe::egui;

use crate::board::{Action, Dashboard};
use crate::layout::{self, Pane};
use crate::portfolio::{Fetched, Portfolio, Selection, SortKey};
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
