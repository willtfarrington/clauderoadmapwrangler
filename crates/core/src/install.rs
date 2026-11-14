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
//!
//! EP-28a adds `crw` on PATH (IN-3) and the shortcuts' icon. The copy of `crw.exe` on PATH sits
//! in [`path_dir`], the folder Claude Code's installer puts `claude.exe` in and on PATH; it is
//! placed, refreshed and removed the way an install copies ([`place_path_copy`],
//! [`remove_path_copy`]), and it touches nothing there but `crw.exe`, its staged copy and its
//! previous copies. [`write_icon`] writes the app's own icon beside the installed exes, which the
//! shortcuts show.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use crate::{files, icon, paths, process};

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

/// The folder under `CRW_HOME` that stands in for the PATH folder in a trial.
pub const TRIAL_PATH_DIR: &str = "path";

/// `%LOCALAPPDATA%\clauderoadmapwrangler\bin`, or `CRW_HOME\bin`.
pub fn bin_dir() -> Option<PathBuf> {
    Some(paths::local_dir()?.join(BIN_DIR))
}

/// The folder of `crw`'s copy on PATH (IN-3): `%USERPROFILE%\.local\bin`, where Claude Code's
/// installer puts `claude.exe` and which it puts on PATH; `CRW_HOME\path` in a trial.
pub fn path_dir() -> Option<PathBuf> {
    match paths::home() {
        Some(home) => Some(home.join(TRIAL_PATH_DIR)),
        None => std::env::var_os("USERPROFILE")
            .filter(|value| !value.is_empty())
            .map(|profile| PathBuf::from(profile).join(".local").join("bin")),
    }
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
    copy_exes(from, to, stamp, &EXES)
}

/// Places `crw`'s copy on PATH (IN-3; EP-28a): the installed `crw.exe` in `bin` copied into
/// `dir`, the folder made if it is missing, the way an install copies. A copy there that runs is
/// renamed to a previous copy, and the previous copies of an earlier time are removed. Nothing
/// else in `dir` is touched: it holds other programs.
pub fn place_path_copy(bin: &Path, dir: &Path, stamp: &str) -> Result<Report, String> {
    let installed = bin.join(CLI_EXE);
    if Stamp::of(&installed).is_none() {
        return Err(format!(
            "{} is missing: install first (pwsh -File scripts\\install-shortcut.ps1)",
            installed.display()
        ));
    }
    let copy = dir.join(CLI_EXE);
    if is_ours(&copy) == Some(false) {
        return Err(foreign(&copy));
    }
    copy_exes(bin, dir, stamp, &[CLI_EXE])
}

/// Whether the `crw.exe` at `path` is this tool's (EP-28a's review): its bytes name the tool's
/// folder, as every build of it does, and another program's `crw.exe` in the PATH folder (a
/// tool that uv or pipx put there) would not. None when the file does not read: absent, or held
/// by a process that shares no reading, which a later remove or rename then names.
fn is_ours(path: &Path) -> Option<bool> {
    let mark = paths::APP_DIR.as_bytes();
    fs::read(path)
        .ok()
        .map(|bytes| bytes.windows(mark.len()).any(|window| window == mark))
}

/// The words for another program's `crw.exe` in the PATH folder, which is left alone.
fn foreign(copy: &Path) -> String {
    format!(
        "{} is another program's crw.exe, left alone: this tool's copy cannot go there",
        copy.display()
    )
}

/// Removes `crw`'s copy on PATH from `dir`, with the previous copies of an earlier time and a
/// staged copy an interrupted pass left: whether there was a copy. A copy that runs cannot be
/// removed, so it is renamed to a previous copy dated `stamp`, which the next `crw path on` or
/// `crw path off` removes. Nothing else in `dir` is touched.
pub fn remove_path_copy(dir: &Path, stamp: &str) -> Result<bool, String> {
    for (_, path) in previous_copies(dir, &[CLI_EXE]) {
        // One that still runs stays for the next pass.
        let _ = fs::remove_file(path);
    }
    let _ = fs::remove_file(dir.join(staged_name(CLI_EXE)));
    let copy = dir.join(CLI_EXE);
    if Stamp::of(&copy).is_none() {
        return Ok(false);
    }
    if is_ours(&copy) == Some(false) {
        return Err(foreign(&copy));
    }
    let refused = match fs::remove_file(&copy) {
        Ok(()) => return Ok(true),
        Err(e) => e,
    };
    // Windows lets a running exe be renamed, not removed.
    let previous = dir.join(previous_name(dir, CLI_EXE, stamp));
    if let Err(e) = fs::rename(&copy, &previous) {
        return Err(format!(
            "{} could not be removed ({refused}) or renamed ({e})",
            copy.display()
        ));
    }
    let _ = fs::remove_file(&previous);
    Ok(true)
}

/// `crw`'s copy on PATH, set against the installed one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathCopy {
    Absent,
    /// The copy, as the installed build: the same size and last write.
    AsInstalled(PathBuf),
    /// The copy, of another build than the installed one, older or newer.
    Differs(PathBuf),
    /// Another program's `crw.exe`, which the tool leaves alone.
    Foreign(PathBuf),
}

/// The copy of `crw.exe` in `dir`, against the installed one in `bin`.
pub fn path_copy(bin: &Path, dir: &Path) -> PathCopy {
    let copy = dir.join(CLI_EXE);
    match Stamp::of(&copy) {
        None => PathCopy::Absent,
        Some(stamp) if Stamp::of(&bin.join(CLI_EXE)) == Some(stamp) => PathCopy::AsInstalled(copy),
        Some(_) if is_ours(&copy) == Some(false) => PathCopy::Foreign(copy),
        Some(_) => PathCopy::Differs(copy),
    }
}

/// Writes the app's own icon as [`icon::ICO_FILE`] in `to`, for the shortcuts (EP-28a): through a
/// temporary name and a rename, and only when the file there differs from the drawing; whether
/// it was written.
pub fn write_icon(to: &Path) -> Result<bool, String> {
    let bytes = icon::ico(&icon::ICO_SIZES);
    let path = to.join(icon::ICO_FILE);
    if fs::read(&path).is_ok_and(|there| there == bytes) {
        return Ok(false);
    }
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    files::write_atomic(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(true)
}

/// What follows an install of the exes into `bin` (EP-28a), in lines: the icon written beside
/// them when it differs, then `crw`'s copy on PATH in `path_dir` refreshed while there is one,
/// dated `stamp`. A failure of either is a warning line: the exes are installed all the same.
pub fn after_install(bin: &Path, path_dir: Option<&Path>, stamp: &str) -> Vec<String> {
    let mut lines = vec![match write_icon(bin) {
        Ok(true) => format!("{} written", icon::ICO_FILE),
        Ok(false) => format!("{} is as drawn already", icon::ICO_FILE),
        Err(e) => format!("warning: {} was not written: {e}", icon::ICO_FILE),
    }];
    if let Some(dir) = path_dir.filter(|dir| dir.join(CLI_EXE).is_file()) {
        match place_path_copy(bin, dir, stamp) {
            Ok(report) => lines.extend(
                report
                    .lines()
                    .into_iter()
                    .map(|line| format!("the copy on PATH in {}: {line}", dir.display())),
            ),
            Err(e) => lines.push(format!(
                "warning: the copy on PATH in {} was not refreshed: {e}",
                dir.display()
            )),
        }
    }
    lines
}

/// Copies the exes called `names` from `from` into `to`, which [`install`] describes: the
/// previous copies of these exes removed, the copies staged, then each renamed into place. Only
/// these exes' files are touched in `to`.
fn copy_exes(from: &Path, to: &Path, stamp: &str, names: &[&str]) -> Result<Report, String> {
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    let mut report = Report {
        from: from.to_path_buf(),
        to: to.to_path_buf(),
        ..Report::default()
    };
    for (name, path) in previous_copies(to, names) {
        match fs::remove_file(&path) {
            Ok(()) => report.removed.push(name),
            Err(e) => report.kept.push((name, e.to_string())),
        }
    }
    // The copies first, beside their places: one that fails leaves the folder as it was.
    let mut staged: Vec<(&str, PathBuf)> = Vec::new();
    for &name in names {
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

/// The previous copies of the exes called `names` an earlier pass left in `folder`, by name,
/// oldest name first.
pub fn previous_copies(folder: &Path, names: &[&str]) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut found: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            is_previous(&name, names).then(|| (name, entry.path()))
        })
        .collect();
    found.sort();
    found
}

/// Whether `name` is a previous copy of one of the exes called `names`.
fn is_previous(name: &str, names: &[&str]) -> bool {
    let name = name.to_ascii_lowercase();
    names.iter().any(|exe| {
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
        assert!(is_previous("crw.previous-20261110-081000.exe", &EXES));
        assert!(is_previous(
            "ClaudeRoadmapWrangler.previous-20261110-081000-2.exe",
            &EXES
        ));
        assert!(!is_previous("crw.exe", &EXES));
        assert!(!is_previous("crw.previous-.exe", &EXES));
        assert!(!is_previous("other.previous-20261110.exe", &EXES));
        assert!(!is_previous("crw.previous-20261110.txt", &EXES));
        // A staged copy is no previous copy.
        assert!(!is_previous(&staged_name(CLI_EXE), &EXES));
        assert_eq!(staged_name(CLI_EXE), "crw.installing.exe");
        // Only the names asked: the PATH folder's pass knows crw's alone.
        assert!(is_previous("crw.previous-20261110-081000.exe", &[CLI_EXE]));
        assert!(!is_previous(
            "clauderoadmapwrangler.previous-20261110-081000.exe",
            &[CLI_EXE]
        ));
    }

    /// The PATH folder in a trial is CRW_HOME's; else the folder Claude Code's installer puts on
    /// PATH, under the profile.
    #[test]
    fn the_path_folder_is_beside_claude_code_or_the_trials() {
        let dir = path_dir();
        match paths::home() {
            Some(home) => assert_eq!(dir, Some(home.join(TRIAL_PATH_DIR))),
            None if std::env::var_os("USERPROFILE").is_some() => {
                assert!(dir.unwrap().ends_with(Path::new(".local").join("bin")));
            }
            None => assert_eq!(dir, None),
        }
    }

    /// The copy on PATH is placed, refreshed while a running copy is renamed, and removed; the
    /// folder's other programs, and a previous copy of the window's, are never touched.
    #[test]
    fn the_path_copy_is_placed_refreshed_and_removed() {
        let scratch = Scratch::new("install-path-copy");
        let (from, bin, dir) = (
            scratch.0.join("release"),
            scratch.0.join("bin"),
            scratch.0.join("path"),
        );
        // Nothing installed: refused, and the folder is not made.
        let error = place_path_copy(&bin, &dir, "20261114-080000").unwrap_err();
        assert!(error.contains("install first"), "{error}");
        assert!(!dir.exists());
        assert_eq!(path_copy(&bin, &dir), PathCopy::Absent);

        // Every build names the tool's folder, by which a copy is known for the tool's.
        build(&from, "one of clauderoadmapwrangler", at(0));
        install(&from, &bin, "20261114-080000").unwrap();
        // A folder made on the way; the first pass copies.
        let report = place_path_copy(&bin, &dir, "20261114-080100").unwrap();
        assert_eq!(report.copied, vec![CLI_EXE]);
        assert_eq!(names(&dir), vec![CLI_EXE]);
        assert_eq!(
            path_copy(&bin, &dir),
            PathCopy::AsInstalled(dir.join(CLI_EXE))
        );
        // The folder's other programs, and a file that looks like the window's previous copy.
        let others = [
            "claude.exe",
            "clauderoadmapwrangler.previous-20261110-080000.exe",
        ];
        for name in others {
            fs::write(dir.join(name), name).unwrap();
        }
        // As installed: left alone.
        let again = place_path_copy(&bin, &dir, "20261114-080200").unwrap();
        assert_eq!(again.unchanged, vec![CLI_EXE]);
        assert!(again.copied.is_empty() && again.removed.is_empty());

        // A newer install: the copy differs until it is refreshed, the one before renamed.
        build(&from, "two of clauderoadmapwrangler", at(10));
        install(&from, &bin, "20261114-081000").unwrap();
        assert_eq!(path_copy(&bin, &dir), PathCopy::Differs(dir.join(CLI_EXE)));
        let refreshed = place_path_copy(&bin, &dir, "20261114-081100").unwrap();
        assert_eq!(
            refreshed.renamed,
            vec![(
                CLI_EXE.to_owned(),
                "crw.previous-20261114-081100.exe".to_owned()
            )]
        );
        assert_eq!(
            fs::read_to_string(dir.join(CLI_EXE)).unwrap(),
            "crw.exe: two of clauderoadmapwrangler"
        );
        // The next refresh removes crw's previous copy, never the window's.
        build(&from, "three of clauderoadmapwrangler", at(20));
        install(&from, &bin, "20261114-082000").unwrap();
        let next = place_path_copy(&bin, &dir, "20261114-082100").unwrap();
        assert_eq!(next.removed, vec!["crw.previous-20261114-081100.exe"]);
        assert_eq!(
            names(&dir),
            vec![
                "claude.exe",
                "clauderoadmapwrangler.previous-20261110-080000.exe",
                CLI_EXE,
                "crw.previous-20261114-082100.exe",
            ]
        );

        // Removed, with crw's previous copies and a staged one; the others stay.
        fs::write(dir.join(staged_name(CLI_EXE)), "staged").unwrap();
        assert!(remove_path_copy(&dir, "20261114-083000").unwrap());
        assert_eq!(names(&dir), others);
        assert_eq!(path_copy(&bin, &dir), PathCopy::Absent);
        assert!(!remove_path_copy(&dir, "20261114-083100").unwrap());
        for name in others {
            assert_eq!(fs::read_to_string(dir.join(name)).unwrap(), name);
        }
    }

    /// Another program's `crw.exe` in the PATH folder (EP-28a's review) is never replaced or
    /// removed: placing, removing and an install's refresh refuse with words that name it.
    #[test]
    fn another_programs_crw_on_path_is_left_alone() {
        let scratch = Scratch::new("install-path-foreign");
        let (from, bin, dir) = (
            scratch.0.join("release"),
            scratch.0.join("bin"),
            scratch.0.join("path"),
        );
        build(&from, "one of clauderoadmapwrangler", at(0));
        install(&from, &bin, "20261114-080000").unwrap();
        fs::create_dir_all(&dir).unwrap();
        let theirs = dir.join(CLI_EXE);
        fs::write(&theirs, "another tool's crw").unwrap();
        assert_eq!(path_copy(&bin, &dir), PathCopy::Foreign(theirs.clone()));
        let error = place_path_copy(&bin, &dir, "20261114-080100").unwrap_err();
        assert!(error.contains("another program's crw.exe"), "{error}");
        let error = remove_path_copy(&dir, "20261114-080200").unwrap_err();
        assert!(error.contains("another program's crw.exe"), "{error}");
        let lines = after_install(&bin, Some(&dir), "20261114-080300");
        assert!(
            lines[1].starts_with("warning: the copy on PATH in") && lines[1].contains("left alone"),
            "{lines:?}"
        );
        assert_eq!(fs::read_to_string(&theirs).unwrap(), "another tool's crw");
        assert_eq!(names(&dir), vec![CLI_EXE]);
    }

    /// A copy that cannot be removed or renamed is an error that names it, and it stays.
    #[cfg(windows)]
    #[test]
    fn a_path_copy_that_is_held_says_so() {
        let scratch = Scratch::new("install-path-held");
        let (from, bin, dir) = (
            scratch.0.join("release"),
            scratch.0.join("bin"),
            scratch.0.join("path"),
        );
        build(&from, "one of clauderoadmapwrangler", at(0));
        install(&from, &bin, "20261114-080000").unwrap();
        place_path_copy(&bin, &dir, "20261114-080100").unwrap();
        let handle = hold(&dir.join(CLI_EXE));
        let error = remove_path_copy(&dir, "20261114-080200").unwrap_err();
        drop(handle);
        assert!(error.contains("could not be removed"), "{error}");
        assert_eq!(names(&dir), vec![CLI_EXE]);
    }

    #[test]
    fn the_icon_is_written_once_then_left() {
        let scratch = Scratch::new("install-icon");
        let bin = scratch.0.join("bin");
        // The folder made on the way.
        assert!(write_icon(&bin).unwrap());
        let path = bin.join(icon::ICO_FILE);
        assert_eq!(fs::read(&path).unwrap(), icon::ico(&icon::ICO_SIZES));
        assert!(!write_icon(&bin).unwrap());
        // A file that differs is written again, through a temporary name that does not stay.
        fs::write(&path, "an older drawing").unwrap();
        assert!(write_icon(&bin).unwrap());
        assert_eq!(names(&bin), vec![icon::ICO_FILE]);
    }

    /// After an install: the icon, then the copy on PATH refreshed only while there is one.
    #[test]
    fn after_an_install_the_icon_is_written_and_a_path_copy_refreshed() {
        let scratch = Scratch::new("install-after");
        let (from, bin, dir) = (
            scratch.0.join("release"),
            scratch.0.join("bin"),
            scratch.0.join("path"),
        );
        build(&from, "one of clauderoadmapwrangler", at(0));
        install(&from, &bin, "20261114-080000").unwrap();
        let lines = after_install(&bin, Some(&dir), "20261114-080000");
        assert_eq!(lines, vec![format!("{} written", icon::ICO_FILE)]);
        assert!(!dir.exists(), "no copy on PATH is placed by an install");
        place_path_copy(&bin, &dir, "20261114-080100").unwrap();
        build(&from, "two of clauderoadmapwrangler", at(10));
        install(&from, &bin, "20261114-081000").unwrap();
        let lines = after_install(&bin, Some(&dir), "20261114-081100");
        assert_eq!(lines[0], format!("{} is as drawn already", icon::ICO_FILE));
        assert_eq!(
            lines[1],
            format!(
                "the copy on PATH in {}: crw.exe copied; the one before is now \
                 crw.previous-20261114-081100.exe",
                dir.display()
            )
        );
        assert_eq!(
            path_copy(&bin, &dir),
            PathCopy::AsInstalled(dir.join(CLI_EXE))
        );
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
