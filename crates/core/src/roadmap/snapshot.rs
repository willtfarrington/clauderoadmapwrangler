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
use crate::manual::{self, Which};
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
    /// Which of the manual's two files are there (D-12 addendum; EP-57a).
    pub manual: manual::Present,
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

    /// The last phase whose rows are all checked, as the Status section names it (its id, else
    /// its name), and the date its last row's brief completed, else the date of a heading that
    /// says it is done (`(done 2026-10-31)`), when either carries one (EP-57a): the manual's date
    /// is weighed against it.
    pub fn last_close(&self) -> Option<(String, Option<String>)> {
        let phase = self.current()?.phases.iter().rev().find(|phase| {
            !phase.rows.is_empty() && phase.rows.iter().all(|row| row.done.is_checked())
        })?;
        let last = phase.rows.last()?;
        let name = phase
            .heading
            .id
            .clone()
            .unwrap_or_else(|| phase.heading.name.clone());
        let closed = self
            .entry(last.id)
            .and_then(Entry::completed_on)
            .or_else(|| {
                phase
                    .heading
                    .status
                    .as_deref()
                    .filter(|status| status.starts_with("done"))
                    .and_then(first_date)
                    .map(str::to_owned)
            });
        Some((name, closed))
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

    /// Whether the project keeps `roadmap/owner-tasks.md`, the one form counted as waiting on
    /// the owner (D-10 addendum of 2026-11-22). A project without it shows no count, and its
    /// older forms, a README's risks list among them, are read-only context that no one waits on.
    pub fn keeps_tasks(&self) -> bool {
        self.tasks.is_some()
    }

    /// The items counted as waiting on the owner: the lines of `owner-tasks.md`, none where the
    /// file is not kept (D-10 addendum of 2026-11-22; EP-50).
    pub fn waiting(&self) -> Vec<OwnerItem> {
        self.tasks.iter().flat_map(Tasks::items).collect()
    }

    /// How many of [`Self::waiting`] are open: the open lines of `owner-tasks.md`.
    pub fn open_waiting(&self) -> usize {
        self.tasks.as_ref().map_or(0, |tasks| tasks.open().count())
    }

    /// The older forms, read-only and counted nowhere (EP-50): a README's risks list, the
    /// registers, the latest brief's blocks and handoff notes, whether or not the project keeps
    /// `owner-tasks.md`.
    pub fn uncounted(&self) -> &[OwnerItem] {
        &self.legacy
    }

    /// How many of [`Self::uncounted`] are open: what the list's "older forms" mark says of a
    /// project without `owner-tasks.md` (EP-50).
    pub fn older_open(&self) -> usize {
        self.legacy.iter().filter(|item| item.is_open()).count()
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
    snapshot.manual = manual::Present::read(folder);
    snapshot
}

/// Reads `folder` whole: the working tree, then `HEAD`'s README and git's status when the folder
/// is a repository (two git children, spaced, and a third for a project that keeps a manual).
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
    /// The last commit of `docs/manual.md` while the file is as committed (EP-57a): asked only
    /// of a project that keeps one; `None` otherwise, when an edit is not yet committed, or when
    /// git does not track the file or could not say (its own time dates it then).
    pub manual: Option<ManualCommit>,
}

/// The manual's last commit as the git pass read it (EP-57a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManualCommit {
    /// The commit's time, in seconds since 1970.
    pub at: i64,
    /// The file's own time when git was asked: a newer one since means an edit the next git pass
    /// has not seen, which the file's own time dates meanwhile.
    pub file: i64,
}

impl GitReading {
    /// Puts the roadmap's part of the reading into a snapshot of the working tree.
    pub fn fill(self, snapshot: &mut Snapshot) {
        snapshot.head = self.head;
        snapshot.git = self.status;
    }
}

/// Asks git about `folder` (two children, spaced, and a third for the manual's date, EP-57a);
/// `None` when it is not a repository.
pub fn read_git(folder: &Path) -> Option<GitReading> {
    if !folder.join(".git").exists() {
        return None;
    }
    let head = git::run(folder, &["show", &format!("HEAD:{README_IN_REPO}")])
        .ok()
        .map(|text| table::parse(&text));
    thread::sleep(process::SPACING);
    let worktree = git::status(folder).ok();
    let manual = manual_commit(folder, worktree.as_ref());
    Some(GitReading {
        head,
        status: worktree.as_ref().map(roadmap_changes),
        worktree,
        manual,
    })
}

/// The manual's last commit while the file is as committed (EP-57a): a third child, spaced, for a
/// project that keeps `docs/manual.md` alone. An edit not yet committed, staged or not, or a file
/// git does not track (a folder of untracked files is named by itself, with its `/`), asks
/// nothing; nor does a status git could not give.
fn manual_commit(folder: &Path, worktree: Option<&git::Status>) -> Option<ManualCommit> {
    let path = Which::Manual.source();
    let file = manual::Present::read(folder).manual?;
    let changed = worktree?.changes.iter().any(|change| {
        change.path == path || (change.path.ends_with('/') && path.starts_with(&change.path))
    });
    if changed {
        return None;
    }
    thread::sleep(process::SPACING);
    let at = git::run(folder, &["log", "-1", "--format=%ct", "--", &path])
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(ManualCommit { at, file })
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

    const TABLE: &str =
        "| # | Brief | Size | Depends on | Core | Done |\n|---|---|---|---|---|---|\n";

    /// The last phase whose rows are all checked, dated by its last brief's completion, else by
    /// a heading that says done, never by one that says planned (EP-57a).
    #[test]
    fn names_the_last_phase_closed_and_its_date() {
        let scratch = crate::files::tests::Scratch::new("snapshot-last-close");
        let folder = scratch.0.join("alpha");
        fs::create_dir_all(folder.join("roadmap")).unwrap();
        let readme = |p1_done: bool| {
            format!(
                "# Roadmap\n\n## Phase P0 — Planning (done 2026-09-14)\n\n{TABLE}| EP-0 | Plan | \
                 L | — | core | ☑ `aaaaaaa` |\n\n## Phase P1 — Build (planned 2026-09-14)\n\n\
                 {TABLE}| EP-1 | One | S | EP-0 | core | ☑ `bbbbbbb` |\n| EP-2 | Re-plan P1 | S \
                 | EP-1 | core | {} |\n\n## Phase P2 — More (planned 2026-09-30)\n\n{TABLE}| EP-3 \
                 | Three | M | EP-2 | core | ☐ |\n",
                if p1_done { "☑ `ccccccc`" } else { "☐" }
            )
        };
        let write = |text: String| fs::write(folder.join("roadmap").join("README.md"), text);
        // P1 open: P0 closed last, dated by its heading.
        write(readme(false)).unwrap();
        assert_eq!(
            read_tree(&folder).last_close(),
            Some(("P0".to_owned(), Some("2026-09-14".to_owned())))
        );
        // P1 closed, its re-plan without a brief file: a heading that says planned dates nothing.
        write(readme(true)).unwrap();
        assert_eq!(
            read_tree(&folder).last_close(),
            Some(("P1".to_owned(), None))
        );
        // With the re-plan's brief and its completion note, that note's date.
        fs::write(
            folder.join("roadmap").join("EP-2-replan-p1.md"),
            "# EP-2 — Re-plan P1\n\n> **Completion note (2026-09-30; opus/max; 40 min).** Done.\n",
        )
        .unwrap();
        assert_eq!(
            read_tree(&folder).last_close(),
            Some(("P1".to_owned(), Some("2026-09-30".to_owned())))
        );
        // No roadmap, no close.
        assert_eq!(Snapshot::default().last_close(), None);
    }

    /// The git pass asks the manual's last commit only of a project that keeps one, and only
    /// while the file is as committed: untracked (its folder named by itself) or edited, the
    /// file's own time dates it (EP-57a).
    #[test]
    fn the_git_pass_dates_the_manual_while_it_is_as_committed() {
        let repo = crate::revert::testing::Repo::new("snapshot-manual-commit");
        let folder = repo.0.clone();
        let date = |folder: &Path| read_git(folder).unwrap().manual.map(|commit| commit.at);
        repo.write("README.md", "alpha\n");
        repo.commit("first");
        assert_eq!(date(&folder), None, "no manual");
        // Untracked, its folder named by itself; then inside a folder git tracks, by its path.
        repo.write("docs/manual.md", "# Alpha\n");
        assert_eq!(date(&folder), None, "an untracked folder");
        repo.write("docs/notes.md", "notes\n");
        repo.commit_paths("notes", &["docs/notes.md"]);
        assert_eq!(date(&folder), None, "an untracked file");
        repo.commit("the manual");
        let committed: i64 = repo
            .git(&["log", "-1", "--format=%ct"])
            .trim()
            .parse()
            .unwrap();
        let reading = read_git(&folder).unwrap().manual.expect("the commit");
        assert_eq!(reading.at, committed);
        assert_eq!(Some(reading.file), manual::Present::read(&folder).manual);
        // An edit, staged or not, asks nothing.
        repo.write("docs/manual.md", "# Alpha\n\nMore.\n");
        assert_eq!(date(&folder), None, "edited");
        repo.git(&["add", "docs/manual.md"]);
        assert_eq!(date(&folder), None, "staged");
        // Without git's status nothing is asked.
        repo.commit("more");
        assert!(date(&folder).is_some());
        assert_eq!(manual_commit(&folder, None), None);
    }
}
