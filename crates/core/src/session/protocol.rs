//! The adapter over `claude-codes` (D-14, D-19): the one module that names the protocol crate's
//! types. It reads each line a session prints in stream-json into the few [`Event`]s the tool uses,
//! and writes the lines the host sends: the handshake that opens the control protocol, the prompt,
//! the answers to the session's requests, the denial at a request's time limit and the interrupt.
//! The protocol is not specified for hosts other than Anthropic's SDKs and changes with the CLI
//! (risk 2), so a line the crate cannot read becomes [`Event::Unreadable`] and never stops a
//! session, and a request the tool does not handle is refused at once, so that the session never
//! waits on it.

use std::collections::HashMap;

use claude_codes::io::{DeferredToolUse, ResultMessage};
use claude_codes::{
    AskUserQuestionInput, ClaudeInput, ClaudeOutput, ContentBlock, ControlRequestMessage,
    ControlRequestPayload, ControlResponse, ControlResponseMessage, ControlResponsePayload,
    PermissionDeniedMessage, SystemSubtype, ToolPermissionRequest, ToolResultBlock,
    ToolResultContent,
};
use serde::Serialize;
use serde_json::Value;

use super::Event;

/// How much of an unreadable line its event keeps.
const EXCERPT: usize = 200;

/// The longest one-line summary of a tool call.
const SUMMARY: usize = 120;

/// How much of a permission's input the Decisions tile shows, as indented JSON (EP-14).
const INPUT_CHARS: usize = 4000;

/// How much of a tool's result the transcript keeps: its first lines, and at most this many
/// characters of them (EP-13).
const RESULT_LINES: usize = 12;
const RESULT_CHARS: usize = 1500;

/// The input fields that say what a tool call works on, in the order they are looked for.
const DETAIL_KEYS: [&str; 8] = [
    "command",
    "file_path",
    "notebook_path",
    "path",
    "pattern",
    "url",
    "query",
    "description",
];

/// The session's `system/init`: the run as the CLI reports it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Init {
    pub session_id: String,
    pub version: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub permission_mode: Option<String>,
    pub cwd: Option<String>,
    pub tools: Vec<String>,
    /// The protocol capabilities the CLI names: an open set that features are detected from, never
    /// assumed (D-15).
    pub capabilities: Vec<String>,
}

/// A tool call, in one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolUse {
    pub id: String,
    pub tool: String,
    /// The tool and what it works on: `Bash cargo test`, `Edit src/lib.rs`.
    pub summary: String,
}

/// What a tool call came back with, as a transcript shows it collapsed under the call (EP-13):
/// its first lines only, held in the window's memory and never written (D-3, D-26).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResult {
    /// The [`ToolUse::id`] it answers.
    pub tool_use_id: String,
    pub is_error: bool,
    /// How many lines the result held.
    pub lines: usize,
    /// Its first [`RESULT_LINES`] lines, cut to [`RESULT_CHARS`] characters.
    pub excerpt: String,
}

/// The session asks the host before a tool runs (D-18, D-19): a permission, or the owner's choice
/// in an `AskUserQuestion`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub id: String,
    pub tool: String,
    /// The call in one line, as [`ToolUse::summary`]; a question's text for `AskUserQuestion`.
    pub summary: String,
    /// Why the session asks, when the CLI says.
    pub reason: Option<String>,
    /// An `AskUserQuestion`'s questions; empty for a permission.
    pub questions: Vec<Question>,
    /// What a permission is asked for: the tool's input as indented JSON, cut to
    /// [`INPUT_CHARS`]; empty for a question, whose input its questions are (EP-14).
    pub input: String,
}

/// A tool call a `PreToolUse` hook deferred (D-19's fallback; EP-14): the run ended at it, and a
/// resume without a prompt runs it again for the hook to decide.
#[derive(Debug, Clone, PartialEq)]
pub struct Deferred {
    /// The tool use's id, by which the answer's hook knows it.
    pub id: String,
    pub tool: String,
    /// The call in one line; a question's text for `AskUserQuestion`.
    pub summary: String,
    /// An `AskUserQuestion`'s questions; empty for another tool.
    pub questions: Vec<Question>,
    /// The input as the session wrote it, which an answer builds on.
    pub input: Value,
}

impl Deferred {
    /// The deferred call as a [`Request`], for the tile's form: its input shown as a
    /// permission's is.
    pub fn request(&self) -> Request {
        Request {
            id: self.id.clone(),
            tool: self.tool.clone(),
            summary: self.summary.clone(),
            reason: Some("deferred by the tool's PreToolUse hook".to_owned()),
            questions: self.questions.clone(),
            input: if self.questions.is_empty() {
                indented(&self.input)
            } else {
                String::new()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub text: String,
    /// The short label the question carries.
    pub header: String,
    pub options: Vec<Choice>,
    pub multi_select: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub label: String,
    pub description: Option<String>,
}

/// A tool use refused without asking: by a deny rule, such as a neighbour folder's (D-18
/// addendum), or by the permission mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Denied {
    pub tool: String,
    pub message: String,
    /// What refused it, when the CLI says.
    pub reason: Option<String>,
}

/// The run's `result`.
#[derive(Debug, Clone, PartialEq)]
pub struct Finish {
    /// `success`, `error_during_execution`, `error_max_budget_usd`, …
    pub subtype: String,
    pub is_error: bool,
    /// The final text.
    pub text: Option<String>,
    pub turns: i32,
    /// The CLI's estimate at list prices; a subscription is not billed in dollars (D-23).
    pub cost_usd: f64,
    pub duration_ms: u64,
    pub stop_reason: Option<String>,
    /// The call a `PreToolUse` hook deferred (D-19's fallback): the run waits to be resumed.
    pub deferred: Option<Deferred>,
    /// How many tool uses were refused during the run.
    pub denials: usize,
    pub errors: Vec<String>,
}

impl Finish {
    /// Whether the run ended waiting on a deferred tool use (D-19).
    pub fn is_deferred(&self) -> bool {
        self.deferred.is_some() || self.stop_reason.as_deref() == Some("tool_deferred")
    }

    /// Whether the run ended as it should: a success, no error, nothing deferred.
    pub fn succeeded(&self) -> bool {
        self.subtype == "success" && !self.is_error && !self.is_deferred()
    }
}

/// A line the adapter could not read, kept for diagnosis (risk 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unreadable {
    pub error: String,
    /// The line's start.
    pub excerpt: String,
}

/// The host's answer to a [`Request`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// The tool runs as asked.
    Allow,
    /// The tool is refused; the session reads the message.
    Deny(String),
    /// An `AskUserQuestion` answered: one answer per question in their order, a multi-select
    /// question's labels joined with ", ", free text as typed; an empty answer leaves its question
    /// unanswered.
    Answers(Vec<String>),
}

/// What one line held: its events, and a line to send back at once.
#[derive(Debug, Default)]
pub struct Reading {
    pub events: Vec<Event>,
    pub reply: Option<String>,
}

/// One session's side of the protocol: the requests waiting for an answer and the handshake's id.
#[derive(Debug, Default)]
pub struct Protocol {
    pending: HashMap<String, ToolPermissionRequest>,
    handshake: Option<String>,
    sent: u64,
}

impl Protocol {
    /// The `initialize` control request that opens the control protocol (D-19); the session's
    /// answer arrives as [`Event::Handshake`].
    pub fn initialize(&mut self) -> String {
        let id = self.next_id("init");
        self.handshake = Some(id.clone());
        line(&ControlRequestMessage::initialize(id))
    }

    /// The prompt, as the run's first user message.
    pub fn prompt(text: &str) -> String {
        line(&ClaudeInput::user_message_without_session(text))
    }

    /// The `interrupt` control request: the session ends its turn with a result.
    pub fn interrupt(&mut self) -> String {
        let id = self.next_id("interrupt");
        line(&ControlRequestMessage::interrupt(id))
    }

    /// The line that answers a request, or why there is none: the request is not waiting
    /// (answered, withdrawn or never asked), or its input does not hold the questions answered.
    pub fn answer(&mut self, request_id: &str, response: &Response) -> Result<String, String> {
        let request = self
            .pending
            .get(request_id)
            .ok_or_else(|| format!("no request {request_id} is waiting for an answer"))?;
        let message = match response {
            Response::Allow => request.allow(request_id),
            Response::Deny(message) => request.deny(message.clone(), request_id),
            Response::Answers(answers) => {
                let by_index: HashMap<usize, String> = answers
                    .iter()
                    .enumerate()
                    .filter(|(_, answer)| !answer.is_empty())
                    .map(|(index, answer)| (index, answer.clone()))
                    .collect();
                request
                    .answer_questions(&by_index, request_id)
                    .map_err(|e| e.to_string())?
            }
        };
        self.pending.remove(request_id);
        Ok(line(&ControlResponseMessage::from(message)))
    }

    /// The line that answers a request at its time limit (EP-14): refused, and the turn
    /// interrupted, so that the session stops there with `message` to read rather than guess.
    pub fn time_out(&mut self, request_id: &str, message: &str) -> Result<String, String> {
        let request = self
            .pending
            .remove(request_id)
            .ok_or_else(|| format!("no request {request_id} is waiting for an answer"))?;
        Ok(line(&ControlResponseMessage::from(
            request.deny_and_stop(message, request_id),
        )))
    }

    /// Reads one line the session printed.
    pub fn read(&mut self, text: &str) -> Reading {
        let text = text.trim();
        if text.is_empty() {
            return Reading::default();
        }
        match ClaudeOutput::parse_json(text) {
            Ok(output) => self.output(output),
            // Of a user line the tool reads only the tool results: one the crate cannot read (the
            // prompt's echo without its session) is no news.
            Err(error)
                if error
                    .raw_json
                    .as_ref()
                    .is_some_and(|json| json["type"] == "user") =>
            {
                Reading::default()
            }
            Err(error) => {
                // A request the crate cannot read still waits for an answer: refuse it.
                let reply = error
                    .raw_json
                    .as_ref()
                    .filter(|json| json["type"] == "control_request")
                    .and_then(|json| json["request_id"].as_str())
                    .map(|id| refusal(id, "this host could not read the request"));
                Reading {
                    events: vec![Event::Unreadable(Unreadable {
                        error: error.error_message,
                        excerpt: shorten(text, EXCERPT),
                    })],
                    reply,
                }
            }
        }
    }

    fn output(&mut self, output: ClaudeOutput) -> Reading {
        let mut reading = Reading::default();
        let events = &mut reading.events;
        match output {
            ClaudeOutput::System(system) => match system.subtype {
                SystemSubtype::Init => events.push(match system.as_init() {
                    Some(init) => Event::Init(Init {
                        session_id: init.session_id,
                        version: init.claude_code_version,
                        model: init.model,
                        effort: init.effort,
                        permission_mode: init.permission_mode.map(|mode| mode.as_str().to_owned()),
                        cwd: init.cwd,
                        tools: init.tools,
                        capabilities: init.capabilities,
                    }),
                    None => unreadable("a system/init without the fields the tool reads"),
                }),
                SystemSubtype::PermissionDenied => {
                    match serde_json::from_value::<PermissionDeniedMessage>(system.data) {
                        // A subagent's refusals belong to its own transcript.
                        Ok(denied) if denied.agent_id.is_none() => {
                            events.push(Event::Denied(Denied {
                                tool: denied.tool_name,
                                message: denied.message,
                                reason: denied.decision_reason,
                            }))
                        }
                        Ok(_) => {}
                        Err(e) => {
                            events.push(unreadable(&format!("system/permission_denied: {e}")))
                        }
                    }
                }
                _ => {}
            },
            // A subagent's messages carry their parent's tool use; the session's own come without.
            ClaudeOutput::Assistant(message) if message.parent_tool_use_id.is_none() => {
                for block in message.message.content {
                    match block {
                        ContentBlock::Text(text) if !text.text.is_empty() => {
                            events.push(Event::Text(text.text));
                        }
                        ContentBlock::ToolUse(call) => events.push(Event::ToolUse(ToolUse {
                            summary: summary(&call.name, &call.input),
                            id: call.id,
                            tool: call.name,
                        })),
                        _ => {}
                    }
                }
            }
            // The tools' results come back as the user's turn; the prompt's echo holds none.
            ClaudeOutput::User(message) if message.parent_tool_use_id.is_none() => {
                for block in message.message.content {
                    if let ContentBlock::ToolResult(result) = block {
                        events.push(Event::ToolResult(tool_result(result)));
                    }
                }
            }
            ClaudeOutput::StreamEvent(stream) if stream.parent_tool_use_id.is_none() => {
                let event = &stream.event;
                if event["type"] == "content_block_delta"
                    && event["delta"]["type"] == "text_delta"
                    && let Some(text) = event["delta"]["text"].as_str()
                {
                    events.push(Event::TextDelta(text.to_owned()));
                }
            }
            ClaudeOutput::Result(result) => events.push(Event::Result(finish(result))),
            ClaudeOutput::ControlRequest(request) => match request.request {
                ControlRequestPayload::CanUseTool(tool) => {
                    let questions = questions(&tool.tool_name, &tool.input);
                    let (summary, input) = match questions.first() {
                        Some(question) => (shorten(&question.text, SUMMARY), String::new()),
                        None => (summary(&tool.tool_name, &tool.input), indented(&tool.input)),
                    };
                    events.push(Event::Request(Request {
                        id: request.request_id.clone(),
                        tool: tool.tool_name.clone(),
                        summary,
                        reason: tool.decision_reason.clone(),
                        questions,
                        input,
                    }));
                    self.pending.insert(request.request_id, tool);
                }
                other => {
                    let kind = match other {
                        ControlRequestPayload::HookCallback(_) => "hook_callback",
                        ControlRequestPayload::McpMessage(_) => "mcp_message",
                        ControlRequestPayload::Initialize(_) => "initialize",
                        ControlRequestPayload::Interrupt => "interrupt",
                        ControlRequestPayload::CanUseTool(_) => unreachable!("matched above"),
                    };
                    events.push(Event::Notice(format!(
                        "refused a {kind} request, which the tool does not handle"
                    )));
                    reading.reply = Some(refusal(
                        &request.request_id,
                        &format!("this host does not handle {kind} requests"),
                    ));
                }
            },
            ClaudeOutput::ControlResponse(response) => {
                let (id, answer) = match response.response {
                    ControlResponsePayload::Success { request_id, .. } => (request_id, Ok(())),
                    ControlResponsePayload::Error {
                        request_id, error, ..
                    } => (request_id, Err(error)),
                };
                if self.handshake.as_deref() == Some(id.as_str()) {
                    self.handshake = None;
                    events.push(Event::Handshake(answer));
                }
            }
            ClaudeOutput::ControlCancelRequest(cancel) => {
                self.pending.remove(&cancel.request_id);
                events.push(Event::Withdrawn(cancel.request_id));
            }
            ClaudeOutput::Error(error) => events.push(Event::Notice(format!(
                "the API answered with an error: {}",
                error.error.message
            ))),
            _ => {}
        }
        reading
    }

    fn next_id(&mut self, kind: &str) -> String {
        self.sent += 1;
        format!("crw-{kind}-{}", self.sent)
    }
}

fn finish(result: ResultMessage) -> Finish {
    Finish {
        subtype: result.subtype.as_str().to_owned(),
        is_error: result.is_error,
        text: result.result,
        turns: result.num_turns,
        cost_usd: result.total_cost_usd,
        duration_ms: result.duration_ms,
        stop_reason: result.stop_reason,
        deferred: result.deferred_tool_use.map(deferred),
        denials: result.permission_denials.len(),
        errors: result.errors,
    }
}

/// A deferred tool use as the tile shows it and an answer builds on it.
fn deferred(tool: DeferredToolUse) -> Deferred {
    let questions = questions(&tool.name, &tool.input);
    let summary = match questions.first() {
        Some(question) => shorten(&question.text, SUMMARY),
        None => summary(&tool.name, &tool.input),
    };
    Deferred {
        id: tool.id,
        tool: tool.name,
        summary,
        questions,
        input: tool.input,
    }
}

/// An `AskUserQuestion`'s input with the owner's `answers` added (D-19's fallback; EP-14): one
/// answer per question in their order, keyed by the question's text, an empty one left out as
/// unanswered, the questions kept as asked. It is what a hook's `allow` hands back as
/// `updatedInput`, the form the control protocol's answer takes too.
pub fn answered_input(input: &Value, answers: &[String]) -> Result<Value, String> {
    let parsed: AskUserQuestionInput = serde_json::from_value(input.clone())
        .map_err(|e| format!("the questions do not read: {e}"))?;
    if answers.len() > parsed.questions.len() {
        return Err(format!(
            "{} answers to {} questions",
            answers.len(),
            parsed.questions.len()
        ));
    }
    let answered: serde_json::Map<String, Value> = parsed
        .questions
        .iter()
        .zip(answers)
        .filter(|(_, answer)| !answer.is_empty())
        .map(|(question, answer)| (question.question.clone(), Value::String(answer.clone())))
        .collect();
    let mut updated = input.clone();
    updated
        .as_object_mut()
        .ok_or("the questions' input is not an object")?
        .insert("answers".to_owned(), Value::Object(answered));
    Ok(updated)
}

/// A tool's result as the transcript keeps it: a text result as it stands, the text blocks of a
/// structured one joined (another block named by its kind, `[image]`), then the first lines.
fn tool_result(block: ToolResultBlock) -> ToolResult {
    let text = match block.content {
        None => String::new(),
        Some(ToolResultContent::Text(text)) => text,
        Some(ToolResultContent::Structured(blocks)) => blocks
            .iter()
            .filter_map(|block| match block["type"].as_str()? {
                "text" => block["text"].as_str().map(str::to_owned),
                kind => Some(format!("[{kind}]")),
            })
            .collect::<Vec<_>>()
            .join("\n"),
    };
    let first: Vec<&str> = text.lines().take(RESULT_LINES).collect();
    ToolResult {
        tool_use_id: block.tool_use_id,
        is_error: block.is_error.unwrap_or(false),
        lines: text.lines().count(),
        excerpt: shorten(&first.join("\n"), RESULT_CHARS),
    }
}

/// An `AskUserQuestion`'s questions; none for any other tool or an input that does not read.
fn questions(tool: &str, input: &Value) -> Vec<Question> {
    if tool != "AskUserQuestion" {
        return Vec::new();
    }
    let Ok(input) = serde_json::from_value::<AskUserQuestionInput>(input.clone()) else {
        return Vec::new();
    };
    input
        .questions
        .into_iter()
        .map(|question| Question {
            text: question.question,
            header: question.header,
            options: question
                .options
                .into_iter()
                .map(|option| Choice {
                    label: option.label,
                    description: option.description,
                })
                .collect(),
            multi_select: question.multi_select,
        })
        .collect()
}

/// `<tool> <what it works on>` from the first field of [`DETAIL_KEYS`] the input holds, its first
/// line only, shortened; the tool alone when it holds none.
fn summary(tool: &str, input: &Value) -> String {
    let detail = DETAIL_KEYS
        .iter()
        .find_map(|key| input.get(*key)?.as_str())
        .and_then(|text| text.lines().map(str::trim).find(|line| !line.is_empty()));
    match detail {
        Some(detail) => shorten(&format!("{tool} {detail}"), SUMMARY),
        None => tool.to_owned(),
    }
}

/// A tool's input as indented JSON, cut to [`INPUT_CHARS`].
fn indented(input: &Value) -> String {
    let text = serde_json::to_string_pretty(input).unwrap_or_else(|_| input.to_string());
    shorten(&text, INPUT_CHARS)
}

/// `text` cut to `max` characters, the last one an ellipsis when it was longer.
fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut short: String = text.chars().take(max - 1).collect();
    short.push('…');
    short
}

fn unreadable(error: &str) -> Event {
    Event::Unreadable(Unreadable {
        error: error.to_owned(),
        excerpt: String::new(),
    })
}

/// The error response to a control request the tool does not answer otherwise.
fn refusal(request_id: &str, message: &str) -> String {
    line(&ControlResponseMessage::from(ControlResponse::error(
        request_id, message,
    )))
}

/// One protocol message as a line of stream-json.
fn line(message: &impl Serialize) -> String {
    let mut text = serde_json::to_string(message).expect("the protocol's messages serialise");
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn json_of(line: &str) -> Value {
        assert!(line.ends_with('\n'), "{line:?}");
        serde_json::from_str(line).unwrap()
    }

    fn read(protocol: &mut Protocol, value: Value) -> Reading {
        protocol.read(&value.to_string())
    }

    fn ask(id: &str) -> Value {
        json!({"type": "control_request", "request_id": id, "request": {
            "subtype": "can_use_tool",
            "tool_name": "AskUserQuestion",
            "tool_use_id": "toolu_ask",
            "input": {"questions": [
                {"question": "Which colour should the badge take?", "header": "Colour",
                 "multiSelect": false, "options": [
                    {"label": "Blue", "description": "The theme's accent"},
                    {"label": "Grey"}]},
                {"question": "Which rows?", "header": "Rows", "multiSelect": true,
                 "options": [{"label": "alpha"}, {"label": "beta"}]}
            ]}
        }})
    }

    #[test]
    fn writes_the_handshake_the_prompt_and_the_interrupt() {
        let mut protocol = Protocol::default();
        let init = json_of(&protocol.initialize());
        assert_eq!(init["type"], "control_request");
        assert_eq!(init["request"]["subtype"], "initialize");
        assert_eq!(init["request_id"], "crw-init-1");
        let prompt = json_of(&Protocol::prompt("Run EP-3 of alpha."));
        assert_eq!(prompt["type"], "user");
        assert_eq!(prompt["message"]["role"], "user");
        assert_eq!(prompt["message"]["content"][0]["type"], "text");
        assert_eq!(
            prompt["message"]["content"][0]["text"],
            "Run EP-3 of alpha."
        );
        let interrupt = json_of(&protocol.interrupt());
        assert_eq!(interrupt["request"]["subtype"], "interrupt");
        assert_eq!(interrupt["request_id"], "crw-interrupt-2");
    }

    #[test]
    fn reads_the_handshakes_answer_once() {
        let mut protocol = Protocol::default();
        let id = json_of(&protocol.initialize())["request_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let other = json!({"type": "control_response",
            "response": {"subtype": "success", "request_id": "elsewhere", "response": {}}});
        assert!(read(&mut protocol, other).events.is_empty());
        let ok = json!({"type": "control_response",
            "response": {"subtype": "success", "request_id": id, "response": {"commands": []}}});
        assert_eq!(
            read(&mut protocol, ok.clone()).events,
            [Event::Handshake(Ok(()))]
        );
        assert!(read(&mut protocol, ok).events.is_empty(), "answered once");

        let mut refused = Protocol::default();
        let id = json_of(&refused.initialize())["request_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let error = json!({"type": "control_response",
            "response": {"subtype": "error", "request_id": id, "error": "not supported"}});
        assert_eq!(
            read(&mut refused, error).events,
            [Event::Handshake(Err("not supported".to_owned()))]
        );
    }

    #[test]
    fn reads_the_init() {
        let line = json!({"type": "system", "subtype": "init", "session_id": "0f0e0d0c",
            "cwd": "C:\\Data\\alpha", "model": "claude-fable-5-1", "effort": "xhigh",
            "permissionMode": "acceptEdits", "claude_code_version": "2.1.316",
            "tools": ["Read", "Edit", "Bash"], "capabilities": ["alpha_feature", "beta_feature"],
            "apiKeySource": "none"});
        let events = Protocol::default().read(&line.to_string()).events;
        assert_eq!(
            events,
            [Event::Init(Init {
                session_id: "0f0e0d0c".to_owned(),
                version: Some("2.1.316".to_owned()),
                model: Some("claude-fable-5-1".to_owned()),
                effort: Some("xhigh".to_owned()),
                permission_mode: Some("acceptEdits".to_owned()),
                cwd: Some("C:\\Data\\alpha".to_owned()),
                tools: vec!["Read".to_owned(), "Edit".to_owned(), "Bash".to_owned()],
                capabilities: vec!["alpha_feature".to_owned(), "beta_feature".to_owned()],
            })]
        );
        // Other system messages say nothing the tool uses.
        let status = json!({"type": "system", "subtype": "status", "session_id": "0f0e0d0c",
            "status": "compacting"});
        assert!(
            Protocol::default()
                .read(&status.to_string())
                .events
                .is_empty()
        );
    }

    #[test]
    fn reads_text_and_tool_uses_in_order_and_leaves_subagents_out() {
        let message = |parent: Value| {
            json!({"type": "assistant", "session_id": "s", "parent_tool_use_id": parent,
            "message": {"id": "msg_1", "role": "assistant", "model": "claude-fable-5-1",
            "content": [
                {"type": "text", "text": "Reading the brief first."},
                {"type": "tool_use", "id": "toolu_1", "name": "Read",
                 "input": {"file_path": "C:\\Data\\alpha\\roadmap\\README.md"}},
                {"type": "thinking", "thinking": "…", "signature": "x"},
                {"type": "tool_use", "id": "toolu_2", "name": "Bash",
                 "input": {"command": "cargo test\ncargo clippy", "description": "Run the tests"}}
            ]}})
        };
        let events = Protocol::default()
            .read(&message(Value::Null).to_string())
            .events;
        assert_eq!(
            events,
            [
                Event::Text("Reading the brief first.".to_owned()),
                Event::ToolUse(ToolUse {
                    id: "toolu_1".to_owned(),
                    tool: "Read".to_owned(),
                    summary: "Read C:\\Data\\alpha\\roadmap\\README.md".to_owned(),
                }),
                Event::ToolUse(ToolUse {
                    id: "toolu_2".to_owned(),
                    tool: "Bash".to_owned(),
                    summary: "Bash cargo test".to_owned(),
                }),
            ]
        );
        let subagent = message(json!("toolu_agent")).to_string();
        assert!(Protocol::default().read(&subagent).events.is_empty());
    }

    #[test]
    fn reads_tool_results_collapsed_and_leaves_subagents_out() {
        let line = |content: Value, parent: Value, error: Value| {
            json!({"type": "user", "session_id": "0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b",
                "parent_tool_use_id": parent, "message": {"role": "user", "content": [
                    {"type": "tool_result", "tool_use_id": "toolu_1", "content": content,
                     "is_error": error}]}})
            .to_string()
        };
        let read = |text: String| Protocol::default().read(&text).events;
        assert_eq!(
            read(line(
                json!("cargo test\nok: 31 passed"),
                Value::Null,
                json!(false)
            )),
            [Event::ToolResult(ToolResult {
                tool_use_id: "toolu_1".to_owned(),
                is_error: false,
                lines: 2,
                excerpt: "cargo test\nok: 31 passed".to_owned(),
            })]
        );
        // Blocks are joined, a block that is not text named by its kind; an error is said.
        let blocks = json!([{"type": "text", "text": "one\ntwo"},
            {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "x"}}]);
        let [Event::ToolResult(result)] = &read(line(blocks, Value::Null, json!(true)))[..] else {
            panic!()
        };
        assert_eq!(
            (result.excerpt.as_str(), result.lines),
            ("one\ntwo\n[image]", 3)
        );
        assert!(result.is_error);
        // A long result keeps its first lines, and at most so many characters of them.
        let long: Vec<String> = (1..=40).map(|n| format!("line {n}")).collect();
        let [Event::ToolResult(result)] =
            &read(line(json!(long.join("\n")), Value::Null, Value::Null))[..]
        else {
            panic!()
        };
        assert_eq!(result.lines, 40);
        assert_eq!(result.excerpt.lines().count(), RESULT_LINES);
        let wide = "y".repeat(RESULT_CHARS * 2);
        let [Event::ToolResult(result)] = &read(line(json!(wide), Value::Null, Value::Null))[..]
        else {
            panic!()
        };
        assert_eq!(result.excerpt.chars().count(), RESULT_CHARS);
        // A subagent's results belong to its own transcript; the prompt's echo holds none.
        assert!(read(line(json!("x"), json!("toolu_agent"), Value::Null)).is_empty());
        let echo = json!({"type": "user", "session_id": "0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b",
            "message": {"role": "user", "content": [{"type": "text", "text": "Run EP-3."}]}});
        assert!(read(echo.to_string()).is_empty());
    }

    #[test]
    fn reads_streamed_text() {
        let delta = json!({"type": "stream_event", "session_id": "s", "uuid": "u",
            "parent_tool_use_id": null,
            "event": {"type": "content_block_delta", "index": 0,
                      "delta": {"type": "text_delta", "text": "Read"}}});
        assert_eq!(
            Protocol::default().read(&delta.to_string()).events,
            [Event::TextDelta("Read".to_owned())]
        );
        let start = json!({"type": "stream_event", "session_id": "s", "uuid": "u",
            "parent_tool_use_id": null, "event": {"type": "message_start"}});
        assert!(
            Protocol::default()
                .read(&start.to_string())
                .events
                .is_empty()
        );
    }

    #[test]
    fn answers_a_permission_once() {
        let mut protocol = Protocol::default();
        let request = json!({"type": "control_request", "request_id": "req-7", "request": {
            "subtype": "can_use_tool", "tool_name": "Bash", "tool_use_id": "toolu_7",
            "decision_reason": "Not in the allow list",
            "input": {"command": "git push", "description": "Push"}}});
        assert_eq!(
            read(&mut protocol, request).events,
            [Event::Request(Request {
                id: "req-7".to_owned(),
                tool: "Bash".to_owned(),
                summary: "Bash git push".to_owned(),
                reason: Some("Not in the allow list".to_owned()),
                questions: Vec::new(),
                input: "{\n  \"command\": \"git push\",\n  \"description\": \"Push\"\n}".to_owned(),
            })]
        );
        let answer = json_of(
            &protocol
                .answer("req-7", &Response::Deny("not now".to_owned()))
                .unwrap(),
        );
        assert_eq!(answer["type"], "control_response");
        assert_eq!(answer["response"]["subtype"], "success");
        assert_eq!(answer["response"]["request_id"], "req-7");
        assert_eq!(answer["response"]["response"]["behavior"], "deny");
        assert_eq!(answer["response"]["response"]["message"], "not now");
        assert!(protocol.answer("req-7", &Response::Allow).is_err());
        assert!(protocol.answer("req-8", &Response::Allow).is_err());
    }

    #[test]
    fn allows_with_the_input_as_asked() {
        let mut protocol = Protocol::default();
        let request = json!({"type": "control_request", "request_id": "req-1", "request": {
            "subtype": "can_use_tool", "tool_name": "Write",
            "input": {"file_path": "C:\\Data\\alpha\\notes.md", "content": "x"}}});
        read(&mut protocol, request);
        let answer = json_of(&protocol.answer("req-1", &Response::Allow).unwrap());
        assert_eq!(answer["response"]["response"]["behavior"], "allow");
        assert_eq!(
            answer["response"]["response"]["updatedInput"]["file_path"],
            "C:\\Data\\alpha\\notes.md"
        );
    }

    #[test]
    fn answers_questions_by_their_text_and_keeps_them() {
        let mut protocol = Protocol::default();
        let events = read(&mut protocol, ask("req-2")).events;
        let Event::Request(request) = &events[0] else {
            panic!("{events:?}")
        };
        assert_eq!(request.summary, "Which colour should the badge take?");
        assert_eq!(request.questions.len(), 2);
        assert_eq!(request.questions[0].header, "Colour");
        assert_eq!(
            request.questions[0].options[0],
            Choice {
                label: "Blue".to_owned(),
                description: Some("The theme's accent".to_owned()),
            }
        );
        assert!(request.questions[1].multi_select);
        // Permissions answers do not fit a permission, answers fit a question.
        let answer = protocol
            .answer(
                "req-2",
                &Response::Answers(vec!["Blue".to_owned(), "alpha, beta".to_owned()]),
            )
            .unwrap();
        let input = &json_of(&answer)["response"]["response"]["updatedInput"];
        assert_eq!(
            input["answers"]["Which colour should the badge take?"],
            "Blue"
        );
        assert_eq!(input["answers"]["Which rows?"], "alpha, beta");
        assert_eq!(input["questions"].as_array().unwrap().len(), 2);

        let mut partly = Protocol::default();
        read(&mut partly, ask("req-3"));
        let answer = partly
            .answer(
                "req-3",
                &Response::Answers(vec![String::new(), "beta".to_owned()]),
            )
            .unwrap();
        let answers = &json_of(&answer)["response"]["response"]["updatedInput"]["answers"];
        assert_eq!(answers.as_object().unwrap().len(), 1, "{answers}");
    }

    #[test]
    fn a_question_shows_no_input_and_a_long_input_is_cut() {
        let mut protocol = Protocol::default();
        let [Event::Request(question)] = &read(&mut protocol, ask("req-9")).events[..] else {
            panic!()
        };
        assert!(question.input.is_empty());
        let write = json!({"type": "control_request", "request_id": "req-10", "request": {
            "subtype": "can_use_tool", "tool_name": "Write",
            "input": {"content": "z".repeat(9000)}}});
        let [Event::Request(permission)] = &read(&mut protocol, write).events[..] else {
            panic!()
        };
        assert_eq!(permission.input.chars().count(), INPUT_CHARS);
        assert!(permission.input.starts_with("{\n  \"content\": \"zzz"));
        assert!(permission.input.ends_with('…'));
    }

    #[test]
    fn a_time_limit_refuses_and_interrupts() {
        let mut protocol = Protocol::default();
        read(&mut protocol, ask("req-11"));
        let reply = json_of(
            &protocol
                .time_out(
                    "req-11",
                    "No answer within 30 minutes: the run is deferred.",
                )
                .unwrap(),
        );
        let answer = &reply["response"]["response"];
        assert_eq!(reply["response"]["request_id"], "req-11");
        assert_eq!(answer["behavior"], "deny");
        assert_eq!(answer["interrupt"], true);
        assert!(answer["message"].as_str().unwrap().contains("deferred"));
        // It is answered: neither the owner nor the limit can answer it again.
        assert!(protocol.answer("req-11", &Response::Allow).is_err());
        assert!(protocol.time_out("req-11", "again").is_err());
    }

    #[test]
    fn a_deferred_tool_use_is_kept_whole() {
        let input = ask("req-0")["request"]["input"].clone();
        let line = json!({"type": "result", "subtype": "success", "is_error": false,
            "duration_ms": 900, "duration_api_ms": 700, "num_turns": 2, "session_id": "s",
            "total_cost_usd": 0.05, "result": "", "stop_reason": "tool_deferred",
            "deferred_tool_use": {"id": "toolu_d", "name": "AskUserQuestion", "input": input}});
        let [Event::Result(finish)] = &Protocol::default().read(&line.to_string()).events[..]
        else {
            panic!()
        };
        let deferred = finish.deferred.as_ref().unwrap();
        assert_eq!(deferred.id, "toolu_d");
        assert_eq!(deferred.summary, "Which colour should the badge take?");
        assert_eq!(deferred.questions.len(), 2);
        assert_eq!(deferred.input, input);
        let request = deferred.request();
        assert_eq!(
            (request.id.as_str(), request.input.as_str()),
            ("toolu_d", "")
        );
        // Another tool's call shows its input as a permission's does.
        let bash = Deferred {
            id: "toolu_e".to_owned(),
            tool: "Bash".to_owned(),
            summary: "Bash git push".to_owned(),
            questions: Vec::new(),
            input: json!({"command": "git push"}),
        };
        assert!(bash.request().input.contains("\"command\": \"git push\""));
    }

    #[test]
    fn answered_input_keys_answers_by_question_and_keeps_the_questions() {
        let input = ask("req-0")["request"]["input"].clone();
        let answered =
            answered_input(&input, &["Grey".to_owned(), "alpha, beta".to_owned()]).unwrap();
        assert_eq!(answered["questions"], input["questions"]);
        assert_eq!(
            answered["answers"],
            json!({"Which colour should the badge take?": "Grey", "Which rows?": "alpha, beta"})
        );
        // An empty answer leaves its question unanswered; too many answers do not fit.
        let partly = answered_input(&input, &[String::new(), "beta".to_owned()]).unwrap();
        assert_eq!(partly["answers"], json!({"Which rows?": "beta"}));
        let three = ["a", "b", "c"].map(str::to_owned);
        assert!(answered_input(&input, &three).is_err());
        assert!(answered_input(&json!({"command": "ls"}), &[]).is_err());
    }

    #[test]
    fn a_withdrawn_request_is_no_longer_answered() {
        let mut protocol = Protocol::default();
        read(&mut protocol, ask("req-4"));
        let cancel = json!({"type": "control_cancel_request", "request_id": "req-4"});
        assert_eq!(
            read(&mut protocol, cancel).events,
            [Event::Withdrawn("req-4".to_owned())]
        );
        assert!(protocol.answer("req-4", &Response::Allow).is_err());
    }

    #[test]
    fn refuses_the_requests_it_does_not_handle() {
        let mut protocol = Protocol::default();
        let hook = json!({"type": "control_request", "request_id": "req-5",
            "request": {"subtype": "hook_callback", "callback_id": "c1", "input": {}}});
        let reading = read(&mut protocol, hook);
        assert!(
            matches!(&reading.events[..], [Event::Notice(text)] if text.contains("hook_callback"))
        );
        let reply = json_of(reading.reply.as_deref().unwrap());
        assert_eq!(reply["response"]["subtype"], "error");
        assert_eq!(reply["response"]["request_id"], "req-5");
        // A request of a kind the crate does not know is refused too.
        let unknown = json!({"type": "control_request", "request_id": "req-6",
            "request": {"subtype": "brand_new_request"}});
        let reading = read(&mut protocol, unknown);
        assert!(matches!(&reading.events[..], [Event::Unreadable(_)]));
        assert_eq!(
            json_of(reading.reply.as_deref().unwrap())["response"]["request_id"],
            "req-6"
        );
    }

    #[test]
    fn an_unreadable_line_is_an_event_not_an_error() {
        let mut protocol = Protocol::default();
        let reading = protocol.read("not json at all");
        assert!(
            matches!(&reading.events[..], [Event::Unreadable(u)] if u.excerpt == "not json at all")
        );
        assert!(reading.reply.is_none());
        let unknown = json!({"type": "brand_new_event", "session_id": "s"});
        assert!(matches!(
            &read(&mut protocol, unknown).events[..],
            [Event::Unreadable(_)]
        ));
        assert!(protocol.read("   ").events.is_empty());
        // A user line the crate cannot read (here without its session) says nothing the tool uses.
        let echo = protocol.read(&Protocol::prompt("Go."));
        assert!(echo.events.is_empty() && echo.reply.is_none());
        let long = "x".repeat(500);
        let reading = protocol.read(&long);
        let [Event::Unreadable(unreadable)] = &reading.events[..] else {
            panic!()
        };
        assert_eq!(unreadable.excerpt.chars().count(), EXCERPT);
        assert!(unreadable.excerpt.ends_with('…'));
    }

    #[test]
    fn reads_a_refused_tool_use() {
        let denied = json!({"type": "system", "subtype": "permission_denied", "session_id": "s",
            "uuid": "u", "tool_name": "Edit", "tool_use_id": "toolu_9",
            "decision_reason": "deny rule", "message": "Edit to C:\\Data\\beta denied"});
        assert_eq!(
            Protocol::default().read(&denied.to_string()).events,
            [Event::Denied(Denied {
                tool: "Edit".to_owned(),
                message: "Edit to C:\\Data\\beta denied".to_owned(),
                reason: Some("deny rule".to_owned()),
            })]
        );
    }

    #[test]
    fn reads_the_result() {
        let result = |extra: Value| {
            let mut line = json!({"type": "result", "subtype": "success", "is_error": false,
                "duration_ms": 61000, "duration_api_ms": 52000, "num_turns": 9,
                "session_id": "s", "total_cost_usd": 1.25, "result": "Done: EP-3.",
                "stop_reason": "end_turn", "permission_denials": [
                    {"tool_name": "Edit", "tool_use_id": "toolu_9", "tool_input": {}}]});
            line.as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            Protocol::default().read(&line.to_string()).events
        };
        let [Event::Result(finish)] = &result(json!({}))[..] else {
            panic!()
        };
        assert!(finish.succeeded());
        assert_eq!(
            (finish.turns, finish.cost_usd, finish.denials),
            (9, 1.25, 1)
        );
        assert_eq!(finish.text.as_deref(), Some("Done: EP-3."));
        let [Event::Result(deferred)] = &result(json!({"stop_reason": "tool_deferred",
            "deferred_tool_use": {"id": "toolu_d", "name": "AskUserQuestion", "input": {}}}))[..]
        else {
            panic!()
        };
        assert!(deferred.is_deferred() && !deferred.succeeded());
        assert_eq!(
            deferred.deferred.as_ref().map(|d| d.tool.as_str()),
            Some("AskUserQuestion")
        );
        let [Event::Result(failed)] = &result(json!({"subtype": "error_during_execution",
            "is_error": true, "errors": ["it broke"]}))[..]
        else {
            panic!()
        };
        assert!(!failed.succeeded());
        assert_eq!(failed.errors, ["it broke"]);
    }

    #[test]
    fn summaries_fit_one_line() {
        assert_eq!(
            summary("Grep", &json!({"pattern": "fn main"})),
            "Grep fn main"
        );
        assert_eq!(summary("TodoWrite", &json!({"todos": []})), "TodoWrite");
        let long = summary(
            "Bash",
            &json!({"command": "echo ".to_owned() + &"y".repeat(300)}),
        );
        assert_eq!(long.chars().count(), SUMMARY);
        assert_eq!(shorten("short", 10), "short");
    }
}
