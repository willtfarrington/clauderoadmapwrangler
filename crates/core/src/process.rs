//! Short-lived child processes (D-29): a tool asked one question, one at a time with a short
//! spacing and a timeout, with no console window and without `ANTHROPIC_API_KEY` in its
//! environment (D-15). `doctor` asks for versions, the registry view for the last commit, the
//! badges git and `gh` (EP-7), the session runner's preflight the CLI (EP-9), `crw manual` the
//! Typst CLI with the document on its standard input (EP-57).

use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

/// The pause before each child after the first: children start one at a time, never in a burst
/// (D-29).
pub const SPACING: Duration = Duration::from_millis(250);

/// A child that has not exited by then is killed and reported.
pub const TIMEOUT: Duration = Duration::from_secs(20);

static TURN: Mutex<()> = Mutex::new(());

/// The hint for a file Windows says security software flagged (its errors 225 and 226).
pub const FLAGGED: &str = "Windows says security software flagged the file: look in its quarantine";
/// The hint for access denied (error 5).
pub const DENIED: &str = "access was denied: security software may hold the file, or a build is \
     replacing it; if it lasts, look in the security software's quarantine";
/// The hint for a file named by its path that is not there (errors 2 and 3).
pub const GONE: &str = "the file is not there: if it was a moment ago, security software may \
     have quarantined it";

/// Why a child did not start, for the errors security software causes on this machine's two AV
/// products (D-29; EP-28): the file flagged, access denied, or a file named by its path gone. A
/// bare name not found on PATH is only missing.
pub fn hint(program: &Path, error: &std::io::Error) -> Option<&'static str> {
    match error.raw_os_error()? {
        225 | 226 => Some(FLAGGED),
        5 => Some(DENIED),
        2 | 3 if program.components().count() > 1 => Some(GONE),
        _ => None,
    }
}

/// A start's error in words, with [`hint`]'s when they apply; the window's rolling log keeps the
/// line (`crw` keeps none). The session runner and the visual launch word their own failures
/// around it.
pub fn explained(program: &Path, error: &std::io::Error) -> String {
    let words = match hint(program, error) {
        Some(hint) => format!("{error}; {hint}"),
        None => error.to_string(),
    };
    crate::log::line(
        "spawn",
        &format!("{} did not start: {words}", program.display()),
    );
    words
}

/// `did not start: <error>`, as [`explained`] words it.
pub fn did_not_start(program: &Path, error: &std::io::Error) -> String {
    format!("did not start: {}", explained(program, error))
}

/// Waits for this thread's turn to start children, then for the spacing: the threads that start
/// children (the dashboard's git and GitHub passes) never run two at once (D-29). Hold the guard
/// while the children run.
pub fn turn() -> MutexGuard<'static, ()> {
    let guard = TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    thread::sleep(SPACING);
    guard
}

/// Whether the process `pid` runs: a VS Code window's lock (EP-11), the window a restart waits
/// for (EP-37).
#[cfg(windows)]
pub fn running(pid: u32) -> bool {
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
pub fn running(_pid: u32) -> bool {
    true
}

/// The standard output of `program` run with `args`, if it exits successfully within `timeout`.
/// A bare program name is looked up on PATH.
pub fn output_of<I, S>(program: &Path, args: I, timeout: Duration) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    output_with(program, args, &[], timeout)
}

/// [`output_of`] with `env` added to the child's environment.
pub fn output_with<I, S>(
    program: &Path,
    args: I,
    env: &[(&str, &str)],
    timeout: Duration,
) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.envs(env.iter().copied());
    output(command, args, timeout, false)
}

/// [`output_with`] for a child whose failures differ only in their words (git's `show`, EP-20d):
/// a failure's reason ends with what the child printed on its standard error, its lines joined.
pub fn output_with_stderr<I, S>(
    program: &Path,
    args: I,
    env: &[(&str, &str)],
    timeout: Duration,
) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.envs(env.iter().copied());
    output(command, args, timeout, true)
}

/// [`output_of`] with the variables named in `remove` taken out of the child's environment too:
/// the session markers a Claude Code child must not inherit (EP-9).
pub fn output_without<I, S>(
    program: &Path,
    args: I,
    remove: &[OsString],
    timeout: Duration,
) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    for name in remove {
        command.env_remove(name);
    }
    output(command, args, timeout, false)
}

/// How a child ended (EP-34): its exit code (`None` when it has none) and what it printed on its
/// standard output and its standard error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exit {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Runs `program` with `args` and `env` added to its end within `timeout`, killing it then: how it
/// ended, whatever its exit code; `Err` only when it did not start or did not end in time. The
/// hosting actions' `gh` and `git` (EP-34), whose failure is shown in their own words.
pub fn exit_of<I, S>(
    program: &Path,
    args: I,
    env: &[(&str, &str)],
    timeout: Duration,
) -> Result<Exit, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.envs(env.iter().copied());
    exit(finish(command, args, timeout, true, None)?, false)
}

/// [`exit_of`] with `input` written to the child's standard input and the variables named in
/// `remove` taken out of its environment: the Typst CLI of `crw manual` (EP-57), which reads the
/// generated document from its standard input. Its standard error is kept after a success too,
/// for the warnings a successful build prints; such a child starts no grandchild that could hold
/// the pipe.
pub fn exit_with_input<I, S>(
    program: &Path,
    args: I,
    input: &str,
    remove: &[&str],
    timeout: Duration,
) -> Result<Exit, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    for name in remove {
        command.env_remove(name);
    }
    exit(finish(command, args, timeout, true, Some(input))?, true)
}

/// How a finished child ended. Its errors' words are read for a failure, as `output` reads them,
/// and after a success only when `always`: a grandchild may hold the pipe after a success.
fn exit(finished: Finished, always: bool) -> Result<Exit, String> {
    let Finished {
        status,
        stdout,
        stderr,
    } = finished;
    let stderr = match stderr {
        Some(reader) if always || !status.success() => reader.join().unwrap_or_default(),
        _ => String::new(),
    };
    Ok(Exit {
        code: status.code(),
        stdout,
        stderr,
    })
}

/// Starts `program` with `args` on its own in `folder` (EP-34's backup job, in its script's
/// folder): no console, nothing piped, so it goes on whatever becomes of the window; the caller
/// holds [`turn`] while it starts.
pub fn start_alone<I, S>(
    program: &Path,
    args: I,
    folder: Option<&Path>,
) -> Result<std::process::Child, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    if let Some(folder) = folder {
        command.current_dir(folder);
    }
    command
        .args(args)
        .env_remove("ANTHROPIC_API_KEY")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command.spawn().map_err(|e| did_not_start(program, &e))
}

/// [`start_alone`] for a program that reads its own command line, given one argument whole in
/// quotes (EP-57a): Explorer splits an unquoted argument at its commas. A Windows path holds no
/// `"`, so the quotes need no escape; the caller holds [`turn`] while it starts.
pub fn start_quoted(program: &Path, arg: &OsStr) -> Result<std::process::Child, String> {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut quoted = OsString::from("\"");
        quoted.push(arg);
        quoted.push("\"");
        command.raw_arg(quoted).creation_flags(0x0800_0000);
    }
    #[cfg(not(windows))]
    command.arg(arg);
    command
        .env_remove("ANTHROPIC_API_KEY")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.spawn().map_err(|e| did_not_start(program, &e))
}

/// Waits up to `timeout` for a child started on its own: its exit code (`None` when it has
/// none), or why it is not known; a child still running then is left to run, never killed.
pub fn wait_alone(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<Option<i32>, String> {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status.code()),
            Ok(None) if started.elapsed() >= timeout => {
                return Err(format!(
                    "still running after {} min: it goes on by itself",
                    timeout.as_secs() / 60
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(250)),
            Err(e) => return Err(format!("could not wait for it: {e}")),
        }
    }
}

/// Reads a child's pipe on a thread, so a chatty child cannot fill it and stall before it exits.
fn reader(mut pipe: impl Read + Send + 'static) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = pipe.read_to_end(&mut bytes);
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

fn output<I, S>(
    command: Command,
    args: I,
    timeout: Duration,
    keep_stderr: bool,
) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let Finished {
        status,
        stdout: text,
        stderr,
    } = finish(command, args, timeout, keep_stderr, None)?;
    if status.success() {
        return Ok(text);
    }
    let said = stderr
        .and_then(|reader| reader.join().ok())
        .map(|said| said.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|said| !said.is_empty());
    Err(match said {
        Some(said) => format!("exited with {status}: {said}"),
        None => format!("exited with {status}"),
    })
}

/// A child run to its end: its status, its standard output and, when kept, the reader of its
/// standard error, which a caller joins for a failure only.
struct Finished {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: Option<thread::JoinHandle<String>>,
}

/// Runs a child to its end within `timeout` (killed then), `input`, when given, written to its
/// standard input on a thread of its own and the pipe closed after it.
fn finish<I, S>(
    mut command: Command,
    args: I,
    timeout: Duration,
    keep_stderr: bool,
    input: Option<&str>,
) -> Result<Finished, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    command
        .args(args)
        .env_remove("ANTHROPIC_API_KEY")
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(if keep_stderr {
            Stdio::piped()
        } else {
            Stdio::null()
        });
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: no console flashes when the desktop app runs the checks (EP-28).
        command.creation_flags(0x0800_0000);
    }
    let mut child = command
        .spawn()
        .map_err(|e| did_not_start(Path::new(command.get_program()), &e))?;
    let stdout = reader(child.stdout.take().expect("stdout is piped"));
    let stderr = child.stderr.take().map(reader);
    // Written on a thread, so a child that prints before it has read everything cannot stall;
    // the pipe closes when the thread ends, which tells the child its input is complete. A child
    // that stops reading breaks the pipe and ends the thread.
    let writer = match (input, child.stdin.take()) {
        (Some(input), Some(mut pipe)) => {
            let bytes = input.as_bytes().to_vec();
            Some(thread::spawn(move || {
                let _ = pipe.write_all(&bytes);
            }))
        }
        _ => None,
    };
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                // The reader is left to finish on its own: a grandchild may still hold the pipe.
                return Err(format!("no answer within {timeout:?}"));
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(format!("could not wait for it: {e}")),
        }
    };
    if let Some(writer) = writer {
        let _ = writer.join();
    }
    Ok(Finished {
        status,
        stdout: stdout.join().unwrap_or_default(),
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Error;

    #[test]
    fn a_start_refused_by_security_software_says_where_to_look() {
        let path = Path::new(r"C:\Tools\tool.exe");
        let bare = Path::new("tool.exe");
        assert_eq!(hint(path, &Error::from_raw_os_error(225)), Some(FLAGGED));
        assert_eq!(hint(bare, &Error::from_raw_os_error(226)), Some(FLAGGED));
        assert_eq!(hint(path, &Error::from_raw_os_error(5)), Some(DENIED));
        assert_eq!(hint(path, &Error::from_raw_os_error(2)), Some(GONE));
        assert_eq!(hint(path, &Error::from_raw_os_error(3)), Some(GONE));
        // A bare name not on PATH is only missing; other errors carry no hint.
        assert_eq!(hint(bare, &Error::from_raw_os_error(2)), None);
        assert_eq!(hint(path, &Error::from_raw_os_error(193)), None);
        assert_eq!(hint(path, &Error::other("no code")), None);
        let words = did_not_start(path, &Error::from_raw_os_error(225));
        assert!(words.starts_with("did not start: "), "{words}");
        assert!(words.ends_with(FLAGGED), "{words}");
    }

    /// A child given its input on its standard input (EP-57): Windows' own `sort`, never `typst`
    /// or `claude` (rule 5), reads all of it, the pipe closed after it, and prints it sorted.
    #[cfg(windows)]
    #[test]
    fn a_childs_input_goes_through_its_standard_input() {
        let root = std::env::var_os("SystemRoot").expect("SystemRoot is set");
        let sort = Path::new(&root).join("System32").join("sort.exe");
        let lines: String = (0..5000)
            .rev()
            .map(|n| format!("line {n:05}\r\n"))
            .collect();
        let exit = exit_with_input(&sort, Vec::<&str>::new(), &lines, &["CRW_NONE"], TIMEOUT)
            .expect("sort runs");
        assert_eq!(exit.code, Some(0));
        let head: String = exit.stdout.chars().take(40).collect();
        assert!(
            exit.stdout.starts_with("line 00000\r\nline 00001\r\n"),
            "{head}"
        );
        assert_eq!(exit.stdout.lines().count(), 5000);
    }

    /// A child that ends without reading its input: the pipe breaks, the writer stops, and the
    /// child's exit code comes back without a hang.
    #[cfg(windows)]
    #[test]
    fn a_child_that_reads_none_of_its_input_ends_cleanly() {
        let root = std::env::var_os("SystemRoot").expect("SystemRoot is set");
        let cmd = Path::new(&root).join("System32").join("cmd.exe");
        let input = "x".repeat(4 << 20);
        let started = Instant::now();
        let exit =
            exit_with_input(&cmd, ["/d", "/c", "exit 3"], &input, &[], TIMEOUT).expect("cmd runs");
        assert_eq!(exit.code, Some(3));
        assert!(started.elapsed() < TIMEOUT, "it ended by itself");
    }

    /// A program named by a path that is not there: the real start's error carries the hint.
    #[cfg(windows)]
    #[test]
    fn a_missing_program_named_by_its_path_hints_at_the_quarantine() {
        let missing = std::env::temp_dir().join(format!("crw-absent-{}.exe", std::process::id()));
        let said = output_of(&missing, ["--version"], TIMEOUT).unwrap_err();
        assert!(said.starts_with("did not start: "), "{said}");
        assert!(said.ends_with(GONE), "{said}");
    }
}
