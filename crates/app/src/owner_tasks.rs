//! The Owner tasks pane (D-10, D-25; EP-50): every project's open owner tasks in one place,
//! grouped by project in the list's order (its groups and sort key; its filters left aside, so a
//! filter never hides what waits), each ticked there as the Detail pane ticked it. A gate is an
//! amber chip when it holds the project's next brief (the autopilot's rule, `prompt::due`), a
//! weak one when it names a later brief. Two toggles, saved with the view: the selected project
//! alone, and the tasks the tool ticked in the last week, as the ledger's `tick` events date
//! them, each with Untick. At each project's foot its older forms, folded shut, read-only and
//! counted nowhere (D-10 addendum of 2026-11-22). The Detail pane keeps one line that leads here.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, TimeDelta, Utc};
use crw_core::ledger::{EventRow, Ledger};
use crw_core::revert::{TICK_KIND, Tick};
use crw_core::tasks::{Gate, OwnerItem};
use eframe::egui::{self, RichText};

use crate::board::{Action, Board};
use crate::markup::{self, Look};
use crate::portfolio::{Filter, Member, ProjectRow, Selection};
use crate::style::{self, icons};

/// How far back the toggle's ticks go.
const WEEK: TimeDelta = TimeDelta::days(7);

/// How often the ledger's ticks are read again while the toggle is on; a tick reads them at once.
const EVERY: Duration = Duration::from_secs(60);

/// The newest `tick` events read: far more than a week holds.
const LIMIT: usize = 1_000;

/// The owner tasks the tool ticked or cleared in the last week, the latest of each, as the
/// ledger's `tick` events give them (EP-50). A tick written into the file by hand is dated
/// nowhere, so it is not among them.
#[derive(Debug, Default)]
pub struct Ticks {
    read: Option<Instant>,
    /// (project in lower case, task id) → when it was last ticked or cleared, and which.
    latest: HashMap<(String, String), (DateTime<Utc>, bool)>,
}

impl Ticks {
    /// Read again at the next chance: a tick or an untick was just made.
    pub fn stale(&mut self) {
        self.read = None;
    }

    /// Reads the ledger's ticks again when they are a minute old or stale. A ledger busy past
    /// the window's wait keeps the ticks read before, tried again a minute later (EP-43).
    fn refresh(&mut self, ledger: Option<&Path>, now: Instant, since: DateTime<Utc>) {
        if self
            .read
            .is_some_and(|at| now.saturating_duration_since(at) < EVERY)
        {
            return;
        }
        self.read = Some(now);
        let Some(path) = ledger else {
            self.latest.clear();
            return;
        };
        let events = Ledger::open_read_only_waiting(path, crate::pace::LEDGER_WAIT)
            .and_then(|ledger| ledger.events(TICK_KIND, Some(LIMIT)));
        match events {
            Ok(events) => self.latest = latest(&events, since),
            // A busy ledger is asked again after the window's retry, not a minute later, as
            // the other reads in its pass are (EP-43; EP-50's review).
            Err(e) if e.is_busy() => {
                self.read = now.checked_sub(EVERY.saturating_sub(crate::pace::LEDGER_RETRY));
            }
            Err(_) => {}
        }
    }

    /// When the tool ticked `id` of `project` in the last week, if its latest tick ticked it.
    pub fn ticked_at(&self, project: &str, id: &str) -> Option<DateTime<Utc>> {
        self.latest
            .get(&(project.to_lowercase(), id.to_owned()))
            .and_then(|&(at, checked)| checked.then_some(at))
    }
}

/// The latest tick or clear of each owner task at `since` or later, from `events` the newest
/// first; a message's ticks and the mailman's deliveries left out.
fn latest(
    events: &[EventRow],
    since: DateTime<Utc>,
) -> HashMap<(String, String), (DateTime<Utc>, bool)> {
    let mut latest = HashMap::new();
    for event in events {
        let (Some(project), Some(at), Some(detail)) = (
            event.project.as_deref(),
            event.at(),
            event.detail.as_deref(),
        ) else {
            continue;
        };
        let Some(tick) = Tick::parse(at, detail) else {
            continue;
        };
        let task = tick
            .id
            .strip_prefix(crw_core::tasks::PREFIX)
            .is_some_and(|rest| rest.starts_with('-'));
        if tick.delivery || !task || at < since {
            continue;
        }
        latest
            .entry((project.to_lowercase(), tick.id))
            .or_insert((at, tick.checked));
    }
    latest
}

impl Board {
    /// Every project's open owner tasks (EP-50); drawn while the pane is in front.
    pub fn owner_tasks_pane(&mut self, ui: &mut egui::Ui) {
        if self.view.tasks_ticked {
            let ledger = self.ledger().map(Path::to_path_buf);
            self.ticks
                .refresh(ledger.as_deref(), Instant::now(), Utc::now() - WEEK);
        }
        let mut actions = Vec::new();
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.view.tasks_one, "This project only")
                .on_hover_text("Show the project selected in the list alone");
            ui.checkbox(&mut self.view.tasks_ticked, "Ticked in the last week")
                .on_hover_text(
                    "Show the tasks the tool ticked in the last seven days (from this pane, the \
                     Detail pane, the phone or crw tick), each with Untick; a tick written into \
                     the file by hand is dated nowhere, so it is not shown",
                );
        });
        let selected = match &self.view.selected {
            Some(Selection::Project(name)) => Some(name.clone()),
            _ => None,
        };
        let rows = self.task_rows(selected.as_deref());
        // Over every project whatever the toggles show, as the tab and the status bar count.
        let projects = self
            .portfolio
            .projects
            .iter()
            .filter(|row| row.open_items() > 0)
            .count();
        let mut words = summary(self.open_tasks(), projects);
        if let (true, Some(name)) = (self.view.tasks_one, selected.as_deref()) {
            words.push_str(&format!(" Shown: {name} alone."));
        }
        ui.label(RichText::new(words).weak());
        ui.separator();
        let ticks = self.view.tasks_ticked.then_some(&self.ticks);
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if self.view.tasks_one && selected.is_none() {
                    ui.label(
                        RichText::new(
                            "Select a project in the list: while This project only is on, the \
                             pane shows it alone.",
                        )
                        .weak(),
                    );
                    return;
                }
                if rows.is_empty() {
                    ui.label(RichText::new("Nothing waits on you: no open owner task.").weak());
                    return;
                }
                for row in &rows {
                    project_block(ui, row, ticks, &mut actions);
                }
            });
        self.actions.extend(actions);
    }

    /// The projects the pane shows, in the list's order: those with an open task, an open older
    /// form or (the toggle on) a task ticked in the last week; with This project only, the
    /// selected one whatever it holds.
    fn task_rows(&self, selected: Option<&str>) -> Vec<&ProjectRow> {
        let (stacks, hosting) = (BTreeSet::new(), BTreeSet::new());
        let every = Filter {
            name: "",
            stacks: &stacks,
            hosting: &hosting,
        };
        let ticks = self.view.tasks_ticked.then_some(&self.ticks);
        self.portfolio
            .groups(self.view.sort, every)
            .into_iter()
            .flat_map(|group| group.members)
            .filter_map(|member| match member {
                Member::Project(i) => self.portfolio.projects.get(i),
                Member::Folder(_) => None,
            })
            .filter(|row| row.folder.is_some())
            .filter(|row| match (self.view.tasks_one, selected) {
                (true, Some(name)) => row.name().eq_ignore_ascii_case(name),
                (true, None) => false,
                (false, _) => holds_any(row, ticks),
            })
            .collect()
    }

    /// The open owner tasks of every project: the pane's tab and the status bar's sum.
    pub fn open_tasks(&self) -> usize {
        self.portfolio
            .projects
            .iter()
            .map(ProjectRow::open_items)
            .sum()
    }
}

/// The line under the toggles.
fn summary(open: usize, projects: usize) -> String {
    match (open, projects) {
        (0, _) => "No open owner task in any project.".to_owned(),
        (1, _) => "1 open owner task.".to_owned(),
        (n, 1) => format!("{n} open owner tasks in 1 project."),
        (n, p) => format!("{n} open owner tasks in {p} projects."),
    }
}

/// Whether the project has anything for the pane: an open task, open older forms where it keeps
/// no task file (a project that keeps one shows its older forms only under its tasks; EP-50's
/// review), or a task ticked in the last week while the toggle is on.
fn holds_any(row: &ProjectRow, ticks: Option<&Ticks>) -> bool {
    row.open_items() > 0
        || (!row.snapshot.keeps_tasks() && row.snapshot.older_open() > 0)
        || ticks.is_some_and(|ticks| !recently_ticked(row, ticks).is_empty())
}

/// The project's tasks ticked now whose latest tick the tool made in the last week, the latest
/// first.
fn recently_ticked(row: &ProjectRow, ticks: &Ticks) -> Vec<(OwnerItem, DateTime<Utc>)> {
    let mut ticked: Vec<(OwnerItem, DateTime<Utc>)> = row
        .snapshot
        .waiting()
        .into_iter()
        .filter(|item| !item.is_open())
        .filter_map(|item| {
            let at = ticks.ticked_at(row.name(), item.id.as_deref()?)?;
            Some((item, at))
        })
        .collect();
    ticked.sort_by_key(|(_, at)| std::cmp::Reverse(*at));
    ticked
}

/// The ids of the project's open tasks a chain into its next brief waits on: gated before the
/// next brief or a row before it (`prompt::due`, the autopilot's gate).
pub(crate) fn due(row: &ProjectRow) -> Vec<String> {
    let Some(roadmap) = row.snapshot.current() else {
        return Vec::new();
    };
    match roadmap.next() {
        Some(next) => crw_core::prompt::due(&row.snapshot, roadmap, next),
        None => Vec::new(),
    }
}

/// One project: its name (a click selects it in the list), its tasks with their boxes, the
/// tasks ticked in the last week when asked, and its older forms folded at its foot.
fn project_block(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    ticks: Option<&Ticks>,
    actions: &mut Vec<Action>,
) {
    let name = row.name();
    let folder = row
        .folder
        .as_ref()
        .map(|f| f.path.clone())
        .unwrap_or_default();
    let tasks = row.snapshot.waiting();
    let open: Vec<&OwnerItem> = tasks.iter().filter(|item| item.is_open()).collect();
    let due = due(row);
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        let state = row.project.state;
        if ui
            .add(
                egui::Label::new(RichText::new(name).strong())
                    .selectable(false)
                    .sense(egui::Sense::click()),
            )
            .on_hover_text("Select it in the list")
            .clicked()
        {
            actions.push(Action::Select(Selection::Project(name.to_owned())));
        }
        style::chip(ui, state.as_str(), style::state_colour(Some(state)));
        let words = ui.label(RichText::new(block_words(row, open.len(), due.len())).weak());
        if !row.snapshot.keeps_tasks() {
            words.on_hover_text(
                "No roadmap/owner-tasks.md, so nothing here is counted as waiting; Formalise… \
                 in the row's menu gives the project one (D-10)",
            );
        }
    });
    for item in &open {
        owner_item(ui, row, item, hold(row, item, &due), &folder, actions);
    }
    if let Some(ticks) = ticks {
        ticked(ui, row, ticks, actions);
    }
    older_forms(ui, row, &folder);
}

/// What the project's line says beside its name: the open tasks and how many hold its next
/// brief, or that it keeps no task file.
fn block_words(row: &ProjectRow, open: usize, due: usize) -> String {
    if !row.snapshot.keeps_tasks() {
        return "no task file".to_owned();
    }
    let tasks = match open {
        0 => "no open task".to_owned(),
        1 => "1 open task".to_owned(),
        n => format!("{n} open tasks"),
    };
    match due {
        0 => tasks,
        n => format!("{tasks}, {n} holding the next brief"),
    }
}

/// The tasks the tool ticked in the last week, each with Untick and when it was ticked.
fn ticked(ui: &mut egui::Ui, row: &ProjectRow, ticks: &Ticks, actions: &mut Vec<Action>) {
    let recent = recently_ticked(row, ticks);
    if recent.is_empty() {
        return;
    }
    ui.add_space(2.0);
    ui.label(RichText::new("Ticked in the last week").small().weak());
    for (item, at) in recent {
        ui.horizontal(|ui| {
            if let Some(id) = &item.id {
                if ui
                    .small_button("Untick")
                    .on_hover_text("Clear its box in owner-tasks.md")
                    .clicked()
                {
                    actions.push(Action::Untick {
                        project: row.name().to_owned(),
                        id: id.clone(),
                    });
                }
                ui.label(RichText::new(id).monospace());
            }
            let when = at
                .with_timezone(&Local)
                .format("%a %d %b %H:%M")
                .to_string();
            ui.label(RichText::new(format!("ticked {when}")).small().weak());
            let weak = Look {
                weak: true,
                ..Look::default()
            };
            let text = markup::job(ui, &markup::reflow(&item.text), weak);
            ui.add(egui::Label::new(text).truncate());
        });
    }
}

/// The project's older forms, folded shut: read-only, each with the lock, counted nowhere.
fn older_forms(ui: &mut egui::Ui, row: &ProjectRow, folder: &Path) {
    let open: Vec<&OwnerItem> = row
        .snapshot
        .uncounted()
        .iter()
        .filter(|item| item.is_open())
        .collect();
    if open.is_empty() {
        return;
    }
    egui::CollapsingHeader::new(
        RichText::new(format!(
            "Older forms: {} open, read-only, not counted",
            open.len()
        ))
        .weak(),
    )
    .id_salt(("older forms", row.name()))
    .default_open(false)
    .show(ui, |ui| {
        ui.label(
            RichText::new(
                "From the README and the older files (D-10): changed only in their files.",
            )
            .weak(),
        );
        ui.add_space(4.0);
        for item in open {
            owner_item(ui, row, item, Hold::Unplaced, folder, &mut Vec::new());
        }
    });
}

/// What an item's gate means for the project's next brief, which sets its chip's colour and
/// its hover (EP-50's review: an older form's gate holds no chain).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Hold {
    /// A task gated before the next brief or a row before it: a chain into it waits (amber).
    Next,
    /// A task gated before a row after the next brief (weak).
    Later,
    /// A gate the table cannot place against a next brief, or an older form's, which no chain
    /// reads: amber, as the Detail pane drew every gate, with the neutral words.
    Unplaced,
}

/// Where `item`'s gate stands against the project's next brief; `due` is [`due`]'s answer.
fn hold(row: &ProjectRow, item: &OwnerItem, due: &[String]) -> Hold {
    if !item.is_tickable() {
        return Hold::Unplaced;
    }
    if item.id.as_ref().is_some_and(|id| due.contains(id)) {
        return Hold::Next;
    }
    let Some(Gate::Before(gate)) = &item.gate else {
        return Hold::Unplaced;
    };
    let later = row.snapshot.current().and_then(|roadmap| {
        let next = roadmap.next()?;
        let at = |id| roadmap.rows().position(|row| row.id == id);
        Some(at(*gate)? > at(next.id)?)
    });
    if later == Some(true) {
        Hold::Later
    } else {
        Hold::Unplaced
    }
}

/// One open owner item: its box when it is a line of `owner-tasks.md`, the lock of an older
/// form, then its id and gate (as [`Hold`] says), its text and its meta line.
pub(crate) fn owner_item(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    item: &OwnerItem,
    hold: Hold,
    folder: &Path,
    actions: &mut Vec<Action>,
) {
    ui.horizontal_top(|ui| {
        if item.is_tickable() {
            let mut done = false;
            if ui
                .checkbox(&mut done, "")
                .on_hover_text("Tick it: the box changes in owner-tasks.md, nothing else")
                .clicked()
                && let Some(id) = &item.id
            {
                actions.push(Action::Tick {
                    project: row.name().to_owned(),
                    id: id.clone(),
                });
            }
        } else {
            ui.label(style::icon(
                icons::ICON_LOCK,
                Some(ui.visuals().weak_text_color()),
            ))
            .on_hover_text("An older form: shown here, changed only in its file");
        }
        ui.vertical(|ui| {
            // An id the text opens with (a register's `**R-2**`) is not said twice.
            let id = item
                .id
                .as_ref()
                .filter(|id| !item.text.contains(id.as_str()));
            if id.is_some() || item.gate.is_some() {
                ui.horizontal(|ui| {
                    if let Some(id) = id {
                        ui.label(RichText::new(id).monospace().strong());
                    }
                    if let Some(gate) = &item.gate {
                        gate_chip(ui, gate, hold);
                    }
                });
            }
            markup::long_note(
                ui,
                // By its id where it has one, so a line added above keeps the fold.
                (
                    row.name(),
                    &item.path,
                    item.id.as_deref(),
                    item.id.is_none().then_some(item.line),
                ),
                crate::detail::without_gate(&item.text, item.gate.as_ref()),
                Look {
                    sentences: true,
                    ..Look::default()
                },
            );
            ui.label(
                RichText::new(crate::detail::meta(item, folder))
                    .small()
                    .weak(),
            );
        });
    });
    ui.add_space(4.0);
}

/// An item's gate as a chip: weak for a later brief's, amber otherwise.
fn gate_chip(ui: &mut egui::Ui, gate: &Gate, hold: Hold) {
    let colour = match hold {
        Hold::Later => ui.visuals().weak_text_color(),
        Hold::Next | Hold::Unplaced => style::amber(ui),
    };
    style::chip(ui, &gate.to_string(), colour).on_hover_text(gate_hint(gate, hold));
}

/// What a gate's chip says on hover.
fn gate_hint(gate: &Gate, hold: Hold) -> String {
    match (gate, hold) {
        (Gate::Before(id), Hold::Next) => format!(
            "Due before {id} starts, the next brief or one before it: a chain into it holds \
             until this is ticked (D-10, D-22)"
        ),
        (Gate::Before(id), Hold::Later) => {
            format!("Due before {id} starts, a brief after the next (D-10)")
        }
        (Gate::Before(id), Hold::Unplaced) => format!("Due before {id} starts (D-10)"),
        (Gate::OnOrAfter(date), _) => format!("Not before {date}"),
        (Gate::When(words), _) => format!("At a decision point: {words}"),
    }
}

/// The Detail pane's line in place of its owner items (EP-50): how many tasks are open and how
/// many hold the next brief, or that the project keeps no task file; and Open in Owner tasks.
pub(crate) fn detail_line(ui: &mut egui::Ui, row: &ProjectRow, actions: &mut Vec<Action>) {
    ui.add_space(10.0);
    ui.label(RichText::new("Owner tasks").strong());
    ui.add_space(2.0);
    let older = row.snapshot.older_open();
    let words = if row.snapshot.keeps_tasks() {
        let open = row.open_items();
        let due = due(row).len();
        let tasks = match (open, due) {
            (0, _) => "No open owner task".to_owned(),
            (1, 1) => "1 open owner task, holding the next brief".to_owned(),
            (1, _) => "1 open owner task".to_owned(),
            (n, 0) => format!("{n} open owner tasks"),
            (n, d) => format!("{n} open owner tasks, {d} holding the next brief"),
        };
        match older {
            0 => format!("{tasks}."),
            n => format!("{tasks}; its older forms hold {n} open, not counted."),
        }
    } else {
        match older {
            0 => "No roadmap/owner-tasks.md, and nothing open in its older forms.".to_owned(),
            n => format!(
                "No roadmap/owner-tasks.md: its older forms hold {n} open, read-only and counted \
                 nowhere."
            ),
        }
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(words);
        if ui
            .small_button("Open in Owner tasks")
            .on_hover_text("The Owner tasks pane on this project alone: its tasks ticked there")
            .clicked()
        {
            actions.push(Action::OpenTasks(row.name().to_owned()));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::testing::{self, Screen};
    use crw_core::tasks::{Form, State};

    /// Whether `output` paints a text that contains `part`.
    fn shows(output: &egui::FullOutput, part: &str) -> bool {
        output.shapes.iter().any(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) => shape.galley.text().contains(part),
            _ => false,
        })
    }

    fn risk(folder: &Path, line: usize, id: &str, text: &str, state: State) -> OwnerItem {
        OwnerItem {
            form: Form::Risk,
            path: folder.join("roadmap").join("README.md"),
            line,
            id: Some(id.to_owned()),
            text: text.to_owned(),
            gate: None,
            state,
            raised: None,
            date: None,
        }
    }

    /// The fixtures' harbor keeps two open tasks and a ticked one; an older form open and one
    /// resolved are added to it.
    fn harbor_with_risks(scratch: &Scratch) -> ProjectRow {
        let portfolio = crate::portfolio::read_files(&fake_store(scratch));
        let mut harbor = portfolio.project("harbor").unwrap().clone();
        let folder = harbor.snapshot.folder.clone();
        harbor.snapshot.legacy = vec![
            risk(
                &folder,
                60,
                "1",
                "**The tide cache goes stale.** Read again in March.",
                State::Open,
            ),
            risk(
                &folder,
                61,
                "2",
                "~~**The CSV's encoding.**~~",
                State::Resolved,
            ),
        ];
        harbor
    }

    /// A project's block: its open tasks with their boxes, the older forms folded shut and
    /// counted nowhere, the ticked task left out until the toggle asks.
    #[test]
    fn a_project_block_lists_its_open_tasks_and_folds_its_older_forms() {
        let scratch = Scratch::new("tasks-block");
        let harbor = harbor_with_risks(&scratch);
        assert_eq!(harbor.open_items(), 2);
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            project_block(ui, &harbor, None, &mut Vec::new());
        });
        assert!(shows(&output, "harbor"));
        assert!(shows(&output, "2 open tasks"));
        assert!(shows(
            &output,
            "Older forms: 1 open, read-only, not counted"
        ));
        // Folded shut: the risk's text is not drawn.
        assert!(!shows(&output, "The tide cache goes stale."));
        assert!(!shows(&output, "Ticked in the last week"));
        // The open tasks, not the ticked one.
        assert!(testing::text_at(&output, "OT-2").is_some());
        assert!(testing::text_at(&output, "OT-3").is_some());
        assert!(testing::text_at(&output, "OT-1").is_none());
        // Its name selects it in the list.
        let at = testing::text_at(&output, "harbor").expect("its name");
        let mut actions = Vec::new();
        for events in [
            testing::moved(at),
            testing::button(at, egui::PointerButton::Primary, true),
            testing::button(at, egui::PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| project_block(ui, &harbor, None, &mut actions));
        }
        assert_eq!(
            actions,
            [Action::Select(Selection::Project("harbor".to_owned()))]
        );
    }

    /// A project without the file shows no count and its older forms folded; the list's mark
    /// says how many and that formalising gives it a file (D-10 addendum of 2026-11-22).
    #[test]
    fn a_project_without_a_task_file_shows_its_older_forms_alone() {
        let scratch = Scratch::new("tasks-no-file");
        let mut harbor = harbor_with_risks(&scratch);
        harbor.snapshot.tasks = None;
        assert_eq!(harbor.open_items(), 0);
        assert_eq!(harbor.snapshot.older_open(), 1);
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            project_block(ui, &harbor, None, &mut Vec::new());
        });
        assert!(shows(&output, "no task file"));
        assert!(shows(
            &output,
            "Older forms: 1 open, read-only, not counted"
        ));
        let hover = crate::list::older_forms_words(&harbor).expect("the mark's words");
        assert!(
            hover.contains("hold 1 open item, read-only and counted nowhere"),
            "{hover}"
        );
        assert!(
            hover.contains("Formalise… gives the project a task file"),
            "{hover}"
        );
        // A project that keeps the file, or whose older forms are all resolved, has no mark.
        let kept = harbor_with_risks(&scratch);
        assert!(crate::list::older_forms_words(&kept).is_none());
        harbor.snapshot.legacy.retain(|item| !item.is_open());
        assert!(crate::list::older_forms_words(&harbor).is_none());
    }

    /// The Detail pane's line says how many wait and leads to the pane on the project alone.
    #[test]
    fn the_detail_line_counts_the_tasks_and_opens_the_pane() {
        let scratch = Scratch::new("tasks-detail-line");
        let mut harbor = harbor_with_risks(&scratch);
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| detail_line(ui, &harbor, &mut Vec::new()));
        assert!(shows(&output, "2 open owner tasks"), "the fixture's two");
        assert!(shows(&output, "its older forms hold 1 open, not counted."));
        let at = testing::text_at(&output, "Open in Owner tasks").expect("its button");
        let mut actions = Vec::new();
        for events in [
            testing::moved(at),
            testing::button(at, egui::PointerButton::Primary, true),
            testing::button(at, egui::PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| detail_line(ui, &harbor, &mut actions));
        }
        assert_eq!(actions, [Action::OpenTasks("harbor".to_owned())]);
        harbor.snapshot.tasks = None;
        let output = screen.pass(Vec::new(), |ui| detail_line(ui, &harbor, &mut Vec::new()));
        assert!(shows(
            &output,
            "No roadmap/owner-tasks.md: its older forms hold 1 open, read-only and counted nowhere."
        ));
    }

    /// The ledger's ticks: the latest of each owner task in the last week, a clear undoing a
    /// tick, a message's tick and a delivery left out.
    #[test]
    fn the_ledgers_ticks_give_the_last_weeks_latest_tick_of_each_task() {
        let scratch = Scratch::new("tasks-ticks");
        let path = scratch.0.join("ledger.sqlite");
        let ledger = Ledger::open(&path).unwrap();
        let now = Utc::now();
        let event = |ago: i64, project: &str, detail: &str| {
            ledger
                .add_event(
                    now - TimeDelta::hours(ago),
                    TICK_KIND,
                    Some(project),
                    detail,
                )
                .unwrap();
        };
        event(200, "harbor", "ticked OT-1"); // more than a week ago
        event(30, "harbor", "ticked OT-2");
        event(20, "harbor", "ticked OT-3");
        event(10, "harbor", "cleared OT-3");
        event(5, "Lantern", "ticked IN-2 · rolled into EP-4");
        event(4, "lantern", "delivered IN-3 · (from harbor EP-2) words");
        event(3, "Lantern", "ticked OT-7");
        let mut ticks = Ticks::default();
        ticks.refresh(Some(&path), Instant::now(), now - WEEK);
        assert_eq!(ticks.ticked_at("harbor", "OT-1"), None);
        assert!(ticks.ticked_at("harbor", "OT-2").is_some());
        assert_eq!(ticks.ticked_at("harbor", "OT-3"), None, "cleared since");
        assert_eq!(ticks.ticked_at("lantern", "IN-2"), None);
        assert_eq!(ticks.ticked_at("lantern", "IN-3"), None);
        assert!(ticks.ticked_at("LANTERN", "OT-7").is_some());
        // Read again only when a minute old or stale.
        event(1, "harbor", "ticked OT-4");
        ticks.refresh(Some(&path), Instant::now(), now - WEEK);
        assert_eq!(ticks.ticked_at("harbor", "OT-4"), None);
        ticks.stale();
        ticks.refresh(Some(&path), Instant::now(), now - WEEK);
        assert!(ticks.ticked_at("harbor", "OT-4").is_some());
    }

    /// With the toggle on, a task ticked in the last week shows with Untick and its time.
    #[test]
    fn a_task_ticked_in_the_last_week_shows_with_untick() {
        let scratch = Scratch::new("tasks-ticked");
        let harbor = harbor_with_risks(&scratch);
        let done = harbor
            .snapshot
            .waiting()
            .into_iter()
            .find(|item| !item.is_open())
            .expect("the fixture's ticked task");
        let id = done.id.clone().unwrap();
        let mut ticks = Ticks::default();
        ticks
            .latest
            .insert(("harbor".to_owned(), id.clone()), (Utc::now(), true));
        assert_eq!(recently_ticked(&harbor, &ticks).len(), 1);
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            project_block(ui, &harbor, Some(&ticks), &mut Vec::new());
        });
        assert!(shows(&output, "Ticked in the last week"));
        let at = testing::text_at(&output, "Untick").expect("its button");
        let mut actions = Vec::new();
        for events in [
            testing::moved(at),
            testing::button(at, egui::PointerButton::Primary, true),
            testing::button(at, egui::PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| {
                project_block(ui, &harbor, Some(&ticks), &mut actions);
            });
        }
        assert_eq!(
            actions,
            [Action::Untick {
                project: "harbor".to_owned(),
                id
            }]
        );
    }

    /// A gate holds the next brief when it names it or a row before it, as the autopilot's gate
    /// reads it; a later brief's gate does not.
    #[test]
    fn a_gate_holds_the_next_brief_by_the_autopilots_rule() {
        use crw_core::roadmap::EpId;

        let scratch = Scratch::new("tasks-due");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let mut harbor = portfolio.project("harbor").unwrap().clone();
        // The next brief is EP-4, and OT-2 is due before EP-5: it holds nothing yet.
        assert_eq!(harbor.next().map(|row| row.id), Some(EpId::new(4)));
        assert!(due(&harbor).is_empty());
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| detail_line(ui, &harbor, &mut Vec::new()));
        assert!(shows(&output, "2 open owner tasks."));
        // Gated before the next brief, it holds it.
        for task in &mut harbor.snapshot.tasks.as_mut().unwrap().tasks {
            if task.id == "OT-2" {
                task.gate = Some(EpId::new(4));
            }
        }
        assert_eq!(due(&harbor), ["OT-2"]);
        let output = screen.pass(Vec::new(), |ui| detail_line(ui, &harbor, &mut Vec::new()));
        assert!(shows(
            &output,
            "2 open owner tasks, 1 holding the next brief"
        ));
        let output = screen.pass(Vec::new(), |ui| {
            project_block(ui, &harbor, None, &mut Vec::new());
        });
        assert!(shows(&output, "2 open tasks, 1 holding the next brief"));
        // Each item's hold: OT-2 the next brief's; a later row's gate weak; an older form's,
        // or a gate on a row the table lacks, unplaced, with the neutral words.
        let due = due(&harbor);
        let tasks = harbor.snapshot.waiting();
        let ot2 = tasks
            .iter()
            .find(|i| i.id.as_deref() == Some("OT-2"))
            .unwrap();
        assert_eq!(hold(&harbor, ot2, &due), Hold::Next);
        let mut later = ot2.clone();
        later.id = Some("OT-9".to_owned());
        later.gate = Some(Gate::Before(EpId::new(5)));
        assert_eq!(hold(&harbor, &later, &due), Hold::Later);
        later.gate = Some(Gate::Before(EpId::new(99)));
        assert_eq!(hold(&harbor, &later, &due), Hold::Unplaced);
        let mut older = ot2.clone();
        older.form = Form::OpenQuestion;
        assert_eq!(hold(&harbor, &older, &due), Hold::Unplaced);
        let next = EpId::new(4);
        assert!(gate_hint(&Gate::Before(next), Hold::Next).contains("a chain into it holds"));
        assert!(gate_hint(&Gate::Before(next), Hold::Later).contains("a brief after the next"));
        assert_eq!(
            gate_hint(&Gate::Before(next), Hold::Unplaced),
            "Due before EP-4 starts (D-10)"
        );
    }

    /// A board over the fake portfolio, its files read (as `published`'s tests build one).
    fn board(scratch: &Scratch) -> Board {
        use crate::board::{Places, View};
        let places = Places {
            store: fake_store(scratch),
            ledger: None,
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        let mut board = Board::new(places, &ctx, View::default(), egui::ThemePreference::Light);
        let deadline = Instant::now() + Duration::from_secs(10);
        while board.loader.reading_files && Instant::now() < deadline {
            board.receive(&ctx);
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!board.loader.reading_files, "the files were read");
        board
    }

    /// The pane's projects: those with an open task and those without a task file whose older
    /// forms are open, in the list's order whatever its filters; This project only, the selected
    /// one alone; Open in Owner tasks selects the project and sets it.
    #[test]
    fn the_pane_lists_the_projects_in_the_lists_order_and_one_alone() {
        let scratch = Scratch::new("tasks-rows");
        let mut board = board(&scratch);
        let names = |board: &Board, selected: Option<&str>| -> Vec<String> {
            board
                .task_rows(selected)
                .iter()
                .map(|row| row.name().to_owned())
                .collect()
        };
        // harbor keeps two open tasks; compass and orchard keep no task file and hold open
        // older forms (the first picture's three); the rest hold nothing for the pane.
        let all = names(&board, None);
        assert_eq!(all, ["harbor", "compass", "orchard"]);
        assert_eq!(board.open_tasks(), 2);
        // A name filter in the list hides nothing here.
        board.view.filter = "lantern".to_owned();
        assert_eq!(names(&board, None), all);
        // This project only: the selected project, whatever it holds.
        board.view.tasks_one = true;
        assert_eq!(names(&board, Some("lantern")), ["lantern"]);
        assert!(names(&board, None).is_empty());
        // The Detail pane's button: the project selected, the pane on it alone.
        board.view.tasks_one = false;
        board.run(Action::OpenTasks("orchard".to_owned()));
        assert!(board.view.tasks_one);
        assert_eq!(
            board.view.selected,
            Some(Selection::Project("orchard".to_owned()))
        );
    }

    #[test]
    fn the_summary_counts_tasks_and_projects() {
        assert_eq!(summary(0, 0), "No open owner task in any project.");
        assert_eq!(summary(1, 1), "1 open owner task.");
        assert_eq!(summary(3, 1), "3 open owner tasks in 1 project.");
        assert_eq!(summary(5, 2), "5 open owner tasks in 2 projects.");
    }
}
