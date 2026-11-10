//! The two checkbox files the tool owns inside a managed project (D-10, D-11, D-13):
//! `roadmap/owner-tasks.md` and `roadmap/inbox.md`, one top-level list item per line of work,
//! `- [ ] XX-n (…) text`, its text wrapping onto continuation lines. Read with byte offsets from
//! `roadmap::markdown`, and ticked in place: the one byte inside the box changes, and a
//! disposition may be appended at the item's end; every other byte stays as it was. The file as
//! it was is copied to the tool's backups folder first, never into the project, and the new text
//! is written atomically (D-29).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::files;
use crate::roadmap::markdown::{Block, blocks, dedent, lines};
use crate::roadmap::table::Problem;

/// One line of work: `- [ ] OT-3 (EP-4, 2026-10-31) text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub checked: bool,
    /// `OT-3`, `IN-12`, as written.
    pub id: String,
    /// The parenthesis after the id, without its brackets.
    pub meta: Option<String>,
    /// The text after the parenthesis; continuation lines joined by line breaks, without the
    /// item's indentation.
    pub text: String,
    pub line: usize,
    /// Where the item's first line starts.
    start: usize,
    /// The byte between the box's brackets.
    mark: usize,
    /// Where the item's last line ends, before its trailing spaces and its line break.
    end: usize,
}

impl Item {
    /// The item as `text`, the file it was read from, writes it: from its first line's start to
    /// its last line's end.
    pub fn written<'a>(&self, text: &'a str) -> &'a str {
        text.get(self.start..self.end).unwrap_or_default()
    }
}

/// A checkbox file as the parser reads it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct List {
    pub items: Vec<Item>,
    /// Boxes without an id of the file's prefix, and ids written twice, by line.
    pub problems: Vec<Problem>,
}

/// What can stop a tick; nothing is written when any of these is returned.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("{0} is not in the file")]
    NotFound(String),
    #[error("{0} is written more than once in the file; tick it by hand")]
    Twice(String),
    #[error("{0} is ticked already")]
    AlreadyTicked(String),
    #[error("{0} is not ticked")]
    NotTicked(String),
    #[error("{0} is ticked with a disposition, and none was given (D-13)")]
    NoDisposition(String),
}

impl Error {
    fn io(path: &Path, source: io::Error) -> Self {
        Error::Io {
            path: path.to_owned(),
            source,
        }
    }
}

/// Reads a checkbox file's text; `prefix` is its ids' letters (`OT`, `IN`).
pub fn parse(text: &str, prefix: &str) -> List {
    let all = lines(text);
    let mut list = List::default();
    for block in blocks(&all) {
        let Block::Item { lines, indent } = block else {
            continue;
        };
        let first = lines[0];
        // The marker and the spaces after it are ASCII, so the column is the byte.
        let content = first.text.get(indent..).unwrap_or_default().trim_start();
        let at = first.start + first.text.len() - content.len();
        let Some(checked) = checkbox(content) else {
            continue;
        };
        let rest = content[3..].trim_start();
        let Some(id_len) = id_length(rest, prefix) else {
            list.problems.push(Problem {
                line: first.number,
                message: format!("a box without an {prefix}-n id"),
            });
            continue;
        };
        let id = &rest[..id_len];
        let after = rest[id_len..].trim_start();
        let (meta, after) = match after.strip_prefix('(').and_then(|inner| {
            closing(inner).map(|close| (&inner[..close], inner[close + 1..].trim_start()))
        }) {
            Some((meta, after)) => (Some(meta.trim().to_owned()), after),
            None => (None, after),
        };
        let mut text = after.trim_end().to_owned();
        for line in &lines[1..] {
            text.push('\n');
            text.push_str(dedent(line.text, indent).trim_end());
        }
        let last = lines.last().expect("an item has its first line");
        if let Some(earlier) = list
            .items
            .iter()
            .find(|item| item.id.eq_ignore_ascii_case(id))
        {
            list.problems.push(Problem {
                line: first.number,
                message: format!("{id} is written twice (line {} too)", earlier.line),
            });
        }
        list.items.push(Item {
            checked,
            id: id.to_owned(),
            meta,
            text: text.trim().to_owned(),
            line: first.number,
            start: first.start,
            mark: at + 1,
            end: last.start + last.text.trim_end().len(),
        });
    }
    list
}

/// `[ ]`, `[x]` or `[X]` opening an item, then a space or nothing: whether it is checked.
fn checkbox(content: &str) -> Option<bool> {
    let checked = match content.get(..3)? {
        "[ ]" => false,
        "[x]" | "[X]" => true,
        _ => return None,
    };
    (content.len() == 3 || content[3..].starts_with([' ', '\t'])).then_some(checked)
}

/// The length of `<prefix>-<digits>` opening `text`, when no letter or digit follows it.
fn id_length(text: &str, prefix: &str) -> Option<usize> {
    let rest = text
        .get(..prefix.len())
        .filter(|start| start.eq_ignore_ascii_case(prefix))
        .and_then(|_| text[prefix.len()..].strip_prefix('-'))?;
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let end = prefix.len() + 1 + digits;
    (digits > 0 && !text[end..].starts_with(|c: char| c.is_alphanumeric())).then_some(end)
}

/// The byte of the `)` that closes a parenthesis whose inside starts `text`.
fn closing(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(i),
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// `text` with item `id` (in any case) checked or cleared, and `append` added after the end of
/// its last line when given.
fn set(
    text: &str,
    prefix: &str,
    id: &str,
    checked: bool,
    append: Option<&str>,
) -> Result<String, Error> {
    let list = parse(text, prefix);
    let mut matching = list
        .items
        .iter()
        .filter(|item| item.id.eq_ignore_ascii_case(id));
    let item = matching
        .next()
        .ok_or_else(|| Error::NotFound(id.to_owned()))?;
    if matching.next().is_some() {
        return Err(Error::Twice(item.id.clone()));
    }
    if item.checked == checked {
        return Err(if checked {
            Error::AlreadyTicked(item.id.clone())
        } else {
            Error::NotTicked(item.id.clone())
        });
    }
    let mut out = String::with_capacity(text.len() + append.map_or(0, |extra| extra.len() + 1));
    out.push_str(&text[..item.mark]);
    out.push(if checked { 'x' } else { ' ' });
    match append {
        Some(extra) => {
            out.push_str(&text[item.mark + 1..item.end]);
            out.push(' ');
            out.push_str(extra);
            out.push_str(&text[item.end..]);
        }
        None => out.push_str(&text[item.mark + 1..]),
    }
    Ok(out)
}

/// Checks or clears item `id` of the file at `path` and appends `append` to it: copies the file
/// as it was to `<backups>\<project folder>\<name>.<date>.md`, then writes the new text
/// atomically. Returns the copy.
pub(crate) fn tick(
    path: &Path,
    prefix: &str,
    id: &str,
    checked: bool,
    append: Option<&str>,
    backups: &Path,
    today: &str,
) -> Result<PathBuf, Error> {
    let old = fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
    let new = set(&old, prefix, id, checked, append)?;
    let copies = copies_dir(path, backups);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (stem, ext) = name.split_at(name.rfind('.').unwrap_or(name.len()));
    let copy = files::dated_copy(old.as_bytes(), &copies, stem, today, ext)
        .map_err(|e| Error::io(&copies, e))?;
    files::write_atomic(path, new.as_bytes()).map_err(|e| Error::io(path, e))?;
    Ok(copy)
}

/// `<backups>\<project folder>` for a file at `<project folder>\roadmap\<name>`.
pub(crate) fn copies_dir(path: &Path, backups: &Path) -> PathBuf {
    let project = path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .map_or_else(|| "project".into(), |name| name.to_owned());
    backups.join(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    const TASKS: &str = "\
# Owner tasks

One line per task: `- [ ] OT-n (EP-k, date) text`.

- [x] OT-1 (EP-0, 2026-10-01) Create the remote.
- [ ] OT-2 (EP-1 (the toolchain), 2026-10-02) Install the linker [before EP-3]
- [ ] OT-3 (EP-2, 2026-10-03) Look at the backups after a week and say whether
  fourteen copies are enough.

  A second paragraph of the same task.
- [ ] OT-4 No parenthesis.
- [ ] Buy a monitor.
- [] OT-5 Not a box.
- [ ] OT-2 (EP-3, 2026-10-04) The same id again.
* [X] ot-6 (EP-3, 2026-10-04) Another marker, a capital X and a lowercase id.
";

    #[test]
    fn reads_items_with_their_parenthesis_and_continuation_lines() {
        let list = parse(TASKS, "OT");
        let summary: Vec<_> = list
            .items
            .iter()
            .map(|item| {
                (
                    item.id.as_str(),
                    item.checked,
                    item.meta.as_deref(),
                    item.line,
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                ("OT-1", true, Some("EP-0, 2026-10-01"), 5),
                ("OT-2", false, Some("EP-1 (the toolchain), 2026-10-02"), 6),
                ("OT-3", false, Some("EP-2, 2026-10-03"), 7),
                ("OT-4", false, None, 11),
                ("OT-2", false, Some("EP-3, 2026-10-04"), 14),
                ("ot-6", true, Some("EP-3, 2026-10-04"), 15),
            ]
        );
        assert_eq!(list.items[1].text, "Install the linker [before EP-3]");
        assert_eq!(
            list.items[2].text,
            "Look at the backups after a week and say whether\nfourteen copies are enough.\n\nA second paragraph of the same task."
        );
        assert_eq!(list.items[3].text, "No parenthesis.");
        let problems: Vec<_> = list.problems.iter().map(ToString::to_string).collect();
        assert_eq!(
            problems,
            [
                "line 12: a box without an OT-n id",
                "line 14: OT-2 is written twice (line 6 too)",
            ]
        );
        assert!(parse("", "OT").items.is_empty());
        assert!(
            parse("- [ ] IN-1 (from beta) wrong file", "OT")
                .items
                .is_empty()
        );
    }

    #[test]
    fn a_tick_changes_the_box_and_nothing_else() {
        let ticked = set(TASKS, "OT", "ot-3", true, None).unwrap();
        assert_eq!(
            ticked,
            TASKS.replacen("- [ ] OT-3", "- [x] OT-3", 1),
            "the id is matched in any case"
        );
        assert_eq!(set(&ticked, "OT", "OT-3", false, None).unwrap(), TASKS);
        let appended = set(TASKS, "OT", "OT-3", true, Some("**Done (2026-11-01).**")).unwrap();
        assert!(appended.contains(
            "- [x] OT-3 (EP-2, 2026-10-03) Look at the backups after a week and say whether\n  fourteen copies are enough.\n\n  A second paragraph of the same task. **Done (2026-11-01).**\n- [ ] OT-4"
        ));
        assert!(matches!(
            set(TASKS, "OT", "OT-1", true, None),
            Err(Error::AlreadyTicked(id)) if id == "OT-1"
        ));
        assert!(matches!(
            set(TASKS, "OT", "OT-3", false, None),
            Err(Error::NotTicked(_))
        ));
        assert!(matches!(
            set(TASKS, "OT", "OT-9", true, None),
            Err(Error::NotFound(id)) if id == "OT-9"
        ));
        assert!(matches!(
            set(TASKS, "OT", "OT-2", true, None),
            Err(Error::Twice(_))
        ));
    }

    #[test]
    fn a_tick_keeps_line_endings_a_byte_order_mark_and_trailing_spaces() {
        let crlf = "\u{feff}# Inbox\r\n\r\n- [ ] IN-1 (from beta EP-4, 2026-10-02) One line.  \r\n- [ ] IN-2 (from gamma, 2026-10-03) Two\r\n  lines.\r\n";
        let ticked = set(
            crlf,
            "IN",
            "IN-2",
            true,
            Some("**Disposition (2026-11-01):** declined."),
        )
        .unwrap();
        assert_eq!(
            ticked,
            "\u{feff}# Inbox\r\n\r\n- [ ] IN-1 (from beta EP-4, 2026-10-02) One line.  \r\n- [x] IN-2 (from gamma, 2026-10-03) Two\r\n  lines. **Disposition (2026-11-01):** declined.\r\n"
        );
        let first = set(
            crlf,
            "IN",
            "IN-1",
            true,
            Some("**Disposition (2026-11-01):** in."),
        )
        .unwrap();
        assert!(first.contains("One line. **Disposition (2026-11-01):** in.  \r\n"));
        let list = parse(crlf, "IN");
        assert_eq!(list.items[1].text, "Two\nlines.");
        assert!(list.problems.is_empty());
    }

    #[test]
    fn a_ticked_file_is_copied_to_the_backups_folder_first() {
        let scratch = Scratch::new("checklist-tick");
        let roadmap = scratch.0.join("alpha").join("roadmap");
        fs::create_dir_all(&roadmap).unwrap();
        let path = roadmap.join("owner-tasks.md");
        fs::write(&path, TASKS).unwrap();
        let backups = scratch.0.join("local").join("backups");
        let copy = tick(&path, "OT", "OT-3", true, None, &backups, "2026-11-01").unwrap();
        assert_eq!(
            copy,
            backups.join("alpha").join("owner-tasks.2026-11-01.md")
        );
        assert_eq!(fs::read_to_string(&copy).unwrap(), TASKS);
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            TASKS.replacen("- [ ] OT-3", "- [x] OT-3", 1)
        );
        // Nothing but the file itself changes in the project's folder.
        assert_eq!(
            scratch.names(Path::new("alpha/roadmap")),
            ["owner-tasks.md"]
        );

        // A refused tick writes nothing and copies nothing.
        let before = fs::read_to_string(&path).unwrap();
        assert!(matches!(
            tick(&path, "OT", "OT-3", true, None, &backups, "2026-11-01"),
            Err(Error::AlreadyTicked(_))
        ));
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        assert_eq!(
            scratch.names(Path::new("local/backups/alpha")),
            ["owner-tasks.2026-11-01.md"]
        );
        // A second tick the same day keeps the first copy and adds another.
        tick(&path, "OT", "OT-3", false, None, &backups, "2026-11-01").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), TASKS);
        assert_eq!(
            scratch.names(Path::new("local/backups/alpha")),
            ["owner-tasks.2026-11-01.2.md", "owner-tasks.2026-11-01.md"]
        );
        assert!(matches!(
            tick(
                &roadmap.join("inbox.md"),
                "IN",
                "IN-1",
                true,
                None,
                &backups,
                "2026-11-01"
            ),
            Err(Error::Io { .. })
        ));
    }
}
