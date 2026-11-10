//! The Markdown the roadmap files write inside their lines, drawn rather than shown as source
//! (D-25). Inline: `**bold**` strong, `` `code` `` in the code face, `~~struck~~` struck through,
//! a `[link](target)` as its text. From EP-15a a run takes its colour from what it is, in three
//! colours the window already uses: a block's opening bold label and a `[before EP-m]` gate in
//! amber, an id (`EP-n`, `D-n`, `OT-n`, `IN-n`) and a link in violet, a path or a command in the
//! code in green. And a text is set in blocks ([`blocks`], drawn by [`note`]): its continuation
//! lines joined, a segment that opens with a bold label ending in a colon set under its label,
//! numbered parts `(1) … (2) …` as an indented list, blank lines kept, paragraphs spaced, at a
//! reading measure of about a hundred characters; an owner item or a message long enough is set
//! a sentence to a bullet. Nothing is reworded or dropped, and a form these rules do not know is
//! one paragraph.

use std::ops::Range;

use eframe::egui::{self, Align, Color32, FontSelection, RichText, text::LayoutJob};

use crate::style;

/// The inline marks in force over a run of text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    pub bold: bool,
    pub code: bool,
    pub struck: bool,
    /// A link's text; its target is not shown.
    pub link: bool,
}

/// What a mark written in the text does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Toggle {
    Bold,
    Code,
    Struck,
    /// A link's `[`.
    LinkOpen,
    /// A link's `](target)`.
    LinkClose,
}

impl Toggle {
    fn apply(self, marks: &mut Marks) {
        match self {
            Toggle::Bold => marks.bold = !marks.bold,
            Toggle::Code => marks.code = !marks.code,
            Toggle::Struck => marks.struck = !marks.struck,
            Toggle::LinkOpen => marks.link = true,
            Toggle::LinkClose => marks.link = false,
        }
    }
}

/// Every mark that takes effect in `text`: its byte offset, its length and what it does. A mark
/// with no closing partner stays as written, and nothing is marked inside code.
fn toggles(text: &str) -> Vec<(usize, usize, Toggle)> {
    let mut found = Vec::new();
    let mut marks = Marks::default();
    let mut at = 0;
    while let Some(c) = text[at..].chars().next() {
        let rest = &text[at..];
        let toggle = if marks.code {
            (c == '`').then_some((1, Toggle::Code))
        } else if marks.link && rest.starts_with("](") {
            // The target runs to its closing parenthesis, which `link_at` found.
            Some((
                rest.find(')').map_or(rest.len(), |end| end + 1),
                Toggle::LinkClose,
            ))
        } else if !marks.link && c == '[' && link_at(rest) {
            Some((1, Toggle::LinkOpen))
        } else {
            [
                ("**", Toggle::Bold),
                ("~~", Toggle::Struck),
                ("`", Toggle::Code),
            ]
            .into_iter()
            .find(|&(mark, toggle)| {
                rest.starts_with(mark)
                    && (is_open(marks, toggle) || rest[mark.len()..].contains(mark))
            })
            .map(|(mark, toggle)| (mark.len(), toggle))
        };
        match toggle {
            Some((len, toggle)) => {
                found.push((at, len, toggle));
                toggle.apply(&mut marks);
                at += len;
            }
            None => at += c.len_utf8(),
        }
    }
    found
}

/// Whether `toggle`'s mark is the one open now, so it closes here.
fn is_open(marks: Marks, toggle: Toggle) -> bool {
    match toggle {
        Toggle::Bold => marks.bold,
        Toggle::Struck => marks.struck,
        _ => marks.code,
    }
}

/// Whether `rest`, which opens with `[`, is a link: `[text](target)`, the target without spaces.
fn link_at(rest: &str) -> bool {
    let Some(close) = rest.find("](") else {
        return false;
    };
    let text = &rest[1..close];
    let target = &rest[close + 2..];
    !text.is_empty()
        && !text.contains(['[', ']'])
        && target
            .find(')')
            .is_some_and(|end| end > 0 && !target[..end].contains(char::is_whitespace))
}

/// The byte ranges of `text`'s code spans, their backticks included.
fn code_ranges(text: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut open = None;
    for (at, len, toggle) in toggles(text) {
        if toggle == Toggle::Code {
            match open.take() {
                None => open = Some(at),
                Some(start) => ranges.push(start..at + len),
            }
        }
    }
    ranges
}

/// `text` split at its markup into runs, each with the marks in force.
pub fn spans(text: &str) -> Vec<(String, Marks)> {
    let mut runs = Vec::new();
    let mut marks = Marks::default();
    let mut from = 0;
    for (at, len, toggle) in toggles(text) {
        if at > from {
            runs.push((text[from..at].to_owned(), marks));
        }
        toggle.apply(&mut marks);
        from = at + len;
    }
    if from < text.len() {
        runs.push((text[from..].to_owned(), marks));
    }
    runs
}

/// What a code span holds, told apart where it is plain (EP-15a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    /// A file or a folder: `roadmap/inbox.md`, `%APPDATA%\…`, `Cargo.toml`.
    Path,
    /// A command line, a program or a flag: `crw scan`, `git status`, `--add-dir`.
    Command,
    /// A name in the code: `list::cell`, `ui.put`, `selectable_labels`, `OwnerItem`.
    Name,
    /// Anything else: `RS+1`, `[x] hash`, a value.
    Other,
}

/// What a run is, past its marks (EP-15a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Text,
    /// An id: `EP-n` (`EP-7a`), `D-n`, `OT-n`, `IN-n`.
    Reference,
    /// A `[before EP-m]` gate.
    Gate,
    Code(Code),
}

/// A run of text with its marks and what it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub text: String,
    pub marks: Marks,
    pub kind: Kind,
}

/// `text` split into runs: [`spans`], each code span told by its kind and the rest split at its
/// ids and gates.
pub fn runs(text: &str) -> Vec<Run> {
    let mut out = Vec::new();
    for (text, marks) in spans(text) {
        if marks.code {
            let kind = Kind::Code(code_kind(&text));
            out.push(Run { text, marks, kind });
        } else {
            for (piece, kind) in tokens(&text) {
                out.push(Run {
                    text: piece.to_owned(),
                    marks,
                    kind,
                });
            }
        }
    }
    out
}

/// The programs whose names open a command in this portfolio's notes.
const PROGRAMS: [&str; 26] = [
    "crw",
    "git",
    "gh",
    "cargo",
    "claude",
    "pwsh",
    "powershell",
    "code",
    "Code.exe",
    "npm",
    "npx",
    "node",
    "uv",
    "uvw",
    "python",
    "py",
    "rustup",
    "rustc",
    "winget",
    "tailscale",
    "curl",
    "ssh",
    "sqlite3",
    "docker",
    "subst",
    "explorer",
];

/// The extensions that make a file name a path.
const EXTENSIONS: [&str; 30] = [
    "md",
    "rs",
    "toml",
    "json",
    "jsonl",
    "ps1",
    "png",
    "svg",
    "ico",
    "txt",
    "yml",
    "yaml",
    "lock",
    "html",
    "css",
    "js",
    "ts",
    "py",
    "sqlite",
    "log",
    "exe",
    "cmd",
    "bat",
    "sh",
    "code-workspace",
    "csv",
    "zip",
    "bak",
    "pdf",
    "xml",
];

/// What the code span `code` holds: a command (its first word a program, an `.exe` given
/// arguments, or a flag), a path (a separator, a home or environment folder, or a file name with
/// a known extension), a name in the code (with `::`, a call, a field, an underscore, or in camel
/// case), or none of these.
pub fn code_kind(code: &str) -> Code {
    let code = code.trim();
    let first = code.split_whitespace().next().unwrap_or_default();
    let arguments = code.contains(char::is_whitespace);
    let flag = first.len() > 1
        && first.starts_with('-')
        && first[1..].starts_with(|c: char| c == '-' || c.is_ascii_alphabetic());
    if PROGRAMS.contains(&first)
        || (arguments && first.to_ascii_lowercase().ends_with(".exe"))
        || flag
    {
        Code::Command
    } else if code.contains(['/', '\\']) || code.starts_with(['~', '%']) || has_extension(code) {
        Code::Path
    } else if is_name(code) {
        Code::Name
    } else {
        Code::Other
    }
}

/// A file name with a known extension: `berths.toml`, `EP-N-<slug>.md`.
fn has_extension(code: &str) -> bool {
    !code.contains(char::is_whitespace)
        && code
            .rsplit_once('.')
            .is_some_and(|(name, extension)| !name.is_empty() && EXTENSIONS.contains(&extension))
}

/// A name in the code: no space outside its brackets, and `::`, a call, a field after a word, an
/// underscore or a lower letter followed by a capital.
fn is_name(code: &str) -> bool {
    let mut depth = 0u32;
    let mut outside = String::new();
    for c in code.chars() {
        match c {
            '(' | '{' | '<' | '[' => {
                depth += 1;
                outside.push(c);
            }
            ')' | '}' | '>' | ']' => {
                depth = depth.saturating_sub(1);
                outside.push(c);
            }
            _ if depth == 0 => outside.push(c),
            _ => {}
        }
    }
    if outside.contains(char::is_whitespace) || !outside.contains(char::is_alphabetic) {
        return false;
    }
    let chars: Vec<char> = outside.chars().collect();
    let field = chars.windows(3).any(|w| {
        (w[0].is_alphabetic() || w[0] == '_' || w[0] == ')')
            && w[1] == '.'
            && (w[2].is_alphabetic() || w[2] == '_')
    });
    let camel = chars
        .windows(2)
        .any(|w| w[0].is_lowercase() && w[1].is_uppercase());
    outside.contains("::") || outside.contains('(') || outside.contains('_') || field || camel
}

/// `text`, plain, split at its ids and its gates.
fn tokens(text: &str) -> Vec<(&str, Kind)> {
    let mut out = Vec::new();
    let mut from = 0;
    let mut at = 0;
    while at < text.len() {
        let boundary = text[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric());
        let found = boundary
            .then(|| {
                gate_at(&text[at..])
                    .map(|len| (len, Kind::Gate))
                    .or_else(|| id_at(&text[at..]).map(|len| (len, Kind::Reference)))
            })
            .flatten();
        match found {
            Some((len, kind)) => {
                if at > from {
                    out.push((&text[from..at], Kind::Text));
                }
                out.push((&text[at..at + len], kind));
                at += len;
                from = at;
            }
            None => at += text[at..].chars().next().map_or(1, char::len_utf8),
        }
    }
    if from < text.len() {
        out.push((&text[from..], Kind::Text));
    }
    out
}

/// The ids a note refers to: briefs, decisions, owner tasks, inbox messages.
const IDS: [&str; 4] = ["EP-", "D-", "OT-", "IN-"];

/// The length of the id `rest` opens with: `EP-15a`, `D-25`, `OT-10`, `IN-6`.
fn id_at(rest: &str) -> Option<usize> {
    let prefix = IDS.into_iter().find(|prefix| rest.starts_with(prefix))?;
    let digits = rest[prefix.len()..]
        .bytes()
        .take_while(u8::is_ascii_digit)
        .count();
    if digits == 0 {
        return None;
    }
    let mut len = prefix.len() + digits;
    let ends = |at: usize| {
        rest[at..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric())
    };
    // A brief added between two others: EP-7a, EP-15b.
    if prefix == "EP-" && rest[len..].starts_with(|c: char| c.is_ascii_lowercase()) && ends(len + 1)
    {
        len += 1;
    }
    ends(len).then_some(len)
}

/// The length of the `[before EP-m]` gate `rest` opens with.
fn gate_at(rest: &str) -> Option<usize> {
    let inner = rest.strip_prefix('[')?;
    if !inner.get(..6)?.eq_ignore_ascii_case("before") {
        return None;
    }
    let after = &inner[6..];
    let spaces = after.len() - after.trim_start_matches(' ').len();
    let id = &after[spaces..];
    if spaces == 0 || !id.starts_with("EP-") {
        return None;
    }
    let len = id_at(id)?;
    id[len..]
        .starts_with(']')
        .then_some(1 + 6 + spaces + len + 1)
}

/// A list item's text with its continuation lines joined by spaces; a line that opens another
/// list item, or follows a blank line, starts a new line.
pub fn reflow(text: &str) -> String {
    let mut out = String::new();
    let mut blank = false;
    for line in text.lines().map(str::trim) {
        if line.is_empty() {
            blank = true;
            continue;
        }
        let item = line.starts_with("- ")
            || line.starts_with("* ")
            || line
                .split_once(". ")
                .is_some_and(|(n, _)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        if !out.is_empty() {
            out.push_str(match (blank, item) {
                (true, _) => "\n\n",
                (false, true) => "\n",
                (false, false) => " ",
            });
        }
        out.push_str(line);
        blank = false;
    }
    out
}

/// A text set for reading: what [`note`] draws (EP-15a).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// A paragraph of inline markup.
    Text(String),
    /// A segment's bold label, on a line of its own and without its `**`.
    Label(String),
    /// A numbered part: its number as written, `(1)`, and its text.
    Part(String, String),
    /// A sentence of a long owner item or message, drawn after a bullet.
    Step(String),
    /// A blank line of the source.
    Gap,
}

/// Over this many characters, or past two sentences, an owner item or a message is split at its
/// sentences (EP-15a).
const LONG: usize = 240;

/// `text` reflowed and set in blocks: each segment that opens with a bold label ending in a colon
/// (`**From EP-7:**`) under its label, numbered parts in sequence as parts, blank lines kept; with
/// `split`, a long paragraph of two sentences or more is set a sentence to a step. Nothing is left
/// out: every character but the marks' and the whitespace the reflow joins is in a block.
pub fn blocks(text: &str, split: bool) -> Vec<Block> {
    let flowed = reflow(text);
    let mut out = Vec::new();
    for (n, paragraph) in flowed.split("\n\n").enumerate() {
        if n > 0 {
            out.push(Block::Gap);
        }
        for line in paragraph.lines() {
            for (label, body) in segments(line) {
                if let Some(label) = label {
                    out.push(Block::Label(label.to_owned()));
                }
                let (lead, parts) = parts(body);
                let steps = if split && is_long(lead) {
                    sentences(lead)
                } else {
                    Vec::new()
                };
                if steps.len() > 1 {
                    out.extend(steps.into_iter().map(|s| Block::Step(s.to_owned())));
                } else if !lead.is_empty() {
                    out.push(Block::Text(lead.to_owned()));
                }
                out.extend(
                    parts
                        .into_iter()
                        .map(|(number, part)| Block::Part(number, part.to_owned())),
                );
            }
        }
    }
    out
}

/// `line` cut before each bold label that ends in a colon and opens a sentence: the text before
/// the first, then each label (its `**` left out) with the text after it.
fn segments(line: &str) -> Vec<(Option<&str>, &str)> {
    let mut labels = Vec::new();
    let mut open = None;
    for (at, len, toggle) in toggles(line) {
        if toggle != Toggle::Bold {
            continue;
        }
        match open.take() {
            None => open = Some(at),
            Some(start) => {
                let label = line[start + 2..at].trim();
                if label.ends_with(':') && opens_sentence(line, start) {
                    labels.push((start, at + len, label));
                }
            }
        }
    }
    let mut out = Vec::new();
    let mut from = 0;
    let mut label = None;
    for (start, end, next) in labels {
        let body = line[from..start].trim();
        if !body.is_empty() || label.is_some() {
            out.push((label, body));
        }
        label = Some(next);
        from = end;
    }
    out.push((label, line[from..].trim()));
    out
}

/// Whether the text at `at` opens a sentence: the line's start, or a space after a full stop, a
/// question or an exclamation mark.
fn opens_sentence(text: &str, at: usize) -> bool {
    let before = &text[..at];
    let trimmed = before.trim_end();
    trimmed.is_empty() || (trimmed.len() < before.len() && trimmed.ends_with(['.', '!', '?']))
}

/// `text`'s lead and its numbered parts, when it numbers them in sequence from `(1)` (at least
/// two, outside code, each between spaces); otherwise all of it is the lead.
fn parts(text: &str) -> (&str, Vec<(String, &str)>) {
    let code = code_ranges(text);
    let mut found: Vec<(usize, String)> = Vec::new();
    let mut from = 0;
    loop {
        let marker = format!("({})", found.len() + 1);
        let next = text[from..]
            .match_indices(&marker)
            .map(|(i, _)| from + i)
            .find(|&at| {
                let end = at + marker.len();
                text[..at]
                    .chars()
                    .next_back()
                    .is_none_or(char::is_whitespace)
                    && text[end..].chars().next().is_none_or(char::is_whitespace)
                    && !code.iter().any(|range| range.contains(&at))
            });
        match next {
            Some(at) => {
                from = at + marker.len();
                found.push((at, marker));
            }
            None => break,
        }
    }
    if found.len() < 2 {
        return (text.trim(), Vec::new());
    }
    let lead = text[..found[0].0].trim();
    let ends: Vec<usize> = found
        .iter()
        .skip(1)
        .map(|(at, _)| *at)
        .chain([text.len()])
        .collect();
    let parts = found
        .into_iter()
        .zip(ends)
        .map(|((at, marker), end)| {
            let part = text[at + marker.len()..end].trim();
            (marker, part)
        })
        .collect();
    (lead, parts)
}

/// Whether a paragraph is long enough to be split at its sentences.
fn is_long(text: &str) -> bool {
    text.chars().count() > LONG || sentences(text).len() > 2
}

/// Words that end with a full stop inside a sentence.
const ABBREVIATIONS: [&str; 9] = [
    "e.g", "i.e", "etc", "vs", "cf", "approx", "incl", "esp", "al",
];

/// `text` cut after each sentence: a full stop, a question or an exclamation mark, then a space
/// and a capital, a digit, code, a bold mark or a quote; never inside code or brackets, nor after
/// an abbreviation.
pub fn sentences(text: &str) -> Vec<&str> {
    let code = code_ranges(text);
    let mut out = Vec::new();
    let mut depth = 0u32;
    let mut from = 0;
    for (at, c) in text.char_indices() {
        if code.iter().any(|range| range.contains(&at)) {
            continue;
        }
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            '.' | '!' | '?' if depth == 0 => {
                let rest = &text[at + 1..];
                let after = rest.trim_start_matches(' ');
                let opens = after.starts_with(|n: char| {
                    n.is_uppercase() || n.is_ascii_digit() || matches!(n, '`' | '*' | '"' | '“')
                });
                if after.len() < rest.len() && opens && !(c == '.' && abbreviated(&text[from..at]))
                {
                    out.push(text[from..=at].trim());
                    from = at + 1;
                }
            }
            _ => {}
        }
    }
    let last = text[from..].trim();
    if !last.is_empty() {
        out.push(last);
    }
    out
}

/// What a run's result says it did, in one sentence (EP-20a, the owner's choice): the first
/// sentence of its first prose paragraph, with list items, headings, rules, a bold label alone on
/// its line, lines of links, tables, fenced code and an intro ending in a colon before a list
/// skipped, and the markdown dropped (bold, code and struck marks gone, a link kept as its text).
/// A first sentence of three words or fewer (`Done.`, `**Not exercised.**`) takes the next with
/// it. With no prose at all, the first list item, else the first heading, else the first line
/// that holds anything, markdown dropped. `None` for an empty text.
pub fn gist(text: &str) -> Option<String> {
    let text = text.replace("\r\n", "\n");
    let mut fenced = false;
    for paragraph in text.split("\n\n") {
        let mut prose: Vec<&str> = Vec::new();
        // The prose ends at a list, a heading or a label after it; the lines after that are still
        // read for the fences they open or close (EP-20a's review).
        let (mut ended, mut listed) = (false, false);
        for line in paragraph.lines().map(str::trim) {
            if is_fence(line) {
                fenced = !fenced;
                continue;
            }
            if fenced || ended || line.is_empty() {
                continue;
            }
            if is_rule(line) {
                // Under prose a rule makes it a heading (setext); alone it is a break.
                prose.clear();
                continue;
            }
            if list_marker(line).is_some() {
                ended = true;
                listed = !prose.is_empty();
                continue;
            }
            if line.starts_with('#') || line.starts_with('|') || label_alone(line) || links(line) {
                ended = !prose.is_empty();
                continue;
            }
            let body = line.trim_start_matches('>').trim_start();
            if !body.is_empty() {
                prose.push(body);
            }
        }
        let joined = prose.join(" ");
        // An intro to the list after it says nothing of its own.
        if prose.is_empty() || listed && joined.trim_end().ends_with(':') {
            continue;
        }
        let plain = plain(&joined);
        let said = sentences(&plain);
        let Some(first) = said.first().copied() else {
            continue;
        };
        return Some(match said.get(1) {
            Some(next) if first.split_whitespace().count() <= 3 => format!("{first} {next}"),
            _ => first.to_owned(),
        });
    }
    // No prose: what a list says first, else a heading, else any line, outside fenced code.
    let (mut item, mut heading, mut any) = (None, None, None);
    fenced = false;
    for line in text.lines().map(str::trim) {
        if is_fence(line) {
            fenced = !fenced;
            continue;
        }
        if fenced || line.is_empty() || is_rule(line) {
            continue;
        }
        if let Some(marker) = list_marker(line) {
            item.get_or_insert(&line[marker..]);
        } else if line.starts_with('#') {
            heading.get_or_insert(line.trim_start_matches('#'));
        } else {
            any.get_or_insert(line.trim_start_matches(['>', '|']));
        }
    }
    let said = plain(item.or(heading).or(any)?);
    (!said.is_empty()).then_some(said)
}

/// Whether `line` opens or closes fenced code.
fn is_fence(line: &str) -> bool {
    line.starts_with("```") || line.starts_with("~~~")
}

/// Whether `line` is a thematic break (`---`, `***`, `___`) or a heading's underline (`===`).
fn is_rule(line: &str) -> bool {
    let marks: Vec<char> = line.chars().filter(|c| !c.is_whitespace()).collect();
    marks.len() >= 3
        && ['-', '*', '_', '=']
            .iter()
            .any(|mark| marks.iter().all(|c| c == mark))
}

/// The length of the list marker `line` opens with (`- `, `* `, `+ `, `1. `, `2) `).
fn list_marker(line: &str) -> Option<usize> {
    if ["- ", "* ", "+ "]
        .iter()
        .any(|marker| line.starts_with(marker))
    {
        return Some(2);
    }
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    let rest = &line[digits..];
    (digits > 0 && (rest.starts_with(". ") || rest.starts_with(") "))).then_some(digits + 2)
}

/// Whether `line` is a bold label alone (`**What was built**`, `**Checks:**`, `**Verified.**`); a
/// bold sentence of more than three words (`**EP-7 is done and committed.**`) is prose.
fn label_alone(line: &str) -> bool {
    let runs = spans(line);
    let bold = runs.iter().any(|(_, marks)| marks.bold)
        && runs
            .iter()
            .all(|(run, marks)| marks.bold || run.chars().all(|c| !c.is_alphanumeric()));
    let said = plain(line);
    bold && !(said.ends_with(['.', '!', '?']) && said.split_whitespace().count() > 3)
}

/// Whether `line` holds links and nothing else that says anything, or is a bare address.
fn links(line: &str) -> bool {
    let runs = spans(line);
    let bare = (line.starts_with("http://") || line.starts_with("https://"))
        && !line.contains(char::is_whitespace);
    bare || (runs.iter().any(|(_, marks)| marks.link)
        && runs
            .iter()
            .filter(|(_, marks)| !marks.link)
            .all(|(run, _)| run.chars().all(|c| !c.is_alphanumeric())))
}

/// `text` without its markdown and with its whitespace runs made single spaces.
fn plain(text: &str) -> String {
    let runs: String = spans(text).into_iter().map(|(run, _)| run).collect();
    runs.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether `before`, the sentence so far, ends with an abbreviation, whose full stop follows.
fn abbreviated(before: &str) -> bool {
    let word = before
        .rsplit(|c: char| c.is_whitespace() || c == '(')
        .next()
        .unwrap_or_default();
    ABBREVIATIONS.contains(&word.to_ascii_lowercase().as_str())
}

/// How a text is drawn: smaller, weaker (a ticked item), and whether a long one is split at its
/// sentences (an owner item, a message, EP-15a).
#[derive(Clone, Copy, Debug, Default)]
pub struct Look {
    pub small: bool,
    pub weak: bool,
    pub sentences: bool,
}

/// `text`'s markup drawn in the label's style: the marks only, no colours (a list row's title).
pub fn job(ui: &egui::Ui, text: &str, look: Look) -> LayoutJob {
    let mut job = LayoutJob::default();
    for (run, marks) in spans(text) {
        let mut rich = RichText::new(run);
        if look.small {
            rich = rich.small();
        }
        if look.weak && !marks.bold {
            rich = rich.weak();
        }
        if marks.code {
            rich = rich.code();
        }
        if marks.bold {
            rich = rich.strong();
        }
        if marks.struck {
            rich = rich.strikethrough();
        }
        rich.append_to(&mut job, ui.style(), FontSelection::Default, Align::Center);
    }
    job
}

/// `text`'s markup drawn with each run in its colour (EP-15a). A text that opens with a bold run
/// opens with a label, drawn in amber as a gate is; with `label`, all of it is one.
pub fn coloured(ui: &egui::Ui, text: &str, look: Look, label: bool) -> LayoutJob {
    let runs = runs(text);
    let leading = if label {
        runs.len()
    } else if text.trim_start().starts_with("**") {
        runs.iter().take_while(|run| run.marks.bold).count()
    } else {
        0
    };
    let mut job = LayoutJob::default();
    for (i, run) in runs.iter().enumerate() {
        let colour = if run.marks.struck {
            Some(ui.visuals().weak_text_color())
        } else if run.marks.link {
            Some(style::reference(ui))
        } else {
            match run.kind {
                Kind::Code(code) => style::code_colour(ui, code),
                _ if i < leading => Some(style::amber(ui)),
                Kind::Reference => Some(style::reference(ui)),
                Kind::Gate => Some(style::amber(ui)),
                Kind::Text => None,
            }
        };
        let mut rich = RichText::new(&run.text);
        if look.small {
            rich = rich.small();
        }
        if run.marks.code {
            rich = rich.code();
        }
        if run.marks.bold {
            rich = rich.strong();
        }
        if run.marks.struck {
            rich = rich.strikethrough();
        }
        rich = match colour {
            Some(colour) if look.weak => rich.color(weaker(ui, colour)),
            Some(colour) => rich.color(colour),
            None if look.weak && !run.marks.bold => rich.weak(),
            None => rich,
        };
        rich.append_to(&mut job, ui.style(), FontSelection::Default, Align::Center);
    }
    job
}

/// `colour` halfway to the weak text's, for a ticked item.
fn weaker(ui: &egui::Ui, colour: Color32) -> Color32 {
    colour.lerp_to_gamma(ui.visuals().weak_text_color(), 0.5)
}

/// A hundred characters of body text: the width a note wraps at in a wide pane (EP-15a).
const MEASURE: &str = "Each line keeps a reading measure of about a hundred characters, so the eye finds the next line fast";

/// The reading measure in points: [`MEASURE`] laid out in the body font.
pub fn measure(ui: &mut egui::Ui) -> f32 {
    style::width_of(ui, |ui| {
        ui.label(MEASURE);
    })
}

/// Before a segment's label, beyond the item spacing.
const LABEL_GAP: f32 = 10.0;
/// Between two paragraphs, beyond the item spacing (the owner's ask at the trial, EP-15a).
const PARAGRAPH_GAP: f32 = 5.0;
/// Between two parts or two steps of one list.
const LIST_GAP: f32 = 2.0;
/// Before a part's number or a step's bullet.
const LIST_INDENT: f32 = 6.0;
/// A step's mark (the owner's ask at the trial, EP-15a).
const BULLET: &str = "•";

/// The space before `block` when `previous` comes before it: a label binds to what follows it, a
/// list's items sit closer than paragraphs.
fn gap(previous: Option<&Block>, block: &Block) -> f32 {
    match (previous, block) {
        (None | Some(Block::Label(_) | Block::Gap), _) => 0.0,
        (Some(_), Block::Label(_)) => LABEL_GAP,
        (Some(Block::Part(..)), Block::Part(..)) | (Some(Block::Step(_)), Block::Step(_)) => {
            LIST_GAP
        }
        _ => PARAGRAPH_GAP,
    }
}

/// `text` set in blocks and drawn at the reading measure (EP-15a): a paragraph wraps under
/// itself, a label stands on its own line, a part wraps beside its number and a step beside its
/// bullet. The text stays selectable for copying.
pub fn note(ui: &mut egui::Ui, text: &str, look: Look) {
    let blocks = blocks(text, look.sentences);
    let width = measure(ui).min(ui.available_width());
    let mark = |mark: &str| {
        let rich = RichText::new(mark).weak();
        if look.small { rich.small() } else { rich }
    };
    let marks = blocks
        .iter()
        .filter_map(|block| match block {
            Block::Part(n, _) => Some(n.as_str()),
            Block::Step(_) => Some(BULLET),
            _ => None,
        })
        .map(|text| {
            style::width_of(ui, |ui| {
                ui.label(mark(text));
            })
        })
        .fold(0.0, f32::max);
    let item = |ui: &mut egui::Ui, text: &str, body: &str| {
        ui.horizontal_top(|ui| {
            ui.add_space(LIST_INDENT);
            style::padded(ui, marks, |ui| {
                ui.label(mark(text));
            });
            ui.add(egui::Label::new(coloured(ui, body, look, false)).wrap());
        });
    };
    ui.scope(|ui| {
        ui.set_max_width(width);
        let mut previous = None;
        for block in &blocks {
            ui.add_space(gap(previous, block));
            match block {
                Block::Text(text) => {
                    ui.add(egui::Label::new(coloured(ui, text, look, false)).wrap());
                }
                Block::Label(label) => {
                    ui.add(egui::Label::new(coloured(ui, label, look, true)).wrap());
                }
                Block::Part(n, text) => item(ui, n, text),
                Block::Step(text) => item(ui, BULLET, text),
                Block::Gap => ui.add_space(PARAGRAPH_GAP),
            }
            previous = Some(block);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A result in the shape the ledger's real ones take (EP-20a's pickup): a summary first, then
    /// labelled paragraphs, lists and a list of sources.
    const RESULT: &str = "EP-7 is done and committed (`a1b2c3d`, then `e4f5a6b` recording the \
hash). Nothing is pushed.

**What was built**
- **The export** in `crates/export`: a column per decade.
- A [README](README.md) section.

**Not exercised.** No session has run it yet. The re-plan reads it.

Sources for the decade rule:
- [Archive guide](https://example.org/guide): folders by decade.
";

    #[test]
    fn the_gist_is_the_first_prose_paragraphs_first_sentence() {
        assert_eq!(
            gist(RESULT).as_deref(),
            Some("EP-7 is done and committed (a1b2c3d, then e4f5a6b recording the hash).")
        );
        // Headings, a label alone, a line of links and a list before the prose are skipped.
        let late = "## Summary\n\n**What it does now**\n- a list item first.\n\n\
                    [one](https://example.org) · [two](https://example.org/2)\n\n\
                    The archive reads its folders by decade. More follows.";
        assert_eq!(
            gist(late).as_deref(),
            Some("The archive reads its folders by decade.")
        );
        // A label that is a sentence of its own takes the next sentence; markdown goes.
        assert_eq!(
            gist("**Done.** EP-5 is committed as `abc1234`; the [push](x) is yours. Then more.")
                .as_deref(),
            Some("Done. EP-5 is committed as abc1234; the push is yours.")
        );
        // Fenced code is no prose; CRLF reads as LF.
        assert_eq!(
            gist("```\nlet x = 1. Not this.\n```\r\n\r\nEP-2 is done and committed. Yes.")
                .as_deref(),
            Some("EP-2 is done and committed.")
        );
        // A result that is only a list says its first item; an empty one nothing.
        assert_eq!(
            gist("- **First** item.\n- Second item.").as_deref(),
            Some("First item.")
        );
        assert_eq!(gist(" \n\n "), None);
        // From EP-20a's review. A fence opened inside a list item, a blank line in its code.
        let done = Some("EP-7 is done and committed.");
        let fenced = "1. Run the install:\n   ```powershell\n   pwsh -File x.ps1\n\n   crw doctor\n   \
                      ```\n2. Restart.\n\nEP-7 is done and committed.";
        assert_eq!(gist(fenced).as_deref(), done);
        // Rules, a heading's underline and an empty quote are no prose.
        for text in [
            "## Summary\n\n---\n\nEP-7 is done and committed.",
            "## Summary\n---\nEP-7 is done and committed.",
            "Summary\n=======\nEP-7 is done and committed.",
            "## Summary\n\n>\n\nEP-7 is done and committed.",
        ] {
            assert_eq!(gist(text).as_deref(), done, "{text:?}");
        }
        // A bold sentence alone is prose; an intro to a list is not; a short first sentence takes
        // the next, bold or not.
        assert_eq!(
            gist("**EP-7 is done and committed.**\n\n- Built X\n\nAsk for the push.").as_deref(),
            done
        );
        assert_eq!(
            gist("Here is what I did:\n- Built X\n\nNothing is pushed, as you asked.").as_deref(),
            Some("Nothing is pushed, as you asked.")
        );
        assert_eq!(
            gist("Done. EP-7 is committed as abc. More.").as_deref(),
            Some("Done. EP-7 is committed as abc.")
        );
        // With no prose, a list's first item, else a heading, outside fenced code.
        assert_eq!(
            gist("```text\nok\n```\n\n- 412 passed").as_deref(),
            Some("412 passed")
        );
        assert_eq!(
            gist("## EP-7 complete\n\n```text\nok\n```").as_deref(),
            Some("EP-7 complete")
        );
        assert_eq!(list_marker("12) twelve"), Some(4));
        assert_eq!(list_marker("2026-11-10 is a date"), None);
    }

    /// The runs as text and a tag: b bold, c code, s struck, l a link.
    fn marked(text: &str) -> Vec<(String, &'static str)> {
        spans(text)
            .into_iter()
            .map(|(run, marks)| {
                let tag = match (marks.bold, marks.code, marks.struck, marks.link) {
                    (false, false, false, false) => "",
                    (true, false, false, false) => "b",
                    (false, true, false, false) => "c",
                    (false, false, true, false) => "s",
                    (true, false, true, false) => "bs",
                    (false, false, false, true) => "l",
                    (false, true, false, true) => "cl",
                    _ => "?",
                };
                (run, tag)
            })
            .collect()
    }

    fn runs_of(expected: &[(&str, &'static str)]) -> Vec<(String, &'static str)> {
        expected
            .iter()
            .map(|&(run, tag)| (run.to_owned(), tag))
            .collect()
    }

    #[test]
    fn splits_text_at_its_inline_marks() {
        assert_eq!(
            marked("**EP-4.** The `berths.toml` file"),
            runs_of(&[
                ("EP-4.", "b"),
                (" The ", ""),
                ("berths.toml", "c"),
                (" file", "")
            ])
        );
        assert_eq!(
            marked("~~**R-2** Gone.~~ **Resolved**"),
            runs_of(&[("R-2", "bs"), (" Gone.", "s"), (" ", ""), ("Resolved", "b")])
        );
        // Nothing is marked inside code; a lone mark stays as written.
        assert_eq!(
            marked("`a **b**` and 2 ** 3"),
            runs_of(&[("a **b**", "c"), (" and 2 ** 3", "")])
        );
        assert_eq!(marked("x ~~ y"), runs_of(&[("x ~~ y", "")]));
        assert_eq!(
            marked("ünïcode `ök`"),
            runs_of(&[("ünïcode ", ""), ("ök", "c")])
        );
        assert!(spans("").is_empty());
    }

    #[test]
    fn a_link_shows_its_text_and_brackets_stay_as_written() {
        assert_eq!(
            marked("See [the house style](../README.md#the-roadmap-house-style) first."),
            runs_of(&[("See ", ""), ("the house style", "l"), (" first.", "")])
        );
        assert_eq!(
            marked("[`README.md`](../README.md) and [x] hash"),
            runs_of(&[("README.md", "cl"), (" and [x] hash", "")])
        );
        // A gate, a box and a link with a space in its target are not links.
        for text in ["Do it [before EP-6]", "- [ ] OT-1 (EP-1)", "[a](b c)"] {
            assert_eq!(marked(text), runs_of(&[(text, "")]), "{text}");
        }
    }

    fn kinds(text: &str) -> Vec<(String, Kind)> {
        runs(text)
            .into_iter()
            .map(|run| (run.text, run.kind))
            .collect()
    }

    fn kinds_of(expected: &[(&str, Kind)]) -> Vec<(String, Kind)> {
        expected
            .iter()
            .map(|&(text, kind)| (text.to_owned(), kind))
            .collect()
    }

    #[test]
    fn ids_and_gates_are_told_from_the_text() {
        use Kind::{Gate, Reference, Text};
        assert_eq!(
            kinds("EP-15a and D-25's addendum, from IN-1 (OT-10) [before EP-16]"),
            kinds_of(&[
                ("EP-15a", Reference),
                (" and ", Text),
                ("D-25", Reference),
                ("'s addendum, from ", Text),
                ("IN-1", Reference),
                (" (", Text),
                ("OT-10", Reference),
                (") ", Text),
                ("[before EP-16]", Gate),
            ])
        );
        // Not ids: inside a word, without a number, with letters after it, inside code.
        for text in ["STEP-6", "the D-n title", "EP-4an", "D-3a", "MID-2"] {
            assert_eq!(kinds(text), kinds_of(&[(text, Text)]), "{text}");
        }
        assert_eq!(
            kinds("`EP-N-<slug>.md` before EP-8"),
            kinds_of(&[
                ("EP-N-<slug>.md", Kind::Code(Code::Path)),
                (" before ", Text),
                ("EP-8", Reference),
            ])
        );
        assert_eq!(
            kinds("[Before  EP-7a]."),
            kinds_of(&[("[Before  EP-7a]", Gate), (".", Text)])
        );
    }

    #[test]
    fn code_is_told_apart_by_what_it_holds() {
        use Code::{Command, Name, Other, Path};
        for (code, kind) in [
            ("crw setting quiet_hours 22:00-07:00", Command),
            ("git show HEAD:roadmap/README.md", Command),
            ("target\\release\\crw.exe doctor", Command),
            ("claude", Command),
            ("--add-dir", Command),
            ("-Force", Command),
            ("roadmap/inbox.md", Path),
            ("%APPDATA%\\clauderoadmapwrangler\\registry.toml", Path),
            ("~/.claude/settings.json", Path),
            ("Cargo.toml", Path),
            ("EP-9-completion-*.md", Path),
            ("target\\release\\crw.exe", Path),
            ("list::cell", Name),
            ("Board::decisions_count()", Name),
            ("Verdict::{Diverged, NotFetched}", Name),
            ("ui.put", Name),
            ("style.interaction.selectable_labels", Name),
            ("ANTHROPIC_API_KEY", Name),
            ("OwnerItem", Name),
            ("acceptEdits", Name),
            ("RS+1", Other),
            ("0.37", Other),
            ("[x] hash", Other),
            ("- [ ] OT-n (EP-k, date) text", Other),
            ("S", Other),
            ("HEAD", Other),
        ] {
            assert_eq!(code_kind(code), kind, "{code}");
        }
    }

    #[test]
    fn joins_continuation_lines_into_one_paragraph() {
        assert_eq!(
            reflow("The ledger view: one row\nper berth, a detail pane;\n  keys ↑ ↓."),
            "The ledger view: one row per berth, a detail pane; keys ↑ ↓."
        );
        assert_eq!(
            reflow("Two parts:\n- one\n- two\n\nAfter a blank line.\n1. first"),
            "Two parts:\n- one\n- two\n\nAfter a blank line.\n1. first"
        );
        assert_eq!(reflow("v2. notes stay"), "v2. notes stay");
    }

    fn text(text: &str) -> Block {
        Block::Text(text.to_owned())
    }

    fn label(text: &str) -> Block {
        Block::Label(text.to_owned())
    }

    fn part(number: &str, text: &str) -> Block {
        Block::Part(number.to_owned(), text.to_owned())
    }

    #[test]
    fn a_note_is_set_in_labelled_paragraphs_and_numbered_parts() {
        let note = "Toasts with the tray icon; a log in the\nledger. **From EP-7:** the badge \
                    events have\ntheir readings. **From EP-10 (the owner's\nanswer at its \
                    close):** closing kills (1) one\n**Where:** here (2) **two**, and (3) \
                    three.\n\nAfter a blank line, `(4)` in code.";
        assert_eq!(
            blocks(note, false),
            [
                text("Toasts with the tray icon; a log in the ledger."),
                label("From EP-7:"),
                text("the badge events have their readings."),
                label("From EP-10 (the owner's answer at its close):"),
                text("closing kills"),
                part("(1)", "one **Where:** here"),
                part("(2)", "**two**, and"),
                part("(3)", "three."),
                Block::Gap,
                text("After a blank line, `(4)` in code."),
            ]
        );
    }

    #[test]
    fn parts_need_a_sequence_and_labels_a_sentence_start() {
        // A lone (1), parts out of order, a number inside a word or code: one paragraph.
        for note in [
            "Only (1) here.",
            "First (2) then (1) and (3).",
            "f(1) and f(2) are calls.",
            "`(1)` and `(2)` are code.",
        ] {
            assert_eq!(blocks(note, false), [text(note)], "{note}");
        }
        // A bold label inside a sentence, or ending in a full stop, is not a segment's.
        for note in [
            "It reads **this:** inside a sentence.",
            "Done. **The start.** Then on.",
            "(2) **Where:** a part's label.",
        ] {
            assert_eq!(blocks(note, false), [text(note)], "{note}");
        }
        // A label at the very start, and lists kept line by line.
        assert_eq!(
            blocks("**Disposition (2026-10-13):** parked.\n- one\n- two", false),
            [
                label("Disposition (2026-10-13):"),
                text("parked."),
                text("- one"),
                text("- two")
            ]
        );
        assert!(blocks("", false).is_empty());
    }

    #[test]
    fn a_long_item_is_split_at_its_sentences() {
        let item = "Open the window, e.g. from its shortcut, and look at the list. Optional, \
                    spends usage: start a run (S, Start headless. Or not.) and watch it. \
                    `crw notifications` lists them. Notes go to `roadmap/inbox.md`.";
        assert_eq!(
            sentences(item),
            [
                "Open the window, e.g. from its shortcut, and look at the list.",
                "Optional, spends usage: start a run (S, Start headless. Or not.) and watch it.",
                "`crw notifications` lists them.",
                "Notes go to `roadmap/inbox.md`.",
            ]
        );
        // Each sentence a step, drawn after a bullet.
        assert_eq!(
            blocks(item, true),
            sentences(item)
                .into_iter()
                .map(|s| Block::Step(s.to_owned()))
                .collect::<Vec<_>>()
        );
        // Not split: a short item, a long one of one sentence, or the item drawn as a note.
        let short = "Create the remote. Then push.";
        assert_eq!(blocks(short, true), [text(short)]);
        let one = format!("A sentence {} without an end", "word ".repeat(60));
        assert_eq!(blocks(&one, true), [text(one.trim())]);
        assert_eq!(blocks(item, false), [text(item)]);
        // Never at a decimal, before a lower-case word or inside code; before code, it is.
        assert_eq!(
            sentences("At 1.5 s. the rest, `a. B` stays."),
            ["At 1.5 s. the rest, `a. B` stays."]
        );
        assert_eq!(
            sentences("Run it. `crw scan` reads."),
            ["Run it.", "`crw scan` reads."]
        );
    }

    #[test]
    fn nothing_is_reworded_or_dropped() {
        let note = "Lead (1) **One:** a. (2) b. **From EP-3 (x):** c [d](e) ~~f~~.\n\nG.";
        let words = |text: &str| -> String {
            text.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        };
        let set: String = blocks(note, true)
            .iter()
            .map(|block| match block {
                Block::Text(text) | Block::Label(text) | Block::Step(text) => text.clone(),
                Block::Part(number, text) => format!("{number} {text}"),
                Block::Gap => String::new(),
            })
            .collect();
        assert_eq!(words(&set), words(note));
    }

    #[test]
    fn paragraphs_are_spaced_and_a_label_binds_to_what_follows() {
        let (lead, step, label) = (text("a"), Block::Step("b".to_owned()), label("c:"));
        let part = part("(1)", "d");
        assert_eq!(gap(None, &lead), 0.0);
        assert_eq!(gap(Some(&lead), &lead), PARAGRAPH_GAP);
        assert_eq!(gap(Some(&lead), &part), PARAGRAPH_GAP);
        assert_eq!(gap(Some(&part), &part), LIST_GAP);
        assert_eq!(gap(Some(&step), &step), LIST_GAP);
        assert_eq!(gap(Some(&lead), &label), LABEL_GAP);
        assert_eq!(gap(Some(&label), &lead), 0.0);
        assert_eq!(gap(Some(&Block::Gap), &lead), 0.0);
    }

    #[test]
    fn the_measure_is_a_hundred_characters() {
        assert_eq!(MEASURE.chars().count(), 100);
    }
}
