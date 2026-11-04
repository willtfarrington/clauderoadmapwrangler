//! The dashboard's look (D-25): a type scale for a 4K-class display at the system's scaling in
//! two densities, the state chips' and the stack chips' colours, the Material Symbols icon font,
//! the measure the list sizes its columns with, and the colours of a note's runs (EP-15a).

use crw_core::badges::stack::Kind;
use crw_core::registry::State;
use eframe::egui::{
    self, Align, Color32, FontFamily, FontId, Layout, Rect, RichText, TextStyle, UiBuilder, vec2,
};
use serde::{Deserialize, Serialize};

use crate::markup::Code;

pub use egui_material_icons::icons;

/// How tightly the list packs its rows (D-25).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Density {
    #[default]
    Compact,
    Comfortable,
}

impl Density {
    pub fn label(self) -> &'static str {
        match self {
            Density::Compact => "Compact",
            Density::Comfortable => "Comfortable",
        }
    }

    /// The body text size, in points; the other styles follow from it.
    fn body(self) -> f32 {
        match self {
            Density::Compact => 14.0,
            Density::Comfortable => 15.5,
        }
    }

    /// The height of one project row in the list.
    pub fn row_height(self) -> f32 {
        match self {
            Density::Compact => 26.0,
            Density::Comfortable => 34.0,
        }
    }

    /// Sets the text styles and the spacing of both themes. egui's defaults (12.5 pt body) are
    /// sized for a 1080p screen at 100 %; a 4K-class screen at its usual 150 % scaling reads
    /// better a step larger, with room between the lines.
    pub fn apply(self, ctx: &egui::Context) {
        let body = self.body();
        let gap = match self {
            Density::Compact => 4.0,
            Density::Comfortable => 7.0,
        };
        ctx.all_styles_mut(|style| {
            style.text_styles = [
                (
                    TextStyle::Heading,
                    FontId::new(body * 1.4, FontFamily::Proportional),
                ),
                (TextStyle::Body, FontId::new(body, FontFamily::Proportional)),
                (
                    TextStyle::Monospace,
                    FontId::new(body * 0.92, FontFamily::Monospace),
                ),
                (
                    TextStyle::Button,
                    FontId::new(body, FontFamily::Proportional),
                ),
                (
                    TextStyle::Small,
                    FontId::new(body * 0.84, FontFamily::Proportional),
                ),
            ]
            .into();
            style.spacing.item_spacing = vec2(8.0, gap);
            style.spacing.button_padding = vec2(7.0, gap / 2.0 + 1.0);
            style.spacing.interact_size.y = body + 8.0;
            style.spacing.menu_margin = egui::Margin::same(8);
        });
    }
}

/// The colour that marks a state (D-6), and the unadopted folders.
pub fn state_colour(state: Option<State>) -> Color32 {
    match state {
        Some(State::Active) => Color32::from_rgb(46, 160, 67),
        Some(State::Paused) => Color32::from_rgb(210, 153, 34),
        Some(State::Maintenance) => Color32::from_rgb(56, 132, 214),
        Some(State::Done) => Color32::from_rgb(120, 130, 140),
        Some(State::Archived) => Color32::from_rgb(110, 110, 110),
        Some(State::Unformalised) => Color32::from_rgb(150, 100, 205),
        None => Color32::from_rgb(225, 110, 40),
    }
}

/// The colour of a stack's chip (D-25), each near the colour its community uses.
pub fn stack_colour(kind: Kind) -> Color32 {
    match kind {
        Kind::Rust => Color32::from_rgb(206, 92, 40),
        Kind::Python => Color32::from_rgb(55, 118, 171),
        Kind::TypeScript => Color32::from_rgb(49, 120, 198),
        Kind::Node => Color32::from_rgb(84, 150, 70),
        Kind::PowerShell => Color32::from_rgb(70, 110, 180),
        Kind::Docker => Color32::from_rgb(36, 150, 237),
        Kind::Static => Color32::from_rgb(214, 90, 60),
    }
}

/// The green of a passing run, a public repository and a fresh backup.
pub fn green(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(87, 187, 105)
    } else {
        Color32::from_rgb(36, 135, 56)
    }
}

/// A small rounded label on a tint of `colour`, readable in both themes.
pub fn chip(ui: &mut egui::Ui, text: &str, colour: Color32) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let fill = colour.gamma_multiply(if dark { 0.32 } else { 0.18 });
    let ink = if dark {
        colour.lerp_to_gamma(Color32::WHITE, 0.55)
    } else {
        colour.lerp_to_gamma(Color32::BLACK, 0.45)
    };
    egui::Frame::new()
        .fill(fill)
        .corner_radius(7.0)
        .inner_margin(egui::Margin::symmetric(7, 1))
        .show(ui, |ui| {
            ui.label(RichText::new(text).small().color(ink));
        })
        .response
}

/// The width `add` takes on one line, laid out by the same code that draws it in an invisible
/// child that is never painted and takes no room: how the list sizes its columns and the Status
/// section its mark column to what they hold (EP-7a).
pub fn width_of(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) -> f32 {
    let at = ui.cursor().min;
    let mut child = ui.new_child(
        UiBuilder::new()
            // Wide enough that nothing truncates or wraps.
            .max_rect(Rect::from_min_size(at, vec2(10_000.0, 100.0)))
            .layout(Layout::left_to_right(Align::Center))
            .sizing_pass()
            .invisible(),
    );
    add(&mut child);
    child.min_rect().width()
}

/// The height `add` takes laid out top to bottom at `width`, measured as [`width_of`] measures:
/// how the legend's panel is made as tall as the key.
pub fn height_of(ui: &mut egui::Ui, width: f32, add: impl FnOnce(&mut egui::Ui)) -> f32 {
    let at = ui.cursor().min;
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(Rect::from_min_size(at, vec2(width, 10_000.0)))
            .layout(Layout::top_down(Align::Min))
            .sizing_pass()
            .invisible(),
    );
    add(&mut child);
    child.min_rect().height()
}

/// Draws `add` on a left-to-right line and pads after it to `width`, so what follows starts at
/// the same place on every line of a column, whatever `add` drew or if it drew nothing.
pub fn padded(ui: &mut egui::Ui, width: f32, add: impl FnOnce(&mut egui::Ui)) {
    let end = ui.cursor().min.x + width + ui.spacing().item_spacing.x;
    add(ui);
    ui.add_space((end - ui.cursor().min.x).max(0.0));
}

/// Makes `ui`'s labels sense hover only, for a row or a card that takes the click itself (EP-15a):
/// a selectable label senses click and drag, and egui gives the pointer to the widget on top. The
/// style is copied only when it changes.
pub fn unselectable(ui: &mut egui::Ui) {
    if ui.style().interaction.selectable_labels {
        ui.style_mut().interaction.selectable_labels = false;
    }
}

/// An icon at the text's size, in the colour given or the text's own.
pub fn icon(icon: egui_material_icons::MaterialIcon, colour: Option<Color32>) -> RichText {
    let text = icon.rich_text();
    match colour {
        Some(colour) => text.color(colour),
        None => text,
    }
}

/// The amber of warnings and of a session in progress.
pub fn amber(ui: &egui::Ui) -> Color32 {
    ui.visuals().warn_fg_color
}

/// The blue of a session at work, so that busy never looks like idle (EP-13).
pub fn blue(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(110, 168, 240)
    } else {
        Color32::from_rgb(30, 110, 200)
    }
}

/// The red of a failure, a refusal and a kill. egui's own `error_fg_color` is a pure red that
/// reads dim on a dark button (EP-15a), so each theme has a shade of its own.
pub fn red(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(255, 118, 108)
    } else {
        Color32::from_rgb(196, 32, 32)
    }
}

// A note's runs take three colours the window already uses (EP-15a, the owner's choice of fewer
// colours at the trial): its labels and gates the amber of a waiting session's words
// ([`amber`]), its references and links the violet of the unformalised state's chip
// ([`reference`]), and the code a reader can use as it is, a path or a command, the green of a
// passing run ([`code_colour`]).

/// The colour of a reference in a note (`EP-n`, `D-n`, `OT-n`, `IN-n`) and of a link's text: the
/// violet of the unformalised state's chip, lighter in the dark theme (EP-15a).
pub fn reference(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(186, 156, 255)
    } else {
        Color32::from_rgb(108, 64, 196)
    }
}

/// The ink of a code span by what it holds (EP-15a): a path or a command, which the reader can
/// open or run as written, in [`green`]; a name in the code and anything else in the code face's
/// own.
pub fn code_colour(ui: &egui::Ui, code: Code) -> Option<Color32> {
    match code {
        Code::Path | Code::Command => Some(green(ui)),
        Code::Name | Code::Other => None,
    }
}
