//! The Projects pane (D-25): a toolbar (the name filter, the sort key, the filters EP-7 feeds),
//! then the groups by state, each with a header that folds it, and one dense row per project:
//! its next brief, its progress, what waits on the owner, whether a session is in progress, and
//! the switches later briefs bring to life. A row expands inline; a right click pins it or sets
//! its priority or its state; an unadopted folder offers Adopt and Ignore.

use crw_core::registry::State;
use eframe::egui::{self, Align, Checkbox, Layout, Rect, RichText, Sense, UiBuilder, vec2};

use crate::board::{Action, Board, View};
use crate::markup::{self, Look};
use crate::portfolio::{GroupKind, Member, ProjectRow, Selection, SortKey};
use crate::style::{self, icons};

/// The columns of a row, in points, left to right; the title takes what is left. EP-7's badges
/// join between the counts and the session status.
const EXPANDER: f32 = 20.0;
const PIN: f32 = 20.0;
const NAME: f32 = 160.0;
const STATE: f32 = 104.0;
const NEXT: f32 = 70.0;
/// Below this the title gives way: first the counts go, then the progress.
const TITLE_AT_LEAST: f32 = 160.0;
const PROGRESS: f32 = 44.0;
const ITEMS: f32 = 72.0;
const SESSION: f32 = 24.0;
const AUTOPILOT: f32 = 26.0;
const START: f32 = 32.0;
const GAP: f32 = 5.0;
/// Adopt and Ignore at the end of an unadopted folder's row.
const FOLDER_BUTTONS: f32 = 150.0;

/// The cells of one row; the progress and the counts only when the row is wide enough.
struct Columns {
    expander: Rect,
    pin: Rect,
    name: Rect,
    state: Rect,
    next: Rect,
    title: Rect,
    progress: Option<Rect>,
    items: Option<Rect>,
    session: Rect,
    autopilot: Rect,
    start: Rect,
}

impl Columns {
    fn of(row: Rect) -> Self {
        let always = [EXPANDER, PIN, NAME, STATE, NEXT, SESSION, AUTOPILOT, START];
        // Eight columns and the title: eight gaps.
        let spare = row.width() - always.iter().sum::<f32>() - GAP * always.len() as f32;
        let (progress, items) = if spare - PROGRESS - ITEMS - 2.0 * GAP >= TITLE_AT_LEAST {
            (true, true)
        } else if spare - PROGRESS - GAP >= TITLE_AT_LEAST {
            (true, false)
        } else {
            (false, false)
        };
        let title = spare
            - if progress { PROGRESS + GAP } else { 0.0 }
            - if items { ITEMS + GAP } else { 0.0 };
        let mut x = row.left();
        let mut next = |width: f32| {
            let cell = Rect::from_min_size(egui::pos2(x, row.top()), vec2(width, row.height()));
            x += width + GAP;
            cell
        };
        Self {
            expander: next(EXPANDER),
            pin: next(PIN),
            name: next(NAME),
            state: next(STATE),
            next: next(NEXT),
            title: next(title.max(40.0)),
            progress: progress.then(|| next(PROGRESS)),
            items: items.then(|| next(ITEMS)),
            session: next(SESSION),
            autopilot: next(AUTOPILOT),
            start: next(START),
        }
    }
}

/// Draws `add` inside `rect`, left to right and centred on the row's middle line.
fn cell<R>(ui: &mut egui::Ui, rect: Rect, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    add(&mut child)
}

impl Board {
    pub fn projects_pane(&mut self, ui: &mut egui::Ui) {
        self.list_toolbar(ui);
        ui.separator();
        if let Some(problem) = &self.portfolio.problem {
            ui.label(RichText::new(problem).color(style::red(ui)));
            return;
        }
        let groups = self.portfolio.groups(self.view.sort, &self.view.filter);
        if groups.is_empty() {
            let text = if self.loader.reading_files {
                "Reading the registry and the projects…"
            } else if self.view.filter.trim().is_empty() {
                "No projects: the registry lists none and the data root holds no folder."
            } else {
                "No project's name contains the filter."
            };
            ui.label(RichText::new(text).weak());
            return;
        }
        let now = chrono::Utc::now();
        let mut actions = Vec::new();
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                for group in &groups {
                    let folded = self.view.is_folded(group.kind);
                    if group_header(ui, group.kind, group.members.len(), folded, &self.view) {
                        actions.push(Action::ToggleGroup(group.kind));
                    }
                    if folded {
                        continue;
                    }
                    for &member in &group.members {
                        match member {
                            Member::Project(i) => actions.extend(project_row(
                                ui,
                                &self.portfolio.projects[i],
                                &self.view,
                                self.scroll_to_selected,
                                now,
                            )),
                            Member::Folder(i) => {
                                let folder = &self.portfolio.unadopted[i];
                                actions.extend(folder_row(
                                    ui,
                                    &folder.name,
                                    folder.git,
                                    folder.roadmap,
                                    &self.view,
                                    self.scroll_to_selected,
                                ));
                            }
                        }
                    }
                    ui.add_space(6.0);
                }
            });
        self.scroll_to_selected = false;
        self.actions.extend(actions);
    }

    fn list_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(style::icon(icons::ICON_SEARCH, None));
            ui.add(
                egui::TextEdit::singleline(&mut self.view.filter)
                    .hint_text("Filter by name")
                    .desired_width(170.0),
            );
            if !self.view.filter.is_empty()
                && ui
                    .small_button(style::icon(icons::ICON_CLOSE, None))
                    .on_hover_text("Clear the filter")
                    .clicked()
            {
                self.view.filter.clear();
            }
            ui.separator();
            let sort = self.view.sort;
            ui.menu_button(
                format!("{} Sort: {}", icons::ICON_SORT.codepoint, sort.label()),
                |ui| {
                    ui.label(RichText::new("Within each group, pinned first, then").weak());
                    for key in SortKey::ALL {
                        let available = key.arrives_at().is_none();
                        let response = ui
                            .add_enabled_ui(available, |ui| {
                                ui.radio_value(&mut self.view.sort, key, key.label())
                                    .on_hover_text(key.hint())
                            })
                            .inner;
                        if let Some(brief) = key.arrives_at() {
                            response.on_disabled_hover_text(format!(
                                "{} needs data that arrives at {brief}.",
                                key.label()
                            ));
                        }
                    }
                },
            );
            ui.menu_button(
                format!("{} Filters", icons::ICON_FILTER_LIST.codepoint),
                |ui| {
                    for name in ["By stack", "By hosting"] {
                        ui.add_enabled(false, egui::Button::new(name))
                            .on_disabled_hover_text(
                                "The stack chips and the hosting badges arrive at EP-7.",
                            );
                    }
                },
            );
        });
    }
}

/// A group's header: the fold chevron, the state and its count. Returns whether it was clicked.
fn group_header(
    ui: &mut egui::Ui,
    kind: GroupKind,
    count: usize,
    folded: bool,
    view: &View,
) -> bool {
    let height = view.density.row_height() - 2.0;
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.weak_bg_fill);
    }
    let state = match kind {
        GroupKind::State(state) => Some(state),
        GroupKind::Unadopted => None,
    };
    cell(ui, rect, |ui| {
        let chevron = if folded {
            icons::ICON_CHEVRON_RIGHT
        } else {
            icons::ICON_EXPAND_MORE
        };
        ui.label(style::icon(chevron, None));
        ui.label(style::icon(icons::ICON_CIRCLE, Some(style::state_colour(state))).small());
        ui.label(RichText::new(heading(kind)).strong());
        ui.label(RichText::new(count.to_string()).weak());
    });
    response
        .on_hover_text(if folded {
            "Unfold the group"
        } else {
            "Fold the group"
        })
        .clicked()
}

fn heading(kind: GroupKind) -> &'static str {
    match kind {
        GroupKind::State(State::Active) => "Active",
        GroupKind::State(State::Paused) => "Paused",
        GroupKind::State(State::Maintenance) => "Maintenance",
        GroupKind::State(State::Unformalised) => "Unformalised",
        GroupKind::State(State::Done) => "Done",
        GroupKind::State(State::Archived) => "Archived",
        GroupKind::Unadopted => "Unadopted folders",
    }
}

/// The row's background: the selection's tint, or the hover's.
fn background(ui: &egui::Ui, rect: Rect, selected: bool, hovered: bool) {
    let visuals = ui.visuals();
    if selected {
        ui.painter()
            .rect_filled(rect, 4.0, visuals.selection.bg_fill.gamma_multiply(0.55));
    } else if hovered {
        ui.painter()
            .rect_filled(rect, 4.0, visuals.widgets.hovered.weak_bg_fill);
    }
}

/// One project's row, and below it the inline expansion when it is open.
fn project_row(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    view: &View,
    scroll: bool,
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<Action> {
    let mut actions = Vec::new();
    let name = row.name();
    let selection = Selection::Project(name.to_owned());
    let selected = view.selected.as_ref() == Some(&selection);
    let expanded = view.expanded.contains(name);
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), view.density.row_height()),
        Sense::click(),
    );
    background(ui, rect, selected, response.hovered());
    if selected && scroll {
        response.scroll_to_me(Some(Align::Center));
    }
    let columns = Columns::of(rect);
    let weak = ui.visuals().weak_text_color();

    cell(ui, columns.expander, |ui| {
        let chevron = if expanded {
            icons::ICON_EXPAND_MORE
        } else {
            icons::ICON_CHEVRON_RIGHT
        };
        let toggle = egui::Button::new(style::icon(chevron, Some(weak))).frame(false);
        if ui
            .add(toggle)
            .on_hover_text("Expand the row (→ and ←)")
            .clicked()
        {
            actions.push(Action::ToggleExpanded(name.to_owned()));
        }
    });
    if row.project.pinned {
        cell(ui, columns.pin, |ui| {
            ui.label(style::icon(icons::ICON_PUSH_PIN, Some(weak)))
                .on_hover_text("Pinned: first in its group");
        });
    }
    cell(ui, columns.name, |ui| {
        ui.add(egui::Label::new(RichText::new(name).strong()).truncate());
    });
    cell(ui, columns.state, |ui| {
        let state = row.project.state;
        style::chip(ui, state.as_str(), style::state_colour(Some(state)));
        if !row.proposals(now).is_empty() {
            ui.label(style::icon(icons::ICON_LIGHTBULB, Some(style::amber(ui))))
                .on_hover_text("A state change is proposed: see the Detail pane");
        }
    });
    let counts = row.counts();
    match (&row.folder, row.next()) {
        (None, _) => {
            cell(ui, columns.title, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("Folder missing: {}", row.path.display()))
                            .color(style::red(ui)),
                    )
                    .truncate(),
                );
            });
        }
        (Some(_), Some(next)) => {
            cell(ui, columns.next, |ui| {
                ui.label(RichText::new(next.id.to_string()).monospace());
                if let Some(size) = next.size {
                    ui.label(RichText::new(size.as_str()).small().strong())
                        .on_hover_text(size_hint(size));
                }
            });
            cell(ui, columns.title, |ui| {
                let title = markup::job(ui, &next.brief.title, Look::default());
                ui.add(egui::Label::new(title).truncate());
            });
        }
        (Some(_), None) => {
            let text = if counts.all_checked() {
                "Every row is checked"
            } else if row.snapshot.current().is_none() {
                "No roadmap/README.md"
            } else {
                "No phase table read"
            };
            cell(ui, columns.next.union(columns.title), |ui| {
                ui.add(egui::Label::new(RichText::new(text).weak()).truncate());
            });
        }
    }
    if let Some(progress) = columns.progress
        && counts.total > 0
    {
        cell(ui, progress, |ui| {
            ui.label(RichText::new(format!("{}/{}", counts.done, counts.total)).weak())
                .on_hover_text(format!(
                    "{} of {} rows done{}",
                    counts.done,
                    counts.total,
                    match counts.stretch {
                        0 => String::new(),
                        n => format!(", {n} stretch"),
                    }
                ));
        });
    }
    if let Some(cells) = columns.items {
        cell(ui, cells, |ui| counts_cell(ui, row));
    }
    cell(ui, columns.session, |ui| {
        let reasons = row.in_progress();
        if !reasons.is_empty() {
            let texts: Vec<String> = reasons.iter().map(ToString::to_string).collect();
            ui.label(style::icon(icons::ICON_EDIT_NOTE, Some(style::amber(ui))))
                .on_hover_text(format!("Session in progress: {}", texts.join("; ")));
        } else if row.awaits_git() {
            ui.label(style::icon(icons::ICON_SCHEDULE, Some(weak)))
                .on_hover_text("Git has not been asked yet");
        }
    });
    cell(ui, columns.autopilot, |ui| {
        let mut on = row.project.autopilot.enabled;
        ui.add_enabled(false, Checkbox::without_text(&mut on))
            .on_disabled_hover_text("Autopilot: arrives at EP-19 (A)");
    });
    cell(ui, columns.start, |ui| {
        ui.add_enabled(
            false,
            egui::Button::new(style::icon(icons::ICON_PLAY_ARROW, None)),
        )
        .on_disabled_hover_text("Start: arrives at EP-10 with the launch dialog (S)");
    });

    if response.double_clicked() {
        actions.push(Action::Open(selection.clone()));
    } else if response.clicked() {
        actions.push(Action::Select(selection.clone()));
    }
    response.context_menu(|ui| {
        let pinned = row.project.pinned;
        if ui.button(if pinned { "Unpin" } else { "Pin" }).clicked() {
            actions.push(Action::SetPinned(name.to_owned(), !pinned));
            ui.close();
        }
        ui.menu_button("Priority", |ui| {
            for priority in 1..=5u8 {
                let label = match priority {
                    1 => "1 (highest)".to_owned(),
                    5 => "5 (lowest)".to_owned(),
                    n => n.to_string(),
                };
                if ui.radio(row.project.priority == priority, label).clicked() {
                    actions.push(Action::SetPriority(name.to_owned(), priority));
                    ui.close();
                }
            }
        });
        ui.menu_button("State", |ui| {
            for state in State::ALL {
                if ui
                    .radio(row.project.state == state, state.as_str())
                    .clicked()
                {
                    actions.push(Action::SetState(name.to_owned(), state));
                    ui.close();
                }
            }
        });
    });

    if expanded {
        expansion(ui, row);
    }
    actions
}

/// What waits on the owner: open owner items, open inbox messages (red when one is urgent).
fn counts_cell(ui: &mut egui::Ui, row: &ProjectRow) {
    // Each icon close to its number, the two pairs apart.
    ui.spacing_mut().item_spacing.x = 3.0;
    let items = row.open_items();
    if items > 0 {
        ui.label(style::icon(icons::ICON_TASK_ALT, Some(style::amber(ui))))
            .on_hover_text(format!("{items} open owner items"));
        ui.label(items.to_string());
        ui.add_space(6.0);
    }
    let (messages, urgent) = row.open_messages();
    if messages > 0 {
        let colour = if urgent > 0 {
            style::red(ui)
        } else {
            style::amber(ui)
        };
        ui.label(style::icon(icons::ICON_INBOX, Some(colour)))
            .on_hover_text(match urgent {
                0 => format!("{messages} open inbox messages"),
                n => format!("{messages} open inbox messages, {n} urgent"),
            });
        ui.label(messages.to_string());
    }
}

fn size_hint(size: crw_core::roadmap::Size) -> &'static str {
    match size {
        crw_core::roadmap::Size::S => "S: about 30 minutes of one session",
        crw_core::roadmap::Size::M => "M: about an hour",
        crw_core::roadmap::Size::L => "L: about two hours",
    }
}

/// The inline expansion: each phase's progress, with the graphs' placeholder (EP-26).
fn expansion(ui: &mut egui::Ui, row: &ProjectRow) {
    let indent = EXPANDER + PIN + GAP * 2.0;
    ui.horizontal(|ui| {
        ui.add_space(indent);
        ui.vertical(|ui| {
            match row.snapshot.current() {
                Some(roadmap) if !roadmap.phases.is_empty() => {
                    for phase in &roadmap.phases {
                        let heading = &phase.heading;
                        let counts = phase.counts();
                        let id = heading.id.as_deref().unwrap_or("-");
                        let status = heading
                            .status
                            .as_deref()
                            .map(|s| format!(" ({s})"))
                            .unwrap_or_default();
                        let progress = match counts.total {
                            0 => "no rows".to_owned(),
                            total => format!("{} of {total}", counts.done),
                        };
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(id).monospace());
                            let fraction = if counts.total > 0 {
                                counts.done as f32 / counts.total as f32
                            } else {
                                0.0
                            };
                            ui.add(
                                egui::ProgressBar::new(fraction)
                                    .desired_width(90.0)
                                    .desired_height(8.0),
                            );
                            ui.label(RichText::new(progress).weak());
                            ui.label(format!("{}{status}", heading.name));
                        });
                    }
                }
                _ => {
                    ui.label(RichText::new("No phases to show.").weak());
                }
            }
            ui.label(
                RichText::new("The heatmap and the burn-up sparkline arrive at EP-26.")
                    .small()
                    .weak(),
            );
        });
    });
    ui.add_space(4.0);
}

/// An unadopted folder's row: what it holds, Adopt and Ignore (D-7).
fn folder_row(
    ui: &mut egui::Ui,
    name: &str,
    git: bool,
    roadmap: bool,
    view: &View,
    scroll: bool,
) -> Vec<Action> {
    let mut actions = Vec::new();
    let selection = Selection::Folder(name.to_owned());
    let selected = view.selected.as_ref() == Some(&selection);
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), view.density.row_height()),
        Sense::click(),
    );
    background(ui, rect, selected, response.hovered());
    if selected && scroll {
        response.scroll_to_me(Some(Align::Center));
    }
    let columns = Columns::of(rect);
    cell(ui, columns.pin, |ui| {
        ui.label(style::icon(
            icons::ICON_FOLDER,
            Some(ui.visuals().weak_text_color()),
        ));
    });
    cell(ui, columns.name, |ui| {
        ui.add(egui::Label::new(RichText::new(name).strong()).truncate());
    });
    cell(ui, columns.state, |ui| {
        style::chip(ui, "unadopted", style::state_colour(None));
    });
    // The two buttons keep their room at the row's end; the description takes the rest.
    let buttons = Rect::from_min_max(
        egui::pos2(rect.right() - FOLDER_BUTTONS, rect.top()),
        rect.right_bottom(),
    );
    let what = Rect::from_min_max(
        columns.next.left_top(),
        egui::pos2(buttons.left() - GAP, rect.bottom()),
    );
    cell(ui, what, |ui| {
        let text = match (git, roadmap) {
            (true, true) => "A git repository with a roadmap",
            (true, false) => "A git repository without a roadmap",
            (false, _) => "A folder, not a repository",
        };
        ui.add(egui::Label::new(RichText::new(text).weak()).truncate());
    });
    cell(ui, buttons, |ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .button("Ignore")
                .on_hover_text("Leave it out of this group; the registry remembers")
                .clicked()
            {
                actions.push(Action::Ignore(name.to_owned()));
            }
            if ui
                .button("Adopt")
                .on_hover_text("Register it and add it to the workspace file")
                .clicked()
            {
                actions.push(Action::Adopt(name.to_owned()));
            }
        });
    });
    if response.double_clicked() {
        actions.push(Action::Open(selection));
    } else if response.clicked() {
        actions.push(Action::Select(selection));
    }
    actions
}
