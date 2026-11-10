//! The launch prompt (D-17): the house-style opener a session of a project's next brief starts
//! from, generated from the project's snapshot: where the project is and what the session runs on
//! (a visual launch leaves the model and effort to VS Code's panel and says so, EP-11); the
//! read-first rule of the project's own
//! `CLAUDE.md`, else the house default; the next row with its brief notes verbatim; its brief when
//! one is written, and the last completed one; the open owner tasks and inbox messages as written,
//! with the triage rule (D-13); and the rules every session keeps, ending before the commit, or
//! with the commit itself while the project's autopilot is on (D-22; EP-19), whose words the
//! Commit button also sends ([`commit_message`]). A project may word it its own way with a template
//! in `[project.launch] template`, whose placeholders are the opener's parts. [`draft`] puts the
//! prompt together with the rule table's resolution, for the launch dialog and `crw prompt`.

use std::fs;
use std::path::Path;

use crate::registry::{LaunchMode, Project, Registry};
use crate::roadmap::EpId;
use crate::roadmap::markdown::{self, Block};
use crate::roadmap::snapshot::Snapshot;
use crate::roadmap::table::{Roadmap, Row};
use crate::roadmap::{Size, readme};
use crate::rules::{self, Resolution};

/// The house's read-first rule, for a project whose `CLAUDE.md` gives none (the house style,
/// rule 1).
pub const HOUSE_READ_FIRST: &str = "**Read first:** the project's `CLAUDE.md`; `roadmap/README.md` \
(the next unchecked EP in table order and its brief notes under the phase table); `DECISIONS.md`; \
the last executed brief's completion note; `roadmap/owner-tasks.md` and `roadmap/inbox.md` where \
they exist.";

/// The triage rule (D-13), said when a message is open.
const TRIAGE: &str = "Triage each open message right after reading the roadmap (D-13): roll it \
into this session if it fits the brief and is at most S-sized; otherwise add it to the brief notes \
of a named later EP or propose a new row in the right phase, confirming with the owner through \
AskUserQuestion; or decline it with a reason. Write the disposition on the message's line and tick \
its box.";

/// What the session runs with, as the launch asks for it.
#[derive(Debug, Clone, Copy)]
pub struct Chosen<'a> {
    /// An alias or an id, as passed to the CLI.
    pub model: &'a str,
    /// `None` leaves the effort to the CLI.
    pub effort: Option<&'a str>,
    pub ultracode: bool,
    /// The launch's date, `YYYY-MM-DD`.
    pub date: &'a str,
    /// A visual launch, whose model and effort VS Code's panel chooses: the ones above are then
    /// the launch's suggestion (D-16).
    pub visual: bool,
}

/// The pieces an opener is made of; each is also a template's placeholder (`{ep}`), and an empty
/// one is left out of the opener.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Parts {
    pub project: String,
    pub folder: String,
    pub date: String,
    pub model: String,
    /// The effort in words: a level, `ultracode`, or the CLI's default.
    pub effort: String,
    pub ep: String,
    pub title: String,
    /// The Size cell as written, which keeps a re-scope.
    pub size: String,
    pub depends: String,
    /// `core` or `stretch`; empty without a Core column.
    pub track: String,
    /// The phase heading's id and name: `P2 — Sessions`.
    pub phase: String,
    /// The brief-notes bullet verbatim.
    pub note: String,
    pub read_first: String,
    /// Whether `read_first` is quoted from the project's `CLAUDE.md` (else the house default).
    pub read_first_quoted: bool,
    /// Whether the launch is visual, so that `model` and `effort` are a suggestion.
    pub visual: bool,
    /// What to do with the next brief's file when one is written.
    pub brief: String,
    pub last_brief: String,
    /// The open owner tasks, one list item each; `none` without any.
    pub tasks: String,
    /// The open tasks gated before this brief, which the owner does first.
    pub due: String,
    pub inbox: String,
    pub triage: String,
    pub rules: String,
    /// Whether the rules tell the session to commit: the project's autopilot is on (D-22).
    pub commits: bool,
}

impl Parts {
    /// The placeholders a template may name, `{opener}` aside.
    pub const NAMES: [&'static str; 21] = [
        "project",
        "folder",
        "date",
        "model",
        "effort",
        "ep",
        "title",
        "size",
        "depends",
        "track",
        "phase",
        "note",
        "read_first",
        "brief",
        "last_brief",
        "tasks",
        "due",
        "inbox",
        "triage",
        "rules",
        "opener",
    ];

    fn get(&self, name: &str) -> Option<&str> {
        Some(match name {
            "project" => &self.project,
            "folder" => &self.folder,
            "date" => &self.date,
            "model" => &self.model,
            "effort" => &self.effort,
            "ep" => &self.ep,
            "title" => &self.title,
            "size" => &self.size,
            "depends" => &self.depends,
            "track" => &self.track,
            "phase" => &self.phase,
            "note" => &self.note,
            "read_first" => &self.read_first,
            "brief" => &self.brief,
            "last_brief" => &self.last_brief,
            "tasks" => &self.tasks,
            "due" => &self.due,
            "inbox" => &self.inbox,
            "triage" => &self.triage,
            "rules" => &self.rules,
            _ => return None,
        })
    }

    /// Sets what the session runs with, as the dialog changes it.
    pub fn choose(&mut self, chosen: &Chosen<'_>) {
        self.model = chosen.model.trim().to_owned();
        self.effort = effort_words(chosen.effort, chosen.ultracode);
        self.date = chosen.date.to_owned();
        self.visual = chosen.visual;
    }
}

/// A prompt and what its template named that the tool does not fill.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Prompt {
    pub text: String,
    pub problems: Vec<String>,
}

/// The project's `CLAUDE.md`, or `.claude\CLAUDE.md`, when it has one.
pub fn claude_md(folder: &Path) -> Option<String> {
    [
        folder.join("CLAUDE.md"),
        folder.join(".claude").join("CLAUDE.md"),
    ]
    .iter()
    .find_map(|path| fs::read_to_string(path).ok())
}

/// The read-first rule of a project's `CLAUDE.md`, as written: the list item or paragraph that
/// opens with **Read first**, without its list marker, or the body of a section headed read
/// order, load order or read first. `None` when it has neither.
pub fn read_first(claude_md: &str) -> Option<String> {
    let lines = markdown::lines(claude_md);
    let blocks = markdown::blocks(&lines);
    for block in &blocks {
        let (block_lines, indent) = match block {
            Block::Item { lines, indent } => (lines, *indent),
            Block::Paragraph { lines } => (lines, 0),
            _ => continue,
        };
        let Some((first, rest)) = block_lines.split_first() else {
            continue;
        };
        // An item's content starts `indent` bytes in: its marker and the spaces are ASCII.
        let opening = first.text.get(indent..).unwrap_or(first.text);
        if opens_read_first(opening) {
            let mut text = vec![opening.trim_end()];
            text.extend(
                rest.iter()
                    .map(|line| markdown::dedent(line.text, indent).trim_end()),
            );
            return Some(text.join("\n").trim().to_owned());
        }
    }
    let headings: Vec<(usize, usize)> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::Heading { level, line, .. } => Some((*level, *line)),
            _ => None,
        })
        .collect();
    blocks.iter().find_map(|block| {
        let Block::Heading { level, text, line } = block else {
            return None;
        };
        if !names_read_order(text) {
            return None;
        }
        let end = headings
            .iter()
            .find(|(other, at)| at > line && other <= level)
            .map_or(usize::MAX, |&(_, at)| at);
        let body: Vec<&str> = lines
            .iter()
            .filter(|l| l.number > *line && l.number < end)
            .map(|l| l.text.trim_end())
            .collect();
        let body = body.join("\n").trim_matches('\n').to_owned();
        (!body.trim().is_empty()).then_some(body)
    })
}

fn opens_read_first(text: &str) -> bool {
    text.trim_start()
        .trim_start_matches(['*', '_'])
        .trim_start()
        .to_lowercase()
        .starts_with("read first")
}

fn names_read_order(heading: &str) -> bool {
    let heading = heading.to_lowercase();
    ["read first", "read order", "reading order", "load order"]
        .iter()
        .any(|words| heading.contains(words))
}

/// The roadmap a launch reads and its next row, or why there is none.
pub fn next_of(snapshot: &Snapshot) -> Result<(&Roadmap, &Row), String> {
    let roadmap = snapshot
        .current()
        .ok_or_else(|| format!("there is no {}", readme(Path::new("")).display()))?;
    match roadmap.next() {
        Some(row) => Ok((roadmap, row)),
        None if roadmap.counts().all_checked() => {
            Err("every row of its roadmap is checked".to_owned())
        }
        None => Err("its roadmap has no phase table the parser reads".to_owned()),
    }
}

/// The opener's parts for `snapshot`'s next brief; why there is none, when there is none. With
/// `commits` (the project's autopilot is on, D-22) the last rule tells the session to commit.
pub fn parts(
    project: &str,
    snapshot: &Snapshot,
    claude_md: Option<&str>,
    chosen: &Chosen<'_>,
    commits: bool,
) -> Result<Parts, String> {
    let (roadmap, row) = next_of(snapshot)?;
    let mut parts = Parts {
        project: project.to_owned(),
        folder: snapshot.folder.display().to_string(),
        ep: row.id.to_string(),
        title: row.brief.title.trim().to_owned(),
        size: match row.size_text.trim() {
            "" => "no size".to_owned(),
            size => size.to_owned(),
        },
        depends: match row.depends.text.trim() {
            "" => "—".to_owned(),
            depends => depends.to_owned(),
        },
        track: row
            .track
            .map(|track| track.as_str().to_owned())
            .unwrap_or_default(),
        phase: roadmap
            .phase_of(row.id)
            .map(|phase| match &phase.heading.id {
                Some(id) => format!("{id} — {}", phase.heading.name),
                None => phase.heading.name.clone(),
            })
            .unwrap_or_default(),
        note: roadmap
            .note(row.id)
            .map(|note| note.text.trim().to_owned())
            .unwrap_or_else(|| format!("(The roadmap has no brief notes for {}.)", row.id)),
        ..Parts::default()
    };
    parts.choose(chosen);
    match claude_md.and_then(read_first) {
        Some(rule) => {
            parts.read_first = rule;
            parts.read_first_quoted = true;
        }
        None => parts.read_first = HOUSE_READ_FIRST.to_owned(),
    }
    let written = snapshot.entry(row.id);
    parts.brief = match written {
        Some(entry) if entry.is_complete() => format!(
            "Its brief, `roadmap/{}`, already has a completion note: the row may only need its \
             commit recorded. Check with the owner before redoing anything.",
            entry.file.name
        ),
        Some(entry) => format!(
            "Its brief is written already: `roadmap/{}`. Pick it up and work from it; do not \
             write a new one.",
            entry.file.name
        ),
        None => String::new(),
    };
    parts.last_brief = snapshot
        .latest_complete()
        .filter(|entry| entry.file.id != row.id)
        .map(|entry| match entry.completed_on() {
            Some(date) => format!(
                "The last completed brief is `roadmap/{}` ({date}): read its completion note.",
                entry.file.name
            ),
            None => format!(
                "The last completed brief is `roadmap/{}`: read its completion note.",
                entry.file.name
            ),
        })
        .unwrap_or_default();

    let due = due(snapshot, roadmap, row);
    let tasks: Vec<String> = snapshot
        .tasks
        .iter()
        .flat_map(|tasks| tasks.open())
        .map(|task| {
            let meta: Vec<String> = task
                .raised
                .map(|ep| ep.to_string())
                .into_iter()
                .chain(task.date.clone())
                .collect();
            item(&task.id, &meta.join(", "), &task.text)
        })
        .collect();
    parts.tasks = list_or_none(&tasks);
    if !due.is_empty() {
        parts.due = format!(
            "Due before this brief and still open: {}. Ask the owner whether it is done before \
             the work starts.",
            due.join(", ")
        );
    }
    let messages: Vec<String> = snapshot
        .inbox
        .iter()
        .flat_map(|inbox| inbox.open())
        .map(|message| {
            let meta: Vec<String> = message
                .from
                .as_ref()
                .map(|from| format!("from {from}"))
                .into_iter()
                .chain(message.date.clone())
                .collect();
            item(&message.id, &meta.join(", "), &message.text)
        })
        .collect();
    parts.inbox = list_or_none(&messages);
    if !messages.is_empty() {
        parts.triage = TRIAGE.to_owned();
    }
    parts.rules = session_rules(
        &parts.ep,
        written.map(|entry| entry.file.name.as_str()),
        commits,
    );
    parts.commits = commits;
    Ok(parts)
}

/// The open tasks gated before `row` or before a row above it, which are due first (D-10).
pub fn due(snapshot: &Snapshot, roadmap: &Roadmap, row: &Row) -> Vec<String> {
    let order: Vec<EpId> = roadmap.rows().map(|row| row.id).collect();
    let Some(next) = order.iter().position(|id| *id == row.id) else {
        return Vec::new();
    };
    snapshot
        .tasks
        .iter()
        .flat_map(|tasks| tasks.open())
        .filter(|task| {
            task.gate
                .and_then(|gate| order.iter().position(|id| *id == gate))
                .is_some_and(|gate| gate <= next)
        })
        .map(|task| task.id.clone())
        .collect()
}

/// A list item `- OT-3 (EP-7, 2026-11-01) text`, its continuation lines indented under it.
fn item(id: &str, meta: &str, text: &str) -> String {
    let text = text.trim().replace('\n', "\n  ");
    if meta.is_empty() {
        format!("- {id} {text}")
    } else {
        format!("- {id} ({meta}) {text}")
    }
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join("\n")
    }
}

pub fn effort_words(effort: Option<&str>, ultracode: bool) -> String {
    match effort {
        _ if ultracode => "ultracode".to_owned(),
        Some(effort) if !effort.trim().is_empty() => effort.trim().to_owned(),
        _ => "the CLI's default".to_owned(),
    }
}

/// The house's commit, said for `ep`: the pattern of its two commits and the hash recorded,
/// never a push.
fn commit_words(ep: &str) -> String {
    format!(
        "commit in the house pattern, `<Title>: <summary> ({ep}, D-x)` then `docs(roadmap): \
         record {ep} commit hash`, and record the hash in the roadmap's row, as the project's \
         `CLAUDE.md` says. Never push: the push stays the owner's."
    )
}

/// What the Commit button resumes a finished session with (D-22): the owner's yes to its commit.
pub fn commit_message(ep: &str) -> String {
    format!("The owner says yes to the commit: {}", commit_words(ep))
}

/// The rules every session keeps (D-12, D-17), ending before the commit; with `commits` (the
/// project's autopilot is on, the owner's standing yes, D-22) the last one commits instead.
fn session_rules(ep: &str, written: Option<&str>, commits: bool) -> String {
    let first = match written {
        Some(name) => format!(
            "1. Work from its brief, `roadmap/{name}`; keep its front matter's `model`, `effort` \
             and `status` true to this session."
        ),
        None => format!(
            "1. Write `roadmap/{ep}-<slug>.md` before touching any other file: YAML front matter \
             (`ep`, `title`, `size`, `model`, `effort`, `status`), then Context, In scope, Out of \
             scope and Verification / acceptance."
        ),
    };
    [
        first,
        "2. Follow the project's `CLAUDE.md`.".to_owned(),
        "3. Put every owner decision to the owner through AskUserQuestion: one question per \
         decision, the recommended option first, each question standing alone."
            .to_owned(),
        format!(
            "4. End the brief with `> **Completion note (date; model/effort; duration).**`, \
             recording each owner decision and its outcome; append what only the owner can do to \
             `roadmap/owner-tasks.md` as `- [ ] OT-n ({ep}, date) text`, and write a `> **Handed \
             on (to <project>).**` block for each message to another project."
        ),
        if commits {
            format!(
                "5. The autopilot is on for this project, the owner's standing yes to its \
                 commits (D-22): {}",
                commit_words(ep)
            )
        } else {
            "5. Stop before committing: the commit is the owner's call.".to_owned()
        },
    ]
    .join("\n")
}

/// The opener in the house's words, its empty parts left out.
pub fn opener(parts: &Parts) -> String {
    let mut out: Vec<String> = Vec::new();
    out.push(format!(
        "Run the next brief of {}: {}, {} ({}).",
        parts.project, parts.ep, parts.title, parts.size
    ));
    out.push(if parts.visual {
        format!(
            "The project's folder is {}, this session's working folder. The session was launched \
             from clauderoadmapwrangler on {} into VS Code, whose Claude Code panel chooses its \
             model and effort (the launch suggested {} at {} effort): write the ones it runs \
             with into the brief's front matter and its completion note.",
            parts.folder, parts.date, parts.model, parts.effort
        )
    } else {
        format!(
            "The project's folder is {}, this session's working folder. The session was launched \
             from clauderoadmapwrangler on {} with {} at {} effort: write those into the brief's \
             front matter and its completion note.",
            parts.folder, parts.date, parts.model, parts.effort
        )
    });
    out.push(if parts.read_first_quoted {
        "Read first, as the project's CLAUDE.md says:".to_owned()
    } else {
        "Read first (the house default: no CLAUDE.md of the project gives a read-first rule):"
            .to_owned()
    });
    out.push(parts.read_first.clone());
    out.push(
        "The next brief is the first unchecked row of roadmap/README.md in table order:".to_owned(),
    );
    let mut row = vec![
        parts.ep.clone(),
        parts.size.clone(),
        format!("depends on {}", parts.depends),
    ];
    if !parts.track.is_empty() {
        row.push(parts.track.clone());
    }
    if !parts.phase.is_empty() {
        row.push(format!("phase {}", parts.phase));
    }
    out.push(format!("- {}\n- {}", row.join(" · "), parts.title));
    out.push("Its brief notes, verbatim:".to_owned());
    out.push(parts.note.clone());
    for paragraph in [&parts.brief, &parts.last_brief] {
        if !paragraph.is_empty() {
            out.push(paragraph.clone());
        }
    }
    out.push(labelled(
        "Open owner tasks in roadmap/owner-tasks.md",
        &parts.tasks,
    ));
    if !parts.due.is_empty() {
        out.push(parts.due.clone());
    }
    out.push(labelled(
        "Open inbox messages in roadmap/inbox.md",
        &parts.inbox,
    ));
    if !parts.triage.is_empty() {
        out.push(parts.triage.clone());
    }
    out.push("Rules for this session:".to_owned());
    out.push(parts.rules.clone());
    out.join("\n\n")
}

/// `label: none`, or the label and its list below it.
fn labelled(label: &str, list: &str) -> String {
    if list == "none" {
        format!("{label}: none.")
    } else {
        format!("{label}:\n\n{list}")
    }
}

/// A template with its placeholders filled; `{{` and `}}` stand for braces. A name the tool does
/// not know is kept as written and returned.
pub fn render(template: &str, parts: &Parts) -> (String, Vec<String>) {
    let mut out = String::with_capacity(template.len() * 2);
    let mut unknown: Vec<String> = Vec::new();
    let mut opener_text: Option<String> = None;
    let mut rest = template;
    while let Some(at) = rest.find(['{', '}']) {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        if let Some(after) = tail.strip_prefix("{{") {
            out.push('{');
            rest = after;
            continue;
        }
        if let Some(after) = tail.strip_prefix("}}") {
            out.push('}');
            rest = after;
            continue;
        }
        if tail.starts_with('{')
            && let Some(close) = tail.find('}')
        {
            let name = &tail[1..close];
            if !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
                match name {
                    "opener" => out.push_str(opener_text.get_or_insert_with(|| opener(parts))),
                    _ => match parts.get(name) {
                        Some(value) => out.push_str(value),
                        None => {
                            out.push_str(&tail[..=close]);
                            if !unknown.iter().any(|known| known == name) {
                                unknown.push(name.to_owned());
                            }
                        }
                    },
                }
                rest = &tail[close + 1..];
                continue;
            }
        }
        // A brace on its own stays as written.
        out.push_str(&tail[..1]);
        rest = &tail[1..];
    }
    out.push_str(rest);
    (out, unknown)
}

/// The prompt for a launch: the project's template filled when it has one, else the opener.
pub fn generate(parts: &Parts, template: Option<&str>) -> Prompt {
    match template {
        Some(template) => {
            let (text, unknown) = render(template, parts);
            Prompt {
                text,
                problems: unknown
                    .iter()
                    .map(|name| {
                        format!(
                            "the template names {{{name}}}, which the tool does not fill; it is \
                             kept as written"
                        )
                    })
                    .collect(),
            }
        }
        None => Prompt {
            text: opener(parts),
            problems: Vec::new(),
        },
    }
}

/// Everything a launch of a project's next brief starts from.
#[derive(Debug, Clone)]
pub struct Draft {
    pub ep: EpId,
    pub title: String,
    pub size: Option<Size>,
    /// The id of the phase heading over the brief's row, when it has one (EP-19c): what the
    /// rules match by phase, and what the launch dialog's Keep for phase Pn keeps for.
    pub phase: Option<String>,
    pub resolution: Resolution,
    pub parts: Parts,
    pub prompt: Prompt,
}

impl Draft {
    /// The draft as a chained run takes it (D-22; EP-19a): headless whatever the project's
    /// launch mode, so the opener says what the session runs with rather than that VS Code's
    /// panel chooses; and with the owner's choice from the chain plan, when they made one, in
    /// place of the rules'. The prompt is generated again, through `template` when the project
    /// has one.
    pub fn for_chain(&mut self, picked: Option<&rules::Picked>, template: Option<&str>) {
        if let Some(picked) = picked {
            self.resolution.pick(picked);
        }
        let date = self.parts.date.clone();
        self.parts.choose(&Chosen {
            model: &self.resolution.model.value,
            effort: self.resolution.effort.value.as_deref(),
            ultracode: self.resolution.ultracode.value,
            date: &date,
            visual: false,
        });
        self.prompt = generate(&self.parts, template);
    }
}

/// The draft for `project`'s next brief: the rule table's resolution (`levels` are the CLI's
/// effort levels in order, empty for the built-in ones; `usage` is what the downshift weighs)
/// and the prompt generated with it.
pub fn draft(
    registry: &Registry,
    project: &Project,
    snapshot: &Snapshot,
    claude_md: Option<&str>,
    levels: &[String],
    usage: &rules::Usage,
    date: &str,
) -> Result<Draft, String> {
    let (roadmap, row) = next_of(snapshot)?;
    let phase = roadmap
        .phase_of(row.id)
        .and_then(|phase| phase.heading.id.as_deref());
    let brief = rules::Brief {
        size: row.size,
        phase,
        title: &row.brief.title,
    };
    let resolution = rules::resolve(registry, project, &brief, levels, usage);
    let chosen = Chosen {
        model: &resolution.model.value,
        effort: resolution.effort.value.as_deref(),
        ultracode: resolution.ultracode.value,
        date,
        visual: resolution.mode.value == LaunchMode::Visual,
    };
    let parts = parts(
        &project.name,
        snapshot,
        claude_md,
        &chosen,
        project.autopilot.enabled,
    )?;
    let prompt = generate(&parts, project.launch.template.as_deref());
    Ok(Draft {
        ep: row.id,
        title: row.brief.title.clone(),
        size: row.size,
        phase: phase.map(str::to_owned),
        resolution,
        parts,
        prompt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEM: &str = "# alpha — session rules

1. **Read first:** `roadmap/README.md` (the next unchecked EP and its
   brief notes), `DECISIONS.md`, then `roadmap/inbox.md`.
2. **Write the brief first.** Front matter.
";

    const SECTION: &str = "# beta — session guide

Intro.

## Read order for every working session

1. `roadmap/charter.md`
2. `roadmap/README.md`

   (and its notes)

### A sub-heading inside the section

- still part of it

## Hard rules

- Never.
";

    #[test]
    fn reads_the_read_first_rule_in_both_shapes() {
        assert_eq!(
            read_first(ITEM).as_deref(),
            Some(
                "**Read first:** `roadmap/README.md` (the next unchecked EP and its\n\
                 brief notes), `DECISIONS.md`, then `roadmap/inbox.md`."
            )
        );
        assert_eq!(
            read_first(SECTION).as_deref(),
            Some(
                "1. `roadmap/charter.md`\n2. `roadmap/README.md`\n\n   (and its notes)\n\n\
                 ### A sub-heading inside the section\n\n- still part of it"
            )
        );
        let paragraph = "Rules.\n\nRead first: the roadmap,\nthen the decisions.\n\nMore.";
        assert_eq!(
            read_first(paragraph).as_deref(),
            Some("Read first: the roadmap,\nthen the decisions.")
        );
        assert_eq!(read_first("# gamma\n\n## Load order\n\n## Next\n"), None);
        assert_eq!(read_first("# delta\n\nNothing about reading.\n"), None);
        // A fenced example is not the rule.
        assert_eq!(read_first("```\n**Read first:** not this\n```\n"), None);
    }

    fn parts_fixture() -> Parts {
        Parts {
            project: "alpha".to_owned(),
            folder: r"C:\Data\alpha".to_owned(),
            date: "2026-11-02".to_owned(),
            model: "fable".to_owned(),
            effort: "xhigh".to_owned(),
            ep: "EP-3".to_owned(),
            title: "Parser".to_owned(),
            size: "M".to_owned(),
            depends: "EP-1".to_owned(),
            track: "core".to_owned(),
            phase: "P1 — Build".to_owned(),
            note: "**EP-3.** Read the tables.".to_owned(),
            read_first: HOUSE_READ_FIRST.to_owned(),
            tasks: "none".to_owned(),
            inbox: "none".to_owned(),
            rules: session_rules("EP-3", None, false),
            ..Parts::default()
        }
    }

    #[test]
    fn templates_fill_their_placeholders_and_keep_the_rest() {
        let parts = parts_fixture();
        let (text, unknown) = render(
            "Run {ep} of {project} ({size}), {{literally}} } {, {nope} {Ep} {nope}.\n{note}",
            &parts,
        );
        assert_eq!(
            text,
            "Run EP-3 of alpha (M), {literally} } {, {nope} {Ep} {nope}.\n**EP-3.** Read the tables."
        );
        assert_eq!(unknown, ["nope"]);
        let wrapped = generate(&parts, Some("Note first.\n\n{opener}"));
        assert!(
            wrapped
                .text
                .starts_with("Note first.\n\nRun the next brief of alpha: EP-3")
        );
        assert_eq!(wrapped.problems, Vec::<String>::new());
        let bad = generate(&parts, Some("{unknown_name}"));
        assert_eq!(bad.text, "{unknown_name}");
        assert_eq!(bad.problems.len(), 1);
        // Every name a template may use is filled.
        for name in Parts::NAMES {
            let (_, unknown) = render(&format!("{{{name}}}"), &parts);
            assert!(unknown.is_empty(), "{name}");
        }
    }

    fn house() -> Snapshot {
        crate::roadmap::snapshot::read_tree(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("fixtures")
                .join("projects")
                .join("house"),
        )
    }

    const CHOSEN: Chosen<'static> = Chosen {
        model: "claude-opus-5-5",
        effort: Some("high"),
        ultracode: false,
        date: "2026-11-02",
        visual: false,
    };

    #[test]
    fn the_opener_of_a_synthetic_project() {
        let snapshot = house();
        let parts = parts("house", &snapshot, None, &CHOSEN, false).unwrap();
        assert_eq!(
            (parts.ep.as_str(), parts.title.as_str(), parts.size.as_str()),
            ("EP-3", "Parser", "M")
        );
        assert_eq!(parts.phase, "P1 — Build");
        assert_eq!(parts.note, "**EP-3.** Read the tables.");
        assert!(!parts.read_first_quoted);
        assert!(
            parts.brief.contains("`roadmap/EP-3-parser.md`"),
            "{}",
            parts.brief
        );
        assert!(
            parts.last_brief.contains("`roadmap/EP-2-registry.md`"),
            "{}",
            parts.last_brief
        );
        // The open tasks as written, the one gated before EP-3 due; the box without an id is
        // the checklist's problem, not a task.
        assert_eq!(
            parts.tasks,
            "- OT-2 (EP-1, 2026-10-02) Install the linker on the laptop [before EP-3]\n\
             - OT-3 (EP-2, 2026-10-03) Look at the backups folder after a week and say whether \
             fourteen\n  daily copies are enough."
        );
        assert!(parts.due.contains("OT-2."), "{}", parts.due);
        assert_eq!(
            parts.inbox,
            "- IN-2 (from the owner, workspace split, 2026-10-04) Sessions start from the \
             project's own\n  workspace file now; nothing to build.\n\
             - IN-3 (from gamma EP-7, 2026-10-05) The export gains a column. [urgent]"
        );
        assert!(parts.triage.starts_with("Triage each open message"));
        assert!(
            parts
                .rules
                .starts_with("1. Work from its brief, `roadmap/EP-3-parser.md`")
        );

        let text = opener(&parts);
        assert!(text.starts_with("Run the next brief of house: EP-3, Parser (M)."));
        assert!(text.contains("with claude-opus-5-5 at high effort"));
        assert!(text.contains("Open inbox messages in roadmap/inbox.md:\n\n- IN-2"));
        assert!(!text.contains("IN-1") && !text.contains("OT-1"));

        // With the project's own rule.
        let quoted = super::parts("house", &snapshot, Some(ITEM), &CHOSEN, false).unwrap();
        assert!(quoted.read_first_quoted);
        assert!(
            opener(&quoted)
                .contains("Read first, as the project's CLAUDE.md says:\n\n**Read first:**")
        );
    }

    #[test]
    fn a_draft_resolves_then_writes_its_prompt() {
        let registry: Registry = toml::from_str(
            "format = 1\n\n[[project]]\nname = \"house\"\npath = \"house\"\nstate = \"active\"\n\n\
             [project.launch]\ntemplate = \"{ep} for {project} on {model}/{effort}: {nope}\"\n",
        )
        .unwrap();
        let project = &registry.projects[0];
        let unread = rules::Usage::unread(chrono::Utc::now());
        let draft = draft(
            &registry,
            project,
            &house(),
            None,
            &[],
            &unread,
            "2026-11-02",
        )
        .unwrap();
        assert_eq!(draft.ep, EpId::new(3));
        // EP-3 sits under the fixture's `## Phase P1 — Build` heading (EP-19c).
        assert_eq!(draft.phase.as_deref(), Some("P1"));
        assert_eq!(draft.resolution.model.value, "fable");
        // The tool's launch mode, as no `[project.launch] mode` is written.
        assert_eq!(
            draft.parts.visual,
            rules::DEFAULT_LAUNCH_MODE == LaunchMode::Visual
        );
        assert_eq!(draft.prompt.text, "EP-3 for house on fable/xhigh: {nope}");
        assert_eq!(draft.prompt.problems.len(), 1);

        // A chained run takes the owner's choice from the plan, headless (EP-19a).
        let mut chained = draft.clone();
        let picked = rules::Picked {
            model: " claude-opus-5-5 ".to_owned(),
            effort: Some("max".to_owned()),
            ultracode: false,
        };
        chained.for_chain(Some(&picked), project.launch.template.as_deref());
        assert_eq!(chained.resolution.model.value, "claude-opus-5-5");
        assert_eq!(chained.resolution.model.source, rules::Source::Owner);
        assert_eq!(chained.resolution.effort.value.as_deref(), Some("max"));
        assert_eq!(
            chained.prompt.text,
            "EP-3 for house on claude-opus-5-5/max: {nope}"
        );
        assert!(!chained.parts.visual);
        // Without a choice, the rules' values stand, and the opener speaks headless.
        let mut plain = draft.clone();
        plain.for_chain(None, None);
        assert_eq!(plain.resolution.model.value, "fable");
        assert_eq!(plain.resolution.model.source, rules::Source::Default);
        assert!(!plain.parts.visual);
        assert!(
            plain
                .prompt
                .text
                .contains("with fable at xhigh effort: write those"),
            "{}",
            plain.prompt.text
        );

        // No next row: why, in words.
        let done = Snapshot {
            tree: Some(crate::roadmap::table::parse(
                "## Phase P1 — Done\n\n| # | Brief | Size | Depends on | Core | Done |\n\
                 |---|---|---|---|---|---|\n| EP-1 | One | S | — | core | ☑ `abc1234` |\n",
            )),
            ..Snapshot::default()
        };
        assert_eq!(
            draft_error(&registry, &done),
            "every row of its roadmap is checked"
        );
        assert!(draft_error(&registry, &Snapshot::default()).starts_with("there is no roadmap"));
    }

    fn draft_error(registry: &Registry, snapshot: &Snapshot) -> String {
        draft(
            registry,
            &registry.projects[0],
            snapshot,
            None,
            &[],
            &rules::Usage::unread(chrono::Utc::now()),
            "2026-11-02",
        )
        .unwrap_err()
    }

    #[test]
    fn the_autopilot_turns_the_last_rule_into_the_commit() {
        let stops = session_rules("EP-7", None, false);
        assert!(stops.ends_with("5. Stop before committing: the commit is the owner's call."));
        let commits = session_rules("EP-7", Some("EP-7-x.md"), true);
        assert!(commits.starts_with("1. Work from its brief, `roadmap/EP-7-x.md`"));
        let last = commits.lines().last().unwrap();
        assert!(
            last.starts_with("5. The autopilot is on for this project"),
            "{last}"
        );
        for words in [
            "(EP-7, D-x)",
            "`docs(roadmap): record EP-7 commit hash`",
            "record the hash in the roadmap's row",
            "Never push",
        ] {
            assert!(last.contains(words), "{words}: {last}");
        }
        assert!(!commits.contains("Stop before committing"));
        let message = commit_message("EP-7");
        assert!(message.starts_with("The owner says yes to the commit: commit in the house"));
        assert!(message.contains("record EP-7 commit hash") && message.contains("Never push"));
    }

    #[test]
    fn the_opener_leaves_empty_parts_out() {
        let parts = parts_fixture();
        let text = opener(&parts);
        assert!(text.contains("on 2026-11-02 with fable at xhigh effort"));
        assert!(text.contains("Read first (the house default"));
        assert!(text.contains("- EP-3 · M · depends on EP-1 · core · phase P1 — Build\n- Parser"));
        assert!(text.contains("Open owner tasks in roadmap/owner-tasks.md: none."));
        assert!(!text.contains("Triage"));
        assert!(text.ends_with("5. Stop before committing: the commit is the owner's call."));
        assert_eq!(effort_words(None, false), "the CLI's default");
        assert_eq!(effort_words(Some("high"), true), "ultracode");
    }

    #[test]
    fn a_visual_opener_leaves_the_model_and_effort_to_vs_code() {
        let mut parts = parts_fixture();
        parts.choose(&Chosen {
            model: "claude-opus-5-5",
            effort: Some("high"),
            ultracode: false,
            date: "2026-11-02",
            visual: true,
        });
        let text = opener(&parts);
        assert!(
            text.contains(
                "launched from clauderoadmapwrangler on 2026-11-02 into VS Code, whose Claude \
                 Code panel chooses its model and effort (the launch suggested claude-opus-5-5 \
                 at high effort): write the ones it runs with into the brief's front matter"
            ),
            "{text}"
        );
        assert!(!text.contains("with claude-opus-5-5 at high effort:"));
        // The rest of the opener is the headless one's.
        let mut headless = parts.clone();
        headless.visual = false;
        let other = opener(&headless);
        assert_eq!(text.lines().count(), other.lines().count());
        assert!(text.ends_with("5. Stop before committing: the commit is the owner's call."));
    }
}
