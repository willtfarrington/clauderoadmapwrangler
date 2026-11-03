//! The session runner against `stub-claude` (EP-9, D-29): every session here is the stub playing
//! a script of synthetic stream-json from `tests/streams/`, with fake names; the real `claude` CLI
//! is never started and no usage is spent. Session starts are two seconds apart here as anywhere.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

use crw_core::ledger::Ledger;
use crw_core::session::{
    Ended, Event, Launch, Outcome, Preflight, Request, Response, Session, edit_deny_rule,
};
use serde_json::Value;

const STUB: &str = env!("CARGO_BIN_EXE_stub-claude");

/// How long a test waits for the next event before it fails.
const WAIT: Duration = Duration::from_secs(20);

/// The stub's preflight, asked once for every test.
fn preflight() -> &'static Preflight {
    static PREFLIGHT: OnceLock<Preflight> = OnceLock::new();
    PREFLIGHT.get_or_init(|| Preflight::run(Path::new(STUB)))
}

/// A scratch folder holding the project `alpha` with the stub's script in it, removed when
/// dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str, script: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("crw-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("alpha")).unwrap();
        let streams = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/streams");
        fs::copy(
            streams.join(format!("{script}.jsonl")),
            dir.join("alpha/stub-script.jsonl"),
        )
        .unwrap();
        Self(dir)
    }

    fn project(&self) -> PathBuf {
        self.0.join("alpha")
    }

    /// What the stub recorded: its arguments and environment first, then each line it read.
    fn record(&self) -> Vec<Value> {
        fs::read_to_string(self.project().join("stub-record.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    /// The lines the stub read from stdin, parsed.
    fn read(&self) -> Vec<Value> {
        self.record()
            .iter()
            .filter_map(|entry| entry["read"].as_str())
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn heartbeat(&self) -> PathBuf {
        self.project().join("heartbeat.txt")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn next(session: &Session) -> Event {
    session
        .events()
        .recv_timeout(WAIT)
        .expect("an event within the wait")
}

/// The events up to the first `wanted` accepts, that one included.
fn until(session: &Session, wanted: impl Fn(&Event) -> bool) -> Vec<Event> {
    let mut events = Vec::new();
    loop {
        let event = next(session);
        let done = wanted(&event);
        events.push(event);
        if done {
            return events;
        }
    }
}

/// Every event to the end, answering each request with what `answer` gives.
fn to_the_end(session: &Session, answer: impl Fn(&Request) -> Option<Response>) -> Vec<Event> {
    let mut events = Vec::new();
    loop {
        let event = next(session);
        if let Event::Request(request) = &event
            && let Some(response) = answer(request)
        {
            assert!(session.respond(&request.id, response));
        }
        let done = matches!(event, Event::Exited(_));
        events.push(event);
        if done {
            return events;
        }
    }
}

fn ended(events: &[Event]) -> &Ended {
    match events.last() {
        Some(Event::Exited(ended)) => ended,
        other => panic!("the last event is not the exit: {other:?}"),
    }
}

/// Whether `file` changes within `within`: a heartbeat that beats.
fn beating(file: &Path, within: Duration) -> bool {
    let deadline = Instant::now() + within;
    let mut last = fs::read_to_string(file).ok();
    while Instant::now() < deadline {
        thread::sleep(Duration::from_millis(200));
        let now = fs::read_to_string(file).ok();
        if now.is_some() && last.is_some() && now != last {
            return true;
        }
        last = now;
    }
    false
}

/// The value after each occurrence of `flag` in the recorded arguments.
fn after(args: &Value, flag: &str) -> Vec<String> {
    let args: Vec<&str> = args
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect();
    args.windows(2)
        .filter(|pair| pair[0] == flag)
        .map(|pair| pair[1].to_owned())
        .collect()
}

#[test]
fn the_preflight_reads_the_stub() {
    let preflight = preflight();
    assert!(preflight.found);
    assert_eq!(preflight.version.as_ref().unwrap().to_string(), "2.1.316");
    let auth = preflight.auth.as_ref().unwrap();
    assert!(auth.logged_in);
    let flags = preflight.flags.as_ref().unwrap();
    assert!(flags.has("--permission-prompts") && flags.has("--settings"));
    assert_eq!(flags.efforts, ["low", "medium", "high", "xhigh", "max"]);
    assert!(
        preflight.refusals().is_empty(),
        "{:?}",
        preflight.refusals()
    );
    // A launch is refused before anything starts when the CLI is missing.
    let missing = Preflight::run(Path::new(r"C:\nowhere\claude.exe"));
    let refused = Session::start(
        &missing,
        Launch::new("alpha", Path::new("."), "opus", "Go."),
    );
    assert!(refused.err().unwrap().contains("no Claude Code CLI"));
}

#[test]
fn a_whole_run_is_read_answered_and_recorded() {
    let scratch = Scratch::new("whole-run", "whole-run");
    let beta = scratch.0.join("beta");
    fs::create_dir_all(&beta).unwrap();
    let ledger = scratch.0.join("ledger.sqlite");
    let mut launch = Launch::new(
        "alpha",
        &scratch.project(),
        "claude-fable-5-1",
        "Run EP-3 of alpha.",
    );
    launch.ep = Some("EP-3".to_owned());
    launch.effort = Some("xhigh".to_owned());
    launch.neighbours = vec![beta.clone()];
    launch.ledger = Some(ledger.clone());
    let session = Session::start(preflight(), launch).unwrap();
    assert!(session.dropped.is_empty(), "{:?}", session.dropped);
    assert!(session.ledger_row.is_some());

    let events = to_the_end(&session, |request| {
        Some(if request.questions.is_empty() {
            Response::Deny("not now".to_owned())
        } else {
            Response::Answers(vec!["Blue".to_owned()])
        })
    });
    assert_eq!(events[0], Event::Handshake(Ok(())));
    assert!(
        !events.iter().any(|e| matches!(e, Event::Unreadable(_))),
        "{events:?}"
    );
    let Some(Event::Init(init)) = events.iter().find(|e| matches!(e, Event::Init(_))) else {
        panic!("{events:?}")
    };
    assert_eq!(init.version.as_deref(), Some("2.1.316"));
    assert_eq!(init.session_id, session.session_id);
    assert_eq!(init.capabilities, ["alpha_capability", "beta_capability"]);
    let streamed: String = events
        .iter()
        .filter_map(|e| match e {
            Event::TextDelta(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(streamed, "Reading the brief.");
    assert!(events.contains(&Event::Text("Reading the brief.".to_owned())));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ToolUse(t) if t.summary == "Edit src/lib.rs"))
    );
    // The tool's result comes back for the transcript, collapsed (EP-13).
    assert!(events.iter().any(|e| matches!(e, Event::ToolResult(r)
        if r.tool_use_id == "toolu_01" && r.lines == 2 && !r.is_error
            && r.excerpt.starts_with("The file src/lib.rs has been updated."))));
    let requests: Vec<&Request> = events
        .iter()
        .filter_map(|e| match e {
            Event::Request(request) => Some(request),
            _ => None,
        })
        .collect();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].questions[0].header, "Colour");
    assert_eq!(requests[1].summary, "Bash git push");
    assert_eq!(requests[1].reason.as_deref(), Some("Not in the allow list"));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Denied(d) if d.tool == "Edit"))
    );
    let ended = ended(&events);
    assert_eq!(ended.outcome, Outcome::Completed);
    assert_eq!(ended.exit_code, Some(0));
    assert_eq!(ended.result.as_ref().unwrap().cost_usd, 0.42);

    // The command line and the environment the stub saw.
    let record = scratch.record();
    let args = &record[0]["args"];
    assert_eq!(after(args, "--session-id"), [session.session_id.as_str()]);
    assert_eq!(after(args, "--name"), ["alpha EP-3"]);
    assert_eq!(after(args, "--model"), ["claude-fable-5-1"]);
    assert_eq!(after(args, "--effort"), ["xhigh"]);
    assert_eq!(after(args, "--permission-mode"), ["acceptEdits"]);
    assert_eq!(after(args, "--permission-prompts"), ["host"]);
    assert_eq!(after(args, "--add-dir"), [beta.display().to_string()]);
    let settings: Value = serde_json::from_str(&after(args, "--settings")[0]).unwrap();
    assert_eq!(
        settings["permissions"]["deny"][0].as_str(),
        edit_deny_rule(&beta).as_deref()
    );
    let env = &record[0]["env"];
    assert_eq!(env["CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD"], "1");
    for withheld in ["ANTHROPIC_API_KEY", "CLAUDECODE", "CLAUDE_CODE_SESSION_ID"] {
        assert!(env[withheld].is_null(), "{withheld}");
    }
    assert!(Path::new(record[0]["cwd"].as_str().unwrap()).ends_with("alpha"));

    // What the runner wrote: the handshake, the prompt, the two answers.
    let read = scratch.read();
    assert_eq!(read.len(), 4, "{read:?}");
    assert_eq!(read[0]["request"]["subtype"], "initialize");
    assert_eq!(
        read[1]["message"]["content"][0]["text"],
        "Run EP-3 of alpha."
    );
    let answered = &read[2]["response"];
    assert_eq!(answered["request_id"], "req-1");
    assert_eq!(
        answered["response"]["updatedInput"]["answers"]["Which colour should the badge take?"],
        "Blue"
    );
    assert_eq!(read[3]["response"]["request_id"], "req-2");
    assert_eq!(read[3]["response"]["response"]["behavior"], "deny");
    assert_eq!(read[3]["response"]["response"]["message"], "not now");

    // The ledger's row.
    let rows = Ledger::open(&ledger).unwrap().sessions().unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(Some(row.id), session.ledger_row);
    assert_eq!(row.session_id.as_deref(), Some(session.session_id.as_str()));
    assert_eq!(
        (row.project.as_str(), row.ep.as_deref(), row.mode.as_str()),
        ("alpha", Some("EP-3"), "headless")
    );
    assert_eq!(
        (row.model.as_deref(), row.effort.as_deref(), row.ultracode),
        (Some("claude-fable-5-1"), Some("xhigh"), false)
    );
    assert_eq!(row.outcome.as_deref(), Some("completed"));
    assert_eq!((row.cost_usd, row.questions), (Some(0.42), 1));
    assert_eq!(row.result.as_deref(), Some("EP-3 is done."));
    assert!(row.ended_at.is_some());
}

/// Resume (EP-13): a follow-up is a run of its own under the session's id, in the ledger as a row
/// of its own.
#[test]
fn a_resume_goes_on_under_the_sessions_id() {
    let scratch = Scratch::new("resume", "prompt-only");
    let ledger = scratch.0.join("ledger.sqlite");
    let mut launch = Launch::new("alpha", &scratch.project(), "opus", "Run EP-3 of alpha.");
    launch.ep = Some("EP-3".to_owned());
    launch.ledger = Some(ledger.clone());
    let first = Session::start(preflight(), launch.clone()).unwrap();
    assert_eq!(
        ended(&to_the_end(&first, |_| None)).outcome,
        Outcome::Completed
    );

    launch.resume = Some(first.session_id.clone());
    launch.prompt = "Go on from the tests.".to_owned();
    let second = Session::start(preflight(), launch).unwrap();
    assert_eq!(second.session_id, first.session_id);
    assert_eq!(
        ended(&to_the_end(&second, |_| None)).outcome,
        Outcome::Completed
    );

    // The second run's command line names the session it resumes and asks for no new one.
    let runs: Vec<Value> = scratch
        .record()
        .into_iter()
        .filter(|entry| entry.get("args").is_some())
        .collect();
    assert_eq!(runs.len(), 2);
    assert_eq!(
        after(&runs[0]["args"], "--session-id"),
        [first.session_id.as_str()]
    );
    assert_eq!(
        after(&runs[1]["args"], "--resume"),
        [first.session_id.as_str()]
    );
    assert!(after(&runs[1]["args"], "--session-id").is_empty());
    assert!(
        scratch
            .read()
            .iter()
            .any(|line| holds(line, "Go on from the tests."))
    );
    let rows = Ledger::open(&ledger).unwrap().sessions().unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(
        |row| row.session_id.as_deref() == Some(first.session_id.as_str())
            && row.outcome.as_deref() == Some("completed")
            && row.ep.as_deref() == Some("EP-3")
    ));
}

#[test]
fn failures_keep_the_exit_code_the_result_and_stderr() {
    // The CLI refuses its command line before it says anything.
    let scratch = Scratch::new("failure-start", "failure-start");
    let launch = Launch::new("alpha", &scratch.project(), "opus", "Go.");
    let session = Session::start(preflight(), launch).unwrap();
    let events = to_the_end(&session, |_| None);
    let failed = ended(&events);
    assert_eq!(failed.outcome, Outcome::Failed);
    assert_eq!(failed.exit_code, Some(1));
    assert!(failed.result.is_none());
    assert!(
        failed
            .stderr
            .iter()
            .any(|line| line.contains("unknown option"))
    );
    assert!(!events.iter().any(|e| matches!(e, Event::Init(_))));

    // An error result and a non-zero exit: the result is read all the same.
    let scratch = Scratch::new("error-result", "error-result");
    let launch = Launch::new("alpha", &scratch.project(), "opus", "Go.");
    let session = Session::start(preflight(), launch).unwrap();
    let events = to_the_end(&session, |_| None);
    let failed = ended(&events);
    assert_eq!(failed.outcome, Outcome::Failed);
    assert_eq!(failed.exit_code, Some(1));
    let result = failed.result.as_ref().unwrap();
    assert_eq!(result.subtype, "error_during_execution");
    assert_eq!(result.errors, ["The run could not continue."]);
    assert_eq!(failed.stderr, ["the run failed"]);
}

#[test]
fn a_graceful_stop_interrupts_and_closes_stdin() {
    let scratch = Scratch::new("stop", "stop");
    let launch = Launch::new("alpha", &scratch.project(), "opus", "Go.");
    let session = Session::start(preflight(), launch).unwrap();
    until(&session, |e| matches!(e, Event::Init(_)));
    assert!(session.stop());
    let events = to_the_end(&session, |_| None);
    let stopped = ended(&events);
    assert_eq!(stopped.outcome, Outcome::Stopped);
    assert_eq!(stopped.exit_code, Some(0));
    let read = scratch.read();
    assert!(
        read.iter()
            .any(|line| line["request"]["subtype"] == "interrupt"),
        "{read:?}"
    );
}

#[test]
fn a_stop_that_runs_out_of_time_kills_the_whole_tree() {
    let scratch = Scratch::new("stop-timeout", "hang");
    let mut launch = Launch::new("alpha", &scratch.project(), "opus", "Go.");
    launch.stop_timeout = Duration::from_secs(1);
    let session = Session::start(preflight(), launch).unwrap();
    until(&session, |e| matches!(e, Event::Init(_)));
    assert!(beating(&scratch.heartbeat(), Duration::from_secs(5)));
    let asked = Instant::now();
    assert!(session.stop());
    let events = to_the_end(&session, |_| None);
    let waited = asked.elapsed();
    assert!(
        waited >= Duration::from_secs(1) && waited < Duration::from_secs(10),
        "{waited:?}"
    );
    let stopped = ended(&events);
    assert_eq!(stopped.outcome, Outcome::Stopped);
    // TerminateJobObject's exit code.
    assert_eq!(stopped.exit_code, Some(1));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Notice(text) if text.contains("killed")))
    );
    assert!(!beating(&scratch.heartbeat(), Duration::from_secs(1)));
}

#[test]
fn dropping_the_handle_kills_the_session() {
    let scratch = Scratch::new("dropped", "hang");
    let launch = Launch::new("alpha", &scratch.project(), "opus", "Go.");
    let session = Session::start(preflight(), launch).unwrap();
    until(&session, |e| matches!(e, Event::Init(_)));
    assert!(beating(&scratch.heartbeat(), Duration::from_secs(5)));
    drop(session);
    // The job dies with its handle: within a few seconds the heartbeat is still.
    let deadline = Instant::now() + Duration::from_secs(10);
    while beating(&scratch.heartbeat(), Duration::from_secs(1)) {
        assert!(
            Instant::now() < deadline,
            "the grandchild outlived the handle"
        );
    }
}

#[test]
fn odd_lines_and_a_refused_handshake_do_not_stop_a_run() {
    let scratch = Scratch::new("odd", "odd");
    let launch = Launch::new("alpha", &scratch.project(), "opus", "Go.");
    let session = Session::start(preflight(), launch).unwrap();
    let events = to_the_end(&session, |_| None);
    assert_eq!(
        events[0],
        Event::Handshake(Err("initialize is not supported here".to_owned()))
    );
    let unreadable = events
        .iter()
        .filter(|e| matches!(e, Event::Unreadable(_)))
        .count();
    assert_eq!(unreadable, 2, "{events:?}");
    assert!(events.iter().any(|e| matches!(e, Event::Init(_))));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Notice(text) if text.contains("hook_callback")))
    );
    let deferred = ended(&events);
    assert_eq!(deferred.outcome, Outcome::Deferred);
    assert_eq!(deferred.exit_code, Some(0));
    // The hook request was refused, so the stub went on.
    let read = scratch.read();
    assert!(
        read.iter()
            .any(|line| line["response"]["request_id"] == "req-9"
                && line["response"]["subtype"] == "error")
    );
}

/// Whether `value` holds the string `text` anywhere inside it.
fn holds(value: &Value, text: &str) -> bool {
    match value {
        Value::String(s) => s == text,
        Value::Array(items) => items.iter().any(|item| holds(item, text)),
        Value::Object(map) => map.values().any(|item| holds(item, text)),
        _ => false,
    }
}

/// The way the launch dialog launches (EP-10): a project's draft, its resolution in the launch,
/// its generated prompt sent whole as the session's first message, the run in the ledger.
#[test]
fn a_drafted_prompt_reaches_the_session_whole() {
    use crw_core::registry::{Project, Registry, State};
    use crw_core::{prompt, roadmap::snapshot};

    let house = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/projects/house");
    let registry = Registry {
        format: 1,
        ignore: Vec::new(),
        settings: Default::default(),
        rules: Vec::new(),
        projects: vec![Project::new("house", Path::new("house"), State::Active)],
    };
    let draft = prompt::draft(
        &registry,
        &registry.projects[0],
        &snapshot::read_tree(&house),
        Some("# house\n\n1. **Read first:** the roadmap — then the inbox…\n"),
        &preflight().flags.as_ref().unwrap().efforts,
        "2026-11-02",
    )
    .unwrap();
    assert!(draft.prompt.text.contains("`roadmap/EP-3-parser.md`"));
    assert!(draft.prompt.text.contains("the roadmap — then the inbox…"));

    let scratch = Scratch::new("drafted", "prompt-only");
    let ledger = scratch.0.join("ledger.sqlite");
    let resolution = &draft.resolution;
    let mut launch = Launch::new(
        "house",
        &scratch.project(),
        &resolution.model.value,
        &draft.prompt.text,
    );
    launch.ep = Some(draft.ep.to_string());
    launch.effort = resolution.effort.value.clone();
    launch.permission_mode = resolution.permission_mode.value.clone();
    launch.ledger = Some(ledger.clone());
    let session = Session::start(preflight(), launch).unwrap();
    let events = to_the_end(&session, |_| None);
    assert_eq!(ended(&events).outcome, Outcome::Completed);

    let read = scratch.read();
    assert!(
        read.iter().any(|line| holds(line, &draft.prompt.text)),
        "the prompt did not arrive whole: {read:?}"
    );
    let record = scratch.record();
    let args = &record[0]["args"];
    assert_eq!(after(args, "--model"), ["fable"]);
    assert_eq!(after(args, "--effort"), ["xhigh"]);
    assert_eq!(after(args, "--name"), ["house EP-3"]);
    let rows = Ledger::open(&ledger).unwrap().sessions().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].ep.as_deref(), Some("EP-3"));
    assert_eq!(rows[0].outcome.as_deref(), Some("completed"));
}
