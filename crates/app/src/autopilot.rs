//! The autopilot in the window (D-22; EP-19). When a headless run of the window's ends and its
//! project's autopilot is on, or the chain started it, the gate is read on a worker thread
//! (`crw_core::autopilot::read`, git one child at a time) and logged. If it passes, the next
//! brief is drafted with the rules' model and effort, the downshift included, and waits for the
//! start slot; it then starts headless as the launch dialog starts one, with a cap. A refusal the
//! pause alone makes holds the chain, read again every five minutes and at the owner's Resume;
//! any other ends it. A run that works past its cap is stopped gracefully, which ends the chain.
//! The owner turns a project's autopilot on in the list, pauses every chain in the Sessions pane
//! or the tray, and reads the log at the top of the Sessions pane's All tab. The **chain plan**
//! (EP-19a) is the owner's word on the briefs a chain may run, made in the launch dialog and
//! kept here from Start headless until the chain ends: the gate takes the owner's choice of
//! model and effort for a brief they chose, the rules' for one they left alone, and ends the
//! chain before one they left out. Each change of a chain's state is a notification of its own
//! kind (the chain's moment). A screenshot run reads no gate and starts nothing.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Utc};
use crw_core::autopilot::{self, Answer, Ask, Cap, Plan, Refusal};
use crw_core::ledger::EventRow;
use crw_core::prompt::Draft;
use crw_core::registry::{Project, Registry};
use crw_core::roadmap::{EpId, Size};
use crw_core::session::{Launch, Outcome, cli};
use crw_core::{files, process};
use eframe::egui::{self, RichText};

use crate::board::{Action, Board};
use crate::sessions::{Run, Started};
use crate::style;
use crate::toasts::Toast;

/// How often a held chain's gate is read again: the meter's poll.
const HOLD_EVERY: Duration = Duration::from_secs(5 * 60);
/// How often the log and the registry's pause are read again while nothing changed them.
const READ_EVERY: Duration = Duration::from_secs(30);
/// The log's lines the Sessions pane shows: `crw autopilot` prints more.
const LOG_LINES: usize = 8;

/// A chain held by the pause alone (D-22), read again once the pause may have lifted.
#[derive(Debug, Clone, PartialEq)]
pub struct Held {
    pub project: String,
    /// The brief whose run completed.
    pub after: Option<EpId>,
    /// The log's words for the hold.
    pub words: String,
}

/// A chained start: waiting for the start slot, then on its way.
struct Queued {
    project: String,
    after: Option<EpId>,
    next: EpId,
    size: Option<Size>,
    launch: Launch,
    /// The effort in words, as the tab says it.
    effort: String,
    /// The model and effort in words, with the downshift's when it chose the model.
    chosen: String,
    program: PathBuf,
}

/// What a gate's worker brings back.
struct Gated {
    project: String,
    after: Option<EpId>,
    refusals: Vec<Refusal>,
    draft: Option<Draft>,
    entry: Project,
    folder: PathBuf,
    data_root: PathBuf,
    program: PathBuf,
}

/// The window's autopilot.
pub struct Pilot {
    gating: Vec<Receiver<Gated>>,
    /// The projects whose gate a worker reads, in lower case.
    reading: BTreeSet<String>,
    pub held: Vec<Held>,
    held_read: Option<Instant>,
    /// Read the held chains again at the next pass: the owner resumed.
    recheck: bool,
    queued: Option<Queued>,
    /// The chain's start on its way: the next start's result is the chain's.
    starting: Option<Queued>,
    /// The log's latest lines, the newest first.
    pub log: Vec<EventRow>,
    pub problem: Option<String>,
    read_at: Option<Instant>,
    /// The log or the registry's pause changed: read them again.
    pub stale: bool,
    /// The owner's Pause and the mark, as the registry last said them.
    pub paused: bool,
    pub mark: u8,
    /// A screenshot run's log: nothing is read or written.
    pub canned: bool,
    /// The chain plans (EP-19a), by project in lower case: each kept from Start headless until
    /// its chain ends.
    pub plans: BTreeMap<String, Plan>,
    /// The chain's moments since the last pass, for the popup.
    moments: Vec<Toast>,
}

impl Default for Pilot {
    /// Nothing read yet: not paused, the default mark.
    fn default() -> Self {
        Self {
            gating: Vec::new(),
            reading: BTreeSet::new(),
            held: Vec::new(),
            held_read: None,
            recheck: false,
            queued: None,
            starting: None,
            log: Vec::new(),
            problem: None,
            read_at: None,
            stale: false,
            paused: false,
            mark: autopilot::DEFAULT_PAUSE_PERCENT,
            canned: false,
            plans: BTreeMap::new(),
            moments: Vec::new(),
        }
    }
}

impl Pilot {
    /// Whether the chain's start is on its way, so that the next start's result is the chain's.
    pub fn is_starting(&self) -> bool {
        self.starting.is_some()
    }

    /// Keeps a project's chain plan (EP-19a), in place of any kept before.
    pub fn keep_plan(&mut self, plan: Plan) {
        self.plans.insert(plan.project.to_lowercase(), plan);
    }

    /// Lets a project's plan go: its chain ended, or its autopilot was turned off.
    pub fn drop_plan(&mut self, project: &str) {
        self.plans.remove(&project.to_lowercase());
    }

    /// A project's plan, while one is kept.
    pub fn plan_of(&self, project: &str) -> Option<&Plan> {
        self.plans.get(&project.to_lowercase())
    }

    /// The chain's moments since the last pass (EP-19a), each a toast of the chain's kind.
    pub fn take_moments(&mut self) -> Vec<Toast> {
        std::mem::take(&mut self.moments)
    }

    /// Whether a chained start waits for the slot or is on its way in `project`.
    pub fn is_starting_in(&self, project: &str) -> bool {
        [&self.queued, &self.starting]
            .into_iter()
            .flatten()
            .any(|queued| queued.project.eq_ignore_ascii_case(project))
    }
}

impl Board {
    /// The autopilot's share of each pass, the window shown or hidden: the caps, the gate after
    /// each run that ended, the gates read, the held chains, the chained start, and the log.
    pub fn autopilot_pass(&mut self) {
        for (key, words) in self.sessions.check_caps(Instant::now()) {
            let project = self
                .sessions
                .live
                .iter()
                .find(|live| live.key == key)
                .map(|live| live.project.clone());
            match project {
                Some(project) => self.chain_moment(&project, &words, true, true),
                None => self.log_chain(None, &words),
            }
        }
        for key in self.sessions.take_ended() {
            self.weigh_run(key);
        }
        self.take_gates();
        self.read_held();
        self.start_queued();
        self.read_pilot();
    }

    /// The gate after the window's run `key` ended, when its project's autopilot is on or the
    /// chain started it.
    fn weigh_run(&mut self, key: u64) {
        if self.fixtures {
            return;
        }
        let Some(live) = self.sessions.live.iter().find(|live| live.key == key) else {
            return;
        };
        let Some(outcome) = live.ended.as_ref().map(|ended| ended.outcome) else {
            return;
        };
        let (project, ran, chained) = (live.project.clone(), live.ep.parse().ok(), live.chained);
        let registry = match self.store().load() {
            Ok(loaded) => loaded.registry,
            Err(e) => {
                self.fail(format!("The autopilot could not read the registry: {e}"));
                return;
            }
        };
        let Some(entry) = registry.project(&project).cloned() else {
            return;
        };
        if !entry.autopilot.enabled && !chained {
            return;
        }
        self.read_gate(registry, entry, ran, outcome);
    }

    /// Reads the gate of `entry` after its run of `after` on a worker thread.
    fn read_gate(
        &mut self,
        registry: Registry,
        entry: Project,
        after: Option<EpId>,
        outcome: Outcome,
    ) {
        let Some(folder) = self
            .portfolio
            .project(&entry.name)
            .and_then(|row| row.folder.as_ref())
            .map(|folder| folder.path.clone())
        else {
            let words = format!(
                "the chain ends{}: the project's folder is missing",
                after.map_or(String::new(), |ep| format!(" after {ep}"))
            );
            self.chain_moment(&entry.name, &words, true, true);
            return;
        };
        if !self.autopilot.reading.insert(entry.name.to_lowercase()) {
            return;
        }
        let now = Utc::now();
        let figures = self.meter.gauge.figures(now);
        let program = cli::locate(registry.settings.claude_cli.as_deref())
            .unwrap_or_else(|| PathBuf::from("claude.exe"));
        let ctx = self.ctx().clone();
        let levels: Vec<String> = self
            .sessions
            .preflight(&program, &ctx)
            .and_then(|preflight| preflight.flags.as_ref().ok().map(|f| f.efforts.clone()))
            .unwrap_or_default();
        let mut others: Vec<String> = self
            .session_reasons(&entry.name)
            .iter()
            .map(ToString::to_string)
            .collect();
        if self.autopilot.is_starting_in(&entry.name) {
            others.push("a chained start is on its way".to_owned());
        }
        let exe = std::env::current_exe().ok();
        let home = std::env::var_os("USERPROFILE").map(PathBuf::from);
        let ledger = self.ledger().map(PathBuf::from);
        let data_root = registry.data_root();
        let (sender, receiver) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("autopilot gate".to_owned())
            .spawn(move || {
                let reading = {
                    // Git's children, one at a time with the window's other passes (D-29).
                    let _turn = process::turn();
                    autopilot::read(&Ask {
                        registry: &registry,
                        project: &entry,
                        folder: &folder,
                        ran: after,
                        outcome,
                        figures: &figures,
                        levels: &levels,
                        now,
                        exe: exe.as_deref(),
                        home: home.as_deref(),
                        ledger: ledger.as_deref(),
                        others,
                    })
                };
                let refusals = autopilot::gate(&reading.facts, now);
                let _ = sender.send(Gated {
                    project: entry.name.clone(),
                    after,
                    refusals,
                    draft: reading.draft,
                    entry,
                    folder,
                    data_root,
                    program,
                });
                ctx.request_repaint();
            });
        match spawned {
            Ok(_) => self.autopilot.gating.push(receiver),
            Err(e) => {
                self.autopilot.reading.clear();
                self.fail(format!(
                    "The autopilot's gate could not start a thread: {e}"
                ));
            }
        }
    }

    /// Takes in the gates the workers have read.
    fn take_gates(&mut self) {
        let mut done = Vec::new();
        self.autopilot
            .gating
            .retain(|receiver| match receiver.try_recv() {
                Ok(gated) => {
                    done.push(gated);
                    false
                }
                Err(TryRecvError::Empty) => true,
                Err(TryRecvError::Disconnected) => false,
            });
        if done.is_empty() && self.autopilot.gating.is_empty() {
            self.autopilot.reading.clear();
        }
        for gated in done {
            self.gated(gated);
        }
    }

    /// A gate read: the next brief queued for its start, the chain held, or ended.
    fn gated(&mut self, gated: Gated) {
        let Gated {
            project,
            after,
            refusals,
            draft,
            entry,
            folder,
            data_root,
            program,
        } = gated;
        self.autopilot.reading.remove(&project.to_lowercase());
        let was_held = self
            .autopilot
            .held
            .iter()
            .position(|held| held.project.eq_ignore_ascii_case(&project));
        if !refusals.is_empty() {
            let words = autopilot::refused_words(after, &refusals);
            if autopilot::holds(&refusals) {
                let unchanged = was_held.is_some_and(|at| self.autopilot.held[at].words == words);
                if let Some(at) = was_held {
                    self.autopilot.held.remove(at);
                }
                self.autopilot.held.push(Held {
                    project: project.clone(),
                    after,
                    words: words.clone(),
                });
                if !unchanged {
                    self.chain_moment(&project, &words, true, false);
                }
            } else {
                if let Some(at) = was_held {
                    self.autopilot.held.remove(at);
                }
                self.chain_moment(&project, &words, true, true);
            }
            return;
        }
        if let Some(at) = was_held {
            self.autopilot.held.remove(at);
        }
        let Some(mut draft) = draft else {
            let words = format!(
                "the chain ends{}: the next brief could not be drafted",
                after.map_or(String::new(), |ep| format!(" after {ep}"))
            );
            self.chain_moment(&project, &words, true, true);
            return;
        };
        // The plan's word on the brief (EP-19a): the owner's choice, the rules', or the end.
        let answer = self
            .autopilot
            .plan_of(&project)
            .map(|plan| plan.answer(draft.ep));
        let (picked, whose) = match answer {
            Some(Answer::LeftOut) => {
                let words = autopilot::left_out_words(after, draft.ep);
                self.chain_moment(&project, &words, true, true);
                return;
            }
            Some(Answer::Picked(picked)) => (Some(picked), "your choice"),
            Some(Answer::Rules) | None => (None, "the rules' choice"),
            Some(Answer::NotListed) => (None, "not in your plan, the rules' choice"),
        };
        // Headless whatever the project's launch mode, the owner's choice in place of the
        // rules' when they made one.
        draft.for_chain(picked.as_ref(), entry.launch.template.as_deref());
        let mut launch = autopilot::launch(
            &draft,
            &entry,
            &folder,
            &data_root,
            self.ledger().map(PathBuf::from),
        );
        // The project's start, for a revert, unless its switch is off (EP-15b).
        if entry.revert {
            launch.starts = Some(self.store().backups.clone());
        }
        launch.hooks = crate::board::crw_exe();
        let resolution = &draft.resolution;
        let mut chosen = format!(
            "{} at {}, {whose}",
            resolution.model.value, draft.parts.effort
        );
        if picked.is_none()
            && let Some(downshift) = resolution.downshift.as_ref().filter(|d| d.applied)
        {
            chosen.push_str(&format!(" (the downshift: {})", downshift.words));
        }
        self.autopilot.queued = Some(Queued {
            project,
            after,
            next: draft.ep,
            size: draft.size,
            effort: draft.parts.effort.clone(),
            chosen,
            launch,
            program,
        });
    }

    /// Reads the held chains' gates again when the owner resumed or every five minutes.
    fn read_held(&mut self) {
        if self.autopilot.held.is_empty() || self.fixtures {
            return;
        }
        let due = std::mem::take(&mut self.autopilot.recheck)
            || self
                .autopilot
                .held_read
                .is_none_or(|at| at.elapsed() >= HOLD_EVERY);
        if !due {
            return;
        }
        self.autopilot.held_read = Some(Instant::now());
        let Ok(loaded) = self.store().load() else {
            return;
        };
        let held: Vec<Held> = self.autopilot.held.clone();
        for held in held {
            match loaded.registry.project(&held.project).cloned() {
                Some(entry) => {
                    self.read_gate(
                        loaded.registry.clone(),
                        entry,
                        held.after,
                        Outcome::Completed,
                    );
                }
                None => self
                    .autopilot
                    .held
                    .retain(|h| !h.project.eq_ignore_ascii_case(&held.project)),
            }
        }
    }

    /// Starts the queued chained brief once the start slot is free and the CLI has answered;
    /// the registry is read again first, so that a Pause or an off since the gate counts.
    fn start_queued(&mut self) {
        if self.autopilot.queued.is_none() || self.sessions.is_starting() {
            return;
        }
        let registry = match self.store().load() {
            Ok(loaded) => loaded.registry,
            Err(e) => {
                self.fail(format!("The autopilot could not read the registry: {e}"));
                return;
            }
        };
        let queued = self.autopilot.queued.as_ref().expect("a start is queued");
        let enabled = registry
            .project(&queued.project)
            .is_some_and(|project| project.autopilot.enabled);
        if !enabled || registry.settings.autopilot_paused() {
            let queued = self.autopilot.queued.take().expect("a start is queued");
            let refusals = [if enabled {
                Refusal::Paused
            } else {
                Refusal::Off
            }];
            let words = autopilot::refused_words(queued.after, &refusals);
            if enabled {
                self.autopilot.held.push(Held {
                    project: queued.project.clone(),
                    after: queued.after,
                    words: words.clone(),
                });
            }
            self.chain_moment(&queued.project, &words, true, !enabled);
            return;
        }
        // A session opened in the project since the gate was read would edit the same files.
        let others: Vec<String> = self
            .session_reasons(&queued.project)
            .iter()
            .map(ToString::to_string)
            .collect();
        if !others.is_empty() {
            let queued = self.autopilot.queued.take().expect("a start is queued");
            let words = autopilot::refused_words(queued.after, &[Refusal::Busy(others)]);
            self.chain_moment(&queued.project, &words, true, true);
            return;
        }
        let ctx = self.ctx().clone();
        let program = queued.program.clone();
        let Some(preflight) = self.sessions.preflight(&program, &ctx) else {
            return;
        };
        let mut queued = self.autopilot.queued.take().expect("a start is queued");
        let refusals = preflight.refusals();
        if !refusals.is_empty() {
            let words = autopilot::not_started_words(
                &queued.next.to_string(),
                queued.after,
                &refusals.join("; "),
            );
            self.chain_moment(&queued.project, &words, true, true);
            return;
        }
        queued.launch.route = self
            .sessions
            .route_for(&preflight, registry.settings.decisions)
            .0;
        let run = Run {
            capped: true,
            size: queued.size,
            chained: true,
            commits: true,
        };
        self.sessions.start(
            preflight,
            queued.launch.clone(),
            queued.effort.clone(),
            run,
            &ctx,
        );
        self.autopilot.starting = Some(queued);
    }

    /// The chain's start came back: logged as chained, or the chain ends there.
    pub fn chain_started(&mut self, started: Started) {
        let Some(queued) = self.autopilot.starting.take() else {
            return;
        };
        match started {
            Started::Running(_) => {
                let cap = Cap::new(queued.size, Instant::now());
                let words =
                    autopilot::chained_words(queued.next, queued.after, &queued.chosen, &cap);
                self.chain_moment(&queued.project, &words, false, false);
            }
            Started::Refused(why) => {
                self.sessions.forget_preflight();
                let words =
                    autopilot::not_started_words(&queued.next.to_string(), queued.after, &why);
                self.chain_moment(&queued.project, &words, true, true);
            }
        }
    }

    /// A change of a chain's state (D-21, D-22; EP-19a): a line of the chain log and a
    /// notification of the chain's own kind, which the status bar says instead while the window
    /// is in front. `ended` lets the project's plan go: the chain is over.
    fn chain_moment(&mut self, project: &str, words: &str, warning: bool, ended: bool) {
        self.log_chain(Some(project), words);
        self.autopilot
            .moments
            .push(Toast::chain(project, words, warning));
        if ended {
            self.autopilot.drop_plan(project);
        }
    }

    /// Keeps a line of the chain log in the ledger; a screenshot run keeps it in the window.
    fn log_chain(&mut self, project: Option<&str>, words: &str) {
        if self.fixtures || self.autopilot.canned {
            self.autopilot.log.insert(
                0,
                EventRow {
                    id: 0,
                    at: Utc::now().to_rfc3339(),
                    kind: autopilot::EVENT_KIND.to_owned(),
                    project: project.map(str::to_owned),
                    detail: Some(words.to_owned()),
                },
            );
            return;
        }
        if let Some(ledger) = self.ledger().map(PathBuf::from) {
            match autopilot::log(&ledger, project, words, Utc::now()) {
                Ok(()) => self.autopilot.stale = true,
                Err(why) => {
                    self.autopilot.problem = Some(format!("The chain log was not kept: {why}"));
                }
            }
        }
    }

    /// Reads the log and the registry's pause again when they changed or every half minute.
    fn read_pilot(&mut self) {
        let pilot = &mut self.autopilot;
        let due = pilot.stale || pilot.read_at.is_none_or(|at| at.elapsed() >= READ_EVERY);
        if !due || pilot.canned {
            return;
        }
        pilot.read_at = Some(Instant::now());
        pilot.stale = false;
        if let Ok(loaded) = self.store().load() {
            let settings = &loaded.registry.settings;
            self.autopilot.paused = settings.autopilot_paused();
            self.autopilot.mark = settings.autopilot_pause_percent();
        }
        if let Some(ledger) = self.ledger().map(PathBuf::from) {
            match autopilot::recent(&ledger, LOG_LINES) {
                Ok(rows) => {
                    self.autopilot.log = rows;
                    self.autopilot.problem = None;
                }
                Err(why) => {
                    self.autopilot.problem = Some(format!("The chain log did not read: {why}"))
                }
            }
        }
    }

    /// Turns a project's autopilot on or off (the list's box, the A key): `true` when the
    /// registry was written.
    pub fn set_autopilot(&mut self, name: &str, on: bool) -> bool {
        let changed = self
            .store()
            .change(&files::today(), |c| c.set_autopilot(name, on));
        match changed {
            Ok(_) => {
                let words = if on {
                    "autopilot on: its runs commit themselves (your standing yes), and after a \
                     headless run completes the gate may chain its next brief"
                } else {
                    "autopilot off: its runs stop before committing, and no brief is chained"
                };
                self.notify(format!("{name}: {words}."));
                let log = if on {
                    "turned on by the owner"
                } else {
                    "turned off by the owner"
                };
                self.log_chain(Some(name), log);
                if !on {
                    self.autopilot
                        .held
                        .retain(|held| !held.project.eq_ignore_ascii_case(name));
                    self.autopilot.drop_plan(name);
                }
            }
            Err(e) => self.fail(e.to_string()),
        }
        true
    }

    /// The owner's Pause of every chain, or its Resume (the Sessions pane, the tray): kept in
    /// the registry so that it holds across restarts; a Resume reads the held chains again.
    pub fn pause_autopilot(&mut self, paused: bool) -> bool {
        let value = if paused { "on" } else { "off" };
        let changed = self.store().change(&files::today(), |c| {
            c.set_setting("autopilot_paused", value)
        });
        match changed {
            Ok(_) => {
                self.autopilot.paused = paused;
                if paused {
                    self.notify(
                        "The autopilot is paused: no brief is chained until you resume it; the \
                         runs that work go on.",
                    );
                    self.log_chain(None, autopilot::PAUSED_BY_OWNER);
                } else {
                    self.autopilot.recheck = true;
                    self.notify("The autopilot is resumed: held chains are read again.");
                    self.log_chain(None, autopilot::RESUMED_BY_OWNER);
                }
            }
            Err(e) => self.fail(e.to_string()),
        }
        true
    }

    /// Resumes the window's ended session `key` with the owner's yes to its commit (D-22), off
    /// the UI thread as a start is; a screenshot run starts nothing.
    pub fn commit_session(&mut self, key: u64) {
        if self.fixtures {
            self.warn("A screenshot run starts no session: its projects are tracked fixtures.");
            return;
        }
        if self.sessions.is_starting() {
            return;
        }
        let Some(program) = self
            .sessions
            .live
            .iter()
            .find(|live| live.key == key && live.offers_commit())
            .map(|live| live.program().to_owned())
        else {
            return;
        };
        let ctx = self.ctx().clone();
        let Some(preflight) = self.sessions.preflight(&program, &ctx) else {
            self.warn("claude has not answered yet: press Commit again in a moment.");
            return;
        };
        let refusals = preflight.refusals();
        if !refusals.is_empty() {
            self.fail(format!(
                "The session cannot resume: {}",
                refusals.join("; ")
            ));
            return;
        }
        let setting = self
            .store()
            .load()
            .ok()
            .and_then(|loaded| loaded.registry.settings.decisions);
        let route = self.sessions.route_for(&preflight, setting).0;
        self.sessions.commit(key, preflight, route, &ctx);
    }

    /// The autopilot's section at the top of the Sessions pane's All tab: its state with Pause
    /// or Resume, the chains held, and the log's latest lines.
    pub fn autopilot_section(&mut self, ui: &mut egui::Ui) {
        let on: Vec<String> = self
            .portfolio
            .projects
            .iter()
            .filter(|row| row.project.autopilot.enabled)
            .map(|row| row.name().to_owned())
            .collect();
        let now = Utc::now();
        let usage = autopilot::usage_pause(
            &self.meter.gauge.figures(now),
            self.autopilot.mark.max(1),
            now,
        );
        let mut actions = Vec::new();
        egui::CollapsingHeader::new(RichText::new("Autopilot").strong())
            .id_salt("autopilot-section")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let (state, colour) = if self.autopilot.paused {
                        ("paused by you", style::amber(ui))
                    } else if !usage.is_empty() {
                        ("held by usage", style::amber(ui))
                    } else if on.is_empty() {
                        ("off everywhere", ui.visuals().weak_text_color())
                    } else {
                        ("on", style::green(ui))
                    };
                    style::chip(ui, state, colour);
                    let projects = match on.len() {
                        0 => "no project has it on".to_owned(),
                        1 => format!("on in {}", on[0]),
                        n => format!("on in {n} projects: {}", on.join(", ")),
                    };
                    ui.label(RichText::new(projects).weak());
                    let (text, hover, pause) = if self.autopilot.paused {
                        (
                            "Resume",
                            "Let the chains go on: the held ones are read again",
                            false,
                        )
                    } else {
                        (
                            "Pause",
                            "Hold every chain until you resume: the runs that work go on",
                            true,
                        )
                    };
                    if ui.button(text).on_hover_text(hover).clicked() {
                        actions.push(Action::PauseAutopilot(pause));
                    }
                });
                for refusal in &usage {
                    ui.label(
                        RichText::new(format!("The chains hold: {}.", refusal.words()))
                            .small()
                            .color(style::amber(ui)),
                    );
                }
                for held in &self.autopilot.held {
                    ui.label(
                        RichText::new(format!("{}: {}", held.project, held.words))
                            .small()
                            .color(style::amber(ui)),
                    );
                }
                // The plans the chains follow (EP-19a), in the autopilot's violet.
                for plan in self.autopilot.plans.values() {
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("{}: your plan {}", plan.project, plan.words()))
                                .small()
                                .color(style::reference(ui)),
                        )
                        .wrap(),
                    )
                    .on_hover_text(
                        "The chain takes your choice for each brief you chose, the rules' for \
                         one you left alone, and ends before one you left out; kept until the \
                         chain ends or the autopilot is turned off",
                    );
                }
                if let Some(problem) = &self.autopilot.problem {
                    ui.label(RichText::new(problem).small().color(style::red(ui)));
                }
                if self.autopilot.log.is_empty() {
                    ui.label(
                        RichText::new(
                            "The chain log is empty: a headless run of a project whose autopilot \
                             is on writes it when it ends.",
                        )
                        .small()
                        .weak(),
                    );
                }
                for row in &self.autopilot.log {
                    ui.add(egui::Label::new(log_line(row)).wrap());
                }
            });
        ui.separator();
        self.actions.extend(actions);
    }
}

/// A line of the chain log: its local time, its project and its words.
fn log_line(row: &EventRow) -> RichText {
    let at = row
        .at()
        .map(|at: DateTime<Utc>| {
            let at = at.with_timezone(&Local);
            if at.date_naive() == Local::now().date_naive() {
                at.format("%H:%M").to_string()
            } else {
                at.format("%d %b %H:%M").to_string()
            }
        })
        .unwrap_or_default();
    let project = row
        .project
        .as_deref()
        .map_or(String::new(), |name| format!("{name}: "));
    RichText::new(format!(
        "{at}  {project}{}",
        row.detail.as_deref().unwrap_or_default()
    ))
    .small()
}
