//! A session's start (D-26; EP-15b): the project as it stood when a session began, recorded so
//! that a revert can put it back. One `git status` reads the HEAD, the branch and every file
//! uncommitted and not ignored. The changed and untracked files are then copied into the tool's
//! backups folder, never into the project (D-11): all of them, or none above [`MAX_FILES`] or
//! [`MAX_BYTES`]. Without its copies a start keeps the list only, and a revert leaves those files
//! as they are. The ledger keeps the start as JSON with the session's row (schema 4).

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::git::{self, ChangeKind, Status};

/// The most files a start copies.
pub const MAX_FILES: usize = 200;

/// The most bytes a start copies: 20 MB.
pub const MAX_BYTES: u64 = 20 * 1024 * 1024;

/// A project as it stood at a session's start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Start {
    pub at: DateTime<Utc>,
    /// The commit `HEAD` named; none before the first commit.
    pub head: Option<String>,
    /// The branch checked out; none when `HEAD` was detached.
    pub branch: Option<String>,
    /// Every file uncommitted and not ignored then.
    #[serde(default)]
    pub files: Vec<StartFile>,
    /// The folder that holds the copies; none when there was nothing to copy, or too much.
    #[serde(default)]
    pub copy: Option<PathBuf>,
    /// Every changed and untracked file was copied; false above the cap.
    pub copied: bool,
}

/// A file uncommitted at the start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartFile {
    /// From the repository's root, with `/` between folders, as git gives it.
    pub path: String,
    pub state: FileState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileState {
    /// Tracked and changed: modified, added, renamed to, or in conflict.
    Changed,
    Untracked,
    /// Tracked and missing from the working tree.
    Deleted,
}

impl Start {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| format!("the recorded start does not read: {e}"))
    }

    /// The start's record of `path`, when the file was uncommitted then.
    pub fn file(&self, path: &str) -> Option<&StartFile> {
        self.files.iter().find(|file| file.path == path)
    }

    /// Where the copy of `path` is, when the start copied it.
    pub fn copy_of(&self, path: &str) -> Option<PathBuf> {
        let file = self
            .file(path)
            .filter(|file| file.state != FileState::Deleted)?;
        Some(self.copy.as_ref()?.join(relative(&file.path)?))
    }

    /// Takes the copies away, for a session that ended without working (EP-13).
    pub fn discard(&self) {
        if let Some(copy) = &self.copy {
            let _ = fs::remove_dir_all(copy);
        }
    }
}

/// `<backups>\<project folder>`, where a tick's copy is kept too (D-10 addendum).
pub fn project_dir(backups: &Path, folder: &Path) -> PathBuf {
    backups.join(folder.file_name().unwrap_or_else(|| "project".as_ref()))
}

/// `<backups>\<project folder>\starts\<session id>`: the copies of a session's start.
pub fn copies_dir(backups: &Path, folder: &Path, session_id: &str) -> PathBuf {
    let name: String = session_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    project_dir(backups, folder).join("starts").join(name)
}

/// A path git gave, as a relative path, when it is one: never absolute, never `..`.
pub(crate) fn relative(path: &str) -> Option<PathBuf> {
    let relative = Path::new(path);
    (!path.is_empty()
        && relative
            .components()
            .all(|c| matches!(c, Component::Normal(_))))
    .then(|| relative.to_path_buf())
}

/// Reads the start of a session in `folder` at `at`, then copies its uncommitted files into
/// `copies` (or `copies.2`, and so on, when that folder is taken). The one `git status` runs
/// under the caller's `process::turn()` (D-29). `Err` says why no start was recorded; a copy
/// that failed halfway is taken away.
pub fn capture(folder: &Path, copies: &Path, at: DateTime<Utc>) -> Result<Start, String> {
    capture_within(folder, copies, at, (MAX_FILES, MAX_BYTES))
}

/// [`capture`] with the cap given: at most `cap.0` files and `cap.1` bytes copied.
fn capture_within(
    folder: &Path,
    copies: &Path,
    at: DateTime<Utc>,
    cap: (usize, u64),
) -> Result<Start, String> {
    let status = git::status_all(folder)?;
    let files = listed(folder, &status);
    let mut start = Start {
        at,
        head: status.oid,
        branch: status.branch,
        files: Vec::new(),
        copy: None,
        copied: true,
    };
    let copyable: Vec<&StartFile> = files
        .iter()
        .filter(|file| file.state != FileState::Deleted)
        .collect();
    if !copyable.is_empty() {
        if fits(folder, &copyable, cap) {
            let dir = fresh_dir(copies)
                .map_err(|e| format!("{}: the start's copies: {e}", copies.display()))?;
            for file in &copyable {
                if let Err(e) = copy_under(folder, &dir, &file.path) {
                    let _ = fs::remove_dir_all(&dir);
                    return Err(format!("{}: the start's copy failed: {e}", file.path));
                }
            }
            start.copy = Some(dir);
        } else {
            start.copied = false;
        }
    }
    start.files = files;
    Ok(start)
}

/// The uncommitted files a status lists, each once: a rename's old path as deleted when it is
/// gone, its new path as changed.
fn listed(folder: &Path, status: &Status) -> Vec<StartFile> {
    let present = |path: &str| match relative(path) {
        Some(relative) if !folder.join(&relative).exists() => FileState::Deleted,
        _ => FileState::Changed,
    };
    let mut files: Vec<StartFile> = Vec::new();
    let mut add = |path: &str, state: FileState| {
        if !files.iter().any(|file| file.path == path) {
            files.push(StartFile {
                path: path.to_owned(),
                state,
            });
        }
    };
    for change in &status.changes {
        match &change.kind {
            ChangeKind::Untracked => add(&change.path, FileState::Untracked),
            ChangeKind::Renamed { from } => {
                add(&change.path, present(&change.path));
                if present(from) == FileState::Deleted {
                    add(from, FileState::Deleted);
                }
            }
            ChangeKind::Changed | ChangeKind::Unmerged => add(&change.path, present(&change.path)),
        }
    }
    files
}

/// Whether the files are within `cap` (files, bytes), each a plain file at a plain path.
fn fits(folder: &Path, files: &[&StartFile], cap: (usize, u64)) -> bool {
    if files.len() > cap.0 {
        return false;
    }
    let mut bytes = 0u64;
    for file in files {
        let Some(relative) = relative(&file.path) else {
            return false;
        };
        match fs::symlink_metadata(folder.join(relative)) {
            Ok(meta) if meta.is_file() => bytes += meta.len(),
            _ => return false,
        }
    }
    bytes <= cap.1
}

/// Copies `path` from under `from` to the same place under `to`, making its folders.
pub(crate) fn copy_under(from: &Path, to: &Path, path: &str) -> io::Result<()> {
    let relative = relative(path)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "not a relative path"))?;
    let target = to.join(&relative);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(from.join(&relative), target).map(|_| ())
}

/// A new folder at `base`, or at `base.2`, `base.3`, … when that is taken.
pub(crate) fn fresh_dir(base: &Path) -> io::Result<PathBuf> {
    if let Some(parent) = base.parent() {
        fs::create_dir_all(parent)?;
    }
    for n in 1..100 {
        let dir = match n {
            1 => base.to_owned(),
            n => {
                let mut name = base.file_name().unwrap_or_default().to_owned();
                name.push(format!(".{n}"));
                base.with_file_name(name)
            }
        };
        match fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other("every folder name is taken"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::revert::testing::Repo;

    #[test]
    fn a_clean_start_is_its_head_and_branch_alone() {
        let repo = Repo::new("start-clean");
        repo.write("a.txt", "one\n");
        let head = repo.commit("first");
        let copies = repo.backups().join("starts").join("s-1");
        let start = repo.capture(&copies);
        assert_eq!(start.head.as_deref(), Some(head.as_str()));
        assert_eq!(start.branch.as_deref(), Some("main"));
        assert!(start.files.is_empty() && start.copied);
        assert_eq!(start.copy, None);
        assert!(!copies.exists(), "nothing to copy, no folder");
        // The JSON reads back as it was written.
        assert_eq!(Start::from_json(&start.to_json()).unwrap(), start);
        assert!(Start::from_json("{").is_err());
    }

    #[test]
    fn a_dirty_start_copies_its_files_or_lists_them_above_the_cap() {
        let repo = Repo::new("start-dirty");
        repo.write("a.txt", "one\n");
        repo.write("gone.txt", "soon gone\n");
        repo.write(".gitignore", "target/\n");
        repo.commit("first");
        repo.write("a.txt", "one, and the owner's edit\n");
        repo.write("notes/new.md", "a draft\n");
        repo.write("target/build.log", "ignored\n");
        repo.remove("gone.txt");
        let copies = repo.backups().join("starts").join("s-2");
        let start = repo.capture(&copies);
        let mut files: Vec<(&str, FileState)> = start
            .files
            .iter()
            .map(|file| (file.path.as_str(), file.state))
            .collect();
        files.sort_by_key(|(path, _)| *path);
        assert_eq!(
            files,
            [
                ("a.txt", FileState::Changed),
                ("gone.txt", FileState::Deleted),
                ("notes/new.md", FileState::Untracked),
            ],
            "the ignored file is never listed"
        );
        assert!(start.copied);
        assert_eq!(start.copy.as_deref(), Some(copies.as_path()));
        let copy = start.copy_of("a.txt").unwrap();
        assert_eq!(
            fs::read_to_string(copy).unwrap(),
            "one, and the owner's edit\n"
        );
        assert!(start.copy_of("notes/new.md").unwrap().is_file());
        assert_eq!(start.copy_of("gone.txt"), None);
        assert!(!copies.join("target").exists());
        // A second start of the same session takes the next folder.
        let again = repo.capture(&copies);
        assert_eq!(
            again.copy.as_deref(),
            Some(copies.with_file_name("s-2.2").as_path())
        );
        again.discard();
        assert!(!copies.with_file_name("s-2.2").exists());
        assert!(copies.exists());

        // Above the cap, in files or in bytes, the list alone.
        let over = repo.backups().join("starts").join("s-3");
        for cap in [(1, MAX_BYTES), (MAX_FILES, 10)] {
            let _turn = crate::process::turn();
            let start = capture_within(&repo.0, &over, Utc::now(), cap).unwrap();
            assert!(!start.copied);
            assert_eq!(start.copy, None);
            assert_eq!(start.files.len(), 3);
            assert_eq!(start.copy_of("a.txt"), None);
        }
        assert!(!over.exists());
    }

    #[test]
    fn paths_and_folders_stay_inside() {
        assert_eq!(relative("a/b.txt"), Some(PathBuf::from("a/b.txt")));
        for odd in ["", "../x", "a/../../x", "/etc/x", r"C:\x", "C:x"] {
            assert_eq!(relative(odd), None, "{odd}");
        }
        let backups = Path::new(r"C:\local\backups");
        assert_eq!(
            copies_dir(backups, Path::new(r"C:\Data\alpha"), "0e1d/2c3b"),
            PathBuf::from(r"C:\local\backups\alpha\starts\0e1d_2c3b")
        );
    }
}
