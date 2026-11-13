//! Analytics, the graphs (D-26; EP-26): what the ledger and the history give the Detail pane's
//! Analytics section. The days of the last weeks (sessions and completions per local day, for a
//! project or the portfolio) drawn as a calendar heatmap with the painter; the burn-up (done from
//! the completions' cumulative count, total from the daily snapshots) and the velocity (EPs per
//! week over the last eight weeks) drawn with `egui_plot`; the measure of a brief's time from the
//! session rows, the median per size, and the projection (the remaining session-hours from the
//! open rows' size mix, the finish at the measured pace as a range). The ledger is read on the UI
//! thread as the meter's week is, when the selection changes, a minute later, or after a read.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, Instant};

use chrono::{DateTime, Datelike, Days, Local, NaiveDate, TimeDelta, Utc};
use crw_core::history::Completion;
use crw_core::ledger::{Ledger, SessionRow, SnapshotRow};
use crw_core::revert;
use crw_core::roadmap::table::{Done, Roadmap};
use crw_core::roadmap::{EpId, Size};
use eframe::egui::{self, Align2, FontId, Pos2, Rect, RichText, Sense, Stroke, vec2};
use egui_plot::{Bar, BarChart, Corner, Legend, Line, Plot, PlotPoints};

use crate::portfolio::{Portfolio, ProjectRow};
use crate::style;

/// The weeks the heatmap spans, the current week last.
pub const WEEKS: usize = 16;
/// The weeks the velocity counts and the pace is measured over.
pub const PACE_WEEKS: usize = 8;
/// A visual or manual row longer than this is a tab left open, not a brief's time.
pub const CEILING: TimeDelta = TimeDelta::hours(3);
/// The rows of a size a project needs before its own median counts.
pub const LEAST_ROWS: usize = 3;
/// How long a read stands before the next frame reads the ledger again.
const FRESH_FOR: Duration = Duration::from_secs(60);
/// The ledger's name for the chat's turns, which are no project's sessions.
const CHAT: &str = crw_core::chat::PROJECT;

/// The plan's minutes per size (D-4), the medians' last fallback.
pub fn planned(size: Size) -> f64 {
    match size {
        Size::S => 30.0,
        Size::M => 60.0,
        Size::L => 120.0,
    }
}

// --- What the ledger gives ------------------------------------------------------------------

/// The ledger's rows for one project's graphs, read at once.
#[derive(Debug, Clone, Default)]
pub struct Loaded {
    pub project: String,
    /// Every project's session rows: the portfolio's heatmap and medians need them all.
    pub sessions: Vec<SessionRow>,
    /// The project's daily counts.
    pub snapshots: Vec<SnapshotRow>,
    /// The sessions a revert undid, as `(project, short id)` (EP-15b's events).
    pub reverted: HashSet<(String, String)>,
    /// Why the read failed, when it did.
    pub problem: Option<String>,
}

impl Loaded {
    /// Reads the ledger at `path` for `project`.
    pub fn read(path: &Path, project: &str) -> Loaded {
        let mut loaded = Loaded {
            project: project.to_owned(),
            ..Loaded::default()
        };
        let read = Ledger::open_read_only(path).and_then(|ledger| {
            loaded.sessions = ledger.sessions()?;
            loaded.snapshots = ledger.snapshots(project)?;
            for event in ledger.events(revert::EVENT_KIND, None)? {
                if let (Some(project), Some(id)) = (event.project, reverted_id(&event.detail)) {
                    loaded.reverted.insert((project, id.to_owned()));
                }
            }
            Ok(())
        });
        if let Err(e) = read {
            loaded.problem = Some(format!("The ledger did not read for the graphs: {e}"));
        }
        loaded
    }

    /// Whether `row`'s session was reverted.
    pub fn is_reverted(&self, row: &SessionRow) -> bool {
        row.session_id.as_deref().is_some_and(|id| {
            self.reverted
                .contains(&(row.project.clone(), revert::short(id).to_owned()))
        })
    }
}

/// The short session id a revert event's detail opens with: `session 97701680 reverted to …`.
fn reverted_id(detail: &Option<String>) -> Option<&str> {
    let rest = detail.as_deref()?.strip_prefix("session ")?;
    let (id, _) = rest.split_once(" reverted")?;
    Some(id).filter(|id| *id != "-")
}

/// The section's state on the board: the latest read, when, and the switches.
#[derive(Debug, Default)]
pub struct Analytics {
    pub loaded: Option<Loaded>,
    /// The measure from `loaded`, computed once per read ([`measured`]).
    measured: Option<Vec<Measured>>,
    read_at: Option<Instant>,
    /// A read is due before its minute: the files or a project were read again.
    stale: bool,
    /// The heatmap shows the portfolio rather than the project.
    pub portfolio: bool,
    /// A screenshot run's rows: the ledger is not read.
    canned: bool,
}

impl Analytics {
    /// Reads the ledger for `project` when due: the first time, when the project changed, when
    /// marked stale, or a minute after the last read.
    pub fn refresh(&mut self, ledger: Option<&Path>, project: &str, now: Instant) {
        if self.canned {
            return;
        }
        let same = self
            .loaded
            .as_ref()
            .is_some_and(|loaded| loaded.project == project);
        let fresh = self
            .read_at
            .is_some_and(|at| now.duration_since(at) < FRESH_FOR);
        if same && fresh && !self.stale {
            return;
        }
        self.stale = false;
        self.read_at = Some(now);
        self.loaded = ledger.map(|path| Loaded::read(path, project));
        self.measured = None;
    }

    /// The next frame reads the ledger again, and the measure is made again from it (the
    /// roadmaps' sizes may have changed too).
    pub fn invalidate(&mut self) {
        self.stale = true;
        self.measured = None;
    }

    /// A screenshot run's or a test's rows in place of the ledger's.
    #[cfg(any(feature = "screenshot", test))]
    pub fn put(&mut self, loaded: Loaded) {
        self.loaded = Some(loaded);
        self.measured = None;
        self.canned = true;
    }
}

// --- The days ------------------------------------------------------------------------------

/// One day of the heatmap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Day {
    pub date: NaiveDate,
    /// Sessions that started that day, a resumed run's rows counted once by their id.
    pub sessions: usize,
    /// EPs completed that day.
    pub completions: usize,
}

/// The local day `at` falls in.
pub fn day_of(at: DateTime<Utc>) -> NaiveDate {
    at.with_timezone(&Local).date_naive()
}

/// The Monday of `date`'s week.
pub fn monday_of(date: NaiveDate) -> NaiveDate {
    date - Days::new(u64::from(date.weekday().num_days_from_monday()))
}

/// The first day the heatmap shows: the Monday `weeks - 1` weeks before `today`'s.
pub fn first_day(today: NaiveDate, weeks: usize) -> NaiveDate {
    monday_of(today) - Days::new(7 * (weeks.max(1) as u64 - 1))
}

/// A session row that counts as a project's session: it has a project of its own (the chat's
/// turns have none) and it is no visual launch that lapsed or still waits for its session.
fn counts_as_session(row: &SessionRow) -> bool {
    row.project != CHAT && !(row.mode == "visual" && row.session_id.is_none())
}

/// The days from `first_day(today, weeks)` to `today`, each with its sessions (of `project`, or
/// of every project when `None`) and its completions.
pub fn days(
    sessions: &[SessionRow],
    completions: &[Completion],
    project: Option<&str>,
    today: NaiveDate,
    weeks: usize,
) -> Vec<Day> {
    let first = first_day(today, weeks);
    let mut by_day: BTreeMap<NaiveDate, Day> = (0..)
        .map_while(|n| {
            let date = first + Days::new(n);
            (date <= today).then_some((
                date,
                Day {
                    date,
                    sessions: 0,
                    completions: 0,
                },
            ))
        })
        .collect();
    // A session's day is its earliest row's start.
    let mut starts: HashMap<String, NaiveDate> = HashMap::new();
    for row in sessions
        .iter()
        .filter(|row| counts_as_session(row))
        .filter(|row| project.is_none_or(|name| row.project == name))
    {
        let Some(started) = row.started() else {
            continue;
        };
        let key = row
            .session_id
            .clone()
            .unwrap_or_else(|| format!("row {}", row.id));
        let date = day_of(started);
        starts
            .entry(key)
            .and_modify(|first| *first = (*first).min(date))
            .or_insert(date);
    }
    for date in starts.into_values() {
        if let Some(day) = by_day.get_mut(&date) {
            day.sessions += 1;
        }
    }
    for completion in completions {
        if let Some(day) = by_day.get_mut(&completion.completed_on) {
            day.completions += 1;
        }
    }
    by_day.into_values().collect()
}

// --- The burn-up and the velocity ----------------------------------------------------------

/// The burn-up's two stepped series: each `(day, count)` a point where the count changed, the
/// last at `today`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BurnUp {
    pub done: Vec<(NaiveDate, usize)>,
    pub total: Vec<(NaiveDate, usize)>,
}

impl BurnUp {
    /// The first day either series starts on.
    pub fn first(&self) -> Option<NaiveDate> {
        self.done
            .first()
            .map(|(date, _)| *date)
            .into_iter()
            .chain(self.total.first().map(|(date, _)| *date))
            .min()
    }
}

/// The burn-up from the completions (done: their cumulative count by day) and the snapshots
/// (total: flat before the first row, stepped at each change); none without either.
pub fn burn_up(
    completions: &[Completion],
    snapshots: &[SnapshotRow],
    today: NaiveDate,
) -> Option<BurnUp> {
    if completions.is_empty() && snapshots.is_empty() {
        return None;
    }
    let mut per_day: BTreeMap<NaiveDate, usize> = BTreeMap::new();
    for completion in completions {
        *per_day.entry(completion.completed_on).or_default() += 1;
    }
    let mut done = Vec::new();
    let mut sum = 0;
    for (date, n) in per_day {
        sum += n;
        done.push((date, sum));
    }
    let mut total: Vec<(NaiveDate, usize)> = Vec::new();
    for snapshot in snapshots {
        let date = day_of(snapshot.taken_at);
        match total.last() {
            Some((_, last_total)) if *last_total == snapshot.total => {}
            Some((last_date, _)) if *last_date == date => {
                total.pop();
                total.push((date, snapshot.total));
            }
            _ => total.push((date, snapshot.total)),
        }
    }
    // The total before the first snapshot is taken as the first snapshot's, from the first
    // completion's day.
    if let (Some((first_done, _)), Some((first_total, total_then))) = (done.first(), total.first())
        && first_done < first_total
    {
        total.insert(0, (*first_done, *total_then));
    }
    for series in [&mut done, &mut total] {
        if let Some((_, last)) = series.last().copied()
            && series.last().is_some_and(|(date, _)| *date < today)
        {
            series.push((today, last));
        }
    }
    Some(BurnUp { done, total })
}

/// One week of the velocity: its Monday and the EPs completed in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Week {
    pub monday: NaiveDate,
    pub completed: usize,
}

/// The EPs completed per week over the `weeks` ending with `today`'s, the oldest first.
pub fn velocity(completions: &[Completion], today: NaiveDate, weeks: usize) -> Vec<Week> {
    let first = first_day(today, weeks);
    let mut out: Vec<Week> = (0..weeks.max(1) as u64)
        .map(|n| Week {
            monday: first + Days::new(7 * n),
            completed: 0,
        })
        .collect();
    for completion in completions {
        let monday = monday_of(completion.completed_on);
        if let Some(week) = out.iter_mut().find(|week| week.monday == monday) {
            week.completed += 1;
        }
    }
    out
}

// --- The measure of a brief's time ----------------------------------------------------------

/// A session row's time as the measure counts it (the brief's decision, EP-26): a headless row's
/// run length less the owner's waits; a visual or manual row's run length when under the
/// ceiling, as wall time.
#[derive(Debug, Clone, PartialEq)]
pub struct Measured {
    pub project: String,
    pub ep: EpId,
    pub size: Option<Size>,
    pub minutes: f64,
    /// A visual or manual row: the owner's waits are counted in.
    pub wall: bool,
}

/// The sessions that measure a brief's time, one sample per session id (a session has several
/// rows: a visual launch's, its own, a resume's; D-26 addenda of EP-13 and EP-35): an EP named,
/// every row ended and the latest `completed`, not reverted, from the earliest start to the
/// latest end, less the headless rows' waits (about: the rows keep a mean latency), and within
/// the ceiling when any row is visual or manual; the size from `sizes` by project and EP.
pub fn measure(loaded: &Loaded, sizes: &HashMap<(String, EpId), Size>) -> Vec<Measured> {
    struct Group<'a> {
        rows: Vec<&'a SessionRow>,
    }
    let mut groups: BTreeMap<String, Group<'_>> = BTreeMap::new();
    for row in &loaded.sessions {
        let key = row
            .session_id
            .clone()
            .unwrap_or_else(|| format!("row {}", row.id));
        groups
            .entry(key)
            .or_insert_with(|| Group { rows: Vec::new() })
            .rows
            .push(row);
    }
    let mut out = Vec::new();
    for group in groups.into_values() {
        let mut rows = group.rows;
        rows.sort_by(|a, b| a.started_at.cmp(&b.started_at));
        let Some(latest) = rows.last() else {
            continue;
        };
        if latest.outcome.as_deref() != Some("completed") || loaded.is_reverted(latest) {
            continue;
        }
        let Some(ep) = rows
            .iter()
            .find_map(|row| row.ep.as_deref().and_then(|ep| ep.parse::<EpId>().ok()))
        else {
            continue;
        };
        let (Some(start), Some(end)) = (
            rows.iter().filter_map(|row| row.started()).min(),
            rows.iter()
                .map(|row| row.ended())
                .collect::<Option<Vec<_>>>()
                .and_then(|ends| ends.into_iter().max()),
        ) else {
            continue;
        };
        let took = end - start;
        let wall = rows.iter().any(|row| row.mode != "headless");
        if wall && took > CEILING {
            continue;
        }
        let waits: f64 = rows
            .iter()
            .filter(|row| row.mode == "headless")
            .map(|row| f64::from(row.questions) * row.answer_latency_s.unwrap_or(0.0))
            .sum();
        let project = latest.project.clone();
        out.push(Measured {
            size: sizes.get(&(project.clone(), ep)).copied(),
            project,
            ep,
            minutes: (took.num_seconds() as f64 - waits).max(0.0) / 60.0,
            wall,
        });
    }
    out
}

/// The sizes of every row of every project's roadmap, for the measure.
pub fn sizes_of(portfolio: &Portfolio) -> HashMap<(String, EpId), Size> {
    let mut sizes = HashMap::new();
    for row in &portfolio.projects {
        if let Some(roadmap) = row.snapshot.current() {
            for r in roadmap.rows() {
                if let Some(size) = r.size {
                    sizes.insert((row.name().to_owned(), r.id), size);
                }
            }
        }
    }
    sizes
}

/// Where a size's median comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The project's own rows of the size, at least [`LEAST_ROWS`].
    Project(usize),
    /// Every project's rows of the size.
    Portfolio(usize),
    /// The plan's minutes (D-4): no row of the size anywhere.
    Planned,
}

/// A size's median minutes and where they come from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Median {
    pub size: Size,
    pub minutes: f64,
    pub source: Source,
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let n = values.len();
    Some(if n % 2 == 1 {
        values[n / 2]
    } else {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    })
}

/// The median per size for `project`: its own when it has [`LEAST_ROWS`] measured rows of the
/// size, else the portfolio's, else the plan's.
pub fn medians(measured: &[Measured], project: &str) -> [Median; 3] {
    [Size::S, Size::M, Size::L].map(|size| {
        let of_size = measured.iter().filter(|m| m.size == Some(size));
        let own: Vec<f64> = of_size
            .clone()
            .filter(|m| m.project == project)
            .map(|m| m.minutes)
            .collect();
        if own.len() >= LEAST_ROWS
            && let Some(minutes) = median(own.clone())
        {
            return Median {
                size,
                minutes,
                source: Source::Project(own.len()),
            };
        }
        let all: Vec<f64> = of_size.map(|m| m.minutes).collect();
        match median(all.clone()) {
            Some(minutes) => Median {
                size,
                minutes,
                source: Source::Portfolio(all.len()),
            },
            None => Median {
                size,
                minutes: planned(size),
                source: Source::Planned,
            },
        }
    })
}

// --- The projection -------------------------------------------------------------------------

/// The remaining work and the finish at the measured pace.
#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    /// Open rows per size, S, M, L.
    pub remaining: [usize; 3],
    /// Open rows with no size.
    pub no_size: usize,
    pub hours: f64,
    /// EPs per week over every week of the pace window, and over its active weeks alone.
    pub pace_all: f64,
    pub pace_active: f64,
    pub active_weeks: usize,
    /// The early and the late finish, from the two paces; none without a completed EP.
    pub finish: Option<(NaiveDate, NaiveDate)>,
}

impl Projection {
    /// The open rows of `roadmap` at the medians, the pace from `velocity`'s weeks.
    pub fn of(
        roadmap: &Roadmap,
        medians: &[Median; 3],
        velocity: &[Week],
        today: NaiveDate,
    ) -> Projection {
        let mut remaining = [0; 3];
        let mut no_size = 0;
        for row in roadmap.rows() {
            // A row done but not landed (`◐`) is done; a cell the parser could not read counts
            // nowhere.
            if !matches!(row.done, Done::Open | Done::InProgress) {
                continue;
            }
            match row.size {
                Some(Size::S) => remaining[0] += 1,
                Some(Size::M) => remaining[1] += 1,
                Some(Size::L) => remaining[2] += 1,
                None => no_size += 1,
            }
        }
        let hours = medians
            .iter()
            .zip(remaining)
            .map(|(median, n)| median.minutes * n as f64)
            .sum::<f64>()
            / 60.0;
        let open = remaining.iter().sum::<usize>() + no_size;
        let completed: usize = velocity.iter().map(|week| week.completed).sum();
        let active_weeks = velocity.iter().filter(|week| week.completed > 0).count();
        let pace_all = if velocity.is_empty() {
            0.0
        } else {
            completed as f64 / velocity.len() as f64
        };
        let pace_active = if active_weeks == 0 {
            0.0
        } else {
            completed as f64 / active_weeks as f64
        };
        let finish = (completed > 0 && open > 0).then(|| {
            let at = |pace: f64| {
                let days = (open as f64 / pace * 7.0).ceil().clamp(0.0, 36500.0);
                today + Days::new(days as u64)
            };
            (at(pace_active), at(pace_all))
        });
        Projection {
            remaining,
            no_size,
            hours,
            pace_all,
            pace_active,
            active_weeks,
            finish,
        }
    }

    pub fn open(&self) -> usize {
        self.remaining.iter().sum::<usize>() + self.no_size
    }

    /// The remaining work in words: `9 open rows (2 S · 6 M · 1 L): about 8.5 session-hours`.
    pub fn remaining_words(&self) -> String {
        let open = self.open();
        if open == 0 {
            return "No open row: nothing remains.".to_owned();
        }
        let mut mix: Vec<String> = [Size::S, Size::M, Size::L]
            .iter()
            .zip(self.remaining)
            .filter(|(_, n)| *n > 0)
            .map(|(size, n)| format!("{n} {}", size.as_str()))
            .collect();
        if self.no_size > 0 {
            mix.push(format!("{} unsized", self.no_size));
        }
        format!(
            "{} ({}): about {} session-hours at the medians.",
            count(open, "open row"),
            mix.join(" · "),
            hours(self.hours)
        )
    }

    /// The pace and the finish in words.
    pub fn finish_words(&self, weeks: usize) -> String {
        match self.finish {
            None if self.open() == 0 => String::new(),
            None => format!(
                "No EP completed in the last {weeks} weeks: the pace is unmeasured, so no finish \
                 is projected."
            ),
            Some((early, late)) => {
                let pace = if (self.pace_active - self.pace_all).abs() < 0.05 {
                    format!("{} EPs a week", tenths(self.pace_all))
                } else {
                    format!(
                        "{} EPs a week over the last {weeks}, {} over the {}",
                        tenths(self.pace_all),
                        tenths(self.pace_active),
                        count(self.active_weeks, "active week")
                    )
                };
                if early == late {
                    format!("At {pace}, the finish is about {early}.")
                } else {
                    format!("At {pace}, the finish is {early} to {late}.")
                }
            }
        }
    }
}

fn count(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

fn tenths(value: f64) -> String {
    let text = format!("{value:.1}");
    text.strip_suffix(".0").map_or(text.clone(), str::to_owned)
}

/// `8.5`, `12`, `0.5`.
fn hours(value: f64) -> String {
    tenths((value * 2.0).round() / 2.0)
}

/// `19 min`, `1 h 05 min`.
pub fn minutes(value: f64) -> String {
    let whole = value.round() as i64;
    if whole < 60 {
        format!("{whole} min")
    } else {
        format!("{} h {:02} min", whole / 60, whole % 60)
    }
}

/// Each median in words: `S 34 min (its 4 rows) · M 52 min (the portfolio's 9) · L 2 h 00 min
/// (planned)`.
pub fn medians_words(medians: &[Median; 3]) -> String {
    medians
        .iter()
        .map(|median| {
            let from = match median.source {
                Source::Project(n) => format!("its {}", count(n, "row")),
                Source::Portfolio(n) => format!("the portfolio's {n}"),
                Source::Planned => "planned".to_owned(),
            };
            format!(
                "{} {} ({from})",
                median.size.as_str(),
                minutes(median.minutes)
            )
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

// --- The section ---------------------------------------------------------------------------

/// The heatmap's cell, in points, and the gap between cells.
const CELL: f32 = 11.0;
const GAP: f32 = 2.0;
/// The plots' height.
const PLOT_HEIGHT: f32 = 110.0;

/// The sessions that measure a brief's time, from the board's latest read and every roadmap's
/// sizes; computed once per read and kept on the board.
pub fn measured(analytics: &mut Analytics, portfolio: &Portfolio) -> Vec<Measured> {
    if let Some(measured) = &analytics.measured {
        return measured.clone();
    }
    let measured = analytics
        .loaded
        .as_ref()
        .map(|loaded| measure(loaded, &sizes_of(portfolio)))
        .unwrap_or_default();
    analytics.measured = Some(measured.clone());
    measured
}

/// The Analytics section for `row`: the heatmap with its switch, the burn-up, the velocity and
/// the projection's words; one weak line when there is nothing to draw.
pub fn section(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    portfolio: &Portfolio,
    analytics: &mut Analytics,
    measured: &[Measured],
) {
    ui.add_space(10.0);
    ui.label(RichText::new("Analytics").strong());
    ui.add_space(2.0);
    let Some(loaded) = analytics.loaded.as_ref() else {
        ui.label(RichText::new("No ledger: nothing to draw (D-26).").weak());
        return;
    };
    if let Some(problem) = &loaded.problem {
        ui.add(egui::Label::new(RichText::new(problem).color(style::red(ui))).wrap());
    }
    let today = Local::now().date_naive();
    let completions: Vec<Completion> = if analytics.portfolio {
        portfolio
            .projects
            .iter()
            .filter_map(|p| p.history.as_ref())
            .flat_map(|summary| summary.import.completions.iter().cloned())
            .collect()
    } else {
        row.history
            .as_ref()
            .map(|summary| summary.import.completions.clone())
            .unwrap_or_default()
    };
    let own: Vec<Completion> = row
        .history
        .as_ref()
        .map(|summary| summary.import.completions.clone())
        .unwrap_or_default();

    // The heatmap and its switch.
    ui.horizontal(|ui| {
        ui.label(RichText::new("Activity, the last 16 weeks:").weak());
        ui.selectable_value(&mut analytics.portfolio, false, "this project");
        ui.selectable_value(&mut analytics.portfolio, true, "the portfolio");
    });
    let scope = (!analytics.portfolio).then_some(row.name());
    let days = days(&loaded.sessions, &completions, scope, today, WEEKS);
    heatmap(ui, &days);

    // The burn-up.
    match burn_up(&own, &loaded.snapshots, today) {
        Some(series) => {
            ui.label(
                RichText::new("Burn-up: done and total by day; a re-plan is a jump in the total.")
                    .weak(),
            );
            burn_up_plot(ui, row.name(), &series);
        }
        None => {
            ui.label(
                RichText::new(
                    "No burn-up yet: no EP dated by the history and no daily count kept.",
                )
                .weak(),
            );
        }
    }

    // The velocity and the projection.
    let weeks = velocity(&own, today, PACE_WEEKS);
    ui.label(RichText::new("Velocity: EPs completed per week, the last 8 weeks.").weak());
    velocity_plot(ui, row.name(), &weeks);
    let medians = medians(measured, row.name());
    if let Some(roadmap) = row.snapshot.current() {
        let projection = Projection::of(roadmap, &medians, &weeks, today);
        ui.add(egui::Label::new(projection.remaining_words()).wrap());
        ui.add(
            egui::Label::new(
                RichText::new(format!("The medians: {}.", medians_words(&medians))).weak(),
            )
            .wrap(),
        );
        let finish = projection.finish_words(PACE_WEEKS);
        if !finish.is_empty() {
            ui.add(egui::Label::new(finish).wrap());
        }
    }
    ui.add(
        egui::Label::new(
            RichText::new(
                "A brief's time is about its headless run less the owner's waits, or a visual \
                 or manual session under 3 h as wall time; a reverted session is left out.",
            )
            .weak()
            .small(),
        )
        .wrap(),
    );
}

/// The calendar heatmap: a column per week, a row per weekday, a cell's fill by its sessions
/// and a dot for a day with a completion; each cell's hover text names the day.
fn heatmap(ui: &mut egui::Ui, days: &[Day]) {
    let Some(first) = days.first().map(|day| day.date) else {
        return;
    };
    let weeks = days.len().div_ceil(7).max(1);
    let label_width = 18.0;
    let header = 14.0;
    let width = label_width + weeks as f32 * (CELL + GAP);
    let height = header + 7.0 * (CELL + GAP);
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    let weak = visuals.weak_text_color();
    let font = FontId::proportional(9.0);
    let most = days
        .iter()
        .map(|day| day.sessions)
        .max()
        .unwrap_or(0)
        .max(1);
    let ink = style::blue(ui);
    let done = style::green(ui);
    let origin = Pos2::new(rect.left() + label_width, rect.top() + header);
    for (label, line) in [("M", 0), ("W", 2), ("F", 4)] {
        let y = origin.y + line as f32 * (CELL + GAP) + CELL / 2.0;
        painter.text(
            Pos2::new(rect.left(), y),
            Align2::LEFT_CENTER,
            label,
            font.clone(),
            weak,
        );
    }
    let mut last_month = None;
    let mut hovered: Option<&Day> = None;
    let pointer = response.hover_pos();
    for day in days {
        let offset = (day.date - first).num_days();
        let (week, weekday) = (offset / 7, offset % 7);
        let min = Pos2::new(
            origin.x + week as f32 * (CELL + GAP),
            origin.y + weekday as f32 * (CELL + GAP),
        );
        let cell = Rect::from_min_size(min, vec2(CELL, CELL));
        if weekday == 0 && last_month != Some(day.date.month()) && day.date.day() <= 7 {
            painter.text(
                Pos2::new(min.x, rect.top()),
                Align2::LEFT_TOP,
                day.date.format("%b").to_string(),
                font.clone(),
                weak,
            );
            last_month = Some(day.date.month());
        }
        let fill = if day.sessions == 0 {
            visuals.extreme_bg_color
        } else {
            let strength = 0.35 + 0.65 * (day.sessions as f32 / most as f32);
            ink.gamma_multiply(strength)
        };
        painter.rect_filled(cell, 2.0, fill);
        if day.completions > 0 {
            painter.circle_filled(cell.center(), 2.2, done);
        }
        if pointer.is_some_and(|at| cell.contains(at)) {
            hovered = Some(day);
            painter.rect_stroke(
                cell,
                2.0,
                Stroke::new(1.0, visuals.text_color()),
                egui::StrokeKind::Outside,
            );
        }
    }
    if let Some(day) = hovered {
        response.on_hover_text(format!(
            "{}: {}, {}",
            day.date.format("%a %Y-%m-%d"),
            count(day.sessions, "session"),
            count(day.completions, "EP completed")
        ));
    }
}

/// The burn-up plot: done and total as stepped lines, days on the x axis.
fn burn_up_plot(ui: &mut egui::Ui, project: &str, series: &BurnUp) {
    let Some(first) = series.first() else {
        return;
    };
    let steps = |points: &[(NaiveDate, usize)]| -> PlotPoints<'static> {
        let mut out: Vec<[f64; 2]> = Vec::new();
        for &(date, n) in points {
            let x = (date - first).num_days() as f64;
            if let Some(last) = out.last().copied() {
                out.push([x, last[1]]);
            }
            out.push([x, n as f64]);
        }
        PlotPoints::from(out)
    };
    let text = ui.visuals().text_color();
    // A step's two points share an x, which egui_plot would aggregate into a band: off.
    let done = Line::new("done", steps(&series.done))
        .color(style::green(ui))
        .width(2.0)
        .allow_band(false);
    let total = Line::new("total", steps(&series.total))
        .color(text)
        .width(1.5)
        .allow_band(false);
    Plot::new(("burn-up", project))
        .height(PLOT_HEIGHT)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .allow_double_click_reset(false)
        .show_background(false)
        .include_y(0.0)
        .legend(Legend::default().position(Corner::LeftTop))
        .x_axis_formatter(move |mark, _| {
            (first + Days::new(mark.value.max(0.0) as u64))
                .format("%m-%d")
                .to_string()
        })
        .label_formatter(move |hover| match hover {
            egui_plot::HoverPosition::NearDataPoint {
                plot_name,
                position,
                ..
            } => Some(format!(
                "{plot_name} {} on {}",
                position.y.round(),
                (first + Days::new(position.x.max(0.0) as u64)).format("%Y-%m-%d")
            )),
            egui_plot::HoverPosition::Elsewhere { .. } => None,
        })
        .show(ui, |plot| {
            plot.line(total);
            plot.line(done);
        });
}

/// The velocity plot: a bar per week.
fn velocity_plot(ui: &mut egui::Ui, project: &str, weeks: &[Week]) {
    let mondays: Vec<NaiveDate> = weeks.iter().map(|week| week.monday).collect();
    let bars: Vec<Bar> = weeks
        .iter()
        .enumerate()
        .map(|(i, week)| {
            Bar::new(i as f64, week.completed as f64)
                .width(0.7)
                .name(format!("the week of {}", week.monday))
        })
        .collect();
    let chart = BarChart::new("EPs", bars).color(style::blue(ui));
    let labels = mondays.clone();
    Plot::new(("velocity", project))
        .height(PLOT_HEIGHT)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .allow_double_click_reset(false)
        .show_background(false)
        .include_y(0.0)
        .include_x(-0.5)
        .include_x(weeks.len() as f64 - 0.5)
        .x_axis_formatter(move |mark, _| {
            let i = mark.value.round();
            if (mark.value - i).abs() > 0.01 || i < 0.0 {
                return String::new();
            }
            labels
                .get(i as usize)
                .map(|monday| monday.format("%m-%d").to_string())
                .unwrap_or_default()
        })
        .label_formatter(|_| None)
        .show(ui, |plot| plot.bar_chart(chart));
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crw_core::history::Flag;
    use crw_core::ledger::RowUsage;

    pub(crate) fn completion(ep: &str, on: &str) -> Completion {
        Completion {
            ep: ep.parse().unwrap(),
            completed_on: on.parse().unwrap(),
            git_date: Some(on.parse().unwrap()),
            note_date: None,
            record: vec!["1111111".to_owned()],
            work: Vec::new(),
            flag: Flag::GitOnly,
        }
    }

    /// A session row; `ended` is its minutes and outcome, none while it runs.
    pub(crate) fn session(
        id: i64,
        session_id: Option<&str>,
        project: &str,
        ep: Option<&str>,
        mode: &str,
        started: DateTime<Utc>,
        ended: Option<(i64, &str)>,
    ) -> SessionRow {
        let (minutes, outcome) = ended.map_or((0, None), |(m, o)| (m, Some(o)));
        SessionRow {
            id,
            session_id: session_id.map(str::to_owned),
            project: project.to_owned(),
            ep: ep.map(str::to_owned),
            mode: mode.to_owned(),
            model: Some("fable".to_owned()),
            effort: Some("high".to_owned()),
            ultracode: false,
            started_at: started.to_rfc3339(),
            ended_at: outcome.map(|_| (started + TimeDelta::minutes(minutes)).to_rfc3339()),
            outcome: outcome.map(str::to_owned),
            cost_usd: None,
            questions: 0,
            result: None,
            context_percent: None,
            answer_latency_s: None,
            start: None,
            usage: RowUsage::default(),
        }
    }

    fn at(date: &str, hour: u32) -> DateTime<Utc> {
        let local = date
            .parse::<NaiveDate>()
            .unwrap()
            .and_hms_opt(hour, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .single()
            .unwrap();
        local.with_timezone(&Utc)
    }

    #[test]
    fn the_days_count_a_resumed_session_once_and_leave_the_chat_and_lapsed_launches_out() {
        let today: NaiveDate = "2026-11-12".parse().unwrap();
        let sessions = vec![
            session(
                1,
                Some("aaaa-1"),
                "harbor",
                Some("EP-3"),
                "headless",
                at("2026-11-10", 9),
                Some((30, "completed")),
            ),
            // The same session resumed: one session that day.
            session(
                2,
                Some("aaaa-1"),
                "harbor",
                Some("EP-3"),
                "manual",
                at("2026-11-10", 11),
                Some((20, "completed")),
            ),
            session(
                3,
                Some("bbbb-2"),
                "lantern",
                Some("EP-6"),
                "visual",
                at("2026-11-10", 14),
                Some((60, "completed")),
            ),
            // A visual launch no session took up.
            session(
                4,
                None,
                "harbor",
                Some("EP-4"),
                "visual",
                at("2026-11-11", 9),
                None,
            ),
            // A chat's turn.
            session(
                5,
                Some("cccc-3"),
                CHAT,
                None,
                "headless",
                at("2026-11-11", 10),
                Some((1, "completed")),
            ),
            // Before the window.
            session(
                6,
                Some("dddd-4"),
                "harbor",
                Some("EP-1"),
                "manual",
                at("2026-01-01", 10),
                Some((10, "completed")),
            ),
        ];
        let completions = [
            completion("EP-3", "2026-11-10"),
            completion("EP-4", "2026-11-12"),
        ];
        let own = days(&sessions, &completions, Some("harbor"), today, WEEKS);
        assert_eq!(
            own.len(),
            7 * 15 + 4,
            "15 full weeks and Monday to Thursday"
        );
        assert_eq!(
            own.first().unwrap().date,
            "2026-07-27".parse::<NaiveDate>().unwrap()
        );
        let day = |date: &str| {
            own.iter()
                .find(|day| day.date == date.parse::<NaiveDate>().unwrap())
                .copied()
                .unwrap()
        };
        assert_eq!(
            (day("2026-11-10").sessions, day("2026-11-10").completions),
            (1, 1)
        );
        assert_eq!(
            (day("2026-11-11").sessions, day("2026-11-11").completions),
            (0, 0)
        );
        assert_eq!(
            (day("2026-11-12").sessions, day("2026-11-12").completions),
            (0, 1)
        );
        let all = days(&sessions, &completions, None, today, WEEKS);
        assert_eq!(
            all.iter()
                .find(|d| d.date == day("2026-11-10").date)
                .unwrap()
                .sessions,
            2,
            "the portfolio adds lantern's"
        );
    }

    #[test]
    fn the_burn_up_steps_done_by_completions_and_total_by_snapshots() {
        let today: NaiveDate = "2026-11-12".parse().unwrap();
        let completions = [
            completion("EP-1", "2026-11-01"),
            completion("EP-2", "2026-11-01"),
            completion("EP-3", "2026-11-05"),
        ];
        let snapshot = |date: &str, done, total| SnapshotRow {
            taken_at: at(date, 12),
            done,
            total,
        };
        let snapshots = [
            snapshot("2026-11-04", 2, 10),
            snapshot("2026-11-05", 3, 10),
            snapshot("2026-11-08", 3, 12),
        ];
        let series = burn_up(&completions, &snapshots, today).unwrap();
        let d = |s: &str| s.parse::<NaiveDate>().unwrap();
        assert_eq!(
            series.done,
            vec![
                (d("2026-11-01"), 2),
                (d("2026-11-05"), 3),
                (d("2026-11-12"), 3)
            ]
        );
        assert_eq!(
            series.total,
            vec![
                (d("2026-11-01"), 10),
                (d("2026-11-04"), 10),
                (d("2026-11-08"), 12),
                (d("2026-11-12"), 12)
            ],
            "flat before the first snapshot, a jump at the re-plan, carried to today"
        );
        assert_eq!(series.first(), Some(d("2026-11-01")));
        assert!(burn_up(&[], &[], today).is_none());
    }

    #[test]
    fn the_velocity_counts_the_last_eight_weeks_from_monday() {
        let today: NaiveDate = "2026-11-12".parse().unwrap();
        let completions = [
            completion("EP-1", "2026-11-09"),
            completion("EP-2", "2026-11-12"),
            completion("EP-3", "2026-11-08"),
            completion("EP-4", "2026-09-01"),
        ];
        let weeks = velocity(&completions, today, PACE_WEEKS);
        assert_eq!(weeks.len(), 8);
        assert_eq!(weeks[0].monday, "2026-09-21".parse::<NaiveDate>().unwrap());
        assert_eq!(weeks[7].monday, "2026-11-09".parse::<NaiveDate>().unwrap());
        assert_eq!(weeks[7].completed, 2);
        assert_eq!(weeks[6].completed, 1, "Sunday the 8th is the week before");
        assert_eq!(weeks.iter().map(|w| w.completed).sum::<usize>(), 3);
    }

    #[test]
    fn the_measure_takes_the_waits_off_a_headless_row_and_leaves_reverted_and_long_rows_out() {
        let mut headless = session(
            1,
            Some("97701680-aaaa"),
            "harbor",
            Some("EP-2"),
            "headless",
            at("2026-11-10", 9),
            Some((40, "completed")),
        );
        headless.questions = 2;
        headless.answer_latency_s = Some(60.0);
        let visual = session(
            2,
            Some("bbbb"),
            "harbor",
            Some("EP-3"),
            "visual",
            at("2026-11-10", 12),
            Some((50, "completed")),
        );
        // The visual launch's own row, under the same id, started ten minutes before the
        // session and closed with it (EP-13): one sample from the launch to the end.
        let mut launch = session(
            20,
            Some("bbbb"),
            "harbor",
            Some("EP-3"),
            "visual",
            at("2026-11-10", 12) - TimeDelta::minutes(10),
            Some((60, "completed")),
        );
        launch.model = None;
        let long = session(
            3,
            Some("cccc"),
            "harbor",
            Some("EP-4"),
            "manual",
            at("2026-11-10", 14),
            Some((20 * 60, "completed")),
        );
        let reverted = session(
            4,
            Some("dddd1234-eeee"),
            "harbor",
            Some("EP-5"),
            "headless",
            at("2026-11-11", 9),
            Some((30, "completed")),
        );
        let stopped = session(
            5,
            Some("ffff"),
            "harbor",
            Some("EP-6"),
            "headless",
            at("2026-11-11", 10),
            Some((30, "stopped")),
        );
        let open = session(
            6,
            Some("gggg"),
            "harbor",
            Some("EP-7"),
            "headless",
            at("2026-11-11", 11),
            None,
        );
        let no_ep = session(
            7,
            Some("hhhh"),
            "harbor",
            None,
            "manual",
            at("2026-11-11", 12),
            Some((30, "completed")),
        );
        let loaded = Loaded {
            project: "harbor".to_owned(),
            sessions: vec![
                headless, visual, launch, long, reverted, stopped, open, no_ep,
            ],
            snapshots: Vec::new(),
            reverted: HashSet::from([("harbor".to_owned(), "dddd1234".to_owned())]),
            problem: None,
        };
        let sizes = HashMap::from([
            (("harbor".to_owned(), "EP-2".parse().unwrap()), Size::M),
            (("harbor".to_owned(), "EP-3".parse().unwrap()), Size::S),
        ]);
        let measured = measure(&loaded, &sizes);
        assert_eq!(measured.len(), 2);
        assert_eq!(measured[0].ep, "EP-2".parse().unwrap());
        assert!(
            (measured[0].minutes - 38.0).abs() < 0.01,
            "40 min less 2 × 60 s"
        );
        assert!(!measured[0].wall);
        assert_eq!(measured[0].size, Some(Size::M));
        assert_eq!(measured[1].ep, "EP-3".parse().unwrap());
        assert!(
            (measured[1].minutes - 60.0).abs() < 0.01,
            "one sample for the session's two rows, from the launch to the end"
        );
        assert!(measured[1].wall);
        // The medians: no size has three rows of harbor's, so the portfolio's rows count, and
        // L falls back to the plan.
        let found = medians(&measured, "harbor");
        assert_eq!(found[0].source, Source::Portfolio(1));
        assert!((found[0].minutes - 60.0).abs() < 0.01);
        assert_eq!(found[1].source, Source::Portfolio(1));
        assert_eq!(found[2].source, Source::Planned);
        assert!((found[2].minutes - 120.0).abs() < 0.01);
        assert_eq!(
            medians_words(&found),
            "S 1 h 00 min (the portfolio's 1) · M 38 min (the portfolio's 1) · L 2 h 00 min (planned)"
        );
        // Three rows of a size of its own: the project's median.
        let own: Vec<Measured> = [30.0, 34.0, 45.0, 40.0]
            .into_iter()
            .enumerate()
            .map(|(i, minutes)| Measured {
                project: "harbor".to_owned(),
                ep: format!("EP-{}", i + 10).parse().unwrap(),
                size: Some(Size::S),
                minutes,
                wall: false,
            })
            .collect();
        let found = medians(&own, "harbor");
        assert_eq!(found[0].source, Source::Project(4));
        assert!(
            (found[0].minutes - 37.0).abs() < 0.01,
            "the mean of the middle two"
        );
    }

    #[test]
    fn the_reverted_id_is_read_from_the_event() {
        assert_eq!(
            reverted_id(&Some("session 97701680 reverted to abc1234 (its recorded start): 2 commits, 3 files; the work is kept in x".to_owned())),
            Some("97701680")
        );
        assert_eq!(
            reverted_id(&Some("session - reverted to abc1234".to_owned())),
            None
        );
        assert_eq!(reverted_id(&None), None);
    }

    #[test]
    fn the_projection_sums_the_mix_and_gives_a_finish_range() {
        use crw_core::roadmap::table;
        let text = "\
# Roadmap

## Phase P1 — One

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | A | S | — | core | ☑ `1111111` |
| EP-2 | B | M | EP-1 | core | ☐ |
| EP-3 | C | M | EP-2 | core | ☐ |
| EP-4 | D | L | EP-3 | core | ☐ |
| EP-5 | E |  | EP-4 | core | ☐ |
";
        let roadmap = table::parse(text);
        let medians = [
            Median {
                size: Size::S,
                minutes: 30.0,
                source: Source::Planned,
            },
            Median {
                size: Size::M,
                minutes: 45.0,
                source: Source::Portfolio(4),
            },
            Median {
                size: Size::L,
                minutes: 120.0,
                source: Source::Planned,
            },
        ];
        let today: NaiveDate = "2026-11-12".parse().unwrap();
        // Four EPs over the eight weeks, in two active weeks.
        let completions = [
            completion("EP-1", "2026-11-02"),
            completion("EP-6", "2026-11-03"),
            completion("EP-7", "2026-11-09"),
            completion("EP-8", "2026-11-10"),
        ];
        let weeks = velocity(&completions, today, PACE_WEEKS);
        let projection = Projection::of(&roadmap, &medians, &weeks, today);
        assert_eq!(projection.remaining, [0, 2, 1]);
        assert_eq!(projection.no_size, 1);
        assert!(
            (projection.hours - 3.5).abs() < 0.01,
            "2 × 45 + 120 minutes"
        );
        assert_eq!(projection.active_weeks, 2);
        assert!((projection.pace_all - 0.5).abs() < 0.01);
        assert!((projection.pace_active - 2.0).abs() < 0.01);
        assert_eq!(
            projection.remaining_words(),
            "4 open rows (2 M · 1 L · 1 unsized): about 3.5 session-hours at the medians."
        );
        // 4 open at 2 a week: 14 days; at 0.5 a week: 56 days.
        assert_eq!(
            projection.finish,
            Some(("2026-11-26".parse().unwrap(), "2027-01-07".parse().unwrap()))
        );
        assert_eq!(
            projection.finish_words(PACE_WEEKS),
            "At 0.5 EPs a week over the last 8, 2 over the 2 active weeks, the finish is \
             2026-11-26 to 2027-01-07."
        );
        let none = Projection::of(&roadmap, &medians, &velocity(&[], today, PACE_WEEKS), today);
        assert_eq!(none.finish, None);
        assert!(none.finish_words(PACE_WEEKS).starts_with("No EP completed"));
    }

    #[test]
    fn the_read_stands_a_minute_and_is_due_again_when_invalidated_or_the_project_changes() {
        let mut analytics = Analytics::default();
        let now = Instant::now();
        analytics.refresh(None, "harbor", now);
        assert!(analytics.loaded.is_none(), "no ledger");
        analytics.put(Loaded {
            project: "harbor".to_owned(),
            ..Loaded::default()
        });
        analytics.refresh(None, "lantern", now + Duration::from_secs(120));
        assert_eq!(
            analytics.loaded.as_ref().unwrap().project,
            "harbor",
            "canned rows are never read over"
        );
        let mut live = Analytics {
            loaded: Some(Loaded {
                project: "harbor".to_owned(),
                ..Loaded::default()
            }),
            read_at: Some(now),
            ..Analytics::default()
        };
        live.refresh(None, "harbor", now + Duration::from_secs(10));
        assert!(live.loaded.is_some(), "fresh: not read again");
        live.invalidate();
        live.refresh(None, "harbor", now + Duration::from_secs(10));
        assert!(
            live.loaded.is_none(),
            "stale: read again, and there is no ledger"
        );
    }
}
