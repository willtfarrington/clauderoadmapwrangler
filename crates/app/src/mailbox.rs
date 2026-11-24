//! The Inbox pane (D-13, D-25; EP-51): every project's open messages in one place, grouped by
//! receiver in the list's order (its groups and sort key; its filters left aside, as the Owner
//! tasks pane does, so a filter never hides a message), each ticked there with a disposition as
//! the Detail pane ticked it, and Compose… on each receiver. Under them the mail the projects'
//! briefs hand on, with what the mailman made of each block (those not delivered shown, the
//! delivered folded), and the mailman's last lines from the ledger. Two toggles, saved with the
//! view: the selected project alone, and the messages ticked in the last week, dated by their
//! dispositions, which date a session's triage as well as the tool's tick. The Detail pane keeps
//! one line that leads here.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::time::{Duration, Instant};

use chrono::{Local, NaiveDate, TimeDelta};
use crw_core::inbox::mailman::{self, State};
use crw_core::inbox::{Message, Outgoing, Recipient};
use crw_core::ledger::{EventRow, Ledger};
use eframe::egui::{self, RichText};

use crate::board::{Action, Board, View};
use crate::markup::{self, Look};
use crate::portfolio::{Filter, Member, Portfolio, ProjectRow, Selection};
use crate::style::{self, icons};

/// How far back the toggle's ticks go, in days.
const WEEK: i64 = 7;

/// How often the mailman's lines are read again; a round's end reads them at once.
const EVERY: Duration = Duration::from_secs(60);

/// How many of the mailman's lines the pane shows.
const LINES: usize = 8;

/// The dispositions being typed, by project and message id (`Board::drafts`).
type Drafts = HashMap<(String, String), String>;

/// The mailman's last lines, as the ledger's `mail` rows last gave them (EP-51).
#[derive(Debug, Default)]
pub struct MailLog {
    read: Option<Instant>,
    lines: Vec<EventRow>,
}

impl MailLog {
    /// Read again at the next chance: a round has just ended.
    pub fn stale(&mut self) {
        self.read = None;
    }

    /// Reads the ledger's latest `mail` rows when they are a minute old or stale. A ledger busy
    /// past the window's wait keeps the lines read before and is asked again after the window's
    /// retry, as the Owner tasks pane's ticks are (EP-43, EP-50).
    fn refresh(&mut self, ledger: Option<&Path>, now: Instant) {
        if self
            .read
            .is_some_and(|at| now.saturating_duration_since(at) < EVERY)
        {
            return;
        }
        self.read = Some(now);
        let Some(path) = ledger else {
            self.lines.clear();
            return;
        };
        let rows = Ledger::open_read_only_waiting(path, crate::pace::LEDGER_WAIT)
            .and_then(|ledger| ledger.events(mailman::EVENT_KIND, Some(LINES)));
        match rows {
            Ok(rows) => self.lines = rows,
            Err(e) if e.is_busy() => {
                self.read = now.checked_sub(EVERY.saturating_sub(crate::pace::LEDGER_RETRY));
            }
            Err(_) => {}
        }
    }
}

impl Board {
    /// Every project's open messages and the mail its briefs hand on (EP-51); drawn while the
    /// pane is in front.
    pub fn inbox_pane(&mut self, ui: &mut egui::Ui) {
        let ledger = self.ledger().map(Path::to_path_buf);
        self.mail_log.refresh(ledger.as_deref(), Instant::now());
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.view.inbox_one, "This project only")
                .on_hover_text(
                    "Show the project selected in the list alone, whatever it holds, so that \
                     Compose… reaches it",
                );
            ui.checkbox(&mut self.view.inbox_ticked, "Ticked in the last week")
                .on_hover_text(
                    "Show the messages ticked in the last seven days, by the date their \
                     disposition carries (the tool's tick and a session's triage both write \
                     one); a message ticked without a dated disposition is not shown",
                );
        });
        let selected = match &self.view.selected {
            Some(Selection::Project(name)) => Some(name.clone()),
            _ => None,
        };
        let since = self
            .view
            .inbox_ticked
            .then(|| Local::now().date_naive() - TimeDelta::days(WEEK));
        let one = self.view.inbox_one;
        // Over every project whatever the toggles show, as the tab and the status bar count.
        let (open, urgent) = self.open_messages();
        let receivers = self
            .portfolio
            .projects
            .iter()
            .filter(|row| row.open_messages().0 > 0)
            .count();
        let mut words = summary(open, urgent, receivers);
        if let (true, Some(name)) = (one, selected.as_deref()) {
            words.push_str(&format!(" Shown: {name} alone."));
        }
        ui.label(RichText::new(words).weak());
        ui.separator();
        let rows = rows(&self.portfolio, &self.view, selected.as_deref());
        let drafts = &mut self.drafts;
        let log = &self.mail_log;
        let mut actions = Vec::new();
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if one && selected.is_none() {
                    ui.label(
                        RichText::new(
                            "Select a project in the list: while This project only is on, the \
                             pane shows it alone.",
                        )
                        .weak(),
                    );
                    return;
                }
                let receiving: Vec<&ProjectRow> = rows
                    .iter()
                    .copied()
                    .filter(|row| one || holds_any(row, since))
                    .collect();
                // The line above says when nothing is open; the body says only what it adds.
                match (receiving.is_empty(), one, since.is_some()) {
                    (true, true, _) => {
                        ui.label(
                            RichText::new(
                                "The selected project's folder is missing: nothing of it to show.",
                            )
                            .weak(),
                        );
                    }
                    (true, false, true) => {
                        ui.label(RichText::new("None was ticked in the last week.").weak());
                    }
                    _ => {}
                }
                for row in &receiving {
                    receiver(ui, row, since, one, drafts, &mut actions);
                }
                ui.add_space(12.0);
                outgoing(ui, &rows);
                ui.add_space(8.0);
                mail_lines(ui, log);
            });
        self.actions.extend(actions);
    }

    /// The open messages of every project, and how many of them are urgent: the pane's tab.
    pub fn open_messages(&self) -> (usize, usize) {
        self.portfolio
            .projects
            .iter()
            .map(ProjectRow::open_messages)
            .fold((0, 0), |(open, urgent), (o, u)| (open + o, urgent + u))
    }

    /// Drops the dispositions typed for messages no longer open (ticked from the phone, by `crw`
    /// or a session's triage, or gone) and the empty ones, once the files are read, so that none
    /// lingers with no box left to clear (EP-51's review).
    pub(crate) fn prune_drafts(&mut self) {
        let portfolio = &self.portfolio;
        self.drafts.retain(|(project, id), text| {
            !text.trim().is_empty() && open_message(portfolio, project, id)
        });
    }
}

/// Whether the files say `project`'s message `id` is no longer open: the project is read and its
/// inbox holds no such open message. A project not read yet says nothing, so a restart stays
/// refused on the safe side until the files land.
pub(crate) fn closed_message(portfolio: &Portfolio, project: &str, id: &str) -> bool {
    portfolio.project(project).is_some() && !open_message(portfolio, project, id)
}

/// Whether `project`'s inbox holds `id` open.
pub(crate) fn open_message(portfolio: &Portfolio, project: &str, id: &str) -> bool {
    portfolio
        .project(project)
        .and_then(|row| row.snapshot.inbox.as_ref())
        .is_some_and(|inbox| inbox.open().any(|message| message.id == id))
}

/// The projects the pane may show, in the list's order (its groups and sort key, its filters
/// left aside) and with their folders there; with This project only, the selected one alone.
fn rows<'a>(portfolio: &'a Portfolio, view: &View, selected: Option<&str>) -> Vec<&'a ProjectRow> {
    let (stacks, hosting) = (BTreeSet::new(), BTreeSet::new());
    let every = Filter {
        name: "",
        stacks: &stacks,
        hosting: &hosting,
    };
    portfolio
        .groups(view.sort, every)
        .into_iter()
        .flat_map(|group| group.members)
        .filter_map(|member| match member {
            Member::Project(i) => portfolio.projects.get(i),
            Member::Folder(_) => None,
        })
        .filter(|row| row.folder.is_some())
        .filter(|row| match (view.inbox_one, selected) {
            (true, Some(name)) => row.name().eq_ignore_ascii_case(name),
            (true, None) => false,
            (false, _) => true,
        })
        .collect()
}

/// The line under the toggles.
fn summary(open: usize, urgent: usize, receivers: usize) -> String {
    let urgent = match urgent {
        0 => String::new(),
        n => format!(", {n} urgent"),
    };
    match (open, receivers) {
        (0, _) => "No open message in any project.".to_owned(),
        (1, _) => format!("1 open message{urgent}."),
        (n, 1) => format!("{n} open messages in 1 project{urgent}."),
        (n, p) => format!("{n} open messages in {p} projects{urgent}."),
    }
}

/// Whether the receiver has messages for the pane: an open one, or (the toggle on, `since` its
/// first day) one ticked in the last week.
fn holds_any(row: &ProjectRow, since: Option<NaiveDate>) -> bool {
    row.open_messages().0 > 0 || since.is_some_and(|since| !recently_ticked(row, since).is_empty())
}

/// The day a ticked message's disposition carries: the first date in it, as the tool writes it
/// (`**Disposition (2026-11-24):** …`) and a session's triage does (`**Triaged at EP-4
/// (2026-11-23):** …`).
fn ticked_on(message: &Message) -> Option<NaiveDate> {
    let disposition = message.disposition.as_deref()?;
    let date = crw_core::roadmap::first_date(disposition)?;
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// The receiver's ticked messages whose disposition is dated `since` or later, the latest first.
fn recently_ticked(row: &ProjectRow, since: NaiveDate) -> Vec<(&Message, NaiveDate)> {
    let Some(inbox) = &row.snapshot.inbox else {
        return Vec::new();
    };
    let mut ticked: Vec<(&Message, NaiveDate)> = inbox
        .messages
        .iter()
        .filter(|message| message.done)
        .filter_map(|message| Some((message, ticked_on(message)?)))
        .filter(|&(_, on)| on >= since)
        .collect();
    ticked.sort_by_key(|&(_, on)| std::cmp::Reverse(on));
    ticked
}

/// One receiver: its name (a click selects it in the list), its state, its counts and Compose…,
/// then its open messages, each with its disposition box and Tick, and, the toggle on, those
/// ticked in the last week; shown alone (`history`), every message it ever ticked, folded, as
/// the Detail pane kept them.
fn receiver(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    since: Option<NaiveDate>,
    history: bool,
    drafts: &mut Drafts,
    actions: &mut Vec<Action>,
) {
    let name = row.name();
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
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
        let state = row.project.state;
        style::chip(ui, state.as_str(), style::state_colour(Some(state)));
        ui.label(RichText::new(receiver_words(row)).weak());
        compose_button(ui, row, actions);
    });
    for message in row.snapshot.inbox.iter().flat_map(|inbox| inbox.open()) {
        message_ui(ui, row, message, drafts, actions);
    }
    if let Some(since) = since {
        ticked(ui, row, since);
    }
    if history {
        every_ticked(ui, row);
    }
}

/// Every message the receiver ticked, whatever its age or its disposition's date, folded shut
/// with its disposition: the Detail pane's "Ticked messages" before EP-51 (EP-51's review).
fn every_ticked(ui: &mut egui::Ui, row: &ProjectRow) {
    let done: Vec<&Message> = row
        .snapshot
        .inbox
        .iter()
        .flat_map(|inbox| inbox.messages.iter().filter(|message| message.done))
        .collect();
    if done.is_empty() {
        return;
    }
    egui::CollapsingHeader::new(
        RichText::new(format!("Every ticked message: {}", done.len())).weak(),
    )
    .id_salt(("ticked messages", row.name()))
    .default_open(false)
    .show(ui, |ui| {
        for message in done {
            ui.label(RichText::new(&message.id).monospace().weak());
            markup::note(
                ui,
                &message.text,
                Look {
                    weak: true,
                    ..Look::default()
                },
            );
            if let Some(disposition) = &message.disposition {
                markup::note(
                    ui,
                    disposition,
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

/// What a receiver's line says beside its name.
fn receiver_words(row: &ProjectRow) -> String {
    let Some(inbox) = &row.snapshot.inbox else {
        return "no roadmap/inbox.md: Compose… creates it".to_owned();
    };
    let total = inbox.messages.len();
    match row.open_messages() {
        (0, _) if total == 0 => "no message yet".to_owned(),
        (0, _) => format!("no open message of {total}"),
        (n, 0) => format!("{n} open of {total}"),
        (n, u) => format!("{n} open of {total}, {u} urgent"),
    }
}

/// Compose… on a project (D-13; EP-20c): the owner's message into its inbox, which the append
/// creates when the project keeps none.
fn compose_button(ui: &mut egui::Ui, row: &ProjectRow, actions: &mut Vec<Action>) {
    if ui
        .add_enabled(row.folder.is_some(), egui::Button::new("Compose…").small())
        .on_hover_text(
            "Write a message from you into this project's roadmap/inbox.md, for its next session \
             to triage (D-13)",
        )
        .on_disabled_hover_text("The project's folder is missing")
        .clicked()
    {
        actions.push(Action::OpenCompose(row.name().to_owned()));
    }
}

/// One open message: its id, the urgent chip, from whom and when, its text in the Detail pane's
/// renderer, and its disposition box with Tick (D-13), as the Detail pane drew them.
fn message_ui(
    ui: &mut egui::Ui,
    row: &ProjectRow,
    message: &Message,
    drafts: &mut Drafts,
    actions: &mut Vec<Action>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&message.id).monospace().strong());
        if message.urgent {
            style::chip(ui, "urgent", style::red(ui))
                .on_hover_text("Flagged [urgent]: the owner is notified, not the session (D-13)");
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
    // A trailing `[urgent]` is the chip's to say (EP-20a).
    let text = if message.urgent {
        crw_core::inbox::without_urgent(&message.text)
    } else {
        &message.text
    };
    markup::note(
        ui,
        text,
        Look {
            sentences: true,
            ..Look::default()
        },
    );
    let key = (row.name().to_owned(), message.id.clone());
    let draft = drafts.entry(key).or_default();
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(draft)
                .id_salt(("disposition", row.name(), &message.id))
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

/// The receiver's messages ticked in the last week, each with the day and its disposition.
fn ticked(ui: &mut egui::Ui, row: &ProjectRow, since: NaiveDate) {
    let recent = recently_ticked(row, since);
    if recent.is_empty() {
        return;
    }
    ui.add_space(2.0);
    ui.label(RichText::new("Ticked in the last week").small().weak());
    for (message, on) in recent {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&message.id).monospace());
            ui.label(
                RichText::new(format!("ticked {}", on.format("%a %d %b")))
                    .small()
                    .weak(),
            );
            let weak = Look {
                weak: true,
                ..Look::default()
            };
            let text = markup::job(ui, &markup::reflow(&message.text), weak);
            ui.add(egui::Label::new(text).truncate());
        });
        if let Some(disposition) = &message.disposition {
            markup::note(
                ui,
                disposition,
                Look {
                    small: true,
                    ..Look::default()
                },
            );
        }
        ui.add_space(2.0);
    }
}

/// A `Handed on` block of a brief and what the mailman made of it (D-13; EP-20).
struct Letter<'a> {
    sender: &'a str,
    block: &'a Outgoing,
    state: Option<&'a State>,
}

/// Where a letter stands for the pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Standing {
    /// It waits, or its receiver is unknown: shown.
    Undelivered,
    /// The mailman has not read it yet, as before the window's first round: folded, so that a
    /// window just started does not list every block ever written as undelivered.
    Unread,
    /// Delivered, or counted at the mailman's first reading as the past, delivered by hand:
    /// folded.
    Delivered,
}

impl Letter<'_> {
    fn standing(&self) -> Standing {
        match (&self.block.recipient, self.state) {
            (Recipient::Unknown, _) | (_, Some(State::Unknown | State::Waits(_))) => {
                Standing::Undelivered
            }
            (_, Some(State::Delivered { .. } | State::Counted)) => Standing::Delivered,
            (Recipient::Project(_), None) => Standing::Unread,
        }
    }

    /// Whether it waits or names an unknown receiver.
    fn undelivered(&self) -> bool {
        self.standing() == Standing::Undelivered
    }

    /// Its state in words, and the colour they take (`None` for the weak text).
    fn words(&self, ui: &egui::Ui) -> (String, Option<egui::Color32>) {
        match (&self.block.recipient, self.state) {
            (Recipient::Unknown, _) | (_, Some(State::Unknown)) => {
                (State::Unknown.words(), Some(style::amber(ui)))
            }
            (_, Some(state @ State::Delivered { .. })) => (state.words(), Some(style::green(ui))),
            (_, Some(state @ State::Waits(_))) => (state.words(), Some(style::amber(ui))),
            (_, Some(state @ State::Counted)) => (state.words(), None),
            (Recipient::Project(name), None) => (
                format!("to {name}: the mailman reads it at its next round"),
                None,
            ),
        }
    }
}

/// The project's `Handed on` blocks with the mailman's marks; a block with no words after its
/// label is no letter, which the mailman never sends nor marks (`mailman::letters`), so it is
/// left out rather than shown unread for good (EP-51's review).
fn letters(row: &ProjectRow) -> impl Iterator<Item = Letter<'_>> {
    let worded = |block: &&Outgoing| !mailman::body(&block.text).is_empty();
    row.outgoing.iter().filter(worded).map(move |block| Letter {
        sender: row.name(),
        block,
        state: row
            .mail
            .iter()
            .find(|mark| mark.path == block.path && mark.line == block.line)
            .map(|mark| &mark.state),
    })
}

/// The mail the projects' briefs hand on, each sender's blocks in the list's order: those that
/// wait or name an unknown receiver shown; those the mailman has not read yet, and the delivered
/// and the counted, each folded shut under a header of its own.
fn outgoing(ui: &mut egui::Ui, rows: &[&ProjectRow]) {
    let all: Vec<Letter<'_>> = rows.iter().flat_map(|&row| letters(row)).collect();
    let standing = |which: Standing| -> Vec<&Letter<'_>> {
        all.iter()
            .filter(|letter| letter.standing() == which)
            .collect()
    };
    let undelivered = standing(Standing::Undelivered);
    ui.label(RichText::new("Handed on to other projects").strong());
    ui.add_space(2.0);
    if all.is_empty() {
        ui.label(RichText::new("No brief here hands a message on.").weak());
        return;
    }
    ui.label(
        RichText::new(match undelivered.len() {
            0 => "No block waits.".to_owned(),
            1 => "1 block is not delivered.".to_owned(),
            n => format!("{n} blocks are not delivered."),
        })
        .weak(),
    );
    for letter in &undelivered {
        letter_ui(ui, letter);
    }
    for (which, words, salt) in [
        (
            Standing::Unread,
            "Not read by the mailman yet",
            "unread letters",
        ),
        (Standing::Delivered, "Delivered", "delivered letters"),
    ] {
        let letters = standing(which);
        if letters.is_empty() {
            continue;
        }
        egui::CollapsingHeader::new(RichText::new(format!("{words}: {}", letters.len())).weak())
            .id_salt(salt)
            .default_open(false)
            .show(ui, |ui| {
                for letter in &letters {
                    letter_ui(ui, letter);
                }
            });
    }
}

/// One block: its sender, brief and receiver, what the mailman made of it, and its words.
fn letter_ui(ui: &mut egui::Ui, letter: &Letter<'_>) {
    let block = letter.block;
    ui.horizontal_wrapped(|ui| {
        ui.label(style::icon(icons::ICON_OUTBOX, None));
        ui.label(
            RichText::new(format!("{} {} → {}", letter.sender, block.brief, block.to)).strong(),
        );
        let (words, colour) = letter.words(ui);
        let text = RichText::new(words);
        ui.label(match colour {
            Some(colour) => text.color(colour),
            None => text.weak(),
        });
    });
    // The block's text after its label, which the line above says (EP-15a).
    markup::note(
        ui,
        mail_text(&block.text),
        Look {
            sentences: true,
            ..Look::default()
        },
    );
    ui.add_space(4.0);
}

/// A handed-on block's text without its `**Handed on (to …).**` label.
fn mail_text(text: &str) -> &str {
    let text = text.trim_start();
    text.strip_prefix("**Handed on")
        .and_then(|rest| rest.find("**").map(|end| rest[end + 2..].trim_start()))
        .unwrap_or(text)
}

/// The mailman's last lines, the newest first, in a fold shut by default whose header says when
/// the newest was written; `crw mail` prints the whole log.
fn mail_lines(ui: &mut egui::Ui, log: &MailLog) {
    let newest = log.lines.first().and_then(EventRow::at).map(|at| {
        at.with_timezone(&Local)
            .format("%a %d %b %H:%M")
            .to_string()
    });
    let title = match newest {
        Some(when) => format!("The mailman's last lines, the newest {when}"),
        None => "The mailman's last lines: none read".to_owned(),
    };
    egui::CollapsingHeader::new(RichText::new(title).weak())
        .id_salt("the mailman's lines")
        .default_open(false)
        .show(ui, |ui| {
            if log.lines.is_empty() {
                ui.label(RichText::new("No mail row in the ledger yet.").weak());
            }
            for row in &log.lines {
                ui.add(egui::Label::new(RichText::new(line_words(row)).small().weak()).wrap());
            }
            ui.label(
                RichText::new("crw mail prints the whole log.")
                    .small()
                    .weak(),
            );
        });
}

/// A `mail` row in words: when, the project and the detail.
fn line_words(row: &EventRow) -> String {
    let when = row.at().map_or_else(
        || row.at.clone(),
        |at| at.with_timezone(&Local).format("%d %b %H:%M").to_string(),
    );
    let project = row.project.as_deref().unwrap_or("-");
    let detail = row.detail.as_deref().unwrap_or_default();
    format!("{when}  {project}  {detail}")
}

/// The Detail pane's line in place of its inbox and its handed-on blocks (EP-51): how many
/// messages are open and urgent and how many letters are not delivered; Open in Inbox and
/// Compose….
pub(crate) fn detail_line(ui: &mut egui::Ui, row: &ProjectRow, actions: &mut Vec<Action>) {
    ui.add_space(10.0);
    ui.label(RichText::new("Inbox").strong());
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        ui.label(detail_words(row));
        if ui
            .small_button("Open in Inbox")
            .on_hover_text("The Inbox pane on this project alone: its messages ticked there")
            .clicked()
        {
            actions.push(Action::OpenInbox(row.name().to_owned()));
        }
        compose_button(ui, row, actions);
    });
}

/// What the Detail pane's line says.
fn detail_words(row: &ProjectRow) -> String {
    let messages = match &row.snapshot.inbox {
        None => "No roadmap/inbox.md: Compose… creates it.".to_owned(),
        Some(inbox) => {
            let total = inbox.messages.len();
            match row.open_messages() {
                (0, _) if total == 0 => "No message yet.".to_owned(),
                (0, _) => format!("No open message of {total}."),
                (1, 0) => format!("1 open message of {total}."),
                (1, _) => format!("1 open message of {total}, urgent."),
                (n, 0) => format!("{n} open messages of {total}."),
                (n, u) => format!("{n} open messages of {total}, {u} urgent."),
            }
        }
    };
    match letters(row).filter(Letter::undelivered).count() {
        0 => messages,
        1 => format!("{messages} 1 block it hands on is not delivered."),
        n => format!("{messages} {n} blocks it hands on are not delivered."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::testing::{self, Screen};
    use chrono::Utc;
    use crw_core::inbox::mailman::Mark;

    /// Whether `output` paints a text that contains `part`.
    fn shows(output: &egui::FullOutput, part: &str) -> bool {
        output.shapes.iter().any(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) => shape.galley.text().contains(part),
            _ => false,
        })
    }

    /// A click on the text `part` drawn by `draw`, the actions it queues.
    fn click(
        screen: &mut Screen,
        part: &str,
        mut draw: impl FnMut(&mut egui::Ui, &mut Vec<Action>),
    ) -> Vec<Action> {
        let output = screen.pass(Vec::new(), |ui| draw(ui, &mut Vec::new()));
        let at = testing::text_at(&output, part).unwrap_or_else(|| panic!("{part} is drawn"));
        let mut actions = Vec::new();
        for events in [
            testing::moved(at),
            testing::button(at, egui::PointerButton::Primary, true),
            testing::button(at, egui::PointerButton::Primary, false),
        ] {
            screen.pass(events, |ui| draw(ui, &mut actions));
        }
        actions
    }

    /// A board over the fake portfolio, its files read (as the Owner tasks pane's tests build
    /// one).
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

    fn names(board: &Board, selected: Option<&str>) -> Vec<String> {
        rows(&board.portfolio, &board.view, selected)
            .into_iter()
            .filter(|row| board.view.inbox_one || holds_any(row, None))
            .map(|row| row.name().to_owned())
            .collect()
    }

    /// The receivers with an open message in the list's order whatever its filters; This project
    /// only, the selected one alone whatever it holds; Open in Inbox selects it and sets it.
    #[test]
    fn the_pane_lists_the_receivers_in_the_lists_order_and_one_alone() {
        let scratch = Scratch::new("inbox-rows");
        let mut board = board(&scratch);
        // harbor holds IN-2 open and lantern its urgent IN-1; the rest hold no open message.
        let all = names(&board, None);
        assert_eq!(all, ["harbor", "lantern"]);
        assert_eq!(board.open_messages(), (2, 1));
        // A name filter in the list hides nothing here.
        board.view.filter = "orchard".to_owned();
        assert_eq!(names(&board, None), all);
        board.view.inbox_one = true;
        assert_eq!(names(&board, Some("orchard")), ["orchard"]);
        assert!(names(&board, None).is_empty());
        board.view.inbox_one = false;
        board.run(Action::OpenInbox("lantern".to_owned()));
        assert!(board.view.inbox_one);
        assert_eq!(
            board.view.selected,
            Some(Selection::Project("lantern".to_owned()))
        );
    }

    /// A receiver's block: its open message with the urgent chip and its sender, the ticked one
    /// left out; its name selects it, Compose… opens the dialog on it, and a disposition typed
    /// lets Tick tick the message.
    #[test]
    fn a_receiver_lists_its_open_messages_with_compose_and_tick() {
        let scratch = Scratch::new("inbox-receiver");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let lantern = portfolio.project("lantern").unwrap();
        let harbor = portfolio.project("harbor").unwrap();
        let mut screen = Screen::new();
        let mut drafts = Drafts::new();
        let output = screen.pass(Vec::new(), |ui| {
            receiver(ui, lantern, None, false, &mut drafts, &mut Vec::new());
        });
        assert!(shows(&output, "1 open of 1, 1 urgent"));
        assert!(shows(&output, "urgent"));
        assert!(shows(&output, "from harbor EP-3, 2026-10-28"));
        // The chip says the flag, so the text leaves it out.
        assert!(!shows(&output, "[urgent]"));
        let output = screen.pass(Vec::new(), |ui| {
            receiver(ui, harbor, None, false, &mut drafts, &mut Vec::new());
        });
        assert!(testing::text_at(&output, "IN-2").is_some());
        assert!(testing::text_at(&output, "IN-1").is_none(), "ticked");
        let mut draw = |ui: &mut egui::Ui, actions: &mut Vec<Action>| {
            receiver(ui, harbor, None, false, &mut drafts, actions);
        };
        assert_eq!(
            click(&mut screen, "harbor", &mut draw),
            [Action::Select(Selection::Project("harbor".to_owned()))]
        );
        assert_eq!(
            click(&mut screen, "Compose…", &mut draw),
            [Action::OpenCompose("harbor".to_owned())]
        );
        // Tick waits for a disposition.
        assert!(click(&mut screen, "Tick", &mut draw).is_empty());
        drafts.insert(
            ("harbor".to_owned(), "IN-2".to_owned()),
            "rolled into EP-5".to_owned(),
        );
        let mut draw = |ui: &mut egui::Ui, actions: &mut Vec<Action>| {
            receiver(ui, harbor, None, false, &mut drafts, actions);
        };
        assert_eq!(
            click(&mut screen, "Tick", &mut draw),
            [Action::TickMessage {
                project: "harbor".to_owned(),
                id: "IN-2".to_owned()
            }]
        );
    }

    /// The toggle's ticked messages are dated by their dispositions: harbor's IN-1, ticked on
    /// 2026-10-13, shows from a week that holds that day and not from a later one.
    #[test]
    fn the_ticked_of_the_week_are_dated_by_their_dispositions() {
        let scratch = Scratch::new("inbox-ticked");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let harbor = portfolio.project("harbor").unwrap();
        let day = |text: &str| NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap();
        let ticked: Vec<&str> = recently_ticked(harbor, day("2026-10-08"))
            .iter()
            .map(|(message, _)| message.id.as_str())
            .collect();
        assert_eq!(ticked, ["IN-1"]);
        assert!(recently_ticked(harbor, day("2026-10-14")).is_empty());
        let mut message = harbor.snapshot.inbox.as_ref().unwrap().messages[0].clone();
        assert_eq!(ticked_on(&message), Some(day("2026-10-13")));
        message.disposition = Some("**Triaged at EP-4 (2026-11-23):** parked".to_owned());
        assert_eq!(ticked_on(&message), Some(day("2026-11-23")));
        message.disposition = Some("**Disposition:** declined, no date".to_owned());
        assert_eq!(ticked_on(&message), None);
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| {
            receiver(
                ui,
                harbor,
                Some(day("2026-10-08")),
                false,
                &mut Drafts::new(),
                &mut Vec::new(),
            );
        });
        assert!(shows(&output, "Ticked in the last week"));
        assert!(shows(&output, "ticked Tue 13 Oct"));
        assert!(shows(&output, "parked in EP-4's brief notes"));
    }

    /// The outgoing mail: a block to an unknown receiver is not delivered and shows; one the
    /// mailman has not read yet folds under its header, as a delivered one does under its own.
    #[test]
    fn the_outgoing_mail_shows_what_is_not_delivered_and_folds_the_rest() {
        let scratch = Scratch::new("inbox-outgoing");
        let mut portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        // lantern's EP-2 hands on to harbor and to skylight, whom the registry does not know.
        let lantern = portfolio.project("lantern").unwrap().clone();
        assert_eq!(lantern.outgoing.len(), 2);
        let to_harbor = lantern
            .outgoing
            .iter()
            .find(|block| block.to == "harbor")
            .unwrap()
            .clone();
        // Before the mailman's first round: the one to harbor unread, the one to skylight not
        // delivered.
        let standings: Vec<(String, Standing)> = letters(&lantern)
            .map(|letter| (letter.block.to.clone(), letter.standing()))
            .collect();
        assert_eq!(
            standings,
            [
                ("harbor".to_owned(), Standing::Unread),
                ("skylight".to_owned(), Standing::Undelivered)
            ]
        );
        assert_eq!(
            detail_words(&lantern),
            "1 open message of 1, urgent. 1 block it hands on is not delivered."
        );
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| outgoing(ui, &[&lantern]));
        assert!(shows(&output, "Not read by the mailman yet: 1"));
        assert!(!shows(&output, "lantern EP-2 → harbor"), "folded");
        // The mailman delivered the one to harbor.
        let row = portfolio
            .projects
            .iter_mut()
            .find(|row| row.name() == "lantern")
            .unwrap();
        row.mail = vec![Mark {
            sender: "lantern".to_owned(),
            path: to_harbor.path.clone(),
            line: to_harbor.line,
            state: State::Delivered {
                receiver: "harbor".to_owned(),
                id: "IN-1".to_owned(),
                rewording: Vec::new(),
            },
        }];
        let lantern = portfolio.project("lantern").unwrap();
        let undelivered: Vec<String> = letters(lantern)
            .filter(Letter::undelivered)
            .map(|letter| letter.block.to.clone())
            .collect();
        assert_eq!(undelivered, ["skylight"]);
        let output = screen.pass(Vec::new(), |ui| outgoing(ui, &[lantern]));
        assert!(shows(&output, "1 block is not delivered."));
        assert!(shows(&output, "lantern EP-2 → skylight"));
        assert!(shows(&output, "not registered: shown, not delivered"));
        assert!(shows(&output, "Delivered: 1"));
        assert!(!shows(&output, "Not read by the mailman yet"));
        // Folded shut: the delivered block's line is not drawn.
        assert!(!shows(&output, "lantern EP-2 → harbor"));
        let output = screen.pass(Vec::new(), |ui| outgoing(ui, &[]));
        assert!(shows(&output, "No brief here hands a message on."));
    }

    /// The mailman's lines come from the ledger's `mail` rows, the newest first, read again only
    /// when a minute old or stale.
    #[test]
    fn the_mailmans_lines_come_from_the_ledger() {
        let scratch = Scratch::new("inbox-log");
        let path = scratch.0.join("ledger.sqlite");
        let ledger = Ledger::open(&path).unwrap();
        let now = Utc::now();
        for (ago, detail) in [
            (30, "first reading: 2 blocks counted"),
            (10, "delivered EP-3"),
        ] {
            ledger
                .add_event(
                    now - TimeDelta::minutes(ago),
                    mailman::EVENT_KIND,
                    Some("harbor"),
                    detail,
                )
                .unwrap();
        }
        ledger
            .add_event(now, "tick", Some("harbor"), "ticked OT-1")
            .unwrap();
        let mut log = MailLog::default();
        log.refresh(Some(&path), Instant::now());
        let details: Vec<&str> = log
            .lines
            .iter()
            .filter_map(|row| row.detail.as_deref())
            .collect();
        assert_eq!(
            details,
            ["delivered EP-3", "first reading: 2 blocks counted"]
        );
        assert!(line_words(&log.lines[0]).ends_with("  harbor  delivered EP-3"));
        ledger
            .add_event(now, mailman::EVENT_KIND, Some("lantern"), "delivered EP-9")
            .unwrap();
        log.refresh(Some(&path), Instant::now());
        assert_eq!(log.lines.len(), 2, "a minute has not passed");
        log.stale();
        log.refresh(Some(&path), Instant::now());
        assert_eq!(log.lines.len(), 3);
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| mail_lines(ui, &log));
        assert!(shows(&output, "The mailman's last lines, the newest "));
        log.refresh(None, Instant::now() + EVERY);
        assert!(log.lines.is_empty());
    }

    /// The Detail pane's line counts the messages and leads to the pane on the project alone.
    #[test]
    fn the_detail_line_counts_the_messages_and_opens_the_pane() {
        let scratch = Scratch::new("inbox-detail-line");
        let portfolio = crate::portfolio::read_files(&fake_store(&scratch));
        let harbor = portfolio.project("harbor").unwrap();
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| detail_line(ui, harbor, &mut Vec::new()));
        assert!(shows(&output, "1 open message of 2."));
        let mut draw = |ui: &mut egui::Ui, actions: &mut Vec<Action>| {
            detail_line(ui, harbor, actions);
        };
        assert_eq!(
            click(&mut screen, "Open in Inbox", &mut draw),
            [Action::OpenInbox("harbor".to_owned())]
        );
        assert_eq!(
            click(&mut screen, "Compose…", &mut draw),
            [Action::OpenCompose("harbor".to_owned())]
        );
        // harbor's one block, to lantern, is not read by the mailman yet here: not counted.
        let mut bare = harbor.clone();
        bare.snapshot.inbox = None;
        assert_eq!(
            detail_words(&bare),
            "No roadmap/inbox.md: Compose… creates it."
        );
    }

    /// The tray's Show brings the Inbox pane forward while a new inbox message is among the badge
    /// events its amber dot marks, and leaves the layout as it stands otherwise.
    #[test]
    fn the_trays_show_brings_the_pane_forward_for_new_mail() {
        use crate::layout::{Pane, default_layout};
        use crate::tray::Order;

        let scratch = Scratch::new("inbox-tray");
        let mut board = board(&scratch);
        let active = |tree: &egui_tiles::Tree<Pane>| -> Vec<Pane> {
            tree.active_tiles()
                .into_iter()
                .filter_map(|id| tree.tiles.get_pane(&id).copied())
                .collect()
        };
        let mut tree = default_layout();
        // An earlier Open in Inbox narrowed the pane to one project.
        board.view.inbox_one = true;
        board.obey(Order::Show);
        board.run_actions(&mut tree);
        assert!(!active(&tree).contains(&Pane::Inbox), "nothing new");
        assert!(board.view.inbox_one, "left as it was");
        // The first reading is taken as seen; the open messages read after it are new.
        board.toasts.mark(BTreeSet::new(), false);
        board
            .toasts
            .mark(crate::toasts::marks(&board.portfolio, Utc::now()), false);
        assert!(
            board
                .toasts
                .attention
                .unseen()
                .iter()
                .any(|mark| mark.kind == crw_core::notify::Kind::Inbox)
        );
        board.obey(Order::Show);
        board.run_actions(&mut tree);
        assert!(active(&tree).contains(&Pane::Inbox));
        assert!(active(&tree).contains(&Pane::Projects), "the list stays");
        // Every project's mail: the new message may be in any of them (the review's finding).
        assert!(!board.view.inbox_one);
        // Open in Inbox brings the pane forward too, on its project alone.
        let mut tree = default_layout();
        board.run(Action::OpenInbox("lantern".to_owned()));
        board.run_actions(&mut tree);
        assert!(active(&tree).contains(&Pane::Inbox));
        assert!(board.view.inbox_one);
    }

    /// The pane as drawn: both receivers and the mail handed on; This project only with nothing
    /// selected asks for a selection, and on a project shows its every ticked message folded.
    #[test]
    fn the_pane_draws_the_receivers_and_one_alone_with_its_history() {
        let scratch = Scratch::new("inbox-pane");
        let mut board = board(&scratch);
        let mut screen = Screen::new();
        let output = screen.pass(Vec::new(), |ui| board.inbox_pane(ui));
        assert!(shows(&output, "2 open messages in 2 projects, 1 urgent."));
        assert!(testing::text_at(&output, "harbor").is_some());
        assert!(testing::text_at(&output, "lantern").is_some());
        assert!(shows(&output, "Handed on to other projects"));
        assert!(shows(&output, "The mailman's last lines: none read"));
        assert!(!shows(&output, "Every ticked message"), "only one alone");
        board.view.inbox_one = true;
        let output = screen.pass(Vec::new(), |ui| board.inbox_pane(ui));
        assert!(shows(&output, "Select a project in the list"));
        board.view.selected = Some(Selection::Project("harbor".to_owned()));
        let output = screen.pass(Vec::new(), |ui| board.inbox_pane(ui));
        assert!(shows(&output, "Shown: harbor alone."));
        assert!(shows(&output, "Every ticked message: 1"));
        assert!(testing::text_at(&output, "lantern").is_none());
    }

    /// A disposition typed for a message ticked elsewhere since (the phone, `crw`, a session's
    /// triage) holds no restart and goes at the next files read (the review's finding).
    #[test]
    fn a_draft_for_a_message_ticked_elsewhere_holds_nothing() {
        let scratch = Scratch::new("inbox-drafts");
        let mut board = board(&scratch);
        let typed = |board: &Board| {
            board
                .restart_refusals()
                .iter()
                .any(|why| why.contains("a disposition is being typed"))
        };
        assert!(open_message(&board.portfolio, "harbor", "IN-2"));
        assert!(!open_message(&board.portfolio, "harbor", "IN-1"), "ticked");
        assert!(!open_message(&board.portfolio, "nowhere", "IN-2"));
        assert!(closed_message(&board.portfolio, "harbor", "IN-1"));
        // A project the files have not given says nothing.
        assert!(!closed_message(&board.portfolio, "nowhere", "IN-2"));
        board.drafts.insert(
            ("harbor".to_owned(), "IN-2".to_owned()),
            "parked in EP-9".to_owned(),
        );
        assert!(typed(&board));
        // harbor's IN-1 is ticked already: its draft is stale, as one would be after a tick
        // made elsewhere.
        board.drafts.clear();
        board.drafts.insert(
            ("harbor".to_owned(), "IN-1".to_owned()),
            "rolled into EP-5".to_owned(),
        );
        board
            .drafts
            .insert(("lantern".to_owned(), "IN-1".to_owned()), String::new());
        assert!(!typed(&board));
        board.prune_drafts();
        assert!(board.drafts.is_empty());
    }

    #[test]
    fn the_summary_counts_messages_and_receivers() {
        assert_eq!(summary(0, 0, 0), "No open message in any project.");
        assert_eq!(summary(1, 1, 1), "1 open message, 1 urgent.");
        assert_eq!(summary(3, 0, 1), "3 open messages in 1 project.");
        assert_eq!(summary(5, 2, 2), "5 open messages in 2 projects, 2 urgent.");
    }

    #[test]
    fn the_mail_leaves_its_label_to_the_line_above() {
        assert_eq!(
            mail_text("**Handed on (to Beta).** The registry's format is 2."),
            "The registry's format is 2."
        );
        assert_eq!(mail_text("No label here."), "No label here.");
    }
}
