//! A project's decision log, `DECISIONS.md` (D-9): the numbered entries `**D-n Title.**`, the
//! dated addenda `> **Addendum (date, source).**` under them, and the variants the survey found,
//! counted only (an index table of `D-n` rows, ADR files). The file sits at the project root or
//! one level down in the package folder. Read-only (D-11); nothing here panics on a malformed
//! file.

use std::fs;
use std::path::{Path, PathBuf};

use crate::roadmap::first_date;
use crate::roadmap::markdown::{Block, blocks, cell_texts, lines, quoted};

pub const FILE: &str = "DECISIONS.md";

/// Folders a nested log is not looked for in.
const SKIPPED: [&str; 6] = [".git", "target", "node_modules", ".venv", "venv", "private"];

/// Where a project keeps ADR files, counted only.
const ADR_DIRS: [&str; 2] = ["roadmap/adr", "docs/adr"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub number: u32,
    pub title: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addendum {
    pub date: Option<String>,
    /// What follows the date in the label: `EP-3`, `maintenance`, `planning`, `from alpha EP-4`.
    pub source: Option<String>,
    pub line: usize,
}

/// A decision log as the parser reads it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Log {
    pub entries: Vec<Entry>,
    pub addenda: Vec<Addendum>,
    /// Rows of an index table whose first cell is a `D-n`.
    pub index_rows: usize,
    /// Entries with a domain id (`**D-PROD-001** — …`), counted only.
    pub domain_entries: usize,
}

impl Log {
    /// The date of the newest addendum.
    pub fn last_addendum(&self) -> Option<&str> {
        self.addenda
            .iter()
            .filter_map(|addendum| addendum.date.as_deref())
            .max()
    }

    pub fn highest(&self) -> Option<u32> {
        self.entries.iter().map(|entry| entry.number).max()
    }

    pub fn entry(&self, number: u32) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.number == number)
    }
}

/// A project's decisions: the log when it has one, and the ADR files counted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Decisions {
    /// The log's path; `None` when the project keeps none.
    pub path: Option<PathBuf>,
    /// Whether the log sits one level down rather than at the root.
    pub nested: bool,
    pub log: Log,
    pub adr_files: usize,
    /// Why the log could not be read, when it exists.
    pub problem: Option<String>,
}

impl Decisions {
    pub fn is_empty(&self) -> bool {
        self.path.is_none() && self.adr_files == 0
    }
}

/// Reads a decision log's text.
pub fn parse(text: &str) -> Log {
    let lines = lines(text);
    let mut log = Log::default();
    for block in blocks(&lines) {
        match block {
            Block::Paragraph { lines } => {
                log.read_entry(lines[0].text.trim(), lines[0].number);
            }
            Block::Item { lines, indent } => {
                let content = lines[0].text.get(indent..).unwrap_or_default().trim();
                log.read_entry(content, lines[0].number);
            }
            Block::Quote { lines } => {
                // A label may wrap onto the next quoted line, so each one is read from the
                // rest of the block.
                let texts: Vec<&str> = lines.iter().map(|line| quoted(line.text)).collect();
                for (i, line) in lines.iter().enumerate() {
                    if texts[i].starts_with("**Addendum")
                        && let Some(addendum) = addendum_of(&texts[i..].join(" "), line.number)
                    {
                        log.addenda.push(addendum);
                    }
                }
            }
            Block::Table { rows, .. } => {
                log.index_rows += rows
                    .iter()
                    .filter(|row| {
                        cell_texts(row.text)
                            .first()
                            .is_some_and(|cell| is_decision_id(cell.trim_matches(['*', '`'])))
                    })
                    .count();
            }
            Block::Heading { .. } => {}
        }
    }
    log
}

impl Log {
    /// A paragraph or list item that opens with an entry, numbered or domain-id.
    fn read_entry(&mut self, text: &str, line: usize) {
        if let Some(entry) = entry_of(text, line) {
            self.entries.push(entry);
        } else if is_domain_entry(text) {
            self.domain_entries += 1;
        }
    }
}

/// `**D-PROD-001**` opening `text`: two or more capitals, a dash and digits, in bold.
fn is_domain_entry(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("**D-") else {
        return false;
    };
    let letters = rest.bytes().take_while(u8::is_ascii_uppercase).count();
    let Some(after) = rest.get(letters..).and_then(|rest| rest.strip_prefix('-')) else {
        return false;
    };
    let digits = after.bytes().take_while(u8::is_ascii_digit).count();
    letters >= 2 && digits > 0 && after[digits..].starts_with("**")
}

/// `D-<digits>` and nothing else.
fn is_decision_id(text: &str) -> bool {
    text.strip_prefix("D-")
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// An entry `**D-n Title.**` opening `text`: the bold run must end in `.**`, so a range label
/// (`**D-9 … D-22**`) and a bold mention in prose are not entries.
fn entry_of(text: &str, line: usize) -> Option<Entry> {
    let rest = text.strip_prefix("**D-")?;
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let number: u32 = rest.get(..digits)?.parse().ok()?;
    let after = rest[digits..].strip_prefix(' ')?;
    let close = after.find(".**")?;
    let title = after[..close].trim();
    (!title.is_empty()).then(|| Entry {
        number,
        title: title.to_owned(),
        line,
    })
}

/// An addendum `**Addendum (date, source).**` opening a quoted line.
fn addendum_of(text: &str, line: usize) -> Option<Addendum> {
    let rest = text.strip_prefix("**Addendum")?;
    let close = rest.find("**")?;
    let label = &rest[..close];
    let group = match (label.find('('), label.rfind(')')) {
        (Some(open), Some(end)) if end > open => label[open + 1..end].trim(),
        _ => "",
    };
    let date = first_date(group);
    let source = match date {
        Some(date) => {
            let at = group.find(date).unwrap_or(0);
            let before = group[..at].trim().trim_end_matches([',', ';']).trim();
            let after = group[at + date.len()..]
                .trim()
                .trim_start_matches([',', ';'])
                .trim();
            match (before.is_empty(), after.is_empty()) {
                (true, true) => None,
                (false, true) => Some(before.to_owned()),
                (true, false) => Some(after.to_owned()),
                (false, false) => Some(format!("{before}, {after}")),
            }
        }
        None => Some(group.to_owned()).filter(|s| !s.is_empty()),
    };
    Some(Addendum {
        date: date.map(str::to_owned),
        source,
        line,
    })
}

/// The decision log of `folder`: `DECISIONS.md` at the root, else the first one level down.
pub fn locate(folder: &Path) -> Option<(PathBuf, bool)> {
    let root = folder.join(FILE);
    if root.is_file() {
        return Some((root, false));
    }
    let mut nested: Vec<PathBuf> = fs::read_dir(folder)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            !SKIPPED.contains(&name.as_str())
                && !name.starts_with('.')
                && entry.file_type().is_ok_and(|t| t.is_dir())
        })
        .map(|entry| entry.path().join(FILE))
        .filter(|path| path.is_file())
        .collect();
    nested.sort();
    nested.into_iter().next().map(|path| (path, true))
}

/// The ADR files of `folder`, in the two places the survey found them.
pub fn adr_files(folder: &Path) -> usize {
    ADR_DIRS
        .iter()
        .filter_map(|dir| fs::read_dir(folder.join(dir)).ok())
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| {
            entry.file_type().is_ok_and(|t| t.is_file())
                && entry.file_name().to_string_lossy().ends_with(".md")
        })
        .count()
}

/// Reads a project's decisions.
pub fn read(folder: &Path) -> Decisions {
    let adr = adr_files(folder);
    let Some((path, nested)) = locate(folder) else {
        return Decisions {
            adr_files: adr,
            ..Decisions::default()
        };
    };
    let (log, problem) = match fs::read_to_string(&path) {
        Ok(text) => (parse(&text), None),
        Err(e) => (Log::default(), Some(e.to_string())),
    };
    Decisions {
        path: Some(path),
        nested,
        log,
        adr_files: adr,
        problem,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    const LOG: &str = "\
# alpha — DECISIONS

Format: **D-n Title.** Decision. *Why.* *Alternatives considered.*

## Index

| # | Title | Decides |
|---|---|---|
| D-1 | Purpose | What the tool is for. |
| D-2 | Identity | Name and licence. |

## Purpose

**D-1 Purpose, scope and tie-breaker.** A desktop application. *Why.* Friction. *Alternatives.*
A showpiece.

> **Addendum (2026-10-31, EP-1).** The short alias is `crw`.

**D-2 Identity, repository, licence.** Folder and crate are `alpha`; a `templates/**` glob
and `docs/**` are the sources. *Why:* the owner's choice.

> **Addendum (2026-10-31, maintenance).** Renamed.
> **Addendum (2026-11-02).** Undated source.

**D-3 … D-5** were settled together, as the planning record says.

- **D-3 Storage.** TOML. *Why.* Editable.

**D-4 Not an entry without its period**

The sentence mentions **D-2** in passing. **D-5 Last one.** Done.

> **Addendum (2026-11-01, from beta EP-4).** Carried by the owner.
> **Addendum (YYYY-MM-DD, EP-9).** A template line.

## Domain ids

- **D-PROD-001** — Primary learners. *(costly to reverse)*
- **D-CLIN-002** — Review model.

**D-ROAD-003** — Planning process, as a paragraph.
**D-X-4** is one capital short, **D-ROAD-** has no number.
";

    #[test]
    fn reads_entries_addenda_and_index_rows() {
        let log = parse(LOG);
        let entries: Vec<_> = log
            .entries
            .iter()
            .map(|e| (e.number, e.title.as_str(), e.line))
            .collect();
        // A range label, a title without its period and a bold entry inside a sentence are not
        // entries.
        assert_eq!(
            entries,
            [
                (1, "Purpose, scope and tie-breaker", 14),
                (2, "Identity, repository, licence", 19),
                (3, "Storage", 27),
            ]
        );
        let addenda: Vec<_> = log
            .addenda
            .iter()
            .map(|a| (a.date.as_deref(), a.source.as_deref(), a.line))
            .collect();
        assert_eq!(
            addenda,
            [
                (Some("2026-10-31"), Some("EP-1"), 17),
                (Some("2026-10-31"), Some("maintenance"), 22),
                (Some("2026-11-02"), None, 23),
                (Some("2026-11-01"), Some("from beta EP-4"), 33),
                (None, Some("YYYY-MM-DD, EP-9"), 34),
            ]
        );
        assert_eq!(log.index_rows, 2);
        assert_eq!(log.domain_entries, 3);
        assert_eq!(log.last_addendum(), Some("2026-11-02"));
        assert_eq!(log.highest(), Some(3));
        assert_eq!(log.entry(3).unwrap().line, 27);
        assert_eq!(log.entry(4), None);
        assert_eq!(log.entry(5), None);
        assert_eq!(parse(""), Log::default());
    }

    #[test]
    fn reads_the_other_addendum_labels() {
        let source = |label: &str| {
            let log = parse(&format!("> **Addendum ({label}).** x\n"));
            let a = &log.addenda[0];
            (a.date.clone(), a.source.clone())
        };
        let date = Some("2026-09-20".to_owned());
        assert_eq!(
            source("2026-09-20, EP-170 close-out"),
            (date.clone(), Some("EP-170 close-out".to_owned()))
        );
        assert_eq!(
            source("2026-09-20, owner — recorded by EP-6; ledger CMP-3"),
            (
                date.clone(),
                Some("owner — recorded by EP-6; ledger CMP-3".to_owned())
            )
        );
        assert_eq!(
            source("EP-3, 2026-09-20"),
            (date.clone(), Some("EP-3".to_owned()))
        );
        assert_eq!(source("2026-09-20"), (date.clone(), None));
        // A label that wraps onto the next quoted line.
        let wrapped = parse(
            "> **Addendum (2026-09-20, EP-50 — a sorted partitioned `COPY` is not order-preserving;\n> owner decision: record for EP-54).** The spine's first validation found it.\n> **Addendum (2026-09-21, EP-51).** Next.\n",
        );
        let labels: Vec<_> = wrapped
            .addenda
            .iter()
            .map(|a| (a.line, a.date.clone(), a.source.clone()))
            .collect();
        assert_eq!(
            labels,
            [
                (
                    1,
                    date,
                    Some(
                        "EP-50 — a sorted partitioned `COPY` is not order-preserving; owner decision: record for EP-54"
                            .to_owned()
                    )
                ),
                (3, Some("2026-09-21".to_owned()), Some("EP-51".to_owned())),
            ]
        );
        assert!(parse("> **Addenda.** none\n").addenda.is_empty());
        assert!(
            parse("```\n> **Addendum (2026-09-20, EP-1).** a template\n```\n")
                .addenda
                .is_empty()
        );
        assert!(
            parse("**Addendum (2026-09-20, EP-1).** not quoted\n")
                .addenda
                .is_empty()
        );
    }

    #[test]
    fn locates_the_log_at_the_root_or_one_level_down() {
        let scratch = Scratch::new("decisions-locate");
        let root = &scratch.0;
        for dir in [".git", "node_modules", "pkg", "docs/adr", "roadmap/adr"] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        fs::write(root.join(".git").join(FILE), "**D-1 Hidden.** x").unwrap();
        fs::write(root.join("node_modules").join(FILE), "**D-1 Hidden.** x").unwrap();
        assert_eq!(read(root), Decisions::default());
        assert!(read(root).is_empty());

        fs::write(root.join("docs/adr/0001-first.md"), "# ADR 1").unwrap();
        fs::write(root.join("docs/adr/README.md"), "# ADRs").unwrap();
        fs::write(root.join("roadmap/adr/0002-second.md"), "# ADR 2").unwrap();
        fs::write(root.join("roadmap/adr/notes.txt"), "not an adr").unwrap();
        let only_adr = read(root);
        assert_eq!((only_adr.path.clone(), only_adr.adr_files), (None, 3));
        assert!(!only_adr.is_empty());

        fs::write(
            root.join("pkg").join(FILE),
            "**D-1 Nested.** x\n\n> **Addendum (2026-10-30, EP-2).** y\n",
        )
        .unwrap();
        let nested = read(root);
        assert_eq!(nested.path, Some(root.join("pkg").join(FILE)));
        assert!(nested.nested);
        assert_eq!(nested.log.entries.len(), 1);
        assert_eq!(nested.log.last_addendum(), Some("2026-10-30"));
        assert_eq!(nested.problem, None);

        fs::write(root.join(FILE), "**D-1 Root.** x\n\n**D-2 Two.** y\n").unwrap();
        let at_root = read(root);
        assert_eq!(at_root.path, Some(root.join(FILE)));
        assert!(!at_root.nested);
        assert_eq!(at_root.log.highest(), Some(2));
        assert_eq!(read(&root.join("missing")), Decisions::default());
    }
}
