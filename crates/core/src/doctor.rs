//! The checks behind `crw doctor` (D-29). Each external tool is asked for its version through
//! [`crate::process`], one child at a time; nothing here starts a session or spends usage.
//! The VS Code extension is read from VS Code's own list of installed extensions, with no process.
//! EP-28 adds the rest of the list: auth status, hooks, Tailscale state, `D:`, `gh` auth, the
//! registry and the ledger.

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs, thread};

use crate::process::{SPACING, TIMEOUT, output_of};
use crate::version::{MIN_CLAUDE_CODE, Version};

/// The Claude Code extension's identifier in VS Code.
const EXTENSION_ID: &str = "anthropic.claude-code";

/// What one check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub name: &'static str,
    pub outcome: Outcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// `source` is the program that answered, or the file the version was read from.
    Found { version: Version, source: PathBuf },
    /// Not on PATH and not where its installer puts it.
    Missing,
    /// Present but unreadable: it failed to start, timed out, exited with an error or printed no
    /// version.
    Failed { source: PathBuf, reason: String },
}

impl Check {
    pub fn version(&self) -> Option<Version> {
        match self.outcome {
            Outcome::Found { version, .. } => Some(version),
            _ => None,
        }
    }
}

/// Whether the Claude Code CLI found by [`run`] is at least [`MIN_CLAUDE_CODE`].
pub fn claude_meets_minimum(checks: &[Check]) -> bool {
    checks
        .iter()
        .find(|check| check.name == "claude")
        .and_then(Check::version)
        .is_some_and(|version| version >= MIN_CLAUDE_CODE)
}

/// Runs every check in order: `claude`, `gh`, `tailscale`, `code`, then the extension.
pub fn run() -> Vec<Check> {
    let dir = |var: &str| env::var_os(var).map(PathBuf::from);
    let home = dir("USERPROFILE");
    let local = dir("LOCALAPPDATA");
    let programs = dir("ProgramFiles");
    let under = |base: &Option<PathBuf>, rest: &str| base.as_ref().map(|b| b.join(rest));
    // Where each tool's installer puts it. The Claude Code launcher in `~\.local\bin` comes before
    // PATH, as the session runner will look for it (EP-9); the others come after PATH.
    let probes = [
        (
            "claude",
            vec![
                under(&home, r".local\bin\claude.exe"),
                on_path("claude.exe"),
            ],
            &["--version"][..],
        ),
        (
            "gh",
            vec![on_path("gh.exe"), under(&programs, r"GitHub CLI\gh.exe")],
            &["--version"][..],
        ),
        (
            "tailscale",
            vec![
                on_path("tailscale.exe"),
                under(&programs, r"Tailscale\tailscale.exe"),
            ],
            &["version"][..],
        ),
        (
            "code",
            vec![
                on_path("code.cmd"),
                under(&local, r"Programs\Microsoft VS Code\bin\code.cmd"),
                under(&programs, r"Microsoft VS Code\bin\code.cmd"),
            ],
            &["--version"][..],
        ),
    ];
    let mut checks = Vec::new();
    for (i, (name, candidates, args)) in probes.into_iter().enumerate() {
        if i > 0 {
            thread::sleep(SPACING);
        }
        let outcome = match candidates.into_iter().flatten().find(|p| p.is_file()) {
            Some(program) => version_of(&program, args, TIMEOUT),
            None => Outcome::Missing,
        };
        checks.push(Check { name, outcome });
    }
    checks.push(Check {
        name: "extension",
        outcome: match under(&home, r".vscode\extensions") {
            Some(folder) => extension_version(&folder),
            None => Outcome::Missing,
        },
    });
    checks
}

/// The first file called `name` in a folder on PATH.
pub(crate) fn on_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// Runs `program` with `args` and reads the first version in what it prints.
fn version_of(program: &Path, args: &[&str], timeout: Duration) -> Outcome {
    let failed = |reason: String| Outcome::Failed {
        source: program.to_owned(),
        reason,
    };
    match output_of(program, args, timeout) {
        Ok(text) => match Version::find_in(&text) {
            Some(version) => Outcome::Found {
                version,
                source: program.to_owned(),
            },
            None => failed("printed no version".to_owned()),
        },
        Err(reason) => failed(reason),
    }
}

/// The Claude Code extension's version from VS Code's `extensions.json` in `folder` (the
/// extensions folder), else from the newest `anthropic.claude-code-*` folder in it.
fn extension_version(folder: &Path) -> Outcome {
    let list = folder.join("extensions.json");
    if let Some(version) = fs::read_to_string(&list)
        .ok()
        .and_then(|json| listed_version(&json))
    {
        return Outcome::Found {
            version,
            source: list,
        };
    }
    let Ok(entries) = fs::read_dir(folder) else {
        return Outcome::Missing;
    };
    let prefix = format!("{EXTENSION_ID}-");
    entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let rest = name.to_ascii_lowercase().strip_prefix(&prefix)?.to_owned();
            Some((Version::find_in(&rest)?, entry.path()))
        })
        .max()
        .map_or(Outcome::Missing, |(version, source)| Outcome::Found {
            version,
            source,
        })
}

/// The highest version VS Code lists for the extension in its `extensions.json`.
fn listed_version(json: &str) -> Option<Version> {
    let entries: serde_json::Value = serde_json::from_str(json).ok()?;
    entries
        .as_array()?
        .iter()
        .filter(|entry| {
            entry["identifier"]["id"]
                .as_str()
                .is_some_and(|id| id.eq_ignore_ascii_case(EXTENSION_ID))
        })
        .filter_map(|entry| entry["version"].as_str()?.parse().ok())
        .max()
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    /// A fresh folder under the system temp folder, removed when dropped (never the project root).
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = env::temp_dir().join(format!("crw-test-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const LIST: &str = r#"[
        {"identifier":{"id":"other.publisher-tool"},"version":"9.9.9","relativeLocation":"other.publisher-tool-9.9.9"},
        {"identifier":{"id":"anthropic.claude-code"},"version":"2.1.314","relativeLocation":"anthropic.claude-code-2.1.314-win32-x64"}
    ]"#;

    #[test]
    fn reads_the_extension_from_vs_codes_list() {
        assert_eq!(listed_version(LIST), Some(Version::new(2, 1, 314)));
        assert_eq!(listed_version("[]"), None);
        assert_eq!(listed_version("not json"), None);
        assert_eq!(listed_version(r#"{"identifier":{}}"#), None);
    }

    #[test]
    fn prefers_the_list_and_falls_back_to_the_newest_folder() {
        let scratch = Scratch::new("extensions");
        for name in [
            "anthropic.claude-code-2.1.99-win32-x64",
            "anthropic.claude-code-2.1.314-win32-x64",
            "other.publisher-tool-9.9.9",
        ] {
            fs::create_dir(scratch.0.join(name)).unwrap();
        }
        let newest = scratch.0.join("anthropic.claude-code-2.1.314-win32-x64");
        assert_eq!(
            extension_version(&scratch.0),
            Outcome::Found {
                version: Version::new(2, 1, 314),
                source: newest,
            }
        );

        let list = scratch.0.join("extensions.json");
        fs::write(&list, LIST.replace("2.1.314", "2.1.400")).unwrap();
        assert_eq!(
            extension_version(&scratch.0),
            Outcome::Found {
                version: Version::new(2, 1, 400),
                source: list,
            }
        );
    }

    #[test]
    fn a_missing_extensions_folder_is_missing() {
        let scratch = Scratch::new("no-extensions");
        assert_eq!(
            extension_version(&scratch.0.join("absent")),
            Outcome::Missing
        );
    }

    #[test]
    fn the_minimum_applies_to_claude_only() {
        let found = |name, version| Check {
            name,
            outcome: Outcome::Found {
                version,
                source: PathBuf::from("x"),
            },
        };
        let old = Version::new(2, 1, 1);
        assert!(claude_meets_minimum(&[found("claude", MIN_CLAUDE_CODE)]));
        assert!(!claude_meets_minimum(&[found("claude", old)]));
        assert!(!claude_meets_minimum(&[found("gh", MIN_CLAUDE_CODE)]));
        assert!(!claude_meets_minimum(&[Check {
            name: "claude",
            outcome: Outcome::Missing,
        }]));
    }

    /// Spawns only Windows' own `cmd.exe` and `ping.exe`, never the `claude` CLI (D-29), and runs
    /// them one after the other in a single test.
    #[cfg(windows)]
    #[test]
    fn probes_read_output_exit_codes_and_time_out() {
        let system = PathBuf::from(env::var_os("SystemRoot").unwrap()).join("System32");
        let cmd = system.join("cmd.exe");
        assert_eq!(
            version_of(&cmd, &["/C", "echo 9.8.7 (stub)"], TIMEOUT),
            Outcome::Found {
                version: Version::new(9, 8, 7),
                source: cmd.clone(),
            }
        );
        assert!(matches!(
            version_of(&cmd, &["/C", "echo no version here"], TIMEOUT),
            Outcome::Failed { reason, .. } if reason == "printed no version"
        ));
        assert!(matches!(
            version_of(&cmd, &["/C", "exit 3"], TIMEOUT),
            Outcome::Failed { reason, .. } if reason.starts_with("exited with")
        ));
        let started = Instant::now();
        assert!(matches!(
            version_of(&system.join("ping.exe"), &["-n", "30", "127.0.0.1"], Duration::from_millis(300)),
            Outcome::Failed { reason, .. } if reason.starts_with("no answer within")
        ));
        assert!(started.elapsed() < Duration::from_secs(10));
    }
}
