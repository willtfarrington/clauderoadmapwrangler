//! The installed copy (D-29; EP-37): the release exes of the repository a build was compiled in,
//! copied from its `target\release` into `%LOCALAPPDATA%\clauderoadmapwrangler\bin` (or
//! `CRW_HOME\bin` in a trial), so that a release build never meets the running window or a hook:
//! the shortcuts, the hooks and the status line run the copy. Each exe is first copied beside its
//! place under a staging name, so that a copy that fails changes nothing; then the exe in place is
//! renamed to a dated previous copy, since Windows lets a running exe be renamed but not replaced,
//! and the staged copy takes its name. The previous copies of an earlier install are removed at
//! the next one, and one that still runs is left for the install after. A copy keeps its build's
//! last write, so an exe the folder holds as built is left alone. The window restarts into the
//! installed copy when it is newer than the build it runs ([`restart_due`]): it starts the copy
//! with [`RESTARTED_FLAG`] and its own process id, and the new window waits for the old one to end
//! before it opens. The window takes a build for its own install only once the build has settled
//! ([`build_ready`]), so that it never copies an exe the linker is still writing.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use crate::{paths, process};

/// The install's folder under the local folder.
pub const BIN_DIR: &str = "bin";
/// The window's exe and the console's, which the hooks and the status line run.
pub const WINDOW_EXE: &str = "clauderoadmapwrangler.exe";
pub const CLI_EXE: &str = "crw.exe";
/// What an install copies, the window first.
pub const EXES: [&str; 2] = [WINDOW_EXE, CLI_EXE];
/// A previous copy's mark between its exe's stem and its date: `crw.previous-20261110-081500.exe`.
const PREVIOUS: &str = ".previous-";
/// A staged copy's mark: `crw.installing.exe`, renamed into place once every copy is made.
const STAGED: &str = ".installing.exe";
/// How long a build's exes must have been left alone before the window installs them on its own:
/// a linker still writing an exe keeps its last write fresh.
pub const BUILD_SETTLE: Duration = Duration::from_secs(5);

/// The flag a restarted window is started with, followed by the old window's process id.
pub const RESTARTED_FLAG: &str = "--restarted";
/// How long a restarted window waits for the old one to end: its state saved, its port free.
pub const WAIT_FOR_OLD: Duration = Duration::from_secs(10);

/// `%LOCALAPPDATA%\clauderoadmapwrangler\bin`, or `CRW_HOME\bin`.
pub fn bin_dir() -> Option<PathBuf> {
    Some(paths::local_dir()?.join(BIN_DIR))
}

/// The release build of the repository this build was compiled in.
pub fn build_dir() -> PathBuf {
    paths::repository().join("target").join("release")
}

/// An exe as the file system gives it: its size and its last write, which a copy keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub len: u64,
    pub modified: SystemTime,
}

impl Stamp {
    /// The stamp of the file at `path`; none when there is no file there.
    pub fn of(path: &Path) -> Option<Stamp> {
        let meta = fs::metadata(path).ok().filter(fs::Metadata::is_file)?;
        Some(Stamp {
            len: meta.len(),
            modified: meta.modified().ok()?,
        })
    }

    /// The last write in local time, for the words: `2026-11-10 08:15`.
    pub fn when(&self) -> String {
        chrono::DateTime::<chrono::Local>::from(self.modified)
            .format("%Y-%m-%d %H:%M")
            .to_string()
    }
}

/// What an install did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub from: PathBuf,
    pub to: PathBuf,
    /// The exes copied, by name.
    pub copied: Vec<String>,
    /// The exes the folder held already as built.
    pub unchanged: Vec<String>,
    /// Each exe renamed before its copy, with its previous copy's name.
    pub renamed: Vec<(String, String)>,
    /// The previous copies of an earlier install, removed.
    pub removed: Vec<String>,
    /// The previous copies left in place, still running or held, with why.
    pub kept: Vec<(String, String)>,
}

impl Report {
    /// The report in lines, for `crw install` and the window's notice.
    pub fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        for name in &self.copied {
            match self.renamed.iter().find(|(exe, _)| exe == name) {
                Some((_, previous)) => {
                    lines.push(format!("{name} copied; the one before is now {previous}"));
                }
                None => lines.push(format!("{name} copied")),
            }
        }
        for name in &self.unchanged {
            lines.push(format!("{name} is installed as built already"));
        }
        for name in &self.removed {
            lines.push(format!(
                "{name} removed (an earlier install's previous copy)"
            ));
        }
        for (name, why) in &self.kept {
            lines.push(format!(
                "{name} kept: it could not be removed ({why}); the next install tries again"
            ));
        }
        lines
    }
}

/// Copies the build in `from` into `to` (EP-37). The previous copies of an earlier install are
/// removed where they can be. Each exe that differs from the one installed is copied beside its
/// place under a staging name, its last write kept: a copy that fails removes the staged ones and
/// changes nothing. Then each installed exe is renamed to a previous copy dated `stamp`
/// (`20261110-081500`) and the staged copy takes its name; a rename that fails puts the exe back,
/// so that the shortcut and the hooks still find one, and the error says what was installed.
pub fn install(from: &Path, to: &Path, stamp: &str) -> Result<Report, String> {
    for name in EXES {
        if Stamp::of(&from.join(name)).is_none() {
            return Err(format!(
                "{} is missing: build the release first (cargo build --release)",
                from.join(name).display()
            ));
        }
    }
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    let mut report = Report {
        from: from.to_path_buf(),
        to: to.to_path_buf(),
        ..Report::default()
    };
    for (name, path) in previous_copies(to) {
        match fs::remove_file(&path) {
            Ok(()) => report.removed.push(name),
            Err(e) => report.kept.push((name, e.to_string())),
        }
    }
    // The copies first, beside their places: one that fails leaves the folder as it was.
    let mut staged: Vec<(&str, PathBuf)> = Vec::new();
    for name in EXES {
        let (source, target) = (from.join(name), to.join(name));
        let copy = to.join(staged_name(name));
        // A staged copy an interrupted install left.
        let _ = fs::remove_file(&copy);
        let built = Stamp::of(&source).ok_or_else(|| format!("{} vanished", source.display()))?;
        if Stamp::of(&target) == Some(built) {
            report.unchanged.push(name.to_owned());
            continue;
        }
        if let Err(e) = copy_keeping_time(&source, &copy, built.modified) {
            let _ = fs::remove_file(&copy);
            remove_staged(&staged);
            return Err(format!(
                "{} could not be copied to {}: {e}; nothing was installed",
                source.display(),
                to.display()
            ));
        }
        staged.push((name, copy));
    }
    // Then each into its place: the installed exe renamed to a previous copy, which Windows
    // allows for a running exe, and the staged copy renamed to its name.
    for (at, (name, copy)) in staged.iter().enumerate() {
        let target = to.join(name);
        let previous = if target.exists() {
            let previous = previous_name(to, name, stamp);
            if let Err(e) = fs::rename(&target, to.join(&previous)) {
                remove_staged(&staged[at..]);
                let why = format!("{} could not be renamed: {e}", target.display());
                return Err(failed_at(&report, name, &why));
            }
            Some(previous)
        } else {
            None
        };
        if let Err(e) = fs::rename(copy, &target) {
            let mut why = format!("{} could not take its place: {e}", copy.display());
            if let Some(previous) = &previous
                && let Err(back) = fs::rename(to.join(previous), &target)
            {
                why.push_str(&format!(
                    "; and {previous} could not be put back as {name} ({back}): rename it by hand"
                ));
            }
            remove_staged(&staged[at..]);
            return Err(failed_at(&report, name, &why));
        }
        if let Some(previous) = previous {
            report.renamed.push(((*name).to_owned(), previous));
        }
        report.copied.push((*name).to_owned());
    }
    Ok(report)
}

/// The staging name of `exe`'s copy: `crw.installing.exe`.
fn staged_name(exe: &str) -> String {
    format!("{}{STAGED}", exe.trim_end_matches(".exe"))
}

/// Removes the staged copies of an install that stopped.
fn remove_staged(staged: &[(&str, PathBuf)]) {
    for (_, copy) in staged {
        let _ = fs::remove_file(copy);
    }
}

/// The error of an install that stopped at `name`: what it installed before, and why it stopped.
fn failed_at(report: &Report, name: &str, why: &str) -> String {
    if report.copied.is_empty() {
        format!("{name} was not installed, and nothing else was: {why}")
    } else {
        format!(
            "{} installed, but {name} was not: {why}",
            report.copied.join(" and ")
        )
    }
}

/// Copies `source` to `target` and gives the copy the build's last write, whatever the copy
/// itself kept.
fn copy_keeping_time(source: &Path, target: &Path, modified: SystemTime) -> std::io::Result<()> {
    fs::copy(source, target)?;
    fs::File::options()
        .write(true)
        .open(target)?
        .set_modified(modified)
}

/// The previous copies an earlier install left in `folder`, by name, oldest name first.
pub fn previous_copies(folder: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut found: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            is_previous(&name).then(|| (name, entry.path()))
        })
        .collect();
    found.sort();
    found
}

/// Whether `name` is a previous copy of one of [`EXES`].
fn is_previous(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    EXES.iter().any(|exe| {
        let stem = exe.trim_end_matches(".exe");
        name.strip_prefix(stem)
            .and_then(|rest| rest.strip_prefix(PREVIOUS))
            .is_some_and(|rest| rest.len() > ".exe".len() && rest.ends_with(".exe"))
    })
}

/// A free name for `exe`'s previous copy in `folder`, dated `stamp`.
fn previous_name(folder: &Path, exe: &str, stamp: &str) -> String {
    let stem = exe.trim_end_matches(".exe");
    let mut name = format!("{stem}{PREVIOUS}{stamp}.exe");
    let mut n = 2;
    while folder.join(&name).exists() {
        name = format!("{stem}{PREVIOUS}{stamp}-{n}.exe");
        n += 1;
    }
    name
}

/// Whether some exe of the build in `from` is newer than its installed copy in `to`, or not
/// installed: what an install would copy.
pub fn build_newer(from: &Path, to: &Path) -> bool {
    EXES.iter().any(
        |name| match (Stamp::of(&from.join(name)), Stamp::of(&to.join(name))) {
            (Some(built), Some(copy)) => built.modified > copy.modified,
            (Some(_), None) => true,
            (None, _) => false,
        },
    )
}

/// Whether the window may install the build in `from` on its own at `now` (its Install and
/// restart, its chip): the build is newer than the copy in `to` ([`build_newer`]), and each of
/// its exes was last written at least [`BUILD_SETTLE`] before `now`, so that no linker still
/// writes one. `crw install`, which the script runs once the build has ended, does not wait.
pub fn build_ready(from: &Path, to: &Path, now: SystemTime) -> bool {
    build_newer(from, to)
        && EXES.iter().all(|name| {
            Stamp::of(&from.join(name)).is_some_and(|built| {
                now.duration_since(built.modified)
                    .is_ok_and(|age| age >= BUILD_SETTLE)
            })
        })
}

/// The running window's exe and its stamp at the start: the build it runs, whatever happens to
/// its file since (an install renames it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Running {
    pub exe: PathBuf,
    pub stamp: Option<Stamp>,
}

impl Running {
    /// This process's exe, read now: at the window's start.
    pub fn now() -> Option<Running> {
        let exe = std::env::current_exe().ok()?;
        Some(Running {
            stamp: Stamp::of(&exe),
            exe,
        })
    }
}

/// Whether the window should restart into the installed window at `installed`: it is newer than
/// the build the window runs, or the same build where the window does not run from (a window
/// started from `target\release` moves into the copy). An older copy never is.
pub fn restart_due(running: &Running, installed: &Path) -> bool {
    let Some(copy) = Stamp::of(installed) else {
        return false;
    };
    match running.stamp {
        Some(stamp) => {
            copy.modified > stamp.modified
                || (copy == stamp && !paths::same(&running.exe, installed))
        }
        None => !paths::same(&running.exe, installed),
    }
}

/// The old window's process id, when the arguments say this window is a restart.
pub fn restarted_from<S: AsRef<str>>(args: &[S]) -> Option<u32> {
    let at = args.iter().position(|arg| arg.as_ref() == RESTARTED_FLAG)?;
    args.get(at + 1)?.as_ref().parse().ok()
}

/// Waits until the process `pid` has ended, at most `limit`: whether it did.
pub fn wait_for_end(pid: u32, limit: Duration) -> bool {
    let until = Instant::now() + limit;
    loop {
        if !process::running(pid) {
            return true;
        }
        if Instant::now() >= until {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    /// A build of both exes in `folder`, `body` their content, written at `modified`.
    fn build(folder: &Path, body: &str, modified: SystemTime) {
        fs::create_dir_all(folder).unwrap();
        for name in EXES {
            let path = folder.join(name);
            fs::write(&path, format!("{name}: {body}")).unwrap();
            fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(modified)
                .unwrap();
        }
    }

    fn at(minutes: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_800_000_000 + minutes * 60)
    }

    fn names(folder: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn a_first_install_copies_both_exes_with_their_time() {
        let scratch = Scratch::new("install-first");
        let (from, to) = (scratch.0.join("release"), scratch.0.join("bin"));
        build(&from, "one", at(0));
        let report = install(&from, &to, "20261110-080000").unwrap();
        assert_eq!(report.copied, vec![WINDOW_EXE, CLI_EXE]);
        assert!(report.renamed.is_empty() && report.removed.is_empty());
        assert_eq!(names(&to), vec![WINDOW_EXE, CLI_EXE]);
        for name in EXES {
            assert_eq!(
                Stamp::of(&to.join(name)),
                Stamp::of(&from.join(name)),
                "{name}"
            );
        }
        // The same build again changes nothing.
        let again = install(&from, &to, "20261110-080500").unwrap();
        assert!(again.copied.is_empty());
        assert_eq!(again.unchanged, vec![WINDOW_EXE, CLI_EXE]);
        assert_eq!(names(&to), vec![WINDOW_EXE, CLI_EXE]);
    }

    #[test]
    fn a_new_build_renames_the_installed_exes_and_the_next_install_removes_them() {
        let scratch = Scratch::new("install-rename");
        let (from, to) = (scratch.0.join("release"), scratch.0.join("bin"));
        build(&from, "one", at(0));
        install(&from, &to, "20261110-080000").unwrap();
        build(&from, "two", at(10));
        let report = install(&from, &to, "20261110-081000").unwrap();
        assert_eq!(report.copied, vec![WINDOW_EXE, CLI_EXE]);
        assert_eq!(
            report.renamed,
            vec![
                (
                    WINDOW_EXE.to_owned(),
                    "clauderoadmapwrangler.previous-20261110-081000.exe".to_owned()
                ),
                (
                    CLI_EXE.to_owned(),
                    "crw.previous-20261110-081000.exe".to_owned()
                ),
            ]
        );
        assert_eq!(
            fs::read_to_string(to.join(CLI_EXE)).unwrap(),
            "crw.exe: two"
        );
        assert_eq!(
            fs::read_to_string(to.join("crw.previous-20261110-081000.exe")).unwrap(),
            "crw.exe: one"
        );
        // The next install removes the previous copies first.
        build(&from, "three", at(20));
        let next = install(&from, &to, "20261110-082000").unwrap();
        assert_eq!(
            next.removed,
            vec![
                "clauderoadmapwrangler.previous-20261110-081000.exe",
                "crw.previous-20261110-081000.exe"
            ]
        );
        assert_eq!(
            names(&to),
            vec![
                WINDOW_EXE,
                "clauderoadmapwrangler.previous-20261110-082000.exe",
                CLI_EXE,
                "crw.previous-20261110-082000.exe",
            ]
        );
        assert!(
            next.lines().iter().any(|line| line
                == "crw.exe copied; the one before is now crw.previous-20261110-082000.exe")
        );
    }

    #[test]
    fn a_previous_copy_that_is_held_is_kept_for_the_next_install() {
        let scratch = Scratch::new("install-held");
        let (from, to) = (scratch.0.join("release"), scratch.0.join("bin"));
        build(&from, "one", at(0));
        install(&from, &to, "20261110-080000").unwrap();
        build(&from, "two", at(10));
        install(&from, &to, "20261110-081000").unwrap();
        // A running exe cannot be removed: an open handle without delete sharing stands in.
        let held = to.join("crw.previous-20261110-081000.exe");
        let handle = hold(&held);
        build(&from, "three", at(20));
        let report = install(&from, &to, "20261110-082000").unwrap();
        drop(handle);
        assert_eq!(
            report.removed,
            vec!["clauderoadmapwrangler.previous-20261110-081000.exe"]
        );
        assert_eq!(report.kept.len(), 1);
        assert_eq!(report.kept[0].0, "crw.previous-20261110-081000.exe");
        assert!(held.exists());
        // An exe renamed on the same second as a kept copy takes a free name.
        assert_eq!(
            previous_name(&to, CLI_EXE, "20261110-081000"),
            "crw.previous-20261110-081000-2.exe"
        );
    }

    /// Opens `path` so that it cannot be removed while the handle lives.
    #[cfg(windows)]
    fn hold(path: &Path) -> fs::File {
        use std::os::windows::fs::OpenOptionsExt;
        fs::File::options()
            .read(true)
            .share_mode(1) // FILE_SHARE_READ: no delete
            .open(path)
            .unwrap()
    }

    #[cfg(not(windows))]
    fn hold(path: &Path) -> fs::File {
        fs::File::open(path).unwrap()
    }

    #[test]
    fn a_missing_build_is_refused_and_nothing_is_written() {
        let scratch = Scratch::new("install-missing");
        let (from, to) = (scratch.0.join("release"), scratch.0.join("bin"));
        fs::create_dir_all(&from).unwrap();
        fs::write(from.join(WINDOW_EXE), "window").unwrap();
        let error = install(&from, &to, "20261110-080000").unwrap_err();
        assert!(error.contains("crw.exe is missing"), "{error}");
        assert!(!to.exists());
    }

    #[test]
    fn a_copy_that_fails_leaves_the_install_as_it_was() {
        let scratch = Scratch::new("install-copy-fails");
        let (from, to) = (scratch.0.join("release"), scratch.0.join("bin"));
        build(&from, "one", at(0));
        install(&from, &to, "20261110-080000").unwrap();
        build(&from, "two", at(10));
        // A folder where crw.exe's staged copy goes: its copy fails after the window's is made.
        fs::create_dir_all(to.join("crw.installing.exe")).unwrap();
        let error = install(&from, &to, "20261110-081000").unwrap_err();
        assert!(error.contains("nothing was installed"), "{error}");
        assert_eq!(
            names(&to),
            vec![WINDOW_EXE, CLI_EXE, "crw.installing.exe"],
            "the window's staged copy removed, no previous copy"
        );
        for name in EXES {
            assert_eq!(
                fs::read_to_string(to.join(name)).unwrap(),
                format!("{name}: one")
            );
        }
        // The next install, the folder gone, goes through.
        fs::remove_dir(to.join("crw.installing.exe")).unwrap();
        let report = install(&from, &to, "20261110-082000").unwrap();
        assert_eq!(report.copied, vec![WINDOW_EXE, CLI_EXE]);
    }

    /// An installed exe that cannot be renamed stops the install there: the window installed
    /// before it stays, the exe keeps its place, no staged copy is left, and the error says so.
    #[cfg(windows)]
    #[test]
    fn a_rename_that_fails_says_what_was_installed() {
        let scratch = Scratch::new("install-rename-fails");
        let (from, to) = (scratch.0.join("release"), scratch.0.join("bin"));
        build(&from, "one", at(0));
        install(&from, &to, "20261110-080000").unwrap();
        build(&from, "two", at(10));
        let handle = hold(&to.join(CLI_EXE));
        let error = install(&from, &to, "20261110-081000").unwrap_err();
        drop(handle);
        assert!(
            error.starts_with("clauderoadmapwrangler.exe installed, but crw.exe was not"),
            "{error}"
        );
        assert_eq!(
            fs::read_to_string(to.join(WINDOW_EXE)).unwrap(),
            "clauderoadmapwrangler.exe: two"
        );
        assert_eq!(
            fs::read_to_string(to.join(CLI_EXE)).unwrap(),
            "crw.exe: one"
        );
        assert_eq!(
            names(&to),
            vec![
                WINDOW_EXE,
                "clauderoadmapwrangler.previous-20261110-081000.exe",
                CLI_EXE,
            ]
        );
    }

    #[test]
    fn the_window_takes_a_build_only_once_it_has_settled() {
        let scratch = Scratch::new("install-settle");
        let (from, to) = (scratch.0.join("release"), scratch.0.join("bin"));
        build(&from, "one", at(0));
        let written = at(0);
        // Just written: a linker may still write it.
        assert!(!build_ready(&from, &to, written + Duration::from_secs(2)));
        assert!(build_ready(&from, &to, written + BUILD_SETTLE));
        // A clock behind the build's last write takes nothing.
        assert!(!build_ready(&from, &to, written - Duration::from_secs(60)));
        // Installed as built: nothing to take, however old.
        install(&from, &to, "20261110-080000").unwrap();
        assert!(!build_ready(
            &from,
            &to,
            written + Duration::from_secs(3600)
        ));
    }

    #[test]
    fn previous_copies_are_known_by_their_mark() {
        assert!(is_previous("crw.previous-20261110-081000.exe"));
        assert!(is_previous(
            "ClaudeRoadmapWrangler.previous-20261110-081000-2.exe"
        ));
        assert!(!is_previous("crw.exe"));
        assert!(!is_previous("crw.previous-.exe"));
        assert!(!is_previous("other.previous-20261110.exe"));
        assert!(!is_previous("crw.previous-20261110.txt"));
        // A staged copy is no previous copy.
        assert!(!is_previous(&staged_name(CLI_EXE)));
        assert_eq!(staged_name(CLI_EXE), "crw.installing.exe");
    }

    #[test]
    fn a_newer_build_or_a_missing_copy_is_one_to_install() {
        let scratch = Scratch::new("install-newer");
        let (from, to) = (scratch.0.join("release"), scratch.0.join("bin"));
        build(&from, "one", at(0));
        assert!(build_newer(&from, &to));
        install(&from, &to, "20261110-080000").unwrap();
        assert!(!build_newer(&from, &to));
        build(&from, "two", at(10));
        assert!(build_newer(&from, &to));
        // An older build is not one the window installs on its own.
        install(&from, &to, "20261110-081000").unwrap();
        build(&from, "zero", at(5));
        assert!(!build_newer(&from, &to));
        assert!(!build_newer(&scratch.0.join("nothing"), &to));
    }

    #[test]
    fn a_restart_is_due_for_a_newer_copy_or_the_same_build_elsewhere() {
        let scratch = Scratch::new("install-due");
        let (from, to) = (scratch.0.join("release"), scratch.0.join("bin"));
        build(&from, "one", at(0));
        let installed = to.join(WINDOW_EXE);
        let in_build = Running {
            exe: from.join(WINDOW_EXE),
            stamp: Stamp::of(&from.join(WINDOW_EXE)),
        };
        // Nothing installed: nothing to restart into.
        assert!(!restart_due(&in_build, &installed));
        install(&from, &to, "20261110-080000").unwrap();
        // The same build, installed where the window does not run from: moved into the copy.
        assert!(restart_due(&in_build, &installed));
        let in_copy = Running {
            exe: installed.clone(),
            stamp: Stamp::of(&installed),
        };
        assert!(!restart_due(&in_copy, &installed));
        // A newer copy, the old exe renamed under the running window.
        build(&from, "two", at(10));
        install(&from, &to, "20261110-081000").unwrap();
        assert!(restart_due(&in_copy, &installed));
        // An older copy never is.
        let newer = Running {
            exe: from.join(WINDOW_EXE),
            stamp: Some(Stamp {
                len: 1,
                modified: at(30),
            }),
        };
        assert!(!restart_due(&newer, &installed));
    }

    #[test]
    fn the_restart_flag_names_the_old_window() {
        assert_eq!(restarted_from(&["--restarted", "4242"]), Some(4242));
        assert_eq!(restarted_from(&["x", "--restarted", "7"]), Some(7));
        assert_eq!(restarted_from(&["--restarted"]), None);
        assert_eq!(restarted_from(&["--restarted", "me"]), None);
        assert_eq!(restarted_from::<&str>(&[]), None);
    }

    #[test]
    fn an_ended_process_is_not_waited_for() {
        // No process has the id 0 to wait on.
        assert!(wait_for_end(0, Duration::from_secs(1)));
    }
}
