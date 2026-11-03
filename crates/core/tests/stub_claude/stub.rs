//! `stub-claude`: the Claude Code CLI as the session runner's tests see it (EP-9, D-29). It is
//! never the real CLI and spends nothing. Asked `--version`, `auth status` or `--help`, it answers
//! with synthetic text. Otherwise it plays `stub-script.jsonl` from its working folder: lines of
//! stream-json with fake names, printed as they stand (`$SESSION` becomes the `--session-id` it
//! was given, or the id it `--resume`s), between directives that stand for the CLI's side of the
//! conversation:
//!
//! - `{"stub": "handshake"}` reads up to the host's `initialize` and answers it, with an error
//!   when the directive carries `"error": "<text>"`;
//! - `{"stub": "prompt"}` reads up to the prompt, echoed back with `--replay-user-messages`;
//! - `{"stub": "answer", "request_id": "<id>"}` reads up to the host's answer to that request;
//! - `{"stub": "eof"}` reads until stdin closes;
//! - `{"stub": "hang"}` waits for ever;
//! - `{"stub": "exit", "code": <n>}` exits;
//! - `{"stub": "stderr" | "raw", "text": "<line>"}` writes a line to stderr, or one to stdout that
//!   need not be JSON;
//! - `{"stub": "grandchild"}` starts a process of its own that beats `heartbeat.txt` until it is
//!   killed, so the tests can see a job take the whole tree.
//!
//! It records its arguments, the environment variables the tests look for, the answer file a
//! resume's `PreToolUse` hook would read (its path and what it holds, when `--settings` names one;
//! EP-14) and every line it reads in `stub-record.jsonl`, one JSON object per line. It runs no
//! hook.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Lines, StdinLock, Write};
use std::path::Path;
use std::process::{self, Child, Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

/// A help in the shape `claude --help` prints into a pipe, listing what the runner looks for.
const HELP: &str = "\
Usage: claude [options] [command] [prompt]

A stand-in for the tests.

Options:
  --add-dir <directories...>            Additional directories to allow tool
                                        access to
  --effort <level>                      Effort level for the current session
                                        (low, medium, high, xhigh, max)
  --include-partial-messages            Include partial message chunks
  --input-format <format>               Input format (choices: \"text\",
                                        \"stream-json\")
  --max-budget-usd <amount>             Maximum dollar amount to spend
  --model <model>                       Model for the current session
  -n, --name <name>                     Set a display name for this session
  --output-format <format>              Output format
  --permission-mode <mode>              Permission mode to use for the session
                                        (choices: \"acceptEdits\", \"auto\",
                                        \"bypassPermissions\", \"manual\",
                                        \"dontAsk\", \"plan\")
  --permission-prompts <target>         Who answers permission prompts with
                                        --print: \"host\" (the SDK host or
                                        --permission-prompt-tool) or \"none\"
                                        (nobody: anything that would prompt is
                                        denied automatically)
  -p, --print                           Print response and exit
  --replay-user-messages                Re-emit user messages from stdin
  -r, --resume [value]                  Resume a conversation by session ID or
                                        name
  --session-id <uuid>                   Use a specific session ID
  --settings <file-or-json>             Path to a settings JSON file or a JSON
                                        string
  --verbose                             Override verbose mode setting

Commands:
  auth                                  Manage authentication
";

/// The variables the tests look for in a session's environment.
const WATCHED: [&str; 4] = [
    "CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD",
    "ANTHROPIC_API_KEY",
    "CLAUDECODE",
    "CLAUDE_CODE_SESSION_ID",
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["--version"] => println!("2.1.316 (Claude Code)"),
        ["auth", "status"] => {
            println!(
                r#"{{"loggedIn": true, "authMethod": "claude.ai", "subscriptionType": "pro"}}"#
            );
        }
        ["--help"] => print!("{HELP}"),
        ["--heartbeat", file] => heartbeat(Path::new(file)),
        _ => play(&args),
    }
}

/// Writes the time to `file` ten times a second, for as long as it lives.
fn heartbeat(file: &Path) {
    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let _ = fs::write(file, now.to_string());
        thread::sleep(Duration::from_millis(100));
    }
}

/// Stdin, read line by line, each line recorded.
struct Input {
    lines: Lines<StdinLock<'static>>,
    record: File,
}

impl Input {
    /// The next line, recorded; `None` when stdin has closed.
    fn next(&mut self) -> Option<String> {
        let line = self.lines.next()?.ok()?;
        let _ = writeln!(self.record, "{}", json!({ "read": line }));
        Some(line)
    }

    /// Reads up to the first line `wanted` accepts and returns it parsed; exits when stdin closes
    /// first, as the CLI does.
    fn until(&mut self, what: &str, wanted: impl Fn(&Value) -> bool) -> Value {
        loop {
            let Some(line) = self.next() else {
                eprintln!("stub-claude: stdin closed while waiting for {what}");
                process::exit(9);
            };
            if let Ok(value) = serde_json::from_str::<Value>(&line)
                && wanted(&value)
            {
                return value;
            }
        }
    }
}

fn play(args: &[String]) {
    let session = args
        .windows(2)
        .find(|pair| pair[0] == "--session-id" || pair[0] == "--resume")
        .map(|pair| pair[1].clone())
        .unwrap_or_default();
    let script =
        fs::read_to_string("stub-script.jsonl").expect("stub-script.jsonl in the working folder");
    let mut record = OpenOptions::new()
        .create(true)
        .append(true)
        .open("stub-record.jsonl")
        .expect("stub-record.jsonl can be written");
    let env: Map<String, Value> = WATCHED
        .iter()
        .map(|name| {
            let value = std::env::var(name).map_or(Value::Null, Value::String);
            ((*name).to_owned(), value)
        })
        .collect();
    let cwd = std::env::current_dir()
        .map(|dir| dir.display().to_string())
        .unwrap_or_default();
    let _ = writeln!(
        record,
        "{}",
        json!({ "args": args, "cwd": cwd, "env": env, "answer_file": answer_file(args) })
    );
    let mut input = Input {
        lines: io::stdin().lock().lines(),
        record,
    };
    // Held while the stub lives and never waited for: a heartbeat beats until a job kills it.
    let mut heartbeats: Vec<Child> = Vec::new();

    for line in script.lines().filter(|line| !line.trim().is_empty()) {
        let directive: Value = serde_json::from_str(line).expect("each script line is JSON");
        let Some(kind) = directive["stub"].as_str() else {
            say(&line.replace("$SESSION", &session));
            continue;
        };
        let text = directive["text"].as_str().unwrap_or_default();
        match kind {
            "handshake" => {
                let request = input.until("the handshake", |value| {
                    value["type"] == "control_request"
                        && value["request"]["subtype"] == "initialize"
                });
                let id = &request["request_id"];
                let answer = match directive["error"].as_str() {
                    Some(error) => json!({"type": "control_response", "response":
                        {"subtype": "error", "request_id": id, "error": error}}),
                    None => json!({"type": "control_response", "response":
                        {"subtype": "success", "request_id": id, "response": {"commands": []}}}),
                };
                say(&answer.to_string());
            }
            "prompt" => {
                let mut prompt = input.until("the prompt", |value| value["type"] == "user");
                // The CLI's echo carries the session and a replay mark.
                if args.iter().any(|arg| arg == "--replay-user-messages") {
                    prompt["session_id"] = json!(session);
                    prompt["isReplay"] = json!(true);
                    say(&prompt.to_string());
                }
            }
            "answer" => {
                let id = directive["request_id"].clone();
                input.until("an answer", |value| {
                    value["type"] == "control_response" && value["response"]["request_id"] == id
                });
            }
            "eof" => while input.next().is_some() {},
            "hang" => loop {
                thread::sleep(Duration::from_secs(1));
            },
            "exit" => process::exit(directive["code"].as_i64().unwrap_or(0) as i32),
            "stderr" => eprintln!("{text}"),
            "raw" => say(text),
            "grandchild" => heartbeats.push(grandchild()),
            other => panic!("unknown directive {other:?}"),
        }
    }
}

/// The answer file of a `--answer <file>` hook in `--settings`, and what it holds when the run
/// starts; null when no hook names one.
fn answer_file(args: &[String]) -> Value {
    let settings = args
        .windows(2)
        .find(|pair| pair[0] == "--settings")
        .and_then(|pair| serde_json::from_str::<Value>(&pair[1]).ok())
        .unwrap_or(Value::Null);
    let path = settings["hooks"]["PreToolUse"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|group| group["hooks"].as_array().into_iter().flatten())
        .filter_map(|handler| handler["args"].as_array())
        .find_map(|args| {
            let at = args.iter().position(|arg| arg == "--answer")?;
            args.get(at + 1)?.as_str().map(str::to_owned)
        });
    match path {
        Some(path) => json!({ "content": fs::read_to_string(&path).ok(), "path": path }),
        None => Value::Null,
    }
}

fn say(line: &str) {
    let mut stdout = io::stdout().lock();
    let _ = writeln!(stdout, "{line}");
    let _ = stdout.flush();
}

/// Starts this program again as a heartbeat in the working folder, without a console window.
fn grandchild() -> Child {
    let exe = std::env::current_exe().expect("the stub knows where it is");
    let mut command = Command::new(exe);
    command
        .args(["--heartbeat", "heartbeat.txt"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command.spawn().expect("the heartbeat starts")
}
