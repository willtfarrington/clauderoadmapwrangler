//! The defer route (D-19's fallback; EP-14): when the control protocol cannot carry a headless
//! run's questions, a `PreToolUse` hook passed with `--settings` ends the run at each
//! `AskUserQuestion` (`crw hook --defer`), the run's result names the deferred call, and the
//! owner's answer resumes the session without a prompt, a one-shot hook (`crw hook --answer
//! <file>`) handing it back for that call. Claude Code documents the round trip on its hooks page
//! ("Defer a tool call for later"): the decision counts only in a `-p` run, a prompt sent with the
//! resume drops the deferred call, and an `AskUserQuestion` needs `allow` with `updatedInput`
//! holding the questions and their answers. The decisions are made here for `crw` to print, and
//! the settings that install them are built here for the runner; the hooks run in exec form.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::protocol::{self, Deferred, Response};

/// The tool the defer route ends a run at.
pub const QUESTION_TOOL: &str = "AskUserQuestion";

/// How long Claude Code gives a decision hook, in seconds: it reads a small file and prints.
const TIMEOUT_S: u64 = 10;

/// What the deferral says, in Claude Code's debug log only.
const DEFERRED: &str = "the owner answers in Roadmap Wrangler, which resumes the session";

/// The owner's answer to a deferred call, as the runner writes it for the resume's hook.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answer {
    /// The deferred call's tool use id: the hook answers that call and no other.
    pub tool_use_id: String,
    /// `allow` or `deny`.
    pub decision: String,
    /// The input the call runs with: for a question, the questions and their answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<Value>,
    /// What the session reads of a refusal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl Answer {
    /// What `response` makes of `deferred`: its questions answered, the call allowed as written,
    /// or refused with a message. A question takes answers or a refusal, not a bare allow.
    pub fn new(deferred: &Deferred, response: &Response) -> Result<Self, String> {
        let (decision, updated_input, reason) = match response {
            Response::Answers(answers) => (
                "allow",
                Some(protocol::answered_input(&deferred.input, answers)?),
                None,
            ),
            Response::Allow if !deferred.questions.is_empty() => {
                return Err("a question takes its answers, not a bare allow".to_owned());
            }
            Response::Allow => ("allow", Some(deferred.input.clone()), None),
            Response::Deny(message) => ("deny", None, Some(message.clone())),
        };
        Ok(Self {
            tool_use_id: deferred.id.clone(),
            decision: decision.to_owned(),
            updated_input,
            reason,
        })
    }
}

/// What `crw hook --defer` prints, whatever the call: deferred, so the run ends at it.
pub fn defer() -> String {
    decision("defer", None, Some(DEFERRED))
}

/// What `crw hook --answer <file>` prints for the hook's input `input` (its stdin): the file's
/// answer for the call the file names; a deferral for any other question, so that a later one ends
/// the run as the first did; nothing for another tool, which takes the usual permission flow. A
/// file that is missing or does not read defers its question again, to be answered once more.
pub fn answer(input: &str, file: &Path) -> Option<String> {
    let hook: Value = serde_json::from_str(input).unwrap_or(Value::Null);
    let tool = hook["tool_name"].as_str().unwrap_or_default();
    let id = hook["tool_use_id"].as_str().unwrap_or_default();
    let answer = fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str::<Answer>(&text).ok());
    match answer {
        Some(answer) if !id.is_empty() && answer.tool_use_id == id => Some(decision(
            &answer.decision,
            answer.updated_input,
            answer.reason.as_deref(),
        )),
        _ if tool == QUESTION_TOOL => Some(defer()),
        _ => None,
    }
}

/// One `PreToolUse` decision, as a hook prints it.
fn decision(decision: &str, updated_input: Option<Value>, reason: Option<&str>) -> String {
    let mut output = json!({"hookEventName": "PreToolUse", "permissionDecision": decision});
    if let Some(reason) = reason {
        output["permissionDecisionReason"] = json!(reason);
    }
    if let Some(input) = updated_input {
        output["updatedInput"] = input;
    }
    json!({ "hookSpecificOutput": output }).to_string()
}

/// The `hooks` block for `--settings` that runs `program` (`crw.exe`) with `args` before each
/// call of the tools `matcher` names, in exec form (no shell, no quoting).
pub fn hooks(program: &Path, args: &[&str], matcher: &str) -> Value {
    json!({"PreToolUse": [{"matcher": matcher, "hooks": [{
        "type": "command",
        "command": program.to_string_lossy(),
        "args": args,
        "timeout": TIMEOUT_S,
    }]}]})
}

/// The matcher of an answer's hook: questions, and the deferred call's own tool when it is
/// another (a project's own hook may defer any tool). A name that is not a plain identifier is
/// left out rather than read as a pattern.
pub fn answer_matcher(tool: &str) -> String {
    let plain = !tool.is_empty() && tool.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if tool == QUESTION_TOOL || !plain {
        QUESTION_TOOL.to_owned()
    } else {
        format!("{QUESTION_TOOL}|{tool}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;
    use crate::session::Question;

    fn deferred() -> Deferred {
        Deferred {
            id: "toolu_d".to_owned(),
            tool: QUESTION_TOOL.to_owned(),
            summary: "Which colour?".to_owned(),
            questions: vec![Question {
                text: "Which colour?".to_owned(),
                header: "Colour".to_owned(),
                options: Vec::new(),
                multi_select: false,
            }],
            input: json!({"questions": [{"question": "Which colour?", "header": "Colour",
                "multiSelect": false, "options": [{"label": "Blue"}, {"label": "Grey"}]}]}),
        }
    }

    fn hook_input(tool: &str, id: &str) -> String {
        json!({"session_id": "s", "hook_event_name": "PreToolUse", "tool_name": tool,
            "tool_input": {}, "tool_use_id": id})
        .to_string()
    }

    fn output(text: &str) -> Value {
        serde_json::from_str::<Value>(text).unwrap()["hookSpecificOutput"].clone()
    }

    #[test]
    fn the_deferral_is_a_pre_tool_use_decision() {
        let out = output(&defer());
        assert_eq!(out["hookEventName"], "PreToolUse");
        assert_eq!(out["permissionDecision"], "defer");
        assert!(out.get("updatedInput").is_none());
    }

    #[test]
    fn an_answer_allows_its_call_with_the_answers_and_defers_another_question() {
        let scratch = Scratch::new("defer-answer");
        let file = scratch.0.join("answer.json");
        let answer = Answer::new(&deferred(), &Response::Answers(vec!["Grey".to_owned()])).unwrap();
        fs::write(&file, serde_json::to_string(&answer).unwrap()).unwrap();

        let out = output(&super::answer(&hook_input(QUESTION_TOOL, "toolu_d"), &file).unwrap());
        assert_eq!(out["permissionDecision"], "allow");
        assert_eq!(
            out["updatedInput"]["answers"],
            json!({"Which colour?": "Grey"})
        );
        assert_eq!(
            out["updatedInput"]["questions"],
            deferred().input["questions"]
        );
        // A later question ends the run again; another tool takes the usual flow.
        let later = super::answer(&hook_input(QUESTION_TOOL, "toolu_e"), &file).unwrap();
        assert_eq!(output(&later)["permissionDecision"], "defer");
        assert_eq!(super::answer(&hook_input("Bash", "toolu_f"), &file), None);
        // No file, or one that does not read: the question waits again.
        let missing = scratch.0.join("gone.json");
        let again = super::answer(&hook_input(QUESTION_TOOL, "toolu_d"), &missing).unwrap();
        assert_eq!(output(&again)["permissionDecision"], "defer");
        fs::write(&file, "not json").unwrap();
        let unread = super::answer(&hook_input(QUESTION_TOOL, "toolu_d"), &file).unwrap();
        assert_eq!(output(&unread)["permissionDecision"], "defer");
        assert_eq!(super::answer("not json", &file), None);
    }

    #[test]
    fn a_refusal_and_another_tools_allow() {
        let refused = Answer::new(&deferred(), &Response::Deny("not now".to_owned())).unwrap();
        assert_eq!(
            (refused.decision.as_str(), refused.reason.as_deref()),
            ("deny", Some("not now"))
        );
        assert!(Answer::new(&deferred(), &Response::Allow).is_err());
        let bash = Deferred {
            tool: "Bash".to_owned(),
            questions: Vec::new(),
            input: json!({"command": "git push"}),
            ..deferred()
        };
        let allowed = Answer::new(&bash, &Response::Allow).unwrap();
        assert_eq!(allowed.updated_input, Some(json!({"command": "git push"})));
        let scratch = Scratch::new("defer-deny");
        let file = scratch.0.join("answer.json");
        fs::write(&file, serde_json::to_string(&refused).unwrap()).unwrap();
        let out = output(&super::answer(&hook_input(QUESTION_TOOL, "toolu_d"), &file).unwrap());
        assert_eq!(out["permissionDecision"], "deny");
        assert_eq!(out["permissionDecisionReason"], "not now");
    }

    #[test]
    fn the_hooks_run_the_program_in_exec_form() {
        let program = Path::new(r"C:\Tools\roadmap\crw.exe");
        let hooks = hooks(program, &["hook", "--defer"], QUESTION_TOOL);
        let group = &hooks["PreToolUse"][0];
        assert_eq!(group["matcher"], QUESTION_TOOL);
        let handler = &group["hooks"][0];
        assert_eq!(handler["type"], "command");
        assert_eq!(handler["command"], r"C:\Tools\roadmap\crw.exe");
        assert_eq!(handler["args"], json!(["hook", "--defer"]));
        assert_eq!(answer_matcher(QUESTION_TOOL), QUESTION_TOOL);
        assert_eq!(answer_matcher("Bash"), "AskUserQuestion|Bash");
        assert_eq!(
            answer_matcher("mcp__notes__save"),
            "AskUserQuestion|mcp__notes__save"
        );
        assert_eq!(answer_matcher("odd.*"), QUESTION_TOOL);
    }
}
