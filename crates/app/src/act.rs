//! The window's hosting and backup actions (D-24; EP-34), on `crw_core::actions`' engine. A
//! project's row menu and its Detail pane's Hosting subsection offer what its hosting allows
//! (Create a private remote for a local repository; Make public or Make private and Remove the
//! remote for a listed one; Archive and Delete for an orphan); the Backup subsection and the File
//! menu offer Back up now. Each opens a dialog over the window, like the formalise's: the command
//! named whole, what it does, the warnings it repeats and what refuses it, then the action's
//! button and Cancel, or Close alone when it is refused. The plan is read on a worker thread
//! (Create and Delete ask `gh` which account it is logged in as), with the window's refusals for a
//! project ([`Board::work_view`]); at the owner's word it is read again and run only when it is
//! the plan shown. A run is logged in the ledger, then the project is read again with its GitHub
//! reading asked afresh, or the backup job is read again. A screenshot run's runner and a test's
//! are canned: nothing real runs (rule 5).
//!
//! Make public's preview also holds the word check of the project's history (EP-42): the commits
//! that hold a word of the owner's list, by short hash and subject, never a word or a line. When
//! any hit, or the history did not read, a second tick is needed before its button works; a plan
//! shown afresh clears it.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crw_core::actions::{self, Check, Command, End, Exit, Facts, Plan, Ran, Runner, Target, Verb};
use crw_core::wordcheck;
use eframe::egui::{self, RichText};

use crate::board::{Action, Board};
use crate::portfolio::{Moment, ProjectRow};
use crate::style;

/// How often the window looks for the worker's answer while it waits.
const POLL: Duration = Duration::from_millis(250);

/// The dialog's widest, in points, and the share of the window's height its body may take.
const WIDTH: f32 = 720.0;
const BODY_SHARE: f32 = 0.55;

/// How many of the commits that hit Make public's check the dialog lists (EP-42's review): a
/// common word would otherwise lay out thousands of lines in every frame.
const HITS_SHOWN: usize = 100;

/// What the window refuses while the dialog's action runs: a quit (EP-34).
pub const ACTING: &str = "An action on GitHub or the backup job is running: quit once it has \
                          finished, in a moment (the backup job takes a minute or two).";

/// The actions a row offers (its menu, its Detail pane): those its hosting allows, for a
/// repository; none for a missing folder or a folder that is not one.
pub fn offered(row: &ProjectRow) -> Vec<Verb> {
    // A screenshot run's fakes are not repositories on disk: their canned git reading says so.
    let repository = row.is_repository() || row.badges.activity.is_some();
    if row.folder.is_none() || !repository {
        return Vec::new();
    }
    Verb::offered(&row.badges.hosting)
}

/// What a verb's hover says before the dialog opens.
pub fn hint(verb: Verb) -> &'static str {
    match verb {
        Verb::CreatePrivate => {
            "Preview creating a private GitHub repository from this folder and pushing its \
             commits: the command, then Create and push or Cancel"
        }
        Verb::MakePublic => {
            "Preview making the GitHub repository public, with gh's warning, what a push to a \
             public remote means and its history read against your word list: the command, then \
             Make public or Cancel"
        }
        Verb::MakePrivate => {
            "Preview making the GitHub repository private, with gh's warning: the command, then \
             Make private or Cancel"
        }
        Verb::RemoveRemote => {
            "Preview removing the origin from the clone's configuration; the GitHub repository is \
             left alone"
        }
        Verb::Archive => "Preview archiving the account's repository of this project's name",
        Verb::Delete => {
            "Preview deleting the account's repository of this project's name: it needs the \
             token's delete_repo scope"
        }
        Verb::BackUpNow => {
            "Preview running the backup job now with -Force: the command, then Back up now or \
             Cancel"
        }
    }
}

/// The dialog's button for the action.
fn button(verb: Verb) -> &'static str {
    match verb {
        Verb::CreatePrivate => "Create and push",
        Verb::MakePublic => "Make public",
        Verb::MakePrivate => "Make private",
        Verb::RemoveRemote => "Remove the remote",
        Verb::Archive => "Archive",
        Verb::Delete => "Delete",
        Verb::BackUpNow => "Back up now",
    }
}

/// The actions whose button is drawn in the window's red: they publish or cannot be undone here.
fn grave(verb: Verb) -> bool {
    matches!(verb, Verb::MakePublic | Verb::Delete)
}

/// A screenshot run's word list for Make public's check (EP-42), synthetic as the fixtures are.
const CANNED_WORDS: &str = "moonstone\n";

/// The history the canned runner's `git log` prints for a screenshot run (EP-42): four commits,
/// one holding the canned word in a file's path, one in its subject.
const CANNED_LOG: &str = "\0d4c3b2a\u{1f}Moonstone notes for the release\u{1f}Moonstone notes \
for the release\n\u{1e}\ndiff --git a/notes.md b/notes.md\n--- a/notes.md\n+++ b/notes.md\n\
@@ -1,0 +2 @@\n+plain words\n\0c3b2a19\u{1f}Tidy the parser\u{1f}Tidy the parser\n\u{1e}\n\
diff --git a/src/parser.rs b/src/parser.rs\n--- a/src/parser.rs\n+++ b/src/parser.rs\n\
@@ -3 +3 @@\n-old\n+new\n\0b2a1908\u{1f}Add the release checklist\u{1f}Add the release \
checklist\n\u{1e}\ndiff --git a/moonstone/checklist.md b/moonstone/checklist.md\nnew file mode \
100644\n--- /dev/null\n+++ b/moonstone/checklist.md\n@@ -0,0 +1,2 @@\n+# Checklist\n+one\n\
\0a190817\u{1f}Start\u{1f}Start\n\u{1e}\ndiff --git a/README.md b/README.md\nnew file mode \
100644\n--- /dev/null\n+++ b/README.md\n@@ -0,0 +1 @@\n+# A project\n";

/// The runner of a screenshot run and of a test given none: `gh auth status` answers with a fake
/// account whose token has `scopes`, `git log` with a synthetic history (EP-42), and every other
/// command is refused, so nothing runs.
pub struct Canned {
    pub scopes: String,
}

impl Default for Canned {
    fn default() -> Self {
        Self {
            scopes: "gist, read:org, repo, workflow".to_owned(),
        }
    }
}

impl Runner for Canned {
    fn run(&self, command: &Command) -> Result<Exit, String> {
        if command.args.first().map(String::as_str) == Some("auth") {
            return Ok(Exit {
                code: Some(0),
                stdout: format!(
                    r#"{{"hosts":{{"github.com":[{{"state":"success","active":true,"host":"github.com","login":"example","scopes":"{}"}}]}}}}"#,
                    self.scopes
                ),
                stderr: String::new(),
            });
        }
        if command.args.iter().any(|arg| arg == "log") {
            return Ok(Exit {
                code: Some(0),
                stdout: CANNED_LOG.to_owned(),
                stderr: String::new(),
            });
        }
        Err("a screenshot run or a test runs nothing".to_owned())
    }
}

/// What a plan is read from, and where its run is logged.
#[derive(Clone)]
pub struct Asked {
    pub facts: Facts,
    pub runner: Arc<dyn Runner>,
    pub ledger: Option<PathBuf>,
}

/// What the worker answers.
enum Answer {
    Planned(Plan),
    /// The plan read again at the owner's word is not the one they saw: nothing ran.
    Changed(Plan),
    Ran(Ran),
}

/// Runs `seen` once the plan read again from `asked` is the one the owner saw; else the new
/// plan, and nothing runs.
fn confirm(asked: &Asked, seen: &Plan) -> Answer {
    let again = actions::plan(&asked.facts, asked.runner.as_ref());
    if again != *seen {
        return Answer::Changed(again);
    }
    match actions::run(seen, asked.runner.as_ref(), asked.ledger.as_deref()) {
        Some(ran) => Answer::Ran(ran),
        None => Answer::Changed(again),
    }
}

/// What became of an action the owner confirmed.
#[derive(Debug)]
pub enum Outcome {
    Ran(Ran),
    /// The worker ended without an answer.
    Lost(String),
    /// The project or its sessions changed since the preview, so nothing ran.
    Changed,
}

/// Where the dialog stands.
#[derive(Debug)]
pub enum Stage {
    Reading,
    /// The preview; `changed` when it was read again at the owner's word and differed.
    Ready {
        plan: Plan,
        changed: bool,
    },
    /// The action runs; the preview stays in view.
    Running(Plan),
    Done(Ran),
    Failed(String),
}

/// A backup the owner left running with its dialog closed: the answer the window still reads.
pub struct Detached(Receiver<Answer>);

impl Detached {
    /// The run's end when it has come, asking for frames while it waits.
    pub fn receive(&self, ctx: &egui::Context) -> Option<Outcome> {
        match self.0.try_recv() {
            Ok(Answer::Ran(ran)) => Some(Outcome::Ran(ran)),
            // A plan changed since the preview ran nothing; the dialog said so before it closed.
            Ok(Answer::Changed(_) | Answer::Planned(_)) => Some(Outcome::Changed),
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(POLL);
                None
            }
            Err(TryRecvError::Disconnected) => Some(Outcome::Lost(
                "the backup's thread ended without an answer: its run log says how it went"
                    .to_owned(),
            )),
        }
    }
}

/// The confirmation of one action, and its run.
pub struct Dialog {
    pub verb: Verb,
    /// The project, or none for Back up now.
    pub project: Option<String>,
    pub stage: Stage,
    receiver: Option<Receiver<Answer>>,
    /// Make public's second tick (EP-42): given by the owner, cleared whenever a plan is shown
    /// afresh.
    anyway: bool,
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
        let verb = asked.facts.verb;
        let project = asked.facts.target.as_ref().map(|t| t.name.clone());
        let receiver = spawn("action preview", ctx, move || {
            Answer::Planned(actions::plan(&asked.facts, asked.runner.as_ref()))
        });
        let stage = match receiver {
            Some(_) => Stage::Reading,
            None => Stage::Failed("the preview's thread did not start".to_owned()),
        };
        Self {
            verb,
            project,
            stage,
            receiver,
            anyway: false,
        }
    }

    /// The preview of a plan read elsewhere: a screenshot run's, a test's.
    pub fn ready(plan: Plan) -> Self {
        Self {
            verb: plan.verb,
            project: plan.project.clone(),
            stage: Stage::Ready {
                plan,
                changed: false,
            },
            receiver: None,
            anyway: false,
        }
    }

    /// Make public's second tick, given or taken back (EP-42); only a plan shown that asks for
    /// it takes it.
    pub fn tick(&mut self, ticked: bool) {
        if matches!(&self.stage, Stage::Ready { plan, .. } if plan.asks_tick()) {
            self.anyway = ticked;
        }
    }

    /// The action runs: nothing closes the dialog but a backup's Close.
    pub fn is_running(&self) -> bool {
        matches!(self.stage, Stage::Running(_))
    }

    /// The backup job runs, which the dialog may leave running when it closes (EP-34's review):
    /// its two minutes need not hold the window.
    pub fn detachable(&self) -> bool {
        self.verb == Verb::BackUpNow && self.is_running()
    }

    /// The running backup's answer, taken out of the dialog that closes, for the window to read.
    pub fn detach(&mut self) -> Option<Detached> {
        if !self.detachable() {
            return None;
        }
        self.receiver.take().map(Detached)
    }

    /// The plan shown, when the owner may run it: not refused, and ticked a second time when its
    /// word check asks for it (EP-42).
    pub fn confirmable(&self) -> Option<&Plan> {
        match &self.stage {
            Stage::Ready { plan, .. }
                if !plan.is_refused() && (self.anyway || !plan.asks_tick()) =>
            {
                Some(plan)
            }
            _ => None,
        }
    }

    /// Runs at the owner's word, on a worker thread: the plan read again from `asked` first.
    pub fn confirm(&mut self, asked: Asked, ctx: &egui::Context) {
        let Some(seen) = self.confirmable().cloned() else {
            return;
        };
        let shown = seen.clone();
        match spawn("action", ctx, move || confirm(&asked, &seen)) {
            Some(receiver) => {
                self.receiver = Some(receiver);
                self.stage = Stage::Running(shown);
            }
            None => {
                self.stage =
                    Stage::Failed("the action's thread did not start: nothing ran".to_owned());
            }
        }
    }

    /// Takes in the worker's answer when it has come, and asks for frames while it waits. What
    /// became of a confirmed action, for the window; nothing for a preview read.
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
                let running = self.is_running();
                let why = if running {
                    "the action's thread ended without an answer: look at the project and the \
                     ledger before trying again"
                } else {
                    "the preview's thread ended without an answer"
                };
                self.stage = Stage::Failed(why.to_owned());
                return running.then(|| Outcome::Lost(why.to_owned()));
            }
        };
        self.receiver = None;
        match answer {
            Answer::Planned(plan) => {
                self.anyway = false;
                self.stage = Stage::Ready {
                    plan,
                    changed: false,
                };
                None
            }
            Answer::Changed(plan) => {
                // A plan shown afresh is looked at afresh (EP-42).
                self.anyway = false;
                self.stage = Stage::Ready {
                    plan,
                    changed: true,
                };
                Some(Outcome::Changed)
            }
            Answer::Ran(ran) => {
                self.stage = Stage::Done(ran.clone());
                Some(Outcome::Ran(ran))
            }
        }
    }

    /// The question the dialog asks, before its plan is read and after.
    fn question(&self) -> String {
        let target = match &self.stage {
            Stage::Ready { plan, .. } | Stage::Running(plan) => plan.target.clone(),
            _ => self
                .project
                .clone()
                .unwrap_or_else(|| "the backup job".to_owned()),
        };
        self.verb.question(&target)
    }

    /// The dialog's contents at `width`, its body at most `height` tall; the owner's choice.
    pub fn ui(&self, ui: &mut egui::Ui, width: f32, height: f32) -> Option<Action> {
        ui.set_width(width);
        let mut chosen = None;
        let close = |ui: &mut egui::Ui, chosen: &mut Option<Action>| {
            buttons(ui, |ui| {
                if ui.button("Close").clicked() {
                    *chosen = Some(Action::CloseAct);
                }
            });
        };
        match &self.stage {
            Stage::Reading => {
                ui.heading(self.question());
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spinner();
                    ui.label("Reading what the action would do…");
                });
                ui.add_space(8.0);
                buttons(ui, |ui| {
                    if ui.button("Cancel").clicked() {
                        chosen = Some(Action::CloseAct);
                    }
                });
            }
            Stage::Ready { plan, changed } => {
                ui.heading(self.question());
                ui.add_space(4.0);
                if *changed {
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                "The project or its sessions changed since the preview, so \
                                 nothing ran. This is the plan as it stands now.",
                            )
                            .color(style::amber(ui)),
                        )
                        .wrap(),
                    );
                    ui.add_space(4.0);
                }
                preview(ui, plan, height);
                ui.add_space(8.0);
                if plan.is_refused() {
                    close(ui, &mut chosen);
                } else {
                    // Make public's second tick, when its word check asks for one (EP-42).
                    if plan.asks_tick() {
                        let words = match &plan.check {
                            Some(Check::Failed(_)) => {
                                "Make it public unchecked: I have looked at its history myself"
                            }
                            _ => "Make it public anyway: I have looked at these commits",
                        };
                        let mut ticked = self.anyway;
                        if ui.checkbox(&mut ticked, words).changed() {
                            chosen = Some(Action::TickActAnyway(ticked));
                        }
                        ui.add_space(4.0);
                    }
                    let ready = self.anyway || !plan.asks_tick();
                    buttons(ui, |ui| {
                        if ui.button("Cancel").clicked() {
                            chosen = Some(Action::CloseAct);
                        }
                        let label = RichText::new(button(plan.verb));
                        let label = if grave(plan.verb) && ready {
                            label.color(style::red(ui))
                        } else {
                            label
                        };
                        if ui
                            .add_enabled(ready, egui::Button::new(label))
                            .on_hover_text(
                                "The plan is read again first: the command runs only if the \
                                 project is as this preview shows it",
                            )
                            .on_disabled_hover_text(
                                "Read what the history check says above, then tick the box to \
                                 go on",
                            )
                            .clicked()
                        {
                            chosen = Some(Action::ConfirmAct);
                        }
                    });
                }
            }
            Stage::Running(plan) => {
                ui.heading(self.question());
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spinner();
                    ui.label(if self.detachable() {
                        "The job runs. Close leaves it running: the status bar says how it ended."
                    } else {
                        "The plan read again, then the command run…"
                    });
                });
                ui.add_space(4.0);
                preview(ui, plan, height);
                if self.detachable() {
                    ui.add_space(8.0);
                    close(ui, &mut chosen);
                }
            }
            Stage::Done(ran) => {
                ui.heading(ran.words());
                ui.add_space(4.0);
                done_lines(ui, ran, height);
                ui.add_space(8.0);
                close(ui, &mut chosen);
            }
            Stage::Failed(why) => {
                ui.heading(self.question());
                ui.add_space(4.0);
                ui.add(egui::Label::new(RichText::new(why).color(style::red(ui))).wrap());
                ui.add_space(8.0);
                close(ui, &mut chosen);
            }
        }
        chosen
    }
}

/// The plan in a scroll area at most `height` tall: the command, what it does, the warnings and
/// what refuses it.
fn preview(ui: &mut egui::Ui, plan: &Plan, height: f32) {
    egui::ScrollArea::vertical()
        .id_salt("act-preview")
        .max_height(height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            if let Some(command) = &plan.command {
                ui.label(RichText::new("It runs:").weak());
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.add(egui::Label::new(RichText::new(command.line()).monospace()).wrap())
                        .on_hover_text(command.program.display().to_string());
                });
                ui.add_space(4.0);
            }
            for words in &plan.words {
                ui.add(egui::Label::new(words.as_str()).wrap());
            }
            if let Some(check) = &plan.check {
                ui.add_space(4.0);
                check_lines(ui, check);
            }
            for warning in &plan.warnings {
                ui.add_space(2.0);
                ui.add(
                    egui::Label::new(RichText::new(warning.as_str()).color(style::amber(ui)))
                        .wrap(),
                );
            }
            for problem in &plan.problems {
                ui.add_space(2.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("Refused: {problem}")).color(style::red(ui)),
                    )
                    .wrap(),
                );
            }
        });
}

/// Make public's word check (EP-42): the history's summary, in amber when nothing was checked or
/// something asks to be looked at, then each commit that hit by short hash and subject. A word or
/// a line that holds one is never shown (D-3).
fn check_lines(ui: &mut egui::Ui, check: &Check) {
    let colour = match check {
        Check::Read(scan) if scan.hits.is_empty() => ui.visuals().text_color(),
        _ => style::amber(ui),
    };
    ui.add(egui::Label::new(RichText::new(check.summary()).color(colour)).wrap())
        .on_hover_text(
            "One read of the history with git log, which writes nothing: every commit's message \
             and added lines against the word list that crw setting private_words names. The \
             words and the lines that hold them are never shown, logged or kept.",
        );
    if let Check::Read(scan) = check
        && !scan.hits.is_empty()
    {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for hit in scan.hits.iter().take(HITS_SHOWN) {
                ui.add(egui::Label::new(RichText::new(hit.words()).monospace()).wrap());
            }
            if scan.hits.len() > HITS_SHOWN {
                ui.label(
                    RichText::new(format!(
                        "and {} more: crw act <project> make-public lists them all",
                        scan.hits.len() - HITS_SHOWN
                    ))
                    .weak(),
                );
            }
        });
        ui.add(
            egui::Label::new(
                RichText::new(
                    "The words and the lines that hold them are not shown here: git show <hash> \
                     in a terminal shows each commit.",
                )
                .weak(),
            )
            .wrap(),
        );
    }
}

/// What a run did: how it ended, what it printed, and whether the ledger took its line.
fn done_lines(ui: &mut egui::Ui, ran: &Ran, height: f32) {
    match &ran.end {
        End::Done => {}
        End::Warned(why) => {
            ui.add(egui::Label::new(RichText::new(why).color(style::amber(ui))).wrap());
        }
        End::Failed(why) => {
            ui.add(egui::Label::new(RichText::new(why).color(style::red(ui))).wrap());
            ui.label(
                RichText::new("Nothing is retried: look, then try again from the menu.").weak(),
            );
        }
    }
    ui.add(egui::Label::new(RichText::new(&ran.command).monospace().small().weak()).wrap());
    if !ran.printed.is_empty() {
        egui::ScrollArea::vertical()
            .id_salt("act-printed")
            .max_height(height * 0.5)
            .show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(&ran.printed).monospace().small()).wrap());
            });
    }
    if let Some(why) = &ran.unlogged {
        ui.add(egui::Label::new(RichText::new(why).color(style::amber(ui))).wrap());
    }
}

/// The dialog's buttons on one line at its right, the first added rightmost.
fn buttons(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add);
    });
}

impl Board {
    /// The runner the actions use: a test's or a screenshot run's canned one, else the real one.
    pub(crate) fn act_runner(&self) -> Arc<dyn Runner> {
        if let Some(runner) = &self.act_runner {
            return runner.clone();
        }
        if self.fixtures || cfg!(test) {
            return Arc::new(Canned::default());
        }
        Arc::new(actions::Real)
    }

    /// What a plan of `verb` (on `project`, none for Back up now) is read from, as the window
    /// stands now: the project's origin read again by path (a screenshot run's canned one), the
    /// account's list as cached, the window's refusals for the project, the backup job's settings.
    pub(crate) fn act_asked(&self, project: Option<&str>, verb: Verb) -> Result<Asked, String> {
        let account = self
            .portfolio
            .account
            .as_ref()
            .and_then(|fetched| fetched.value.as_ref().ok())
            .cloned();
        let registry = || {
            self.store()
                .load()
                .map(|loaded| loaded.registry)
                .map_err(|e| format!("the registry could not be read: {e}"))
        };
        let backup = match verb {
            Verb::BackUpNow => registry()?.backup_files().0,
            _ => None,
        };
        // Make public's word list (EP-42), read again for each plan; a screenshot run's is
        // synthetic, as its runner's history is.
        let words = match verb {
            Verb::MakePublic if self.fixtures => Some(Ok(wordcheck::List::parse(CANNED_WORDS))),
            Verb::MakePublic => wordcheck::List::named(registry()?.private_words()),
            _ => None,
        };
        let (target, refusals) = match project {
            Some(name) => {
                let row = self
                    .portfolio
                    .project(name)
                    .ok_or_else(|| format!("{name} is not a registered project"))?;
                let folder = row
                    .folder
                    .as_ref()
                    .ok_or_else(|| format!("{name}: its folder is missing"))?;
                let target = if self.fixtures {
                    Target {
                        name: row.name().to_owned(),
                        folder: folder.path.clone(),
                        data: row.project.data,
                        has_commit: Some(true),
                        repository: true,
                        remote: row.badges.remote.clone(),
                    }
                } else {
                    // Unknown while git's status is unread or failed (EP-34's review).
                    let has_commit = row.git.as_ref().and_then(crate::portfolio::Git::has_commit);
                    Target::read(row.name(), &folder.path, row.project.data, has_commit)
                };
                // A screenshot run polls no session, so it sees none there (the formalise's way).
                let view = if self.fixtures {
                    crate::revert::View::default()
                } else {
                    self.work_view(name, "act on it")
                };
                let mut refusals: Vec<String> = view
                    .working
                    .iter()
                    .map(|working| format!("{working}: act on it once it has ended"))
                    .collect();
                refusals.extend(view.refusals);
                (Some(target), refusals)
            }
            // A second run of the job while the first goes on: -Force would let both write.
            None if self.act_detached.is_some() => (
                None,
                vec!["the backup job the window started is still running".to_owned()],
            ),
            None => (None, Vec::new()),
        };
        Ok(Asked {
            facts: Facts {
                verb,
                target,
                account,
                refusals,
                backup,
                words,
            },
            runner: self.act_runner(),
            ledger: self.ledger().map(Path::to_owned),
        })
    }

    /// Opens the confirmation of `verb` on `project` (EP-34), its plan read on a worker thread; a
    /// screenshot run reads it at once with its canned runner and runs nothing.
    pub fn open_act(&mut self, project: Option<String>, verb: Verb) {
        if self.act.is_some() {
            return;
        }
        match self.act_asked(project.as_deref(), verb) {
            Ok(asked) if self.fixtures => {
                let plan = actions::plan(&asked.facts, asked.runner.as_ref());
                self.act = Some(Dialog::ready(plan));
            }
            Ok(asked) => self.act = Some(Dialog::open(asked, self.ctx())),
            Err(why) => self.warn(why),
        }
    }

    /// The dialog's action button: the plan read again with the window as it stands now, and run
    /// if it is the one shown; a screenshot run runs nothing.
    pub fn confirm_act(&mut self) {
        if self.fixtures {
            self.warn("A screenshot run runs nothing: its projects are fixtures or a copy.");
            return;
        }
        let Some((project, verb)) = self
            .act
            .as_ref()
            .and_then(Dialog::confirmable)
            .map(|plan| (plan.project.clone(), plan.verb))
        else {
            return;
        };
        match self.act_asked(project.as_deref(), verb) {
            Ok(asked) => {
                let ctx = self.ctx().clone();
                if let Some(dialog) = &mut self.act {
                    dialog.confirm(asked, &ctx);
                }
            }
            Err(why) => self.fail(format!("nothing ran: {why}")),
        }
    }

    /// The dialog put away, unless its action runs; a running backup is left to run, its answer
    /// read by the window (EP-34's review).
    pub fn close_act(&mut self) {
        let Some(dialog) = self.act.as_mut() else {
            return;
        };
        if let Some(detached) = dialog.detach() {
            self.act_detached = Some(detached);
            self.act = None;
            self.notify("The backup job runs on: the status bar says how it ended.");
        } else if !dialog.is_running() {
            self.act = None;
        }
    }

    /// Whether an action runs, the dialog's or a backup left running, which quitting waits for
    /// (EP-34).
    pub fn acting(&self) -> bool {
        self.act.as_ref().is_some_and(Dialog::is_running) || self.act_detached.is_some()
    }

    /// Takes in the workers' answers, the dialog's and a backup's left running.
    pub fn receive_act(&mut self, ctx: &egui::Context) {
        if let Some(outcome) = self
            .act_detached
            .as_ref()
            .and_then(|detached| detached.receive(ctx))
        {
            self.act_detached = None;
            self.after_act(outcome);
        }
        let Some(dialog) = self.act.as_mut() else {
            return;
        };
        if let Some(outcome) = dialog.receive(ctx) {
            self.after_act(outcome);
        }
    }

    /// After a run, the project is read again and GitHub asked again as F5 asks it (the account's
    /// list with every head and run, so the daily look keeps its pace: EP-34's review), or the
    /// backup job is read again; the status bar says how it ended.
    fn after_act(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Changed => {}
            Outcome::Lost(why) => self.fail(why),
            Outcome::Ran(ran) => {
                match &ran.project {
                    Some(project) => {
                        self.loader.read_project(project, Moment::Acted);
                        self.loader.read_online(true);
                    }
                    None => self.loader.read_backups(),
                }
                let mut text = ran.words();
                if let Some(why) = &ran.unlogged {
                    text.push_str(&format!(" ({why})"));
                }
                match &ran.end {
                    End::Done if ran.unlogged.is_none() => self.notify(text),
                    End::Done | End::Warned(_) => self.warn(text),
                    End::Failed(_) => self.fail(text),
                }
            }
        }
    }

    /// The dialog over the window, while it is open. Escape and Cancel close it, a click beside it
    /// does not, and nothing does while its action runs but a backup's Close.
    pub fn act_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.act.as_ref() else {
            return;
        };
        let rect = ctx.content_rect();
        let width = WIDTH.min(rect.width() - 80.0);
        let height = (rect.height() * BODY_SHARE).max(160.0);
        let modal = egui::Modal::new(egui::Id::unique("act-dialog"))
            .show(ctx, |ui| dialog.ui(ui, width, height));
        let closable =
            (!dialog.is_running() || dialog.detachable()) && !modal.backdrop_response.clicked();
        match modal.inner {
            Some(action) => self.actions.push(action),
            None if closable && modal.should_close() => self.actions.push(Action::CloseAct),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::Instant;

    use crw_core::ledger::Ledger;
    use eframe::egui::PointerButton;

    use super::*;
    use crate::board::{Places, View as BoardView};
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::testing::{self, Screen};

    /// A runner that keeps what it ran and answers each command with `exit`.
    struct Recording {
        ran: Mutex<Vec<String>>,
        exit: Exit,
    }

    impl Runner for Recording {
        fn run(&self, command: &Command) -> Result<Exit, String> {
            if command.args.first().map(String::as_str) == Some("auth") {
                return Canned::default().run(command);
            }
            self.ran.lock().unwrap().push(command.line());
            Ok(self.exit.clone())
        }
    }

    fn board(scratch: &Scratch) -> Board {
        let places = Places {
            store: fake_store(scratch),
            ledger: Some(scratch.0.join("ledger.sqlite")),
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        Board::new(
            places,
            &ctx,
            BoardView::default(),
            egui::ThemePreference::Light,
        )
    }

    /// A project folder that is a repository with `origin` (written once, so a test may change
    /// it), and what a plan of `verb` on it is read from.
    fn asked(scratch: &Scratch, verb: Verb, runner: Arc<dyn Runner>) -> Asked {
        let folder = scratch.0.join("lantern");
        let config = folder.join(".git").join("config");
        if !config.exists() {
            std::fs::create_dir_all(folder.join(".git")).unwrap();
            std::fs::write(
                &config,
                "[remote \"origin\"]\n\turl = https://github.com/example/lantern.git\n",
            )
            .unwrap();
        }
        let account = vec![crw_core::badges::hosting::Listed {
            repo: crw_core::badges::hosting::Repo {
                owner: "example".to_owned(),
                name: "lantern".to_owned(),
            },
            visibility: crw_core::badges::hosting::Visibility::Private,
            archived: false,
            pushed_at: None,
            default_branch: Some("main".to_owned()),
        }];
        Asked {
            facts: Facts {
                verb,
                target: Some(Target::read("lantern", &folder, false, Some(true))),
                account: Some(account),
                refusals: Vec::new(),
                backup: None,
                words: None,
            },
            runner,
            ledger: Some(scratch.0.join("ledger.sqlite")),
        }
    }

    /// Make public's second tick (EP-42): the commits that hit listed, the button refused until
    /// the tick, which a plan shown afresh clears; the history read with the canned runner and a
    /// synthetic list, so nothing real runs.
    #[test]
    fn make_public_asks_a_second_tick_when_the_history_hits() {
        let scratch = Scratch::new("act-tick");
        let mut listed = asked(&scratch, Verb::MakePublic, Arc::new(Canned::default()));
        listed.facts.words = Some(Ok(wordcheck::List::parse(CANNED_WORDS)));
        let ctx = egui::Context::default();
        let mut dialog = Dialog::open(listed.clone(), &ctx);
        wait(&mut dialog);
        let Stage::Ready { plan, .. } = &dialog.stage else {
            panic!("{:?}", dialog.stage);
        };
        assert!(plan.asks_tick(), "{:?}", plan.check);
        assert!(dialog.confirmable().is_none(), "not before the tick");
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            dialog.ui(ui, 700.0, 900.0);
        });
        assert!(shows(
            &output,
            "The history: 4 commits read against a list of 1 word: 2 hit."
        ));
        assert!(shows(
            &output,
            "b2a1908 Add the release checklist (2 lines)"
        ));
        assert!(shows(
            &output,
            "d4c3b2a (its subject holds a listed word) (1 line)"
        ));
        assert!(
            !output.shapes.iter().any(|clipped| matches!(
                &clipped.shape,
                egui::Shape::Text(text) if text.galley.text().to_lowercase().contains("moonstone")
            )),
            "no word is shown"
        );
        assert_eq!(press(&dialog, "Make public"), None, "the button waits");
        assert_eq!(
            press(
                &dialog,
                "Make it public anyway: I have looked at these commits"
            ),
            Some(Action::TickActAnyway(true))
        );
        dialog.tick(true);
        assert!(dialog.confirmable().is_some());
        assert_eq!(press(&dialog, "Make public"), Some(Action::ConfirmAct));
        // The press reads the plan again, the history with it: unchanged, the command runs (the
        // canned runner refuses it, so it fails in its words), which shows the pass is stable.
        dialog.confirm(listed.clone(), &ctx);
        let Some(Outcome::Ran(ran)) = wait(&mut dialog) else {
            panic!("ran: {:?}", dialog.stage);
        };
        assert!(matches!(&ran.end, End::Failed(why) if why.contains("runs nothing")));
        // A history changed since the preview shows afresh, unticked, and runs nothing.
        let mut dialog = Dialog::open(listed.clone(), &ctx);
        wait(&mut dialog);
        dialog.tick(true);
        let mut changed = listed.clone();
        changed.facts.words = Some(Ok(wordcheck::List::parse("moonstone\nchecklist\n")));
        dialog.confirm(changed, &ctx);
        assert!(matches!(wait(&mut dialog), Some(Outcome::Changed)));
        assert!(dialog.confirmable().is_none(), "the tick is cleared");
        // Without a list, the dialog says so and asks for no tick.
        let mut plain = Dialog::open(
            asked(&scratch, Verb::MakePublic, Arc::new(Canned::default())),
            &ctx,
        );
        wait(&mut plain);
        assert!(plain.confirmable().is_some());
        let output = screen.pass(Vec::new(), |ui| {
            plain.ui(ui, 700.0, 900.0);
        });
        assert!(
            output.shapes.iter().any(|clipped| matches!(
                &clipped.shape,
                egui::Shape::Text(text) if text.galley.text().starts_with("No word list is named")
            )),
            "said"
        );
    }

    /// Presses the button that reads `label` in the dialog; what it chose.
    fn press(dialog: &Dialog, label: &str) -> Option<Action> {
        let mut screen = Screen::new();
        let mut chosen = None;
        screen.pass(Vec::new(), |ui| chosen = dialog.ui(ui, 700.0, 500.0));
        let output = screen.pass(Vec::new(), |ui| chosen = dialog.ui(ui, 700.0, 500.0));
        let at = testing::text_at(&output, label)?;
        for events in [
            testing::moved(at),
            testing::button(at, PointerButton::Primary, true),
            testing::button(at, PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| {
                if let Some(action) = dialog.ui(ui, 700.0, 500.0) {
                    chosen = Some(action);
                }
            });
        }
        chosen
    }

    /// A screenshot run's readings in short: lantern a private repository with its origin,
    /// compass an orphan of the fake account, each a repository by its git reading.
    fn hosted(portfolio: &mut crate::portfolio::Portfolio) {
        use crw_core::badges::hosting::{self, Listed, Repo, Visibility};
        let listed = |name: &str| Listed {
            repo: Repo {
                owner: "example".to_owned(),
                name: name.to_owned(),
            },
            visibility: Visibility::Private,
            archived: false,
            pushed_at: None,
            default_branch: Some("main".to_owned()),
        };
        let account = vec![listed("lantern"), listed("compass")];
        for row in &mut portfolio.projects {
            if row.name() == "lantern" {
                row.badges.remote = Some("https://github.com/example/lantern.git".to_owned());
            }
            row.badges.activity = Some(crw_core::badges::activity::Activity {
                oid: Some("f".repeat(40)),
                last_commit: None,
                changed: 0,
                branch: Some("main".to_owned()),
                upstream: None,
                ahead_behind: None,
            });
            row.badges.hosting =
                hosting::classify(&[row.name()], row.badges.remote.as_deref(), Some(&account));
        }
        portfolio.account = Some(crate::portfolio::Fetched {
            value: Ok(account),
            at: chrono::Utc::now(),
        });
    }

    fn shows(output: &egui::FullOutput, part: &str) -> bool {
        output.shapes.iter().any(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) => shape.galley.text().contains(part),
            _ => false,
        })
    }

    fn wait(dialog: &mut Dialog) -> Option<Outcome> {
        let ctx = egui::Context::default();
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(20) {
            let outcome = dialog.receive(&ctx);
            if dialog.receiver.is_none() {
                return outcome;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("the worker did not answer");
    }

    #[test]
    fn the_dialog_runs_the_plan_it_shows_and_logs_it() {
        let scratch = Scratch::new("act-run");
        let runner = Arc::new(Recording {
            ran: Mutex::new(Vec::new()),
            exit: Exit {
                code: Some(0),
                stdout: "✓ Edited repository example/lantern\n".to_owned(),
                stderr: String::new(),
            },
        });
        let ctx = egui::Context::default();
        let mut dialog = Dialog::open(asked(&scratch, Verb::MakePublic, runner.clone()), &ctx);
        assert!(matches!(dialog.stage, Stage::Reading));
        assert!(wait(&mut dialog).is_none(), "a preview read is no outcome");
        let plan = dialog.confirmable().expect("a plan to run").clone();
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            dialog.ui(ui, 700.0, 900.0);
        });
        assert!(shows(&output, "Make lantern public?"));
        assert!(shows(
            &output,
            "gh repo edit example/lantern --visibility public"
        ));
        assert!(shows(
            &output,
            "Changing repository visibility can have unexpected"
        ));
        assert!(shows(&output, "publishes it at once"));
        assert_eq!(press(&dialog, "Make public"), Some(Action::ConfirmAct));
        assert_eq!(press(&dialog, "Cancel"), Some(Action::CloseAct));
        assert!(runner.ran.lock().unwrap().is_empty(), "nothing ran yet");

        dialog.confirm(asked(&scratch, Verb::MakePublic, runner.clone()), &ctx);
        assert!(dialog.is_running());
        let Some(Outcome::Ran(ran)) = wait(&mut dialog) else {
            panic!("ran: {:?}", dialog.stage);
        };
        assert_eq!(ran.end, End::Done);
        assert_eq!(
            runner.ran.lock().unwrap().as_slice(),
            [plan.command.unwrap().line()]
        );
        let read = Ledger::open_read_only(&scratch.0.join("ledger.sqlite")).unwrap();
        let rows = read.events(actions::HOSTING, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].project.as_deref(), Some("lantern"));
        assert_eq!(press(&dialog, "Close"), Some(Action::CloseAct));
    }

    #[test]
    fn a_plan_changed_since_the_preview_runs_nothing() {
        let scratch = Scratch::new("act-changed");
        let runner = Arc::new(Recording {
            ran: Mutex::new(Vec::new()),
            exit: Exit {
                code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            },
        });
        let ctx = egui::Context::default();
        let mut dialog = Dialog::open(asked(&scratch, Verb::RemoveRemote, runner.clone()), &ctx);
        wait(&mut dialog);
        // The origin changed by hand since the preview.
        std::fs::write(
            scratch.0.join("lantern").join(".git").join("config"),
            "[remote \"origin\"]\n\turl = https://gitlab.com/example/lantern.git\n",
        )
        .unwrap();
        dialog.confirm(asked(&scratch, Verb::RemoveRemote, runner.clone()), &ctx);
        assert!(matches!(wait(&mut dialog), Some(Outcome::Changed)));
        let Stage::Ready { plan, changed } = &dialog.stage else {
            panic!("{:?}", dialog.stage);
        };
        assert!(*changed && plan.words[0].contains("gitlab.com"));
        assert!(runner.ran.lock().unwrap().is_empty(), "nothing ran");
        // A session at work there since: shown again, refused, Close alone.
        let mut busy = asked(&scratch, Verb::RemoveRemote, runner.clone());
        busy.facts.refusals =
            vec!["a visual session 3c9d2e71 is busy: act on it once it has ended".to_owned()];
        dialog.confirm(busy, &ctx);
        assert!(matches!(wait(&mut dialog), Some(Outcome::Changed)));
        assert!(dialog.confirmable().is_none());
        assert_eq!(press(&dialog, "Close"), Some(Action::CloseAct));
        assert!(runner.ran.lock().unwrap().is_empty());
    }

    #[test]
    fn a_failure_is_shown_in_gh_s_words() {
        let scratch = Scratch::new("act-failed");
        let runner = Arc::new(Recording {
            ran: Mutex::new(Vec::new()),
            exit: Exit {
                code: Some(1),
                stdout: String::new(),
                stderr: "HTTP 403: Must have admin rights to Repository.".to_owned(),
            },
        });
        let ctx = egui::Context::default();
        let mut dialog = Dialog::open(asked(&scratch, Verb::MakePublic, runner.clone()), &ctx);
        wait(&mut dialog);
        dialog.confirm(asked(&scratch, Verb::MakePublic, runner.clone()), &ctx);
        let Some(Outcome::Ran(ran)) = wait(&mut dialog) else {
            panic!("{:?}", dialog.stage);
        };
        assert!(matches!(&ran.end, End::Failed(why) if why.contains("HTTP 403")));
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            dialog.ui(ui, 700.0, 900.0);
        });
        assert!(shows(&output, "lantern: Make public did not go through"));
        assert!(shows(&output, "Nothing is retried"));
        assert_eq!(runner.ran.lock().unwrap().len(), 1, "run once");
    }

    /// A running backup's dialog may close (EP-34's review): the window keeps its answer, Quit and
    /// a restart wait for it, a second run is refused meanwhile, and its end is said.
    #[test]
    fn a_running_backup_may_be_left_to_run() {
        let scratch = Scratch::new("act-detach");
        let jobs = scratch.0.join("jobs");
        std::fs::create_dir_all(&jobs).unwrap();
        let settings = jobs.join("nightly.psd1");
        let target = scratch.0.join("target").display().to_string();
        std::fs::write(&settings, format!("@{{\n    Target = '{target}'\n}}\n")).unwrap();
        std::fs::write(jobs.join("nightly.ps1"), "").unwrap();
        let runner = Arc::new(Recording {
            ran: Mutex::new(Vec::new()),
            exit: Exit {
                code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            },
        });
        let asked = Asked {
            facts: Facts {
                verb: Verb::BackUpNow,
                target: None,
                account: None,
                refusals: Vec::new(),
                backup: Some(settings),
                words: None,
            },
            runner: runner.clone(),
            ledger: Some(scratch.0.join("ledger.sqlite")),
        };
        let ctx = egui::Context::default();
        let mut dialog = Dialog::open(asked.clone(), &ctx);
        wait(&mut dialog);
        assert!(dialog.confirmable().is_some(), "{:?}", dialog.stage);
        dialog.confirm(asked, &ctx);
        assert!(dialog.detachable());
        assert_eq!(press(&dialog, "Close"), Some(Action::CloseAct));

        let mut board = board(&scratch);
        board.act = Some(dialog);
        board.close_act();
        assert!(board.act.is_none() && board.act_detached.is_some());
        assert!(board.acting(), "Quit waits for it");
        assert!(
            board
                .restart_refusals()
                .iter()
                .any(|why| why.contains("a backup the window started is still running"))
        );
        let again = board.act_asked(None, Verb::BackUpNow).unwrap();
        assert_eq!(
            again.facts.refusals,
            ["the backup job the window started is still running"]
        );
        let started = Instant::now();
        while board.act_detached.is_some() {
            assert!(started.elapsed() < Duration::from_secs(20), "no answer");
            board.receive_act(&ctx);
            thread::sleep(Duration::from_millis(20));
        }
        assert!(!board.acting());
        assert!(
            board
                .notice_text()
                .is_some_and(|text| text.starts_with("the backup job: backed up")),
            "{:?}",
            board.notice_text()
        );
        assert_eq!(runner.ran.lock().unwrap().len(), 1, "one run");
    }

    /// The window's dialog over the fixtures: the row's offered actions, a plan read at once with
    /// the canned runner (Delete refused for the scope), and a confirm refused.
    #[test]
    fn the_window_opens_a_fixtures_plan_and_runs_nothing() {
        let scratch = Scratch::new("act-board");
        let mut board = board(&scratch);
        board.portfolio = crate::portfolio::read_files(board.store());
        board.fixtures = true;
        hosted(&mut board.portfolio);
        let lantern = board.portfolio.project("lantern").unwrap();
        assert_eq!(offered(lantern), [Verb::MakePublic, Verb::RemoveRemote]);
        board.open_act(Some("lantern".to_owned()), Verb::MakePublic);
        // The canned history hits the canned list (EP-42): the tick comes first.
        assert!(board.act.as_ref().unwrap().confirmable().is_none());
        board.run(Action::TickActAnyway(true));
        let dialog = board.act.as_ref().expect("the dialog");
        let plan = dialog
            .confirmable()
            .expect("lantern is private and listed")
            .clone();
        assert!(matches!(&plan.check, Some(Check::Read(scan)) if scan.hits.len() == 2));
        assert_eq!(
            plan.command.as_ref().unwrap().args[..3],
            ["repo", "edit", "example/lantern"]
        );
        board.confirm_act();
        assert!(
            board
                .notice_text()
                .is_some_and(|text| text.starts_with("A screenshot run runs nothing")),
            "{:?}",
            board.notice_text()
        );
        assert!(
            board.act.as_ref().unwrap().confirmable().is_some(),
            "still the preview"
        );
        board.close_act();
        assert!(board.act.is_none());
        // The orphan's Delete, refused for the token's scope; the restart refused while open.
        board.open_act(Some("compass".to_owned()), Verb::Delete);
        let dialog = board.act.as_ref().unwrap();
        let Stage::Ready { plan, .. } = &dialog.stage else {
            panic!("{:?}", dialog.stage);
        };
        assert_eq!(plan.target, "example/compass");
        assert!(plan.problems[0].contains("gh auth refresh -s delete_repo"));
        assert!(
            board
                .restart_refusals()
                .iter()
                .any(|why| why.contains("hosting or backup action"))
        );
        board.close_act();
        board.open_act(Some("nowhere".to_owned()), Verb::RemoveRemote);
        assert!(board.act.is_none());
    }
}
