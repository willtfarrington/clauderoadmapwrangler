//! A roadmap README read in the house style (D-9): the phase headings, the phase tables with their
//! rows, the brief notes, the next EP and the counts. A phase table is found by its header row,
//! never by a `| #` prefix, and its columns are located by name, so the house header, the older
//! five-column one and the work-packet one read alike. Rows are single lines. Fenced code, HTML
//! comments and blockquotes are never tables, headings or notes (`markdown`). Nothing here writes
//! a file (D-11) or panics on a malformed roadmap: what cannot be read becomes a problem with its
//! line number.

use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

use super::markdown::{Block, Line, blocks, cell_texts, cells, dedent, lines, pipe};
use super::{EpId, Size, decisions, ep_ids, scan_ep_id, starts_word};

/// A roadmap README as the parser reads it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Roadmap {
    /// The phases, in document order.
    pub phases: Vec<Phase>,
    /// Every brief note, in document order.
    pub notes: Vec<Note>,
    /// What could not be read, by line.
    pub problems: Vec<Problem>,
}

/// A heading with a phase table under it, or a heading that carries a phase id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase {
    pub heading: Heading,
    /// The header form of its first phase table; `None` for a phase without one.
    pub profile: Option<Profile>,
    /// The rows of its phase tables, in order.
    pub rows: Vec<Row>,
}

/// The header forms of a phase table (D-9), told apart by their columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// `| # | Brief | Size | Depends on | Core | Done |`, the house style.
    House,
    /// `| # | Brief | Size | Depends on | Done |`, the older house style without Core.
    House5,
    /// `| # | Packet | Size | Depends on | Status |`, the work-packet style.
    Packet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// 1 to 6; 0 for a phase table above the first heading.
    pub level: usize,
    pub line: usize,
    /// The heading as written, without its `#` marks.
    pub text: String,
    /// `P1`, `1`, `R0`, `M0` or `M4–M8`; `None` for a heading of words only.
    pub id: Option<String>,
    pub name: String,
    /// The last parenthesised group (`done 2026-10-31`, `charter`), or what follows ` · `
    /// (`recurring`, `[x] 9bcb7b2`).
    pub status: Option<String>,
    /// A milestone's own mark in a packet-style heading (`[x] hash`, `[ ]`).
    pub mark: Option<Done>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: EpId,
    pub brief: Brief,
    /// The size the Size cell ends on; `None` when it names none.
    pub size: Option<Size>,
    /// The Size cell as written, which keeps a re-scope (`S → L (re-scoped …)`).
    pub size_text: String,
    pub depends: Depends,
    /// `None` without a Core column, or for a word other than core and stretch.
    pub track: Option<Track>,
    pub done: Done,
    pub line: usize,
    /// The row's bytes in the text, without its line break.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brief {
    /// The cell as written.
    pub text: String,
    /// The cell without link markup and without a trailing group that only cites decisions.
    pub title: String,
    /// The target of the first link to an `EP-….md` file (the brief), else of the first link.
    pub link: Option<String>,
    /// The decisions the cell cites, inside or after the link, ranges expanded.
    pub decisions: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Depends {
    /// The cell as written: `—`, ids, ranges, prose such as `everything`.
    pub text: String,
    /// The ids it depends on, ranges expanded; parenthesised annotations (`(met)`, `(cleanest
    /// after EP-4)`) and `before` clauses (`run before EP-21`) are not dependencies.
    pub ids: Vec<EpId>,
}

/// The Core column: the cutline that drops stretch briefs first when time runs short.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    Core,
    Stretch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    /// `☐`, or `[ ]` in the packet style.
    Open,
    /// `[~]`: a packet in progress.
    InProgress,
    /// `◐`: done but not landed, held until its evidence (a CI run) exists.
    NotLanded,
    /// `☑` or `[x]`, with the commit hashes the cell names, backticked or bare, joined by ` + `.
    Checked(Vec<String>),
    /// Anything else, as written.
    Unknown(String),
}

/// One EP's brief notes: a bullet under `**Brief notes.**` (or any label), or a segment
/// `EP-n: …` of a charter phase's prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: EpId,
    /// The bullet without its list marker and its continuation lines without the bullet's
    /// indentation, or the segment from its `EP-n:`; otherwise as written.
    pub text: String,
    pub line: usize,
    /// Its phase's index in [`Roadmap::phases`]; `None` outside every phase.
    pub phase: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// Rows that are checked; `◐` and `[~]` are not.
    pub done: usize,
    pub total: usize,
    pub stretch: usize,
}

impl Roadmap {
    /// Every row, in document order.
    pub fn rows(&self) -> impl Iterator<Item = &Row> {
        self.phases.iter().flat_map(|phase| &phase.rows)
    }

    /// The next EP (D-9): the first row in document order that is not checked, which is not
    /// always the lowest number.
    pub fn next(&self) -> Option<&Row> {
        self.rows().find(|row| !row.done.is_checked())
    }

    pub fn counts(&self) -> Counts {
        Counts::of(self.rows())
    }

    pub fn row(&self, id: EpId) -> Option<&Row> {
        self.rows().find(|row| row.id == id)
    }

    pub fn phase_of(&self, id: EpId) -> Option<&Phase> {
        self.phases
            .iter()
            .find(|phase| phase.rows.iter().any(|row| row.id == id))
    }

    /// The brief notes of `id`: the first note in its own phase, else the first anywhere.
    pub fn note(&self, id: EpId) -> Option<&Note> {
        let phase = self
            .phases
            .iter()
            .position(|phase| phase.rows.iter().any(|row| row.id == id));
        self.notes
            .iter()
            .find(|note| note.id == id && phase.is_some() && note.phase == phase)
            .or_else(|| self.notes.iter().find(|note| note.id == id))
    }

    /// The header form of the first phase table.
    pub fn profile(&self) -> Option<Profile> {
        self.phases.iter().find_map(|phase| phase.profile)
    }
}

impl Phase {
    pub fn counts(&self) -> Counts {
        Counts::of(self.rows.iter())
    }
}

impl Counts {
    fn of<'a>(rows: impl Iterator<Item = &'a Row>) -> Self {
        rows.fold(Counts::default(), |counts, row| Counts {
            done: counts.done + usize::from(row.done.is_checked()),
            total: counts.total + 1,
            stretch: counts.stretch + usize::from(row.track == Some(Track::Stretch)),
        })
    }

    /// Every row is checked, and there is at least one (D-6's "every row checked" fact).
    pub fn all_checked(&self) -> bool {
        self.total > 0 && self.done == self.total
    }
}

impl Profile {
    pub fn as_str(self) -> &'static str {
        match self {
            Profile::House => "house",
            Profile::House5 => "house5",
            Profile::Packet => "packet",
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Track {
    fn parse(cell: &str) -> Option<Track> {
        match cell.trim().to_ascii_lowercase().as_str() {
            "core" => Some(Track::Core),
            "stretch" => Some(Track::Stretch),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Track::Core => "core",
            Track::Stretch => "stretch",
        }
    }
}

impl Done {
    pub fn is_checked(&self) -> bool {
        matches!(self, Done::Checked(_))
    }

    /// A Done or Status cell, or a milestone heading's mark; backticks around it are ignored.
    fn parse(cell: &str) -> Done {
        let text = cell.trim().trim_matches('`').trim();
        if let Some(rest) = ["☑", "[x]", "[X]"]
            .iter()
            .find_map(|mark| text.strip_prefix(*mark))
        {
            Done::Checked(hashes(rest))
        } else if text.starts_with('☐') || text.starts_with("[ ]") {
            Done::Open
        } else if text.starts_with("[~]") {
            Done::InProgress
        } else if text.starts_with('◐') {
            Done::NotLanded
        } else {
            Done::Unknown(text.to_owned())
        }
    }
}

impl fmt::Display for Done {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Done::Open => f.write_str("☐"),
            Done::InProgress => f.write_str("[~]"),
            Done::NotLanded => f.write_str("◐"),
            Done::Checked(hashes) if hashes.is_empty() => f.write_str("☑"),
            Done::Checked(hashes) => write!(f, "☑ {}", hashes.join(" + ")),
            Done::Unknown(text) => f.write_str(text),
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

/// Reads a roadmap README.
pub fn parse(text: &str) -> Roadmap {
    let lines = lines(text);
    let blocks = blocks(&lines);
    let problems = headers_without_delimiters(&blocks);
    assemble(&sections(blocks), problems)
}

/// A phase table's header with no delimiter row under it is a paragraph to Markdown, and the
/// table would vanish without a word.
fn headers_without_delimiters(blocks: &[Block]) -> Vec<Problem> {
    blocks
        .iter()
        .filter_map(|block| match block {
            Block::Paragraph { lines } => Some(lines[0]),
            _ => None,
        })
        .filter(|line| pipe(line.text) && phase_like(&cell_texts(line.text)))
        .map(|line| Problem {
            line: line.number,
            message: "a phase table's header without a delimiter row under it".to_owned(),
        })
        .collect()
}

// --- Sections and phases ------------------------------------------------------------------------

/// A heading and the blocks under it, up to the next heading of any level.
struct Section<'a> {
    level: usize,
    heading: Option<(&'a str, usize)>,
    blocks: Vec<Block<'a>>,
}

fn sections(blocks: Vec<Block<'_>>) -> Vec<Section<'_>> {
    let mut found = vec![Section {
        level: 0,
        heading: None,
        blocks: Vec::new(),
    }];
    for block in blocks {
        match block {
            Block::Heading { level, text, line } => found.push(Section {
                level,
                heading: Some((text, line)),
                blocks: Vec::new(),
            }),
            block => found
                .last_mut()
                .expect("the first section is always there")
                .blocks
                .push(block),
        }
    }
    found
}

/// Phases from the sections: one per section with a phase table or a phase id in its heading; a
/// deeper section with neither belongs to the phase above it. The notes are read last, once the
/// rows they split at are known.
fn assemble(sections: &[Section], mut problems: Vec<Problem>) -> Roadmap {
    let mut phases: Vec<Phase> = Vec::new();
    let mut note_blocks: Vec<(&Block, Option<usize>)> = Vec::new();
    let mut current: Option<(usize, usize)> = None;
    let mut first_line: HashMap<EpId, usize> = HashMap::new();
    for section in sections {
        let mut profile = None;
        let mut first_table = None;
        let mut rows = Vec::new();
        for block in &section.blocks {
            let Block::Table {
                header,
                rows: lines,
            } = block
            else {
                continue;
            };
            let columns = match columns(&cell_texts(header.text)) {
                Ok(Some(columns)) => columns,
                Ok(None) => continue,
                Err(message) => {
                    problems.push(Problem {
                        line: header.number,
                        message,
                    });
                    continue;
                }
            };
            profile = profile.or(Some(columns.profile));
            first_table = first_table.or(Some(header.number));
            for line in lines {
                let Some(row) = row(*line, &columns, &mut problems) else {
                    continue;
                };
                match first_line.get(&row.id) {
                    Some(&first) => problems.push(Problem {
                        line: row.line,
                        message: format!("{} appears again (first at line {first})", row.id),
                    }),
                    None => {
                        first_line.insert(row.id, row.line);
                    }
                }
                rows.push(row);
            }
        }
        let heading = section
            .heading
            .map(|(text, line)| parse_heading(section.level, text, line));
        let has_id = section.level >= 2 && heading.as_ref().is_some_and(|h| h.id.is_some());
        let phase = if profile.is_some() || has_id {
            phases.push(Phase {
                heading: heading.unwrap_or_else(|| Heading::untitled(first_table.unwrap_or(1))),
                profile,
                rows,
            });
            current = Some((phases.len() - 1, section.level));
            current.map(|(index, _)| index)
        } else {
            match current {
                Some((index, level)) if section.level > level => Some(index),
                _ => {
                    current = None;
                    None
                }
            }
        };
        note_blocks.extend(
            section
                .blocks
                .iter()
                .filter(|block| matches!(block, Block::Item { .. } | Block::Paragraph { .. }))
                .map(|block| (block, phase)),
        );
    }
    let notes = notes(&note_blocks, &phases);
    problems.sort_by_key(|problem| problem.line);
    Roadmap {
        phases,
        notes,
        problems,
    }
}

impl Heading {
    /// The heading of a phase table that sits above the first heading.
    fn untitled(line: usize) -> Self {
        Heading {
            level: 0,
            line,
            text: String::new(),
            id: None,
            name: String::new(),
            status: None,
            mark: None,
        }
    }
}

/// A heading's id, name, status and mark (see [`Heading`]).
fn parse_heading(level: usize, text: &str, line: usize) -> Heading {
    let (main, status, mark) = match trailing_group(text) {
        Some(open) if !text[..open].trim().is_empty() => (
            text[..open].trim_end(),
            Some(text[open + 1..text.len() - 1].trim().to_owned()),
            None,
        ),
        _ => match text.rfind(" · ") {
            Some(at) => {
                let status = text[at + " · ".len()..].replace('`', "").trim().to_owned();
                let mark = Some(Done::parse(&status)).filter(|d| !matches!(d, Done::Unknown(_)));
                (text[..at].trim_end(), Some(status), mark)
            }
            None => (text, None, None),
        },
    };
    let (id, name) = match phase_id(main.strip_prefix("Phase ").unwrap_or(main)) {
        Some((id, name)) => (Some(id.to_owned()), name),
        None => (None, main),
    };
    Heading {
        level,
        line,
        text: text.to_owned(),
        id,
        name: name.trim().to_owned(),
        status,
        mark,
    }
}

/// A phase id at the start of `text` (`P1`, `12`, `R0`, `M0`, or a range `M4–M8`) followed by a
/// separator (` — `, ` – `, ` - `, `: `) or nothing: the id and the name after the separator.
fn phase_id(text: &str) -> Option<(&str, &str)> {
    let one = |s: &str| {
        let b = s.as_bytes();
        let letter = usize::from(b.first().is_some_and(u8::is_ascii_uppercase));
        let digits = b[letter..]
            .iter()
            .take_while(|c| c.is_ascii_digit())
            .count();
        let end = letter + digits;
        let suffix = b.get(end).is_some_and(u8::is_ascii_lowercase)
            && !b.get(end + 1).is_some_and(u8::is_ascii_alphanumeric);
        if digits == 0 {
            0
        } else {
            end + usize::from(suffix)
        }
    };
    let mut end = one(text);
    if end == 0 {
        return None;
    }
    if let Some((dash, more)) = ["–", "-"].iter().find_map(|dash| {
        let more = one(text[end..].strip_prefix(*dash)?);
        (more > 0).then_some((dash.len(), more))
    }) {
        end += dash + more;
    }
    let rest = &text[end..];
    if rest.trim().is_empty() {
        return Some((&text[..end], ""));
    }
    [" — ", " – ", " - ", ": "]
        .iter()
        .find_map(|separator| rest.strip_prefix(*separator))
        .map(|name| (&text[..end], name))
}

/// Where a phase table keeps each column.
#[derive(Debug, Clone, Copy)]
struct Columns {
    count: usize,
    id: usize,
    brief: usize,
    size: usize,
    depends: Option<usize>,
    track: Option<usize>,
    done: usize,
    profile: Profile,
}

/// Whether a header starts like a phase table's: `#`, and a Brief or Packet column.
fn phase_like(header: &[&str]) -> bool {
    header.first().is_some_and(|cell| cell.trim() == "#")
        && header.iter().any(|cell| {
            let cell = cell.trim().to_ascii_lowercase();
            cell == "brief" || cell == "packet"
        })
}

/// A phase table's columns, located by name in its header: `None` for any other table; an error
/// for a header that starts like a phase table's but lacks a column the rows need.
fn columns(header: &[&str]) -> Result<Option<Columns>, String> {
    if !phase_like(header) {
        return Ok(None);
    }
    let names: Vec<String> = header
        .iter()
        .map(|cell| {
            let words: Vec<&str> = cell.split_whitespace().collect();
            words.join(" ").to_ascii_lowercase()
        })
        .collect();
    let find = |wanted: &[&str]| {
        names
            .iter()
            .position(|name| wanted.contains(&name.as_str()))
    };
    let brief = find(&["brief", "packet"]).expect("phase_like found one");
    let (Some(size), Some(done)) = (find(&["size"]), find(&["done", "status"])) else {
        let missing = if find(&["size"]).is_none() {
            "Size"
        } else {
            "Done"
        };
        return Err(format!(
            "a table that starts like a phase table has no {missing} column"
        ));
    };
    let track = find(&["core"]);
    let profile = match (names[brief].as_str(), track) {
        ("packet", _) => Profile::Packet,
        (_, Some(_)) => Profile::House,
        (_, None) => Profile::House5,
    };
    Ok(Some(Columns {
        count: names.len(),
        id: 0,
        brief,
        size,
        depends: find(&["depends on"]),
        track,
        done,
        profile,
    }))
}

// --- Rows and cells -----------------------------------------------------------------------------

/// A phase table's row, or `None` with a problem when it cannot be read.
fn row(line: Line, columns: &Columns, problems: &mut Vec<Problem>) -> Option<Row> {
    let cells = cells(line.text);
    let mut problem = |message: String| {
        problems.push(Problem {
            line: line.number,
            message,
        });
    };
    if cells.len() < columns.count {
        problem(format!(
            "a row with {} cells under a header of {}",
            cells.len(),
            columns.count
        ));
        return None;
    }
    // More cells than the header: an unescaped `|` in the Brief cell, whose pieces they are.
    let extra = cells.len() - columns.count;
    let cell = |column: usize| {
        let at = if column > columns.brief {
            column + extra
        } else {
            column
        };
        cells[at].0
    };
    let Some(id) = id_cell(cell(columns.id)) else {
        problem(format!(
            "a row without an EP id: {}",
            quoted(cell(columns.id))
        ));
        return None;
    };
    let brief_bytes = cells[columns.brief].1.start..cells[columns.brief + extra].1.end;
    let done = Done::parse(cell(columns.done));
    if let Done::Unknown(text) = &done {
        problem(format!(
            "{id}: the Done cell {} is not one of the known forms",
            quoted(text)
        ));
    }
    let depends = columns.depends.map(cell).unwrap_or_default();
    Some(Row {
        id,
        brief: brief(&line.text[brief_bytes]),
        size: Size::last_in(cell(columns.size)),
        size_text: cell(columns.size).to_owned(),
        depends: Depends {
            text: depends.to_owned(),
            ids: dependencies(depends),
        },
        track: columns.track.map(cell).and_then(Track::parse),
        done,
        line: line.number,
        span: line.start..line.start + line.text.len(),
    })
}

/// `` `text` ``, cut short when long.
fn quoted(text: &str) -> String {
    match text.char_indices().nth(40) {
        Some((at, _)) => format!("`{}…`", &text[..at]),
        None => format!("`{text}`"),
    }
}

/// The id cell, also when it is bold, code or a link.
fn id_cell(text: &str) -> Option<EpId> {
    let markup = |c: char| matches!(c, '*' | '_' | '`');
    let text = text.trim_matches(markup);
    let text = match text.find("](") {
        Some(close) if text.starts_with('[') => &text[1..close],
        _ => text,
    };
    text.trim_matches(markup).parse().ok()
}

/// The commit hashes in a Done cell: words of 7 to 40 lowercase hex digits.
fn hashes(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| {
            (7..=40).contains(&word.len())
                && word
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
        .map(str::to_owned)
        .collect()
}

fn brief(text: &str) -> Brief {
    let (plain, link) = without_links(text);
    Brief {
        text: text.to_owned(),
        title: title(&plain),
        link,
        decisions: decisions(text),
    }
}

/// `text` with each link `[label](target)` replaced by its label (code spans are copied as they
/// are), and the target of the first link to an `EP-….md` file, else of the first link.
fn without_links(text: &str) -> (String, Option<String>) {
    let mut out = String::with_capacity(text.len());
    let mut targets: Vec<&str> = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find(['[', '`']) {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        if rest.starts_with('`') {
            let run = rest.bytes().take_while(|&b| b == b'`').count();
            let end = rest[run..]
                .find(&rest[..run])
                .map_or(run, |k| run + k + run);
            out.push_str(&rest[..end]);
            rest = &rest[end..];
        } else if let Some((label, target, len)) = link_at(rest) {
            out.push_str(label);
            targets.push(target);
            rest = &rest[len..];
        } else {
            out.push('[');
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    let brief_file = |target: &str| {
        let path = target.split('#').next().unwrap_or_default();
        let name = path.rsplit(['/', '\\']).next().unwrap_or_default();
        name.starts_with("EP-") && name.ends_with(".md")
    };
    let link = targets.iter().copied().find(|target| brief_file(target));
    (out, link.or(targets.first().copied()).map(str::to_owned))
}

/// The link `[label](target)` that `text` starts with: its label, target and length in bytes.
fn link_at(text: &str) -> Option<(&str, &str, usize)> {
    let close = matching(text, b'[', b']')?;
    let after = text[close + 1..].strip_prefix('(')?;
    let end = matching(&text[close + 1..], b'(', b')')?;
    let destination = after[..end - 1].trim();
    let target = match destination.strip_prefix('<') {
        Some(inside) => inside.split('>').next().unwrap_or_default(),
        None => destination.split_whitespace().next().unwrap_or_default(),
    };
    Some((&text[1..close], target, close + 1 + end + 1))
}

/// The byte of the bracket that closes the one `text` opens with, counting nested pairs and
/// skipping backslash escapes.
fn matching(text: &str, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0usize;
    let mut escaped = false;
    for (i, b) in text.bytes().enumerate() {
        if escaped {
            escaped = false;
        } else if b == b'\\' {
            escaped = true;
        } else if b == open {
            depth += 1;
        } else if b == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// A Brief cell's title: the text without a trailing parenthesised group that only cites
/// decisions (`(D-2, D-14)`, `(D-1 … D-8)`), escaped pipes unescaped.
fn title(plain: &str) -> String {
    let plain = plain.trim();
    let title = match trailing_group(plain) {
        Some(open) if cites_only_decisions(&plain[open + 1..plain.len() - 1]) => {
            plain[..open].trim_end()
        }
        _ => plain,
    };
    title.replace("\\|", "|")
}

/// The byte of the `(` whose group ends `text`, when `text` ends with `)`.
fn trailing_group(text: &str) -> Option<usize> {
    if !text.ends_with(')') {
        return None;
    }
    let mut depth = 0usize;
    for (i, b) in text.bytes().enumerate().rev() {
        if b == b')' {
            depth += 1;
        } else if b == b'(' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// Whether a group holds decision references and separators only.
fn cites_only_decisions(inner: &str) -> bool {
    let is_reference = |word: &str| {
        word.strip_prefix("D-")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    };
    !decisions(inner).is_empty()
        && inner
            .split([',', ';', ' '])
            .filter(|word| !word.is_empty())
            .all(|word| matches!(word, "…" | "..." | "–" | "and") || is_reference(word))
}

/// The ids a Depends on cell names, without parenthesised annotations and `before` clauses.
fn dependencies(text: &str) -> Vec<EpId> {
    let plain = without_groups(text);
    let kept: Vec<&str> = plain
        .split(['—', ';', ','])
        .map(|clause| before_word(clause, "before"))
        .collect();
    ep_ids(&kept.join(", "))
}

/// `text` without its parenthesised groups; an unclosed one runs to the end.
fn without_groups(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// The part of `clause` before the word `word` (in any case), or all of it.
fn before_word<'a>(clause: &'a str, word: &str) -> &'a str {
    let lower = clause.to_ascii_lowercase();
    let mut from = 0;
    while let Some(k) = lower[from..].find(word) {
        let at = from + k;
        let end = at + word.len();
        let alone = lower[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric())
            && lower[end..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric());
        if alone {
            return &clause[..at];
        }
        from = end;
    }
    clause
}

// --- Brief notes --------------------------------------------------------------------------------

/// The brief notes: bullets anywhere (a phase's own come first in [`Roadmap::note`]), prose
/// segments only inside a phase, where its rows say which ids start one.
fn notes(blocks: &[(&Block, Option<usize>)], phases: &[Phase]) -> Vec<Note> {
    let mut found = Vec::new();
    for &(block, phase) in blocks {
        match (block, phase) {
            (Block::Item { lines, indent }, _) => {
                if let Some((id, text)) = bullet(lines, *indent) {
                    found.push(Note {
                        id,
                        text,
                        line: lines[0].number,
                        phase,
                    });
                }
            }
            (Block::Paragraph { lines }, Some(index)) => {
                let ids: Vec<EpId> = phases[index].rows.iter().map(|row| row.id).collect();
                found.extend(
                    segments(lines, &ids)
                        .into_iter()
                        .map(|(id, text, line)| Note {
                            id,
                            text,
                            line,
                            phase,
                        }),
                );
            }
            _ => {}
        }
    }
    found
}

/// A brief-notes bullet: a list item whose text opens with a bold EP id (`**EP-3.**`,
/// `**EP-7**`); its text from that label on, continuation lines without the item's indentation.
fn bullet(lines: &[Line], indent: usize) -> Option<(EpId, String)> {
    let content = lines[0].text.get(indent..)?;
    let id = bold_id(content)?;
    let mut text = content.trim_end().to_owned();
    for line in &lines[1..] {
        text.push('\n');
        text.push_str(dedent(line.text, indent).trim_end());
    }
    Some((id, text))
}

/// The EP id of a bold label `**EP-n.**`, `**EP-n:**` or `**EP-n**` at the start of `text`.
fn bold_id(text: &str) -> Option<EpId> {
    let rest = text.strip_prefix("**")?;
    let (id, end) = scan_ep_id(rest, 0)?;
    let after = &rest[end..];
    let after = after.strip_prefix(['.', ':']).unwrap_or(after);
    after.starts_with("**").then_some(id)
}

/// The segments `EP-n: …` of a paragraph that opens with a bold label (`**Scope sketch (refine
/// at EP-26).** EP-27: … EP-28: …`), split at the ids in `ids` only: each id, its text up to the
/// next segment, and the line it starts on.
fn segments(lines: &[Line], ids: &[EpId]) -> Vec<(EpId, String, usize)> {
    if !lines[0].text.trim_start().starts_with("**") {
        return Vec::new();
    }
    let mut text = String::new();
    let mut line_starts = Vec::new();
    for line in lines {
        if !text.is_empty() {
            text.push('\n');
        }
        line_starts.push((text.len(), line.number));
        text.push_str(line.text.trim());
    }
    let mut marks = Vec::new();
    let mut at = 0;
    while let Some(k) = text[at..].find("EP-") {
        let start = at + k;
        at = start + "EP-".len();
        if let Some((id, end)) = scan_ep_id(&text, start)
            && starts_word(&text, start)
            && text[end..].starts_with(':')
            && ids.contains(&id)
        {
            marks.push((start, id));
            at = end;
        }
    }
    marks
        .iter()
        .enumerate()
        .map(|(k, &(start, id))| {
            let end = marks.get(k + 1).map_or(text.len(), |&(next, _)| next);
            let line = line_starts
                .iter()
                .rev()
                .find(|&&(byte, _)| byte <= start)
                .map_or(lines[0].number, |&(_, number)| number);
            (id, text[start..end].trim_end().to_owned(), line)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep(number: u32) -> EpId {
        EpId::new(number)
    }

    fn ids(roadmap: &Roadmap) -> Vec<String> {
        roadmap.rows().map(|row| row.id.to_string()).collect()
    }

    fn messages(roadmap: &Roadmap) -> Vec<String> {
        roadmap.problems.iter().map(Problem::to_string).collect()
    }

    const HOUSE: &str = "\
# demo — Roadmap

## Phase P0 — Planning (done 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-0 | [The plan](EP-0-plan.md) | L | — | core | ☑ `abc1234` |

## Phase P1 — Build (planned 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-1 | [Toolchain](EP-1-toolchain.md) (D-2, D-3) | M | EP-0 | core | ☑ `bcd2345` |
| EP-3 | Parser (D-4) | S → L (re-scoped 2026-10-02) | EP-1 | core | ☐ |
| EP-2 | Viewer | M | EP-1, EP-3 | stretch | ☐ |

**Brief notes.**

- **EP-3.** Read the tables;
  keep the rows.
- **EP-2.** The viewer.
";

    #[test]
    fn reads_a_house_style_roadmap() {
        let roadmap = parse(HOUSE);
        assert!(roadmap.problems.is_empty(), "{:?}", messages(&roadmap));
        assert_eq!(roadmap.profile(), Some(Profile::House));
        assert_eq!(ids(&roadmap), ["EP-0", "EP-1", "EP-3", "EP-2"]);
        let names: Vec<_> = roadmap
            .phases
            .iter()
            .map(|phase| (phase.heading.id.as_deref(), phase.heading.name.as_str()))
            .collect();
        assert_eq!(names, [(Some("P0"), "Planning"), (Some("P1"), "Build")]);

        let next = roadmap.next().unwrap();
        assert_eq!(next.id, ep(3));
        assert_eq!(next.size, Some(Size::L));
        assert_eq!(next.size_text, "S → L (re-scoped 2026-10-02)");
        assert_eq!(next.depends.ids, [ep(1)]);
        assert_eq!(next.brief.title, "Parser");
        assert_eq!(next.brief.link, None);
        assert_eq!(next.brief.decisions, [4]);
        assert_eq!(&HOUSE[next.span.clone()], HOUSE.lines().nth(13).unwrap());
        assert_eq!(next.line, 14);
        assert_eq!(
            roadmap.note(next.id).unwrap().text,
            "**EP-3.** Read the tables;\nkeep the rows."
        );
        assert_eq!(roadmap.phase_of(ep(2)).unwrap().heading.line, 9);
        assert_eq!(
            roadmap.counts(),
            Counts {
                done: 2,
                total: 4,
                stretch: 1
            }
        );
        assert!(!roadmap.counts().all_checked());
        assert_eq!(roadmap.phases[0].counts().done, 1);
        let toolchain = roadmap.row(ep(1)).unwrap();
        assert_eq!(toolchain.brief.link.as_deref(), Some("EP-1-toolchain.md"));
        assert_eq!(toolchain.brief.title, "Toolchain");
        assert_eq!(toolchain.done, Done::Checked(vec!["bcd2345".to_owned()]));
        assert_eq!(toolchain.track, Some(Track::Core));
        assert_eq!(roadmap.row(ep(9)), None);
    }

    #[test]
    fn the_next_ep_is_the_first_unchecked_row_in_table_order() {
        let checked = HOUSE.replace(
            "| EP-1, EP-3 | stretch | ☐ |",
            "| EP-1, EP-3 | stretch | ☑ `cde3456` |",
        );
        assert_eq!(parse(&checked).next().unwrap().id, ep(3));
        let both = checked.replace("| EP-1 | core | ☐ |", "| EP-1 | core | ☑ `def4567` |");
        let roadmap = parse(&both);
        assert_eq!(roadmap.next(), None);
        assert!(roadmap.counts().all_checked());
        assert!(!parse("").counts().all_checked());
    }

    #[test]
    fn reads_every_done_form() {
        let hash = |h: &str| h.to_owned();
        for (cell, done) in [
            ("☐", Done::Open),
            ("[ ]", Done::Open),
            ("[~]", Done::InProgress),
            ("◐", Done::NotLanded),
            ("☑ `3706992`", Done::Checked(vec![hash("3706992")])),
            ("☑ 3706992", Done::Checked(vec![hash("3706992")])),
            (
                "☑ `907b061` + `10b8a47`",
                Done::Checked(vec![hash("907b061"), hash("10b8a47")]),
            ),
            (
                "☑ c29c95a + a4403e6",
                Done::Checked(vec![hash("c29c95a"), hash("a4403e6")]),
            ),
            ("[x] 102af00", Done::Checked(vec![hash("102af00")])),
            ("[X] 102af00", Done::Checked(vec![hash("102af00")])),
            ("`[x] 9bcb7b2`", Done::Checked(vec![hash("9bcb7b2")])),
            ("☑", Done::Checked(Vec::new())),
            // Words that are not hashes: too short, not hex, upper case.
            (
                "☑ `abc12` + `xyz1234` + `ABC1234`",
                Done::Checked(Vec::new()),
            ),
            ("deferred", Done::Unknown("deferred".to_owned())),
            ("", Done::Unknown(String::new())),
        ] {
            assert_eq!(Done::parse(cell), done, "{cell:?}");
        }
        let eight: Vec<String> = (0..8).map(|n| format!("{n}aaaaaa")).collect();
        let cell = format!("☑ `{}`", eight.join("` + `"));
        assert_eq!(Done::parse(&cell), Done::Checked(eight.clone()));
        assert_eq!(Done::Checked(eight).to_string().matches(" + ").count(), 7);
        assert_eq!(Done::NotLanded.to_string(), "◐");
    }

    #[test]
    fn reads_brief_cells() {
        let cases = [
            // A link around the whole title, decisions after it.
            (
                "[Toolchain bootstrap: workspace (`core`, `app`)](EP-1-toolchain.md) (D-2, D-14)",
                "Toolchain bootstrap: workspace (`core`, `app`)",
                Some("EP-1-toolchain.md"),
                vec![2, 14],
            ),
            // Decisions inside the link text, and a trailing space.
            (
                "[Client: typed wrappers, run log (D-5, D-16)](EP-2-client.md) ",
                "Client: typed wrappers, run log",
                Some("EP-2-client.md"),
                vec![5, 16],
            ),
            // A range of decisions in the title, which stays.
            (
                "[Planning: 14 rounds; D-1 … D-3; phases P1 … P4](EP-0-plan.md)",
                "Planning: 14 rounds; D-1 … D-3; phases P1 … P4",
                Some("EP-0-plan.md"),
                vec![1, 2, 3],
            ),
            // No link; a range after the title.
            (
                "Clean-out + manifest runner (D-1 … D-3)",
                "Clean-out + manifest runner",
                None,
                vec![1, 2, 3],
            ),
            // A link around part of the title; a group that is not only decisions stays.
            (
                "Re-plan [P1](EP-6-replan.md): retro (`wsola`, D-14)",
                "Re-plan P1: retro (`wsola`, D-14)",
                Some("EP-6-replan.md"),
                vec![14],
            ),
            // Brackets in code are not links; the brief file wins over an earlier link.
            (
                "`[x]` cells, see [the log](../DECISIONS.md#d-9) and [brief](EP-4-cells.md#scope)",
                "`[x]` cells, see the log and brief",
                Some("EP-4-cells.md#scope"),
                vec![],
            ),
            (
                "[Raw inventory ⏱](<EP-10 raw.md> \"title\")",
                "Raw inventory ⏱",
                Some("EP-10 raw.md"),
                vec![],
            ),
            (
                "See [the charter](charter.md)",
                "See the charter",
                Some("charter.md"),
                vec![],
            ),
            (
                "Unclosed [bracket and (D-7)",
                "Unclosed [bracket and",
                None,
                vec![7],
            ),
            ("A \\| pipe", "A | pipe", None, vec![]),
        ];
        for (cell, title, link, decisions) in cases {
            let brief = brief(cell);
            assert_eq!(brief.text, cell);
            assert_eq!(brief.title, title, "{cell:?}");
            assert_eq!(brief.link.as_deref(), link, "{cell:?}");
            assert_eq!(brief.decisions, decisions, "{cell:?}");
        }
    }

    #[test]
    fn reads_depends_on_cells() {
        let cases: [(&str, &[u32]); 12] = [
            ("—", &[]),
            ("", &[]),
            ("EP-0", &[0]),
            ("EP-6, EP-4", &[6, 4]),
            ("EP-6 … EP-9, EP-14", &[6, 7, 8, 9, 14]),
            ("EP-26–28", &[26, 27, 28]),
            ("EP-3 (met)", &[3]),
            ("EP-1 (cleanest after EP-4)", &[1]),
            ("EP-8 (met) — run before EP-21/EP-17 (LOS regime)", &[8]),
            ("EP-5 (= EP-5b)", &[5]),
            ("EP-20, ≥3 case EPs", &[20]),
            ("all UI EPs", &[]),
        ];
        for (cell, expected) in cases {
            let numbers: Vec<u32> = dependencies(cell).iter().map(|id| id.number).collect();
            assert_eq!(numbers, expected, "{cell:?}");
        }
        assert_eq!(
            dependencies("EP-9 (checkpoint), EP-4b"),
            [
                ep(9),
                EpId {
                    number: 4,
                    suffix: Some('b')
                }
            ]
        );
        assert!(dependencies("everything").is_empty());
        // "beforehand" is not the word before.
        assert_eq!(dependencies("EP-2 beforehand EP-3"), [ep(2), ep(3)]);
    }

    #[test]
    fn reads_phase_headings() {
        let cases: [(&str, Option<&str>, &str, Option<&str>); 12] = [
            (
                "Phase P0 — Planning (done 2026-10-31)",
                Some("P0"),
                "Planning",
                Some("done 2026-10-31"),
            ),
            (
                "Phase 1 — single-campus refit (complete)",
                Some("1"),
                "single-campus refit",
                Some("complete"),
            ),
            (
                "Phase P0 — Foundations: hygiene, toolchain, backups",
                Some("P0"),
                "Foundations: hygiene, toolchain, backups",
                None,
            ),
            (
                "Phase P10 — Text (optional; gated at EP-127) (charter briefs; planned 2026-08-16)",
                Some("P10"),
                "Text (optional; gated at EP-127)",
                Some("charter briefs; planned 2026-08-16"),
            ),
            (
                "R1 — Vertical slice → v0.1",
                Some("R1"),
                "Vertical slice → v0.1",
                None,
            ),
            (
                "M0 — Governance bootstrap · `[x] 9bcb7b2`",
                Some("M0"),
                "Governance bootstrap",
                Some("[x] 9bcb7b2"),
            ),
            (
                "M4–M8 · `[ ]` refinement gates pending",
                Some("M4–M8"),
                "",
                Some("[ ] refinement gates pending"),
            ),
            (
                "Checkpoints · recurring",
                None,
                "Checkpoints",
                Some("recurring"),
            ),
            ("Phase overview", None, "Phase overview", None),
            ("Risks / open items", None, "Risks / open items", None),
            ("1. Introduction", None, "1. Introduction", None),
            ("P0-retro", None, "P0-retro", None),
        ];
        for (text, id, name, status) in cases {
            let heading = parse_heading(2, text, 7);
            assert_eq!(heading.text, text);
            assert_eq!(heading.id.as_deref(), id, "{text:?}");
            assert_eq!(heading.name, name, "{text:?}");
            assert_eq!(heading.status.as_deref(), status, "{text:?}");
        }
        let mark = |text| parse_heading(3, text, 1).mark;
        assert_eq!(
            mark("M0 — Governance bootstrap · `[x] 9bcb7b2`"),
            Some(Done::Checked(vec!["9bcb7b2".to_owned()]))
        );
        assert_eq!(
            mark("M4–M8 · `[ ]` refinement gates pending"),
            Some(Done::Open)
        );
        assert_eq!(mark("Checkpoints · recurring"), None);
        assert_eq!(mark("Phase P0 — Planning (done 2026-10-31)"), None);
    }

    #[test]
    fn tables_are_found_by_their_header_not_their_first_cell() {
        let text = "\
## Notes

| # | File | Why |
|---|---|---|
| 1 | `CLAUDE.md` | rules |

| # | Term | Meaning |
|---|---|---|
| 2 | Brief | one session's work |

| Brief says | It means |
|---|---|
| EP-3 | something |

## Phase P1 — Build

| # | Packet | Size | Status | Depends on | Model |
|:--|:------:|-----:|---|---|---|
| EP-4a | Engine | S | [x] 361b1eb | — | opus |
| EP-4b | Runner | S | [ ] | EP-4a | fable |

| # | Brief | Why |
|---|---|---|
| EP-5 | Lacks a size | — |
";
        let roadmap = parse(text);
        assert_eq!(ids(&roadmap), ["EP-4a", "EP-4b"]);
        assert_eq!(roadmap.profile(), Some(Profile::Packet));
        assert_eq!(roadmap.phases.len(), 1);
        let runner = &roadmap.phases[0].rows[1];
        assert_eq!(runner.done, Done::Open);
        assert_eq!(runner.depends.text, "EP-4a");
        assert_eq!(runner.track, None);
        assert_eq!(
            messages(&roadmap),
            ["line 22: a table that starts like a phase table has no Size column"]
        );

        let five = "| # | Brief | Size | Depends on | Done |\n|---|---|---|---|---|\n| EP-1 | One | S | — | ☑ 0a1b2c3 |\n";
        let roadmap = parse(five);
        assert_eq!(roadmap.profile(), Some(Profile::House5));
        assert_eq!(
            roadmap.phases[0].heading.level, 0,
            "above the first heading"
        );
        assert_eq!(roadmap.phases[0].heading.line, 1);
        assert_eq!(roadmap.phases[0].rows[0].track, None);
    }

    #[test]
    fn code_comments_and_quotes_are_not_read() {
        let text = "\
## Phase P1 — Build

```text
| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-90 | In a fence | S | — | core | ☐ |
````

~~~~
## Phase P9 — In a tilde fence
- **EP-3.** Not a note.
~~~~

<!-- | # | Brief | Size | Depends on | Core | Done | -->
<!--
| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-91 | In a comment | S | — | core | ☐ |
-->

> | # | Brief | Size | Depends on | Core | Done |
> |---|---|---|---|---|---|
> | EP-92 | In a quote | S | — | core | ☐ |
> - **EP-3.** Not a note either.

    | # | Brief | Size | Depends on | Core | Done |
    |---|---|---|---|---|---|

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-3 | Real | S | — | core | ☐ |

```
an unclosed fence hides the rest
| EP-4 | Hidden | S | — | core | ☐ |
";
        let roadmap = parse(text);
        assert_eq!(ids(&roadmap), ["EP-3"]);
        assert_eq!(roadmap.phases.len(), 1);
        assert!(roadmap.notes.is_empty());
        assert!(roadmap.problems.is_empty(), "{:?}", messages(&roadmap));
    }

    #[test]
    fn crlf_and_a_bom_read_the_same() {
        let crlf = format!("\u{feff}{}", HOUSE.replace('\n', "\r\n"));
        let (lf, crlf) = (parse(HOUSE), parse(&crlf));
        assert_eq!(ids(&crlf), ids(&lf));
        assert_eq!(crlf.notes.len(), lf.notes.len());
        for (a, b) in crlf.notes.iter().zip(&lf.notes) {
            assert_eq!((a.id, &a.text, a.line), (b.id, &b.text, b.line));
        }
        let next = crlf.next().unwrap();
        let text = format!("\u{feff}{}", HOUSE.replace('\n', "\r\n"));
        assert!(text[next.span.clone()].starts_with("| EP-3 |"));
        assert!(
            text[next.span.clone()].ends_with("| ☐ |"),
            "no \\r in the span"
        );
    }

    #[test]
    fn reads_brief_note_bullets_in_their_forms() {
        let text = "\
## Phase P1 — Build

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-5 | Zones | M | — | core | ☐ |
| EP-8 | Template | S | EP-5 | core | ☐ |
| EP-7 | Reader | M | EP-8 | core | ☐ |
| EP-9 | Re-plan | S | EP-5 … EP-8 | core | ☐ |

**Brief notes** (reconciled with the design at EP-4: the open items;
D-9 addendum).

- **EP-5.** Create the zones:
  - `raw/`
  - `derived/`

  A second paragraph of the same bullet.
- **EP-8** (runs right after EP-5). The template;
a lazy continuation line.
- Not a note: an ordinary bullet.
* **EP-7:** Read and split front matter.
- **Note.** Not an EP label.
- **EP-9 and EP-10.** Two ids are not one label.

### Details

- **EP-9.** Retro and addenda, under a subsection of the phase.

## Risks

1. **EP-5.** A numbered note outside every phase.
";
        let roadmap = parse(text);
        let notes: Vec<_> = roadmap
            .notes
            .iter()
            .map(|n| (n.id.to_string(), n.line, n.phase))
            .collect();
        assert_eq!(
            notes,
            [
                ("EP-5".to_owned(), 13, Some(0)),
                ("EP-8".to_owned(), 18, Some(0)),
                ("EP-7".to_owned(), 21, Some(0)),
                ("EP-9".to_owned(), 27, Some(0)),
                ("EP-5".to_owned(), 31, None),
            ]
        );
        assert_eq!(
            roadmap.notes[0].text,
            "**EP-5.** Create the zones:\n- `raw/`\n- `derived/`\n\nA second paragraph of the same bullet."
        );
        assert_eq!(
            roadmap.notes[1].text,
            "**EP-8** (runs right after EP-5). The template;\na lazy continuation line."
        );
        assert_eq!(
            roadmap.note(ep(9)).unwrap().text,
            "**EP-9.** Retro and addenda, under a subsection of the phase."
        );
        // The note in the row's own phase wins over one outside every phase.
        assert_eq!(roadmap.note(ep(5)).unwrap().line, 13);
    }

    #[test]
    fn reads_prose_segments_at_the_phase_rows_only() {
        let text = "\
## Phase P0 — Foundations

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-3 | Backups | M | — | core | ☐ |

## Phase P4 — Household (charter)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-27 | Snapshots | M | EP-3 | core | ☐ |
| EP-28 | Liabilities | M | EP-27 | core | ☐ |

**Scope sketch (refine at EP-26).** EP-27: a dated balance per account, as in EP-3: the
archive keeps it (EP-9). EP-28: liabilities carry
a rate and a period.

Ordering rationale: EP-27: not a label, so not a segment.
";
        let roadmap = parse(text);
        let notes: Vec<_> = roadmap
            .notes
            .iter()
            .map(|n| (n.id.number, n.text.as_str(), n.line, n.phase))
            .collect();
        assert_eq!(
            notes,
            [
                (
                    27,
                    "EP-27: a dated balance per account, as in EP-3: the\narchive keeps it (EP-9).",
                    14,
                    Some(1)
                ),
                (
                    28,
                    "EP-28: liabilities carry\na rate and a period.",
                    15,
                    Some(1)
                ),
            ]
        );
        assert_eq!(roadmap.note(ep(3)), None);
    }

    #[test]
    fn what_cannot_be_read_is_reported() {
        let text = "\
## Phase P1 — Build

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | One | S | — | core | ☑ `abc1234` |
| 2 | No id | S | — | core | ☐ |
| EP-3 | Too few cells | S |
| EP-4 | Unknown done | S | — | core | dropped |
| EP-1 | Again | S | — | core | ☐ |
| EP-5 | A | B in the title, unescaped | M | EP-1 | core | ☐ |

| # | Brief | Size | Depends on | Core | Done |
| EP-6 | No delimiter row | S | — | core | ☐ |
";
        let roadmap = parse(text);
        assert_eq!(
            messages(&roadmap),
            [
                "line 6: a row without an EP id: `2`",
                "line 7: a row with 3 cells under a header of 6",
                "line 8: EP-4: the Done cell `dropped` is not one of the known forms",
                "line 9: EP-1 appears again (first at line 5)",
                "line 12: a phase table's header without a delimiter row under it",
            ]
        );
        assert_eq!(ids(&roadmap), ["EP-1", "EP-4", "EP-1", "EP-5"]);
        let joined = roadmap.row(ep(5)).unwrap();
        assert_eq!(joined.brief.text, "A | B in the title, unescaped");
        assert_eq!(joined.size, Some(Size::M));
        assert_eq!(joined.depends.ids, [ep(1)]);
        // An unknown Done cell is not checked, so it can be the next EP.
        assert_eq!(roadmap.next().unwrap().id, ep(4));

        for garbage in [
            "",
            "\n\n",
            "|",
            "| # |",
            "| # | Brief |\n|---|",
            "```",
            "<!--",
            "- ",
            "#",
            "**EP-1.**",
        ] {
            let roadmap = parse(garbage);
            assert!(roadmap.phases.is_empty(), "{garbage:?}");
        }
    }
}
