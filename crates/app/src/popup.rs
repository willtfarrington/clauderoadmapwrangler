//! The window's popup (D-21 addendum; EP-15): the toasts the rules send at once, in a small window
//! of the tool's own at the lower right of the screen, in place of Windows' toasts, which the owner
//! keeps off. It is always on top, never takes the keyboard focus when it appears, and has no
//! taskbar button. It lists the latest toasts, the newest first, three at most with a count of the
//! rest, each with Show (where it leads) and Dismiss, and stays until the owner dismisses them or
//! acts on them. It is an immediate viewport of eframe 0.37, drawn by the window's pass whether the
//! window is shown or hidden in the tray (`Board::attend` asks a minimized window to paint while
//! the popup holds anything); in a screenshot run it is drawn inside the window, so that the
//! picture holds it.

use chrono::{DateTime, Local};
use crw_core::notify::Kind;
use eframe::egui::{self, RichText};

use crate::style;
use crate::toasts::{Target, Toast};

/// The popup's width in points.
const WIDTH: f32 = 420.0;
/// Its height before its contents are measured: a frame and three toasts.
const FIRST_HEIGHT: f32 = 320.0;
/// The room around its contents.
const MARGIN: f32 = 12.0;
/// How many toasts it shows at once, the rest counted, and how many it keeps at most.
pub const SHOWN: usize = 3;
const KEPT: usize = 50;
/// Its distance from the screen's right edge, and from the bottom: the taskbar's room.
const RIGHT: f32 = 16.0;
const BOTTOM: f32 = 64.0;

/// A toast in the popup, and when it came.
struct Item {
    toast: Toast,
    at: DateTime<Local>,
}

/// What the owner did in the popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Show(usize),
    Dismiss(usize),
    All,
}

/// The popup's toasts, the newest first, and the height its contents took.
#[derive(Default)]
pub struct Popup {
    items: Vec<Item>,
    height: Option<f32>,
}

impl Popup {
    pub fn push(&mut self, toast: Toast) {
        self.items.insert(
            0,
            Item {
                toast,
                at: Local::now(),
            },
        );
        self.items.truncate(KEPT);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Draws the popup while it holds toasts. Returns those the owner asked to see, which leave
    /// it, as dismissed ones do.
    pub fn ui(&mut self, ctx: &egui::Context) -> Vec<Toast> {
        if self.items.is_empty() {
            self.height = None;
            return Vec::new();
        }
        let choice = if ctx.embed_viewports() {
            // Drawn inside the window (a screenshot run, a test), where the real one would sit.
            egui::Area::new(egui::Id::unique("popup"))
                .anchor(egui::Align2::RIGHT_BOTTOM, [-RIGHT, -40.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::window(ui.style())
                        .inner_margin(MARGIN)
                        .show(ui, |ui| self.contents(ui))
                        .inner
                })
                .inner
        } else {
            self.window(ctx)
        };
        match choice {
            Some(Choice::Show(n)) if n < self.items.len() => vec![self.items.remove(n).toast],
            Some(Choice::Dismiss(n)) if n < self.items.len() => {
                self.items.remove(n);
                Vec::new()
            }
            Some(Choice::All) => {
                self.items.clear();
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    /// The popup as a window of its own at the screen's lower right, as tall as its contents
    /// were in the last frame; closing it dismisses everything.
    fn window(&mut self, ctx: &egui::Context) -> Option<Choice> {
        let size = egui::vec2(WIDTH, self.height.unwrap_or(FIRST_HEIGHT));
        let screen = ctx
            .input(|i| i.viewport().monitor_size)
            .unwrap_or(egui::vec2(1920.0, 1080.0));
        let corner = (screen - size - egui::vec2(RIGHT, BOTTOM)).to_pos2();
        let builder = egui::ViewportBuilder::default()
            .with_title("Roadmap Wrangler")
            .with_decorations(false)
            .with_always_on_top()
            .with_taskbar(false)
            .with_active(false)
            .with_resizable(false)
            .with_inner_size(size)
            .with_position(corner);
        let (choice, height) = ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("popup"),
            builder,
            |ui, _| {
                let rect = ui.max_rect();
                ui.painter()
                    .rect_filled(rect, 0.0, ui.visuals().window_fill());
                ui.painter().rect_stroke(
                    rect.shrink(0.5),
                    0.0,
                    ui.visuals().window_stroke(),
                    egui::StrokeKind::Inside,
                );
                let closed = ui.ctx().input(|i| i.viewport().close_requested());
                let frame = egui::Frame::NONE
                    .inner_margin(MARGIN)
                    .show(ui, |ui| self.contents(ui));
                let choice = if closed {
                    Some(Choice::All)
                } else {
                    frame.inner
                };
                (choice, frame.response.rect.height())
            },
        );
        self.height = Some(height.ceil());
        choice
    }

    /// The popup's contents: its name with the count of the toasts out of view and Dismiss all,
    /// then the latest toasts.
    fn contents(&self, ui: &mut egui::Ui) -> Option<Choice> {
        ui.set_width(WIDTH - 2.0 * MARGIN);
        let mut choice = None;
        let more = self.items.len().saturating_sub(SHOWN);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Roadmap Wrangler").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button("Dismiss all")
                    .on_hover_text("Close the popup: the tray icon still counts what waits")
                    .clicked()
                {
                    choice = Some(Choice::All);
                }
                if more > 0 {
                    ui.label(RichText::new(format!("and {more} more")).weak());
                }
            });
        });
        for (n, item) in self.items.iter().take(SHOWN).enumerate() {
            ui.separator();
            ui.horizontal(|ui| {
                let colour = match item.toast.kind {
                    Some(Kind::Waiting) => style::amber(ui),
                    Some(Kind::Failed) => style::red(ui),
                    Some(Kind::Finished) => style::green(ui),
                    _ => style::blue(ui),
                };
                ui.label(RichText::new("●").color(colour));
                // The time at the right; the title in the room left of it, cut to fit.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(item.at.format("%H:%M").to_string()).weak());
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add(
                            egui::Label::new(RichText::new(&item.toast.title).strong()).truncate(),
                        );
                    });
                });
            });
            ui.add(egui::Label::new(&item.toast.text).wrap());
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Dismiss").clicked() {
                        choice = Some(Choice::Dismiss(n));
                    }
                    let hover = match &item.toast.target {
                        Target::VsCode(_) => "Bring its VS Code window forward",
                        Target::Decisions => "Show the window on the Decisions pane",
                        Target::Session(_) => "Show the window on the session's tab",
                        Target::Window => "Show the window",
                    };
                    if ui
                        .button(RichText::new("Show").strong())
                        .on_hover_text(hover)
                        .clicked()
                    {
                        choice = Some(Choice::Show(n));
                    }
                });
            });
        }
        choice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_popup_keeps_the_newest_first_and_a_bounded_list() {
        let mut popup = Popup::default();
        assert!(popup.is_empty());
        for _ in 0..KEPT + 5 {
            popup.push(Toast::tray(0));
        }
        popup.push(Toast::test());
        assert_eq!(popup.len(), KEPT);
        assert_eq!(popup.items[0].toast, Toast::test());
    }

    #[test]
    fn the_popup_draws_inside_a_bare_context_and_asks_nothing_unasked() {
        // A bare context draws viewports inside itself, as a screenshot run does.
        let ctx = egui::Context::default();
        assert!(ctx.embed_viewports());
        let mut popup = Popup::default();
        popup.push(Toast::test());
        popup.push(Toast::tray(1));
        for _ in 0..2 {
            let mut chosen = Vec::new();
            ctx.run_ui(egui::RawInput::default(), |ui| {
                chosen = popup.ui(ui.ctx());
            })
            .drop_without_applying_deltas();
            assert!(chosen.is_empty());
        }
        assert_eq!(popup.len(), 2);
    }
}
