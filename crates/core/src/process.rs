//! Short-lived child processes (D-29): a tool asked one question, one at a time with a short
//! spacing and a timeout, with no console window and without `ANTHROPIC_API_KEY` in its
//! environment (D-15). `doctor` asks for versions, the registry view for the last commit, the
//! badges git and `gh` (EP-7), the session runner's preflight the CLI (EP-9).

use std::ffi::{OsStr, OsString};
use std::io::Read;
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
    output(command, args, timeout)
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
    output(command, args, timeout)
}

fn output<I, S>(mut command: Command, args: I, timeout: Duration) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    command
        .args(args)
        .env_remove("ANTHROPIC_API_KEY")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: no console flashes when the desktop app runs the checks (EP-28).
        command.creation_flags(0x0800_0000);
    }
    let mut child = command.spawn().map_err(|e| format!("did not start: {e}"))?;
    let mut stdout = child.stdout.take().expect("stdout is piped");
    // Read on a thread so a chatty child cannot fill the pipe and stall before it exits.
    let reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stdout.read_to_end(&mut bytes);
        String::from_utf8_lossy(&bytes).into_owned()
    });
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
    let text = reader.join().unwrap_or_default();
    if status.success() {
        Ok(text)
    } else {
        Err(format!("exited with {status}"))
    }
}
