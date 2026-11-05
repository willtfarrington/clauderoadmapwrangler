//! The usage meter in the window (D-23; EP-17). A worker thread of its own polls the endpoint
//! (`crw_core::usage`) on the meter's schedule: the token read at each poll, a reading written to
//! the ledger as a sample and as the cache the next start shows first, a failure shown and never
//! stored. The status lines the watch hears (EP-12) are its second series, a sample each time a
//! session's figures change. The frame loop takes in the polls, logs the meter's changes of state
//! (degraded, back) as `usage` events, and hands the 80 % and 95 % marks to the notifications,
//! once per mark and limit window. It draws the strip in the status bar and the Usage pane: each
//! limit's bar with its reset countdown, the week's use by surface, extra usage and a sparkline of
//! the last seven days. Nothing polls before `main` starts it: never in a test or a screenshot
//! run, whose meter is canned.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Local, TimeDelta, Utc};
use crw_core::badges::short_age;
use crw_core::ledger::{Ledger, Sample};
use crw_core::registry::Store;
use crw_core::signals::StatusLine;
use crw_core::usage::{
    self, Alerts, Crossing, Extra, Fetched, Figure, Figures, Gauge, Line, Problem, Reading,
    Schedule, Shift, Source,
};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, vec2};

use crate::board::Board;
use crate::style;

/// The days the sparkline spans, and the samples the window keeps.
const DAYS: i64 = 7;

/// A gap between two samples longer than this breaks the sparkline's line.
const GAP: TimeDelta = TimeDelta::minutes(30);

/// The pane's countdowns are drawn again this often.
const TICK: Duration = Duration::from_secs(30);

/// One series of the sparkline: a sample's figure.
type Pick = fn(&Sample) -> Option<f64>;

/// What the frame loop tells the worker.
enum Order {
    /// A session the tool launched has ended: the next poll comes soon (D-23).
    SessionEnded,
}

/// One poll, as the worker hands it over.
struct Polled {
    at: DateTime<Utc>,
    outcome: Result<Reading, Problem>,
    next: DateTime<Utc>,
    /// Whether the ledger took the reading.
    stored: Result<(), String>,
}

struct Worker {
    orders: Sender<Order>,
    polls: Receiver<Polled>,
}

#[derive(Default)]
pub struct Meter {
    pub gauge: Gauge,
    /// The samples of the last [`DAYS`] days, the oldest first.
    pub samples: Vec<Sample>,
    /// When the worker polls next.
    pub next: Option<DateTime<Utc>>,
    /// A ledger write or read that failed, until the window starts again.
    pub problem: Option<String>,
    /// The marks reached since the notifications last took them.
    crossings: Vec<Crossing>,
    /// This run's latest reading of the endpoint, never the cache a start shows: the split
    /// counts only what the window watched (EP-18a).
    previous: Option<Sample>,
    /// The readings since the board last took them, each with this run's one before.
    readings: Vec<(Option<Sample>, Sample)>,
    ledger: Option<PathBuf>,
    worker: Option<Worker>,
    /// Started by `main`, or canned in a screenshot run.
    on: bool,
}

impl Meter {
    /// Shows the ledger's cached reading and the week's samples, then starts the polls of the
    /// endpoint with the token in `credentials` (D-23).
    pub fn start(
        &mut self,
        credentials: Option<PathBuf>,
        ledger: Option<PathBuf>,
        ctx: &egui::Context,
    ) {
        self.on = true;
        self.ledger = ledger.clone();
        self.restore(Utc::now());
        let (orders, inbox) = mpsc::channel();
        let (outbox, polls) = mpsc::channel();
        let wake = ctx.clone();
        let spawned = thread::Builder::new()
            .name("usage meter".to_owned())
            .spawn(move || run(credentials, ledger, &inbox, &outbox, &wake));
        match spawned {
            Ok(_) => self.worker = Some(Worker { orders, polls }),
            Err(e) => self.problem = Some(format!("The meter's thread did not start: {e}")),
        }
    }

    /// The cache first (EP-7's pattern): the endpoint's last reading, the marks notified, the
    /// week's samples.
    fn restore(&mut self, now: DateTime<Utc>) {
        let Some(path) = &self.ledger else {
            return;
        };
        let read = Ledger::open(path).and_then(|ledger| {
            Ok((
                ledger.cached_one("", usage::CACHE_SOURCE)?,
                ledger.cached_one("", usage::ALERTS_SOURCE)?,
                ledger.samples_since(now - TimeDelta::days(DAYS))?,
            ))
        });
        match read {
            Ok((reading, alerts, samples)) => {
                if let Some(row) = reading
                    && let Some(reading) = Reading::from_json(&row.data)
                {
                    self.gauge.fetched = Some(Fetched {
                        at: row.fetched_at,
                        reading,
                    });
                }
                if let Some(row) = alerts
                    && let Some(alerts) = Alerts::from_json(&row.data)
                {
                    self.gauge.alerts = alerts;
                }
                self.samples = samples;
            }
            Err(e) => self.problem = Some(format!("The ledger's usage did not read: {e}")),
        }
    }

    /// Takes in the polls since the last frame.
    pub fn receive(&mut self, now: DateTime<Utc>) {
        let polls: Vec<Polled> = self
            .worker
            .as_ref()
            .map(|worker| worker.polls.try_iter().collect())
            .unwrap_or_default();
        if polls.is_empty() {
            return;
        }
        for polled in polls {
            self.next = Some(polled.next);
            if let Ok(reading) = &polled.outcome {
                let sample = usage::endpoint_sample(reading, polled.at);
                let before = self.previous.replace(sample.clone());
                self.readings.push((before, sample.clone()));
                self.samples.push(sample);
            }
            if let Err(e) = polled.stored {
                self.problem = Some(format!("The ledger did not keep a usage reading: {e}"));
            }
            if let Some(shift) = self.gauge.polled(polled.at, polled.outcome) {
                self.log(&shift);
            }
        }
        let since = now - TimeDelta::days(DAYS);
        self.samples.retain(|sample| sample.taken_at >= since);
        self.check(now);
    }

    /// Takes in a session's status line: a sample when its figures changed (D-23).
    pub fn status_line(&mut self, status: &StatusLine, now: DateTime<Utc>) {
        if !self.on || status.session_id.is_empty() {
            return;
        }
        let Some(line) = Line::of(status, now) else {
            return;
        };
        let sample = usage::line_sample(&status.session_id, &line);
        if !self.gauge.line(&status.session_id, line) {
            return;
        }
        if let Some(path) = &self.ledger
            && let Err(e) = Ledger::open(path).and_then(|ledger| ledger.add_sample(&sample))
        {
            self.problem = Some(format!(
                "The ledger did not keep a status line's figures: {e}"
            ));
        }
        self.samples.push(sample);
        self.check(now);
    }

    /// A session the tool launched has ended: the worker polls soon.
    pub fn session_ended(&self) {
        if let Some(worker) = &self.worker {
            let _ = worker.orders.send(Order::SessionEnded);
        }
    }

    /// The marks reached since the last call, for the notifications.
    pub fn take_crossings(&mut self) -> Vec<Crossing> {
        std::mem::take(&mut self.crossings)
    }

    /// The endpoint's readings since the last call, each with this run's reading before it (none
    /// for the run's first), for the usage's split (D-23; EP-18a).
    pub fn take_readings(&mut self) -> Vec<(Option<Sample>, Sample)> {
        std::mem::take(&mut self.readings)
    }

    /// The marks the standing figures reach, kept in the ledger's cache so that a restart does
    /// not notify them again.
    fn check(&mut self, now: DateTime<Utc>) {
        let crossings = self.gauge.crossings(now);
        if crossings.is_empty() {
            return;
        }
        self.crossings.extend(crossings);
        if let Some(path) = &self.ledger
            && let Err(e) = Ledger::open(path).and_then(|ledger| {
                ledger.cache("", usage::ALERTS_SOURCE, now, &self.gauge.alerts.to_json())
            })
        {
            self.problem = Some(format!(
                "The ledger did not keep the usage notifications: {e}"
            ));
        }
    }

    /// A change of state, as a line of the ledger's event log.
    fn log(&mut self, shift: &Shift) {
        let detail = match shift {
            Shift::Degraded(why) => format!("degraded · {why}"),
            Shift::Recovered => "back · the endpoint answers again".to_owned(),
        };
        if let Some(path) = &self.ledger
            && let Err(e) = Ledger::open(path)
                .and_then(|ledger| ledger.add_event(Utc::now(), usage::EVENT_KIND, None, &detail))
        {
            self.problem = Some(format!("The ledger did not log the meter's state: {e}"));
        }
    }

    /// The meter's state in a sentence, and whether it warns.
    fn state(&self, now: DateTime<Utc>) -> (String, bool) {
        let next = match self.next {
            Some(at) if at > now + TimeDelta::minutes(1) => {
                format!(" The next poll is in {}.", usage::countdown(at, now))
            }
            Some(_) => " The next poll is due.".to_owned(),
            None if self.worker.is_some() => " The first poll is due.".to_owned(),
            None => String::new(),
        };
        match (&self.gauge.problem, &self.gauge.fetched) {
            (Some((problem, at)), fetched) => {
                let last = fetched.as_ref().map_or_else(
                    || "No reading of the endpoint yet".to_owned(),
                    |fetched| format!("Its last reading is from {}", ago(now - fetched.at)),
                );
                (
                    format!(
                        "Degraded {}: {}.\n{last}; newer figures come from the status \
                         lines.{next}",
                        ago(now - *at),
                        problem.words()
                    ),
                    true,
                )
            }
            (None, Some(fetched)) => (
                format!("Read from the endpoint {}.{next}", ago(now - fetched.at)),
                false,
            ),
            (None, None) => (format!("Not read yet.{next}"), false),
        }
    }

    /// The strip in the status bar (D-23): weekly, Fable and the five-hour window while one is
    /// open, each a small bar with its percent, and a degraded marker. The status bar's right
    /// side runs right to left, so the parts are added from the last.
    pub fn strip(&self, ui: &mut egui::Ui, now: DateTime<Utc>) {
        if !self.on {
            return;
        }
        ui.separator();
        let figures = self.gauge.figures(now);
        let parts: Vec<(&str, &Figure)> = [
            ("5 h", figures.five_hour.as_ref()),
            ("Fable", figures.fable.as_ref()),
            ("Week", figures.weekly.as_ref()),
        ]
        .into_iter()
        .filter_map(|(name, figure)| Some((name, figure?)))
        .collect();
        if parts.is_empty() {
            let (state, _) = self.state(now);
            ui.label(RichText::new("Usage: not read yet").weak())
                .on_hover_text(state);
        }
        for (name, figure) in parts {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(RichText::new(usage::percent(figure.limit.percent)).small());
                bar(ui, figure.limit.percent, vec2(40.0, 7.0));
                ui.label(RichText::new(name).small().weak());
            })
            .response
            .on_hover_text(hover(name, figure, &figures, now));
        }
        if let Some(why) = &figures.degraded {
            style::chip(ui, "degraded", style::amber(ui)).on_hover_text(format!(
                "The usage endpoint fails: {why}. The strip shows the newest of the status \
                 lines' figures and its last reading."
            ));
        }
    }

    /// The Usage pane (D-23, D-25).
    pub fn tile(&self, ui: &mut egui::Ui, now: DateTime<Utc>) {
        if !self.on {
            ui.add_space(24.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("The meter is off: a screenshot run, or not started.").weak(),
                )
            });
            return;
        }
        ui.ctx().request_repaint_after(TICK);
        let figures = self.gauge.figures(now);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let (state, warns) = self.state(now);
                let state = RichText::new(state);
                ui.label(if warns {
                    state.color(style::amber(ui))
                } else {
                    state.weak()
                });
                if let Some(problem) = &self.problem {
                    ui.label(RichText::new(problem).color(style::amber(ui)));
                }
                ui.add_space(6.0);
                for (name, figure) in limits(&figures, self.gauge.fetched.as_ref()) {
                    limit_block(ui, &name, &figure, now);
                    ui.add_space(4.0);
                }
                if let Some(fetched) = &self.gauge.fetched {
                    surfaces(ui, fetched, now);
                    if let Some(extra) = &fetched.reading.extra {
                        ui.add_space(4.0);
                        ui.label(RichText::new(extra_words(extra)).weak());
                    }
                }
                ui.add_space(8.0);
                ui.label(RichText::new("The last seven days").strong());
                sparkline(ui, &self.samples, now);
            });
    }
}

impl Board {
    pub fn usage_pane(&mut self, ui: &mut egui::Ui) {
        self.meter.tile(ui, Utc::now());
    }
}

/// The worker's loop: a poll when the schedule says, the registry's switch read first; a session's
/// end brings the next forward. It ends when the window lets go of its channel.
fn run(
    credentials: Option<PathBuf>,
    ledger: Option<PathBuf>,
    orders: &Receiver<Order>,
    polls: &Sender<Polled>,
    ctx: &egui::Context,
) {
    let mut schedule = Schedule::new(Utc::now());
    loop {
        let wait = (schedule.due() - Utc::now())
            .to_std()
            .unwrap_or(Duration::ZERO);
        match orders.recv_timeout(wait) {
            Ok(Order::SessionEnded) => {
                schedule.session_ended(Utc::now());
                continue;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        let at = Utc::now();
        let on = Store::locate()
            .and_then(|store| store.load().ok())
            .is_none_or(|loaded| loaded.registry.settings.usage_endpoint_on());
        let outcome = match (&credentials, on) {
            (_, false) => Err(Problem::Off),
            (None, true) => Err(Problem::NoCredentials),
            (Some(path), true) => usage::poll(path, usage::ENDPOINT, at),
        };
        schedule.polled(at, outcome.as_ref().err());
        let stored = match (&outcome, &ledger) {
            (Ok(reading), Some(path)) => keep(path, reading, at),
            _ => Ok(()),
        };
        let polled = Polled {
            at,
            outcome,
            next: schedule.due(),
            stored,
        };
        if polls.send(polled).is_err() {
            return;
        }
        ctx.request_repaint();
    }
}

/// A reading kept: its sample, and the cache the next start shows first.
fn keep(path: &Path, reading: &Reading, at: DateTime<Utc>) -> Result<(), String> {
    let ledger = Ledger::open(path).map_err(|e| e.to_string())?;
    ledger
        .add_sample(&usage::endpoint_sample(reading, at))
        .map_err(|e| e.to_string())?;
    ledger
        .cache("", usage::CACHE_SOURCE, at, &reading.to_json())
        .map_err(|e| e.to_string())
}

/// `age` as words: `just now`, `3 min ago`.
fn ago(age: TimeDelta) -> String {
    match short_age(age).as_str() {
        "now" => "just now".to_owned(),
        age => format!("{age} ago"),
    }
}

/// A limit's reset in words: `resets in 3 d 7 h, Sat 14:00`.
fn reset_words(figure: &Figure, now: DateTime<Utc>) -> String {
    figure.limit.resets_at.map_or_else(
        || "no reset time given".to_owned(),
        |at| {
            format!(
                "resets in {}, {}",
                usage::countdown(at, now),
                at.with_timezone(&Local).format("%a %H:%M")
            )
        },
    )
}

fn source_words(figure: &Figure, now: DateTime<Utc>) -> String {
    let from = match figure.source {
        Source::Endpoint => "the usage endpoint",
        Source::StatusLine => "a session's status line",
    };
    format!("from {from}, {}", ago(now - figure.at))
}

/// A strip part's hover.
fn hover(name: &str, figure: &Figure, figures: &Figures, now: DateTime<Utc>) -> String {
    let name = match name {
        "Week" => "The weekly limit, all models",
        "Fable" => "The Fable-scoped weekly limit",
        _ => "The five-hour window",
    };
    let mut text = format!(
        "{name}: {} used, {}\nRead {}",
        usage::percent(figure.limit.percent),
        reset_words(figure, now),
        source_words(figure, now)
    );
    if let Some(why) = &figures.degraded {
        text.push_str(&format!("\nDegraded: {why}"));
    }
    text
}

/// The pane's limits: weekly, Fable, the other scoped limits of the endpoint's reading and those
/// of a kind it does not know, and the five-hour window while one is open.
fn limits(figures: &Figures, fetched: Option<&Fetched>) -> Vec<(String, Figure)> {
    let mut limits = Vec::new();
    if let Some(weekly) = &figures.weekly {
        limits.push(("Weekly, all models".to_owned(), weekly.clone()));
    }
    if let Some(fable) = &figures.fable {
        limits.push(("Fable".to_owned(), fable.clone()));
    }
    if let Some(fetched) = fetched {
        let scoped = fetched
            .reading
            .scoped
            .iter()
            .filter(|scoped| !scoped.name.to_lowercase().contains("fable"))
            .map(|scoped| (scoped.name.clone(), &scoped.limit));
        // A kind the parser does not know, by its name: a limit the endpoint adds or brings back.
        let other = fetched
            .reading
            .other
            .iter()
            .map(|other| (other.name.replace('_', " "), &other.limit));
        for (name, limit) in scoped.chain(other) {
            limits.push((
                name,
                Figure {
                    limit: limit.clone(),
                    source: Source::Endpoint,
                    at: fetched.at,
                },
            ));
        }
    }
    if let Some(five_hour) = &figures.five_hour {
        limits.push(("Five-hour window".to_owned(), five_hour.clone()));
    }
    limits
}

/// One limit of the pane: its name and percent, a bar the pane's width, its reset and source.
fn limit_block(ui: &mut egui::Ui, name: &str, figure: &Figure, now: DateTime<Utc>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(name).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(usage::percent(figure.limit.percent)).strong());
        });
    });
    let width = ui.available_width().max(40.0);
    bar(ui, figure.limit.percent, vec2(width, 10.0));
    ui.label(
        RichText::new(format!(
            "{}; {}",
            reset_words(figure, now),
            source_words(figure, now)
        ))
        .small()
        .weak(),
    );
}

/// The week's use by surface, from the endpoint's reading.
fn surfaces(ui: &mut egui::Ui, fetched: &Fetched, now: DateTime<Utc>) {
    let surfaces = &fetched.reading.surfaces;
    if surfaces.is_empty() {
        return;
    }
    ui.add_space(6.0);
    ui.label(RichText::new("This week by surface").strong())
        .on_hover_text(format!(
            "The endpoint's breakdown, read {}",
            ago(now - fetched.at)
        ));
    let name_width = surfaces
        .iter()
        .map(|surface| {
            style::width_of(ui, |ui| {
                ui.label(&surface.name);
            })
        })
        .fold(0.0, f32::max);
    for surface in surfaces {
        ui.horizontal(|ui| {
            style::padded(ui, name_width, |ui| {
                ui.label(&surface.name);
            });
            let width = (ui.available_width() - 56.0).max(30.0);
            bar(ui, surface.percent, vec2(width, 7.0));
            ui.label(usage::percent(surface.percent));
        });
    }
}

fn extra_words(extra: &Extra) -> String {
    match (extra.enabled, extra.percent) {
        (Some(false), _) => "Extra usage: off".to_owned(),
        (_, Some(percent)) => format!(
            "Extra usage: on, {} of the monthly limit",
            usage::percent(percent)
        ),
        (Some(true), None) => "Extra usage: on".to_owned(),
        (None, None) => "Extra usage: not given".to_owned(),
    }
}

/// The colour of a share: the window's green, amber from 80 %, red from 95 %.
fn colour(ui: &egui::Ui, percent: f64) -> Color32 {
    if percent >= usage::ALARM {
        style::red(ui)
    } else if percent >= usage::WARN {
        style::amber(ui)
    } else {
        style::green(ui)
    }
}

/// A bar of `size` filled to `percent`.
fn bar(ui: &mut egui::Ui, percent: f64, size: egui::Vec2) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let radius = size.y / 2.0;
    let painter = ui.painter();
    painter.rect_filled(rect, radius, ui.visuals().extreme_bg_color);
    let share = (percent / 100.0).clamp(0.0, 1.0) as f32;
    if share > 0.0 {
        let width = (rect.width() * share).max(size.y.min(rect.width()));
        let fill = Rect::from_min_size(rect.min, vec2(width, rect.height()));
        painter.rect_filled(fill, radius, colour(ui, percent));
    }
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
        StrokeKind::Inside,
    );
    response
}

/// The last seven days of samples: the endpoint's weekly line in the text's colour and its
/// Fable line in the notes' violet, the status lines' weekly figures as points, faint lines at
/// 80 % and 95 % and at each local midnight; a line breaks where the samples have a gap.
fn sparkline(ui: &mut egui::Ui, samples: &[Sample], now: DateTime<Utc>) {
    let width = ui.available_width().max(60.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 72.0), Sense::hover());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    painter.rect_filled(rect, 3.0, visuals.extreme_bg_color);
    let start = now - TimeDelta::days(DAYS);
    let span = (now - start).num_seconds() as f32;
    let at = |taken: DateTime<Utc>, percent: f64| {
        let x = rect.left() + rect.width() * ((taken - start).num_seconds() as f32 / span);
        let y = rect.bottom() - rect.height() * (percent.clamp(0.0, 100.0) as f32 / 100.0);
        Pos2::new(x, y)
    };
    for (mark, ink) in [
        (usage::WARN, style::amber(ui)),
        (usage::ALARM, style::red(ui)),
    ] {
        let y = at(now, mark).y;
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            Stroke::new(1.0, ink.gamma_multiply(0.35)),
        );
    }
    // A faint line at each local midnight, so that the days can be counted.
    let day_line = Stroke::new(1.0, visuals.weak_text_color().gamma_multiply(0.3));
    let mut day = Some(now.with_timezone(&Local).date_naive());
    for _ in 0..=DAYS {
        let Some(date) = day else {
            break;
        };
        if let Some(midnight) = date
            .and_hms_opt(0, 0, 0)
            .and_then(|time| time.and_local_timezone(Local).earliest())
            .map(|midnight| midnight.with_timezone(&Utc))
            .filter(|midnight| *midnight > start)
        {
            let x = at(midnight, 0.0).x;
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                day_line,
            );
        }
        day = date.pred_opt();
    }
    let text = visuals.text_color();
    let violet = style::reference(ui);
    let endpoint: Vec<&Sample> = samples
        .iter()
        .filter(|sample| sample.source == Source::Endpoint.as_str() && sample.taken_at >= start)
        .collect();
    let series: [(Pick, Color32); 2] = [
        (|sample| sample.weekly_percent, text),
        (|sample| sample.fable_percent, violet),
    ];
    for (pick, ink) in series {
        let mut run: Vec<Pos2> = Vec::new();
        let mut last: Option<DateTime<Utc>> = None;
        for sample in &endpoint {
            let Some(percent) = pick(sample) else {
                continue;
            };
            if last.is_some_and(|last| sample.taken_at - last > GAP) {
                draw_run(&painter, std::mem::take(&mut run), ink);
            }
            run.push(at(sample.taken_at, percent));
            last = Some(sample.taken_at);
        }
        draw_run(&painter, run, ink);
    }
    let weak = visuals.weak_text_color();
    let mut lines = 0;
    for sample in samples
        .iter()
        .filter(|sample| sample.source == Source::StatusLine.as_str() && sample.taken_at >= start)
    {
        if let Some(percent) = sample.weekly_percent {
            painter.circle_filled(at(sample.taken_at, percent), 1.6, weak);
            lines += 1;
        }
    }
    response.on_hover_text(format!(
        "{} readings of the endpoint and {lines} of the status lines since {}",
        endpoint.len(),
        start.with_timezone(&Local).format("%a %d %b %H:%M")
    ));
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("━ weekly").small().color(text));
        ui.label(RichText::new("━ Fable").small().color(violet));
        ui.label(RichText::new("• status lines").small().color(weak));
        ui.label(
            RichText::new("| midnights; now at the right")
                .small()
                .weak(),
        );
    });
}

fn draw_run(painter: &egui::Painter, run: Vec<Pos2>, ink: Color32) {
    match run.len() {
        0 => {}
        1 => {
            painter.circle_filled(run[0], 1.6, ink);
        }
        _ => {
            painter.add(egui::Shape::line(run, Stroke::new(1.5, ink)));
        }
    }
}

/// A screenshot run's meter (rule 5): an endpoint's reading two minutes old with Fable past its
/// 80 % mark, a status line, and a week of hourly samples across the weekly reset; `degraded`
/// adds the endpoint's failure a minute ago and a newer status line.
#[cfg(feature = "screenshot")]
pub fn canned(now: DateTime<Utc>, degraded: bool) -> Meter {
    use crw_core::usage::{Limit, Scoped, Surface};

    let resets = now + TimeDelta::hours(3 * 24 + 7);
    let limit = |percent, resets_at| Limit {
        percent,
        resets_at: Some(resets_at),
        active: Some(true),
        severity: None,
    };
    let five_hour_resets = now + TimeDelta::minutes(130);
    let reading = Reading {
        weekly: Some(limit(41.0, resets)),
        scoped: vec![Scoped {
            name: "Fable".to_owned(),
            limit: limit(82.0, resets),
        }],
        five_hour: Some(limit(23.0, five_hour_resets)),
        surfaces: [
            ("Claude Code", 35.0),
            ("Chats", 4.0),
            ("Cowork", 1.5),
            ("Other", 0.5),
        ]
        .into_iter()
        .map(|(name, percent)| Surface {
            name: name.to_owned(),
            percent,
        })
        .collect(),
        extra: Some(Extra {
            enabled: Some(false),
            ..Extra::default()
        }),
        ..Reading::default()
    };
    let mut gauge = Gauge::default();
    gauge.polled(now - TimeDelta::minutes(2), Ok(reading));
    // The week began 89 hours ago; the one before ended at 72 %.
    let began = resets - TimeDelta::days(7);
    let mut samples: Vec<Sample> = (1..=7 * 24)
        .rev()
        .map(|hours| {
            let taken = now - TimeDelta::hours(hours);
            let weekly = if taken < began {
                72.0 * (1.0 - (began - taken).num_minutes() as f64 / (7.0 * 1440.0))
            } else {
                41.0 * (taken - began).num_minutes() as f64 / (now - began).num_minutes() as f64
            };
            Sample {
                taken_at: taken,
                source: Source::Endpoint.as_str().to_owned(),
                session_id: None,
                weekly_percent: Some(weekly),
                weekly_resets_at: None,
                fable_percent: Some((weekly * 2.0).min(99.0)),
                fable_resets_at: None,
                five_hour_percent: None,
                five_hour_resets_at: None,
            }
        })
        // A night the window was closed.
        .filter(|sample| {
            !(now - sample.taken_at > TimeDelta::hours(50)
                && now - sample.taken_at < TimeDelta::hours(58))
        })
        .collect();
    for minutes in [300, 240, 180, 120, 60, 1] {
        let line = Line {
            at: now - TimeDelta::minutes(minutes),
            weekly: Some(limit(41.0 - minutes as f64 / 120.0, resets)),
            five_hour: Some(limit(23.0, five_hour_resets)),
        };
        samples.push(usage::line_sample("canned-session", &line));
        gauge.line("canned-session", line);
    }
    if degraded {
        gauge.polled(
            now - TimeDelta::minutes(1),
            Err(Problem::Network(
                "WinHTTP could not receive the answer: The operation timed out".to_owned(),
            )),
        );
        gauge.line(
            "canned-session",
            Line {
                at: now - TimeDelta::seconds(20),
                weekly: Some(limit(43.0, resets)),
                five_hour: Some(limit(26.0, five_hour_resets)),
            },
        );
    }
    samples.sort_by_key(|sample| sample.taken_at);
    Meter {
        gauge,
        samples,
        next: Some(now + TimeDelta::minutes(if degraded { 1 } else { 3 })),
        on: true,
        ..Meter::default()
    }
}

#[cfg(test)]
mod tests {
    use crw_core::signals::{RateLimits, Window};

    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    /// A status line with its weekly figure, resetting 2026-11-07T19:00:00Z.
    fn status(weekly: f64) -> StatusLine {
        StatusLine {
            session_id: "s-1".to_owned(),
            rate_limits: Some(RateLimits {
                five_hour: None,
                seven_day: Some(Window {
                    used_percentage: Some(weekly),
                    resets_at: Some(1_794_078_000.0),
                }),
            }),
            ..StatusLine::default()
        }
    }

    #[test]
    fn a_meter_that_is_off_takes_no_status_line_and_keeps_no_sample() {
        let mut meter = Meter::default();
        meter.status_line(&status(91.0), at("2026-11-04T12:00:00Z"));
        assert!(meter.samples.is_empty());
        assert!(meter.gauge.lines.is_empty());
    }

    #[test]
    fn a_status_line_past_a_mark_is_kept_and_notified_once() {
        let mut meter = Meter {
            on: true,
            ..Meter::default()
        };
        let now = at("2026-11-04T12:00:00Z");
        meter.status_line(&status(91.0), now);
        meter.status_line(&status(91.0), now + TimeDelta::seconds(30));
        assert_eq!(meter.samples.len(), 1, "the same figures make one sample");
        let crossings = meter.take_crossings();
        assert_eq!(crossings.len(), 1);
        assert_eq!(crossings[0].mark, usage::WARN);
        assert!(meter.take_crossings().is_empty());
    }

    #[test]
    fn the_state_says_how_the_last_poll_went() {
        let now = at("2026-11-04T12:00:00Z");
        let mut meter = Meter {
            on: true,
            next: Some(now + TimeDelta::minutes(4)),
            ..Meter::default()
        };
        assert_eq!(
            meter.state(now),
            ("Not read yet. The next poll is in 4 min.".to_owned(), false)
        );
        meter.gauge.polled(now, Ok(Reading::default()));
        assert_eq!(
            meter.state(now).0,
            "Read from the endpoint just now. The next poll is in 4 min."
        );
        meter
            .gauge
            .polled(now, Err(Problem::Network("timed out".to_owned())));
        let (state, warns) = meter.state(now + TimeDelta::minutes(3));
        assert!(warns);
        assert!(
            state.starts_with("Degraded 3 min ago: the endpoint did not answer: timed out."),
            "{state}"
        );
        assert!(
            state.contains(
                ".\nIts last reading is from 3 min ago; newer figures come from the status lines."
            ),
            "{state}"
        );
    }
}
