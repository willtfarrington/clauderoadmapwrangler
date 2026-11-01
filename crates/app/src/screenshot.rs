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
//! - `light` / `dark` (the theme); `compact` / `comfortable` (the density);
//! - `select=<project or folder>` (the selection); `expand=<project>` (its row expanded inline);
//! - `tile=<pane>` (bring `projects`, `detail`, `decisions`, `sessions`, `chat` or `usage` to
//!   the front of its group);
//! - `keys=<chord>;<chord>…` (key presses such as `Down`, `Enter` or `ctrl+F`, key names as
//!   egui spells them, fed one per frame through eframe's input hook as if typed);
//! - `adopt=<folder>`, `ignore=<folder>`, `pin=<project>`, `tick=<project>:<OT-n>` (the action a
//!   click queues, through the same path: the first three write the run's registry, the tick is
//!   refused);
//! - `size=<width>x<height>` (the window's inner size in points; 1600x1000 by default).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crw_core::paths;
use crw_core::registry::{Defaults, Store};
use eframe::egui;

use crate::board::{Action, Dashboard};
use crate::layout::{self, Pane};
use crate::portfolio::Selection;
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
