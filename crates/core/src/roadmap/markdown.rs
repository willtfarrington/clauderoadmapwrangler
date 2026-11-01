//! The Markdown blocks the roadmap parsers look at, read line by line with their line numbers and
//! byte offsets: ATX headings, pipe tables (a header line, a delimiter row, body lines), list items
//! with their continuation lines, blockquotes (the completion notes and the other labelled blocks
//! of a brief, EP-4) and paragraphs. Blank lines, fenced code, HTML comments and thematic breaks
//! are dropped. A table is one whose lines start with `|`, as every surveyed roadmap writes them; a
//! row is one line.

use std::ops::Range;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Line<'a> {
    pub number: usize,
    /// Where the line starts in the whole text.
    pub start: usize,
    /// The line without its line break (`\n` or `\r\n`).
    pub text: &'a str,
}

/// The lines of `text`, after a byte-order mark.
pub(crate) fn lines(text: &str) -> Vec<Line<'_>> {
    let mut start = if text.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let mut found = Vec::new();
    for (index, piece) in text[start..].split('\n').enumerate() {
        found.push(Line {
            number: index + 1,
            start,
            text: piece.strip_suffix('\r').unwrap_or(piece),
        });
        start += piece.len() + 1;
    }
    found
}

#[derive(Debug)]
pub(crate) enum Block<'a> {
    Heading {
        level: usize,
        text: &'a str,
        line: usize,
    },
    /// A table's header line and body lines; its delimiter row is left out.
    Table {
        header: Line<'a>,
        rows: Vec<Line<'a>>,
    },
    /// A list item at the top level, and the column its content starts at.
    Item {
        lines: Vec<Line<'a>>,
        indent: usize,
    },
    /// A blockquote: its lines as written (see [`quoted`] for a line without its marker).
    Quote {
        lines: Vec<Line<'a>>,
    },
    Paragraph {
        lines: Vec<Line<'a>>,
    },
}

pub(crate) fn blocks<'a>(lines: &[Line<'a>]) -> Vec<Block<'a>> {
    let mut found = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let text = line.text;
        if blank(text) || rule(text) {
            i += 1;
        } else if quote(text) {
            let mut end = i + 1;
            while end < lines.len() {
                let next = lines[end].text;
                if quote(next) || (!blank(next) && !starts_block(lines, end)) {
                    end += 1;
                } else {
                    break;
                }
            }
            found.push(Block::Quote {
                lines: lines[i..end].to_vec(),
            });
            i = end;
        } else if let Some(fence) = fence(text) {
            i = after_fence(lines, i, fence);
        } else if opens_comment(text) {
            i = after_comment(lines, i);
        } else if let Some((level, title)) = heading(text) {
            found.push(Block::Heading {
                level,
                text: title,
                line: line.number,
            });
            i += 1;
        } else if table_starts(lines, i) {
            let rows: Vec<Line> = lines[i + 2..]
                .iter()
                .take_while(|row| pipe(row.text))
                .copied()
                .collect();
            i += 2 + rows.len();
            found.push(Block::Table { header: line, rows });
        } else if let Some(indent) = item(text) {
            let end = item_end(lines, i, indent);
            found.push(Block::Item {
                lines: lines[i..end].to_vec(),
                indent,
            });
            i = end;
        } else {
            let mut end = i + 1;
            while end < lines.len() && !blank(lines[end].text) && !starts_block(lines, end) {
                end += 1;
            }
            found.push(Block::Paragraph {
                lines: lines[i..end].to_vec(),
            });
            i = end;
        }
    }
    found
}

/// The line after a list item: indented lines and lazy continuations belong to it, and blank
/// lines do when an indented line follows them.
fn item_end(lines: &[Line], start: usize, indent: usize) -> usize {
    let mut end = start + 1;
    while end < lines.len() {
        let text = lines[end].text;
        if blank(text) {
            match lines[end..].iter().position(|line| !blank(line.text)) {
                Some(k) if indentation(lines[end + k].text) >= indent => end += k,
                _ => break,
            }
        } else if indentation(text) >= indent || !starts_block(lines, end) {
            end += 1;
        } else {
            break;
        }
    }
    end
}

/// Whether the line at `at` starts a block, which ends a paragraph or a lazy continuation.
fn starts_block(lines: &[Line], at: usize) -> bool {
    let text = lines[at].text;
    heading(text).is_some()
        || fence(text).is_some()
        || opens_comment(text)
        || quote(text)
        || rule(text)
        || item(text).is_some()
        || table_starts(lines, at)
}

fn blank(text: &str) -> bool {
    text.trim().is_empty()
}

/// The width of a line's leading whitespace, a tab reaching the next multiple of four.
fn indentation(text: &str) -> usize {
    let mut width = 0;
    for c in text.chars() {
        match c {
            ' ' => width += 1,
            '\t' => width += 4 - width % 4,
            _ => break,
        }
    }
    width
}

/// `text` without up to `indent` columns of leading whitespace.
pub(crate) fn dedent(text: &str, indent: usize) -> &str {
    let mut width = 0;
    for (i, c) in text.char_indices() {
        if width >= indent {
            return &text[i..];
        }
        match c {
            ' ' => width += 1,
            '\t' => width += 4 - width % 4,
            _ => return &text[i..],
        }
    }
    ""
}

/// The line after at most three leading spaces; `None` when it is indented further.
fn unindented(text: &str) -> Option<&str> {
    let spaces = text.len() - text.trim_start_matches(' ').len();
    (spaces <= 3 && !text[spaces..].starts_with('\t')).then(|| &text[spaces..])
}

/// An ATX heading: its level and its title without the closing `#` marks.
fn heading(text: &str) -> Option<(usize, &str)> {
    let text = unindented(text)?;
    let level = text.bytes().take_while(|&b| b == b'#').count();
    let rest = &text[level..];
    if !(1..=6).contains(&level) || !(rest.is_empty() || rest.starts_with([' ', '\t'])) {
        return None;
    }
    let title = rest.trim();
    let open = title.trim_end_matches('#');
    if open.len() < title.len() && (open.is_empty() || open.ends_with([' ', '\t'])) {
        Some((level, open.trim_end()))
    } else {
        Some((level, title))
    }
}

/// A code fence's character and length: three or more backticks or tildes.
fn fence(text: &str) -> Option<(u8, usize)> {
    let text = unindented(text)?;
    let c = *text
        .as_bytes()
        .first()
        .filter(|&&c| c == b'`' || c == b'~')?;
    let run = text.bytes().take_while(|&b| b == c).count();
    (run >= 3).then_some((c, run))
}

/// The line after the one that closes the fence opened at `start`; the end when none does.
fn after_fence(lines: &[Line], start: usize, (c, run): (u8, usize)) -> usize {
    lines[start + 1..]
        .iter()
        .position(|line| {
            unindented(line.text).is_some_and(|text| {
                let closing = text.bytes().take_while(|&b| b == c).count();
                closing >= run && text[closing..].trim().is_empty()
            })
        })
        .map_or(lines.len(), |k| start + k + 2)
}

fn opens_comment(text: &str) -> bool {
    unindented(text).is_some_and(|text| text.starts_with("<!--"))
}

/// The line after the one that closes the HTML comment opened at `start`.
fn after_comment(lines: &[Line], start: usize) -> usize {
    let first = lines[start].text;
    let inside = first.find("<!--").map_or(0, |at| at + 4);
    if first[inside..].contains("-->") {
        return start + 1;
    }
    lines[start + 1..]
        .iter()
        .position(|line| line.text.contains("-->"))
        .map_or(lines.len(), |k| start + k + 2)
}

fn quote(text: &str) -> bool {
    unindented(text).is_some_and(|text| text.starts_with('>'))
}

/// A blockquote's line without its `>` marker and the one space after it; a lazy continuation
/// line (no marker) as written. A nested quote keeps its inner marker.
pub(crate) fn quoted(text: &str) -> &str {
    match unindented(text).and_then(|text| text.strip_prefix('>')) {
        Some(rest) => rest.strip_prefix(' ').unwrap_or(rest),
        None => text.trim_start(),
    }
}

/// A thematic break: three or more of one of `-`, `*`, `_`, with spaces only between.
fn rule(text: &str) -> bool {
    let Some(text) = unindented(text) else {
        return false;
    };
    let marks: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    marks.len() >= 3 && matches!(marks[0], '-' | '*' | '_') && marks.iter().all(|&c| c == marks[0])
}

/// A list item (`-`, `*`, `+`, `1.` or `1)` at most three spaces in, then spaces and text): the
/// column its content starts at.
fn item(text: &str) -> Option<usize> {
    let lead = text.len() - text.trim_start_matches(' ').len();
    let rest = &text[lead..];
    let marker = if rest.starts_with(['-', '*', '+']) {
        1
    } else {
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if !(1..=9).contains(&digits) || !rest[digits..].starts_with(['.', ')']) {
            return None;
        }
        digits + 1
    };
    let after = &rest[marker..];
    let gap = after.len() - after.trim_start_matches(' ').len();
    if lead > 3 || gap == 0 || after.trim().is_empty() {
        return None;
    }
    // Five spaces or more after the marker start indented code: the content is one space in.
    Some(lead + marker + if gap > 4 { 1 } else { gap })
}

/// A line that starts with `|`, at most three spaces in.
pub(crate) fn pipe(text: &str) -> bool {
    unindented(text).is_some_and(|text| text.starts_with('|'))
}

/// A table starts at `at`: a line that starts with `|`, and a delimiter row under it.
fn table_starts(lines: &[Line], at: usize) -> bool {
    pipe(lines[at].text) && lines.get(at + 1).is_some_and(|next| delimiter(next.text))
}

/// A delimiter row: every cell dashes, with an optional colon at either end.
fn delimiter(text: &str) -> bool {
    let cells = cells(text);
    text.contains('-')
        && !cells.is_empty()
        && cells.iter().all(|(cell, _)| {
            let cell = cell.strip_prefix(':').unwrap_or(cell);
            let cell = cell.strip_suffix(':').unwrap_or(cell);
            !cell.is_empty() && cell.bytes().all(|b| b == b'-')
        })
}

/// The cells of a table line, trimmed, with their byte ranges in the line: split at each `|`
/// that no backslash escapes, without the empty pieces outside a leading and a trailing pipe.
pub(crate) fn cells(text: &str) -> Vec<(&str, Range<usize>)> {
    let mut pipes = Vec::new();
    let mut escaped = false;
    for (i, b) in text.bytes().enumerate() {
        if escaped {
            escaped = false;
        } else if b == b'\\' {
            escaped = true;
        } else if b == b'|' {
            pipes.push(i);
        }
    }
    let lead = text.len() - text.trim_start().len();
    let trail = text.trim_end().len();
    let mut pieces = Vec::new();
    let mut from = lead;
    for &at in &pipes {
        pieces.push(from..at);
        from = at + 1;
    }
    pieces.push(from..trail.max(from));
    if pipes.first() == Some(&lead) {
        pieces.remove(0);
    }
    if pipes.last().is_some_and(|&at| at + 1 == trail) {
        pieces.pop();
    }
    pieces
        .into_iter()
        .map(|range| {
            let raw = &text[range.clone()];
            let start = range.start + (raw.len() - raw.trim_start().len());
            let end = (range.start + raw.trim_end().len()).max(start);
            (&text[start..end], start..end)
        })
        .collect()
}

pub(crate) fn cell_texts(text: &str) -> Vec<&str> {
    cells(text).into_iter().map(|(cell, _)| cell).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_table_lines_into_cells() {
        assert_eq!(cell_texts("| a | b |"), ["a", "b"]);
        assert_eq!(cell_texts("|a|b"), ["a", "b"]);
        assert_eq!(cell_texts("a | b |"), ["a", "b"]);
        assert_eq!(cell_texts("| a \\| b | c |"), ["a \\| b", "c"]);
        assert_eq!(cell_texts("| | |"), ["", ""]);
        let (cell, bytes) = &cells("|  EP-3 | x |")[0];
        assert_eq!((*cell, bytes.clone()), ("EP-3", 3..7));
        assert!(delimiter("|---|:---:|--:|"));
        assert!(delimiter("| --- | --- |"));
        assert!(!delimiter("| a | --- |"));
        assert!(!delimiter("| |"));
    }

    #[test]
    fn splits_text_into_blocks() {
        let text = "\
# Title

Some text
| # | a |
|---|---|
| 1 | b |
- item
  continued

    indented, still the item
- next
***
> quoted
> **Label.** more
lazy line
plain

> second
";
        let lines = lines(text);
        let kinds: Vec<String> = blocks(&lines)
            .iter()
            .map(|block| match block {
                Block::Heading { level, text, line } => format!("h{level} {text} @{line}"),
                Block::Table { header, rows } => format!("table @{} {}", header.number, rows.len()),
                Block::Item { lines, indent } => {
                    format!(
                        "item @{} {} lines, indent {indent}",
                        lines[0].number,
                        lines.len()
                    )
                }
                Block::Quote { lines } => {
                    let texts: Vec<&str> = lines.iter().map(|line| quoted(line.text)).collect();
                    format!("quote @{} {}", lines[0].number, texts.join(" / "))
                }
                Block::Paragraph { lines } => {
                    format!("paragraph @{} {} lines", lines[0].number, lines.len())
                }
            })
            .collect();
        assert_eq!(
            kinds,
            [
                "h1 Title @1",
                "paragraph @3 1 lines",
                "table @4 1",
                "item @7 4 lines, indent 2",
                "item @11 1 lines, indent 2",
                "quote @13 quoted / **Label.** more / lazy line / plain",
                "quote @18 second",
            ]
        );
        assert_eq!(quoted(">no space"), "no space");
        assert_eq!(quoted("  > two in"), "two in");
        assert_eq!(quoted("> > nested"), "> nested");
        assert_eq!(dedent("    indented", 2), "  indented");
        assert_eq!(dedent("\tx", 2), "x");
        assert_eq!(dedent("lazy", 2), "lazy");
        assert_eq!(heading("## Closed ##"), Some((2, "Closed")));
        assert_eq!(heading("#hashtag"), None);
        assert_eq!(item("**bold** text"), None);
        assert_eq!(item("12. twelve"), Some(4));
    }
}
