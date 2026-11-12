//! The Projects pane (D-25): a toolbar (the name filter, the sort key, the stack and hosting
//! filters, the legend's switch), a header row that names the columns, then the groups by state,
//! each with a header that folds it, and one dense row per project: its next brief, its progress,
//! what waits on the owner, its badges (EP-7) with this week's usage share (EP-18b), whether a
//! session is in progress, and the switches
//! later briefs bring to life. Each column is as wide as the widest value the portfolio holds
//! (EP-7a); a wide window shares its spare width between the title and the gaps, a narrow one
//! drops the optional columns in D-25's order. A row expands inline; a right click pins it or
//! sets its priority or its state; an unadopted folder offers Adopt and Ignore. The legend, a key
//! to every mark of the row, opens at the foot of the pane.

use chrono::{DateTime, Utc};
use crw_core::badges::hosting::Class;
use crw_core::badges::stack::Kind;
use crw_core::registry::State;
use crw_core::roadmap::table::{Counts, Row};
use eframe::egui::{self, Align, Checkbox, Layout, Rect, RichText, Sense, UiBuilder, vec2};

use crate::badges::{
    self, FOLDER, GIT_PENDING, MESSAGES, OWNER_ITEMS, PINNED, PROPOSAL, SESSION, URGENT,
};
use crate::board::{Action, Board, View};
use crate::markup::{self, Look};
use crate::portfolio::{GroupKind, Member, Portfolio, ProjectRow, Selection, SortKey};
use crate::style::{self, icons};

/// The fixed columns: the row's expander and pin, and the two switches at its end.
const EXPANDER: f32 = 20.0;
const PIN: f32 = 20.0;
const AUTOPILOT: f32 = 26.0;
const START: f32 = 32.0;
/// The title's least in the full form: below it the badges shrink to their compact form.
const TITLE_AT_LEAST: f32 = 160.0;
/// The title's least in the compact forms (D-25 addendum of 2026-11-11; EP-38): the title, cut
/// with an ellipsis and whole on hover, gives way down to it before the counts leave, and again
/// before the compact badges leave and before the progress does, so a long project name costs
/// the title first and a badge column last. Set from pictures of a copy of the real roadmaps at
/// 1440 and 1600 points (the owner's choice at EP-38: the compact badges show at 1600 at the
/// default layout, without the counts, the title cut at 89).
const TITLE_FLOOR: f32 = 80.0;
/// The title's floor when even the fewest columns leave it less.
const TITLE_MIN: f32 = 40.0;
const _: () = assert!(TITLE_MIN < TITLE_FLOOR && TITLE_FLOOR < TITLE_AT_LEAST);
/// Between two columns; the expander, the pin and the name sit closer.
const GAP: f32 = 10.0;
const TIGHT: f32 = 2.0;
/// On a wide row the gaps after the name take this share of the width beyond the title's least,
/// each widening by at most [`SPREAD`]; the title takes the rest.
const SPREAD_SHARE: f32 = 0.3;
const SPREAD: f32 = 12.0;
/// Around the progress pill's text.
const PILL_PAD: f32 = 16.0;
/// Adopt and Ignore at the end of an unadopted folder's row.
const FOLDER_BUTTONS: f32 = 150.0;
/// The most of the pane's height the legend takes before it scrolls, and the room above it.
const LEGEND_AT_MOST: f32 = 0.45;
const LEGEND_TOP: i8 = 8;

/// The columns of a row, left to right; the title takes what the others leave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Column {
    Expander,
    Pin,
    Name,
    State,
    Next,
    Title,
    Progress,
    Items,
    Stack,
    Hosting,
    Sync,
    Age,
    Health,
    Backup,
    Usage,
    Session,
    Autopilot,
    Start,
}

impl Column {
    const ALL: [Column; 18] = [
        Column::Expander,
        Column::Pin,
        Column::Name,
        Column::State,
        Column::Next,
        Column::Title,
        Column::Progress,
        Column::Items,
        Column::Stack,
        Column::Hosting,
        Column::Sync,
        Column::Age,
        Column::Health,
        Column::Backup,
        Column::Usage,
        Column::Session,
        Column::Autopilot,
        Column::Start,
    ];
    /// Before the optional columns, always.
    const LEFT: [Column; 6] = [
        Column::Expander,
        Column::Pin,
        Column::Name,
        Column::State,
        Column::Next,
        Column::Title,
    ];
    /// After them, always.
    const RIGHT: [Column; 3] = [Column::Session, Column::Autopilot, Column::Start];
    /// The optional columns, from the most a row shows to the least: the badges in full (the
    /// stack chip and the age as well), compact, then the counts go, then the badges, then the
    /// progress (D-25, EP-7 addendum). The week's usage share is a badge of both forms (EP-18b).
    const GIVE_WAY: [&'static [Column]; 5] = [
        &[
            Column::Progress,
            Column::Items,
            Column::Stack,
            Column::Hosting,
            Column::Sync,
            Column::Age,
            Column::Health,
            Column::Backup,
            Column::Usage,
        ],
        &[
            Column::Progress,
            Column::Items,
            Column::Hosting,
            Column::Sync,
            Column::Health,
            Column::Backup,
            Column::Usage,
        ],
        &[
            Column::Progress,
            Column::Hosting,
            Column::Sync,
            Column::Health,
            Column::Backup,
            Column::Usage,
        ],
        &[Column::Progress],
        &[],
    ];

    /// The column's name in the header row and what it holds; the expander, the pin and the two
    /// switches go unnamed.
    fn header(self) -> Option<(&'static str, &'static str)> {
        Some(match self {
            Column::Name => (
                "Project",
                "The registry's name; a right click pins it or sets its priority or state",
            ),
            Column::State => ("State", "Set by hand (D-6); a bulb marks a proposed change"),
            Column::Next => (
                "Next",
                "The next brief in table order and its size: S about 30 minutes, M about an \
                 hour, L about two hours",
            ),
            Column::Title => ("Brief", "The next brief's one-line summary"),
            Column::Progress => ("Done", "Rows done of all rows"),
            Column::Items => (
                "Waiting",
                "Open owner tasks (the older forms' open items where there is no owner-tasks.md), \
                 and open inbox messages",
            ),
            Column::Stack => ("Stack", "The first stack found, and how many more"),
            Column::Hosting => ("Hosting", "Where the repository lives"),
            Column::Sync => ("Sync", "The clone against its origin"),
            Column::Age => ("Age", "The last commit's age; a dot for a dirty tree"),
            Column::Health => (
                "CI",
                "The latest GitHub Actions run; a gavel for a public repository without a \
                 license",
            ),
            Column::Backup => ("Backup", "The Documents job's last archive"),
            Column::Usage => (
                "Usage",
                "This week's share of the usage: the points of the weekly limit the project's \
                 sessions took since the reset",
            ),
            Column::Session => ("Session", "A session in progress, or git not asked yet"),
            Column::Expander | Column::Pin | Column::Autopilot | Column::Start => return None,
        })
    }

    /// The gap after the column at its least: the expander, the pin and the name sit close.
    fn gap(self) -> f32 {
        if self.spreads() { GAP } else { TIGHT }
    }

    /// Whether the gap after the column widens on a wide row: every gap from the name's on.
    fn spreads(self) -> bool {
        !matches!(self, Column::Expander | Column::Pin)
    }
}

/// How wide each column is but the title: the fixed ones, and for the others the widest value
/// the portfolio holds, at least its header's name.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Widths([f32; Column::ALL.len()]);

impl Widths {
    fn of(&self, column: Column) -> f32 {
        self.0[column as usize]
    }

    fn widen(&mut self, column: Column, width: f32) {
        let slot = &mut self.0[column as usize];
        *slot = slot.max(width);
    }

    /// Widens `column` to what `add` draws.
    fn fit(&mut self, ui: &mut egui::Ui, column: Column, add: impl FnOnce(&mut egui::Ui)) {
        let width = style::width_of(ui, add);
        self.widen(column, width);
    }

    /// Every project's and unadopted folder's cells laid out by the code that draws them, so no
    /// column clips its widest value; the rows folded away or filtered out count too, so the
    /// columns stay put while the owner folds, filters and selects.
    fn measure(ui: &mut egui::Ui, portfolio: &Portfolio, now: DateTime<Utc>) -> Self {
        let mut widths = Self([0.0; Column::ALL.len()]);
        for (column, width) in [
            (Column::Expander, EXPANDER),
            (Column::Pin, PIN),
            (Column::Autopilot, AUTOPILOT),
            (Column::Start, START),
        ] {
            widths.widen(column, width);
        }
        for column in Column::ALL {
            if let Some((name, _)) = column.header() {
                widths.fit(ui, column, |ui| {
                    header_label(ui, name);
                });
            }
        }
        for row in &portfolio.projects {
            widths.fit(ui, Column::Name, |ui| name_label(ui, row.name()));
            widths.fit(ui, Column::State, |ui| state_cell(ui, row, now));
            widths.fit(ui, Column::Usage, |ui| usage_cell(ui, row));
            widths.fit(ui, Column::Session, |ui| session_cell(ui, row, None));
            if row.folder.is_none() {
                continue;
            }
            if let Some(next) = row.next() {
                widths.fit(ui, Column::Next, |ui| next_cell(ui, next));
            }
            let counts = row.counts();
            if counts.total > 0 {
                let text = style::width_of(ui, |ui| {
                    ui.label(progress_text(counts));
                });
                widths.widen(Column::Progress, text + PILL_PAD);
            }
            widths.fit(ui, Column::Items, |ui| counts_cell(ui, row));
            widths.fit(ui, Column::Stack, |ui| stack_cell(ui, row));
            widths.fit(ui, Column::Hosting, |ui| hosting_cell(ui, row, now));
            widths.fit(ui, Column::Sync, |ui| sync_cell(ui, row, now));
            widths.fit(ui, Column::Age, |ui| age_cell(ui, row, now));
            widths.fit(ui, Column::Health, |ui| health_cell(ui, row, now));
            widths.fit(ui, Column::Backup, |ui| backup_cell(ui, row, now));
        }
        for folder in &portfolio.unadopted {
            widths.fit(ui, Column::Name, |ui| name_label(ui, &folder.name));
            widths.fit(ui, Column::State, unadopted_chip);
        }
        widths
    }
}

/// Where each column of a row sits: the measured widths left to right with the gaps between
/// them. The optional columns give way while the title would have less than its least; then the
/// gaps after the name share part of the spare width, up to a cap, and the title takes the rest,
/// so the row is filled to its right edge. One plan serves the header row and every row.
#[derive(Clone, Debug, PartialEq)]
struct Plan {
    /// Each column shown: its left edge from the row's left, and its width.
    cells: Vec<(Column, f32, f32)>,
}

impl Plan {
    fn new(widths: &Widths, width: f32) -> Self {
        let columns_with = |optional: &[Column]| -> Vec<Column> {
            Column::LEFT
                .iter()
                .chain(optional)
                .chain(&Column::RIGHT)
                .copied()
                .collect()
        };
        let columns = Column::GIVE_WAY
            .iter()
            .enumerate()
            .map(|(choice, optional)| (choice, columns_with(optional)))
            .find(|(choice, columns)| {
                width - Self::least(widths, columns) >= Self::title_least(*choice)
            })
            .map(|(_, columns)| columns)
            .unwrap_or_else(|| columns_with(&[]));
        let room = width - Self::least(widths, &columns);
        let spreading = columns
            .iter()
            .take(columns.len().saturating_sub(1))
            .filter(|column| column.spreads())
            .count() as f32;
        let extra = if spreading > 0.0 {
            ((room - TITLE_AT_LEAST).max(0.0) * SPREAD_SHARE / spreading).min(SPREAD)
        } else {
            0.0
        };
        let title = (room - extra * spreading).max(TITLE_MIN);
        let mut x = 0.0;
        let cells = columns
            .iter()
            .map(|&column| {
                let width = match column {
                    Column::Title => title,
                    column => widths.of(column),
                };
                let cell = (column, x, width);
                x += width + column.gap();
                if column.spreads() {
                    x += extra;
                }
                cell
            })
            .collect();
        Self { cells }
    }

    /// The title's least with the `choice`th of [`Column::GIVE_WAY`]: the full form keeps
    /// [`TITLE_AT_LEAST`], and every narrower one lets the title give way to [`TITLE_FLOOR`]
    /// before the next column leaves (EP-38).
    fn title_least(choice: usize) -> f32 {
        if choice == 0 {
            TITLE_AT_LEAST
        } else {
            TITLE_FLOOR
        }
    }

    /// What a row of `columns` takes besides its title: their widths and the gaps between them
    /// at their least.
    fn least(widths: &Widths, columns: &[Column]) -> f32 {
        let cells: f32 = columns
            .iter()
            .filter(|&&column| column != Column::Title)
            .map(|&column| widths.of(column))
            .sum();
        let gaps: f32 = columns
            .iter()
            .take(columns.len().saturating_sub(1))
            .map(|column| column.gap())
            .sum();
        cells + gaps
    }

    /// The column's left edge from the row's left and its width, when the row shows it.
    fn span(&self, column: Column) -> Option<(f32, f32)> {
        self.cells
            .iter()
            .find(|cell| cell.0 == column)
            .map(|&(_, x, width)| (x, width))
    }

    /// The column's cell in `row`.
    fn rect(&self, column: Column, row: Rect) -> Option<Rect> {
        self.span(column).map(|(x, width)| {
            Rect::from_min_size(
                egui::pos2(row.left() + x, row.top()),
                vec2(width, row.height()),
            )
        })
    }

    /// Draws `add` in the column's cell of `row`, when the row shows the column.
    fn draw(&self, ui: &mut egui::Ui, row: Rect, column: Column, add: impl FnOnce(&mut egui::Ui)) {
        if let Some(rect) = self.rect(column, row) {
            cell(ui, rect, add);
        }
    }
}

/// Draws `add` inside `rect`, left to right and centred on the row's middle line. Its labels are
/// not selectable, so they sense hover only: the row under them takes the click, the right click
/// and the hover, and each label keeps its hover text (EP-15a). A selectable label senses click
/// and drag, and egui gives the pointer to the widget on top.
fn cell<R>(ui: &mut egui::Ui, rect: Rect, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    style::unselectable(&mut child);
    add(&mut child)
}

impl Board {
    pub fn projects_pane(&mut self, ui: &mut egui::Ui) {
        self.list_toolbar(ui);
        ui.separator();
        if self.view.legend {
            // As tall as the key, up to a share of the pane; a scroll area inside fills only the
            // height its panel has, so the height is measured first.
            let key = style::height_of(ui, ui.available_width(), badges::legend);
            let height = (key + f32::from(LEGEND_TOP)).min(ui.available_height() * LEGEND_AT_MOST);
            egui::Panel::bottom("legend")
                .exact_size(height)
                .frame(egui::Frame::new().inner_margin(egui::Margin {
                    top: LEGEND_TOP,
                    ..egui::Margin::ZERO
                }))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, badges::legend);
                });
        }
        if let Some(problem) = &self.portfolio.problem {
            ui.label(RichText::new(problem).color(style::red(ui)));
            return;
        }
        let groups = self
            .portfolio
            .groups(self.view.sort, self.view.list_filter());
        if groups.is_empty() {
            let text = if self.loader.reading_files {
                "Reading the registry and the projects…"
            } else if !self.view.stacks.is_empty() || !self.view.hosting.is_empty() {
                "No project passes the filters."
            } else if self.view.filter.trim().is_empty() {
                "No projects: the registry lists none and the data root holds no folder."
            } else {
                "No project's name contains the filter."
            };
            ui.label(RichText::new(text).weak());
            return;
        }
        let now = chrono::Utc::now();
        let widths = Widths::measure(ui, &self.portfolio, now);
        let plan = Plan::new(&widths, ui.available_width());
        header_row(ui, &plan);
        ui.separator();
        let mut actions = Vec::new();
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                // Once for the list, so that every cell shares the style (EP-15a).
                style::unselectable(ui);
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
                            Member::Project(i) => {
                                let row = &self.portfolio.projects[i];
                                let live = self.session_summary(row.name());
                                actions.extend(project_row(
                                    ui,
                                    row,
                                    &self.view,
                                    &plan,
                                    self.scroll_to_selected,
                                    live.as_deref(),
                                    now,
                                ));
                            }
                            Member::Folder(i) => {
                                let folder = &self.portfolio.unadopted[i];
                                actions.extend(folder_row(
                                    ui,
                                    &folder.name,
                                    folder.git,
                                    folder.roadmap,
                                    &self.view,
                                    &plan,
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
                        ui.radio_value(&mut self.view.sort, key, key.label())
                            .on_hover_text(key.hint());
                    }
                },
            );
            let chosen = self.view.stacks.len() + self.view.hosting.len();
            let label = match chosen {
                0 => format!("{} Filters", icons::ICON_FILTER_LIST.codepoint),
                n => format!("{} Filters ({n})", icons::ICON_FILTER_LIST.codepoint),
            };
            ui.menu_button(label, |ui| {
                ui.label(RichText::new("Stack: any of").weak());
                for kind in Kind::ALL {
                    let mut on = self.view.stacks.contains(&kind);
                    if ui.checkbox(&mut on, kind.label()).changed() {
                        if on {
                            self.view.stacks.insert(kind);
                        } else {
                            self.view.stacks.remove(&kind);
                        }
                    }
                }
                ui.separator();
                ui.label(RichText::new("Hosting: any of").weak());
                for class in Class::ALL {
                    let mut on = self.view.hosting.contains(&class);
                    if ui.checkbox(&mut on, class.label()).changed() {
                        if on {
                            self.view.hosting.insert(class);
                        } else {
                            self.view.hosting.remove(&class);
                        }
                    }
                }
                ui.separator();
                if ui
                    .add_enabled(chosen > 0, egui::Button::new("Clear the filters"))
                    .clicked()
                {
                    self.view.stacks.clear();
                    self.view.hosting.clear();
                }
                ui.label(
                    RichText::new("Unadopted folders show only while no filter is chosen.")
                        .small()
                        .weak(),
                );
            });
            ui.separator();
            ui.toggle_value(
                &mut self.view.legend,
                format!("{} Legend", icons::ICON_LEGEND_TOGGLE.codepoint),
            )
            .on_hover_text("Show or hide the key to every mark of the rows");
        });
    }
}

/// The header row (EP-7a): each shown column's name over it, small and weak, with what the
/// column holds as hover text.
fn header_row(ui: &mut egui::Ui, plan: &Plan) {
    let height = ui.text_style_height(&egui::TextStyle::Small) + 4.0;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    for &(column, ..) in &plan.cells {
        if let Some((name, hover)) = column.header() {
            plan.draw(ui, rect, column, |ui| {
                header_label(ui, name).on_hover_text(hover);
            });
        }
    }
}

fn header_label(ui: &mut egui::Ui, name: &str) -> egui::Response {
    ui.label(RichText::new(name).small().weak())
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

/// One project's row, and below it the inline expansion when it is open; `scroll` brings it
/// into view, and `live` is what the window's own sessions say of the project.
fn project_row(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    view: &View,
    plan: &Plan,
    scroll: bool,
    live: Option<&str>,
    now: DateTime<Utc>,
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
    let weak = ui.visuals().weak_text_color();

    plan.draw(ui, rect, Column::Expander, |ui| {
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
        plan.draw(ui, rect, Column::Pin, |ui| {
            PINNED.show(ui, "Pinned: first in its group");
        });
    }
    plan.draw(ui, rect, Column::Name, |ui| name_label(ui, name));
    plan.draw(ui, rect, Column::State, |ui| state_cell(ui, row, now));
    let counts = row.counts();
    match (&row.folder, row.next()) {
        (None, _) => {
            plan.draw(ui, rect, Column::Title, |ui| {
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
            plan.draw(ui, rect, Column::Next, |ui| next_cell(ui, next));
            plan.draw(ui, rect, Column::Title, |ui| {
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
            if let (Some(next), Some(title)) = (
                plan.rect(Column::Next, rect),
                plan.rect(Column::Title, rect),
            ) {
                cell(ui, next.union(title), |ui| {
                    ui.add(egui::Label::new(RichText::new(text).weak()).truncate());
                });
            }
        }
    }
    if let Some(progress) = plan.rect(Column::Progress, rect)
        && counts.total > 0
    {
        progress_cell(ui, progress, counts);
    }
    plan.draw(ui, rect, Column::Items, |ui| counts_cell(ui, row));
    if row.folder.is_some() {
        plan.draw(ui, rect, Column::Stack, |ui| stack_cell(ui, row));
        plan.draw(ui, rect, Column::Hosting, |ui| hosting_cell(ui, row, now));
        plan.draw(ui, rect, Column::Sync, |ui| sync_cell(ui, row, now));
        plan.draw(ui, rect, Column::Age, |ui| age_cell(ui, row, now));
        plan.draw(ui, rect, Column::Health, |ui| health_cell(ui, row, now));
        plan.draw(ui, rect, Column::Backup, |ui| backup_cell(ui, row, now));
    }
    // The ledger's figures need no folder.
    plan.draw(ui, rect, Column::Usage, |ui| usage_cell(ui, row));
    plan.draw(ui, rect, Column::Session, |ui| session_cell(ui, row, live));
    plan.draw(ui, rect, Column::Autopilot, |ui| {
        let on = row.project.autopilot.enabled;
        let mut checked = on;
        let hover = if on {
            format!(
                "Autopilot on (A): its runs commit themselves, and after a headless run \
                 completes the gate may chain its next brief, {} at most, {} a day; the Sessions \
                 pane's All tab logs each step",
                row.project.autopilot.max_size(),
                row.project.autopilot.daily()
            )
        } else {
            "Autopilot off (A): its runs stop before committing; turn it on to let a headless \
             run's completion chain the next brief and to say yes to its commits (D-22)"
                .to_owned()
        };
        if ui
            .add(Checkbox::without_text(&mut checked))
            .on_hover_text(hover)
            .changed()
        {
            actions.push(Action::SetAutopilot(name.to_owned(), !on));
        }
    });
    plan.draw(ui, rect, Column::Start, |ui| {
        let next = row.folder.as_ref().and(row.next());
        let button = ui.add_enabled(
            next.is_some(),
            egui::Button::new(style::icon(icons::ICON_PLAY_ARROW, None)),
        );
        match next {
            Some(next) => {
                if button
                    .on_hover_text(format!("Start {}: the launch dialog (S)", next.id))
                    .clicked()
                {
                    actions.push(Action::OpenLaunch(name.to_owned()));
                }
            }
            None => {
                button.on_disabled_hover_text("Nothing to start: the project has no next brief");
            }
        }
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
        // This project alone read again (EP-25a): what F5 does for every project.
        ui.separator();
        if ui
            .add_enabled(
                row.folder.is_some(),
                egui::Button::new("Refresh this project"),
            )
            .on_hover_text(
                "Read this project's files again, then git and its history; the other rows \
                 stay as they are (F5 reads every project)",
            )
            .on_disabled_hover_text("The project's folder is missing")
            .clicked()
        {
            actions.push(Action::RefreshProject(name.to_owned()));
            ui.close();
        }
        // The bulb's proposals, each dismissed until the fact it rests on changes (D-6).
        for proposal in row.proposals(now).into_iter().filter(|p| !p.dismissed) {
            if ui
                .button(format!("Dismiss: {}", proposal.reason))
                .on_hover_text(format!(
                    "Leave the proposal out until {}",
                    proposal.reason.returns_when()
                ))
                .clicked()
            {
                actions.push(Action::Dismiss(name.to_owned(), proposal.reason));
                ui.close();
            }
        }
        // A message from the owner into the project's inbox (D-13; EP-20c).
        ui.separator();
        if ui
            .add_enabled(row.folder.is_some(), egui::Button::new("Send a message…"))
            .on_hover_text(
                "Write a message from you into the project's roadmap/inbox.md, for its next \
                 session to triage (D-13)",
            )
            .on_disabled_hover_text("The project's folder is missing")
            .clicked()
        {
            actions.push(Action::OpenCompose(name.to_owned()));
            ui.close();
        }
        // The house-style files a project lacks, behind their preview, and the migration of an
        // older roadmap (D-9, D-11; EP-21a): for a project that needs one or the other.
        let house = &row.house;
        let formalise = house.wants() || row.project.state == State::Unformalised;
        if row.folder.is_some() && (formalise || house.migrates.is_some()) {
            ui.separator();
            if formalise
                && ui
                    .button("Formalise…")
                    .on_hover_text(house_hint(house))
                    .clicked()
            {
                actions.push(Action::OpenFormalise(name.to_owned()));
                ui.close();
            }
            if house.migrates.is_some()
                && ui
                    .button("Migrate the roadmap…")
                    .on_hover_text(crate::formalise::MIGRATE_HINT)
                    .clicked()
            {
                actions.push(Action::OpenMigration(name.to_owned()));
                ui.close();
            }
        }
        // The project's latest session undone, behind its preview (D-11; EP-15c).
        ui.separator();
        let available = crate::revert::availability(Some(row), name);
        let item = ui.add_enabled(
            available.is_ok(),
            egui::Button::new("Revert the latest session…"),
        );
        match &available {
            Ok(()) => {
                if item
                    .on_hover_text(
                        "Preview undoing the project's latest session in the ledger: the commits \
                         and files it would undo, then Revert or Cancel",
                    )
                    .clicked()
                {
                    actions.push(Action::OpenRevert {
                        project: name.to_owned(),
                        session: None,
                    });
                    ui.close();
                }
            }
            Err(why) => {
                item.on_disabled_hover_text(why);
            }
        }
    });

    if expanded {
        let indent = plan.span(Column::Name).map_or(0.0, |(x, _)| x);
        expansion(ui, row, indent);
    }
    actions
}

/// What Formalise… would do, on hover (EP-21a).
fn house_hint(house: &crate::portfolio::House) -> String {
    let what = if house.writes.is_empty() {
        "Preview the house-style files: the project has each one".to_owned()
    } else {
        format!(
            "Preview writing the house-style files it lacks: {}",
            house.writes_words()
        )
    };
    match house.refused.first() {
        Some(why) => format!("{what}. Refused: {why}"),
        None => format!("{what}, then Write or Cancel (D-11)"),
    }
}

fn name_label(ui: &mut egui::Ui, name: &str) {
    ui.add(egui::Label::new(RichText::new(name).strong()).truncate());
}

/// The state chip, and the bulb of a proposed change the owner has not dismissed.
fn state_cell(ui: &mut egui::Ui, row: &ProjectRow, now: DateTime<Utc>) {
    let state = row.project.state;
    style::chip(ui, state.as_str(), style::state_colour(Some(state)));
    if row.proposes(now) {
        PROPOSAL.show(
            ui,
            "A state change is proposed: see the Detail pane, or dismiss it from the row's menu",
        );
    }
}

fn unadopted_chip(ui: &mut egui::Ui) {
    style::chip(ui, "unadopted", style::state_colour(None));
}

/// The next EP's id and its size.
fn next_cell(ui: &mut egui::Ui, next: &Row) {
    ui.label(RichText::new(next.id.to_string()).monospace());
    if let Some(size) = next.size {
        ui.label(RichText::new(size.as_str()).small().strong())
            .on_hover_text(size_hint(size));
    }
}

fn progress_text(counts: Counts) -> RichText {
    RichText::new(format!("{}/{}", counts.done, counts.total)).small()
}

/// Done over total on a small pill filled to the fraction done, the percent on hover (D-25).
fn progress_cell(ui: &mut egui::Ui, rect: Rect, counts: Counts) {
    let fraction = counts.done as f32 / counts.total as f32;
    let pill = Rect::from_center_size(
        rect.center(),
        vec2(rect.width() - 4.0, (rect.height() - 8.0).min(18.0)),
    );
    let painter = ui.painter_at(rect);
    painter.rect_filled(pill, 5.0, ui.visuals().faint_bg_color);
    painter.rect_filled(
        Rect::from_min_size(pill.min, vec2(pill.width() * fraction, pill.height())),
        5.0,
        style::green(ui).gamma_multiply(0.3),
    );
    // Hover only, so that the row takes the click (EP-15a).
    let response = ui.put(
        pill,
        egui::Label::new(progress_text(counts)).selectable(false),
    );
    response.on_hover_text(format!(
        "{} of {} rows done ({:.0} %){}",
        counts.done,
        counts.total,
        fraction * 100.0,
        match counts.stretch {
            0 => String::new(),
            n => format!(", {n} stretch"),
        }
    ));
}

/// What waits on the owner: open owner items, open inbox messages (red when one is urgent).
fn counts_cell(ui: &mut egui::Ui, row: &ProjectRow) {
    // Each icon close to its number, the two pairs apart.
    ui.spacing_mut().item_spacing.x = 3.0;
    let items = row.open_items();
    if items > 0 {
        OWNER_ITEMS.show(ui, &format!("{items} open owner items"));
        ui.label(items.to_string());
        ui.add_space(6.0);
    }
    let (messages, urgent) = row.open_messages();
    if messages > 0 {
        match urgent {
            0 => MESSAGES.show(ui, &format!("{messages} open inbox messages")),
            n => URGENT.show(ui, &format!("{messages} open inbox messages, {n} urgent")),
        };
        ui.label(messages.to_string());
    }
}

/// The first stack chip (`RS+1` when there are more), every chip on hover.
fn stack_cell(ui: &mut egui::Ui, row: &ProjectRow) {
    let Some(first) = row.badges.stack.first() else {
        return;
    };
    let hover = row
        .badges
        .stack
        .iter()
        .map(|chip| format!("{} ({})", chip.text(), chip.source))
        .collect::<Vec<_>>()
        .join("\n");
    let text = match row.badges.stack.len() - 1 {
        0 => first.kind.short().to_owned(),
        n => format!("{}+{n}", first.kind.short()),
    };
    style::chip(ui, &text, style::stack_colour(first.kind)).on_hover_text(hover);
}

fn hosting_cell(ui: &mut egui::Ui, row: &ProjectRow, now: DateTime<Utc>) {
    badges::hosting(row, now).show(ui);
}

fn sync_cell(ui: &mut egui::Ui, row: &ProjectRow, now: DateTime<Utc>) {
    if let Some(mark) = badges::sync(row, now) {
        mark.show(ui);
    }
}

/// The last commit's age, and a small dot for a dirty tree.
fn age_cell(ui: &mut egui::Ui, row: &ProjectRow, now: DateTime<Utc>) {
    let Some((age, dirty)) = badges::activity(row, now) else {
        return;
    };
    ui.spacing_mut().item_spacing.x = 3.0;
    age.show(ui);
    if let Some(dirty) = dirty {
        ui.label(dirty.rich(ui).small()).on_hover_text(&dirty.hover);
    }
}

/// The CI run, and the license warning for a public repository.
fn health_cell(ui: &mut egui::Ui, row: &ProjectRow, now: DateTime<Utc>) {
    ui.spacing_mut().item_spacing.x = 2.0;
    if let Some(mark) = badges::ci(row, now) {
        mark.show(ui);
    }
    if let Some(mark) = badges::license_warning(row) {
        mark.show(ui);
    }
}

fn backup_cell(ui: &mut egui::Ui, row: &ProjectRow, now: DateTime<Utc>) {
    if let Some(mark) = badges::backup(row, now) {
        mark.show(ui);
    }
}

/// This week's share of the usage (EP-18b).
fn usage_cell(ui: &mut egui::Ui, row: &ProjectRow) {
    if let Some(mark) = badges::usage(row) {
        mark.show(ui);
    }
}

/// A session in progress, by the files' evidence (D-9) or because one runs (`live`, the sessions'
/// own words: the window's headless runs and those Claude Code reports, EP-12); else a
/// repository git has not been asked about yet.
fn session_cell(ui: &mut egui::Ui, row: &ProjectRow, live: Option<&str>) {
    let texts: Vec<String> = live
        .map(str::to_owned)
        .into_iter()
        .chain(row.in_progress().iter().map(ToString::to_string))
        .collect();
    if !texts.is_empty() {
        SESSION.show(ui, &format!("Session in progress: {}", texts.join("; ")));
    } else if row.awaits_git() {
        GIT_PENDING.show(ui, "Git has not been asked yet");
    }
}

fn size_hint(size: crw_core::roadmap::Size) -> &'static str {
    match size {
        crw_core::roadmap::Size::S => "S: about 30 minutes of one session",
        crw_core::roadmap::Size::M => "M: about an hour",
        crw_core::roadmap::Size::L => "L: about two hours",
    }
}

/// The inline expansion under the name: each phase's progress, with the graphs' placeholder
/// (EP-26).
fn expansion(ui: &mut egui::Ui, row: &ProjectRow, indent: f32) {
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
    plan: &Plan,
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
    plan.draw(ui, rect, Column::Pin, |ui| {
        FOLDER.show(ui, "A folder of the data root, not adopted yet");
    });
    plan.draw(ui, rect, Column::Name, |ui| name_label(ui, name));
    plan.draw(ui, rect, Column::State, unadopted_chip);
    // The two buttons keep their room at the row's end; the description takes the rest.
    let buttons = Rect::from_min_max(
        egui::pos2(rect.right() - FOLDER_BUTTONS, rect.top()),
        rect.right_bottom(),
    );
    let from = plan
        .rect(Column::Next, rect)
        .map_or(rect.left(), |next| next.left());
    let what = Rect::from_min_max(
        egui::pos2(from, rect.top()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::testing::{self, Screen};

    /// harbor's row drawn at the top of a pass of `screen` with `events`: its actions, its rect,
    /// its name's cell, and what the pass painted.
    fn harbor_pass(
        screen: &mut Screen,
        portfolio: &Portfolio,
        events: Vec<egui::Event>,
    ) -> (Vec<Action>, Rect, Rect, egui::FullOutput) {
        harbor_cell_pass(screen, portfolio, events, Column::Name)
    }

    /// The same with the cell of `column` in place of the name's.
    fn harbor_cell_pass(
        screen: &mut Screen,
        portfolio: &Portfolio,
        events: Vec<egui::Event>,
        column: Column,
    ) -> (Vec<Action>, Rect, Rect, egui::FullOutput) {
        let now = Utc::now();
        let view = View::default();
        let mut found = (Vec::new(), Rect::NOTHING, Rect::NOTHING);
        let output = screen.pass(events, |ui| {
            let widths = Widths::measure(ui, portfolio, now);
            let plan = Plan::new(&widths, ui.available_width());
            let rect = Rect::from_min_size(
                ui.cursor().min,
                vec2(ui.available_width(), view.density.row_height()),
            );
            let row = portfolio.project("harbor").unwrap();
            let actions = project_row(ui, row, &view, &plan, false, None, now);
            found = (actions, rect, plan.rect(column, rect).unwrap());
        });
        (found.0, found.1, found.2, output)
    }

    #[test]
    fn the_autopilot_box_turns_the_projects_autopilot_on() {
        let scratch = Scratch::new("list-autopilot-box");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        assert!(
            !portfolio
                .project("harbor")
                .unwrap()
                .project
                .autopilot
                .enabled
        );
        let mut screen = Screen::new();
        let pass = |screen: &mut Screen, events| {
            harbor_cell_pass(screen, &portfolio, events, Column::Autopilot)
        };
        let (_, _, cell, _) = pass(&mut screen, Vec::new());
        let at = cell.center();
        let press = |pressed| testing::button(at, egui::PointerButton::Primary, pressed);
        pass(&mut screen, testing::moved(at));
        pass(&mut screen, press(true));
        let (actions, ..) = pass(&mut screen, press(false));
        assert_eq!(
            actions,
            [Action::SetAutopilot("harbor".to_owned(), true)],
            "the box answers, and the row does not take its click"
        );
    }

    #[test]
    fn the_whole_row_answers_over_its_text() {
        let scratch = Scratch::new("list-row-clicks");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let mut screen = Screen::new();
        let (_, _, name, _) = harbor_pass(&mut screen, &portfolio, Vec::new());
        // Over the name's first letters, which a selectable label would take.
        let at = name.left_center() + vec2(8.0, 0.0);
        let press = |button, pressed| testing::button(at, button, pressed);
        harbor_pass(&mut screen, &portfolio, testing::moved(at));
        let (_, rect, _, output) = harbor_pass(&mut screen, &portfolio, Vec::new());
        let hovered = screen
            .ctx
            .global_style()
            .visuals
            .widgets
            .hovered
            .weak_bg_fill;
        assert!(
            testing::fills(&output, rect, hovered),
            "the row is drawn hovered"
        );

        harbor_pass(
            &mut screen,
            &portfolio,
            press(egui::PointerButton::Primary, true),
        );
        let (actions, ..) = harbor_pass(
            &mut screen,
            &portfolio,
            press(egui::PointerButton::Primary, false),
        );
        assert_eq!(
            actions,
            [Action::Select(Selection::Project("harbor".to_owned()))]
        );

        // A right click over the name opens the row's menu.
        harbor_pass(
            &mut screen,
            &portfolio,
            press(egui::PointerButton::Secondary, true),
        );
        harbor_pass(
            &mut screen,
            &portfolio,
            press(egui::PointerButton::Secondary, false),
        );
        let (.., output) = harbor_pass(&mut screen, &portfolio, Vec::new());
        assert!(
            egui::Popup::is_any_open(&screen.ctx),
            "the row's menu is open"
        );
        // Its Refresh this project (EP-25a) asks for the row's read on its own.
        let item = testing::text_at(&output, "Refresh this project").expect("the menu item");
        harbor_pass(&mut screen, &portfolio, testing::moved(item));
        harbor_pass(
            &mut screen,
            &portfolio,
            testing::button(item, egui::PointerButton::Primary, true),
        );
        let (actions, ..) = harbor_pass(
            &mut screen,
            &portfolio,
            testing::button(item, egui::PointerButton::Primary, false),
        );
        assert_eq!(actions, [Action::RefreshProject("harbor".to_owned())]);
    }

    /// Every measured column `width` wide, the fixed ones as they are.
    fn widths(width: f32) -> Widths {
        let mut widths = Widths([width; Column::ALL.len()]);
        for (column, fixed) in [
            (Column::Expander, EXPANDER),
            (Column::Pin, PIN),
            (Column::Autopilot, AUTOPILOT),
            (Column::Start, START),
        ] {
            widths.0[column as usize] = fixed;
        }
        widths
    }

    fn columns_with(optional: &[Column]) -> Vec<Column> {
        Column::LEFT
            .iter()
            .chain(optional)
            .chain(&Column::RIGHT)
            .copied()
            .collect()
    }

    fn shown(plan: &Plan) -> Vec<Column> {
        plan.cells.iter().map(|cell| cell.0).collect()
    }

    /// No cell narrower than its column's widest value; the title at least its floor; the cells
    /// in order without overlap.
    fn check(plan: &Plan, widths: &Widths) {
        let mut right = 0.0;
        for &(column, x, width) in &plan.cells {
            assert!(x >= right, "{column:?} overlaps the cell before it");
            match column {
                Column::Title => assert!(width >= TITLE_MIN),
                column => assert!(width >= widths.of(column), "{column:?} is clipped"),
            }
            right = x + width;
        }
    }

    fn right_edge(plan: &Plan) -> f32 {
        plan.cells.last().map_or(0.0, |&(_, x, width)| x + width)
    }

    #[test]
    fn a_wide_row_shows_every_column_and_loosens_its_gaps() {
        let widths = widths(30.0);
        let plan = Plan::new(&widths, 3000.0);
        check(&plan, &widths);
        assert_eq!(shown(&plan), Column::ALL);
        assert!((right_edge(&plan) - 3000.0).abs() < 0.01);
        // So much spare width that every gap after the name widens by the cap.
        let (state, _) = plan.span(Column::State).unwrap();
        let (name, name_width) = plan.span(Column::Name).unwrap();
        assert!((state - (name + name_width) - (GAP + SPREAD)).abs() < 0.01);
        // The expander and the pin stay close to the name.
        let (pin, _) = plan.span(Column::Pin).unwrap();
        assert!((name - (pin + PIN) - TIGHT).abs() < 0.01);
    }

    #[test]
    fn the_spare_width_is_shared_and_fills_the_row() {
        let widths = widths(30.0);
        let full = columns_with(Column::GIVE_WAY[0]);
        let least = Plan::least(&widths, &full);
        // A row with the title's least and 100 points to spare: the gaps take 40 of them.
        let plan = Plan::new(&widths, least + TITLE_AT_LEAST + 100.0);
        check(&plan, &widths);
        assert_eq!(shown(&plan), full);
        let (_, title) = plan.span(Column::Title).unwrap();
        assert!((title - (TITLE_AT_LEAST + 100.0 * (1.0 - SPREAD_SHARE))).abs() < 0.01);
        assert!((right_edge(&plan) - (least + TITLE_AT_LEAST + 100.0)).abs() < 0.01);
        // At the title's least exactly, the gaps keep theirs.
        let tight = Plan::new(&widths, least + TITLE_AT_LEAST);
        let (_, title) = tight.span(Column::Title).unwrap();
        assert!((title - TITLE_AT_LEAST).abs() < 0.01);
    }

    /// The title gives way before a column does (D-25 addendum of 2026-11-11): the badges
    /// compact first, then the title shrinks from its least to the floor while the compact
    /// badges and the counts stay, and only under the floor do the counts leave, then the
    /// badges, each time with the title at the floor again first.
    #[test]
    fn the_title_gives_way_before_the_counts_and_the_compact_badges() {
        let widths = widths(30.0);
        let full = columns_with(Column::GIVE_WAY[0]);
        let compact = columns_with(Column::GIVE_WAY[1]);
        let fewer = columns_with(Column::GIVE_WAY[2]);
        // A point short of the full form's least: the badges compact, the title keeps its least.
        let plan = Plan::new(&widths, Plan::least(&widths, &full) + TITLE_AT_LEAST - 1.0);
        check(&plan, &widths);
        assert_eq!(shown(&plan), compact);
        assert!(plan.span(Column::Title).unwrap().1 >= TITLE_AT_LEAST);
        // From there down to the floor the title alone gives way: the compact badges and the
        // counts stay, the gaps keep their least, and the row is still filled to its edge.
        let least = Plan::least(&widths, &compact);
        for room in [
            TITLE_AT_LEAST - 1.0,
            (TITLE_AT_LEAST + TITLE_FLOOR) / 2.0,
            TITLE_FLOOR,
        ] {
            let plan = Plan::new(&widths, least + room);
            check(&plan, &widths);
            assert_eq!(shown(&plan), compact, "{room}");
            let (_, title) = plan.span(Column::Title).unwrap();
            assert!((title - room).abs() < 0.01, "{room}: {title}");
            assert!((right_edge(&plan) - (least + room)).abs() < 0.01, "{room}");
        }
        // A point under the floor and the counts leave; the title takes the room they free.
        let plan = Plan::new(&widths, least + TITLE_FLOOR - 1.0);
        check(&plan, &widths);
        assert_eq!(shown(&plan), fewer);
        assert!(plan.span(Column::Title).unwrap().1 > TITLE_FLOOR);
        // The compact badges leave only once the title is at the floor again.
        let least = Plan::least(&widths, &fewer);
        assert_eq!(shown(&Plan::new(&widths, least + TITLE_FLOOR)), fewer);
        assert_eq!(
            shown(&Plan::new(&widths, least + TITLE_FLOOR - 1.0)),
            columns_with(Column::GIVE_WAY[3])
        );
    }

    #[test]
    fn a_narrowing_row_gives_way_in_d25s_order() {
        let widths = widths(30.0);
        let fitting = |choice: usize| {
            Plan::least(&widths, &columns_with(Column::GIVE_WAY[choice]))
                + Plan::title_least(choice)
        };
        for choice in 0..Column::GIVE_WAY.len() {
            let plan = Plan::new(&widths, fitting(choice));
            check(&plan, &widths);
            assert_eq!(
                shown(&plan),
                columns_with(Column::GIVE_WAY[choice]),
                "{choice}"
            );
            if choice + 1 < Column::GIVE_WAY.len() {
                // A point less and the next choice takes over.
                let plan = Plan::new(&widths, fitting(choice) - 1.0);
                assert_eq!(shown(&plan), columns_with(Column::GIVE_WAY[choice + 1]));
            }
        }
        // Narrower than the fewest columns allow: the title keeps its floor, nothing is clipped.
        let plan = Plan::new(&widths, 200.0);
        check(&plan, &widths);
        assert_eq!(shown(&plan), columns_with(&[]));
    }

    #[test]
    fn a_wide_value_widens_its_column_alone() {
        let mut widths = widths(30.0);
        widths.widen(Column::Stack, 52.0);
        widths.widen(Column::Stack, 40.0);
        assert_eq!(widths.of(Column::Stack), 52.0);
        let plan = Plan::new(&widths, 3000.0);
        check(&plan, &widths);
        let (_, stack) = plan.span(Column::Stack).unwrap();
        assert_eq!(stack, 52.0);
        let rect = plan
            .rect(
                Column::Stack,
                Rect::from_min_size(egui::pos2(100.0, 10.0), vec2(3000.0, 26.0)),
            )
            .unwrap();
        assert_eq!(rect.width(), 52.0);
        assert_eq!(rect.height(), 26.0);
    }

    #[test]
    fn every_column_but_the_controls_is_named() {
        for column in Column::ALL {
            let named = column.header().is_some();
            let control = matches!(
                column,
                Column::Expander | Column::Pin | Column::Autopilot | Column::Start
            );
            assert_eq!(named, !control, "{column:?}");
        }
    }
}
