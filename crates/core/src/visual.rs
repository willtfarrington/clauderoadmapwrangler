//! The visual launch (D-16): a session of a project's next brief opened in the VS Code extension
//! with its prompt filled in, for the owner to send. The extension's link opens a Claude Code tab
//! in whichever VS Code window has the focus, so the project's own workspace file is opened first
//! (the D-7 and D-16 addenda), which opens the project's window or brings it to the front; once the
//! extension shows that window in its lock file, the link goes to `Code.exe --open-url`, as
//! Windows' handler for `vscode://` hands it over. The tool never sees the session itself: it
//! records the launch as visual, with no process, and the hooks see the rest (EP-12). The tests
//! inject the processes; nothing here is started by a test (D-29).

use std::ffi::OsString;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::{env, fs, thread};

use chrono::Utc;
use serde::Deserialize;

use crate::discovery::{self, OwnFile};
use crate::doctor::{self, Outcome};
use crate::ledger::{Ledger, NewSession};
use crate::version::Version;
use crate::{paths, process, session, workspace};

/// The extension's link that opens a Claude Code tab (Claude Code's VS Code page).
pub const LINK: &str = "vscode://anthropic.claude-code/open";

/// The longest link handed over: a Windows command line holds 32,767 characters, the program's
/// path and the flags among them.
pub const MAX_LINK: usize = 32_000;

/// How long a launch waits for the project's window to show the extension: a VS Code that was not
/// running takes some seconds to start.
pub const WAIT: Duration = Duration::from_secs(60);

/// The pause between finding the window and handing over the link, so that the window has the
/// focus the link goes to.
pub const SETTLE: Duration = Duration::from_millis(500);

/// A `Code.exe` that hands its arguments to a running VS Code exits within this; one still running
/// then is VS Code itself, started by it, and is left to run.
const HANDOVER: Duration = Duration::from_secs(10);

/// How often the lock files are read while a launch waits.
const POLL: Duration = Duration::from_millis(250);

/// What a cancelled launch says.
pub const CANCELLED: &str = "cancelled before the prompt was handed to VS Code";

/// The link that opens a Claude Code tab with `prompt` filled in, or why it cannot be handed over.
pub fn link(prompt: &str) -> Result<String, String> {
    let link = encode(prompt);
    if link.len() > MAX_LINK {
        return Err(format!(
            "the prompt makes a link of {} characters, more than the {MAX_LINK} a Windows command \
             line passes: shorten it, or copy it into a Claude Code tab by hand",
            link.len()
        ));
    }
    Ok(link)
}

/// The link with `prompt` as its `prompt` parameter. VS Code decodes a link once and the
/// extension reads its query with `URLSearchParams`, which decodes it again and takes `&`, `+`
/// and `%` its own way; those three are escaped twice, every other byte outside the unreserved
/// set once.
fn encode(prompt: &str) -> String {
    let mut out = String::with_capacity(LINK.len() + 8 + prompt.len() * 3);
    out.push_str(LINK);
    out.push_str("?prompt=");
    for byte in prompt.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(char::from(byte));
            }
            b'%' | b'&' | b'+' => {
                let _ = write!(out, "%25{byte:02X}");
            }
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

/// A VS Code window the extension runs in, as its lock file says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// VS Code's main process, which its windows share.
    pub pid: u32,
    /// The window's folders; the first is its sessions' working folder.
    pub folders: Vec<PathBuf>,
    pub ide: Option<String>,
    /// Whether that process runs: a lock file outlives a VS Code that was killed.
    pub alive: bool,
}

impl Window {
    /// Whether this is a running window whose first folder is `folder`.
    pub fn opens(&self, folder: &Path) -> bool {
        self.alive
            && self
                .folders
                .first()
                .is_some_and(|first| paths::same(first, folder))
    }
}

/// What the tool reads of a lock file; the rest, the auth token among it, is never kept.
#[derive(Deserialize)]
struct Lock {
    pid: u32,
    #[serde(rename = "workspaceFolders", default)]
    folders: Vec<PathBuf>,
    #[serde(rename = "ideName", default)]
    ide: Option<String>,
}

/// The folder the extension writes its lock files in: `%USERPROFILE%\.claude\ide`.
pub fn lock_folder() -> Option<PathBuf> {
    env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join(r".claude\ide"))
}

/// The windows of the lock files in `dir`, by file name; a file that does not read is passed over.
pub fn windows(dir: &Path) -> Vec<Window> {
    windows_with(dir, running)
}

fn windows_with(dir: &Path, alive: impl Fn(u32) -> bool) -> Vec<Window> {
    let mut found: Vec<(String, Window)> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("lock"))
        })
        .filter_map(|entry| {
            let lock: Lock = serde_json::from_str(&fs::read_to_string(entry.path()).ok()?).ok()?;
            let window = Window {
                pid: lock.pid,
                folders: lock.folders,
                ide: lock.ide,
                alive: alive(lock.pid),
            };
            Some((entry.file_name().to_string_lossy().into_owned(), window))
        })
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found.into_iter().map(|(_, window)| window).collect()
}

/// Whether the process `pid` runs.
#[cfg(windows)]
fn running(pid: u32) -> bool {
    use windows::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
    use windows::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    // SAFETY: the handle is the one OpenProcess returned; it is asked once and closed.
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return false;
        };
        let mut code = 0u32;
        let alive = GetExitCodeProcess(handle, &mut code).is_ok() && code == STILL_ACTIVE.0 as u32;
        let _ = CloseHandle(handle);
        alive
    }
}

#[cfg(not(windows))]
fn running(_pid: u32) -> bool {
    true
}

/// Where VS Code is: the registry's `settings.vscode` when set, else the user install, the
/// system install, or the install whose `bin\code.cmd` is on PATH.
pub fn locate(setting: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = setting {
        return Some(path.to_owned());
    }
    let dir = |var: &str| env::var_os(var).map(PathBuf::from);
    candidates(
        dir("LOCALAPPDATA"),
        dir("ProgramFiles"),
        doctor::on_path("code.cmd"),
    )
    .into_iter()
    .find(|path| path.is_file())
}

/// The places `Code.exe` is looked for, in order.
fn candidates(
    local: Option<PathBuf>,
    programs: Option<PathBuf>,
    code_cmd: Option<PathBuf>,
) -> Vec<PathBuf> {
    let install = code_cmd
        .as_deref()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf);
    [
        local.map(|dir| dir.join(r"Programs\Microsoft VS Code")),
        programs.map(|dir| dir.join("Microsoft VS Code")),
        install,
    ]
    .into_iter()
    .flatten()
    .map(|dir| dir.join("Code.exe"))
    .collect()
}

/// What a visual launch of a project would meet, for the dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Readiness {
    /// `Code.exe`; none when it was not found.
    pub code: Option<PathBuf>,
    /// The Claude Code extension's version, when VS Code lists it.
    pub extension: Option<Version>,
    /// The project's own workspace file, and whether it is there.
    pub workspace: Option<PathBuf>,
    pub workspace_exists: bool,
    /// Whether a running VS Code window has the project as its first folder.
    pub window_open: bool,
}

impl Readiness {
    /// What `folder`'s launch would meet with this VS Code, extension and windows.
    pub fn of(
        folder: &Path,
        code: Option<PathBuf>,
        extension: Option<Version>,
        windows: &[Window],
    ) -> Self {
        let workspace = workspace::own_file(folder);
        Self {
            code,
            extension,
            workspace_exists: workspace.as_deref().is_some_and(Path::is_file),
            workspace,
            window_open: windows.iter().any(|window| window.opens(folder)),
        }
    }
}

/// What a visual launch of `folder` would meet on this machine; `setting` is the registry's
/// `settings.vscode`.
pub fn readiness(folder: &Path, setting: Option<&Path>) -> Readiness {
    let extension = env::var_os("USERPROFILE")
        .map(|home| PathBuf::from(home).join(r".vscode\extensions"))
        .and_then(|dir| match doctor::extension_version(&dir) {
            Outcome::Found { version, .. } => Some(version),
            _ => None,
        });
    let windows = lock_folder().map(|dir| windows(&dir)).unwrap_or_default();
    Readiness::of(folder, locate(setting), extension, &windows)
}

/// A visual launch, as its caller asks for it.
#[derive(Debug, Clone, PartialEq)]
pub struct Launch {
    /// The project's name in the registry.
    pub project: String,
    /// The project's folder, whose own workspace file is opened (D-16 addendum).
    pub folder: PathBuf,
    /// The EP the session works on, such as `EP-3`.
    pub ep: Option<String>,
    pub prompt: String,
    /// `Code.exe`.
    pub code: PathBuf,
    /// The folder of the extension's lock files.
    pub locks: PathBuf,
    /// How long the launch waits for the project's window to show the extension.
    pub wait: Duration,
    /// The pause before the link, once the window is there.
    pub settle: Duration,
    /// The ledger the launch is recorded in; none for a trial.
    pub ledger: Option<PathBuf>,
}

impl Launch {
    /// A launch of `prompt` in `folder`'s window: the usual wait and pause, nothing recorded.
    pub fn new(project: &str, folder: &Path, prompt: &str, code: &Path, locks: &Path) -> Self {
        Self {
            project: project.to_owned(),
            folder: folder.to_owned(),
            ep: None,
            prompt: prompt.to_owned(),
            code: code.to_owned(),
            locks: locks.to_owned(),
            wait: WAIT,
            settle: SETTLE,
            ledger: None,
        }
    }
}

/// What a visual launch did.
#[derive(Debug, Clone, PartialEq)]
pub struct Opened {
    /// The project's own workspace file: written just now, or kept as it was.
    pub workspace: OwnFile,
    /// Whether the project's window ran the extension before the launch.
    pub was_open: bool,
    /// From the workspace file's opening to the window's showing.
    pub waited: Duration,
    pub ledger_row: Option<i64>,
    /// Why the ledger did not take the launch, when it did not.
    pub ledger_problem: Option<String>,
}

/// Opens the project's window and fills the prompt into a Claude Code tab there (D-16), waiting
/// while the window starts; a `cancel` raised meanwhile stops it before the link is handed over.
pub fn open(launch: &Launch, cancel: &AtomicBool) -> Result<Opened, String> {
    open_with(launch, cancel, &mut Children)
}

/// How a launch starts `Code.exe`: [`Children`], or a test's stand-in.
trait Starter {
    /// Starts `program` with `args` and waits up to `wait` for it to exit: `true` when it exited,
    /// `false` when it still runs.
    fn run(&mut self, program: &Path, args: &[OsString], wait: Duration) -> Result<bool, String>;
}

fn open_with(
    launch: &Launch,
    cancel: &AtomicBool,
    starter: &mut dyn Starter,
) -> Result<Opened, String> {
    let link = link(&launch.prompt)?;
    let workspace = discovery::write_own_file(&launch.folder).map_err(|e| e.to_string())?;
    let file = workspace
        .path()
        .ok_or("the project's own workspace file was not written")?
        .to_owned();
    let was_open = windows(&launch.locks)
        .iter()
        .any(|window| window.opens(&launch.folder));
    let started = Instant::now();
    starter.run(&launch.code, &[file.clone().into_os_string()], HANDOVER)?;
    while !windows(&launch.locks)
        .iter()
        .any(|window| window.opens(&launch.folder))
    {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.to_owned());
        }
        if started.elapsed() >= launch.wait {
            return Err(format!(
                "no VS Code window on {} showed the Claude Code extension within {} s of opening \
                 {}, so nothing was filled in; the extension runs only in a trusted workspace",
                launch.folder.display(),
                launch.wait.as_secs(),
                file.display()
            ));
        }
        thread::sleep(POLL);
    }
    let waited = started.elapsed();
    thread::sleep(launch.settle);
    if cancel.load(Ordering::Relaxed) {
        return Err(CANCELLED.to_owned());
    }
    let args: Vec<OsString> = vec!["--open-url".into(), "--".into(), link.into()];
    starter.run(&launch.code, &args, HANDOVER)?;
    let (ledger_row, ledger_problem) = match record(launch) {
        Ok(row) => (row, None),
        Err(e) => (None, Some(e)),
    };
    Ok(Opened {
        workspace,
        was_open,
        waited,
        ledger_row,
        ledger_problem,
    })
}

/// The launch's ledger row: `visual`, with no session id, model or effort, since VS Code chooses
/// them; none when no ledger was asked for.
fn record(launch: &Launch) -> Result<Option<i64>, String> {
    let Some(path) = &launch.ledger else {
        return Ok(None);
    };
    Ledger::open(path)
        .and_then(|ledger| {
            ledger.start_session(&NewSession {
                session_id: None,
                project: &launch.project,
                ep: launch.ep.as_deref(),
                mode: "visual",
                model: None,
                effort: None,
                ultracode: false,
                started_at: Utc::now(),
            })
        })
        .map(Some)
        .map_err(|e| format!("the ledger did not record the launch: {e}"))
}

/// `Code.exe` started for real: one child at a time (D-29), without what it must not inherit.
struct Children;

impl Starter for Children {
    fn run(&mut self, program: &Path, args: &[OsString], wait: Duration) -> Result<bool, String> {
        let _turn = process::turn();
        let mut command = Command::new(program);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        for name in withheld(env::vars_os().map(|(name, _)| name)) {
            command.env_remove(name);
        }
        let mut child = command
            .spawn()
            .map_err(|e| format!("could not start {}: {e}", program.display()))?;
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => return Ok(true),
                Ok(Some(status)) => {
                    return Err(format!("{} exited with {status}", program.display()));
                }
                Ok(None) if started.elapsed() >= wait => return Ok(false),
                Ok(None) => thread::sleep(Duration::from_millis(50)),
                Err(e) => return Err(format!("could not wait for {}: {e}", program.display())),
            }
        }
    }
}

/// The variables among `names` a `Code.exe` child does not inherit: the API key and the session
/// markers a headless child is spared (D-15), and Electron's and VS Code's own, which the tool
/// inherits when it runs under VS Code: with `ELECTRON_RUN_AS_NODE`, `Code.exe` starts as Node.
/// VS Code hands the environment of the `Code.exe` that asks to the windows it opens.
fn withheld(names: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let names: Vec<OsString> = names.into_iter().collect();
    let mut found = session::markers(names.iter().cloned());
    found.extend(names.into_iter().filter(|name| {
        let upper = name.to_string_lossy().to_uppercase();
        upper.starts_with("ELECTRON_") || upper.starts_with("VSCODE_")
    }));
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    /// VS Code's reading of a link's query: every `%HH` decoded, the bytes read as UTF-8.
    fn vscode_decodes(query: &str) -> String {
        String::from_utf8(percent_decode(query)).expect("UTF-8")
    }

    /// `URLSearchParams` on a query: pairs split at `&` and their first `=`, `+` a space, `%HH`
    /// decoded, a `%` without two hex digits kept.
    fn search_params(query: &str) -> Vec<(String, String)> {
        query
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
                let read = |text: &str| {
                    String::from_utf8_lossy(&percent_decode(&text.replace('+', " "))).into_owned()
                };
                (read(name), read(value))
            })
            .collect()
    }

    fn percent_decode(text: &str) -> Vec<u8> {
        let bytes = text.as_bytes();
        let hex = |b: u8| char::from(b).to_digit(16);
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%'
                && let (Some(high), Some(low)) = (
                    bytes.get(i + 1).copied().and_then(hex),
                    bytes.get(i + 2).copied().and_then(hex),
                )
            {
                out.push((high * 16 + low) as u8);
                i += 3;
                continue;
            }
            out.push(bytes[i]);
            i += 1;
        }
        out
    }

    /// The prompt the extension reads from `link`.
    fn read_back(link: &str) -> Vec<(String, String)> {
        let query = link
            .strip_prefix(LINK)
            .and_then(|rest| rest.strip_prefix('?'))
            .expect("the extension's link");
        search_params(&vscode_decodes(query))
    }

    #[test]
    fn the_prompt_survives_both_decodings() {
        // The page's own example reads as it shows.
        assert_eq!(
            encode("review my changes"),
            "vscode://anthropic.claude-code/open?prompt=review%20my%20changes"
        );
        for prompt in [
            "review my changes",
            "a & b + c = d; `RS+1` + 2 == 3",
            "100% sure: %41 is not A, %zz stays, %",
            "#hash ?query /path \\back \"quoted\" 'single' <angle>",
            "line one\nline two\r\n\ttabbed",
            "Fable 5.1 — the opener … → ü, 中文, 🦀",
            "",
            "&&&+++%%%===",
        ] {
            assert_eq!(
                read_back(&encode(prompt)),
                [("prompt".to_owned(), prompt.to_owned())],
                "{prompt:?}"
            );
        }
        // Escaped once, as the page's example is, an `&` cuts the prompt and a `+` turns into a
        // space: why the three are escaped twice.
        let once = format!("{LINK}?prompt=a%20%26%20b%20%2B%20c");
        assert_eq!(read_back(&once)[0].1, "a ");
    }

    #[test]
    fn a_link_too_long_for_a_command_line_is_refused() {
        assert!(link(&"x".repeat(MAX_LINK - 100)).is_ok());
        // A space takes three characters in the link.
        let why = link(&" ".repeat(MAX_LINK / 2)).unwrap_err();
        assert!(why.contains("48043 characters"), "{why}");
    }

    fn lock(dir: &Path, name: &str, pid: u32, folders: &[&str]) {
        let text = serde_json::json!({
            "pid": pid,
            "workspaceFolders": folders,
            "ideName": "Visual Studio Code",
            "transport": "ws",
            "runningInWindows": true,
            "authToken": "secret-token-never-kept",
        });
        fs::write(dir.join(name), text.to_string()).unwrap();
    }

    #[test]
    fn the_lock_files_name_the_windows_and_keep_no_token() {
        let scratch = Scratch::new("visual-locks");
        let dir = &scratch.0;
        assert!(windows_with(dir, |_| true).is_empty());
        lock(dir, "41000.lock", 7, &[r"c:\data\alpha", r"c:\data\beta"]);
        lock(dir, "42000.LOCK", 8, &[r"c:\data\beta"]);
        fs::write(dir.join("43000.lock"), "not json").unwrap();
        fs::write(
            dir.join("44000.lock"),
            r#"{"workspaceFolders": ["c:\\data\\gamma"]}"#,
        )
        .unwrap();
        fs::write(dir.join("notes.txt"), "{\"pid\": 9}").unwrap();
        let found = windows_with(dir, |pid| pid == 7);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].pid, 7);
        assert_eq!(found[0].ide.as_deref(), Some("Visual Studio Code"));
        assert!(!format!("{found:?}").contains("secret-token"));
        // The first folder is the window's; a window of a process that is gone counts for none.
        assert!(found[0].opens(Path::new(r"C:\Data\Alpha")));
        assert!(!found[0].opens(Path::new(r"C:\Data\beta")));
        assert!(!found[1].alive);
        assert!(!found[1].opens(Path::new(r"C:\Data\beta")));
        // This test's own process runs; a process id Windows never gives (not a multiple of
        // four) does not.
        assert!(running(std::process::id()));
        assert!(!running(u32::MAX - 2));
    }

    #[test]
    fn vs_code_is_looked_for_in_the_installs_order() {
        let found = candidates(
            Some(PathBuf::from(r"C:\Users\someone\AppData\Local")),
            Some(PathBuf::from(r"C:\Program Files")),
            Some(PathBuf::from(r"D:\Tools\VS Code\bin\code.cmd")),
        );
        assert_eq!(
            found,
            [
                PathBuf::from(
                    r"C:\Users\someone\AppData\Local\Programs\Microsoft VS Code\Code.exe"
                ),
                PathBuf::from(r"C:\Program Files\Microsoft VS Code\Code.exe"),
                PathBuf::from(r"D:\Tools\VS Code\Code.exe"),
            ]
        );
        assert!(candidates(None, None, None).is_empty());
        let set = Path::new(r"E:\Insiders\Code - Insiders.exe");
        assert_eq!(locate(Some(set)).as_deref(), Some(set));
    }

    #[test]
    fn a_code_child_inherits_neither_vs_code_nor_a_session() {
        let names = [
            "ELECTRON_RUN_AS_NODE",
            "VSCODE_PID",
            "vscode_cwd",
            "CLAUDECODE",
            "CLAUDE_CODE_SESSION_ID",
            "PATH",
            "USERPROFILE",
            "ANTHROPIC_API_KEY",
        ]
        .map(OsString::from);
        let mut withheld: Vec<String> = withheld(names)
            .into_iter()
            .map(|name| name.to_string_lossy().into_owned())
            .collect();
        withheld.sort();
        withheld.dedup();
        assert_eq!(
            withheld,
            [
                "ANTHROPIC_API_KEY",
                "CLAUDECODE",
                "CLAUDE_CODE_SESSION_ID",
                "ELECTRON_RUN_AS_NODE",
                "VSCODE_PID",
                "vscode_cwd"
            ]
        );
    }

    /// The test's stand-in for `Code.exe`: it records each call, and when it is asked to open a
    /// workspace file it writes the lock file of a window on the project, unless told not to.
    struct Fake {
        calls: Vec<Vec<String>>,
        locks: PathBuf,
        folder: String,
        window_appears: bool,
    }

    impl Starter for Fake {
        fn run(&mut self, _: &Path, args: &[OsString], _: Duration) -> Result<bool, String> {
            let args: Vec<String> = args
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
            if self.window_appears && args.len() == 1 {
                lock(
                    &self.locks,
                    "45000.lock",
                    std::process::id(),
                    &[self.folder.as_str()],
                );
            }
            self.calls.push(args);
            Ok(true)
        }
    }

    struct Setup {
        scratch: Scratch,
        launch: Launch,
        fake: Fake,
    }

    fn setup(name: &str, window_appears: bool) -> Setup {
        let scratch = Scratch::new(name);
        let folder = scratch.0.join("data").join("alpha");
        let locks = scratch.0.join("ide");
        fs::create_dir_all(&folder).unwrap();
        fs::create_dir_all(&locks).unwrap();
        let mut launch = Launch::new(
            "alpha",
            &folder,
            "Run EP-3 & the tests + 1",
            Path::new(r"C:\Tools\VS Code\Code.exe"),
            &locks,
        );
        launch.ep = Some("EP-3".to_owned());
        launch.wait = Duration::from_millis(600);
        launch.settle = Duration::ZERO;
        launch.ledger = Some(scratch.0.join("ledger.sqlite"));
        let fake = Fake {
            calls: Vec::new(),
            locks,
            folder: folder.to_string_lossy().to_lowercase(),
            window_appears,
        };
        Setup {
            scratch,
            launch,
            fake,
        }
    }

    #[test]
    fn a_launch_opens_the_workspace_file_waits_for_the_window_then_hands_over_the_link() {
        let Setup {
            scratch,
            launch,
            mut fake,
        } = setup("visual-launch", true);
        let opened = open_with(&launch, &AtomicBool::new(false), &mut fake).unwrap();
        let own = scratch.0.join("data").join("alpha.code-workspace");
        assert_eq!(opened.workspace, OwnFile::Written(own.clone()));
        assert_eq!(
            fs::read_to_string(&own).unwrap(),
            workspace::single_folder("alpha")
        );
        assert!(!opened.was_open);
        assert_eq!(
            fake.calls,
            [
                vec![own.to_string_lossy().into_owned()],
                vec![
                    "--open-url".to_owned(),
                    "--".to_owned(),
                    encode("Run EP-3 & the tests + 1")
                ],
            ]
        );
        // The ledger keeps the launch as visual, with nothing VS Code chooses.
        let rows = Ledger::open(launch.ledger.as_deref().unwrap())
            .unwrap()
            .sessions()
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(opened.ledger_row, Some(rows[0].id));
        assert_eq!(
            (
                rows[0].mode.as_str(),
                rows[0].project.as_str(),
                rows[0].ep.as_deref()
            ),
            ("visual", "alpha", Some("EP-3"))
        );
        assert_eq!(
            (&rows[0].session_id, &rows[0].model, &rows[0].effort),
            (&None, &None, &None)
        );
        assert_eq!(rows[0].outcome, None);
    }

    #[test]
    fn an_open_window_and_an_existing_workspace_file_are_used_as_they_are() {
        let Setup {
            scratch,
            mut launch,
            mut fake,
        } = setup("visual-open", false);
        launch.ledger = None;
        let own = scratch.0.join("data").join("alpha.code-workspace");
        fs::write(
            &own,
            "{ \"folders\": [{ \"path\": \"alpha\" }] } // the owner's",
        )
        .unwrap();
        lock(
            &launch.locks,
            "41000.lock",
            std::process::id(),
            &[launch.folder.to_string_lossy().as_ref()],
        );
        let opened = open_with(&launch, &AtomicBool::new(false), &mut fake).unwrap();
        assert_eq!(opened.workspace, OwnFile::Kept(own.clone()));
        assert!(
            fs::read_to_string(&own)
                .unwrap()
                .ends_with("// the owner's")
        );
        assert!(opened.was_open);
        assert_eq!(opened.ledger_row, None);
        // The file is opened all the same: it brings the window to the front.
        assert_eq!(fake.calls.len(), 2);
        assert_eq!(fake.calls[0], [own.to_string_lossy().into_owned()]);
    }

    #[test]
    fn nothing_is_handed_over_without_the_window_or_after_a_cancel() {
        // A stale lock file names the project, and no window shows up. (The scratch folder is
        // bound, or it would go with the pattern's `..` at once.)
        let Setup {
            scratch: _none,
            launch,
            mut fake,
        } = setup("visual-none", false);
        lock(
            &launch.locks,
            "41000.lock",
            u32::MAX - 2,
            &[launch.folder.to_string_lossy().as_ref()],
        );
        let why = open_with(&launch, &AtomicBool::new(false), &mut fake).unwrap_err();
        assert!(why.contains("within 0 s"), "{why}");
        assert_eq!(fake.calls.len(), 1, "only the workspace file was opened");
        assert!(
            Ledger::open(launch.ledger.as_deref().unwrap())
                .unwrap()
                .sessions()
                .unwrap()
                .is_empty()
        );

        let Setup {
            scratch: _cancel,
            launch,
            mut fake,
        } = setup("visual-cancel", false);
        let why = open_with(&launch, &AtomicBool::new(true), &mut fake).unwrap_err();
        assert_eq!(why, CANCELLED);
        assert_eq!(fake.calls.len(), 1);

        // A prompt too long for a link stops the launch before anything is written or started.
        let Setup {
            scratch,
            mut launch,
            mut fake,
        } = setup("visual-long", true);
        launch.prompt = " ".repeat(MAX_LINK);
        assert!(
            open_with(&launch, &AtomicBool::new(false), &mut fake)
                .unwrap_err()
                .contains("characters")
        );
        assert!(fake.calls.is_empty());
        assert!(!scratch.0.join("data").join("alpha.code-workspace").exists());
    }

    #[test]
    fn readiness_says_what_a_launch_would_meet() {
        let scratch = Scratch::new("visual-readiness");
        let folder = scratch.0.join("alpha");
        fs::create_dir_all(&folder).unwrap();
        let window = Window {
            pid: 7,
            folders: vec![folder.clone()],
            ide: None,
            alive: true,
        };
        let code = PathBuf::from(r"C:\Tools\VS Code\Code.exe");
        let missing = Readiness::of(&folder, Some(code.clone()), None, &[]);
        assert_eq!(
            missing.workspace.as_deref(),
            Some(scratch.0.join("alpha.code-workspace").as_path())
        );
        assert!(!missing.workspace_exists && !missing.window_open);
        fs::write(
            scratch.0.join("alpha.code-workspace"),
            workspace::single_folder("alpha"),
        )
        .unwrap();
        let ready = Readiness::of(
            &folder,
            Some(code),
            Some(Version::new(2, 1, 314)),
            &[window],
        );
        assert!(ready.workspace_exists && ready.window_open);
    }
}
