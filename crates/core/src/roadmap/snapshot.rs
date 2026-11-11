//! A project's roadmap as a snapshot (D-9): the README at `HEAD` and in the working tree, both
//! parsed and both kept; the brief files, the decision log, the owner tasks, the inbox and the
//! older forms of the owner's to-dos as they stand on disk (EP-5); what git says is changed under
//! `roadmap/`; and what follows from them: whether a session is in progress, with its reasons,
//! and the two facts `proposal` reads (every row checked, a pickup gate on the next brief). Git is
//! asked one question at a time with the usual spacing (D-29); a folder that is not a repository
//! is read from the working tree alone.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use super::brief::{self, Brief, BriefFile, Evidence, Files, HandoffFile};
use super::table::{self, Done, Roadmap, Row};
use super::{EpId, first_date, readme};
use crate::git::{self, ChangeKind};
use crate::inbox::{self, Inbox};
use crate::proposal::Facts;
use crate::tasks::{self, OwnerItem, Tasks, legacy};
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
    /// `roadmap/owner-tasks.md` (D-10); `None` when the project keeps none.
    pub tasks: Option<Tasks>,
    /// `roadmap/inbox.md` (D-13); `None` when the project keeps none.
    pub inbox: Option<Inbox>,
    /// The owner's to-dos in the older forms, read-only (D-10).
    pub legacy: Vec<OwnerItem>,
    /// Files that could not be read.
    pub problems: Vec<String>,
    /// The briefs among them, by path: the mailman's first reading of the project waits for
    /// them (EP-20c).
    pub unread_briefs: Vec<PathBuf>,
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

    /// The date its completion carries: the marker's, a done status's, a filled handoff
    /// payload's, else the handoff file's.
    pub fn completed_on(&self) -> Option<String> {
        let marked = self.brief.completion().and_then(|evidence| match evidence {
            Evidence::CompletionNote { date, .. }
            | Evidence::Complete { date, .. }
            | Evidence::CompletionHeading { date, .. }
            | Evidence::HandoffPayload { date, .. } => date,
            Evidence::Status { value } => first_date(&value).map(str::to_owned),
            Evidence::FrontMatter { .. } => None,
        });
        marked
            .or_else(|| self.brief.handoff.as_ref().and_then(|h| h.date.clone()))
            .or_else(|| {
                self.handoff
                    .as_ref()
                    .and_then(|h| h.date().map(str::to_owned))
            })
    }
}

/// What git reports changed under `roadmap/`.
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
    /// A session runs in the project, in its own words: one the window started, or one the hooks
    /// and the poll of live sessions report (EP-12). The files cannot say it; the window adds it.
    Session(String),
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
            InProgress::Session(words) => f.write_str(words),
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

    /// Fills the facts the parsers own (D-6): every row checked and the row count, from the
    /// current reading; a pickup gate, from the next EP's brief, and that EP.
    pub fn fill(&self, facts: &mut Facts) {
        if let Some(roadmap) = self.current() {
            let counts = roadmap.counts();
            facts.all_rows_checked = Some(counts.all_checked());
            facts.rows = counts.total;
            let next = roadmap.next();
            facts.next = next.map(|row| row.id);
            facts.pickup_gate = Some(
                next.and_then(|row| self.entry(row.id))
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

    /// The brief completed last: the newest completion date, then the later row in table order.
    /// Its blocks and its handoff notes are the older forms' hand-offs still to be read (EP-5).
    pub fn latest_complete(&self) -> Option<&Entry> {
        let position = |id: EpId| {
            self.current()
                .and_then(|roadmap| roadmap.rows().position(|row| row.id == id))
        };
        self.briefs
            .iter()
            .filter(|entry| entry.is_complete())
            .max_by_key(|entry| (entry.completed_on(), position(entry.file.id)))
    }

    /// Everything waiting on the owner (D-10): the lines of `owner-tasks.md`, then the older
    /// forms.
    pub fn owner_items(&self) -> Vec<OwnerItem> {
        self.tasks
            .iter()
            .flat_map(Tasks::items)
            .chain(self.legacy.iter().cloned())
            .collect()
    }

    /// Whether the older forms are what waits on the owner (D-10 addendum of 2026-11-11): only
    /// where the project keeps no `owner-tasks.md`. A project that keeps the file counts its
    /// tasks (and its inbox, counted beside them), and its older forms, a README's risks list
    /// among them, are read-only context that no one waits on.
    pub fn older_forms_wait(&self) -> bool {
        self.tasks.is_none()
    }

    /// The items counted as waiting on the owner: the lines of `owner-tasks.md` where the file
    /// is kept, else the older forms.
    pub fn waiting(&self) -> Vec<OwnerItem> {
        match &self.tasks {
            Some(tasks) => tasks.items().collect(),
            None => self.legacy.clone(),
        }
    }

    /// How many of [`Self::waiting`] are open.
    pub fn open_waiting(&self) -> usize {
        match &self.tasks {
            Some(tasks) => tasks.open().count(),
            None => self.legacy.iter().filter(|item| item.is_open()).count(),
        }
    }

    /// The older forms shown uncounted, under a heading of their own: every older form of a
    /// project that keeps `owner-tasks.md`; none elsewhere, where [`Self::waiting`] holds them.
    pub fn uncounted(&self) -> &[OwnerItem] {
        if self.tasks.is_some() {
            &self.legacy
        } else {
            &[]
        }
    }
}

/// Reads `folder` from the working tree alone: the README, the briefs, the decision log, the
/// owner tasks, the inbox and the older forms. No child process runs, so `crw scan` can call it
/// for every project.
pub fn read_tree(folder: &Path) -> Snapshot {
    let mut snapshot = Snapshot {
        folder: folder.to_owned(),
        ..Snapshot::default()
    };
    let readme = readme(folder);
    let readme_text = match fs::read_to_string(&readme) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            snapshot.problems.push(format!("{}: {e}", readme.display()));
            None
        }
    };
    snapshot.tree = readme_text.as_deref().map(table::parse);
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
            Err(e) => {
                snapshot
                    .problems
                    .push(format!("{}: {e}", file.path.display()));
                snapshot.unread_briefs.push(file.path.clone());
            }
        }
    }
    snapshot.decisions = decisions::read(folder);
    match tasks::read(folder) {
        Ok(tasks) => snapshot.tasks = tasks,
        Err(e) => snapshot
            .problems
            .push(format!("{}: {e}", tasks::path(folder).display())),
    }
    match inbox::read(folder) {
        Ok(inbox) => snapshot.inbox = inbox,
        Err(e) => snapshot
            .problems
            .push(format!("{}: {e}", inbox::path(folder).display())),
    }
    let (items, problems) =
        legacy::read(folder, readme_text.as_deref(), snapshot.latest_complete());
    snapshot.legacy = items;
    snapshot.problems.extend(problems);
    snapshot
}

/// Reads `folder` whole: the working tree, then `HEAD`'s README and git's status when the folder
/// is a repository (two git children, spaced).
pub fn take(folder: &Path) -> Snapshot {
    let mut snapshot = read_tree(folder);
    if let Some(reading) = read_git(folder) {
        reading.fill(&mut snapshot);
    }
    snapshot
}

/// What git says about a repository: the README at `HEAD`, the changes under `roadmap/`, and the
/// whole working tree's status, which the badges read (EP-7). The dashboard reads it after the
/// working tree, on its own thread (EP-6).
#[derive(Debug, Clone, Default)]
pub struct GitReading {
    pub head: Option<Roadmap>,
    pub status: Option<GitStatus>,
    pub worktree: Option<git::Status>,
}

impl GitReading {
    /// Puts the roadmap's part of the reading into a snapshot of the working tree.
    pub fn fill(self, snapshot: &mut Snapshot) {
        snapshot.head = self.head;
        snapshot.git = self.status;
    }
}

/// Asks git about `folder` (two children, spaced); `None` when it is not a repository.
pub fn read_git(folder: &Path) -> Option<GitReading> {
    if !folder.join(".git").exists() {
        return None;
    }
    let head = git::run(folder, &["show", &format!("HEAD:{README_IN_REPO}")])
        .ok()
        .map(|text| table::parse(&text));
    thread::sleep(process::SPACING);
    let worktree = git::status(folder).ok();
    Some(GitReading {
        head,
        status: worktree.as_ref().map(roadmap_changes),
        worktree,
    })
}

/// The changes under `roadmap/` that matter (D-9): the README, and the brief files git does not
/// track or sees changed (a renamed brief by its new name).
fn roadmap_changes(status: &git::Status) -> GitStatus {
    let mut changes = GitStatus::default();
    for change in &status.changes {
        let path = change.path.as_str();
        if path == README_IN_REPO {
            changes.readme_dirty = true;
        } else if let Some(name) = path.strip_prefix("roadmap/")
            && !name.contains('/')
            && brief::file_id(name).is_some_and(|(_, slug)| !brief::is_companion(slug))
        {
            if change.kind == ChangeKind::Untracked {
                changes.untracked.push(name.to_owned());
            } else {
                changes.modified.push(name.to_owned());
            }
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_roadmaps_changes_from_the_status() {
        let changes = |text: &str| roadmap_changes(&git::parse_status(text));
        let text = "# branch.oid 1111111111111111111111111111111111111111\0\
                    # branch.head main\0\
                    ? roadmap/EP-4-parser-b.md\0\
                    1 .M N... 100644 100644 100644 aaaa aaaa roadmap/README.md\0\
                    1 M. N... 100644 100644 100644 bbbb cccc roadmap/EP-3-parser-a.md\0\
                    2 R. N... 100644 100644 100644 dddd dddd R100 roadmap/EP-5-new.md\0roadmap/EP-5-old.md\0\
                    ? roadmap/EP-9-completion-report.md\0\
                    ? roadmap/inbox.md\0\
                    ? roadmap/sub/EP-1-x.md\0\
                    1 A. N... 000000 100644 100644 0000 eeee roadmap/_TEMPLATE.md\0\
                    1 .M N... 100644 100644 100644 ffff ffff src/main.rs\0";
        assert_eq!(
            changes(text),
            GitStatus {
                readme_dirty: true,
                untracked: vec!["EP-4-parser-b.md".to_owned()],
                modified: vec!["EP-3-parser-a.md".to_owned(), "EP-5-new.md".to_owned()],
            }
        );
        assert!(changes("").is_clean());
        assert_eq!(
            changes("? roadmap/README.md\0"),
            GitStatus {
                readme_dirty: true,
                ..GitStatus::default()
            }
        );
        assert!(!changes("? roadmap/README.md\0").is_clean());
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
