//! The checks behind `crw doctor` and the Diagnostics tile (D-29). Each external tool is asked
//! for its version through [`crate::process`], one child at a time in its turn; nothing here
//! starts a session or spends usage. The VS Code extension is read from VS Code's own list of
//! installed extensions, with no process. EP-28 adds the [`Report`]: [`probe`] asks the children
//! (the versions, `gh auth status`, `tailscale status --json`), [`report`] reads the files (the
//! hooks and the status line, the installed copy, the backup drive, the registry, the ledger,
//! the crash log, the rolling log, the last readings, VS Code's lock files) and words each line.
//! EP-28a adds the start-at-login entry and what PATH finds for `crw` to the exes' group.

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs};

use chrono::{DateTime, Local, TimeDelta, Utc};

use crate::badges::{backup, online, short_age};
use crate::inbox::mailman;
use crate::ledger::{self, Ledger};
use crate::process::{self, TIMEOUT, output_of};
use crate::prompt::{Draft, Parts, Prompt};
use crate::registry::{self as registry_file, Project, Registry, Settings, State, Store};
use crate::roadmap::{EpId, Size};
use crate::session::PUSH_DENY_RULE;
use crate::session::cli::Flags;
use crate::session::launch::{CommandLine, command_line};
use crate::signals::install::{self as entries, Hooked, Line as StatusLine};
use crate::version::{MIN_CLAUDE_CODE, Version};
use crate::{actions, autostart, install, log, notify, plugin, usage, visual};
use crate::{autopilot, rules};

/// The protocol crate's pin (D-15): `claude-codes` moves only in a re-plan; a test holds this
/// to `Cargo.lock`.
pub const PROTOCOL_PIN: &str = "2.1.316";

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
    for (name, candidates, args) in probes {
        let outcome = match candidates.into_iter().flatten().find(|p| p.is_file()) {
            Some(program) => {
                // In its turn, so the window's git and GitHub passes never run beside it.
                let _turn = process::turn();
                version_of(&program, args, TIMEOUT)
            }
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
pub(crate) fn extension_version(folder: &Path) -> Outcome {
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

/// How a line of the report stands (EP-28).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Ok,
    /// Worth knowing; nothing to do.
    Note,
    /// A part of the tool works less well until it is looked at.
    Warn,
    /// Something the tool cannot work without.
    Fail,
}

impl Level {
    pub fn word(self) -> &'static str {
        match self {
            Level::Ok => "ok",
            Level::Note => "note",
            Level::Warn => "warn",
            Level::Fail => "fail",
        }
    }
}

/// The report's groups, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Tools,
    Exes,
    Accounts,
    Network,
    Backup,
    Records,
    Readings,
    VsCode,
}

impl Group {
    pub const ALL: [Group; 8] = [
        Group::Tools,
        Group::Exes,
        Group::Accounts,
        Group::Network,
        Group::Backup,
        Group::Records,
        Group::Readings,
        Group::VsCode,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Group::Tools => "Tools",
            Group::Exes => "The tool's exes and entries",
            Group::Accounts => "Accounts",
            Group::Network => "Network",
            Group::Backup => "Backup drive",
            Group::Records => "Records",
            Group::Readings => "Last readings",
            Group::VsCode => "VS Code",
        }
    }
}

/// One line of the report: its group, its name, how it stands and what was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub group: Group,
    pub name: String,
    pub level: Level,
    pub text: String,
}

/// The doctor's report (D-29; EP-28), shared by `crw doctor` and the Diagnostics tile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub lines: Vec<Line>,
}

impl Report {
    fn push(&mut self, group: Group, name: &str, level: Level, text: impl Into<String>) {
        self.lines.push(Line {
            group,
            name: name.to_owned(),
            level,
            text: text.into(),
        });
    }

    /// The lines of one group, in their order.
    pub fn group(&self, group: Group) -> impl Iterator<Item = &Line> {
        self.lines.iter().filter(move |line| line.group == group)
    }

    /// The worst level of any line; ok for an empty report.
    pub fn worst(&self) -> Level {
        self.lines
            .iter()
            .map(|line| line.level)
            .max()
            .unwrap_or(Level::Ok)
    }

    /// The warn and fail lines, in their order.
    pub fn problems(&self) -> impl Iterator<Item = &Line> {
        self.lines.iter().filter(|line| line.level >= Level::Warn)
    }

    /// The first line called `name`.
    pub fn line(&self, name: &str) -> Option<&Line> {
        self.lines.iter().find(|line| line.name == name)
    }
}

/// What the children said (EP-28): the tools' versions, `gh auth status` and `tailscale status`.
#[derive(Debug, Clone, Default)]
pub struct Probed {
    pub tools: Vec<Check>,
    /// `None` when `gh` is not found.
    pub gh: Option<Result<actions::Account, String>>,
    /// `None` when Tailscale is not found.
    pub tailscale: Option<Result<Tailnet, String>>,
}

/// Asks the children, one at a time in their turn: the versions ([`run`]), then `gh auth status`
/// and `tailscale status --json` where those tools were found. The `claude` CLI is asked its
/// version only (the preflight is the caller's).
pub fn probe() -> Probed {
    let tools = run();
    let found = |name: &str| {
        tools
            .iter()
            .find(|check| check.name == name)
            .and_then(|check| match &check.outcome {
                Outcome::Found { source, .. } | Outcome::Failed { source, .. } => {
                    Some(source.clone())
                }
                Outcome::Missing => None,
            })
    };
    let gh = found("gh").map(|program| {
        let _turn = process::turn();
        gh_account(&program)
    });
    let tailscale = found("tailscale").map(|program| {
        let _turn = process::turn();
        tailnet(&program)
    });
    Probed {
        tools,
        gh,
        tailscale,
    }
}

/// `gh auth status` for the active account on github.com (EP-34's reader): it exits with an
/// error when not logged in and still prints its JSON.
fn gh_account(program: &Path) -> Result<actions::Account, String> {
    let exit = process::exit_of(
        program,
        actions::AUTH,
        &crate::badges::hosting::GH_ENV,
        TIMEOUT,
    )?;
    actions::parse_auth(&exit.stdout)
}

/// Tailscale's state as `tailscale status --json` gives it: the backend's state and whether this
/// machine is online. Its peers, names and addresses are never kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tailnet {
    pub backend: String,
    pub online: Option<bool>,
}

/// Reads `tailscale status --json`'s `BackendState` and `Self.Online`, and nothing else.
pub fn parse_tailscale(json: &str) -> Result<Tailnet, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("tailscale printed no status: {e}"))?;
    let backend = value["BackendState"]
        .as_str()
        .filter(|state| !state.is_empty())
        .ok_or("tailscale's status names no BackendState")?
        .to_owned();
    Ok(Tailnet {
        backend,
        online: value["Self"]["Online"].as_bool(),
    })
}

fn tailnet(program: &Path) -> Result<Tailnet, String> {
    // It exits with an error while stopped or logged out, and may still print its JSON.
    let exit = process::exit_of(program, ["status", "--json"], &[], TIMEOUT)?;
    parse_tailscale(&exit.stdout).map_err(|why| {
        let said = exit.stderr.split_whitespace().collect::<Vec<_>>().join(" ");
        match exit.code {
            Some(0) => why,
            _ if said.is_empty() => why,
            _ => format!("tailscale status: {said}"),
        }
    })
}

/// The label of the drive whose root is `root` (`X:\`), through `GetVolumeInformationW`; an
/// empty label reads as `None`.
#[cfg(windows)]
pub fn volume_label(root: &Path) -> Result<Option<String>, String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::GetVolumeInformationW;
    use windows::core::PCWSTR;
    let mut wide: Vec<u16> = root.as_os_str().encode_wide().collect();
    if wide.last() != Some(&u16::from(b'\\')) {
        wide.push(u16::from(b'\\'));
    }
    wide.push(0);
    let mut name = [0u16; 261];
    // SAFETY: `wide` is a NUL-terminated wide string that outlives the call, and the name buffer
    // is passed with its length.
    unsafe {
        GetVolumeInformationW(
            PCWSTR(wide.as_ptr()),
            Some(&mut name),
            None,
            None,
            None,
            None,
        )
    }
    .map_err(|e| format!("its label did not read: {}", e.message()))?;
    let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
    let label = String::from_utf16_lossy(&name[..len]).trim().to_owned();
    Ok((!label.is_empty()).then_some(label))
}

#[cfg(not(windows))]
pub fn volume_label(_root: &Path) -> Result<Option<String>, String> {
    Err("only Windows reads a drive's label here".to_owned())
}

/// The crash log's name in the local folder (EP-14a).
pub const CRASH_FILE: &str = "crash.log";
/// A crash report's head line starts so; the version, time, thread and place follow, apart by
/// [`CRASH_SEP`] (EP-14a's form, which `app::crash` writes).
pub const CRASH_HEAD: &str = "== crw ";
pub const CRASH_SEP: &str = " · ";

/// The last report of a crash log, as its head line and its message's first line give it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crash {
    pub at: String,
    pub place: String,
    pub message: String,
}

impl Crash {
    /// Its time in the owner's zone, or as written when it does not read.
    pub fn when(&self) -> String {
        DateTime::parse_from_rfc3339(&self.at).map_or_else(
            |_| self.at.clone(),
            |at| {
                at.with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            },
        )
    }

    /// Its place's file name and line: a library's place is a long registry path.
    pub fn short_place(&self) -> &str {
        self.place.rsplit(['\\', '/']).next().unwrap_or_default()
    }
}

/// The last report in a crash log's text.
pub fn last_crash(log: &str) -> Option<Crash> {
    let start = log.rfind(CRASH_HEAD)?;
    let mut lines = log[start + CRASH_HEAD.len()..].lines();
    let mut head = lines.next()?.split(CRASH_SEP);
    let (_version, at, _thread) = (head.next()?, head.next()?, head.next()?);
    let place = head.collect::<Vec<_>>().join(CRASH_SEP);
    Some(Crash {
        at: at.to_owned(),
        place,
        message: lines.next().unwrap_or_default().trim().to_owned(),
    })
}

/// Where the report reads (EP-28): the owner's places from [`Places::real`], or scratch ones.
#[derive(Default)]
pub struct Places {
    /// The registry's store.
    pub store: Option<Store>,
    pub ledger: Option<PathBuf>,
    /// The local folder: the crash log and the rolling log's folder.
    pub local: Option<PathBuf>,
    /// Claude Code's user settings, which hold the hooks and the status line.
    pub settings: Option<PathBuf>,
    /// The extension's lock files (`~\.claude\ide`).
    pub locks: Option<PathBuf>,
    /// The install folder and the plugin's copy.
    pub bin: Option<PathBuf>,
    pub plugin: Option<PathBuf>,
    /// The exe that asks.
    pub this_exe: Option<PathBuf>,
    /// Where the start-at-login entry lives (EP-28a).
    pub autostart: Option<autostart::Place>,
    /// The folder of `crw`'s copy on PATH (IN-3; EP-28a).
    pub path_dir: Option<PathBuf>,
}

impl Places {
    /// The owner's places, or `CRW_HOME`'s in a trial.
    pub fn real() -> Places {
        Places {
            store: Store::locate(),
            ledger: crate::paths::ledger_file(),
            local: crate::paths::local_dir(),
            settings: entries::settings_file(),
            locks: visual::lock_folder(),
            bin: install::bin_dir(),
            plugin: plugin::dir(),
            this_exe: env::current_exe().ok(),
            autostart: Some(autostart::Place::current()),
            path_dir: install::path_dir(),
        }
    }
}

/// Words every line from what the children said and the files in `places`, at `now`: reads
/// only, nothing is written or deleted.
pub fn report(probed: &Probed, places: &Places, now: DateTime<Utc>) -> Report {
    let mut report = Report::default();
    tools(&mut report, &probed.tools);
    exes(&mut report, places);
    accounts(&mut report, probed.gh.as_ref());
    network(&mut report, probed.tailscale.as_ref());
    let loaded = places.store.as_ref().map(Store::load);
    let registry = loaded
        .as_ref()
        .and_then(|loaded| loaded.as_ref().ok())
        .map(|loaded| &loaded.registry);
    backup_drive(&mut report, registry, now);
    records(&mut report, places, loaded.as_ref(), now);
    readings(&mut report, places.ledger.as_deref(), registry, now);
    vs_code(&mut report, places.locks.as_deref(), registry);
    if report.worst() >= Level::Warn {
        for line in report.problems() {
            log::line(
                "doctor",
                &format!("{} {}: {}", line.level.word(), line.name, line.text),
            );
        }
    }
    report
}

/// The push guard's line's name (EP-42): under Headless in `crw doctor`, beside the preflight,
/// and after the headless route's line among the Diagnostics pane's own.
pub const PUSH_GUARD: &str = "push guard";

/// The push guard as an autopilot run passes it (D-22 addenda of EP-19b and EP-29; EP-42): the
/// launch [`autopilot::launch`] builds for a chained start of a stand-in project whose autopilot
/// is on, its command line built with the CLI's `flags` as the runner builds it, and the deny
/// rules read back from the `--settings` it would pass. It fails when a later change drops the
/// rule unsaid. It shows that the rule is passed, not that Claude Code honours it (risk 17).
pub fn push_guard(flags: &Flags) -> (Level, String) {
    match stand_in(flags) {
        Ok(line) => guard_words(&line),
        Err(why) => (
            Level::Warn,
            format!("an autopilot run's launch did not build with this claude: {why}"),
        ),
    }
}

/// The command line of the stand-in's chained start: no folder of the owner's is named, and the
/// effort is left to the CLI, so that a level its help does not list refuses nothing here.
fn stand_in(flags: &Flags) -> Result<CommandLine, String> {
    let folder = env::temp_dir();
    let mut project = Project::new(PUSH_GUARD, &folder, State::Active);
    project.autopilot.enabled = true;
    let registry = Registry {
        format: registry_file::FORMAT,
        ignore: Vec::new(),
        settings: Settings::default(),
        rules: Vec::new(),
        projects: vec![project.clone()],
    };
    let brief = rules::Brief {
        size: Some(Size::M),
        phase: None,
        title: "the doctor's stand-in",
    };
    let usage = rules::Usage::unread(Utc::now());
    let mut resolution = rules::resolve(&registry, &project, &brief, &[], &usage);
    resolution.effort.value = None;
    resolution.ultracode.value = false;
    let draft = Draft {
        ep: EpId::new(1),
        title: brief.title.to_owned(),
        size: brief.size,
        phase: None,
        resolution,
        parts: Parts::default(),
        prompt: Prompt::default(),
    };
    let launch = autopilot::launch(&draft, &project, &folder, &folder, None);
    command_line(
        &launch,
        flags,
        "00000000-0000-4000-8000-000000000000",
        std::iter::empty(),
    )
}

/// The push guard's line from a built command line: the rule among the `--settings` deny rules,
/// dropped and said by a claude without settings, or missing unsaid.
fn guard_words(line: &CommandLine) -> (Level, String) {
    let denied: Vec<String> = line
        .args
        .iter()
        .position(|arg| arg.as_os_str() == "--settings")
        .and_then(|at| line.args.get(at + 1))
        .and_then(|value| serde_json::from_str::<serde_json::Value>(&value.to_string_lossy()).ok())
        .and_then(|value| value["permissions"]["deny"].as_array().cloned())
        .map(|rules| {
            rules
                .iter()
                .filter_map(|rule| rule.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    if denied.iter().any(|rule| rule == PUSH_DENY_RULE) {
        return (
            Level::Ok,
            format!(
                "an autopilot run is passed the deny rule {PUSH_DENY_RULE} in its --settings; \
                 whether Claude Code refuses a push by it is not checked here"
            ),
        );
    }
    if let Some(dropped) = line.dropped.iter().find(|why| why.contains("git push")) {
        return (
            Level::Warn,
            format!("an autopilot run carries no push deny rule: {dropped}"),
        );
    }
    (
        Level::Fail,
        format!(
            "an autopilot run's launch passes no {PUSH_DENY_RULE} and says nothing of it: a \
             change dropped the push guard (D-22)"
        ),
    )
}

fn tools(report: &mut Report, checks: &[Check]) {
    let cli = checks
        .iter()
        .find(|check| check.name == "claude")
        .and_then(Check::version);
    for check in checks {
        let (level, text) = match (&check.outcome, check.name) {
            (Outcome::Found { version, source }, "claude") if *version < MIN_CLAUDE_CODE => (
                Level::Fail,
                format!(
                    "{version} at {}, older than the minimum {MIN_CLAUDE_CODE}: headless runs are \
                     refused",
                    source.display()
                ),
            ),
            (Outcome::Found { version, source }, "claude") => (
                Level::Ok,
                format!(
                    "{version} at {} (the minimum is {MIN_CLAUDE_CODE})",
                    source.display()
                ),
            ),
            (Outcome::Found { version, source }, "extension") => (
                Level::Ok,
                match cli {
                    Some(cli) if cli != *version => format!(
                        "{version} in {}; the standalone CLI is {cli}",
                        source.display()
                    ),
                    _ => format!("{version} in {}", source.display()),
                },
            ),
            (Outcome::Found { version, source }, _) => {
                (Level::Ok, format!("{version} at {}", source.display()))
            }
            (Outcome::Missing, name) => missing(name),
            (Outcome::Failed { source, reason }, name) => (
                if name == "claude" {
                    Level::Fail
                } else {
                    Level::Warn
                },
                format!("{reason}: {}", source.display()),
            ),
        };
        report.push(Group::Tools, check.name, level, text);
    }
    report.push(
        Group::Tools,
        "protocol",
        Level::Ok,
        format!("claude-codes {PROTOCOL_PIN}, pinned: it moves only in a re-plan (D-15)"),
    );
}

/// A tool not found, and what it costs.
fn missing(name: &str) -> (Level, String) {
    let (level, words) = match name {
        "claude" => (
            Level::Fail,
            "not found: headless runs are refused and the poll of live sessions has no CLI",
        ),
        "gh" => (
            Level::Warn,
            "not found: the hosting badges and actions cannot ask GitHub",
        ),
        "tailscale" => (Level::Note, "not found"),
        "code" => (
            Level::Note,
            "not found: Visual launches look for Code.exe by its install (VS Code, below)",
        ),
        "extension" => (
            Level::Warn,
            "not found in VS Code's extensions: Visual launches need it",
        ),
        _ => (Level::Warn, "not found"),
    };
    (level, words.to_owned())
}

/// A `crw.exe` an entry runs, in words: the installed copy, this exe, or its path.
fn exe_words(theirs: &Path, this: Option<&Path>, installed: Option<&Path>) -> String {
    if installed.is_some_and(|installed| crate::paths::same(theirs, installed)) {
        "the installed copy".to_owned()
    } else if this.is_some_and(|this| crate::paths::same(theirs, this)) {
        "this exe".to_owned()
    } else {
        theirs.display().to_string()
    }
}

/// An entry's line: the exe it runs gone (security software's quarantine, or a move), another
/// exe than the installed copy, or as it should be.
fn entry(exe: &Path, this: Option<&Path>, installed: Option<&Path>, run: &str) -> (Level, String) {
    // A bare name (`crw hook`, written by hand) runs what PATH finds, not a file here.
    let bare = exe
        .parent()
        .is_none_or(|parent| parent.as_os_str().is_empty());
    if bare {
        let name = if exe.extension().is_none() {
            exe.with_extension("exe")
        } else {
            exe.to_owned()
        };
        return match name.to_str().and_then(on_path) {
            Some(found) => (
                Level::Note,
                format!(
                    "runs {} {run}, which PATH finds as {}: crw signals install writes the \
                     installed copy's full path",
                    exe.display(),
                    found.display()
                ),
            ),
            None => (
                Level::Warn,
                format!(
                    "runs {} {run}, which PATH does not find: crw signals install writes the \
                     installed copy's full path",
                    exe.display()
                ),
            ),
        };
    }
    if !exe.is_file() {
        return (
            Level::Fail,
            format!(
                "runs {} {run}, which is not there: {}; crw signals install points the entries \
                 at the installed copy",
                exe.display(),
                process::GONE
            ),
        );
    }
    let words = exe_words(exe, this, installed);
    match installed {
        Some(installed) if !crate::paths::same(exe, installed) => (
            Level::Note,
            format!("runs {words} {run}: crw signals install points it at the installed copy"),
        ),
        _ => (Level::Ok, format!("runs {words} {run}")),
    }
}

fn exes(report: &mut Report, places: &Places) {
    let this = places.this_exe.as_deref();
    report.push(
        Group::Exes,
        "this exe",
        Level::Ok,
        this.map_or_else(|| "?".to_owned(), |this| this.display().to_string()),
    );
    let installed = places
        .bin
        .as_ref()
        .map(|bin| bin.join(install::CLI_EXE))
        .filter(|path| path.is_file());
    let (level, text) = match &places.bin {
        Some(bin) => match install::Stamp::of(&bin.join(install::CLI_EXE)) {
            Some(stamp) => (
                Level::Ok,
                format!("{}, built {}", bin.display(), stamp.when()),
            ),
            None => (
                Level::Note,
                format!(
                    "none in {}: crw install copies the release build there",
                    bin.display()
                ),
            ),
        },
        None => (Level::Warn, "no local folder".to_owned()),
    };
    report.push(Group::Exes, "installed", level, text);
    let (level, text) = match &places.plugin {
        Some(dir) => {
            let status = plugin::Status::of(dir);
            let level = if matches!(status, plugin::Status::Matches) {
                Level::Ok
            } else {
                Level::Note
            };
            (level, format!("{}: {}", dir.display(), status.words()))
        }
        None => (Level::Warn, "no local folder".to_owned()),
    };
    report.push(Group::Exes, "plugin", level, text);
    if let Some(place) = &places.autostart {
        let window = places.bin.as_ref().map(|bin| bin.join(install::WINDOW_EXE));
        let (level, text) = autostart_line(autostart::status(place, window.as_deref()));
        report.push(Group::Exes, "start at login", level, text);
    }
    if let Some(dir) = &places.path_dir {
        let (level, text) = crw_on_path(places.bin.as_deref(), dir);
        report.push(Group::Exes, "crw on PATH", level, text);
    }
    let Some(settings) = &places.settings else {
        report.push(
            Group::Exes,
            "hooks",
            Level::Warn,
            "no profile folder: Claude Code's user settings are not found",
        );
        return;
    };
    let status = match entries::status(settings) {
        Ok(status) => status,
        Err(e) => {
            report.push(
                Group::Exes,
                "hooks",
                Level::Warn,
                format!("{}: {e}", settings.display()),
            );
            return;
        }
    };
    for (event, hooked) in &status.hooks {
        let (level, text) = match hooked {
            Hooked::Missing => (
                Level::Warn,
                "not installed (crw signals install)".to_owned(),
            ),
            Hooked::Ours { exe, inbox } => {
                let run = if *inbox { "hook --inbox" } else { "hook" };
                entry(exe, this, installed.as_deref(), run)
            }
            Hooked::Several => (
                Level::Warn,
                "more than one entry of the tool's: an install puts one back".to_owned(),
            ),
        };
        report.push(Group::Exes, event, level, text);
    }
    let (level, text) = match &status.line {
        StatusLine::Missing => (Level::Note, "none (crw signals install adds it)".to_owned()),
        StatusLine::Ours(exe) => entry(exe, this, installed.as_deref(), "statusline"),
        StatusLine::Other(command) => (
            Level::Note,
            format!("another command, which an install leaves: {command}"),
        ),
    };
    report.push(Group::Exes, "status line", level, text);
}

/// The start-at-login line (D-29; EP-28a): not registered, starting the installed window, or
/// starting another exe or one that is gone.
fn autostart_line(status: Result<autostart::Status, String>) -> (Level, String) {
    match status {
        Err(e) => (Level::Warn, e),
        Ok(autostart::Status::Absent) => (
            Level::Note,
            "not registered: crw autostart on (or File > Start at login in the window) starts the \
             installed window hidden in the tray at login"
                .to_owned(),
        ),
        Ok(autostart::Status::Starts {
            exe: None, command, ..
        }) => (
            Level::Warn,
            format!("its command names no program ({command}): crw autostart on writes it again"),
        ),
        Ok(autostart::Status::Starts {
            installed: true,
            present: true,
            ..
        }) => (
            Level::Ok,
            "starts the installed window hidden in the tray".to_owned(),
        ),
        // The installed window itself gone: the entry is right, the install is not.
        Ok(autostart::Status::Starts {
            exe: Some(exe),
            installed: true,
            present: false,
            ..
        }) => (
            Level::Warn,
            format!(
                "starts the installed window {}, which is not there: {}; install it again with \
                 pwsh -File scripts\\install-shortcut.ps1",
                exe.display(),
                process::GONE
            ),
        ),
        Ok(autostart::Status::Starts {
            exe: Some(exe),
            present: false,
            ..
        }) => (
            Level::Warn,
            format!(
                "starts {}, which is not there: {}; crw autostart on points it at the installed \
                 window",
                exe.display(),
                process::GONE
            ),
        ),
        Ok(autostart::Status::Starts { exe: Some(exe), .. }) => (
            Level::Warn,
            format!(
                "starts {}, not the installed window: crw autostart on points it there",
                exe.display()
            ),
        ),
    }
}

/// The "crw on PATH" line (IN-3; EP-28a): what PATH finds for `crw.exe`, set against the
/// installed copy in `bin` and the copy in `dir`, the PATH folder `crw path on` puts it in.
pub fn crw_on_path(bin: Option<&Path>, dir: &Path) -> (Level, String) {
    path_line(on_path(install::CLI_EXE).as_deref(), bin, dir)
}

/// The "crw on PATH" line for what PATH found.
fn path_line(found: Option<&Path>, bin: Option<&Path>, dir: &Path) -> (Level, String) {
    let Some(found) = found else {
        let copy = dir.join(install::CLI_EXE);
        let text = if copy.is_file() {
            format!(
                "not on PATH: {} is there, but its folder is not on PATH",
                copy.display()
            )
        } else {
            format!("not on PATH: crw path on puts a copy in {}", dir.display())
        };
        return (Level::Note, text);
    };
    let installed = bin.map(|bin| bin.join(install::CLI_EXE));
    if installed
        .as_deref()
        .is_some_and(|installed| crate::paths::same(found, installed))
    {
        return (Level::Ok, "PATH finds the installed copy".to_owned());
    }
    if let Some(bin) = bin
        && crate::paths::same(found, &dir.join(install::CLI_EXE))
    {
        match install::path_copy(bin, dir) {
            install::PathCopy::AsInstalled(_) => {
                return (
                    Level::Ok,
                    format!(
                        "PATH finds {}, a copy of the installed build",
                        found.display()
                    ),
                );
            }
            install::PathCopy::Differs(_) => {
                return (
                    Level::Warn,
                    format!(
                        "PATH finds {}, a copy older or newer than the installed build: crw \
                         install refreshes it",
                        found.display()
                    ),
                );
            }
            install::PathCopy::Foreign(_) => {
                return (
                    Level::Note,
                    format!(
                        "PATH finds {}, another program's crw.exe, which the tool leaves alone",
                        found.display()
                    ),
                );
            }
            install::PathCopy::Absent => {}
        }
    }
    (
        Level::Note,
        format!("PATH finds {}, not the installed copy", found.display()),
    )
}

fn accounts(report: &mut Report, gh: Option<&Result<actions::Account, String>>) {
    let (level, text) = match gh {
        None => (Level::Note, "gh not found".to_owned()),
        Some(Err(e)) => (Level::Warn, e.clone()),
        Some(Ok(account)) if account.scopes.iter().any(|s| s == actions::DELETE_SCOPE) => (
            Level::Ok,
            format!(
                "logged in to github.com as {}; the token may delete a repository",
                account.login
            ),
        ),
        Some(Ok(account)) => (
            Level::Ok,
            format!(
                "logged in to github.com as {}; the token lacks {}, so an orphan's Delete is \
                 refused until {}",
                account.login,
                actions::DELETE_SCOPE,
                actions::ADD_DELETE_SCOPE
            ),
        ),
    };
    report.push(Group::Accounts, "gh", level, text);
}

fn network(report: &mut Report, tailnet: Option<&Result<Tailnet, String>>) {
    let (level, text) = match tailnet {
        None => (Level::Note, "Tailscale not found".to_owned()),
        Some(Err(e)) => (Level::Warn, e.clone()),
        Some(Ok(state)) if state.backend == "Running" => match state.online {
            Some(false) => (
                Level::Warn,
                "running, but this machine is offline in the tailnet".to_owned(),
            ),
            Some(true) => (
                Level::Ok,
                "running; this machine is online in the tailnet".to_owned(),
            ),
            None => (Level::Ok, "running".to_owned()),
        },
        Some(Ok(state)) => (
            Level::Warn,
            format!("{}: the tailnet is not up", state.backend),
        ),
    };
    report.push(Group::Network, "tailscale", level, text);
}

/// The backup drive's line from what was found: mounted or not, and its label against the job's
/// `VolumeLabel`.
pub fn drive_line(
    root: &Path,
    mounted: bool,
    expected: Option<&str>,
    found: Result<Option<String>, String>,
) -> (Level, String) {
    let root_words = root.display();
    if !mounted {
        return (
            Level::Warn,
            format!("{root_words} is not mounted: the job skips its runs until it is"),
        );
    }
    match (expected, found) {
        (None, _) => (
            Level::Ok,
            format!("{root_words} is mounted (the job names no label)"),
        ),
        (Some(_), Err(why)) => (Level::Warn, format!("{root_words} is mounted, but {why}")),
        (Some(expected), Ok(Some(label))) if label.eq_ignore_ascii_case(expected) => (
            Level::Ok,
            format!("{root_words} is mounted, labelled {label} as the job expects"),
        ),
        (Some(expected), Ok(Some(label))) => (
            Level::Warn,
            format!(
                "{root_words} is labelled {label}, but the job expects {expected}: another drive \
                 may be mounted there"
            ),
        ),
        (Some(expected), Ok(None)) => (
            Level::Warn,
            format!("{root_words} has no label, but the job expects {expected}"),
        ),
    }
}

fn backup_drive(report: &mut Report, registry: Option<&Registry>, now: DateTime<Utc>) {
    let Some(registry) = registry else {
        report.push(
            Group::Backup,
            "drive",
            Level::Note,
            "the registry did not read",
        );
        return;
    };
    let (settings, log) = registry.backup_files();
    let Some(settings) = settings else {
        report.push(
            Group::Backup,
            "drive",
            Level::Note,
            "no job set (crw setting backup_settings <its settings file>)",
        );
        return;
    };
    let reading = backup::Reading::read(&settings, log.as_deref());
    let job = match &reading.job {
        Ok(job) => job,
        Err(e) => {
            report.push(
                Group::Backup,
                "drive",
                Level::Warn,
                format!(
                    "the job's settings did not read ({}): {e}",
                    settings.display()
                ),
            );
            return;
        }
    };
    let Some(root) = backup::drive_root(&job.target) else {
        report.push(
            Group::Backup,
            "drive",
            Level::Note,
            format!(
                "the job's target {} names no drive to check",
                job.target.display()
            ),
        );
        return;
    };
    let found = if reading.mounted {
        volume_label(&root)
    } else {
        Ok(None)
    };
    let (level, text) = drive_line(&root, reading.mounted, job.label.as_deref(), found);
    report.push(Group::Backup, "drive", level, text);
    let (level, text) = match reading.last_run() {
        Some(run) if run.status != "ok" => (
            Level::Warn,
            format!(
                "{}, {}: the job's log names a problem",
                run.status,
                ago(now - run.at)
            ),
        ),
        Some(run) if now - run.at > backup::STALE_AFTER => (
            Level::Warn,
            format!(
                "ok, {}: older than {} h, so the job may not be running",
                ago(now - run.at),
                backup::STALE_AFTER.num_hours()
            ),
        ),
        Some(run) => (Level::Ok, format!("ok, {}", ago(now - run.at))),
        None => (Level::Note, "no run in the job's log".to_owned()),
    };
    report.push(Group::Backup, "last run", level, text);
}

/// An age in words: "just now", "5 min ago", "3 h ago", "2 days ago".
fn ago(age: TimeDelta) -> String {
    match short_age(age).as_str() {
        "now" => "just now".to_owned(),
        age => format!("{age} ago"),
    }
}

fn records(
    report: &mut Report,
    places: &Places,
    loaded: Option<&Result<crate::registry::Loaded, crate::registry::Error>>,
    now: DateTime<Utc>,
) {
    let path = places
        .store
        .as_ref()
        .map_or_else(|| "?".to_owned(), |store| store.path.display().to_string());
    let (level, text) = match loaded {
        None => (Level::Fail, "no profile folder".to_owned()),
        Some(Err(e)) => (Level::Fail, format!("{path}: {e}")),
        Some(Ok(loaded)) if !loaded.written => (
            Level::Note,
            format!("{path}: not written yet, the default reads"),
        ),
        Some(Ok(loaded)) => (
            Level::Ok,
            format!("{path}: {} projects", loaded.registry.projects.len()),
        ),
    };
    report.push(Group::Records, "registry", level, text);
    let (level, text) = match &places.ledger {
        None => (Level::Fail, "no local folder".to_owned()),
        Some(path) if !path.exists() => (
            Level::Note,
            format!(
                "none yet at {}: the window makes it at its first start",
                path.display()
            ),
        ),
        Some(path) => ledger_line(path),
    };
    report.push(Group::Records, "ledger", level, text);
    let crash_file = places.local.as_ref().map(|dir| dir.join(CRASH_FILE));
    let (level, text) = match crash_file
        .as_ref()
        .and_then(|file| fs::read_to_string(file).ok())
        .as_deref()
        .and_then(last_crash)
    {
        None => (Level::Ok, "no report".to_owned()),
        Some(crash) => (
            Level::Note,
            format!(
                "the last report: {}, {} ({}); {}",
                crash.when(),
                crash.message,
                crash.short_place(),
                crash_file
                    .as_ref()
                    .map_or_else(String::new, |file| file.display().to_string())
            ),
        ),
    };
    report.push(Group::Records, "crash log", level, text);
    let (level, text) = match &places.local {
        None => (Level::Warn, "no local folder".to_owned()),
        Some(local) => {
            let dir = local.join(log::FOLDER);
            let files = log::files(&dir);
            match files.first() {
                None => (
                    Level::Note,
                    format!(
                        "{}: no file yet, the window writes one from its start",
                        dir.display()
                    ),
                ),
                Some((day, _, _)) => {
                    let bytes: u64 = files.iter().map(|(_, _, size)| size).sum();
                    let today = now.with_timezone(&Local).date_naive();
                    let latest = if *day == today {
                        "today's".to_owned()
                    } else {
                        format!("the latest of {day}")
                    };
                    (
                        Level::Ok,
                        format!(
                            "{}: {} files of the last {} days, {} KB; {latest} is {}",
                            dir.display(),
                            files.len(),
                            log::KEEP_DAYS,
                            bytes.div_ceil(1024),
                            log::file_name(*day)
                        ),
                    )
                }
            }
        }
    };
    report.push(Group::Records, "log", level, text);
}

/// The ledger read only: its schema against the build's, and SQLite's quick check.
fn ledger_line(path: &Path) -> (Level, String) {
    let size = fs::metadata(path).map_or(0, |meta| meta.len());
    let read = Ledger::open_read_only(path)
        .and_then(|ledger| Ok((ledger.schema_version()?, ledger.check()?)));
    match read {
        Err(e) => (Level::Fail, format!("{}: {e}", path.display())),
        Ok((_, problems)) if !problems.is_empty() => (
            Level::Fail,
            format!(
                "{}: SQLite's check found {} problem(s), the first: {}",
                path.display(),
                problems.len(),
                problems[0]
            ),
        ),
        Ok((found, _)) if found < ledger::SCHEMA_VERSION => (
            Level::Note,
            format!(
                "{}: schema {found}, this build's is {}: the window brings it up to date at its \
                 next start",
                path.display(),
                ledger::SCHEMA_VERSION
            ),
        ),
        Ok((found, _)) => (
            Level::Ok,
            format!(
                "{}: schema {found}, sound by SQLite's quick check, {} KB",
                path.display(),
                size.div_ceil(1024)
            ),
        ),
    }
}

/// How the notifications' log starts a push's row (`notify::line` with `Fate::Pushed`) and a
/// failed push's (`Fate::PushFailed`); a test holds them to `notify`'s words.
const PUSHED_ROW: &str = "ntfy · ";
const PUSH_FAILED_ROW: &str = "ntfy failed · ";

fn readings(
    report: &mut Report,
    ledger: Option<&Path>,
    registry: Option<&Registry>,
    now: DateTime<Utc>,
) {
    let opened = ledger
        .filter(|path| path.exists())
        .map(Ledger::open_read_only);
    let ledger = match opened {
        Some(Ok(ledger)) => ledger,
        Some(Err(e)) => {
            report.push(
                Group::Readings,
                "ledger",
                Level::Warn,
                format!("did not read: {e}"),
            );
            return;
        }
        None => {
            report.push(Group::Readings, "ledger", Level::Note, "no ledger yet");
            return;
        }
    };
    let age = |at: DateTime<Utc>| ago(now - at);
    let (level, text) = match ledger.cached_one("", usage::CACHE_SOURCE) {
        Ok(Some(row)) if now - row.fetched_at > TimeDelta::minutes(15) => (
            Level::Note,
            format!(
                "read {}: the window polls every 5 minutes while it runs",
                age(row.fetched_at)
            ),
        ),
        Ok(Some(row)) => (Level::Ok, format!("read {}", age(row.fetched_at))),
        Ok(None) => (Level::Note, "never read".to_owned()),
        Err(e) => (Level::Warn, format!("did not read: {e}")),
    };
    report.push(Group::Readings, "usage", level, text);
    let (level, text) = match ledger.cached_one("", online::ACCOUNT) {
        Ok(Some(row)) if now - row.fetched_at > online::FRESH_FOR => (
            Level::Note,
            format!(
                "the account's list read {}: the window reads it again daily and at F5",
                age(row.fetched_at)
            ),
        ),
        Ok(Some(row)) => (
            Level::Ok,
            format!("the account's list read {}", age(row.fetched_at)),
        ),
        Ok(None) => (Level::Note, "the account's list never read".to_owned()),
        Err(e) => (Level::Warn, format!("did not read: {e}")),
    };
    report.push(Group::Readings, "GitHub", level, text);
    let (level, text) = match ledger.events(mailman::EVENT_KIND, Some(1)) {
        Ok(rows) => match rows.first() {
            Some(row) => (
                Level::Ok,
                format!(
                    "the last row {}: {}",
                    row.at().map_or_else(|| row.at.clone(), age),
                    row.detail.as_deref().unwrap_or_default()
                ),
            ),
            None => (Level::Note, "no row yet".to_owned()),
        },
        Err(e) => (Level::Warn, format!("did not read: {e}")),
    };
    report.push(Group::Readings, "mailman", level, text);
    // The newest push's row, gone out (`ntfy · …`) or failed (`ntfy failed · … · <why>`).
    let pushed = ledger.events(notify::EVENT_KIND, Some(500)).map(|rows| {
        rows.into_iter().find(|row| {
            row.detail.as_deref().is_some_and(|detail| {
                detail.starts_with(PUSHED_ROW) || detail.starts_with(PUSH_FAILED_ROW)
            })
        })
    });
    let settings = registry.map(|registry| &registry.settings);
    let host = settings.and_then(|settings| settings.ntfy_host.as_deref());
    let topic = settings.is_some_and(|settings| settings.ntfy_topic.is_some());
    let (level, text) = match (host, topic, pushed) {
        (None, _, _) | (_, false, _) => (
            Level::Note,
            "no ntfy server or topic set: nothing is pushed".to_owned(),
        ),
        (Some(host), true, Ok(Some(row))) => {
            let when = row.at().map_or_else(|| row.at.clone(), age);
            let detail = row.detail.as_deref().unwrap_or_default();
            if detail.starts_with(PUSH_FAILED_ROW) {
                let why = detail.rsplit(" · ").next().unwrap_or_default();
                (
                    Level::Warn,
                    format!("to {host}, a topic set; the last push failed {when}: {why}"),
                )
            } else {
                (
                    Level::Ok,
                    format!("to {host}, a topic set; the last push went {when}"),
                )
            }
        }
        (Some(host), true, Ok(None)) => (
            Level::Note,
            format!("to {host}, a topic set; no push in the latest notifications"),
        ),
        (Some(_), true, Err(e)) => (Level::Warn, format!("did not read: {e}")),
    };
    report.push(Group::Readings, "ntfy", level, text);
    let (level, text) = match ledger.latest_samples(50) {
        Ok(samples) => match samples.iter().find(|sample| sample.source == "statusline") {
            Some(sample) => (
                Level::Ok,
                format!("the last status line's figures {}", age(sample.taken_at)),
            ),
            None => (
                Level::Note,
                "no status line's figures among the latest samples (the VS Code extension shows \
                 no status line)"
                    .to_owned(),
            ),
        },
        Err(e) => (Level::Warn, format!("did not read: {e}")),
    };
    report.push(Group::Readings, "status line", level, text);
}

fn vs_code(report: &mut Report, locks: Option<&Path>, registry: Option<&Registry>) {
    let setting = registry.and_then(|registry| registry.settings.vscode.as_deref());
    let (level, text) = match visual::locate(setting) {
        // The registry's setting is taken as written: it may name a file that is gone.
        Some(code) if !code.is_file() => (
            Level::Warn,
            format!(
                "Visual launches would open {}, which is not there: {}",
                code.display(),
                process::GONE
            ),
        ),
        Some(code) => (
            Level::Ok,
            format!("Visual launches open {}", code.display()),
        ),
        None => (
            Level::Warn,
            "Code.exe not found: Visual launches are refused (crw setting vscode <Code.exe>)"
                .to_owned(),
        ),
    };
    report.push(Group::VsCode, "Code.exe", level, text);
    let (level, text) = match locks {
        None => (Level::Note, "no profile folder".to_owned()),
        Some(dir) if visual::windows(dir).is_empty() => {
            (Level::Ok, format!("no lock file in {}", dir.display()))
        }
        Some(dir) => {
            let windows = visual::windows(dir);
            let live = windows.iter().filter(|window| window.alive).count();
            let stale = windows.len() - live;
            let level = if stale > 0 { Level::Note } else { Level::Ok };
            let left = if stale > 0 {
                "; the extension leaves a stale one when VS Code is killed, and the tool never \
                 deletes them"
            } else {
                ""
            };
            (
                level,
                format!(
                    "{} in {}: {live} live, {stale} stale{left}",
                    match windows.len() {
                        1 => "1 lock file".to_owned(),
                        n => format!("{n} lock files"),
                    },
                    dir.display()
                ),
            )
        }
    };
    report.push(Group::VsCode, "lock files", level, text);
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

    /// The push guard's line (EP-42): built through the autopilot's own launch, ok with a help
    /// that lists `--settings`, a warning that quotes the runner's words without it, and a fail
    /// when a command line passes no rule and says nothing.
    #[test]
    fn the_push_guard_is_read_back_from_the_autopilots_launch() {
        use crate::session::launch::tests::HELP;
        let (level, text) = push_guard(&Flags::parse(HELP));
        assert_eq!(level, Level::Ok, "{text}");
        assert!(text.contains("Bash(git push *)") && text.contains("not checked here"));
        let without = Flags::parse(&HELP.replace("--settings", "--other"));
        let (level, text) = push_guard(&without);
        assert_eq!(level, Level::Warn, "{text}");
        assert!(text.contains("this claude has no --settings"), "{text}");
        // A claude that lacks a flag every run needs: the launch does not build, said.
        let bare = Flags::parse(&HELP.replace("--permission-prompts", "--other-prompts"));
        let (level, text) = push_guard(&bare);
        assert_eq!(level, Level::Warn, "{text}");
        assert!(text.contains("did not build"), "{text}");
        // A command line with neither the rule nor a word on it: a change dropped it.
        let line = stand_in(&Flags::parse(HELP)).unwrap();
        let dropped = CommandLine {
            args: line
                .args
                .iter()
                .filter(|arg| !arg.to_string_lossy().contains("git push"))
                .cloned()
                .collect(),
            ..line
        };
        let (level, text) = guard_words(&dropped);
        assert_eq!(level, Level::Fail, "{text}");
        assert!(text.contains("dropped the push guard"), "{text}");
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

    /// Tailscale's status gives its state and this machine's; nothing of its peers is kept.
    #[test]
    fn reads_tailscales_state_and_nothing_of_its_peers() {
        let json = r#"{"Version":"1.0.0","BackendState":"Running",
            "Self":{"HostName":"desk","DNSName":"desk.example.ts.net.","Online":true},
            "Peer":{"nodekey:1":{"HostName":"phone","Online":false}}}"#;
        assert_eq!(
            parse_tailscale(json).unwrap(),
            Tailnet {
                backend: "Running".to_owned(),
                online: Some(true),
            }
        );
        let stopped = parse_tailscale(r#"{"BackendState":"Stopped","Self":{}}"#).unwrap();
        assert_eq!(stopped.online, None);
        assert!(parse_tailscale("{}").is_err());
        assert!(parse_tailscale("not json").is_err());
    }

    #[test]
    fn the_backup_drive_is_set_against_the_jobs_label() {
        let root = Path::new(r"X:\");
        let level = |mounted, expected, found| drive_line(root, mounted, expected, found).0;
        assert_eq!(level(false, Some("SPARE"), Ok(None)), Level::Warn);
        assert_eq!(level(true, None, Ok(None)), Level::Ok);
        assert_eq!(
            level(true, Some("SPARE"), Ok(Some("spare".into()))),
            Level::Ok
        );
        let (other, words) = drive_line(root, true, Some("SPARE"), Ok(Some("OTHER".into())));
        assert_eq!(other, Level::Warn);
        assert!(
            words.contains("labelled OTHER, but the job expects SPARE"),
            "{words}"
        );
        assert_eq!(level(true, Some("SPARE"), Ok(None)), Level::Warn);
        assert_eq!(level(true, Some("SPARE"), Err("no".into())), Level::Warn);
    }

    /// The system drive's label reads (whatever it is): the call itself works.
    #[cfg(windows)]
    #[test]
    fn a_drives_label_reads() {
        let system = env::var("SystemDrive").unwrap_or_else(|_| "C:".to_owned());
        assert!(volume_label(Path::new(&format!("{system}\\"))).is_ok());
    }

    #[test]
    fn the_protocol_pin_is_the_locked_version() {
        let lock = fs::read_to_string(crate::paths::repository().join("Cargo.lock")).unwrap();
        let at = lock.find("name = \"claude-codes\"").unwrap();
        let version = lock[at..].lines().nth(1).unwrap();
        assert_eq!(version, format!("version = \"{PROTOCOL_PIN}\""));
    }

    /// An entry written with a bare name runs what PATH finds: never called gone.
    #[cfg(windows)]
    #[test]
    fn a_bare_entry_is_looked_up_on_path() {
        let (level, words) = entry(Path::new("cmd"), None, None, "hook");
        assert_eq!(level, Level::Note, "{words}");
        assert!(words.contains("which PATH finds as"), "{words}");
        let absent = format!("crw-absent-{}", std::process::id());
        let (level, words) = entry(Path::new(&absent), None, None, "hook");
        assert_eq!(level, Level::Warn, "{words}");
        assert!(!words.contains(process::GONE), "{words}");
    }

    #[test]
    fn a_push_and_a_failed_push_are_told_apart_in_the_log() {
        let pushed = notify::line(&notify::Fate::Pushed, "waiting", "alpha waits", "");
        let failed = notify::line(&notify::Fate::PushFailed, "waiting", "alpha", "HTTP 403");
        assert!(pushed.starts_with(PUSHED_ROW), "{pushed}");
        assert!(failed.starts_with(PUSH_FAILED_ROW), "{failed}");
        assert!(!failed.starts_with(PUSHED_ROW));
    }

    #[test]
    fn reads_the_last_crash_report() {
        let log = "== crw 0.1.0 · 2026-11-03T00:34:00Z · thread 'main' · src\\a.rs:1:2\nfirst\nbacktrace:\n\n\
                   == crw 0.1.0 · 2026-11-03T00:35:00Z · thread 'main' · C:\\reg\\src\\b.rs:3:4\nsecond one\nbacktrace:\n";
        let crash = last_crash(log).unwrap();
        assert_eq!(crash.at, "2026-11-03T00:35:00Z");
        assert_eq!(crash.message, "second one");
        assert_eq!(crash.short_place(), "b.rs:3:4");
        assert_eq!(last_crash("nothing"), None);
    }

    /// The report over scratch places and what synthetic children said: each group's lines,
    /// read without a write (EP-28).
    #[test]
    fn the_report_words_each_group_from_its_places() {
        use crate::registry::Defaults;
        let scratch = Scratch::new("report");
        let root = &scratch.0;
        let local = root.join("local");
        let ledger = local.join("ledger.sqlite");
        drop(Ledger::open(&ledger).unwrap());
        fs::write(
            local.join(CRASH_FILE),
            "== crw 0.1.0 · 2026-11-03T00:35:00Z · thread 'main' · src\\b.rs:3:4\nboom\n",
        )
        .unwrap();
        let mut log = log::Log::new(local.join(log::FOLDER));
        log.write(Local::now(), "start", "the window").unwrap();
        let bin = root.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let installed = bin.join(install::CLI_EXE);
        fs::write(&installed, "").unwrap();
        let gone = root.join("old").join(install::CLI_EXE);
        let settings = root.join("settings.json");
        let handler = |exe: &Path, args: &[&str]| serde_json::json!({"hooks": [{"type": "command", "command": exe, "args": args}]});
        let json = serde_json::json!({
            "hooks": {
                "SessionStart": [handler(&gone, &["hook", "--inbox"])],
                "Stop": [handler(&installed, &["hook"])],
            },
            "statusLine": {"type": "command", "command": entries::status_command(&installed)},
        });
        fs::write(&settings, json.to_string()).unwrap();
        let locks = root.join("ide");
        fs::create_dir_all(&locks).unwrap();
        let lock = |name: &str, pid: u32| {
            let json = format!(r#"{{"pid":{pid},"workspaceFolders":[],"ideName":"VS Code"}}"#);
            fs::write(locks.join(name), json).unwrap();
        };
        lock("1.lock", std::process::id());
        lock("2.lock", u32::MAX - 3);
        let places = Places {
            store: Some(Store::new(
                root.join("registry.toml"),
                root.join("backups"),
                Defaults {
                    data_root: root.join("data"),
                    repository: root.join("data").join("tool"),
                },
            )),
            ledger: Some(ledger),
            local: Some(local),
            settings: Some(settings),
            locks: Some(locks),
            bin: Some(bin),
            plugin: Some(root.join("plugin")),
            this_exe: Some(root.join("target").join(install::CLI_EXE)),
            autostart: Some(autostart::Place::File(root.join(autostart::SCRATCH_FILE))),
            path_dir: Some(root.join("path")),
        };
        let found = |name, version| Check {
            name,
            outcome: Outcome::Found {
                version,
                source: PathBuf::from(format!("{name}.exe")),
            },
        };
        let probed = Probed {
            tools: vec![
                found("claude", Version::new(2, 1, 1)),
                Check {
                    name: "gh",
                    outcome: Outcome::Missing,
                },
                found("extension", Version::new(2, 1, 400)),
            ],
            gh: None,
            tailscale: Some(Ok(Tailnet {
                backend: "Stopped".to_owned(),
                online: None,
            })),
        };
        let report = report(&probed, &places, Utc::now());
        let line = |name: &str| report.line(name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(line("claude").level, Level::Fail);
        assert_eq!(line("gh").level, Level::Warn);
        assert!(
            line("extension")
                .text
                .contains("the standalone CLI is 2.1.1")
        );
        assert_eq!(line("protocol").level, Level::Ok);
        let start = line("SessionStart");
        assert_eq!(start.level, Level::Fail);
        assert!(
            start
                .text
                .ends_with("points the entries at the installed copy")
        );
        assert!(start.text.contains(process::GONE), "{}", start.text);
        assert_eq!(line("Stop").text, "runs the installed copy hook");
        assert_eq!(line("Notification").level, Level::Warn);
        assert_eq!(
            line("status line").text,
            "runs the installed copy statusline"
        );
        assert_eq!(line("plugin").level, Level::Note);
        let login = line("start at login");
        assert_eq!(login.level, Level::Note);
        assert!(login.text.starts_with("not registered"), "{}", login.text);
        // What PATH finds is the machine's own; the line is there.
        assert_eq!(line("crw on PATH").group, Group::Exes);
        let network = report.group(Group::Network).next().unwrap();
        assert_eq!(network.level, Level::Warn);
        assert!(network.text.starts_with("Stopped"));
        assert_eq!(line("drive").level, Level::Note);
        assert_eq!(line("registry").level, Level::Note);
        let ledger = line("ledger");
        assert_eq!(ledger.level, Level::Ok);
        assert!(ledger.text.contains("schema 7, sound"), "{}", ledger.text);
        assert!(line("crash log").text.contains("boom (b.rs:3:4)"));
        assert!(line("log").text.contains("1 files of the last 14 days"));
        assert_eq!(line("usage").text, "never read");
        assert!(line("ntfy").text.starts_with("no ntfy server"));
        let locks = line("lock files");
        assert_eq!(locks.level, Level::Note);
        assert!(locks.text.contains("1 live, 1 stale"), "{}", locks.text);
        assert_eq!(report.worst(), Level::Fail);
        // Every group answers.
        for group in Group::ALL {
            assert!(report.group(group).next().is_some(), "{group:?}");
        }
    }

    /// The start-at-login line over a trial's file: not registered, the installed window, another
    /// window, one that is gone, a command without a program, and a place that does not read.
    #[test]
    fn the_start_at_login_line_says_what_the_entry_starts() {
        let scratch = Scratch::new("doctor-autostart");
        let place = autostart::Place::File(scratch.0.join(autostart::SCRATCH_FILE));
        let bin = scratch.0.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let window = bin.join(install::WINDOW_EXE);
        fs::write(&window, "window").unwrap();
        let line = || autostart_line(autostart::status(&place, Some(window.as_path())));

        let (level, text) = line();
        assert_eq!(level, Level::Note);
        assert!(text.contains("crw autostart on"), "{text}");
        autostart::register(&place, &window).unwrap();
        assert_eq!(
            line(),
            (
                Level::Ok,
                "starts the installed window hidden in the tray".to_owned()
            )
        );
        let other = scratch.0.join("other.exe");
        fs::write(&other, "other").unwrap();
        autostart::register(&place, &other).unwrap();
        let (level, text) = line();
        assert_eq!(level, Level::Warn);
        assert!(
            text.ends_with("not the installed window: crw autostart on points it there"),
            "{text}"
        );
        fs::remove_file(&other).unwrap();
        let (level, text) = line();
        assert_eq!(level, Level::Warn);
        assert!(text.contains(process::GONE), "{text}");
        fs::write(scratch.0.join(autostart::SCRATCH_FILE), "  ").unwrap();
        let (level, text) = line();
        assert_eq!(level, Level::Warn);
        assert!(text.contains("names no program"), "{text}");
        // A folder where the file goes does not read.
        let folder = autostart::Place::File(bin.clone());
        let (level, _) = autostart_line(autostart::status(&folder, Some(window.as_path())));
        assert_eq!(level, Level::Warn);
    }

    /// The PATH line from what PATH found: nothing, the installed copy, the copy in the PATH
    /// folder as installed or not, or another exe.
    #[test]
    fn the_path_line_says_what_path_finds() {
        let scratch = Scratch::new("doctor-path");
        let (bin, dir) = (scratch.0.join("bin"), scratch.0.join("path"));
        fs::create_dir_all(&bin).unwrap();
        let installed = bin.join(install::CLI_EXE);
        fs::write(&installed, "crw").unwrap();
        let copy = dir.join(install::CLI_EXE);
        let line = |found: &Path| path_line(Some(found), Some(bin.as_path()), &dir);

        let (level, text) = path_line(None, Some(bin.as_path()), &dir);
        assert_eq!(level, Level::Note);
        assert_eq!(
            text,
            format!("not on PATH: crw path on puts a copy in {}", dir.display())
        );
        assert_eq!(
            line(&installed),
            (Level::Ok, "PATH finds the installed copy".to_owned())
        );
        install::place_path_copy(&bin, &dir, "20261114-080000").unwrap();
        assert_eq!(
            line(&copy),
            (
                Level::Ok,
                format!(
                    "PATH finds {}, a copy of the installed build",
                    copy.display()
                )
            )
        );
        // An older build of the tool names its folder as every build does.
        fs::write(&copy, "an older crw of clauderoadmapwrangler").unwrap();
        let (level, text) = line(&copy);
        assert_eq!(level, Level::Warn);
        assert!(text.ends_with("crw install refreshes it"), "{text}");
        // Another program's crw.exe there is said, and left alone.
        fs::write(&copy, "another tool's crw").unwrap();
        let (level, text) = line(&copy);
        assert_eq!(level, Level::Note);
        assert!(text.contains("another program's crw.exe"), "{text}");
        fs::write(&copy, "an older crw of clauderoadmapwrangler").unwrap();
        let elsewhere = scratch.0.join("tools").join(install::CLI_EXE);
        assert_eq!(
            line(&elsewhere),
            (
                Level::Note,
                format!("PATH finds {}, not the installed copy", elsewhere.display())
            )
        );
        // A copy placed in a folder PATH does not hold.
        assert_eq!(
            path_line(None, Some(bin.as_path()), &dir),
            (
                Level::Note,
                format!(
                    "not on PATH: {} is there, but its folder is not on PATH",
                    copy.display()
                )
            )
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
