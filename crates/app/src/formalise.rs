//! The window's Formalise… (D-11; EP-21a), on `crw_core::formalise`'s engine (EP-21). A
//! project's row menu and its Detail pane's House style section open a preview over the window,
//! like the Revert's: the four house-style files, each with its change, the roadmap's form, the
//! rule block and each new file's text, then Write and Cancel, or Close alone when the plan is
//! refused or writes nothing. The plan reads files by path only, on a worker thread, and the
//! window adds what refuses a write besides the files ([`Board::work_view`], the Revert's): its
//! own runs and starts in the project, a visual launch on its way, a session the hooks or the poll
//! report at work or waiting there, and the poll unanswered. A tab open there with nothing to do
//! is named, never a refusal. At Write the plan is read again and written only when it is the plan
//! shown; otherwise the dialog shows the new one and nothing is written. The write is logged as
//! the `formalise` row that a revert of a session started before it redoes (EP-21a), and the
//! files are read again. A screenshot run reads a fixture's plan and writes nothing (rule 5).

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crw_core::files;
use crw_core::formalise::{self, Applied, Change, Plan, Readme};
use crw_core::registry::State;
use crw_core::roadmap::table::Profile;
use eframe::egui::{self, RichText};

use crate::board::{Action, Board};
use crate::revert::View;
use crate::style;

/// How often the window looks for the worker's answer while it waits.
const POLL: Duration = Duration::from_millis(250);

/// The dialog's widest, in points, and the share of the window's height its body may take.
const WIDTH: f32 = 760.0;
const BODY_SHARE: f32 = 0.6;

/// What a plan is read from: the project, what the window knows of the sessions there, and where
/// the ledger and the backups are.
#[derive(Debug, Clone)]
pub struct Asked {
    pub project: String,
    pub folder: PathBuf,
    pub view: View,
    pub ledger: Option<PathBuf>,
    pub backups: PathBuf,
}

/// The plan of `asked`'s project, with the window's refusals among its problems, so that the
/// preview says them and the engine's write refuses them too.
pub(crate) fn read_plan(asked: &Asked) -> Plan {
    let mut plan = formalise::plan(&asked.folder, &asked.project);
    plan.problems.extend(
        asked
            .view
            .working
            .iter()
            .map(|working| format!("{working}: formalise once it has ended")),
    );
    plan.problems.extend(asked.view.refusals.iter().cloned());
    plan
}

/// What a confirmed write did.
#[derive(Debug, Clone)]
pub struct Written {
    pub applied: Applied,
    /// Why the ledger did not take the write's row, which a revert then does not redo.
    pub unlogged: Option<String>,
}

/// What the worker answers.
enum Answer {
    Planned(Plan),
    /// The plan read again at the owner's word is not the one they saw: nothing was written.
    Changed(Plan),
    Written(Result<Written, String>),
}

/// Writes `seen` once the plan read again from `asked` is the one the owner saw; else the new
/// plan, and nothing is written. The write is logged for a revert to redo.
fn confirm(asked: &Asked, seen: &Plan, today: &str) -> Answer {
    let again = read_plan(asked);
    if again != *seen {
        return Answer::Changed(again);
    }
    match formalise::apply(seen, &asked.backups, today) {
        Ok(applied) => {
            let unlogged = match &asked.ledger {
                Some(path) if !applied.written.is_empty() => {
                    formalise::log(path, &asked.project, &applied).err()
                }
                _ => None,
            };
            Answer::Written(Ok(Written { applied, unlogged }))
        }
        // A file changed in the moment between the second reading and the write.
        Err(formalise::Error::Changed(_)) => Answer::Changed(read_plan(asked)),
        Err(e) => Answer::Written(Err(e.to_string())),
    }
}

/// What became of a write the owner confirmed, for the window to say and to read the files again.
#[derive(Debug)]
pub enum Outcome {
    Written(Written),
    NotWritten(String),
    /// The project or its sessions changed since the preview, so nothing was written.
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
    /// The write runs; the preview stays in view.
    Writing(Plan),
    Done(Written),
    /// Nothing to preview, or the write did not run: why.
    Failed(String),
}

/// The preview of formalising one project, and the write it leads to.
pub struct Dialog {
    pub project: String,
    pub stage: Stage,
    /// Tabs open in the project with nothing to do when the dialog opened: named, for the owner
    /// to know that they keep the rules they read at their start.
    pub idle: Vec<String>,
    /// The project is in the unformalised state, which a formalise leaves as it is (D-6).
    pub unformalised: bool,
    receiver: Option<Receiver<Answer>>,
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
    pub fn open(asked: Asked, unformalised: bool, ctx: &egui::Context) -> Self {
        let project = asked.project.clone();
        let idle = asked.view.idle.clone();
        let receiver = spawn("formalise preview", ctx, move || {
            Answer::Planned(read_plan(&asked))
        });
        let stage = match receiver {
            Some(_) => Stage::Reading,
            None => Stage::Failed("the preview's thread did not start".to_owned()),
        };
        Self {
            project,
            stage,
            idle,
            unformalised,
            receiver,
        }
    }

    /// The preview of a plan read elsewhere: a screenshot run's, a test's.
    pub fn ready(plan: Plan, idle: Vec<String>, unformalised: bool) -> Self {
        Self {
            project: plan.project.clone(),
            stage: Stage::Ready {
                plan,
                changed: false,
            },
            idle,
            unformalised,
            receiver: None,
        }
    }

    /// A write runs: nothing closes the dialog.
    pub fn is_writing(&self) -> bool {
        matches!(self.stage, Stage::Writing(_))
    }

    /// The plan shown, when the owner may write it: not refused, and with a file to write.
    pub fn confirmable(&self) -> Option<&Plan> {
        match &self.stage {
            Stage::Ready { plan, .. } if !plan.is_refused() && plan.writes() > 0 => Some(plan),
            _ => None,
        }
    }

    /// Writes at the owner's word, on a worker thread: the plan read again from `asked` first.
    pub fn confirm(&mut self, asked: Asked, ctx: &egui::Context) {
        let Some(seen) = self.confirmable().cloned() else {
            return;
        };
        let shown = seen.clone();
        let today = files::today();
        match spawn("formalise", ctx, move || confirm(&asked, &seen, &today)) {
            Some(receiver) => {
                self.receiver = Some(receiver);
                self.stage = Stage::Writing(shown);
            }
            None => {
                self.stage =
                    Stage::Failed("the write's thread did not start: nothing changed".to_owned());
            }
        }
    }

    /// Takes in the worker's answer when it has come, and asks for frames while it waits. What
    /// became of a confirmed write, for the window; nothing for a preview read.
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
                let writing = self.is_writing();
                let why = if writing {
                    "the write's thread ended without an answer: look at the project's files"
                } else {
                    "the preview's thread ended without an answer"
                };
                self.stage = Stage::Failed(why.to_owned());
                return writing.then(|| Outcome::NotWritten(why.to_owned()));
            }
        };
        self.receiver = None;
        match answer {
            Answer::Planned(plan) => {
                self.stage = Stage::Ready {
                    plan,
                    changed: false,
                };
                None
            }
            Answer::Changed(plan) => {
                self.stage = Stage::Ready {
                    plan,
                    changed: true,
                };
                Some(Outcome::Changed)
            }
            Answer::Written(Ok(written)) => {
                self.stage = Stage::Done(written.clone());
                Some(Outcome::Written(written))
            }
            Answer::Written(Err(why)) => {
                self.stage = Stage::Failed(why.clone());
                Some(Outcome::NotWritten(why))
            }
        }
    }

    /// The dialog's contents at `width`, its body at most `height` tall; the owner's choice.
    pub fn ui(&self, ui: &mut egui::Ui, width: f32, height: f32) -> Option<Action> {
        ui.set_width(width);
        let project = &self.project;
        let mut chosen = None;
        let close = |ui: &mut egui::Ui, chosen: &mut Option<Action>| {
            buttons(ui, |ui| {
                if ui.button("Close").clicked() {
                    *chosen = Some(Action::CloseFormalise);
                }
            });
        };
        match &self.stage {
            Stage::Reading => {
                ui.heading(format!("Formalise {project}?"));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!("Reading {project}'s files…"));
                });
                ui.add_space(8.0);
                buttons(ui, |ui| {
                    if ui.button("Cancel").clicked() {
                        chosen = Some(Action::CloseFormalise);
                    }
                });
            }
            Stage::Ready { plan, changed } => {
                ui.heading(format!("Formalise {project}?"));
                ui.add_space(4.0);
                if *changed {
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                "The project or its sessions changed since the preview, so \
                                 nothing was written. This is the plan as it stands now.",
                            )
                            .color(style::amber(ui)),
                        )
                        .wrap(),
                    );
                    ui.add_space(4.0);
                }
                self.preview(ui, plan, height, Some(&mut chosen));
                ui.add_space(8.0);
                if plan.is_refused() || plan.writes() == 0 {
                    close(ui, &mut chosen);
                } else {
                    buttons(ui, |ui| {
                        if ui.button("Cancel").clicked() {
                            chosen = Some(Action::CloseFormalise);
                        }
                        if ui
                            .button("Write")
                            .on_hover_text(
                                "The plan is read again first: the files are written only if \
                                 the project is as this preview shows it",
                            )
                            .clicked()
                        {
                            chosen = Some(Action::ConfirmFormalise);
                        }
                    });
                }
            }
            Stage::Writing(plan) => {
                ui.heading(format!("Formalising {project}…"));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("The plan read again, then the files written…");
                });
                ui.add_space(4.0);
                self.preview(ui, plan, height, None);
            }
            Stage::Done(written) => {
                let stopped = written.applied.failed.is_some();
                ui.heading(if stopped {
                    format!("{project}: the write stopped")
                } else {
                    format!("{project} is formalised")
                });
                ui.add_space(4.0);
                done_lines(ui, written);
                ui.add_space(8.0);
                close(ui, &mut chosen);
            }
            Stage::Failed(why) => {
                ui.heading(format!("Formalise {project}"));
                ui.add_space(4.0);
                ui.add(egui::Label::new(RichText::new(why).color(style::red(ui))).wrap());
                ui.add_space(8.0);
                close(ui, &mut chosen);
            }
        }
        chosen
    }

    /// The plan in a scroll area at most `height` tall: each file and its change, the roadmap's
    /// form (with Migrate… for an older one, where `chosen` takes a press: not while the write
    /// runs), what refuses the write, the idle tabs, the state, then the block and each new
    /// file's text.
    fn preview(
        &self,
        ui: &mut egui::Ui,
        plan: &Plan,
        height: f32,
        chosen: Option<&mut Option<Action>>,
    ) {
        ui.label(
            RichText::new(format!(
                "{}: a file that is there is never replaced, and CLAUDE.md changes only between \
                 the roadmap-wrangler markers, after a copy to the backups folder (D-11).",
                plan.folder.display()
            ))
            .small()
            .weak(),
        );
        ui.add_space(4.0);
        egui::ScrollArea::vertical()
            .id_salt("formalise-preview")
            .max_height(height)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                egui::Grid::new("formalise-files")
                    .num_columns(2)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        for step in &plan.steps {
                            ui.label(RichText::new(&step.name).monospace());
                            let words = RichText::new(step.words());
                            ui.add(
                                egui::Label::new(match step.change {
                                    Change::Refused(_) => words.color(style::red(ui)),
                                    Change::Keep => words.weak(),
                                    _ => words,
                                })
                                .wrap(),
                            );
                            ui.end_row();
                        }
                        if !plan.steps.is_empty() {
                            ui.label(RichText::new("roadmap/README.md").monospace());
                            ui.horizontal_wrapped(|ui| {
                                ui.label(readme_words(&plan.readme));
                                if plan.migrates().is_some()
                                    && let Some(chosen) = chosen
                                    && ui
                                        .small_button("Migrate…")
                                        .on_hover_text(MIGRATE_HINT)
                                        .clicked()
                                {
                                    *chosen = Some(Action::OpenMigration(self.project.clone()));
                                }
                            });
                            ui.end_row();
                        }
                    });
                for problem in &plan.problems {
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("Refused: {problem}")).color(style::red(ui)),
                        )
                        .wrap(),
                    );
                }
                if !self.idle.is_empty() {
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!(
                                "Open there with nothing to do: {}. It keeps the rules it read at \
                                 its start; the next session reads the block.",
                                self.idle.join("; ")
                            ))
                            .color(style::amber(ui)),
                        )
                        .wrap(),
                    );
                }
                if !plan.is_refused() && plan.writes() == 0 {
                    ui.label("Nothing to write: the project has every house-style file.");
                }
                if self.unformalised {
                    ui.label(
                        RichText::new(
                            "Its state stays unformalised: set it from the row's menu once its \
                             roadmap is written (D-6).",
                        )
                        .weak(),
                    );
                }
                for step in &plan.steps {
                    let Some(text) = step.shown() else {
                        continue;
                    };
                    let block = matches!(step.change, Change::AddBlock | Change::ReplaceBlock);
                    let title = if block {
                        format!("{}: the block", step.name)
                    } else {
                        format!("{}: the new file", step.name)
                    };
                    // The rule file's text opens at once, a new file's too: it holds the block.
                    egui::CollapsingHeader::new(title)
                        .id_salt(("formalise-text", step.name.as_str()))
                        .default_open(step.file == formalise::File::Rules)
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(RichText::new(text).monospace().small()).wrap(),
                            );
                        });
                }
            });
    }
}

/// What Migrate… does, on hover.
pub const MIGRATE_HINT: &str = "Open the launch dialog in Headless with the prompt of a session \
                                that converts the roadmap's tables to the house form (D-9): you \
                                send it";

/// The roadmap README's line of the preview, worded for the window.
fn readme_words(readme: &Readme) -> String {
    match readme {
        Readme::Missing => {
            "missing: a session writes the first roadmap (the plugin's plan skill)".to_owned()
        }
        Readme::Unreadable(why) => format!("could not be read: {why}"),
        Readme::Read(None) => "no phase table the parser reads".to_owned(),
        Readme::Read(Some(Profile::House)) => "house tables".to_owned(),
        Readme::Read(Some(profile)) => format!(
            "{}, read-only (D-9): a session migrates them",
            formalise::form_words(*profile)
        ),
    }
}

/// What a write did, a line each: each file written and the copy of `CLAUDE.md`, why it stopped,
/// and whether the ledger took its row.
fn done_lines(ui: &mut egui::Ui, written: &Written) {
    let applied = &written.applied;
    if applied.written.is_empty() {
        ui.label("Nothing was written: each file appeared since the preview, and is kept.");
    }
    for file in &applied.written {
        let what = match file.change {
            Change::Create => "created",
            Change::AddBlock => "the rule block added",
            Change::ReplaceBlock => "the rule block replaced",
            Change::Keep | Change::Refused(_) => "kept",
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(&file.name).monospace());
            ui.label(what);
        });
        if let Some(copy) = &file.copy {
            ui.indent(("formalise-copy", file.name.as_str()), |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("the file as it was is kept as {}", copy.display()))
                            .small()
                            .weak(),
                    )
                    .wrap(),
                );
            });
        }
    }
    if let Some(why) = &applied.failed {
        ui.add(
            egui::Label::new(
                RichText::new(format!(
                    "Stopped: {why}. Formalise… again finishes the job."
                ))
                .color(style::red(ui)),
            )
            .wrap(),
        );
    }
    if let Some(why) = &written.unlogged {
        ui.add(
            egui::Label::new(
                RichText::new(format!("{why}: a revert would not redo the write."))
                    .color(style::amber(ui)),
            )
            .wrap(),
        );
    }
}

/// The dialog's buttons on one line at its right, the first added rightmost.
fn buttons(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add);
    });
}

/// The status bar's words for a write: each file, then where `CLAUDE.md` as it was is kept.
fn notice(project: &str, written: &Written) -> String {
    let applied = &written.applied;
    let files: Vec<&str> = applied
        .written
        .iter()
        .map(|file| file.name.as_str())
        .collect();
    let mut text = if files.is_empty() {
        format!("{project}: nothing was written, each file appeared since the preview.")
    } else {
        format!("Formalised {project}: wrote {}.", files.join(", "))
    };
    if let Some(copy) = applied.written.iter().find_map(|file| file.copy.as_ref()) {
        text.push_str(&format!(
            " CLAUDE.md as it was is kept as {}.",
            copy.display()
        ));
    }
    text
}

impl Board {
    /// What a plan of `project` is read from, as the window stands now, and whether the project
    /// is in the unformalised state.
    pub(crate) fn formalise_asked(&self, project: &str) -> Result<(Asked, bool), String> {
        let row = self
            .portfolio
            .project(project)
            .ok_or_else(|| format!("{project} is not a registered project"))?;
        let folder = row
            .folder
            .as_ref()
            .ok_or_else(|| format!("{project}: its folder is missing"))?;
        let unformalised = row.project.state == State::Unformalised;
        Ok((
            Asked {
                project: row.name().to_owned(),
                folder: folder.path.clone(),
                view: self.work_view(project, "formalise"),
                ledger: self.ledger().map(Path::to_owned),
                backups: self.store().backups.clone(),
            },
            unformalised,
        ))
    }

    /// Opens the preview of formalising `project` (EP-21a), read on a worker thread. A screenshot
    /// run reads its fixture's plan at once, by path, with no session seen there (it polls none;
    /// `formalise=<project>:busy` cans one), and writes nothing.
    pub fn open_formalise(&mut self, project: &str) {
        if self.formalise.is_some() {
            return;
        }
        match self.formalise_asked(project) {
            Ok((mut asked, unformalised)) if self.fixtures => {
                asked.view = View::default();
                self.formalise = Some(Dialog::ready(read_plan(&asked), Vec::new(), unformalised));
            }
            Ok((asked, unformalised)) => {
                self.formalise = Some(Dialog::open(asked, unformalised, self.ctx()));
            }
            Err(why) => self.warn(why),
        }
    }

    /// The preview's Write: the plan read again with the window as it stands now, and written if
    /// it is the one shown; a screenshot run writes nothing.
    pub fn confirm_formalise(&mut self) {
        if self.fixtures {
            self.warn(
                "A screenshot run writes nothing into its projects: they are fixtures or a copy.",
            );
            return;
        }
        let Some(project) = self
            .formalise
            .as_ref()
            .and_then(Dialog::confirmable)
            .map(|plan| plan.project.clone())
        else {
            return;
        };
        match self.formalise_asked(&project) {
            Ok((asked, _)) => {
                let ctx = self.ctx().clone();
                if let Some(dialog) = &mut self.formalise {
                    dialog.confirm(asked, &ctx);
                }
            }
            Err(why) => self.fail(format!("{project}: nothing was written: {why}")),
        }
    }

    /// The preview put away, unless a write runs.
    pub fn close_formalise(&mut self) {
        if !self.formalise.as_ref().is_some_and(Dialog::is_writing) {
            self.formalise = None;
        }
    }

    /// Takes in the worker's answer: after a write the files are read again, and the status bar
    /// says what was written and where the copy of `CLAUDE.md` is.
    pub fn receive_formalise(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.formalise.as_mut() else {
            return;
        };
        let project = dialog.project.clone();
        let Some(outcome) = dialog.receive(ctx) else {
            return;
        };
        match outcome {
            Outcome::Changed => {}
            Outcome::Written(written) => {
                self.loader.read_files();
                let text = notice(&project, &written);
                if written.applied.failed.is_some() || written.unlogged.is_some() {
                    self.warn(format!("{text} Part of it needs a look: see the dialog."));
                } else {
                    self.notify(text);
                }
            }
            Outcome::NotWritten(why) => {
                self.loader.read_files();
                self.fail(format!("{project}: {why}"));
            }
        }
    }

    /// The preview over the window, while it is open. Escape and Cancel close it, a click beside
    /// it does not, and nothing does while a write runs.
    pub fn formalise_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.formalise.as_ref() else {
            return;
        };
        let rect = ctx.content_rect();
        let width = WIDTH.min(rect.width() - 80.0);
        let height = (rect.height() * BODY_SHARE).max(160.0);
        let modal = egui::Modal::new(egui::Id::unique("formalise-preview"))
            .show(ctx, |ui| dialog.ui(ui, width, height));
        let closable = !dialog.is_writing() && !modal.backdrop_response.clicked();
        match modal.inner {
            Some(action) => self.actions.push(action),
            None if closable && modal.should_close() => self.actions.push(Action::CloseFormalise),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Instant;

    use crw_core::ledger::Ledger;
    use eframe::egui::PointerButton;

    use super::*;
    use crate::board::{Places, View as BoardView};
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::testing::{self, Screen};

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

    fn asked(folder: &Path, scratch: &Scratch, view: View) -> Asked {
        Asked {
            project: "meadow".to_owned(),
            folder: folder.to_owned(),
            view,
            ledger: Some(scratch.0.join("ledger.sqlite")),
            backups: scratch.0.join("backups"),
        }
    }

    /// Presses the button that reads `label` in the dialog; what it chose. The preview's grid
    /// measures its columns in its first pass, so the button is found in the second.
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

    /// Whether `output` paints a text that holds `part`.
    fn shows(output: &egui::FullOutput, part: &str) -> bool {
        output.shapes.iter().any(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) => shape.galley.text().contains(part),
            _ => false,
        })
    }

    /// Waits for the worker's answer, a pass at a time.
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
    fn the_preview_writes_the_plan_it_shows_and_logs_it() {
        let scratch = Scratch::new("formalise-write");
        let folder = scratch.0.join("meadow");
        fs::create_dir_all(folder.join("roadmap")).unwrap();
        fs::write(folder.join("CLAUDE.md"), "# meadow\n\nOwn rules.\n").unwrap();
        let ctx = egui::Context::default();
        let mut dialog = Dialog::open(asked(&folder, &scratch, View::default()), false, &ctx);
        assert!(matches!(dialog.stage, Stage::Reading));
        assert!(wait(&mut dialog).is_none(), "a preview read is no outcome");
        let plan = dialog.confirmable().expect("a plan to write").clone();
        assert_eq!(plan.writes(), 4);
        // Drawn: each file, the block's text, Write and Cancel; nothing written yet.
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            dialog.ui(ui, 700.0, 900.0);
        });
        for text in [
            "CLAUDE.md",
            "roadmap/owner-tasks.md",
            "CLAUDE.md: the block",
            "roadmap/inbox.md: the new file",
            "Write",
        ] {
            assert!(testing::text_at(&output, text).is_some(), "{text}");
        }
        assert_eq!(press(&dialog, "Write"), Some(Action::ConfirmFormalise));
        assert_eq!(press(&dialog, "Cancel"), Some(Action::CloseFormalise));
        assert!(!folder.join("roadmap").join("inbox.md").exists());

        dialog.confirm(asked(&folder, &scratch, View::default()), &ctx);
        assert!(dialog.is_writing());
        let Some(Outcome::Written(written)) = wait(&mut dialog) else {
            panic!("written: {:?}", dialog.stage);
        };
        assert_eq!(written.applied.written.len(), 4);
        assert!(written.unlogged.is_none(), "{:?}", written.unlogged);
        let rules = fs::read_to_string(folder.join("CLAUDE.md")).unwrap();
        assert!(
            rules.starts_with("# meadow\n\nOwn rules.\n\n") && rules.contains(formalise::BEGIN)
        );
        let said = notice("meadow", &written);
        assert!(
            said.starts_with("Formalised meadow: wrote CLAUDE.md, roadmap/owner-tasks.md")
                && said.contains("CLAUDE.md as it was is kept as "),
            "{said}"
        );
        let read = Ledger::open_read_only(&scratch.0.join("ledger.sqlite")).unwrap();
        let rows = read.events(formalise::EVENT_KIND, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].project.as_deref(), Some("meadow"));
        assert_eq!(
            rows[0].detail.as_deref(),
            Some(written.applied.detail().as_str())
        );
        // Done: Close alone.
        assert_eq!(press(&dialog, "Close"), Some(Action::CloseFormalise));
    }

    #[test]
    fn a_plan_changed_since_the_preview_is_shown_again_and_nothing_is_written() {
        let scratch = Scratch::new("formalise-changed");
        let folder = scratch.0.join("meadow");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("CLAUDE.md"), "# meadow\n").unwrap();
        let ctx = egui::Context::default();
        let mut dialog = Dialog::open(asked(&folder, &scratch, View::default()), false, &ctx);
        wait(&mut dialog);
        fs::write(folder.join("CLAUDE.md"), "# meadow\n\nEdited since.\n").unwrap();
        dialog.confirm(asked(&folder, &scratch, View::default()), &ctx);
        assert!(matches!(wait(&mut dialog), Some(Outcome::Changed)));
        let Stage::Ready { plan, changed } = &dialog.stage else {
            panic!("{:?}", dialog.stage);
        };
        assert!(*changed);
        assert_eq!(
            plan.steps[0].before.as_deref(),
            Some("# meadow\n\nEdited since.\n")
        );
        assert!(!folder.join("roadmap").exists(), "nothing written");
        // A session at work there since the preview: shown again, refused.
        let busy = View {
            working: vec!["a visual session 3c9d2e71 is busy".to_owned()],
            ..View::default()
        };
        dialog.confirm(asked(&folder, &scratch, busy), &ctx);
        assert!(matches!(wait(&mut dialog), Some(Outcome::Changed)));
        assert!(dialog.confirmable().is_none(), "refused now");
        assert_eq!(press(&dialog, "Close"), Some(Action::CloseFormalise));
        assert!(!folder.join("roadmap").exists());
    }

    #[test]
    fn a_busy_project_is_refused_and_an_idle_tab_is_named() {
        let scratch = Scratch::new("formalise-busy");
        let folder = scratch.0.join("meadow");
        fs::create_dir_all(&folder).unwrap();
        let view = View {
            working: vec!["a visual session 3c9d2e71 is busy".to_owned()],
            idle: vec!["a manual session 9a3e7d41".to_owned()],
            refusals: vec![
                "the list of live sessions has not answered yet, so whether a session works \
                 there is not known: try again in a moment"
                    .to_owned(),
            ],
        };
        let plan = read_plan(&asked(&folder, &scratch, view.clone()));
        assert!(plan.is_refused());
        assert_eq!(
            plan.problems[0],
            "a visual session 3c9d2e71 is busy: formalise once it has ended"
        );
        let dialog = Dialog::ready(plan, view.idle, true);
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            dialog.ui(ui, 700.0, 500.0);
        });
        assert!(testing::text_at(&output, "Write").is_none());
        assert!(shows(&output, "Its state stays unformalised"));
        assert!(shows(
            &output,
            "Open there with nothing to do: a manual session 9a3e7d41."
        ));
        assert!(shows(&output, "Refused: a visual session 3c9d2e71 is busy"));
        assert_eq!(press(&dialog, "Close"), Some(Action::CloseFormalise));
        // An idle tab alone refuses nothing.
        let idle = View {
            idle: vec!["a manual session 9a3e7d41".to_owned()],
            ..View::default()
        };
        assert!(!read_plan(&asked(&folder, &scratch, idle)).is_refused());
    }

    /// The row's reading says what a formalise would write; Migrate… opens the launch dialog
    /// with the migration's prompt, and only for an older roadmap outside a screenshot run, which
    /// cans one. The CLI's answers and VS Code are canned: nothing real is asked (rule 5).
    #[test]
    fn the_rows_reading_and_migrate() {
        let scratch = Scratch::new("formalise-migrate");
        let mut board = board(&scratch);
        board.portfolio = crate::portfolio::read_files(board.store());
        board.sessions.canned = Some(std::sync::Arc::new(crate::launch::canned_preflight()));
        board.visual.canned = Some((
            PathBuf::from(r"C:\Tools\VS Code\Code.exe"),
            crw_core::version::Version::new(2, 1, 314),
        ));
        board.catalog_folder = None;
        let harbor = board.portfolio.project("harbor").unwrap();
        assert_eq!(
            harbor.house.writes,
            [
                ("CLAUDE.md".to_owned(), Change::Create),
                ("roadmap/_TEMPLATE.md".to_owned(), Change::Create),
            ]
        );
        assert!(harbor.house.refused.is_empty() && harbor.house.migrates.is_none());
        assert_eq!(
            harbor.house.writes_words(),
            "CLAUDE.md (created) and roadmap/_TEMPLATE.md (created)"
        );
        // A house roadmap has nothing to migrate.
        board.fixtures = false;
        board.open_migration("harbor");
        assert!(board.launch.is_none());
        assert!(
            board
                .notice_text()
                .is_some_and(|text| text.contains("nothing to migrate")),
            "{:?}",
            board.notice_text()
        );
        // A screenshot run cans a work-packet form.
        board.fixtures = true;
        board.open_migration("harbor");
        let dialog = board.launch.as_ref().expect("the launch dialog");
        assert_eq!((dialog.ep, dialog.migrates), (None, Some(Profile::Packet)));
        assert!(
            dialog
                .prompt
                .starts_with("Migrate the roadmap of harbor to the house style"),
            "{}",
            dialog.prompt
        );
        assert!(dialog.prompt.contains(r"C:\Tools\Roadmap Wrangler\crw.exe"));
        assert!(dialog.plan.is_none() && !dialog.visual());
        assert_eq!(dialog.label(), crate::launch::MIGRATION);
        // The formalise preview's Migrate… gives way to the launch dialog.
        board.launch = None;
        board.open_formalise("harbor");
        board
            .actions
            .push(Action::OpenMigration("harbor".to_owned()));
        let mut tree = crate::layout::default_layout();
        board.run_actions(&mut tree);
        assert!(board.formalise.is_none() && board.launch.is_some());
    }

    #[test]
    fn the_window_opens_a_fixtures_plan_and_refuses_its_write() {
        let scratch = Scratch::new("formalise-board");
        let mut board = board(&scratch);
        board.portfolio = crate::portfolio::read_files(board.store());
        board.fixtures = true;
        board.open_formalise("harbor");
        let dialog = board.formalise.as_ref().expect("the dialog");
        let plan = dialog
            .confirmable()
            .expect("harbor lacks a file or two")
            .clone();
        // harbor keeps its owner tasks and inbox; it has no CLAUDE.md and no template.
        let changes: Vec<(&str, &Change)> = plan
            .steps
            .iter()
            .map(|step| (step.name.as_str(), &step.change))
            .collect();
        assert_eq!(
            changes,
            [
                ("CLAUDE.md", &Change::Create),
                ("roadmap/owner-tasks.md", &Change::Keep),
                ("roadmap/inbox.md", &Change::Keep),
                ("roadmap/_TEMPLATE.md", &Change::Create),
            ]
        );
        board.confirm_formalise();
        assert!(
            board
                .notice_text()
                .is_some_and(|text| text.starts_with("A screenshot run writes nothing")),
            "{:?}",
            board.notice_text()
        );
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("portfolio")
            .join("harbor")
            .join("CLAUDE.md");
        assert!(!fixture.exists(), "the fixture untouched");
        board.close_formalise();
        assert!(board.formalise.is_none());
        board.open_formalise("nowhere");
        assert!(board.formalise.is_none());
        assert!(
            board
                .notice_text()
                .is_some_and(|text| text.contains("nowhere"))
        );
    }
}
