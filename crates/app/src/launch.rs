//! The launch dialog (D-16, D-17, D-18, D-22): what a session of a project's next brief starts
//! with, put together by the rule table and the prompt generator (`crw_core::prompt::draft`),
//! shown for the owner to change, and handed on: to the runner (EP-9) as a headless session, or
//! to VS Code as a visual one, its prompt filled into a Claude Code tab for the owner to send
//! (EP-11). The prompt follows the choices, the launch mode among them, until the owner edits it;
//! the models come from Claude Code's catalog, the effort levels, ultracode and the permission
//! modes from what the CLI's help lists (D-15), and each choice shows the reason the rules gave
//! it, the Fable downshift's among them (D-23; EP-18). A visual launch hands VS Code the prompt
//! alone, so its model, effort and ultracode pickers are greyed out: the new tab's panel chooses,
//! and the opener names the rules' suggestion. A headless launch says how its questions reach the
//! owner: the route and the project's time limit (D-19; EP-14). For a project whose autopilot is
//! on, the **chain plan** (D-22; EP-19a) lists under the columns the briefs a chain may run after
//! this one, each with the same pickers, the rules' resolution the default and a Chain box; the
//! window keeps it at Start headless for the chain to follow. Beside the Permissions picker a
//! **Keep** menu (D-18; EP-19b, the owner's choice) writes the mode shown into the project's
//! `[project.launch]` at once, or takes the project's own mode out. Beside the Model and Effort
//! pickers the same pin (D-22; EP-19c) keeps the value shown in the project's rule for every
//! brief or for the brief's phase, or takes it back to the rules; the dialog's brief and its
//! chain plan are then resolved again from the registry.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crw_core::autopilot::{Plan, Planned};
use crw_core::catalog::Catalog;
use crw_core::prompt::{self, Chosen, Draft, Parts};
use crw_core::registry::{LaunchMode, Project};
use crw_core::roadmap::{EpId, Size};
use crw_core::rules::{self, Choice, Kept, Key, Picked, Resolution, Scope, Source};
use crw_core::session::{self, Preflight, Route};
use crw_core::visual::{self, Readiness};
use eframe::egui::{self, RichText};

use crate::board::Action;
use crate::style::{self, icons};

/// The permission modes offered when the CLI's help does not list them (D-18).
const MODES: [&str; 5] = [
    "acceptEdits",
    "auto",
    "plan",
    "dontAsk",
    "bypassPermissions",
];

/// The dialog's widest, its column of choices, and the gap between the columns, in points.
const WIDTH: f32 = 1180.0;
const CHOICES: f32 = 390.0;
const GAP: f32 = 16.0;
/// The width of a choice's name.
const NAME: f32 = 84.0;
/// The chain plan's pickers, in points: the model's and the effort's.
const PLAN_MODEL: f32 = 180.0;
const PLAN_EFFORT: f32 = 110.0;
/// The room the Keep menu's pin takes beside a picker, in points (EP-19b).
const KEEP: f32 = 34.0;
/// How often an open dialog in Visual looks again at VS Code's windows.
const READINESS_EVERY: Duration = Duration::from_secs(2);

/// The dialog of one launch, open over the window.
pub struct Dialog {
    pub project: String,
    pub folder: PathBuf,
    /// The CLI the preflight asks (D-15).
    pub program: PathBuf,
    pub ep: EpId,
    pub size: Option<Size>,
    pub title: String,
    /// The choices, each with its reason: the rule table's, or the owner's here.
    pub resolution: Resolution,
    pub neighbours: Vec<PathBuf>,
    /// The editor's text: what the session is sent.
    pub prompt: String,
    parts: Parts,
    template: Option<String>,
    /// The prompt as last generated; the editor holds edits while it differs.
    generated: String,
    problems: Vec<String>,
    /// What the owner should know first: a task due before this brief, a session in progress.
    pub warnings: Vec<String>,
    catalog: Catalog,
    date: String,
    /// What a visual launch would meet: VS Code, the extension, the own workspace file, the
    /// project's window.
    pub readiness: Readiness,
    /// The registry's `settings.vscode`, for a fresh look at VS Code.
    pub vscode_setting: Option<PathBuf>,
    /// When `readiness` was read; none when it is due again.
    read_at: Option<Instant>,
    /// Why the last start was refused.
    pub error: Option<String>,
    /// How long a headless session's request may wait before the run is deferred (D-19).
    pub answer_timeout: Option<Duration>,
    /// How a headless session's questions reach the owner, and why the defer route when it is.
    pub route: (Route, Option<String>),
    /// The chain plan (EP-19a), for a project whose autopilot is on, as the owner leaves it.
    pub plan: Option<Plan>,
    /// The project's own permission mode, its `[project.launch] permission_mode`, when it has
    /// one (EP-19b): what the Keep menu beside Permissions can take back.
    pub kept_mode: Option<String>,
    /// The brief's phase id, when its heading has one (EP-19c): what Keep for phase Pn keeps
    /// for.
    pub phase: Option<String>,
    /// The CLI's effort levels in order, which a rule's step moves along (none for the built-in
    /// ones): the brief is resolved again with them after a keep (EP-19c).
    pub levels: Vec<String>,
    /// What the project keeps for the model, for every brief and for this brief's phase
    /// (EP-19c): what the Keep menu beside Model can take back.
    pub kept_model: Kept,
    /// The same for the effort, beside Effort.
    pub kept_effort: Kept,
}

/// What a dialog opens on, besides the draft.
pub struct Opening {
    pub folder: PathBuf,
    pub program: PathBuf,
    pub neighbours: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub template: Option<String>,
    pub catalog: Catalog,
    pub date: String,
    pub readiness: Readiness,
    pub vscode_setting: Option<PathBuf>,
    pub answer_timeout: Option<Duration>,
    pub route: (Route, Option<String>),
    pub plan: Option<Plan>,
    pub kept_mode: Option<String>,
    pub kept_model: Kept,
    pub kept_effort: Kept,
    pub levels: Vec<String>,
}

impl Dialog {
    pub fn new(draft: Draft, opening: Opening) -> Self {
        let Draft {
            ep,
            title,
            size,
            phase,
            resolution,
            parts,
            prompt,
        } = draft;
        Self {
            project: parts.project.clone(),
            folder: opening.folder,
            program: opening.program,
            ep,
            size,
            title,
            resolution,
            neighbours: opening.neighbours,
            generated: prompt.text.clone(),
            prompt: prompt.text,
            parts,
            template: opening.template,
            problems: prompt.problems,
            warnings: opening.warnings,
            catalog: opening.catalog,
            date: opening.date,
            readiness: opening.readiness,
            vscode_setting: opening.vscode_setting,
            read_at: Some(Instant::now()),
            error: None,
            answer_timeout: opening.answer_timeout,
            route: opening.route,
            plan: opening.plan,
            kept_mode: opening.kept_mode,
            phase,
            levels: opening.levels,
            kept_model: opening.kept_model,
            kept_effort: opening.kept_effort,
        }
    }

    /// The project's rules as the registry says them after a Keep beside the Model or the
    /// Effort picker (EP-19c): the project as written (`project`), what it keeps then read for
    /// both pickers, and the brief resolved again from it (`fresh`). For `key`, after Back to the
    /// rules (`asked` none) the picker takes the rules' value; after a keep it takes the new
    /// resolution when its value is the one kept, so that the reason names the project's rule,
    /// and keeps the value shown when something stands over the kept one for this brief. Every
    /// other value the rules gave takes its new reason too, since a keep may number the project's
    /// rules afresh (EP-19c's review); the owner's own choices stay. The plan made again (`plan`)
    /// keeps the rows the owner touched, and the prompt follows unless the owner has edited it.
    pub fn kept_rule(
        &mut self,
        key: Key,
        asked: Option<&str>,
        project: &Project,
        fresh: Resolution,
        plan: Option<Plan>,
    ) {
        let phase = self.phase.as_deref();
        self.kept_model = Kept::of(project, phase, Key::Model);
        self.kept_effort = Kept::of(project, phase, Key::Effort);
        let rules_gave = |source: &Source| *source != Source::Owner;
        let take_model = match key {
            Key::Model => asked.is_none_or(|value| fresh.model.value == value),
            Key::Effort => rules_gave(&self.resolution.model.source),
        };
        let take_effort = match key {
            Key::Effort => asked.is_none_or(|value| fresh.effort.value.as_deref() == Some(value)),
            Key::Model => rules_gave(&self.resolution.effort.source),
        };
        if take_model {
            self.resolution.model = fresh.model;
        }
        if take_effort {
            self.resolution.effort = fresh.effort;
        }
        // The CLI's own limits are applied again at the next frame (`fit`).
        if rules_gave(&self.resolution.ultracode.source) {
            self.resolution.ultracode = fresh.ultracode;
        }
        self.resolution.downshift = fresh.downshift;
        self.resolution.notes = fresh.notes;
        if let Some(mut plan) = plan {
            if let Some(before) = &self.plan {
                plan.keep_choices(before);
            }
            self.plan = Some(plan);
        }
        self.chose();
    }

    /// The action a Keep menu beside the Model or the Effort picker asked for (EP-19c), the value
    /// kept being the one shown when the menu was open (`current`).
    fn rule_action(&self, keep: RuleKeep, key: Key, current: Option<String>) -> Action {
        let (scope, value) = match keep {
            RuleKeep::Keep(scope) => (scope, current),
            RuleKeep::Back(scope) => (scope, None),
        };
        Action::KeepRule {
            project: self.project.clone(),
            scope,
            key,
            value,
        }
    }

    /// The project's permission mode as the registry says it after a Keep (EP-19b): the mode it
    /// keeps, if any, and the dialog's choice with where it comes from.
    pub fn kept_permission_mode(&mut self, kept: Option<String>, choice: Choice<String>) {
        self.kept_mode = kept;
        self.resolution.permission_mode = choice;
        self.chose();
    }

    /// Whether the launch is visual (D-16).
    pub fn visual(&self) -> bool {
        self.resolution.mode.value == LaunchMode::Visual
    }

    /// The launch mode chosen for the owner (a screenshot run's `launch=<project>:headless`),
    /// the prompt following it.
    pub fn set_mode(&mut self, mode: LaunchMode) {
        self.resolution.mode = owners(mode);
        self.read_at = None;
        self.chose();
    }

    /// Whether VS Code's windows are worth a fresh look: in Visual, every few seconds.
    pub fn readiness_due(&self) -> bool {
        self.visual()
            && self
                .read_at
                .is_none_or(|at| at.elapsed() >= READINESS_EVERY)
    }

    pub fn set_readiness(&mut self, readiness: Readiness) {
        self.readiness = readiness;
        self.read_at = Some(Instant::now());
    }

    /// Whether the owner has changed the prompt since it was generated.
    pub fn edited(&self) -> bool {
        self.prompt != self.generated
    }

    /// The effort in words, as the prompt says it.
    pub fn effort_words(&self) -> &str {
        &self.parts.effort
    }

    /// Whether the prompt tells the session to commit: the project's autopilot was on when the
    /// dialog opened (D-22; EP-19).
    pub fn commits(&self) -> bool {
        self.parts.commits
    }

    /// Generates the prompt again for the choices; the editor takes it when `follow` holds.
    fn regenerate(&mut self, follow: bool) {
        self.parts.choose(&Chosen {
            model: &self.resolution.model.value,
            effort: self.resolution.effort.value.as_deref(),
            ultracode: self.resolution.ultracode.value,
            date: &self.date,
            visual: self.visual(),
        });
        let prompt = prompt::generate(&self.parts, self.template.as_deref());
        self.problems = prompt.problems;
        self.generated = prompt.text;
        if follow {
            self.prompt = self.generated.clone();
        }
    }

    /// After a choice changed: the prompt follows it, unless the owner has edited it.
    fn chose(&mut self) {
        let follow = !self.edited();
        self.regenerate(follow);
    }

    /// The launch the runner starts (EP-9), with the ledger it records in.
    pub fn launch(&self, ledger: Option<PathBuf>) -> session::Launch {
        let resolution = &self.resolution;
        let mut launch = session::Launch::new(
            &self.project,
            &self.folder,
            &resolution.model.value,
            &self.prompt,
        );
        launch.ep = Some(self.ep.to_string());
        launch.effort = resolution.effort.value.clone();
        launch.ultracode = resolution.ultracode.value;
        launch.permission_mode = resolution.permission_mode.value.clone();
        launch.neighbours = self.neighbours.clone();
        launch.ledger = ledger;
        launch.answer_timeout = self.answer_timeout;
        launch.route = self.route.0;
        launch
    }

    /// How a headless session's questions reach the owner, in words (D-19).
    pub fn questions_words(&self) -> String {
        let limit = match self.answer_timeout {
            Some(limit) => format!(
                "; a request unanswered for {} defers the run",
                crate::decisions::minutes(limit)
            ),
            None => String::new(),
        };
        match &self.route {
            (Route::Protocol, _) => {
                format!("answered in the Decisions pane while the session waits{limit}")
            }
            (Route::Defer, why) => format!(
                "the defer route ({}): a question ends the run, and the answer resumes it{limit}",
                why.as_deref().unwrap_or("chosen")
            ),
        }
    }

    /// The visual launch (EP-11): `code` opens the project's window, `locks` is where the
    /// extension's lock files show it.
    pub fn visual_launch(
        &self,
        code: &Path,
        locks: &Path,
        ledger: Option<PathBuf>,
    ) -> visual::Launch {
        let mut launch =
            visual::Launch::new(&self.project, &self.folder, &self.prompt, code, locks);
        launch.ep = Some(self.ep.to_string());
        launch.ledger = ledger;
        launch
    }

    /// Brings the choices in line with what this CLI takes: ultracode off when its help does not
    /// list it (D-15), so that the prompt says the effort the session gets.
    pub fn fit(&mut self, preflight: &Preflight) {
        let listed = preflight
            .flags
            .as_ref()
            .is_ok_and(|flags| flags.ultracode());
        if self.resolution.ultracode.value && !listed {
            let ultracode = &mut self.resolution.ultracode;
            ultracode.value = false;
            ultracode.detail = Some("off: this claude does not list --effort ultracode".to_owned());
            self.chose();
        }
    }

    /// The effort levels to offer: the model's that the CLI lists too, in the CLI's order.
    pub fn efforts(&self, preflight: Option<&Preflight>) -> Vec<String> {
        efforts_of(
            &self.catalog,
            &cli_efforts(preflight),
            &self.resolution.model.value,
        )
    }

    /// The permission modes to offer: the CLI's, else the usual ones.
    pub fn modes(preflight: Option<&Preflight>) -> Vec<String> {
        flags_of(preflight)
            .map(|flags| flags.permission_modes.clone())
            .filter(|modes| !modes.is_empty())
            .unwrap_or_else(|| MODES.map(str::to_owned).to_vec())
    }

    /// What is worth saying about the choices, against the catalog and this CLI.
    pub fn checks(&self, preflight: Option<&Preflight>) -> Vec<String> {
        let mut found = Vec::new();
        let resolution = &self.resolution;
        match self.catalog.model(&resolution.model.value) {
            None => found.push(format!(
                "{} is not in Claude Code's model catalog: it is passed as written.",
                resolution.model.value
            )),
            Some(model) => {
                if let (Some(least), Some(Ok(version))) =
                    (model.min_cli, preflight.map(|p| &p.version))
                    && *version < least
                {
                    found.push(format!(
                        "{} needs claude {least} or newer; this one is {version}.",
                        model.name
                    ));
                }
                if let Some(effort) = &resolution.effort.value
                    && !resolution.ultracode.value
                    && !model.efforts.is_empty()
                    && !model.efforts.contains(effort)
                {
                    found.push(format!(
                        "{} does not list the effort {effort} ({}).",
                        model.name,
                        model.efforts.join(", ")
                    ));
                }
            }
        }
        if self.visual() {
            if let Err(why) = visual::link(&self.prompt) {
                found.push(format!("Visual: {why}."));
            }
        } else if resolution.permission_mode.value == "bypassPermissions" {
            found.push(
                "bypassPermissions: every tool runs without asking, the neighbours' deny rules \
                 aside (D-18)."
                    .to_owned(),
            );
        }
        found
    }

    /// The dialog's contents; what the owner asked for, to run after the frame. `starting` is a
    /// headless start on its way, `opening` a visual launch waiting for its window.
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        preflight: Option<&Preflight>,
        asking: bool,
        starting: bool,
        opening: bool,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let screen = ui.ctx().content_rect();
        let width = (screen.width() - 80.0).clamp(600.0, WIDTH);
        let height = (screen.height() - 300.0 - self.plan_height()).clamp(200.0, 620.0);
        ui.set_width(width);
        ui.horizontal(|ui| {
            ui.heading(format!("Launch {} {}", self.project, self.ep));
            if let Some(size) = self.size {
                style::chip(ui, size.as_str(), ui.visuals().weak_text_color());
            }
        });
        ui.add(egui::Label::new(RichText::new(&self.title).weak()).truncate());
        ui.separator();
        let choices = CHOICES.min(width * 0.42);
        let spacing = ui.spacing().item_spacing.x;
        // A separator between the columns would stretch to the whole height a modal may take,
        // so the divider is drawn once both columns have their height.
        let (left, right) = ui
            .horizontal_top(|ui| {
                let left = ui
                    .vertical(|ui| {
                        ui.set_width(width - choices - GAP - 2.0 * spacing);
                        self.editor(ui, height);
                    })
                    .response
                    .rect;
                ui.add_space(GAP);
                let right = ui
                    .vertical(|ui| {
                        ui.set_width(choices);
                        self.choices(ui, preflight, asking, &mut actions);
                    })
                    .response
                    .rect;
                (left, right)
            })
            .inner;
        ui.painter().vline(
            (left.right() + right.left()) / 2.0,
            egui::Rangef::new(left.top(), left.bottom().max(right.bottom())),
            ui.visuals().widgets.noninteractive.bg_stroke,
        );
        self.plan_section(ui, preflight);
        ui.separator();
        self.footer(ui, preflight, starting, opening, &mut actions);
        actions
    }

    /// The room the chain plan takes under the columns, which the editor gives up.
    fn plan_height(&self) -> f32 {
        match &self.plan {
            Some(plan) if !self.visual() => 80.0 + 30.0 * plan.briefs.len().max(1) as f32,
            Some(_) => 40.0,
            None => 0.0,
        }
    }

    /// The chain plan (EP-19a) under the columns, for a project whose autopilot is on. In
    /// Headless, one row per brief a chain may run after this one: its Chain box, its id and
    /// size, its title, and the Model and Effort pickers and the Ultracode box as the choices
    /// above offer them, the rules' resolution over its row the default and the reason on
    /// hover; a row the owner touches keeps their choice as shown. In Visual, one line, since a
    /// chain follows a headless run only (D-22).
    fn plan_section(&mut self, ui: &mut egui::Ui, preflight: Option<&Preflight>) {
        let visual = self.visual();
        let listed = flags_of(preflight).is_some_and(|flags| flags.ultracode());
        let cli = cli_efforts(preflight);
        let catalog = &self.catalog;
        let Some(plan) = self.plan.as_mut() else {
            return;
        };
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Chain plan").strong());
            let note = if visual {
                "a chain follows a headless run only (D-22): switch to Headless to plan one"
                    .to_owned()
            } else {
                format!(
                    "the briefs a chain may run after {}, each as the rules resolve it until \
                     you choose; a row you leave alone takes the rules' choice when the chain \
                     reaches it, the downshift judged then; untick a row and the chain ends \
                     before it",
                    plan.after
                )
            };
            ui.label(RichText::new(note).small().weak());
        });
        if visual {
            return;
        }
        if plan.briefs.is_empty() {
            let why = match &plan.stop {
                Some(stop) => format!("Nothing to chain after {}: {}.", plan.after, stop.why),
                None => format!("No brief follows {} in the roadmap.", plan.after),
            };
            ui.label(RichText::new(why).small().color(style::amber(ui)));
            return;
        }
        let spacing = ui.spacing().item_spacing.x;
        let title_width =
            (ui.available_width() - 76.0 - 40.0 - PLAN_MODEL - PLAN_EFFORT - 120.0 - 7.0 * spacing)
                .max(120.0);
        egui::Grid::new("chain-plan")
            .num_columns(6)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                for (n, planned) in plan.briefs.iter_mut().enumerate() {
                    plan_row(ui, n, planned, catalog, &cli, listed, title_width);
                    ui.end_row();
                }
            });
        for planned in &plan.briefs {
            if !planned.due.is_empty() {
                ui.label(
                    RichText::new(format!(
                        "{}: owner tasks due before it are open ({}); the gate refuses it until \
                         they are ticked.",
                        planned.ep,
                        planned.due.join(", ")
                    ))
                    .small()
                    .color(style::amber(ui)),
                );
            }
        }
        let mut notes = Vec::new();
        if let Some(stop) = &plan.stop {
            notes.push(format!("The chain would end at {}: {}.", stop.ep, stop.why));
        }
        notes.push(format!(
            "At most {} chained run{} a day in the project (its daily); the day's caps and the \
             gate's other conditions still hold when the chain reaches each brief.",
            plan.daily,
            if plan.daily == 1 { "" } else { "s" }
        ));
        ui.add(egui::Label::new(RichText::new(notes.join(" ")).small().weak()).wrap());
    }

    fn editor(&mut self, ui: &mut egui::Ui, height: f32) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Prompt").strong());
            let note = if self.edited() {
                "edited here: the choices no longer rewrite it"
            } else {
                "generated: it follows the choices until you edit it"
            };
            ui.label(RichText::new(note).small().weak());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(self.edited(), egui::Button::new("Regenerate"))
                    .on_hover_text("Drop the edits and generate the prompt for the choices again")
                    .clicked()
                {
                    self.regenerate(true);
                }
                if ui
                    .button(style::icon(icons::ICON_CONTENT_COPY, None))
                    .on_hover_text("Copy the prompt, to paste into a session by hand")
                    .clicked()
                {
                    ui.ctx().copy_text(self.prompt.clone());
                }
            });
        });
        let row = ui.text_style_height(&egui::TextStyle::Monospace);
        egui::ScrollArea::vertical()
            .id_salt("launch-prompt")
            .max_height(height)
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.prompt)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows((height / row).floor().max(4.0) as usize),
                );
            });
        let length = format!("{} characters", self.prompt.chars().count());
        ui.label(RichText::new(length).small().weak());
        for problem in &self.problems {
            ui.label(RichText::new(problem).small().color(style::amber(ui)));
        }
    }

    fn choices(
        &mut self,
        ui: &mut egui::Ui,
        preflight: Option<&Preflight>,
        asking: bool,
        actions: &mut Vec<Action>,
    ) {
        let mut changed = false;
        let combo = ui.available_width() - NAME - 12.0;

        let reason = self.resolution.mode.reason();
        field(ui, "Launch", &reason, |ui| {
            let current = self.resolution.mode.value;
            let mut chosen = current;
            ui.selectable_value(&mut chosen, LaunchMode::Visual, "Visual")
                .on_hover_text(
                    "The VS Code extension: the project's window with the prompt filled into a \
                     Claude Code tab, for you to send (D-16)",
                );
            ui.selectable_value(&mut chosen, LaunchMode::Headless, "Headless")
                .on_hover_text(
                    "claude -p in the project's folder, held by this window, its requests in \
                     the Decisions pane (D-16)",
                );
            if chosen != current {
                self.resolution.mode = owners(chosen);
                // VS Code's windows are looked at again before the next frame.
                self.read_at = None;
                changed = true;
            }
        });
        // A visual launch hands VS Code the prompt alone: the new tab's panel chooses the model
        // and effort, so the pickers serve Headless only (EP-18, the owner's choice).
        let visual = self.visual();

        let reason = self.model_reason();
        field(ui, "Model", &reason, |ui| {
            let current = self.resolution.model.value.clone();
            let mut picked = None;
            let mut keep = None;
            ui.add_enabled_ui(!visual, |ui| {
                picked = model_picker(ui, "launch-model", &self.catalog, &current, combo - KEEP);
                // The Keep menu (EP-19c): the model shown, kept for the project's briefs or for
                // this phase's, or taken back to the rules.
                keep = rule_keep_menu(
                    ui,
                    "keep-model",
                    Some(&current),
                    &self.kept_model,
                    self.phase.as_deref(),
                );
            });
            if let Some(model) = picked {
                self.resolution.model = owners(model);
                changed = true;
            }
            if let Some(keep) = keep {
                actions.push(self.rule_action(keep, Key::Model, Some(current)));
            }
        });

        let efforts = self.efforts(preflight);
        let reason = picker_reason(visual, self.resolution.effort.reason());
        field(ui, "Effort", &reason, |ui| {
            let current = self.resolution.effort.value.clone();
            let mut picked = None;
            let mut keep = None;
            ui.add_enabled_ui(!visual, |ui| {
                picked = effort_picker(
                    ui,
                    "launch-effort",
                    &efforts,
                    current.as_deref(),
                    combo - KEEP,
                );
                keep = rule_keep_menu(
                    ui,
                    "keep-effort",
                    current.as_deref(),
                    &self.kept_effort,
                    self.phase.as_deref(),
                );
            });
            if let Some(effort) = picked {
                self.resolution.effort = owners(effort);
                changed = true;
            }
            if let Some(keep) = keep {
                actions.push(self.rule_action(keep, Key::Effort, current));
            }
        });

        let listed = flags_of(preflight).is_some_and(|flags| flags.ultracode());
        let reason = picker_reason(visual, self.resolution.ultracode.reason());
        field(ui, "Ultracode", &reason, |ui| {
            let mut on = self.resolution.ultracode.value;
            let why = if visual {
                "Headless only: the new tab's panel chooses a visual session's effort"
            } else {
                "This claude's help does not list --effort ultracode: it is passed only when \
                 listed (D-15)"
            };
            ui.add_enabled(
                listed && !visual,
                egui::Checkbox::new(&mut on, "--effort ultracode"),
            )
            .on_disabled_hover_text(why);
            if on != self.resolution.ultracode.value {
                self.resolution.ultracode = owners(on);
                changed = true;
            }
        });

        let modes = Self::modes(preflight);
        let reason = if visual {
            "headless only: VS Code's panel sets a visual session's mode".to_owned()
        } else {
            self.resolution.permission_mode.reason()
        };
        field(ui, "Permissions", &reason, |ui| {
            let current = self.resolution.permission_mode.value.clone();
            let mut chosen = current.clone();
            let mut keep = None;
            ui.add_enabled_ui(!visual, |ui| {
                egui::ComboBox::from_id_salt("launch-mode")
                    .width(combo - KEEP)
                    .selected_text(&current)
                    .show_ui(ui, |ui| {
                        for mode in &modes {
                            ui.selectable_value(&mut chosen, mode.clone(), mode);
                        }
                        if !modes.contains(&current) {
                            ui.selectable_value(&mut chosen, current.clone(), &current);
                        }
                    });
                // The Keep menu (EP-19b): the mode shown, kept for the project's launches and
                // chains, or the project's own taken back.
                keep = keep_menu(
                    ui,
                    &current,
                    self.kept_mode.as_deref(),
                    rules::DEFAULT_PERMISSION_MODE,
                );
            });
            if chosen != current {
                self.resolution.permission_mode = owners(chosen);
                changed = true;
            }
            match keep {
                Some(Keep::Project) => actions.push(Action::KeepPermissionMode(
                    self.project.clone(),
                    Some(current),
                )),
                Some(Keep::Default) => {
                    actions.push(Action::KeepPermissionMode(self.project.clone(), None));
                }
                None => {}
            }
        });

        let reason = if visual {
            "headless only: a visual session reads the folders the user settings grant"
        } else {
            "read, never edited: an Edit deny rule each (D-16, D-18 addenda)"
        };
        field(ui, "Neighbours", reason, |ui| {
            if self.neighbours.is_empty() {
                ui.label(RichText::new("none in the registry").weak());
            } else {
                ui.vertical(|ui| {
                    for folder in &self.neighbours {
                        ui.label(RichText::new(folder.display().to_string()).small());
                    }
                });
            }
        });

        let reason = if visual {
            "headless only: a visual session has the plugin you installed into Claude Code, if any"
        } else {
            "the build's copy, written before the start and loaded with --plugin-dir (D-27)"
        };
        field(ui, "Plugin", reason, |ui| {
            ui.vertical(|ui| {
                match crw_core::plugin::dir() {
                    Some(dir) if !visual => {
                        ui.label(RichText::new(crw_core::plugin::NAME).small())
                            .on_hover_text(dir.display().to_string());
                    }
                    Some(_) => {
                        ui.label(RichText::new("the session's own").weak());
                    }
                    None => {
                        ui.label(
                            RichText::new(format!(
                                "no local folder: the run goes without {}",
                                crw_core::plugin::NAME
                            ))
                            .small()
                            .color(style::amber(ui)),
                        );
                    }
                }
                if !visual
                    && flags_of(preflight).is_some_and(|flags| !flags.has(crw_core::plugin::FLAG))
                {
                    ui.label(
                        RichText::new(format!(
                            "this claude lists no {}: the run goes without it",
                            crw_core::plugin::FLAG
                        ))
                        .small()
                        .color(style::amber(ui)),
                    );
                }
            });
        });

        let reason = if visual {
            "headless only: a visual session asks in VS Code, which the tool never answers"
        } else {
            "the route and the project's time limit (D-19)"
        };
        field(ui, "Questions", reason, |ui| {
            let words = if visual {
                "asked in VS Code".to_owned()
            } else {
                self.questions_words()
            };
            ui.add(egui::Label::new(RichText::new(words).small()).wrap());
        });

        if visual {
            self.vscode(ui);
        } else {
            field(
                ui,
                "Claude",
                "its version, login and flags (D-15)",
                |ui| match preflight {
                    None if asking => {
                        ui.spinner();
                        ui.label(RichText::new("asking…").weak());
                    }
                    None => {
                        ui.label(RichText::new("not asked yet").weak());
                    }
                    Some(preflight) => {
                        let version = match &preflight.version {
                            Ok(version) => format!("claude {version}"),
                            Err(_) => "claude ?".to_owned(),
                        };
                        let login = match &preflight.auth {
                            Ok(auth) if auth.logged_in => "logged in",
                            Ok(_) => "not logged in",
                            Err(_) => "login unknown",
                        };
                        ui.label(format!("{version}, {login}"))
                            .on_hover_text(preflight.program.display().to_string());
                    }
                },
            );
            if let Some(preflight) = preflight {
                for refusal in preflight.refusals() {
                    ui.label(RichText::new(refusal).small().color(style::red(ui)));
                }
            }
        }
        if changed {
            self.chose();
        }
    }

    /// The model's reason, and below it the downshift's judgement when the rules' reason does not
    /// say it already (D-23; EP-18).
    fn model_reason(&self) -> String {
        let model = &self.resolution.model;
        let reason = picker_reason(self.visual(), model.reason());
        match &self.resolution.downshift {
            Some(judged) if model.source != Source::Downshift => {
                let words = if judged.applied {
                    format!(
                        "the downshift proposed {}: {}",
                        rules::DOWNSHIFT_MODEL,
                        judged.words
                    )
                } else {
                    judged.words.clone()
                };
                format!("{reason}\n{words}")
            }
            _ => reason,
        }
    }

    /// What a visual launch meets: VS Code, the extension, the project's own workspace file and
    /// its window.
    fn vscode(&self, ui: &mut egui::Ui) {
        let readiness = &self.readiness;
        let file = readiness
            .workspace
            .as_deref()
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "the workspace file".to_owned());
        field(
            ui,
            "VS Code",
            "the project's own workspace file opens its window (D-16 addendum)",
            |ui| {
                ui.vertical(|ui| {
                    match &readiness.code {
                        Some(code) => {
                            ui.label("found").on_hover_text(code.display().to_string());
                        }
                        None => {
                            ui.label(
                                RichText::new("not found: crw setting vscode <its Code.exe>")
                                    .color(style::red(ui)),
                            );
                        }
                    }
                    match readiness.extension {
                        Some(version) => ui.label(format!("Claude Code extension {version}")),
                        None => ui.label(
                            RichText::new("no Claude Code extension listed")
                                .color(style::amber(ui)),
                        ),
                    };
                    let workspace = if readiness.workspace_exists {
                        format!("{file} beside the folder")
                    } else {
                        format!("{file} missing: written in the owner's form")
                    };
                    ui.label(RichText::new(workspace).small());
                    let window = if readiness.window_open {
                        "its window is open: it comes to the front"
                    } else {
                        "no window has it open: one opens"
                    };
                    ui.label(RichText::new(window).small());
                });
            },
        );
    }

    fn footer(
        &mut self,
        ui: &mut egui::Ui,
        preflight: Option<&Preflight>,
        starting: bool,
        opening: bool,
        actions: &mut Vec<Action>,
    ) {
        let notes: Vec<String> = self
            .warnings
            .iter()
            .chain(&self.resolution.notes)
            .cloned()
            .chain(self.checks(preflight))
            .collect();
        for note in &notes {
            ui.label(RichText::new(note).color(style::amber(ui)));
        }
        if let Some(error) = &self.error {
            ui.label(RichText::new(error).color(style::red(ui)));
        }
        let visual = self.visual();
        let note = if visual {
            "Opens the project's own workspace file in VS Code, which brings its window to the \
             front or opens it, then fills the prompt into a new Claude Code tab there: you send \
             it with Enter, after choosing the model and effort in its panel. The ledger records \
             the launch as visual. The first time, VS Code may ask whether Claude Code may open \
             the link."
                .to_owned()
        } else {
            format!(
                "Runs headless in {}; the ledger records it. Closing the window keeps it in the \
                 tray, and quitting stops it.",
                self.folder.display()
            )
        };
        ui.add(egui::Label::new(RichText::new(note).small().weak()).wrap());
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // A visual launch that waits for its window may be called off; a headless start
                // on its way may not.
                let cancel = ui
                    .add_enabled(!starting, egui::Button::new("Cancel"))
                    .on_hover_text(if opening {
                        "Stop waiting for VS Code: nothing is handed over"
                    } else {
                        "Close the dialog"
                    });
                if cancel.clicked() {
                    actions.push(Action::CloseLaunch);
                }
                let (label, blocked, hover, action) = if visual {
                    (
                        "Open in VS Code",
                        self.visual_blocked(starting, opening),
                        "Open the project's window and fill the prompt into a Claude Code tab \
                         there: you send it (D-16)",
                        Action::StartVisual,
                    )
                } else {
                    (
                        "Start headless",
                        self.headless_blocked(preflight, starting, opening),
                        "Start the session: stream-json in and out, its requests waiting for the \
                         Decisions pane (EP-14)",
                        Action::StartHeadless,
                    )
                };
                let button = egui::Button::new(RichText::new(label).strong());
                let response = ui.add_enabled(blocked.is_none(), button);
                let response = match &blocked {
                    Some(why) => response.on_disabled_hover_text(why),
                    None => response.on_hover_text(hover),
                };
                if response.clicked() {
                    actions.push(action);
                }
                if starting || opening {
                    ui.spinner();
                }
                if opening {
                    ui.label(RichText::new("waiting for the project's window in VS Code…").weak());
                }
            });
        });
    }

    /// Why Start headless cannot be pressed now, if it cannot.
    fn headless_blocked(
        &self,
        preflight: Option<&Preflight>,
        starting: bool,
        opening: bool,
    ) -> Option<String> {
        let refusals = preflight.map(Preflight::refusals).unwrap_or_default();
        if starting {
            Some(
                "Starting: the runner waits its turn and two seconds after the last start"
                    .to_owned(),
            )
        } else if opening {
            Some("A visual launch is waiting for VS Code".to_owned())
        } else if preflight.is_none() {
            Some("Waiting for claude's answers: its version, login and flags".to_owned())
        } else if !refusals.is_empty() {
            Some(refusals.join("; "))
        } else if self.prompt.trim().is_empty() {
            Some("The prompt is empty".to_owned())
        } else {
            None
        }
    }

    /// Why Open in VS Code cannot be pressed now, if it cannot.
    fn visual_blocked(&self, starting: bool, opening: bool) -> Option<String> {
        if opening {
            Some("Waiting for the project's window to show the Claude Code extension".to_owned())
        } else if starting {
            Some("A headless session is starting".to_owned())
        } else if self.readiness.code.is_none() {
            Some("VS Code was not found: set its Code.exe with crw setting vscode".to_owned())
        } else if self.prompt.trim().is_empty() {
            Some("The prompt is empty".to_owned())
        } else {
            visual::link(&self.prompt).err()
        }
    }
}

/// A picker's reason: in Visual it serves Headless only, since the new tab's panel chooses the
/// model and effort and the link carries the prompt alone; the rules' reason stays, for the
/// suggestion the opener names (EP-18).
fn picker_reason(visual: bool, reason: String) -> String {
    if !visual {
        return reason;
    }
    let by = if reason == Source::Owner.to_string() {
        "your choice in Headless".to_owned()
    } else {
        reason
    };
    format!("headless only: choose it in the new tab's panel; suggested by {by}")
}

/// What the Keep menu beside a picker asked for (EP-19b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Keep {
    /// The value shown, kept for the project's launches and chains.
    Project,
    /// The project's own value taken out: back to the default.
    Default,
}

/// The Keep menu beside a picker (D-18; EP-19b, the owner's choice): a pin that opens **Keep
/// for this project**, which writes the value shown (`current`), and **Back to the default**
/// when the project keeps a value of its own (`own`), which takes it out. The pin is lit in the
/// reference colour while the project keeps one. The item that would change nothing is greyed
/// out with its reason.
fn keep_menu(ui: &mut egui::Ui, current: &str, own: Option<&str>, default: &str) -> Option<Keep> {
    let lit = own.map(|_| style::reference(ui));
    let mut asked = None;
    let hover = match own {
        Some(own) => format!("The project keeps {own}: its launches and chains take it"),
        None => "Keep the choice for the project's launches and chains".to_owned(),
    };
    ui.menu_button(style::icon(icons::ICON_PUSH_PIN, lit), |ui| {
        let same = match own {
            Some(own) => own == current,
            None => current.eq_ignore_ascii_case(default),
        };
        let why = if own.is_some() {
            "the project keeps it already"
        } else {
            "it is the default already"
        };
        if ui
            .add_enabled(
                !same,
                egui::Button::new(format!("Keep {current} for this project")),
            )
            .on_hover_text("Written into the project's [project.launch] in the registry")
            .on_disabled_hover_text(why)
            .clicked()
        {
            asked = Some(Keep::Project);
            ui.close();
        }
        if ui
            .add_enabled(
                own.is_some(),
                egui::Button::new(format!("Back to the default ({default})")),
            )
            .on_hover_text("The project's own value is taken out of the registry")
            .on_disabled_hover_text("the project keeps no value of its own")
            .clicked()
        {
            asked = Some(Keep::Default);
            ui.close();
        }
    })
    .response
    .on_hover_text(hover);
    asked
}

/// What a Keep menu beside the Model or the Effort picker asked for (EP-19c).
#[derive(Debug, Clone, PartialEq, Eq)]
enum RuleKeep {
    /// The value shown, kept in the project's rule for the scope.
    Keep(Scope),
    /// The key taken out of the scope's rule: back to the rules.
    Back(Scope),
}

/// The Keep menu beside the Model or the Effort picker (D-22; EP-19c, the owner's choice), the
/// pin of [`keep_menu`]: **Keep for this project** writes the value shown (`current`) into the
/// project's rule for every brief; **Keep for phase Pn**, when the brief's phase has an id,
/// writes it into the rule of that phase alone; and **Back to the rules**, for each of the two
/// that keeps a value (`kept`), takes it out. The pin is lit in the reference colour while the
/// project keeps one for this brief. An item that would change nothing is greyed out with its
/// reason, and the CLI's own effort (`current` none) is no value a rule can keep.
fn rule_keep_menu(
    ui: &mut egui::Ui,
    salt: &str,
    current: Option<&str>,
    kept: &Kept,
    phase: Option<&str>,
) -> Option<RuleKeep> {
    let lit = kept.any().then(|| style::reference(ui));
    let hover = match (&kept.phase, phase, &kept.project) {
        (Some(value), Some(phase), _) => {
            format!("Phase {phase} keeps {value}: its briefs and chains take it")
        }
        (_, _, Some(value)) => format!("The project keeps {value}: its briefs and chains take it"),
        _ => "Keep the choice for the project's briefs, or for this phase's".to_owned(),
    };
    let shown = current.unwrap_or("the CLI's default");
    let mut asked = None;
    ui.push_id(salt, |ui| {
        ui.menu_button(style::icon(icons::ICON_PUSH_PIN, lit), |ui| {
            let item = |ui: &mut egui::Ui, on: bool, text: String, hover: &str, why: &str| {
                let clicked = ui
                    .add_enabled(on, egui::Button::new(text))
                    .on_hover_text(hover)
                    .on_disabled_hover_text(why)
                    .clicked();
                if clicked {
                    ui.close();
                }
                clicked
            };
            let why = |kept: &Option<String>, already: &'static str| match current {
                None => "the CLI's own effort is no value a rule can keep",
                Some(_) if kept.as_deref() == current => already,
                Some(_) => "",
            };
            let project_why = why(&kept.project, "the project keeps it already");
            if item(
                ui,
                project_why.is_empty(),
                format!("Keep {shown} for this project"),
                "Written into the project's rule for every brief in the registry, which comes \
                 after the rule table, so an S brief takes it too",
                project_why,
            ) {
                asked = Some(RuleKeep::Keep(Scope::Project));
            }
            if let Some(phase) = phase {
                let phase_why = why(&kept.phase, "the phase keeps it already");
                if item(
                    ui,
                    phase_why.is_empty(),
                    format!("Keep {shown} for phase {phase}"),
                    "Written into the project's rule for this phase alone, which comes before its \
                     rule for every brief",
                    phase_why,
                ) {
                    asked = Some(RuleKeep::Keep(Scope::Phase(phase.to_owned())));
                }
            }
            ui.separator();
            if let Some(value) = &kept.project
                && item(
                    ui,
                    true,
                    format!("Back to the rules for this project (drops {value})"),
                    "The value is taken out of the project's rule for every brief, and the rule \
                     with it once it keeps nothing",
                    "",
                )
            {
                asked = Some(RuleKeep::Back(Scope::Project));
            }
            if let (Some(value), Some(phase)) = (&kept.phase, phase)
                && item(
                    ui,
                    true,
                    format!("Back to the rules for phase {phase} (drops {value})"),
                    "The value is taken out of the project's rule for this phase, and the rule \
                     with it once it keeps nothing",
                    "",
                )
            {
                asked = Some(RuleKeep::Back(Scope::Phase(phase.to_owned())));
            }
            if !kept.any() {
                item(
                    ui,
                    false,
                    "Back to the rules".to_owned(),
                    "",
                    "the project keeps no value of its own here",
                );
            }
        })
        .response
        .on_hover_text(hover);
    });
    asked
}

/// A choice the owner made here.
fn owners<T>(value: T) -> Choice<T> {
    Choice {
        value,
        source: Source::Owner,
        detail: None,
    }
}

fn flags_of(preflight: Option<&Preflight>) -> Option<&session::cli::Flags> {
    preflight.and_then(|preflight| preflight.flags.as_ref().ok())
}

/// The effort levels the CLI's help lists, ultracode aside.
fn cli_efforts(preflight: Option<&Preflight>) -> Vec<String> {
    flags_of(preflight)
        .map(|flags| flags.efforts.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|level| level != "ultracode")
        .collect()
}

/// The effort levels to offer for `model`: the model's that the CLI lists too (`cli`), in the
/// CLI's order; the built-in ones when neither says.
fn efforts_of(catalog: &Catalog, cli: &[String], model: &str) -> Vec<String> {
    let model: Vec<String> = catalog
        .model(model)
        .map(|model| model.efforts.clone())
        .unwrap_or_default();
    match (cli.is_empty(), model.is_empty()) {
        (false, false) => cli.iter().filter(|l| model.contains(l)).cloned().collect(),
        (false, true) => cli.to_vec(),
        (true, false) => model,
        (true, true) => rules::EFFORTS.map(str::to_owned).to_vec(),
    }
}

/// The model picker: the catalog's main models, the rest below a line, and the current value
/// when the catalog lacks it. The new value, when the owner chose another.
fn model_picker(
    ui: &mut egui::Ui,
    salt: impl std::hash::Hash + std::fmt::Debug,
    catalog: &Catalog,
    current: &str,
    width: f32,
) -> Option<String> {
    let mut chosen = current.to_owned();
    egui::ComboBox::from_id_salt(salt)
        .width(width)
        .selected_text(model_text(catalog, current))
        .show_ui(ui, |ui| {
            let (main, more): (Vec<_>, Vec<_>) =
                catalog.models.iter().partition(|model| model.main);
            for model in main {
                ui.selectable_value(&mut chosen, model.id.clone(), &model.name);
            }
            if !more.is_empty() {
                ui.separator();
                for model in more {
                    ui.selectable_value(
                        &mut chosen,
                        model.id.clone(),
                        RichText::new(&model.name).weak(),
                    );
                }
            }
            if catalog.model(current).is_none() {
                ui.separator();
                ui.selectable_value(&mut chosen, current.to_owned(), current);
            }
        });
    (chosen != current).then_some(chosen)
}

/// The effort picker: `efforts` in the CLI's order, the current level marked when it is not
/// offered. The new value, when the owner chose another.
fn effort_picker(
    ui: &mut egui::Ui,
    salt: impl std::hash::Hash + std::fmt::Debug,
    efforts: &[String],
    current: Option<&str>,
    width: f32,
) -> Option<Option<String>> {
    let current = current.map(str::to_owned);
    let mut chosen = current.clone();
    egui::ComboBox::from_id_salt(salt)
        .width(width)
        .selected_text(current.as_deref().unwrap_or("the CLI's default"))
        .show_ui(ui, |ui| {
            for level in efforts {
                ui.selectable_value(&mut chosen, Some(level.clone()), level);
            }
            if let Some(level) = &current
                && !efforts.contains(level)
            {
                ui.selectable_value(
                    &mut chosen,
                    current.clone(),
                    format!("{level} (not offered)"),
                );
            }
        });
    (chosen != current).then_some(chosen)
}

/// One row of the chain plan (EP-19a): the Chain box with the brief's id, its size, its title,
/// and the pickers, which keep the owner's choice as shown once one is touched.
fn plan_row(
    ui: &mut egui::Ui,
    n: usize,
    planned: &mut Planned,
    catalog: &Catalog,
    cli: &[String],
    listed: bool,
    title_width: f32,
) {
    let hover = if planned.chain {
        "The chain may run it; untick it and the chain ends before it"
    } else {
        "Left out: the chain ends before it"
    };
    ui.checkbox(&mut planned.chain, planned.ep.to_string())
        .on_hover_text(hover);
    match planned.size {
        Some(size) => {
            style::chip(ui, size.as_str(), ui.visuals().weak_text_color());
        }
        None => {
            ui.label(RichText::new("no size").weak());
        }
    }
    ui.add_sized(
        [title_width, 20.0],
        egui::Label::new(RichText::new(&planned.title)).truncate(),
    )
    .on_hover_text(&planned.title);
    let shown = planned.shown();
    let reason = plan_reason(planned);
    let mut picked = None;
    ui.add_enabled_ui(planned.chain, |ui| {
        if let Some(model) = model_picker(ui, ("plan-model", n), catalog, &shown.model, PLAN_MODEL)
        {
            picked = Some(Picked {
                model,
                ..shown.clone()
            });
        }
    })
    .response
    .on_hover_text(&reason);
    ui.add_enabled_ui(planned.chain, |ui| {
        let efforts = efforts_of(catalog, cli, &shown.model);
        if let Some(effort) = effort_picker(
            ui,
            ("plan-effort", n),
            &efforts,
            shown.effort.as_deref(),
            PLAN_EFFORT,
        ) {
            picked = Some(Picked {
                effort,
                ..shown.clone()
            });
        }
    })
    .response
    .on_hover_text(&reason);
    ui.add_enabled_ui(planned.chain && listed, |ui| {
        let mut on = shown.ultracode;
        if ui
            .checkbox(&mut on, "ultracode")
            .on_disabled_hover_text(
                "This claude's help does not list --effort ultracode: it is passed only when \
                 listed (D-15)",
            )
            .changed()
        {
            picked = Some(Picked {
                ultracode: on,
                ..shown.clone()
            });
        }
    });
    if let Some(picked) = picked {
        planned.picked = Some(picked);
    }
}

/// A planned brief's reason on hover: the rules' resolution over its row, with the downshift's
/// judgement, and whether the owner's choice stands over it.
fn plan_reason(planned: &Planned) -> String {
    let resolution = &planned.resolution;
    let rules = format!(
        "the rules: {} at {} (model: {}; effort: {})",
        resolution.model.value,
        prompt::effort_words(
            resolution.effort.value.as_deref(),
            resolution.ultracode.value
        ),
        resolution.model.reason(),
        resolution.effort.reason()
    );
    let downshift = resolution
        .downshift
        .as_ref()
        .map_or(String::new(), |judged| format!("\n{}", judged.words));
    match &planned.picked {
        Some(_) => format!("your choice; {rules}{downshift}"),
        None => format!("{rules}{downshift}"),
    }
}

/// A model as the picker names it: the catalog's name, an alias with what it stands for, or the
/// id as written.
fn model_text(catalog: &Catalog, value: &str) -> String {
    match catalog.model(value) {
        Some(model) if model.id.eq_ignore_ascii_case(value) => model.name.clone(),
        Some(model) => format!("{value} → {}", model.name),
        None => value.to_owned(),
    }
}

/// One choice: its name, its widget, and the reason below in small type.
fn field(ui: &mut egui::Ui, name: &str, reason: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized([NAME, 20.0], egui::Label::new(RichText::new(name).strong()));
        add(ui);
    });
    ui.horizontal(|ui| {
        ui.add_space(NAME + ui.spacing().item_spacing.x);
        ui.add(egui::Label::new(RichText::new(reason).small().weak()).wrap());
    });
    ui.add_space(6.0);
}

/// The CLI's answers a screenshot run and the tests launch on: a recent version, logged in, and
/// the flags a headless run needs, from a synthetic help (never the real CLI, rule 5).
#[cfg(any(test, feature = "screenshot"))]
pub fn canned_preflight() -> Preflight {
    use crw_core::session::cli::{Auth, Flags};
    use crw_core::version::Version;

    const HELP: &str = "Options:
  --add-dir <directories...>  Additional directories
  --effort <level>  Effort level (low, medium, high, xhigh, max)
  --include-partial-messages  Partial chunks
  --input-format <format>  Input format
  --max-budget-usd <amount>  Budget
  --model <model>  Model
  -n, --name <name>  Display name
  --output-format <format>  Output format
  --permission-mode <mode>  Mode (choices: \"acceptEdits\", \"auto\", \"bypassPermissions\", \"manual\", \"dontAsk\", \"plan\")
  --permission-prompts <target>  Who answers
  --plugin-dir <path>  Load a plugin from a directory for this session only
  -p, --print  Print
  --replay-user-messages  Echo
  --session-id <uuid>  Session id
  --settings <file-or-json>  Settings
  --verbose  Verbose
";
    Preflight {
        program: PathBuf::from(r"C:\Tools\claude.exe"),
        found: true,
        version: Ok(Version::new(2, 1, 316)),
        auth: Ok(Auth {
            logged_in: true,
            method: Some("claude.ai".to_owned()),
            subscription: None,
        }),
        flags: Ok(Flags::parse(HELP)),
        taken: std::time::Instant::now(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crw_core::registry::{Project, Registry, State};
    use crw_core::roadmap::snapshot;
    use crw_core::rules::{Match, Rule, Set, Words};

    use super::*;

    /// A registry of harbor alone, the project's own rules given.
    fn registry(rules: Vec<Rule>) -> Registry {
        let mut harbor = Project::new("harbor", Path::new("harbor"), State::Active);
        harbor.rules = rules;
        Registry {
            format: 1,
            ignore: Vec::new(),
            settings: Default::default(),
            rules: Vec::new(),
            projects: vec![harbor],
        }
    }

    fn dialog(registry: Registry) -> Dialog {
        dialog_with(registry, &rules::Usage::unread(chrono::Utc::now()))
    }

    /// The dialog of the registry's first project, a folder of the fake portfolio, its downshift
    /// weighing `usage`.
    fn dialog_with(registry: Registry, usage: &rules::Usage) -> Dialog {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("portfolio")
            .join(&registry.projects[0].name);
        let snapshot = snapshot::read_tree(&folder);
        let draft = prompt::draft(
            &registry,
            &registry.projects[0],
            &snapshot,
            None,
            &[],
            usage,
            "2026-11-02",
        )
        .unwrap();
        let phase = draft.phase.as_deref();
        let kept_model = Kept::of(&registry.projects[0], phase, Key::Model);
        let kept_effort = Kept::of(&registry.projects[0], phase, Key::Effort);
        Dialog::new(
            draft,
            Opening {
                readiness: Readiness::of(
                    &folder,
                    Some(PathBuf::from(r"C:\Tools\VS Code\Code.exe")),
                    None,
                    &[],
                ),
                folder,
                program: PathBuf::from(r"C:\Tools\claude.exe"),
                neighbours: vec![PathBuf::from(r"C:\Data\beacon")],
                warnings: Vec::new(),
                template: registry.projects[0].launch.template.clone(),
                catalog: Catalog::built_in("for the test".to_owned()),
                date: "2026-11-02".to_owned(),
                vscode_setting: None,
                answer_timeout: None,
                route: (Route::Protocol, None),
                plan: None,
                kept_mode: registry.projects[0].launch.permission_mode.clone(),
                kept_model,
                kept_effort,
                levels: Vec::new(),
            },
        )
    }

    /// Harbor's dialog with its autopilot on and the chain plan after EP-4: EP-5 (M), then EP-6,
    /// a re-plan, where the list stops.
    fn dialog_with_plan() -> Dialog {
        let mut registry = registry(Vec::new());
        registry.projects[0].autopilot.enabled = true;
        let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("portfolio")
            .join("harbor");
        let snapshot = snapshot::read_tree(&folder);
        let usage = rules::Usage::unread(chrono::Utc::now());
        let plan = crw_core::autopilot::plan(
            &registry,
            &registry.projects[0],
            &snapshot,
            EpId::new(4),
            &[],
            &usage,
        );
        let mut dialog = dialog_with(registry, &usage);
        dialog.plan = Some(plan);
        dialog
    }

    #[test]
    fn the_chain_plan_lists_the_briefs_after_the_launched_one_and_takes_the_owners_box() {
        use crate::testing::{self, Screen};

        let mut dialog = dialog_with_plan();
        let plan = dialog.plan.as_ref().unwrap();
        assert_eq!(plan.after, EpId::new(4));
        assert_eq!(plan.briefs.len(), 1);
        assert_eq!(plan.briefs[0].ep, EpId::new(5));
        assert_eq!(plan.stop.as_ref().map(|stop| stop.ep), Some(EpId::new(6)));
        assert!(plan.briefs[0].chain && plan.briefs[0].picked.is_none());
        // In Visual the section is a line, and the editor keeps its room.
        assert!(dialog.visual());
        assert_eq!(dialog.plan_height(), 40.0);
        dialog.set_mode(LaunchMode::Headless);
        assert!(!dialog.visual());
        assert_eq!(dialog.plan_height(), 110.0);
        assert!(dialog.prompt.contains("with fable at xhigh effort"));

        // The plan's row is drawn with its box, which a click unticks (EP-15a's screen).
        let preflight = canned_preflight();
        let mut screen = Screen::new();
        let mut draw = |dialog: &mut Dialog, events| {
            screen.pass(events, |ui| {
                dialog.ui(ui, Some(&preflight), false, false, false);
            })
        };
        let output = draw(&mut dialog, Vec::new());
        let at = testing::text_at(&output, "EP-5").expect("the plan's row names EP-5");
        draw(&mut dialog, testing::moved(at));
        draw(
            &mut dialog,
            testing::button(at, egui::PointerButton::Primary, true),
        );
        draw(
            &mut dialog,
            testing::button(at, egui::PointerButton::Primary, false),
        );
        draw(&mut dialog, Vec::new());
        let plan = dialog.plan.as_ref().unwrap();
        assert!(!plan.briefs[0].chain, "the box is unticked by the click");
        assert_eq!(
            plan.answer(EpId::new(5)),
            crw_core::autopilot::Answer::LeftOut
        );
        // The owner's choice, as a picker would keep it, over the rules' resolution.
        let plan = dialog.plan.as_mut().unwrap();
        plan.briefs[0].chain = true;
        let rules = plan.briefs[0].shown();
        assert_eq!(
            (rules.model.as_str(), rules.effort.as_deref()),
            ("fable", Some("xhigh"))
        );
        assert!(plan_reason(&plan.briefs[0]).starts_with("the rules: fable at xhigh"));
        plan.briefs[0].picked = Some(Picked {
            effort: Some("max".to_owned()),
            ..rules
        });
        assert!(plan_reason(&plan.briefs[0]).starts_with("your choice; the rules:"));
        assert_eq!(
            plan.words(),
            "after EP-4: EP-5 fable at max (your choice); the chain would end at EP-6: EP-6 is \
             a re-plan, the owner's to run"
        );
    }

    #[test]
    fn the_keep_menu_beside_permissions_keeps_the_mode_or_takes_it_back() {
        use crate::testing::{self, Screen};

        let mut dialog = dialog(registry(Vec::new()));
        dialog.set_mode(LaunchMode::Headless);
        dialog.resolution.permission_mode = owners("auto".to_owned());
        let preflight = canned_preflight();
        let mut screen = Screen::new();
        // A click at `at`, over four passes: what the last painted, and what the dialog asked.
        let mut click = |dialog: &mut Dialog, at| {
            let mut asked = Vec::new();
            for events in [
                testing::moved(at),
                testing::button(at, egui::PointerButton::Primary, true),
                testing::button(at, egui::PointerButton::Primary, false),
            ] {
                screen.pass(events, |ui| {
                    asked.extend(dialog.ui(ui, Some(&preflight), false, false, false));
                });
            }
            let output = screen.pass(Vec::new(), |ui| {
                asked.extend(dialog.ui(ui, Some(&preflight), false, false, false));
            });
            (output, asked)
        };
        let pin = icons::ICON_PUSH_PIN.codepoint;
        let (output, _) = click(&mut dialog, egui::Pos2::new(-10.0, -10.0));
        // Model's, Effort's and the last, beside Permissions (EP-19c).
        let pins = testing::texts_at(&output, pin);
        assert_eq!(pins.len(), 3, "a pin beside Model, Effort and Permissions");
        let at = pins[2];
        let (output, _) = click(&mut dialog, at);
        let item = testing::text_at(&output, "Keep auto for this project").expect("the menu");
        let (_, asked) = click(&mut dialog, item);
        assert!(
            asked.contains(&Action::KeepPermissionMode(
                "harbor".to_owned(),
                Some("auto".to_owned())
            )),
            "{asked:?}"
        );

        // Once the project keeps it, the dialog says so, and the menu takes it back.
        dialog.kept_permission_mode(
            Some("auto".to_owned()),
            Choice {
                value: "auto".to_owned(),
                source: Source::Project,
                detail: None,
            },
        );
        assert_eq!(
            dialog.resolution.permission_mode.reason(),
            "the project's launch defaults"
        );
        let (output, _) = click(&mut dialog, at);
        let item = testing::text_at(&output, "Back to the default (acceptEdits)")
            .expect("the menu's way back");
        let (_, asked) = click(&mut dialog, item);
        assert!(
            asked.contains(&Action::KeepPermissionMode("harbor".to_owned(), None)),
            "{asked:?}"
        );
    }

    /// A rule the window keeps (EP-19c): for every brief, or for phase `phase` alone.
    fn kept(phase: Option<&str>, set: Set) -> Rule {
        let scope = phase.map_or(Scope::Project, |phase| Scope::Phase(phase.to_owned()));
        Rule {
            name: Some(scope.name()),
            when: Match {
                phase: Words(phase.map(str::to_owned).into_iter().collect()),
                ..Match::default()
            },
            set,
        }
    }

    fn effort(level: &str) -> Set {
        Set {
            effort: Some(level.to_owned()),
            ..Set::default()
        }
    }

    /// The dialog's brief resolved again over `registry`, as the board does after a keep.
    fn fresh(dialog: &Dialog, registry: &Registry) -> Resolution {
        let brief = rules::Brief {
            size: dialog.size,
            phase: dialog.phase.as_deref(),
            title: &dialog.title,
        };
        let usage = rules::Usage::unread(chrono::Utc::now());
        rules::resolve(registry, &registry.projects[0], &brief, &[], &usage)
    }

    #[test]
    fn the_keep_menus_beside_model_and_effort_keep_for_the_project_or_the_phase() {
        use crate::testing::{self, Screen};

        let mut dialog = dialog(registry(Vec::new()));
        dialog.set_mode(LaunchMode::Headless);
        // Harbor's next brief, EP-4, is in phase P1.
        assert_eq!(dialog.phase.as_deref(), Some("P1"));
        let preflight = canned_preflight();
        let mut screen = Screen::new();
        let mut click = |dialog: &mut Dialog, at| {
            let mut asked = Vec::new();
            for events in [
                testing::moved(at),
                testing::button(at, egui::PointerButton::Primary, true),
                testing::button(at, egui::PointerButton::Primary, false),
            ] {
                screen.pass(events, |ui| {
                    asked.extend(dialog.ui(ui, Some(&preflight), false, false, false));
                });
            }
            let output = screen.pass(Vec::new(), |ui| {
                asked.extend(dialog.ui(ui, Some(&preflight), false, false, false));
            });
            (output, asked)
        };
        let keep = |scope: Scope, key: Key, value: Option<&str>| Action::KeepRule {
            project: "harbor".to_owned(),
            scope,
            key,
            value: value.map(str::to_owned),
        };
        let pin = icons::ICON_PUSH_PIN.codepoint;
        let (output, _) = click(&mut dialog, egui::Pos2::new(-10.0, -10.0));
        let pins = testing::texts_at(&output, pin);
        assert_eq!(pins.len(), 3);

        // Model's menu: the model shown, for the project or for the phase; nothing to take back.
        dialog.resolution.model = owners("claude-opus-5-5".to_owned());
        let (output, _) = click(&mut dialog, pins[0]);
        assert!(testing::text_at(&output, "Keep claude-opus-5-5 for phase P1").is_some());
        assert!(testing::text_at(&output, "Back to the rules").is_some());
        let item = testing::text_at(&output, "Keep claude-opus-5-5 for this project")
            .expect("Model's menu");
        let (_, asked) = click(&mut dialog, item);
        let wanted = keep(Scope::Project, Key::Model, Some("claude-opus-5-5"));
        assert!(asked.contains(&wanted), "{asked:?}");

        // The board's answer: the registry keeps it for every brief, and the dialog follows.
        let opus = registry(vec![kept(
            None,
            Set {
                model: Some("claude-opus-5-5".to_owned()),
                ..Set::default()
            },
        )]);
        let resolved = fresh(&dialog, &opus);
        let asked = Some("claude-opus-5-5");
        dialog.kept_rule(Key::Model, asked, &opus.projects[0], resolved, None);
        assert_eq!(
            dialog.resolution.model.reason(),
            "the project's rule 1: kept for this project"
        );
        assert_eq!(dialog.kept_model.project.as_deref(), asked);
        assert!(
            dialog
                .prompt
                .contains("with claude-opus-5-5 at xhigh effort")
        );
        let (output, _) = click(&mut dialog, pins[0]);
        assert!(
            testing::text_at(&output, "Back to the rules").is_none(),
            "the way back names what it drops"
        );
        let item = testing::text_at(
            &output,
            "Back to the rules for this project (drops claude-opus-5-5)",
        )
        .expect("the way back");
        let (_, asked) = click(&mut dialog, item);
        assert!(
            asked.contains(&keep(Scope::Project, Key::Model, None)),
            "{asked:?}"
        );

        // Effort's menu keeps the level shown for the phase alone.
        let (output, _) = click(&mut dialog, pins[1]);
        let item = testing::text_at(&output, "Keep xhigh for phase P1").expect("Effort's menu");
        let (_, asked) = click(&mut dialog, item);
        let wanted = keep(Scope::Phase("P1".to_owned()), Key::Effort, Some("xhigh"));
        assert!(asked.contains(&wanted), "{asked:?}");

        // The board's answer puts the phase's rule first: the model's reason follows the new
        // numbers too (EP-19c's review), and the kept model is still read for its pin.
        let renumbered = registry(vec![
            kept(Some("P1"), effort("xhigh")),
            kept(
                None,
                Set {
                    model: Some("claude-opus-5-5".to_owned()),
                    ..Set::default()
                },
            ),
        ]);
        let resolved = fresh(&dialog, &renumbered);
        let project = &renumbered.projects[0];
        dialog.kept_rule(Key::Effort, Some("xhigh"), project, resolved, None);
        assert_eq!(
            dialog.resolution.effort.reason(),
            "the project's rule 1: kept for phase P1"
        );
        assert_eq!(
            dialog.resolution.model.reason(),
            "the project's rule 2: kept for this project"
        );
        assert_eq!(
            dialog.kept_model.project.as_deref(),
            Some("claude-opus-5-5")
        );
        assert_eq!(dialog.kept_effort.phase.as_deref(), Some("xhigh"));
    }

    #[test]
    fn a_kept_value_that_an_earlier_rule_stands_over_leaves_the_choice_shown() {
        let mut dialog = dialog(registry(Vec::new()));
        dialog.set_mode(LaunchMode::Headless);
        dialog.resolution.effort = owners(Some("max".to_owned()));
        // Phase P1 keeps high, and max is kept for every brief after it.
        let both = registry(vec![
            kept(Some("P1"), effort("high")),
            kept(None, effort("max")),
        ]);
        let project = &both.projects[0];
        let resolved = fresh(&dialog, &both);
        assert_eq!(
            rules::stands_over(project, &Scope::Project, Key::Effort, &resolved).as_deref(),
            Some("the project's rule 1: kept for phase P1")
        );
        dialog.kept_rule(Key::Effort, Some("max"), project, resolved, None);
        assert_eq!(dialog.resolution.effort.value.as_deref(), Some("max"));
        assert_eq!(dialog.resolution.effort.source, Source::Owner);
        assert_eq!(
            dialog.kept_effort,
            Kept {
                project: Some("max".to_owned()),
                phase: Some("high".to_owned()),
            }
        );

        // Back to the rules for the phase: the picker takes the rules' value, the project's max.
        let after = registry(vec![kept(None, effort("max"))]);
        let resolved = fresh(&dialog, &after);
        dialog.kept_rule(Key::Effort, None, &after.projects[0], resolved, None);
        assert_eq!(
            dialog.resolution.effort.reason(),
            "the project's rule 1: kept for this project"
        );
        assert_eq!(dialog.kept_effort.phase, None);
    }

    #[test]
    fn a_keep_resolves_the_plans_rows_left_alone_and_keeps_the_owners() {
        let mut dialog = dialog_with_plan();
        let mut with_max = registry(vec![kept(None, effort("max"))]);
        with_max.projects[0].autopilot.enabled = true;
        let snapshot = snapshot::read_tree(&dialog.folder);
        let usage = rules::Usage::unread(chrono::Utc::now());
        let project = &with_max.projects[0];
        let plan =
            || crw_core::autopilot::plan(&with_max, project, &snapshot, EpId::new(4), &[], &usage);
        // EP-5, left alone, shows the kept effort.
        let resolved = fresh(&dialog, &with_max);
        dialog.kept_rule(Key::Effort, Some("max"), project, resolved, Some(plan()));
        let row = &dialog.plan.as_ref().unwrap().briefs[0];
        assert_eq!(row.shown().effort.as_deref(), Some("max"));
        assert!(row.picked.is_none());

        // A row the owner touched keeps their choice, and one they unticked stays unticked.
        let planned = &mut dialog.plan.as_mut().unwrap().briefs[0];
        planned.picked = Some(Picked {
            model: "fable".to_owned(),
            effort: Some("low".to_owned()),
            ultracode: false,
        });
        planned.chain = false;
        let resolved = fresh(&dialog, &with_max);
        dialog.kept_rule(Key::Effort, Some("max"), project, resolved, Some(plan()));
        let row = &dialog.plan.as_ref().unwrap().briefs[0];
        assert_eq!(row.shown().effort.as_deref(), Some("low"));
        assert!(!row.chain);
        assert_eq!(row.resolution.effort.value.as_deref(), Some("max"));
    }

    #[test]
    fn the_efforts_offered_are_the_models_that_the_cli_lists() {
        let catalog = Catalog::built_in("for the test".to_owned());
        let cli: Vec<String> = ["low", "medium", "high", "xhigh", "max"]
            .map(str::to_owned)
            .to_vec();
        assert_eq!(efforts_of(&catalog, &cli, "fable"), cli);
        assert_eq!(
            efforts_of(&catalog, &[], "fable"),
            catalog.model("fable").unwrap().efforts
        );
        assert_eq!(efforts_of(&catalog, &cli, "model-unknown"), cli);
        assert_eq!(efforts_of(&catalog, &[], "model-unknown"), rules::EFFORTS);
        assert_eq!(cli_efforts(None), Vec::<String>::new());
        assert!(!cli_efforts(Some(&canned_preflight())).contains(&"ultracode".to_owned()));
    }

    #[test]
    fn the_launch_carries_every_choice() {
        let mut dialog = dialog(registry(Vec::new()));
        // Harbor's next brief is EP-4, an L: the tool's default, no built-in step.
        assert_eq!(dialog.ep, EpId::new(4));
        dialog.resolution.permission_mode = owners("auto".to_owned());
        assert_eq!(
            dialog.questions_words(),
            "answered in the Decisions pane while the session waits"
        );
        dialog.answer_timeout = Some(Duration::from_secs(1800));
        dialog.route = (Route::Defer, Some("settings.decisions".to_owned()));
        assert_eq!(
            dialog.questions_words(),
            "the defer route (settings.decisions): a question ends the run, and the answer \
             resumes it; a request unanswered for 30 min defers the run"
        );
        let launch = dialog.launch(Some(PathBuf::from(r"C:\ledger.sqlite")));
        assert_eq!(launch.answer_timeout, Some(Duration::from_secs(1800)));
        assert_eq!(launch.route, Route::Defer);
        assert_eq!(launch.project, "harbor");
        assert_eq!(launch.ep.as_deref(), Some("EP-4"));
        assert_eq!(launch.model, "fable");
        assert_eq!(launch.effort.as_deref(), Some("xhigh"));
        assert!(!launch.ultracode);
        assert_eq!(launch.permission_mode, "auto");
        assert_eq!(launch.neighbours, [PathBuf::from(r"C:\Data\beacon")]);
        assert_eq!(launch.prompt, dialog.prompt);
        assert_eq!(launch.ledger, Some(PathBuf::from(r"C:\ledger.sqlite")));
        assert!(launch.prompt.starts_with("Run the next brief of harbor:"));
    }

    #[test]
    fn the_prompt_follows_the_choices_until_it_is_edited() {
        let mut dialog = dialog(registry(Vec::new()));
        assert!(!dialog.edited());
        dialog.resolution.model = owners("claude-opus-5-5".to_owned());
        dialog.chose();
        assert!(dialog.prompt.contains("claude-opus-5-5 at xhigh effort"));
        assert!(!dialog.edited());
        dialog.prompt.push_str("\n\nAnd mind the tests.");
        dialog.resolution.effort = owners(Some("low".to_owned()));
        dialog.chose();
        assert!(dialog.edited());
        assert!(dialog.prompt.ends_with("And mind the tests."));
        assert!(dialog.prompt.contains("at xhigh effort"));
        assert_eq!(dialog.resolution.effort.reason(), "chosen here");
        dialog.regenerate(true);
        assert!(!dialog.edited());
        assert!(dialog.prompt.contains("claude-opus-5-5 at low effort"));
    }

    #[test]
    fn the_launch_mode_follows_the_registry_then_the_switch() {
        let mut registry = registry(Vec::new());
        registry.projects[0].launch.mode = Some(LaunchMode::Headless);
        let mut dialog = dialog(registry);
        assert!(!dialog.visual());
        assert_eq!(
            dialog.resolution.mode.reason(),
            "the project's launch defaults"
        );
        assert!(
            dialog
                .prompt
                .contains("with fable at xhigh effort: write those")
        );
        assert!(!dialog.readiness_due());
        // Visual: the prompt says that VS Code's panel chooses, and VS Code is looked at again.
        dialog.resolution.mode = owners(LaunchMode::Visual);
        dialog.read_at = None;
        dialog.chose();
        assert!(dialog.visual() && dialog.readiness_due());
        assert!(
            dialog
                .prompt
                .contains("into VS Code, whose Claude Code panel chooses its model and effort")
        );
        assert!(
            dialog
                .prompt
                .contains("(the launch suggested fable at xhigh effort)")
        );
        assert_eq!(dialog.visual_blocked(false, false), None);
        assert!(dialog.visual_blocked(false, true).is_some());
        dialog.readiness.code = None;
        assert!(
            dialog
                .visual_blocked(false, false)
                .unwrap()
                .contains("crw setting vscode")
        );

        let launch = dialog.visual_launch(
            Path::new(r"C:\Tools\VS Code\Code.exe"),
            Path::new(r"C:\Users\someone\.claude\ide"),
            Some(PathBuf::from(r"C:\ledger.sqlite")),
        );
        assert_eq!(
            (launch.project.as_str(), launch.ep.as_deref()),
            ("harbor", Some("EP-4"))
        );
        assert_eq!(launch.folder, dialog.folder);
        assert_eq!(launch.prompt, dialog.prompt);
        assert_eq!(launch.code, Path::new(r"C:\Tools\VS Code\Code.exe"));
        assert_eq!(launch.locks, Path::new(r"C:\Users\someone\.claude\ide"));
        assert_eq!(launch.ledger, Some(PathBuf::from(r"C:\ledger.sqlite")));
        assert_eq!(launch.wait, visual::WAIT);

        // An edited prompt stays as the owner left it when the mode changes.
        dialog.prompt.push_str("\n\nKeep this.");
        dialog.resolution.mode = owners(LaunchMode::Headless);
        dialog.chose();
        assert!(dialog.prompt.ends_with("Keep this."));
        assert!(dialog.prompt.contains("into VS Code"));
    }

    #[test]
    fn the_downshift_reaches_a_visual_dialog_as_the_suggestion_its_panel_takes() {
        use chrono::{TimeDelta, Utc};
        use crw_core::usage::Limit;

        // Lantern's next brief is EP-3, an S, launched visually; Fable stands at 82 %.
        let mut registry = registry(Vec::new());
        let mut lantern = Project::new("lantern", Path::new("lantern"), State::Active);
        lantern.launch.mode = Some(LaunchMode::Visual);
        registry.projects = vec![lantern];
        let now = Utc::now();
        let usage = rules::Usage {
            fable: Some(rules::Fable {
                limit: Limit {
                    percent: 82.0,
                    resets_at: Some(now + TimeDelta::days(3)),
                    active: Some(true),
                    severity: None,
                },
                at: now - TimeDelta::minutes(2),
            }),
            now,
        };
        let mut dialog = dialog_with(registry, &usage);
        assert_eq!(dialog.size, Some(Size::S));
        assert!(dialog.visual());
        assert_eq!(dialog.resolution.model.value, rules::DOWNSHIFT_MODEL);
        assert_eq!(dialog.resolution.model.source, Source::Downshift);
        // In Visual the pickers serve Headless only; the rules' reason stays as the suggestion.
        let reason = dialog.model_reason();
        assert_eq!(
            reason,
            "headless only: choose it in the new tab's panel; suggested by the Fable downshift \
             (D-23) (Fable at 82 %, read 2 min ago, above 60 %)"
        );
        assert!(
            dialog
                .prompt
                .contains("(the launch suggested claude-opus-5-5 at high effort)"),
            "{}",
            dialog.prompt
        );
        // Headless: the reason as the rules gave it.
        dialog.resolution.mode = owners(LaunchMode::Headless);
        assert_eq!(
            dialog.model_reason(),
            "the Fable downshift (D-23) (Fable at 82 %, read 2 min ago, above 60 %)"
        );
        assert_eq!(picker_reason(false, "why".to_owned()), "why");
        // The owner's own choice keeps what the downshift proposed in view.
        dialog.resolution.model = owners("fable".to_owned());
        assert_eq!(
            dialog.model_reason(),
            "chosen here\nthe downshift proposed claude-opus-5-5: Fable at 82 %, read 2 min ago, \
             above 60 %"
        );
        // Back in Visual, a choice made in Headless is the suggestion.
        dialog.resolution.mode = owners(LaunchMode::Visual);
        assert!(
            dialog.model_reason().starts_with(
                "headless only: choose it in the new tab's panel; suggested by your \
                              choice in Headless\n"
            ),
            "{}",
            dialog.model_reason()
        );
    }

    #[test]
    fn the_cli_and_the_model_decide_what_is_offered() {
        let preflight = canned_preflight();
        let mut dialog = dialog(registry(vec![Rule {
            set: Set {
                ultracode: Some(true),
                ..Set::default()
            },
            ..Rule::default()
        }]));
        assert!(dialog.resolution.ultracode.value);
        assert!(dialog.prompt.contains("at ultracode effort"));
        // This claude does not list ultracode: it is turned off and the prompt says the level.
        dialog.fit(&preflight);
        assert!(!dialog.resolution.ultracode.value);
        assert!(
            dialog
                .resolution
                .ultracode
                .reason()
                .contains("does not list")
        );
        assert!(!dialog.prompt.contains("ultracode effort"));
        assert_eq!(
            dialog.efforts(Some(&preflight)),
            ["low", "medium", "high", "xhigh", "max"]
        );
        assert_eq!(Dialog::modes(Some(&preflight)).len(), 6);
        assert_eq!(Dialog::modes(None), MODES);
        dialog.resolution.model = owners("model-unknown".to_owned());
        assert!(dialog.checks(Some(&preflight))[0].contains("not in Claude Code's model catalog"));
        assert_eq!(model_text(&dialog.catalog, "fable"), "fable → Fable 5.1");
        assert_eq!(model_text(&dialog.catalog, "claude-opus-5-5"), "Opus 5.5");
    }
}
