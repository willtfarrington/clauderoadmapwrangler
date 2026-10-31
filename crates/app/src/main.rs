//! clauderoadmapwrangler's desktop window (D-14, D-25). EP-1 opens an empty window; the tile shell,
//! the project list and the `screenshot` feature's capture arrive at EP-6.

// Release builds open without a console window; debug builds keep one for diagnostics.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

/// Names the window and eframe's folder in `%APPDATA%`, beside the registry (D-2, D-8).
const APP_ID: &str = "clauderoadmapwrangler";

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_ID)
            .with_app_id(APP_ID)
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(APP_ID, options, Box::new(|_cc| Ok(Box::new(Skeleton))))
}

/// The window until EP-6 replaces it with the tile shell.
struct Skeleton;

impl eframe::App for Skeleton {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(APP_ID);
            ui.label(format!(
                "Version {}: the workspace skeleton (EP-1). The project list arrives at EP-6.",
                env!("CARGO_PKG_VERSION")
            ));
        });
    }
}
