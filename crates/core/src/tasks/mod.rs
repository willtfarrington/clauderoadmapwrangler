//! What waits on the owner between sessions (D-10): `roadmap/owner-tasks.md`, one line per task,
//! `- [ ] OT-n (EP-k, date) text`, optionally `before EP-m`, which the owner ticks in the tool and
//! the tool writes back (D-11); and the older forms of the same to-dos, read-only (`legacy`).
//! Every form reads into one [`OwnerItem`], so the dashboard asks one question.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::checklist;
use crate::roadmap::table::Problem;
use crate::roadmap::{EpId, ep_ids, first_date, scan_ep_id, starts_word};

pub mod legacy;

pub const FILE: &str = "owner-tasks.md";

/// The letters of an owner task's id: `OT-3`.
pub const PREFIX: &str = "OT";

/// `<folder>\roadmap\owner-tasks.md`.
pub fn path(folder: &Path) -> PathBuf {
    folder.join("roadmap").join(FILE)
}

/// A line of `owner-tasks.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub done: bool,
    /// The EP whose session wrote it.
    pub raised: Option<EpId>,
    pub date: Option<String>,
    pub text: String,
    /// `before EP-m`: due before that EP starts.
    pub gate: Option<EpId>,
    pub line: usize,
}

/// `owner-tasks.md` as the parser reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tasks {
    pub path: PathBuf,
    pub tasks: Vec<Task>,
    /// Boxes without an `OT-n` id and ids written twice.
    pub problems: Vec<Problem>,
}

impl Tasks {
    pub fn open(&self) -> impl Iterator<Item = &Task> {
        self.tasks.iter().filter(|task| !task.done)
    }

    pub fn items(&self) -> impl Iterator<Item = OwnerItem> + '_ {
        self.tasks.iter().map(|task| task.item(&self.path))
    }
}

impl Task {
    pub fn item(&self, path: &Path) -> OwnerItem {
        OwnerItem {
            form: Form::Task,
            path: path.to_owned(),
            line: self.line,
            id: Some(self.id.clone()),
            text: self.text.clone(),
            gate: self.gate.map(Gate::Before),
            state: if self.done {
                State::Resolved
            } else {
                State::Open
            },
            raised: self.raised,
            date: self.date.clone(),
        }
    }
}

/// Reads the text of `owner-tasks.md`.
pub fn parse(text: &str) -> (Vec<Task>, Vec<Problem>) {
    let list = checklist::parse(text, PREFIX);
    let tasks = list
        .items
        .into_iter()
        .map(|item| {
            let meta = item.meta.as_deref().unwrap_or_default();
            Task {
                raised: ep_ids(meta).first().copied(),
                date: first_date(meta).map(str::to_owned),
                gate: before(&item.text),
                done: item.checked,
                id: item.id,
                text: item.text,
                line: item.line,
            }
        })
        .collect();
    (tasks, list.problems)
}

/// Reads a project's `owner-tasks.md`; `None` when it has none.
pub fn read(folder: &Path) -> io::Result<Option<Tasks>> {
    let path = path(folder);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let (tasks, problems) = parse(&text);
    Ok(Some(Tasks {
        path,
        tasks,
        problems,
    }))
}

/// Ticks owner task `id` of the project in `folder` (D-10, D-11): the box alone changes, after a
/// copy of the file in `backups`. Returns the copy.
pub fn tick(
    folder: &Path,
    id: &str,
    backups: &Path,
    today: &str,
) -> Result<PathBuf, checklist::Error> {
    checklist::tick(&path(folder), PREFIX, id, true, None, backups, today)
}

/// Clears owner task `id`'s tick, as [`tick`] sets it.
pub fn untick(
    folder: &Path,
    id: &str,
    backups: &Path,
    today: &str,
) -> Result<PathBuf, checklist::Error> {
    checklist::tick(&path(folder), PREFIX, id, false, None, backups, today)
}

/// The EP a `before EP-m` in `text` names (`[before EP-6]`, `(D-25, before EP-21)`), the first
/// when there are several.
pub(crate) fn before(text: &str) -> Option<EpId> {
    text.match_indices("EP-").find_map(|(at, _)| {
        let word = text[..at].trim_end();
        let lead = word.len().checked_sub("before".len())?;
        let precedes = word.get(lead..)?.eq_ignore_ascii_case("before")
            && word.len() < at
            && starts_word(word, lead);
        if !precedes || !starts_word(text, at) {
            return None;
        }
        scan_ep_id(text, at).map(|(id, _)| id)
    })
}

/// One thing waiting on the owner, in any of the forms the projects write (D-10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerItem {
    pub form: Form,
    /// The file it is written in.
    pub path: PathBuf,
    pub line: usize,
    /// `OT-3`, `R-2`, `OD-4`, `OQ-B` or a list's number; `None` for the forms without one.
    pub id: Option<String>,
    /// As written: markup kept, continuation lines joined by line breaks.
    pub text: String,
    pub gate: Option<Gate>,
    pub state: State,
    /// The EP that raised it, when the form says.
    pub raised: Option<EpId>,
    pub date: Option<String>,
}

impl OwnerItem {
    pub fn is_open(&self) -> bool {
        self.state != State::Resolved
    }

    /// Only a line of `owner-tasks.md` is ticked from the tool; the older forms are read-only.
    pub fn is_tickable(&self) -> bool {
        self.form == Form::Task
    }
}

/// Where an owner item is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Form {
    /// A line of `roadmap/owner-tasks.md` (D-10), the one form the tool ticks.
    Task,
    /// An item of a README's `## Risks …` or `## Open items …` list.
    Risk,
    /// An item of a README's `Open items for the owner` list.
    OpenItem,
    /// One action of a README's `**Owner actions outside any brief**` paragraph.
    OwnerAction,
    /// A row of `roadmap/owner-decisions.md`.
    OwnerDecision,
    /// A row of `roadmap/open-questions.md`.
    OpenQuestion,
    /// A `## Risks / notes for the next session` bullet of the latest handoff file.
    HandoffNote,
    /// A `> **Parked.**` block of the latest completed brief.
    Parked,
    /// A `> **Handed on.**` or `> **Handed on (to the owner).**` block of the latest completed
    /// brief.
    HandedOn,
}

impl Form {
    pub fn as_str(self) -> &'static str {
        match self {
            Form::Task => "owner task",
            Form::Risk => "risk",
            Form::OpenItem => "open item",
            Form::OwnerAction => "owner action",
            Form::OwnerDecision => "owner decision",
            Form::OpenQuestion => "open question",
            Form::HandoffNote => "handoff note",
            Form::Parked => "parked",
            Form::HandedOn => "handed on",
        }
    }
}

impl fmt::Display for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Open,
    /// A clause struck through, the rest still open.
    Partly,
    Resolved,
}

impl State {
    /// By strike-through, as the Risks lists write it: struck at the start → resolved, struck
    /// inside → partly resolved.
    pub(crate) fn struck(text: &str) -> State {
        if text.trim_start().starts_with("~~") {
            State::Resolved
        } else if text.matches("~~").nth(1).is_some() {
            State::Partly
        } else {
            State::Open
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            State::Open => "open",
            State::Partly => "partly resolved",
            State::Resolved => "resolved",
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// When an owner item falls due.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    /// `before EP-m`: before that EP starts (D-10, D-22's gate).
    Before(EpId),
    /// `On or after <date>`: not before that date.
    OnOrAfter(String),
    /// A decision point in words: `M4 acceptance`, `v2 gate`.
    When(String),
}

impl fmt::Display for Gate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Gate::Before(id) => write!(f, "before {id}"),
            Gate::OnOrAfter(date) => write!(f, "on or after {date}"),
            Gate::When(words) => f.write_str(words),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    fn ep(number: u32) -> EpId {
        EpId::new(number)
    }

    #[test]
    fn reads_owner_task_lines() {
        let text = "\
# Owner tasks

- [x] OT-1 (EP-0, 2026-10-01) Create the private remote.
- [ ] OT-2 (EP-1, 2026-10-02) Install the linker on the laptop [before EP-3]
- [ ] OT-3 (2026-10-03) Look at the backups after a week, before EP-12 starts, and before EP-14.
- [ ] OT-4 No parenthesis; beforehand EP-5 is not a gate, nor is xbefore EP-6.
";
        let (tasks, problems) = parse(text);
        assert!(problems.is_empty(), "{problems:?}");
        let summary: Vec<_> = tasks
            .iter()
            .map(|t| (t.id.as_str(), t.done, t.raised, t.date.as_deref(), t.gate))
            .collect();
        assert_eq!(
            summary,
            [
                ("OT-1", true, Some(ep(0)), Some("2026-10-01"), None),
                ("OT-2", false, Some(ep(1)), Some("2026-10-02"), Some(ep(3))),
                ("OT-3", false, None, Some("2026-10-03"), Some(ep(12))),
                ("OT-4", false, None, None, None),
            ]
        );
        let item = tasks[1].item(Path::new("owner-tasks.md"));
        assert_eq!(item.form, Form::Task);
        assert_eq!(item.gate, Some(Gate::Before(ep(3))));
        assert_eq!(item.state, State::Open);
        assert!(item.is_open() && item.is_tickable());
        assert!(!tasks[0].item(Path::new("x")).is_open());
        assert_eq!(Gate::Before(ep(3)).to_string(), "before EP-3");
        assert_eq!(
            Gate::OnOrAfter("2026-11-02".to_owned()).to_string(),
            "on or after 2026-11-02"
        );
    }

    #[test]
    fn finds_before_gates() {
        assert_eq!(before("Do it [before EP-6]"), Some(ep(6)));
        assert_eq!(before("(D-25, Before  EP-21)"), Some(ep(21)));
        assert_eq!(
            before("before EP-4a").map(|id| id.to_string()),
            Some("EP-4a".to_owned())
        );
        for none in [
            "",
            "EP-6 first",
            "after EP-6",
            "beforehand EP-6",
            "xbefore EP-6",
            "before STEP-6",
            "before EP-",
        ] {
            assert_eq!(before(none), None, "{none:?}");
        }
    }

    #[test]
    fn states_by_strike_through() {
        assert_eq!(
            State::struck("~~Gone.~~ **Resolved by EP-2.**"),
            State::Resolved
        );
        assert_eq!(State::struck("  ~~**R-3** Gone.~~"), State::Resolved);
        assert_eq!(
            State::struck("Traps: ~~one~~ **Resolved**; two open."),
            State::Partly
        );
        assert_eq!(State::struck("Open, with one ~~ stray mark."), State::Open);
        assert_eq!(State::struck("Open."), State::Open);
    }

    #[test]
    fn a_project_without_the_file_has_no_tasks() {
        let scratch = Scratch::new("tasks-read");
        assert_eq!(read(&scratch.0).unwrap(), None);
        fs::create_dir_all(scratch.0.join("roadmap")).unwrap();
        fs::write(path(&scratch.0), "- [ ] OT-1 (EP-2, 2026-10-03) One.\n").unwrap();
        let file = read(&scratch.0).unwrap().unwrap();
        assert_eq!(file.path, path(&scratch.0));
        assert_eq!(file.open().count(), 1);
        assert_eq!(file.items().next().unwrap().id.as_deref(), Some("OT-1"));
        let backups = scratch.0.join("backups");
        tick(&scratch.0, "OT-1", &backups, "2026-11-01").unwrap();
        assert_eq!(read(&scratch.0).unwrap().unwrap().open().count(), 0);
        untick(&scratch.0, "OT-1", &backups, "2026-11-01").unwrap();
        assert_eq!(
            fs::read_to_string(path(&scratch.0)).unwrap(),
            "- [ ] OT-1 (EP-2, 2026-10-03) One.\n"
        );
    }
}
