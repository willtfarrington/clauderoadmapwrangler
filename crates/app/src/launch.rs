//! The launch dialog (D-16, D-17, D-18, D-22): what a session of a project's next brief starts
//! with, put together by the rule table and the prompt generator (`crw_core::prompt::draft`),
//! shown for the owner to change, and handed on: to the runner (EP-9) as a headless session, or
//! to VS Code as a visual one, its prompt filled into a Claude Code tab for the owner to send
//! (EP-11). The prompt follows the choices, the launch mode among them, until the owner edits it;
//! the models come from Claude Code's catalog, the effort levels, ultracode and the permission
//! modes from what the CLI's help lists (D-15), and each choice shows the reason the rules gave
//! it. A visual launch takes the model and effort as a suggestion, since VS Code's panel chooses
//! them. A headless launch says how its questions reach the owner: the route and the project's
//! time limit (D-19; EP-14).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crw_core::catalog::Catalog;
use crw_core::prompt::{self, Chosen, Draft, Parts};
use crw_core::registry::LaunchMode;
use crw_core::roadmap::{EpId, Size};
use crw_core::rules::{self, Choice, Resolution, Source};
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
}

impl Dialog {
    pub fn new(draft: Draft, opening: Opening) -> Self {
        let Draft {
            ep,
            title,
            size,
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
        }
    }

    /// Whether the launch is visual (D-16).
    pub fn visual(&self) -> bool {
        self.resolution.mode.value == LaunchMode::Visual
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
        let cli: Vec<String> = flags_of(preflight)
            .map(|flags| flags.efforts.clone())
            .unwrap_or_default()
            .into_iter()
            .filter(|level| level != "ultracode")
            .collect();
        let model: Vec<String> = self
            .catalog
            .model(&self.resolution.model.value)
            .map(|model| model.efforts.clone())
            .unwrap_or_default();
        match (cli.is_empty(), model.is_empty()) {
            (false, false) => cli.into_iter().filter(|l| model.contains(l)).collect(),
            (false, true) => cli,
            (true, false) => model,
            (true, true) => rules::EFFORTS.map(str::to_owned).to_vec(),
        }
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
        let height = (screen.height() - 300.0).clamp(200.0, 620.0);
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
                        self.choices(ui, preflight, asking);
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
        ui.separator();
        self.footer(ui, preflight, starting, opening, &mut actions);
        actions
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

    fn choices(&mut self, ui: &mut egui::Ui, preflight: Option<&Preflight>, asking: bool) {
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
        let visual = self.visual();
        let suggested = |reason: String| {
            if visual {
                format!("a suggestion: VS Code's panel chooses ({reason})")
            } else {
                reason
            }
        };

        let reason = suggested(self.resolution.model.reason());
        field(ui, "Model", &reason, |ui| {
            let current = self.resolution.model.value.clone();
            let mut chosen = current.clone();
            egui::ComboBox::from_id_salt("launch-model")
                .width(combo)
                .selected_text(model_text(&self.catalog, &current))
                .show_ui(ui, |ui| {
                    let (main, more): (Vec<_>, Vec<_>) =
                        self.catalog.models.iter().partition(|model| model.main);
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
                    if self.catalog.model(&current).is_none() {
                        ui.separator();
                        ui.selectable_value(&mut chosen, current.clone(), &current);
                    }
                });
            if chosen != current {
                self.resolution.model = owners(chosen);
                changed = true;
            }
        });

        let efforts = self.efforts(preflight);
        let reason = suggested(self.resolution.effort.reason());
        field(ui, "Effort", &reason, |ui| {
            let current = self.resolution.effort.value.clone();
            let mut chosen = current.clone();
            egui::ComboBox::from_id_salt("launch-effort")
                .width(combo)
                .selected_text(current.as_deref().unwrap_or("the CLI's default"))
                .show_ui(ui, |ui| {
                    for level in &efforts {
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
            if chosen != current {
                self.resolution.effort = owners(chosen);
                changed = true;
            }
        });

        let listed = flags_of(preflight).is_some_and(|flags| flags.ultracode());
        let reason = suggested(self.resolution.ultracode.reason());
        field(ui, "Ultracode", &reason, |ui| {
            let mut on = self.resolution.ultracode.value;
            ui.add_enabled(listed, egui::Checkbox::new(&mut on, "--effort ultracode"))
                .on_disabled_hover_text(
                    "This claude's help does not list --effort ultracode: it is passed only when \
                     listed (D-15)",
                );
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
            ui.add_enabled_ui(!visual, |ui| {
                egui::ComboBox::from_id_salt("launch-mode")
                    .width(combo)
                    .selected_text(&current)
                    .show_ui(ui, |ui| {
                        for mode in &modes {
                            ui.selectable_value(&mut chosen, mode.clone(), mode);
                        }
                        if !modes.contains(&current) {
                            ui.selectable_value(&mut chosen, current.clone(), &current);
                        }
                    });
            });
            if chosen != current {
                self.resolution.permission_mode = owners(chosen);
                changed = true;
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
                "Runs headless in {}; the ledger records it, and closing the window stops it.",
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
    use crw_core::rules::{Rule, Set};

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
        let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("portfolio")
            .join("harbor");
        let snapshot = snapshot::read_tree(&folder);
        let draft = prompt::draft(
            &registry,
            &registry.projects[0],
            &snapshot,
            None,
            &[],
            "2026-11-02",
        )
        .unwrap();
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
            },
        )
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
