//! The formalise step (D-10 … D-13; EP-21): the house-style files a project is given when it is
//! formalised or adopted into the house style. They are the rule block in its `CLAUDE.md`, between
//! [`BEGIN`] and [`END`], `roadmap/owner-tasks.md` and `roadmap/inbox.md` with their headers, and
//! the brief template, `roadmap/_TEMPLATE.md`. [`plan`] says what each file would get and changes
//! nothing, for the owner's preview; [`apply`] writes the plan on the owner's yes, after reading
//! every file again. A file that is there is never replaced, and `CLAUDE.md` changes only between
//! the markers, after a copy of it to the backups folder (D-11). For a roadmap in an older form,
//! the five-column table or the work-packet style (D-9), [`migration_prompt`] is the prompt of the
//! session that migrates it to the house style.

use std::fs;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::files;
use crate::inbox::{self, mailman};
use crate::roadmap::readme;
use crate::roadmap::table::{self, Profile};
use crate::tasks;

/// The line that opens the tool's block in a `CLAUDE.md`.
pub const BEGIN: &str = "<!-- roadmap-wrangler:begin -->";
/// The line that closes it.
pub const END: &str = "<!-- roadmap-wrangler:end -->";

/// The brief template's name in `roadmap/`, which the brief discovery leaves out (EP-4).
pub const TEMPLATE_FILE: &str = "_TEMPLATE.md";

/// The `event` rows of a formalise write, under the project.
pub const EVENT_KIND: &str = "formalise";

/// The rule block's text between the markers (the owner's choice at EP-21's pickup: the house's
/// rules in short, so that a project formalised without the owner's user-level rules has them).
const RULES: &str = "\
## Roadmap house style

Written by Roadmap Wrangler (`crw formalise`; clauderoadmapwrangler D-10 … D-13) between these
markers, which it may rewrite: edit outside them. This file's own rules win where they differ.

- **Read first:** `roadmap/README.md` (the next unchecked EP in table order and its brief notes
  under the phase table), `DECISIONS.md`, the last executed brief's completion note,
  `roadmap/owner-tasks.md` and `roadmap/inbox.md`.
- **Inbox triage:** for each open message, `- [ ] IN-n (from <project> EP-k, date) text`, roll it
  into this session if it fits the brief and is at most S-sized; otherwise add it to the brief
  notes of a named later EP or propose a new row in the right phase, confirming with the owner
  through `AskUserQuestion`; or decline it with a reason. Write the disposition on its line and
  tick its box.
- **Brief first:** an EP session starts by writing `roadmap/EP-N-<slug>.md` from
  `roadmap/_TEMPLATE.md`: YAML front matter (`ep`, `title`, `size`, `model`, `effort`, `status`),
  then Context, In scope, Out of scope and Verification / acceptance.
- **Completion note:** the brief ends with
  `> **Completion note (YYYY-MM-DD; model/effort; duration).**`, recording each owner decision and
  its outcome. A message for another project follows as a `> **Handed on (to <project>).**` block,
  which the tool delivers into that project's `roadmap/inbox.md` once the brief is committed.
- **Owner tasks:** what only the owner can do is appended to `roadmap/owner-tasks.md` as
  `- [ ] OT-n (EP-k, date) text`, ending `[before EP-m]` only when EP-m builds on it.
- **The roadmap:** phase tables `| # | Brief | Size | Depends on | Core | Done |`, sizes S, M or L,
  a done row's cell `☑` and one backticked commit hash; each upcoming EP's detail is a
  `- **EP-n.** …` bullet under `**Brief notes.**`.
";

/// The header of an `owner-tasks.md` the formalise step creates (D-10); the example line stays in
/// backticks, since any other box is a problem to the parser (EP-5).
pub const OWNER_TASKS_HEADER: &str = "\
# Owner tasks — what only the owner can do

One line per task (clauderoadmapwrangler D-10): `- [ ] OT-n (EP-k, date) text`, ending
`[before EP-m]` only when EP-m builds on it. Sessions append at completion; the owner ticks a task
in Roadmap Wrangler or here.
";

/// The brief template (D-12): front matter, the four sections, the residual-risk line and the
/// completion note's placeholder.
pub const TEMPLATE: &str = "\
---
ep: N
title: The row's title
size: S
model: the model's id
effort: the effort level
status: in progress
---

# EP-N — The row's title

**Size:** S · **Depends on:** EP-k

## Context

What the row and its brief notes ask for, and what exists. **Session start (YYYY-MM-DD).** How the
session was launched and what it read first.

## In scope

1. …

## Out of scope

- …

## Verification / acceptance

- The tests and the checks the project's rules name.
- **Residual risk.** What the session could not exercise, and why it expects it to hold.

> **Completion note (YYYY-MM-DD; model/effort; duration).** What was built; each owner decision
> and its outcome; the routine choices; notes for later sessions.
";

/// The rule block with its markers, its lines ended by `eol` (none after the end marker).
pub fn block(eol: &str) -> String {
    format!("{BEGIN}\n{RULES}{END}").replace('\n', eol)
}

/// The house-style files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum File {
    /// The project's `CLAUDE.md`, which takes the rule block.
    Rules,
    OwnerTasks,
    Inbox,
    Template,
}

impl File {
    /// What the file is, in words.
    pub fn words(self) -> &'static str {
        match self {
            File::Rules => "the session rules",
            File::OwnerTasks => "the owner tasks",
            File::Inbox => "the inbox",
            File::Template => "the brief template",
        }
    }
}

/// What a file gets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// The file is new.
    Create,
    /// The rule block is added at the end of the file.
    AddBlock,
    /// The text between the markers is replaced by the rule block.
    ReplaceBlock,
    /// Nothing is written: the file is there, or its block reads as the tool would write it.
    Keep,
    /// The markers cannot be read, so nothing is written into the file.
    Refused(String),
}

/// One file of the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub file: File,
    pub path: PathBuf,
    /// Its path in the project, with forward slashes: `roadmap/inbox.md`.
    pub name: String,
    pub change: Change,
    /// The file as read: the rule file's text; `None` for a missing file, and for the other
    /// files, which are only ever created.
    pub before: Option<String>,
    /// The file's new text; `None` when the step writes nothing.
    pub after: Option<String>,
}

impl Step {
    /// Whether the step writes the file.
    pub fn writes(&self) -> bool {
        matches!(
            self.change,
            Change::Create | Change::AddBlock | Change::ReplaceBlock
        )
    }

    /// What the preview shows of the new text: the rule block for a change to `CLAUDE.md`, the
    /// whole text of a new file; nothing when the step writes nothing.
    pub fn shown(&self) -> Option<String> {
        match self.change {
            Change::AddBlock | Change::ReplaceBlock => Some(block("\n")),
            Change::Create => self.after.as_ref().map(|text| text.replace("\r\n", "\n")),
            Change::Keep | Change::Refused(_) => None,
        }
    }

    /// The step in words, for the preview: `created with its header`, `kept: it is there`.
    pub fn words(&self) -> String {
        let lines = match self
            .before
            .as_deref()
            .map_or(0, |text| text.lines().count())
        {
            1 => "1 line".to_owned(),
            n => format!("{n} lines"),
        };
        match (&self.change, self.file) {
            (Change::Create, File::Rules) => "created with a title and the rule block".to_owned(),
            (Change::Create, File::Template) => "created".to_owned(),
            (Change::Create, _) => "created with its header".to_owned(),
            (Change::AddBlock, _) => {
                format!("the rule block added at the end, the file's {lines} kept")
            }
            (Change::ReplaceBlock, _) => {
                "the rule block replaced between the markers, the text around it kept".to_owned()
            }
            (Change::Keep, File::Rules) => "kept: its rule block reads as written".to_owned(),
            (Change::Keep, _) => "kept: it is there".to_owned(),
            (Change::Refused(why), _) => format!("refused: {why}"),
        }
    }
}

/// What the roadmap README says of the project's form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Readme {
    Missing,
    Unreadable(String),
    /// Read: the form of its first phase table, none when the parser reads no phase table.
    Read(Option<Profile>),
}

/// What the formalise step would write in one project, for the owner's preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub project: String,
    pub folder: PathBuf,
    /// The four files, in the order they are written.
    pub steps: Vec<Step>,
    pub readme: Readme,
    /// What refuses the plan besides a step's refusal: the project's folder is not there.
    pub problems: Vec<String>,
}

impl Plan {
    /// Whether nothing may be written: a file could not be read, or a block's markers cannot be.
    pub fn is_refused(&self) -> bool {
        !self.problems.is_empty()
            || self
                .steps
                .iter()
                .any(|step| matches!(step.change, Change::Refused(_)))
    }

    /// How many files the plan writes.
    pub fn writes(&self) -> usize {
        self.steps.iter().filter(|step| step.writes()).count()
    }

    /// The form of an older roadmap, which a session migrates (D-9): the five-column table or
    /// the work-packet style; none for a house roadmap or none read.
    pub fn migrates(&self) -> Option<Profile> {
        match self.readme {
            Readme::Read(Some(profile @ (Profile::House5 | Profile::Packet))) => Some(profile),
            _ => None,
        }
    }

    /// The preview in words: one line per file, then the roadmap's form and what can stop it;
    /// `target` is how a command names the project (its registered name, else its folder).
    pub fn lines(&self, target: &str) -> Vec<String> {
        let mut lines = vec![format!(
            "formalise {} ({})",
            self.project,
            self.folder.display()
        )];
        let width = self
            .steps
            .iter()
            .map(|step| step.name.chars().count())
            .max()
            .unwrap_or(0)
            .max("roadmap/README.md".len());
        for step in &self.steps {
            lines.push(format!("  {:width$}  {}", step.name, step.words()));
        }
        let readme = match &self.readme {
            Readme::Missing => {
                "missing: a session writes the first roadmap (the plan skill)".to_owned()
            }
            Readme::Unreadable(why) => format!("could not be read: {why}"),
            Readme::Read(None) => "no phase table the parser reads".to_owned(),
            Readme::Read(Some(Profile::House)) => "house tables".to_owned(),
            Readme::Read(Some(profile)) => format!(
                "{}, read-only: `crw formalise {target} --migration` prints the prompt of a \
                 session that migrates it to the house style (D-9)",
                form_words(*profile),
            ),
        };
        if !self.steps.is_empty() {
            lines.push(format!("  {:width$}  {readme}", "roadmap/README.md"));
        }
        for problem in &self.problems {
            lines.push(format!("  refused: {problem}"));
        }
        lines
    }
}

/// A form of the phase tables, in words: `the five-column tables`.
pub fn form_words(profile: Profile) -> &'static str {
    match profile {
        Profile::House => "the house tables",
        Profile::House5 => "the five-column tables",
        Profile::Packet => "the work-packet tables",
    }
}

/// What the formalise step would write in the project in `folder` (named `project`), reading
/// files by path only and asking git nothing.
pub fn plan(folder: &Path, project: &str) -> Plan {
    if !folder.is_dir() {
        // A registered folder moved or renamed: nothing is made in its old place (EP-21's review).
        return Plan {
            project: project.to_owned(),
            folder: folder.to_owned(),
            steps: Vec::new(),
            readme: Readme::Missing,
            problems: vec![format!("{} is not a folder", folder.display())],
        };
    }
    let problems = Vec::new();
    let mut steps = vec![rules_step(folder, project)];
    let roadmap = folder.join("roadmap");
    for (file, path, text) in [
        (File::OwnerTasks, tasks::path(folder), OWNER_TASKS_HEADER),
        (File::Inbox, inbox::path(folder), mailman::HEADER),
        (File::Template, roadmap.join(TEMPLATE_FILE), TEMPLATE),
    ] {
        let name = format!(
            "roadmap/{}",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        let change = match fs::symlink_metadata(&path) {
            Ok(_) => Change::Keep,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Change::Create,
            Err(e) => Change::Refused(format!("it could not be looked at: {e}")),
        };
        let after = (change == Change::Create).then(|| text.to_owned());
        steps.push(Step {
            file,
            path,
            name,
            change,
            before: None,
            after,
        });
    }
    let readme = match fs::read_to_string(readme(folder)) {
        Ok(text) => Readme::Read(table::parse(&text).profile()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Readme::Missing,
        Err(e) => Readme::Unreadable(e.to_string()),
    };
    Plan {
        project: project.to_owned(),
        folder: folder.to_owned(),
        steps,
        readme,
        problems,
    }
}

/// The rule file's step: the root's `CLAUDE.md`, else `.claude\CLAUDE.md` (where the opener reads
/// them, `prompt::claude_md`), else a new root `CLAUDE.md`.
fn rules_step(folder: &Path, project: &str) -> Step {
    let root = folder.join("CLAUDE.md");
    let nested = folder.join(".claude").join("CLAUDE.md");
    let (path, name) = if !root.exists() && nested.exists() {
        (nested, ".claude/CLAUDE.md")
    } else {
        (root, "CLAUDE.md")
    };
    let refused = |path: PathBuf, why: String| Step {
        file: File::Rules,
        path,
        name: name.to_owned(),
        change: Change::Refused(why),
        before: None,
        after: None,
    };
    // A rename over a link would replace the link with a copy, and the file it points to would
    // never take the block (EP-21's review).
    if fs::symlink_metadata(&path).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return refused(
            path,
            format!("{name} is a link: add the block to the file it points to by hand"),
        );
    }
    let before = match fs::read_to_string(&path) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return refused(path, format!("it could not be read: {e}")),
    };
    let (change, after) = match &before {
        None => (
            Change::Create,
            Some(format!("# {project} — session rules\n\n{}\n", block("\n"))),
        ),
        Some(text) => {
            let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
            match markers(text) {
                Markers::None => (Change::AddBlock, Some(with_block_added(text, eol))),
                Markers::Block(range) if text[range.clone()] == block(eol) => (Change::Keep, None),
                Markers::Block(range) => {
                    let mut new = text[..range.start].to_owned();
                    new.push_str(&block(eol));
                    new.push_str(&text[range.end..]);
                    (Change::ReplaceBlock, Some(new))
                }
                Markers::Broken { begins, ends } => {
                    let numbers = |lines: &[usize]| match lines {
                        [] => "none".to_owned(),
                        lines => lines
                            .iter()
                            .map(usize::to_string)
                            .collect::<Vec<_>>()
                            .join(", "),
                    };
                    let why = format!(
                        "the roadmap-wrangler markers in {name} cannot be read (begin on lines \
                         {}, end on lines {}): leave one begin before one end, or none, by hand",
                        numbers(&begins),
                        numbers(&ends)
                    );
                    (Change::Refused(why), None)
                }
            }
        }
    };
    Step {
        file: File::Rules,
        path,
        name: name.to_owned(),
        change,
        before,
        after,
    }
}

/// Where a file's block is.
enum Markers {
    None,
    /// From the begin line's start to the end line's end, without its line break.
    Block(Range<usize>),
    /// The lines of the begin and end markers found, which make no single block.
    Broken {
        begins: Vec<usize>,
        ends: Vec<usize>,
    },
}

/// The block's place in `text`, by the lines that hold a marker alone outside a code fence (a
/// byte-order mark before the first line's marker ignored).
fn markers(text: &str) -> Markers {
    let (mut begins, mut ends) = (Vec::new(), Vec::new());
    let mut fence: Option<&str> = None;
    let mut at = 0;
    for (n, line) in text.split_inclusive('\n').enumerate() {
        let line_start = at;
        at += line.len();
        let content = line.trim_end_matches(['\r', '\n']);
        let unmarked = content.trim_start_matches(|c: char| c as u32 == 0xFEFF);
        let trimmed = unmarked.trim();
        if let Some(open) = fence {
            if trimmed.starts_with(open) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = ["```", "~~~"].into_iter().find(|f| trimmed.starts_with(f)) {
            fence = Some(open);
            continue;
        }
        match trimmed {
            // A byte-order mark stays where it is: the block starts after it.
            BEGIN => begins.push((line_start + content.len() - unmarked.len(), n + 1)),
            END => ends.push((line_start + content.len(), n + 1)),
            _ => {}
        }
    }
    match (begins.as_slice(), ends.as_slice()) {
        ([], []) => Markers::None,
        ([(start, _)], [(end, _)]) if start < end => Markers::Block(*start..*end),
        _ => Markers::Broken {
            begins: begins.iter().map(|&(_, n)| n).collect(),
            ends: ends.iter().map(|&(_, n)| n).collect(),
        },
    }
}

/// `text` without the tool's block, when it holds one that reads: where the opener looks first
/// for a project's own read-first rule, so that the block's never stands over it (EP-21's
/// review).
pub(crate) fn without_block(text: &str) -> Option<String> {
    match markers(text) {
        Markers::Block(range) => Some(format!("{}{}", &text[..range.start], &text[range.end..])),
        Markers::None | Markers::Broken { .. } => None,
    }
}

/// `text` with the rule block after it and a blank line between: every byte of `text` kept.
fn with_block_added(text: &str, eol: &str) -> String {
    let mut new = text.to_owned();
    if !new.is_empty() {
        let breaks = new[new.trim_end_matches(['\r', '\n']).len()..]
            .matches('\n')
            .count();
        for _ in breaks..2 {
            new.push_str(eol);
        }
    }
    new.push_str(&block(eol));
    new.push_str(eol);
    new
}

/// What can stop a write before anything is written.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("nothing is written: {0}")]
    Refused(String),
    #[error("{} changed since the preview, so nothing is written: preview again", .0.display())]
    Changed(PathBuf),
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
}

/// One file written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    pub file: File,
    pub path: PathBuf,
    pub name: String,
    pub change: Change,
    /// The copy of the file as it was, in the backups folder; none for a new file.
    pub copy: Option<PathBuf>,
}

/// What a write did: the files written, in order, and the failure that stopped it after the
/// first write, when one did (formalising again finishes the job).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    pub written: Vec<Written>,
    pub failed: Option<String>,
}

impl Applied {
    /// The write in words, for the ledger's line: `CLAUDE.md the rule block added; …`.
    pub fn detail(&self) -> String {
        let mut parts: Vec<String> = self
            .written
            .iter()
            .map(|written| {
                let what = match written.change {
                    Change::Create => "created",
                    Change::AddBlock => "the rule block added",
                    Change::ReplaceBlock => "the rule block replaced",
                    Change::Keep | Change::Refused(_) => "kept",
                };
                format!("{} {what}", written.name)
            })
            .collect();
        if let Some(failed) = &self.failed {
            parts.push(format!("stopped: {failed}"));
        }
        parts.join("; ")
    }
}

/// Writes `plan` (D-11): every file the plan writes is read again first, and nothing is written
/// when one changed since the plan; then the new files are created, never over a file, and the
/// rule file is changed after a copy of it to `<backups>\<project folder>\CLAUDE.<date>.md`. An
/// inbox is created under the process's inbox lock, as the mailman's deliveries are.
pub fn apply(plan: &Plan, backups: &Path, today: &str) -> Result<Applied, Error> {
    if plan.is_refused() {
        let why = plan
            .problems
            .iter()
            .cloned()
            .chain(plan.steps.iter().filter_map(|step| match &step.change {
                Change::Refused(why) => Some(why.clone()),
                _ => None,
            }))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(Error::Refused(why));
    }
    for step in plan.steps.iter().filter(|step| step.writes()) {
        let io = |source| Error::Io {
            path: step.path.clone(),
            source,
        };
        let changed = if step.change == Change::Create {
            // Anything there now, readable or not, appeared since the plan saw nothing.
            match fs::symlink_metadata(&step.path) {
                Ok(_) => true,
                Err(e) if e.kind() == io::ErrorKind::NotFound => false,
                Err(e) => return Err(io(e)),
            }
        } else {
            match fs::read_to_string(&step.path) {
                Ok(text) => step.before.as_deref() != Some(text.as_str()),
                Err(e) if e.kind() == io::ErrorKind::NotFound => true,
                Err(e) => return Err(io(e)),
            }
        };
        if changed {
            return Err(Error::Changed(step.path.clone()));
        }
    }
    let copies = backups.join(
        plan.folder
            .file_name()
            .map_or_else(|| "project".into(), ToOwned::to_owned),
    );
    let mut applied = Applied {
        written: Vec::new(),
        failed: None,
    };
    for step in plan.steps.iter().filter(|step| step.writes()) {
        match write(step, &copies, today) {
            Ok(Wrote::Appeared) => {}
            Ok(Wrote::File(copy)) => applied.written.push(Written {
                file: step.file,
                path: step.path.clone(),
                name: step.name.clone(),
                change: step.change.clone(),
                copy,
            }),
            Err(why) => {
                applied.failed = Some(why);
                break;
            }
        }
    }
    Ok(applied)
}

/// Logs a write in the ledger at `path` as an `event` row of kind [`EVENT_KIND`] under
/// `project`: the row a revert of a session that started before it redoes (EP-21a). `Err` says
/// why the ledger did not take it. `crw formalise --yes` and the window's Formalise… log alike.
pub fn log(path: &Path, project: &str, applied: &Applied) -> Result<(), String> {
    crate::ledger::Ledger::open(path)
        .and_then(|ledger| {
            ledger.add_event(
                chrono::Utc::now(),
                EVENT_KIND,
                Some(project),
                &applied.detail(),
            )
        })
        .map(|_| ())
        .map_err(|e| format!("the ledger did not take the write's line: {e}"))
}

/// What writing one step did.
enum Wrote {
    /// Written; the copy of the file as it was, for a change.
    File(Option<PathBuf>),
    /// A new file's place was taken in the moment since the check (the mailman's delivery, in
    /// another process): the file there is kept, as the plan keeps one, and the write goes on.
    Appeared,
}

fn write(step: &Step, copies: &Path, today: &str) -> Result<Wrote, String> {
    let at = |e: io::Error| format!("{}: {e}", step.path.display());
    let after = step.after.as_deref().unwrap_or_default();
    if step.change == Change::Create {
        if let Some(parent) = step.path.parent() {
            fs::create_dir_all(parent).map_err(at)?;
        }
        let _writing = (step.file == File::Inbox).then(inbox::writing);
        return match files::create_new(&step.path, after.as_bytes()) {
            Ok(()) => Ok(Wrote::File(None)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(Wrote::Appeared),
            Err(e) => Err(at(e)),
        };
    }
    let before = step.before.as_deref().unwrap_or_default();
    let copy = files::dated_copy(before.as_bytes(), copies, "CLAUDE", today, ".md")
        .map_err(|e| format!("{}: {e}", copies.display()))?;
    files::write_atomic(&step.path, after.as_bytes()).map_err(at)?;
    Ok(Wrote::File(Some(copy)))
}

/// The prompt of a session that migrates an older roadmap to the house style (D-9), for a
/// headless session the owner launches in the project; `crw` is the console exe the session
/// checks the roadmap with. `None` for a house roadmap.
pub fn migration_prompt(
    project: &str,
    folder: &Path,
    profile: Profile,
    crw: &Path,
    date: &str,
) -> Option<String> {
    let (form, columns, handoffs) = match profile {
        Profile::House => return None,
        Profile::House5 => (
            "the five-column header `| # | Brief | Size | Depends on | Done |`",
            "Add a Core column after Depends on, `core` for every row (or `stretch` where the \
             roadmap marks a row as optional).",
            false,
        ),
        Profile::Packet => (
            "the work-packet header `| # | Packet | Size | Depends on | Status |`",
            "The Packet column becomes Brief and Status becomes Done, and a Core column after \
             Depends on says `core` for every row (or `stretch` where the roadmap marks a row as \
             optional).",
            true,
        ),
    };
    let folder = folder.display();
    let crw = crw.display();
    let check = format!("`& \"{crw}\" roadmap \"{folder}\"`");
    let mut steps = vec![
        format!(
            "Read first: the project's `CLAUDE.md`, `roadmap/README.md`, the briefs in \
             `roadmap/`, {}and `DECISIONS.md` where it exists. Then run {check} and keep what it \
             prints: the next EP, the counts of done and total rows, and the problems.",
            if handoffs {
                "the handoff files in `docs/handoffs/` where they exist, "
            } else {
                ""
            }
        ),
        "Before any edit, put the plan to the owner through AskUserQuestion: the tables and rows \
         to convert, the completion notes to add, and anything that does not map; one question \
         per decision, the recommended option first."
            .to_owned(),
        format!(
            "Convert every phase table to the house header. Keep each row where it is, with its \
             id as written (letter suffixes too), its brief text and link, its size cell and its \
             depends-on cell. {columns} A done row's cell becomes `☑` and its commit hash in \
             backticks (`[x] 9bcb7b2` becomes ``☑ `9bcb7b2` ``; several hashes stay joined by \
             ` + `); an open row's becomes `☐`; a row in progress (`[~]`) becomes `☐` and is named \
             in this migration's record."
        ),
    ];
    if handoffs {
        steps.push(
            "Move each packet's handoff status into its brief: a brief whose packet is done (its \
             `## Handoff payload (filled <date>)` section, or a `docs/handoffs/EP-N.md` with \
             `**Status:** … · **Date:** …`) gets a closing `> **Completion note (<date>).**` that \
             says the status and the commit and names the handoff it comes from. The handoff \
             sections and files stay as they are."
                .to_owned(),
        );
    }
    steps.extend([
        "Keep every file: rename, move and delete nothing, and leave the old briefs' wording as \
         it is, since the tool reads the old forms forever. Each upcoming EP keeps its detail as \
         a `- **EP-n.** …` bullet under a `**Brief notes.**` label. The block between the \
         `roadmap-wrangler` markers in `CLAUDE.md`, `roadmap/owner-tasks.md`, `roadmap/inbox.md` \
         and `roadmap/_TEMPLATE.md` are the tool's formalise step's (`crw formalise`): leave them \
         as they are."
            .to_owned(),
        format!(
            "Check: {check} reads the house profile, names the same next EP and the same counts \
             as in step 1, and shows no new problem."
        ),
        format!(
            "Record the migration in `DECISIONS.md` in the log's own form (a new decision or a \
             dated addendum), and say in the roadmap README's opening paragraph that its tables \
             follow the house style from {date}. Stop before committing: the commit is the \
             owner's."
        ),
    ]);
    let numbered: Vec<String> = steps
        .iter()
        .enumerate()
        .map(|(n, step)| format!("{}. {step}", n + 1))
        .collect();
    Some(format!(
        "Migrate the roadmap of {project} to the house style, as the owner asked from Roadmap \
         Wrangler on {date}. The project's folder is {folder}, this session's working folder.\n\n\
         Its roadmap README's phase tables use {form}, which the tool reads read-only \
         (clauderoadmapwrangler D-9). The house form is `| # | Brief | Size | Depends on | Core | \
         Done |`: one row per brief, its Done cell `☐`, or `☑` and one backticked commit hash; \
         the next brief is the first unchecked row in table order.\n\n{}",
        numbered.join("\n")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checklist;
    use crate::files::tests::Scratch;
    use crate::prompt;

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn changes(plan: &Plan) -> Vec<(&str, Change)> {
        plan.steps
            .iter()
            .map(|step| (step.name.as_str(), step.change.clone()))
            .collect()
    }

    #[test]
    fn the_block_holds_the_read_first_rule_the_opener_quotes() {
        let block = block("\n");
        assert!(block.starts_with(&format!("{BEGIN}\n## Roadmap house style\n")));
        assert!(block.ends_with(&format!("`**Brief notes.**`.\n{END}")));
        let quoted = prompt::read_first(&format!("# alpha — session rules\n\n{block}\n")).unwrap();
        assert!(
            quoted.starts_with("**Read first:** `roadmap/README.md`"),
            "{quoted}"
        );
        assert!(quoted.ends_with("`roadmap/inbox.md`."), "{quoted}");
        // A project's own read-first rule above the block stays the one quoted.
        let own = format!("1. **Read first:** `notes.md`.\n\n{block}\n");
        assert_eq!(
            prompt::read_first(&own).as_deref(),
            Some("**Read first:** `notes.md`.")
        );
        for line in block.lines() {
            assert!(line.chars().count() <= 100, "{line}");
        }
    }

    #[test]
    fn the_headers_hold_no_line_of_work() {
        for (text, prefix) in [
            (OWNER_TASKS_HEADER, tasks::PREFIX),
            (mailman::HEADER, inbox::PREFIX),
        ] {
            let list = checklist::parse(text, prefix);
            assert!(list.items.is_empty(), "{text}");
            assert!(list.problems.is_empty(), "{:?}", list.problems);
            for line in text.lines() {
                assert!(line.chars().count() <= 100, "{line}");
            }
        }
        // A task appended below the header reads as the file's first.
        let (tasks, problems) = tasks::parse(&format!(
            "{OWNER_TASKS_HEADER}\n- [ ] OT-1 (EP-2, 2026-11-10) Look.\n"
        ));
        assert!(problems.is_empty());
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn a_bare_folder_gets_the_four_files() {
        let scratch = Scratch::new("formalise-bare");
        let folder = scratch.0.join("alpha");
        fs::create_dir_all(&folder).unwrap();
        let plan = plan(&folder, "alpha");
        assert_eq!(
            changes(&plan),
            [
                ("CLAUDE.md", Change::Create),
                ("roadmap/owner-tasks.md", Change::Create),
                ("roadmap/inbox.md", Change::Create),
                ("roadmap/_TEMPLATE.md", Change::Create),
            ]
        );
        assert_eq!(plan.readme, Readme::Missing);
        assert!(!plan.is_refused());
        assert_eq!(plan.writes(), 4);
        let rules = plan.steps[0].after.as_deref().unwrap();
        assert!(rules.starts_with("# alpha — session rules\n\n<!-- roadmap-wrangler:begin -->"));

        let applied = apply(&plan, &scratch.0.join("backups"), "2026-11-10").unwrap();
        assert_eq!(applied.failed, None);
        assert_eq!(applied.written.len(), 4);
        assert!(applied.written.iter().all(|written| written.copy.is_none()));
        assert_eq!(
            fs::read_to_string(folder.join("roadmap").join("inbox.md")).unwrap(),
            mailman::HEADER
        );
        assert!(!scratch.0.join("backups").exists());
        // Formalised: every file is kept the next time.
        let again = super::plan(&folder, "alpha");
        assert!(again.steps.iter().all(|step| step.change == Change::Keep));
        assert_eq!(again.writes(), 0);
        assert_eq!(
            applied.detail(),
            "CLAUDE.md created; roadmap/owner-tasks.md created; roadmap/inbox.md created; \
             roadmap/_TEMPLATE.md created"
        );
    }

    #[test]
    fn a_rule_file_without_markers_takes_the_block_at_its_end() {
        let scratch = Scratch::new("formalise-add");
        let folder = scratch.0.join("beta");
        let old = "# beta — session rules\n\n1. **Read first:** `roadmap/README.md`.\n";
        write(&folder.join("CLAUDE.md"), old);
        write(&folder.join("roadmap").join("inbox.md"), "# Inbox\n");
        let plan = plan(&folder, "beta");
        assert_eq!(plan.steps[0].change, Change::AddBlock);
        assert_eq!(plan.steps[2].change, Change::Keep);
        let new = plan.steps[0].after.clone().unwrap();
        assert_eq!(new, format!("{old}\n{}\n", block("\n")));
        assert_eq!(
            plan.steps[0].words(),
            "the rule block added at the end, the file's 3 lines kept"
        );

        let backups = scratch.0.join("backups");
        let applied = apply(&plan, &backups, "2026-11-10").unwrap();
        assert_eq!(fs::read_to_string(folder.join("CLAUDE.md")).unwrap(), new);
        let copy = applied.written[0].copy.clone().unwrap();
        assert_eq!(copy, backups.join("beta").join("CLAUDE.2026-11-10.md"));
        assert_eq!(fs::read_to_string(copy).unwrap(), old);
        // The inbox there is never replaced.
        assert_eq!(
            fs::read_to_string(folder.join("roadmap").join("inbox.md")).unwrap(),
            "# Inbox\n"
        );
        assert_eq!(super::plan(&folder, "beta").steps[0].change, Change::Keep);
    }

    #[test]
    fn a_file_without_a_final_line_break_and_crlf_endings_keep_their_bytes() {
        assert_eq!(
            with_block_added("# gamma", "\n"),
            format!("# gamma\n\n{}\n", block("\n"))
        );
        assert_eq!(
            with_block_added("# gamma\n\n\n", "\n"),
            format!("# gamma\n\n\n{}\n", block("\n"))
        );
        let crlf = with_block_added("# gamma\r\nrule\r\n", "\r\n");
        assert!(crlf.starts_with("# gamma\r\nrule\r\n\r\n<!-- roadmap-wrangler:begin -->\r\n"));
        assert!(!crlf.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn an_older_block_is_replaced_and_the_text_around_it_kept() {
        let scratch = Scratch::new("formalise-replace");
        let folder = scratch.0.join("delta");
        let before = "# delta\r\n\r\n1. Own rule.\r\n";
        let after = "\r\n2. Another own rule.\r\n";
        let old = format!("{before}{BEGIN}\r\nAn older block.\r\n{END}{after}");
        write(&folder.join("CLAUDE.md"), &old);
        let plan = plan(&folder, "delta");
        assert_eq!(plan.steps[0].change, Change::ReplaceBlock);
        assert_eq!(
            plan.steps[0].after.as_deref().unwrap(),
            format!("{before}{}{after}", block("\r\n"))
        );
        apply(&plan, &scratch.0.join("backups"), "2026-11-10").unwrap();
        assert_eq!(super::plan(&folder, "delta").steps[0].change, Change::Keep);
    }

    #[test]
    fn markers_that_cannot_be_read_refuse_the_plan() {
        let scratch = Scratch::new("formalise-broken");
        let folder = scratch.0.join("epsilon");
        for text in [
            format!("# epsilon\n{BEGIN}\nno end\n"),
            format!("# epsilon\n{END}\n{BEGIN}\n"),
            format!("{BEGIN}\n{END}\n{BEGIN}\n{END}\n"),
        ] {
            write(&folder.join("CLAUDE.md"), &text);
            let plan = plan(&folder, "epsilon");
            assert!(plan.is_refused(), "{text}");
            assert!(
                matches!(&plan.steps[0].change, Change::Refused(why) if why.contains("cannot be read"))
            );
            assert!(matches!(
                apply(&plan, &scratch.0.join("backups"), "2026-11-10"),
                Err(Error::Refused(_))
            ));
            assert_eq!(fs::read_to_string(folder.join("CLAUDE.md")).unwrap(), text);
            assert!(!folder.join("roadmap").exists());
        }
    }

    #[test]
    fn the_nested_rule_file_takes_the_block_when_the_root_has_none() {
        let scratch = Scratch::new("formalise-nested");
        let folder = scratch.0.join("zeta");
        write(&folder.join(".claude").join("CLAUDE.md"), "# zeta\n");
        let plan = plan(&folder, "zeta");
        assert_eq!(plan.steps[0].name, ".claude/CLAUDE.md");
        assert_eq!(plan.steps[0].change, Change::AddBlock);
    }

    #[test]
    fn a_file_changed_since_the_plan_stops_the_write_before_anything_is_written() {
        let scratch = Scratch::new("formalise-changed");
        let folder = scratch.0.join("eta");
        write(&folder.join("CLAUDE.md"), "# eta\n");
        let plan = plan(&folder, "eta");
        write(&folder.join("CLAUDE.md"), "# eta, edited\n");
        assert!(matches!(
            apply(&plan, &scratch.0.join("backups"), "2026-11-10"),
            Err(Error::Changed(path)) if path == folder.join("CLAUDE.md")
        ));
        assert!(!folder.join("roadmap").exists());

        // A file that appeared where the plan creates one, the last step, stops it too: the
        // rule file and the files before it are left as they were (EP-21's review).
        let plan = super::plan(&folder, "eta");
        let template = folder.join("roadmap").join(TEMPLATE_FILE);
        write(&template, "mine\n");
        assert!(matches!(
            apply(&plan, &scratch.0.join("backups"), "2026-11-10"),
            Err(Error::Changed(path)) if path == template
        ));
        assert_eq!(fs::read_to_string(&template).unwrap(), "mine\n");
        assert_eq!(
            fs::read_to_string(folder.join("CLAUDE.md")).unwrap(),
            "# eta, edited\n"
        );
        assert!(!folder.join("roadmap").join("owner-tasks.md").exists());
        assert!(!folder.join("roadmap").join("inbox.md").exists());
        assert!(!scratch.0.join("backups").exists());
    }

    #[test]
    fn a_folder_that_is_not_there_is_refused_and_never_made() {
        let scratch = Scratch::new("formalise-missing");
        let folder = scratch.0.join("moved");
        let plan = plan(&folder, "moved");
        assert!(plan.is_refused());
        assert!(plan.steps.is_empty());
        assert_eq!(
            plan.lines("moved"),
            [
                format!("formalise moved ({})", folder.display()),
                format!("  refused: {} is not a folder", folder.display()),
            ]
        );
        assert!(matches!(
            apply(&plan, &scratch.0.join("backups"), "2026-11-10"),
            Err(Error::Refused(_))
        ));
        assert!(!folder.exists());
    }

    #[test]
    fn markers_in_a_code_fence_are_text_and_a_byte_order_mark_stays() {
        let fenced = format!("# iota\n\n```\n{BEGIN}\n{END}\n```\n");
        assert!(matches!(markers(&fenced), Markers::None));
        assert_eq!(without_block(&fenced), None);
        let bom = char::from_u32(0xFEFF).unwrap();
        let marked = format!("{bom}{BEGIN}\nold\n{END}\n# iota\n");
        let Markers::Block(range) = markers(&marked) else {
            panic!("the block after the mark is read");
        };
        assert_eq!(&marked[..range.start], bom.to_string());
        assert_eq!(
            without_block(&marked).as_deref(),
            Some(format!("{bom}\n# iota\n").as_str())
        );
    }

    #[test]
    fn a_projects_own_read_order_section_stands_over_the_block() {
        let own = format!(
            "# kappa\n\n## Read order for every session\n\n1. `roadmap/charter.md`\n\n{}\n",
            block("\n")
        );
        assert_eq!(
            prompt::read_first(&own).as_deref(),
            Some("1. `roadmap/charter.md`")
        );
        // Without a rule of its own, the block's is quoted.
        let bare = format!("# kappa\n\n{}\n", block("\n"));
        assert!(
            prompt::read_first(&bare)
                .unwrap()
                .starts_with("**Read first:**")
        );
    }

    #[cfg(windows)]
    #[test]
    fn a_linked_rule_file_is_refused() {
        let scratch = Scratch::new("formalise-link");
        let folder = scratch.0.join("lambda");
        write(&folder.join("AGENTS.md"), "# lambda\n");
        let linked =
            std::os::windows::fs::symlink_file(folder.join("AGENTS.md"), folder.join("CLAUDE.md"));
        if linked.is_err() {
            // No right to make links in this shell (a non-admin one without Developer Mode).
            return;
        }
        let plan = plan(&folder, "lambda");
        assert!(matches!(&plan.steps[0].change, Change::Refused(why) if why.contains("is a link")));
        assert!(plan.is_refused());
    }

    #[test]
    fn the_readme_form_names_the_migration() {
        let scratch = Scratch::new("formalise-readme");
        let folder = scratch.0.join("theta");
        let readme_path = folder.join("roadmap").join("README.md");
        for (header, readme) in [
            (
                "| # | Brief | Size | Depends on | Core | Done |\n|---|---|---|---|---|---|\n| EP-1 | One | S | — | core | ☐ |",
                Readme::Read(Some(Profile::House)),
            ),
            (
                "| # | Brief | Size | Depends on | Done |\n|---|---|---|---|---|\n| EP-1 | One | S | — | ☐ |",
                Readme::Read(Some(Profile::House5)),
            ),
            (
                "| # | Packet | Size | Depends on | Status |\n|---|---|---|---|---|\n| EP-1 | One | S | — | [ ] |",
                Readme::Read(Some(Profile::Packet)),
            ),
            ("No table.", Readme::Read(None)),
        ] {
            write(
                &readme_path,
                &format!("# Roadmap\n\n## Phase 1 — Start\n\n{header}\n"),
            );
            let plan = plan(&folder, "theta");
            assert_eq!(plan.readme, readme, "{header}");
            let migrates = matches!(
                readme,
                Readme::Read(Some(Profile::House5 | Profile::Packet))
            );
            assert_eq!(plan.migrates().is_some(), migrates);
            let last = plan.lines("theta").pop().unwrap();
            assert_eq!(last.contains("--migration"), migrates, "{last}");
        }
    }

    #[test]
    fn the_migration_prompt_follows_the_form() {
        let crw = Path::new(r"C:\Tools\crw.exe");
        let folder = Path::new(r"C:\Data\iota");
        assert_eq!(
            migration_prompt("iota", folder, Profile::House, crw, "2026-11-10"),
            None
        );
        let packet = migration_prompt("iota", folder, Profile::Packet, crw, "2026-11-10").unwrap();
        assert!(packet.starts_with(
            "Migrate the roadmap of iota to the house style, as the owner asked from Roadmap \
             Wrangler on 2026-11-10. The project's folder is C:\\Data\\iota"
        ));
        for words in [
            "`| # | Packet | Size | Depends on | Status |`",
            "The Packet column becomes Brief",
            "docs/handoffs/",
            "## Handoff payload (filled <date>)",
            r#"`& "C:\Tools\crw.exe" roadmap "C:\Data\iota"`"#,
            "Stop before committing",
            "7. Record the migration",
        ] {
            assert!(packet.contains(words), "{words}");
        }
        let five = migration_prompt("iota", folder, Profile::House5, crw, "2026-11-10").unwrap();
        assert!(five.contains("`| # | Brief | Size | Depends on | Done |`"));
        assert!(five.contains("Add a Core column after Depends on"));
        assert!(!five.contains("docs/handoffs/"));
        assert!(five.contains("6. Record the migration"));
    }
}
