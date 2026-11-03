//! `crw hook --defer` and `crw hook --answer <file>`, the defer route's decisions (D-19; EP-14),
//! as Claude Code runs them: the hook's input on stdin, one JSON object or nothing on stdout,
//! exit 0 whatever happens. The answer file is a scratch one; no session runs.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const CRW: &str = env!("CARGO_BIN_EXE_crw");

/// `crw` with `args` and `input` on stdin, in a scratch `CRW_HOME`: its exit code and what it
/// printed.
fn crw(args: &[&str], input: &str) -> (i32, String) {
    let home = std::env::temp_dir().join(format!("crw-cli-{}-decisions-home", std::process::id()));
    let mut child = Command::new(CRW)
        .args(args)
        .env("CRW_HOME", &home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

/// A `PreToolUse` hook's input as Claude Code writes it.
fn hook_input(tool: &str, id: &str) -> String {
    format!(
        r#"{{"session_id":"s","cwd":"C:\\Data\\alpha","hook_event_name":"PreToolUse","tool_name":"{tool}","tool_input":{{"questions":[]}},"tool_use_id":"{id}"}}"#
    )
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_defer_hook_defers_and_the_answer_hook_answers_its_call() {
    let (code, out) = crw(
        &["hook", "--defer"],
        &hook_input("AskUserQuestion", "toolu_1"),
    );
    assert_eq!(code, 0);
    assert!(out.contains(r#""hookEventName":"PreToolUse""#), "{out}");
    assert!(out.contains(r#""permissionDecision":"defer""#), "{out}");
    assert_eq!(out.lines().count(), 1, "{out}");

    let scratch =
        Scratch(std::env::temp_dir().join(format!("crw-cli-{}-decisions", std::process::id())));
    fs::create_dir_all(&scratch.0).unwrap();
    let file = scratch.0.join("answer.json");
    fs::write(
        &file,
        r#"{"tool_use_id":"toolu_1","decision":"allow","updated_input":{"questions":[],"answers":{"Which?":"Top"}}}"#,
    )
    .unwrap();
    let path = file.to_string_lossy().into_owned();
    let answer = |tool: &str, id: &str| crw(&["hook", "--answer", &path], &hook_input(tool, id));

    let (code, out) = answer("AskUserQuestion", "toolu_1");
    assert_eq!(code, 0);
    assert!(out.contains(r#""permissionDecision":"allow""#), "{out}");
    assert!(out.contains(r#""answers":{"Which?":"Top"}"#), "{out}");
    // Another question ends the run again; another tool is left to the usual flow.
    let (_, out) = answer("AskUserQuestion", "toolu_2");
    assert!(out.contains(r#""permissionDecision":"defer""#), "{out}");
    let (code, out) = answer("Bash", "toolu_3");
    assert_eq!((code, out.trim()), (0, ""));
    // Input that does not read never makes the hook fail.
    let (code, _) = crw(&["hook", "--answer", &path], "not json");
    assert_eq!(code, 0);
}
