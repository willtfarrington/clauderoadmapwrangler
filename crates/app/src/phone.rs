//! The window's side of the phone's forms (D-28; EP-30a). A page's form reaches the window as a
//! [`PageRequest`] through the server's channel, drained in each pass, the window shown or
//! hidden (`Board::pass`), and is run on the window's own thread with the desk's own calls: an
//! owner task ticked as the Detail pane's box ticks it, an inbox message ticked with the
//! disposition the form carries, the owner's message written by the compose dialog's write on a
//! worker thread, a question or a permission request answered by the Decisions tile's rules.
//! What came of it goes back to the page in one line, and to the status bar and the rolling log
//! with the caller's identity, so the desk sees what the phone did. An order whose page stopped
//! waiting is dropped unrun; a screenshot run refuses every order, as it refuses the desk's.
//!
//! EP-30b adds the orders that run things. A start of a project's next brief, headless with the
//! rules' choice, is the chained start's launch with what the dialog's Start headless adds; its
//! preview, for the start page, is the same reading, and a start is refused for the dialog's
//! reasons and, stricter than the dialog, for a session at work in the project. Its page waits
//! for the runner's answer. A Stop is the desk's graceful stop of the run the page names by its
//! number and its brief; Pause and Resume are the desk's own write.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use crw_core::autopilot::{self as chains, Cap, Plan, Refusal};
use crw_core::files;
use crw_core::inbox::mailman::{self, Sent};
use crw_core::prompt;
use crw_core::registry;
use crw_core::roadmap::EpId;
use crw_core::rules;
use crw_core::session::{Launch, cli::Preflight};
use tokio::sync::oneshot;

use crate::board::Board;
use crate::decisions::{self, Draft};
use crate::published::{Next, Start};
use crate::server::{AnswerForm, Order, PageRequest, Said};
use crate::sessions::Started;

/// What a form for a request that no longer waits says (the brief's "already answered").
const ALREADY: &str = "Already answered: the session no longer waits on that request.";

/// How long a message sent from the phone is remembered, so that a second tap or a form sent
/// again from a page not yet reloaded writes it once (EP-30a's review).
const SENT_KEPT: Duration = Duration::from_secs(300);

/// The phone's messages being written, and those written lately (EP-30a); its start on its way
/// (EP-30b).
#[derive(Default)]
pub struct Phone {
    sending: Vec<Sending>,
    /// By the project's lowercase name and the message's words: its id and when.
    sent: Vec<(String, String, String, Instant)>,
    starting: Option<Starting>,
}

/// The phone's start on its way to the runner (EP-30b): its project and brief, who sent it, the
/// words its page hears once it runs, and the page's reply.
struct Starting {
    project: String,
    ep: EpId,
    caller: String,
    words: String,
    reply: oneshot::Sender<Said>,
}

/// What a start from the phone runs once nothing refuses it (EP-30b): the launch, the effort in
/// words, what the autopilot asks of the run, the chain plan to keep, and the words its page
/// hears.
pub(crate) struct Ready {
    pub launch: Launch,
    pub effort: String,
    pub run: crate::sessions::Run,
    pub plan: Option<Plan>,
    pub preflight: Arc<Preflight>,
    pub ep: EpId,
    pub words: String,
}

impl Phone {
    /// Whether a message from the phone is being written.
    pub fn is_sending(&self) -> bool {
        !self.sending.is_empty()
    }

    /// Whether the phone's start waits for the runner's answer (EP-30b).
    pub fn is_starting(&self) -> bool {
        self.starting.is_some()
    }

    /// Why the same words to `project` are not written again: one being written, or one written
    /// within [`SENT_KEPT`].
    fn repeat(&mut self, project: &str, words: &str) -> Option<String> {
        self.sent.retain(|(.., at)| at.elapsed() < SENT_KEPT);
        let key = project.to_lowercase();
        if self
            .sending
            .iter()
            .any(|sending| sending.project.to_lowercase() == key && sending.words == words)
        {
            return Some(format!(
                "The same message to {project} is being written: it is not sent twice."
            ));
        }
        self.sent
            .iter()
            .find(|(name, said, ..)| *name == key && said == words)
            .map(|(_, _, id, _)| {
                format!(
                    "The same message went to {project} as {id} a moment ago: it is not sent \
                     twice. Change its words to send another."
                )
            })
    }
}

/// Where a message's write answers.
type Writing = Receiver<Result<Sent, String>>;

/// One message on its worker thread: whose, its words as written, who sent it, a session's words,
/// the write's answer and the page's reply.
struct Sending {
    project: String,
    words: String,
    caller: String,
    busy: Option<String>,
    answer: Writing,
    reply: oneshot::Sender<Said>,
}

impl Board {
    /// The start pages' asks for a preview (EP-30b), then the pages' forms since the last pass,
    /// each run and answered, then the phone's messages that were written meanwhile (EP-30a).
    pub fn obey_pages(&mut self) {
        for ask in self.watch.take_asks() {
            let preview = self.start_preview(&ask.project);
            let _ = ask.reply.send(preview);
        }
        let orders = self.watch.take_orders();
        self.obey_orders(orders);
    }

    /// Runs `orders` and answers each; an order whose page stopped waiting, or let it go at its
    /// ten seconds, is dropped unrun, as a restart request's is (EP-37). A write reads the files
    /// again, and the pages are built afresh at once, so the page a form's redirect brings shows
    /// what it changed (the review's). An order that waited two seconds or more for the window
    /// says in the rolling log what the window did meanwhile (EP-43).
    pub(crate) fn obey_orders(&mut self, orders: Vec<PageRequest>) {
        let mut wrote = false;
        let mut ran = false;
        for request in orders {
            let waited = self.pace.waited(request.sent);
            if request.reply.is_closed() || !request.hand.take() {
                let mut line = format!(
                    "A form from {} was dropped: its page stopped waiting.",
                    request.caller
                );
                if let Some(waited) = &waited {
                    line.push_str(&format!(" It {waited}"));
                }
                crw_core::log::line("notice", &line);
                continue;
            }
            if let Some(waited) = &waited {
                crw_core::log::line("slow", &format!("A form from {} {waited}", request.caller));
            }
            ran = true;
            let PageRequest {
                order,
                caller,
                reply,
                ..
            } = request;
            if let Order::Send {
                project,
                text,
                urgent,
            } = order
            {
                match self.start_send(&project, &text, urgent) {
                    Ok((project, words, busy, answer)) => self.phone.sending.push(Sending {
                        project,
                        words,
                        caller,
                        busy,
                        answer,
                        reply,
                    }),
                    Err(why) => self.answer_page(reply, &caller, Err(why)),
                }
                continue;
            }
            // A start's page waits for the runner's answer, which a later pass takes (EP-30b).
            if let Order::Start { project, ep } = order {
                match self.start_from_page(&project, &ep) {
                    Ok((project, ep, words)) => {
                        self.phone.starting = Some(Starting {
                            project,
                            ep,
                            caller,
                            words,
                            reply,
                        });
                    }
                    Err(why) => self.answer_page(reply, &caller, Err(why)),
                }
                continue;
            }
            let said = self.run_order(order, &mut wrote);
            self.answer_page(reply, &caller, said);
        }
        self.phone_pass();
        if wrote {
            self.loader.read_files();
        }
        if ran {
            self.pages.refresh();
        }
    }

    /// One order but a message: its outcome; `wrote` is set when a file was written.
    fn run_order(&mut self, order: Order, wrote: &mut bool) -> Said {
        let today = files::today();
        match order {
            Order::TickTask { project, id } => {
                let (name, folder) = self.writable_folder(&project)?;
                self.no_revert(&name)?;
                *wrote = true;
                // The Owner tasks pane's ticks of the week read again (EP-50's review).
                self.ticks.stale();
                self.tick_in(&folder, &name, &id, true, &today)
            }
            Order::TickMessage {
                project,
                id,
                disposition,
            } => {
                let (name, folder) = self.writable_folder(&project)?;
                self.no_revert(&name)?;
                *wrote = true;
                self.tick_message_in(&folder, &name, &id, &disposition, &today)
            }
            Order::Answer { key, request, form } => self.answer_from_page(key, &request, &form),
            Order::Stop {
                project,
                key,
                ep,
                session,
            } => self.stop_from_page(&project, key, &ep, session.as_deref()),
            Order::Pause { paused } => self.pause_from_page(paused),
            Order::Check => self.check_from_page(),
            Order::Send { .. } => Err("A message goes through its own write.".to_owned()),
            Order::Start { .. } => Err("A start goes through its own path.".to_owned()),
        }
    }

    /// The start page's preview of `name`'s start (EP-30b); none for a project the window does
    /// not list.
    pub fn start_preview(&mut self, name: &str) -> Option<Start> {
        self.prepare_start(name).map(|(start, _)| start)
    }

    /// What a start of `name`'s next brief from the phone would run, and what refuses it now
    /// (EP-30b): one reading for the start page's preview and for the start itself, the draft as
    /// the dialog drafts it, worded headless with the rules' choice as a chained start is. The
    /// launch comes with it only when nothing refuses. None when the window lists no such
    /// project. The CLI's preflight is asked last, once nothing else refuses, so that a test
    /// with no canned preflight never starts the CLI (rule 5).
    pub(crate) fn prepare_start(&mut self, name: &str) -> Option<(Start, Option<Ready>)> {
        let row = self.portfolio.project(name)?;
        let project = row.name().to_owned();
        let (folder, snapshot, reasons, fallback) = (
            row.folder.as_ref().map(|folder| folder.path.clone()),
            row.snapshot.clone(),
            row.in_progress(),
            row.project.clone(),
        );
        let missing = folder.is_none().then(|| crate::list::missing_folder(row));
        let mut start = Start {
            project: project.clone(),
            next: Next::None {
                words: String::new(),
                missing: false,
            },
            model: Default::default(),
            effort: Default::default(),
            ultracode: false,
            permission_mode: Default::default(),
            downshift: None,
            run: Vec::new(),
            notes: Vec::new(),
            refusals: Vec::new(),
        };
        let mut refusals = Vec::new();
        if self.fixtures {
            refusals
                .push("a screenshot run starts no session: its projects are fixtures or a copy");
        }
        if self.restart.closing_at.is_some() {
            refusals.push("the window is restarting into a newer build");
        }
        start.refusals = refusals.into_iter().map(str::to_owned).collect();
        let Some(folder) = folder else {
            let words = missing.unwrap_or_default();
            start.refusals.push(words.clone());
            start.next = Next::None {
                words,
                missing: true,
            };
            return Some((start, None));
        };
        let registry = match self.store().load() {
            Ok(loaded) => loaded.registry,
            Err(e) => {
                start
                    .refusals
                    .push(format!("the registry did not read: {e}"));
                return Some((start, None));
            }
        };
        let entry = registry.project(&project).cloned().unwrap_or(fallback);
        // The CLI's levels as last read, without asking yet: the built-in ones before.
        let levels: Vec<String> = self
            .sessions
            .preflight_ready()
            .and_then(|preflight| preflight.flags.as_ref().ok().map(|f| f.efforts.clone()))
            .unwrap_or_default();
        let claude_md = prompt::claude_md(&folder);
        let now = Utc::now();
        let figures = self.meter.gauge.figures(now);
        let usage = rules::Usage::of(&figures, now);
        let today = files::today();
        let mut draft = match prompt::draft(
            &registry,
            &entry,
            &snapshot,
            claude_md.as_deref(),
            &levels,
            &usage,
            &today,
        ) {
            Ok(draft) => draft,
            Err(why) => {
                let words = format!("nothing to launch: {why}");
                start.refusals.push(words.clone());
                start.next = Next::None {
                    words,
                    missing: false,
                };
                return Some((start, None));
            }
        };
        // Headless whatever the project's launch mode, the rules' choice (D-22's chained form).
        draft.for_chain(None, entry.launch.template.as_deref());
        start.next = Next::Brief {
            id: draft.ep.to_string(),
            size: draft.size.map(|size| size.as_str().to_owned()),
            title: draft.title.clone(),
        };
        let resolution = &draft.resolution;
        start.model = (resolution.model.value.clone(), resolution.model.reason());
        start.effort = (draft.parts.effort.clone(), resolution.effort.reason());
        start.ultracode = resolution.ultracode.value;
        start.permission_mode = (
            resolution.permission_mode.value.clone(),
            resolution.permission_mode.reason(),
        );
        start.downshift = resolution
            .downshift
            .as_ref()
            .filter(|downshift| downshift.applied)
            .map(|downshift| downshift.words.clone());
        let autopilot = entry.autopilot.enabled;
        let plan = autopilot
            .then(|| chains::plan(&registry, &entry, &snapshot, draft.ep, &levels, &usage));
        start.run = run_words(&draft, autopilot, plan.as_ref());
        start.notes = self.start_notes(&project, &draft, &reasons, &figures, now);
        // What refuses it now: the dialog's reasons, and a session at work there (the gate's).
        if self.sessions.is_starting() || self.phone.is_starting() {
            start
                .refusals
                .push("a session is starting in the window: start again in a moment".to_owned());
        }
        if self.visual.is_opening() {
            start.refusals.push(
                "a visual launch is on its way to VS Code: start again in a moment".to_owned(),
            );
        }
        if let Some(running) = self.sessions.summary(&project) {
            start
                .refusals
                .push(format!("the window already has {running}"));
        }
        let (busy, _) = self.open_elsewhere(&project);
        if !busy.is_empty() {
            start.refusals.push(format!(
                "Claude Code reports {} there: a second session would edit the same files",
                busy.join("; ")
            ));
        }
        if self.autopilot.is_starting_in(&project) {
            start
                .refusals
                .push("the autopilot's chained start is on its way there".to_owned());
        }
        // A run's end whose gate is still read would chain the same brief (the review's).
        if self.autopilot.is_reading(&project) {
            start.refusals.push(
                "the autopilot reads its gate after a run there: start again in a moment"
                    .to_owned(),
            );
        }
        // A deferred run's answer would resume a second session in the same tree (the review's).
        for live in self.sessions.live.iter().filter(|live| {
            !live.is_chat()
                && live.project.eq_ignore_ascii_case(&project)
                && live.ended.is_some()
                && live.open.is_some()
        }) {
            start.refusals.push(format!(
                "the window's {} there was deferred with a request open: answer it on the \
                 decisions page, or dismiss its tab in the window, first",
                live.ep
            ));
        }
        if self.revert_open_in(&project) {
            start
                .refusals
                .push("a revert is open there in the window".to_owned());
        }
        if !start.refusals.is_empty() {
            return Some((start, None));
        }
        let program = crw_core::session::cli::locate(registry.settings.claude_cli.as_deref())
            .unwrap_or_else(|| PathBuf::from("claude.exe"));
        let ctx = self.ctx().clone();
        let Some(preflight) = self.sessions.preflight(&program, &ctx) else {
            start.refusals.push(
                "claude has not answered yet: the window asks it now, so start again in a moment"
                    .to_owned(),
            );
            return Some((start, None));
        };
        start.refusals.extend(preflight.refusals());
        if !start.refusals.is_empty() {
            return Some((start, None));
        }
        let ready = self.phone_launch(&registry, &entry, &folder, draft, plan, preflight);
        Some((start, Some(ready)))
    }

    /// What the start page notes first, as the launch dialog would (EP-30b): owner tasks due
    /// before the brief, the files' evidence of a session in progress, sessions open and idle
    /// there, the session window's notice, and the gate's standing verdict.
    fn start_notes(
        &self,
        project: &str,
        draft: &prompt::Draft,
        reasons: &[crw_core::roadmap::snapshot::InProgress],
        figures: &crw_core::usage::Figures,
        now: chrono::DateTime<Utc>,
    ) -> Vec<String> {
        let mut notes = Vec::new();
        if !draft.parts.due.is_empty() {
            notes.push(draft.parts.due.clone());
        }
        if !reasons.is_empty() {
            let texts: Vec<String> = reasons.iter().map(ToString::to_string).collect();
            notes.push(format!(
                "A session may be in progress already ({}): a second one would edit the same \
                 files.",
                texts.join("; ")
            ));
        }
        let (_, idle) = self.open_elsewhere(project);
        if !idle.is_empty() {
            notes.push(format!(
                "Open and idle there: {}. Typing into it while this run works would make two \
                 sessions edit one tree.",
                idle.join("; ")
            ));
        }
        let mark = self.autopilot.five_hour_mark.max(1);
        notes.extend(chains::session_notice(figures, mark, false, now));
        for refusal in self
            .autopilot
            .standing
            .get(&project.to_lowercase())
            .into_iter()
            .flatten()
        {
            notes.push(match refusal {
                Refusal::Waits(mode) => format!(
                    "Its commands would each wait for your answer on the decisions page \
                     (permission mode {mode}, no allow rule), and the gate refuses its chains \
                     for it."
                ),
                refusal => format!("The gate refuses its chains: {}.", refusal.words()),
            });
        }
        notes
    }

    /// The launch of a start from the phone (EP-30b), once nothing refuses it: the chained
    /// start's (`autopilot::launch`), with what the dialog's Start headless adds: the route, the
    /// hooks and the revert's start; the cap, the budget, the push guard and the chain plan only
    /// when the project's autopilot is on, as the dialog does. Nothing starts here.
    fn phone_launch(
        &self,
        registry: &registry::Registry,
        entry: &registry::Project,
        folder: &std::path::Path,
        draft: prompt::Draft,
        plan: Option<Plan>,
        preflight: Arc<Preflight>,
    ) -> Ready {
        let mut launch = chains::launch(
            &draft,
            entry,
            folder,
            &registry.data_root(),
            self.ledger().map(PathBuf::from),
        );
        let autopilot = entry.autopilot.enabled;
        let run = if autopilot {
            crate::sessions::Run {
                capped: true,
                size: draft.size,
                chained: false,
                commits: draft.parts.commits,
                migration: false,
            }
        } else {
            launch.max_budget_usd = None;
            launch.deny_push = false;
            crate::sessions::Run {
                commits: draft.parts.commits,
                ..crate::sessions::Run::default()
            }
        };
        if entry.revert {
            launch.starts = Some(self.store().backups.clone());
        }
        launch.route = self
            .sessions
            .route_for(&preflight, registry.settings.decisions)
            .0;
        launch.hooks = crate::board::crw_exe();
        let after = if autopilot {
            " It commits itself, within its cap, and a chain may follow as the gate allows."
        } else {
            " It stops before committing, for the window's Commit."
        };
        let words = format!(
            "Started {} {} headless with {} at {}, the rules' choice: the project's page shows the \
             run, and its Stop there ends it.{after}",
            entry.name, draft.ep, draft.resolution.model.value, draft.parts.effort
        );
        Ready {
            launch,
            effort: draft.parts.effort.clone(),
            run,
            plan: plan.filter(|_| autopilot),
            preflight,
            ep: draft.ep,
            words,
        }
    }

    /// The phone's start of `project`'s next brief, `ep` being the brief its page showed
    /// (EP-30b): refused in words, or handed to the runner, the chain plan kept as the dialog's
    /// start keeps it; the project's name as registered, the brief and the words its page hears
    /// once it runs.
    fn start_from_page(
        &mut self,
        project: &str,
        ep: &str,
    ) -> Result<(String, EpId, String), String> {
        let Some((start, ready)) = self.prepare_start(project) else {
            return Err(format!("{project} is not a registered project."));
        };
        let name = start.project.clone();
        let shown = match &start.next {
            Next::Brief { id, .. } => id.clone(),
            Next::None { words, .. } => {
                return Err(format!("{name}: nothing was started: {words}."));
            }
        };
        if !shown.eq_ignore_ascii_case(ep) {
            return Err(format!(
                "{name}: the next brief is {shown} now, not {ep}: look at its start page again. \
                 Nothing was started."
            ));
        }
        let Some(ready) = ready else {
            return Err(format!(
                "{name} {shown} was not started: {}.",
                start.refusals.join("; ")
            ));
        };
        let Ready {
            launch,
            effort,
            run,
            plan,
            preflight,
            ep,
            words,
        } = ready;
        // The plan as the rules leave it, kept until the chain ends (EP-19a); a start the
        // runner refuses lets it go again.
        let kept = plan.is_some();
        if let Some(plan) = plan {
            self.autopilot.keep_plan(plan);
        }
        let ctx = self.ctx().clone();
        self.sessions.start(preflight, launch, effort, run, &ctx);
        // No thread for the start: no answer would ever come, so none is waited for (the
        // review's: the page's mark would stay and take another start's answer).
        if !self.sessions.is_starting() {
            if kept {
                self.autopilot.drop_plan(&name);
            }
            return Err(format!(
                "{name} {ep} did not start: the window could not make the start's thread."
            ));
        }
        Ok((name, ep, words))
    }

    /// The phone's start came back from the runner (EP-30b): its page told; a chain held there
    /// gives way to it, as to the dialog's start; a refusal lets the plan go and the preflight
    /// be asked again, as the dialog's does. The desk's panes stay as they are.
    pub(crate) fn phone_started(&mut self, started: Started) {
        let Some(starting) = self.phone.starting.take() else {
            return;
        };
        let said = match started {
            Started::Running(_) => {
                self.started_by_hand(&starting.project, Some(starting.ep));
                Ok(starting.words)
            }
            Started::Refused(why) => {
                self.sessions.forget_preflight();
                self.autopilot.drop_plan(&starting.project);
                Err(format!(
                    "{} {} did not start: {why}",
                    starting.project, starting.ep
                ))
            }
        };
        self.answer_page(starting.reply, &starting.caller, said);
        self.pages.refresh();
    }

    /// The phone's Stop of the window's run `key` (EP-30b): `Sessions::stop`, never Kill, for
    /// the run of `project`'s `ep` the page names, so that a page drawn before the window's
    /// restart, whose numbers start again at 1, stops nothing else; and of its `session` when
    /// the page names it, since the same brief run again may take the same number (EP-31's
    /// review).
    fn stop_from_page(&mut self, project: &str, key: u64, ep: &str, session: Option<&str>) -> Said {
        if self.fixtures {
            return Err("A screenshot run stops nothing: its sessions are canned.".to_owned());
        }
        let Some(live) = self.sessions.live.iter().find(|live| {
            live.key == key
                && !live.is_chat()
                && live.project.eq_ignore_ascii_case(project)
                && live.ep.eq_ignore_ascii_case(ep)
                && session.is_none_or(|id| live.session_id.eq_ignore_ascii_case(id))
        }) else {
            return Err(format!(
                "The window has no run of {project} {ep} under that number: nothing was stopped \
                 (the page may be older than the window's restart)."
            ));
        };
        let title = live.title();
        if !live.is_running() {
            return Err(format!("{title} is not running any more: nothing to stop."));
        }
        if live.stopping {
            return Err(format!("{title} is being stopped already."));
        }
        let chain = live.chained || live.cap.is_some();
        self.sessions.stop(key);
        if !self
            .sessions
            .live
            .iter()
            .any(|live| live.key == key && live.stopping)
        {
            return Err(format!(
                "{title}: the stop was not taken: the run has ended."
            ));
        }
        let ends = if chain {
            " A stopped run ends its chain: the gate reads it as stopped."
        } else {
            ""
        };
        Ok(format!(
            "Stopping {title}: its turn is interrupted and the run ends, its work left in the \
             tree.{ends}"
        ))
    }

    /// The phone's Pause or Resume (EP-30b): the desk's own write, in the orders' pass shown or
    /// hidden; a pause already in force, as the registry says it now, is said and not written.
    /// The projects' files are not read again (the review's): the setting is the window's own,
    /// which the autopilot block shows from its next build.
    fn pause_from_page(&mut self, paused: bool) -> Said {
        if self.fixtures {
            return Err(
                "A screenshot run changes nothing: its registry is the run's own.".to_owned(),
            );
        }
        let now = self
            .store()
            .load()
            .map(|loaded| loaded.registry.settings.autopilot_paused())
            .map_err(|e| format!("The registry could not be read: {e}"))?;
        if now == paused {
            self.autopilot.paused = paused;
            return Ok(if paused {
                "The autopilot is paused already: no chain goes on until you resume it."
            } else {
                "The autopilot is not paused: the chains go on as the gate allows."
            }
            .to_owned());
        }
        self.set_paused(paused)
    }

    /// A revert open or running in `project` refuses the phone's writes there (the review's):
    /// at the desk its dialog is modal, and a write made under a running revert would be put
    /// back with the start's files and not redone.
    fn no_revert(&self, project: &str) -> Result<(), String> {
        if self.revert_open_in(project) {
            Err(format!(
                "A revert is open in {project}: tick or send there once it is done."
            ))
        } else {
            Ok(())
        }
    }

    /// Whether the revert's preview, or the revert it runs, is the project's (EP-15c).
    pub(crate) fn revert_open_in(&self, project: &str) -> bool {
        self.revert
            .as_ref()
            .is_some_and(|dialog| dialog.project.eq_ignore_ascii_case(project))
    }

    /// The page's answer to request `id` of the window's session `key`: sent while the session
    /// waits on it, or the deferred run resumed with it; "already answered" when it no longer
    /// waits. The response is built from the form by the tile's rules against the request as
    /// the window holds it.
    fn answer_from_page(&mut self, key: u64, id: &str, form: &AnswerForm) -> Said {
        if self.fixtures {
            return Err("A screenshot run answers nothing: its sessions are canned.".to_owned());
        }
        let Some(live) = self
            .sessions
            .live
            .iter()
            .find(|live| live.key == key && !live.is_chat())
        else {
            return Err(ALREADY.to_owned());
        };
        let title = live.title();
        if let Some(waiting) = live.waiting.iter().find(|w| w.request.id == id) {
            let request = waiting.request.clone();
            let draft = Draft::from_form(&request, &form.picks, &form.other, &form.reason);
            let response = draft
                .response(&request, form.verb)
                .map_err(|why| format!("{title}: not sent: {why}."))?;
            return self.answer_live(key, id, response);
        }
        let Some(open) = live
            .open
            .as_ref()
            .filter(|open| live.ended.is_some() && open.request.id == id)
        else {
            return Err(ALREADY.to_owned());
        };
        let request = open.request.clone();
        let draft = Draft::from_form(&request, &form.picks, &form.other, &form.reason);
        let response = draft
            .response(&request, form.verb)
            .map_err(|why| format!("{title}: not sent: {why}."))?;
        let words = decisions::answer_words(&request, &response);
        self.resume_deferred(key, response)
            .map(|()| format!("{title}: {words}; the session resumes with it."))
            .map_err(|why| why.words())
    }

    /// The compose dialog's refusals, then its write on a worker thread (EP-20c): a screenshot
    /// run's projects, a revert open there, a message of the flag alone; and the same words to
    /// the same project being written or written lately (the review's). The project's registry
    /// name, the words as written, a session's words there and where the write answers.
    fn start_send(
        &mut self,
        project: &str,
        text: &str,
        urgent: bool,
    ) -> Result<(String, String, Option<String>, Writing), String> {
        let (name, folder) = self.writable_folder(project)?;
        self.no_revert(&name)?;
        let busy = self.busy_projects().remove(&name.to_lowercase());
        if mailman::is_empty_message(&mailman::compose_text(text, false)) {
            return Err("The message is empty: nothing was sent.".to_owned());
        }
        let words = mailman::compose_text(text, urgent);
        if let Some(why) = self.phone.repeat(&name, &words) {
            return Err(why);
        }
        let (tx, answer) = mpsc::channel();
        let backups = self.store().backups.clone();
        let ledger = self.ledger().map(PathBuf::from);
        let ctx = self.ctx().clone();
        let (text, receiver) = (text.to_owned(), name.clone());
        let today = files::today();
        thread::Builder::new()
            .name("phone compose".to_owned())
            .spawn(move || {
                let sent = mailman::send_from_owner(
                    &folder,
                    &receiver,
                    &text,
                    urgent,
                    &backups,
                    &today,
                    ledger.as_deref(),
                );
                let _ = tx.send(sent);
                ctx.request_repaint();
            })
            .map_err(|e| format!("The message could not be sent: {e}"))?;
        Ok((name, words, busy, answer))
    }

    /// The phone's messages written since the last pass, each answered as the compose dialog
    /// says its own, the files read again.
    fn phone_pass(&mut self) {
        let mut done = Vec::new();
        for sending in std::mem::take(&mut self.phone.sending) {
            match sending.answer.try_recv() {
                Ok(answer) => done.push((sending, answer)),
                Err(TryRecvError::Empty) => self.phone.sending.push(sending),
                Err(TryRecvError::Disconnected) => done.push((
                    sending,
                    Err("the write's thread ended without a word".to_owned()),
                )),
            }
        }
        let mut wrote = false;
        for (sending, answer) in done {
            let said = match answer {
                Ok(sent) => {
                    wrote = true;
                    self.phone.sent.push((
                        sending.project.to_lowercase(),
                        sending.words.clone(),
                        sent.id.clone(),
                        Instant::now(),
                    ));
                    Ok(sent_words(&sending.project, &sent, sending.busy.as_deref()))
                }
                Err(why) => Err(format!("Not sent to {}: {why}", sending.project)),
            };
            self.answer_page(sending.reply, &sending.caller, said);
        }
        if wrote {
            self.loader.read_files();
        }
    }

    /// The outcome to the page, and to the status bar and the rolling log with the caller.
    fn answer_page(&mut self, reply: oneshot::Sender<Said>, caller: &str, said: Said) {
        match &said {
            Ok(words) => self.notify(format!("From the phone's page ({caller}): {words}")),
            Err(why) => self.warn(format!("From the phone's page ({caller}): {why}")),
        }
        let _ = reply.send(said);
    }
}

/// What a start from the phone runs, in sentences for its page (EP-30b): its commits and its cap
/// while the autopilot is on, with the chain that may follow, or its stop before committing;
/// where its questions go.
fn run_words(draft: &prompt::Draft, autopilot: bool, plan: Option<&Plan>) -> Vec<String> {
    let mut words = Vec::new();
    if autopilot {
        let cap = Cap::new(draft.size, Instant::now());
        words.push(format!(
            "It commits itself in the house pattern, or the project's own when its CLAUDE.md \
             states one, and never pushes (the autopilot's standing yes), and is stopped at {} \
             of work.",
            cap.words()
        ));
        if let Some(plan) = plan {
            words.push(format!(
                "A chain may follow as the gate allows, each brief with the rules' choice: {}.",
                plan.words()
            ));
        }
    } else {
        words.push(
            "It stops before committing: its Commit is the window's, in its tab, which the phone \
             does not offer."
                .to_owned(),
        );
    }
    words.push(
        "Its questions and permission requests come to the decisions page, and quitting the \
         window stops it."
            .to_owned(),
    );
    words
}

/// What a message written from the phone says, as the compose dialog says its own (EP-20c).
fn sent_words(project: &str, sent: &Sent, busy: Option<&str>) -> String {
    let kept = match &sent.copy {
        Some(copy) => format!("the file as it was is kept as {}", copy.display()),
        None => "roadmap/inbox.md created with its header".to_owned(),
    };
    let unlogged = match &sent.unlogged {
        Some(why) => format!(
            " {}; a revert would not redo it.",
            crate::compose::capital(why)
        ),
        None if !sent.revert_keeps => " It is longer than the ledger keeps: a revert of a \
                                       session there would name it instead of putting it back."
            .to_owned(),
        None => String::new(),
    };
    let session = match busy {
        Some(busy) => format!(
            " {}: it has read its inbox already, so the session after it triages the message.",
            crate::compose::capital(busy)
        ),
        None => String::new(),
    };
    format!("Sent {} to {project}; {kept}.{unlogged}{session}", sent.id)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, Instant};

    use crw_core::ledger::Ledger;
    use crw_core::session::{Choice, Ended, Event, Launch, Outcome, Question, Request};
    use eframe::egui;
    use tokio::sync::oneshot::error::TryRecvError as Pending;

    use super::*;
    use crate::board::{Places, View};
    use crate::decisions::Verb;
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::sessions::Live;

    /// A board over the fake portfolio whose harbor lives in a scratch folder of its own, with
    /// two owner tasks and an inbox message, and a scratch ledger; never the tracked fixtures.
    fn board(scratch: &Scratch) -> (Board, PathBuf) {
        let places = Places {
            store: fake_store(scratch),
            ledger: Some(scratch.0.join("ledger.sqlite")),
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        let mut board = Board::new(places, &ctx, View::default(), egui::ThemePreference::Light);
        board.fixtures = false;
        board.portfolio = crate::portfolio::read_files(board.store());
        let folder = scratch.0.join("harbor-copy");
        fs::create_dir_all(folder.join("roadmap")).unwrap();
        fs::write(
            folder.join("roadmap").join("owner-tasks.md"),
            "# Owner tasks\n\n- [ ] OT-1 (EP-2, 2026-11-01) Look at the list.\n\
             - [ ] OT-2 (EP-3, 2026-11-02) Look at the pane.\n",
        )
        .unwrap();
        fs::write(
            folder.join("roadmap").join("inbox.md"),
            "# Inbox\n\n- [ ] IN-1 (from beta EP-4, 2026-11-02) The lock drifted.\n",
        )
        .unwrap();
        let row = board
            .portfolio
            .projects
            .iter_mut()
            .find(|row| row.name() == "harbor")
            .expect("the fake harbor");
        row.folder.as_mut().expect("its folder").path = folder.clone();
        (board, folder)
    }

    fn request(order: Order) -> (PageRequest, oneshot::Receiver<Said>) {
        let (reply, said) = oneshot::channel();
        let request = PageRequest {
            order,
            caller: "owner@example.com".to_owned(),
            reply,
            hand: crate::server::Hand::default(),
            sent: Instant::now(),
        };
        (request, said)
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    /// A form that came while the window slept is taken at the start of its pass, before the
    /// pass's own reads and writes (EP-43). Here a second connection holds the ledger's lock, as
    /// a slow commit would. Before this brief the pass's periodic accesses (the minute's lapse
    /// of visual launches, the chain log's half-minute read, the phone's pages' two reads) each
    /// waited out the ledger's five seconds or more, and the form waited for the first two; now
    /// the form is answered at once, each access gives up within its moment, and none says a
    /// problem.
    #[test]
    fn a_form_is_taken_before_the_pass_waits_on_a_locked_ledger() {
        let scratch = Scratch::new("phone-pace");
        let (mut board, _) = board(&scratch);
        let ledger = scratch.0.join("ledger.sqlite");
        drop(Ledger::open(&ledger).unwrap());
        let holder = rusqlite::Connection::open(&ledger).unwrap();
        holder.execute_batch("BEGIN EXCLUSIVE").unwrap();
        // A stop of a run the window does not have: refused without a write of its own.
        let (stop, said) = request(Order::Stop {
            project: "harbor".to_owned(),
            key: 41,
            ep: "EP-9".to_owned(),
            session: None,
        });
        board.watch.handed.push(stop);
        let began = Instant::now();
        let answered = std::thread::spawn(move || (said.blocking_recv(), Instant::now()));
        let ctx = egui::Context::default();
        board.pass(&ctx);
        let took = began.elapsed();
        let (said, at) = answered.join().unwrap();
        holder.execute_batch("ROLLBACK").unwrap();
        let why = said.unwrap().unwrap_err();
        assert!(why.contains("no run of harbor EP-9"), "{why}");
        let answered_after = at.duration_since(began);
        assert!(
            answered_after < Duration::from_secs(1),
            "answered after {answered_after:?}"
        );
        let (part, longest) = board.pace.longest_last().expect("the pass was timed");
        assert!(
            longest < Duration::from_secs(4),
            "{part} took {longest:?} of the pass's {took:?}"
        );
        assert_eq!(board.watch.problem, None);
        assert_eq!(board.autopilot.problem, None);
        assert_eq!(board.sessions.history.problem, None);
    }

    #[test]
    fn the_phones_ticks_write_as_the_desks_and_say_so() {
        let scratch = Scratch::new("phone-ticks");
        let (mut board, folder) = board(&scratch);
        let tasks = folder.join("roadmap").join("owner-tasks.md");
        let inbox = folder.join("roadmap").join("inbox.md");
        // A task, the project named in another case.
        let (tick, mut said) = request(Order::TickTask {
            project: "HARBOR".to_owned(),
            id: "OT-1".to_owned(),
        });
        board.obey_orders(vec![tick]);
        let words = said.try_recv().unwrap().unwrap();
        assert!(words.starts_with("Ticked OT-1 in harbor;"), "{words}");
        assert!(read(&tasks).contains("- [x] OT-1 "), "{}", read(&tasks));
        assert!(read(&tasks).contains("- [ ] OT-2 "));
        assert!(
            board.notice_text().is_some_and(|notice| notice
                .starts_with("From the phone's page (owner@example.com): Ticked OT-1")),
            "{:?}",
            board.notice_text()
        );
        // A message without its disposition: refused, the file as it was.
        let before = read(&inbox);
        let (empty, mut said) = request(Order::TickMessage {
            project: "harbor".to_owned(),
            id: "IN-1".to_owned(),
            disposition: String::new(),
        });
        board.obey_orders(vec![empty]);
        let why = said.try_recv().unwrap().unwrap_err();
        assert!(why.starts_with("harbor IN-1: "), "{why}");
        assert_eq!(read(&inbox), before);
        // With it: ticked, the disposition on its line.
        let (message, mut said) = request(Order::TickMessage {
            project: "harbor".to_owned(),
            id: "IN-1".to_owned(),
            disposition: "parked in EP-9".to_owned(),
        });
        board.obey_orders(vec![message]);
        let words = said.try_recv().unwrap().unwrap();
        assert!(
            words.starts_with("Ticked IN-1 in harbor with its disposition"),
            "{words}"
        );
        assert!(
            read(&inbox).contains("- [x] IN-1 ") && read(&inbox).contains("parked in EP-9"),
            "{}",
            read(&inbox)
        );
        // Each tick is in the ledger, for a revert to redo.
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let ticks = ledger.events(crw_core::revert::TICK_KIND, None).unwrap();
        assert_eq!(ticks.len(), 2, "{ticks:?}");
        assert!(
            ticks
                .iter()
                .all(|row| row.project.as_deref() == Some("harbor"))
        );
        // An order whose page stopped waiting is dropped unrun, and so is one its page let go
        // at its ten seconds; a page cannot let go of an order the window took.
        let (dropped, said) = request(Order::TickTask {
            project: "harbor".to_owned(),
            id: "OT-2".to_owned(),
        });
        drop(said);
        board.obey_orders(vec![dropped]);
        assert!(read(&tasks).contains("- [ ] OT-2 "), "dropped unrun");
        let (let_go, _said) = request(Order::TickTask {
            project: "harbor".to_owned(),
            id: "OT-2".to_owned(),
        });
        assert!(let_go.hand.let_go());
        board.obey_orders(vec![let_go]);
        assert!(read(&tasks).contains("- [ ] OT-2 "), "let go, never run");
        let taken = crate::server::Hand::default();
        assert!(taken.take());
        assert!(
            !taken.let_go(),
            "taken first: the page says it is under way"
        );
        // A project the registry does not know.
        let (unknown, mut said) = request(Order::TickTask {
            project: "nowhere".to_owned(),
            id: "OT-1".to_owned(),
        });
        board.obey_orders(vec![unknown]);
        assert_eq!(
            said.try_recv().unwrap().unwrap_err(),
            "nowhere is not a registered project."
        );
        // A screenshot run refuses every order, its files untouched.
        board.fixtures = true;
        let (fixture, mut said) = request(Order::TickTask {
            project: "harbor".to_owned(),
            id: "OT-2".to_owned(),
        });
        let (sending, mut sent) = request(Order::Send {
            project: "harbor".to_owned(),
            text: "Again.".to_owned(),
            urgent: false,
        });
        board.obey_orders(vec![fixture, sending]);
        assert!(
            said.try_recv()
                .unwrap()
                .unwrap_err()
                .starts_with("A screenshot run writes nothing")
        );
        assert!(
            sent.try_recv()
                .unwrap()
                .unwrap_err()
                .starts_with("A screenshot run writes nothing")
        );
        assert!(read(&tasks).contains("- [ ] OT-2 "));
    }

    #[test]
    fn the_phones_message_is_written_by_the_composes_write_and_answered_once_written() {
        let scratch = Scratch::new("phone-send");
        let (mut board, folder) = board(&scratch);
        let inbox = folder.join("roadmap").join("inbox.md");
        // The flag alone is no message: refused at once.
        let (empty, mut said) = request(Order::Send {
            project: "harbor".to_owned(),
            text: " [urgent] ".to_owned(),
            urgent: true,
        });
        board.obey_orders(vec![empty]);
        assert_eq!(
            said.try_recv().unwrap().unwrap_err(),
            "The message is empty: nothing was sent."
        );
        // A message: written on its thread, answered once written.
        let (send, mut said) = request(Order::Send {
            project: "harbor".to_owned(),
            text: "Note the  trap.".to_owned(),
            urgent: true,
        });
        board.obey_orders(vec![send]);
        assert!(board.phone.is_sending());
        let started = Instant::now();
        let answer = loop {
            match said.try_recv() {
                Ok(answer) => break answer,
                Err(Pending::Empty) if started.elapsed() < Duration::from_secs(20) => {
                    std::thread::sleep(Duration::from_millis(20));
                    board.obey_orders(Vec::new());
                }
                Err(e) => panic!("no answer: {e:?}"),
            }
        };
        let words = answer.unwrap();
        assert!(
            words.starts_with("Sent IN-2 to harbor; the file as it was is kept as "),
            "{words}"
        );
        assert!(!board.phone.is_sending());
        assert!(
            read(&inbox).contains("- [ ] IN-2 (from the owner, ")
                && read(&inbox).ends_with(") Note the trap. [urgent]\n"),
            "{}",
            read(&inbox)
        );
        // The same words again, a second tap or a page not reloaded: written once (the
        // review's); other words go.
        let written = read(&inbox);
        let (again, mut said) = request(Order::Send {
            project: "harbor".to_owned(),
            text: "Note the trap.".to_owned(),
            urgent: true,
        });
        board.obey_orders(vec![again]);
        let why = said.try_recv().unwrap().unwrap_err();
        assert!(
            why.starts_with("The same message went to harbor as IN-2"),
            "{why}"
        );
        assert_eq!(read(&inbox), written);
        // Two taps at once: the second meets the first on its way.
        let (first, mut first_said) = request(Order::Send {
            project: "harbor".to_owned(),
            text: "Another note.".to_owned(),
            urgent: false,
        });
        let (second, mut second_said) = request(Order::Send {
            project: "harbor".to_owned(),
            text: "Another  note.".to_owned(),
            urgent: false,
        });
        board.obey_orders(vec![first, second]);
        let why = second_said.try_recv().unwrap().unwrap_err();
        assert!(why.contains("is being written"), "{why}");
        let started = Instant::now();
        loop {
            match first_said.try_recv() {
                Ok(answer) => {
                    let words = answer.unwrap();
                    assert!(words.starts_with("Sent IN-3 to harbor"), "{words}");
                    break;
                }
                Err(Pending::Empty) if started.elapsed() < Duration::from_secs(20) => {
                    std::thread::sleep(Duration::from_millis(20));
                    board.obey_orders(Vec::new());
                }
                Err(e) => panic!("no answer: {e:?}"),
            }
        }
        assert_eq!(read(&inbox).matches("Another note.").count(), 1);
    }

    /// A board over the fake portfolio as its fixtures stand (nothing here writes into them), a
    /// scratch registry and ledger, and the canned preflight, so that no test asks the real CLI
    /// (rule 5). A test never reaches `Sessions::start`: it reads the launch a start would hand
    /// the runner.
    fn start_board(scratch: &Scratch) -> Board {
        let places = Places {
            store: fake_store(scratch),
            ledger: Some(scratch.0.join("ledger.sqlite")),
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        let mut board = Board::new(places, &ctx, View::default(), egui::ThemePreference::Light);
        board.fixtures = false;
        board.portfolio = crate::portfolio::read_files(board.store());
        board.sessions.canned = Some(std::sync::Arc::new(crate::launch::canned_preflight()));
        board
    }

    #[test]
    fn a_start_from_the_phone_is_the_rules_choice_headless_or_refused_for_the_windows_reasons() {
        let scratch = Scratch::new("phone-start");
        let mut board = start_board(&scratch);
        // harbor, its autopilot off: the dialog's headless launch, no cap, no plan.
        let (start, ready) = board.prepare_start("HARBOR").expect("harbor is listed");
        assert!(start.refusals.is_empty(), "{:?}", start.refusals);
        let Next::Brief { id, .. } = &start.next else {
            panic!("{:?}", start.next)
        };
        let id = id.clone();
        let ready = ready.expect("nothing refuses it");
        assert_eq!(ready.launch.ep.as_deref(), Some(id.as_str()));
        assert_eq!(ready.launch.model, start.model.0);
        assert!(!ready.launch.deny_push && ready.launch.max_budget_usd.is_none());
        assert!(ready.plan.is_none() && !ready.run.capped && !ready.run.chained);
        assert!(
            ready
                .words
                .starts_with(&format!("Started harbor {id} headless with ")),
            "{}",
            ready.words
        );
        assert!(
            start
                .run
                .iter()
                .any(|words| words.starts_with("It stops before committing")),
            "{:?}",
            start.run
        );
        // lantern launches visually at the desk: from the phone it is headless all the same.
        let (_, ready) = board.prepare_start("lantern").unwrap();
        let prompt = ready.expect("lantern may start").launch.prompt;
        assert!(!prompt.contains("panel chooses"), "{prompt}");
        // The autopilot on: the cap, the push guard, the commits and the plan of the rules.
        board
            .store()
            .change(&files::today(), |c| c.set_autopilot("harbor", true))
            .unwrap();
        let (start, ready) = board.prepare_start("harbor").unwrap();
        let ready = ready.expect("nothing refuses it");
        assert!(ready.launch.deny_push && ready.run.capped && ready.run.commits);
        assert!(
            ready
                .plan
                .as_ref()
                .is_some_and(|plan| plan.project == "harbor")
        );
        assert!(
            start.run[0].starts_with("It commits itself"),
            "{:?}",
            start.run
        );
        // The page's old brief: nothing starts.
        let why = board.start_from_page("harbor", "EP-999").unwrap_err();
        assert!(
            why.contains(&format!("the next brief is {id} now")),
            "{why}"
        );
        // A run of the window's working there: refused, as the gate would.
        board.sessions.live.push(live(5, "harbor", Vec::new()));
        let (start, ready) = board.prepare_start("harbor").unwrap();
        assert!(ready.is_none());
        assert!(
            start
                .refusals
                .iter()
                .any(|why| why.starts_with("the window already has a headless EP-4")),
            "{:?}",
            start.refusals
        );
        let why = board.start_from_page("harbor", &id).unwrap_err();
        assert!(
            why.contains("was not started: the window already has"),
            "{why}"
        );
        assert!(!board.phone.is_starting());
        // A deferred run there with its request open: its answer would resume a second session
        // in the same tree (the review's).
        board.sessions.live.clear();
        let deferred = vec![
            Event::Request(asked("req-9")),
            Event::TimedOut("req-9".to_owned()),
            Event::Exited(Ended {
                outcome: Outcome::Deferred,
                exit_code: Some(0),
                result: None,
                stderr: Vec::new(),
            }),
        ];
        board.sessions.live.push(live(6, "harbor", deferred));
        let (start, ready) = board.prepare_start("harbor").unwrap();
        assert!(ready.is_none());
        assert!(
            start
                .refusals
                .iter()
                .any(|why| why.contains("was deferred with a request open")),
            "{:?}",
            start.refusals
        );
        // A folder that is gone, a project the window does not list, a screenshot run.
        let (start, ready) = board.prepare_start("tundra").unwrap();
        assert!(ready.is_none() && !start.refusals.is_empty());
        assert!(matches!(start.next, Next::None { missing: true, .. }));
        assert!(board.prepare_start("nowhere").is_none());
        board.sessions.live.clear();
        board.fixtures = true;
        let (start, _) = board.prepare_start("meadow").unwrap();
        assert!(
            start.refusals[0].starts_with("a screenshot run starts no session"),
            "{:?}",
            start.refusals
        );
    }

    #[test]
    fn a_stop_from_the_phone_names_its_run_by_number_and_brief() {
        let scratch = Scratch::new("phone-stop");
        let (mut board, _) = board(&scratch);
        board.sessions.live.push(live(3, "harbor", Vec::new()));
        let stopping =
            |board: &mut Board, project: &str, key: u64, ep: &str, session: Option<&str>| {
                let (order, mut said) = request(Order::Stop {
                    project: project.to_owned(),
                    key,
                    ep: ep.to_owned(),
                    session: session.map(str::to_owned),
                });
                board.obey_orders(vec![order]);
                said.try_recv().unwrap()
            };
        let stop = |board: &mut Board, project: &str, key: u64, ep: &str| {
            stopping(board, project, key, ep, None)
        };
        // Another brief, another project, another number: nothing is stopped.
        for (project, key, ep) in [
            ("harbor", 3, "EP-5"),
            ("meadow", 3, "EP-4"),
            ("harbor", 1, "EP-4"),
        ] {
            let why = stop(&mut board, project, key, ep).unwrap_err();
            assert!(
                why.starts_with("The window has no run of"),
                "{project} {key} {ep}: {why}"
            );
        }
        // Another session under the same number and brief (the same brief run again after a
        // restart, EP-31's review): nothing is stopped; its own session reaches it.
        let other = "0f1e2d3c-0000-4000-8000-000000000009";
        let why = stopping(&mut board, "harbor", 3, "EP-4", Some(other)).unwrap_err();
        assert!(why.starts_with("The window has no run of"), "{why}");
        let own = board.sessions.live[0].session_id.to_uppercase();
        let why = stopping(&mut board, "harbor", 3, "EP-4", Some(&own)).unwrap_err();
        assert!(why.ends_with("the run has ended."), "{why}");
        // The run it names goes to the session's own stop: this canned one has no process, so
        // the stop is not taken, which shows the order reached it.
        let why = stop(&mut board, "HARBOR", 3, "ep-4").unwrap_err();
        assert!(
            why.ends_with("the stop was not taken: the run has ended."),
            "{why}"
        );
        // A run being stopped, and one that ended.
        board.sessions.live[0].stopping = true;
        let why = stop(&mut board, "harbor", 3, "EP-4").unwrap_err();
        assert!(why.ends_with("is being stopped already."), "{why}");
        let ended = Event::Exited(Ended {
            outcome: Outcome::Completed,
            exit_code: Some(0),
            result: None,
            stderr: Vec::new(),
        });
        board.sessions.live.push(live(4, "harbor", vec![ended]));
        let why = stop(&mut board, "harbor", 4, "EP-4").unwrap_err();
        assert!(
            why.ends_with("is not running any more: nothing to stop."),
            "{why}"
        );
        // A screenshot run stops nothing.
        board.fixtures = true;
        let why = stop(&mut board, "harbor", 3, "EP-4").unwrap_err();
        assert!(why.starts_with("A screenshot run stops nothing"), "{why}");
    }

    #[test]
    fn the_phones_pause_is_written_once_and_a_pause_in_force_is_said() {
        let scratch = Scratch::new("phone-pause");
        let (mut board, _) = board(&scratch);
        let pause = |board: &mut Board, paused: bool| {
            let (order, mut said) = request(Order::Pause { paused });
            board.obey_orders(vec![order]);
            said.try_recv().unwrap()
        };
        let chain_lines = || {
            Ledger::open(&scratch.0.join("ledger.sqlite"))
                .and_then(|ledger| ledger.events(chains::EVENT_KIND, None))
                .map(|rows| rows.len())
                .unwrap_or_default()
        };
        let words = pause(&mut board, true).unwrap();
        assert!(words.starts_with("The autopilot is paused:"), "{words}");
        assert!(
            board
                .store()
                .load()
                .unwrap()
                .registry
                .settings
                .autopilot_paused()
        );
        assert!(board.autopilot.paused);
        assert_eq!(chain_lines(), 1);
        // Pressed again from a page not reloaded: said, not written again.
        let words = pause(&mut board, true).unwrap();
        assert!(
            words.starts_with("The autopilot is paused already"),
            "{words}"
        );
        assert_eq!(chain_lines(), 1);
        let words = pause(&mut board, false).unwrap();
        assert!(words.starts_with("The autopilot is resumed"), "{words}");
        assert!(
            !board
                .store()
                .load()
                .unwrap()
                .registry
                .settings
                .autopilot_paused()
        );
        assert!(board.autopilot.recheck, "the held chains are read again");
        assert_eq!(chain_lines(), 2);
        assert!(
            board.notice_text().is_some_and(|notice| notice.starts_with(
                "From the phone's page (owner@example.com): The autopilot is resumed"
            )),
            "{:?}",
            board.notice_text()
        );
        board.fixtures = true;
        assert!(
            pause(&mut board, true)
                .unwrap_err()
                .starts_with("A screenshot run changes nothing")
        );
    }

    fn question(text: &str, several: bool) -> Question {
        Question {
            text: text.to_owned(),
            header: text.to_owned(),
            options: ["Top", "Bottom", "Middle"]
                .iter()
                .map(|label| Choice {
                    label: (*label).to_owned(),
                    description: None,
                })
                .collect(),
            multi_select: several,
        }
    }

    fn asked(id: &str) -> Request {
        Request {
            id: id.to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which shelf?".to_owned(),
            reason: None,
            questions: vec![question("Shelf", false), question("Rows", true)],
            input: String::new(),
        }
    }

    fn form(verb: Verb, picks: &[(usize, &[usize])], reason: &str) -> AnswerForm {
        AnswerForm {
            verb,
            picks: picks
                .iter()
                .map(|(q, options)| (*q, options.iter().copied().collect::<BTreeSet<_>>()))
                .collect(),
            other: BTreeMap::new(),
            reason: reason.to_owned(),
        }
    }

    fn live(key: u64, project: &str, events: Vec<Event>) -> Live {
        let mut launch = Launch::new(project, Path::new(project), "fable", "go");
        launch.ep = Some("EP-4".to_owned());
        Live::canned(key, launch, "high", events)
    }

    #[test]
    fn an_answer_reaches_the_request_it_names_by_the_tiles_rules() {
        let scratch = Scratch::new("phone-answers");
        let (mut board, _) = board(&scratch);
        let deferred = vec![
            Event::Request(asked("req-9")),
            Event::TimedOut("req-9".to_owned()),
            Event::Exited(Ended {
                outcome: Outcome::Deferred,
                exit_code: Some(0),
                result: None,
                stderr: Vec::new(),
            }),
        ];
        board.sessions.live.extend([
            live(1, "harbor", vec![Event::Request(asked("req-1"))]),
            live(2, "meadow", deferred),
        ]);
        let answer = |board: &mut Board, key: u64, id: &str, form: AnswerForm| {
            let (order, mut said) = request(Order::Answer {
                key,
                request: id.to_owned(),
                form,
            });
            board.obey_orders(vec![order]);
            said.try_recv().unwrap()
        };
        // No answer given, a decline without its reason, a button of the other kind: refused
        // by the tile's rules before anything is sent.
        let why = answer(&mut board, 1, "req-1", form(Verb::Answer, &[], "")).unwrap_err();
        assert!(why.contains("no answer was given"), "{why}");
        let why = answer(&mut board, 1, "req-1", form(Verb::Decline, &[], " ")).unwrap_err();
        assert!(why.contains("a decline needs its reason"), "{why}");
        let why = answer(&mut board, 1, "req-1", form(Verb::Allow, &[], "")).unwrap_err();
        assert!(why.contains("it is a question"), "{why}");
        // A valid answer goes to the session's own call: this canned one has no process, so
        // the call says it has ended, which shows the answer was routed there.
        let why = answer(
            &mut board,
            1,
            "req-1",
            form(Verb::Answer, &[(0, &[1]), (1, &[0, 2])], ""),
        )
        .unwrap_err();
        assert!(
            why.contains("the answer was not sent: the session has ended"),
            "{why}"
        );
        // A request no longer waiting, a key the window does not hold, a deferred run's
        // request of another id: already answered.
        for (key, id) in [(1, "req-0"), (8, "req-1"), (2, "req-1")] {
            let why =
                answer(&mut board, key, id, form(Verb::Answer, &[(0, &[0])], "")).unwrap_err();
            assert_eq!(why, ALREADY, "{key} {id}");
        }
        // A deferred run's own request, refused by the rules before any resume.
        let why = answer(&mut board, 2, "req-9", form(Verb::Answer, &[], "")).unwrap_err();
        assert!(why.contains("no answer was given"), "{why}");
        // A chat's question is the Chat pane's: never answered from the page.
        let mut chat_launch = Launch::new("chat", Path::new("chat"), "fable", "go");
        chat_launch.conversation = Some("What is next?".to_owned());
        board.sessions.live.push(Live::canned(
            4,
            chat_launch,
            "high",
            vec![Event::Request(asked("req-6"))],
        ));
        let why = answer(&mut board, 4, "req-6", form(Verb::Answer, &[(0, &[0])], "")).unwrap_err();
        assert_eq!(why, ALREADY);
        // A screenshot run answers nothing.
        board.fixtures = true;
        let why = answer(&mut board, 1, "req-1", form(Verb::Answer, &[(0, &[0])], "")).unwrap_err();
        assert!(why.starts_with("A screenshot run answers nothing"), "{why}");
    }

    /// The doctor's page's Check again (EP-31, the owner's choice): a test's board gathers no
    /// report, so it is refused in words and starts no child (rule 5).
    #[test]
    fn a_check_from_the_phone_is_refused_where_the_window_gathers_none() {
        let scratch = Scratch::new("phone-check");
        let (mut board, _) = board(&scratch);
        let (order, mut said) = request(Order::Check);
        board.obey_orders(vec![order]);
        let why = said.try_recv().unwrap().unwrap_err();
        assert!(why.starts_with("This window gathers no report"), "{why}");
        assert!(!board.diagnostics.checking());
    }
}
