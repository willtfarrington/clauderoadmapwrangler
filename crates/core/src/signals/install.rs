//! The signals' entries in the owner's Claude Code user settings (D-20, D-23; EP-12): a
//! `hooks.<Event>` group for SessionStart, Notification, Stop and SessionEnd, and for
//! UserPromptSubmit since EP-20b (a prompted session busy at once, a prompt's `@<project>`
//! mentions given their context), that runs the release
//! `crw.exe` directly with `hook` (Claude Code's exec form: no shell, so no quoting and no Git Bash
//! to start), with a 5 s timeout behind the relay's own half second; and a `statusLine` that runs
//! `<crw.exe> statusline`, every 30 s besides Claude Code's own triggers (a status line has no exec
//! form, so its path is written with forward slashes, which Git Bash wants). They are written only
//! on the owner's yes (rule 7), after a dated copy of the file in the tool's backups folder; the
//! file's other keys, their order and the two-space layout Claude Code writes are kept. The tool's
//! entries are known by the program they run (`crw.exe` with `hook`, or with `statusline`), so a
//! second install changes nothing, an install from a moved repository replaces them, and a removal
//! takes them out and nothing else. Another status line is never replaced, and the owner's stopgap
//! SessionStart hook (IN-2) is taken out only by an install whose start hook prints the inbox
//! itself (`crw hook --inbox`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::{files, paths};

/// The hooks the tool listens to (D-20); the prompt's last, added by EP-20b.
pub const EVENTS: [&str; 5] = ["SessionStart", "Notification", "Stop", "SessionEnd", PROMPT];

/// The prompt hook's event (EP-20b).
pub const PROMPT: &str = "UserPromptSubmit";

/// The longest a hook may run before Claude Code cancels it, in seconds: a backstop behind the
/// relay's own budget (Claude Code's default is 600).
pub const HOOK_TIMEOUT: u64 = 5;

/// How often the status line runs besides Claude Code's own triggers, in seconds (D-23).
pub const REFRESH: u64 = 30;

/// The program the entries run, by its file name.
const EXE: &str = "crw.exe";

/// The script of the owner's stopgap SessionStart hook (IN-2).
const STOPGAP: &str = "session-start-inbox.ps1";

/// The folder of the backups folder that holds the settings file's dated copies.
const COPIES: &str = "claude-settings";

/// Claude Code's user settings: `settings.json` in `CLAUDE_CONFIG_DIR`, else in `~\.claude`.
pub fn settings_file() -> Option<PathBuf> {
    let var = |name| std::env::var_os(name).filter(|value| !value.is_empty());
    let dir = var("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| var("USERPROFILE").map(|home| PathBuf::from(home).join(".claude")))?;
    Some(dir.join("settings.json"))
}

/// What an install writes besides the four hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// The status line (D-23).
    pub status_line: bool,
    /// The start hook prints the project's open inbox messages (`crw hook --inbox`), and the
    /// stopgap hook that did is taken out (IN-2).
    pub inbox: bool,
}

/// What one of the four events runs of the tool's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hooked {
    Missing,
    /// One entry of the tool's, the exe it runs and whether it prints the inbox.
    Ours {
        exe: PathBuf,
        inbox: bool,
    },
    /// More than one entry of the tool's: an install puts one back.
    Several,
}

/// What the status line runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Missing,
    Ours(PathBuf),
    /// Another command, as written.
    Other(String),
}

/// The tool's entries as the settings hold them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub hooks: Vec<(&'static str, Hooked)>,
    pub line: Line,
    /// The owner's stopgap SessionStart hook is there.
    pub stopgap: bool,
}

impl Status {
    /// Whether every hook runs `exe` once.
    pub fn hooks_run(&self, exe: &Path) -> bool {
        self.hooks
            .iter()
            .all(|(_, hooked)| matches!(hooked, Hooked::Ours { exe: e, .. } if paths::same(e, exe)))
    }

    /// Whether the tool's start hook prints the inbox (`hook --inbox`).
    pub fn prints_inbox(&self) -> bool {
        self.hooks.iter().any(|(event, hooked)| {
            *event == "SessionStart" && matches!(hooked, Hooked::Ours { inbox: true, .. })
        })
    }

    /// Whether none of the four hooks has an entry of the tool's.
    pub fn no_hooks(&self) -> bool {
        self.hooks
            .iter()
            .all(|(_, hooked)| *hooked == Hooked::Missing)
    }

    /// Whether every hook but the prompt's runs `exe` once and the prompt's is missing: an
    /// install from before EP-20b, which the next install completes.
    pub fn lacks_prompt_hook(&self, exe: &Path) -> bool {
        self.hooks.iter().all(|(event, hooked)| match hooked {
            Hooked::Missing => *event == PROMPT,
            Hooked::Ours { exe: e, .. } => *event != PROMPT && paths::same(e, exe),
            Hooked::Several => false,
        })
    }

    /// The status in a few words for `exe`: installed, partly, pointing elsewhere, or not.
    pub fn summary(&self, exe: &Path) -> String {
        let hooks = if self.hooks_run(exe) {
            "hooks installed".to_owned()
        } else if self.lacks_prompt_hook(exe) {
            "hooks installed but not the prompt hook (crw signals install adds it)".to_owned()
        } else if self.no_hooks() {
            "hooks not installed".to_owned()
        } else if self.hooks.iter().any(
            |(_, hooked)| matches!(hooked, Hooked::Ours { exe: e, .. } if !paths::same(e, exe)),
        ) {
            "hooks run another crw.exe".to_owned()
        } else {
            "hooks partly installed".to_owned()
        };
        let line = match &self.line {
            Line::Ours(e) if paths::same(e, exe) => "status line installed",
            Line::Ours(_) => "status line runs another crw.exe",
            Line::Other(_) => "another status line",
            Line::Missing => "no status line",
        };
        format!("{hooks}, {line}")
    }
}

/// What an install or a removal did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub path: PathBuf,
    /// The file as it was, when it was changed and existed.
    pub copy: Option<PathBuf>,
    /// The changes in words; empty when the file was left as it was.
    pub changes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("{} is not JSON Claude Code reads: {source}", path.display())]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("{0}")]
    Shape(String),
    #[error(
        "{} is a debug build, which every test run replaces: install from the release build \
         (target\\release\\crw.exe)",
        .0.display()
    )]
    DebugBuild(PathBuf),
    #[error("{} kept changing while it was read: try again", .0.display())]
    Busy(PathBuf),
}

fn io_error(path: &Path) -> impl FnOnce(io::Error) -> Error + '_ {
    move |source| Error::Io {
        path: path.to_owned(),
        source,
    }
}

/// The tool's entries in the settings file at `path`; a missing file holds none.
pub fn status(path: &Path) -> Result<Status, Error> {
    let bytes = read(path)?;
    Ok(read_status(&parse(bytes.as_deref(), path)?))
}

/// The tool's entries in `settings`.
pub fn read_status(settings: &Value) -> Status {
    let hooks = EVENTS
        .into_iter()
        .map(|event| {
            let found: Vec<(PathBuf, bool)> = handlers(settings, event).filter_map(ours).collect();
            let hooked = match found.as_slice() {
                [] => Hooked::Missing,
                [(exe, inbox)] => Hooked::Ours {
                    exe: exe.clone(),
                    inbox: *inbox,
                },
                _ => Hooked::Several,
            };
            (event, hooked)
        })
        .collect();
    let line = match settings.get("statusLine") {
        None | Some(Value::Null) => Line::Missing,
        Some(line) => {
            let command = line
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or_default();
            match ours_status_line(command) {
                Some(exe) => Line::Ours(exe),
                None => Line::Other(command.to_owned()),
            }
        }
    };
    Status {
        hooks,
        line,
        stopgap: handlers(settings, "SessionStart").any(is_stopgap),
    }
}

/// What an install for `exe` would change in the file at `path`, without writing it, and whether
/// the rest of the file would stay byte for byte (it is laid out as Claude Code writes it).
pub fn preview(path: &Path, exe: &Path, options: Options) -> Result<(Vec<String>, bool), Error> {
    if is_debug_build(exe) {
        return Err(Error::DebugBuild(exe.to_owned()));
    }
    let bytes = read(path)?;
    let settings = parse(bytes.as_deref(), path)?;
    let kept = bytes
        .as_deref()
        .is_none_or(|bytes| text(&settings).as_bytes() == bytes);
    Ok((installed(&settings, exe, options)?.1, kept))
}

/// Writes the tool's entries into the settings file at `path` for `exe`, after a dated copy of the
/// file in `backups`; leaves the file as it is when they are there already.
pub fn install(
    path: &Path,
    exe: &Path,
    options: Options,
    backups: &Path,
    today: &str,
) -> Result<Report, Error> {
    if is_debug_build(exe) {
        return Err(Error::DebugBuild(exe.to_owned()));
    }
    write_with(path, backups, today, |settings| {
        installed(settings, exe, options)
    })
}

/// Takes the tool's entries out of the settings file at `path`, after a dated copy of the file.
pub fn remove(path: &Path, backups: &Path, today: &str) -> Result<Report, Error> {
    write_with(path, backups, today, removed)
}

/// `settings` with the tool's entries for `exe`, and the changes in words.
pub fn installed(
    settings: &Value,
    exe: &Path,
    options: Options,
) -> Result<(Value, Vec<String>), Error> {
    let mut settings = settings.clone();
    let mut changes = Vec::new();
    let root = settings
        .as_object_mut()
        .ok_or_else(|| Error::Shape("the settings are not a JSON object".to_owned()))?;
    let hooks = root
        .entry("hooks")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| Error::Shape("the settings' hooks are not a JSON object".to_owned()))?;
    for event in EVENTS {
        let inbox = options.inbox && event == "SessionStart";
        let wanted = handler(exe, inbox);
        let groups = hooks
            .entry(event)
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .ok_or_else(|| Error::Shape(format!("the settings' hooks.{event} is not a list")))?;
        let mine: Vec<&Value> = groups
            .iter()
            .flat_map(group_handlers)
            .filter(|handler| ours(handler).is_some())
            .collect();
        if !(mine.len() == 1 && *mine[0] == wanted) {
            let had = !mine.is_empty();
            strip(groups, |handler| ours(handler).is_some());
            groups.push(serde_json::json!({ "hooks": [wanted] }));
            changes.push(format!(
                "{event} hook {}: {} {}",
                if had { "replaced" } else { "added" },
                exe.display(),
                if inbox { "hook --inbox" } else { "hook" }
            ));
        }
        if inbox && strip(groups, is_stopgap) {
            changes.push(
                "stopgap SessionStart hook taken out: crw hook --inbox prints the same lines \
                 (its script stays where it is)"
                    .to_owned(),
            );
        }
    }
    if options.status_line {
        let command = status_command(exe);
        match root.get_mut("statusLine") {
            Some(line @ Value::Object(_))
                if ours_status_line(line["command"].as_str().unwrap_or_default()).is_some() =>
            {
                let entry = line.as_object_mut().expect("an object");
                let before = entry.clone();
                entry.insert("type".to_owned(), "command".into());
                entry.insert("command".to_owned(), command.clone().into());
                entry.insert("refreshInterval".to_owned(), REFRESH.into());
                if *entry != before {
                    changes.push(format!("status line replaced: {command}"));
                }
            }
            Some(Value::Null) | None => {
                root.insert(
                    "statusLine".to_owned(),
                    serde_json::json!({
                        "type": "command",
                        "command": command,
                        "refreshInterval": REFRESH,
                    }),
                );
                changes.push(format!("status line added: {command}, every {REFRESH} s"));
            }
            Some(_) => {} // another status line stays (D-23): `Status::line` says so
        }
    }
    Ok((settings, changes))
}

/// `settings` without the tool's entries, and the changes in words. An event or the `hooks` table
/// left empty by the removal goes too.
pub fn removed(settings: &Value) -> Result<(Value, Vec<String>), Error> {
    let mut settings = settings.clone();
    let mut changes = Vec::new();
    let Some(root) = settings.as_object_mut() else {
        return Err(Error::Shape(
            "the settings are not a JSON object".to_owned(),
        ));
    };
    if let Some(Value::Object(hooks)) = root.get_mut("hooks") {
        for event in EVENTS {
            let emptied = match hooks.get_mut(event) {
                Some(Value::Array(groups)) => {
                    if strip(groups, |handler| ours(handler).is_some()) {
                        changes.push(format!("{event} hook taken out"));
                        groups.is_empty()
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if emptied {
                hooks.shift_remove(event);
            }
        }
        if hooks.is_empty() && !changes.is_empty() {
            root.shift_remove("hooks");
        }
    }
    let line_is_ours = root
        .get("statusLine")
        .and_then(|line| line.get("command"))
        .and_then(Value::as_str)
        .is_some_and(|command| ours_status_line(command).is_some());
    if line_is_ours {
        root.shift_remove("statusLine");
        changes.push("status line taken out".to_owned());
    }
    Ok((settings, changes))
}

/// The settings as text, as Claude Code writes them: two-space JSON and a final line break.
pub fn text(settings: &Value) -> String {
    let mut text = serde_json::to_string_pretty(settings).expect("a JSON value serialises");
    text.push('\n');
    text
}

/// The hook entry that runs `exe` (exec form: no shell).
fn handler(exe: &Path, inbox: bool) -> Value {
    let args: &[&str] = if inbox {
        &["hook", "--inbox"]
    } else {
        &["hook"]
    };
    serde_json::json!({
        "type": "command",
        "command": exe.to_string_lossy(),
        "args": args,
        "timeout": HOOK_TIMEOUT,
    })
}

/// The status line's command for `exe`: the path with forward slashes for Git Bash, in double
/// quotes when it holds a space or a character the shell reads.
pub fn status_command(exe: &Path) -> String {
    let path = exe.to_string_lossy().replace('\\', "/");
    let plain = path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "/:._-+,@%=".contains(c));
    if plain {
        format!("{path} statusline")
    } else {
        format!("\"{path}\" statusline")
    }
}

/// The handlers of every group of `event`.
fn handlers<'a>(settings: &'a Value, event: &str) -> impl Iterator<Item = &'a Value> {
    settings
        .get("hooks")
        .and_then(|hooks| hooks.get(event))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(group_handlers)
}

fn group_handlers(group: &Value) -> impl Iterator<Item = &Value> {
    group
        .get("hooks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// Takes the handlers `drop` picks out of `groups`, then the groups it left without a handler;
/// whether it took any.
fn strip(groups: &mut Vec<Value>, drop: impl Fn(&Value) -> bool) -> bool {
    let mut took = false;
    groups.retain_mut(|group| {
        let Some(list) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
            return true;
        };
        let before = list.len();
        list.retain(|handler| !drop(handler));
        if list.len() == before {
            return true;
        }
        took = true;
        !list.is_empty()
    });
    took
}

/// The exe and the inbox flag of a hook entry of the tool's: `crw.exe` with `hook`, in exec form
/// (what an install writes) or written as one shell command.
fn ours(handler: &Value) -> Option<(PathBuf, bool)> {
    let command = handler.get("command")?.as_str()?.trim();
    let (program, args): (String, Vec<String>) = match handler.get("args") {
        Some(Value::Array(args)) => (
            unquote(command).to_owned(),
            args.iter()
                .filter_map(|arg| arg.as_str().map(str::to_owned))
                .collect(),
        ),
        _ => {
            let (program, rest) = split_program(command)?;
            (
                program.to_owned(),
                rest.split_whitespace().map(str::to_owned).collect(),
            )
        }
    };
    if !is_crw(&program) || args.first().map(String::as_str) != Some("hook") {
        return None;
    }
    let inbox = args.iter().any(|arg| arg == "--inbox");
    Some((exe_path(&program), inbox))
}

/// The exe of a status line of the tool's: `<crw.exe> statusline`.
fn ours_status_line(command: &str) -> Option<PathBuf> {
    let (program, rest) = split_program(command.trim())?;
    (is_crw(program) && rest.trim() == "statusline").then(|| exe_path(program))
}

/// A shell command's program, unquoted, and the rest of it.
fn split_program(command: &str) -> Option<(&str, &str)> {
    if let Some(quoted) = command.strip_prefix('"') {
        let end = quoted.find('"')?;
        return Some((&quoted[..end], &quoted[end + 1..]));
    }
    Some(command.split_once(' ').unwrap_or((command, "")))
}

fn unquote(text: &str) -> &str {
    text.strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(text)
}

fn is_crw(program: &str) -> bool {
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    name.eq_ignore_ascii_case(EXE) || name.eq_ignore_ascii_case("crw")
}

/// A program as a Windows path.
fn exe_path(program: &str) -> PathBuf {
    PathBuf::from(program.replace('/', "\\"))
}

fn is_stopgap(handler: &Value) -> bool {
    let mentions = |value: Option<&Value>| {
        value.is_some_and(|value| match value {
            Value::String(text) => text.to_lowercase().contains(STOPGAP),
            Value::Array(args) => args
                .iter()
                .filter_map(Value::as_str)
                .any(|arg| arg.to_lowercase().contains(STOPGAP)),
            _ => false,
        })
    };
    mentions(handler.get("command")) || mentions(handler.get("args"))
}

/// Whether `exe` lies in a Cargo target's `debug` folder.
fn is_debug_build(exe: &Path) -> bool {
    let name = |dir: Option<&Path>| {
        dir.and_then(Path::file_name)
            .map(|name| name.to_string_lossy().to_lowercase())
    };
    let folder = exe.parent();
    name(folder).as_deref() == Some("debug")
        && name(folder.and_then(Path::parent)).as_deref() == Some("target")
}

/// The file's bytes, or `None` when there is none.
fn read(path: &Path) -> Result<Option<Vec<u8>>, Error> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io_error(path)(e)),
    }
}

/// The settings in `bytes`: an empty object for no file or an empty one.
fn parse(bytes: Option<&[u8]>, path: &Path) -> Result<Value, Error> {
    let text = bytes.map(String::from_utf8_lossy).unwrap_or_default();
    let text = text.trim_start_matches('\u{feff}');
    if text.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    serde_json::from_str(text).map_err(|source| Error::Json {
        path: path.to_owned(),
        source,
    })
}

/// Reads the file, lets `change` work on it, and writes the result when it changed something:
/// after a dated copy of the file as it was, and only over the bytes it read (Claude Code writes
/// the file too).
fn write_with(
    path: &Path,
    backups: &Path,
    today: &str,
    change: impl Fn(&Value) -> Result<(Value, Vec<String>), Error>,
) -> Result<Report, Error> {
    for _ in 0..3 {
        let before = read(path)?;
        let (after, changes) = change(&parse(before.as_deref(), path)?)?;
        if changes.is_empty() {
            return Ok(Report {
                path: path.to_owned(),
                copy: None,
                changes,
            });
        }
        if read(path)? != before {
            continue;
        }
        let copy = match &before {
            Some(bytes) => Some(
                files::dated_copy(bytes, &backups.join(COPIES), "settings", today, ".json")
                    .map_err(io_error(backups))?,
            ),
            None => None,
        };
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(io_error(dir))?;
        }
        files::write_atomic(path, text(&after).as_bytes()).map_err(io_error(path))?;
        return Ok(Report {
            path: path.to_owned(),
            copy,
            changes,
        });
    }
    Err(Error::Busy(path.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    const EXE_PATH: &str = r"C:\Tools\roadmap\target\release\crw.exe";
    const BOTH: Options = Options {
        status_line: true,
        inbox: false,
    };

    /// Settings in the owner's form: other keys around the hooks, a stopgap start hook.
    const OWNER: &str = r#"{
  "cleanupPeriodDays": 3650,
  "permissions": {
    "deny": [
      "Read(~/.ssh/**)"
    ],
    "additionalDirectories": []
  },
  "model": "opus",
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume",
        "hooks": [
          {
            "type": "command",
            "command": "pwsh -NoProfile -ExecutionPolicy Bypass -File C:/Home/.claude/hooks/session-start-inbox.ps1",
            "timeout": 15
          }
        ]
      }
    ]
  },
  "theme": "dark",
  "remoteControlAtStartup": true
}
"#;

    fn value(text: &str) -> Value {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn the_owners_form_reads_back_byte_for_byte() {
        assert_eq!(text(&value(OWNER)), OWNER);
    }

    #[test]
    fn an_install_adds_five_hooks_and_the_status_line_and_keeps_the_rest() {
        let exe = Path::new(EXE_PATH);
        let (after, changes) = installed(&value(OWNER), exe, BOTH).unwrap();
        assert_eq!(changes.len(), 6, "{changes:?}");
        let written = text(&after);
        // The other keys keep their places, the stopgap its entry.
        let order: Vec<&str> = after
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            order,
            [
                "cleanupPeriodDays",
                "permissions",
                "model",
                "hooks",
                "theme",
                "remoteControlAtStartup",
                "statusLine"
            ]
        );
        assert!(written.contains("session-start-inbox.ps1"));
        let status = read_status(&after);
        assert!(status.hooks_run(exe));
        assert!(status.stopgap);
        assert!(!status.prints_inbox());
        assert_eq!(status.line, Line::Ours(exe.to_owned()));
        assert_eq!(
            status.summary(exe),
            "hooks installed, status line installed"
        );
        // The exec form, and the status line in Git Bash's spelling.
        let stop = &after["hooks"]["Stop"][0]["hooks"][0];
        assert_eq!(
            *stop,
            serde_json::json!({
                "type": "command",
                "command": EXE_PATH,
                "args": ["hook"],
                "timeout": 5
            })
        );
        assert_eq!(
            after["statusLine"],
            serde_json::json!({
                "type": "command",
                "command": "C:/Tools/roadmap/target/release/crw.exe statusline",
                "refreshInterval": 30
            })
        );
        // The prompt hook runs the same relay (EP-20b).
        assert_eq!(after["hooks"][PROMPT][0]["hooks"][0], *stop);
        // A second install changes nothing.
        let (again, changes) = installed(&after, exe, BOTH).unwrap();
        assert!(changes.is_empty(), "{changes:?}");
        assert_eq!(text(&again), written);
        // A removal leaves the owner's file as it was.
        let (back, changes) = removed(&after).unwrap();
        assert_eq!(changes.len(), 6, "{changes:?}");
        assert_eq!(text(&back), OWNER);
    }

    #[test]
    fn an_install_from_before_the_prompt_hook_adds_it_alone() {
        let exe = Path::new(EXE_PATH);
        // The four entries and the status line an install before EP-20b wrote.
        let mut before = installed(&value(OWNER), exe, BOTH).unwrap().0;
        before["hooks"]
            .as_object_mut()
            .unwrap()
            .shift_remove(PROMPT);
        let status = read_status(&before);
        assert!(!status.hooks_run(exe));
        assert!(status.lacks_prompt_hook(exe));
        assert_eq!(
            status.summary(exe),
            "hooks installed but not the prompt hook (crw signals install adds it), status line \
             installed"
        );
        let (after, changes) = installed(&before, exe, BOTH).unwrap();
        assert_eq!(
            changes,
            [format!("UserPromptSubmit hook added: {EXE_PATH} hook")]
        );
        assert!(read_status(&after).hooks_run(exe));
        // Another exe's entries are not that case: they point elsewhere.
        let moved = Path::new(r"D:\Elsewhere\target\release\crw.exe");
        assert!(!status.lacks_prompt_hook(moved));
        assert!(!read_status(&value("{}")).lacks_prompt_hook(exe));
    }

    #[test]
    fn the_inbox_install_replaces_the_stopgap_and_a_move_replaces_the_exe() {
        let exe = Path::new(EXE_PATH);
        let options = Options {
            status_line: false,
            inbox: true,
        };
        let (after, changes) = installed(&value(OWNER), exe, options).unwrap();
        assert!(changes.iter().any(|c| c.contains("stopgap")), "{changes:?}");
        let status = read_status(&after);
        assert!(!status.stopgap);
        assert!(status.prints_inbox());
        assert_eq!(status.line, Line::Missing);
        assert_eq!(
            status.hooks[0],
            (
                "SessionStart",
                Hooked::Ours {
                    exe: exe.to_owned(),
                    inbox: true
                }
            )
        );
        // The stopgap's group went with its one entry.
        assert_eq!(after["hooks"]["SessionStart"].as_array().unwrap().len(), 1);
        // The repository moved: the entries follow the new exe, one each.
        let moved = Path::new(r"D:\Elsewhere\target\release\crw.exe");
        assert_eq!(
            read_status(&after).summary(moved),
            "hooks run another crw.exe, no status line"
        );
        let (after, changes) = installed(&after, moved, options).unwrap();
        assert_eq!(changes.len(), 5, "{changes:?}");
        assert!(read_status(&after).hooks_run(moved));
        assert_eq!(after["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn another_status_line_stays_and_shells_are_read_too() {
        let exe = Path::new(EXE_PATH);
        let settings = value(
            r#"{"statusLine": {"type": "command", "command": "~/.claude/statusline.sh", "padding": 2}}"#,
        );
        let (after, changes) = installed(&settings, exe, BOTH).unwrap();
        assert_eq!(changes.len(), 5, "{changes:?}");
        assert_eq!(after["statusLine"], settings["statusLine"]);
        assert_eq!(
            read_status(&after).line,
            Line::Other("~/.claude/statusline.sh".to_owned())
        );
        // Entries written by hand as shell commands are known as the tool's.
        let by_hand = value(
            r#"{"hooks": {"Stop": [{"hooks": [{"type": "command", "command": "\"C:/Program Files/roadmap/crw.exe\" hook"}]}]},
                "statusLine": {"type": "command", "command": "\"C:/Program Files/roadmap/crw.exe\" statusline"}}"#,
        );
        let status = read_status(&by_hand);
        let spaced = Path::new(r"C:\Program Files\roadmap\crw.exe");
        assert_eq!(
            status.hooks[2],
            (
                "Stop",
                Hooked::Ours {
                    exe: spaced.to_owned(),
                    inbox: false
                }
            )
        );
        assert_eq!(status.line, Line::Ours(spaced.to_owned()));
        assert_eq!(
            status_command(spaced),
            "\"C:/Program Files/roadmap/crw.exe\" statusline"
        );
        let (gone, _) = removed(&by_hand).unwrap();
        assert_eq!(gone, value("{}"));
    }

    #[test]
    fn the_file_is_copied_before_it_changes_and_a_debug_build_is_refused() {
        let scratch = Scratch::new("signals-install");
        let path = scratch.0.join(".claude").join("settings.json");
        let backups = scratch.0.join("backups");
        let exe = Path::new(EXE_PATH);
        // No file yet: written new, nothing to copy.
        let report = install(&path, exe, BOTH, &backups, "2026-11-02").unwrap();
        assert_eq!(report.copy, None);
        assert!(status(&path).unwrap().hooks_run(exe));
        // The owner's file: copied as it was, then written.
        fs::write(&path, OWNER).unwrap();
        let report = install(&path, exe, BOTH, &backups, "2026-11-02").unwrap();
        let copy = report.copy.unwrap();
        assert_eq!(copy, backups.join(COPIES).join("settings.2026-11-02.json"));
        assert_eq!(fs::read_to_string(&copy).unwrap(), OWNER);
        let written = fs::read_to_string(&path).unwrap();
        // Nothing to do: no copy, no write.
        let report = install(&path, exe, BOTH, &backups, "2026-11-02").unwrap();
        assert_eq!((report.copy, report.changes.len()), (None, 0));
        assert_eq!(fs::read_to_string(&path).unwrap(), written);
        // The removal's copy is the next of the day; the file is the owner's again.
        let report = remove(&path, &backups, "2026-11-02").unwrap();
        assert_eq!(
            report.copy.unwrap(),
            backups.join(COPIES).join("settings.2026-11-02.2.json")
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), OWNER);
        // A file that is not JSON is left alone; a debug build is refused.
        fs::write(&path, "{ not json").unwrap();
        assert!(matches!(
            install(&path, exe, BOTH, &backups, "2026-11-02"),
            Err(Error::Json { .. })
        ));
        assert_eq!(fs::read_to_string(&path).unwrap(), "{ not json");
        let debug = Path::new(r"C:\Tools\roadmap\target\debug\crw.exe");
        assert!(matches!(
            install(&path, debug, BOTH, &backups, "2026-11-02"),
            Err(Error::DebugBuild(_))
        ));
        assert!(!is_debug_build(exe));
    }
}
