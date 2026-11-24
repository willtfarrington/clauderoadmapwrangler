//! The Projects pane (D-25): a toolbar (the name filter, the sort key, the stack and hosting
//! filters, the legend's switch, Expand all and Collapse all, the Cards switch), a header row
//! that names the columns, then the groups by state, each with a header that folds it, and one
//! dense row per project: its next brief's id and size (the one-liner on the id's hover, the
//! Detail pane showing the brief whole; EP-46), its progress, what waits on the owner, its badges
//! (EP-7) with this week's usage share (EP-18b), whether a session is in progress, and the
//! switches. Each column is as wide as the widest value the portfolio holds (EP-7a); a wide
//! window shares its spare width among the gaps, a narrow one drops the optional columns in
//! D-25's order. A row expands inline to its phases and the burn-up sparkline
//! (EP-26a); a right click pins it or sets its priority or its state; an unadopted folder offers
//! Adopt and Ignore. The cards view (EP-26a) draws the same groups as cards, as many to a line as
//! fit, each with the name, the state chip, the next brief, the progress pill, the badges and the
//! sparkline; the whole card takes the click as the row does. The legend, a key to every mark of
//! the row, opens at the foot of the pane.

use chrono::{DateTime, Local, NaiveDate, Utc};
use crw_core::badges::hosting::Class;
use crw_core::badges::stack::Kind;
use crw_core::history::Completion;
use crw_core::registry::State;
use crw_core::roadmap::table::{Counts, Row};
use eframe::egui::{self, Align, Checkbox, Layout, Rect, RichText, Sense, Stroke, UiBuilder, vec2};

use crate::analytics::{self, Loaded};
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
/// Between two columns; the expander, the pin and the name sit closer.
const GAP: f32 = 10.0;
const TIGHT: f32 = 2.0;
/// On a wide row the gaps after the name share the spare width, each widening by at most this
/// (EP-7a); what they leave goes before the session column, so that the session mark and the two
/// switches keep the row's right edge, as they did while the brief's title took it (EP-46).
const SPREAD: f32 = 12.0;
/// Around the progress pill's text.
const PILL_PAD: f32 = 16.0;
/// Adopt and Ignore at the end of an unadopted folder's row.
const FOLDER_BUTTONS: f32 = 150.0;
/// The most of the pane's height the legend takes before it scrolls, and the room above it.
const LEGEND_AT_MOST: f32 = 0.45;
const LEGEND_TOP: i8 = 8;
/// The sparkline's width in the expanded row, and the room beside the phases it asks for.
const SPARK_WIDTH: f32 = 220.0;
/// Under this width a phase's line wraps as a whole: its id, bar and counts take about 190.
const PHASE_LINE_AT_LEAST: f32 = 300.0;
/// A card's width when the pane fits several to a line (EP-26a); a narrower pane gives one card
/// its whole width. The gap between two cards, a card's inner margin and its stroke, which the
/// frame adds outside the margin.
const CARD_WIDTH: f32 = 280.0;
const CARD_GAP: f32 = 8.0;
const CARD_PAD: i8 = 8;
const CARD_STROKE: f32 = 1.0;
/// The lines a card gives the next brief's title before an ellipsis.
const TITLE_ROWS: usize = 2;

/// The columns of a row, left to right. The brief's one-liner has none since EP-46: the next
/// EP's id carries it on hover, and the Detail pane shows the brief.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Column {
    Expander,
    Pin,
    Name,
    State,
    Next,
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
    const ALL: [Column; 17] = [
        Column::Expander,
        Column::Pin,
        Column::Name,
        Column::State,
        Column::Next,
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
    const LEFT: [Column; 5] = [
        Column::Expander,
        Column::Pin,
        Column::Name,
        Column::State,
        Column::Next,
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
                 hour, L about two hours; its one-line summary on the id's hover, the brief in \
                 the Detail pane",
            ),
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

/// How wide each column is: the fixed ones, and for the others the widest value the portfolio
/// holds, at least its header's name.
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
            match row.next() {
                Some(next) => widths.fit(ui, Column::Next, |ui| next_cell(ui, next)),
                // Why there is none is the cell's value then, in its short words.
                None => widths.fit(ui, Column::Next, |ui| no_next_label(ui, row)),
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
/// them. The optional columns give way, in D-25's order, while the row cannot hold them; then
/// the gaps after the name share the spare width, each up to [`SPREAD`], and what they leave
/// goes before the session column, so the row is filled to its right edge (EP-46). One plan
/// serves the header row and every row.
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
            .map(|optional| columns_with(optional))
            .find(|columns| Self::least(widths, columns) <= width)
            .unwrap_or_else(|| columns_with(&[]));
        let room = (width - Self::least(widths, &columns)).max(0.0);
        let spreading = columns
            .iter()
            .take(columns.len().saturating_sub(1))
            .filter(|column| column.spreads())
            .count() as f32;
        let extra = if spreading > 0.0 {
            (room / spreading).min(SPREAD)
        } else {
            0.0
        };
        let rest = room - extra * spreading;
        let mut x = 0.0;
        let cells = columns
            .iter()
            .map(|&column| {
                if column == Column::RIGHT[0] {
                    x += rest;
                }
                let width = widths.of(column);
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

    /// What a row of `columns` takes: their widths and the gaps between them at their least.
    fn least(widths: &Widths, columns: &[Column]) -> f32 {
        let cells: f32 = columns.iter().map(|&column| widths.of(column)).sum();
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

    /// The room in `row` from `from`'s left edge to the gap before the first of `until` the plan
    /// shows: where a row's words go across the cells it leaves empty.
    fn reach(&self, row: Rect, from: Column, until: &[Column]) -> Option<Rect> {
        let (left, _) = self.span(from)?;
        let (right, _) = until.iter().find_map(|&column| self.span(column))?;
        Some(Rect::from_min_max(
            egui::pos2(row.left() + left, row.top()),
            egui::pos2(row.left() + right - GAP, row.bottom()),
        ))
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
        // The sparklines' read of the ledger (EP-26a), when a row is expanded or the cards show.
        if self.view.cards || !self.view.expanded.is_empty() {
            let ledger = self.ledger().map(std::path::Path::to_path_buf);
            self.analytics
                .refresh(ledger.as_deref(), std::time::Instant::now());
        }
        let drawing = Drawing {
            view: &self.view,
            scroll: self.scroll_to_selected,
            now,
            today: Local::now().date_naive(),
            loaded: self.analytics.loaded.as_ref(),
        };
        let plan = (!self.view.cards).then(|| {
            let widths = Widths::measure(ui, &self.portfolio, now);
            Plan::new(&widths, ui.available_width())
        });
        if let Some(plan) = &plan {
            header_row(ui, plan);
            ui.separator();
        }
        let mut actions = Vec::new();
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                // Once for the list, so that every cell shares the style (EP-15a).
                style::unselectable(ui);
                let (per_line, width) = cards_per_line(ui.available_width());
                for group in &groups {
                    let folded = self.view.is_folded(group.kind);
                    if group_header(ui, group.kind, group.members.len(), folded, &self.view) {
                        actions.push(Action::ToggleGroup(group.kind));
                    }
                    if folded {
                        continue;
                    }
                    match &plan {
                        Some(plan) => {
                            for &member in &group.members {
                                actions.extend(self.member_row(ui, member, plan, &drawing));
                            }
                        }
                        None => {
                            for line in group.members.chunks(per_line) {
                                // The cards of a line as tall as its tallest: each card's own
                                // height is kept from the pass before, so a line settles in a
                                // pass and each card is drawn once.
                                let keys: Vec<egui::Id> = line
                                    .iter()
                                    .map(|&member| {
                                        ui.scope_id()
                                            .with(("card height", self.member_name(member)))
                                    })
                                    .collect();
                                let height = keys
                                    .iter()
                                    .filter_map(|key| ui.data(|data| data.get_temp::<f32>(*key)))
                                    .fold(0.0, f32::max);
                                let size = vec2(width, height);
                                let mut heights = Vec::with_capacity(line.len());
                                ui.horizontal_top(|ui| {
                                    ui.spacing_mut().item_spacing.x = CARD_GAP;
                                    for &member in line {
                                        let (more, own) =
                                            self.member_card(ui, member, size, &drawing);
                                        actions.extend(more);
                                        heights.push(own);
                                    }
                                });
                                for (key, own) in keys.into_iter().zip(heights) {
                                    let kept = ui.data(|data| data.get_temp::<f32>(key));
                                    // A height that moved (a first pass, a re-read, a wider
                                    // pane) asks for this pass again, so that no frame shows
                                    // the line unequal and an idle window needs no input.
                                    if kept.is_none_or(|kept| (kept - own).abs() > 0.5) {
                                        ui.ctx().request_discard("a card's height changed");
                                    }
                                    ui.data_mut(|data| data.insert_temp(key, own));
                                }
                                ui.add_space(CARD_GAP - ui.spacing().item_spacing.y);
                            }
                        }
                    }
                    ui.add_space(6.0);
                }
            });
        self.scroll_to_selected = false;
        self.actions.extend(actions);
    }

    /// One member of a group as a row of the list.
    fn member_row(
        &self,
        ui: &mut egui::Ui,
        member: Member,
        plan: &Plan,
        drawing: &Drawing<'_>,
    ) -> Vec<Action> {
        match member {
            Member::Project(i) => {
                let row = &self.portfolio.projects[i];
                let live = self.session_summary(row.name());
                project_row(ui, row, plan, live.as_deref(), drawing)
            }
            Member::Folder(i) => {
                let folder = &self.portfolio.unadopted[i];
                folder_row(ui, &folder.name, folder.git, folder.roadmap, plan, drawing)
            }
        }
    }

    /// A member's name: the project's or the unadopted folder's.
    fn member_name(&self, member: Member) -> &str {
        match member {
            Member::Project(i) => self.portfolio.projects[i].name(),
            Member::Folder(i) => &self.portfolio.unadopted[i].name,
        }
    }

    /// One member of a group as a card of `size`, its content at least as tall (EP-26a): its
    /// actions and its content's own height.
    fn member_card(
        &self,
        ui: &mut egui::Ui,
        member: Member,
        size: egui::Vec2,
        drawing: &Drawing<'_>,
    ) -> (Vec<Action>, f32) {
        let (selection, row) = match member {
            Member::Project(i) => {
                let row = &self.portfolio.projects[i];
                (Selection::Project(row.name().to_owned()), Some(row))
            }
            Member::Folder(i) => (
                Selection::Folder(self.portfolio.unadopted[i].name.clone()),
                None,
            ),
        };
        card(ui, selection, row, size, drawing, |ui| {
            self.card_content(ui, member, drawing)
        })
    }

    /// What a member's card holds (EP-26a), with the actions of its buttons.
    fn card_content(
        &self,
        ui: &mut egui::Ui,
        member: Member,
        drawing: &Drawing<'_>,
    ) -> Vec<Action> {
        match member {
            Member::Project(i) => {
                let row = &self.portfolio.projects[i];
                let live = self.session_summary(row.name());
                project_card(ui, row, live.as_deref(), drawing);
                Vec::new()
            }
            Member::Folder(i) => {
                let folder = &self.portfolio.unadopted[i];
                folder_card(ui, &folder.name, folder.git, folder.roadmap)
            }
        }
    }

    fn list_toolbar(&mut self, ui: &mut egui::Ui) {
        // Wrapped, so that a narrow pane takes the switches on a second line rather than
        // widening the pane past its edge (EP-13's trap; EP-26a).
        ui.horizontal_wrapped(|ui| {
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
            ui.separator();
            // Expand all, Collapse all and the Cards switch (D-25; EP-26a).
            if ui
                .add_enabled(
                    !self.view.cards && self.view.can_expand_all(&self.portfolio),
                    egui::Button::new(style::icon(icons::ICON_UNFOLD_MORE, None)),
                )
                .on_hover_text("Expand all: every row the list shows, to its phases and sparkline")
                .on_disabled_hover_text(if self.view.cards {
                    "The cards show every project's sparkline already"
                } else {
                    "Every shown row is expanded"
                })
                .clicked()
            {
                self.view.expand_all(&self.portfolio);
            }
            if ui
                .add_enabled(
                    !self.view.cards && !self.view.expanded.is_empty(),
                    egui::Button::new(style::icon(icons::ICON_UNFOLD_LESS, None)),
                )
                .on_hover_text("Collapse all: fold every expanded row")
                .on_disabled_hover_text(if self.view.cards {
                    "The rows are not shown"
                } else {
                    "No row is expanded"
                })
                .clicked()
            {
                self.view.collapse_all();
            }
            ui.toggle_value(
                &mut self.view.cards,
                format!("{} Cards", icons::ICON_GRID_VIEW.codepoint),
            )
            .on_hover_text(
                "One card per project in place of the rows, in the same groups and order: the \
                 name, the state, the next brief, the progress, the badges and the burn-up \
                 sparkline (D-25)",
            );
        });
    }
}

/// How many cards fit on a line of `available` points and how wide each is: as many as fit at
/// [`CARD_WIDTH`], the line's spare width shared; one as wide as the line when it is narrower.
fn cards_per_line(available: f32) -> (usize, f32) {
    let count = ((available + CARD_GAP) / (CARD_WIDTH + CARD_GAP))
        .floor()
        .max(1.0) as usize;
    let width = ((available - CARD_GAP * (count - 1) as f32) / count as f32).floor();
    (count, width.max(1.0))
}

/// The width a card's content takes inside its margins and its stroke.
fn card_inner(width: f32) -> f32 {
    (width - 2.0 * (f32::from(CARD_PAD) + CARD_STROKE)).max(1.0)
}

/// What every row and card of one pass draws with (EP-26a): the list's view, whether the
/// selection is brought into view, the clock and the local day, and the ledger's read the
/// sparklines draw from.
struct Drawing<'a> {
    view: &'a View,
    scroll: bool,
    now: DateTime<Utc>,
    today: NaiveDate,
    loaded: Option<&'a Loaded>,
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

pub(crate) fn heading(kind: GroupKind) -> &'static str {
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

/// The tint of a row or a card: the selection's, or the hover's, or none.
fn tint(ui: &egui::Ui, selected: bool, hovered: bool) -> Option<egui::Color32> {
    let visuals = ui.visuals();
    if selected {
        Some(visuals.selection.bg_fill.gamma_multiply(0.55))
    } else if hovered {
        Some(visuals.widgets.hovered.weak_bg_fill)
    } else {
        None
    }
}

/// The row's background: the selection's tint, or the hover's.
fn background(ui: &egui::Ui, rect: Rect, selected: bool, hovered: bool) {
    if let Some(fill) = tint(ui, selected, hovered) {
        ui.painter().rect_filled(rect, 4.0, fill);
    }
}

/// One project's row, and below it the inline expansion when it is open (EP-26a: its phases and
/// the burn-up sparkline), which answers the pointer as the row does, the two under one tint;
/// `live` is what the window's own sessions say of the project.
fn project_row(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    plan: &Plan,
    live: Option<&str>,
    drawing: &Drawing<'_>,
) -> Vec<Action> {
    let Drawing { view, now, .. } = *drawing;
    let mut actions = Vec::new();
    let name = row.name();
    let selection = Selection::Project(name.to_owned());
    let selected = view.selected.as_ref() == Some(&selection);
    let expanded = view.expanded.contains(name);
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), view.density.row_height()),
        Sense::click(),
    );
    // The tint is painted once the expansion's hover is known, under everything drawn after.
    let backdrop = ui.painter().add(egui::Shape::Noop);
    if selected && drawing.scroll {
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
        // A gone folder has no reading: its words take the cells up to the ledger's figures.
        (None, _) => {
            if let Some(room) = plan.reach(rect, Column::Next, &[Column::Usage, Column::Session]) {
                cell(ui, room, |ui| {
                    ui.add(
                        egui::Label::new(RichText::new(missing_folder(row)).color(style::red(ui)))
                            .truncate(),
                    );
                });
            }
        }
        (Some(_), Some(next)) => {
            plan.draw(ui, rect, Column::Next, |ui| next_cell(ui, next));
        }
        (Some(_), None) => {
            plan.draw(ui, rect, Column::Next, |ui| no_next_label(ui, row));
        }
    }
    if row.folder.is_some() {
        if let Some(progress) = plan.rect(Column::Progress, rect)
            && counts.total > 0
        {
            progress_cell(ui, progress, counts);
        }
        plan.draw(ui, rect, Column::Items, |ui| counts_cell(ui, row));
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

    // The expansion senses the click as a whole, under its labels, which sense hover only
    // (EP-15a), so that it answers over its text as the row does.
    let below = expanded.then(|| {
        let indent = plan.span(Column::Name).map_or(0.0, |(x, _)| x);
        ui.scope_builder(
            UiBuilder::new()
                .id_salt(("expansion", name))
                .sense(Sense::click()),
            |ui| {
                style::unselectable(ui);
                // As wide as the row, so that it answers and tints across it (the review's).
                ui.set_min_width(ui.available_width());
                expansion(ui, row, indent, drawing);
            },
        )
        .response
    });
    let hovered = response.hovered() || below.as_ref().is_some_and(egui::Response::hovered);
    if let Some(fill) = tint(ui, selected, hovered) {
        let area = below.as_ref().map_or(rect, |below| rect.union(below.rect));
        ui.painter()
            .set(backdrop, egui::Shape::rect_filled(area, 4.0, fill));
    }
    for response in std::iter::once(&response).chain(below.as_ref()) {
        if response.double_clicked() {
            actions.push(Action::Open(selection.clone()));
        } else if response.clicked() {
            actions.push(Action::Select(selection.clone()));
        }
        row_menu(response, row, now, &mut actions);
    }
    actions
}

/// The row's menu on a right click, shared by the row and the card (EP-26a): pin, priority and
/// state (D-6's "by hand"), Refresh this project (EP-25a), the proposals' dismissals (D-6),
/// Send a message… (D-13), Formalise… and Migrate the roadmap… (D-11), the actions its hosting
/// allows (D-24; EP-34), Revert the latest session… (EP-15c).
fn row_menu(
    response: &egui::Response,
    row: &ProjectRow,
    now: DateTime<Utc>,
    actions: &mut Vec<Action>,
) {
    let name = row.name();
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
        // What the project's hosting allows on GitHub and on its origin, each behind its
        // confirmation (D-24; EP-34).
        let offered = crate::act::offered(row);
        if !offered.is_empty() {
            ui.separator();
            for verb in offered {
                if ui
                    .button(verb.label())
                    .on_hover_text(crate::act::hint(verb))
                    .clicked()
                {
                    actions.push(Action::OpenAct {
                        project: Some(name.to_owned()),
                        verb,
                    });
                    ui.close();
                }
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

/// Why a project with its folder shows no next brief, in the words of the Next cell's hover and
/// of the cards; the phone's list says the same (EP-30).
pub(crate) fn no_next(row: &ProjectRow) -> &'static str {
    why_no_next(row).1
}

/// The same, short and long: the short words fill the Next cell, which is measured to hold them
/// and stays about as wide as an EP's id and size (EP-46).
fn why_no_next(row: &ProjectRow) -> (&'static str, &'static str) {
    if row.counts().all_checked() {
        ("all checked", "Every row is checked")
    } else if row.snapshot.current().is_none() {
        ("no roadmap", "No roadmap/README.md")
    } else {
        ("no table", "No phase table read")
    }
}

/// A registered project whose folder is gone, in the row's words.
pub(crate) fn missing_folder(row: &ProjectRow) -> String {
    format!("Folder missing: {}", row.path.display())
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

/// Why a project with its folder has no next brief, short in the Next cell, whole on hover
/// (EP-46).
fn no_next_label(ui: &mut egui::Ui, row: &ProjectRow) {
    let (short, words) = why_no_next(row);
    ui.label(RichText::new(short).weak()).on_hover_text(words);
}

/// The next EP's id and its size; the id's hover carries the brief's one-liner, its markup drawn
/// (EP-46: the row has no column for it).
fn next_cell(ui: &mut egui::Ui, next: &Row) {
    ui.label(RichText::new(next.id.to_string()).monospace())
        .on_hover_ui(|ui| {
            let title = markup::job(ui, &next.brief.title, Look::default());
            ui.label(title);
        });
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
    painter.rect_filled(pill, 5.0, style::track(ui));
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
    let texts = session_words(row, live);
    if !texts.is_empty() {
        SESSION.show(ui, &format!("Session in progress: {}", texts.join("; ")));
    } else if row.awaits_git() {
        GIT_PENDING.show(ui, "Git has not been asked yet");
    }
}

/// What says a session is in progress: the sessions' own words (`live`), then the files'
/// evidence (D-9); the row's Session cell and the phone's list (EP-30).
pub(crate) fn session_words(row: &ProjectRow, live: Option<&str>) -> Vec<String> {
    live.map(str::to_owned)
        .into_iter()
        .chain(row.in_progress().iter().map(ToString::to_string))
        .collect()
}

fn size_hint(size: crw_core::roadmap::Size) -> &'static str {
    match size {
        crw_core::roadmap::Size::S => "S: about 30 minutes of one session",
        crw_core::roadmap::Size::M => "M: about an hour",
        crw_core::roadmap::Size::L => "L: about two hours",
    }
}

/// The inline expansion under the name: each phase's progress, and the burn-up sparkline beside
/// them when the row is wide enough, else under them (EP-26a); the phase names wrap in a narrow
/// pane. Beside, the phases take the row's width less the sparkline's, so the sparkline is set
/// from the row's right and every expanded row's starts at one place, whatever its phases' names
/// (EP-41; under a shared least width a long name pushed its row's further right).
fn expansion(ui: &mut egui::Ui, row: &ProjectRow, indent: f32, drawing: &Drawing<'_>) {
    let beside = ui.available_width() - indent >= 2.0 * SPARK_WIDTH + GAP;
    ui.horizontal_top(|ui| {
        ui.add_space(indent);
        let phases_width = if beside {
            ui.available_width() - SPARK_WIDTH - GAP
        } else {
            ui.available_width()
        };
        ui.vertical(|ui| {
            ui.set_max_width(phases_width);
            if beside {
                ui.set_min_width(phases_width);
            }
            phases(ui, row);
            if !beside {
                ui.add_space(2.0);
                spark_block(ui, row, SPARK_WIDTH.min(ui.available_width()), drawing);
            }
        });
        if beside {
            ui.vertical(|ui| {
                ui.set_width(SPARK_WIDTH);
                spark_block(ui, row, SPARK_WIDTH, drawing);
            });
        }
    });
    ui.add_space(4.0);
}

/// Each phase of the roadmap on a line: its id, a bar, its counts and its name, which wraps.
fn phases(ui: &mut egui::Ui, row: &ProjectRow) {
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
                let line = |ui: &mut egui::Ui| {
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
                    ui.label(RichText::new(progress.as_str()).weak());
                    ui.add(egui::Label::new(format!("{}{status}", heading.name)).wrap());
                };
                // The name wraps under itself; a pane too narrow for the id, the bar and the
                // counts beside a name lets the line itself wrap rather than run past its edge.
                if ui.available_width() >= PHASE_LINE_AT_LEAST {
                    ui.horizontal_top(line);
                } else {
                    ui.horizontal_wrapped(line);
                }
            }
        }
        _ => {
            ui.label(RichText::new("No phases to show.").weak());
        }
    }
}

/// The project's completions as the history imported them (EP-25); none before the import.
fn completions_of(row: &ProjectRow) -> &[Completion] {
    row.history
        .as_ref()
        .map_or(&[], |summary| summary.import.completions.as_slice())
}

/// The burn-up sparkline `width` wide under a small caption, or one weak line when there is
/// nothing to draw (EP-26a).
fn spark_block(ui: &mut egui::Ui, row: &ProjectRow, width: f32, drawing: &Drawing<'_>) {
    let today = drawing.today;
    let series = drawing.loaded.and_then(|loaded| {
        analytics::burn_up(completions_of(row), loaded.snapshots_of(row.name()), today)
    });
    let why = match series {
        Some(series) => {
            ui.label(
                RichText::new("Burn-up: done and total by day")
                    .small()
                    .weak(),
            );
            analytics::sparkline(ui, &series, width, today);
            return;
        }
        None if drawing.loaded.is_none() => "No ledger: no burn-up to draw.",
        // The Detail pane's Analytics section says why the read failed.
        None if drawing
            .loaded
            .is_some_and(|loaded| loaded.problem.is_some()) =>
        {
            "The ledger did not read: no burn-up to draw."
        }
        None => "No burn-up yet: no EP dated by the history, no daily count kept.",
    };
    ui.add(egui::Label::new(RichText::new(why).small().weak()).wrap());
}

/// A card of `size` around what `add` draws, with `add`'s actions and the content's own height:
/// a click selects, a double click opens, and a project's card (`row`) opens the row's menu on a
/// right click, as a row does (EP-26a).
fn card(
    ui: &mut egui::Ui,
    selection: Selection,
    row: Option<&ProjectRow>,
    size: egui::Vec2,
    drawing: &Drawing<'_>,
    add: impl FnOnce(&mut egui::Ui) -> Vec<Action>,
) -> (Vec<Action>, f32) {
    let selected = drawing.view.selected.as_ref() == Some(&selection);
    let name = match &selection {
        Selection::Project(name) | Selection::Folder(name) => name.clone(),
    };
    let mut actions = Vec::new();
    let (response, own) = card_frame(ui, &name, size, selected, |ui| actions.extend(add(ui)));
    if selected && drawing.scroll {
        response.scroll_to_me(Some(Align::Center));
    }
    if response.double_clicked() {
        actions.push(Action::Open(selection));
    } else if response.clicked() {
        actions.push(Action::Select(selection));
    }
    if let Some(row) = row {
        row_menu(&response, row, drawing.now, &mut actions);
    }
    (actions, own)
}

/// A card's frame `size.x` wide, its content at least `size.y` tall: the selection's tint or the
/// hover's over the faint fill, a thin stroke; the whole card senses the click under its labels,
/// which sense hover only (EP-15a), so the card answers over its text as the row does (EP-26a).
/// Returns the card's response and its content's own height.
fn card_frame(
    ui: &mut egui::Ui,
    name: &str,
    size: egui::Vec2,
    selected: bool,
    add: impl FnOnce(&mut egui::Ui),
) -> (egui::Response, f32) {
    let (width, height) = (size.x, size.y);
    let mut own = 0.0;
    let response = ui
        .scope_builder(
            // Top to bottom, whatever the line of cards around it lays out (EP-26a).
            UiBuilder::new()
                .id_salt(("card", name))
                .layout(Layout::top_down(Align::Min))
                .sense(Sense::click()),
            |ui| {
                ui.set_width(width);
                style::unselectable(ui);
                // The card's response from the last pass: whether the pointer is over it.
                let hovered = ui.response().hovered();
                let visuals = ui.visuals();
                let fill = tint(ui, selected, hovered).unwrap_or(visuals.faint_bg_color);
                egui::Frame::new()
                    .fill(fill)
                    .stroke(Stroke::new(
                        CARD_STROKE,
                        visuals.widgets.noninteractive.bg_stroke.color,
                    ))
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::same(CARD_PAD))
                    .show(ui, |ui| {
                        ui.set_width(card_inner(width));
                        // Before the content: egui measures a minimum height from the cursor.
                        ui.set_min_height(height);
                        own = ui.scope(add).response.rect.height();
                    });
            },
        )
        .response;
    (response, own)
}

/// One project's card (D-25; EP-26a): the pin and the name; the state chip, the next EP and its
/// size; the next brief's title on two lines at most, whole on hover; the progress pill; what
/// waits, the badges and the session mark on a wrapping line; the sparkline. The card's frame
/// takes the click, the double click and the right click ([`Board::member_card`]).
fn project_card(ui: &mut egui::Ui, row: &ProjectRow, live: Option<&str>, drawing: &Drawing<'_>) {
    let now = drawing.now;
    let name = row.name();
    ui.horizontal(|ui| {
        if row.project.pinned {
            PINNED.show(ui, "Pinned: first in its group");
        }
        name_label(ui, name);
    });
    ui.horizontal_wrapped(|ui| {
        state_cell(ui, row, now);
        if let (Some(_), Some(next)) = (&row.folder, row.next()) {
            next_cell(ui, next);
        }
    });
    let counts = row.counts();
    match (&row.folder, row.next()) {
        (None, _) => {
            ui.add(
                egui::Label::new(RichText::new(missing_folder(row)).color(style::red(ui))).wrap(),
            );
        }
        (Some(_), Some(next)) => {
            let mut title = markup::job(ui, &next.brief.title, Look::default());
            title.wrap.max_rows = TITLE_ROWS;
            title.wrap.overflow_character = Some('…');
            ui.add(egui::Label::new(title).wrap());
        }
        (Some(_), None) => {
            ui.label(RichText::new(no_next(row)).weak());
        }
    }
    if counts.total > 0 {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::hover());
        progress_cell(ui, rect, counts);
    }
    ui.horizontal_wrapped(|ui| {
        // Straight into the wrapped line, the spacing put back after each cell (some set their
        // own): a child Ui per cell would advance the line even when its cell draws nothing,
        // and each mark after it would sit lower.
        let spacing = ui.spacing().item_spacing;
        let put = |ui: &mut egui::Ui, draw: &dyn Fn(&mut egui::Ui)| {
            let before = ui.cursor().min;
            draw(ui);
            // egui spaced the cell's last mark by the cell's own spacing: the line's gap after it.
            if ui.cursor().min != before {
                ui.add_space((spacing.x - ui.spacing().item_spacing.x).max(0.0));
            }
            ui.spacing_mut().item_spacing = spacing;
        };
        put(ui, &|ui| counts_cell(ui, row));
        if row.folder.is_some() {
            put(ui, &|ui| stack_cell(ui, row));
            put(ui, &|ui| hosting_cell(ui, row, now));
            put(ui, &|ui| sync_cell(ui, row, now));
            put(ui, &|ui| age_cell(ui, row, now));
            put(ui, &|ui| health_cell(ui, row, now));
            put(ui, &|ui| backup_cell(ui, row, now));
        }
        put(ui, &|ui| usage_cell(ui, row));
        put(ui, &|ui| session_cell(ui, row, live));
    });
    spark_block(ui, row, ui.available_width(), drawing);
}

/// An unadopted folder's card (D-7; EP-26a): the folder mark and the name, the chip, what it
/// holds, Adopt and Ignore, whose clicks it returns.
fn folder_card(ui: &mut egui::Ui, name: &str, git: bool, roadmap: bool) -> Vec<Action> {
    let mut actions = Vec::new();
    ui.horizontal(|ui| {
        FOLDER.show(ui, "A folder of the data root, not adopted yet");
        name_label(ui, name);
    });
    unadopted_chip(ui);
    ui.add(egui::Label::new(RichText::new(folder_words(git, roadmap)).weak()).wrap());
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Adopt")
            .on_hover_text("Register it and add it to the workspace file")
            .clicked()
        {
            actions.push(Action::Adopt(name.to_owned()));
        }
        if ui
            .button("Ignore")
            .on_hover_text("Leave it out of this group; the registry remembers")
            .clicked()
        {
            actions.push(Action::Ignore(name.to_owned()));
        }
    });
    actions
}

/// What an unadopted folder holds, in words.
fn folder_words(git: bool, roadmap: bool) -> &'static str {
    match (git, roadmap) {
        (true, true) => "A git repository with a roadmap",
        (true, false) => "A git repository without a roadmap",
        (false, _) => "A folder, not a repository",
    }
}

/// An unadopted folder's row: what it holds, Adopt and Ignore (D-7).
fn folder_row(
    ui: &mut egui::Ui,
    name: &str,
    git: bool,
    roadmap: bool,
    plan: &Plan,
    drawing: &Drawing<'_>,
) -> Vec<Action> {
    let view = drawing.view;
    let mut actions = Vec::new();
    let selection = Selection::Folder(name.to_owned());
    let selected = view.selected.as_ref() == Some(&selection);
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), view.density.row_height()),
        Sense::click(),
    );
    background(ui, rect, selected, response.hovered());
    if selected && drawing.scroll {
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
        ui.add(egui::Label::new(RichText::new(folder_words(git, roadmap)).weak()).truncate());
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
            let drawing = drawing(&view, None);
            let actions = project_row(ui, row, &plan, None, &drawing);
            found = (actions, rect, plan.rect(column, rect).unwrap());
        });
        (found.0, found.1, found.2, output)
    }

    /// A pass's drawing context over `view` and `loaded`, now.
    fn drawing<'a>(view: &'a View, loaded: Option<&'a Loaded>) -> Drawing<'a> {
        Drawing {
            view,
            scroll: false,
            now: Utc::now(),
            today: Local::now().date_naive(),
            loaded,
        }
    }

    /// harbor's card drawn at the top of a pass of `screen` with `events`, `CARD_WIDTH` wide:
    /// its actions and what the pass painted.
    fn harbor_card_pass(
        screen: &mut Screen,
        portfolio: &Portfolio,
        loaded: Option<&Loaded>,
        events: Vec<egui::Event>,
    ) -> (Vec<Action>, egui::FullOutput) {
        let view = View::default();
        let mut actions = Vec::new();
        let output = screen.pass(events, |ui| {
            let row = portfolio.project("harbor").unwrap();
            let drawing = drawing(&view, loaded);
            let selection = Selection::Project("harbor".to_owned());
            actions = card(
                ui,
                selection,
                Some(row),
                vec2(CARD_WIDTH, 0.0),
                &drawing,
                |ui| {
                    project_card(ui, row, None, &drawing);
                    Vec::new()
                },
            )
            .0;
        });
        (actions, output)
    }

    /// The ledger's rows a sparkline test draws from: harbor's counts over two weeks.
    fn canned_loaded() -> Loaded {
        let now = Utc::now();
        let snapshot = |days_ago: i64, done, total| crw_core::ledger::SnapshotRow {
            taken_at: now - chrono::TimeDelta::days(days_ago),
            done,
            total,
        };
        Loaded {
            snapshots: [(
                "harbor".to_owned(),
                vec![snapshot(14, 2, 8), snapshot(7, 3, 11), snapshot(0, 4, 11)],
            )]
            .into(),
            ..Loaded::default()
        }
    }

    /// Whether `output` paints a line in `colour`: the sparkline's done series.
    /// A line of two points at least (an empty one is a path too).
    fn strokes(output: &egui::FullOutput, colour: egui::Color32) -> bool {
        output.shapes.iter().any(|clipped| match &clipped.shape {
            egui::Shape::Path(path) => {
                path.points.len() >= 2
                    && path.stroke.color == egui::epaint::ColorMode::Solid(colour)
            }
            _ => false,
        })
    }

    #[test]
    fn a_card_answers_over_its_name_and_opens_its_menu() {
        use crate::analytics::tests::completion;
        use crw_core::history::{Import, Summary};

        let scratch = Scratch::new("list-card-clicks");
        let mut portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        // Two EPs dated in the last fortnight: the done series the sparkline draws in green.
        let today = Local::now().date_naive();
        let day = |ago: u64| (today - chrono::Days::new(ago)).to_string();
        let harbor = portfolio
            .projects
            .iter_mut()
            .find(|row| row.name() == "harbor")
            .unwrap();
        harbor.history = Some(Summary {
            imported_at: Utc::now(),
            head: None,
            import: Import {
                completions: vec![completion("EP-1", &day(12)), completion("EP-2", &day(5))],
                commits: 4,
                single: false,
            },
        });
        let loaded = canned_loaded();
        let mut screen = Screen::new();
        let (_, output) = harbor_card_pass(&mut screen, &portfolio, Some(&loaded), Vec::new());
        let name = testing::text_at(&output, "harbor").expect("the card's name");
        assert!(
            testing::text_containing(&output, "Burn-up").is_some(),
            "the sparkline's caption"
        );
        let green = {
            let mut colour = egui::Color32::TRANSPARENT;
            screen.pass(Vec::new(), |ui| colour = style::green(ui));
            colour
        };
        assert!(strokes(&output, green), "the done series is drawn");
        // Over the name's first letters, which a selectable label would take.
        let at = name + vec2(-10.0, 0.0);
        let press = |button, pressed| testing::button(at, button, pressed);
        harbor_card_pass(&mut screen, &portfolio, Some(&loaded), testing::moved(at));
        harbor_card_pass(
            &mut screen,
            &portfolio,
            Some(&loaded),
            press(egui::PointerButton::Primary, true),
        );
        let (actions, _) = harbor_card_pass(
            &mut screen,
            &portfolio,
            Some(&loaded),
            press(egui::PointerButton::Primary, false),
        );
        assert_eq!(
            actions,
            [Action::Select(Selection::Project("harbor".to_owned()))]
        );
        // A right click over the name opens the row's menu, the card's own.
        harbor_card_pass(
            &mut screen,
            &portfolio,
            Some(&loaded),
            press(egui::PointerButton::Secondary, true),
        );
        harbor_card_pass(
            &mut screen,
            &portfolio,
            Some(&loaded),
            press(egui::PointerButton::Secondary, false),
        );
        let (_, output) = harbor_card_pass(&mut screen, &portfolio, Some(&loaded), Vec::new());
        assert!(
            egui::Popup::is_any_open(&screen.ctx),
            "the row's menu is open"
        );
        assert!(testing::text_at(&output, "Refresh this project").is_some());
    }

    /// Under Expand all the sparklines start at one place (EP-41): a row whose phase has a long
    /// name sets its sparkline where a row of short names does, from the row's right.
    #[test]
    fn the_expanded_rows_sparklines_start_at_one_place() {
        let scratch = Scratch::new("list-spark-edge");
        let mut portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let harbor = portfolio
            .projects
            .iter_mut()
            .find(|row| row.name() == "harbor")
            .unwrap();
        let roadmap = harbor.snapshot.tree.as_mut().expect("harbor's roadmap");
        roadmap.phases[0].heading.name = "A phase whose name runs on and on, well past the \
            least width the phases block once had, so that it wraps"
            .to_owned();
        let mut loaded = canned_loaded();
        let rows = loaded.snapshots["harbor"].clone();
        loaded.snapshots.insert("lantern".to_owned(), rows);
        let view = View {
            expanded: ["harbor".to_owned(), "lantern".to_owned()].into(),
            ..View::default()
        };
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            let widths = Widths::measure(ui, &portfolio, Utc::now());
            let plan = Plan::new(&widths, ui.available_width());
            for name in ["harbor", "lantern"] {
                let row = portfolio.project(name).unwrap();
                project_row(ui, row, &plan, None, &drawing(&view, Some(&loaded)));
            }
        });
        let captions = testing::texts_at(&output, "Burn-up: done and total by day");
        assert_eq!(captions.len(), 2, "both rows draw their sparkline");
        assert!(
            (captions[0].x - captions[1].x).abs() < 0.5,
            "the sparklines start at one place: {captions:?}"
        );
        assert!(captions[0].y < captions[1].y, "one row below the other");
    }

    #[test]
    fn an_expanded_row_draws_the_sparkline_or_says_why_not() {
        let scratch = Scratch::new("list-row-sparkline");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let loaded = canned_loaded();
        let mut screen = Screen::new();
        let view = View {
            expanded: ["harbor".to_owned()].into(),
            ..View::default()
        };
        let pass = |screen: &mut Screen, loaded: Option<&Loaded>, events| {
            let mut actions = Vec::new();
            let output = screen.pass(events, |ui| {
                let widths = Widths::measure(ui, &portfolio, Utc::now());
                let plan = Plan::new(&widths, ui.available_width());
                let row = portfolio.project("harbor").unwrap();
                actions = project_row(ui, row, &plan, None, &drawing(&view, loaded));
            });
            (actions, output)
        };
        let (_, output) = pass(&mut screen, Some(&loaded), Vec::new());
        assert!(testing::text_containing(&output, "Burn-up").is_some());
        let (_, output) = pass(&mut screen, None, Vec::new());
        assert!(testing::text_containing(&output, "No ledger").is_some());
        let empty = Loaded::default();
        let (_, output) = pass(&mut screen, Some(&empty), Vec::new());
        assert!(testing::text_containing(&output, "No burn-up yet").is_some());

        // The expansion answers over its text as the row does: a click over its caption
        // selects the project, a right click opens the row's menu.
        let at = testing::text_containing(&output, "No burn-up yet").unwrap();
        let press = |button, pressed| testing::button(at, button, pressed);
        pass(&mut screen, Some(&empty), testing::moved(at));
        pass(
            &mut screen,
            Some(&empty),
            press(egui::PointerButton::Primary, true),
        );
        let (actions, _) = pass(
            &mut screen,
            Some(&empty),
            press(egui::PointerButton::Primary, false),
        );
        assert_eq!(
            actions,
            [Action::Select(Selection::Project("harbor".to_owned()))]
        );
        pass(
            &mut screen,
            Some(&empty),
            press(egui::PointerButton::Secondary, true),
        );
        pass(
            &mut screen,
            Some(&empty),
            press(egui::PointerButton::Secondary, false),
        );
        let (_, output) = pass(&mut screen, Some(&empty), Vec::new());
        assert!(
            egui::Popup::is_any_open(&screen.ctx),
            "the row's menu is open over the expansion"
        );
        assert!(testing::text_at(&output, "Refresh this project").is_some());
    }

    #[test]
    fn the_cards_fill_the_line_and_a_narrow_pane_gets_one() {
        let (count, width) = cards_per_line(1000.0);
        assert_eq!(count, 3, "three at 280 with two gaps fit in 1000");
        assert!((width - 328.0).abs() < 0.01, "{width}");
        assert_eq!(cards_per_line(200.0), (1, 200.0));
        let (count, width) = cards_per_line(CARD_WIDTH * 2.0 + CARD_GAP);
        assert_eq!((count, width), (2, CARD_WIDTH));
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

    /// The row's menu offers what the project's hosting allows (EP-34): a public repository's
    /// Make private… and Remove the remote…, each opening its confirmation.
    #[test]
    fn the_rows_menu_offers_what_its_hosting_allows() {
        use crw_core::actions::Verb;
        use crw_core::badges::hosting::{Hosting, Listed, Repo, Visibility};
        let scratch = Scratch::new("list-row-hosting");
        let mut portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let harbor = portfolio
            .projects
            .iter_mut()
            .find(|row| row.name() == "harbor")
            .unwrap();
        harbor.badges.remote = Some("https://github.com/example/harbor.git".to_owned());
        harbor.badges.hosting = Hosting::Listed(Listed {
            repo: Repo {
                owner: "example".to_owned(),
                name: "harbor".to_owned(),
            },
            visibility: Visibility::Public,
            archived: false,
            pushed_at: None,
            default_branch: Some("main".to_owned()),
        });
        harbor.badges.activity = Some(crw_core::badges::activity::Activity {
            oid: Some("f".repeat(40)),
            last_commit: None,
            changed: 0,
            branch: Some("main".to_owned()),
            upstream: None,
            ahead_behind: None,
        });
        let mut screen = Screen::new();
        let (_, _, name, _) = harbor_pass(&mut screen, &portfolio, Vec::new());
        let at = name.left_center() + vec2(8.0, 0.0);
        let press = |pressed| testing::button(at, egui::PointerButton::Secondary, pressed);
        harbor_pass(&mut screen, &portfolio, testing::moved(at));
        harbor_pass(&mut screen, &portfolio, press(true));
        harbor_pass(&mut screen, &portfolio, press(false));
        let (.., output) = harbor_pass(&mut screen, &portfolio, Vec::new());
        assert!(testing::text_at(&output, "Remove the remote…").is_some());
        assert!(testing::text_at(&output, "Make public…").is_none());
        let item = testing::text_at(&output, "Make private…").expect("the menu item");
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
        assert_eq!(
            actions,
            [Action::OpenAct {
                project: Some("harbor".to_owned()),
                verb: Verb::MakePrivate,
            }]
        );
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

    /// No cell narrower than its column's widest value; the cells in order without overlap.
    fn check(plan: &Plan, widths: &Widths) {
        let mut right = 0.0;
        for &(column, x, width) in &plan.cells {
            assert!(x >= right, "{column:?} overlaps the cell before it");
            assert!(width >= widths.of(column), "{column:?} is clipped");
            right = x + width;
        }
    }

    fn right_edge(plan: &Plan) -> f32 {
        plan.cells.last().map_or(0.0, |&(_, x, width)| x + width)
    }

    /// The space between the end of `before`'s cell and the start of `after`'s.
    fn space(plan: &Plan, before: Column, after: Column) -> f32 {
        let (left, width) = plan.span(before).unwrap();
        plan.span(after).unwrap().0 - (left + width)
    }

    #[test]
    fn a_wide_row_shows_every_column_and_keeps_its_switches_at_the_edge() {
        let widths = widths(30.0);
        let plan = Plan::new(&widths, 3000.0);
        check(&plan, &widths);
        assert_eq!(shown(&plan), Column::ALL);
        assert!((right_edge(&plan) - 3000.0).abs() < 0.01);
        // So much spare width that every gap after the name widens by the cap…
        assert!((space(&plan, Column::Name, Column::State) - (GAP + SPREAD)).abs() < 0.01);
        assert!((space(&plan, Column::Session, Column::Autopilot) - (GAP + SPREAD)).abs() < 0.01);
        // … and the rest goes before the session column.
        assert!(space(&plan, Column::Usage, Column::Session) > 2000.0);
        // The expander and the pin stay close to the name.
        assert!((space(&plan, Column::Pin, Column::Name) - TIGHT).abs() < 0.01);
    }

    #[test]
    fn the_spare_width_is_shared_among_the_gaps_and_fills_the_row() {
        let widths = widths(30.0);
        let full = columns_with(Column::GIVE_WAY[0]);
        let least = Plan::least(&widths, &full);
        let spreading = full
            .iter()
            .take(full.len() - 1)
            .filter(|column| column.spreads())
            .count() as f32;
        // Half the gaps' cap to spare: each gap after the name widens by half of it, the
        // session column keeps its gap, and the row is filled to its edge.
        let plan = Plan::new(&widths, least + spreading * SPREAD / 2.0);
        check(&plan, &widths);
        assert_eq!(shown(&plan), full);
        assert!((right_edge(&plan) - (least + spreading * SPREAD / 2.0)).abs() < 0.01);
        for (before, after) in [
            (Column::Name, Column::State),
            (Column::Next, Column::Progress),
            (Column::Usage, Column::Session),
        ] {
            let space = space(&plan, before, after);
            assert!(
                (space - (GAP + SPREAD / 2.0)).abs() < 0.01,
                "{after:?}: {space}"
            );
        }
        // At the full form's least exactly, the gaps keep theirs.
        let tight = Plan::new(&widths, least);
        assert_eq!(shown(&tight), full);
        assert!((right_edge(&tight) - least).abs() < 0.01);
        assert!((space(&tight, Column::Next, Column::Progress) - GAP).abs() < 0.01);
    }

    #[test]
    fn a_narrowing_row_gives_way_in_d25s_order() {
        let widths = widths(30.0);
        let fitting = |choice: usize| Plan::least(&widths, &columns_with(Column::GIVE_WAY[choice]));
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
        // Narrower than the fewest columns allow: nothing is clipped, the row runs past its edge.
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

    /// The row draws no brief's title (EP-46): the next EP's id carries it on hover. A project
    /// with no next brief says why in the Next cell, in short words drawn whole; one whose folder
    /// is gone says so across the cells it leaves empty.
    #[test]
    fn the_brief_is_on_the_ids_hover() {
        let scratch = Scratch::new("list-row-title");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let mut screen = Screen::new();
        let title = "Ledger view";
        let (_, _, next, output) =
            harbor_cell_pass(&mut screen, &portfolio, Vec::new(), Column::Next);
        assert!(
            testing::text_containing(&output, title).is_none(),
            "the row draws no title"
        );
        let at = next.left_center() + vec2(8.0, 0.0);
        let mut output =
            harbor_cell_pass(&mut screen, &portfolio, testing::moved(at), Column::Next).3;
        // The hover waits for the pointer to rest.
        for _ in 0..20 {
            output = harbor_cell_pass(&mut screen, &portfolio, Vec::new(), Column::Next).3;
        }
        assert!(
            testing::text_containing(&output, title).is_some(),
            "the id's hover"
        );

        let view = View::default();
        let mut next = 0.0;
        let output = screen.pass(Vec::new(), |ui| {
            let widths = Widths::measure(ui, &portfolio, Utc::now());
            next = widths.of(Column::Next);
            let plan = Plan::new(&widths, ui.available_width());
            let drawing = drawing(&view, None);
            for name in ["orchard", "atlas", "tundra"] {
                project_row(ui, portfolio.project(name).unwrap(), &plan, None, &drawing);
            }
        });
        let drawn = |text: &str| {
            output
                .shapes
                .iter()
                .find_map(|clipped| match &clipped.shape {
                    egui::Shape::Text(shape) if shape.galley.text() == text => {
                        Some(shape.galley.clone())
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{text} is not drawn"))
        };
        for (name, short) in [("orchard", "all checked"), ("atlas", "no roadmap")] {
            let row = portfolio.project(name).unwrap();
            assert!(row.next().is_none(), "{name}");
            assert_eq!(why_no_next(row).0, short);
            assert!(!drawn(short).elided, "{name}: {short}");
            // The Next cell stays about as wide as an EP's id and size.
            assert!(drawn(short).rect.width() <= next, "{name}");
        }
        assert!(next < 100.0, "{next}");
        // A path may be long: cut where the empty cells end, whole on hover, and wider than the
        // Next cell alone.
        let tundra = portfolio.project("tundra").unwrap();
        assert!(tundra.folder.is_none());
        assert!(drawn(&missing_folder(tundra)).rect.width() > next + GAP);
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
