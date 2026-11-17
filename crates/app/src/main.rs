//! clauderoadmapwrangler's desktop window (D-14, D-25): the docking layout over the portfolio,
//! read-only but for the writes the tool owns (D-11). EP-6 brought the shell, the project list,
//! the detail pane and the `screenshot` feature's capture, EP-7 the badges, EP-10 the launch
//! dialog and the headless sessions it starts, EP-11 its visual launch, EP-12 the loopback server
//! for the hooks and the status line and the sessions they report, EP-15 the notifications'
//! popup and the tray icon, which the window closes to, EP-15c a session's revert behind its
//! preview, EP-17 the usage meter's strip and tile, EP-37 the restart into the installed build
//! (a window started with `--restarted <pid>` waits for the old one to end before it opens),
//! EP-30 the phone's pages, which the loopback server renders from a snapshot the window
//! publishes; later briefs fill the other tiles.

// Release builds open without a console window; debug builds keep one for diagnostics.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod act;
mod analytics;
mod autopilot;
mod badges;
mod board;
mod chat;
mod compose;
mod crash;
mod decisions;
mod detail;
mod diagnostics;
mod form;
mod formalise;
mod launch;
mod layout;
mod list;
mod mail;
mod markup;
mod meter;
mod pages;
mod phone;
mod popup;
mod portfolio;
mod published;
mod remote_view;
mod restart;
mod revert;
#[cfg(feature = "screenshot")]
mod screenshot;
mod server;
mod sessions;
mod style;
#[cfg(test)]
mod testing;
mod toasts;
mod transcript;
mod tray;
mod visual;
mod watch;

use std::path::PathBuf;

use crw_core::install::{self, Running, Stamp};
use crw_core::registry::Store;
use crw_core::session::cli;
use crw_core::{autostart, icon, paths, signals, usage};
use eframe::egui;

use board::{Dashboard, Places};

/// Names the window and eframe's folder in `%APPDATA%`, beside the registry (D-2, D-8).
const APP_ID: &str = "clauderoadmapwrangler";

/// The window's size on a first launch.
const DEFAULT_SIZE: [f32; 2] = [1440.0, 900.0];

fn main() -> eframe::Result {
    #[cfg(feature = "screenshot")]
    let run = screenshot::Run::from_env();
    #[cfg(feature = "screenshot")]
    if let Some(run) = &run {
        return screenshot_run(run);
    }

    // A restart (EP-37): the old window saves its layout and frees the port as it ends, which
    // this one waits for before it opens.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let restarted = install::restarted_from(&args);
    if let Some(pid) = restarted {
        install::wait_for_end(pid, install::WAIT_FOR_OLD);
    }
    // A start at login (D-29; EP-28a): hidden in the tray, and quiet when a window runs already.
    let at_login = restarted.is_none() && autostart::started_at_login(&args);
    // With CRW_HOME set, eframe's state lives there too, so a trial leaves the owner's layout
    // alone (rule 13).
    let persistence_path = paths::home().map(|home| home.join("app.ron"));
    let options = options(DEFAULT_SIZE, persistence_path, true);
    // Without a console a panic ends the window without a word: it leaves its report in the local
    // folder instead, and the next start names it (EP-14a).
    let crash_log = paths::local_dir().map(|dir| dir.join(crash::FILE));
    if let Some(file) = &crash_log {
        crash::install(file.clone());
    }
    // The rolling log (D-29; EP-28): a file a day in the local folder's `log`, fourteen kept.
    if let Some(dir) = paths::local_dir() {
        crw_core::log::start(dir.join(crw_core::log::FOLDER));
        let build = Running::now()
            .and_then(|running| running.stamp)
            .map_or_else(String::new, |stamp| format!(", built {}", stamp.when()));
        let restart = match restarted {
            Some(pid) => format!(", restarted from the window of process {pid}"),
            None if at_login => ", at login".to_owned(),
            None => String::new(),
        };
        crw_core::log::line(
            "start",
            &format!(
                "the window starts: crw {}{build}, process {}{restart}",
                env!("CARGO_PKG_VERSION"),
                std::process::id()
            ),
        );
    }
    let ended = eframe::run_native(
        APP_ID,
        options,
        Box::new(move |cc| {
            let store = Store::locate().ok_or("no profile folder: APPDATA is not set")?;
            // The signals' port and the CLI the poll of live sessions asks (EP-12).
            let registry = store.load().ok().map(|loaded| loaded.registry);
            let port = registry
                .as_ref()
                .map_or(signals::DEFAULT_PORT, |registry| registry.port());
            let program = cli::locate(
                registry
                    .as_ref()
                    .and_then(|registry| registry.settings.claude_cli.as_deref()),
            )
            .unwrap_or_else(|| PathBuf::from("claude.exe"));
            let places = Places {
                store,
                ledger: paths::ledger_file(),
                online: true,
                crash_log,
            };
            let mut dashboard = Dashboard::new(cc, places);
            let board = dashboard.board_mut();
            if let Some(home) = paths::home() {
                board.home_note = Some(format!("{}={}", paths::HOME_VAR, home.display()));
            }
            // The build this window runs, the installed copy and the repository's build (EP-37),
            // and the local folder that the handover goes to (EP-37a).
            let running = Running::now();
            let when = running
                .as_ref()
                .and_then(|running| running.stamp)
                .map_or_else(String::new, |stamp: Stamp| format!(" of {}", stamp.when()));
            board.restart =
                restart::Restart::new(running, install::bin_dir(), install::build_dir());
            board.restart.local = paths::local_dir();
            board.restart.path_dir = install::path_dir();
            let backups = board.store().backups.clone();
            // The phone's pages read what the window publishes, the owner's login from the
            // registry and write their log to the ledger (EP-30).
            board.watch.pages = Some(server::Pages {
                published: board.pages.reader(),
                store: board.loader.shared_store(),
                ledger: board.ledger().map(std::path::Path::to_path_buf),
            });
            // A window started by a restart never yields to another (EP-40): it binds once the
            // old one ended, and it holds the handover. A start at login yields without asking
            // the running window to show itself (EP-28a).
            board.watch.quiet = at_login;
            board
                .watch
                .start(port, program, backups, &cc.egui_ctx, restarted.is_none());
            // The usage meter (D-23; EP-17): the cached reading first, then the endpoint's polls
            // on a thread of their own, the token read from Claude Code's file at each one.
            board.meter.start(
                usage::credentials_file(),
                paths::ledger_file(),
                &cc.egui_ctx,
            );
            // The notifications and the tray icon (D-21, D-29; EP-15), on this thread, whose
            // message loop serves the icon. Without the icon, closing the window quits as before.
            board.toasts.start(paths::ledger_file());
            // The Diagnostics tile's checks may start children in this window (EP-28).
            board.diagnostics.start();
            match tray::Tray::start(&cc.egui_ctx) {
                Ok(tray) => board.tray = Some(tray),
                Err(why) => board.warn(format!(
                    "The tray icon did not start ({why}): closing the window quits it."
                )),
            }
            // The handover (EP-37a), once the tray is there to hide in: a restarted window's held
            // chains and plans, said in the start's notice with the build; a file no restart took
            // is dropped, and what it held said.
            board.take_handover(paths::local_dir().as_deref(), restarted, &when);
            // A start at login opens hidden in the tray (EP-28a); without the tray it shows. The
            // first close's notice stays for the owner's own first close.
            if at_login && board.tray.is_some() {
                let told = board.told_tray;
                board.hide_at_start();
                board.told_tray = told;
            }
            Ok(Box::new(dashboard))
        }),
    );
    match &ended {
        Ok(()) => crw_core::log::line("quit", "the window ends"),
        Err(e) => crw_core::log::line("error", &format!("the window ends on an error: {e}")),
    }
    ended
}

fn options(
    size: [f32; 2],
    persistence_path: Option<std::path::PathBuf>,
    persist_window: bool,
) -> eframe::NativeOptions {
    // The app's own icon (EP-28a): the tray's tile without its badges, which eframe scales for
    // the title bar, Alt+Tab and the taskbar in place of egui's logo.
    let side = 256;
    let icon = egui::IconData {
        rgba: icon::plain(side),
        width: side,
        height: side,
    };
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_ID)
            .with_app_id(APP_ID)
            .with_icon(std::sync::Arc::new(icon))
            .with_inner_size(size)
            .with_min_inner_size([900.0, 560.0]),
        persistence_path,
        persist_window,
        ..Default::default()
    }
}

/// The screenshot feature's run (rule 5): the fake portfolio or a copy (EP-36), the window at a
/// fixed size.
#[cfg(feature = "screenshot")]
fn screenshot_run(run: &screenshot::Run) -> eframe::Result {
    let store = match run.store() {
        Ok(store) => store,
        Err(e) => {
            eprintln!("screenshot: cannot start in {}: {e}", run.home.display());
            std::process::exit(1);
        }
    };
    let options = options(run.size(), Some(run.state_file()), false);
    eframe::run_native(
        APP_ID,
        options,
        Box::new(move |cc| {
            // The run's own ledger, and no network: the projects' git, GitHub and backup
            // readings come from `screenshot::readings`.
            let places = Places {
                store,
                ledger: Some(run.home.join("ledger.sqlite")),
                online: false,
                crash_log: Some(run.home.join(crate::crash::FILE)),
            };
            let mut dashboard = Dashboard::new(cc, places);
            run.arrange(&mut dashboard, &cc.egui_ctx);
            Ok(Box::new(dashboard))
        }),
    )
}
