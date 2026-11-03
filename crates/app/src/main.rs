//! clauderoadmapwrangler's desktop window (D-14, D-25): the docking layout over the portfolio,
//! read-only but for the writes the tool owns (D-11). EP-6 brought the shell, the project list,
//! the detail pane and the `screenshot` feature's capture, EP-7 the badges, EP-10 the launch
//! dialog and the headless sessions it starts, EP-11 its visual launch, EP-12 the loopback server
//! for the hooks and the status line and the sessions they report; later briefs fill the other
//! tiles.

// Release builds open without a console window; debug builds keep one for diagnostics.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod badges;
mod board;
mod detail;
mod launch;
mod layout;
mod list;
mod markup;
mod portfolio;
#[cfg(feature = "screenshot")]
mod screenshot;
mod server;
mod sessions;
mod style;
mod transcript;
mod visual;
mod watch;

use std::path::PathBuf;

use crw_core::registry::Store;
use crw_core::session::cli;
use crw_core::{paths, signals};
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

    // With CRW_HOME set, eframe's state lives there too, so a trial leaves the owner's layout
    // alone (rule 13).
    let persistence_path = paths::home().map(|home| home.join("app.ron"));
    let options = options(DEFAULT_SIZE, persistence_path, true);
    eframe::run_native(
        APP_ID,
        options,
        Box::new(|cc| {
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
            };
            let mut dashboard = Dashboard::new(cc, places);
            let board = dashboard.board_mut();
            if let Some(home) = paths::home() {
                board.home_note = Some(format!("{}={}", paths::HOME_VAR, home.display()));
            }
            board.watch.start(port, program, &cc.egui_ctx);
            Ok(Box::new(dashboard))
        }),
    )
}

fn options(
    size: [f32; 2],
    persistence_path: Option<std::path::PathBuf>,
    persist_window: bool,
) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_ID)
            .with_app_id(APP_ID)
            .with_inner_size(size)
            .with_min_inner_size([900.0, 560.0]),
        persistence_path,
        persist_window,
        ..Default::default()
    }
}

/// The screenshot feature's run (rule 5): the fake portfolio, the window at a fixed size.
#[cfg(feature = "screenshot")]
fn screenshot_run(run: &screenshot::Run) -> eframe::Result {
    let store = match run.store() {
        Ok(store) => store,
        Err(e) => {
            eprintln!(
                "screenshot: cannot write the registry into {}: {e}",
                run.home.display()
            );
            std::process::exit(1);
        }
    };
    let options = options(run.size(), Some(run.state_file()), false);
    eframe::run_native(
        APP_ID,
        options,
        Box::new(move |cc| {
            // The run's own ledger, and no network: the fake projects' readings come from
            // `screenshot::readings`.
            let places = Places {
                store,
                ledger: Some(run.home.join("ledger.sqlite")),
                online: false,
            };
            let mut dashboard = Dashboard::new(cc, places);
            run.arrange(&mut dashboard, &cc.egui_ctx);
            Ok(Box::new(dashboard))
        }),
    )
}
