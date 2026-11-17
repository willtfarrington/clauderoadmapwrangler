//! Enable and Disable remote view (D-28; EP-31a), on `crw_core::serve`'s engine. The confirmation
//! is a dialog in the form of the hosting actions' (`act`): the command named whole before it
//! runs, the owner's login shown and written to the registry only at the click, the link it gives
//! and, once on, Copy and Send the link to my phone (ntfy, the push's tap opening the page);
//! Serve's own words shown whole. The plan reads Tailscale's status and Serve's configuration on a
//! worker thread, and again at the owner's word: the command runs only when the plan is the one
//! shown. The window keeps the registry's record (`remote_view`, this machine's tailnet name while
//! remote view is on), read every half minute and after a run, for the tray's entry and the
//! Diagnostics pane's controls. The tray's entry and the pane's buttons open it. A test's runner
//! and a screenshot run's are canned: nothing runs (rule 5).

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use crw_core::actions::{self, Command, End, Exit, Runner};
use crw_core::registry::Store;
use crw_core::serve::{self, Facts, Plan, Ran, Verb};
use eframe::egui::{self, RichText};

use crate::board::{Action, Board};
use crate::server::Listening;
use crate::style;

/// How often the window reads the registry's record.
const READ_EVERY: Duration = Duration::from_secs(30);

/// How often the window looks for the worker's answer while it waits.
const POLL: Duration = Duration::from_millis(250);

/// The dialog's widest, in points, and the share of the window's height its body may take.
const WIDTH: f32 = 680.0;
const BODY_SHARE: f32 = 0.55;

/// What quitting says while Enable or Disable runs.
pub const SERVING: &str = "Enable or Disable remote view is running: quit once it has finished, \
                           in a moment.";

/// A screenshot run's and a test's machine and login, synthetic as the fixtures are (D-3).
pub const CANNED_MACHINE: &str = "box.example.ts.net";
pub const CANNED_LOGIN: &str = "owner@example.com";

/// `tailscale status --json` as a screenshot run's and a test's runner answers it: a running,
/// untagged machine with its user's login and a certificate domain.
pub fn canned_node() -> String {
    format!(
        r#"{{"BackendState":"Running","Self":{{"UserID":7,"DNSName":"{CANNED_MACHINE}.","Online":true}},"User":{{"7":{{"ID":7,"LoginName":"{CANNED_LOGIN}","DisplayName":"Owner"}}}},"CertDomains":["{CANNED_MACHINE}"]}}"#
    )
}

/// `tailscale serve status --json` with the phone's mount as Enable makes it, to the window's
/// `port`.
pub fn canned_mount(port: u16) -> String {
    format!(
        r#"{{"TCP":{{"9443":{{"HTTPS":true}}}},"Web":{{"{CANNED_MACHINE}:9443":{{"Handlers":{{"/m":{{"Proxy":"http://127.0.0.1:{port}/m"}}}}}}}}}}"#
    )
}

/// The runner of a screenshot run and of a test given none: Tailscale's status answers with
/// [`canned_node`], Serve's with the phone's mount when `on`, else none; every other command is
/// refused, so nothing runs.
pub struct Canned {
    pub on: bool,
    pub port: u16,
}

impl Runner for Canned {
    fn run(&self, command: &Command) -> Result<Exit, String> {
        let args: Vec<&str> = command.args.iter().map(String::as_str).collect();
        let printed = |stdout: String| {
            Ok(Exit {
                code: Some(0),
                stdout,
                stderr: String::new(),
            })
        };
        match args.as_slice() {
            ["status", "--json"] => printed(canned_node()),
            ["serve", "status", "--json"] if self.on => printed(canned_mount(self.port)),
            ["serve", "status", "--json"] => printed("{}".to_owned()),
            _ => Err("a screenshot run or a test runs nothing".to_owned()),
        }
    }
}

/// What a plan is read from, and where its run writes.
#[derive(Clone)]
pub struct Asked {
    pub facts: Facts,
    pub runner: Arc<dyn Runner>,
    pub store: Option<Arc<Store>>,
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
    let again = serve::plan(&asked.facts, asked.runner.as_ref());
    if again != *seen {
        return Answer::Changed(again);
    }
    let today = crw_core::files::today();
    match serve::run(
        seen,
        asked.runner.as_ref(),
        asked.store.as_deref(),
        asked.ledger.as_deref(),
        &today,
    ) {
        Some(ran) => Answer::Ran(ran),
        None => Answer::Changed(again),
    }
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

/// Where the dialog stands.
#[derive(Debug)]
pub enum Stage {
    Reading,
    /// The preview; `changed` when it was read again at the owner's word and differed.
    Ready {
        plan: Plan,
        changed: bool,
    },
    /// The command runs; the preview stays in view.
    Running(Plan),
    Done(Ran),
    Failed(String),
}

/// What became of a confirmed Enable or Disable, for the window.
#[derive(Debug)]
pub enum Outcome {
    Ran(Ran),
    /// The worker ended without an answer.
    Lost(String),
    /// Tailscale or Serve changed since the preview, so nothing ran.
    Changed,
}

/// The confirmation of Enable or Disable, and its run.
pub struct Dialog {
    pub verb: Verb,
    pub stage: Stage,
    receiver: Option<Receiver<Answer>>,
}

impl Dialog {
    /// Starts reading the plan `asked` names, on a worker thread.
    pub fn open(asked: Asked, ctx: &egui::Context) -> Self {
        let verb = asked.facts.verb;
        let receiver = spawn("remote view preview", ctx, move || {
            Answer::Planned(serve::plan(&asked.facts, asked.runner.as_ref()))
        });
        let stage = match receiver {
            Some(_) => Stage::Reading,
            None => Stage::Failed("the preview's thread did not start".to_owned()),
        };
        Self {
            verb,
            stage,
            receiver,
        }
    }

    /// The preview of a plan read elsewhere: a screenshot run's, a test's.
    pub fn ready(plan: Plan) -> Self {
        Self {
            verb: plan.verb,
            stage: Stage::Ready {
                plan,
                changed: false,
            },
            receiver: None,
        }
    }

    /// A run's end shown as it would be: a screenshot run's, a test's.
    #[cfg(any(test, feature = "screenshot"))]
    pub fn done(ran: Ran) -> Self {
        Self {
            verb: ran.verb,
            stage: Stage::Done(ran),
            receiver: None,
        }
    }

    /// The command runs: nothing closes the dialog until it has ended.
    pub fn is_running(&self) -> bool {
        matches!(self.stage, Stage::Running(_))
    }

    /// The plan shown, when the owner may run it.
    pub fn confirmable(&self) -> Option<&Plan> {
        match &self.stage {
            Stage::Ready { plan, .. } if !plan.is_refused() => Some(plan),
            _ => None,
        }
    }

    /// The link the dialog names: the plan's, or the run's.
    pub fn url(&self) -> Option<&str> {
        match &self.stage {
            Stage::Ready { plan, .. } | Stage::Running(plan) => plan.url.as_deref(),
            Stage::Done(ran) => ran.url.as_deref(),
            Stage::Reading | Stage::Failed(_) => None,
        }
    }

    /// Runs at the owner's word, on a worker thread: the plan read again from `asked` first.
    pub fn confirm(&mut self, asked: Asked, ctx: &egui::Context) {
        let Some(seen) = self.confirmable().cloned() else {
            return;
        };
        let shown = seen.clone();
        match spawn("remote view", ctx, move || confirm(&asked, &seen)) {
            Some(receiver) => {
                self.receiver = Some(receiver);
                self.stage = Stage::Running(shown);
            }
            None => {
                self.stage =
                    Stage::Failed("the command's thread did not start: nothing ran".to_owned());
            }
        }
    }

    /// Takes in the worker's answer when it has come, and asks for frames while it waits. What
    /// became of a confirmed run, for the window; nothing for a preview read.
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
                    "the command's thread ended without an answer: look at Serve's status in the \
                     Diagnostics pane before trying again"
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
            Answer::Ran(ran) => {
                self.stage = Stage::Done(ran.clone());
                Some(Outcome::Ran(ran))
            }
        }
    }

    /// The dialog's contents at `width`, its body at most `height` tall; the owner's choice.
    pub fn ui(&self, ui: &mut egui::Ui, width: f32, height: f32) -> Option<Action> {
        ui.set_width(width);
        let mut chosen = None;
        let close = |ui: &mut egui::Ui, chosen: &mut Option<Action>| {
            buttons(ui, |ui| {
                if ui.button("Close").clicked() {
                    *chosen = Some(Action::CloseRemoteView);
                }
            });
        };
        match &self.stage {
            Stage::Reading => {
                ui.heading(self.verb.question());
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spinner();
                    ui.label("Reading Tailscale's status and Serve's configuration…");
                });
                ui.add_space(8.0);
                buttons(ui, |ui| {
                    if ui.button("Cancel").clicked() {
                        chosen = Some(Action::CloseRemoteView);
                    }
                });
            }
            Stage::Ready { plan, changed } => {
                ui.heading(self.verb.question());
                ui.add_space(4.0);
                if *changed {
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                "Tailscale or Serve changed since the preview, so nothing ran. \
                                 This is the plan as it stands now.",
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
                    buttons(ui, |ui| {
                        if ui.button("Cancel").clicked() {
                            chosen = Some(Action::CloseRemoteView);
                        }
                        if ui
                            .button(plan.verb.button())
                            .on_hover_text(
                                "Tailscale's status and Serve's configuration are read again \
                                 first: the command runs only if they are as this preview shows \
                                 them",
                            )
                            .clicked()
                        {
                            chosen = Some(Action::ConfirmRemoteView);
                        }
                    });
                }
            }
            Stage::Running(plan) => {
                ui.heading(self.verb.question());
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spinner();
                    ui.label("The plan read again, then the command run…");
                });
                ui.add_space(4.0);
                preview(ui, plan, height);
            }
            Stage::Done(ran) => {
                ui.heading(ran.words());
                ui.add_space(4.0);
                done_lines(ui, ran, height);
                // Only when remote view is on: the registry holds the login the link answers
                // (the review's: an unwritten login would answer it with a refusal).
                if ran.verb == Verb::Enable
                    && matches!(ran.end, End::Done)
                    && ran.unwritten.is_none()
                    && let Some(url) = &ran.url
                {
                    ui.add_space(6.0);
                    chosen = link_row(ui, url).or(chosen);
                }
                ui.add_space(8.0);
                close(ui, &mut chosen);
            }
            Stage::Failed(why) => {
                ui.heading(self.verb.question());
                ui.add_space(4.0);
                ui.add(egui::Label::new(RichText::new(why).color(style::red(ui))).wrap());
                ui.add_space(8.0);
                close(ui, &mut chosen);
            }
        }
        chosen
    }
}

/// The link on a line of its own with Copy and Send the link to my phone; the choice, if any.
fn link_row(ui: &mut egui::Ui, url: &str) -> Option<Action> {
    let mut chosen = None;
    ui.label(RichText::new("The link:").weak());
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.add(egui::Label::new(RichText::new(url).monospace()).wrap());
    });
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Copy")
            .on_hover_text("Puts the link on the clipboard")
            .clicked()
        {
            chosen = Some(Action::CopyRemoteLink);
        }
        if ui
            .button("Send the link to my phone")
            .on_hover_text(
                "Pushes the link through ntfy, when crw setting ntfy_host and ntfy_topic name the \
                 server: a tap on the notification opens the page",
            )
            .clicked()
        {
            chosen = Some(Action::SendRemoteLink);
        }
    });
    chosen
}

/// The plan in a scroll area at most `height` tall: the command, the login and the link, what it
/// does, the warnings and what refuses it.
fn preview(ui: &mut egui::Ui, plan: &Plan, height: f32) {
    egui::ScrollArea::vertical()
        .id_salt("remote-view-preview")
        .max_height(height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            match &plan.command {
                Some(command) => {
                    ui.label(RichText::new("It runs:").weak());
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.add(egui::Label::new(RichText::new(command.line()).monospace()).wrap())
                            .on_hover_text(command.program.display().to_string());
                    });
                }
                None if !plan.is_refused() => {
                    ui.label(RichText::new("It runs no command: it writes the registry.").weak());
                }
                None => {}
            }
            ui.add_space(4.0);
            // The login and the link only where the plan may run (the review's: a refused
            // plan has no Enable to write them).
            if !plan.is_refused() {
                if plan.verb == Verb::Enable
                    && let Some(login) = &plan.login
                {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Your login:").weak());
                        ui.label(RichText::new(login).strong());
                        ui.label(
                            RichText::new(
                                "written to the registry's owner_login when you press Enable",
                            )
                            .weak(),
                        );
                    });
                }
                if let Some(url) = &plan.url {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("The link:").weak());
                        ui.label(RichText::new(url).monospace());
                    });
                }
                ui.add_space(4.0);
            }
            for words in &plan.words {
                ui.add(egui::Label::new(words.as_str()).wrap());
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

/// What a run did: how it ended, Serve's own words, and whether the registry and the ledger took
/// their writes.
fn done_lines(ui: &mut egui::Ui, ran: &Ran, height: f32) {
    if let End::Failed(why) = &ran.end {
        ui.add(egui::Label::new(RichText::new(why).color(style::red(ui))).wrap());
        ui.label(
            RichText::new(
                "Nothing is retried, and the registry is left as it was: look, then try again.",
            )
            .weak(),
        );
    }
    if let Some(command) = &ran.command {
        ui.add(egui::Label::new(RichText::new(command).monospace().small().weak()).wrap());
    }
    if !ran.printed.is_empty() {
        egui::ScrollArea::vertical()
            .id_salt("remote-view-printed")
            .max_height(height * 0.5)
            .show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(&ran.printed).monospace().small()).wrap());
            });
    }
    for why in [&ran.unwritten, &ran.unlogged].into_iter().flatten() {
        ui.add(egui::Label::new(RichText::new(why).color(style::amber(ui))).wrap());
    }
}

/// The dialog's buttons on one line at its right, the first added rightmost.
fn buttons(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add);
    });
}

/// Remote view as the window holds it: the dialog while it is open, the registry's record as
/// last read, and a test's runner.
#[derive(Default)]
pub struct RemoteView {
    /// The confirmation of Enable or Disable, while it is open.
    pub dialog: Option<Dialog>,
    /// The registry's `remote_view` as last read: this machine's tailnet name while remote view
    /// is on.
    pub machine: Option<String>,
    read_at: Option<Instant>,
    /// The record changed: read it again at the next pass.
    stale: bool,
    /// The runner when not the real one: a test's.
    pub runner: Option<Arc<dyn Runner>>,
}

impl RemoteView {
    /// Remote view is on, as the registry's record last read says.
    pub fn is_on(&self) -> bool {
        self.machine.is_some()
    }

    /// What the tray's entry and the pane offer: Disable while on, else Enable.
    pub fn verb_offered(&self) -> Verb {
        if self.is_on() {
            Verb::Disable
        } else {
            Verb::Enable
        }
    }

    /// The link Copy and Send take: the open dialog's, else the record's.
    pub fn url(&self) -> Option<String> {
        self.dialog
            .as_ref()
            .and_then(Dialog::url)
            .map(str::to_owned)
            .or_else(|| self.machine.as_deref().map(serve::url))
    }

    /// The dialog's command runs, which quitting waits for.
    pub fn running(&self) -> bool {
        self.dialog.as_ref().is_some_and(Dialog::is_running)
    }
}

impl Board {
    /// Reads the registry's record when it changed or every half minute, hidden too: the tray's
    /// entry and the pane's buttons follow it.
    pub fn read_remote_view(&mut self) {
        let due = self.remote.stale
            || self
                .remote
                .read_at
                .is_none_or(|at| at.elapsed() >= READ_EVERY);
        if !due {
            return;
        }
        self.remote.read_at = Some(Instant::now());
        self.remote.stale = false;
        if let Ok(loaded) = self.store().load() {
            self.remote.machine = loaded.registry.settings.remote_view().map(str::to_owned);
        }
    }

    /// Whether Enable or Disable runs, which quitting waits for (EP-31a).
    pub fn serving(&self) -> bool {
        self.remote.running()
    }

    /// The runner Enable and Disable use: a test's, a screenshot run's and a test's canned one,
    /// else the real one.
    fn remote_runner(&self, port: u16) -> Arc<dyn Runner> {
        if let Some(runner) = &self.remote.runner {
            return runner.clone();
        }
        if self.fixtures || cfg!(test) {
            return Arc::new(Canned {
                on: self.remote.is_on(),
                port,
            });
        }
        Arc::new(actions::Real)
    }

    /// What a plan of `verb` is read from, as the window stands now: its port and whether its
    /// server listens there (a screenshot run's listens by its word), the registry's login and
    /// record, the window's own refusals.
    pub(crate) fn remote_asked(&self, verb: Verb) -> Result<Asked, String> {
        let loaded = self
            .store()
            .load()
            .map_err(|e| format!("the registry could not be read: {e}"))?;
        let settings = &loaded.registry.settings;
        let (port, listening) = match &self.watch.listening {
            Some(Listening::On(port)) => (*port, true),
            _ => (loaded.registry.port(), self.fixtures),
        };
        Ok(Asked {
            facts: Facts {
                verb,
                port,
                listening,
                login: settings.owner_login.clone(),
                machine: settings.remote_view().map(str::to_owned),
                refusals: Vec::new(),
                program: serve::program(),
            },
            runner: self.remote_runner(port),
            store: Some(self.loader.shared_store()),
            ledger: self.ledger().map(PathBuf::from),
        })
    }

    /// Opens the confirmation of `verb`, its plan read on a worker thread; a screenshot run reads
    /// it at once with its canned runner and runs nothing.
    pub fn open_remote_view(&mut self, verb: Verb) {
        if self.remote.dialog.is_some() {
            return;
        }
        match self.remote_asked(verb) {
            Ok(asked) if self.fixtures => {
                let plan = serve::plan(&asked.facts, asked.runner.as_ref());
                self.remote.dialog = Some(Dialog::ready(plan));
            }
            Ok(asked) => self.remote.dialog = Some(Dialog::open(asked, self.ctx())),
            Err(why) => self.warn(why),
        }
    }

    /// The dialog's Enable or Disable: the plan read again with the window as it stands now, and
    /// run if it is the one shown; a screenshot run runs nothing.
    pub fn confirm_remote_view(&mut self) {
        if self.fixtures {
            self.warn("A screenshot run runs nothing: Serve and the registry are left alone.");
            return;
        }
        let Some(verb) = self
            .remote
            .dialog
            .as_ref()
            .and_then(Dialog::confirmable)
            .map(|plan| plan.verb)
        else {
            return;
        };
        match self.remote_asked(verb) {
            Ok(asked) => {
                let ctx = self.ctx().clone();
                if let Some(dialog) = &mut self.remote.dialog {
                    dialog.confirm(asked, &ctx);
                }
            }
            Err(why) => self.fail(format!("nothing ran: {why}")),
        }
    }

    /// The dialog put away, unless its command runs.
    pub fn close_remote_view(&mut self) {
        if !self.serving() {
            self.remote.dialog = None;
        }
    }

    /// Takes in the worker's answer: after a run, the record is read again and the status bar
    /// says how it ended.
    pub fn receive_remote_view(&mut self, ctx: &egui::Context) {
        let Some(outcome) = self
            .remote
            .dialog
            .as_mut()
            .and_then(|dialog| dialog.receive(ctx))
        else {
            return;
        };
        match outcome {
            Outcome::Changed => {}
            Outcome::Lost(why) => self.fail(why),
            Outcome::Ran(ran) => {
                self.remote.stale = true;
                // The doctor's "remote view" line read again, so the pane and the phone's
                // doctor's page agree with the buttons (the review's); only where the window
                // gathers a report.
                if self.diagnostics.is_live() {
                    self.check_diagnostics();
                }
                let mut text = ran.words();
                if let Some(why) = &ran.unlogged {
                    text.push_str(&format!(" ({why})"));
                }
                match &ran.end {
                    End::Done if ran.unwritten.is_none() && ran.unlogged.is_none() => {
                        self.notify(text);
                    }
                    End::Done | End::Warned(_) => self.warn(text),
                    End::Failed(_) => self.fail(text),
                }
            }
        }
    }

    /// Copy: the link on the clipboard, or why there is none.
    pub fn copy_remote_link(&mut self) {
        match self.remote.url() {
            Some(url) => {
                self.ctx().copy_text(url);
                self.notify("The remote view's link is on the clipboard.");
            }
            None => self.warn("Remote view is off: there is no link to copy."),
        }
    }

    /// Send the link to my phone: one push through ntfy, whose tap opens the page.
    pub fn send_remote_link(&mut self) {
        let Some(url) = self.remote.url() else {
            self.warn("Remote view is off: there is no link to send.");
            return;
        };
        let channels = self.notify_settings();
        match self.toasts.send_link(&url, channels.ntfy.as_ref()) {
            Ok(()) => self.notify(
                "The link goes to your phone through ntfy: a tap on the notification opens the \
                 page. The status bar's hover says how the push went.",
            ),
            Err(why) => self.warn(why),
        }
    }

    /// The dialog over the window, while it is open. Escape and Cancel close it, a click beside it
    /// does not, and nothing does while its command runs.
    pub fn remote_view_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.remote.dialog.as_ref() else {
            return;
        };
        let rect = ctx.content_rect();
        let width = WIDTH.min(rect.width() - 80.0);
        let height = (rect.height() * BODY_SHARE).max(160.0);
        let modal = egui::Modal::new(egui::Id::unique("remote-view-dialog"))
            .show(ctx, |ui| dialog.ui(ui, width, height));
        let closable = !dialog.is_running() && !modal.backdrop_response.clicked();
        match modal.inner {
            Some(action) => self.actions.push(action),
            None if closable && modal.should_close() => {
                self.actions.push(Action::CloseRemoteView);
            }
            None => {}
        }
    }

    /// The Diagnostics pane's remote view buttons, under the Network group's lines: Enable or
    /// Disable, and while on Copy and Send the link to my phone; the choice, if any.
    pub fn remote_view_buttons(&self, ui: &mut egui::Ui) -> Option<Action> {
        let mut chosen = None;
        ui.horizontal_wrapped(|ui| {
            let verb = self.remote.verb_offered();
            let hover = match verb {
                Verb::Enable => {
                    "Preview mounting the phone's pages on your tailnet with Tailscale Serve: the \
                     command, your login and the link, then Enable or Cancel"
                }
                Verb::Disable => {
                    "Preview taking the phone's mount off Serve: the command, then Disable or \
                     Cancel"
                }
            };
            if ui.button(verb.label()).on_hover_text(hover).clicked() {
                chosen = Some(Action::OpenRemoteView(verb));
            }
            if self.remote.is_on() {
                if ui
                    .button("Copy the link")
                    .on_hover_text("Puts the remote view's link on the clipboard")
                    .clicked()
                {
                    chosen = Some(Action::CopyRemoteLink);
                }
                if ui
                    .button("Send the link to my phone")
                    .on_hover_text(
                        "Pushes the link through ntfy: a tap on the notification opens the page",
                    )
                    .clicked()
                {
                    chosen = Some(Action::SendRemoteLink);
                }
            }
        });
        chosen
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crw_core::ledger::Ledger;

    use super::*;
    use crate::board::{Places, View as BoardView};
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::testing;

    /// A runner that answers the two reads as the canned one does and keeps every command it
    /// was asked, answering the change with `change`.
    struct Recording {
        /// Whether Serve holds the phone's mount: a change that ends well moves it, as the real
        /// one would, so the run's read again finds it (EP-31a's review).
        on: Mutex<bool>,
        asked: Mutex<Vec<String>>,
        change: Exit,
    }

    impl Recording {
        fn new(on: bool, change: Exit) -> Arc<Self> {
            Arc::new(Self {
                on: Mutex::new(on),
                asked: Mutex::new(Vec::new()),
                change,
            })
        }

        fn asked(&self) -> Vec<String> {
            self.asked.lock().unwrap().clone()
        }

        /// Serve changed behind the dialog's back.
        fn set_on(&self, on: bool) {
            *self.on.lock().unwrap() = on;
        }
    }

    impl Runner for Recording {
        fn run(&self, command: &Command) -> Result<Exit, String> {
            self.asked.lock().unwrap().push(command.line());
            let canned = Canned {
                on: *self.on.lock().unwrap(),
                port: 47831,
            };
            match canned.run(command) {
                Ok(exit) => Ok(exit),
                Err(_) => {
                    if self.change.code == Some(0) {
                        self.set_on(!command.args.iter().any(|arg| arg == "off"));
                    }
                    Ok(self.change.clone())
                }
            }
        }
    }

    fn done() -> Exit {
        Exit {
            code: Some(0),
            stdout: format!("Available within your tailnet:\n\nhttps://{CANNED_MACHINE}:9443/m\n"),
            stderr: String::new(),
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
        let mut board = Board::new(
            places,
            &ctx,
            BoardView::default(),
            egui::ThemePreference::Light,
        );
        // As the real window, whose server listens (a test's board starts none).
        board.watch.listening = Some(Listening::On(47831));
        board
    }

    /// Waits for the dialog's worker, passing the window's receive as each frame does.
    fn settle(board: &mut Board) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while board
            .remote
            .dialog
            .as_ref()
            .is_some_and(|dialog| matches!(dialog.stage, Stage::Reading | Stage::Running(_)))
        {
            assert!(Instant::now() < deadline, "the worker never answered");
            let ctx = board.ctx().clone();
            board.receive_remote_view(&ctx);
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Enable from the preview to the registry: the two reads, then the command at the owner's
    /// word, the login and the machine's name written, the run logged without the name; the
    /// record read again, so the tray and the pane offer Disable.
    #[test]
    fn enable_runs_serve_at_the_owners_word_and_writes_the_registry() {
        let scratch = Scratch::new("remote-view-enable");
        let mut board = board(&scratch);
        let runner = Recording::new(false, done());
        board.remote.runner = Some(runner.clone());
        board.read_remote_view();
        assert!(!board.remote.is_on());
        assert_eq!(board.remote.verb_offered(), Verb::Enable);
        board.open_remote_view(Verb::Enable);
        settle(&mut board);
        let plan = board
            .remote
            .dialog
            .as_ref()
            .and_then(Dialog::confirmable)
            .cloned()
            .expect("a plan to confirm");
        assert_eq!(
            plan.command.as_ref().map(Command::line).as_deref(),
            Some("tailscale serve --bg --https=9443 --set-path=/m http://127.0.0.1:47831/m")
        );
        assert_eq!(plan.login.as_deref(), Some(CANNED_LOGIN));
        assert_eq!(
            plan.url.as_deref(),
            Some("https://box.example.ts.net:9443/m/")
        );
        // Nothing was run but the two reads, and nothing written, before the owner's word.
        assert_eq!(
            runner.asked(),
            ["tailscale status --json", "tailscale serve status --json"]
        );
        assert_eq!(
            board.store().load().unwrap().registry.settings.owner_login,
            None
        );
        board.run(Action::ConfirmRemoteView);
        settle(&mut board);
        let asked = runner.asked();
        assert_eq!(
            &asked[asked.len() - 2..],
            [
                "tailscale serve --bg --https=9443 --set-path=/m http://127.0.0.1:47831/m",
                "tailscale serve status --json"
            ],
            "the change, then Serve read again: {asked:#?}"
        );
        let settings = board.store().load().unwrap().registry.settings;
        assert_eq!(settings.owner_login.as_deref(), Some(CANNED_LOGIN));
        assert_eq!(settings.remote_view(), Some(CANNED_MACHINE));
        let Some(Stage::Done(ran)) = board.remote.dialog.as_ref().map(|dialog| &dialog.stage)
        else {
            panic!("the run's end is shown");
        };
        assert_eq!(ran.end, End::Done);
        assert!(ran.printed.contains("Available within your tailnet"));
        let rows = Ledger::open(&scratch.0.join("ledger.sqlite"))
            .unwrap()
            .events(crw_core::remote::EVENT_KIND, None)
            .unwrap();
        assert_eq!(rows.len(), 1);
        let detail = rows[0].detail.clone().unwrap_or_default();
        assert!(detail.starts_with("enable: "), "{detail}");
        assert!(!detail.contains("box.example"), "{detail}");
        // The record is read again at the next pass: the tray and the pane now offer Disable.
        board.read_remote_view();
        assert!(board.remote.is_on());
        assert_eq!(board.remote.verb_offered(), Verb::Disable);
        assert_eq!(
            board.remote.url().as_deref(),
            Some("https://box.example.ts.net:9443/m/")
        );
        board.run(Action::CloseRemoteView);
        assert!(board.remote.dialog.is_none());
    }

    /// The plan is read again at the owner's word: Serve changed since the preview, so nothing
    /// runs, the dialog shows the plan as it stands now, and the registry is untouched (the
    /// review's: the "runs only the plan shown" property).
    #[test]
    fn a_plan_changed_since_the_preview_runs_nothing() {
        let scratch = Scratch::new("remote-view-changed");
        let mut board = board(&scratch);
        let runner = Recording::new(false, done());
        board.remote.runner = Some(runner.clone());
        board.open_remote_view(Verb::Enable);
        settle(&mut board);
        assert!(
            board
                .remote
                .dialog
                .as_ref()
                .and_then(Dialog::confirmable)
                .is_some()
        );
        // The phone's path mounted by hand meanwhile.
        runner.set_on(true);
        board.run(Action::ConfirmRemoteView);
        settle(&mut board);
        let Some(Stage::Ready { plan, changed }) =
            board.remote.dialog.as_ref().map(|dialog| &dialog.stage)
        else {
            panic!("the plan as it stands now is shown");
        };
        assert!(*changed);
        assert!(
            plan.command.is_none(),
            "the mount is there now: the plan writes the registry alone"
        );
        assert!(
            !runner
                .asked()
                .iter()
                .any(|line| line.starts_with("tailscale serve --bg")),
            "nothing ran: {:?}",
            runner.asked()
        );
        assert_eq!(
            board.store().load().unwrap().registry.settings.owner_login,
            None
        );
    }

    /// A refusal by Serve is shown in its own words and changes no registry setting.
    #[test]
    fn a_failed_enable_writes_nothing_and_says_serves_words() {
        let scratch = Scratch::new("remote-view-failed");
        let mut board = board(&scratch);
        let runner = Recording::new(
            false,
            Exit {
                code: Some(1),
                stdout: String::new(),
                stderr: "error: port 9443 is already in use".to_owned(),
            },
        );
        board.remote.runner = Some(runner);
        board.open_remote_view(Verb::Enable);
        settle(&mut board);
        board.run(Action::ConfirmRemoteView);
        settle(&mut board);
        let Some(Stage::Done(ran)) = board.remote.dialog.as_ref().map(|dialog| &dialog.stage)
        else {
            panic!("the run's end is shown");
        };
        match &ran.end {
            End::Failed(why) => assert!(why.contains("already in use"), "{why}"),
            other => panic!("{other:?}"),
        }
        let settings = board.store().load().unwrap().registry.settings;
        assert_eq!(settings.owner_login, None);
        assert_eq!(settings.remote_view(), None);
    }

    /// The dialog's buttons: Enable and Cancel on a plan, Close alone on a refusal; the done
    /// stage's link with Copy and Send.
    #[test]
    fn the_dialog_offers_what_the_plan_allows() {
        let runner = Canned {
            on: false,
            port: 47831,
        };
        let facts = |listening| Facts {
            verb: Verb::Enable,
            port: 47831,
            listening,
            login: None,
            machine: None,
            refusals: Vec::new(),
            program: PathBuf::from("tailscale"),
        };
        let dialog = Dialog::ready(serve::plan(&facts(true), &runner));
        assert_eq!(press(&dialog, "Enable"), Some(Action::ConfirmRemoteView));
        assert_eq!(press(&dialog, "Cancel"), Some(Action::CloseRemoteView));
        assert!(shows(&dialog, CANNED_LOGIN));
        assert!(shows(&dialog, "https://box.example.ts.net:9443/m/"));
        let refused = Dialog::ready(serve::plan(&facts(false), &runner));
        assert!(refused.confirmable().is_none());
        assert_eq!(press(&refused, "Close"), Some(Action::CloseRemoteView));
        let enabled = Dialog::done(Ran {
            verb: Verb::Enable,
            command: Some("tailscale serve --bg --https=9443 --set-path=/m x".to_owned()),
            end: End::Done,
            printed: String::new(),
            unwritten: None,
            unlogged: None,
            url: Some("https://box.example.ts.net:9443/m/".to_owned()),
        });
        assert_eq!(press(&enabled, "Copy"), Some(Action::CopyRemoteLink));
        assert_eq!(
            press(&enabled, "Send the link to my phone"),
            Some(Action::SendRemoteLink)
        );
    }

    /// The tray's order shows the window and opens the confirmation the window offers.
    #[test]
    fn the_trays_entry_opens_the_offered_confirmation() {
        let scratch = Scratch::new("remote-view-tray");
        let mut board = board(&scratch);
        board.remote.runner = Some(Recording::new(false, done()));
        board.hidden = true;
        board.obey(crate::tray::Order::RemoteView);
        assert!(!board.hidden, "shown first");
        assert_eq!(
            board.actions.last(),
            Some(&Action::OpenRemoteView(Verb::Enable))
        );
        board.remote.machine = Some(CANNED_MACHINE.to_owned());
        board.obey(crate::tray::Order::RemoteView);
        assert_eq!(
            board.actions.last(),
            Some(&Action::OpenRemoteView(Verb::Disable))
        );
    }

    /// The dialog refuses the window's restart while it is open, and the keys stay its own.
    #[test]
    fn an_open_dialog_refuses_the_restart() {
        let scratch = Scratch::new("remote-view-restart");
        let mut board = board(&scratch);
        board.remote.runner = Some(Recording::new(false, done()));
        board.open_remote_view(Verb::Enable);
        settle(&mut board);
        assert!(
            board
                .restart_refusals()
                .iter()
                .any(|why| why.contains("Enable or Disable remote view is open")),
            "{:?}",
            board.restart_refusals()
        );
    }

    /// The pane's buttons: Enable while off; Disable, Copy and Send while on.
    #[test]
    fn the_panes_buttons_follow_the_record() {
        let scratch = Scratch::new("remote-view-pane");
        let mut board = board(&scratch);
        let mut screen = testing::Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            board.remote_view_buttons(ui);
        });
        assert!(testing::text_containing(&output, "Enable remote view").is_some());
        assert!(testing::text_containing(&output, "Copy the link").is_none());
        board.remote.machine = Some(CANNED_MACHINE.to_owned());
        let output = screen.pass(Vec::new(), |ui| {
            board.remote_view_buttons(ui);
        });
        for words in [
            "Disable remote view",
            "Copy the link",
            "Send the link to my phone",
        ] {
            assert!(
                testing::text_containing(&output, words).is_some(),
                "{words}"
            );
        }
    }

    /// The text of `dialog` drawn once, as one string per shape.
    fn shows(dialog: &Dialog, words: &str) -> bool {
        let mut screen = testing::Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            dialog.ui(ui, 600.0, 400.0);
        });
        testing::text_containing(&output, words).is_some()
    }

    /// What pressing the button labelled `label` chooses: the first pass lays the dialog out,
    /// the button is found in the second (EP-21a's trap), then moved over, pressed and released.
    fn press(dialog: &Dialog, label: &str) -> Option<Action> {
        let mut screen = testing::Screen::new();
        let mut chosen = None;
        screen.pass(Vec::new(), |ui| chosen = dialog.ui(ui, 600.0, 400.0));
        let output = screen.pass(Vec::new(), |ui| chosen = dialog.ui(ui, 600.0, 400.0));
        let at = testing::text_at(&output, label).unwrap_or_else(|| panic!("no {label}"));
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
}
