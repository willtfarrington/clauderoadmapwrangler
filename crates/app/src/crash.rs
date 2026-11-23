//! The window's crash log (EP-14a, D-14 addendum): a release build has no console, so a panic
//! would end the window without a word. The hook appends each panic's report to `crash.log` in the
//! local folder (`%LOCALAPPDATA%\clauderoadmapwrangler`, or `CRW_HOME`), and the next start names
//! the last report in the status bar, once. The report's form and its reader are
//! `crw_core::doctor`'s, which the Diagnostics tile and `crw doctor` read too (EP-28); a panic
//! also leaves a line in the rolling log.
//!
//! A renderer's panic starts the window again (the night of 2026-11-23: the graphics driver's
//! reset lost the device, and egui-wgpu panics on a lost device rather than recovering). A panic
//! on the main thread whose place is in a wgpu crate starts the window's exe with
//! `--restarted <pid> --after-panic`, and `--hidden` when the window was hidden in the tray, so
//! the new window waits for this one to end as a restart's does. At most once in
//! [`AGAIN_GAP_MINUTES`]: a renderer that panics at each start is left closed, and the rolling log
//! says why. Any other panic ends the window as before.

use std::backtrace::Backtrace;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{DateTime, Local, SecondsFormat, TimeDelta, Utc};
use crw_core::doctor::{CRASH_FILE, CRASH_HEAD as HEAD, CRASH_SEP as SEP, last_crash};
use crw_core::install;

/// The log's name in the local folder.
pub const FILE: &str = CRASH_FILE;
/// Where a full log goes.
const OLD: &str = "crash.old.log";
/// Beside the log: the time of the last report the status bar named.
const SEEN: &str = "crash.seen";
/// Beside the log: the time of the last panic that started the window again, the same as its
/// report's.
const AGAIN: &str = "crash.again";
/// Past this size the log moves to [`OLD`] and a new one starts.
const CAP: u64 = 1 << 20;
/// A renderer's panic starts the window again at most once in this many minutes.
const AGAIN_GAP_MINUTES: i64 = 10;
/// Beside [`install::RESTARTED_FLAG`]: the old window ended on a panic and started this one.
pub const AFTER_PANIC_FLAG: &str = "--after-panic";
/// Beside [`AFTER_PANIC_FLAG`]: the old window was hidden in the tray, and this one opens there.
pub const HIDDEN_FLAG: &str = "--hidden";

/// Whether the window is hidden in the tray, which each pass mirrors here for the hook.
static HIDDEN: AtomicBool = AtomicBool::new(false);

/// Mirrors the window's hidden state for a start after a panic (each pass calls it).
pub fn set_hidden(hidden: bool) {
    HIDDEN.store(hidden, Ordering::Relaxed);
}

/// Installs the hook: each panic, on any thread, appends its report to `file` and a line to the
/// rolling log; a renderer's panic starts `again` (the window's exe) when it is given; then the
/// hook that was there runs as before (a debug build's console still shows the panic). Tests
/// give no exe, so no test starts a window.
pub fn install(file: PathBuf, again: Option<PathBuf>) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let thread = thread.name().unwrap_or("unnamed");
        let (place, message) = (place(info), message(info));
        let now = Utc::now();
        let text = report(
            now,
            thread,
            &place,
            &message,
            &Backtrace::force_capture().to_string(),
        );
        let _ = append(&file, &text, CAP);
        crw_core::log::try_line(
            "panic",
            &format!("{message} ({place}); {} has the report", file.display()),
        );
        if let Some(exe) = &again {
            start_again_if_due(&file, exe, thread, &place, now);
        }
        previous(info);
    }));
}

/// What a panic does about the window: nothing, or start it again, or not, since it started
/// again a moment ago (the time, in words).
#[derive(Debug, PartialEq, Eq)]
enum Again {
    No,
    TooSoon(String),
    Yes,
}

/// Whether a panic on `thread` at `place` starts the window again, the last such start at `last`
/// (the [`AGAIN`] file's text): only the main thread's, since a panic elsewhere leaves the window
/// up, only a renderer's, and not within [`AGAIN_GAP_MINUTES`] of the last start.
fn again(thread: &str, place: &str, last: Option<&str>, now: DateTime<Utc>) -> Again {
    if thread != "main" || !renderer_place(place) {
        return Again::No;
    }
    let last = last.and_then(|text| DateTime::parse_from_rfc3339(text.trim()).ok());
    if let Some(at) = last.map(|at| at.to_utc())
        && at <= now
        && now - at < TimeDelta::minutes(AGAIN_GAP_MINUTES)
    {
        return Again::TooSoon(at.with_timezone(&Local).format("%H:%M").to_string());
    }
    Again::Yes
}

/// Whether a panic's place is in a wgpu crate (`egui-wgpu-0.37.0`, `wgpu-27.0.1`,
/// `wgpu-core-27.0.0`, … in the registry's folder names): the renderer's.
fn renderer_place(place: &str) -> bool {
    place.split(['\\', '/']).any(|part| {
        let part = part.strip_prefix("egui-").unwrap_or(part);
        let Some(rest) = part.strip_prefix("wgpu-") else {
            return false;
        };
        // `core-27.0.0`: a sub-crate's name before the version.
        let version = rest
            .split_once('-')
            .filter(|(name, _)| !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase()))
            .map_or(rest, |(_, version)| version);
        version.starts_with(|c: char| c.is_ascii_digit())
    })
}

/// The new window's arguments: a restart from process `pid`, after a panic, hidden or not.
fn again_args(pid: u32, hidden: bool) -> Vec<String> {
    let mut args = vec![
        install::RESTARTED_FLAG.to_owned(),
        pid.to_string(),
        AFTER_PANIC_FLAG.to_owned(),
    ];
    if hidden {
        args.push(HIDDEN_FLAG.to_owned());
    }
    args
}

/// Starts `exe` again when [`again`] says so, marks the time beside `file`, and says what it did
/// in the rolling log.
fn start_again_if_due(file: &Path, exe: &Path, thread: &str, place: &str, now: DateTime<Utc>) {
    let mark = file.with_file_name(AGAIN);
    let last = fs::read_to_string(&mark).ok();
    match again(thread, place, last.as_deref(), now) {
        Again::No => {}
        Again::TooSoon(at) => crw_core::log::try_line(
            "panic",
            &format!(
                "the window is not started again: it started again after a renderer's panic at \
                 {at}, under {AGAIN_GAP_MINUTES} minutes ago"
            ),
        ),
        Again::Yes => {
            let _ = fs::write(&mark, now.to_rfc3339_opts(SecondsFormat::Secs, true));
            let hidden = HIDDEN.load(Ordering::Relaxed);
            let words = match start(exe, &again_args(std::process::id(), hidden)) {
                Ok(pid) => format!(
                    "the window starts again after the renderer's panic: {} as process {pid}{}",
                    exe.display(),
                    if hidden { ", hidden in the tray" } else { "" }
                ),
                Err(e) => format!("the window could not start again: {e}"),
            };
            crw_core::log::try_line("panic", &words);
        }
    }
}

/// Starts `exe` with `args` from its own folder, detached from this process's streams.
fn start(exe: &Path, args: &[String]) -> io::Result<u32> {
    let mut command = Command::new(exe);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(folder) = exe.parent() {
        command.current_dir(folder);
    }
    command.spawn().map(|child| child.id())
}

/// The panic's message: its payload when that is text.
fn message(info: &PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "(a panic without a message)".to_owned())
}

fn place(info: &PanicHookInfo<'_>) -> String {
    info.location().map_or_else(
        || "an unknown place".to_owned(),
        |at| format!("{}:{}:{}", at.file(), at.line(), at.column()),
    )
}

/// One report: the head line, the message, the backtrace.
fn report(at: DateTime<Utc>, thread: &str, place: &str, message: &str, backtrace: &str) -> String {
    let at = at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let version = env!("CARGO_PKG_VERSION");
    format!(
        "{HEAD}{version}{SEP}{at}{SEP}thread '{thread}'{SEP}{place}\n{message}\nbacktrace:\n{backtrace}\n\n"
    )
}

/// Appends `report` to `file`; a log past `cap` moves to [`OLD`] first.
fn append(file: &Path, report: &str, cap: u64) -> io::Result<()> {
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    if fs::metadata(file).is_ok_and(|meta| meta.len() > cap) {
        fs::rename(file, file.with_file_name(OLD))?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)?
        .write_all(report.as_bytes())
}

/// The status bar's word on the last report in `file`, once: none without one, or when the
/// window named it already (its time is kept in [`SEEN`]).
pub fn unseen(file: &Path) -> Option<String> {
    let last = last_crash(&fs::read_to_string(file).ok()?)?;
    let mark = file.with_file_name(SEEN);
    if fs::read_to_string(&mark).is_ok_and(|seen| seen.trim() == last.at) {
        return None;
    }
    let _ = fs::write(&mark, &last.at);
    // The panic that started the window again marks its report's time; said first, since the
    // status bar cuts a long notice.
    let started_again =
        fs::read_to_string(file.with_file_name(AGAIN)).is_ok_and(|at| at.trim() == last.at);
    let again = if started_again {
        " and started again on its own"
    } else {
        ""
    };
    // A library's place is a long registry path: its file name and line say enough.
    Some(format!(
        "The window closed on a panic at {}{again}: {} ({}); {} has the report.",
        last.when(),
        last.message,
        last.short_place(),
        file.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A fresh folder under the temp folder, removed when dropped.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            static COUNT: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "crw-crash-{}-{}",
                std::process::id(),
                COUNT.fetch_add(1, Ordering::Relaxed)
            ));
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

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text).unwrap().to_utc()
    }

    #[test]
    fn the_last_report_reads_back_with_its_first_line_of_message() {
        let log = [
            report(
                at("2026-11-03T17:40:00Z"),
                "main",
                "a.rs:1:2",
                "first",
                "bt",
            ),
            report(
                at("2026-11-03T17:47:53Z"),
                "session x",
                r"C:\src\board.rs:1043:39",
                "overflow when subtracting durations\nmore",
                "  0: frame",
            ),
        ]
        .concat();
        let last = last_crash(&log).unwrap();
        assert_eq!(last.at, "2026-11-03T17:47:53Z");
        assert_eq!(last.place, r"C:\src\board.rs:1043:39");
        assert_eq!(last.message, "overflow when subtracting durations");
        assert_eq!(last_crash("no report here"), None);
    }

    #[test]
    fn the_status_bar_names_a_report_once_and_a_newer_one_again() {
        let scratch = Scratch::new();
        let file = scratch.0.join(FILE);
        assert_eq!(unseen(&file), None, "no log, nothing to say");
        let one = report(
            at("2026-11-03T17:47:53Z"),
            "main",
            r"C:\x\board.rs:1043:39",
            "boom — é",
            "",
        );
        append(&file, &one, CAP).unwrap();
        let said = unseen(&file).unwrap();
        assert!(said.contains("boom — é (board.rs:1043:39)"), "{said}");
        assert!(said.contains(&file.display().to_string()), "{said}");
        assert_eq!(unseen(&file), None, "named once");
        let two = report(at("2026-11-03T18:00:00Z"), "main", "c.rs:9:1", "again", "");
        append(&file, &two, CAP).unwrap();
        assert!(unseen(&file).unwrap().contains("again (c.rs:9:1)"));
    }

    /// The night's place, as the crash log wrote it.
    const RENDERER: &str = r"C:\Users\someone\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\egui-wgpu-0.37.0\src\renderer.rs:1033:17";

    #[test]
    fn a_place_in_a_wgpu_crate_is_the_renderers() {
        assert!(renderer_place(RENDERER));
        assert!(renderer_place(
            "/home/x/.cargo/registry/src/i/wgpu-27.0.1/src/backend/wgpu_core.rs:1:1"
        ));
        assert!(renderer_place(
            r"C:\r\wgpu-core-27.0.0\src\device\queue.rs:9:9"
        ));
        assert!(renderer_place(
            r"C:\r\wgpu-hal-27.0.0-rc.1\src\dx12\mod.rs:9:9"
        ));
        // egui's own panic (the popup's, 2026-11-04), the app's, and look-alike names.
        assert!(!renderer_place(
            r"C:\r\index.crates.io-1\egui-0.37.0\src\context.rs:4714:17"
        ));
        assert!(!renderer_place(r"crates\app\src\board.rs:1043:39"));
        assert!(!renderer_place(r"C:\src\mywgpu-0.1\src\a.rs:1:1"));
        assert!(!renderer_place(r"C:\src\wgpu-tools\src\a.rs:1:1"));
        assert!(!renderer_place("an unknown place"));
    }

    #[test]
    fn only_the_main_threads_renderer_panic_starts_the_window_again_once_in_ten_minutes() {
        let now = at("2026-11-23T09:00:03Z");
        assert_eq!(again("main", RENDERER, None, now), Again::Yes);
        assert_eq!(again("session x", RENDERER, None, now), Again::No);
        assert_eq!(
            again("main", r"crates\app\src\board.rs:1:1", None, now),
            Again::No
        );
        assert!(matches!(
            again("main", RENDERER, Some("2026-11-23T08:55:00Z\n"), now),
            Again::TooSoon(_)
        ));
        assert_eq!(
            again("main", RENDERER, Some("2026-11-23T08:50:02Z"), now),
            Again::Yes
        );
        // A mark from a clock that ran ahead, or none legible, holds nothing back.
        assert_eq!(
            again("main", RENDERER, Some("2026-11-23T09:05:00Z"), now),
            Again::Yes
        );
        assert_eq!(again("main", RENDERER, Some("garbage"), now), Again::Yes);
    }

    #[test]
    fn the_new_window_reads_a_restart_after_a_panic_hidden_or_not() {
        let shown = again_args(4242, false);
        assert_eq!(install::restarted_from(&shown), Some(4242));
        assert!(shown.iter().any(|arg| arg == AFTER_PANIC_FLAG));
        assert!(!shown.iter().any(|arg| arg == HIDDEN_FLAG));
        let hidden = again_args(4242, true);
        assert_eq!(install::restarted_from(&hidden), Some(4242));
        assert!(hidden.iter().any(|arg| arg == HIDDEN_FLAG));
    }

    #[test]
    fn the_status_bar_says_first_that_the_window_started_again() {
        let scratch = Scratch::new();
        let file = scratch.0.join(FILE);
        let one = report(at("2026-11-23T09:00:03Z"), "main", RENDERER, "lost", "");
        append(&file, &one, CAP).unwrap();
        fs::write(scratch.0.join(AGAIN), "2026-11-23T09:00:03Z").unwrap();
        let said = unseen(&file).unwrap();
        assert!(
            said.contains(" and started again on its own: lost (renderer.rs:1033:17)"),
            "{said}"
        );
        // A later panic that did not start it again says nothing of it.
        let two = report(at("2026-11-23T10:00:00Z"), "main", "c.rs:9:1", "other", "");
        append(&file, &two, CAP).unwrap();
        let said = unseen(&file).unwrap();
        assert!(!said.contains("started again"), "{said}");
    }

    #[test]
    fn a_full_log_moves_aside() {
        let scratch = Scratch::new();
        let file = scratch.0.join(FILE);
        append(&file, &"x".repeat(64), 32).unwrap();
        append(&file, "fresh", 32).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "fresh");
        assert_eq!(fs::read_to_string(scratch.0.join(OLD)).unwrap().len(), 64);
    }

    #[test]
    fn the_hook_writes_a_panicking_threads_report() {
        let scratch = Scratch::new();
        let file = scratch.0.join(FILE);
        let before = std::panic::take_hook();
        install(file.clone(), None);
        let joined = std::thread::Builder::new()
            .name("probe".to_owned())
            .spawn(|| panic!("probe panic {}", 7))
            .unwrap()
            .join();
        let _ = std::panic::take_hook();
        std::panic::set_hook(before);
        assert!(joined.is_err());
        let log = fs::read_to_string(&file).unwrap();
        assert!(log.starts_with(HEAD), "{log}");
        assert!(log.contains("thread 'probe'"), "{log}");
        assert!(log.contains("crash.rs:"), "{log}");
        assert_eq!(last_crash(&log).unwrap().message, "probe panic 7");
    }
}
