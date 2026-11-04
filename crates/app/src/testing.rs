//! Test support: a bare egui context driven pass by pass, with a pointer that moves, presses and
//! releases (EP-15a), so a test can ask what a click over a widget reaches, and find a button by
//! its label (EP-15c).

use eframe::egui::{self, Pos2, Rect, vec2};

/// A bare context at 1600 × 1000 points, its passes a twentieth of a second apart.
pub struct Screen {
    pub ctx: egui::Context,
    time: f64,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            ctx: egui::Context::default(),
            time: 0.0,
        }
    }

    /// One pass with `events`, `draw` drawing it; what it painted, its textures let go.
    pub fn pass(
        &mut self,
        events: Vec<egui::Event>,
        draw: impl FnMut(&mut egui::Ui),
    ) -> egui::FullOutput {
        self.time += 0.05;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 1000.0))),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(input, draw);
        // A delta left unapplied panics when dropped.
        output.textures_delta.clear();
        output
    }
}

/// The pointer moved to `at`.
pub fn moved(at: Pos2) -> Vec<egui::Event> {
    vec![egui::Event::PointerMoved(at)]
}

/// `button` pressed (`pressed`) or released at `at`.
pub fn button(at: Pos2, button: egui::PointerButton, pressed: bool) -> Vec<egui::Event> {
    vec![egui::Event::PointerButton {
        pos: at,
        button,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }]
}

/// The middle of the first text `output` paints that reads `text` exactly: where a test presses a
/// button by its label (EP-15c).
pub fn text_at(output: &egui::FullOutput, text: &str) -> Option<Pos2> {
    output
        .shapes
        .iter()
        .find_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) if shape.galley.text() == text => {
                Some(shape.pos + shape.galley.rect.center().to_vec2())
            }
            _ => None,
        })
}

/// Whether `output` paints a rectangle over `rect` filled with `fill`.
pub fn fills(output: &egui::FullOutput, rect: Rect, fill: egui::Color32) -> bool {
    output.shapes.iter().any(|clipped| match &clipped.shape {
        egui::Shape::Rect(shape) => shape.rect == rect && shape.fill == fill,
        _ => false,
    })
}
