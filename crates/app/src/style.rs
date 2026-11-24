//! The dashboard's look (D-25): a type scale for a 4K-class display at the system's scaling in
//! two densities, the dark theme's palette (EP-49), the state chips' and the stack chips'
//! colours, the Material Symbols icon font, the measure the list sizes its columns with, and the
//! colours of a note's runs (EP-15a).

use crw_core::badges::stack::Kind;
use crw_core::registry::State;
use eframe::egui::style::{Selection, WidgetVisuals, Widgets};
use eframe::egui::{
    self, Align, Color32, FontFamily, FontId, Layout, Rect, RichText, Stroke, TextStyle, Theme,
    UiBuilder, vec2,
};
use serde::{Deserialize, Serialize};

use crate::markup::Code;

pub use egui_material_icons::icons;

// The dark theme (EP-49; D-25 addendum of 2026-11-22): a near-black base with more contrast,
// after the One Hunter extension's Vercel theme (`OneHunter-Vercel-color-theme.json`, its
// `colors`), each value named by the theme's keys it comes from. An alpha value of the theme's
// is composed over the black, as VS Code composes it. Light stays egui's own.

/// The panels and a text box's well: the theme's `editor.background`, `panel.background` and
/// `input.background`.
const BASE: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
/// Dialogs, menus, tooltips and the popup, and a pressed or focused button (darker than at rest,
/// as egui's own Dark has it, inside the white outline): `list.hoverBackground` and
/// `editorGroupHeader.tabsBackground`.
const RAISED: Color32 = Color32::from_rgb(0x0A, 0x0A, 0x0A);
/// The cards: `editorWidget.background` at 35 % over the black.
const CARD: Color32 = Color32::from_rgb(0x12, 0x14, 0x16);
/// A button at rest: `editorWidget.background` at 50 %.
const BUTTON: Color32 = Color32::from_rgb(0x1A, 0x1C, 0x1F);
/// The wells, egui's extreme background: the tab bars, a progress bar's and a meter's track, a
/// plot's and a sparkline's ground, a heatmap's empty day and a code block. egui's Dark sinks
/// them below its panels, which the black leaves no room for, so they stand a step above it:
/// `editorWidget.background` at 50 %.
const WELL: Color32 = Color32::from_rgb(0x1A, 0x1C, 0x1F);
/// Code's background, and a progress pill's track: `editorWidget.background` at 60 %.
const CODE: Color32 = Color32::from_rgb(0x1F, 0x22, 0x25);
/// A hovered button, row or group header: `editorWidget.background` at 75 %.
const HOVER: Color32 = Color32::from_rgb(0x27, 0x2B, 0x2F);
/// The separators, and the fill of a checkbox, a radio button and a slider's rail:
/// `editorWidget.background` and `editorRuler.foreground`.
const WIDGET: Color32 = Color32::from_rgb(0x34, 0x39, 0x3E);
/// The frame of dialogs, buttons and text boxes, and the active tab's outline: `panel.border`,
/// `editorWidget.border` and `input.border`.
const BORDER: Color32 = Color32::from_rgb(0x45, 0x4D, 0x54);
/// A hovered frame: the border two fifths of the way to the text.
const BORDER_HOVER: Color32 = Color32::from_rgb(0x81, 0x89, 0x90);
/// The text: `foreground` and `editor.foreground`.
const TEXT: Color32 = Color32::from_rgb(0xDC, 0xE3, 0xEA);
/// A hovered widget's text: the text halfway to white.
const TEXT_HOVER: Color32 = Color32::from_rgb(0xEE, 0xF1, 0xF5);
/// Weak text: `tab.inactiveForeground` (the text at 53 % over the black) lifted to 72 %, so that
/// it keeps 4.5 to 1 on a card, a hovered row and the selection.
const WEAK: Color32 = Color32::from_rgb(0x9E, 0xA3, 0xA8);
/// The selection: `editor.selectionBackground`, `#43AAF9` at a third over the black.
const SELECTION: Color32 = Color32::from_rgb(0x16, 0x39, 0x53);
/// A selected button's text and a focused text box's outline: `terminal.ansiBrightBlue`.
const SELECTION_INK: Color32 = Color32::from_rgb(0xA7, 0xD1, 0xF5);

/// The red of a failure in the dark theme (EP-15a), egui's own error colour there too (EP-49).
const RED_DARK: Color32 = Color32::from_rgb(255, 118, 108);
/// The violet of a reference in the dark theme (EP-15a), and of a link (EP-49).
const VIOLET_DARK: Color32 = Color32::from_rgb(186, 156, 255);

/// The dark theme's visuals (EP-49): the palette above, over egui's own Dark for what it does
/// not name (the shadows, the corner radii, the text cursor, the handles). The warnings' amber
/// stays egui's orange, since it is [`amber`].
pub fn dark() -> egui::Visuals {
    let stock = egui::Visuals::dark();
    egui::Visuals {
        dark_mode: true,
        weak_text_color: Some(WEAK),
        widgets: Widgets {
            noninteractive: WidgetVisuals {
                bg_fill: BASE,
                weak_bg_fill: BASE,
                bg_stroke: Stroke::new(1.0, WIDGET),
                fg_stroke: Stroke::new(1.0, TEXT),
                ..stock.widgets.noninteractive
            },
            inactive: WidgetVisuals {
                bg_fill: WIDGET,
                weak_bg_fill: BUTTON,
                bg_stroke: Stroke::new(1.0, BORDER),
                fg_stroke: Stroke::new(1.0, TEXT),
                ..stock.widgets.inactive
            },
            hovered: WidgetVisuals {
                bg_fill: BORDER,
                weak_bg_fill: HOVER,
                bg_stroke: Stroke::new(1.0, BORDER_HOVER),
                fg_stroke: Stroke::new(1.5, TEXT_HOVER),
                ..stock.widgets.hovered
            },
            active: WidgetVisuals {
                bg_fill: BORDER,
                weak_bg_fill: RAISED,
                bg_stroke: Stroke::new(1.0, Color32::WHITE),
                fg_stroke: Stroke::new(2.0, Color32::WHITE),
                ..stock.widgets.active
            },
            open: WidgetVisuals {
                bg_fill: RAISED,
                weak_bg_fill: HOVER,
                bg_stroke: Stroke::new(1.0, BORDER),
                fg_stroke: Stroke::new(1.0, TEXT),
                ..stock.widgets.open
            },
        },
        selection: Selection {
            bg_fill: SELECTION,
            stroke: Stroke::new(1.0, SELECTION_INK),
        },
        hyperlink_color: VIOLET_DARK,
        faint_bg_color: CARD,
        extreme_bg_color: WELL,
        text_edit_bg_color: Some(BASE),
        code_bg_color: CODE,
        error_fg_color: RED_DARK,
        window_fill: RAISED,
        window_stroke: Stroke::new(1.0, BORDER),
        panel_fill: BASE,
        ..stock
    }
}

/// The track of a progress pill (EP-49): in the dark theme code's background, which stands off
/// the black and a card alike, since the cards take the faint background the track had; in the
/// light theme the faint background, as before.
pub fn track(ui: &egui::Ui) -> Color32 {
    let visuals = ui.visuals();
    if visuals.dark_mode {
        visuals.code_bg_color
    } else {
        visuals.faint_bg_color
    }
}

/// Gives the window its two themes once, before the owner's choice of them is set: the dark
/// theme's visuals (EP-49) and egui's own Light. egui keeps both and never stores them, so the
/// View menu's System, Light and Dark pick between them for the window's life.
pub fn apply_themes(ctx: &egui::Context) {
    ctx.set_visuals_of(Theme::Dark, dark());
}

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

/// A chip's tint and its ink for `colour`: lifted toward white in the dark theme, lowered toward
/// black in the light one.
fn chip_colours(dark: bool, colour: Color32) -> (Color32, Color32) {
    if dark {
        (
            colour.gamma_multiply(0.32),
            colour.lerp_to_gamma(Color32::WHITE, 0.55),
        )
    } else {
        (
            colour.gamma_multiply(0.18),
            colour.lerp_to_gamma(Color32::BLACK, 0.45),
        )
    }
}

/// A small rounded label on a tint of `colour`, readable in both themes.
pub fn chip(ui: &mut egui::Ui, text: &str, colour: Color32) -> egui::Response {
    let (fill, ink) = chip_colours(ui.visuals().dark_mode, colour);
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
        RED_DARK
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
        VIOLET_DARK
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

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG 2's relative luminance of an opaque colour.
    fn luminance(colour: Color32) -> f32 {
        let channel = |value: u8| {
            let c = f32::from(value) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(colour.r()) + 0.7152 * channel(colour.g()) + 0.0722 * channel(colour.b())
    }

    /// The contrast of two opaque colours, from 1 to 21 (WCAG 2).
    fn contrast(a: Color32, b: Color32) -> f32 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// `top`, premultiplied as egui keeps a colour, laid over the opaque `bottom`, in the gamma
    /// space egui blends in.
    fn over(top: Color32, bottom: Color32) -> Color32 {
        let keep = 1.0 - f32::from(top.a()) / 255.0;
        let mix = |top: u8, bottom: u8| {
            (f32::from(top) + f32::from(bottom) * keep)
                .round()
                .min(255.0) as u8
        };
        Color32::from_rgb(
            mix(top.r(), bottom.r()),
            mix(top.g(), bottom.g()),
            mix(top.b(), bottom.b()),
        )
    }

    /// What `read` takes from a pass's `Ui` in the dark theme, set as the window sets it.
    fn in_dark<T>(read: impl FnOnce(&egui::Ui) -> T) -> T {
        let ctx = egui::Context::default();
        apply_themes(&ctx);
        ctx.set_theme(egui::ThemePreference::Dark);
        let mut read = Some(read);
        let mut taken = None;
        ctx.run_ui(egui::RawInput::default(), |ui| {
            if let Some(read) = read.take() {
                taken = Some(read(ui));
            }
        })
        .drop_without_applying_deltas();
        taken.expect("the pass ran")
    }

    /// Every accent's colour in the dark theme, as the code picks it.
    fn accents(ui: &egui::Ui) -> Vec<(&'static str, Color32)> {
        vec![
            ("green", green(ui)),
            ("amber", amber(ui)),
            ("blue", blue(ui)),
            ("red", red(ui)),
            ("violet", reference(ui)),
        ]
    }

    /// The surfaces the dark theme draws text on, filled as the code fills them.
    fn surfaces(visuals: &egui::Visuals) -> Vec<(&'static str, Color32)> {
        let panel = visuals.panel_fill;
        vec![
            ("the panel", panel),
            ("a dialog", visuals.window_fill),
            ("a card", visuals.faint_bg_color),
            ("code", visuals.code_bg_color),
            ("a well (a tab bar, a code block)", visuals.extreme_bg_color),
            ("a button", visuals.widgets.inactive.weak_bg_fill),
            (
                "a pressed or focused button",
                visuals.widgets.active.weak_bg_fill,
            ),
            (
                "a hovered row",
                over(visuals.widgets.hovered.weak_bg_fill, panel),
            ),
            // `list::tint`'s selected row.
            (
                "the selected row",
                over(visuals.selection.bg_fill.gamma_multiply(0.55), panel),
            ),
            // A selected entry, whose weak or coloured words keep their colours (the model
            // picker's further models are weak).
            ("the selection", visuals.selection.bg_fill),
        ]
    }

    /// The window's Dark is EP-49's, its Light egui's own (the owner's choice at EP-45).
    #[test]
    fn the_window_gets_the_dark_theme_and_keeps_egui_s_light() {
        let ctx = egui::Context::default();
        apply_themes(&ctx);
        assert_eq!(ctx.style_of(Theme::Dark).visuals, dark());
        assert_eq!(ctx.style_of(Theme::Light).visuals, egui::Visuals::light());
        let visuals = dark();
        assert!(visuals.dark_mode);
        assert_eq!(visuals.panel_fill, Color32::BLACK);
        assert_eq!(visuals.weak_text_color, Some(WEAK));
        // The warnings' amber is egui's orange, as before.
        assert_eq!(visuals.warn_fg_color, egui::Visuals::dark().warn_fg_color);
        // A pass in Dark draws it.
        let panel = in_dark(|ui| ui.visuals().panel_fill);
        assert_eq!(panel, BASE);
    }

    /// Each accent keeps its meaning and its shade unless its contrast as text falls under 4.5
    /// to 1 on a surface of the new base (EP-49): measured on each, with the text colours.
    #[test]
    fn every_ink_keeps_four_and_a_half_to_one_on_every_dark_surface() {
        let (visuals, mut inks) = in_dark(|ui| (ui.visuals().clone(), accents(ui)));
        inks.extend([
            ("the text", visuals.text_color()),
            ("weak text", visuals.weak_text_color()),
            ("strong text", visuals.strong_text_color()),
            ("a link", visuals.hyperlink_color),
            ("egui's error", visuals.error_fg_color),
        ]);
        let mut low = Vec::new();
        for (surface, fill) in surfaces(&visuals) {
            for (ink, colour) in &inks {
                let ratio = contrast(*colour, fill);
                if ratio < 4.5 {
                    low.push(format!("{ink} on {surface}: {ratio:.2}"));
                }
            }
        }
        // A selected button's own text on the selection.
        let ratio = contrast(visuals.selection.stroke.color, visuals.selection.bg_fill);
        if ratio < 4.5 {
            low.push(format!("a selected button's text: {ratio:.2}"));
        }
        assert!(low.is_empty(), "under 4.5 to 1: {low:#?}");
    }

    /// The wells and a progress pill's track stand off the black and the cards they sit on
    /// (the review's finding: egui's extreme background a hair off the black, and the pill's
    /// track the cards' own fill, left them unseen).
    #[test]
    fn the_wells_and_the_tracks_stand_off_the_black_and_the_cards() {
        let (visuals, pill) = in_dark(|ui| (ui.visuals().clone(), track(ui)));
        // Steps between near-blacks are small by nature: egui's Dark set its cards 1.05 to 1
        // off its panels, and the review found a well 1.06 to 1 off the black unseen.
        for (name, fill) in [("a well", visuals.extreme_bg_color), ("a track", pill)] {
            for (under, ground, least) in [
                ("the panel", visuals.panel_fill, 1.15),
                ("a card", visuals.faint_bg_color, 1.07),
            ] {
                let ratio = contrast(fill, ground);
                assert!(ratio >= least, "{name} on {under}: {ratio:.3}");
            }
        }
        // A pressed button is darker than one at rest, as in egui's own Dark.
        assert!(
            luminance(visuals.widgets.active.weak_bg_fill)
                < luminance(visuals.widgets.inactive.weak_bg_fill)
        );
        // Light keeps the faint track.
        let ctx = egui::Context::default();
        apply_themes(&ctx);
        ctx.set_theme(egui::ThemePreference::Light);
        let mut light = None;
        ctx.run_ui(egui::RawInput::default(), |ui| {
            light = Some((track(ui), ui.visuals().faint_bg_color));
        })
        .drop_without_applying_deltas();
        let (light_track, faint) = light.expect("the pass ran");
        assert_eq!(light_track, faint);
    }

    /// A chip's ink on its tint, for every colour a chip takes, on the surfaces chips sit on.
    #[test]
    fn every_chip_s_ink_keeps_four_and_a_half_to_one_on_its_tint_in_dark() {
        let (visuals, accents) = in_dark(|ui| (ui.visuals().clone(), accents(ui)));
        let states = [
            None,
            Some(State::Active),
            Some(State::Paused),
            Some(State::Maintenance),
            Some(State::Done),
            Some(State::Archived),
            Some(State::Unformalised),
        ];
        let colours: Vec<Color32> = states
            .into_iter()
            .map(state_colour)
            .chain(Kind::ALL.into_iter().map(stack_colour))
            .chain(accents.into_iter().map(|(_, colour)| colour))
            .chain([visuals.weak_text_color()])
            .collect();
        let mut low = Vec::new();
        for (surface, fill) in [
            ("the panel", visuals.panel_fill),
            ("a card", visuals.faint_bg_color),
            ("a dialog", visuals.window_fill),
        ] {
            for colour in &colours {
                let (tint, ink) = chip_colours(true, *colour);
                let ratio = contrast(ink, over(tint, fill));
                if ratio < 4.5 {
                    low.push(format!("{colour:?}'s chip on {surface}: {ratio:.2}"));
                }
            }
        }
        assert!(low.is_empty(), "under 4.5 to 1: {low:#?}");
    }

    /// The arithmetic itself, on WCAG's own extremes and a known pair.
    #[test]
    fn the_contrast_arithmetic_is_wcag_s() {
        assert!((contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
        assert!((contrast(BASE, BASE) - 1.0).abs() < 0.001);
        // #767676 on white is WCAG's usual 4.54 to 1.
        let grey = Color32::from_gray(0x76);
        assert!((contrast(grey, Color32::WHITE) - 4.54).abs() < 0.01);
        // Half of white over black is a mid grey.
        assert_eq!(
            over(Color32::WHITE.gamma_multiply(0.5), BASE),
            Color32::from_gray(128)
        );
    }
}
