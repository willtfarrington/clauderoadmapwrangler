//! The Decisions tile (D-19; EP-14): what the window's headless sessions ask the owner, answered
//! here. A permission request shows its tool, its call, why it asks and its input, with Allow and
//! Deny; an `AskUserQuestion` shows each question with its options, one or several, and a field
//! for the owner's own words, with Send answers or Decline. A run deferred at the project's time
//! limit or by the defer route's hook keeps its open request, answered by resuming the session.
//! The sessions Claude Code reports waiting elsewhere are listed with Focus for VS Code's, never
//! answered (D-19), and each session's tool uses refused without asking are listed with it.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use crw_core::session::{Request, Response};
use crw_core::sessions::{Mode, Seen, Status as SeenStatus};
use eframe::egui::{self, RichText};

use crate::board::{Action, Board};
use crate::style;

/// What a refusal says when the owner gives no reason.
const DENIED: &str = "The owner denied this.";

/// The answer form of one request, kept while the owner fills it in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Draft {
    /// The options picked, by question, by their index.
    pub picks: Vec<BTreeSet<usize>>,
    /// The owner's own words, by question.
    pub other: Vec<String>,
    /// A refusal's reason, or the message of a question declined.
    pub reason: String,
}

impl Draft {
    /// A blank form for `request`'s questions.
    pub fn new(request: &Request) -> Self {
        let questions = request.questions.len();
        Self {
            picks: vec![BTreeSet::new(); questions],
            other: vec![String::new(); questions],
            reason: String::new(),
        }
    }

    /// One answer per question (D-19): the labels picked, joined with ", ", then the owner's own
    /// words, which take the pick's place for a question of one answer; empty for a question left
    /// unanswered.
    pub fn answers(&self, request: &Request) -> Vec<String> {
        request
            .questions
            .iter()
            .enumerate()
            .map(|(q, question)| {
                let own = self
                    .other
                    .get(q)
                    .map(|text| text.trim())
                    .unwrap_or_default();
                let mut parts: Vec<&str> = self
                    .picks
                    .get(q)
                    .into_iter()
                    .flatten()
                    .filter_map(|&i| question.options.get(i).map(|c| c.label.as_str()))
                    .collect();
                if !own.is_empty() {
                    if !question.multi_select {
                        parts.clear();
                    }
                    parts.push(own);
                }
                parts.join(", ")
            })
            .collect()
    }

    /// How many questions have an answer.
    pub fn answered(&self, request: &Request) -> usize {
        self.answers(request)
            .iter()
            .filter(|answer| !answer.is_empty())
            .count()
    }

    /// Picks option `i` of question `q`: one pick for a question of one answer, a toggle for one
    /// of several.
    fn pick(&mut self, q: usize, i: usize, several: bool) {
        let Some(picks) = self.picks.get_mut(q) else {
            return;
        };
        if several {
            if !picks.remove(&i) {
                picks.insert(i);
            }
        } else {
            *picks = BTreeSet::from([i]);
        }
    }

    /// A refusal with the reason typed, or the plain words when none was.
    fn refusal(&self) -> Response {
        let reason = self.reason.trim();
        Response::Deny(if reason.is_empty() {
            DENIED.to_owned()
        } else {
            reason.to_owned()
        })
    }
}

/// What the owner's `response` to `request` was, in words, for the transcript and the status bar.
pub fn answer_words(request: &Request, response: &Response) -> String {
    match response {
        Response::Allow => "allowed".to_owned(),
        Response::Deny(reason) if request.questions.is_empty() => format!("denied: {reason}"),
        Response::Deny(reason) => format!("declined: {reason}"),
        Response::Answers(answers) => {
            let given: Vec<String> = request
                .questions
                .iter()
                .zip(answers)
                .map(|(question, answer)| match answer.as_str() {
                    "" => format!("{}: no answer", question.header),
                    answer => format!("{}: {answer}", question.header),
                })
                .collect();
            format!("answered: {}", given.join("; "))
        }
    }
}

/// The message that resumes a run deferred at its time limit with the owner's `response`: the
/// session reads in it what it asked and what the owner said (D-19).
pub fn resume_message(request: &Request, response: &Response) -> String {
    let what = if request.questions.is_empty() {
        "your permission request"
    } else {
        "your question"
    };
    let opening =
        format!("The run was deferred while {what} waited for the owner (D-19), who answers now.");
    match response {
        Response::Answers(answers) => {
            let mut text = format!("{opening} The answers:\n");
            for (question, answer) in request.questions.iter().zip(answers) {
                let answer = if answer.is_empty() {
                    "(no answer)"
                } else {
                    answer
                };
                text.push_str(&format!("- {}: {answer}\n", question.text));
            }
            text.push_str("Go on from where you stopped.");
            text
        }
        Response::Allow => format!(
            "{opening} {} is allowed: ask for it again, and go on from where you stopped.",
            request.summary
        ),
        Response::Deny(reason) if request.questions.is_empty() => format!(
            "{opening} {} is not allowed: {reason} Go on without it.",
            request.summary
        ),
        Response::Deny(reason) => {
            format!("{opening} They declined to answer: {reason} Go on from where you stopped.")
        }
    }
}

/// A span in minutes, or in seconds below one.
pub fn minutes(span: Duration) -> String {
    match span.as_secs() {
        s if s < 60 => format!("{s} s"),
        s => format!("{} min", s / 60),
    }
}

/// How a request has waited.
enum Wait {
    /// A running session's: since when, and the project's time limit.
    Live {
        since: Instant,
        limit: Option<Duration>,
    },
    /// An ended run's: when it was deferred, and whether by the hook (else by the time limit).
    Deferred { at: DateTime<Local>, hook: bool },
}

impl Wait {
    fn words(&self) -> String {
        match self {
            Wait::Live { since, limit } => {
                let waited = minutes(since.elapsed());
                match limit {
                    Some(limit) => {
                        let left = limit.saturating_sub(since.elapsed());
                        format!("waiting {waited} · deferred in {}", minutes(left))
                    }
                    None => format!("waiting {waited}"),
                }
            }
            Wait::Deferred { at, hook } => format!(
                "deferred at {} {}",
                at.format("%H:%M"),
                if *hook {
                    "by the defer route's hook"
                } else {
                    "at the project's time limit"
                }
            ),
        }
    }

    fn resumes(&self) -> bool {
        matches!(self, Wait::Deferred { .. })
    }
}

impl Board {
    /// What the Decisions tile holds for the owner: requests waiting here, deferred runs and the
    /// sessions waiting elsewhere.
    pub fn decisions_count(&self) -> usize {
        self.sessions.waiting() + self.sessions.deferred() + self.waiting_elsewhere().len()
    }

    /// The sessions Claude Code reports waiting for the owner outside the window (EP-12).
    fn waiting_elsewhere(&self) -> Vec<&Seen> {
        self.seen_sessions()
            .into_iter()
            .filter(|seen| seen.status == SeenStatus::Waiting)
            .collect()
    }

    /// The Decisions tile.
    pub fn decisions_pane(&mut self, ui: &mut egui::Ui) {
        let elsewhere: Vec<Seen> = self.waiting_elsewhere().into_iter().cloned().collect();
        let refused = self
            .sessions
            .live
            .iter()
            .any(|live| !live.refusals.is_empty());
        if self.sessions.waiting() == 0
            && self.sessions.deferred() == 0
            && elsewhere.is_empty()
            && !refused
        {
            placeholder(
                ui,
                "Nothing waits for you. A headless session's questions and permission requests \
                 come here while it waits, a run deferred at its time limit or by the defer \
                 route stays here until you answer it, and a session waiting in VS Code shows \
                 with Focus.",
            );
            return;
        }
        let starting = self.sessions.is_starting();
        let mut actions = Vec::new();
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if self.sessions.waiting() > 0 {
                    heading(ui, "Waiting here: the session holds until you answer");
                    for live in &mut self.sessions.live {
                        let (title, limit, key) = (live.title(), live.launch_timeout(), live.key);
                        for waiting in &live.waiting {
                            let id = &waiting.request.id;
                            let draft = live
                                .drafts
                                .entry(id.clone())
                                .or_insert_with(|| Draft::new(&waiting.request));
                            let wait = Wait::Live {
                                since: waiting.since,
                                limit,
                            };
                            if let Some(response) =
                                request_card(ui, &title, &waiting.request, draft, &wait, true)
                            {
                                actions.push(Action::Respond {
                                    key,
                                    request: id.clone(),
                                    response,
                                });
                            }
                            ui.add_space(6.0);
                        }
                    }
                }
                if self.sessions.deferred() > 0 {
                    heading(ui, "Deferred: your answer resumes the session");
                    for live in &mut self.sessions.live {
                        let (title, key) = (live.title(), live.key);
                        let Some(open) = live.open.as_ref().filter(|_| live.ended.is_some()) else {
                            continue;
                        };
                        let draft = live
                            .drafts
                            .entry(open.request.id.clone())
                            .or_insert_with(|| Draft::new(&open.request));
                        let wait = Wait::Deferred {
                            at: open.at,
                            hook: open.deferred.is_some(),
                        };
                        if let Some(response) =
                            request_card(ui, &title, &open.request, draft, &wait, !starting)
                        {
                            actions.push(Action::AnswerDeferred { key, response });
                        }
                        ui.add_space(6.0);
                    }
                }
                if !elsewhere.is_empty() {
                    heading(ui, "Waiting elsewhere: the tool never answers for them");
                    for seen in &elsewhere {
                        if elsewhere_card(ui, seen)
                            && let Some(project) = &seen.project
                        {
                            actions.push(Action::Focus(project.clone()));
                        }
                        ui.add_space(6.0);
                    }
                }
                if refused {
                    heading(
                        ui,
                        "Refused without asking: a deny rule, a neighbour's say (D-18)",
                    );
                    for live in &self.sessions.live {
                        refusals(ui, live);
                    }
                }
            });
        self.actions.extend(actions);
    }
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).weak());
    ui.add_space(2.0);
}

fn placeholder(ui: &mut egui::Ui, text: &str) {
    ui.add_space(24.0);
    ui.vertical_centered(|ui| ui.label(RichText::new(text).weak()));
}

fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

/// One request's card with its form; the owner's response when a button was pressed. `enabled`
/// is false while a resume is on its way.
fn request_card(
    ui: &mut egui::Ui,
    title: &str,
    request: &Request,
    draft: &mut Draft,
    wait: &Wait,
    enabled: bool,
) -> Option<Response> {
    let mut sent = None;
    card(ui, |ui| {
        // The title, the chip and the wait on rows of their own, so that a narrow pane is never
        // widened past its edge (EP-13) nor a line begun with half of another.
        ui.label(RichText::new(title).strong());
        ui.horizontal(|ui| style::chip(ui, &request.tool, style::amber(ui)));
        ui.add(egui::Label::new(RichText::new(wait.words()).small().weak()).wrap());
        if let Some(reason) = &request.reason {
            ui.add(egui::Label::new(RichText::new(reason).small().weak()).wrap());
        }
        ui.add_enabled_ui(enabled, |ui| {
            sent = if request.questions.is_empty() {
                permission_form(ui, request, draft, wait.resumes())
            } else {
                question_form(ui, request, draft, wait.resumes())
            };
        });
    });
    sent
}

/// A permission: the call, its input folded, Allow and Deny with a reason.
fn permission_form(
    ui: &mut egui::Ui,
    request: &Request,
    draft: &mut Draft,
    resumes: bool,
) -> Option<Response> {
    let mut sent = None;
    ui.add(egui::Label::new(RichText::new(&request.summary).monospace()).wrap());
    if !request.input.is_empty() {
        egui::CollapsingHeader::new(RichText::new("Input").small().weak())
            .id_salt(("input", &request.id))
            .show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(&request.input).monospace().small()).wrap());
            });
    }
    ui.add(
        egui::TextEdit::singleline(&mut draft.reason)
            .hint_text("A reason, if you deny it")
            .desired_width(f32::INFINITY),
    );
    let (allow, deny) = if resumes {
        ("Allow and resume", "Deny and resume")
    } else {
        ("Allow", "Deny")
    };
    ui.horizontal_wrapped(|ui| {
        if ui
            .button(allow)
            .on_hover_text("The tool runs as the session asked")
            .clicked()
        {
            sent = Some(Response::Allow);
        }
        if ui
            .button(RichText::new(deny).color(style::red(ui)))
            .on_hover_text("The session reads the reason and goes on without it")
            .clicked()
        {
            sent = Some(draft.refusal());
        }
    });
    sent
}

/// An `AskUserQuestion`: each question with its options and the owner's own words, Send answers,
/// or Decline with a message.
fn question_form(
    ui: &mut egui::Ui,
    request: &Request,
    draft: &mut Draft,
    resumes: bool,
) -> Option<Response> {
    let mut sent = None;
    for (q, question) in request.questions.iter().enumerate() {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            style::chip(ui, &question.header, ui.visuals().weak_text_color());
            if question.multi_select {
                ui.label(RichText::new("pick several").small().weak());
            }
        });
        ui.add(egui::Label::new(RichText::new(&question.text).strong()).wrap());
        for (i, choice) in question.options.iter().enumerate() {
            let picked = draft.picks.get(q).is_some_and(|picks| picks.contains(&i));
            let clicked = if question.multi_select {
                let mut on = picked;
                ui.checkbox(&mut on, &choice.label).changed()
            } else {
                ui.radio(picked, &choice.label).clicked()
            };
            if clicked {
                draft.pick(q, i, question.multi_select);
            }
            if let Some(description) = &choice.description {
                ui.indent((&request.id, q, i), |ui| {
                    ui.add(egui::Label::new(RichText::new(description).small().weak()).wrap());
                });
            }
        }
        if let Some(other) = draft.other.get_mut(q) {
            let hint = if question.multi_select {
                "Other: more, in your own words"
            } else {
                "Other: your own answer"
            };
            ui.add(
                egui::TextEdit::singleline(other)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY),
            );
        }
    }
    ui.add_space(6.0);
    let (answered, total) = (draft.answered(request), request.questions.len());
    let send = if resumes {
        "Answer and resume"
    } else {
        "Send answers"
    };
    let hover = if answered < total {
        format!("{answered} of {total} answered: the rest go unanswered")
    } else {
        "Every question has its answer".to_owned()
    };
    if ui
        .add_enabled(answered > 0, egui::Button::new(send))
        .on_hover_text(hover)
        .clicked()
    {
        sent = Some(Response::Answers(draft.answers(request)));
    }
    ui.add(
        egui::TextEdit::singleline(&mut draft.reason)
            .hint_text("Or decline them, saying why")
            .desired_width(f32::INFINITY),
    );
    let decline = if resumes {
        "Decline and resume"
    } else {
        "Decline"
    };
    if ui
        .add_enabled(!draft.reason.trim().is_empty(), egui::Button::new(decline))
        .on_hover_text("The session reads that you declined, and why")
        .clicked()
    {
        sent = Some(draft.refusal());
    }
    sent
}

/// Where a session waits, in words, and whether Focus can bring it forward: VS Code's can.
fn place(seen: &Seen) -> (&'static str, bool) {
    match seen.entrypoint.as_deref() {
        Some("claude-vscode") => ("in VS Code", true),
        _ if seen.mode == Mode::Visual => ("in VS Code", true),
        Some("cli") => ("in its terminal", false),
        _ => ("where it runs", false),
    }
}

/// A session Claude Code reports waiting outside the window: `true` when Focus was pressed.
fn elsewhere_card(ui: &mut egui::Ui, seen: &Seen) -> bool {
    let mut focus = false;
    card(ui, |ui| {
        ui.label(RichText::new(seen.project.as_deref().unwrap_or_default()).strong());
        ui.horizontal(|ui| {
            style::chip(ui, seen.mode.as_str(), ui.visuals().weak_text_color());
            style::chip(ui, "waiting", style::amber(ui));
        });
        let (where_, focusable) = place(seen);
        ui.add(
            egui::Label::new(
                RichText::new(format!("It {}.", seen.waits_for_you(where_)))
                    .color(style::amber(ui)),
            )
            .wrap(),
        );
        if let Some(name) = &seen.name {
            ui.add(egui::Label::new(RichText::new(name).weak()).truncate());
        }
        if focusable
            && ui
                .button("Focus")
                .on_hover_text(
                    "Bring its VS Code window forward: the project's own workspace file, or its \
                     folder, opened with Code.exe",
                )
                .clicked()
        {
            focus = true;
        }
    });
    focus
}

/// A session's tool uses refused without asking, folded under its title.
fn refusals(ui: &mut egui::Ui, live: &crate::sessions::Live) {
    if live.refusals.is_empty() {
        return;
    }
    // A folding header's text never wraps, so it stays short (EP-13).
    egui::CollapsingHeader::new(format!("{} · {}", live.title(), live.denied))
        .id_salt(("refusals", live.key))
        .show(ui, |ui| {
            for denied in &live.refusals {
                let reason = denied
                    .reason
                    .as_deref()
                    .map(|reason| format!(" ({reason})"))
                    .unwrap_or_default();
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{}: {}{reason}", denied.tool, denied.message))
                            .small()
                            .color(style::red(ui)),
                    )
                    .wrap(),
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use crw_core::session::{Choice, Question};

    use super::*;

    fn question(text: &str, header: &str, several: bool, labels: &[&str]) -> Question {
        Question {
            text: text.to_owned(),
            header: header.to_owned(),
            options: labels
                .iter()
                .map(|label| Choice {
                    label: (*label).to_owned(),
                    description: None,
                })
                .collect(),
            multi_select: several,
        }
    }

    fn asked() -> Request {
        Request {
            id: "req-1".to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which shelf?".to_owned(),
            reason: None,
            questions: vec![
                question("Which shelf?", "Shelf", false, &["Top", "Bottom"]),
                question("Which rows?", "Rows", true, &["alpha", "beta", "gamma"]),
            ],
            input: String::new(),
        }
    }

    #[test]
    fn a_draft_answers_one_several_and_in_the_owners_words() {
        let request = asked();
        let mut draft = Draft::new(&request);
        assert_eq!(draft.answers(&request), ["", ""]);
        assert_eq!(draft.answered(&request), 0);
        draft.pick(0, 0, false);
        draft.pick(0, 1, false);
        draft.pick(1, 2, true);
        draft.pick(1, 0, true);
        assert_eq!(draft.answers(&request), ["Bottom", "alpha, gamma"]);
        draft.pick(1, 0, true);
        assert_eq!(draft.answers(&request)[1], "gamma");
        // The owner's words take a single answer's place and join several.
        draft.other = vec!["  Middle ".to_owned(), "delta".to_owned()];
        assert_eq!(draft.answers(&request), ["Middle", "gamma, delta"]);
        assert_eq!(draft.answered(&request), 2);
        // A question left alone goes unanswered.
        let mut partly = Draft::new(&request);
        partly.pick(1, 1, true);
        assert_eq!(partly.answers(&request), ["", "beta"]);
        assert_eq!(partly.answered(&request), 1);
        assert_eq!(partly.refusal(), Response::Deny(DENIED.to_owned()));
        partly.reason = "Ask again tomorrow.".to_owned();
        assert_eq!(
            partly.refusal(),
            Response::Deny("Ask again tomorrow.".to_owned())
        );
    }

    #[test]
    fn answers_read_in_words_and_a_deferred_runs_resume_says_them() {
        let request = asked();
        let answers = Response::Answers(vec!["Top".to_owned(), String::new()]);
        assert_eq!(
            answer_words(&request, &answers),
            "answered: Shelf: Top; Rows: no answer"
        );
        let message = resume_message(&request, &answers);
        assert!(message.contains("- Which shelf?: Top\n- Which rows?: (no answer)\n"));
        assert!(message.ends_with("Go on from where you stopped."));
        let declined = Response::Deny("Not now.".to_owned());
        assert_eq!(answer_words(&request, &declined), "declined: Not now.");
        let push = Request {
            questions: Vec::new(),
            summary: "Bash git push".to_owned(),
            tool: "Bash".to_owned(),
            ..request
        };
        assert_eq!(answer_words(&push, &Response::Allow), "allowed");
        assert_eq!(answer_words(&push, &declined), "denied: Not now.");
        assert!(resume_message(&push, &Response::Allow).contains("Bash git push is allowed"));
        assert!(resume_message(&push, &declined).contains("is not allowed: Not now."));
        assert_eq!(minutes(Duration::from_secs(45)), "45 s");
        assert_eq!(minutes(Duration::from_secs(1830)), "30 min");
    }
}
