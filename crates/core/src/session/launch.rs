//! What a headless run is started with (D-15, D-16, D-18, D-22): the [`Launch`] its caller fills,
//! and the command line and environment the runner builds from it and from the flags the CLI's
//! help lists. A feature is passed only when the help lists it (D-15): a wish the CLI cannot take,
//! `--effort ultracode` or a budget, is left out and said so, and a flag the run cannot do without
//! refuses the launch. Neighbour folders join with `--add-dir`, kept read-only by an `Edit` deny
//! rule each in `--settings` (D-16, D-18 addenda).

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf, Prefix};
use std::time::Duration;

use super::cli::{Flags, REQUIRED};
use crate::paths;

/// How long a graceful stop waits for the session to exit before its job is killed.
pub const STOP_TIMEOUT: Duration = Duration::from_secs(30);

/// With it, Claude Code loads the neighbours' `CLAUDE.md` and rules along with their files, which
/// a folder granted in a settings file never does (D-16 addendum).
pub const NEIGHBOUR_CLAUDE_MD: &str = "CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD";

/// What a session's child never inherits besides the API key (D-15): the markers of a Claude
/// Code session the tool itself may run in, with which a nested `claude -p` does not start; the
/// prefix covers the `CLAUDE_CODE_*` ones. Settings meant for every session belong in the `env`
/// block of the user settings, which the child reads itself.
const MARKERS: [&str; 4] = [
    "CLAUDECODE",
    "CLAUDE_PID",
    "CLAUDE_AGENT_SDK_VERSION",
    "AI_AGENT",
];
const MARKER_PREFIX: &str = "CLAUDE_CODE_";

/// The key that would bill the API instead of the subscription (D-15).
const API_KEY: &str = "ANTHROPIC_API_KEY";

/// A headless run, as its caller asks for it.
#[derive(Debug, Clone, PartialEq)]
pub struct Launch {
    /// The project's name in the registry; the session is named after it.
    pub project: String,
    /// The project's folder: the session's working folder (D-16).
    pub folder: PathBuf,
    /// The EP the run works on, such as `EP-3`.
    pub ep: Option<String>,
    /// A model alias or id, passed as written: `fable`, `claude-opus-5-5`.
    pub model: String,
    pub effort: Option<String>,
    /// `--effort ultracode` in place of the effort, when the CLI lists it (D-22).
    pub ultracode: bool,
    /// `acceptEdits` unless the project trusts more (D-18).
    pub permission_mode: String,
    /// Full paths of the folders the session may read and never edit (D-16, D-18 addenda).
    pub neighbours: Vec<PathBuf>,
    pub prompt: String,
    /// `--max-budget-usd`, when the CLI lists it (D-22).
    pub max_budget_usd: Option<f64>,
    /// How long a graceful stop waits before the job is killed.
    pub stop_timeout: Duration,
    /// The ledger the run is recorded in; none for a trial.
    pub ledger: Option<PathBuf>,
    /// The session this run continues (`--resume <id>`), the prompt its next message; none for a
    /// new session (EP-13). A resumed run keeps the session's id (the CLI's `--fork-session` is
    /// what would change it).
    pub resume: Option<String>,
}

impl Launch {
    /// A run of `prompt` in `folder` with `model`: `acceptEdits` (D-18), no neighbour, a stop
    /// that waits 30 seconds, nothing recorded, a new session.
    pub fn new(project: &str, folder: &Path, model: &str, prompt: &str) -> Self {
        Self {
            project: project.to_owned(),
            folder: folder.to_owned(),
            ep: None,
            model: model.to_owned(),
            effort: None,
            ultracode: false,
            permission_mode: "acceptEdits".to_owned(),
            neighbours: Vec::new(),
            prompt: prompt.to_owned(),
            max_budget_usd: None,
            stop_timeout: STOP_TIMEOUT,
            ledger: None,
            resume: None,
        }
    }

    /// The name the session goes by in Claude Code's lists: `<project> EP-n` (D-16).
    pub fn name(&self) -> String {
        match &self.ep {
            Some(ep) => format!("{} {ep}", self.project),
            None => self.project.clone(),
        }
    }
}

/// One run's arguments and environment, built from its [`Launch`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CommandLine {
    pub args: Vec<OsString>,
    /// Each variable set to its value, or removed when `None`.
    pub env: Vec<(OsString, Option<OsString>)>,
    /// The effort passed, `ultracode` included.
    pub effort: Option<String>,
    /// What the run asked for that the CLI cannot take, in words.
    pub dropped: Vec<String>,
}

/// The command line for `launch` with the pre-assigned `session_id` (the resumed session's own for
/// a resume), given the flags the CLI lists and the names of the variables the tool's own
/// environment holds.
pub(crate) fn command_line(
    launch: &Launch,
    flags: &Flags,
    session_id: &str,
    inherited: impl IntoIterator<Item = OsString>,
) -> Result<CommandLine, String> {
    let missing: Vec<&str> = REQUIRED
        .into_iter()
        .filter(|flag| !flags.has(flag))
        .collect();
    if !missing.is_empty() {
        return Err(format!("claude --help lists no {}", missing.join(", ")));
    }
    let mut args: Vec<OsString> = Vec::new();
    let mut push = |values: &[&OsStr]| args.extend(values.iter().map(|v| v.to_os_string()));
    let os = OsStr::new;
    let mut dropped = Vec::new();

    push(&[
        os("-p"),
        os("--verbose"),
        os("--output-format"),
        os("stream-json"),
        os("--input-format"),
        os("stream-json"),
    ]);
    // Without them the run still works: no text as it streams, no echo of the prompt.
    for optional in ["--include-partial-messages", "--replay-user-messages"] {
        if flags.has(optional) {
            push(&[os(optional)]);
        }
    }
    // A new session takes its pre-assigned id; a resumed one is named by the id it has.
    match &launch.resume {
        Some(id) if !flags.has("--resume") => {
            return Err(format!(
                "this claude has no --resume, which resuming the session {id} needs"
            ));
        }
        Some(id) => push(&[os("--resume"), os(id)]),
        None => push(&[os("--session-id"), os(session_id)]),
    }
    let name = launch.name();
    if flags.has("--name") {
        push(&[os("--name"), os(&name)]);
    }
    push(&[os("--model"), os(&launch.model)]);

    let effort = if launch.ultracode && flags.ultracode() {
        Some("ultracode")
    } else {
        if launch.ultracode {
            dropped.push("--effort ultracode: this claude does not list it".to_owned());
        }
        launch.effort.as_deref()
    };
    let effort = match effort {
        Some(level) if !flags.has("--effort") => {
            dropped.push(format!("--effort {level}: this claude has no --effort"));
            None
        }
        Some(level) if !flags.efforts.is_empty() && !flags.efforts.iter().any(|l| l == level) => {
            return Err(format!(
                "this claude takes --effort {}, not {level}",
                flags.efforts.join(", ")
            ));
        }
        other => other,
    };
    if let Some(level) = effort {
        push(&[os("--effort"), os(level)]);
    }

    let mode = &launch.permission_mode;
    if !flags.permission_modes.is_empty() && !flags.permission_modes.contains(mode) {
        return Err(format!(
            "this claude takes --permission-mode {}, not {mode}",
            flags.permission_modes.join(", ")
        ));
    }
    push(&[
        os("--permission-mode"),
        os(mode),
        os("--permission-prompts"),
        os("host"),
    ]);

    let mut neighbours = Vec::new();
    let mut rules = Vec::new();
    for folder in &launch.neighbours {
        if !folder.is_dir() {
            dropped.push(format!("neighbour {}: no such folder", folder.display()));
            continue;
        }
        if within(&launch.folder, folder) {
            return Err(format!(
                "neighbour {} holds the project's own folder, which its deny rule would cover",
                folder.display()
            ));
        }
        let rule = edit_deny_rule(folder).ok_or_else(|| {
            format!(
                "neighbour {} is not on a drive, so no deny rule can keep it read-only",
                folder.display()
            )
        })?;
        neighbours.push(folder);
        rules.push(rule);
    }
    if !neighbours.is_empty() {
        for flag in ["--add-dir", "--settings"] {
            if !flags.has(flag) {
                return Err(format!(
                    "this claude has no {flag}, which read-only neighbours need"
                ));
            }
        }
        for folder in &neighbours {
            push(&[os("--add-dir"), folder.as_os_str()]);
        }
        let settings = serde_json::json!({"permissions": {"deny": rules}}).to_string();
        push(&[os("--settings"), os(&settings)]);
    }

    if let Some(budget) = launch.max_budget_usd {
        if flags.has("--max-budget-usd") {
            push(&[os("--max-budget-usd"), os(&budget.to_string())]);
        } else {
            dropped.push("--max-budget-usd: this claude does not list it".to_owned());
        }
    }

    let mut env: Vec<(OsString, Option<OsString>)> = markers(inherited)
        .into_iter()
        .map(|name| (name, None))
        .collect();
    if !neighbours.is_empty() {
        env.push((NEIGHBOUR_CLAUDE_MD.into(), Some("1".into())));
    }
    Ok(CommandLine {
        args,
        env,
        effort: effort.map(str::to_owned),
        dropped,
    })
}

/// The variables among `names` a session's child must not inherit, with the API key always
/// among them (Windows compares names without case).
pub(crate) fn markers(names: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut found: Vec<OsString> = names
        .into_iter()
        .filter(|name| {
            let upper = name.to_string_lossy().to_uppercase();
            upper != API_KEY
                && (MARKERS.contains(&upper.as_str()) || upper.starts_with(MARKER_PREFIX))
        })
        .collect();
    found.push(API_KEY.into());
    found
}

/// The `Edit` deny rule that keeps `folder` read-only: `Edit(//c/Data/beta/**)` for
/// `C:\Data\beta`, the absolute POSIX form Claude Code matches Windows paths in, the drive in
/// lower case and gitignore's pattern characters escaped so that the rule names that folder only
/// (D-18 addendum). `None` for a folder not on a drive: a network share or a relative path.
pub fn edit_deny_rule(folder: &Path) -> Option<String> {
    let folder = paths::normalize(folder);
    let mut components = folder.components();
    let drive = match components.next()? {
        Component::Prefix(prefix) => match prefix.kind() {
            Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
                char::from(letter).to_ascii_lowercase()
            }
            _ => return None,
        },
        _ => return None,
    };
    let mut rule = format!("Edit(//{drive}");
    for component in components {
        match component {
            Component::RootDir => {}
            Component::Normal(name) => {
                rule.push('/');
                for c in name.to_string_lossy().chars() {
                    if matches!(c, '*' | '?' | '[' | ']' | '\\') {
                        rule.push('\\');
                    }
                    rule.push(c);
                }
            }
            _ => return None,
        }
    }
    rule.push_str("/**)");
    Some(rule)
}

/// Whether `inner` is `outer` or a folder inside it, compared as Windows does.
fn within(inner: &Path, outer: &Path) -> bool {
    let key = |path: &Path| {
        let text = paths::normalize(path)
            .to_string_lossy()
            .replace('/', "\\")
            .to_lowercase();
        text.trim_end_matches('\\').to_owned()
    };
    let (inner, outer) = (key(inner), key(outer));
    inner == outer || inner.starts_with(&format!("{outer}\\"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::files::tests::Scratch;

    const HELP: &str = "Options:
  --add-dir <directories...>  Additional directories
  --effort <level>  Effort level (low, medium, high, xhigh, max)
  --include-partial-messages  Partial chunks
  --input-format <format>  Input format
  --max-budget-usd <amount>  Budget
  --model <model>  Model
  -n, --name <name>  Display name
  --output-format <format>  Output format
  --permission-mode <mode>  Mode (choices: \"acceptEdits\", \"auto\", \"plan\")
  --permission-prompts <target>  Who answers
  -p, --print  Print
  --replay-user-messages  Echo
  -r, --resume [value]  Resume a conversation by session ID or name
  --session-id <uuid>  Session id
  --settings <file-or-json>  Settings
  --verbose  Verbose
";

    fn strings(line: &CommandLine) -> Vec<String> {
        line.args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    fn value_after<'a>(args: &'a [String], flag: &str) -> Vec<&'a str> {
        args.windows(2)
            .filter(|pair| pair[0] == flag)
            .map(|pair| pair[1].as_str())
            .collect()
    }

    #[test]
    fn builds_the_planned_command_line() {
        let scratch = Scratch::new("launch-line");
        let project = scratch.0.join("alpha");
        let beta = scratch.0.join("beta");
        let gamma = scratch.0.join("gamma");
        for folder in [&project, &beta, &gamma] {
            fs::create_dir_all(folder).unwrap();
        }
        let mut launch = Launch::new("alpha", &project, "claude-fable-5-1", "Run EP-3.");
        launch.ep = Some("EP-3".to_owned());
        launch.effort = Some("xhigh".to_owned());
        launch.neighbours = vec![beta.clone(), gamma.clone(), scratch.0.join("gone")];
        launch.max_budget_usd = Some(2.5);
        let line = command_line(
            &launch,
            &Flags::parse(HELP),
            "0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b",
            [OsString::from("PATH")],
        )
        .unwrap();
        let args = strings(&line);
        assert_eq!(
            args[..8],
            [
                "-p",
                "--verbose",
                "--output-format",
                "stream-json",
                "--input-format",
                "stream-json",
                "--include-partial-messages",
                "--replay-user-messages"
            ]
        );
        assert_eq!(
            value_after(&args, "--session-id"),
            ["0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b"]
        );
        assert_eq!(value_after(&args, "--name"), ["alpha EP-3"]);
        assert_eq!(value_after(&args, "--model"), ["claude-fable-5-1"]);
        assert_eq!(value_after(&args, "--effort"), ["xhigh"]);
        assert_eq!(value_after(&args, "--permission-mode"), ["acceptEdits"]);
        assert_eq!(value_after(&args, "--permission-prompts"), ["host"]);
        assert_eq!(value_after(&args, "--max-budget-usd"), ["2.5"]);
        let added: Vec<String> = [&beta, &gamma]
            .iter()
            .map(|f| f.display().to_string())
            .collect();
        assert_eq!(value_after(&args, "--add-dir"), added);
        let settings: serde_json::Value =
            serde_json::from_str(value_after(&args, "--settings")[0]).unwrap();
        assert_eq!(
            settings["permissions"]["deny"],
            serde_json::json!([edit_deny_rule(&beta), edit_deny_rule(&gamma)])
        );
        assert!(!args.iter().any(|arg| arg == "--bare"));
        assert_eq!(line.effort.as_deref(), Some("xhigh"));
        assert_eq!(line.dropped.len(), 1, "{:?}", line.dropped);
        assert!(line.dropped[0].contains("no such folder"));
        assert_eq!(
            line.env,
            [
                (OsString::from(API_KEY), None),
                (
                    OsString::from(NEIGHBOUR_CLAUDE_MD),
                    Some(OsString::from("1"))
                )
            ]
        );
    }

    #[test]
    fn leaves_out_what_the_cli_does_not_list() {
        let scratch = Scratch::new("launch-dropped");
        let mut launch = Launch::new("alpha", &scratch.0, "opus", "Go.");
        launch.ultracode = true;
        launch.effort = Some("xhigh".to_owned());
        launch.max_budget_usd = Some(1.0);
        let bare = HELP
            .lines()
            .filter(|line| !line.contains("--max-budget-usd") && !line.contains("--name"))
            .collect::<Vec<_>>()
            .join("\n");
        let line = command_line(&launch, &Flags::parse(&bare), "id", []).unwrap();
        let args = strings(&line);
        assert_eq!(value_after(&args, "--effort"), ["xhigh"]);
        assert!(value_after(&args, "--name").is_empty());
        assert!(value_after(&args, "--max-budget-usd").is_empty());
        assert!(value_after(&args, "--add-dir").is_empty());
        assert_eq!(line.dropped.len(), 2, "{:?}", line.dropped);
        assert_eq!(line.env, [(OsString::from(API_KEY), None)]);

        // A CLI that lists ultracode gets it in place of the level.
        let ultra = HELP.replace("xhigh, max)", "xhigh, max, ultracode)");
        let line = command_line(&launch, &Flags::parse(&ultra), "id", []).unwrap();
        assert_eq!(value_after(&strings(&line), "--effort"), ["ultracode"]);
        assert_eq!(line.effort.as_deref(), Some("ultracode"));
    }

    #[test]
    fn a_resume_names_the_session_it_continues() {
        let scratch = Scratch::new("launch-resume");
        let mut launch = Launch::new("alpha", &scratch.0, "opus", "Go on from the tests.");
        launch.ep = Some("EP-3".to_owned());
        launch.resume = Some("0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b".to_owned());
        let flags = Flags::parse(HELP);
        let line =
            command_line(&launch, &flags, "0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b", []).unwrap();
        let args = strings(&line);
        assert_eq!(
            value_after(&args, "--resume"),
            ["0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b"]
        );
        assert!(value_after(&args, "--session-id").is_empty());
        assert_eq!(value_after(&args, "--name"), ["alpha EP-3"]);
        // A CLI without the flag cannot resume.
        let without = Flags::parse(
            &HELP
                .lines()
                .filter(|line| !line.contains("--resume"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        assert!(
            command_line(&launch, &without, "id", [])
                .unwrap_err()
                .contains("--resume")
        );
    }

    #[test]
    fn refuses_what_cannot_run() {
        let scratch = Scratch::new("launch-refused");
        let flags = Flags::parse(HELP);
        let launch = Launch::new("alpha", &scratch.0, "opus", "Go.");
        let refused =
            |launch: &Launch, flags: &Flags| command_line(launch, flags, "id", []).unwrap_err();
        let mut wrong = launch.clone();
        wrong.effort = Some("extreme".to_owned());
        assert!(refused(&wrong, &flags).contains("not extreme"));
        let mut wrong = launch.clone();
        wrong.permission_mode = "yolo".to_owned();
        assert!(refused(&wrong, &flags).contains("not yolo"));
        let old = Flags::parse(&HELP.replace("--permission-prompts", "--other"));
        assert!(refused(&launch, &old).contains("--permission-prompts"));
        // A neighbour that holds the project would make the project read-only.
        let mut holder = launch.clone();
        holder.folder = scratch.0.join("alpha");
        fs::create_dir_all(&holder.folder).unwrap();
        holder.neighbours = vec![scratch.0.clone()];
        assert!(refused(&holder, &flags).contains("holds the project"));
        let mut no_settings = launch.clone();
        no_settings.folder = scratch.0.join("alpha");
        let beta = scratch.0.join("beta");
        fs::create_dir_all(&beta).unwrap();
        no_settings.neighbours = vec![beta];
        let without = Flags::parse(&HELP.replace("--settings", "--other"));
        assert!(refused(&no_settings, &without).contains("--settings"));
    }

    #[test]
    fn deny_rules_name_the_folder_in_posix_form() {
        assert_eq!(
            edit_deny_rule(Path::new(r"C:\Data\beta")).as_deref(),
            Some("Edit(//c/Data/beta/**)")
        );
        assert_eq!(
            edit_deny_rule(Path::new(r"D:\Work\alpha\..\Finance (2024)\")).as_deref(),
            Some("Edit(//d/Work/Finance (2024)/**)")
        );
        assert_eq!(
            edit_deny_rule(Path::new(r"\\?\E:\odd[1]*")).as_deref(),
            Some(r"Edit(//e/odd\[1\]\*/**)")
        );
        assert_eq!(edit_deny_rule(Path::new(r"\\server\share\beta")), None);
        assert_eq!(edit_deny_rule(Path::new(r"relative\beta")), None);
    }

    #[test]
    fn markers_and_the_key_stay_out_of_the_child() {
        let names = [
            "PATH",
            "CLAUDECODE",
            "claude_code_session_id",
            "CLAUDE_CODE_ENTRYPOINT",
            "CLAUDE_PID",
            "AI_AGENT",
            "CLAUDE_CONFIG_DIR",
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_BASE_URL",
        ]
        .map(OsString::from);
        let removed = markers(names);
        assert_eq!(
            removed,
            [
                "CLAUDECODE",
                "claude_code_session_id",
                "CLAUDE_CODE_ENTRYPOINT",
                "CLAUDE_PID",
                "AI_AGENT",
                "ANTHROPIC_API_KEY"
            ]
            .map(OsString::from)
        );
    }

    #[test]
    fn within_compares_as_windows_does() {
        assert!(within(Path::new(r"C:\Data\alpha"), Path::new(r"c:\data")));
        assert!(within(
            Path::new(r"C:\Data\alpha"),
            Path::new(r"C:\Data\Alpha\")
        ));
        assert!(!within(
            Path::new(r"C:\Data\alphabet"),
            Path::new(r"C:\Data\alpha")
        ));
        assert!(!within(Path::new(r"C:\Data"), Path::new(r"C:\Data\alpha")));
    }
}
