//! The Detail pane (D-25): everything the files say about the selected project — its proposals
//! (confirmed or dismissed here, a dismissed one restored), its next brief with the brief notes
//! verbatim, why a session is taken to be in progress, the progress per phase, its badges spelt
//! out (EP-7) with this week's usage share (EP-18b), a line saying what waits on the owner that
//! leads to the Owner tasks pane, where the tasks are ticked (EP-50), and one saying what its
//! inbox holds and what its briefs hand on that leads to the Inbox pane (EP-51). After Status
//! three headers fold as the owner left them, saved with the view (EP-51): the decision log and
//! the house style shut by default, the problems the readers met open. For an unadopted folder:
//! what it holds, Adopt and Ignore. From EP-12 a session that runs there, by the window's or
//! Claude Code's account, joins the in-progress reasons, which from EP-13 show whatever the
//! roadmap holds.

use std::collections::BTreeMap;

use chrono::{Local, Utc};
use crw_core::badges::short_age;
use crw_core::roadmap::brief::Kind;
use crw_core::roadmap::snapshot::{InProgress, Snapshot};
use crw_core::roadmap::table::Done;
use crw_core::tasks::{self, Form, Gate, OwnerItem};
use eframe::egui::{self, RichText};

use crate::analytics::{self, Analytics, Measured};
use crate::badges::{self, Mark};
use crate::board::{Action, Board};
use crate::markup::{self, Look};
use crate::meter::Week;
use crate::portfolio::{Portfolio, ProjectRow, Selection};
use crate::style::{self, icons};

impl Board {
    pub fn detail_pane(&mut self, ui: &mut egui::Ui) {
        let mut actions = Vec::new();
        // The sessions that run there, which the files cannot tell (EP-12).
        let running = match &self.view.selected {
            Some(Selection::Project(name)) => self.session_reasons(name),
            _ => Vec::new(),
        };
        // The graphs' read of the ledger (EP-26), when due.
        if let Some(Selection::Project(_)) = &self.view.selected {
            let ledger = self.ledger().map(std::path::Path::to_path_buf);
            self.analytics
                .refresh(ledger.as_deref(), std::time::Instant::now());
        }
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| match self.view.selected.clone() {
                Some(Selection::Project(name)) => match self.portfolio.project(&name) {
                    Some(row) => project(
                        ui,
                        row,
                        &running,
                        &self.portfolio,
                        Figures {
                            week: &self.meter.week,
                            analytics: &mut self.analytics,
                        },
                        &mut self.view.folds,
                        &mut actions,
                    ),
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
                            "Adopt registers it as {}, writes {}.code-workspace beside it unless \
                             one is there, and adds it to the workspace file; Ignore leaves it \
                             out of the list (D-7).",
                            folder.first_state(),
                            folder.name
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

/// What the ledger gives the pane beyond the files (EP-18b, EP-26): the week's usage shares and
/// the graphs' latest read.
struct Figures<'a> {
    week: &'a Week,
    analytics: &'a mut Analytics,
}

fn project(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    running: &[InProgress],
    portfolio: &Portfolio,
    figures: Figures<'_>,
    folds: &mut BTreeMap<String, bool>,
    actions: &mut Vec<Action>,
) {
    let Figures { week, analytics } = figures;
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
    for proposal in proposals.iter().filter(|p| !p.dismissed) {
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
                    if ui
                        .button("Dismiss")
                        .on_hover_text(format!(
                            "Leave it out until {}; the registry keeps the fact (D-6)",
                            proposal.reason.returns_when()
                        ))
                        .clicked()
                    {
                        actions.push(Action::Dismiss(name.to_owned(), proposal.reason));
                    }
                });
            });
    }
    for proposal in proposals.iter().filter(|p| p.dismissed) {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "Dismissed: {}; it returns when {}.",
                    proposal.reason,
                    proposal.reason.returns_when()
                ))
                .weak(),
            );
            if ui
                .small_button("Restore")
                .on_hover_text("Show the proposal again")
                .clicked()
            {
                actions.push(Action::Restore(name.to_owned(), proposal.reason.kind()));
            }
        });
    }

    in_progress(ui, row, running);
    next_brief(ui, row, actions);
    progress(ui, snapshot);
    let measured = analytics::measured(analytics, portfolio);
    history(ui, row, &measured, actions);
    analytics::section(ui, row, portfolio, analytics, &measured);
    status(ui, row, portfolio, week, actions);
    // The tasks and the older forms live in the Owner tasks pane, the mail in and out in the
    // Inbox pane; a line leads to each (EP-50, EP-51).
    crate::owner_tasks::detail_line(ui, row, actions);
    crate::mailbox::detail_line(ui, row, actions);
    // Then the three headers, folded as the owner left them (EP-51).
    decisions(ui, snapshot, folds);
    house_style(ui, row, folds, actions);
    problems(ui, row, folds);
}

/// The Detail pane's folding headers after Status by the names the view keeps them under, each
/// with its default: the decision log and the house style shut, the problems open, since that
/// header shows only when the project has a problem (EP-51).
pub(crate) const FOLDS: [(&str, bool); 3] = [
    ("Decisions", false),
    ("House style", false),
    ("Problems", true),
];

/// A header after Status (EP-51): open or shut as the owner left it, which `folds` keeps by
/// `name` with the view, else as [`FOLDS`] says; `title` is what it reads.
fn fold(
    ui: &mut egui::Ui,
    folds: &mut BTreeMap<String, bool>,
    name: &str,
    title: &str,
    body: impl FnOnce(&mut egui::Ui),
) {
    let default = FOLDS
        .iter()
        .find(|(fold, _)| *fold == name)
        .is_some_and(|&(_, open)| open);
    let open = folds.get(name).copied().unwrap_or(default);
    ui.add_space(8.0);
    let shown = egui::CollapsingHeader::new(RichText::new(title).strong())
        .id_salt(("detail fold", name))
        .open(Some(open))
        .show(ui, body);
    if shown.header_response.clicked() {
        folds.insert(name.to_owned(), !open);
    }
}

/// What the house style still asks of the project (D-9, D-11; EP-21a): the files Formalise…
/// would write, what refuses it, and an older roadmap's Migrate…; only when that matters
/// (`House::matters`: the tasks file or the inbox to create, a refusal, an older form; D-11
/// addendum of 2026-11-11) or in the unformalised state. A missing template or rule block alone
/// leaves Formalise… in the row's menu and no section here. Folded shut by default (EP-51).
fn house_style(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    folds: &mut BTreeMap<String, bool>,
    actions: &mut Vec<Action>,
) {
    let house = &row.house;
    let unformalised = row.project.state == crw_core::registry::State::Unformalised;
    let formalise = house.wants() || unformalised;
    if row.folder.is_none() || !(house.matters() || unformalised) {
        return;
    }
    fold(ui, folds, "House style", "House style", |ui| {
        house_style_body(ui, row, formalise, unformalised, actions);
    });
}

/// The House style header's lines and buttons.
fn house_style_body(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    formalise: bool,
    unformalised: bool,
    actions: &mut Vec<Action>,
) {
    let house = &row.house;
    if !house.writes.is_empty() {
        ui.add(
            egui::Label::new(format!("Formalise… would write {}.", house.writes_words())).wrap(),
        );
    } else if unformalised && house.refused.is_empty() {
        ui.label(
            RichText::new(
                "Every house-style file is there; the state stays unformalised until you set it \
                 (D-6).",
            )
            .weak(),
        );
    }
    for why in &house.refused {
        ui.add(
            egui::Label::new(RichText::new(format!("Refused: {why}")).color(style::amber(ui)))
                .wrap(),
        );
    }
    if let Some(profile) = house.migrates {
        ui.add(
            egui::Label::new(
                RichText::new(format!(
                    "Its roadmap's tables are {}, read-only (D-9): Migrate… opens the launch \
                     dialog with the prompt of a headless session that converts them to the house \
                     tables.",
                    crw_core::formalise::form_words(profile)
                ))
                .weak(),
            )
            .wrap(),
        );
    }
    ui.horizontal(|ui| {
        if formalise
            && ui
                .button("Formalise…")
                .on_hover_text(
                    "Preview the house-style files: each file and its change, the block and the \
                     new files' text, then Write or Cancel (D-11)",
                )
                .clicked()
        {
            actions.push(Action::OpenFormalise(row.name().to_owned()));
        }
        if house.migrates.is_some()
            && ui
                .button("Migrate…")
                .on_hover_text(crate::formalise::MIGRATE_HINT)
                .clicked()
        {
            actions.push(Action::OpenMigration(row.name().to_owned()));
        }
    });
}

/// Why a session is taken to be in progress: the sessions that run there and what the files say
/// (D-9), for every project, whether or not it has a next brief (EP-13).
fn in_progress(ui: &mut egui::Ui, row: &ProjectRow, running: &[InProgress]) {
    let mut reasons = running.to_vec();
    reasons.extend(row.in_progress());
    if !reasons.is_empty() {
        let texts: Vec<String> = reasons.iter().map(ToString::to_string).collect();
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!("Session in progress: {}.", texts.join("; ")))
                .color(style::amber(ui)),
        );
    } else if row.awaits_git() {
        ui.add_space(6.0);
        ui.label(RichText::new("Git has not been asked yet.").weak());
    }
}

fn next_brief(ui: &mut egui::Ui, row: &ProjectRow, actions: &mut Vec<Action>) {
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
        if ui
            .button(style::icon(icons::ICON_PLAY_ARROW, None))
            .on_hover_text(format!("Start {}: the launch dialog (S)", next.id))
            .clicked()
        {
            actions.push(Action::OpenLaunch(row.name().to_owned()));
        }
        ui.label(RichText::new(next.id.to_string()).monospace().strong());
        if let Some(size) = next.size {
            ui.label(RichText::new(size.as_str()).strong());
        }
        ui.label(markup::coloured(
            ui,
            &next.brief.title,
            Look::default(),
            false,
        ));
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
            // The text as wide as the reading measure, or the pane when that is narrower (EP-15a).
            let measure = markup::measure(ui);
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width().min(measure));
                    ui.label(
                        RichText::new(format!("Brief notes (README line {})", note.line))
                            .small()
                            .weak(),
                    );
                    markup::note(ui, text, Look::default());
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
    // Three columns laid out by hand rather than a `Grid`, whose cells do not wrap: the phase's
    // id, its counts, and its name wrapping under itself in a narrow pane (EP-26).
    let id_width = roadmap
        .phases
        .iter()
        .map(|phase| {
            style::width_of(ui, |ui| {
                ui.label(RichText::new(phase.heading.id.as_deref().unwrap_or("-")).monospace());
            })
        })
        .fold(0.0_f32, f32::max)
        + NAME_GAP;
    let counts_width = roadmap
        .phases
        .iter()
        .map(|phase| {
            style::width_of(ui, |ui| {
                ui.label(phase_counts(phase));
            })
        })
        .fold(0.0_f32, f32::max)
        + NAME_GAP;
    for phase in &roadmap.phases {
        let heading = &phase.heading;
        ui.horizontal_top(|ui| {
            style::padded(ui, id_width, |ui| {
                ui.label(RichText::new(heading.id.as_deref().unwrap_or("-")).monospace());
            });
            style::padded(ui, counts_width, |ui| {
                ui.label(phase_counts(phase));
            });
            let status = heading
                .status
                .as_deref()
                .map(|s| format!(" ({s})"))
                .unwrap_or_default();
            ui.add(
                egui::Label::new(RichText::new(format!("{}{status}", heading.name)).weak()).wrap(),
            );
        });
    }
}

/// A phase's counts: `3 of 8`, or `no rows`.
fn phase_counts(phase: &crw_core::roadmap::table::Phase) -> String {
    let counts = phase.counts();
    match counts.total {
        0 => "no rows".to_owned(),
        total => format!("{} of {total}", counts.done),
    }
}

/// The History section (D-26; EP-25): what the import of the project's log and briefs settled,
/// the EPs whose two dates disagree, when it ran and at which HEAD, and Re-import, which runs
/// it again whatever HEAD says. Shown for a repository, or for a folder the import has read.
/// From EP-26 a folded list of the dated EPs, each with its completion note's model, effort and
/// duration where the new form gives them, and the ledger's measured time where a row gives it.
fn history(ui: &mut egui::Ui, row: &ProjectRow, measured: &[Measured], actions: &mut Vec<Action>) {
    if !row.is_repository() && row.history.is_none() {
        return;
    }
    section(ui, "History");
    match &row.history {
        Some(summary) => {
            let import = &summary.import;
            let dates = |ui: &mut egui::Ui| {
                let counted = match import.completions.len() {
                    1 => "1 EP dated".to_owned(),
                    n => format!("{n} EPs dated"),
                };
                let from = if import.commits == 0 {
                    "from the briefs' completion notes alone (no git log read)".to_owned()
                } else if import.single {
                    "from the briefs' completion notes alone (a single-commit history)".to_owned()
                } else {
                    format!(
                        "from {} commits and the briefs' completion notes",
                        import.commits
                    )
                };
                let span = import
                    .span()
                    .map(|(first, last)| {
                        if first == last {
                            format!("; on {first}")
                        } else {
                            format!("; the first on {first}, the latest on {last}")
                        }
                    })
                    .unwrap_or_default();
                ui.add(egui::Label::new(format!("{counted} {from}{span}.")).wrap());
            };
            dates(ui);
            let flagged: Vec<_> = import.flagged().collect();
            if !flagged.is_empty() {
                let lines: Vec<String> = flagged
                    .iter()
                    .map(|c| {
                        format!(
                            "{}: git says {}, the note {} (the note wins)",
                            c.ep,
                            c.git_date.map_or("nothing".to_owned(), |d| d.to_string()),
                            c.note_date.map_or("nothing".to_owned(), |d| d.to_string())
                        )
                    })
                    .collect();
                ui.add(
                    egui::Label::new(
                        RichText::new(format!(
                            "{} to look at: {}.",
                            match flagged.len() {
                                1 => "1 EP".to_owned(),
                                n => format!("{n} EPs"),
                            },
                            lines.join("; ")
                        ))
                        .color(style::amber(ui)),
                    )
                    .wrap(),
                );
            }
            let head = summary
                .head
                .as_deref()
                .map(|head| format!(" at HEAD {}", crw_core::history::short(head)))
                .unwrap_or_default();
            let age = short_age(Utc::now() - summary.imported_at);
            let when = if age == "now" {
                "just now".to_owned()
            } else {
                format!("{age} ago")
            };
            ui.label(
                RichText::new(format!(
                    "Imported {when}{head}; the git pass imports again when HEAD moves."
                ))
                .weak(),
            );
            eps_list(ui, row, import, measured);
        }
        None => {
            ui.add(
                egui::Label::new(
                    RichText::new(
                        "Not imported yet: the git pass imports a repository's log at its first \
                         sight and when HEAD moves.",
                    )
                    .weak(),
                )
                .wrap(),
            );
        }
    }
    let button = ui
        .add_enabled(row.is_repository(), egui::Button::new("Re-import"))
        .on_hover_text(
            "Read the log and the briefs again and replace the ledger's rows for this project \
             (D-26)",
        )
        .on_disabled_hover_text("Not a repository: nothing to import");
    if button.clicked() {
        actions.push(Action::Reimport(row.name().to_owned()));
    }
}

/// The dated EPs, the newest first, folded under the History section's lines (EP-26): each with
/// its date, the completion note's model, effort and duration in the new form, and the ledger's
/// measured time (`19 min (the ledger)`, or `wall time` for a visual or manual session).
fn eps_list(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    import: &crw_core::history::Import,
    measured: &[Measured],
) {
    if import.completions.is_empty() {
        return;
    }
    let mut completions: Vec<_> = import.completions.iter().collect();
    completions.sort_by(|a, b| {
        b.completed_on
            .cmp(&a.completed_on)
            .then_with(|| b.ep.cmp(&a.ep))
    });
    egui::CollapsingHeader::new(RichText::new("The EPs, the newest first").weak())
        .id_salt(("eps", row.name()))
        .show(ui, |ui| {
            for completion in completions {
                // The note with the figures: a later note appended by another brief carries
                // none, so the last note that gives a model or a duration wins, else the last.
                let note = row
                    .snapshot
                    .briefs
                    .iter()
                    .find(|entry| entry.file.id == completion.ep)
                    .and_then(|entry| {
                        let notes: Vec<_> = entry
                            .brief
                            .completion_notes()
                            .filter_map(|quote| match &quote.kind {
                                Kind::Completion(note) => Some(note.clone()),
                                _ => None,
                            })
                            .collect();
                        notes
                            .iter()
                            .rev()
                            .find(|note| note.model.is_some() || note.duration.is_some())
                            .or(notes.last())
                            .cloned()
                    });
                let mut parts = vec![format!("{} · {}", completion.ep, completion.completed_on)];
                if let Some(note) = &note {
                    if let (Some(model), Some(effort)) = (&note.model, &note.effort) {
                        parts.push(format!("{model}/{effort}"));
                    }
                    if let Some(duration) = &note.duration {
                        parts.push(format!("{duration} (the note)"));
                    }
                }
                if let Some(measure) = measured
                    .iter()
                    .find(|m| m.project == row.name() && m.ep == completion.ep)
                {
                    parts.push(format!(
                        "{} ({})",
                        analytics::minutes(measure.minutes),
                        if measure.wall {
                            "the ledger, wall time"
                        } else {
                            "the ledger"
                        }
                    ));
                }
                ui.add(egui::Label::new(RichText::new(parts.join(" · ")).weak()).wrap());
            }
        });
}

/// The Status section's subsections, in order.
const SUBSECTIONS: [&str; 6] = ["Stack", "Activity", "Hosting", "Health", "Backup", "Usage"];
/// The mark column's least width, so the sentences start at the same place for most projects.
const MARK_AT_LEAST: f32 = 40.0;
/// Between a subsection's name and its lines.
const NAME_GAP: f32 = 10.0;

/// The Status section's three columns (EP-7a): each subsection's name, level with its first
/// line; a mark; and the mark's sentence, which wraps under itself. The widths are measured in
/// points from what the section holds, so it reads the same at every window size.
struct Columns {
    name: f32,
    mark: f32,
}

impl Columns {
    /// A subsection: its name beside its first line, its lines below one another.
    fn subsection(&self, ui: &mut egui::Ui, name: &str, lines: impl FnOnce(&mut egui::Ui)) {
        ui.horizontal_top(|ui| {
            style::padded(ui, self.name, |ui| {
                ui.label(RichText::new(name).weak());
            });
            ui.vertical(lines);
        });
    }

    /// One badge spelt out: its mark in the mark column, then its sentence.
    fn mark_line(&self, ui: &mut egui::Ui, mark: &Mark) {
        ui.horizontal_top(|ui| {
            style::padded(ui, self.mark, |ui| {
                mark.show(ui);
            });
            ui.add(egui::Label::new(&mark.hover).wrap());
        });
    }

    /// A line of small buttons where the sentences start, wrapping in a narrow pane (EP-34).
    fn button_line(&self, ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
        ui.horizontal_top(|ui| {
            style::padded(ui, self.mark, |_| {});
            ui.horizontal_wrapped(add);
        });
    }

    /// A line without a mark, its text where the sentences start.
    fn weak_line(&self, ui: &mut egui::Ui, text: impl Into<String>) {
        ui.horizontal_top(|ui| {
            style::padded(ui, self.mark, |_| {});
            ui.add(egui::Label::new(RichText::new(text.into()).weak()).wrap());
        });
    }
}

/// A line between two subsections, in the theme's separator colour.
fn divider(ui: &mut egui::Ui) {
    ui.add(egui::Separator::default().horizontal().spacing(9.0));
}

/// The badges spelt out (D-24, D-25): stack, activity, hosting with the head check, health,
/// backup and this week's usage share (EP-18b), each with where it comes from and how old it is;
/// the hosting's actions and Back up now under their subsections (EP-34).
fn status(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    portfolio: &Portfolio,
    week: &Week,
    actions: &mut Vec<Action>,
) {
    let now = Utc::now();
    let badges = &row.badges;
    let activity = badges::activity(row, now);
    let hosting = badges::hosting(row, now);
    let sync = badges::sync(row, now);
    let ci = badges::ci(row, now);
    let warning = badges::license_warning(row);
    let backup = badges::backup(row, now);
    let usage = badges::usage(row);
    section(ui, "Status");
    let name = SUBSECTIONS
        .iter()
        .map(|name| {
            style::width_of(ui, |ui| {
                ui.label(RichText::new(*name).weak());
            })
        })
        .fold(0.0, f32::max)
        + NAME_GAP;
    let marks = [
        activity.as_ref().map(|(age, _)| age),
        activity.as_ref().and_then(|(_, dirty)| dirty.as_ref()),
        Some(&hosting),
        sync.as_ref(),
        ci.as_ref(),
        warning.as_ref(),
        backup.as_ref(),
        usage.as_ref(),
    ];
    let mark = marks
        .into_iter()
        .flatten()
        .map(|mark| {
            style::width_of(ui, |ui| {
                mark.show(ui);
            })
        })
        .fold(MARK_AT_LEAST, f32::max);
    let columns = Columns { name, mark };

    columns.subsection(ui, SUBSECTIONS[0], |ui| {
        ui.horizontal_wrapped(|ui| {
            if badges.stack.is_empty() {
                ui.label(RichText::new("No manifest at the root or one folder down").weak());
            }
            for chip in &badges.stack {
                style::chip(ui, &chip.text(), style::stack_colour(chip.kind))
                    .on_hover_text(&chip.source);
            }
        });
    });
    divider(ui);

    columns.subsection(ui, SUBSECTIONS[1], |ui| {
        match &activity {
            Some((age, dirty)) => {
                columns.mark_line(ui, age);
                match dirty {
                    Some(dirty) => columns.mark_line(ui, dirty),
                    None => columns.weak_line(ui, "A clean working tree"),
                }
            }
            None if row.awaits_git() => columns.weak_line(ui, "Git has not been asked yet"),
            None => columns.weak_line(ui, "Not a git repository"),
        }
        // The row's latest read on its own (EP-25a), until every project reads again.
        if let Some(reread) = &row.reread {
            columns.weak_line(ui, reread.line(now));
        }
    });
    divider(ui);

    columns.subsection(ui, SUBSECTIONS[2], |ui| {
        columns.mark_line(ui, &hosting);
        if let Some(remote) = &badges.remote {
            columns.weak_line(ui, format!("origin {remote}"));
        }
        if let Some(mark) = &sync {
            columns.mark_line(ui, mark);
        }
        match &portfolio.account {
            Some(fetched) => {
                let age = short_age(now - fetched.at);
                match &fetched.value {
                    Ok(_) => columns.weak_line(
                        ui,
                        format!(
                            "GitHub read {age} ago; it is asked daily and on F5, never fetched \
                             from"
                        ),
                    ),
                    Err(e) => columns.weak_line(ui, format!("GitHub could not be asked: {e}")),
                }
            }
            None => columns.weak_line(ui, "GitHub not read yet: it is asked daily and on F5"),
        }
        // What the hosting allows, each behind its confirmation (D-24; EP-34).
        let offered = crate::act::offered(row);
        if !offered.is_empty() {
            columns.button_line(ui, |ui| {
                for verb in offered {
                    if ui
                        .small_button(verb.label())
                        .on_hover_text(crate::act::hint(verb))
                        .clicked()
                    {
                        actions.push(Action::OpenAct {
                            project: Some(row.name().to_owned()),
                            verb,
                        });
                    }
                }
            });
        }
    });
    divider(ui);

    columns.subsection(ui, SUBSECTIONS[3], |ui| {
        match &ci {
            Some(mark) => columns.mark_line(ui, mark),
            None => columns.weak_line(ui, "No GitHub Actions workflows"),
        }
        match (&badges.license, &warning) {
            (_, Some(warning)) => columns.mark_line(ui, warning),
            (Some(file), None) => columns.weak_line(ui, format!("License: {file}")),
            (None, None) => columns.weak_line(ui, "No license file at the root"),
        }
    });
    divider(ui);

    columns.subsection(ui, SUBSECTIONS[4], |ui| {
        match &backup {
            Some(mark) => columns.mark_line(ui, mark),
            None if portfolio.backup.is_none() => columns.weak_line(
                ui,
                "No backup job is set: crw setting backup_settings <its settings file>",
            ),
            None => columns.weak_line(ui, "Not read yet"),
        }
        if let Some(backup) = &badges.backup
            && !backup.excluded.is_empty()
        {
            columns.weak_line(
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
            columns.weak_line(ui, format!("The job's {target}{run}"));
        }
        // The whole job run now, behind its confirmation (D-24; EP-34).
        if portfolio.backup.is_some() || badges.backup.is_some() {
            let verb = crw_core::actions::Verb::BackUpNow;
            columns.button_line(ui, |ui| {
                if ui
                    .small_button(verb.label())
                    .on_hover_text(crate::act::hint(verb))
                    .clicked()
                {
                    actions.push(Action::OpenAct {
                        project: None,
                        verb,
                    });
                }
            });
        }
    });
    divider(ui);

    columns.subsection(ui, SUBSECTIONS[5], |ui| {
        match &usage {
            Some(mark) => columns.mark_line(ui, mark),
            None => columns.weak_line(
                ui,
                match (&week.problem, week.since) {
                    (Some(problem), _) => problem.clone(),
                    (None, Some(since)) => format!(
                        "No session of this project has taken a share of the weekly limit since \
                         the reset on {}",
                        since.with_timezone(&Local).format("%a %d %b %H:%M")
                    ),
                    (None, None) => {
                        "No reading of the weekly limit yet, so no week to sum the shares over"
                            .to_owned()
                    }
                },
            ),
        }
        columns.weak_line(
            ui,
            "From the ledger: each poll's rise split among the sessions that worked since the \
             poll before, manual ones too (D-23)",
        );
    });
}

/// The item's meta line: its form, file and line, state and origin; its gate is the chip beside
/// its id (EP-15a).
pub(crate) fn meta(item: &OwnerItem, folder: &std::path::Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    if item.form != Form::Task {
        parts.push(item.form.to_string());
    }
    let file = item.path.strip_prefix(folder).unwrap_or(&item.path);
    parts.push(format!("{}:{}", file.display(), item.line));
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

/// `text` without the `[before EP-m]` it ends with when the chip beside the item's id says it
/// (EP-15a, the owner's choice); any other text as written.
pub(crate) fn without_gate<'a>(text: &'a str, gate: Option<&Gate>) -> &'a str {
    let Some(Gate::Before(id)) = gate else {
        return text;
    };
    let trimmed = text.trim_end();
    let tail = format!("[before {id}]");
    trimmed
        .len()
        .checked_sub(tail.len())
        .filter(|&at| trimmed.is_char_boundary(at) && trimmed[at..].eq_ignore_ascii_case(&tail))
        .map_or(text, |at| trimmed[..at].trim_end())
}

/// The decision log's counts, under a header folded shut by default (EP-51).
fn decisions(ui: &mut egui::Ui, snapshot: &Snapshot, folds: &mut BTreeMap<String, bool>) {
    let decisions = &snapshot.decisions;
    let Some(path) = &decisions.path else {
        return;
    };
    let name = path.strip_prefix(&snapshot.folder).unwrap_or(path);
    let last = decisions
        .log
        .last_addendum()
        .map(|date| format!(", the last {date}"))
        .unwrap_or_default();
    fold(ui, folds, "Decisions", "Decision log", |ui| {
        ui.label(format!(
            "{}: {} decisions, {} addenda{last}",
            name.display(),
            decisions.log.entries.len(),
            decisions.log.addenda.len()
        ));
    });
}

/// The problems the readers met, under a header open by default (EP-51), shown only when there
/// is one.
fn problems(ui: &mut egui::Ui, row: &ProjectRow, folds: &mut BTreeMap<String, bool>) {
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
    let title = format!("Problems the readers met: {}", lines.len());
    fold(ui, folds, "Problems", &title, |ui| {
        for line in lines {
            ui.add(egui::Label::new(RichText::new(line).color(style::amber(ui))).wrap());
        }
    });
}

#[cfg(test)]
mod tests {
    use crw_core::roadmap::EpId;

    use super::*;

    #[test]
    fn the_chip_says_the_gate_so_the_text_leaves_it_out() {
        let due = Gate::Before(EpId::new(16));
        assert_eq!(
            without_gate(
                "Note what it gets wrong. Notes go to `roadmap/inbox.md` [before EP-16]",
                Some(&due)
            ),
            "Note what it gets wrong. Notes go to `roadmap/inbox.md`"
        );
        assert_eq!(
            without_gate("Ask first [Before EP-16]  ", Some(&due)),
            "Ask first"
        );
        // Another gate, a gate inside the text, a gate in words, or none: as written.
        let words = Gate::When("v2 gate".to_owned());
        for (text, gate) in [
            ("Ask first [before EP-15]", Some(&due)),
            ("Due (D-25, before EP-16) soon", Some(&due)),
            ("At the v2 gate", Some(&words)),
            ("Ask first [before EP-16]", None),
            ("ünïcode [before EP-16]", Some(&Gate::Before(EpId::new(6)))),
        ] {
            assert_eq!(without_gate(text, gate), text, "{text}");
        }
    }

    /// Whether `output` paints a text that contains `part`.
    fn shows(output: &egui::FullOutput, part: &str) -> bool {
        output.shapes.iter().any(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) => shape.galley.text().contains(part),
            _ => false,
        })
    }

    /// The House style section (EP-21a): what Formalise… would write and its button, Migrate…
    /// for an older roadmap, and nothing for a project with every file and house tables. Since
    /// EP-38 it shows only when it matters (D-11 addendum of 2026-11-11): a project lacking only
    /// the rule block and the template keeps Formalise… in its row's menu and gets no section.
    #[test]
    fn the_house_style_section_says_what_formalise_would_write() {
        use crate::portfolio::House;
        use crate::portfolio::tests::{Scratch, fake_store};
        use crate::testing::{self, Screen};
        use crw_core::roadmap::table::Profile;

        let scratch = Scratch::new("detail-house");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        // lantern keeps an inbox and no owner-tasks.md: the section matters.
        let lantern = portfolio.project("lantern").unwrap();
        assert!(lantern.house.matters());
        let mut screen = Screen::new();
        let mut actions = Vec::new();
        // Folded shut by default (EP-51): the header alone.
        let output = screen.pass(Vec::new(), |ui| {
            house_style(ui, lantern, &mut BTreeMap::new(), &mut actions);
        });
        assert!(testing::text_at(&output, "House style").is_some());
        assert!(!shows(&output, "Formalise… would write"));
        // The owner opened it: the view keeps it so.
        let open = || BTreeMap::from([("House style".to_owned(), true)]);
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            house_style(ui, lantern, &mut open(), &mut actions);
        });
        assert!(shows(
            &output,
            "Formalise… would write CLAUDE.md (created), roadmap/owner-tasks.md (created) and \
             roadmap/_TEMPLATE.md (created)."
        ));
        assert!(testing::text_at(&output, "Migrate…").is_none());
        let at = testing::text_at(&output, "Formalise…").expect("its button");
        for events in [
            testing::moved(at),
            testing::button(at, egui::PointerButton::Primary, true),
            testing::button(at, egui::PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| {
                house_style(ui, lantern, &mut open(), &mut actions);
            });
        }
        assert_eq!(actions, [Action::OpenFormalise("lantern".to_owned())]);
        // harbor lacks only CLAUDE.md and the template: Formalise… stays in the row's menu
        // (`wants`), and the pane shows no section (EP-38).
        let harbor = portfolio.project("harbor").unwrap();
        assert!(harbor.house.wants() && !harbor.house.matters());
        let output = screen.pass(Vec::new(), |ui| {
            house_style(ui, harbor, &mut open(), &mut Vec::new());
        });
        assert!(testing::text_at(&output, "House style").is_none());
        // Unformalised, the section stays, saying what Formalise… would write.
        let mut unformalised = harbor.clone();
        unformalised.project.state = crw_core::registry::State::Unformalised;
        let output = screen.pass(Vec::new(), |ui| {
            house_style(ui, &unformalised, &mut open(), &mut Vec::new());
        });
        assert!(shows(
            &output,
            "Formalise… would write CLAUDE.md (created) and roadmap/_TEMPLATE.md (created)."
        ));
        // Every file there and house tables: no section.
        let mut whole = harbor.clone();
        whole.house = House::default();
        let output = screen.pass(Vec::new(), |ui| {
            house_style(ui, &whole, &mut open(), &mut Vec::new());
        });
        assert!(testing::text_at(&output, "House style").is_none());
        // A refusal matters.
        whole.house.refused = vec!["CLAUDE.md: an end marker without a begin".to_owned()];
        let output = screen.pass(Vec::new(), |ui| {
            house_style(ui, &whole, &mut open(), &mut Vec::new());
        });
        assert!(shows(
            &output,
            "Refused: CLAUDE.md: an end marker without a begin"
        ));
        whole.house.refused.clear();
        // An older roadmap: Migrate…, and no Formalise… while every file is there.
        whole.house.migrates = Some(Profile::House5);
        let output = screen.pass(Vec::new(), |ui| {
            house_style(ui, &whole, &mut open(), &mut Vec::new());
        });
        assert!(testing::text_at(&output, "Migrate…").is_some());
        assert!(testing::text_at(&output, "Formalise…").is_none());
        assert!(shows(&output, "the five-column tables, read-only (D-9)"));
    }

    /// The Status section's Activity lines say a row's latest read on its own (EP-25a), and
    /// nothing of it for a row never read alone.
    #[test]
    fn the_status_section_says_the_rows_read_on_its_own() {
        use crate::meter::Week;
        use crate::portfolio::tests::{Scratch, fake_store};
        use crate::portfolio::{Moment, Reread};
        use crate::testing::{self, Screen};

        let scratch = Scratch::new("detail-reread");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let harbor = portfolio.project("harbor").unwrap();
        let week = Week::default();
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            status(ui, harbor, &portfolio, &week, &mut Vec::new())
        });
        assert!(testing::text_containing(&output, "Read again").is_none());
        let mut reread = harbor.clone();
        reread.reread = Some(Reread {
            at: Utc::now() - chrono::TimeDelta::minutes(5),
            moment: Moment::SessionEnded,
            what: "the files, git and the history".to_owned(),
        });
        let output = screen.pass(Vec::new(), |ui| {
            status(ui, &reread, &portfolio, &week, &mut Vec::new())
        });
        assert!(
            testing::text_at(
                &output,
                "Read again 5 min ago after a session ended there: the files, git and the history"
            )
            .is_some()
        );
    }

    /// The Status section offers what the hosting allows under Hosting and Back up now under
    /// Backup (EP-34): a local repository's Create a private remote…, each button pushing its
    /// action; neither for a folder that is not a repository and has no backup.
    #[test]
    fn the_status_section_offers_the_hosting_actions_and_back_up_now() {
        use crate::meter::Week;
        use crate::portfolio::tests::{Scratch, fake_store};
        use crate::testing::{self, Screen};
        use crw_core::actions::Verb;
        use eframe::egui::PointerButton;

        let scratch = Scratch::new("detail-actions");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let week = Week::default();
        let mut quarry = portfolio.project("quarry").unwrap().clone();
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            status(ui, &quarry, &portfolio, &week, &mut Vec::new())
        });
        assert!(testing::text_at(&output, "Create a private remote…").is_none());
        assert!(testing::text_at(&output, "Back up now…").is_none());
        // A repository with no remote, and a unit in the backup job.
        quarry.badges.activity = Some(crw_core::badges::activity::Activity {
            oid: Some("f".repeat(40)),
            last_commit: None,
            changed: 0,
            branch: Some("main".to_owned()),
            upstream: None,
            ahead_behind: None,
        });
        quarry.badges.backup = Some(crw_core::badges::backup::Backup::default());
        let press = |screen: &mut Screen, label: &str| {
            let mut actions = Vec::new();
            let output = screen.pass(Vec::new(), |ui| {
                status(ui, &quarry, &portfolio, &week, &mut actions)
            });
            let at = testing::text_at(&output, label).expect(label);
            for events in [
                testing::moved(at),
                testing::button(at, PointerButton::Primary, true),
                testing::button(at, PointerButton::Primary, false),
            ] {
                screen.pass(events, |ui| {
                    status(ui, &quarry, &portfolio, &week, &mut actions)
                });
            }
            actions
        };
        assert_eq!(
            press(&mut screen, "Create a private remote…"),
            [Action::OpenAct {
                project: Some("quarry".to_owned()),
                verb: Verb::CreatePrivate,
            }]
        );
        assert_eq!(
            press(&mut screen, "Back up now…"),
            [Action::OpenAct {
                project: None,
                verb: Verb::BackUpNow,
            }]
        );
    }

    /// The History section (EP-25): a canned import's dates and flagged EPs, Re-import pushing
    /// its action for a repository, and the section absent for a folder with neither.
    #[test]
    fn the_history_section_says_the_import_and_offers_reimport() {
        use crate::portfolio::tests::{Scratch, fake_store};
        use crate::testing::{self, Screen};
        use crw_core::history::{Completion, Flag, Import, Summary};

        let scratch = Scratch::new("detail-history");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let harbor = portfolio.project("harbor").unwrap();
        assert!(!harbor.is_repository());
        let mut screen = Screen::new();
        // Neither a repository nor an import: no section.
        let output = screen.pass(Vec::new(), |ui| history(ui, harbor, &[], &mut Vec::new()));
        assert!(testing::text_at(&output, "History").is_none());
        // An import kept for it: the section, Re-import disabled (no repository).
        let completion = |ep: &str, git: &str, note: Option<&str>, flag: Flag| Completion {
            ep: ep.parse().unwrap(),
            completed_on: note.unwrap_or(git).parse().unwrap(),
            git_date: Some(git.parse().unwrap()),
            note_date: note.map(|d| d.parse().unwrap()),
            record: vec!["1111111".to_owned()],
            work: vec!["2222222".to_owned()],
            flag,
        };
        let mut imported = harbor.clone();
        imported.history = Some(Summary {
            imported_at: Utc::now() - chrono::TimeDelta::minutes(3),
            head: Some("abcdef0123456789".to_owned()),
            import: Import {
                completions: vec![
                    completion("EP-1", "2026-10-31", Some("2026-10-31"), Flag::Agreed),
                    completion("EP-2", "2026-11-01", Some("2026-11-04"), Flag::NoteWins),
                    completion("EP-3", "2026-11-05", None, Flag::GitOnly),
                ],
                commits: 12,
                single: false,
            },
        });
        let mut actions = Vec::new();
        let output = screen.pass(Vec::new(), |ui| history(ui, &imported, &[], &mut actions));
        assert!(shows(
            &output,
            "3 EPs dated from 12 commits and the briefs' completion notes; the first on \
             2026-10-31, the latest on 2026-11-05."
        ));
        assert!(shows(
            &output,
            "1 EP to look at: EP-2: git says 2026-11-01, the note 2026-11-04 (the note wins)."
        ));
        assert!(shows(&output, "Imported 3 min ago at HEAD abcdef0"));
        let at = testing::text_at(&output, "Re-import").expect("its button");
        for events in [
            testing::moved(at),
            testing::button(at, egui::PointerButton::Primary, true),
            testing::button(at, egui::PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| history(ui, &imported, &[], &mut actions));
        }
        assert!(
            actions.is_empty(),
            "disabled for a folder that is no repository"
        );
        // A repository without an import yet: the section says so, and Re-import acts.
        let mut repository = harbor.clone();
        repository.folder.as_mut().unwrap().git = true;
        let output = screen.pass(Vec::new(), |ui| history(ui, &repository, &[], &mut actions));
        assert!(shows(&output, "Not imported yet"));
        let at = testing::text_at(&output, "Re-import").expect("its button");
        for events in [
            testing::moved(at),
            testing::button(at, egui::PointerButton::Primary, true),
            testing::button(at, egui::PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| history(ui, &repository, &[], &mut actions));
        }
        assert_eq!(actions, [Action::Reimport("harbor".to_owned())]);
    }

    /// The Analytics section (EP-26) draws from canned rows: the projection's words, the
    /// medians with their source, and the History section's EPs list with a note's model and
    /// effort and the ledger's measured time.
    #[test]
    fn the_analytics_section_projects_and_the_eps_list_says_model_effort_and_time() {
        use crate::analytics::tests::{completion, session};
        use crate::analytics::{Analytics, Loaded};
        use crate::portfolio::tests::{Scratch, fake_store};
        use crate::testing::{self, Screen};
        use crw_core::history::{Import, Summary};

        let scratch = Scratch::new("detail-analytics");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let mut harbor = portfolio.project("harbor").unwrap().clone();
        let today = chrono::Local::now().date_naive();
        let days_ago = |n: u64| today - chrono::Days::new(n);
        // EP-1 is a brief of the fixture with a completion note in the new form.
        harbor.history = Some(Summary {
            imported_at: Utc::now(),
            head: None,
            import: Import {
                completions: vec![
                    completion("EP-1", &days_ago(9).to_string()),
                    completion("EP-2", &days_ago(2).to_string()),
                ],
                commits: 5,
                single: false,
            },
        });
        let start = Utc::now() - chrono::TimeDelta::days(9);
        let mut analytics = Analytics::default();
        analytics.put(Loaded {
            sessions: vec![session(
                1,
                Some("aaaa"),
                "harbor",
                Some("EP-1"),
                "headless",
                start,
                Some((19, "completed")),
            )],
            snapshots: Default::default(),
            reverted: Default::default(),
            problem: None,
        });
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            let measured = analytics::measured(&mut analytics, &portfolio);
            history(ui, &harbor, &measured, &mut Vec::new());
            analytics::section(ui, &harbor, &portfolio, &mut analytics, &measured);
        });
        assert!(shows(&output, "Analytics"));
        assert!(shows(&output, "Burn-up: done and total by day"));
        assert!(
            testing::text_containing(&output, "open rows (").is_some(),
            "the remaining work from the mix"
        );
        assert!(
            testing::text_containing(&output, "the finish is").is_some(),
            "a finish range from two completions in the window"
        );
        assert!(
            testing::text_containing(&output, "(the portfolio's 1)").is_some(),
            "one measured row of harbor's: the portfolio's median"
        );
        // The EPs list opens on a click and says the note's model and effort and the time.
        let at = testing::text_at(&output, "The EPs, the newest first").expect("its header");
        for events in [
            testing::moved(at),
            testing::button(at, egui::PointerButton::Primary, true),
            testing::button(at, egui::PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| history(ui, &harbor, &[], &mut Vec::new()));
        }
        let measured = analytics::measured(&mut analytics, &portfolio);
        let output = screen.pass(Vec::new(), |ui| {
            history(ui, &harbor, &measured, &mut Vec::new());
        });
        let line = testing::text_containing(&output, "19 min (the ledger)");
        assert!(line.is_some(), "the measured time of EP-1's headless row");
    }

    /// The headers after Status fold as the owner left them, saved with the view (EP-51): the
    /// decision log shut by default, a click on its header opening it and the view keeping that.
    #[test]
    fn the_headers_after_status_fold_as_the_view_keeps_them() {
        use crate::portfolio::tests::{Scratch, fake_store};
        use crate::testing::{self, Screen};

        let scratch = Scratch::new("detail-folds");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let harbor = portfolio.project("harbor").unwrap();
        let snapshot = &harbor.snapshot;
        assert!(snapshot.decisions.path.is_some(), "the fixture keeps a log");
        let mut folds = BTreeMap::new();
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| decisions(ui, snapshot, &mut folds));
        assert!(shows(&output, "Decision log"));
        assert!(!shows(&output, " decisions, "), "shut by default");
        let at = testing::text_at(&output, "Decision log").expect("its header");
        for events in [
            testing::moved(at),
            testing::button(at, egui::PointerButton::Primary, true),
            testing::button(at, egui::PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| decisions(ui, snapshot, &mut folds));
        }
        assert_eq!(folds.get("Decisions"), Some(&true));
        // Drawn open from then on, the next frames' animation done.
        for _ in 0..30 {
            screen.pass(Vec::new(), |ui| decisions(ui, snapshot, &mut folds));
        }
        let output = screen.pass(Vec::new(), |ui| decisions(ui, snapshot, &mut folds));
        assert!(shows(&output, " decisions, "));
        // The problems open by default, shut once the owner shuts them.
        let mut broken = harbor.clone();
        broken
            .snapshot
            .problems
            .push("roadmap/EP-9.md: no front matter".to_owned());
        let mut folds = BTreeMap::new();
        let output = screen.pass(Vec::new(), |ui| problems(ui, &broken, &mut folds));
        assert!(shows(&output, "Problems the readers met: 1"));
        assert!(shows(&output, "no front matter"));
        folds.insert("Problems".to_owned(), false);
        for _ in 0..30 {
            screen.pass(Vec::new(), |ui| problems(ui, &broken, &mut folds));
        }
        let output = screen.pass(Vec::new(), |ui| problems(ui, &broken, &mut folds));
        assert!(!shows(&output, "no front matter"));
        // No problem, no header.
        let output = screen.pass(Vec::new(), |ui| problems(ui, harbor, &mut BTreeMap::new()));
        assert!(!shows(&output, "Problems the readers met"));
        assert_eq!(
            FOLDS.map(|(name, _)| name),
            ["Decisions", "House style", "Problems"]
        );
    }
}
