//! The signals (D-20, D-23, D-28; EP-12): what a Claude Code session tells the tool through the
//! hooks and the status-line command installed in the owner's user settings ([`install`]). `crw
//! hook` and `crw statusline` read the session's input into the narrow forms below (which session,
//! in which folder, which event; the model, cost, context and rate limits) and post them to the
//! window's loopback server ([`relay`]) within half a second. A transcript's text (a Stop hook's
//! last message, a notification's words) is never read into them, so it never leaves the session.
//! With no window listening, or on any error, the relays print nothing and exit 0: a session never
//! waits on the tool or shows its errors. The forms are the wire format too, so the server reads
//! them back as they were sent, and a field a later build adds reads as absent in an older one.
//! A prompt's text is read by the relay alone, for its `@<project>` [`mentions`] (EP-20b), and
//! never posted.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::inbox;
use crate::registry::Store;
use crate::sessions::{self, Registered};

pub mod install;
pub mod mentions;
pub mod relay;

/// The window's port when the registry names none (D-28: on 127.0.0.1 only).
pub const DEFAULT_PORT: u16 = 47831;

/// A relay's whole budget: connecting, writing and reading the window's answer (risk 4).
pub const BUDGET: Duration = Duration::from_millis(500);

pub const HOOK_ROUTE: &str = "/hook";
pub const STATUS_ROUTE: &str = "/statusline";
pub const HEALTH_ROUTE: &str = "/health";
/// `crw install`'s request that the window restart into the installed build (EP-37).
pub const RESTART_ROUTE: &str = "/restart";

/// How long `crw install` waits for the window's answer to its request: the window answers at
/// its next pass, which a hidden window runs when woken.
pub const RESTART_BUDGET: Duration = Duration::from_secs(8);

/// A restart request, who sends it, in words, and the install folder it wrote, which the window
/// compares with its own: a trial's install (`CRW_HOME\bin`) never restarts the owner's window
/// (EP-37).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestartAsk {
    pub from: String,
    #[serde(default)]
    pub bin: Option<PathBuf>,
}

/// The window's answer to a restart request: whether it restarts, and why or why not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestartAnswer {
    pub restarting: bool,
    pub words: String,
}

/// What came of a restart request (EP-37).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartReply {
    /// The window answered.
    Answered(RestartAnswer),
    /// No window listens on the port.
    NoWindow,
    /// The window has no restart route: a build from before EP-37.
    NoRoute,
    /// Something else, in words.
    Failed(String),
}

/// Asks the window on `port` to restart into the installed build, saying the request comes
/// `from` (`crw install`) and installed into `bin`: first whether a window listens at all, then
/// the request itself, whose answer the window gives at its next pass.
pub fn ask_restart(port: u16, from: &str, bin: Option<&Path>) -> RestartReply {
    let health = relay::request(port, "GET", HEALTH_ROUTE, None, Duration::from_millis(500));
    if !health.is_ok_and(|answer| answer.status == 200) {
        return RestartReply::NoWindow;
    }
    let body = match serde_json::to_vec(&RestartAsk {
        from: from.to_owned(),
        bin: bin.map(Path::to_path_buf),
    }) {
        Ok(body) => body,
        Err(e) => return RestartReply::Failed(e.to_string()),
    };
    match relay::request(port, "POST", RESTART_ROUTE, Some(&body), RESTART_BUDGET) {
        Ok(answer) if answer.status == 200 => {
            match serde_json::from_slice::<RestartAnswer>(&answer.body) {
                Ok(answer) => RestartReply::Answered(answer),
                Err(e) => RestartReply::Failed(format!("an answer that did not read: {e}")),
            }
        }
        Ok(answer) if answer.status == 404 || answer.status == 405 => RestartReply::NoRoute,
        Ok(answer) if answer.status == 504 => {
            RestartReply::Failed("the window did not answer in time".to_owned())
        }
        Ok(answer) => RestartReply::Failed(format!("the window answered {}", answer.status)),
        Err(e) => RestartReply::Failed(e),
    }
}

/// The variable a session's children inherit naming its surface: `claude-vscode`, `cli`,
/// `claude-desktop`, or an SDK's (`sdk-…`) for a `-p` run.
pub const ENTRYPOINT_VAR: &str = "CLAUDE_CODE_ENTRYPOINT";

/// The most of a session's input a relay reads (a Stop hook's carries the turn's last message).
const INPUT_CAP: u64 = 8 << 20;

/// The longest inbox text a session start is given, as the stopgap hook cut it (IN-2).
const INBOX_CAP: usize = 6000;

/// A hook's input as the tool keeps it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hook {
    /// `SessionStart`, `UserPromptSubmit`, `Notification`, `Stop` or `SessionEnd`, as Claude Code
    /// names the event.
    #[serde(rename = "hook_event_name")]
    pub event: String,
    pub session_id: String,
    /// The session's working folder.
    pub cwd: PathBuf,
    /// A start's `startup`, `resume`, `clear`, `compact` or `fork`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// An end's `clear`, `resume`, `logout`, `prompt_input_exit` or `other`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// A notification's `permission_prompt`, `idle_prompt`, `agent_needs_input`, …
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_type: Option<String>,
    /// The session's surface, from the relay's environment rather than Claude Code's input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
}

/// A status line's input as the tool keeps it (D-23): the documented fields the meter (EP-17) and
/// the sessions tile (EP-13) read.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StatusLine {
    pub session_id: String,
    pub cwd: PathBuf,
    pub workspace: Workspace,
    pub model: Model,
    pub cost: Cost,
    pub context_window: ContextWindow,
    /// Pro, Max and Ultra plans only, after the session's first response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<RateLimits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<Effort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The name given with `--name` or `/rename`, else the session's generated title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_name: Option<String>,
    /// The session's surface, from the relay's environment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    pub current_dir: PathBuf,
    /// The folder the session was launched in.
    pub project_dir: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Model {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Cost {
    /// Claude Code's own estimate at list price; a subscription is not billed in dollars (D-23).
    pub total_cost_usd: Option<f64>,
    pub total_duration_ms: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ContextWindow {
    pub used_percentage: Option<f64>,
    pub context_window_size: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RateLimits {
    pub five_hour: Option<Window>,
    pub seven_day: Option<Window>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Window {
    /// From 0 to 100.
    pub used_percentage: Option<f64>,
    /// Unix seconds.
    pub resets_at: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Effort {
    pub level: Option<String>,
}

impl StatusLine {
    /// The session's folder: where it was launched, else where it works now.
    pub fn folder(&self) -> &Path {
        [
            &self.workspace.project_dir,
            &self.workspace.current_dir,
            &self.cwd,
        ]
        .into_iter()
        .find(|path| !path.as_os_str().is_empty())
        .map_or(Path::new(""), PathBuf::as_path)
    }
}

/// What the window's server hears.
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Hook(Hook),
    StatusLine(Box<StatusLine>),
}

/// The port the window listens on: the registry's `port` (`CRW_HOME`'s registry in a trial),
/// else [`DEFAULT_PORT`]; a registry that does not read changes nothing for the relays.
pub fn port() -> u16 {
    Store::locate()
        .and_then(|store| store.load().ok())
        .map_or(DEFAULT_PORT, |loaded| loaded.registry.port())
}

/// The registered projects with their folders, for a prompt's mentions (EP-20b): the registry's
/// (`CRW_HOME`'s in a trial); none when it does not read.
pub fn registered() -> Vec<Registered> {
    let Some(loaded) = Store::locate().and_then(|store| store.load().ok()) else {
        return Vec::new();
    };
    let root = loaded.registry.data_root();
    loaded
        .registry
        .projects
        .iter()
        .map(|project| Registered {
            name: project.name.clone(),
            folder: project.folder(&root),
            revert: project.revert,
        })
        .collect()
}

/// The session's surface from this process's environment, which a relay inherits.
pub fn entrypoint() -> Option<String> {
    std::env::var(ENTRYPOINT_VAR)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// Up to [`INPUT_CAP`] bytes of a session's input.
fn read_input(input: impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    input
        .take(INPUT_CAP)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("the input did not read: {e}"))?;
    Ok(bytes)
}

/// What `crw hook` does: reads the hook's input, keeps the tool's fields with the session's
/// `entrypoint`, writes a start's inbox text to `out` when `inbox` is asked (the stopgap hook's
/// job, IN-2) and a prompt's mentions of `projects` (asked for only when the prompt holds an `@`,
/// EP-20b), then posts the hook to the window on `port`. What went wrong is returned for the
/// tests; the relay itself drops it.
pub fn relay_hook(
    input: impl Read,
    entrypoint: Option<String>,
    inbox: bool,
    projects: impl FnOnce() -> Vec<Registered>,
    out: &mut impl Write,
    port: u16,
) -> Result<(), String> {
    let bytes = read_input(input)?;
    let mut hook: Hook =
        serde_json::from_slice(&bytes).map_err(|e| format!("the input is not a hook's: {e}"))?;
    hook.entrypoint = entrypoint;
    let start = inbox.then(|| inbox_context(&hook)).flatten();
    // Claude Code adds a start's or a prompt's standard output to the session's context.
    for text in start
        .into_iter()
        .chain(prompt_context(&hook, &bytes, projects))
    {
        writeln!(out, "{text}").map_err(|e| format!("the context did not write: {e}"))?;
        out.flush().ok();
    }
    let body = serde_json::to_vec(&hook).expect("a hook serialises");
    relay::request(port, "POST", HOOK_ROUTE, Some(&body), BUDGET).map(drop)
}

/// The text a prompt is given (IN-2; EP-20b): the context of the registered projects its `@`
/// mentions ([`mentions::context`]), read from the hook's `input`, whose `prompt` stays in the
/// relay; `None` for any other event, for an SDK's session (a `-p` run, whose prompt a program
/// wrote: the tool's own headless runs among them), and for a prompt without an `@`, which asks
/// no registry.
pub fn prompt_context(
    hook: &Hook,
    input: &[u8],
    projects: impl FnOnce() -> Vec<Registered>,
) -> Option<String> {
    #[derive(Deserialize)]
    struct Prompt {
        #[serde(default)]
        prompt: String,
    }
    if hook.event != "UserPromptSubmit" || !sessions::interactive(hook.entrypoint.as_deref()) {
        return None;
    }
    let prompt: Prompt = serde_json::from_slice(input).ok()?;
    if !prompt.prompt.contains('@') {
        return None;
    }
    mentions::context(&prompt.prompt, &hook.cwd, &projects())
}

/// What `crw statusline` does: reads the status line's input, keeps the tool's fields with the
/// session's `entrypoint` and posts them to the window on `port`; it prints nothing (D-23).
pub fn relay_status_line(
    input: impl Read,
    entrypoint: Option<String>,
    port: u16,
) -> Result<(), String> {
    let bytes = read_input(input)?;
    let mut line: StatusLine = serde_json::from_slice(&bytes)
        .map_err(|e| format!("the input is not a status line's: {e}"))?;
    line.entrypoint = entrypoint;
    let body = serde_json::to_vec(&line).expect("a status line serialises");
    relay::request(port, "POST", STATUS_ROUTE, Some(&body), BUDGET).map(drop)
}

/// The text a session start is given (IN-2's stopgap hook, in the tool): the open messages of the
/// session folder's `roadmap/inbox.md` as they are written, under the triage rule (D-13); `None`
/// for any other event or source than a startup or a resume, and for a folder with no open
/// message.
pub fn inbox_context(hook: &Hook) -> Option<String> {
    if hook.event != "SessionStart"
        || !matches!(hook.source.as_deref(), None | Some("startup" | "resume"))
        || hook.cwd.as_os_str().is_empty()
    {
        return None;
    }
    let text = fs::read_to_string(inbox::path(&hook.cwd)).ok()?;
    let open = inbox::open_as_written(&text);
    if open.is_empty() {
        return None;
    }
    let mut messages = open.join("\n");
    if messages.chars().count() > INBOX_CAP {
        messages = messages.chars().take(INBOX_CAP).collect();
        messages.push_str("\n  [truncated: read roadmap/inbox.md]");
    }
    Some(format!(
        "Open cross-project inbox messages in roadmap/inbox.md ({} open). Triage rule (crw \
         D-13): roll a message into this session if it fits the brief and is at most S-sized; \
         otherwise park it in a named later EP's brief notes or propose a new row, confirming \
         with the owner through AskUserQuestion; or decline with a reason. Write the \
         disposition on its line and tick the box.\n{messages}",
        open.len()
    ))
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use super::*;
    use crate::files::tests::Scratch;

    /// A Stop hook's input as Claude Code writes it, with the fields the tool leaves behind.
    const STOP: &str = r#"{
        "session_id": "0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b",
        "prompt_id": "p-1",
        "transcript_path": "C:\\Home\\.claude\\projects\\alpha\\0e1d2c3b.jsonl",
        "cwd": "C:\\Data\\alpha",
        "permission_mode": "acceptEdits",
        "effort": {"level": "xhigh"},
        "hook_event_name": "Stop",
        "stop_hook_active": false,
        "last_assistant_message": "The brief is written; the tests pass."
    }"#;

    #[test]
    fn a_hook_keeps_its_session_folder_and_event_and_never_the_text() {
        let hook: Hook = serde_json::from_str(STOP).unwrap();
        assert_eq!(hook.event, "Stop");
        assert_eq!(hook.session_id, "0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b");
        assert_eq!(hook.cwd, Path::new(r"C:\Data\alpha"));
        let sent = serde_json::to_string(&Hook {
            entrypoint: Some("claude-vscode".to_owned()),
            ..hook.clone()
        })
        .unwrap();
        assert!(!sent.contains("brief is written"), "{sent}");
        assert!(!sent.contains("transcript"), "{sent}");
        assert!(sent.contains(r#""entrypoint":"claude-vscode""#), "{sent}");
        // What the server reads back is what was sent.
        let back: Hook = serde_json::from_str(&sent).unwrap();
        assert_eq!(back.event, "Stop");
        assert_eq!(back.entrypoint.as_deref(), Some("claude-vscode"));
        // The other events' own fields, and an input with almost nothing in it.
        let start: Hook = serde_json::from_str(
            r#"{"hook_event_name":"SessionStart","session_id":"s","cwd":"C:\\Data\\beta","source":"resume","model":"claude-opus-5-5"}"#,
        )
        .unwrap();
        assert_eq!(start.source.as_deref(), Some("resume"));
        let note: Hook = serde_json::from_str(
            r#"{"hook_event_name":"Notification","session_id":"s","message":"Claude needs your permission to use Bash","notification_type":"permission_prompt"}"#,
        )
        .unwrap();
        assert_eq!(note.notification_type.as_deref(), Some("permission_prompt"));
        let end: Hook =
            serde_json::from_str(r#"{"hook_event_name":"SessionEnd","reason":"other"}"#).unwrap();
        assert_eq!(
            (end.reason.as_deref(), end.session_id.as_str()),
            (Some("other"), "")
        );
    }

    #[test]
    fn a_status_line_keeps_the_documented_figures() {
        let line: StatusLine = serde_json::from_str(
            r#"{
                "cwd": "C:\\Data\\alpha\\crates",
                "session_id": "abc123",
                "session_name": "alpha EP-3",
                "transcript_path": "C:\\Home\\.claude\\projects\\alpha\\abc123.jsonl",
                "model": {"id": "claude-opus-5-5", "display_name": "Opus"},
                "workspace": {"current_dir": "C:\\Data\\alpha\\crates", "project_dir": "C:\\Data\\alpha", "added_dirs": []},
                "version": "2.1.316",
                "cost": {"total_cost_usd": 0.0123, "total_duration_ms": 45000, "total_lines_added": 156},
                "context_window": {"total_input_tokens": 15500, "context_window_size": 200000, "used_percentage": 8, "current_usage": null},
                "exceeds_200k_tokens": false,
                "effort": {"level": "high"},
                "rate_limits": {"five_hour": {"used_percentage": 23.5, "resets_at": 1738425600}, "seven_day": {"used_percentage": 41.2, "resets_at": 1738857600}}
            }"#,
        )
        .unwrap();
        assert_eq!(line.folder(), Path::new(r"C:\Data\alpha"));
        assert_eq!(line.model.id, "claude-opus-5-5");
        assert_eq!(line.cost.total_cost_usd, Some(0.0123));
        assert_eq!(line.context_window.used_percentage, Some(8.0));
        let limits = line.rate_limits.as_ref().unwrap();
        assert_eq!(
            limits.seven_day.as_ref().unwrap().used_percentage,
            Some(41.2)
        );
        assert_eq!(line.effort.unwrap().level.as_deref(), Some("high"));
        // Early in a session: no rate limits, a context percentage of null.
        let early: StatusLine = serde_json::from_str(
            r#"{"session_id":"s","cwd":"C:\\Data\\beta","context_window":{"used_percentage":null}}"#,
        )
        .unwrap();
        assert_eq!(early.rate_limits, None);
        assert_eq!(early.context_window.used_percentage, None);
        assert_eq!(early.folder(), Path::new(r"C:\Data\beta"));
    }

    #[test]
    fn a_relay_posts_the_kept_fields_and_prints_nothing() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || relay::tests::answer_once(&listener, 204));
        let mut out = Vec::new();
        relay_hook(
            STOP.as_bytes(),
            Some("claude-vscode".to_owned()),
            true,
            || unreachable!("a Stop asks no registry"),
            &mut out,
            port,
        )
        .unwrap();
        let request = server.join().unwrap();
        assert!(request.starts_with("POST /hook HTTP/1.1\r\n"), "{request}");
        assert!(request.contains("Content-Type: application/json\r\n"));
        let body = request.split("\r\n\r\n").nth(1).unwrap();
        let hook: Hook = serde_json::from_str(body).unwrap();
        assert_eq!(hook.event, "Stop");
        assert_eq!(hook.entrypoint.as_deref(), Some("claude-vscode"));
        assert!(!body.contains("brief is written"));
        // A Stop is no start: nothing is printed, inbox or not.
        assert!(out.is_empty());

        // With no window listening, the relay gives up after the connection's share of its
        // budget, not the whole of it (a closed port is refused late on Windows).
        let closed = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = closed.local_addr().unwrap().port();
        drop(closed);
        let started = std::time::Instant::now();
        assert!(relay_status_line(r#"{"session_id":"s"}"#.as_bytes(), None, port).is_err());
        let took = started.elapsed();
        assert!(
            took < relay::CONNECT + Duration::from_millis(250),
            "{took:?}"
        );
        // An input that is not a hook's is not posted.
        assert!(relay_hook(&b"not json"[..], None, false, Vec::new, &mut out, port).is_err());
    }

    #[test]
    fn a_prompt_is_given_its_mentions_and_its_text_stays_in_the_relay() {
        let scratch = Scratch::new("signals-prompt");
        let beta = scratch.0.join("beta");
        fs::create_dir_all(&beta).unwrap();
        fs::write(beta.join("CLAUDE.md"), "# beta\n\n1. Beta's own rule.\n").unwrap();
        let projects = || {
            vec![Registered {
                name: "beta".to_owned(),
                folder: beta.clone(),
                revert: true,
            }]
        };
        let input = serde_json::json!({
            "session_id": "s1",
            "transcript_path": "C:\\Home\\.claude\\projects\\alpha\\s1.jsonl",
            "cwd": scratch.0.join("alpha"),
            "permission_mode": "acceptEdits",
            "hook_event_name": "UserPromptSubmit",
            "prompt": "What does @beta do next? A secret word: tangerine."
        })
        .to_string();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || relay::tests::answer_once(&listener, 204));
        let mut out = Vec::new();
        let vscode = Some("claude-vscode".to_owned());
        relay_hook(input.as_bytes(), vscode, false, projects, &mut out, port).unwrap();
        let printed = String::from_utf8(out).unwrap();
        assert!(
            printed.starts_with("The prompt mentions @beta"),
            "{printed}"
        );
        assert!(printed.contains("1. Beta's own rule."), "{printed}");
        assert!(printed.contains("beta has no next brief") || printed.contains("It has no "));
        // The window hears the event, never the prompt.
        let request = server.join().unwrap();
        let body = request.split("\r\n\r\n").nth(1).unwrap();
        assert!(!body.contains("tangerine"), "{body}");
        let hook: Hook = serde_json::from_str(body).unwrap();
        assert_eq!(hook.event, "UserPromptSubmit");

        let hook: Hook = serde_json::from_str(&input).unwrap();
        // An SDK's run, whose prompt a program wrote, is given nothing and asks no registry.
        let sdk = Hook {
            entrypoint: Some("sdk-cli".to_owned()),
            ..hook.clone()
        };
        let never = || -> Vec<Registered> { unreachable!("no registry for an SDK's run") };
        assert_eq!(prompt_context(&sdk, input.as_bytes(), never), None);
        // A prompt without an @ asks no registry either.
        let plain = input.replace("@beta", "beta");
        let never = || -> Vec<Registered> { unreachable!("no registry without an @") };
        assert_eq!(prompt_context(&hook, plain.as_bytes(), never), None);
        // Another event is no prompt, whatever its input holds.
        let stop = Hook {
            event: "Stop".to_owned(),
            ..hook.clone()
        };
        assert_eq!(prompt_context(&stop, input.as_bytes(), projects), None);
        // A session that does not say its surface is taken for an interactive one.
        assert!(prompt_context(&hook, input.as_bytes(), projects).is_some());
    }

    #[test]
    fn a_start_is_given_the_open_inbox_messages_as_written() {
        let scratch = Scratch::new("signals-inbox");
        let folder = scratch.0.join("alpha");
        fs::create_dir_all(folder.join("roadmap")).unwrap();
        fs::write(
            inbox::path(&folder),
            "# Inbox\n\nThe header.\n\n- [x] IN-1 (from beta EP-2, 2026-10-30) Done already. **Disposition (2026-10-31):** rolled in.\n- [ ] IN-2 (from beta EP-3, 2026-11-01) A message that\n  wraps onto a second line.\n- [ ] IN-3 (from the owner, 2026-11-02) Another. [urgent]\n",
        )
        .unwrap();
        let start = |source: Option<&str>| Hook {
            event: "SessionStart".to_owned(),
            session_id: "s".to_owned(),
            cwd: folder.clone(),
            source: source.map(str::to_owned),
            ..Hook::default()
        };
        let text = inbox_context(&start(Some("startup"))).unwrap();
        assert!(
            text.starts_with("Open cross-project inbox messages in roadmap/inbox.md (2 open).")
        );
        assert!(text.ends_with(
            "- [ ] IN-2 (from beta EP-3, 2026-11-01) A message that\n  wraps onto a second line.\n- [ ] IN-3 (from the owner, 2026-11-02) Another. [urgent]"
        ), "{text}");
        assert!(!text.contains("IN-1"));
        assert!(inbox_context(&start(Some("resume"))).is_some());
        assert!(inbox_context(&start(None)).is_some());
        // After a compaction or a clear the session has read them already; another event never.
        assert_eq!(inbox_context(&start(Some("compact"))), None);
        assert_eq!(inbox_context(&start(Some("clear"))), None);
        assert_eq!(
            inbox_context(&Hook {
                event: "Stop".to_owned(),
                ..start(Some("startup"))
            }),
            None
        );
        // A folder without an inbox, or without an open message, is given nothing.
        let quiet = Hook {
            cwd: scratch.0.join("beta"),
            ..start(Some("startup"))
        };
        assert_eq!(inbox_context(&quiet), None);
        // The relay prints the text even when no window listens.
        let mut out = Vec::new();
        let input = serde_json::to_vec(&start(Some("startup"))).unwrap();
        let closed = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = closed.local_addr().unwrap().port();
        drop(closed);
        assert!(relay_hook(&input[..], None, true, Vec::new, &mut out, port).is_err());
        assert!(String::from_utf8(out).unwrap().contains("IN-3"));
    }
}
