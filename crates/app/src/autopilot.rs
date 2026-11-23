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
//! kind (the chain's moment). From EP-19b the window sets what a chain needs (D-18): turning a
//! project's autopilot on where its runs would wait at each shell command **offers** permission
//! mode `auto` first, and the Autopilot section shows each project's permission mode, the gate's
//! verdict on the conditions that stand between runs, and its limits, which it sets; an idle
//! session no longer refuses a chain (the owner's choice), it is named in the chained line. A
//! screenshot run reads no gate and starts nothing.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Utc};
use crw_core::autopilot::{self, Answer, Ask, Cap, Plan, Refusal};
use crw_core::ledger::EventRow;
use crw_core::prompt::Draft;
use crw_core::registry::{self, Limit, Project, Registry};
use crw_core::roadmap::{EpId, Size};
use crw_core::session::{Launch, Outcome, cli};
use crw_core::sessions::Status;
use crw_core::{files, process, rules};
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
/// How long a chained start waits for a poll to confirm an idle session in its project before
/// it counts it (EP-19b's review): two of the poll's turns while the window is not in front.
const CONFIRM_FOR: chrono::TimeDelta = chrono::TimeDelta::seconds(75);
/// The runs a day and the budgets the section's pickers offer (EP-19b); `crw set` takes any.
const DAILY_CHOICES: [u32; 8] = [0, 1, 2, 3, 4, 5, 6, 8];
const BUDGET_CHOICES: [f64; 6] = [5.0, 10.0, 15.0, 20.0, 30.0, 50.0];
/// The chained runs a day in all projects the section's own picker offers (EP-31b); `crw
/// setting autopilot_daily_total` takes any up to `registry::MAX_DAILY`.
const TOTAL_CHOICES: [u32; 12] = [0, 1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20];
/// The day's line (EP-31b), the Autopilot section's and the phone's block's: the words after
/// the day's count and the total, what they say of the two caps (D-22), and the words once the
/// count is at or over the total.
pub(crate) const TODAY_WORDS: &str = "chained runs today in all projects";
pub(crate) const BOTH_CAPS: &str = "a chain ends at this total or at its project's own runs a \
    day, whichever it meets first";
pub(crate) const REACHED: &str = "The total is reached for today: a chain ends at its next gate.";
/// The words after the total while the day's count did not read.
const UNREAD_TODAY: &str = "chained runs a day in all projects (today's count did not read)";
/// The marks a chain's own picker offers (EP-41), in percent of the session (five-hour) window,
/// beside the setting's.
const MARK_CHOICES: [u8; 7] = [30, 40, 50, 60, 70, 80, 90];

/// What `auto` means, as the offer says it (D-18; EP-19b), from Claude Code's permission-modes
/// and permissions pages (read 2026-11-09).
pub const AUTO_WORDS: &str = "With auto, Claude Code runs a session's commands without asking \
    you. A classifier model reviews each action that is not a read or an edit in the project's \
    folder, and blocks one that goes beyond the task or looks risky: downloading and running \
    code, sending data out, a force push, git reset --hard, a destroy command. Your own ask and \
    deny rules still apply.";
/// What it means for the project's commits (EP-19b): the push the classifier allows, the deny
/// rule that refuses it, and the push that stays the owner's.
pub const PUSH_WORDS: &str = "A plain git push to the project's own remote is one auto allows, \
    so each headless run of a project whose autopilot is on is also given a deny rule that \
    refuses git push as written, and is told never to push: its commits stay local until you \
    push them yourself, after reading the diff. A push written another way (git -C . push) \
    would get past the rule.";

/// The offer of `auto` (D-18; EP-19b): the autopilot being turned on in a project whose
/// headless runs would wait for the owner at each shell command, which the gate refuses.
#[derive(Debug, Clone, PartialEq)]
pub struct Offer {
    pub project: String,
    /// The permission mode under which they would wait.
    pub mode: String,
}

/// A held chain: by the pause alone (D-22), or for the restart at the gate's moment (EP-37a). It
/// lives in core beside the plan, since the handover carries both.
pub use crw_core::autopilot::Held;

/// How long the held chains a restart carried wait for the first poll of live sessions before
/// their gates are read anyway (EP-37a): four of the poll's turns while the window is not in
/// front.
const CARRIED_WAIT: Duration = Duration::from_secs(120);

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
    /// The sessions open and idle in the project at the start, which the chained line names
    /// (EP-19b): they no longer refuse a chain.
    idle: Vec<String>,
    /// When the start was queued: a poll after it confirms an idle session (EP-19b's review).
    queued_at: DateTime<Utc>,
    /// The chain's own five-hour mark, from its plan (EP-41), which the chained line names.
    mark: Option<u8>,
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
    pub recheck: bool,
    /// When a restart's handover brought held chains (EP-37a): their gates wait for the first
    /// poll of live sessions, so that a session open in the project counts, or for
    /// [`CARRIED_WAIT`].
    pub carried: Option<Instant>,
    queued: Option<Queued>,
    /// The chain's start on its way: the next start's result is the chain's.
    starting: Option<Queued>,
    /// The log's latest lines, the newest first.
    pub log: Vec<EventRow>,
    pub problem: Option<String>,
    read_at: Option<Instant>,
    /// When a read of the log that found the ledger locked tries again (EP-43).
    retry_at: Option<Instant>,
    /// The log or the registry's pause changed: read them again.
    pub stale: bool,
    /// The owner's Pause and the marks, as the registry last said them: the weekly and Fable
    /// limits' and the session (five-hour) window's (EP-40).
    pub paused: bool,
    pub mark: u8,
    pub five_hour_mark: u8,
    /// The chained runs a day in all projects, as the registry last said it, and the day's
    /// chained starts in all projects as the gate counts them, from the chain log's latest
    /// [`autopilot::DAY_LINES`] lines at its last read: none while the log does not read
    /// (EP-31b).
    pub daily_total: u32,
    pub chained_today: Option<u32>,
    /// A screenshot run's log: nothing is read or written.
    pub canned: bool,
    /// The chain plans (EP-19a), by project in lower case: each kept from Start headless until
    /// its chain ends.
    pub plans: BTreeMap<String, Plan>,
    /// The projects whose chain's own mark changed while their gate was read (EP-41's review):
    /// a hold that comes back is read again at the next pass, with the new mark.
    pub recheck_after: BTreeSet<String>,
    /// The chain's moments since the last pass, for the popup.
    moments: Vec<Toast>,
    /// The gate's verdict on the conditions that stand between runs (EP-19b), by project in
    /// lower case, for each project whose autopilot is on: read with the log, never each frame.
    pub standing: BTreeMap<String, Vec<Refusal>>,
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
            carried: None,
            queued: None,
            starting: None,
            log: Vec::new(),
            problem: None,
            read_at: None,
            retry_at: None,
            stale: false,
            paused: false,
            mark: autopilot::DEFAULT_PAUSE_PERCENT,
            five_hour_mark: autopilot::DEFAULT_FIVE_HOUR_PERCENT,
            daily_total: autopilot::DEFAULT_DAILY_TOTAL,
            chained_today: None,
            canned: false,
            plans: BTreeMap::new(),
            recheck_after: BTreeSet::new(),
            moments: Vec::new(),
            standing: BTreeMap::new(),
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

    /// Whether a gate is read or a chained start waits for the slot or is on its way: a restart
    /// would lose it (EP-37).
    pub fn is_busy(&self) -> bool {
        !self.gating.is_empty()
            || !self.reading.is_empty()
            || self.queued.is_some()
            || self.starting.is_some()
    }

    /// Whether a gate is read for `project` (EP-30b): a start from the phone then would race the
    /// chain the gate may queue.
    pub fn is_reading(&self, project: &str) -> bool {
        self.reading.contains(&project.to_lowercase())
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
        // A restart is decided and the window closes in a moment (EP-37a's review): no gate is
        // read again, so the chain just held for it is weighed once, in the new window.
        if self.restart.closing_at.is_some() {
            return;
        }
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
            // The project read again first (EP-25a), so the row says the brief the run closed
            // while the gate is read (which reads the files for itself).
            self.reread_after_run(key);
            self.weigh_run(key);
        }
        self.take_gates();
        self.read_held();
        self.start_queued();
        self.read_pilot();
    }

    /// The project of the window's run `key`, which ended, read again on its own (EP-25a): its
    /// files, git and history, the other rows left as they are. A chat's turn reads nothing
    /// (D-27: it writes nothing); a screenshot run's canned sessions read nothing.
    pub(crate) fn reread_after_run(&mut self, key: u64) {
        if self.fixtures {
            return;
        }
        let Some(live) = self.sessions.live.iter().find(|live| live.key == key) else {
            return;
        };
        if live.is_chat() || live.ended.is_none() {
            return;
        }
        let (project, chained) = (live.project.clone(), live.chained);
        self.loader
            .read_project(&project, crate::portfolio::Moment::RunEnded { chained });
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
        // A migration is no brief: no gate follows it, and no chain (EP-21a); nor is a chat
        // conversation's turn (EP-24).
        if live.migration || live.is_chat() {
            return;
        }
        let (project, ran, chained) = (live.project.clone(), live.ep.parse().ok(), live.chained);
        // A usage limit that cut the run short is named in the chain log (EP-40).
        let limit_hit = live
            .ended
            .as_ref()
            .and_then(|ended| ended.result.as_ref())
            .and_then(|finish| finish.text.as_deref())
            .and_then(crw_core::usage::limit_hit);
        let registry = match self.store().load() {
            Ok(loaded) => loaded.registry,
            Err(e) => {
                // IN-14: a chain the failure would stop with no word is held and said, or ended
                // by its run's outcome, which no registry changes.
                let on = chained
                    || self
                        .portfolio
                        .project(&project)
                        .is_some_and(|row| row.project.autopilot.enabled);
                match (on, outcome) {
                    (true, Outcome::Completed) => match e.for_newer_build() {
                        Some(why) => self.hold_unread(&project, ran, &why, true),
                        None => self.hold_unread(&project, ran, &e.to_string(), false),
                    },
                    (true, outcome) => {
                        let words = autopilot::refused_words(ran, &[Refusal::Outcome(outcome)]);
                        self.chain_moment(&project, &words, true, true);
                    }
                    (false, _) => {
                        self.fail(format!("The autopilot could not read the registry: {e}"));
                    }
                }
                return;
            }
        };
        let Some(entry) = registry.project(&project).cloned() else {
            return;
        };
        if !entry.autopilot.enabled && !chained {
            return;
        }
        self.read_gate(registry, entry, ran, outcome, limit_hit);
    }

    /// IN-14 (the owner's choice at EP-30): the registry did not read where the gate needs it,
    /// after a run, at a held chain's re-read or at a chained start, which before stopped the
    /// chain with no word or failed at each pass. The chain is held, its words naming the
    /// failure, said once (a line of the chain log and the chain's moment). When a newer build
    /// may read it (`newer`: a key, a value or a format this build does not take, in the
    /// parser's message alone, never the file's line, EP-30b's review), the words name File >
    /// Install and restart, and with a newer build installed the window restarts into it at
    /// once, the chain carried, for the new window to weigh the gate. A hold whose registry
    /// reads again is weighed at the next re-read, as any held chain is.
    pub(crate) fn hold_unread(
        &mut self,
        project: &str,
        after: Option<EpId>,
        why: &str,
        newer: bool,
    ) {
        let words = autopilot::unread_words(after, why, newer);
        let at = self
            .autopilot
            .held
            .iter()
            .position(|held| held.project.eq_ignore_ascii_case(project));
        let fresh = at.is_none_or(|at| self.autopilot.held[at].words != words);
        if fresh {
            if let Some(at) = at {
                self.autopilot.held.remove(at);
            }
            self.autopilot.held.push(Held {
                project: project.to_owned(),
                after,
                words: words.clone(),
            });
            self.chain_moment(project, &words, true, false);
        }
        if !newer || self.restart.closing_at.is_some() || !self.restart.pending_install() {
            return;
        }
        match self.restart_at_gate() {
            Ok(()) => self.log_chain(Some(project), autopilot::UNREAD_RESTART),
            // Said once with the hold; the next re-read tries again.
            Err(why) if fresh => self.log_chain(
                Some(project),
                &format!("the restart into the installed build waits: {why}"),
            ),
            Err(_) => {}
        }
    }

    /// Reads the gate of `entry` after its run of `after` on a worker thread.
    fn read_gate(
        &mut self,
        registry: Registry,
        entry: Project,
        after: Option<EpId>,
        outcome: Outcome,
        limit_hit: Option<String>,
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
            // Ended, a held chain goes with its plan (EP-37a's review).
            self.autopilot
                .held
                .retain(|held| !held.project.eq_ignore_ascii_case(&entry.name));
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
        let (mut others, _idle) = self.gate_sessions(&entry.name);
        if self.autopilot.is_starting_in(&entry.name) {
            others.push("a chained start is on its way".to_owned());
        }
        let home = std::env::var_os("USERPROFILE").map(PathBuf::from);
        let ledger = self.ledger().map(PathBuf::from);
        let data_root = registry.data_root();
        // The chain's own five-hour mark, from its plan (EP-41); none holds at the setting.
        let five_hour_mark = self
            .autopilot
            .plan_of(&entry.name)
            .and_then(|plan| plan.five_hour_mark);
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
                        limit_hit,
                        figures: &figures,
                        levels: &levels,
                        now,
                        home: home.as_deref(),
                        ledger: ledger.as_deref(),
                        others,
                        five_hour_mark,
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
        // The chain's own mark changed while this gate was read (EP-41's review).
        let again = self.autopilot.recheck_after.remove(&project.to_lowercase());
        let was_held = self
            .autopilot
            .held
            .iter()
            .position(|held| held.project.eq_ignore_ascii_case(&project));
        if !refusals.is_empty() {
            let words = autopilot::refused_words(after, &refusals);
            if autopilot::holds(&refusals) {
                if again {
                    self.autopilot.held_read = None;
                }
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
                // Between runs: a pending install is taken now, the held chain carried (EP-37a).
                let _ = self.restart_at_gate();
            } else {
                // A chain's end restarts nothing (the owner's choice at EP-37a's pickup): its
                // popup stays, and the chip says a newer build waits.
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
        // The gate's moment (EP-37a): with a newer build installed, the chain is held for the
        // window's restart, and the new window's gate starts the brief; a refused restart lets the
        // chain go on here, the chip saying that the build waits.
        if self.restart.pending_install() {
            let words = autopilot::restart_words(after, draft.ep);
            self.autopilot.held.push(Held {
                project: project.clone(),
                after,
                words: words.clone(),
            });
            match self.restart_at_gate() {
                Ok(()) => {
                    self.log_chain(Some(&project), &words);
                    return;
                }
                Err(why) => {
                    self.autopilot.held.pop();
                    self.log_chain(
                        Some(&project),
                        &format!("the restart into the installed build waits: {why}"),
                    );
                }
            }
        }
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
        let mark = self
            .autopilot
            .plan_of(&project)
            .and_then(|plan| plan.five_hour_mark);
        self.autopilot.queued = Some(Queued {
            mark,
            project,
            after,
            next: draft.ep,
            size: draft.size,
            effort: draft.parts.effort.clone(),
            chosen,
            launch,
            program,
            idle: Vec::new(),
            queued_at: Utc::now(),
        });
    }

    /// Reads the held chains' gates again when the owner resumed or every five minutes.
    fn read_held(&mut self) {
        if self.autopilot.held.is_empty() || self.fixtures {
            return;
        }
        // The held chains a restart carried wait for the first poll of live sessions (EP-37a), so
        // that a session open in their project counts, and for the first files read, which
        // gives the gate their projects' folders; then they are read at once.
        if let Some(at) = self.autopilot.carried {
            let model = &self.watch.model;
            let polled = model.polled.is_some() || model.poll_problem.is_some();
            let read = !self.loader.reading_files && !self.portfolio.projects.is_empty();
            if !(polled && read) && at.elapsed() < CARRIED_WAIT {
                return;
            }
            self.autopilot.carried = None;
            self.autopilot.held_read = None;
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
        let loaded = match self.store().load() {
            Ok(loaded) => loaded,
            // IN-14: a registry a newer build may read is no longer passed over in silence: each
            // hold is said once, and a newer build taken. Any other failure waits for the next
            // re-read, as before.
            Err(e) => {
                let Some(why) = e.for_newer_build() else {
                    return;
                };
                for held in self.autopilot.held.clone() {
                    if self.restart.closing_at.is_some() {
                        break;
                    }
                    self.hold_unread(&held.project, held.after, &why, true);
                }
                return;
            }
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
                        None,
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
            // IN-14: a registry a newer build may read holds the chain, said once, where this
            // failed again at every pass; any other failure is tried again at the next pass.
            Err(e) => {
                match e.for_newer_build() {
                    Some(why) => {
                        let queued = self.autopilot.queued.take().expect("a start is queued");
                        self.hold_unread(&queued.project, queued.after, &why, true);
                    }
                    None => self.fail(format!("The autopilot could not read the registry: {e}")),
                }
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
        // A session working or waiting in the project since the gate was read would edit the
        // same files; an idle one is named in the chained line (EP-19b). Without the prompt hook
        // (EP-20b, installed by the owner) a tab just prompted reads idle until the next poll, so an idle
        // one is left out only once a poll newer than the queued start has listed it; none
        // within `CONFIRM_FOR`, and it counts as every open session did before (EP-19b's
        // review).
        let (mut others, idle) = self.gate_sessions(&queued.project);
        if others.is_empty() && !idle.is_empty() {
            match idle_confirmed(self.watch.model.polled, queued.queued_at, Utc::now()) {
                None => return,
                Some(true) => {}
                Some(false) => others.clone_from(&idle),
            }
        }
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
        queued.idle = idle;
        let run = Run {
            capped: true,
            size: queued.size,
            chained: true,
            commits: true,
            migration: false,
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
                let mut words =
                    autopilot::chained_words(queued.next, queued.after, &queued.chosen, &cap);
                words.push_str(&autopilot::own_mark_words(queued.mark));
                words.push_str(&idle_words(&queued.idle));
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
    pub(crate) fn log_chain(&mut self, project: Option<&str>, words: &str) {
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

    /// Reads the log, the registry's pause, marks and day's total, the day's chained runs and the
    /// gate's standing verdicts again when they changed or every half minute; a screenshot run
    /// keeps its canned log, pause and count. The log's read waits a moment at most for another
    /// connection's lock, in the window's pass: one that found the ledger locked keeps the lines
    /// and the count it has, without a word, and tries again a second later (EP-43).
    fn read_pilot(&mut self) {
        let pilot = &mut self.autopilot;
        let due = pilot.stale || pilot.read_at.is_none_or(|at| at.elapsed() >= READ_EVERY);
        if !due || pilot.retry_at.is_some_and(|at| Instant::now() < at) {
            return;
        }
        pilot.read_at = Some(Instant::now());
        pilot.stale = false;
        pilot.retry_at = None;
        let loaded = self.store().load().ok();
        if let Some(loaded) = &loaded {
            self.autopilot.standing = self.standing_of(&loaded.registry);
        }
        if self.autopilot.canned {
            return;
        }
        if let Some(loaded) = &loaded {
            let settings = &loaded.registry.settings;
            self.autopilot.paused = settings.autopilot_paused();
            self.autopilot.mark = settings.autopilot_pause_percent();
            self.autopilot.five_hour_mark = settings.autopilot_five_hour_percent();
            self.autopilot.daily_total = settings.autopilot_daily_total();
        }
        if let Some(ledger) = self.ledger().map(PathBuf::from) {
            // One read for the section's lines and the day's count, which the gate makes of the
            // same lines on the same local day (EP-31b).
            match autopilot::recent_waiting(&ledger, autopilot::DAY_LINES, crate::pace::LEDGER_WAIT)
            {
                Ok(mut rows) => {
                    let day = Local::now().date_naive();
                    self.autopilot.chained_today = Some(autopilot::chained_in_all(&rows, day));
                    rows.truncate(LOG_LINES);
                    self.autopilot.log = rows;
                    self.autopilot.problem = None;
                }
                Err(e) if e.is_busy() => {
                    self.autopilot.stale = true;
                    self.autopilot.retry_at = Some(crate::pace::retry_after(Instant::now()));
                }
                Err(e) => {
                    self.autopilot.chained_today = None;
                    self.autopilot.problem = Some(format!("The chain log did not read: {e}"))
                }
            }
        }
    }

    /// Turns a project's autopilot on or off (the list's box, the A key): `true` when the
    /// registry was written. Turned on where the project's runs would wait at each shell command,
    /// it opens the offer of `auto` instead and writes nothing yet (EP-19b).
    pub fn set_autopilot(&mut self, name: &str, on: bool) -> bool {
        if on && let Some(mode) = self.would_wait(name) {
            self.offer = Some(Offer {
                project: name.to_owned(),
                mode,
            });
            return false;
        }
        let changed = self
            .store()
            .change(&files::today(), |c| c.set_autopilot(name, on));
        self.switched(
            name,
            on,
            false,
            changed.map(|_| ()).map_err(|e| e.to_string()),
        );
        true
    }

    /// The offer's answer (EP-19b): the autopilot turned on, and permission mode `auto` set
    /// with it in the same write when `auto` holds.
    pub fn autopilot_on(&mut self, name: &str, auto: bool) -> bool {
        self.offer = None;
        let changed = self.store().change(&files::today(), |c| {
            if auto {
                c.set_permission_mode(name, Some("auto"))?;
            }
            c.set_autopilot(name, true)
        });
        self.switched(
            name,
            true,
            auto,
            changed.map(|_| ()).map_err(|e| e.to_string()),
        );
        true
    }

    /// What follows a switch's write: the notice, the log's line and, when off, the held chain
    /// and the plan let go.
    fn switched(&mut self, name: &str, on: bool, auto: bool, changed: Result<(), String>) {
        if let Err(e) = changed {
            self.fail(e);
            return;
        }
        let (words, log) = match (on, auto) {
            (true, true) => (
                "autopilot on, with permission mode auto: its runs commit themselves (your \
                 standing yes) and run their commands unasked, never pushing, and after a \
                 headless run completes the gate may chain its next brief",
                "turned on by the owner, with permission mode auto",
            ),
            (true, false) => (
                "autopilot on: its runs commit themselves (your standing yes), and after a \
                 headless run completes the gate may chain its next brief",
                "turned on by the owner",
            ),
            (false, _) => (
                "autopilot off: its runs stop before committing, and no brief is chained",
                "turned off by the owner",
            ),
        };
        self.notify(format!("{name}: {words}."));
        self.log_chain(Some(name), log);
        self.autopilot.stale = true;
        if !on {
            self.autopilot
                .held
                .retain(|held| !held.project.eq_ignore_ascii_case(name));
            self.autopilot.drop_plan(name);
        }
    }

    /// The permission mode under which `name`'s headless runs would wait at each shell command,
    /// when they would (D-22): the gate would refuse its chains (EP-19b).
    fn would_wait(&self, name: &str) -> Option<String> {
        let loaded = self.store().load().ok()?;
        let project = loaded.registry.project(name)?;
        let folder = project.folder(&loaded.registry.data_root());
        let home = self.user_home();
        autopilot::standing(project, &folder, home.as_deref())
            .into_iter()
            .find_map(|refusal| match refusal {
                Refusal::Waits(mode) => Some(mode),
                _ => None,
            })
    }

    /// The user's profile folder, whose `.claude\settings.json` may hold allow rules; none in a
    /// screenshot run, which reads no Claude Code settings.
    fn user_home(&self) -> Option<PathBuf> {
        if self.fixtures {
            return None;
        }
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }

    /// The gate's verdict on the condition that stands between runs for each project whose
    /// autopilot is on (EP-19b): runs that would wait. The tool's own project stood there too
    /// until EP-37a. A screenshot run reads no user settings.
    fn standing_of(&self, registry: &Registry) -> BTreeMap<String, Vec<Refusal>> {
        let home = self.user_home();
        let root = registry.data_root();
        registry
            .projects
            .iter()
            .filter(|project| project.autopilot.enabled)
            .map(|project| {
                let folder = project.folder(&root);
                let refusals = autopilot::standing(project, &folder, home.as_deref());
                (project.name.to_lowercase(), refusals)
            })
            .collect()
    }

    /// The sessions open in `project` as the gate weighs them (D-22; EP-19b, the owner's
    /// choice): those that refuse a chain, the window's own runs and the sessions seen busy or
    /// waiting, and the idle ones, whose turn is over, which do not.
    pub fn gate_sessions(&self, project: &str) -> (Vec<String>, Vec<String>) {
        let (seen, idle) = self.open_elsewhere(project);
        let refusing = self
            .sessions
            .summary(project)
            .into_iter()
            .chain(seen)
            .collect();
        (refusing, idle)
    }

    /// The sessions started elsewhere that are open in `project`: those busy or waiting, and the
    /// idle ones. The Autopilot section's verdict reads these alone, since the window's own run
    /// there is the one whose end the gate weighs (EP-19b's review); so does the phone's start
    /// (EP-30b), beside the window's own runs.
    pub(crate) fn open_elsewhere(&self, project: &str) -> (Vec<String>, Vec<String>) {
        let own = |id: &str| self.sessions.owns(id);
        let mut refusing = Vec::new();
        let mut idle = Vec::new();
        for seen in self.watch.running(project, own) {
            if seen.status == Status::Idle {
                idle.push(seen.summary());
            } else {
                refusing.push(seen.summary());
            }
        }
        (refusing, idle)
    }

    /// Keeps a permission mode for a project from the launch dialog (D-18; EP-19b): written into
    /// its `[project.launch]`, or taken out with `None` (back to the default). The open dialog's
    /// choice then says where the mode comes from.
    pub fn keep_permission_mode(&mut self, name: &str, mode: Option<&str>) -> bool {
        let changed = self
            .store()
            .change(&files::today(), |c| c.set_permission_mode(name, mode));
        let registry = match changed {
            Ok((registry, ())) => registry,
            Err(e) => {
                self.fail(e.to_string());
                return true;
            }
        };
        let Some(project) = registry.project(name) else {
            return true;
        };
        let kept = rules::permission_mode(project);
        let words = if project.launch.permission_mode.is_some() {
            format!(
                "{name}: permission mode {} kept for its launches and chains.",
                kept.value
            )
        } else {
            format!(
                "{name}: permission mode back to the default, {}.",
                kept.value
            )
        };
        if let Some(dialog) = self
            .launch
            .as_mut()
            .filter(|dialog| dialog.project.eq_ignore_ascii_case(name))
        {
            dialog.kept_permission_mode(project.launch.permission_mode.clone(), kept);
        }
        self.autopilot.stale = true;
        self.notify(words);
        true
    }

    /// Keeps a model or an effort for a project from the launch dialog (D-22; EP-19c): written
    /// into its rule for every brief or for one phase, or taken out with `None` (back to the
    /// rules). The open dialog's brief is resolved again from the registry, its chain plan made
    /// again with the owner's rows kept, and the notice names what stands over a kept value for
    /// that brief: an earlier rule, or the Fable downshift.
    pub fn keep_rule(
        &mut self,
        name: &str,
        scope: &rules::Scope,
        key: rules::Key,
        value: Option<&str>,
    ) -> bool {
        let changed = self
            .store()
            .change(&files::today(), |c| c.keep(name, scope, key, value));
        let registry = match changed {
            Ok((registry, ())) => registry,
            Err(e) => {
                self.fail(e.to_string());
                return true;
            }
        };
        let Some(project) = registry.project(name) else {
            return true;
        };
        let what = key.as_str();
        let mut words = match value {
            Some(value) => format!("{name}: {what} {value} kept for {}.", scope.words()),
            None => format!("{name}: {what} back to the rules for {}.", scope.words()),
        };
        if let Some(dialog) = self
            .launch
            .as_mut()
            .filter(|dialog| dialog.project.eq_ignore_ascii_case(name))
        {
            let now = Utc::now();
            let usage = rules::Usage::of(&self.meter.gauge.figures(now), now);
            let brief = rules::Brief {
                size: dialog.size,
                phase: dialog.phase.as_deref(),
                title: &dialog.title,
            };
            let fresh = rules::resolve(&registry, project, &brief, &dialog.levels, &usage);
            if let Some(value) = value
                && let Some(over) = rules::stands_over(project, scope, key, &fresh)
            {
                words = format!(
                    "{name}: {what} {value} kept for {}, but for {} {over} stands over it.",
                    scope.words(),
                    dialog.label()
                );
            }
            // A migration has no brief and no chain plan (EP-21a).
            let plan = dialog
                .plan
                .as_ref()
                .and_then(|_| self.portfolio.project(name))
                .zip(dialog.ep)
                .map(|(row, ep)| {
                    autopilot::plan(
                        &registry,
                        project,
                        &row.snapshot,
                        ep,
                        &dialog.levels,
                        &usage,
                    )
                });
            dialog.kept_rule(key, value, project, fresh, plan);
        }
        self.notify(words);
        true
    }

    /// Sets one of a project's autopilot limits from the Autopilot section (D-22; EP-19b).
    pub fn set_autopilot_limit(&mut self, name: &str, limit: &Limit) -> bool {
        let changed = self
            .store()
            .change(&files::today(), |c| c.set_autopilot_limit(name, limit));
        match changed {
            Ok(_) => {
                self.autopilot.stale = true;
                self.notify(format!("{name}: {}.", limit_words(limit)));
            }
            Err(e) => self.fail(e.to_string()),
        }
        true
    }

    /// Sets the chained runs a day in all projects from the Autopilot section (D-22; EP-31b),
    /// with the write `crw setting autopilot_daily_total` makes: `None` takes the setting out of
    /// the registry, the default then holding, as the other pickers do with theirs.
    pub fn set_daily_total(&mut self, total: Option<u32>) -> bool {
        let value = total.map_or_else(|| "default".to_owned(), |runs| runs.to_string());
        let changed = self.store().change(&files::today(), |c| {
            c.set_setting("autopilot_daily_total", &value)
        });
        match changed {
            Ok(_) => {
                // A screenshot run reads no setting again, so the section follows the click here.
                self.autopilot.daily_total = total.unwrap_or(autopilot::DEFAULT_DAILY_TOTAL);
                self.autopilot.stale = true;
                self.notify(total_words(total));
            }
            Err(e) => self.fail(e.to_string()),
        }
        true
    }

    /// A running chain's own five-hour mark changed in the Autopilot section (EP-41): its plan
    /// takes it from now, the change is a line of the chain log, and a chain held by the pause is
    /// read again at once, since a higher mark may let it go; a gate read while the change came
    /// is read again once it is in, if it held (the review's finding). Nothing in the registry
    /// changes, so no file is written and the files are not read again (`false`).
    pub fn set_chain_mark(&mut self, name: &str, mark: Option<u8>) -> bool {
        let setting = self.autopilot.five_hour_mark.max(1);
        let key = name.to_lowercase();
        let Some(plan) = self.autopilot.plans.get_mut(&key) else {
            return false;
        };
        if plan.five_hour_mark == mark {
            return false;
        }
        plan.five_hour_mark = mark;
        let project = plan.project.clone();
        self.log_chain(
            Some(&project),
            &autopilot::mark_words(mark, setting, "your change"),
        );
        if self
            .autopilot
            .held
            .iter()
            .any(|held| held.project.eq_ignore_ascii_case(&project))
        {
            self.autopilot.recheck = true;
        }
        if self.autopilot.reading.contains(&key) {
            self.autopilot.recheck_after.insert(key);
        }
        let words = match mark {
            Some(mark) => format!(
                "{project}: this chain holds at {mark} % of the session (five-hour) window from \
                 now."
            ),
            None => format!(
                "{project}: this chain holds at the setting's {setting} % of the session \
                 (five-hour) window from now."
            ),
        };
        self.notify(words);
        false
    }

    /// A headless run the owner started from the launch dialog runs (EP-41): a chain held in its
    /// project gives way to it, since the run's plan took the held chain's place (the review's
    /// finding: the held chain's next read counted the new run as a busy session, ended the
    /// chain and dropped the new plan with it); and a plan holding its own five-hour mark is
    /// named in the chain log.
    pub(crate) fn started_by_hand(&mut self, project: &str, ep: Option<EpId>) {
        if let Some(at) = self
            .autopilot
            .held
            .iter()
            .position(|held| held.project.eq_ignore_ascii_case(project))
        {
            let held = self.autopilot.held.remove(at);
            let after = held
                .after
                .map_or(String::new(), |ep| format!(" after {ep}"));
            let started = ep.map_or_else(|| "a run".to_owned(), |ep| ep.to_string());
            self.log_chain(
                Some(project),
                &format!("the chain ends{after}: you started {started} headless yourself"),
            );
        }
        self.log_own_mark(project);
    }

    /// The line of the chain log when a run whose plan holds its own five-hour mark starts
    /// (EP-41): the chain's mark, from then until the chain ends, is in the log.
    pub(crate) fn log_own_mark(&mut self, project: &str) {
        let setting = self.autopilot.five_hour_mark.max(1);
        let mark = self
            .autopilot
            .plan_of(project)
            .and_then(|plan| plan.five_hour_mark);
        if let Some(mark) = mark {
            let words = autopilot::mark_words(Some(mark), setting, "your choice in its plan");
            self.log_chain(Some(project), &words);
        }
    }

    /// The offer of `auto` over the window (D-18; EP-19b): why the project's runs would wait,
    /// what `auto` means, and three ways on. Escape or a click beside it puts it away.
    pub fn offer_dialog(&mut self, ctx: &egui::Context) {
        let Some(offer) = self.offer.clone() else {
            return;
        };
        let width = 600.0_f32.min(ctx.content_rect().width() - 80.0);
        let modal = egui::Modal::new(egui::Id::unique("autopilot-offer")).show(ctx, |ui| {
            ui.set_width(width);
            offer_ui(ui, &offer)
        });
        match modal.inner {
            Some(action) => self.actions.push(action),
            None if modal.should_close() => self.actions.push(Action::CloseOffer),
            None => {}
        }
    }

    /// The owner's Pause of every chain, or its Resume (the Sessions pane, the tray): kept in
    /// the registry so that it holds across restarts; a Resume reads the held chains again.
    pub fn pause_autopilot(&mut self, paused: bool) -> bool {
        match self.set_paused(paused) {
            Ok(words) => self.notify(words),
            Err(why) => self.fail(why),
        }
        true
    }

    /// The Pause or the Resume written, with its line of the chain log: its words, or why it was
    /// not written, which the desk says in the status bar and the phone's page on its own
    /// (EP-30b).
    pub(crate) fn set_paused(&mut self, paused: bool) -> Result<String, String> {
        let value = if paused { "on" } else { "off" };
        self.store()
            .change(&files::today(), |c| {
                c.set_setting("autopilot_paused", value)
            })
            .map_err(|e| e.to_string())?;
        self.autopilot.paused = paused;
        if paused {
            self.log_chain(None, autopilot::PAUSED_BY_OWNER);
            Ok(
                "The autopilot is paused: no brief is chained until you resume it; the runs that \
                work go on."
                    .to_owned(),
            )
        } else {
            self.autopilot.recheck = true;
            self.log_chain(None, autopilot::RESUMED_BY_OWNER);
            Ok("The autopilot is resumed: held chains are read again.".to_owned())
        }
    }

    /// Resumes the window's ended session `key` with the owner's yes to its commit (D-22), off
    /// the UI thread as a start is; a screenshot run starts nothing.
    pub fn commit_session(&mut self, key: u64) {
        if self.fixtures {
            self.warn("A screenshot run starts no session: its projects are fixtures or a copy.");
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
            self.autopilot.five_hour_mark.max(1),
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
                    // The session window's at the setting: a chain's own mark may let it go
                    // (EP-41).
                    let who = match refusal {
                        Refusal::Usage { which, .. } if *which == autopilot::FIVE_HOUR => {
                            "Chains without a mark of their own hold"
                        }
                        _ => "The chains hold",
                    };
                    ui.label(
                        RichText::new(format!("{who}: {}.", refusal.words()))
                            .small()
                            .color(style::amber(ui)),
                    );
                }
                self.day_total(ui, &mut actions);
                self.autopilot_projects(ui, &mut actions);
                for held in &self.autopilot.held {
                    ui.label(
                        RichText::new(format!("{}: {}", held.project, held.words))
                            .small()
                            .color(style::amber(ui)),
                    );
                }
                // The plans the chains follow (EP-19a), in the autopilot's violet, each with
                // its chain's own five-hour mark, which a change takes from now (EP-41).
                let setting = self.autopilot.five_hour_mark.max(1);
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
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("This chain holds at").small());
                        let id = ("chain-mark", plan.project.to_lowercase());
                        if let Some(mark) = mark_picker(ui, id, plan.five_hour_mark, setting) {
                            actions.push(Action::SetChainMark(plan.project.clone(), mark));
                        }
                        ui.label(RichText::new("of the session (five-hour) window").small());
                    });
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

    /// The day's line above the projects' (D-22; EP-31b): the day's chained runs in all
    /// projects as the gate counts them, of the total, which a picker of its own sets, and what
    /// the two caps do; amber, with a line of its own, once the count is at the total.
    fn day_total(&self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let total = self.autopilot.daily_total;
        let count = self.autopilot.chained_today;
        let reached = count.is_some_and(|count| count >= total);
        let colour = if reached {
            style::amber(ui)
        } else {
            ui.visuals().text_color()
        };
        ui.horizontal_wrapped(|ui| {
            if let Some(count) = count {
                ui.label(RichText::new(format!("{count} of")).small().color(colour));
            }
            if let Some(chosen) = total_picker(ui, total) {
                actions.push(Action::SetDailyTotal(chosen));
            }
            let after = if count.is_some() {
                TODAY_WORDS
            } else {
                UNREAD_TODAY
            };
            ui.label(RichText::new(format!("{after};")).small().color(colour));
            ui.label(RichText::new(BOTH_CAPS).small().weak());
        });
        if reached {
            verdict(ui, REACHED.to_owned(), style::amber(ui));
        }
    }

    /// One entry for each project whose autopilot is on (EP-19b): its permission mode and where
    /// it comes from, the gate's verdict on what stands between runs (runs that would wait, the
    /// tool's own project, the sessions open there now), and its limits as pickers that write at
    /// once.
    fn autopilot_projects(&self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let rows = self
            .portfolio
            .projects
            .iter()
            .filter(|row| row.project.autopilot.enabled);
        for row in rows {
            let name = row.name();
            let project = &row.project;
            let mode = rules::permission_mode(project);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(name).strong());
                let from = if project.launch.permission_mode.is_some() {
                    "the project's"
                } else {
                    "the default"
                };
                ui.label(RichText::new(format!("permission mode {} ({from})", mode.value)).small())
                    .on_hover_text(
                        "Kept from the launch dialog (the pin beside Permissions), set by the \
                         offer when the autopilot is turned on, or crw set <project> \
                         permission_mode <mode|default>",
                    );
                limit_pickers(ui, name, &project.autopilot, actions);
            });
            let standing = self.autopilot.standing.get(&name.to_lowercase());
            let (others, idle) = self.open_elsewhere(name);
            let (amber, weak, green) = (
                style::amber(ui),
                ui.visuals().weak_text_color(),
                style::green(ui),
            );
            let mut refused = false;
            for refusal in standing.into_iter().flatten() {
                refused = true;
                // The log's words carry every way out; the section names the window's.
                let words = match refusal {
                    Refusal::Waits(mode) => format!(
                        "The gate refuses its chains: its runs would wait for you at each shell \
                         command ({mode}, no allow rule); keep auto with the pin beside the launch \
                         dialog's Permissions."
                    ),
                    refusal => format!("The gate refuses its chains: {}.", refusal.words()),
                };
                verdict(ui, words, amber);
            }
            if !others.is_empty() {
                refused = true;
                let words = format!(
                    "A chain would end here now: {}.",
                    Refusal::Busy(others).words()
                );
                verdict(ui, words, amber);
            }
            if !idle.is_empty() {
                let words = format!(
                    "Open and idle, which does not stop a chain: {}.",
                    idle.join("; ")
                );
                verdict(ui, words, weak);
            }
            if !refused && standing.is_some() {
                let words = "Its runs would not wait, and no session there stops a chain.";
                verdict(ui, words.to_owned(), green);
            }
        }
    }
}

/// A chain's own five-hour mark as a small picker (EP-41), for the launch dialog's Chain plan
/// and the Autopilot section: the setting's entry, which holds the chain at the setting, then
/// the choices; a mark another way set is offered as it stands. The new mark when the owner
/// chose another, `Some(None)` for the setting.
pub(crate) fn mark_picker(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    mark: Option<u8>,
    setting: u8,
) -> Option<Option<u8>> {
    let words = |value: Option<u8>| match value {
        None => format!("{setting} % (the setting)"),
        Some(value) => format!("{value} %"),
    };
    let mut options: Vec<u8> = MARK_CHOICES
        .iter()
        .copied()
        .filter(|&value| value != setting)
        .collect();
    if let Some(own) = mark
        && !options.contains(&own)
    {
        options.push(own);
        options.sort_unstable();
    }
    let mut chosen = mark;
    egui::ComboBox::from_id_salt(id)
        .width(132.0)
        .selected_text(words(mark))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut chosen, None, words(None));
            for option in options {
                ui.selectable_value(&mut chosen, Some(option), words(Some(option)));
            }
        })
        .response
        .on_hover_text(
            "The share of the session (five-hour) window at which this chain holds until the \
             window resets (D-22): a higher mark lets it run further into the window. The chain \
             keeps it across the window's restarts and drops it when it ends; a chain without \
             its own holds at the setting (crw setting autopilot_five_hour_percent).",
        );
    (chosen != mark).then_some(chosen)
}

/// A line of a project's verdict in the Autopilot section, small and wrapped.
fn verdict(ui: &mut egui::Ui, words: String, colour: egui::Color32) {
    ui.add(egui::Label::new(RichText::new(words).small().color(colour)).wrap());
}

/// A project's limits as small pickers (D-22; EP-19b): the largest brief a chain starts, the
/// chained runs a day and the budget of each run, the default taken out when chosen. `crw set`
/// takes any figure; one the pickers do not list is offered as it stands.
fn limit_pickers(
    ui: &mut egui::Ui,
    name: &str,
    limits: &registry::Autopilot,
    actions: &mut Vec<Action>,
) {
    let size = limits.max_size();
    let mut chosen = size;
    egui::ComboBox::from_id_salt(("autopilot-size", name))
        .width(64.0)
        .selected_text(format!("up to {size}"))
        .show_ui(ui, |ui| {
            for option in [Size::S, Size::M, Size::L] {
                let label = if option == autopilot::DEFAULT_MAX_SIZE {
                    format!("{option} (default)")
                } else {
                    option.to_string()
                };
                ui.selectable_value(&mut chosen, option, label);
            }
        })
        .response
        .on_hover_text("The largest brief a chain starts (D-22); a larger one needs your click");
    if chosen != size {
        let limit = Limit::MaxSize((chosen != autopilot::DEFAULT_MAX_SIZE).then_some(chosen));
        actions.push(Action::SetAutopilotLimit(name.to_owned(), limit));
    }

    let daily = limits.daily();
    let mut chosen = daily;
    egui::ComboBox::from_id_salt(("autopilot-daily", name))
        .width(64.0)
        .selected_text(format!("{daily} a day"))
        .show_ui(ui, |ui| {
            let mut options = DAILY_CHOICES.to_vec();
            if !options.contains(&daily) {
                options.push(daily);
                options.sort_unstable();
            }
            for option in options {
                let label = if option == autopilot::DEFAULT_DAILY {
                    format!("{option} (default)")
                } else {
                    option.to_string()
                };
                ui.selectable_value(&mut chosen, option, label);
            }
        })
        .response
        .on_hover_text(
            "The chained runs a day in this project (D-22); the total in all projects, set \
             above, holds too: a chain ends at whichever it meets first",
        );
    if chosen != daily {
        let limit = Limit::Daily((chosen != autopilot::DEFAULT_DAILY).then_some(chosen));
        actions.push(Action::SetAutopilotLimit(name.to_owned(), limit));
    }

    let budget = limits.max_budget_usd;
    let mut chosen = budget;
    egui::ComboBox::from_id_salt(("autopilot-budget", name))
        .width(84.0)
        .selected_text(budget_words(budget))
        .show_ui(ui, |ui| {
            let mut options: Vec<Option<f64>> = std::iter::once(None)
                .chain(BUDGET_CHOICES.iter().map(|&dollars| Some(dollars)))
                .collect();
            if !options.contains(&budget) {
                options.push(budget);
            }
            for option in options {
                ui.selectable_value(&mut chosen, option, budget_words(option));
            }
        })
        .response
        .on_hover_text(
            "--max-budget-usd for each of its runs, in dollars of the CLI's own estimate; a run \
             that reaches it stops, which ends the chain",
        );
    if chosen != budget {
        actions.push(Action::SetAutopilotLimit(
            name.to_owned(),
            Limit::Budget(chosen),
        ));
    }
}

/// The chained runs a day in all projects as a small picker (D-22; EP-31b): the choices, the
/// default labelled, a total set another way (`crw setting autopilot_daily_total`) offered as it
/// stands. The new total when the owner chose another, `Some(None)` for the default, which the
/// write takes out of the registry.
fn total_picker(ui: &mut egui::Ui, total: u32) -> Option<Option<u32>> {
    let mut chosen = total;
    egui::ComboBox::from_id_salt("autopilot-daily-total")
        .width(48.0)
        .selected_text(total.to_string())
        .show_ui(ui, |ui| {
            let mut options = TOTAL_CHOICES.to_vec();
            if !options.contains(&total) {
                options.push(total);
                options.sort_unstable();
            }
            for option in options {
                let label = if option == autopilot::DEFAULT_DAILY_TOTAL {
                    format!("{option} (default)")
                } else {
                    option.to_string()
                };
                ui.selectable_value(&mut chosen, option, label);
            }
        })
        .response
        .on_hover_text(
            "The chained runs a day in all projects (D-22), counted from the chain log on the \
             local day as the gate counts them: a chain ends once the day's total is reached, \
             as it ends at its project's own runs a day; raising it later does not bring an \
             ended chain back. crw setting autopilot_daily_total <n|default> sets it too.",
        );
    (chosen != total).then_some((chosen != autopilot::DEFAULT_DAILY_TOTAL).then_some(chosen))
}

/// The day's line as one sentence (EP-31b), the phone's block's, and whether the count is at or
/// over the total: the Autopilot section draws the same words around its picker.
pub(crate) fn day_line(count: Option<u32>, total: u32) -> (String, bool) {
    match count {
        Some(count) if count >= total => (
            format!("{count} of {total} {TODAY_WORDS}; {BOTH_CAPS}. {REACHED}"),
            true,
        ),
        Some(count) => (
            format!("{count} of {total} {TODAY_WORDS}; {BOTH_CAPS}."),
            false,
        ),
        None => (format!("{total} {UNREAD_TODAY}; {BOTH_CAPS}."), false),
    }
}

/// The day's total's change in words, for the status bar (EP-31b).
fn total_words(total: Option<u32>) -> String {
    let runs = total.unwrap_or(autopilot::DEFAULT_DAILY_TOTAL);
    let default = if total.is_none() { ", the default" } else { "" };
    format!(
        "The autopilot: {runs} chained runs a day at most in all projects{default}; each \
         project's own runs a day holds too."
    )
}

/// A budget as the picker says it: `no budget`, `$20 a run`.
fn budget_words(budget: Option<f64>) -> String {
    match budget {
        None => "no budget".to_owned(),
        Some(dollars) => format!("{} a run", dollars_text(dollars)),
    }
}

/// Dollars without cents when whole: `$20`, `$12.50`.
fn dollars_text(dollars: f64) -> String {
    if dollars.fract() == 0.0 {
        format!("${dollars:.0}")
    } else {
        format!("${dollars:.2}")
    }
}

/// A limit's change in words, for the status bar (EP-19b).
fn limit_words(limit: &Limit) -> String {
    match limit {
        Limit::MaxSize(Some(size)) => format!("the autopilot chains briefs up to {size}"),
        Limit::MaxSize(None) => format!(
            "the autopilot chains briefs up to {}, the default",
            autopilot::DEFAULT_MAX_SIZE
        ),
        Limit::Daily(Some(runs)) => format!("{runs} chained runs a day at most"),
        Limit::Daily(None) => format!(
            "{} chained runs a day at most, the default",
            autopilot::DEFAULT_DAILY
        ),
        Limit::Budget(Some(dollars)) => format!(
            "each run of the autopilot's has a budget of {} (--max-budget-usd)",
            dollars_text(*dollars)
        ),
        Limit::Budget(None) => "the autopilot's runs have no budget".to_owned(),
    }
}

/// Whether a queued start may leave its project's idle sessions out (EP-19b's review): yes once
/// the poll has answered after `queued_at` (it lists a tab the owner prompted as busy), `None`
/// to wait for that until `CONFIRM_FOR` has passed, then no: they count.
fn idle_confirmed(
    polled: Option<DateTime<Utc>>,
    queued_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Option<bool> {
    if polled.is_some_and(|at| at > queued_at) {
        Some(true)
    } else if now - queued_at < CONFIRM_FOR {
        None
    } else {
        Some(false)
    }
}

/// The idle sessions a chained line names (EP-19b), or nothing.
fn idle_words(idle: &[String]) -> String {
    if idle.is_empty() {
        String::new()
    } else {
        format!("; open there and idle, not counted: {}", idle.join("; "))
    }
}

/// The offer's contents (D-18; EP-19b): why the project's runs would wait, what `auto` means and
/// what it means for the project's commits, and the three ways on; the owner's answer, once
/// given.
fn offer_ui(ui: &mut egui::Ui, offer: &Offer) -> Option<Action> {
    ui.heading(format!("Turn on the autopilot of {}?", offer.project));
    ui.add_space(4.0);
    ui.add(
        egui::Label::new(format!(
            "Its headless runs would wait for you at each shell command: its permission mode is \
             {} and no allow rule lets a command through, in its .claude settings or yours. The \
             gate refuses to chain a brief whose run would wait (D-22), so no chain would start.",
            offer.mode
        ))
        .wrap(),
    );
    ui.add_space(4.0);
    ui.add(egui::Label::new(AUTO_WORDS).wrap());
    ui.add_space(4.0);
    ui.add(egui::Label::new(PUSH_WORDS).wrap());
    ui.add_space(8.0);
    let mut chosen = None;
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Cancel").clicked() {
                chosen = Some(Action::CloseOffer);
            }
            if ui
                .button("Turn on as it is")
                .on_hover_text(
                    "Its runs commit themselves (your standing yes); the gate refuses its chains \
                     while its runs would wait",
                )
                .clicked()
            {
                chosen = Some(Action::AutopilotOn {
                    project: offer.project.clone(),
                    auto: false,
                });
            }
            if ui
                .button(RichText::new("Set auto and turn on").strong())
                .on_hover_text(
                    "Writes permission_mode = \"auto\" under the project's [project.launch] and \
                     turns its autopilot on, in one change of the registry",
                )
                .clicked()
            {
                chosen = Some(Action::AutopilotOn {
                    project: offer.project.clone(),
                    auto: true,
                });
            }
        });
    });
    chosen
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

#[cfg(test)]
mod tests {
    use crw_core::registry::LaunchMode;
    use crw_core::sessions::{Mode, Seen};

    use super::*;
    use crate::board::{Places, View};
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::testing::{self, Screen};

    /// A board over the fake portfolio's registry in `scratch`, as a screenshot run has it: it
    /// reads no user settings, so the fixtures' runs would wait (no allow rule anywhere).
    fn board(scratch: &Scratch) -> Board {
        let places = Places {
            store: fake_store(scratch),
            ledger: None,
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        let mut board = Board::new(places, &ctx, View::default(), egui::ThemePreference::Light);
        board.fixtures = true;
        board
    }

    fn harbor(board: &Board) -> Project {
        board
            .store()
            .load()
            .unwrap()
            .registry
            .project("harbor")
            .cloned()
            .unwrap()
    }

    /// A run's end reads its project again on its own (EP-25a): a canned run of harbor that
    /// ended asks the loader for harbor and the row says so once read; a chat's turn asks for
    /// nothing, nor does a run still going.
    #[test]
    fn a_runs_end_reads_its_project_again() {
        use crw_core::session::{Ended, Event};

        use crate::portfolio::Moment;
        use crate::sessions::Live;

        let scratch = Scratch::new("autopilot-reread");
        let mut board = board(&scratch);
        // The real path, not a screenshot run's.
        board.fixtures = false;
        let ended = |outcome| {
            Event::Exited(Ended {
                outcome,
                exit_code: Some(0),
                result: None,
                stderr: Vec::new(),
            })
        };
        let launch = |project: &str, ep: Option<&str>| {
            let mut launch = Launch::new(project, std::path::Path::new(project), "fable", "go");
            launch.ep = ep.map(str::to_owned);
            launch
        };
        let mut run = Live::canned(
            31,
            launch("harbor", Some("EP-4")),
            "high",
            vec![ended(Outcome::Completed)],
        );
        run.chained = true;
        let going = Live::canned(32, launch("lantern", Some("EP-2")), "high", Vec::new());
        let mut chat_launch = launch("chat", None);
        chat_launch.conversation = Some("What is next?".to_owned());
        let chat = Live::canned(33, chat_launch, "high", vec![ended(Outcome::Completed)]);
        board.sessions.live.extend([run, going, chat]);
        board.reread_after_run(32);
        board.reread_after_run(33);
        assert!(
            board.loader.rereading.is_empty(),
            "a chat's turn and a run going on"
        );
        board.reread_after_run(31);
        assert!(board.loader.rereading.contains("harbor"));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while (board.loader.rereading.contains("harbor") || board.loader.reading_files)
            && std::time::Instant::now() < deadline
        {
            board.loader.receive(&mut board.portfolio);
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let harbor = board.portfolio.project("harbor").expect("harbor read");
        let reread = harbor
            .reread
            .as_ref()
            .expect("the row says it was read again");
        assert_eq!(reread.moment, Moment::RunEnded { chained: true });
        assert_eq!(reread.what, "the files");
        assert!(board.portfolio.project("lantern").unwrap().reread.is_none());
        assert_eq!(
            board.loader.reread_notice.take(),
            Some(Ok(
                "harbor read again after its chained run ended: the files".to_owned()
            ))
        );
    }

    /// A gate read for harbor after EP-3 with `refusals`, its next brief drafted as the worker
    /// drafts it.
    fn gate_read(board: &Board, refusals: Vec<Refusal>) -> Gated {
        let registry = board.store().load().unwrap().registry;
        let entry = registry.project("harbor").cloned().unwrap();
        let data_root = registry.data_root();
        let folder = entry.folder(&data_root);
        let snapshot = crw_core::roadmap::snapshot::read_tree(&folder);
        let usage = rules::Usage::unread(Utc::now());
        let draft = crw_core::prompt::draft(
            &registry,
            &entry,
            &snapshot,
            None,
            &[],
            &usage,
            "2026-11-10",
        )
        .ok();
        Gated {
            project: "harbor".to_owned(),
            after: "EP-3".parse().ok(),
            refusals,
            draft,
            entry,
            folder,
            data_root,
            program: PathBuf::from("claude.exe"),
        }
    }

    /// A board between two chained runs with a newer build installed (EP-37a): not a screenshot
    /// run, its chain log kept in the window.
    fn between_runs(scratch: &Scratch) -> Board {
        let mut board = board(scratch);
        board.fixtures = false;
        board.autopilot.canned = true;
        crate::restart::tests::installed(&mut board, scratch);
        board
    }

    /// The gate's moment (EP-37a): a passing gate with a newer build installed holds the chain for
    /// the restart and restarts, the handover carrying the chain; a refused restart lets the chain
    /// go on in this window as before.
    #[test]
    fn a_pending_install_is_taken_at_the_gates_moment() {
        let scratch = Scratch::new("autopilot-gate-restart");
        let mut board = between_runs(&scratch);
        assert!(board.restart.pending_install());
        let gated = gate_read(&board, Vec::new());
        let next = gated.draft.as_ref().unwrap().ep;
        board.gated(gated);
        assert!(board.autopilot.queued.is_none(), "the new window starts it");
        assert!(board.restart.closing_at.is_some(), "the window restarts");
        let words = format!(
            "held after EP-3 for the window's restart into the installed build: {next} starts in \
             the new window"
        );
        assert_eq!(
            board.autopilot.log[0].detail.as_deref(),
            Some(words.as_str())
        );
        let local = scratch.0.join("local");
        let crw_core::handover::Taken::Ours(found) =
            crw_core::handover::take(&local, Some(std::process::id()))
        else {
            panic!("the handover carries the chain");
        };
        assert_eq!(found.held.len(), 1);
        assert_eq!(found.held[0].words, words);

        // Something else in the way: the chain goes on here, the log saying why the build waits.
        let refused = Scratch::new("autopilot-gate-refused");
        let mut board = between_runs(&refused);
        board.offer = Some(Offer {
            project: "meadow".to_owned(),
            mode: "acceptEdits".to_owned(),
        });
        board.gated(gate_read(&board, Vec::new()));
        assert!(board.restart.closing_at.is_none());
        assert!(board.autopilot.held.is_empty());
        assert_eq!(board.autopilot.queued.as_ref().map(|q| q.next), Some(next));
        assert_eq!(
            board.autopilot.log[0].detail.as_deref(),
            Some("the restart into the installed build waits: the offer of auto is open")
        );
    }

    /// IN-14 (EP-30b): a registry this build cannot read (a key a newer build wrote) at a held
    /// chain's re-read, silent before, holds the chain with words naming the failure and the way
    /// out, said once; with a newer build installed the window restarts into it, the chain
    /// carried. A run's end then holds a completed run's chain and ends a failed one's.
    #[test]
    fn an_unread_registry_holds_the_chain_once_and_takes_a_newer_build() {
        let scratch = Scratch::new("autopilot-unread");
        let mut board = board(&scratch);
        board.fixtures = false;
        board.autopilot.canned = true;
        board.autopilot.held.push(Held {
            project: "harbor".to_owned(),
            after: "EP-3".parse().ok(),
            words: "held after EP-3 until the pause lifts: the autopilot is paused (your Pause)"
                .to_owned(),
        });
        let registry = scratch.0.join("registry.toml");
        let text = std::fs::read_to_string(&registry).unwrap();
        std::fs::write(
            &registry,
            text.replace("[settings]", "[settings]\nkey_of_a_newer_build = \"x\""),
        )
        .unwrap();
        assert!(board.store().load().is_err(), "this build refuses the key");
        board.read_held();
        let words = board.autopilot.held[0].words.clone();
        assert!(
            words.starts_with("held after EP-3: the registry did not read ("),
            "{words}"
        );
        assert!(
            words.ends_with(
                "File > Install and restart opens a newer build, whose gate weighs this chain"
            ),
            "{words}"
        );
        assert!(!words.contains('\n'), "one line: {words}");
        // The parser's message alone: never the file's line with its value, nor its path.
        assert!(
            !words.contains("\"x\"") && !words.contains("registry.toml"),
            "{words}"
        );
        assert_eq!(board.autopilot.log.len(), 1);
        assert_eq!(
            board.autopilot.log[0].detail.as_deref(),
            Some(words.as_str())
        );
        // Read again: said once.
        board.autopilot.recheck = true;
        board.read_held();
        assert_eq!(board.autopilot.log.len(), 1, "no second line");
        assert!(board.restart.closing_at.is_none());
        // A newer build installed: the next re-read restarts into it, the chain carried.
        crate::restart::tests::installed(&mut board, &scratch);
        board.autopilot.recheck = true;
        board.read_held();
        assert!(board.restart.closing_at.is_some(), "the window restarts");
        assert_eq!(
            board.autopilot.log[0].detail.as_deref(),
            Some(autopilot::UNREAD_RESTART)
        );
        let crw_core::handover::Taken::Ours(found) =
            crw_core::handover::take(&scratch.0.join("local"), Some(std::process::id()))
        else {
            panic!("the handover carries the chain");
        };
        assert_eq!(found.held.len(), 1);
        assert_eq!(found.held[0].words, words);

        // A run's end with the registry unread: a completed chained run's chain is held, a
        // failed one's ends, whatever the registry would say.
        let ended = Scratch::new("autopilot-unread-end");
        drop(board);
        let mut board = self::board(&ended);
        board.fixtures = false;
        board.autopilot.canned = true;
        let registry = ended.0.join("registry.toml");
        let text = std::fs::read_to_string(&registry).unwrap();
        std::fs::write(
            &registry,
            text.replace("[settings]", "[settings]\nkey_of_a_newer_build = \"x\""),
        )
        .unwrap();
        let ended_run = |key: u64, outcome: Outcome| {
            let mut launch = Launch::new("harbor", std::path::Path::new("harbor"), "fable", "go");
            launch.ep = Some("EP-3".to_owned());
            let mut live = crate::sessions::Live::canned(
                key,
                launch,
                "high",
                vec![crw_core::session::Event::Exited(crw_core::session::Ended {
                    outcome,
                    exit_code: Some(0),
                    result: None,
                    stderr: Vec::new(),
                })],
            );
            live.chained = true;
            live
        };
        board.sessions.live.push(ended_run(41, Outcome::Completed));
        board.weigh_run(41);
        assert_eq!(board.autopilot.held.len(), 1);
        assert!(
            board.autopilot.held[0]
                .words
                .contains("the registry did not read")
        );
        board.autopilot.held.clear();
        board.sessions.live.push(ended_run(42, Outcome::Failed));
        board.weigh_run(42);
        assert!(board.autopilot.held.is_empty(), "a failed run's chain ends");
        assert!(
            board.autopilot.log[0]
                .detail
                .as_deref()
                .is_some_and(|line| line.starts_with("the chain ends after EP-3: the run ended")),
            "{:?}",
            board.autopilot.log[0]
        );
    }

    /// A chain that ends restarts nothing (the owner's choice at EP-37a's pickup); one the pause
    /// holds is carried across the restart.
    #[test]
    fn a_chains_end_restarts_nothing_and_a_held_one_is_carried() {
        let scratch = Scratch::new("autopilot-gate-end");
        let mut board = between_runs(&scratch);
        let next: EpId = "EP-4".parse().unwrap();
        board.gated(gate_read(&board, vec![Refusal::Replan(next)]));
        assert!(board.restart.closing_at.is_none());
        assert!(!crw_core::handover::path(&scratch.0.join("local")).exists());
        assert!(board.autopilot.held.is_empty());
        // Held by the pause: the window restarts, the held chain in the handover.
        board.gated(gate_read(&board, vec![Refusal::Paused]));
        assert!(board.restart.closing_at.is_some());
        let crw_core::handover::Taken::Ours(found) =
            crw_core::handover::take(&scratch.0.join("local"), Some(std::process::id()))
        else {
            panic!("the held chain is carried");
        };
        assert!(
            found.held[0]
                .words
                .starts_with("held after EP-3 until the pause lifts")
        );
    }

    /// The restarted window's carried chains (EP-37a): taken from the handover, their gates wait
    /// for the first poll of live sessions, then are read at once. The project is one the
    /// registry no longer lists, so the read lets the chain go without a gate, whose reading
    /// would ask the CLI (rule 5).
    #[test]
    fn a_carried_chain_waits_for_the_first_poll() {
        let scratch = Scratch::new("autopilot-carried");
        let mut board = board(&scratch);
        board.fixtures = false;
        board.autopilot.canned = true;
        let local = scratch.0.join("local");
        let held = Held {
            project: "nowhere".to_owned(),
            after: "EP-3".parse().ok(),
            words: "held after EP-3 for the window's restart".to_owned(),
        };
        let written = crw_core::handover::Handover::new(
            4242,
            Utc::now().to_rfc3339(),
            vec![held.clone()],
            Vec::new(),
            false,
            None,
            None,
        );
        crw_core::handover::write(&local, &written).unwrap();
        board.take_handover(Some(&local), Some(4242), " of 2026-11-10 16:40");
        assert_eq!(
            board.notice_text(),
            Some(
                "Restarted into the installed build of 2026-11-10 16:40, carrying 1 held chain (nowhere)"
            )
        );
        assert_eq!(board.autopilot.held, vec![held]);
        assert!(board.autopilot.carried.is_some());
        // No poll yet: the gate is not read.
        board.read_held();
        assert!(board.autopilot.held_read.is_none() && board.autopilot.carried.is_some());
        // The poll answered, the files not yet read (the review of EP-37a): still waiting.
        board.watch.model.polled = Some(Utc::now());
        board.loader.reading_files = true;
        board.read_held();
        assert!(board.autopilot.held_read.is_none() && board.autopilot.carried.is_some());
        assert_eq!(board.autopilot.held.len(), 1);
        // The first files read landed: read at once.
        board.portfolio = crate::portfolio::read_files(board.store());
        board.loader.reading_files = false;
        board.read_held();
        assert!(board.autopilot.carried.is_none() && board.autopilot.held_read.is_some());
        assert!(
            board.autopilot.held.is_empty(),
            "a project no longer listed lets go"
        );
        // A file no restart took (another version) is dropped, the notice and the log saying what
        // was lost, even in a restarted window.
        let mut other = written.clone();
        other.version = crw_core::handover::VERSION + 1;
        crw_core::handover::write(&local, &other).unwrap();
        board.take_handover(Some(&local), Some(7), "");
        let lost =
            "a handover no restart took was dropped, its chains lost: a handover of version 2";
        assert!(
            board
                .notice_text()
                .is_some_and(|text| text
                    .starts_with(&format!("Restarted into the installed build, but {lost}"))),
            "{:?}",
            board.notice_text()
        );
        assert!(
            board.autopilot.log[0]
                .detail
                .as_deref()
                .is_some_and(|line| line.starts_with(lost))
        );
        assert!(!crw_core::handover::path(&local).exists());
    }

    #[test]
    fn turning_the_autopilot_on_where_runs_would_wait_offers_auto_first() {
        let scratch = Scratch::new("autopilot-offer");
        let mut board = board(&scratch);
        // harbor has no settings file and the default mode: its runs would wait.
        assert!(
            !board.set_autopilot("harbor", true),
            "nothing is written yet"
        );
        assert_eq!(
            board.offer,
            Some(Offer {
                project: "harbor".to_owned(),
                mode: "acceptEdits".to_owned(),
            })
        );
        assert!(!harbor(&board).autopilot.enabled);
        // The offer's yes: the mode and the switch in one write.
        assert!(board.autopilot_on("harbor", true));
        assert_eq!(board.offer, None);
        let entry = harbor(&board);
        assert!(entry.autopilot.enabled);
        assert_eq!(entry.launch.permission_mode.as_deref(), Some("auto"));
        assert_eq!(
            entry.launch.mode,
            Some(LaunchMode::Headless),
            "the owner's [project.launch] keeps its other keys"
        );
        assert_eq!(
            board.autopilot.log[0].detail.as_deref(),
            Some("turned on by the owner, with permission mode auto")
        );
        // Its runs no longer wait: off and on again write at once, with no offer.
        assert!(board.set_autopilot("harbor", false));
        assert!(board.set_autopilot("harbor", true));
        assert_eq!(board.offer, None);
        assert!(harbor(&board).autopilot.enabled);
        // Turned on as it is, a waiting project's switch is written without the mode.
        assert!(!board.set_autopilot("lantern", true));
        assert!(board.autopilot_on("lantern", false));
        let registry = board.store().load().unwrap().registry;
        let lantern = registry.project("lantern").unwrap();
        assert!(lantern.autopilot.enabled);
        assert_eq!(lantern.launch.permission_mode, None);
        // The section's verdicts: harbor's runs would not wait, lantern's would.
        let standing = board.standing_of(&registry);
        assert_eq!(standing.get("harbor"), Some(&Vec::new()));
        assert_eq!(
            standing.get("lantern"),
            Some(&vec![Refusal::Waits("acceptEdits".to_owned())])
        );
    }

    #[test]
    fn a_kept_permission_mode_is_written_and_taken_back() {
        let scratch = Scratch::new("autopilot-keep");
        let mut board = board(&scratch);
        assert!(board.keep_permission_mode("harbor", Some("auto")));
        assert_eq!(
            harbor(&board).launch.permission_mode.as_deref(),
            Some("auto")
        );
        assert!(board.keep_permission_mode("harbor", None));
        let entry = harbor(&board);
        assert_eq!(entry.launch.permission_mode, None);
        assert_eq!(entry.launch.mode, Some(LaunchMode::Headless));
        // A limit, and its default taken out.
        assert!(board.set_autopilot_limit("harbor", &Limit::Daily(Some(4))));
        assert_eq!(harbor(&board).autopilot.daily(), 4);
        assert!(board.set_autopilot_limit("harbor", &Limit::Daily(None)));
        assert_eq!(harbor(&board).autopilot.daily, None);
    }

    /// The day's total (EP-31b): the day's chained starts in all projects read from the chain
    /// log as the gate counts them, and the total from the registry; the section's line with its
    /// picker, amber with a line of its own once the total is reached; the picker's action writes
    /// the setting as `crw setting` does, its default taken out, and a total set from a terminal
    /// counts at the next read.
    #[test]
    fn the_days_total_is_read_said_and_set() {
        let scratch = Scratch::new("autopilot-day-total");
        let ledger = scratch.0.join("ledger.sqlite");
        let places = Places {
            store: fake_store(&scratch),
            ledger: Some(ledger.clone()),
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        let mut board = Board::new(places, &ctx, View::default(), egui::ThemePreference::Light);
        board.fixtures = true;
        let set = |board: &Board, value: &str| {
            board
                .store()
                .change(&files::today(), |c| {
                    c.set_setting("autopilot_daily_total", value)
                })
                .unwrap();
        };
        set(&board, "3");
        let line = |project: &str, words: &str, at: DateTime<Utc>| {
            autopilot::log(&ledger, Some(project), words, at).unwrap();
        };
        let now = Utc::now();
        line(
            "harbor",
            "chained EP-4 after EP-3: fable at xhigh, M's cap of 1 h 30 min",
            now - chrono::TimeDelta::days(2),
        );
        line(
            "harbor",
            "chained EP-5 after EP-4: claude-opus-5-5 at max, the rules' choice, M's cap of 1 h \
             30 min",
            now,
        );
        line(
            "lantern",
            "chained EP-2 after EP-1: fable at high, the rules' choice, S's cap of 1 h 15 min",
            now,
        );
        line(
            "harbor",
            "the chain ends after EP-5: EP-6 is a re-plan, the owner's to run",
            now,
        );
        board.read_pilot();
        assert_eq!(
            board.autopilot.chained_today,
            Some(2),
            "the day's two starts"
        );
        assert_eq!(board.autopilot.daily_total, 3);
        assert_eq!(board.autopilot.log.len(), 4);
        assert_eq!(board.autopilot.problem, None);

        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| board.autopilot_section(ui));
        assert!(testing::text_at(&output, "2 of").is_some());
        assert!(
            testing::text_at(&output, "3").is_some(),
            "the picker's total"
        );
        assert!(testing::text_at(&output, "chained runs today in all projects;").is_some());
        assert!(testing::text_at(&output, BOTH_CAPS).is_some());
        assert!(testing::text_at(&output, REACHED).is_none());

        // A third start today reaches the total: the line is amber and says so.
        line(
            "quarry",
            "chained EP-7 after EP-6: fable at high, the rules' choice, M's cap of 1 h 30 min",
            Utc::now(),
        );
        board.autopilot.stale = true;
        board.read_pilot();
        assert_eq!(board.autopilot.chained_today, Some(3));
        let output = screen.pass(Vec::new(), |ui| board.autopilot_section(ui));
        assert!(testing::text_at(&output, "3 of").is_some());
        assert!(testing::text_at(&output, REACHED).is_some(), "reached");

        // The picker's action: the setting written, then its default taken out.
        assert!(board.run(Action::SetDailyTotal(Some(6))));
        let total = |board: &Board| {
            board
                .store()
                .load()
                .unwrap()
                .registry
                .settings
                .autopilot_daily_total
        };
        assert_eq!(total(&board), Some(6));
        assert_eq!(board.autopilot.daily_total, 6);
        assert_eq!(
            board.notice_text(),
            Some(
                "The autopilot: 6 chained runs a day at most in all projects; each project's own \
                 runs a day holds too."
            )
        );
        assert!(board.run(Action::SetDailyTotal(None)));
        assert_eq!(total(&board), None, "the default taken out of the registry");
        assert_eq!(board.autopilot.daily_total, autopilot::DEFAULT_DAILY_TOTAL);
        // A total set from a terminal counts at the next read.
        set(&board, "7");
        board.autopilot.stale = true;
        board.read_pilot();
        assert_eq!(board.autopilot.daily_total, 7);
    }

    #[test]
    fn a_kept_model_or_effort_is_written_and_taken_back() {
        use rules::{Kept, Key, Scope};

        let scratch = Scratch::new("autopilot-keep-rule");
        let mut board = board(&scratch);
        let p1 = Scope::Phase("P1".to_owned());
        assert!(board.keep_rule(
            "harbor",
            &Scope::Project,
            Key::Model,
            Some("claude-opus-5-5")
        ));
        assert_eq!(
            board.notice_text(),
            Some("harbor: model claude-opus-5-5 kept for this project.")
        );
        assert!(board.keep_rule("harbor", &p1, Key::Effort, Some("high")));
        let entry = harbor(&board);
        // The phase's rule comes before the one for every brief.
        assert_eq!(rules::kept_rule(&entry, &p1).map(|(n, _)| n), Some(0));
        assert_eq!(
            Kept::of(&entry, Some("P1"), Key::Model).project.as_deref(),
            Some("claude-opus-5-5")
        );
        assert_eq!(
            Kept::of(&entry, Some("P1"), Key::Effort).phase.as_deref(),
            Some("high")
        );
        // Back to the rules takes each out, and the rules with them.
        assert!(board.keep_rule("harbor", &Scope::Project, Key::Model, None));
        assert_eq!(
            board.notice_text(),
            Some("harbor: model back to the rules for this project.")
        );
        assert!(board.keep_rule("harbor", &p1, Key::Effort, None));
        let entry = harbor(&board);
        assert!(entry.rules.is_empty());
        assert_eq!(entry.launch.mode, Some(LaunchMode::Headless));
        // A value of two words is refused, and nothing is written.
        assert!(board.keep_rule("harbor", &Scope::Project, Key::Model, Some("opus five")));
        assert!(harbor(&board).rules.is_empty());
    }

    #[test]
    fn the_offer_says_why_and_takes_the_owners_answer() {
        let offer = Offer {
            project: "harbor".to_owned(),
            mode: "acceptEdits".to_owned(),
        };
        let mut screen = Screen::new();
        let mut answer = None;
        let mut pass = |screen: &mut Screen, events| {
            screen.pass(events, |ui| {
                if let Some(action) = offer_ui(ui, &offer) {
                    answer = Some(action);
                }
            })
        };
        let output = pass(&mut screen, Vec::new());
        let at = testing::text_at(&output, "Set auto and turn on").expect("the offer's yes");
        pass(&mut screen, testing::moved(at));
        pass(
            &mut screen,
            testing::button(at, egui::PointerButton::Primary, true),
        );
        pass(
            &mut screen,
            testing::button(at, egui::PointerButton::Primary, false),
        );
        assert_eq!(
            answer,
            Some(Action::AutopilotOn {
                project: "harbor".to_owned(),
                auto: true,
            })
        );
    }

    #[test]
    fn an_idle_session_no_longer_stops_a_chain_and_is_named() {
        let scratch = Scratch::new("autopilot-idle");
        let mut board = board(&scratch);
        let now = Utc::now();
        let ago = |minutes| now - chrono::TimeDelta::minutes(minutes);
        let busy = Seen::canned(
            "a1",
            Some("harbor"),
            Mode::Visual,
            Status::Busy,
            None,
            ago(3),
        );
        let idle = Seen::canned(
            "b2",
            Some("harbor"),
            Mode::Manual,
            Status::Idle,
            None,
            ago(40),
        );
        let waiting = Seen::canned(
            "c3",
            Some("quarry"),
            Mode::Manual,
            Status::Waiting,
            Some("a permission prompt"),
            ago(5),
        );
        for mut seen in [busy, idle, waiting] {
            seen.heard = now;
            board.watch.model.insert(seen);
        }
        let (refusing, idle) = board.gate_sessions("harbor");
        assert_eq!(refusing.len(), 1, "{refusing:?}");
        assert!(refusing[0].contains("busy"), "{refusing:?}");
        assert_eq!(idle.len(), 1, "{idle:?}");
        assert!(idle[0].contains("idle"), "{idle:?}");
        let (refusing, idle) = board.gate_sessions("quarry");
        assert_eq!(
            (refusing.len(), idle.len()),
            (1, 0),
            "a waiting one refuses"
        );
        assert_eq!(idle_words(&[]), "");
        assert_eq!(
            idle_words(&["a manual session is idle (since 13:11)".to_owned()]),
            "; open there and idle, not counted: a manual session is idle (since 13:11)"
        );
    }

    #[test]
    fn an_idle_session_is_left_out_once_a_newer_poll_confirms_it() {
        let queued = Utc::now();
        let seconds = chrono::TimeDelta::seconds;
        // A poll after the queue confirms; none yet waits; none within the time counts them.
        assert_eq!(
            idle_confirmed(Some(queued + seconds(5)), queued, queued + seconds(6)),
            Some(true)
        );
        assert_eq!(
            idle_confirmed(Some(queued - seconds(1)), queued, queued + seconds(30)),
            None
        );
        assert_eq!(idle_confirmed(None, queued, queued + seconds(74)), None);
        assert_eq!(
            idle_confirmed(Some(queued - seconds(1)), queued, queued + seconds(76)),
            Some(false)
        );
    }

    #[test]
    fn the_chained_line_names_the_idle_sessions() {
        let scratch = Scratch::new("autopilot-chained-idle");
        let mut board = board(&scratch);
        board.autopilot.starting = Some(Queued {
            project: "harbor".to_owned(),
            after: Some(EpId::new(4)),
            next: EpId::new(5),
            size: Some(Size::M),
            launch: Launch::new("harbor", std::path::Path::new("harbor"), "fable", "go"),
            effort: "xhigh".to_owned(),
            chosen: "fable at xhigh, the rules' choice".to_owned(),
            program: PathBuf::from("claude.exe"),
            idle: vec!["a manual session is idle (since 13:11)".to_owned()],
            queued_at: Utc::now(),
            mark: Some(70),
        });
        board.chain_started(Started::Running("a1".to_owned()));
        let line = board.autopilot.log[0].detail.clone().unwrap();
        assert!(
            line.starts_with("chained EP-5 after EP-4: fable at xhigh, the rules' choice"),
            "{line}"
        );
        // The chain's own mark is named (EP-41), then the idle sessions.
        assert!(
            line.ends_with(
                "; it holds at 70 % of the session (five-hour) window, your mark for this chain; \
                 open there and idle, not counted: a manual session is idle (since 13:11)"
            ),
            "{line}"
        );
    }

    /// A chain's own five-hour mark (EP-41): named in the log when its run starts, changed in
    /// the Autopilot section from then on with a line of its own, a held chain read again at
    /// once; a plan at the setting logs nothing at its start.
    #[test]
    fn a_chains_own_mark_is_logged_and_changed_from_the_autopilot_section() {
        let scratch = Scratch::new("autopilot-chain-mark");
        let mut board = board(&scratch);
        let plan = Plan {
            project: "harbor".to_owned(),
            after: EpId::new(4),
            briefs: Vec::new(),
            stop: None,
            daily: 2,
            five_hour_mark: None,
        };
        board.autopilot.keep_plan(plan.clone());
        board.log_own_mark("harbor");
        assert!(
            board.autopilot.log.is_empty(),
            "a plan at the setting logs nothing"
        );
        board.autopilot.keep_plan(Plan {
            five_hour_mark: Some(70),
            ..plan
        });
        board.log_own_mark("harbor");
        assert_eq!(
            board.autopilot.log[0].detail.as_deref(),
            Some(
                "the chain's mark: 70 % of the session (five-hour) window, your choice in its \
                 plan (the setting holds at 50 %)"
            )
        );
        // Held by the pause, the chain is read again at once after a change, which the section's
        // action brings without reading the files again (no file is written); a change while
        // its gate is read asks for a read after it.
        board.autopilot.held.push(Held {
            project: "harbor".to_owned(),
            after: Some(EpId::new(4)),
            words: "held after EP-4 until the pause lifts".to_owned(),
        });
        board.autopilot.reading.insert("harbor".to_owned());
        assert!(!board.run(Action::SetChainMark("harbor".to_owned(), Some(80))));
        assert_eq!(
            board.autopilot.plan_of("harbor").unwrap().five_hour_mark,
            Some(80)
        );
        assert!(board.autopilot.recheck);
        assert!(board.autopilot.recheck_after.contains("harbor"));
        board.autopilot.reading.clear();
        assert!(
            board.autopilot.log[0]
                .detail
                .as_deref()
                .is_some_and(
                    |line| line.starts_with("the chain's mark: 80 % of the session")
                        && line.contains("your change")
                ),
            "{:?}",
            board.autopilot.log[0]
        );
        // Back to the setting; the same mark again changes nothing; no plan, nothing.
        assert!(!board.set_chain_mark("harbor", None));
        assert_eq!(
            board.autopilot.plan_of("harbor").unwrap().five_hour_mark,
            None
        );
        let lines = board.autopilot.log.len();
        board.set_chain_mark("harbor", None);
        board.set_chain_mark("meadow", Some(60));
        assert_eq!(board.autopilot.log.len(), lines);
        assert!(board.autopilot.plan_of("meadow").is_none());

        // The owner starts the next brief by hand, its plan in the held chain's place: the held
        // chain gives way, logged as its end, and the new plan with its mark stays (the
        // review's finding).
        board.autopilot.keep_plan(Plan {
            five_hour_mark: Some(85),
            ..board.autopilot.plan_of("harbor").unwrap().clone()
        });
        board.started_by_hand("harbor", Some(EpId::new(5)));
        assert!(board.autopilot.held.is_empty());
        assert_eq!(
            board.autopilot.plan_of("harbor").unwrap().five_hour_mark,
            Some(85)
        );
        let newest: Vec<&str> = board.autopilot.log[..2]
            .iter()
            .filter_map(|row| row.detail.as_deref())
            .collect();
        assert!(
            newest[0].starts_with("the chain's mark: 85 %"),
            "{newest:?}"
        );
        assert_eq!(
            newest[1],
            "the chain ends after EP-4: you started EP-5 headless yourself"
        );
    }

    #[test]
    fn escape_puts_the_offer_away() {
        let scratch = Scratch::new("autopilot-offer-escape");
        let mut board = board(&scratch);
        board.offer = Some(Offer {
            project: "harbor".to_owned(),
            mode: "acceptEdits".to_owned(),
        });
        let escape = |pressed| egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        let mut screen = Screen::new();
        screen.pass(Vec::new(), |ui| board.offer_dialog(ui.ctx()));
        assert!(board.actions.is_empty(), "{:?}", board.actions);
        screen.pass(vec![escape(true)], |ui| board.offer_dialog(ui.ctx()));
        screen.pass(vec![escape(false)], |ui| board.offer_dialog(ui.ctx()));
        assert!(
            board.actions.contains(&Action::CloseOffer),
            "{:?}",
            board.actions
        );
    }

    #[test]
    fn the_limits_read_in_words() {
        assert_eq!(budget_words(None), "no budget");
        assert_eq!(budget_words(Some(20.0)), "$20 a run");
        assert_eq!(budget_words(Some(12.5)), "$12.50 a run");
        assert_eq!(
            limit_words(&Limit::MaxSize(Some(Size::S))),
            "the autopilot chains briefs up to S"
        );
        assert_eq!(
            limit_words(&Limit::Daily(None)),
            "2 chained runs a day at most, the default"
        );
        assert_eq!(
            limit_words(&Limit::Budget(None)),
            "the autopilot's runs have no budget"
        );
        // The day's line and the total's change (EP-31b).
        assert_eq!(
            day_line(Some(2), 8),
            (
                "2 of 8 chained runs today in all projects; a chain ends at this total or at its \
                 project's own runs a day, whichever it meets first."
                    .to_owned(),
                false
            )
        );
        let (line, reached) = day_line(Some(9), 8);
        assert!(reached && line.starts_with("9 of 8 ") && line.ends_with(REACHED));
        assert!(day_line(Some(0), 0).1, "a total of none is reached at once");
        assert_eq!(
            day_line(None, 4),
            (
                "4 chained runs a day in all projects (today's count did not read); a chain ends \
                 at this total or at its project's own runs a day, whichever it meets first."
                    .to_owned(),
                false
            )
        );
        assert_eq!(
            total_words(None),
            "The autopilot: 4 chained runs a day at most in all projects, the default; each \
             project's own runs a day holds too."
        );
    }
}
