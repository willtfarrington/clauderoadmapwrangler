//! The window's popup (D-21 addendum; EP-15): the toasts the rules send at once, in a small window
//! of the tool's own at the lower right of the screen, in place of Windows' toasts, which the owner
//! keeps off. It is always on top, never takes the keyboard focus when it appears, and has no
//! taskbar button. It lists the latest toasts, the newest first, three at most with a count of the
//! rest, each with Show (where it leads) and Dismiss, and stays until the owner dismisses them or
//! acts on them.
//!
//! It is a deferred viewport of eframe 0.37 (EP-15d). The window's pass keeps it open while it
//! holds toasts, shown or hidden in the tray (`Board::attend` asks a minimized window to paint
//! while the popup holds anything). The popup's own pass draws the toasts and applies the owner's
//! choice there and then, over state the two passes share. An immediate viewport, as EP-15 had
//! it, is given its window through a thread-local that eframe leaves unset on one of the paths
//! that paint a hidden window (`new_events`), which panicked the window on the first notification
//! while it was in the tray. A deferred one is given its window with the event loop in hand at the
//! start of every pass. In a screenshot run the popup is drawn inside the window, so that the
//! picture holds it.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

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

/// What the owner did in the popup, by the place of a toast as it was drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Show(usize),
    Dismiss(usize),
    All,
}

/// What the window's pass and the popup's own share (EP-15d).
#[derive(Default)]
struct Shared {
    /// The newest first.
    items: Vec<Item>,
    /// The toasts the owner asked to see, for the window's next pass.
    shown: Vec<Toast>,
    /// The height the popup's contents took in its last pass.
    height: Option<f32>,
    /// A toast came since the window's last pass: the popup is woken to draw it, since showing a
    /// deferred viewport again repaints nothing.
    fresh: bool,
}

/// The popup's viewport.
fn popup_id() -> egui::ViewportId {
    egui::ViewportId::from_hash_of("popup")
}

impl Shared {
    /// Applies the owner's choice in the pass that drew the toasts, so that a toast that arrives
    /// before the window's pass never shifts the one chosen. Whether anything changed.
    fn apply(&mut self, choice: Option<Choice>) -> bool {
        match choice {
            Some(Choice::Show(n)) if n < self.items.len() => {
                let item = self.items.remove(n);
                self.shown.push(item.toast);
                true
            }
            Some(Choice::Dismiss(n)) if n < self.items.len() => {
                self.items.remove(n);
                true
            }
            Some(Choice::All) => {
                self.items.clear();
                true
            }
            _ => false,
        }
    }
}

/// The popup's toasts, shared with its own pass.
#[derive(Default)]
pub struct Popup {
    shared: Arc<Mutex<Shared>>,
}

/// The shared state, whatever a panic left in it: a toast list is never half written.
fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Popup {
    pub fn push(&mut self, toast: Toast) {
        let mut shared = lock(&self.shared);
        shared.items.insert(
            0,
            Item {
                toast,
                at: Local::now(),
            },
        );
        shared.items.truncate(KEPT);
        shared.fresh = true;
    }

    pub fn len(&self) -> usize {
        lock(&self.shared).items.len()
    }

    pub fn is_empty(&self) -> bool {
        lock(&self.shared).items.is_empty()
    }

    /// Keeps the popup open while it holds toasts. Returns those the owner asked to see since the
    /// last call, which have left it, as dismissed ones have.
    pub fn ui(&mut self, ctx: &egui::Context) -> Vec<Toast> {
        if ctx.embed_viewports() {
            // Drawn inside the window (a screenshot run, a test), where the real one would sit.
            let mut shared = lock(&self.shared);
            if !shared.items.is_empty() {
                let choice = egui::Area::new(egui::Id::unique("popup"))
                    .anchor(egui::Align2::RIGHT_BOTTOM, [-RIGHT, -40.0])
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        egui::Frame::window(ui.style())
                            .inner_margin(MARGIN)
                            .show(ui, |ui| contents(ui, &shared.items))
                            .inner
                    })
                    .inner;
                shared.apply(choice);
            }
            return std::mem::take(&mut shared.shown);
        }
        let (height, fresh) = {
            let mut shared = lock(&self.shared);
            if shared.items.is_empty() {
                // The next popup starts at its first height; its viewport, not shown this pass,
                // closes.
                shared.height = None;
            }
            let height = (!shared.items.is_empty()).then_some(shared.height);
            (height, std::mem::take(&mut shared.fresh))
        };
        if let Some(height) = height {
            self.window(ctx, height);
            if fresh {
                ctx.request_repaint_of(popup_id());
            }
        }
        std::mem::take(&mut lock(&self.shared).shown)
    }

    /// The popup's own window at the screen's lower right, as tall as its contents were in its
    /// last pass; closing it dismisses everything. The lock is never held while eframe has the
    /// viewport, whose pass takes it.
    fn window(&self, ctx: &egui::Context, height: Option<f32>) {
        let size = egui::vec2(WIDTH, height.unwrap_or(FIRST_HEIGHT));
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
        let shared = Arc::clone(&self.shared);
        ctx.show_viewport_deferred(popup_id(), builder, move |ui, _| {
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
            let mut shared = lock(&shared);
            let frame = egui::Frame::NONE
                .inner_margin(MARGIN)
                .show(ui, |ui| contents(ui, &shared.items));
            let choice = if closed {
                Some(Choice::All)
            } else {
                frame.inner
            };
            let measured = frame.response.rect.height().ceil();
            let resized = shared.height != Some(measured);
            shared.height = Some(measured);
            let changed = shared.apply(choice);
            drop(shared);
            if changed || resized {
                // The window's pass takes what was shown and sizes the popup anew.
                ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
            }
        });
    }
}

/// The popup's contents: its name with the count of the toasts out of view and Dismiss all,
/// then the latest toasts.
fn contents(ui: &mut egui::Ui, items: &[Item]) -> Option<Choice> {
    ui.set_width(WIDTH - 2.0 * MARGIN);
    let mut choice = None;
    let more = items.len().saturating_sub(SHOWN);
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
    for (n, item) in items.iter().take(SHOWN).enumerate() {
        ui.separator();
        ui.horizontal(|ui| {
            let colour = match item.toast.kind {
                Some(Kind::Waiting) => style::amber(ui),
                Some(Kind::Failed) => style::red(ui),
                Some(Kind::Finished) => style::green(ui),
                // The autopilot's violet, as a chained tab's line (EP-19, EP-19a).
                Some(Kind::Chain) => style::reference(ui),
                _ => style::blue(ui),
            };
            ui.label(RichText::new("●").color(colour));
            // The time at the right; the title in the room left of it, cut to fit.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(item.at.format("%H:%M").to_string()).weak());
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add(egui::Label::new(RichText::new(&item.toast.title).strong()).truncate());
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
                    Target::Usage => "Show the window on the Usage pane",
                    Target::ChainLog => {
                        "Show the window on the Sessions pane's All tab, where the chain log is"
                    }
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
        assert_eq!(lock(&popup.shared).items[0].toast, Toast::test());
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

    /// The owner's choice is applied in the pass that drew the toasts (EP-15d): a toast that
    /// arrives before the window's pass shifts nothing, and Show hands the chosen one on.
    #[test]
    fn a_choice_takes_the_toast_that_was_drawn() {
        let mut popup = Popup::default();
        popup.push(Toast::tray(0));
        popup.push(Toast::test());
        // The popup's pass drew the test first and the owner pressed its Show.
        assert!(lock(&popup.shared).apply(Some(Choice::Show(0))));
        // A new toast arrives before the window's pass.
        popup.push(Toast::tray(2));
        let ctx = egui::Context::default();
        let mut chosen = Vec::new();
        ctx.run_ui(egui::RawInput::default(), |ui| {
            chosen = popup.ui(ui.ctx());
        })
        .drop_without_applying_deltas();
        assert_eq!(chosen, [Toast::test()]);
        assert_eq!(popup.len(), 2);
        // Dismiss takes the one drawn there, and Dismiss all the rest; a place past the end is
        // nothing.
        assert!(lock(&popup.shared).apply(Some(Choice::Dismiss(1))));
        assert_eq!(lock(&popup.shared).items[0].toast, Toast::tray(2));
        assert!(!lock(&popup.shared).apply(Some(Choice::Dismiss(5))));
        assert!(lock(&popup.shared).apply(Some(Choice::All)));
        assert!(popup.is_empty());
        assert!(!lock(&popup.shared).apply(None));
    }

    /// What a window's pass showed of the popup: whether as a deferred viewport, when it asked to
    /// paint it, and its callback, as eframe receives them.
    type Seen = (
        bool,
        std::time::Duration,
        Option<std::sync::Arc<egui::DeferredViewportUiCallback>>,
    );

    fn window_pass(ctx: &egui::Context, popup: &mut Popup) -> Option<Seen> {
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            popup.ui(ui.ctx());
        });
        let seen = output.viewport_output.get(&popup_id()).map(|viewport| {
            (
                matches!(viewport.class, egui::ViewportClass::Deferred),
                viewport.repaint_delay,
                viewport.viewport_ui_cb.clone(),
            )
        });
        output.drop_without_applying_deltas();
        seen
    }

    /// Outside a screenshot run the popup is a deferred viewport (EP-15d): the window's pass keeps
    /// it open while it holds toasts and wakes it for a new one; its own pass, run as eframe runs
    /// it, draws the toasts and measures them; once empty it is left out, and eframe closes it.
    #[test]
    fn the_popup_is_a_deferred_viewport_with_a_pass_of_its_own() {
        let ctx = egui::Context::default();
        ctx.set_embed_viewports(false);
        let now = std::time::Duration::ZERO;
        let mut popup = Popup::default();
        popup.push(Toast::test());
        let (deferred, delay, callback) = window_pass(&ctx, &mut popup).unwrap();
        assert!(deferred, "a viewport of its own");
        assert_eq!(delay, now, "woken for its new toast");
        // The popup's own pass draws the toast and measures its height for the window's next.
        let callback = callback.expect("a deferred viewport's callback");
        let mut input = egui::RawInput {
            viewport_id: popup_id(),
            ..egui::RawInput::default()
        };
        // eframe lists every viewport it holds in each pass's input.
        input
            .viewports
            .insert(popup_id(), egui::ViewportInfo::default());
        ctx.run_ui(input, |ui| callback(ui))
            .drop_without_applying_deltas();
        assert!(
            lock(&popup.shared)
                .height
                .is_some_and(|height| height > 0.0)
        );
        assert_eq!(popup.len(), 1, "nothing chosen");
        // A new toast wakes it again, and an empty popup is left out.
        popup.push(Toast::tray(1));
        let (_, delay, _) = window_pass(&ctx, &mut popup).unwrap();
        assert_eq!(delay, now);
        lock(&popup.shared).items.clear();
        assert!(window_pass(&ctx, &mut popup).is_none());
    }
}
