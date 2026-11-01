//! The Detail pane (D-25): everything the files say about the selected project — its proposals,
//! its next brief with the brief notes verbatim, why a session is taken to be in progress, the
//! progress per phase, its badges spelt out (EP-7), what waits on the owner (ticked here when it
//! lives in `owner-tasks.md`), the inbox (ticked here with a disposition), the mail its briefs
//! hand on, and the problems the readers met. For an unadopted folder: what it holds, Adopt and
//! Ignore.

use chrono::Utc;
use crw_core::badges::short_age;
use crw_core::inbox::Recipient;
use crw_core::roadmap::snapshot::Snapshot;
use crw_core::roadmap::table::Done;
use crw_core::tasks::{self, Form, OwnerItem};
use eframe::egui::{self, RichText};

use crate::badges::{self, Mark};
use crate::board::{Action, Board};
use crate::markup::{self, Look};
use crate::portfolio::{Portfolio, ProjectRow, Selection};
use crate::style::{self, icons};

impl Board {
    pub fn detail_pane(&mut self, ui: &mut egui::Ui) {
        let mut actions = Vec::new();
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| match self.view.selected.clone() {
                Some(Selection::Project(name)) => match self.portfolio.project(&name) {
                    Some(row) => project(ui, row, &self.portfolio, &mut self.drafts, &mut actions),
                    None => gone(ui, &name),
                },
                Some(Selection::Folder(name)) => match self.portfolio.folder(&name) {
                    Some(folder) => {
                        ui.horizontal(|ui| {
                            ui.heading(&folder.name);
                            style::chip(ui, "unadopted", style::state_colour(None));
                        });
                        ui.label(RichText::new(folder.path.display().to_string()).weak());
                        ui.add_space(6.0);
                        ui.label(match (folder.git, folder.roadmap) {
                            (true, true) => "A git repository with roadmap/README.md.",
                            (true, false) => "A git repository without roadmap/README.md.",
                            (false, _) => "A folder that is not a git repository.",
                        });
                        ui.label(format!(
                            "Adopt registers it as {} and adds it to the workspace file; Ignore \
                             leaves it out of the list (D-7).",
                            folder.first_state()
                        ));
                        ui.horizontal(|ui| {
                            if ui.button("Adopt").clicked() {
                                actions.push(Action::Adopt(folder.name.clone()));
                            }
                            if ui.button("Ignore").clicked() {
                                actions.push(Action::Ignore(folder.name.clone()));
                            }
                        });
                    }
                    None => gone(ui, &name),
                },
                None => {
                    ui.add_space(24.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Select a project in the list (↑ ↓, or a click).").weak(),
                        )
                    });
                }
            });
        self.actions.extend(actions);
    }
}

fn gone(ui: &mut egui::Ui, name: &str) {
    ui.label(RichText::new(format!("{name} is no longer in the list.")).weak());
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(10.0);
    ui.label(RichText::new(title).strong());
    ui.add_space(2.0);
}

fn project(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    portfolio: &Portfolio,
    drafts: &mut std::collections::HashMap<(String, String), String>,
    actions: &mut Vec<Action>,
) {
    let name = row.name();
    let snapshot = &row.snapshot;
    ui.horizontal(|ui| {
        ui.heading(name);
        let state = row.project.state;
        style::chip(ui, state.as_str(), style::state_colour(Some(state)));
        if row.project.pinned {
            ui.label(style::icon(icons::ICON_PUSH_PIN, None))
                .on_hover_text("Pinned");
        }
        ui.label(RichText::new(format!("priority {}", row.project.priority)).weak());
    });
    ui.label(RichText::new(row.path.display().to_string()).weak());
    if row.folder.is_none() {
        ui.label(
            RichText::new("The folder is missing: archive the project, or bring the folder back.")
                .color(style::red(ui)),
        );
        return;
    }

    let proposals = row.proposals(chrono::Utc::now());
    for proposal in &proposals {
        ui.add_space(6.0);
        egui::Frame::new()
            .fill(style::amber(ui).gamma_multiply(0.12))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(style::icon(icons::ICON_LIGHTBULB, Some(style::amber(ui))));
                    ui.label(format!("Proposed, because {}:", proposal.reason));
                    for &state in proposal.to {
                        if ui
                            .button(format!("Set {state}"))
                            .on_hover_text("Confirm: the registry's state changes (D-6)")
                            .clicked()
                        {
                            actions.push(Action::SetState(name.to_owned(), state));
                        }
                    }
                });
            });
    }

    next_brief(ui, row);
    progress(ui, snapshot);
    status(ui, row, portfolio);
    owner_items(ui, row, actions);
    inbox(ui, row, drafts, actions);
    handed_on(ui, row);
    decisions(ui, snapshot);
    problems(ui, row);
}

fn next_brief(ui: &mut egui::Ui, row: &ProjectRow) {
    let snapshot = &row.snapshot;
    let Some(roadmap) = snapshot.current() else {
        section(ui, "Roadmap");
        ui.label(
            RichText::new("No roadmap/README.md: the project is not in the house style yet.")
                .weak(),
        );
        return;
    };
    let Some(next) = roadmap.next() else {
        section(ui, "Next brief");
        let text = if roadmap.counts().all_checked() {
            "Every row is checked."
        } else {
            "No phase table was read."
        };
        ui.label(RichText::new(text).weak());
        return;
    };
    section(ui, "Next brief");
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(next.id.to_string()).monospace().strong());
        if let Some(size) = next.size {
            ui.label(RichText::new(size.as_str()).strong());
        }
        ui.label(&next.brief.title);
    });
    let phase = roadmap
        .phase_of(next.id)
        .map(|phase| {
            let heading = &phase.heading;
            format!(
                "Phase {} {} · ",
                heading.id.as_deref().unwrap_or("-"),
                heading.name
            )
        })
        .unwrap_or_default();
    let depends = match next.depends.text.trim() {
        "" => "—",
        text => text,
    };
    // The next EP is open by definition; a `◐` or `[~]` cell is worth saying.
    let done = match &next.done {
        Done::Open => String::new(),
        other => format!(" · {other}"),
    };
    ui.label(RichText::new(format!("{phase}depends on {depends}{done}")).weak());
    if let Some(entry) = snapshot.entry(next.id) {
        let brief = &entry.brief;
        let mut parts = vec![entry.file.name.clone()];
        if let Some(front) = &brief.front {
            if let (Some(model), Some(effort)) = (front.model(), front.effort()) {
                parts.push(format!("{model}/{effort}"));
            }
            if let Some(status) = front.status() {
                parts.push(status.to_owned());
            }
        }
        parts.push(match brief.completion() {
            Some(evidence) => format!("complete: {evidence}"),
            None => "no completion note".to_owned(),
        });
        ui.label(RichText::new(format!("Brief {}", parts.join(" · "))).weak());
        if let Some(gate) = brief.pickup_gate() {
            ui.label(
                RichText::new(format!(
                    "Pickup gate (line {}): owner input before it starts.",
                    gate.line
                ))
                .color(style::amber(ui)),
            );
        }
    } else {
        ui.label(RichText::new("No brief file yet.").weak());
    }

    let reasons = row.in_progress();
    if !reasons.is_empty() {
        let texts: Vec<String> = reasons.iter().map(ToString::to_string).collect();
        ui.label(
            RichText::new(format!("Session in progress: {}.", texts.join("; ")))
                .color(style::amber(ui)),
        );
    } else if row.awaits_git() {
        ui.label(RichText::new("Git has not been asked yet.").weak());
    }
    if snapshot.differs()
        && let Some(head) = &snapshot.head
    {
        let counts = head.counts();
        let next = head
            .next()
            .map_or("none".to_owned(), |row| row.id.to_string());
        ui.label(
            RichText::new(format!(
                "At HEAD: {} of {} done, next {next}; the working tree is shown.",
                counts.done, counts.total
            ))
            .weak(),
        );
    }

    match roadmap.note(next.id) {
        Some(note) => {
            // The bullet's own label repeats the id shown above.
            let label = format!("**{}.**", next.id);
            let text = note.text.strip_prefix(&label).unwrap_or(&note.text);
            ui.add_space(4.0);
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(format!("Brief notes (README line {})", note.line))
                            .small()
                            .weak(),
                    );
                    markup::label(ui, &markup::reflow(text), Look::default());
                });
        }
        None => {
            ui.label(RichText::new("No brief notes for it.").weak());
        }
    }
}

fn progress(ui: &mut egui::Ui, snapshot: &Snapshot) {
    let Some(roadmap) = snapshot.current() else {
        return;
    };
    let counts = roadmap.counts();
    section(
        ui,
        &format!(
            "Progress: {} of {} done{}",
            counts.done,
            counts.total,
            match counts.stretch {
                0 => String::new(),
                n => format!(", {n} stretch"),
            }
        ),
    );
    egui::Grid::new("phases")
        .num_columns(3)
        .spacing([10.0, 3.0])
        .show(ui, |ui| {
            for phase in &roadmap.phases {
                let heading = &phase.heading;
                let counts = phase.counts();
                ui.label(RichText::new(heading.id.as_deref().unwrap_or("-")).monospace());
                ui.label(match counts.total {
                    0 => "no rows".to_owned(),
                    total => format!("{} of {total}", counts.done),
                });
                let status = heading
                    .status
                    .as_deref()
                    .map(|s| format!(" ({s})"))
                    .unwrap_or_default();
                ui.label(RichText::new(format!("{}{status}", heading.name)).weak());
                ui.end_row();
            }
        });
}

/// One badge spelt out: its mark, then its sentence.
fn mark_line(ui: &mut egui::Ui, mark: &Mark) {
    ui.horizontal_wrapped(|ui| {
        mark.show(ui);
        ui.label(&mark.hover);
    });
}

fn weak_line(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.add(egui::Label::new(RichText::new(text.into()).weak()).wrap());
}

/// The badges spelt out (D-24, D-25): stack, activity, hosting with the head check, health and
/// backup, each with where it comes from and how old it is.
fn status(ui: &mut egui::Ui, row: &ProjectRow, portfolio: &Portfolio) {
    let now = Utc::now();
    let badges = &row.badges;
    section(ui, "Status");
    egui::Grid::new(("status", row.name()))
        .num_columns(2)
        .spacing([14.0, 5.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Stack").weak());
            ui.horizontal_wrapped(|ui| {
                if badges.stack.is_empty() {
                    ui.label(RichText::new("No manifest at the root or one folder down").weak());
                }
                for chip in &badges.stack {
                    style::chip(ui, &chip.text(), style::stack_colour(chip.kind))
                        .on_hover_text(&chip.source);
                }
            });
            ui.end_row();

            ui.label(RichText::new("Activity").weak());
            ui.vertical(|ui| match badges::activity(row, now) {
                Some((age, dirty)) => {
                    mark_line(ui, &age);
                    match dirty {
                        Some(dirty) => mark_line(ui, &dirty),
                        None => weak_line(ui, "A clean working tree"),
                    }
                }
                None if row.awaits_git() => weak_line(ui, "Git has not been asked yet"),
                None => weak_line(ui, "Not a git repository"),
            });
            ui.end_row();

            ui.label(RichText::new("Hosting").weak());
            ui.vertical(|ui| {
                mark_line(ui, &badges::hosting(row, now));
                if let Some(remote) = &badges.remote {
                    weak_line(ui, format!("origin {remote}"));
                }
                if let Some(mark) = badges::sync(row, now) {
                    mark_line(ui, &mark);
                }
                match &portfolio.account {
                    Some(fetched) => {
                        let age = short_age(now - fetched.at);
                        match &fetched.value {
                            Ok(_) => weak_line(
                                ui,
                                format!(
                                    "GitHub read {age} ago; it is asked daily and on F5, never \
                                     fetched from"
                                ),
                            ),
                            Err(e) => weak_line(ui, format!("GitHub could not be asked: {e}")),
                        }
                    }
                    None => weak_line(ui, "GitHub not read yet: it is asked daily and on F5"),
                }
            });
            ui.end_row();

            ui.label(RichText::new("Health").weak());
            ui.vertical(|ui| {
                match badges::ci(row, now) {
                    Some(mark) => mark_line(ui, &mark),
                    None => weak_line(ui, "No GitHub Actions workflows"),
                }
                match (&badges.licence, badges::licence_warning(row)) {
                    (_, Some(warning)) => mark_line(ui, &warning),
                    (Some(file), None) => weak_line(ui, format!("Licence: {file}")),
                    (None, None) => weak_line(ui, "No licence file at the root"),
                }
            });
            ui.end_row();

            ui.label(RichText::new("Backup").weak());
            ui.vertical(|ui| {
                match badges::backup(row, now) {
                    Some(mark) => mark_line(ui, &mark),
                    None if portfolio.backup.is_none() => weak_line(
                        ui,
                        "No backup job is set: crw setting backup_settings <its settings file>",
                    ),
                    None => weak_line(ui, "Not read yet"),
                }
                if let Some(backup) = &badges.backup
                    && !backup.excluded.is_empty()
                {
                    weak_line(
                        ui,
                        format!("The job leaves out: {}", backup.excluded.join(", ")),
                    );
                }
                if let Some(job) = &portfolio.backup {
                    let target = match &job.job {
                        Ok(settings) => format!(
                            "target {} ({})",
                            settings.target.display(),
                            if job.mounted {
                                "mounted"
                            } else {
                                "not mounted"
                            }
                        ),
                        Err(e) => format!("its settings could not be read: {e}"),
                    };
                    let run = match job.last_run() {
                        Some(run) => format!(
                            "; the last run ended {} ago: {}",
                            short_age(now - run.at),
                            run.status
                        ),
                        None => String::new(),
                    };
                    weak_line(ui, format!("The job's {target}{run}"));
                }
            });
            ui.end_row();
        });
}

/// The item's meta line: its form, file and line, gate, state and origin.
fn meta(item: &OwnerItem, folder: &std::path::Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    if item.form != Form::Task {
        parts.push(item.form.to_string());
    }
    let file = item.path.strip_prefix(folder).unwrap_or(&item.path);
    parts.push(format!("{}:{}", file.display(), item.line));
    if let Some(gate) = &item.gate {
        parts.push(gate.to_string());
    }
    if item.state == tasks::State::Partly {
        parts.push("partly resolved".to_owned());
    }
    match (item.raised, &item.date) {
        (Some(ep), Some(date)) => parts.push(format!("{ep}, {date}")),
        (Some(ep), None) => parts.push(ep.to_string()),
        (None, Some(date)) => parts.push(date.clone()),
        (None, None) => {}
    }
    parts.join(" · ")
}

fn owner_items(ui: &mut egui::Ui, row: &ProjectRow, actions: &mut Vec<Action>) {
    let items = row.snapshot.owner_items();
    let folder = row
        .folder
        .as_ref()
        .map(|f| f.path.clone())
        .unwrap_or_default();
    let open: Vec<&OwnerItem> = items.iter().filter(|item| item.is_open()).collect();
    let resolved = items.len() - open.len();
    section(
        ui,
        &format!(
            "Waiting on the owner: {} open, {resolved} resolved",
            open.len()
        ),
    );
    if items.is_empty() {
        ui.label(RichText::new("Nothing: no owner tasks and none of the older forms.").weak());
        return;
    }
    for item in &open {
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
                let text = match &item.id {
                    // An id the text opens with (a register's `**R-2**`) is not said twice.
                    Some(id) if !item.text.contains(id.as_str()) => {
                        format!("`{id}` {}", markup::reflow(&item.text))
                    }
                    _ => markup::reflow(&item.text),
                };
                markup::label(ui, &text, Look::default());
                ui.label(RichText::new(meta(item, &folder)).small().weak());
            });
        });
    }
    let ticked: Vec<&OwnerItem> = items
        .iter()
        .filter(|item| !item.is_open() && item.is_tickable())
        .collect();
    if !ticked.is_empty() {
        egui::CollapsingHeader::new(format!("Ticked owner tasks ({})", ticked.len()))
            .id_salt(("ticked", row.name()))
            .show(ui, |ui| {
                for item in ticked {
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
                        ui.add(egui::Label::new(RichText::new(&item.text).weak()).truncate());
                    });
                }
            });
    }
}

fn inbox(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    drafts: &mut std::collections::HashMap<(String, String), String>,
    actions: &mut Vec<Action>,
) {
    let Some(inbox) = &row.snapshot.inbox else {
        section(ui, "Inbox");
        ui.label(RichText::new("No roadmap/inbox.md.").weak());
        return;
    };
    let open: Vec<_> = inbox.open().collect();
    section(
        ui,
        &format!("Inbox: {} open of {}", open.len(), inbox.messages.len()),
    );
    for message in open {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(&message.id).monospace().strong());
            if message.urgent {
                style::chip(ui, "urgent", style::red(ui));
            }
            let from = [message.from.clone(), message.date.clone()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ");
            if !from.is_empty() {
                ui.label(RichText::new(format!("from {from}")).weak());
            }
        });
        markup::label(ui, &markup::reflow(&message.text), Look::default());
        let key = (row.name().to_owned(), message.id.clone());
        let draft = drafts.entry(key).or_default();
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(draft)
                    .hint_text("Disposition: rolled into EP-n, parked in …, declined because …")
                    .desired_width(ui.available_width() - 70.0),
            );
            let ready = !draft.trim().is_empty();
            if ui
                .add_enabled(ready, egui::Button::new("Tick"))
                .on_hover_text("Write the disposition on its line and tick it (D-13)")
                .on_disabled_hover_text("A disposition first: it is written on the message's line")
                .clicked()
            {
                actions.push(Action::TickMessage {
                    project: row.name().to_owned(),
                    id: message.id.clone(),
                });
            }
        });
        ui.add_space(4.0);
    }
    let done: Vec<_> = inbox.messages.iter().filter(|m| m.done).collect();
    if !done.is_empty() {
        egui::CollapsingHeader::new(format!("Ticked messages ({})", done.len()))
            .id_salt(("ticked messages", row.name()))
            .show(ui, |ui| {
                for message in done {
                    let text = format!("`{}` {}", message.id, markup::reflow(&message.text));
                    markup::label(
                        ui,
                        &text,
                        Look {
                            weak: true,
                            ..Look::default()
                        },
                    );
                    if let Some(disposition) = &message.disposition {
                        markup::label(
                            ui,
                            &markup::reflow(disposition),
                            Look {
                                small: true,
                                ..Look::default()
                            },
                        );
                    }
                    ui.add_space(4.0);
                }
            });
    }
}

fn handed_on(ui: &mut egui::Ui, row: &ProjectRow) {
    if row.outgoing.is_empty() {
        return;
    }
    section(
        ui,
        &format!("Handed on to other projects: {}", row.outgoing.len()),
    );
    for block in &row.outgoing {
        ui.horizontal_wrapped(|ui| {
            ui.label(style::icon(icons::ICON_OUTBOX, None));
            ui.label(RichText::new(format!("{} → {}", block.brief, block.to)).strong());
            match &block.recipient {
                Recipient::Project(name) => {
                    ui.label(
                        RichText::new(format!("{name} is registered; delivery arrives at EP-20"))
                            .weak(),
                    );
                }
                Recipient::Unknown => {
                    ui.label(
                        RichText::new("not registered: shown, not delivered")
                            .color(style::amber(ui)),
                    );
                }
            }
        });
    }
}

fn decisions(ui: &mut egui::Ui, snapshot: &Snapshot) {
    let decisions = &snapshot.decisions;
    let Some(path) = &decisions.path else {
        return;
    };
    section(ui, "Decision log");
    let name = path.strip_prefix(&snapshot.folder).unwrap_or(path);
    let last = decisions
        .log
        .last_addendum()
        .map(|date| format!(", the last {date}"))
        .unwrap_or_default();
    ui.label(format!(
        "{}: {} decisions, {} addenda{last}",
        name.display(),
        decisions.log.entries.len(),
        decisions.log.addenda.len()
    ));
}

fn problems(ui: &mut egui::Ui, row: &ProjectRow) {
    let snapshot = &row.snapshot;
    let folder = &snapshot.folder;
    let relative = |path: &std::path::Path| {
        path.strip_prefix(folder)
            .unwrap_or(path)
            .display()
            .to_string()
    };
    let mut lines: Vec<String> = Vec::new();
    if let Some(roadmap) = snapshot.current() {
        lines.extend(
            roadmap
                .problems
                .iter()
                .map(|p| format!("roadmap/README.md {p}")),
        );
    }
    lines.extend(snapshot.problems.iter().cloned());
    if let Some(file) = &snapshot.tasks {
        let path = relative(&file.path);
        lines.extend(file.problems.iter().map(|p| format!("{path} {p}")));
    }
    if let Some(file) = &snapshot.inbox {
        let path = relative(&file.path);
        lines.extend(file.problems.iter().map(|p| format!("{path} {p}")));
    }
    if let Some(problem) = &snapshot.decisions.problem {
        lines.push(format!("DECISIONS.md: {problem}"));
    }
    if lines.is_empty() {
        return;
    }
    section(ui, &format!("Problems the readers met: {}", lines.len()));
    for line in lines {
        ui.add(egui::Label::new(RichText::new(line).color(style::amber(ui))).wrap());
    }
}
