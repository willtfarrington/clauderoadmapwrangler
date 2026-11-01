//! A project's roadmap as a snapshot (D-9): the README at `HEAD` and in the working tree, both
//! parsed and both kept; the brief files and the decision log as they stand on disk; what git
//! says is changed under `roadmap/`; and what follows from them: whether a session is in
//! progress, with its reasons, and the two facts `proposal` reads (every row checked, a pickup
//! gate on the next brief). Git is asked one question at a time with the usual spacing (D-29);
//! a folder that is not a repository is read from the working tree alone.

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use super::brief::{self, Brief, BriefFile, Files, HandoffFile};
use super::table::{self, Done, Roadmap, Row};
use super::{EpId, readme};
use crate::proposal::Facts;
use crate::{decisions, process};

/// The README's path inside the repository, as git names it.
const README_IN_REPO: &str = "roadmap/README.md";

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub folder: PathBuf,
    /// The README as committed at `HEAD`; `None` outside git, before the first commit, or when
    /// the file is not committed.
    pub head: Option<Roadmap>,
    /// The README in the working tree; `None` when the file is missing.
    pub tree: Option<Roadmap>,
    /// What git reports under `roadmap/`; `None` outside git or when git is missing.
    pub git: Option<GitStatus>,
    pub files: Files,
    /// The briefs, parsed, in the order of [`Files::briefs`].
    pub briefs: Vec<Entry>,
    pub decisions: decisions::Decisions,
    /// Files that could not be read.
    pub problems: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub file: BriefFile,
    pub brief: Brief,
    /// The packet style's `docs/handoffs/EP-N.md`, when the project keeps one.
    pub handoff: Option<HandoffFile>,
}

impl Entry {
    /// Complete by the brief's own markers or by its handoff file.
    pub fn is_complete(&self) -> bool {
        self.brief.is_complete() || self.handoff.as_ref().is_some_and(HandoffFile::is_complete)
    }
}

/// `git status --porcelain -- roadmap`, read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitStatus {
    /// The README is modified, staged or untracked.
    pub readme_dirty: bool,
    /// Brief files git does not track yet, by name.
    pub untracked: Vec<String>,
    /// Brief files changed since `HEAD`, by name.
    pub modified: Vec<String>,
}

impl GitStatus {
    pub fn is_clean(&self) -> bool {
        !self.readme_dirty && self.untracked.is_empty() && self.modified.is_empty()
    }
}

/// Why a session is taken to be in progress (D-6, D-9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InProgress {
    ReadmeDirty,
    UntrackedBrief(String),
    ModifiedBrief(String),
    /// A `[~]` cell in the packet style.
    RowInProgress(EpId),
    /// The next EP's brief exists without a completion note, it is the only open row with a
    /// brief, and git could not be asked: outside a repository the files are the only evidence.
    /// With git, a committed brief in a clean tree is a pre-written one, and the files a session
    /// is editing show up as the reasons above.
    BriefWithoutCompletion(EpId),
}

impl fmt::Display for InProgress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InProgress::ReadmeDirty => f.write_str("the roadmap README is changed"),
            InProgress::UntrackedBrief(name) => write!(f, "{name} is untracked"),
            InProgress::ModifiedBrief(name) => write!(f, "{name} is changed"),
            InProgress::RowInProgress(id) => write!(f, "{id} is marked [~]"),
            InProgress::BriefWithoutCompletion(id) => {
                write!(f, "the brief of {id} has no completion note")
            }
        }
    }
}

impl Snapshot {
    /// The reading the dashboard shows: the working tree, else `HEAD`.
    pub fn current(&self) -> Option<&Roadmap> {
        self.tree.as_ref().or(self.head.as_ref())
    }

    /// The next EP of the current reading (D-9).
    pub fn next(&self) -> Option<&Row> {
        self.current().and_then(Roadmap::next)
    }

    pub fn entry(&self, id: EpId) -> Option<&Entry> {
        self.briefs.iter().find(|entry| entry.file.id == id)
    }

    /// Whether `HEAD` and the working tree read differently in what matters: the next EP or
    /// the counts.
    pub fn differs(&self) -> bool {
        match (&self.head, &self.tree) {
            (Some(head), Some(tree)) => {
                head.next().map(|row| row.id) != tree.next().map(|row| row.id)
                    || head.counts() != tree.counts()
            }
            (None, None) => false,
            _ => true,
        }
    }

    /// Whether `id` is complete by any evidence: its row checked, its brief's markers, or its
    /// handoff file (the three styles' evidence is disjoint, so one rule serves them all).
    pub fn is_complete(&self, id: EpId) -> bool {
        self.current()
            .and_then(|roadmap| roadmap.row(id))
            .is_some_and(|row| row.done.is_checked())
            || self.entry(id).is_some_and(Entry::is_complete)
    }

    /// The reasons a session is in progress; none when it is not.
    pub fn in_progress(&self) -> Vec<InProgress> {
        let mut reasons = Vec::new();
        if let Some(git) = &self.git {
            if git.readme_dirty {
                reasons.push(InProgress::ReadmeDirty);
            }
            reasons.extend(
                git.untracked
                    .iter()
                    .cloned()
                    .map(InProgress::UntrackedBrief),
            );
            reasons.extend(git.modified.iter().cloned().map(InProgress::ModifiedBrief));
        }
        let Some(roadmap) = self.current() else {
            return reasons;
        };
        reasons.extend(
            roadmap
                .rows()
                .filter(|row| row.done == Done::InProgress)
                .map(|row| InProgress::RowInProgress(row.id)),
        );
        if self.git.is_none()
            && let Some(next) = roadmap.next()
        {
            let open_with_brief: Vec<EpId> = roadmap
                .rows()
                .filter(|row| !row.done.is_checked() && self.entry(row.id).is_some())
                .map(|row| row.id)
                .collect();
            if open_with_brief == [next.id] && !self.is_complete(next.id) {
                reasons.push(InProgress::BriefWithoutCompletion(next.id));
            }
        }
        reasons
    }

    /// Fills the facts the parsers own (D-6): every row checked, from the current reading; a
    /// pickup gate, from the next EP's brief.
    pub fn fill(&self, facts: &mut Facts) {
        if let Some(roadmap) = self.current() {
            facts.all_rows_checked = Some(roadmap.counts().all_checked());
            facts.pickup_gate = Some(
                roadmap
                    .next()
                    .and_then(|row| self.entry(row.id))
                    .is_some_and(|entry| entry.brief.pickup_gate().is_some()),
            );
        }
    }

    /// Whether the next EP's brief carries a pickup gate.
    pub fn pickup_gate(&self) -> bool {
        let mut facts = Facts::default();
        self.fill(&mut facts);
        facts.pickup_gate == Some(true)
    }
}

/// Reads `folder` from the working tree alone: the README, the briefs, the decision log. No
/// child process runs, so `crw scan` can call it for every project.
pub fn read_tree(folder: &Path) -> Snapshot {
    let mut snapshot = Snapshot {
        folder: folder.to_owned(),
        ..Snapshot::default()
    };
    let readme = readme(folder);
    match fs::read_to_string(&readme) {
        Ok(text) => snapshot.tree = Some(table::parse(&text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => snapshot.problems.push(format!("{}: {e}", readme.display())),
    }
    match brief::discover(&folder.join("roadmap")) {
        Ok(files) => snapshot.files = files,
        Err(e) => snapshot
            .problems
            .push(format!("{}: {e}", folder.join("roadmap").display())),
    }
    for file in &snapshot.files.briefs {
        match brief::read(&file.path) {
            Ok(parsed) => snapshot.briefs.push(Entry {
                file: file.clone(),
                brief: parsed,
                handoff: brief::handoff_file(folder, file.id),
            }),
            Err(e) => snapshot
                .problems
                .push(format!("{}: {e}", file.path.display())),
        }
    }
    snapshot.decisions = decisions::read(folder);
    snapshot
}

/// Reads `folder` whole: the working tree, then `HEAD`'s README and git's status under
/// `roadmap/` when the folder is a repository (two git children, spaced).
pub fn take(folder: &Path) -> Snapshot {
    let mut snapshot = read_tree(folder);
    if !folder.join(".git").exists() {
        return snapshot;
    }
    snapshot.head = git(folder, &["show", &format!("HEAD:{README_IN_REPO}")])
        .ok()
        .map(|text| table::parse(&text));
    thread::sleep(process::SPACING);
    snapshot.git = git(folder, &["status", "--porcelain", "-z", "--", "roadmap"])
        .ok()
        .map(|text| status_of(&text));
    snapshot
}

fn git(folder: &Path, args: &[&str]) -> Result<String, String> {
    let mut all: Vec<&OsStr> = vec![OsStr::new("-C"), folder.as_os_str()];
    all.extend(args.iter().map(OsStr::new));
    process::output_of(Path::new("git"), all, process::TIMEOUT)
}

/// `git status --porcelain -z -- roadmap`: `XY path` records separated by NUL, a renamed or
/// copied file followed by its old path.
fn status_of(text: &str) -> GitStatus {
    let mut status = GitStatus::default();
    let mut records = text.split('\0').filter(|record| !record.is_empty());
    while let Some(record) = records.next() {
        let Some((marks, path)) = record.split_at_checked(3) else {
            continue;
        };
        if marks.starts_with(['R', 'C']) {
            records.next();
        }
        let untracked = marks.starts_with("??");
        if path == README_IN_REPO {
            status.readme_dirty = true;
        } else if let Some(name) = path.strip_prefix("roadmap/")
            && !name.contains('/')
            && brief::file_id(name).is_some_and(|(_, slug)| !brief::is_companion(slug))
        {
            if untracked {
                status.untracked.push(name.to_owned());
            } else {
                status.modified.push(name.to_owned());
            }
        }
    }
    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_porcelain_status() {
        let text = "?? roadmap/EP-4-parser-b.md\0 M roadmap/README.md\0M  roadmap/EP-3-parser-a.md\0R  roadmap/EP-5-new.md\0roadmap/EP-5-old.md\0?? roadmap/EP-9-completion-report.md\0?? roadmap/inbox.md\0?? roadmap/sub/EP-1-x.md\0A  roadmap/_TEMPLATE.md\0";
        assert_eq!(
            status_of(text),
            GitStatus {
                readme_dirty: true,
                untracked: vec!["EP-4-parser-b.md".to_owned()],
                modified: vec!["EP-3-parser-a.md".to_owned(), "EP-5-new.md".to_owned()],
            }
        );
        assert!(status_of("").is_clean());
        assert_eq!(
            status_of("?? roadmap/README.md\0"),
            GitStatus {
                readme_dirty: true,
                ..GitStatus::default()
            }
        );
        assert!(!status_of("?? roadmap/README.md\0").is_clean());
        assert_eq!(
            InProgress::ReadmeDirty.to_string(),
            "the roadmap README is changed"
        );
        assert_eq!(
            InProgress::BriefWithoutCompletion(EpId::new(4)).to_string(),
            "the brief of EP-4 has no completion note"
        );
    }

    #[test]
    fn an_empty_folder_reads_as_nothing() {
        let scratch = crate::files::tests::Scratch::new("snapshot-empty");
        let snapshot = take(&scratch.0);
        assert!(snapshot.head.is_none() && snapshot.tree.is_none() && snapshot.git.is_none());
        assert!(snapshot.briefs.is_empty() && snapshot.problems.is_empty());
        assert!(snapshot.in_progress().is_empty());
        assert!(!snapshot.differs());
        let mut facts = Facts::default();
        snapshot.fill(&mut facts);
        assert_eq!(facts, Facts::default());
        assert!(!snapshot.pickup_gate());
    }
}
