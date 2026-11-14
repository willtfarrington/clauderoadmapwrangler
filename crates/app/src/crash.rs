//! The window's crash log (EP-14a, D-14 addendum): a release build has no console, so a panic
//! would end the window without a word. The hook appends each panic's report to `crash.log` in the
//! local folder (`%LOCALAPPDATA%\clauderoadmapwrangler`, or `CRW_HOME`), and the next start names
//! the last report in the status bar, once. The report's form and its reader are
//! `crw_core::doctor`'s, which the Diagnostics tile and `crw doctor` read too (EP-28); a panic
//! also leaves a line in the rolling log.

use std::backtrace::Backtrace;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};

use chrono::{DateTime, SecondsFormat, Utc};
use crw_core::doctor::{CRASH_FILE, CRASH_HEAD as HEAD, CRASH_SEP as SEP, last_crash};

/// The log's name in the local folder.
pub const FILE: &str = CRASH_FILE;
/// Where a full log goes.
const OLD: &str = "crash.old.log";
/// Beside the log: the time of the last report the status bar named.
const SEEN: &str = "crash.seen";
/// Past this size the log moves to [`OLD`] and a new one starts.
const CAP: u64 = 1 << 20;

/// Installs the hook: each panic, on any thread, appends its report to `file` and a line to the
/// rolling log, then the hook that was there runs as before (a debug build's console still shows
/// the panic).
pub fn install(file: PathBuf) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let (place, message) = (place(info), message(info));
        let text = report(
            Utc::now(),
            thread.name().unwrap_or("unnamed"),
            &place,
            &message,
            &Backtrace::force_capture().to_string(),
        );
        let _ = append(&file, &text, CAP);
        crw_core::log::try_line(
            "panic",
            &format!("{message} ({place}); {} has the report", file.display()),
        );
        previous(info);
    }));
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
    // A library's place is a long registry path: its file name and line say enough.
    Some(format!(
        "The window closed on a panic at {}: {} ({}); {} has the report.",
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
        install(file.clone());
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
