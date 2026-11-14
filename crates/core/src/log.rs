//! The window's rolling log (D-29; EP-28): one file a local day in the local folder's `log`
//! folder (`CRW_HOME\log` in a trial), `crw-<date>.log`, each line the time, a kind and the words.
//! The files older than [`KEEP_DAYS`] are removed when the log starts and when the day turns, and
//! only files of that name. The window alone starts it ([`start`]); `crw` and the hooks never do,
//! a hook running in about 20 ms at every event, so [`line`] writes nothing there, nor in tests.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Local, NaiveDate, TimeDelta};

/// The log's folder in the local folder.
pub const FOLDER: &str = "log";
/// How many days of files are kept, today's included.
pub const KEEP_DAYS: i64 = 14;
const PREFIX: &str = "crw-";
const SUFFIX: &str = ".log";

/// A day's file name: `crw-2026-11-13.log`.
pub fn file_name(day: NaiveDate) -> String {
    format!("{PREFIX}{}{SUFFIX}", day.format("%Y-%m-%d"))
}

/// The day a file of the log's name is for.
fn day_of(name: &str) -> Option<NaiveDate> {
    let date = name.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// The log's day files in `dir` with their sizes, the newest first; none for a missing folder.
pub fn files(dir: &Path) -> Vec<(NaiveDate, PathBuf, u64)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<(NaiveDate, PathBuf, u64)> = entries
        .flatten()
        .filter_map(|entry| {
            let day = day_of(entry.file_name().to_str()?)?;
            let meta = entry.metadata().ok().filter(fs::Metadata::is_file)?;
            Some((day, entry.path(), meta.len()))
        })
        .collect();
    found.sort_by_key(|(day, _, _)| std::cmp::Reverse(*day));
    found
}

/// Removes the day files in `dir` older than [`KEEP_DAYS`] before `today` (today's file is the
/// first kept day), one at a time; any other file stays. How many were removed.
pub fn prune(dir: &Path, today: NaiveDate) -> usize {
    let oldest_kept = today - TimeDelta::days(KEEP_DAYS - 1);
    files(dir)
        .into_iter()
        .filter(|(day, _, _)| *day < oldest_kept)
        .filter(|(_, path, _)| fs::remove_file(path).is_ok())
        .count()
}

/// One line as written: the local time, the kind and the words on one line.
fn format_line(at: DateTime<Local>, kind: &str, text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    format!(
        "{}  {kind:<8} {}\n",
        at.format("%Y-%m-%d %H:%M:%S"),
        words.join(" ")
    )
}

/// A line written within this long is not written again, whatever came between: a notice a pass
/// sets again and again, or a child that fails at every poll, stays one line in five minutes.
const REPEAT: TimeDelta = TimeDelta::minutes(5);

/// The log of one process: its folder, the day it last wrote, so a new day prunes, and the lines
/// it wrote within [`REPEAT`].
#[derive(Debug)]
pub struct Log {
    dir: PathBuf,
    day: Option<NaiveDate>,
    recent: HashMap<(String, String), DateTime<Local>>,
}

impl Log {
    pub fn new(dir: PathBuf) -> Log {
        Log {
            dir,
            day: None,
            recent: HashMap::new(),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Appends one line at `at` to that day's file, creating the folder, and prunes the old files
    /// first when the day is not the one it last wrote. A line written within five minutes is
    /// left out.
    pub fn write(&mut self, at: DateTime<Local>, kind: &str, text: &str) -> io::Result<()> {
        self.recent.retain(|_, written| at - *written < REPEAT);
        let key = (kind.to_owned(), text.to_owned());
        if self.recent.contains_key(&key) {
            return Ok(());
        }
        self.recent.insert(key, at);
        let day = at.date_naive();
        fs::create_dir_all(&self.dir)?;
        if self.day != Some(day) {
            prune(&self.dir, day);
            self.day = Some(day);
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join(file_name(day)))?
            .write_all(format_line(at, kind, text).as_bytes())
    }
}

static LOG: Mutex<Option<Log>> = Mutex::new(None);

fn started() -> MutexGuard<'static, Option<Log>> {
    LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Starts this process's log in `dir`: from here [`line`] writes there (the window, at its start).
pub fn start(dir: PathBuf) {
    *started() = Some(Log::new(dir));
}

/// Writes one line to the started log, its failure ignored: the log never stops the window.
/// Nothing when no log is started (`crw`, the hooks, tests and screenshot runs).
pub fn line(kind: &str, text: &str) {
    if let Some(log) = started().as_mut() {
        let _ = log.write(Local::now(), kind, text);
    }
}

/// [`line`] for a panic's hook: it gives up when the log is busy rather than wait, so a panic
/// while a line is written never holds the process.
pub fn try_line(kind: &str, text: &str) {
    if let Ok(mut log) = LOG.try_lock()
        && let Some(log) = log.as_mut()
    {
        let _ = log.write(Local::now(), kind, text);
    }
}

/// The started log's folder, when one is.
pub fn folder() -> Option<PathBuf> {
    started().as_ref().map(|log| log.dir.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeZone;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("crw-test-{}-log-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn at(day: u32, hour: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(2026, 11, day, hour, 5, 9)
            .single()
            .unwrap()
    }

    #[test]
    fn a_line_goes_to_its_days_file_on_one_line() {
        let scratch = Scratch::new("write");
        let mut log = Log::new(scratch.0.join("log"));
        log.write(at(13, 23), "notice", "Read the files:\n  12 projects")
            .unwrap();
        // A line written within five minutes is left out, whatever came between; an hour later
        // it is written again.
        log.write(at(13, 23), "notice", "Read the files:\n  12 projects")
            .unwrap();
        log.write(at(14, 0), "warning", "the port is taken")
            .unwrap();
        log.write(at(14, 0), "spawn", "claude.exe did not start")
            .unwrap();
        log.write(at(14, 0), "warning", "the port is taken")
            .unwrap();
        log.write(at(14, 1), "warning", "the port is taken")
            .unwrap();
        let first = fs::read_to_string(scratch.0.join("log").join("crw-2026-11-13.log")).unwrap();
        assert_eq!(
            first,
            "2026-11-13 23:05:09  notice   Read the files: 12 projects\n"
        );
        let second = fs::read_to_string(scratch.0.join("log").join("crw-2026-11-14.log")).unwrap();
        assert_eq!(second.lines().count(), 3, "{second}");
        assert!(second.ends_with("warning  the port is taken\n"), "{second}");
        let listed: Vec<NaiveDate> = files(&scratch.0.join("log"))
            .into_iter()
            .map(|(day, _, _)| day)
            .collect();
        assert_eq!(
            listed,
            [
                NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
                NaiveDate::from_ymd_opt(2026, 11, 13).unwrap()
            ]
        );
    }

    #[test]
    fn the_turn_of_the_day_keeps_fourteen_days_and_other_files() {
        let scratch = Scratch::new("prune");
        let dir = scratch.0.join("log");
        fs::create_dir_all(&dir).unwrap();
        for day in 1..=13 {
            let name = file_name(NaiveDate::from_ymd_opt(2026, 11, day).unwrap());
            fs::write(dir.join(name), "x\n").unwrap();
        }
        fs::write(dir.join("notes.txt"), "kept").unwrap();
        fs::write(dir.join("crw-not-a-date.log"), "kept").unwrap();
        let mut log = Log::new(dir.clone());
        // The 14th keeps the 1st (fourteen days, today's included); the 15th drops it.
        log.write(at(14, 9), "start", "the window").unwrap();
        assert_eq!(files(&dir).len(), 14);
        log.write(at(15, 9), "start", "the window").unwrap();
        let kept = files(&dir);
        assert_eq!(kept.len(), 14);
        assert_eq!(
            kept.last().unwrap().0,
            NaiveDate::from_ymd_opt(2026, 11, 2).unwrap()
        );
        assert!(dir.join("notes.txt").is_file());
        assert!(dir.join("crw-not-a-date.log").is_file());
        // A second line the same day prunes nothing more.
        log.write(at(15, 10), "quit", "the window").unwrap();
        assert_eq!(files(&dir).len(), 14);
    }

    #[test]
    fn no_started_log_writes_nothing() {
        // Tests never start the process's log, so a line goes nowhere and nothing fails.
        assert!(folder().is_none());
        line("notice", "nothing is written");
    }
}
