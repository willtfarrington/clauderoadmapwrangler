//! Finding the Claude Code CLI and asking it, before a headless launch, whether it can run one
//! (D-15, D-29): its version against the minimum, whether it is logged in, and the flags its help
//! lists. Three short children, one at a time, without the API key or a parent session's markers
//! in their environment; nothing here starts a session or spends usage. The answers are kept for
//! an hour, since the CLI updates itself (risk 3). The help is where the features come from, never
//! an assumption: a flag it does not list is one the runner does without, or a reason to refuse
//! when the runner cannot do without it.

use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::launch;
use crate::doctor::on_path;
use crate::process::{self, SPACING, TIMEOUT};
use crate::version::{MIN_CLAUDE_CODE, Version};

/// How long a preflight's answers are trusted.
pub const MAX_AGE: Duration = Duration::from_secs(60 * 60);

/// The flags the runner cannot start a session without.
pub const REQUIRED: [&str; 8] = [
    "--print",
    "--verbose",
    "--output-format",
    "--input-format",
    "--session-id",
    "--model",
    "--permission-mode",
    "--permission-prompts",
];

/// Where the CLI is: the registry's `settings.claude_cli` when set, else the launcher Claude
/// Code's installer puts in `~\.local\bin` (it updates itself), else the first `claude.exe` on
/// PATH.
pub fn locate(setting: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = setting {
        return Some(path.to_owned());
    }
    env::var_os("USERPROFILE")
        .map(|home| PathBuf::from(home).join(r".local\bin\claude.exe"))
        .filter(|launcher| launcher.is_file())
        .or_else(|| on_path("claude.exe"))
}

/// What the CLI said before a launch.
#[derive(Debug, Clone)]
pub struct Preflight {
    pub program: PathBuf,
    /// Whether `program` is a file at all; the three answers are errors when it is not.
    pub found: bool,
    pub version: Result<Version, String>,
    pub auth: Result<Auth, String>,
    pub flags: Result<Flags, String>,
    pub taken: Instant,
}

impl Preflight {
    /// Asks `program` for its version, its login and its help, one child after the other.
    pub fn run(program: &Path) -> Self {
        let taken = Instant::now();
        if !program.is_file() {
            let missing = format!("no Claude Code CLI at {}", program.display());
            return Self {
                program: program.to_owned(),
                found: false,
                version: Err(missing.clone()),
                auth: Err(missing.clone()),
                flags: Err(missing),
                taken,
            };
        }
        let markers = launch::markers(env::vars_os().map(|(name, _)| name));
        let ask = |args: &[&str]| process::output_without(program, args, &markers, TIMEOUT);
        let _turn = process::turn();
        let version = ask(&["--version"]).and_then(|text| {
            Version::find_in(&text).ok_or_else(|| "printed no version".to_owned())
        });
        thread::sleep(SPACING);
        let auth = ask(&["auth", "status"]).and_then(|text| Auth::parse(&text));
        thread::sleep(SPACING);
        let flags = ask(&["--help"]).map(|text| Flags::parse(&text));
        Self {
            program: program.to_owned(),
            found: true,
            version,
            auth,
            flags,
            taken,
        }
    }

    /// Why a headless launch is refused, every reason in order; none when it may start.
    pub fn refusals(&self) -> Vec<String> {
        if !self.found {
            return vec![format!("no Claude Code CLI at {}", self.program.display())];
        }
        let mut reasons = Vec::new();
        match &self.version {
            Ok(version) if *version < MIN_CLAUDE_CODE => reasons.push(format!(
                "claude {version} is older than {MIN_CLAUDE_CODE}, the oldest the tool drives"
            )),
            Ok(_) => {}
            Err(e) => reasons.push(format!("claude --version: {e}")),
        }
        match &self.auth {
            Ok(auth) if !auth.logged_in => {
                reasons.push("claude is not logged in (claude auth login)".to_owned());
            }
            Ok(_) => {}
            Err(e) => reasons.push(format!("claude auth status: {e}")),
        }
        match &self.flags {
            Ok(flags) => {
                let missing: Vec<&str> = REQUIRED
                    .into_iter()
                    .filter(|flag| !flags.has(flag))
                    .collect();
                if !missing.is_empty() {
                    reasons.push(format!("claude --help lists no {}", missing.join(", ")));
                }
            }
            Err(e) => reasons.push(format!("claude --help: {e}")),
        }
        reasons
    }

    /// Whether the answers are recent enough to launch on.
    pub fn is_fresh(&self) -> bool {
        self.taken.elapsed() < MAX_AGE
    }
}

/// The latest preflight, asked again when it is an hour old or was taken of another program.
#[derive(Debug, Default)]
pub struct Cache(Option<Arc<Preflight>>);

impl Cache {
    pub fn get(&mut self, program: &Path) -> Arc<Preflight> {
        match &self.0 {
            Some(preflight) if preflight.program == program && preflight.is_fresh() => {
                Arc::clone(preflight)
            }
            _ => {
                let fresh = Arc::new(Preflight::run(program));
                self.0 = Some(Arc::clone(&fresh));
                fresh
            }
        }
    }

    /// Drops the answers, so that the next launch asks again (after a failure, or an update).
    pub fn forget(&mut self) {
        self.0 = None;
    }
}

/// `claude auth status`, read: its JSON's `loggedIn`, `authMethod` and `subscriptionType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Auth {
    pub logged_in: bool,
    pub method: Option<String>,
    pub subscription: Option<String>,
}

impl Auth {
    pub fn parse(text: &str) -> Result<Self, String> {
        let json: serde_json::Value = serde_json::from_str(text.trim())
            .map_err(|e| format!("its answer is not the JSON expected: {e}"))?;
        let logged_in = json["loggedIn"]
            .as_bool()
            .ok_or_else(|| "its answer has no loggedIn".to_owned())?;
        let text = |key: &str| json[key].as_str().map(str::to_owned);
        Ok(Self {
            logged_in,
            method: text("authMethod"),
            subscription: text("subscriptionType"),
        })
    }
}

/// The options a CLI's help lists, with the levels of `--effort` and the choices of
/// `--permission-mode` when it lists them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Flags {
    names: BTreeSet<String>,
    /// `--effort`'s levels; empty when the help does not say them.
    pub efforts: Vec<String>,
    /// `--permission-mode`'s choices; empty when the help does not say them.
    pub permission_modes: Vec<String>,
}

impl Flags {
    /// Reads the `Options:` section of a help: an option's line starts with a dash at most four
    /// spaces in, its spellings before the first two spaces in a row, and its description runs on
    /// over the lines indented further, which may start with a flag's name themselves.
    pub fn parse(help: &str) -> Self {
        let mut options: Vec<(Vec<String>, String)> = Vec::new();
        let mut in_options = false;
        for line in help.lines() {
            let text = line.trim_start();
            if text.is_empty() {
                continue;
            }
            let indent = line.len() - text.len();
            if indent == 0 {
                in_options = text.trim_end().eq_ignore_ascii_case("options:");
                continue;
            }
            if !in_options {
                continue;
            }
            if indent <= 4 && text.starts_with('-') {
                let (spec, description) = match text.find("  ") {
                    Some(at) => (&text[..at], text[at..].trim()),
                    None => (text.trim_end(), ""),
                };
                let names = spec
                    .split(',')
                    .filter_map(|spelling| spelling.split_whitespace().next())
                    .filter(|name| name.starts_with('-'))
                    .map(str::to_owned)
                    .collect();
                options.push((names, description.to_owned()));
            } else if let Some((_, description)) = options.last_mut() {
                if !description.is_empty() {
                    description.push(' ');
                }
                description.push_str(text.trim_end());
            }
        }
        let description = |flag: &str| {
            options
                .iter()
                .find(|(names, _)| names.iter().any(|name| name == flag))
                .map(|(_, description)| description.as_str())
        };
        Self {
            efforts: description("--effort").map(listed).unwrap_or_default(),
            permission_modes: description("--permission-mode")
                .map(listed)
                .unwrap_or_default(),
            names: options.into_iter().flat_map(|(names, _)| names).collect(),
        }
    }

    /// Whether the help lists `flag` under any of its spellings.
    pub fn has(&self, flag: &str) -> bool {
        self.names.contains(flag)
    }

    /// Whether `--effort ultracode` is listed (D-22): passed only then.
    pub fn ultracode(&self) -> bool {
        self.efforts.iter().any(|level| level == "ultracode")
    }
}

/// The values a description lists in its first parentheses: `(low, medium, high)`, or `(choices:
/// "a", "b", default: "a")` without the default.
fn listed(description: &str) -> Vec<String> {
    let Some(inside) = description
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')'))
        .map(|(inside, _)| inside)
    else {
        return Vec::new();
    };
    let inside = inside.split("default:").next().unwrap_or_default();
    let inside = inside.trim().strip_prefix("choices:").unwrap_or(inside);
    inside
        .split([',', '|'])
        .map(|value| value.trim().trim_matches('"').trim())
        .filter(|value| !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric()))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A help in the shape `claude --help` prints into a pipe (2.1.316): two spaces before an
    /// option, a long spelling alone on its line with the description six spaces in, wrapped
    /// descriptions forty spaces in, some of their lines starting with a flag's name.
    const HELP: &str = "\
Usage: claude [options] [command] [prompt]

A synthetic help for the tests.

Arguments:
  prompt                                Your prompt

Options:
  --add-dir <directories...>            Additional directories to allow tool
                                        access to
  --allowedTools, --allowed-tools <tools...>
      Comma or space-separated list of tool names to allow
  --bare                                Minimal mode. Explicitly provide context
                                        via: --system-prompt[-file],
                                        --made-up-in-a-description, --add-dir
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
                                        --print (choices: \"host\", \"none\",
                                        default: \"host\")
  -p, --print                           Print response and exit
  --replay-user-messages                Re-emit user messages from stdin
  --session-id <uuid>                   Use a specific session ID
  --settings <file-or-json>             Path to a settings JSON file
  --verbose                             Override verbose mode setting
  -w, --worktree [name]                 Create a new git worktree

Commands:
  agents [options]                      Manage background agents
  --not-an-option                       a command section's line
";

    #[test]
    fn reads_the_options_and_their_lists() {
        let flags = Flags::parse(HELP);
        for flag in REQUIRED {
            assert!(flags.has(flag), "{flag}");
        }
        for flag in [
            "-p",
            "-n",
            "--name",
            "--add-dir",
            "--allowedTools",
            "--allowed-tools",
            "--settings",
            "--max-budget-usd",
            "--worktree",
        ] {
            assert!(flags.has(flag), "{flag}");
        }
        // A flag named inside a description, or in another section, is not an option.
        assert!(!flags.has("--made-up-in-a-description"));
        assert!(!flags.has("--not-an-option"));
        assert!(!flags.has("--max-turns"));
        assert_eq!(flags.efforts, ["low", "medium", "high", "xhigh", "max"]);
        assert!(!flags.ultracode());
        assert_eq!(
            flags.permission_modes,
            [
                "acceptEdits",
                "auto",
                "bypassPermissions",
                "manual",
                "dontAsk",
                "plan"
            ]
        );
    }

    #[test]
    fn lists_read_in_their_forms() {
        assert_eq!(
            listed("Level (low|max|ultracode)"),
            ["low", "max", "ultracode"]
        );
        assert_eq!(
            listed(r#"Who (choices: "host", "none", default: "host")"#),
            ["host", "none"]
        );
        assert!(listed("no list here").is_empty());
        let flags = Flags::parse("Options:\n  --effort <level>  Effort (low, ultracode)\n");
        assert!(flags.ultracode());
        assert!(Flags::parse("").efforts.is_empty());
    }

    #[test]
    fn reads_the_login() {
        assert_eq!(
            Auth::parse(
                r#"{"loggedIn": true, "authMethod": "claude.ai", "subscriptionType": "pro"}"#
            ),
            Ok(Auth {
                logged_in: true,
                method: Some("claude.ai".to_owned()),
                subscription: Some("pro".to_owned()),
            })
        );
        assert!(!Auth::parse(r#"{"loggedIn": false}"#).unwrap().logged_in);
        assert!(Auth::parse("Logged in as someone").is_err());
        assert!(Auth::parse(r#"{"authMethod": "none"}"#).is_err());
    }

    fn preflight(version: Version, logged_in: bool, help: &str) -> Preflight {
        Preflight {
            program: PathBuf::from(r"C:\Tools\claude.exe"),
            found: true,
            version: Ok(version),
            auth: Ok(Auth {
                logged_in,
                method: None,
                subscription: None,
            }),
            flags: Ok(Flags::parse(help)),
            taken: Instant::now(),
        }
    }

    #[test]
    fn refuses_with_every_reason() {
        assert!(preflight(MIN_CLAUDE_CODE, true, HELP).refusals().is_empty());
        let old = preflight(
            Version::new(2, 1, 1),
            false,
            "Options:\n  -p, --print  Print\n",
        );
        let reasons = old.refusals();
        assert_eq!(reasons.len(), 3, "{reasons:?}");
        assert!(reasons[0].contains("older than"));
        assert!(reasons[1].contains("not logged in"));
        assert!(reasons[2].contains("--permission-prompts"));
        assert!(!reasons[2].contains("--print"));
        let missing = Preflight::run(Path::new(r"C:\nowhere\claude.exe"));
        assert_eq!(
            missing.refusals(),
            [r"no Claude Code CLI at C:\nowhere\claude.exe"]
        );
    }

    #[test]
    fn the_setting_wins() {
        let set = Path::new(r"D:\Tools\claude.exe");
        assert_eq!(locate(Some(set)), Some(set.to_owned()));
    }
}
