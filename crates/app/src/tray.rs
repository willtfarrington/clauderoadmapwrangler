//! The tray icon (D-21, D-29; EP-15): the window's mark in the notification area while it runs,
//! shown or hidden. The tool draws the icon itself until EP-28 brings the app's own: a blue tile
//! with a roadmap's three rows, a red disc with the count of what waits for the owner
//! (`Board::decisions_count`), and an amber dot while a badge event is new since the window was
//! last in front (`toasts::Attention`). The tooltip says both in words. A left click shows the
//! window. The menu says what waits and what is new, and holds Show, Start next for each pinned
//! project (the launch dialog on its next brief, D-17), Pause autopilot or Resume autopilot (the
//! owner's Pause of every chain, D-22; EP-19) and Quit. The icon and the menu change only when what they show does: the menu's texts are set
//! in place, and a menu replaced for a new list of pinned projects is kept a minute longer, in
//! case it was open. `main` starts it once, on the main thread, whose message loop serves it:
//! never in a test or a screenshot run.

use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use eframe::egui;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// The icon's side until the window knows its scale: Windows' small icon at 200 %.
pub const SIZE: u32 = 32;

/// Windows keeps a tooltip to 127 characters, and tray-icon copies one more without its end.
const TOOLTIP: usize = 120;

/// A replaced menu is kept this long, in case it was open when it was replaced.
const RETIRED_FOR: Duration = Duration::from_secs(60);

const BLUE: [u8; 3] = [37, 99, 235];
const WHITE: [u8; 3] = [255, 255, 255];
const RED: [u8; 3] = [220, 38, 38];
const AMBER: [u8; 3] = [245, 158, 11];

/// The count's digits and its plus, three pixels wide and five high: each row's three bits, the
/// leftmost the highest.
const GLYPHS: [(char, [u8; 5]); 11] = [
    ('0', [0b111, 0b101, 0b101, 0b101, 0b111]),
    ('1', [0b010, 0b110, 0b010, 0b010, 0b111]),
    ('2', [0b111, 0b001, 0b111, 0b100, 0b111]),
    ('3', [0b111, 0b001, 0b111, 0b001, 0b111]),
    ('4', [0b101, 0b101, 0b111, 0b001, 0b001]),
    ('5', [0b111, 0b100, 0b111, 0b001, 0b111]),
    ('6', [0b111, 0b100, 0b111, 0b101, 0b111]),
    ('7', [0b111, 0b001, 0b001, 0b001, 0b001]),
    ('8', [0b111, 0b101, 0b111, 0b101, 0b111]),
    ('9', [0b111, 0b101, 0b111, 0b001, 0b111]),
    ('+', [0b000, 0b010, 0b111, 0b010, 0b000]),
];

/// What the tray asks of the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    Show,
    /// The launch dialog on the project's next brief.
    Start(String),
    /// The owner's Pause of every chain, or its Resume (D-22; EP-19).
    Pause,
    /// The newer build installed and opened in place of the window (EP-37).
    Restart,
    Quit,
}

impl Order {
    fn from_id(id: &str) -> Option<Order> {
        match id {
            "show" => Some(Order::Show),
            "pause" => Some(Order::Pause),
            "restart" => Some(Order::Restart),
            "quit" => Some(Order::Quit),
            _ => id
                .strip_prefix("start:")
                .map(|name| Order::Start(name.to_owned())),
        }
    }
}

/// What the tray shows, as the window gives it each pass.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    /// What waits for the owner (`Board::decisions_count`).
    pub waiting: usize,
    /// The badge events new since the window was last in front, counted in words; empty when
    /// none is.
    pub new: String,
    /// Each pinned project with what Start next opens: its next brief in words, or none.
    pub starts: Vec<(String, Option<String>)>,
    /// The owner paused every chain (D-22; EP-19).
    pub paused: bool,
    /// Why the chains hold meanwhile when usage holds them, in a few words.
    pub held: Option<String>,
    /// A newer build is built or installed, to restart into (EP-37).
    pub restart: bool,
    /// The icon's side in pixels.
    pub size: u32,
}

/// What the icon's pixels show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    pub waiting: usize,
    pub new: bool,
    pub size: u32,
}

/// One line of the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry {
    Item {
        id: String,
        text: String,
        enabled: bool,
    },
    Separator,
}

impl Entry {
    fn item(id: impl Into<String>, text: impl Into<String>, enabled: bool) -> Entry {
        Entry::Item {
            id: id.into(),
            text: text.into(),
            enabled,
        }
    }

    fn id(&self) -> Option<&str> {
        match self {
            Entry::Item { id, .. } => Some(id),
            Entry::Separator => None,
        }
    }
}

/// A menu as built: the menu, its items by entry (none for a separator), and the entries.
struct Built {
    menu: Menu,
    items: Vec<Option<MenuItem>>,
    entries: Vec<Entry>,
}

impl Built {
    fn new(entries: Vec<Entry>) -> Result<Built, String> {
        let menu = Menu::new();
        let mut items = Vec::new();
        for entry in &entries {
            match entry {
                Entry::Item { id, text, enabled } => {
                    let item = MenuItem::with_id(id.as_str(), mnemonic_free(text), *enabled, None);
                    menu.append(&item).map_err(|e| e.to_string())?;
                    items.push(Some(item));
                }
                Entry::Separator => {
                    menu.append(&PredefinedMenuItem::separator())
                        .map_err(|e| e.to_string())?;
                    items.push(None);
                }
            }
        }
        Ok(Built {
            menu,
            items,
            entries,
        })
    }
}

/// The tray icon while it runs.
pub struct Tray {
    icon: TrayIcon,
    built: Built,
    /// The menu replaced last, and when.
    retired: Option<(Built, Instant)>,
    orders: Receiver<Order>,
    look: Look,
    tooltip: String,
    /// The last change Windows refused, for the status bar's hover.
    pub problem: Option<String>,
}

impl Tray {
    /// Puts the icon in the notification area; its clicks and its menu reach the window through
    /// [`Tray::orders`], each waking it through `ctx`. Called once, on the main thread.
    pub fn start(ctx: &egui::Context) -> Result<Tray, String> {
        let (sender, orders) = mpsc::channel();
        let (clicks, wake) = (sender.clone(), ctx.clone());
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            let show = matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            );
            if show {
                let _ = clicks.send(Order::Show);
                wake.request_repaint();
            }
        }));
        let wake = ctx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if let Some(order) = Order::from_id(event.id.as_ref()) {
                let _ = sender.send(order);
                wake.request_repaint();
            }
        }));
        let state = State {
            waiting: 0,
            new: String::new(),
            starts: Vec::new(),
            paused: false,
            held: None,
            restart: false,
            size: SIZE,
        };
        let look = look(&state);
        let tooltip = tooltip(&state);
        let built = Built::new(entries(&state))?;
        let icon = TrayIconBuilder::new()
            .with_icon(icon(&look)?)
            .with_tooltip(&tooltip)
            .with_menu(Box::new(built.menu.clone()))
            .with_menu_on_left_click(false)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Tray {
            icon,
            built,
            retired: None,
            orders,
            look,
            tooltip,
            problem: None,
        })
    }

    /// What the tray was asked since the last look.
    pub fn orders(&self) -> Vec<Order> {
        self.orders.try_iter().collect()
    }

    /// Shows `state`: the icon drawn again, the tooltip and the menu changed, each only when it
    /// differs from what is shown.
    pub fn update(&mut self, state: &State) {
        let look = look(state);
        if look != self.look {
            match icon(&look)
                .and_then(|icon| self.icon.set_icon(Some(icon)).map_err(|e| e.to_string()))
            {
                Ok(()) => self.look = look,
                Err(why) => self.problem = Some(format!("the icon did not change: {why}")),
            }
        }
        let tooltip = tooltip(state);
        if tooltip != self.tooltip {
            match self.icon.set_tooltip(Some(&tooltip)) {
                Ok(()) => self.tooltip = tooltip,
                Err(e) => self.problem = Some(format!("the tooltip did not change: {e}")),
            }
        }
        let entries = entries(state);
        if entries != self.built.entries {
            self.change_menu(entries);
        }
        if self
            .retired
            .as_ref()
            .is_some_and(|(_, at)| at.elapsed() >= RETIRED_FOR)
        {
            self.retired = None;
        }
    }

    /// The menu's texts and switches set in place when its lines are the same ones; else a new
    /// menu, the old one kept a while.
    fn change_menu(&mut self, entries: Vec<Entry>) {
        let same_lines = entries.len() == self.built.entries.len()
            && entries
                .iter()
                .zip(&self.built.entries)
                .all(|(new, old)| new.id() == old.id());
        if same_lines {
            for ((new, old), item) in entries
                .iter()
                .zip(&self.built.entries)
                .zip(&self.built.items)
            {
                if let (
                    Entry::Item { text, enabled, .. },
                    Entry::Item {
                        text: was,
                        enabled: on,
                        ..
                    },
                    Some(item),
                ) = (new, old, item)
                {
                    if text != was {
                        item.set_text(mnemonic_free(text));
                    }
                    if enabled != on {
                        item.set_enabled(*enabled);
                    }
                }
            }
            self.built.entries = entries;
            return;
        }
        match Built::new(entries) {
            Ok(built) => {
                self.icon.set_menu(Some(Box::new(built.menu.clone())));
                let old = std::mem::replace(&mut self.built, built);
                self.retired = Some((old, Instant::now()));
            }
            Err(why) => self.problem = Some(format!("the menu did not change: {why}")),
        }
    }

    /// The tray in words, for the status bar's hover.
    pub fn line(&self) -> String {
        match &self.problem {
            Some(problem) => format!("the tray icon shows, but {problem}"),
            None => "the tray icon shows".to_owned(),
        }
    }
}

fn look(state: &State) -> Look {
    Look {
        waiting: state.waiting,
        new: !state.new.is_empty(),
        size: state.size.clamp(16, 64),
    }
}

fn icon(look: &Look) -> Result<Icon, String> {
    Icon::from_rgba(draw(look), look.size, look.size).map_err(|e| e.to_string())
}

/// `text` with its ampersands doubled, which a Windows menu would read as a mnemonic's mark.
fn mnemonic_free(text: &str) -> String {
    text.replace('&', "&&")
}

/// What waits for the owner, in words.
fn waiting_words(waiting: usize) -> String {
    match waiting {
        0 => "nothing waits for you".to_owned(),
        1 => "1 decision waits for you".to_owned(),
        n => format!("{n} decisions wait for you"),
    }
}

/// The tooltip: `Roadmap Wrangler · 2 decisions wait for you · new: 1 inbox message`, within
/// [`TOOLTIP`] characters.
fn tooltip(state: &State) -> String {
    let mut text = format!("Roadmap Wrangler · {}", waiting_words(state.waiting));
    if !state.new.is_empty() {
        text.push_str(&format!(" · new: {}", state.new));
    }
    if text.chars().count() > TOOLTIP {
        let cut: String = text.chars().take(TOOLTIP - 1).collect();
        text = format!("{}…", cut.trim_end());
    }
    text
}

/// The menu's lines for `state`.
fn entries(state: &State) -> Vec<Entry> {
    let mut waiting = waiting_words(state.waiting);
    if let Some(first) = waiting.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    let new = if state.new.is_empty() {
        "Nothing new since you looked".to_owned()
    } else {
        format!("New since you looked: {}", state.new)
    };
    let mut entries = vec![
        Entry::item("waiting", waiting, false),
        Entry::item("new", new, false),
        Entry::Separator,
        Entry::item("show", "Show Roadmap Wrangler", true),
        Entry::Separator,
    ];
    for (name, next) in &state.starts {
        entries.push(match next {
            Some(next) => Entry::item(
                format!("start:{name}"),
                format!("Start {name} {next}…"),
                true,
            ),
            None => Entry::item(
                format!("start:{name}"),
                format!("{name}: no brief to start"),
                false,
            ),
        });
    }
    if !state.starts.is_empty() {
        entries.push(Entry::Separator);
    }
    let pause = match (&state.paused, &state.held) {
        (true, _) => "Resume autopilot".to_owned(),
        (false, Some(held)) => format!("Pause autopilot (held now: {held})"),
        (false, None) => "Pause autopilot".to_owned(),
    };
    entries.extend([
        Entry::item("pause", pause, true),
        Entry::Separator,
        Entry::item("restart", "Install and restart", state.restart),
        Entry::item("quit", "Quit Roadmap Wrangler", true),
    ]);
    entries
}

/// The icon's pixels for `look`, RGBA, `look.size` square.
pub fn draw(look: &Look) -> Vec<u8> {
    let s = look.size as f32;
    let mut canvas = Canvas::new(look.size);
    canvas.rounded([0.06 * s, 0.06 * s, 0.94 * s, 0.94 * s], 0.22 * s, BLUE);
    let half = 0.055 * s;
    for (y, length) in [(0.32, 0.50), (0.50, 0.38), (0.68, 0.26)] {
        let rect = [0.24 * s, y * s - half, (0.24 + length) * s, y * s + half];
        canvas.rounded(rect, half, WHITE);
    }
    let ring = (s / 24.0).max(1.0);
    if look.new {
        let (x, y, r) = (0.80 * s, 0.20 * s, 0.17 * s);
        canvas.disc(x, y, r + ring, WHITE);
        canvas.disc(x, y, r, AMBER);
    }
    if look.waiting > 0 {
        let r = 0.30 * s;
        let centre = s - r - 0.02 * s;
        canvas.disc(centre, centre, r + ring, WHITE);
        canvas.disc(centre, centre, r, RED);
        let count = if look.waiting > 9 {
            "9+".to_owned()
        } else {
            look.waiting.to_string()
        };
        canvas.text(&count, centre, centre, 2.0 * (r - ring), WHITE);
    }
    canvas.pixels
}

/// Straight-alpha RGBA pixels, drawn with each pixel covered by how much of it a shape holds.
struct Canvas {
    size: u32,
    pixels: Vec<u8>,
}

impl Canvas {
    fn new(size: u32) -> Self {
        Self {
            size,
            pixels: vec![0; (size * size * 4) as usize],
        }
    }

    /// Lays `rgb` over the pixel at `x`, `y` by `cover`, from 0 to 1.
    fn blend(&mut self, x: u32, y: u32, rgb: [u8; 3], cover: f32) {
        let a = cover.clamp(0.0, 1.0);
        if a == 0.0 {
            return;
        }
        let at = ((y * self.size + x) * 4) as usize;
        let below = f32::from(self.pixels[at + 3]) / 255.0;
        let alpha = a + below * (1.0 - a);
        for (channel, value) in rgb.into_iter().enumerate() {
            let was = f32::from(self.pixels[at + channel]);
            let mixed = (f32::from(value) * a + was * below * (1.0 - a)) / alpha;
            self.pixels[at + channel] = mixed.round() as u8;
        }
        self.pixels[at + 3] = (alpha * 255.0).round() as u8;
    }

    /// Fills `rgb` by `cover`, which gives how much of the pixel centred at `x`, `y` lies inside.
    fn fill(&mut self, rgb: [u8; 3], cover: impl Fn(f32, f32) -> f32) {
        for y in 0..self.size {
            for x in 0..self.size {
                let inside = cover(x as f32 + 0.5, y as f32 + 0.5);
                self.blend(x, y, rgb, inside);
            }
        }
    }

    fn disc(&mut self, cx: f32, cy: f32, r: f32, rgb: [u8; 3]) {
        self.fill(rgb, |x, y| r - (x - cx).hypot(y - cy) + 0.5);
    }

    /// A rectangle `[left, top, right, bottom]` with its corners rounded by `radius`.
    fn rounded(&mut self, rect: [f32; 4], radius: f32, rgb: [u8; 3]) {
        let [left, top, right, bottom] = rect;
        let (cx, cy) = ((left + right) / 2.0, (top + bottom) / 2.0);
        let (hx, hy) = ((right - left) / 2.0 - radius, (bottom - top) / 2.0 - radius);
        self.fill(rgb, |x, y| {
            let (qx, qy) = ((x - cx).abs() - hx, (y - cy).abs() - hy);
            let outside = qx.max(0.0).hypot(qy.max(0.0));
            let inside = qx.max(qy).min(0.0);
            0.5 - (outside + inside - radius)
        });
    }

    /// Writes `text` in [`GLYPHS`] centred on `cx`, `cy`: as large as fits in `width`, by whole
    /// pixels, a twelfth of the icon at most.
    fn text(&mut self, text: &str, cx: f32, cy: f32, width: f32, rgb: [u8; 3]) {
        let glyphs: Vec<[u8; 5]> = text
            .chars()
            .filter_map(|c| GLYPHS.iter().find(|(glyph, _)| *glyph == c))
            .map(|(_, rows)| *rows)
            .collect();
        let count = glyphs.len() as u32;
        if count == 0 {
            return;
        }
        let span = |scale: u32| count * 4 * scale - scale;
        let mut scale = (self.size / 12).max(1);
        while scale > 1 && span(scale) as f32 > width {
            scale -= 1;
        }
        let left = (cx - span(scale) as f32 / 2.0).round() as i64;
        let top = (cy - (5 * scale) as f32 / 2.0).round() as i64;
        let size = i64::from(self.size);
        for (n, rows) in glyphs.iter().enumerate() {
            for (row, bits) in rows.iter().enumerate() {
                for column in 0..3 {
                    if bits & (0b100 >> column) == 0 {
                        continue;
                    }
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let x = left + i64::from((n as u32 * 4 + column) * scale + dx);
                            let y = top + i64::from(row as u32 * scale + dy);
                            if (0..size).contains(&x) && (0..size).contains(&y) {
                                self.blend(x as u32, y as u32, rgb, 1.0);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(pixels: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
        let at = ((y * size + x) * 4) as usize;
        [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
    }

    fn opaque(rgb: [u8; 3]) -> [u8; 4] {
        [rgb[0], rgb[1], rgb[2], 255]
    }

    fn state(waiting: usize, new: &str) -> State {
        State {
            waiting,
            new: new.to_owned(),
            starts: Vec::new(),
            paused: false,
            held: None,
            restart: false,
            size: 24,
        }
    }

    #[test]
    fn the_icon_is_a_tile_with_a_count_and_a_dot_when_asked() {
        let plain = draw(&Look {
            waiting: 0,
            new: false,
            size: 24,
        });
        assert_eq!(plain.len(), 24 * 24 * 4);
        // The corner lies outside the tile's rounding; above the rows, the tile.
        assert_eq!(pixel(&plain, 24, 0, 0)[3], 0);
        assert_eq!(pixel(&plain, 24, 12, 3), opaque(BLUE));
        assert_eq!(pixel(&plain, 24, 8, 7), opaque(WHITE));
        let has =
            |pixels: &[u8], rgb: [u8; 3]| pixels.chunks(4).any(|px| px == opaque(rgb).as_slice());
        assert!(!has(&plain, RED) && !has(&plain, AMBER));

        let marked = draw(&Look {
            waiting: 3,
            new: true,
            size: 24,
        });
        // The disc's centre is the 3's middle bar; beside the digit, the disc.
        assert_eq!(pixel(&marked, 24, 16, 16), opaque(WHITE));
        assert_eq!(pixel(&marked, 24, 11, 16), opaque(RED));
        assert_eq!(pixel(&marked, 24, 19, 4), opaque(AMBER));
        // Many waiting: 9+, its two glyphs inside the disc.
        let many = draw(&Look {
            waiting: 14,
            new: false,
            size: 16,
        });
        assert!(has(&many, RED));
        assert_eq!(many.len(), 16 * 16 * 4);
    }

    #[test]
    fn the_tooltip_says_what_waits_and_stays_under_the_limit() {
        assert_eq!(
            tooltip(&state(0, "")),
            "Roadmap Wrangler · nothing waits for you"
        );
        assert_eq!(
            tooltip(&state(2, "1 inbox message")),
            "Roadmap Wrangler · 2 decisions wait for you · new: 1 inbox message"
        );
        let long = tooltip(&state(1, &"7 unadopted folders, ".repeat(10)));
        assert_eq!(long.chars().count(), TOOLTIP);
        assert!(long.ends_with('…'));
        // Under Windows' 128 UTF-16 units, its end included.
        assert!(long.encode_utf16().count() < 128);
    }

    #[test]
    fn the_menu_lists_the_pinned_projects_and_disables_what_cannot_run() {
        let mut with = state(1, "");
        with.starts = vec![
            (
                "harbor".to_owned(),
                Some("EP-4 (M) The sessions tile".to_owned()),
            ),
            ("R&D".to_owned(), None),
        ];
        let lines = entries(&with);
        let items: Vec<(&str, &str, bool)> = lines
            .iter()
            .filter_map(|entry| match entry {
                Entry::Item { id, text, enabled } => Some((id.as_str(), text.as_str(), *enabled)),
                Entry::Separator => None,
            })
            .collect();
        assert_eq!(
            items,
            [
                ("waiting", "1 decision waits for you", false),
                ("new", "Nothing new since you looked", false),
                ("show", "Show Roadmap Wrangler", true),
                (
                    "start:harbor",
                    "Start harbor EP-4 (M) The sessions tile…",
                    true
                ),
                ("start:R&D", "R&D: no brief to start", false),
                ("pause", "Pause autopilot", true),
                ("restart", "Install and restart", false),
                ("quit", "Quit Roadmap Wrangler", true),
            ]
        );
        // A newer build to restart into enables the restart (EP-37).
        with.restart = true;
        assert!(entries(&with).contains(&Entry::item("restart", "Install and restart", true)));
        // The item says the owner's Pause, and usage's hold meanwhile.
        let pause_text = |paused: bool, held: Option<&str>| {
            let mut state = state(0, "");
            state.paused = paused;
            state.held = held.map(str::to_owned);
            entries(&state).into_iter().find_map(|entry| match entry {
                Entry::Item { id, text, .. } if id == "pause" => Some(text),
                _ => None,
            })
        };
        assert_eq!(
            pause_text(true, Some("x")).as_deref(),
            Some("Resume autopilot")
        );
        assert_eq!(
            pause_text(false, Some("Fable at 91 %")).as_deref(),
            Some("Pause autopilot (held now: Fable at 91 %)")
        );
        assert_eq!(lines.iter().filter(|e| **e == Entry::Separator).count(), 4);
        // Without pinned projects, no empty section.
        assert_eq!(
            entries(&state(0, "x"))
                .iter()
                .filter(|e| **e == Entry::Separator)
                .count(),
            3
        );
        assert_eq!(mnemonic_free("R&D"), "R&&D");
        assert_eq!(
            Order::from_id("start:R&D"),
            Some(Order::Start("R&D".to_owned()))
        );
        assert_eq!(Order::from_id("show"), Some(Order::Show));
        assert_eq!(Order::from_id("pause"), Some(Order::Pause));
        assert_eq!(Order::from_id("restart"), Some(Order::Restart));
    }
}
