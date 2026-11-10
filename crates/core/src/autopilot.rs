//! The autopilot (D-22; EP-19): after a headless run of the window's ends, the **gate** decides
//! whether the project's next brief starts on its own, and names every condition that fails. It
//! weighs [`Facts`] read afresh from the project ([`read`]: the roadmap, the briefs, git's status,
//! the ledger's day, the settings files) and from the meter. A refusal made by the pause alone (the
//! owner's Pause, a limit at or over the mark, a figure too old to trust) holds the chain until it
//! lifts; any other ends it. A chained run is drafted as the launch dialog drafts one, with the
//! commit variant of the prompt (`prompt`), and has a **cap** by size counted in working time: the
//! time a request waits for the owner is left out (the owner's choice). The **log** is the
//! ledger's `event` rows of kind [`EVENT_KIND`], whose words begin with what happened, so the
//! day's chained runs are counted from it. Nothing here starts a run: the window does.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, NaiveDate, TimeDelta, Utc};

use crate::ledger::{self, EventRow, Ledger};
use crate::prompt::{self, Draft};
use crate::registry::{Project, Registry};
use crate::roadmap::table::Row;
use crate::roadmap::{EpId, Size, snapshot};
use crate::rules::{Picked, Resolution};
use crate::session::{Launch, Outcome};
use crate::usage::{self, Figure, Figures};
use crate::{git, paths, rules};

/// The ledger's `event.kind` of the chain log.
pub const EVENT_KIND: &str = "autopilot";

/// The largest brief a chain starts unless the project says otherwise (D-22: S and M; an L
/// needs a click).
pub const DEFAULT_MAX_SIZE: Size = Size::M;
/// Chained runs a day in one project, and in all of them (D-22).
pub const DEFAULT_DAILY: u32 = 2;
pub const DEFAULT_DAILY_TOTAL: u32 = 4;
/// The share of the weekly or the Fable limit at which chains hold until the reset (D-22).
pub const DEFAULT_PAUSE_PERCENT: u8 = 90;
/// How long a figure under the mark is trusted after its reading: the meter's own freshness.
pub const TRUSTED_FOR: TimeDelta = usage::FRESH_FOR;

/// The permission modes under which a headless run asks the owner at each shell command unless
/// an allow rule lets the command through (D-18).
pub const WAITING_MODES: [&str; 4] = ["acceptEdits", "manual", "plan", "default"];

/// The first word of a chained start's line in the log, by which the day's runs are counted.
const CHAINED: &str = "chained ";

/// The owner's Pause and its end, as the log says them.
pub const PAUSED_BY_OWNER: &str = "paused by the owner: no chain goes on until the Resume";
pub const RESUMED_BY_OWNER: &str = "resumed by the owner";

/// A run's cap by its brief's size (D-22): S 45 min, M 90 min, L 3 h; a brief with no size
/// takes L's.
pub fn cap(size: Option<Size>) -> Duration {
    match size {
        Some(Size::S) => Duration::from_secs(45 * 60),
        Some(Size::M) => Duration::from_secs(90 * 60),
        Some(Size::L) | None => Duration::from_secs(3 * 60 * 60),
    }
}

/// A duration in the ledger's words: `1 h 30 min`.
pub fn length(duration: Duration) -> String {
    ledger::length(TimeDelta::from_std(duration).unwrap_or(TimeDelta::MAX))
}

/// A run's working time (the owner's choice at EP-19): the time since its start less the time a
/// request of its waited for the owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    started: Instant,
    waited: Duration,
    /// Since when a request waits, while one does.
    since: Option<Instant>,
}

impl Clock {
    pub fn start(at: Instant) -> Self {
        Self {
            started: at,
            waited: Duration::ZERO,
            since: None,
        }
    }

    /// Takes in whether a request waits for the owner at `now`.
    pub fn tick(&mut self, waiting: bool, now: Instant) {
        match (waiting, self.since) {
            (true, None) => self.since = Some(now),
            (false, Some(since)) => {
                self.waited += now.saturating_duration_since(since);
                self.since = None;
            }
            _ => {}
        }
    }

    /// The working time at `now`.
    pub fn worked(&self, now: Instant) -> Duration {
        let waiting = self
            .since
            .map_or(Duration::ZERO, |since| now.saturating_duration_since(since));
        now.saturating_duration_since(self.started)
            .saturating_sub(self.waited + waiting)
    }
}

/// A run's cap and its clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cap {
    pub size: Option<Size>,
    pub limit: Duration,
    pub clock: Clock,
}

impl Cap {
    /// The cap of a run of a brief of `size` that starts at `at`.
    pub fn new(size: Option<Size>, at: Instant) -> Self {
        Self {
            size,
            limit: cap(size),
            clock: Clock::start(at),
        }
    }

    /// Whether the run has worked its cap at `now`.
    pub fn reached(&self, now: Instant) -> bool {
        self.clock.worked(now) >= self.limit
    }

    /// The cap in words: `M's cap of 1 h 30 min`.
    pub fn words(&self) -> String {
        match self.size {
            Some(size) => format!("{size}'s cap of {}", length(self.limit)),
            None => format!("the cap of {} (no size)", length(self.limit)),
        }
    }
}

/// The roadmap's next brief, as the run left it.
#[derive(Debug, Clone, PartialEq)]
pub struct Next {
    pub id: EpId,
    pub title: String,
    pub size: Option<Size>,
}

/// What the gate weighs after a run (D-22).
#[derive(Debug, Clone, PartialEq)]
pub struct Facts {
    /// The project's autopilot is on.
    pub enabled: bool,
    /// The owner paused every chain.
    pub paused: bool,
    pub outcome: Outcome,
    /// The EP the run worked on.
    pub ran: Option<EpId>,
    /// Its brief has a completion note.
    pub completed: bool,
    /// The next brief, or why there is none.
    pub next: Result<Next, String>,
    /// The open owner tasks gated before the next brief, by id.
    pub due: Vec<String>,
    pub max_size: Size,
    pub weekly: Option<Figure>,
    pub fable: Option<Figure>,
    pub pause_percent: u8,
    /// The working tree's changes, by path, or why git could not say.
    pub tree: Result<Vec<String>, String>,
    /// The chained runs today in the project, and its cap.
    pub chained_today: u32,
    pub daily: u32,
    /// The chained runs today in every project, and the cap.
    pub chained_total: u32,
    pub daily_total: u32,
    /// The other sessions open in the project, in words.
    pub others: Vec<String>,
    /// The project is the repository the running window was built from (the owner's choice at
    /// EP-19; by the build's repository from EP-37, since the window runs from its installed
    /// copy).
    pub own: bool,
    /// The permission mode under which its runs would wait at each shell command, when they
    /// would.
    pub waits: Option<String>,
}

/// A condition of the gate that fails, which the log names.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    Off,
    Paused,
    Outcome(Outcome),
    NoEp,
    NoCompletionNote(EpId),
    /// The run's row is still the next one: its hash was not recorded.
    RowOpen(EpId),
    NoNext(String),
    Due {
        next: EpId,
        tasks: Vec<String>,
    },
    Replan(EpId),
    Size {
        next: EpId,
        size: Option<Size>,
        max: Size,
    },
    /// A limit at or over the mark until its reset: the automatic pause.
    Usage {
        which: &'static str,
        percent: f64,
        mark: u8,
        until: Option<DateTime<Utc>>,
    },
    /// A figure too old to trust, or read before its window reset.
    Stale {
        which: &'static str,
        minutes: i64,
        reset: bool,
    },
    Unread(&'static str),
    Dirty(Vec<String>),
    GitUnknown(String),
    DailyProject {
        count: u32,
        cap: u32,
    },
    DailyTotal {
        count: u32,
        cap: u32,
    },
    Busy(Vec<String>),
    OwnProject,
    Waits(String),
}

impl Refusal {
    /// A refusal the pause makes, which holds a chain until it lifts rather than ending it.
    pub fn is_pause(&self) -> bool {
        matches!(
            self,
            Refusal::Paused | Refusal::Usage { .. } | Refusal::Stale { .. }
        )
    }

    /// The condition in words, for the log.
    pub fn words(&self) -> String {
        match self {
            Refusal::Off => "the autopilot is off for the project".to_owned(),
            Refusal::Paused => "the autopilot is paused (your Pause)".to_owned(),
            Refusal::Outcome(Outcome::Deferred) => "the run was deferred with a request open: \
                                                    your answer resumes it, and its end weighs \
                                                    the gate again"
                .to_owned(),
            Refusal::Outcome(outcome) => {
                format!("the run ended {} rather than completed", outcome.as_str())
            }
            Refusal::NoEp => "the run named no EP".to_owned(),
            Refusal::NoCompletionNote(ep) => format!("{ep}'s brief has no completion note"),
            Refusal::RowOpen(ep) => {
                format!("{ep} is still the next brief: its row's hash is not recorded")
            }
            Refusal::NoNext(why) => format!("no next brief: {why}"),
            Refusal::Due { next, tasks } => format!(
                "owner tasks due before {next} are open: {}",
                tasks.join(", ")
            ),
            Refusal::Replan(next) => format!("{next} is a re-plan, the owner's to run"),
            Refusal::Size {
                next,
                size: Some(size),
                max,
            } => format!("{next} is {size}, over the autopilot's {max}"),
            Refusal::Size {
                next, size: None, ..
            } => format!("{next} has no size"),
            Refusal::Usage {
                which,
                percent,
                mark,
                until,
            } => {
                let until = until.map_or(String::new(), |at| {
                    format!(
                        " until its reset, {}",
                        at.with_timezone(&Local).format("%a %d %b %H:%M")
                    )
                });
                format!("{which} is at {percent:.0} %, at or over the pause at {mark} %{until}")
            }
            Refusal::Stale {
                which, reset: true, ..
            } => format!("the {which} figure's window has reset since its reading"),
            Refusal::Stale { which, minutes, .. } => format!(
                "the {which} figure was read {minutes} min ago, too old to trust (over {} min)",
                TRUSTED_FOR.num_minutes()
            ),
            Refusal::Unread(which) => format!("no {which} figure has been read"),
            Refusal::Dirty(paths) => {
                let shown: Vec<&str> = paths.iter().take(3).map(String::as_str).collect();
                let more = if paths.len() > shown.len() {
                    ", …"
                } else {
                    ""
                };
                format!(
                    "the tree is not clean after the commit: {} change{} ({}{more})",
                    paths.len(),
                    if paths.len() == 1 { "" } else { "s" },
                    shown.join(", ")
                )
            }
            Refusal::GitUnknown(why) => format!("git's status did not read: {why}"),
            Refusal::DailyProject { count, cap } => {
                format!("{count} chained runs today in the project, its cap is {cap}")
            }
            Refusal::DailyTotal { count, cap } => {
                format!("{count} chained runs today in all projects, the cap is {cap}")
            }
            Refusal::Busy(others) => {
                format!(
                    "another session is open in the project: {}",
                    others.join("; ")
                )
            }
            Refusal::OwnProject => "it is the tool's own project: the window's restart into \
                                    each brief's build would end its chain"
                .to_owned(),
            Refusal::Waits(mode) => format!(
                "its runs would wait for you at each shell command: permission mode {mode} and \
                 no allow rule (keep auto with the pin beside the launch dialog's Permissions, \
                 crw set <project> permission_mode auto, or allow rules in its .claude settings)"
            ),
        }
    }
}

/// Whether a brief's title names a re-plan, which the owner runs (D-22).
pub fn is_replan(title: &str) -> bool {
    let title = title.to_lowercase();
    title.contains("re-plan") || title.contains("replan")
}

/// The gate (D-22): every condition that fails, in the order the log names them; none when the
/// next brief may start.
pub fn gate(facts: &Facts, now: DateTime<Utc>) -> Vec<Refusal> {
    let mut no = Vec::new();
    if !facts.enabled {
        no.push(Refusal::Off);
    }
    if facts.paused {
        no.push(Refusal::Paused);
    }
    if facts.outcome != Outcome::Completed {
        no.push(Refusal::Outcome(facts.outcome));
    }
    match facts.ran {
        None => no.push(Refusal::NoEp),
        Some(ran) if !facts.completed => no.push(Refusal::NoCompletionNote(ran)),
        Some(_) => {}
    }
    match &facts.next {
        Err(why) => no.push(Refusal::NoNext(why.clone())),
        Ok(next) => {
            if facts.ran == Some(next.id) {
                no.push(Refusal::RowOpen(next.id));
            }
            if !facts.due.is_empty() {
                no.push(Refusal::Due {
                    next: next.id,
                    tasks: facts.due.clone(),
                });
            }
            if is_replan(&next.title) {
                no.push(Refusal::Replan(next.id));
            }
            match next.size {
                Some(size) if size <= facts.max_size => {}
                size => no.push(Refusal::Size {
                    next: next.id,
                    size,
                    max: facts.max_size,
                }),
            }
        }
    }
    for (which, figure) in [("weekly", &facts.weekly), ("Fable", &facts.fable)] {
        no.extend(weigh(which, figure.as_ref(), facts.pause_percent, now));
    }
    match &facts.tree {
        Ok(changes) if changes.is_empty() => {}
        Ok(changes) => no.push(Refusal::Dirty(changes.clone())),
        Err(why) => no.push(Refusal::GitUnknown(why.clone())),
    }
    if facts.chained_today >= facts.daily {
        no.push(Refusal::DailyProject {
            count: facts.chained_today,
            cap: facts.daily,
        });
    }
    if facts.chained_total >= facts.daily_total {
        no.push(Refusal::DailyTotal {
            count: facts.chained_total,
            cap: facts.daily_total,
        });
    }
    if !facts.others.is_empty() {
        no.push(Refusal::Busy(facts.others.clone()));
    }
    if facts.own {
        no.push(Refusal::OwnProject);
    }
    if let Some(mode) = &facts.waits {
        no.push(Refusal::Waits(mode.clone()));
    }
    no
}

/// Whether a refused chain is held rather than ended: the pause alone refused it.
pub fn holds(refusals: &[Refusal]) -> bool {
    !refusals.is_empty() && refusals.iter().all(Refusal::is_pause)
}

/// One usage figure against the mark (the brief's rule): at or over it, it counts until its
/// reset whatever its age; under it, only while it is fresh; never read, it refuses.
fn weigh(
    which: &'static str,
    figure: Option<&Figure>,
    mark: u8,
    now: DateTime<Utc>,
) -> Option<Refusal> {
    let Some(figure) = figure else {
        return Some(Refusal::Unread(which));
    };
    let minutes = (now - figure.at).num_minutes();
    if !figure.limit.before_reset(now) {
        return Some(Refusal::Stale {
            which,
            minutes,
            reset: true,
        });
    }
    if figure.limit.percent >= f64::from(mark) {
        return Some(Refusal::Usage {
            which,
            percent: figure.limit.percent,
            mark,
            until: figure.limit.resets_at,
        });
    }
    (now - figure.at > TRUSTED_FOR).then_some(Refusal::Stale {
        which,
        minutes,
        reset: false,
    })
}

/// The automatic pause as it stands: each limit at or over the mark until its reset.
pub fn usage_pause(figures: &Figures, mark: u8, now: DateTime<Utc>) -> Vec<Refusal> {
    [("weekly", &figures.weekly), ("Fable", &figures.fable)]
        .into_iter()
        .filter_map(|(which, figure)| weigh(which, figure.as_ref(), mark, now))
        .filter(|refusal| matches!(refusal, Refusal::Usage { .. }))
        .collect()
}

/// The settings files whose allow rules let a run's commands through: the project's two and the
/// user's.
pub fn settings_files(folder: &Path, home: Option<&Path>) -> Vec<PathBuf> {
    let mut files = vec![
        folder.join(".claude").join("settings.json"),
        folder.join(".claude").join("settings.local.json"),
    ];
    files.extend(home.map(|home| home.join(".claude").join("settings.json")));
    files
}

/// The allow rules in a settings file's text: the entries of `permissions.allow`.
pub fn allow_rules(text: &str) -> usize {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|value| Some(value.get("permissions")?.get("allow")?.as_array()?.len()))
        .unwrap_or(0)
}

/// The mode, when a run in `mode` would wait for the owner at each shell command: a mode of
/// [`WAITING_MODES`] with no allow rule in any of `files`.
pub fn would_wait(mode: &str, files: &[PathBuf]) -> Option<String> {
    let mode = mode.trim();
    if !WAITING_MODES
        .iter()
        .any(|waiting| waiting.eq_ignore_ascii_case(mode))
    {
        return None;
    }
    let allowed: usize = files
        .iter()
        .filter_map(|file| fs::read_to_string(file).ok())
        .map(|text| allow_rules(&text))
        .sum();
    (allowed == 0).then(|| mode.to_owned())
}

/// Whether `folder` is `repository`, the one the running window was built from: the tool's own
/// project, which is not chained (the owner's choice at EP-19). Until EP-37 the test was the
/// window's exe inside the folder; the window now runs from its installed copy, outside it.
pub fn is_own(repository: &Path, folder: &Path) -> bool {
    paths::same(repository, folder)
}

/// The gate's conditions that stand between runs (D-22; EP-19b), for the window's Autopilot
/// section, its offer of `auto` and `crw autopilot`, in the gate's order: [`Refusal::Waits`] when
/// the project's runs would wait for the owner at each shell command (its permission mode, over
/// the project's settings files and the user's), then [`Refusal::OwnProject`] when `folder` is
/// `repository`, the one the running window was built from. `home` is the user's profile folder;
/// both are none in a screenshot run.
pub fn standing(
    project: &Project,
    folder: &Path,
    home: Option<&Path>,
    repository: Option<&Path>,
) -> Vec<Refusal> {
    let mut no = Vec::new();
    let mode = rules::permission_mode(project).value;
    if let Some(mode) = would_wait(&mode, &settings_files(folder, home)) {
        no.push(Refusal::Waits(mode));
    }
    if repository.is_some_and(|repository| is_own(repository, folder)) {
        no.push(Refusal::OwnProject);
    }
    no
}

/// The chained starts the log records on `day` (local time): the project's and every project's.
pub fn chained_on(events: &[EventRow], project: &str, day: NaiveDate) -> (u32, u32) {
    let (mut mine, mut all) = (0, 0);
    for event in events {
        let chained = event.kind == EVENT_KIND
            && event
                .detail
                .as_deref()
                .is_some_and(|detail| detail.starts_with(CHAINED));
        let today = event
            .at()
            .is_some_and(|at| at.with_timezone(&Local).date_naive() == day);
        if chained && today {
            all += 1;
            if event
                .project
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case(project))
            {
                mine += 1;
            }
        }
    }
    (mine, all)
}

/// A chained start, as the log says it.
pub fn chained_words(next: EpId, after: Option<EpId>, chosen: &str, cap: &Cap) -> String {
    let after = after.map_or(String::new(), |ep| format!(" after {ep}"));
    format!("{CHAINED}{next}{after}: {chosen}, {}", cap.words())
}

/// A refused chain, as the log says it: held until the pause lifts, or ended.
pub fn refused_words(after: Option<EpId>, refusals: &[Refusal]) -> String {
    let after = after.map_or(String::new(), |ep| format!(" after {ep}"));
    let why: Vec<String> = refusals.iter().map(Refusal::words).collect();
    if holds(refusals) {
        format!("held{after} until the pause lifts: {}", why.join("; "))
    } else {
        format!("the chain ends{after}: {}", why.join("; "))
    }
}

/// A run stopped at its cap, as the log says it.
pub fn capped_words(ep: &str, cap: &Cap, worked: Duration) -> String {
    format!(
        "capped {ep}: stopped after {} of work, {}; the chain ends there",
        length(worked),
        cap.words()
    )
}

/// A chained start the runner refused, as the log says it.
pub fn not_started_words(next: &str, after: Option<EpId>, why: &str) -> String {
    let after = after.map_or(String::new(), |ep| format!(" after {ep}"));
    format!("the chain ends{after}: {next} did not start: {why}")
}

/// One brief a chain may run after the launched one (EP-19a): its row, the rule table's
/// resolution over it as the pickers' default, and what the owner made of it.
#[derive(Debug, Clone, PartialEq)]
pub struct Planned {
    pub ep: EpId,
    pub title: String,
    pub size: Option<Size>,
    /// The rules' resolution over its row when the plan was made, the downshift included.
    pub resolution: Resolution,
    /// The open owner tasks gated before it, by id, which the gate would refuse it for.
    pub due: Vec<String>,
    /// Whether the chain may run it (the Chain box); unticked, the chain ends before it.
    pub chain: bool,
    /// The owner's choice, once they changed a picker: the chain takes it as shown.
    pub picked: Option<Picked>,
}

impl Planned {
    /// The model, effort and ultracode as the pickers show them.
    pub fn shown(&self) -> Picked {
        self.picked
            .clone()
            .unwrap_or_else(|| Picked::of(&self.resolution))
    }

    /// The brief's line of the plan in words: `EP-5 claude-opus-5-5 at max (your choice)`.
    pub fn words(&self) -> String {
        let whose = match (&self.picked, self.chain) {
            (_, false) => return format!("{} left out", self.ep),
            (Some(_), true) => "your choice",
            (None, true) => "the rules' choice",
        };
        let picked = self.shown();
        format!(
            "{} {} at {} ({whose})",
            self.ep,
            picked.model,
            prompt::effort_words(picked.effort.as_deref(), picked.ultracode)
        )
    }
}

/// Where a plan stops of its own: a row the gate would refuse whatever the owner does, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stop {
    pub ep: EpId,
    pub why: String,
}

/// What the plan says of a brief the chain reaches.
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    /// The owner's choice.
    Picked(Picked),
    /// A row the owner left alone: the rules' choice as the chain resolves it then.
    Rules,
    /// A row the owner unticked: the chain ends before it.
    LeftOut,
    /// A brief the plan does not list (a row added since): the rules' choice.
    NotListed,
}

/// The plan of a chain (EP-19a, the owner's choice: kept in the window from Start headless
/// until the chain ends): the briefs it may run after `after`, in table order, up to the
/// project's `daily`.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub project: String,
    /// The brief the owner starts, which the chain follows.
    pub after: EpId,
    pub briefs: Vec<Planned>,
    /// The row the list stopped at, when the gate would refuse it on its own account.
    pub stop: Option<Stop>,
    /// The project's chained runs a day: the most briefs listed.
    pub daily: u32,
}

impl Plan {
    /// What the plan says of `ep`.
    pub fn answer(&self, ep: EpId) -> Answer {
        match self.briefs.iter().find(|planned| planned.ep == ep) {
            None => Answer::NotListed,
            Some(planned) if !planned.chain => Answer::LeftOut,
            Some(planned) => match &planned.picked {
                Some(picked) => Answer::Picked(picked.clone()),
                None => Answer::Rules,
            },
        }
    }

    /// The plan in words: `after EP-4: EP-5 claude-opus-5-5 at max (your choice), EP-6 left
    /// out; the chain would end at EP-7: EP-7 is a re-plan, the owner's to run`.
    pub fn words(&self) -> String {
        let briefs = if self.briefs.is_empty() {
            "no brief to chain".to_owned()
        } else {
            self.briefs
                .iter()
                .map(Planned::words)
                .collect::<Vec<_>>()
                .join(", ")
        };
        let stop = self.stop.as_ref().map_or(String::new(), |stop| {
            format!("; the chain would end at {}: {}", stop.ep, stop.why)
        });
        format!("after {}: {briefs}{stop}", self.after)
    }

    /// A plan made again after a keep (EP-19c) takes each row's Chain box and the owner's choice
    /// from `before`, by EP id; a row left alone keeps its new resolution, and so does a row
    /// `before` did not list.
    pub fn keep_choices(&mut self, before: &Plan) {
        for planned in &mut self.briefs {
            if let Some(old) = before.briefs.iter().find(|old| old.ep == planned.ep) {
                planned.chain = old.chain;
                planned.picked = old.picked.clone();
            }
        }
    }
}

/// The plan of a chain after `after` (EP-19a): the next unchecked rows in table order, up to the
/// project's `daily`, each with the rules' resolution over its row (`levels` and `usage` as
/// [`read`] takes them); the list stops at a row the gate would refuse on its own account (a
/// re-plan, a size over `max_size`, no size), named with the reason.
pub fn plan(
    registry: &Registry,
    project: &Project,
    snapshot: &snapshot::Snapshot,
    after: EpId,
    levels: &[String],
    usage: &rules::Usage,
) -> Plan {
    let daily = project.autopilot.daily();
    let max = project.autopilot.max_size();
    let mut plan = Plan {
        project: project.name.clone(),
        after,
        briefs: Vec::new(),
        stop: None,
        daily,
    };
    let Some(roadmap) = snapshot.current() else {
        return plan;
    };
    let rows: Vec<&Row> = roadmap.rows().collect();
    let Some(at) = rows.iter().position(|row| row.id == after) else {
        return plan;
    };
    for row in rows
        .iter()
        .skip(at + 1)
        .filter(|row| !row.done.is_checked())
    {
        if plan.briefs.len() >= daily as usize {
            break;
        }
        let refused = if is_replan(&row.brief.title) {
            Some(Refusal::Replan(row.id))
        } else {
            match row.size {
                Some(size) if size <= max => None,
                size => Some(Refusal::Size {
                    next: row.id,
                    size,
                    max,
                }),
            }
        };
        if let Some(refused) = refused {
            plan.stop = Some(Stop {
                ep: row.id,
                why: refused.words(),
            });
            break;
        }
        let phase = roadmap
            .phase_of(row.id)
            .and_then(|phase| phase.heading.id.as_deref());
        let brief = rules::Brief {
            size: row.size,
            phase,
            title: &row.brief.title,
        };
        plan.briefs.push(Planned {
            ep: row.id,
            title: row.brief.title.trim().to_owned(),
            size: row.size,
            resolution: rules::resolve(registry, project, &brief, levels, usage),
            due: prompt::due(snapshot, roadmap, row),
            chain: true,
            picked: None,
        });
    }
    plan
}

/// A brief the owner left out of the plan, which ends the chain, as the log says it.
pub fn left_out_words(after: Option<EpId>, ep: EpId) -> String {
    let after = after.map_or(String::new(), |ep| format!(" after {ep}"));
    format!("the chain ends{after}: {ep} is left out of your plan")
}

/// Keeps one line of the chain log.
pub fn log(
    ledger: &Path,
    project: Option<&str>,
    detail: &str,
    at: DateTime<Utc>,
) -> Result<(), String> {
    Ledger::open(ledger)
        .and_then(|ledger| ledger.add_event(at, EVENT_KIND, project, detail))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// The chain log's latest `limit` lines, the newest first, read without a write.
pub fn recent(ledger: &Path, limit: usize) -> Result<Vec<EventRow>, String> {
    if !ledger.exists() {
        return Ok(Vec::new());
    }
    Ledger::open_read_only(ledger)
        .and_then(|ledger| ledger.events(EVENT_KIND, Some(limit)))
        .map_err(|e| e.to_string())
}

/// What the gate is read from after a run of `project`.
pub struct Ask<'a> {
    pub registry: &'a Registry,
    pub project: &'a Project,
    pub folder: &'a Path,
    pub ran: Option<EpId>,
    pub outcome: Outcome,
    pub figures: &'a Figures,
    /// The CLI's effort levels in order, for the rule table's steps.
    pub levels: &'a [String],
    pub now: DateTime<Utc>,
    /// The repository the running window was built from (`paths::repository`).
    pub repository: Option<&'a Path>,
    /// The user's profile folder, for its settings file.
    pub home: Option<&'a Path>,
    pub ledger: Option<&'a Path>,
    /// The other sessions open in the project, in words.
    pub others: Vec<String>,
}

/// The gate's facts read afresh, and the next brief's draft when there is one.
pub struct Reading {
    pub facts: Facts,
    pub draft: Option<Draft>,
}

/// Reads the facts after a run (git: two children, one at a time): the project's files and
/// git's status, the ledger's day, the settings files; and drafts the next brief as the dialog
/// would, with the commit variant when the autopilot is on.
pub fn read(ask: &Ask<'_>) -> Reading {
    let mut snapshot = snapshot::read_tree(ask.folder);
    let tree = match snapshot::read_git(ask.folder) {
        Some(mut reading) => {
            let changes = reading
                .worktree
                .take()
                .map(|status| {
                    status
                        .changes
                        .into_iter()
                        .map(|change| change.path)
                        .collect()
                })
                .ok_or_else(|| "git did not answer".to_owned());
            reading.fill(&mut snapshot);
            changes
        }
        None => Err("the folder is not a git repository".to_owned()),
    };
    let completed = ask.ran.is_some_and(|ran| {
        snapshot
            .entry(ran)
            .is_some_and(snapshot::Entry::is_complete)
    });
    let due = prompt::next_of(&snapshot)
        .map(|(roadmap, row)| prompt::due(&snapshot, roadmap, row))
        .unwrap_or_default();
    let usage = rules::Usage::of(ask.figures, ask.now);
    let claude_md = prompt::claude_md(ask.folder);
    let date = ask.now.with_timezone(&Local).format("%Y-%m-%d").to_string();
    let draft = prompt::draft(
        ask.registry,
        ask.project,
        &snapshot,
        claude_md.as_deref(),
        ask.levels,
        &usage,
        &date,
    );
    let next = draft.as_ref().map(|draft| Next {
        id: draft.ep,
        title: draft.title.clone(),
        size: draft.size,
    });
    let mode = match &draft {
        Ok(draft) => draft.resolution.permission_mode.value.clone(),
        Err(_) => rules::permission_mode(ask.project).value,
    };
    let day = ask.now.with_timezone(&Local).date_naive();
    let (chained_today, chained_total) = ask
        .ledger
        .and_then(|ledger| recent(ledger, 500).ok())
        .map_or((0, 0), |events| chained_on(&events, &ask.project.name, day));
    let settings = &ask.registry.settings;
    let facts = Facts {
        enabled: ask.project.autopilot.enabled,
        paused: settings.autopilot_paused(),
        outcome: ask.outcome,
        ran: ask.ran,
        completed,
        next: next.map_err(Clone::clone),
        due,
        max_size: ask.project.autopilot.max_size(),
        weekly: ask.figures.weekly.clone(),
        fable: ask.figures.fable.clone(),
        pause_percent: settings.autopilot_pause_percent(),
        tree,
        chained_today,
        daily: ask.project.autopilot.daily(),
        chained_total,
        daily_total: settings.autopilot_daily_total(),
        others: ask.others.clone(),
        own: ask
            .repository
            .is_some_and(|repository| is_own(repository, ask.folder)),
        waits: would_wait(&mode, &settings_files(ask.folder, ask.home)),
    };
    Reading {
        facts,
        draft: draft.ok(),
    }
}

/// The headless launch of a chained brief (D-22): the draft's model, effort, ultracode and
/// permission mode as the launch dialog takes them, with the project's neighbours, time limit
/// and budget, and the push guard, since a chained run is always of a project whose autopilot is
/// on (EP-19b). The caller adds the route, the hooks and the revert's start, as a headless start
/// from the dialog does.
pub fn launch(
    draft: &Draft,
    project: &Project,
    folder: &Path,
    data_root: &Path,
    ledger: Option<PathBuf>,
) -> Launch {
    let resolution = &draft.resolution;
    let mut launch = Launch::new(
        &project.name,
        folder,
        &resolution.model.value,
        &draft.prompt.text,
    );
    launch.ep = Some(draft.ep.to_string());
    launch.effort = resolution.effort.value.clone();
    launch.ultracode = resolution.ultracode.value;
    launch.permission_mode = resolution.permission_mode.value.clone();
    launch.neighbours = project.neighbour_folders(data_root);
    launch.ledger = ledger;
    launch.answer_timeout = project.answer_timeout();
    launch.max_budget_usd = project.autopilot.max_budget_usd;
    launch.deny_push = true;
    launch
}

/// Changes of the working tree, by path, from git's status.
pub fn changes(status: &git::Status) -> Vec<String> {
    status
        .changes
        .iter()
        .map(|change| change.path.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;
    use crate::usage::{Limit, Source};
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 6, 12, 0, 0).unwrap()
    }

    fn figure(percent: f64, minutes_ago: i64, resets_in_hours: i64) -> Figure {
        Figure {
            limit: Limit {
                percent,
                resets_at: Some(now() + TimeDelta::hours(resets_in_hours)),
                active: Some(true),
                severity: None,
            },
            source: Source::Endpoint,
            at: now() - TimeDelta::minutes(minutes_ago),
        }
    }

    fn ep(text: &str) -> EpId {
        text.parse().unwrap()
    }

    /// A change to the passing facts, and the one refusal it brings.
    type Case = (Box<dyn Fn(&mut Facts)>, Refusal);

    fn passing() -> Facts {
        Facts {
            enabled: true,
            paused: false,
            outcome: Outcome::Completed,
            ran: Some(ep("EP-4")),
            completed: true,
            next: Ok(Next {
                id: ep("EP-5"),
                title: "Owner tasks".to_owned(),
                size: Some(Size::M),
            }),
            due: Vec::new(),
            max_size: Size::M,
            weekly: Some(figure(40.0, 3, 48)),
            fable: Some(figure(30.0, 3, 48)),
            pause_percent: 90,
            tree: Ok(Vec::new()),
            chained_today: 1,
            daily: 2,
            chained_total: 3,
            daily_total: 4,
            others: Vec::new(),
            own: false,
            waits: None,
        }
    }

    #[test]
    fn the_gate_passes_when_every_condition_holds_and_names_each_that_fails() {
        assert_eq!(gate(&passing(), now()), Vec::new());
        let next = |id: &str, title: &str, size: Option<Size>| {
            Ok(Next {
                id: ep(id),
                title: title.to_owned(),
                size,
            })
        };
        let cases: Vec<Case> = vec![
            (Box::new(|f: &mut Facts| f.enabled = false), Refusal::Off),
            (Box::new(|f: &mut Facts| f.paused = true), Refusal::Paused),
            (
                Box::new(|f: &mut Facts| f.outcome = Outcome::Deferred),
                Refusal::Outcome(Outcome::Deferred),
            ),
            (Box::new(|f: &mut Facts| f.ran = None), Refusal::NoEp),
            (
                Box::new(|f: &mut Facts| f.completed = false),
                Refusal::NoCompletionNote(ep("EP-4")),
            ),
            (
                Box::new(move |f: &mut Facts| f.next = next("EP-4", "Parser B", Some(Size::M))),
                Refusal::RowOpen(ep("EP-4")),
            ),
            (
                Box::new(|f: &mut Facts| {
                    f.next = Err("every row of its roadmap is checked".to_owned())
                }),
                Refusal::NoNext("every row of its roadmap is checked".to_owned()),
            ),
            (
                Box::new(|f: &mut Facts| f.due = vec!["OT-3".to_owned()]),
                Refusal::Due {
                    next: ep("EP-5"),
                    tasks: vec!["OT-3".to_owned()],
                },
            ),
            (
                Box::new(move |f: &mut Facts| f.next = next("EP-5", "Re-plan P1", Some(Size::S))),
                Refusal::Replan(ep("EP-5")),
            ),
            (
                Box::new(move |f: &mut Facts| f.next = next("EP-5", "The engine", Some(Size::L))),
                Refusal::Size {
                    next: ep("EP-5"),
                    size: Some(Size::L),
                    max: Size::M,
                },
            ),
            (
                Box::new(move |f: &mut Facts| f.next = next("EP-5", "The engine", None)),
                Refusal::Size {
                    next: ep("EP-5"),
                    size: None,
                    max: Size::M,
                },
            ),
            (
                Box::new(|f: &mut Facts| f.tree = Ok(vec!["src/main.rs".to_owned()])),
                Refusal::Dirty(vec!["src/main.rs".to_owned()]),
            ),
            (
                Box::new(|f: &mut Facts| f.tree = Err("git is missing".to_owned())),
                Refusal::GitUnknown("git is missing".to_owned()),
            ),
            (
                Box::new(|f: &mut Facts| f.chained_today = 2),
                Refusal::DailyProject { count: 2, cap: 2 },
            ),
            (
                Box::new(|f: &mut Facts| f.chained_total = 4),
                Refusal::DailyTotal { count: 4, cap: 4 },
            ),
            (
                Box::new(|f: &mut Facts| f.others = vec!["a visual session".to_owned()]),
                Refusal::Busy(vec!["a visual session".to_owned()]),
            ),
            (Box::new(|f: &mut Facts| f.own = true), Refusal::OwnProject),
            (
                Box::new(|f: &mut Facts| f.waits = Some("acceptEdits".to_owned())),
                Refusal::Waits("acceptEdits".to_owned()),
            ),
        ];
        for (change, expected) in cases {
            let mut facts = passing();
            change(&mut facts);
            assert_eq!(gate(&facts, now()), vec![expected.clone()], "{expected:?}");
            assert!(!expected.words().is_empty());
        }
        // Several at once are all named, in order.
        let mut facts = passing();
        facts.enabled = false;
        facts.own = true;
        assert_eq!(gate(&facts, now()), vec![Refusal::Off, Refusal::OwnProject]);
        // An S brief passes under M, and an L one under an L limit.
        let mut facts = passing();
        facts.next = next("EP-5", "Small", Some(Size::S));
        assert!(gate(&facts, now()).is_empty());
        facts.next = next("EP-5", "Large", Some(Size::L));
        facts.max_size = Size::L;
        assert!(gate(&facts, now()).is_empty());
    }

    #[test]
    fn a_figure_at_the_mark_pauses_until_its_reset_and_an_old_one_under_it_holds() {
        let weigh_fable = |figure: Option<Figure>| {
            let mut facts = passing();
            facts.fable = figure;
            gate(&facts, now())
        };
        // At or over the mark: the pause, however old the reading.
        let over = weigh_fable(Some(figure(90.0, 300, 20)));
        assert!(matches!(
            over.as_slice(),
            [Refusal::Usage {
                which: "Fable",
                mark: 90,
                ..
            }]
        ));
        assert!(holds(&over));
        assert!(
            over[0].words().contains("until its reset"),
            "{}",
            over[0].words()
        );
        // Under it and fresh: it counts; under it and old: held.
        assert!(weigh_fable(Some(figure(89.0, 15, 20))).is_empty());
        let old = weigh_fable(Some(figure(50.0, 16, 20)));
        assert_eq!(
            old,
            vec![Refusal::Stale {
                which: "Fable",
                minutes: 16,
                reset: false
            }]
        );
        assert!(holds(&old));
        // Read before its window reset: held until a new reading.
        let reset = weigh_fable(Some(figure(95.0, 10, -1)));
        assert!(matches!(
            reset.as_slice(),
            [Refusal::Stale { reset: true, .. }]
        ));
        assert!(holds(&reset));
        // Never read: refused, and the chain ends.
        let unread = weigh_fable(None);
        assert_eq!(unread, vec![Refusal::Unread("Fable")]);
        assert!(!holds(&unread));
        // A pause with another refusal ends the chain.
        let mut facts = passing();
        facts.weekly = Some(figure(92.0, 1, 5));
        facts.tree = Ok(vec!["a".to_owned()]);
        assert!(!holds(&gate(&facts, now())));
        // The automatic pause alone, for the window's state.
        let figures = Figures {
            weekly: Some(figure(92.0, 1, 5)),
            fable: Some(figure(40.0, 30, 5)),
            ..Figures::default()
        };
        let pause = usage_pause(&figures, 90, now());
        assert!(matches!(
            pause.as_slice(),
            [Refusal::Usage {
                which: "weekly",
                ..
            }]
        ));
        assert!(usage_pause(&Figures::default(), 90, now()).is_empty());
    }

    #[test]
    fn a_run_waits_at_each_command_without_an_allow_rule_in_an_asking_mode() {
        let scratch = Scratch::new("autopilot-waits");
        let folder = scratch.0.join("alpha");
        let home = scratch.0.join("home");
        fs::create_dir_all(folder.join(".claude")).unwrap();
        fs::create_dir_all(home.join(".claude")).unwrap();
        let files = settings_files(&folder, Some(&home));
        assert_eq!(files.len(), 3);
        assert_eq!(
            would_wait("acceptEdits", &files).as_deref(),
            Some("acceptEdits")
        );
        assert_eq!(would_wait(" manual ", &files).as_deref(), Some("manual"));
        assert_eq!(would_wait("auto", &files), None);
        assert_eq!(would_wait("dontAsk", &files), None);
        assert_eq!(would_wait("bypassPermissions", &files), None);
        // A settings file with no allow rule, or one that does not read, changes nothing.
        fs::write(
            &files[0],
            r#"{"permissions": {"deny": ["Edit(//c/x/**)"]}}"#,
        )
        .unwrap();
        fs::write(&files[2], "not json").unwrap();
        assert_eq!(
            would_wait("acceptEdits", &files).as_deref(),
            Some("acceptEdits")
        );
        // An allow rule in any of them lets the commands through.
        fs::write(
            &files[1],
            r#"{"permissions": {"allow": ["Bash(cargo test:*)"]}}"#,
        )
        .unwrap();
        assert_eq!(would_wait("acceptEdits", &files), None);
        assert_eq!(allow_rules(r#"{"permissions":{"allow":["a","b"]}}"#), 2);
        assert_eq!(allow_rules("{}"), 0);
    }

    /// The tool's own project is the repository the window was built from, wherever its exe runs
    /// (EP-37: from the installed copy).
    #[test]
    fn the_tools_own_project_is_the_repository_the_window_was_built_from() {
        let folder = Path::new(r"C:\Data\Wrangler");
        assert!(is_own(Path::new(r"c:\data\wrangler"), folder));
        assert!(is_own(Path::new("C:/Data/Wrangler/"), folder));
        assert!(!is_own(Path::new(r"C:\Data\Wrangler2"), folder));
        assert!(!is_own(Path::new(r"C:\Data"), folder));
        assert!(!is_own(Path::new(r"C:\Data\Wrangler\crates"), folder));
    }

    /// The conditions that stand between runs (EP-19b): a waiting mode with no allow rule, then
    /// an allow rule, then `auto`; the tool's own project.
    #[test]
    fn the_standing_conditions_are_a_waiting_mode_and_the_tools_own_project() {
        let scratch = Scratch::new("autopilot-standing");
        let folder = scratch.0.join("alpha");
        let home = scratch.0.join("home");
        fs::create_dir_all(&folder).unwrap();
        let mut project = Project::new("alpha", Path::new("alpha"), crate::registry::State::Active);
        // No settings file anywhere: the tool's default waits.
        assert_eq!(
            standing(&project, &folder, Some(&home), None),
            vec![Refusal::Waits("acceptEdits".to_owned())]
        );
        // An allow rule in the project's settings lets its commands through.
        let settings = folder.join(".claude").join("settings.json");
        fs::create_dir_all(settings.parent().unwrap()).unwrap();
        fs::write(
            &settings,
            r#"{"permissions": {"allow": ["Bash(cargo test:*)"]}}"#,
        )
        .unwrap();
        assert_eq!(standing(&project, &folder, Some(&home), None), Vec::new());
        fs::remove_file(&settings).unwrap();
        // Under auto nothing waits; under plan, with no rule, it does.
        project.launch.permission_mode = Some("auto".to_owned());
        assert_eq!(standing(&project, &folder, Some(&home), None), Vec::new());
        project.launch.permission_mode = Some("plan".to_owned());
        assert_eq!(
            standing(&project, &folder, None, None),
            vec![Refusal::Waits("plan".to_owned())]
        );
        // The window built from the folder: the tool's own project, after the waiting.
        assert_eq!(
            standing(&project, &folder, None, Some(&folder)),
            vec![Refusal::Waits("plan".to_owned()), Refusal::OwnProject]
        );
        project.launch.permission_mode = None;
        fs::write(
            &settings,
            r#"{"permissions": {"allow": ["Bash(git status)"]}}"#,
        )
        .unwrap();
        assert_eq!(
            standing(&project, &folder, None, Some(&folder)),
            vec![Refusal::OwnProject]
        );
        let elsewhere = scratch.0.join("beta");
        assert_eq!(
            standing(&project, &folder, None, Some(&elsewhere)),
            Vec::new()
        );
    }

    /// A chained run always has the push guard: its project's autopilot is on (EP-19b).
    #[test]
    fn a_chained_launch_has_the_push_guard() {
        let (registry, snapshot) = house();
        let project = &registry.projects[0];
        let unread = rules::Usage::unread(now());
        let draft = prompt::draft(
            &registry,
            project,
            &snapshot,
            None,
            &[],
            &unread,
            "2026-11-06",
        )
        .unwrap();
        let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("projects")
            .join("house");
        let launch = launch(&draft, project, &folder, folder.parent().unwrap(), None);
        assert!(launch.deny_push);
        assert_eq!(launch.ep.as_deref(), Some("EP-3"));
        assert_eq!(launch.permission_mode, rules::DEFAULT_PERMISSION_MODE);
    }

    #[test]
    fn the_days_chained_runs_are_counted_from_the_log() {
        let local = |day: u32, hour: u32| {
            Local
                .with_ymd_and_hms(2026, 11, day, hour, 30, 0)
                .unwrap()
                .with_timezone(&Utc)
                .to_rfc3339()
        };
        let event = |at: String, project: &str, detail: &str| EventRow {
            id: 0,
            at,
            kind: EVENT_KIND.to_owned(),
            project: Some(project.to_owned()),
            detail: Some(detail.to_owned()),
        };
        let events = vec![
            event(
                local(6, 9),
                "alpha",
                "chained EP-5 after EP-4: fable at xhigh, M's cap",
            ),
            event(
                local(6, 11),
                "Alpha",
                "chained EP-6 after EP-5: fable at xhigh, M's cap",
            ),
            event(
                local(6, 12),
                "beta",
                "chained EP-2 after EP-1: fable at high, S's cap",
            ),
            event(
                local(6, 13),
                "alpha",
                "the chain ends after EP-6: EP-7 is a re-plan",
            ),
            event(
                local(5, 23),
                "alpha",
                "chained EP-4 after EP-3: fable at xhigh, M's cap",
            ),
        ];
        let day = NaiveDate::from_ymd_opt(2026, 11, 6).unwrap();
        assert_eq!(chained_on(&events, "alpha", day), (2, 3));
        assert_eq!(chained_on(&events, "gamma", day), (0, 3));
    }

    #[test]
    fn a_runs_cap_counts_its_working_time_only() {
        assert_eq!(cap(Some(Size::S)), Duration::from_secs(45 * 60));
        assert_eq!(cap(Some(Size::M)), Duration::from_secs(90 * 60));
        assert_eq!(cap(Some(Size::L)), Duration::from_secs(180 * 60));
        assert_eq!(cap(None), cap(Some(Size::L)));
        let start = Instant::now();
        let at = |minutes: u64| start + Duration::from_secs(minutes * 60);
        let mut cap = Cap::new(Some(Size::S), start);
        cap.clock.tick(false, at(10));
        cap.clock.tick(true, at(20));
        // Waiting from minute 20: the clock stands.
        assert_eq!(cap.clock.worked(at(50)), Duration::from_secs(20 * 60));
        assert!(!cap.reached(at(60)));
        cap.clock.tick(false, at(60));
        assert_eq!(cap.clock.worked(at(60)), Duration::from_secs(20 * 60));
        assert!(!cap.reached(at(84)));
        assert!(cap.reached(at(85)));
        assert_eq!(cap.words(), "S's cap of 45 min");
        let words = capped_words("EP-5", &cap, cap.clock.worked(at(85)));
        assert!(
            words.starts_with("capped EP-5: stopped after 45 min"),
            "{words}"
        );
    }

    /// The house fixture (`crates/core/fixtures/projects/house`): EP-3 next, then EP-4 (L) and
    /// EP-5 (a re-plan), OT-2 due before EP-3.
    fn house() -> (Registry, snapshot::Snapshot) {
        let registry: Registry = toml::from_str(
            "format = 1\n\n[[project]]\nname = \"house\"\npath = \"house\"\nstate = \"active\"\n\
             autopilot = { enabled = true, max_size = \"L\", daily = 3 }\n",
        )
        .unwrap();
        let snapshot = snapshot::read_tree(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("fixtures")
                .join("projects")
                .join("house"),
        );
        (registry, snapshot)
    }

    #[test]
    fn the_plan_lists_the_rows_after_the_launched_brief_and_stops_where_the_gate_would() {
        let (registry, snapshot) = house();
        let project = &registry.projects[0];
        let unread = rules::Usage::unread(now());
        // After EP-3: EP-4 (L, within the project's L) and then EP-5, a re-plan, where it stops.
        let plan = super::plan(&registry, project, &snapshot, ep("EP-3"), &[], &unread);
        assert_eq!(plan.after, ep("EP-3"));
        assert_eq!(plan.daily, 3);
        assert_eq!(plan.briefs.len(), 1);
        let viewer = &plan.briefs[0];
        assert_eq!((viewer.ep, viewer.title.as_str()), (ep("EP-4"), "Viewer"));
        assert_eq!(viewer.size, Some(Size::L));
        assert_eq!(viewer.resolution.model.value, "fable");
        assert_eq!(viewer.resolution.effort.value.as_deref(), Some("xhigh"));
        assert_eq!(viewer.due, vec!["OT-2".to_owned()]);
        assert!(viewer.chain && viewer.picked.is_none());
        assert_eq!(
            plan.stop,
            Some(Stop {
                ep: ep("EP-5"),
                why: "EP-5 is a re-plan, the owner's to run".to_owned()
            })
        );
        assert_eq!(
            plan.words(),
            "after EP-3: EP-4 fable at xhigh (the rules' choice); the chain would end at EP-5: \
             EP-5 is a re-plan, the owner's to run"
        );
        // The plan's answers: the rules', the owner's, left out, not listed.
        assert_eq!(plan.answer(ep("EP-4")), Answer::Rules);
        assert_eq!(plan.answer(ep("EP-9")), Answer::NotListed);
        let mut planned = plan.clone();
        let picked = Picked {
            model: "claude-opus-5-5".to_owned(),
            effort: Some("max".to_owned()),
            ultracode: false,
        };
        planned.briefs[0].picked = Some(picked.clone());
        assert_eq!(planned.answer(ep("EP-4")), Answer::Picked(picked));
        assert_eq!(
            planned.briefs[0].words(),
            "EP-4 claude-opus-5-5 at max (your choice)"
        );
        planned.briefs[0].chain = false;
        assert_eq!(planned.answer(ep("EP-4")), Answer::LeftOut);
        assert_eq!(planned.briefs[0].words(), "EP-4 left out");
        assert_eq!(
            left_out_words(Some(ep("EP-3")), ep("EP-4")),
            "the chain ends after EP-3: EP-4 is left out of your plan"
        );
        // Under the default M, EP-4's L is where the list stops, so nothing is planned.
        let mut under_m = registry.clone();
        under_m.projects[0].autopilot.max_size = None;
        let plan = super::plan(
            &under_m,
            &under_m.projects[0],
            &snapshot,
            ep("EP-3"),
            &[],
            &unread,
        );
        assert!(plan.briefs.is_empty());
        assert!(
            plan.words()
                .starts_with("after EP-3: no brief to chain; the chain would end")
        );
        assert_eq!(
            plan.stop.map(|stop| stop.why),
            Some("EP-4 is L, over the autopilot's M".to_owned())
        );
        // A launched brief the roadmap does not list, or no roadmap: an empty plan.
        let plan = super::plan(&registry, project, &snapshot, ep("EP-9"), &[], &unread);
        assert!(plan.briefs.is_empty() && plan.stop.is_none());
        let none = plan_of(&registry, &snapshot::Snapshot::default());
        assert!(none.briefs.is_empty());
    }

    #[test]
    fn the_plan_keeps_to_the_daily_cap_and_the_rules_per_row() {
        // Four open rows after EP-1, the project's daily 2 and a rule for M rows.
        let registry: Registry = toml::from_str(
            "format = 1\n\n[[project]]\nname = \"alpha\"\npath = \"alpha\"\nstate = \"active\"\n\
             autopilot = { enabled = true, daily = 2 }\n\n[[project.rule]]\nname = \"an M on \
             Opus\"\nmatch = { size = \"M\" }\nset = { model = \"claude-opus-5-5\", effort = \
             \"max\" }\n",
        )
        .unwrap();
        let snapshot = snapshot::Snapshot {
            tree: Some(crate::roadmap::table::parse(
                "## Phase P1 — Build\n\n| # | Brief | Size | Depends on | Core | Done |\n\
                 |---|---|---|---|---|---|\n\
                 | EP-1 | One | S | — | core | ☐ |\n\
                 | EP-2 | Two | M | EP-1 | core | ☐ |\n\
                 | EP-3 | Three | S | EP-2 | core | ☑ `abc1234` |\n\
                 | EP-4 | Four | S | EP-2 | core | ☐ |\n\
                 | EP-5 | Five | M | EP-4 | core | ☐ |\n",
            )),
            ..snapshot::Snapshot::default()
        };
        let plan = plan_of(&registry, &snapshot);
        let ids: Vec<EpId> = plan.briefs.iter().map(|planned| planned.ep).collect();
        // The checked EP-3 is skipped; two rows, the daily.
        assert_eq!(ids, vec![ep("EP-2"), ep("EP-4")]);
        assert!(plan.stop.is_none());
        assert_eq!(
            plan.briefs[0].resolution.model.value, "claude-opus-5-5",
            "the project's rule over an M"
        );
        assert_eq!(
            plan.briefs[0].resolution.effort.value.as_deref(),
            Some("max")
        );
        // An S under the built-in rule: one step lower than xhigh.
        assert_eq!(plan.briefs[1].resolution.model.value, "fable");
        assert_eq!(
            plan.briefs[1].resolution.effort.value.as_deref(),
            Some("high")
        );
        assert_eq!(plan.briefs[1].shown().model, "fable");
        assert_eq!(
            plan.words(),
            "after EP-1: EP-2 claude-opus-5-5 at max (the rules' choice), EP-4 fable at high \
             (the rules' choice)"
        );
    }

    /// A plan made again after a keep (EP-19c): the owner's Chain boxes and choices carry over by
    /// EP id; a row they left alone shows the new resolution.
    #[test]
    fn a_plan_made_again_keeps_the_owners_choices_and_shows_the_new_resolution() {
        let registry = |rule: &str| -> Registry {
            toml::from_str(&format!(
                "format = 1\n\n[[project]]\nname = \"alpha\"\npath = \"alpha\"\n\
                 state = \"active\"\nautopilot = {{ enabled = true, daily = 3 }}\n{rule}"
            ))
            .unwrap()
        };
        let snapshot = snapshot::Snapshot {
            tree: Some(crate::roadmap::table::parse(
                "## Phase P1 — Build\n\n| # | Brief | Size | Depends on | Core | Done |\n\
                 |---|---|---|---|---|---|\n\
                 | EP-1 | One | S | — | core | ☐ |\n\
                 | EP-2 | Two | M | EP-1 | core | ☐ |\n\
                 | EP-3 | Three | M | EP-2 | core | ☐ |\n\
                 | EP-4 | Four | M | EP-3 | core | ☐ |\n",
            )),
            ..snapshot::Snapshot::default()
        };
        let mut before = plan_of(&registry(""), &snapshot);
        let picked = Picked {
            model: "claude-opus-5-5".to_owned(),
            effort: Some("max".to_owned()),
            ultracode: true,
        };
        before.briefs[0].picked = Some(picked.clone());
        before.briefs[1].chain = false;
        // The project now keeps low for every brief.
        let kept = registry(
            "\n[[project.rule]]\nname = \"kept for this project\"\nset = { effort = \"low\" }\n",
        );
        let mut again = plan_of(&kept, &snapshot);
        assert!(again.briefs.iter().all(|planned| planned.chain));
        again.keep_choices(&before);
        let rows: Vec<(EpId, bool, Option<Picked>)> = again
            .briefs
            .iter()
            .map(|planned| (planned.ep, planned.chain, planned.picked.clone()))
            .collect();
        assert_eq!(
            rows,
            vec![
                (ep("EP-2"), true, Some(picked)),
                (ep("EP-3"), false, None),
                (ep("EP-4"), true, None),
            ]
        );
        // Every row has the new resolution; the one left alone shows it.
        assert!(
            again
                .briefs
                .iter()
                .all(|planned| planned.resolution.effort.value.as_deref() == Some("low"))
        );
        assert_eq!(again.briefs[2].shown().effort.as_deref(), Some("low"));
        assert_eq!(
            again.words(),
            "after EP-1: EP-2 claude-opus-5-5 at ultracode (your choice), EP-3 left out, EP-4 \
             fable at low (the rules' choice)"
        );
        // A plan before that lists none of the rows leaves the new one as it is.
        let fresh = plan_of(&kept, &snapshot);
        let mut unchanged = fresh.clone();
        unchanged.keep_choices(&Plan {
            briefs: Vec::new(),
            ..before
        });
        assert_eq!(unchanged, fresh);
    }

    fn plan_of(registry: &Registry, snapshot: &snapshot::Snapshot) -> Plan {
        plan(
            registry,
            &registry.projects[0],
            snapshot,
            ep("EP-1"),
            &[],
            &rules::Usage::unread(now()),
        )
    }

    #[test]
    fn the_logs_words_say_what_ran_and_why_the_chain_stopped() {
        let cap = Cap::new(Some(Size::M), Instant::now());
        let words = chained_words(ep("EP-5"), Some(ep("EP-4")), "fable at xhigh", &cap);
        assert_eq!(
            words,
            "chained EP-5 after EP-4: fable at xhigh, M's cap of 1 h 30 min"
        );
        let ended = refused_words(Some(ep("EP-4")), &[Refusal::Replan(ep("EP-5"))]);
        assert!(
            ended.starts_with("the chain ends after EP-4: EP-5 is a re-plan"),
            "{ended}"
        );
        let held = refused_words(Some(ep("EP-4")), &[Refusal::Paused]);
        assert!(
            held.starts_with("held after EP-4 until the pause lifts"),
            "{held}"
        );
        assert!(not_started_words("EP-5", None, "claude is too old").contains("did not start"));
        assert!(is_replan("Re-plan P3") && is_replan("Replan and close") && !is_replan("Plans"));
    }

    #[test]
    fn the_log_is_kept_in_the_ledger_and_read_back() {
        let scratch = Scratch::new("autopilot-log");
        let path = scratch.0.join("ledger.sqlite");
        assert_eq!(recent(&path, 10).unwrap(), Vec::new());
        log(&path, Some("alpha"), "chained EP-5 after EP-4: x", now()).unwrap();
        log(&path, None, PAUSED_BY_OWNER, now() + TimeDelta::minutes(1)).unwrap();
        let rows = recent(&path, 10).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].detail.as_deref(), Some(PAUSED_BY_OWNER));
        assert_eq!(rows[1].project.as_deref(), Some("alpha"));
    }
}
