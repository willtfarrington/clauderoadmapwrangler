//! Reverting a session (D-11, D-26; EP-15b): a project put back as it was when a session started.
//! The revert uses git, back to a start the tool recorded ([`start`]). With none recorded, it goes
//! back to the end of the last brief recorded before the session, the newest `docs(roadmap):
//! record EP-n commit hash` commit made before it started.
//!
//! [`plan`] reads the project and says what a revert would do, or why it is refused. [`revert`]
//! first copies the work out into the backups folder: the commits as a bundle, the uncommitted
//! changes as a binary patch, and the untracked files it removes or overwrites. It then resets,
//! removes the session's new files, puts the start's files back and redoes the tool's own ticks
//! made since. Nothing is fetched or pushed, ignored files are never touched, and git runs one
//! child at a time (D-29). `crw revert` is the console's way in. The window's Revert (EP-15c)
//! reads the plan again at the owner's word and reverts only when [`Plan::is_as_seen`].

pub mod start;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, Utc};

use crate::ledger::{Ledger, SessionRow};
use crate::{git, inbox, process, tasks};
pub use start::{FileState, Start, StartFile};

/// The `event` rows of a revert.
pub const EVENT_KIND: &str = "revert";

/// The `event` rows of the tool's ticks in a project (D-10, D-13), which a revert redoes.
pub const TICK_KIND: &str = "tick";

/// How many commits back the derived target is looked for.
const DERIVED_DEPTH: &str = "500";

/// A tick the tool wrote into a project: an owner task ticked or cleared, an inbox message
/// ticked with its disposition, or (from EP-20) a message the mailman delivered, its words then
/// the message as written after its id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tick {
    pub at: DateTime<Utc>,
    pub id: String,
    pub checked: bool,
    pub disposition: Option<String>,
    /// A delivery (EP-20): `disposition` holds `(<meta>) <text>`.
    pub delivery: bool,
}

impl Tick {
    /// The delivery of message `id`, `words` being `(<meta>) <text>` (EP-20).
    pub fn delivery(at: DateTime<Utc>, id: &str, words: &str) -> Self {
        Self {
            at,
            id: id.to_owned(),
            checked: true,
            disposition: Some(words.to_owned()),
            delivery: true,
        }
    }

    /// Its words in the event row: `ticked OT-3`, `cleared OT-3`, `ticked IN-2 · <disposition>`,
    /// `delivered IN-4 · (<meta>) <text>`.
    pub fn detail(&self) -> String {
        let verb = match (self.delivery, self.checked) {
            (true, _) => "delivered",
            (false, true) => "ticked",
            (false, false) => "cleared",
        };
        match &self.disposition {
            Some(words) => format!("{verb} {} · {words}", self.id),
            None => format!("{verb} {}", self.id),
        }
    }

    /// The tick an event row's words record.
    pub fn parse(at: DateTime<Utc>, detail: &str) -> Option<Self> {
        let (verb, rest) = detail.split_once(' ')?;
        let (checked, delivery) = match verb {
            "ticked" => (true, false),
            "cleared" => (false, false),
            "delivered" => (true, true),
            _ => return None,
        };
        let (id, disposition) = match rest.split_once(" · ") {
            Some((id, words)) => (id, Some(words.to_owned())),
            None => (rest, None),
        };
        Some(Self {
            at,
            id: id.trim().to_owned(),
            checked,
            disposition,
            delivery,
        })
    }
}

/// Logs a tick the tool made in `project` in the ledger at `path`, for a revert to redo. `Err`
/// says why the ledger did not take it.
pub fn log_tick(
    path: &Path,
    project: &str,
    id: &str,
    checked: bool,
    disposition: Option<&str>,
) -> Result<(), String> {
    let tick = Tick {
        at: Utc::now(),
        id: id.to_owned(),
        checked,
        disposition: disposition.map(str::to_owned),
        delivery: false,
    };
    Ledger::open(path)
        .and_then(|ledger| ledger.add_event(tick.at, TICK_KIND, Some(project), &tick.detail()))
        .map(|_| ())
        .map_err(|e| format!("the ledger did not log the tick: {e}"))
}

/// The ticks the tool made in `project` at `since` or later, the oldest first.
pub fn ticks_since(
    ledger: &Ledger,
    project: &str,
    since: DateTime<Utc>,
) -> Result<Vec<Tick>, String> {
    let events = ledger
        .events(TICK_KIND, None)
        .map_err(|e| format!("the ledger's ticks did not read: {e}"))?;
    Ok(events
        .iter()
        .rev()
        .filter(|event| {
            event
                .project
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case(project))
        })
        .filter_map(|event| Tick::parse(event.at()?, event.detail.as_deref()?))
        .filter(|tick| tick.at >= since)
        .collect())
}

/// The session a revert undoes, as the ledger holds it.
#[derive(Debug, Clone)]
pub struct Chosen {
    /// Its first row: when it started, on which EP.
    pub first: SessionRow,
    /// Its recorded start, when one reads.
    pub start: Option<Start>,
    /// Why its recorded start does not read, when it does not.
    pub unreadable: Option<String>,
    /// The other sessions of the project that started after it, each by its first row.
    pub later: Vec<SessionRow>,
}

/// The session of `project` among the ledger's `rows` that `id` names (the whole id, or its first
/// eight characters or more), or the latest one when `id` is none. A visual launch no session took
/// up is none.
pub fn choose(rows: &[SessionRow], project: &str, id: Option<&str>) -> Result<Chosen, String> {
    let mut own: Vec<&SessionRow> = rows
        .iter()
        .filter(|row| row.project.eq_ignore_ascii_case(project) && row.session_id.is_some())
        .collect();
    own.sort_by_key(|row| (row.started(), row.id));
    let session_id = match id {
        Some(id) => {
            let mut matching: BTreeSet<&str> = BTreeSet::new();
            for row in &own {
                let found = row.session_id.as_deref().unwrap_or_default();
                if found == id || (id.len() >= 8 && found.starts_with(id)) {
                    matching.insert(found);
                }
            }
            match matching.len() {
                0 => {
                    return Err(format!(
                        "no session of {project} in the ledger has the id {id}"
                    ));
                }
                1 => matching.into_iter().next().unwrap_or_default().to_owned(),
                n => {
                    return Err(format!(
                        "{n} sessions of {project} have ids that start {id}"
                    ));
                }
            }
        }
        None => own
            .last()
            .and_then(|row| row.session_id.clone())
            .ok_or_else(|| format!("the ledger holds no session of {project}"))?,
    };
    let of_session: Vec<&&SessionRow> = own
        .iter()
        .filter(|row| row.session_id.as_deref() == Some(session_id.as_str()))
        .collect();
    let first = (**of_session.first().ok_or("no such session")?).clone();
    let (start, unreadable) = match of_session.iter().find_map(|row| row.start.as_deref()) {
        Some(json) => match Start::from_json(json) {
            Ok(start) => (Some(start), None),
            Err(why) => (None, Some(why)),
        },
        None => (None, None),
    };
    let mut seen = BTreeSet::new();
    let later = own
        .iter()
        .filter(|row| row.started() > first.started())
        .filter(|row| row.session_id.as_deref() != Some(session_id.as_str()))
        .filter(|row| seen.insert(row.session_id.clone()))
        .map(|row| (*row).clone())
        .collect();
    Ok(Chosen {
        first,
        start,
        unreadable,
        later,
    })
}

/// What the caller knows of the project besides git (EP-15b): its switch, the sessions that work
/// or wait there and those open idle, in words, the tool's ticks since the start, and the
/// backups folder.
#[derive(Debug, Clone)]
pub struct Context<'a> {
    pub project: &'a str,
    pub folder: &'a Path,
    /// The project's `revert` switch in the registry.
    pub enabled: bool,
    /// Sessions that work or wait in the project: each refuses the revert.
    pub working: Vec<String>,
    /// Sessions open there with nothing to do: named, for the owner to close.
    pub idle: Vec<String>,
    pub ticks: Vec<Tick>,
    pub backups: &'a Path,
    pub now: DateTime<Utc>,
}

/// Where a revert goes back to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub commit: String,
    /// The recorded start's HEAD; else derived from the house style's commits.
    pub recorded: bool,
    /// A derived target's subject: `docs(roadmap): record EP-n commit hash`.
    pub subject: Option<String>,
}

/// A commit a revert undoes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub oid: String,
    pub subject: String,
    /// A remote-tracking ref holds it: the revert is refused.
    pub pushed: bool,
}

/// How a file differs from the target, or from the start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Changed,
    Created,
    Deleted,
}

/// What a revert does with a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Back to the target's content, or gone when the target has no such file (`git reset`).
    Undo,
    /// An untracked file the session made: removed.
    Remove,
    /// The start's own uncommitted file, written back as it was then, from the start's copy.
    PutBack,
    /// Deleted at the start: deleted again after the reset brings it back.
    DeleteAgain,
    /// The start's own uncommitted file, which the start could not copy: left as it is now.
    Keep,
}

/// A file a revert changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fate {
    pub path: String,
    pub change: Change,
    pub action: Action,
}

impl Fate {
    /// `changed  src/lib.rs: back as it was at the target`.
    pub fn line(&self) -> String {
        let change = match self.change {
            Change::Changed => "changed",
            Change::Created => "created",
            Change::Deleted => "deleted",
        };
        let action = match self.action {
            Action::Undo => match self.change {
                Change::Created => "removed (the target has no such file)",
                Change::Deleted => "restored as it was at the target",
                Change::Changed => "back as it was at the target",
            },
            Action::Remove => "removed",
            Action::PutBack => "put back as it was at the start",
            Action::DeleteAgain => "deleted again, as it was at the start",
            Action::Keep => "kept as it is (the start did not copy it)",
        };
        format!("{change:<8} {}: {action}", self.path)
    }
}

/// What a revert of one session would do, or why it is refused: [`plan`]'s answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub project: String,
    pub folder: PathBuf,
    pub session_id: Option<String>,
    pub ep: Option<String>,
    pub started: Option<DateTime<Utc>>,
    pub target: Option<Target>,
    /// The branch checked out now.
    pub branch: Option<String>,
    pub commits: Vec<Commit>,
    pub files: Vec<Fate>,
    pub start: Option<Start>,
    pub ticks: Vec<Tick>,
    /// The later sessions whose work is undone too, in words.
    pub later: Vec<String>,
    pub idle: Vec<String>,
    /// Where the work is copied first (a folder named so, or with a number after it).
    pub copy: PathBuf,
    pub refusals: Vec<String>,
    /// The untracked files now that the revert removes or overwrites: copied first.
    untracked: Vec<String>,
    /// Tracked files differ from HEAD: the patch is written.
    uncommitted: bool,
}

impl Plan {
    /// A plan with nothing found in it yet, whose caller sets its fields: a picture's or a
    /// test's, which asks no git (the window's screenshot run, EP-15c). [`plan`] reads the real
    /// one; a plan made so has no uncommitted or untracked files to copy out.
    pub fn new(project: &str, folder: &Path, copy: PathBuf) -> Self {
        Self {
            project: project.to_owned(),
            folder: folder.to_owned(),
            session_id: None,
            ep: None,
            started: None,
            target: None,
            branch: None,
            commits: Vec::new(),
            files: Vec::new(),
            start: None,
            ticks: Vec::new(),
            later: Vec::new(),
            idle: Vec::new(),
            copy,
            refusals: Vec::new(),
            untracked: Vec::new(),
            uncommitted: false,
        }
    }

    pub fn is_refused(&self) -> bool {
        !self.refusals.is_empty()
    }

    /// Whether this plan, read again at the owner's word, is the plan `seen` they confirmed: the
    /// same target, commits, files, start, ticks and sessions, and no refusal new (the window's
    /// Revert, EP-15c). The copy's folder, named by the moment it was read, is left out.
    pub fn is_as_seen(&self, seen: &Plan) -> bool {
        let mut again = self.clone();
        again.copy.clone_from(&seen.copy);
        again == *seen
    }

    fn refuse(&mut self, why: impl Into<String>) {
        self.refusals.push(why.into());
    }

    /// The preview in words, a line each (the window's dialog shows the same, EP-15c).
    pub fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let session = self.session_id.as_deref().map_or("-", short);
        let ep = self
            .ep
            .as_deref()
            .map(|ep| format!(" on {ep}"))
            .unwrap_or_default();
        let started = self
            .started
            .map(|at| format!(", started {}", local(at)))
            .unwrap_or_default();
        lines.push(format!(
            "Revert {}'s session {session}{ep}{started}.",
            self.project
        ));
        match &self.target {
            Some(target) if target.recorded => lines.push(format!(
                "Target: {} on {}, the project's HEAD at the session's start.",
                short(&target.commit),
                self.branch.as_deref().unwrap_or("a detached HEAD")
            )),
            Some(target) => lines.push(format!(
                "Target: {} \"{}\", the last brief recorded before the session. No start was \
                 recorded, so changes left uncommitted before the session cannot be told from \
                 its own: they are undone with them, and kept in the copy.",
                short(&target.commit),
                target.subject.as_deref().unwrap_or_default()
            )),
            None => {}
        }
        if !self.commits.is_empty() {
            lines.push(format!("Commits undone ({}):", self.commits.len()));
            for commit in &self.commits {
                let pushed = if commit.pushed {
                    " (on the remote)"
                } else {
                    ""
                };
                lines.push(format!(
                    "  {} {}{pushed}",
                    short(&commit.oid),
                    commit.subject
                ));
            }
        }
        if !self.files.is_empty() {
            lines.push(format!("Files ({}):", self.files.len()));
            lines.extend(self.files.iter().map(|fate| format!("  {}", fate.line())));
        }
        let mut stays = vec!["ignored files (such as target\\) are not touched".to_owned()];
        if let Some(start) = &self.start
            && !start.files.is_empty()
        {
            let n = start.files.len();
            stays.push(match (start.copied, n) {
                (true, 1) => "the start's own uncommitted file is as it was then".to_owned(),
                (true, n) => format!("the start's own {n} uncommitted files are as they were then"),
                (false, 1) => "the start's own uncommitted file stays as it is: it was more than \
                               the tool copies"
                    .to_owned(),
                (false, n) => format!(
                    "the start's own {n} uncommitted files stay as they are: there were more \
                     than the tool copies"
                ),
            });
        }
        lines.push(format!("Stays: {}.", stays.join("; ")));
        if !self.later.is_empty() {
            lines.push(format!(
                "Later sessions here, whose work is undone too: {}.",
                self.later.join("; ")
            ));
        }
        if !self.idle.is_empty() {
            lines.push(format!(
                "Open there with nothing to do: {}. Close it after the revert: its context still \
                 holds the undone work.",
                self.idle.join("; ")
            ));
        }
        if !self.ticks.is_empty() {
            let ticks: Vec<String> = self.ticks.iter().map(Tick::detail).collect();
            lines.push(format!(
                "The tool's ticks since the start, redone after: {}.",
                ticks.join("; ")
            ));
        }
        lines.push(format!(
            "The work is copied first to {}.",
            self.copy.display()
        ));
        for why in &self.refusals {
            lines.push(format!("Refused: {why}"));
        }
        lines
    }
}

/// What a revert did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Done {
    /// The folder that holds the work copied out.
    pub copy: PathBuf,
    pub target: String,
    pub commits: usize,
    pub undone: usize,
    pub removed: usize,
    pub put_back: usize,
    pub kept: usize,
    pub redone: Vec<String>,
    pub not_redone: Vec<String>,
    /// What went wrong after the reset, file by file; the copy holds the work.
    pub problems: Vec<String>,
}

impl Done {
    pub fn lines(&self) -> Vec<String> {
        let parts: Vec<String> = [
            (self.commits, "commit", "undone"),
            (self.undone, "file", "back as at the target"),
            (self.removed, "file", "removed"),
            (self.put_back, "file", "put back as at the start"),
            (self.kept, "file", "kept as it was"),
        ]
        .into_iter()
        .filter(|(n, ..)| *n > 0)
        .map(|(n, what, how)| format!("{} {how}", count(n, what)))
        .collect();
        let mut lines = vec![format!(
            "Reverted to {}: {}.",
            short(&self.target),
            if parts.is_empty() {
                "nothing to undo".to_owned()
            } else {
                parts.join(", ")
            }
        )];
        lines.push(format!(
            "The work is kept in {} (revert.txt says what each part holds).",
            self.copy.display()
        ));
        if !self.redone.is_empty() {
            lines.push(format!("Redone: {}.", self.redone.join("; ")));
        }
        if !self.not_redone.is_empty() {
            lines.push(format!(
                "Not redone, to do by hand: {}.",
                self.not_redone.join("; ")
            ));
        }
        lines.extend(self.problems.iter().map(|why| format!("Problem: {why}")));
        lines
    }
}

/// An id or a commit's first eight characters.
pub fn short(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

/// `1 commit`, `2 commits`.
fn count(n: usize, what: &str) -> String {
    match n {
        1 => format!("1 {what}"),
        n => format!("{n} {what}s"),
    }
}

fn local(at: DateTime<Utc>) -> String {
    at.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

/// `git <args>` in `folder` in its turn (D-29).
fn git_turn(folder: &Path, args: &[&str]) -> Result<String, String> {
    let _turn = process::turn();
    git::run(folder, args)
}

/// Reads the project and says what a revert of the `chosen` session would do (EP-15b): the
/// target, the commits and files it undoes, what stays, and the refusals. Git is asked one child
/// at a time; nothing is written.
pub fn plan(chosen: &Chosen, cx: &Context<'_>) -> Plan {
    let stamp = cx.now.format("%Y%m%d-%H%M%S").to_string();
    let copy = start::project_dir(cx.backups, cx.folder)
        .join("reverts")
        .join(stamp);
    let mut plan = Plan {
        session_id: chosen.first.session_id.clone(),
        ep: chosen.first.ep.clone(),
        started: chosen.first.started(),
        start: chosen.start.clone(),
        ticks: cx.ticks.clone(),
        later: chosen.later.iter().map(later_words).collect(),
        idle: cx.idle.clone(),
        ..Plan::new(cx.project, cx.folder, copy)
    };
    if !cx.enabled {
        plan.refuse(format!(
            "revert is off for {}: `crw set {} revert yes` turns it on",
            cx.project, cx.project
        ));
    }
    for working in &cx.working {
        plan.refuse(format!("{working}: revert once it has ended"));
    }
    if let Some(why) = &chosen.unreadable {
        plan.refuse(why.clone());
    }
    let status = match git_turn_status(cx.folder) {
        Ok(status) => status,
        Err(why) => {
            plan.refuse(format!("git status did not answer: {why}"));
            return plan;
        }
    };
    plan.branch = status.branch.clone();
    if in_progress(cx.folder)
        || status
            .changes
            .iter()
            .any(|change| change.kind == git::ChangeKind::Unmerged)
    {
        plan.refuse("a merge, a rebase or a cherry-pick is in progress there");
    }
    if status.oid.is_none() {
        plan.refuse("the project has no commit");
        return plan;
    }
    if status.branch.is_none() {
        plan.refuse("HEAD is detached: check out the session's branch first");
    }
    let target = match &chosen.start {
        Some(start) => match &start.head {
            Some(commit) => {
                if start.branch != status.branch {
                    plan.refuse(format!(
                        "the branch changed since the start: {} then, {} now",
                        start.branch.as_deref().unwrap_or("a detached HEAD"),
                        status.branch.as_deref().unwrap_or("a detached HEAD")
                    ));
                }
                Target {
                    commit: commit.clone(),
                    recorded: true,
                    subject: None,
                }
            }
            None => {
                plan.refuse("the project had no commit at the session's start");
                return plan;
            }
        },
        None => match derived(cx.folder, chosen.first.started()) {
            Ok(Some((commit, subject))) => Target {
                commit,
                recorded: false,
                subject: Some(subject),
            },
            Ok(None) => {
                plan.refuse(
                    "no start was recorded, and no `record EP-n commit hash` commit was made \
                     before the session",
                );
                return plan;
            }
            Err(why) => {
                plan.refuse(format!("git log did not answer: {why}"));
                return plan;
            }
        },
    };
    let range = format!("{}..HEAD", target.commit);
    let counts = git_turn(
        cx.folder,
        &[
            "rev-list",
            "--left-right",
            "--count",
            &format!("{}...HEAD", target.commit),
        ],
    );
    let (behind, ahead) = match counts.as_deref().map(str::split_whitespace) {
        Ok(mut words) => (
            words.next().and_then(|n| n.parse::<u32>().ok()),
            words.next().and_then(|n| n.parse::<u32>().ok()),
        ),
        Err(_) => (None, None),
    };
    let (Some(behind), Some(ahead)) = (behind, ahead) else {
        plan.refuse(format!(
            "the target {} is not in the repository any more",
            short(&target.commit)
        ));
        plan.target = Some(target);
        return plan;
    };
    if behind > 0 {
        plan.refuse(format!(
            "the target {} is no ancestor of HEAD: the history was rewritten since (a rebase or \
             a reset)",
            short(&target.commit)
        ));
    }
    if ahead > 0 {
        match commits(cx.folder, &range) {
            Ok(commits) => {
                let pushed: Vec<&Commit> = commits.iter().filter(|c| c.pushed).collect();
                if !pushed.is_empty() {
                    let names: Vec<&str> = pushed.iter().map(|c| short(&c.oid)).collect();
                    plan.refuse(format!(
                        "the remote has {} of its commits ({}): the tool never force-pushes; undo \
                         them by hand with new commits (git revert)",
                        pushed.len(),
                        names.join(", ")
                    ));
                }
                plan.commits = commits;
            }
            Err(why) => plan.refuse(format!("git log did not answer: {why}")),
        }
    }
    let diff = git_turn(
        cx.folder,
        &[
            "--no-optional-locks",
            "diff",
            "--name-status",
            "-z",
            "--no-renames",
            &target.commit,
        ],
    );
    let diff = match diff {
        Ok(text) => parse_name_status(&text),
        Err(why) => {
            plan.refuse(format!("git diff did not answer: {why}"));
            Vec::new()
        }
    };
    let untracked: BTreeSet<String> = status
        .changes
        .iter()
        .filter(|change| change.kind == git::ChangeKind::Untracked)
        .map(|change| change.path.clone())
        .collect();
    plan.uncommitted = status
        .changes
        .iter()
        .any(|change| change.kind != git::ChangeKind::Untracked);
    plan.files = fates(cx.folder, chosen.start.as_ref(), &diff, &untracked);
    plan.untracked = plan
        .files
        .iter()
        .filter(|fate| untracked.contains(&fate.path))
        .map(|fate| fate.path.clone())
        .collect();
    if let Some(odd) = plan
        .files
        .iter()
        .find(|fate| start::relative(&fate.path).is_none())
    {
        plan.refuse(format!(
            "git gave a path the tool will not touch: {}",
            odd.path
        ));
    }
    if ahead == 0 && plan.files.is_empty() && plan.refusals.is_empty() {
        plan.refuse("there is nothing to revert: the project is as it was at the start");
    }
    plan.target = Some(target);
    plan
}

fn git_turn_status(folder: &Path) -> Result<git::Status, String> {
    let _turn = process::turn();
    git::status_all(folder)
}

/// A later session in words: `3c9d2e71 on EP-5 (manual, 2026-11-03 21:40)`.
fn later_words(row: &SessionRow) -> String {
    let id = row.session_id.as_deref().map_or("-", short);
    let ep = row
        .ep
        .as_deref()
        .map(|ep| format!(" on {ep}"))
        .unwrap_or_default();
    let when = row.started().map(local).unwrap_or_default();
    format!("{id}{ep} ({}, {when})", row.mode)
}

/// Whether git is in the middle of a merge, a rebase, a cherry-pick or a revert there.
fn in_progress(folder: &Path) -> bool {
    let git = folder.join(".git");
    git.is_dir()
        && [
            "MERGE_HEAD",
            "rebase-merge",
            "rebase-apply",
            "CHERRY_PICK_HEAD",
            "REVERT_HEAD",
        ]
        .iter()
        .any(|name| git.join(name).exists())
}

/// The newest `record EP-n commit hash` commit on HEAD's history made at `before` or earlier,
/// with its subject: the end of the last brief recorded before the session.
fn derived(
    folder: &Path,
    before: Option<DateTime<Utc>>,
) -> Result<Option<(String, String)>, String> {
    let Some(before) = before else {
        return Ok(None);
    };
    let text = git_turn(
        folder,
        &[
            "log",
            "-z",
            "--format=%H%x1f%ct%x1f%s",
            "-n",
            DERIVED_DEPTH,
            "HEAD",
        ],
    )?;
    Ok(text.split('\0').find_map(|record| {
        let mut parts = record.trim().splitn(3, '\x1f');
        let (oid, time, subject) = (parts.next()?, parts.next()?, parts.next()?);
        let time: i64 = time.parse().ok()?;
        let lower = subject.to_lowercase();
        (time <= before.timestamp() && lower.contains("record ep-") && lower.contains("hash"))
            .then(|| (oid.to_owned(), subject.to_owned()))
    }))
}

/// The commits of `range`, the newest first, each marked when a remote-tracking ref holds it
/// (read without a fetch, D-24 addendum).
fn commits(folder: &Path, range: &str) -> Result<Vec<Commit>, String> {
    let log = git_turn(folder, &["log", "-z", "--format=%H%x1f%s", range])?;
    let unpushed = git_turn(folder, &["rev-list", range, "--not", "--remotes"])?;
    let unpushed: BTreeSet<&str> = unpushed.lines().map(str::trim).collect();
    Ok(log
        .split('\0')
        .filter_map(|record| {
            let (oid, subject) = record.trim().split_once('\x1f')?;
            Some(Commit {
                oid: oid.to_owned(),
                subject: subject.to_owned(),
                pushed: !unpushed.contains(oid),
            })
        })
        .collect())
}

/// `git diff --name-status -z`: a status letter, then its path, each ending in NUL.
fn parse_name_status(text: &str) -> Vec<(Change, String)> {
    let mut fields = text.split('\0').filter(|field| !field.is_empty());
    let mut out = Vec::new();
    while let (Some(status), Some(path)) = (fields.next(), fields.next()) {
        let change = match status.chars().next() {
            Some('A') => Change::Created,
            Some('D') => Change::Deleted,
            _ => Change::Changed,
        };
        out.push((change, path.to_owned()));
    }
    out
}

/// Whether the file at `a` differs from the one at `b`, a missing file differing from any.
fn differs(a: &Path, b: &Path) -> bool {
    match (fs::read(a), fs::read(b)) {
        (Ok(a), Ok(b)) => a != b,
        _ => true,
    }
}

/// What a revert does with each file: those that differ from the target (`diff`), the untracked
/// ones, and the start's own.
fn fates(
    folder: &Path,
    start: Option<&Start>,
    diff: &[(Change, String)],
    untracked: &BTreeSet<String>,
) -> Vec<Fate> {
    let copied = start.is_some_and(|start| start.copied);
    let of_start = |path: &str| start.and_then(|start| start.file(path));
    let mut fates: BTreeMap<String, Fate> = BTreeMap::new();
    // A start's file already as it was then is not listed: the reset changes it and the revert
    // puts it back, which leaves it as it is.
    let as_at_start = |path: &str| {
        start
            .and_then(|start| start.copy_of(path))
            .is_some_and(|copy| !differs(&folder.join(path), &copy))
    };
    for (change, path) in diff {
        let action = match of_start(path) {
            Some(file) if file.state == FileState::Deleted => {
                if !folder.join(path).exists() {
                    continue;
                }
                Action::DeleteAgain
            }
            Some(_) if copied => {
                if as_at_start(path) {
                    continue;
                }
                Action::PutBack
            }
            Some(_) => Action::Keep,
            None => Action::Undo,
        };
        fates.insert(
            path.clone(),
            Fate {
                path: path.clone(),
                change: *change,
                action,
            },
        );
    }
    for path in untracked {
        if fates.contains_key(path) {
            continue;
        }
        let action = match (of_start(path), start) {
            (None, _) => Action::Remove,
            (Some(file), Some(start)) if copied && file.state != FileState::Deleted => {
                match start.copy_of(path) {
                    Some(copy) if differs(&folder.join(path), &copy) => Action::PutBack,
                    // Unchanged since the start.
                    _ => continue,
                }
            }
            // Not copied: left alone.
            _ => continue,
        };
        fates.insert(
            path.clone(),
            Fate {
                path: path.clone(),
                change: Change::Created,
                action,
            },
        );
    }
    for file in start
        .map(|start| start.files.as_slice())
        .unwrap_or_default()
    {
        if fates.contains_key(&file.path) {
            continue;
        }
        let here = folder.join(&file.path);
        let fate = match file.state {
            FileState::Deleted if here.exists() => Some((Change::Created, Action::DeleteAgain)),
            FileState::Deleted => None,
            _ if copied => {
                let copy = start.and_then(|start| start.copy_of(&file.path));
                match copy {
                    Some(copy) if differs(&here, &copy) => {
                        let change = if here.exists() {
                            Change::Changed
                        } else {
                            Change::Deleted
                        };
                        Some((change, Action::PutBack))
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some((change, action)) = fate {
            fates.insert(
                file.path.clone(),
                Fate {
                    path: file.path.clone(),
                    change,
                    action,
                },
            );
        }
    }
    fates.into_values().collect()
}

/// Reverts as `plan` says (EP-15b), once the owner has seen it: copies the work out, resets to
/// the target, removes the session's untracked files, puts the start's files back, writes the
/// kept files back, redoes the tool's ticks, and logs a `revert` event in `ledger`. `Err` says
/// why nothing was changed; [`Done::problems`] what went wrong after the reset.
pub fn revert(
    plan: &Plan,
    ledger: Option<&Path>,
    backups: &Path,
    today: &str,
) -> Result<Done, String> {
    if plan.is_refused() {
        return Err(plan.refusals.join("; "));
    }
    let target = plan.target.as_ref().ok_or("the plan has no target")?;
    let folder = plan.folder.as_path();
    let copy = start::fresh_dir(&plan.copy)
        .map_err(|e| format!("{}: the copy's folder: {e}", plan.copy.display()))?;
    copy_out(plan, target, &copy).map_err(|why| {
        format!(
            "the work was not copied out, so nothing was changed: {why} (what was copied is in {})",
            copy.display()
        )
    })?;
    git_turn(folder, &["reset", "--hard", "--quiet", &target.commit]).map_err(|why| {
        format!(
            "git reset --hard did not run ({why}); the work is copied to {}",
            copy.display()
        )
    })?;
    let mut done = Done {
        copy: copy.clone(),
        target: target.commit.clone(),
        commits: plan.commits.len(),
        ..Done::default()
    };
    for fate in &plan.files {
        let here = folder.join(&fate.path);
        let result = match fate.action {
            Action::Undo => {
                done.undone += 1;
                Ok(())
            }
            Action::Remove => remove(folder, &fate.path).map(|()| done.removed += 1),
            Action::DeleteAgain => remove_file(&here),
            Action::Keep => {
                let kept = copy.join("kept").join(&fate.path);
                let back = if kept.is_file() {
                    write_back(&kept, &here)
                } else {
                    remove_file(&here)
                };
                back.map(|()| done.kept += 1)
            }
            // Below, with every file the start copied.
            Action::PutBack => Ok(()),
        };
        if let Err(why) = result {
            done.problems.push(format!("{}: {why}", fate.path));
        }
    }
    // Every file of the start as it was then, the ones it copied and the ones it had deleted: the
    // reset may have changed one the preview did not list.
    if let Some(start) = &plan.start {
        for file in &start.files {
            let here = folder.join(&file.path);
            let result = match (file.state, start.copy_of(&file.path)) {
                (FileState::Deleted, _) => remove_file(&here),
                (_, Some(copy)) if differs(&here, &copy) => {
                    write_back(&copy, &here).map(|()| done.put_back += 1)
                }
                _ => Ok(()),
            };
            if let Err(why) = result {
                done.problems.push(format!("{}: {why}", file.path));
            }
        }
    }
    redo_ticks(plan, backups, today, &mut done);
    if let Some(path) = ledger {
        let detail = format!(
            "session {} reverted to {} ({}): {}, {}; the work is kept in {}",
            plan.session_id.as_deref().map_or("-", short),
            short(&target.commit),
            if target.recorded {
                "its recorded start"
            } else {
                "the last brief recorded before it"
            },
            count(done.commits, "commit"),
            count(plan.files.len(), "file"),
            copy.display()
        );
        let logged = Ledger::open(path).and_then(|ledger| {
            ledger.add_event(Utc::now(), EVENT_KIND, Some(&plan.project), &detail)
        });
        if let Err(e) = logged {
            done.problems
                .push(format!("the ledger did not log the revert: {e}"));
        }
    }
    Ok(done)
}

/// Copies the work into `copy` before anything changes: the commits, the uncommitted changes,
/// the untracked files touched, the files kept, and a note on each part.
fn copy_out(plan: &Plan, target: &Target, copy: &Path) -> Result<(), String> {
    let folder = plan.folder.as_path();
    let branch = plan.branch.as_deref().unwrap_or("HEAD");
    let mut parts = vec![format!(
        "The revert of {}'s session {} to {}, at {} (crw, EP-15b).",
        plan.project,
        plan.session_id.as_deref().unwrap_or("-"),
        target.commit,
        local(Utc::now())
    )];
    if !plan.commits.is_empty() {
        let bundle = copy.join("commits.bundle");
        let bundle_arg = bundle.to_string_lossy().into_owned();
        let range = format!("{}..{branch}", target.commit);
        git_turn(
            folder,
            &["bundle", "create", "--quiet", &bundle_arg, &range],
        )?;
        parts.push(format!(
            "commits.bundle: the {} undone; `git fetch <this folder>\\commits.bundle \
             {branch}:recovered` in the project brings them back on a branch of their own.",
            count(plan.commits.len(), "commit")
        ));
    }
    if plan.uncommitted {
        let patch = copy.join("uncommitted.patch");
        let output = format!("--output={}", patch.display());
        git_turn(
            folder,
            &["--no-optional-locks", "diff", "--binary", &output, "HEAD"],
        )?;
        parts.push(
            "uncommitted.patch: the uncommitted changes to tracked files, against the HEAD \
             before the revert; `git apply --binary` applies it there."
                .to_owned(),
        );
    }
    if !plan.untracked.is_empty() {
        for path in &plan.untracked {
            start::copy_under(folder, &copy.join("files"), path)
                .map_err(|e| format!("{path}: {e}"))?;
        }
        parts.push(
            "files\\: the untracked files the revert removed or overwrote, at their paths."
                .to_owned(),
        );
    }
    let kept: Vec<&Fate> = plan
        .files
        .iter()
        .filter(|fate| fate.action == Action::Keep && folder.join(&fate.path).is_file())
        .collect();
    if !kept.is_empty() {
        for fate in &kept {
            start::copy_under(folder, &copy.join("kept"), &fate.path)
                .map_err(|e| format!("{}: {e}", fate.path))?;
        }
        parts.push("kept\\: the files kept as they were, written back after the reset.".to_owned());
    }
    let mut note = parts.join("\n");
    note.push('\n');
    fs::write(copy.join("revert.txt"), note).map_err(|e| format!("revert.txt: {e}"))
}

/// Removes the untracked file `path` under `folder`, then its folders that are left empty.
fn remove(folder: &Path, path: &str) -> Result<(), String> {
    let relative = start::relative(path).ok_or("not a relative path")?;
    remove_file(&folder.join(&relative))?;
    let mut parent = relative.parent();
    while let Some(dir) = parent.filter(|dir| !dir.as_os_str().is_empty()) {
        if fs::remove_dir(folder.join(dir)).is_err() {
            break;
        }
        parent = dir.parent();
    }
    Ok(())
}

/// Removes the file at `path` when it is there.
fn remove_file(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Writes the file at `from` over `to`, making `to`'s folders.
fn write_back(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(from, to).map(|_| ()).map_err(|e| e.to_string())
}

/// Redoes the tool's ticks made since the start, the oldest first (D-10, D-13).
fn redo_ticks(plan: &Plan, backups: &Path, today: &str, done: &mut Done) {
    let folder = plan.folder.as_path();
    // A delivery redone under another id (the old one taken meanwhile): a later tick of the
    // old id in the same redo ticks the message where it is now (EP-20c, the review's (d)).
    let mut renamed: BTreeMap<String, String> = BTreeMap::new();
    for tick in &plan.ticks {
        if tick.delivery {
            // A message the mailman delivered (EP-20): appended again unless it is there. A row
            // the ledger cut at its cap holds no whole message, so it is named instead.
            let words = tick.disposition.as_deref().unwrap_or_default();
            if tick.detail().chars().count() >= crate::ledger::DETAIL_CAP {
                done.not_redone.push(format!(
                    "{} (too long for the log: deliver it again by hand)",
                    tick.id
                ));
                continue;
            }
            match crate::inbox::mailman::redeliver(folder, words, backups, today) {
                Ok(redelivered) if redelivered.id.eq_ignore_ascii_case(&tick.id) => {
                    done.redone.push(tick.detail());
                }
                Ok(redelivered) => {
                    done.redone
                        .push(format!("{} (as {})", tick.detail(), redelivered.id));
                    renamed.insert(tick.id.to_ascii_uppercase(), redelivered.id);
                }
                Err(why) => done.not_redone.push(format!("{} ({why})", tick.id)),
            }
            continue;
        }
        let id = renamed
            .get(&tick.id.to_ascii_uppercase())
            .map_or(tick.id.as_str(), String::as_str);
        let said = if id == tick.id {
            tick.detail()
        } else {
            format!("{} (as {id})", tick.detail())
        };
        let result = match (tick.id.split('-').next(), tick.checked) {
            (Some("OT"), true) => tasks::tick(folder, &tick.id, backups, today),
            (Some("OT"), false) => tasks::untick(folder, &tick.id, backups, today),
            (Some("IN"), true) => inbox::tick(
                folder,
                id,
                tick.disposition.as_deref().unwrap_or_default(),
                backups,
                today,
            ),
            _ => {
                done.not_redone.push(tick.detail());
                continue;
            }
        };
        match result {
            Ok(_) => done.redone.push(said),
            Err(
                crate::checklist::Error::AlreadyTicked(_) | crate::checklist::Error::NotTicked(_),
            ) => {}
            Err(why) => done.not_redone.push(format!("{} ({why})", tick.detail())),
        }
    }
}

#[cfg(test)]
pub(crate) mod testing {
    //! Scratch git repositories for the revert's tests, in temp folders and never the owner's
    //! projects (rule 13); each git child in its turn (D-29).

    use std::fs;
    use std::path::{Path, PathBuf};

    use chrono::Utc;

    use crate::files::tests::Scratch;
    use crate::{git, process};

    use super::Start;

    /// A project folder named `alpha` holding a repository on `main`, in a scratch folder that
    /// also holds the backups (`local\backups`).
    pub(crate) struct Repo(pub PathBuf, Scratch);

    impl Repo {
        pub(crate) fn new(name: &str) -> Self {
            let scratch = Scratch::new(name);
            let folder = scratch.0.join("alpha");
            fs::create_dir_all(&folder).unwrap();
            let repo = Self(folder, scratch);
            repo.git(&["init", "--quiet", "-b", "main"]);
            // Bytes as written, whatever the machine's own setting.
            repo.git(&["config", "core.autocrlf", "false"]);
            repo
        }

        pub(crate) fn scratch(&self) -> &Path {
            &self.1.0
        }

        pub(crate) fn backups(&self) -> PathBuf {
            self.1.0.join("local").join("backups")
        }

        pub(crate) fn git(&self, args: &[&str]) -> String {
            let _turn = process::turn();
            git::run(&self.0, args).unwrap_or_else(|e| panic!("git {args:?}: {e}"))
        }

        pub(crate) fn write(&self, path: &str, text: &str) {
            let file = self.0.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, text).unwrap();
        }

        pub(crate) fn read(&self, path: &str) -> Option<String> {
            fs::read_to_string(self.0.join(path)).ok()
        }

        pub(crate) fn remove(&self, path: &str) {
            fs::remove_file(self.0.join(path)).unwrap();
        }

        /// Commits everything not ignored; the new HEAD.
        pub(crate) fn commit(&self, message: &str) -> String {
            self.commit_paths(message, &["-A"])
        }

        /// Commits the changes to `paths` alone, as a session that leaves the owner's files be.
        pub(crate) fn commit_paths(&self, message: &str, paths: &[&str]) -> String {
            let mut add = vec!["add"];
            add.extend(paths);
            self.git(&add);
            self.git(&[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--quiet",
                "-m",
                message,
            ]);
            self.head()
        }

        pub(crate) fn head(&self) -> String {
            self.git(&["rev-parse", "HEAD"]).trim().to_owned()
        }

        pub(crate) fn capture(&self, copies: &Path) -> Start {
            let _turn = process::turn();
            super::start::capture(&self.0, copies, Utc::now()).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::Repo;
    use super::*;
    use chrono::TimeDelta;

    fn row(id: i64, session: Option<&str>, project: &str, at: DateTime<Utc>) -> SessionRow {
        SessionRow {
            id,
            session_id: session.map(str::to_owned),
            project: project.to_owned(),
            ep: Some("EP-4".to_owned()),
            mode: "headless".to_owned(),
            model: None,
            effort: None,
            ultracode: false,
            started_at: at.to_rfc3339(),
            ended_at: None,
            outcome: None,
            cost_usd: None,
            questions: 0,
            result: None,
            context_percent: None,
            answer_latency_s: None,
            start: None,
            usage: Default::default(),
        }
    }

    fn context<'a>(repo: &'a Repo, backups: &'a Path) -> Context<'a> {
        Context {
            project: "alpha",
            folder: &repo.0,
            enabled: true,
            working: Vec::new(),
            idle: Vec::new(),
            ticks: Vec::new(),
            backups,
            now: Utc::now(),
        }
    }

    fn chosen(first: SessionRow, start: Option<Start>) -> Chosen {
        Chosen {
            first,
            start,
            unreadable: None,
            later: Vec::new(),
        }
    }

    #[test]
    fn the_session_is_chosen_by_its_id_or_as_the_latest() {
        let at = |minute: i64| Utc::now() - TimeDelta::minutes(100 - minute);
        let mut first = row(1, Some("aaaaaaaa-1111"), "alpha", at(0));
        first.start = Some("{".to_owned());
        let rows = [
            first,
            row(2, Some("bbbbbbbb-2222"), "alpha", at(10)),
            row(3, Some("aaaaaaaa-1111"), "alpha", at(20)),
            row(4, None, "alpha", at(30)),
            row(5, Some("cccccccc-3333"), "beta", at(40)),
        ];
        let ids = |rows: &[SessionRow]| rows.iter().map(|row| row.id).collect::<Vec<_>>();
        // The session of the latest row with an id (a visual launch no session took up has
        // none), by its first row: row 3 resumed row 1's session.
        let latest = choose(&rows, "ALPHA", None).unwrap();
        assert_eq!(latest.first.id, 1);
        assert_eq!(ids(&latest.later), [2], "the session started after it");
        assert!(latest.unreadable.is_some(), "its start does not read");
        assert_eq!(
            choose(&rows, "alpha", Some("aaaaaaaa")).unwrap().first.id,
            1
        );
        let other = choose(&rows, "alpha", Some("bbbbbbbb")).unwrap();
        assert_eq!(other.first.id, 2);
        assert_eq!(ids(&other.later), [3], "the resumed run worked after it");
        assert!(other.start.is_none() && other.unreadable.is_none());
        assert!(choose(&rows, "alpha", Some("aaaa")).is_err(), "too short");
        assert!(choose(&rows, "alpha", Some("cccccccc-3333")).is_err());
        assert!(choose(&rows, "gamma", None).is_err());
    }

    #[test]
    fn ticks_read_back_from_their_words() {
        let at = Utc::now();
        for (id, checked, words) in [
            ("OT-3", true, None),
            ("OT-3", false, None),
            ("IN-2", true, Some("rolled into EP-5 · as asked")),
        ] {
            let tick = Tick {
                at,
                id: id.to_owned(),
                checked,
                disposition: words.map(str::to_owned),
                delivery: false,
            };
            assert_eq!(Tick::parse(at, &tick.detail()), Some(tick));
        }
        let delivered = Tick::delivery(at, "IN-4", "(from beta EP-6, 2026-11-10) A note · two");
        assert_eq!(
            delivered.detail(),
            "delivered IN-4 · (from beta EP-6, 2026-11-10) A note · two"
        );
        assert_eq!(Tick::parse(at, &delivered.detail()), Some(delivered));
        assert_eq!(Tick::parse(at, "toast · waiting"), None);
    }

    /// A session that committed, edited the owner's uncommitted file, created, deleted and
    /// ignored files, reverted to its recorded start: the work copied out first, HEAD back,
    /// the start's file as it was, the new file gone, the ignored file and the start's
    /// untracked draft kept, a tick redone.
    #[test]
    fn a_session_is_reverted_to_its_recorded_start() {
        let repo = Repo::new("revert-recorded");
        repo.write(".gitignore", "target/\n");
        repo.write("src/lib.rs", "fn main() {}\n");
        repo.write("gone.txt", "kept by the start\n");
        repo.write("wip.md", "the owner's draft\n");
        repo.write(
            "roadmap/owner-tasks.md",
            "# Owner tasks\n\n- [ ] OT-1 (EP-1, 2026-11-01) Look.\n",
        );
        repo.commit("first");
        repo.write("wip.md", "the owner's draft, edited before the session\n");
        repo.write("notes/idea.txt", "the owner's untracked note\n");
        let backups = repo.backups();
        let start = repo.capture(&start::copies_dir(&backups, &repo.0, "s-1"));
        let start_head = start.head.clone().unwrap();

        // The session's work: two commits, then more left uncommitted.
        repo.write("src/lib.rs", "fn main() { session(); }\n");
        repo.write("src/new.rs", "// the session's module\n");
        repo.commit_paths("Session work: a module", &["src"]);
        repo.remove("gone.txt");
        repo.commit_paths("Session work: a deletion", &["gone.txt"]);
        repo.write("wip.md", "the session rewrote the owner's draft\n");
        repo.write("scratch/out.txt", "the session's untracked output\n");
        repo.write("target/build.log", "ignored\n");
        // The owner ticks OT-1 in the window meanwhile; the session's own edit loses it.
        let ticked = Tick {
            at: Utc::now(),
            id: "OT-1".to_owned(),
            checked: true,
            disposition: None,
            delivery: false,
        };

        let mut cx = context(&repo, &backups);
        cx.ticks = vec![ticked];
        cx.idle = vec!["a manual session (since 10:02)".to_owned()];
        let first = row(1, Some("s-1"), "alpha", start.at);
        let plan = plan(&chosen(first, Some(start)), &cx);
        assert!(!plan.is_refused(), "{:?}", plan.refusals);
        let target = plan.target.clone().unwrap();
        assert_eq!(
            (target.commit.as_str(), target.recorded),
            (start_head.as_str(), true)
        );
        assert_eq!(plan.commits.len(), 2);
        assert!(plan.commits.iter().all(|commit| !commit.pushed));
        let fates: Vec<(&str, Action)> = plan
            .files
            .iter()
            .map(|fate| (fate.path.as_str(), fate.action))
            .collect();
        assert_eq!(
            fates,
            [
                ("gone.txt", Action::Undo),
                ("scratch/out.txt", Action::Remove),
                ("src/lib.rs", Action::Undo),
                ("src/new.rs", Action::Undo),
                ("wip.md", Action::PutBack),
            ]
        );
        let preview = plan.lines().join("\n");
        assert!(preview.contains("Commits undone (2):"), "{preview}");
        assert!(preview.contains("put back as it was at the start"));
        assert!(preview.contains("Close it after the revert"));
        assert!(preview.contains("ticked OT-1"));

        let ledger = repo.scratch().join("ledger.sqlite");
        let done = revert(&plan, Some(&ledger), &backups, "2026-11-04").unwrap();
        assert!(done.problems.is_empty(), "{:?}", done.problems);
        assert_eq!(repo.head(), start_head);
        assert_eq!(repo.read("src/lib.rs").as_deref(), Some("fn main() {}\n"));
        assert_eq!(repo.read("src/new.rs"), None);
        assert_eq!(
            repo.read("gone.txt").as_deref(),
            Some("kept by the start\n")
        );
        assert_eq!(
            repo.read("wip.md").as_deref(),
            Some("the owner's draft, edited before the session\n")
        );
        assert_eq!(
            repo.read("notes/idea.txt").as_deref(),
            Some("the owner's untracked note\n")
        );
        assert_eq!(repo.read("scratch/out.txt"), None);
        assert!(!repo.0.join("scratch").exists(), "its emptied folder too");
        assert_eq!(repo.read("target/build.log").as_deref(), Some("ignored\n"));
        assert!(
            repo.read("roadmap/owner-tasks.md")
                .unwrap()
                .contains("- [x] OT-1"),
            "the owner's tick redone"
        );
        assert_eq!(done.redone, ["ticked OT-1"]);
        assert_eq!((done.commits, done.removed, done.put_back), (2, 1, 1));
        let said = &done.lines()[0];
        assert!(
            said.contains("2 commits undone") && said.contains("1 file removed"),
            "{said}"
        );

        // The work is in the copy.
        let copy = &done.copy;
        assert!(copy.join("commits.bundle").is_file());
        let patch = fs::read_to_string(copy.join("uncommitted.patch")).unwrap();
        assert!(patch.contains("the session rewrote the owner's draft"));
        assert_eq!(
            fs::read_to_string(copy.join("files").join("scratch").join("out.txt")).unwrap(),
            "the session's untracked output\n"
        );
        assert!(
            fs::read_to_string(copy.join("revert.txt"))
                .unwrap()
                .contains("commits.bundle")
        );
        let read = Ledger::open_read_only(&ledger).unwrap();
        let events = read.events(EVENT_KIND, None).unwrap();
        assert_eq!(events.len(), 1);
        assert!(
            events[0]
                .detail
                .as_deref()
                .unwrap()
                .contains("its recorded start")
        );

        // Again: only the redone tick differs from the start.
        let again = super::plan(
            &chosen(
                row(1, Some("s-1"), "alpha", plan.started.unwrap()),
                plan.start.clone(),
            ),
            &context(&repo, &backups),
        );
        assert!(again.commits.is_empty());
        assert_eq!(
            again.files,
            [Fate {
                path: "roadmap/owner-tasks.md".to_owned(),
                change: Change::Changed,
                action: Action::Undo,
            }]
        );
    }

    /// A message the mailman delivered during the session is the tool's write: the revert removes
    /// the file the session's span created, then delivers the message again (EP-20).
    #[test]
    fn a_delivery_during_the_session_is_redone() {
        let repo = Repo::new("revert-delivery");
        repo.write("src/lib.rs", "fn main() {}\n");
        repo.commit("first");
        let backups = repo.backups();
        let start = repo.capture(&start::copies_dir(&backups, &repo.0, "s-1"));
        repo.write("src/lib.rs", "fn main() { session(); }\n");
        repo.commit("Session work");
        let words = "(from beta EP-6, 2026-11-10) The export gains a column.";
        crate::inbox::mailman::redeliver(&repo.0, words, &backups, "2026-11-10").unwrap();
        let mut cx = context(&repo, &backups);
        cx.ticks = vec![Tick::delivery(Utc::now(), "IN-1", words)];
        let plan = plan(
            &chosen(row(1, Some("s-1"), "alpha", start.at), Some(start)),
            &cx,
        );
        assert!(!plan.is_refused(), "{:?}", plan.refusals);
        let preview = plan.lines().join("\n");
        assert!(preview.contains("delivered IN-1"), "{preview}");
        let ledger = repo.scratch().join("ledger.sqlite");
        let done = revert(&plan, Some(&ledger), &backups, "2026-11-10").unwrap();
        assert!(done.problems.is_empty(), "{:?}", done.problems);
        assert_eq!(repo.read("src/lib.rs").as_deref(), Some("fn main() {}\n"));
        assert_eq!(done.redone, [format!("delivered IN-1 · {words}")]);
        let inbox = repo.read("roadmap/inbox.md").unwrap();
        assert!(
            inbox.ends_with("- [ ] IN-1 (from beta EP-6, 2026-11-10) The export gains a column.\n"),
            "{inbox}"
        );
    }

    /// A delivery redone under another id, the old one taken at the redo (the session had
    /// created the inbox with a message of its own before the mailman's): a later tick of the
    /// old id in the same redo ticks the message where it is now (EP-20c, the review's (d)).
    #[test]
    fn a_delivery_redone_under_another_id_carries_its_tick_along() {
        let repo = Repo::new("revert-delivery-id");
        repo.write("src/lib.rs", "fn main() {}\n");
        repo.commit("first");
        let backups = repo.backups();
        let start = repo.capture(&start::copies_dir(&backups, &repo.0, "s-1"));
        // The session writes the inbox itself, with a message of its own, and commits it.
        repo.write(
            "roadmap/inbox.md",
            "# Inbox\n\n- [ ] IN-1 (from the owner, 2026-11-10) The session's own.\n",
        );
        repo.commit("Session work");
        // Then the mailman delivers, under the next id, and the owner ticks that message.
        let words = "(from beta EP-6, 2026-11-10) The export gains a column.";
        let delivered =
            crate::inbox::mailman::redeliver(&repo.0, words, &backups, "2026-11-10").unwrap();
        assert_eq!(delivered.id, "IN-2");
        inbox::tick(&repo.0, "IN-2", "rolled in", &backups, "2026-11-10").unwrap();
        let mut cx = context(&repo, &backups);
        let at = Utc::now();
        cx.ticks = vec![
            Tick::delivery(at, "IN-2", words),
            Tick {
                at,
                id: "IN-2".to_owned(),
                checked: true,
                disposition: Some("rolled in".to_owned()),
                delivery: false,
            },
        ];
        let plan = plan(
            &chosen(row(1, Some("s-1"), "alpha", start.at), Some(start)),
            &cx,
        );
        assert!(!plan.is_refused(), "{:?}", plan.refusals);
        let done = revert(&plan, None, &backups, "2026-11-10").unwrap();
        assert!(done.problems.is_empty(), "{:?}", done.problems);
        // The session's commit undone, the inbox is gone: the message comes back as IN-1, and
        // the tick of IN-2 follows it there.
        assert_eq!(
            done.redone,
            [
                format!("delivered IN-2 · {words} (as IN-1)"),
                "ticked IN-2 · rolled in (as IN-1)".to_owned()
            ]
        );
        assert!(done.not_redone.is_empty(), "{:?}", done.not_redone);
        let inbox = repo.read("roadmap/inbox.md").unwrap();
        assert!(
            inbox.ends_with(
                "- [x] IN-1 (from beta EP-6, 2026-11-10) The export gains a column. \
                 **Disposition (2026-11-10):** rolled in\n"
            ),
            "{inbox}"
        );
        assert!(!inbox.contains("IN-2"), "{inbox}");
    }

    /// Without a recorded start, the target is the last brief recorded before the session; a
    /// start above the cap keeps its files as they are.
    #[test]
    fn a_session_without_a_start_goes_back_to_the_last_recorded_brief() {
        let repo = Repo::new("revert-derived");
        repo.write("roadmap/README.md", "| EP-1 | ☐ |\n");
        repo.commit("First: a roadmap (EP-1)");
        repo.write("roadmap/README.md", "| EP-1 | ☑ `1234567` |\n");
        let recorded = repo.commit("docs(roadmap): record EP-1 commit hash");
        // The session starts a second later; the commit times count in whole seconds.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let started = Utc::now();
        repo.write("src/a.rs", "// the session\n");
        repo.commit("Session work (EP-2)");
        repo.write("draft.txt", "untracked\n");
        let backups = repo.backups();
        let plan = plan(
            &chosen(row(1, Some("s-2"), "alpha", started), None),
            &context(&repo, &backups),
        );
        assert!(!plan.is_refused(), "{:?}", plan.refusals);
        let target = plan.target.clone().unwrap();
        assert_eq!(target.commit, recorded);
        assert!(!target.recorded);
        assert_eq!(
            target.subject.as_deref(),
            Some("docs(roadmap): record EP-1 commit hash")
        );
        assert!(plan.lines().join(" ").contains("cannot be told from"));
        let done = revert(&plan, None, &backups, "2026-11-04").unwrap();
        assert_eq!(repo.head(), recorded);
        assert_eq!(repo.read("draft.txt"), None);
        assert_eq!(done.removed, 1);

        // Before any recorded brief, there is no target.
        let early = Utc::now() - TimeDelta::days(1);
        let none = super::plan(
            &chosen(row(1, Some("s-3"), "alpha", early), None),
            &context(&repo, &backups),
        );
        assert!(
            none.refusals
                .iter()
                .any(|why| why.contains("no start was recorded"))
        );
    }

    /// A start above the cap listed its files and copied none: a revert leaves them as they are.
    #[test]
    fn a_start_without_its_copies_keeps_its_files_as_they_are() {
        let repo = Repo::new("revert-kept");
        repo.write("big.dat", "committed\n");
        let first_head = repo.commit("first");
        repo.write("big.dat", "the owner's large edit\n");
        repo.write("huge/draft.dat", "the owner's large untracked draft\n");
        let start = Start {
            at: Utc::now(),
            head: Some(first_head.clone()),
            branch: Some("main".to_owned()),
            files: vec![
                StartFile {
                    path: "big.dat".to_owned(),
                    state: FileState::Changed,
                },
                StartFile {
                    path: "huge/draft.dat".to_owned(),
                    state: FileState::Untracked,
                },
            ],
            copy: None,
            copied: false,
        };
        repo.write("src/a.rs", "// the session\n");
        repo.commit_paths("Session work", &["src"]);
        repo.write("big.dat", "the owner's large edit, and the session's\n");
        let backups = repo.backups();
        let plan = plan(
            &chosen(row(1, Some("s-4"), "alpha", start.at), Some(start)),
            &context(&repo, &backups),
        );
        assert!(!plan.is_refused(), "{:?}", plan.refusals);
        let fates: Vec<(&str, Action)> = plan
            .files
            .iter()
            .map(|fate| (fate.path.as_str(), fate.action))
            .collect();
        assert_eq!(
            fates,
            [("big.dat", Action::Keep), ("src/a.rs", Action::Undo)]
        );
        assert!(plan.lines().join(" ").contains("stay as they are"));
        let done = revert(&plan, None, &backups, "2026-11-04").unwrap();
        assert_eq!(repo.head(), first_head);
        assert_eq!(
            repo.read("big.dat").as_deref(),
            Some("the owner's large edit, and the session's\n")
        );
        assert_eq!(
            repo.read("huge/draft.dat").as_deref(),
            Some("the owner's large untracked draft\n")
        );
        assert_eq!(repo.read("src/a.rs"), None);
        assert_eq!(done.kept, 1);
        assert!(done.copy.join("kept").join("big.dat").is_file());
    }

    #[test]
    fn a_revert_is_refused_when_history_moved_or_was_pushed() {
        let repo = Repo::new("revert-refused");
        repo.write("a.txt", "one\n");
        repo.commit("first");
        let backups = repo.backups();
        let start = repo.capture(&start::copies_dir(&backups, &repo.0, "s-1"));
        let first = || row(1, Some("s-1"), "alpha", start.at);
        // Before the session works, there is nothing to revert.
        let early = plan(
            &chosen(first(), Some(start.clone())),
            &context(&repo, &backups),
        );
        assert!(
            early
                .refusals
                .iter()
                .any(|why| why.contains("nothing to revert")),
            "{:?}",
            early.refusals
        );
        repo.write("a.txt", "two\n");
        repo.commit("Session work");

        // The switch off, a session at work.
        let mut cx = context(&repo, &backups);
        cx.enabled = false;
        cx.working = vec!["a visual session waits for you".to_owned()];
        let off = plan(&chosen(first(), Some(start.clone())), &cx);
        assert_eq!(off.refusals.len(), 2, "{:?}", off.refusals);
        assert!(revert(&off, None, &backups, "2026-11-04").is_err());

        // Pushed: a scratch bare remote that holds the session's commit.
        let remote = repo.scratch().join("remote.git");
        {
            let _turn = process::turn();
            git::run(repo.scratch(), &["init", "--quiet", "--bare", "remote.git"]).unwrap();
        }
        let remote_arg = remote.to_string_lossy().into_owned();
        repo.git(&["remote", "add", "origin", &remote_arg]);
        repo.git(&["push", "--quiet", "origin", "main"]);
        let pushed = plan(
            &chosen(first(), Some(start.clone())),
            &context(&repo, &backups),
        );
        assert!(
            pushed
                .refusals
                .iter()
                .any(|why| why.contains("never force-pushes")),
            "{:?}",
            pushed.refusals
        );
        assert!(pushed.commits.iter().all(|commit| commit.pushed));

        // Another branch, and a start no ancestor of HEAD.
        repo.git(&["checkout", "--quiet", "-b", "other"]);
        let moved = plan(
            &chosen(first(), Some(start.clone())),
            &context(&repo, &backups),
        );
        assert!(
            moved
                .refusals
                .iter()
                .any(|why| why.contains("the branch changed"))
        );
        let mut elsewhere = start.clone();
        elsewhere.branch = Some("other".to_owned());
        repo.git(&["checkout", "--quiet", "--orphan", "fresh"]);
        repo.commit("an unrelated root");
        repo.git(&["branch", "--quiet", "-M", "other"]);
        let rewritten = plan(&chosen(first(), Some(elsewhere)), &context(&repo, &backups));
        assert!(
            rewritten
                .refusals
                .iter()
                .any(|why| why.contains("no ancestor")),
            "{:?}",
            rewritten.refusals
        );
        // Nothing was changed by any of them.
        assert!(
            !start::project_dir(&backups, &repo.0)
                .join("reverts")
                .exists()
        );
    }

    /// The window's Revert reads the plan again at the owner's word (EP-15c): the same project
    /// gives the plan they saw, whatever moment names its copy; a file made since, a new tick or
    /// a session at work does not.
    #[test]
    fn a_plan_read_again_is_as_seen_until_the_project_changes() {
        let repo = Repo::new("revert-as-seen");
        repo.write("a.txt", "one\n");
        repo.commit("first");
        let backups = repo.backups();
        let start = repo.capture(&start::copies_dir(&backups, &repo.0, "s-1"));
        repo.write("a.txt", "two\n");
        repo.commit("Session work");
        let session = || chosen(row(1, Some("s-1"), "alpha", start.at), Some(start.clone()));
        let seen = plan(&session(), &context(&repo, &backups));
        assert!(!seen.is_refused(), "{:?}", seen.refusals);

        let mut later = context(&repo, &backups);
        later.now += TimeDelta::seconds(30);
        let again = plan(&session(), &later);
        assert_ne!(again.copy, seen.copy, "named by the moment it was read");
        assert!(again.is_as_seen(&seen));

        let mut working = context(&repo, &backups);
        working.working = vec!["a manual session is busy".to_owned()];
        assert!(!plan(&session(), &working).is_as_seen(&seen));
        let mut ticked = context(&repo, &backups);
        ticked.ticks = vec![Tick {
            at: Utc::now(),
            id: "OT-1".to_owned(),
            checked: true,
            disposition: None,
            delivery: false,
        }];
        assert!(!plan(&session(), &ticked).is_as_seen(&seen));
        repo.write("notes.md", "written after the preview\n");
        let changed = plan(&session(), &context(&repo, &backups));
        assert!(!changed.is_refused());
        assert!(!changed.is_as_seen(&seen), "a new file to remove");

        // A plan no git was asked for: nothing in it but what its caller sets.
        let bare = Plan::new("alpha", &repo.0, backups.join("copy"));
        assert!(!bare.is_refused() && bare.files.is_empty() && bare.target.is_none());
    }

    #[test]
    fn the_ledger_logs_ticks_for_a_revert_to_redo() {
        let scratch = crate::files::tests::Scratch::new("revert-ticks");
        let path = scratch.0.join("ledger.sqlite");
        let before = Utc::now() - TimeDelta::seconds(5);
        log_tick(&path, "alpha", "OT-2", true, None).unwrap();
        log_tick(&path, "beta", "OT-9", true, None).unwrap();
        log_tick(&path, "Alpha", "IN-1", true, Some("declined: out of scope")).unwrap();
        let ledger = Ledger::open_read_only(&path).unwrap();
        let ticks = ticks_since(&ledger, "alpha", before).unwrap();
        let words: Vec<String> = ticks.iter().map(Tick::detail).collect();
        assert_eq!(
            words,
            ["ticked OT-2", "ticked IN-1 · declined: out of scope"]
        );
        assert!(
            ticks_since(&ledger, "alpha", Utc::now() + TimeDelta::seconds(5))
                .unwrap()
                .is_empty()
        );
    }
}
