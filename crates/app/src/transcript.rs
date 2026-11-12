//! A headless session's transcript as the Sessions tile shows it (EP-13), built from the runner's
//! events as they come: the message a run started or resumed with, the assistant's text (its
//! partial text appended while it streams), each tool call in one line with its result collapsed
//! under it, the refusals, what the session asked the host with the owner's answer (EP-14), the
//! notices and the result. It lives in the window's memory only: the tool writes no transcript
//! and reads none from disk (D-3, D-26).

use chrono::{DateTime, Local};
use crw_core::session::{Denied, Event, Finish, Request, ToolResult, ToolUse};
use eframe::egui::{self, RichText};

use crate::style::{self, icons};

/// The most entries a transcript keeps; the oldest go first, and the count is said.
const ENTRIES: usize = 2000;

/// One thing a transcript shows.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    /// The message a run started with, or a resumed run's (D-16's EP-9 addendum).
    Prompt {
        text: String,
        at: DateTime<Local>,
        resumed: bool,
    },
    Text(String),
    /// A tool call, and its result once it came back.
    Tool {
        call: ToolUse,
        result: Option<ToolResult>,
    },
    Denied(Denied),
    /// A permission or a question the session asked the host (D-19), and what became of it in
    /// words once it is known: the owner's answer, withdrawn, or deferred at the time limit.
    Asked {
        request: Request,
        answer: Option<String>,
    },
    Notice(String),
    Result(Finish),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Transcript {
    pub entries: Vec<Entry>,
    /// The text streaming in, until its block is whole.
    pub streaming: String,
    /// How many of the oldest entries were let go to keep within [`ENTRIES`].
    pub dropped: usize,
}

impl Transcript {
    /// The message a run starts or resumes with.
    pub fn prompt(&mut self, text: &str, resumed: bool) {
        self.streaming.clear();
        self.push(Entry::Prompt {
            text: text.to_owned(),
            at: Local::now(),
            resumed,
        });
    }

    /// Takes in one of the session's events.
    pub fn take(&mut self, event: &Event) {
        match event {
            Event::TextDelta(text) => self.streaming.push_str(text),
            Event::Text(text) => {
                self.streaming.clear();
                self.push(Entry::Text(text.clone()));
            }
            Event::ToolUse(call) => self.push(Entry::Tool {
                call: call.clone(),
                result: None,
            }),
            Event::ToolResult(came) => {
                // Under the call it answers, the latest first; a result whose call was let go
                // goes with it.
                let slot = self.entries.iter_mut().rev().find_map(|entry| match entry {
                    Entry::Tool { call, result } if call.id == came.tool_use_id => Some(result),
                    _ => None,
                });
                if let Some(slot) = slot {
                    *slot = Some(came.clone());
                }
            }
            Event::Request(request) => self.push(Entry::Asked {
                request: request.clone(),
                answer: None,
            }),
            Event::Withdrawn(id) => self.answered(id, "withdrawn by the session"),
            Event::TimedOut(id) => self.answered(
                id,
                "no answer within the project's time limit: the run is deferred",
            ),
            Event::Denied(denied) => self.push(Entry::Denied(denied.clone())),
            Event::Notice(text) => self.push(Entry::Notice(text.clone())),
            Event::Handshake(Err(e)) => self.push(Entry::Notice(format!(
                "the control protocol's handshake was refused: {e}"
            ))),
            Event::Result(finish) => {
                self.streaming.clear();
                self.push(Entry::Result(finish.clone()));
            }
            Event::Init(_) | Event::Handshake(Ok(())) | Event::Unreadable(_) | Event::Exited(_) => {
            }
        }
    }

    /// Says under the request `id` what became of it, unless that is said already.
    pub fn answered(&mut self, id: &str, words: &str) {
        let slot = self.entries.iter_mut().rev().find_map(|entry| match entry {
            Entry::Asked { request, answer } if request.id == id => Some(answer),
            _ => None,
        });
        if let Some(answer) = slot
            && answer.is_none()
        {
            *answer = Some(words.to_owned());
        }
    }

    fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
        if self.entries.len() > ENTRIES {
            let excess = self.entries.len() - ENTRIES;
            self.entries.drain(..excess);
            self.dropped += excess;
        }
    }

    /// Draws the transcript top to bottom; `salt` keeps each session's collapsed results apart.
    pub fn ui(&self, ui: &mut egui::Ui, salt: u64) {
        if self.dropped > 0 {
            ui.label(
                RichText::new(format!(
                    "{} earlier entries were let go to keep the transcript short.",
                    self.dropped
                ))
                .small()
                .weak(),
            );
        }
        for (index, entry) in self.entries.iter().enumerate() {
            entry_ui(ui, entry, (salt, index));
            ui.add_space(3.0);
        }
        if !self.streaming.is_empty() {
            ui.label(RichText::new(format!("{}…", self.streaming)).weak());
        }
    }
}

fn entry_ui(ui: &mut egui::Ui, entry: &Entry, id: (u64, usize)) {
    let weak = ui.visuals().weak_text_color();
    match entry {
        Entry::Prompt { text, at, resumed } => {
            if *resumed {
                ui.separator();
            }
            // A header's text never wraps, so it stays short: a longer one would widen the
            // transcript past a narrow pane's edge.
            let (title, sent) = if *resumed {
                (
                    "Resumed",
                    format!("Resumed at {} with:", at.format("%H:%M")),
                )
            } else {
                ("Prompt", format!("Sent at {}:", at.format("%H:%M")))
            };
            egui::CollapsingHeader::new(RichText::new(title).color(weak))
                .id_salt(id)
                .default_open(*resumed)
                .show(ui, |ui| {
                    ui.label(RichText::new(sent).small().weak());
                    ui.label(RichText::new(text).weak());
                });
        }
        Entry::Text(text) => {
            ui.label(text);
        }
        Entry::Tool { call, result } => {
            tool_line(ui, &call.tool, &call.summary);
            if let Some(result) = result {
                let (head, colour) = if result.is_error {
                    (format!("failed · {} lines", result.lines), style::red(ui))
                } else {
                    (lines(result.lines), weak)
                };
                ui.indent(id, |ui| {
                    egui::CollapsingHeader::new(RichText::new(head).small().color(colour))
                        .id_salt(id)
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.label(RichText::new(&result.excerpt).monospace().small().weak());
                            let shown = result.excerpt.lines().count();
                            if result.lines > shown {
                                ui.label(
                                    RichText::new(format!(
                                        "… {} more",
                                        lines(result.lines - shown)
                                    ))
                                    .small()
                                    .weak(),
                                );
                            }
                        });
                });
            }
        }
        Entry::Denied(denied) => {
            ui.label(
                RichText::new(format!(
                    "Refused without asking: {} ({})",
                    denied.tool, denied.message
                ))
                .color(style::red(ui)),
            );
        }
        Entry::Asked { request, answer } => {
            ui.horizontal_wrapped(|ui| {
                ui.label(style::icon(
                    icons::ICON_HOURGLASS_TOP,
                    Some(style::amber(ui)),
                ));
                ui.label(
                    RichText::new(format!("Asked the host: {}", request.summary))
                        .color(style::amber(ui)),
                );
            });
            let (words, colour) = match answer {
                Some(words) => (words.as_str(), weak),
                None => ("waiting: the Decisions pane answers it", style::amber(ui)),
            };
            ui.indent(id, |ui| {
                ui.label(RichText::new(format!("→ {words}")).small().color(colour));
            });
        }
        Entry::Notice(text) => {
            ui.label(RichText::new(text).small().color(style::amber(ui)));
        }
        Entry::Result(finish) => {
            let colour = if finish.succeeded() {
                style::green(ui)
            } else if finish.is_deferred() {
                style::amber(ui)
            } else {
                style::red(ui)
            };
            if let Some(deferred) = &finish.deferred {
                ui.label(
                    RichText::new(format!(
                        "Deferred at {}: the Decisions pane answers it and resumes the session.",
                        deferred.summary
                    ))
                    .color(colour),
                );
            }
            ui.label(
                RichText::new(format!(
                    "Result: {} · {} turns · {} s · an estimated ${:.3}",
                    finish.subtype,
                    finish.turns,
                    finish.duration_ms / 1000,
                    finish.cost_usd
                ))
                .strong()
                .color(colour),
            );
            if let Some(text) = &finish.text {
                ui.label(text);
            }
            for error in &finish.errors {
                ui.label(RichText::new(error).color(style::red(ui)));
            }
        }
    }
}

/// A tool call in one line: its icon, then what it works on in monospace, cut at the pane's edge.
/// One label rather than a row of two, whose cut a narrow pane would not make.
pub(crate) fn tool_line(ui: &mut egui::Ui, tool: &str, summary: &str) {
    let icon = tool_icon(tool);
    let size = egui::TextStyle::Body.resolve(ui.style()).size;
    let mut job = egui::text::LayoutJob::default();
    job.append(
        icon.into(),
        0.0,
        egui::TextFormat::simple(
            egui::FontId::new(size, icon.font_family()),
            ui.visuals().weak_text_color(),
        ),
    );
    job.append(
        summary,
        6.0,
        egui::TextFormat::simple(
            egui::TextStyle::Monospace.resolve(ui.style()),
            ui.visuals().text_color(),
        ),
    );
    ui.add(egui::Label::new(job).truncate());
}

fn lines(count: usize) -> String {
    match count {
        0 => "no output".to_owned(),
        1 => "1 line".to_owned(),
        n => format!("{n} lines"),
    }
}

fn tool_icon(tool: &str) -> egui_material_icons::MaterialIcon {
    match tool {
        "Bash" | "PowerShell" => icons::ICON_TERMINAL,
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" => icons::ICON_EDIT,
        "Read" => icons::ICON_DESCRIPTION,
        "Grep" | "Glob" | "WebSearch" | "WebFetch" => icons::ICON_SEARCH,
        "AskUserQuestion" => icons::ICON_CHAT,
        "Agent" | "Task" => icons::ICON_SMART_TOY,
        _ => icons::ICON_BUILD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(id: &str, summary: &str) -> Event {
        Event::ToolUse(ToolUse {
            id: id.to_owned(),
            tool: summary.split(' ').next().unwrap().to_owned(),
            summary: summary.to_owned(),
        })
    }

    fn result(id: &str, lines: usize, error: bool) -> Event {
        Event::ToolResult(ToolResult {
            tool_use_id: id.to_owned(),
            is_error: error,
            lines,
            excerpt: "ok".to_owned(),
        })
    }

    #[test]
    fn a_runs_events_make_its_transcript() {
        let mut transcript = Transcript::default();
        transcript.prompt("Run EP-3 of alpha.", false);
        transcript.take(&Event::TextDelta("Reading ".to_owned()));
        transcript.take(&Event::TextDelta("the brief.".to_owned()));
        assert_eq!(transcript.streaming, "Reading the brief.");
        transcript.take(&Event::Text("Reading the brief.".to_owned()));
        assert!(transcript.streaming.is_empty());
        transcript.take(&call("toolu_1", "Bash cargo test"));
        transcript.take(&call("toolu_2", "Edit src/lib.rs"));
        // Results land under the call they answer, whatever their order.
        transcript.take(&result("toolu_2", 1, false));
        transcript.take(&result("toolu_1", 40, true));
        transcript.take(&result("toolu_9", 3, false));
        transcript.take(&Event::Handshake(Ok(())));
        transcript.take(&Event::Notice("the job was killed".to_owned()));
        let kinds: Vec<String> = transcript
            .entries
            .iter()
            .map(|entry| match entry {
                Entry::Prompt { resumed, .. } => format!("prompt {resumed}"),
                Entry::Text(text) => format!("text {text}"),
                Entry::Tool { call, result } => format!(
                    "tool {} {:?}",
                    call.summary,
                    result.as_ref().map(|r| (r.lines, r.is_error))
                ),
                Entry::Notice(text) => format!("notice {text}"),
                other => format!("{other:?}"),
            })
            .collect();
        assert_eq!(
            kinds,
            [
                "prompt false",
                "text Reading the brief.",
                "tool Bash cargo test Some((40, true))",
                "tool Edit src/lib.rs Some((1, false))",
                "notice the job was killed",
            ]
        );
        transcript.prompt("Go on from the tests.", true);
        assert!(matches!(
            transcript.entries.last(),
            Some(Entry::Prompt { resumed: true, .. })
        ));
    }

    #[test]
    fn a_request_says_what_became_of_it_once() {
        let request = |id: &str| {
            Event::Request(Request {
                id: id.to_owned(),
                tool: "Bash".to_owned(),
                summary: "Bash git push".to_owned(),
                reason: None,
                questions: Vec::new(),
                input: String::new(),
            })
        };
        let answer = |transcript: &Transcript, at: usize| match &transcript.entries[at] {
            Entry::Asked { answer, .. } => answer.clone(),
            other => panic!("{other:?}"),
        };
        let mut transcript = Transcript::default();
        transcript.take(&request("req-1"));
        transcript.take(&request("req-2"));
        transcript.take(&request("req-3"));
        assert_eq!(answer(&transcript, 0), None);
        transcript.answered("req-1", "allowed");
        transcript.answered("req-1", "denied: late");
        transcript.take(&Event::Withdrawn("req-2".to_owned()));
        transcript.take(&Event::TimedOut("req-3".to_owned()));
        assert_eq!(answer(&transcript, 0).as_deref(), Some("allowed"));
        assert_eq!(
            answer(&transcript, 1).as_deref(),
            Some("withdrawn by the session")
        );
        assert!(answer(&transcript, 2).unwrap().contains("deferred"));
    }

    #[test]
    fn a_long_run_lets_its_oldest_entries_go() {
        let mut transcript = Transcript::default();
        for n in 0..ENTRIES + 5 {
            transcript.take(&Event::Text(format!("line {n}")));
        }
        assert_eq!(transcript.entries.len(), ENTRIES);
        assert_eq!(transcript.dropped, 5);
        assert_eq!(transcript.entries[0], Entry::Text("line 5".to_owned()));
        assert_eq!(lines(0), "no output");
        assert_eq!(lines(1), "1 line");
    }
}
