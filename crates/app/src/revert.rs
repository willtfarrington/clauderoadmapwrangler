//! The window's Revert (D-11, D-25; EP-15c), on `crw_core::revert`'s engine (EP-15b). An ended
//! session's tab, a ledger card of the Sessions pane and a project's row menu each open a
//! preview: a dialog over the window, like the Quit question, with the plan's lines, then Revert
//! in the window's red and Cancel, or Close alone when it is refused. Reading a plan asks git
//! about six children, each in its turn, and a revert about five more, so both run on a worker
//! thread while the dialog shows a spinner. The window adds what `crw revert` cannot see
//! ([`Board::revert_view`]): its own runs in the project, a start on its way there, and the
//! sessions the hooks and the poll report. At the owner's word the plan is read again, and the
//! revert runs only when it is the plan they saw ([`Plan::is_as_seen`]); otherwise the dialog
//! shows the new one and nothing changes. A screenshot run's plan is canned, and its Revert
//! changes nothing (rule 5).

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use chrono::Utc;
use crw_core::files;
use crw_core::ledger::Ledger;
use crw_core::revert::{self, Done, Plan};
use crw_core::sessions::{Sessions as Model, Status};
use eframe::egui::{self, RichText};

use crate::board::{Action, Board};
use crate::portfolio::ProjectRow;
use crate::style;

/// How often the window looks for the worker's answer while it waits.
const POLL: Duration = Duration::from_millis(250);

/// The dialog's widest, in points, and the share of the window's height its lines may take.
const WIDTH: f32 = 700.0;
const LINES_SHARE: f32 = 0.55;

/// What the window knows of the sessions in a project besides git (EP-15c).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct View {
    /// At work or waiting there, each in words that the plan's "revert once it has ended"
    /// follows: each refuses the revert.
    pub working: Vec<String>,
    /// Open there with nothing to do: named, for the owner to close after the revert.
    pub idle: Vec<String>,
    /// The window's other reasons to refuse, each said as it is.
    pub refusals: Vec<String>,
}

/// What a plan is read from: the project, the session (by its id, or the latest), what the window
/// knows of the sessions there, and where the ledger and the backups are.
#[derive(Debug, Clone)]
pub struct Asked {
    pub project: String,
    pub folder: PathBuf,
    /// The project's `revert` switch in the registry.
    pub enabled: bool,
    pub session: Option<String>,
    pub view: View,
    pub ledger: Option<PathBuf>,
    pub backups: PathBuf,
}

/// What the worker answers.
enum Answer {
    Planned(Result<Plan, String>),
    /// The plan read again at the owner's word is not the one they saw: nothing ran.
    Changed(Plan),
    Reverted(Result<Done, String>),
}

/// What became of a revert the owner confirmed, for the window to say and to read the project
/// again.
#[derive(Debug)]
pub enum Outcome {
    Reverted(Done),
    /// It did not run, or did not finish: why.
    NotReverted(String),
    /// The project or its sessions changed since the preview, so nothing ran.
    Changed,
}

/// Where the dialog stands.
#[derive(Debug)]
pub enum Stage {
    /// The plan is being read.
    Reading,
    /// The preview; `changed` when it was read again at the owner's word and differed.
    Ready {
        plan: Plan,
        changed: bool,
    },
    /// The revert runs; the preview stays in view.
    Reverting(Plan),
    Done(Done),
    /// Nothing to preview, or the revert did not run: why.
    Failed(String),
}

/// The preview of undoing one session, and the revert it leads to.
pub struct Dialog {
    pub project: String,
    pub stage: Stage,
    receiver: Option<Receiver<Answer>>,
}

/// Whether a session of `project` (its row in the portfolio, if any) may be reverted from the
/// window: the reason when not, which a disabled Revert gives on hover.
pub fn availability(row: Option<&ProjectRow>, project: &str) -> Result<(), String> {
    let row = row.ok_or_else(|| format!("{project} is not a registered project"))?;
    if row.folder.is_none() {
        return Err(format!("{project}: its folder is missing"));
    }
    if !row.project.revert {
        return Err(format!(
            "Revert is off for {project}: `crw set {project} revert yes` turns it on"
        ));
    }
    Ok(())
}

/// A session's Revert… (`small` on a ledger card): it opens the preview, which changes nothing;
/// disabled, with its reason on hover, when the project allows none. `true` when pressed.
pub fn revert_button(ui: &mut egui::Ui, available: &Result<(), String>, small: bool) -> bool {
    let text = RichText::new("Revert…");
    let text = if small { text.small() } else { text };
    let button = ui.add_enabled(available.is_ok(), egui::Button::new(text));
    match available {
        Ok(()) => button
            .on_hover_text(
                "Preview undoing this session: the commits and files it would undo, then Revert \
                 or Cancel",
            )
            .clicked(),
        Err(why) => {
            button.on_disabled_hover_text(why);
            false
        }
    }
}

/// What the hooks and the poll report of the sessions in `project` (EP-15c), the window's own
/// (`own`) and the ended ones left out: those at work or waiting, each refusing a revert; those
/// idle, named (a tab open with no prompt yet holds none of the work, so is not); and, while the
/// poll of live sessions has not answered or fails, a refusal, since a session may then work there
/// unseen, as `crw revert` refuses when its listing does not answer.
pub fn seen_view(model: &Model, project: &str, own: impl Fn(&str) -> bool) -> View {
    let mut view = View::default();
    for seen in model.of_project(project) {
        if seen.status.is_over() || own(&seen.session_id) {
            continue;
        }
        let (mode, id) = (seen.mode.as_str(), revert::short(&seen.session_id));
        match seen.status {
            Status::Busy => view.working.push(format!("a {mode} session {id} is busy")),
            Status::Waiting => view.working.push(match &seen.waiting_for {
                Some(what) => format!("a {mode} session {id} waits for you ({what})"),
                None => format!("a {mode} session {id} waits for you"),
            }),
            Status::Idle if !seen.is_new() => view.idle.push(format!("a {mode} session {id}")),
            _ => {}
        }
    }
    match (&model.poll_problem, model.polled) {
        (Some(why), _) => view.refusals.push(format!(
            "the list of live sessions did not read ({why}), so whether a session works there is \
             not known"
        )),
        (None, None) => view.refusals.push(
            "the list of live sessions has not answered yet, so whether a session works there is \
             not known: try again in a moment"
                .to_owned(),
        ),
        (None, Some(_)) => {}
    }
    view
}

/// Reads the plan `asked` names: the session chosen from the ledger's rows, the tool's ticks since
/// its start, git asked one child at a time (D-29), and the window's own refusals added.
fn read_plan(asked: &Asked) -> Result<Plan, String> {
    let path = asked
        .ledger
        .as_deref()
        .ok_or("the window has no ledger, so no session is recorded")?;
    if !path.exists() {
        return Err(format!(
            "no ledger yet at {}: no session of {} is recorded",
            path.display(),
            asked.project
        ));
    }
    let read = Ledger::open_read_only(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let rows = read
        .sessions()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let chosen = revert::choose(&rows, &asked.project, asked.session.as_deref())?;
    let ticks = match chosen.first.started() {
        Some(since) => revert::ticks_since(&read, &asked.project, since)?,
        None => Vec::new(),
    };
    drop(read);
    let cx = revert::Context {
        project: &asked.project,
        folder: &asked.folder,
        enabled: asked.enabled,
        working: asked.view.working.clone(),
        idle: asked.view.idle.clone(),
        ticks,
        backups: &asked.backups,
        now: Utc::now(),
    };
    let mut plan = revert::plan(&chosen, &cx);
    plan.refusals.extend(asked.view.refusals.iter().cloned());
    Ok(plan)
}

/// Reverts as `seen` says once the plan read again from `asked` is the one the owner saw, so that
/// nothing made after the preview goes uncopied; else the new plan, and nothing changes.
fn confirm(asked: &Asked, seen: &Plan, today: &str) -> Answer {
    let again = match read_plan(asked) {
        Ok(again) => again,
        Err(why) => return Answer::Reverted(Err(why)),
    };
    if !again.is_as_seen(seen) {
        return Answer::Changed(again);
    }
    Answer::Reverted(revert::revert(
        seen,
        asked.ledger.as_deref(),
        &asked.backups,
        today,
    ))
}

/// Runs `work` on a thread named `name`, waking the window when it answers.
fn spawn(
    name: &str,
    ctx: &egui::Context,
    work: impl FnOnce() -> Answer + Send + 'static,
) -> Option<Receiver<Answer>> {
    let (sender, receiver) = mpsc::channel();
    let ctx = ctx.clone();
    thread::Builder::new()
        .name(name.to_owned())
        .spawn(move || {
            let _ = sender.send(work());
            ctx.request_repaint();
        })
        .ok()
        .map(|_| receiver)
}

impl Dialog {
    /// Starts reading the plan `asked` names, on a worker thread.
    pub fn open(asked: Asked, ctx: &egui::Context) -> Self {
        let project = asked.project.clone();
        let receiver = spawn("revert preview", ctx, move || {
            Answer::Planned(read_plan(&asked))
        });
        let stage = match receiver {
            Some(_) => Stage::Reading,
            None => Stage::Failed("the preview's thread did not start".to_owned()),
        };
        Self {
            project,
            stage,
            receiver,
        }
    }

    /// The preview of a plan read elsewhere: a screenshot run's canned one, a test's.
    #[cfg(any(test, feature = "screenshot"))]
    pub fn ready(plan: Plan) -> Self {
        Self {
            project: plan.project.clone(),
            stage: Stage::Ready {
                plan,
                changed: false,
            },
            receiver: None,
        }
    }

    /// A revert runs: nothing closes the dialog, and quitting waits.
    pub fn is_reverting(&self) -> bool {
        matches!(self.stage, Stage::Reverting(_))
    }

    /// The worker has yet to answer.
    #[cfg(test)]
    pub fn is_busy(&self) -> bool {
        matches!(self.stage, Stage::Reading | Stage::Reverting(_))
    }

    /// The plan shown, when the owner may confirm it.
    pub fn confirmable(&self) -> Option<&Plan> {
        match &self.stage {
            Stage::Ready { plan, .. } if !plan.is_refused() => Some(plan),
            _ => None,
        }
    }

    /// Reverts at the owner's word, on a worker thread: the plan read again from `asked` first.
    pub fn confirm(&mut self, asked: Asked, ctx: &egui::Context) {
        let Some(seen) = self.confirmable().cloned() else {
            return;
        };
        let shown = seen.clone();
        let today = files::today();
        match spawn("revert", ctx, move || confirm(&asked, &seen, &today)) {
            Some(receiver) => {
                self.receiver = Some(receiver);
                self.stage = Stage::Reverting(shown);
            }
            None => {
                self.stage =
                    Stage::Failed("the revert's thread did not start: nothing changed".to_owned());
            }
        }
    }

    /// Takes in the worker's answer when it has come, and asks for frames while it waits. What
    /// became of a confirmed revert, for the window; nothing for a preview read.
    pub fn receive(&mut self, ctx: &egui::Context) -> Option<Outcome> {
        let receiver = self.receiver.as_ref()?;
        let answer = match receiver.try_recv() {
            Ok(answer) => answer,
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(POLL);
                return None;
            }
            Err(TryRecvError::Disconnected) => {
                self.receiver = None;
                let reverting = self.is_reverting();
                let why = if reverting {
                    "the revert's thread ended without an answer: look at the project, and at the \
                     copy of its work in the backups folder"
                } else {
                    "the preview's thread ended without an answer"
                };
                self.stage = Stage::Failed(why.to_owned());
                return reverting.then(|| Outcome::NotReverted(why.to_owned()));
            }
        };
        self.receiver = None;
        match answer {
            Answer::Planned(Ok(plan)) => {
                self.stage = Stage::Ready {
                    plan,
                    changed: false,
                };
                None
            }
            Answer::Planned(Err(why)) => {
                self.stage = Stage::Failed(why);
                None
            }
            Answer::Changed(plan) => {
                self.stage = Stage::Ready {
                    plan,
                    changed: true,
                };
                Some(Outcome::Changed)
            }
            Answer::Reverted(Ok(done)) => {
                self.stage = Stage::Done(done.clone());
                Some(Outcome::Reverted(done))
            }
            Answer::Reverted(Err(why)) => {
                self.stage = Stage::Failed(why.clone());
                Some(Outcome::NotReverted(why))
            }
        }
    }

    /// The dialog's contents at `width`, its lines at most `height` tall; the owner's choice.
    pub fn ui(&self, ui: &mut egui::Ui, width: f32, height: f32) -> Option<Action> {
        ui.set_width(width);
        let project = &self.project;
        let mut chosen = None;
        match &self.stage {
            Stage::Reading => {
                ui.heading(format!("Revert a session of {project}?"));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!(
                        "Reading {project}: the ledger, then git, one child at a time…"
                    ));
                });
                ui.add_space(8.0);
                buttons(ui, |ui| {
                    if ui.button("Cancel").clicked() {
                        chosen = Some(Action::CloseRevert);
                    }
                });
            }
            Stage::Ready { plan, changed } => {
                ui.heading(format!("Revert a session of {project}?"));
                ui.add_space(4.0);
                if *changed {
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                "The project or its sessions changed since the preview, so \
                                 nothing was reverted. This is the plan as it stands now.",
                            )
                            .color(style::amber(ui)),
                        )
                        .wrap(),
                    );
                    ui.add_space(4.0);
                }
                lines(ui, &plan.lines(), height);
                ui.add_space(8.0);
                buttons(ui, |ui| {
                    if plan.is_refused() {
                        if ui.button("Close").clicked() {
                            chosen = Some(Action::CloseRevert);
                        }
                        return;
                    }
                    if ui.button("Cancel").clicked() {
                        chosen = Some(Action::CloseRevert);
                    }
                    if ui
                        .button(RichText::new("Revert").color(style::red(ui)))
                        .on_hover_text(
                            "The plan is read again first: the revert runs only if the project \
                             is as this preview shows it",
                        )
                        .clicked()
                    {
                        chosen = Some(Action::ConfirmRevert);
                    }
                });
            }
            Stage::Reverting(plan) => {
                ui.heading(format!("Reverting {project}…"));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        "The plan read again, then the work copied out, then git and the files…",
                    );
                });
                ui.add_space(4.0);
                lines(ui, &plan.lines(), height);
            }
            Stage::Done(done) => {
                ui.heading(format!("{project} is reverted"));
                ui.add_space(4.0);
                lines(ui, &done.lines(), height);
                ui.add_space(8.0);
                buttons(ui, |ui| {
                    if ui.button("Close").clicked() {
                        chosen = Some(Action::CloseRevert);
                    }
                });
            }
            Stage::Failed(why) => {
                ui.heading(format!("Revert a session of {project}"));
                ui.add_space(4.0);
                ui.add(egui::Label::new(RichText::new(why).color(style::red(ui))).wrap());
                ui.add_space(8.0);
                buttons(ui, |ui| {
                    if ui.button("Close").clicked() {
                        chosen = Some(Action::CloseRevert);
                    }
                });
            }
        }
        chosen
    }
}

/// The dialog's buttons on one line at its right, the first added rightmost.
fn buttons(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add);
    });
}

/// A plan's or a revert's lines in a scroll area at most `height` tall: the first line strong,
/// the commits and files indented in the code face, a refusal or a problem in the window's red,
/// the sessions to close and the ticks to redo by hand in its amber.
fn lines(ui: &mut egui::Ui, lines: &[String], height: f32) {
    egui::ScrollArea::vertical()
        .id_salt("revert-lines")
        .max_height(height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for (n, line) in lines.iter().enumerate() {
                if let Some(item) = line.strip_prefix("  ") {
                    ui.indent(("revert-line", n), |ui| {
                        ui.add(egui::Label::new(RichText::new(item).monospace()).wrap());
                    });
                    continue;
                }
                let text = RichText::new(line);
                let text = if line.starts_with("Refused:") || line.starts_with("Problem:") {
                    text.color(style::red(ui))
                } else if line.starts_with("Open there") || line.starts_with("Not redone") {
                    text.color(style::amber(ui))
                } else if n == 0 {
                    text.strong()
                } else {
                    text
                };
                ui.add(egui::Label::new(text).wrap());
            }
        });
}

impl Board {
    /// What the window knows of the sessions in `project` that `crw revert` cannot see (EP-15c):
    /// its own runs and starts there, a visual launch on its way, a session's start being read,
    /// and the sessions the hooks and the poll report.
    pub fn revert_view(&self, project: &str) -> View {
        let (mut working, mut refusals) = self.sessions.at_work(project);
        if let Some(ep) = self.visual.opening_in(project) {
            refusals.push(format!(
                "a visual launch of {ep} is on its way to VS Code: revert once it has opened"
            ));
        }
        if self.watch.reading_start(project) {
            refusals
                .push("a session's start there is being read: try again in a moment".to_owned());
        }
        let seen = seen_view(&self.watch.model, project, |id| self.sessions.owns(id));
        working.extend(seen.working);
        refusals.extend(seen.refusals);
        View {
            working,
            idle: seen.idle,
            refusals,
        }
    }

    /// What a plan of `project`'s session `session` (or its latest) is read from, as the window
    /// stands now.
    fn revert_asked(&self, project: &str, session: Option<String>) -> Result<Asked, String> {
        let row = self.portfolio.project(project);
        availability(row, project)?;
        let row = row.ok_or_else(|| format!("{project} is not a registered project"))?;
        let folder = row
            .folder
            .as_ref()
            .ok_or_else(|| format!("{project}: its folder is missing"))?;
        Ok(Asked {
            project: row.name().to_owned(),
            folder: folder.path.clone(),
            enabled: row.project.revert,
            session,
            view: self.revert_view(project),
            ledger: self.ledger().map(Path::to_owned),
            backups: self.store().backups.clone(),
        })
    }

    /// Opens the preview of undoing `project`'s session `session`, or its latest (EP-15c), read on
    /// a worker thread; a screenshot run's plan is canned, since its projects are the tracked
    /// fixtures.
    pub fn open_revert(&mut self, project: &str, session: Option<String>) {
        if self.revert.is_some() {
            return;
        }
        if self.fixtures {
            #[cfg(feature = "screenshot")]
            {
                let plan = crate::screenshot::canned_plan(
                    project,
                    session.as_deref(),
                    &self.sessions.history.rows,
                    false,
                );
                self.revert = Some(Dialog::ready(plan));
            }
            return;
        }
        match self.revert_asked(project, session) {
            Ok(asked) => self.revert = Some(Dialog::open(asked, self.ctx())),
            Err(why) => self.warn(why),
        }
    }

    /// The preview's Revert (EP-15c): the plan read again with the window as it stands now, and
    /// the revert run if it is the one shown; a screenshot run reverts nothing.
    pub fn confirm_revert(&mut self) {
        if self.fixtures {
            self.warn("A screenshot run reverts nothing: its projects are tracked fixtures.");
            return;
        }
        let Some(plan) = self.revert.as_ref().and_then(Dialog::confirmable) else {
            return;
        };
        let (project, session) = (plan.project.clone(), plan.session_id.clone());
        match self.revert_asked(&project, session) {
            Ok(asked) => {
                let ctx = self.ctx().clone();
                if let Some(dialog) = &mut self.revert {
                    dialog.confirm(asked, &ctx);
                }
            }
            Err(why) => self.fail(format!("{project}: nothing was reverted: {why}")),
        }
    }

    /// The preview put away, unless a revert runs.
    pub fn close_revert(&mut self) {
        if !self.revert.as_ref().is_some_and(Dialog::is_reverting) {
            self.revert = None;
        }
    }

    /// Takes in the worker's answer (EP-15c): after a revert, the files and git are read again,
    /// the All tab's rows go stale and the status bar says where the work is kept.
    pub fn receive_revert(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.revert.as_mut() else {
            return;
        };
        let project = dialog.project.clone();
        let Some(outcome) = dialog.receive(ctx) else {
            return;
        };
        if matches!(outcome, Outcome::Changed) {
            return;
        }
        self.loader.read_files();
        self.loader.read_git();
        self.sessions.history.stale = true;
        match outcome {
            Outcome::Reverted(done) => {
                let text = format!(
                    "Reverted {project}: the work is kept in {}.",
                    done.copy.display()
                );
                if done.problems.is_empty() && done.not_redone.is_empty() {
                    self.notify(text);
                } else {
                    self.warn(format!("{text} Part of it needs a look: see the dialog."));
                }
            }
            Outcome::NotReverted(why) => self.fail(format!("{project}: {why}")),
            Outcome::Changed => {}
        }
    }

    /// The preview over the window, while it is open. Escape and Cancel close it, a click beside
    /// it does not, and nothing does while a revert runs.
    pub fn revert_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.revert.as_ref() else {
            return;
        };
        let rect = ctx.content_rect();
        let width = WIDTH.min(rect.width() - 80.0);
        let height = (rect.height() * LINES_SHARE).max(160.0);
        let modal = egui::Modal::new(egui::Id::unique("revert-preview"))
            .show(ctx, |ui| dialog.ui(ui, width, height));
        let closable = !dialog.is_reverting() && !modal.backdrop_response.clicked();
        match modal.inner {
            Some(action) => self.actions.push(action),
            None if closable && modal.should_close() => self.actions.push(Action::CloseRevert),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Instant;

    use chrono::TimeDelta;
    use crw_core::ledger::NewSession;
    use crw_core::revert::{Action as FileAction, Change, Commit, Fate, Target};
    use crw_core::sessions::{Mode, Seen};
    use crw_core::{git, process};

    use super::*;
    use crate::portfolio::tests::Scratch;
    use crate::testing::{self, Screen};

    /// A plan of two lines' work, as `crw revert` would show it; `refused` adds a refusal.
    fn shown(refused: bool) -> Plan {
        let mut plan = Plan::new(
            "alpha",
            Path::new("alpha"),
            PathBuf::from(r"C:\backups\alpha\reverts\1"),
        );
        plan.session_id = Some("0e1d2c3b-4a59-4687-9a0b-1c2d3e4f5a6b".to_owned());
        plan.ep = Some("EP-3".to_owned());
        plan.target = Some(Target {
            commit: "3c1e9b7a5d2f4e60".to_owned(),
            recorded: true,
            subject: None,
        });
        plan.branch = Some("main".to_owned());
        plan.commits = vec![Commit {
            oid: "e41d7c2098b3a6f5".to_owned(),
            subject: "Session work".to_owned(),
            pushed: false,
        }];
        plan.files = vec![Fate {
            path: "src/lib.rs".to_owned(),
            change: Change::Changed,
            action: FileAction::Undo,
        }];
        if refused {
            plan.refusals
                .push("a manual session 9a3e7d41 is busy: revert once it has ended".to_owned());
        }
        plan
    }

    /// Presses the button that reads `label` in the dialog, a pass at a time; what it chose.
    fn press(dialog: &Dialog, label: &str) -> Option<Action> {
        let mut screen = Screen::new();
        let mut chosen = None;
        let output = screen.pass(Vec::new(), |ui| chosen = dialog.ui(ui, 600.0, 400.0));
        let at = testing::text_at(&output, label)?;
        for events in [
            testing::moved(at),
            testing::button(at, egui::PointerButton::Primary, true),
            testing::button(at, egui::PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| {
                if let Some(action) = dialog.ui(ui, 600.0, 400.0) {
                    chosen = Some(action);
                }
            });
        }
        chosen
    }

    /// The preview's buttons in a bare context (EP-15c): Revert reaches the confirmation and
    /// Cancel closes; a refused plan offers Close alone, in its red line.
    #[test]
    fn the_previews_revert_reaches_the_confirmation() {
        let ready = Dialog::ready(shown(false));
        assert_eq!(press(&ready, "Revert"), Some(Action::ConfirmRevert));
        assert_eq!(press(&ready, "Cancel"), Some(Action::CloseRevert));
        assert!(ready.confirmable().is_some());

        let refused = Dialog::ready(shown(true));
        assert_eq!(press(&refused, "Revert"), None, "no Revert to press");
        assert_eq!(press(&refused, "Cancel"), None);
        assert_eq!(press(&refused, "Close"), Some(Action::CloseRevert));
        assert!(refused.confirmable().is_none());
    }

    /// The window's view of the sessions in a project (EP-15c): a busy or waiting one refuses, an
    /// idle one is named, a tab open with no prompt yet, an ended one, the window's own and another
    /// project's are left out; and the poll unheard or failing refuses too.
    #[test]
    fn the_sessions_reported_refuse_or_are_named() {
        let at = Utc::now() - TimeDelta::minutes(5);
        let mut model = Model::default();
        let mut fresh = Seen::canned(
            "41d8e2c0-new",
            Some("alpha"),
            Mode::Manual,
            Status::Idle,
            None,
            at,
        );
        fresh.worked = false;
        for seen in [
            Seen::canned(
                "5f1c0a2e-v",
                Some("alpha"),
                Mode::Visual,
                Status::Waiting,
                Some("a permission prompt"),
                at,
            ),
            Seen::canned(
                "9a3e7d41-m",
                Some("Alpha"),
                Mode::Manual,
                Status::Busy,
                None,
                at,
            ),
            Seen::canned(
                "77aa0000-i",
                Some("alpha"),
                Mode::Manual,
                Status::Idle,
                None,
                at,
            ),
            Seen::canned(
                "88bb0000-d",
                Some("alpha"),
                Mode::Manual,
                Status::Done,
                None,
                at,
            ),
            Seen::canned(
                "99cc0000-o",
                Some("alpha"),
                Mode::Headless,
                Status::Busy,
                None,
                at,
            ),
            Seen::canned(
                "aadd0000-b",
                Some("beta"),
                Mode::Manual,
                Status::Busy,
                None,
                at,
            ),
            fresh,
        ] {
            model.insert(seen);
        }
        let own = |id: &str| id == "99cc0000-o";
        let unheard = seen_view(&model, "alpha", own);
        assert_eq!(
            unheard.working,
            [
                "a visual session 5f1c0a2e waits for you (a permission prompt)",
                "a manual session 9a3e7d41 is busy",
            ]
        );
        assert_eq!(unheard.idle, ["a manual session 77aa0000"]);
        assert_eq!(unheard.refusals.len(), 1, "the poll has not answered yet");
        model.polled = Some(Utc::now());
        assert!(seen_view(&model, "alpha", own).refusals.is_empty());
        model.poll_problem = Some("claude did not answer".to_owned());
        let failing = seen_view(&model, "alpha", own);
        assert!(failing.refusals[0].contains("claude did not answer"));
        assert!(seen_view(&model, "gamma", own).working.is_empty());
    }

    /// Waits for the dialog's worker to answer; what became of a confirmed revert.
    fn settle(dialog: &mut Dialog, ctx: &egui::Context) -> Option<Outcome> {
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            let outcome = dialog.receive(ctx);
            if !dialog.is_busy() {
                return outcome;
            }
            assert!(Instant::now() < deadline, "the worker did not answer");
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn git_in(folder: &Path, args: &[&str]) -> String {
        let _turn = process::turn();
        git::run(folder, args).unwrap_or_else(|e| panic!("git {args:?}: {e}"))
    }

    fn commit(folder: &Path, message: &str) -> String {
        git_in(folder, &["add", "-A"]);
        git_in(
            folder,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--quiet",
                "-m",
                message,
            ],
        );
        git_in(folder, &["rev-parse", "HEAD"]).trim().to_owned()
    }

    /// The window's path on a scratch repository in a temp folder, never the owner's projects
    /// (rule 13): the preview read on the worker, the window's refusals in it, a file made after
    /// the preview turning the confirmation into a new preview, then the revert of the preview
    /// read again: HEAD back at the start, the new file gone, the work copied out, the ledger's
    /// event.
    #[test]
    fn the_window_reverts_only_the_plan_it_showed() {
        let scratch = Scratch::new("window-revert");
        let folder = scratch.0.join("alpha");
        fs::create_dir_all(&folder).unwrap();
        git_in(&folder, &["init", "--quiet", "-b", "main"]);
        git_in(&folder, &["config", "core.autocrlf", "false"]);
        fs::write(folder.join("a.txt"), "one\n").unwrap();
        let first = commit(&folder, "first");
        let ledger = scratch.0.join("ledger.sqlite");
        let backups = scratch.0.join("backups");
        let id = "0e1d2c3b-4a59-4687-9a0b-1c2d3e4f5a6b";
        let start = {
            let _turn = process::turn();
            let copies = revert::start::copies_dir(&backups, &folder, id);
            revert::start::capture(&folder, &copies, Utc::now()).unwrap()
        };
        {
            let ledger = Ledger::open(&ledger).unwrap();
            let row = ledger
                .start_session(&NewSession {
                    session_id: Some(id),
                    project: "alpha",
                    ep: Some("EP-2"),
                    mode: "headless",
                    model: None,
                    effort: None,
                    ultracode: false,
                    started_at: start.at,
                })
                .unwrap();
            ledger.set_start(row, &start.to_json()).unwrap();
        }
        // The session's work: a commit and a new file.
        fs::write(folder.join("a.txt"), "two\n").unwrap();
        commit(&folder, "Session work");
        fs::write(folder.join("new.txt"), "the session's\n").unwrap();
        let asked = Asked {
            project: "alpha".to_owned(),
            folder: folder.clone(),
            enabled: true,
            session: None,
            view: View::default(),
            ledger: Some(ledger.clone()),
            backups: backups.clone(),
        };
        let ctx = egui::Context::default();

        // The window's refusals reach the plan, said as they are.
        let mut busy = asked.clone();
        busy.view.working = vec!["the window's headless EP-4 is running".to_owned()];
        busy.view.refusals = vec!["a session's start there is being read".to_owned()];
        let mut dialog = Dialog::open(busy, &ctx);
        assert!(
            settle(&mut dialog, &ctx).is_none(),
            "a preview is no outcome"
        );
        let refused = dialog.confirmable();
        assert!(refused.is_none());
        let Stage::Ready { plan, .. } = &dialog.stage else {
            panic!("{:?}", dialog.stage);
        };
        assert_eq!(
            plan.refusals,
            [
                "the window's headless EP-4 is running: revert once it has ended",
                "a session's start there is being read",
            ]
        );

        // A file made after the preview: the confirmation shows the new plan, and nothing ran.
        let mut dialog = Dialog::open(asked.clone(), &ctx);
        settle(&mut dialog, &ctx);
        assert_eq!(dialog.confirmable().map(|plan| plan.commits.len()), Some(1));
        fs::write(folder.join("later.txt"), "after the preview\n").unwrap();
        dialog.confirm(asked.clone(), &ctx);
        assert!(dialog.is_reverting());
        assert!(matches!(settle(&mut dialog, &ctx), Some(Outcome::Changed)));
        let Stage::Ready { plan, changed } = &dialog.stage else {
            panic!("{:?}", dialog.stage);
        };
        assert!(*changed && plan.files.iter().any(|fate| fate.path == "later.txt"));
        assert_ne!(git_in(&folder, &["rev-parse", "HEAD"]).trim(), first);

        // The new plan confirmed: reverted.
        dialog.confirm(asked, &ctx);
        let Some(Outcome::Reverted(done)) = settle(&mut dialog, &ctx) else {
            panic!("{:?}", dialog.stage);
        };
        assert_eq!(git_in(&folder, &["rev-parse", "HEAD"]).trim(), first);
        assert_eq!(fs::read_to_string(folder.join("a.txt")).unwrap(), "one\n");
        assert!(!folder.join("new.txt").exists() && !folder.join("later.txt").exists());
        assert!(done.copy.join("commits.bundle").is_file());
        assert!(done.copy.join("files").join("later.txt").is_file());
        assert!(matches!(dialog.stage, Stage::Done(_)));
        let events = Ledger::open_read_only(&ledger)
            .unwrap()
            .events(revert::EVENT_KIND, None)
            .unwrap();
        assert_eq!(events.len(), 1);
    }

    /// No ledger, or no session in it: the preview says so, with Close.
    #[test]
    fn a_project_with_no_session_has_nothing_to_preview() {
        let scratch = Scratch::new("window-revert-none");
        let ctx = egui::Context::default();
        let asked = Asked {
            project: "alpha".to_owned(),
            folder: scratch.0.join("alpha"),
            enabled: true,
            session: None,
            view: View::default(),
            ledger: Some(scratch.0.join("ledger.sqlite")),
            backups: scratch.0.join("backups"),
        };
        let mut dialog = Dialog::open(asked, &ctx);
        settle(&mut dialog, &ctx);
        let Stage::Failed(why) = &dialog.stage else {
            panic!("{:?}", dialog.stage);
        };
        assert!(why.contains("no session of alpha"), "{why}");
        assert_eq!(press(&dialog, "Close"), Some(Action::CloseRevert));
    }
}
