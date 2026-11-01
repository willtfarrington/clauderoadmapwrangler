//! A managed project's roadmap files, read in the house style (D-9, D-12) and never written
//! (D-11). `table` reads the README: phase headings, phase tables, brief notes, the next EP and the
//! counts (EP-3); `brief` reads the brief files with their front matter, header line and
//! completion notes, and `snapshot` reads a project whole, at `HEAD` and in the working tree, and
//! says whether a session is in progress (EP-4; the decision log is `crate::decisions`). The ids,
//! sizes and reference scanners here are shared by all of them.

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub mod brief;
pub(crate) mod markdown;
pub mod snapshot;
pub mod table;

/// `<folder>\roadmap\README.md`, where the house style keeps a project's roadmap.
pub fn readme(folder: &Path) -> PathBuf {
    folder.join("roadmap").join("README.md")
}

/// A brief's id: `EP-12`, or `EP-4a` for a packet split at pickup (the packet style's remedy,
/// which keeps the number and adds a letter).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EpId {
    pub number: u32,
    /// A lowercase letter: `a` for `EP-4a`.
    pub suffix: Option<char>,
}

impl EpId {
    pub const fn new(number: u32) -> Self {
        Self {
            number,
            suffix: None,
        }
    }
}

impl fmt::Display for EpId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EP-{}", self.number)?;
        match self.suffix {
            Some(suffix) => write!(f, "{suffix}"),
            None => Ok(()),
        }
    }
}

impl FromStr for EpId {
    type Err = String;

    /// Exactly `EP-<number>` with an optional lowercase letter, nothing around it.
    fn from_str(s: &str) -> Result<Self, String> {
        match scan_ep_id(s, 0) {
            Some((id, end)) if end == s.len() => Ok(id),
            _ => Err(format!("{s:?} is not an EP id")),
        }
    }
}

/// A brief's size (D-4): S ≈ 30 min, M ≈ 1 h, L ≈ 2 h of one session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Size {
    S,
    M,
    L,
}

impl Size {
    pub fn as_str(self) -> &'static str {
        match self {
            Size::S => "S",
            Size::M => "M",
            Size::L => "L",
        }
    }

    /// The size a Size cell ends on: the last `S`, `M` or `L` before any parenthesis, so a
    /// re-scope written into the cell (`S → L (re-scoped …)`, `S→M (…)`) gives the new size, and
    /// `S/M` gives M.
    pub fn last_in(text: &str) -> Option<Size> {
        let before = text.split('(').next().unwrap_or_default();
        before
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter_map(|word| match word {
                "S" => Some(Size::S),
                "M" => Some(Size::M),
                "L" => Some(Size::L),
                _ => None,
            })
            .next_back()
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The EP ids `text` names, each once in order of first mention; a range (`EP-6 … EP-12`,
/// `EP-6...EP-12`, `EP-26–34`) stands for every id in it.
pub fn ep_ids(text: &str) -> Vec<EpId> {
    let mut found = Vec::new();
    for (from, to) in references(text, "EP-", true) {
        match (from, to) {
            (from, Some(to)) if from.suffix.is_none() && to.suffix.is_none() => {
                push_range(&mut found, from.number, to.number, EpId::new);
            }
            (from, to) => {
                push_new(&mut found, from);
                if let Some(to) = to {
                    push_new(&mut found, to);
                }
            }
        }
    }
    found
}

/// The decision numbers `text` cites (`D-14`, a range `D-1 … D-8`), each once in order of first
/// mention.
pub fn decisions(text: &str) -> Vec<u32> {
    let mut found = Vec::new();
    for (from, to) in references(text, "D-", false) {
        push_range(
            &mut found,
            from.number,
            to.map_or(from.number, |to| to.number),
            |n| n,
        );
    }
    found
}

/// The first date written `YYYY-MM-DD` in `text`, as the house style dates its notes and addenda.
pub fn first_date(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let digits = |range: std::ops::Range<usize>| bytes[range].iter().all(u8::is_ascii_digit);
    (0..bytes.len().saturating_sub(9)).find_map(|i| {
        let shaped = digits(i..i + 4)
            && bytes[i + 4] == b'-'
            && digits(i + 5..i + 7)
            && bytes[i + 7] == b'-'
            && digits(i + 8..i + 10)
            && !bytes.get(i + 10).is_some_and(u8::is_ascii_digit)
            && (i == 0 || !bytes[i - 1].is_ascii_digit());
        shaped.then(|| &text[i..i + 10])
    })
}

/// The longest range expanded into single references; a longer one keeps its two ends.
const MAX_RANGE: u32 = 500;

fn push_range<T: PartialEq>(found: &mut Vec<T>, from: u32, to: u32, make: impl Fn(u32) -> T) {
    if from <= to && to - from <= MAX_RANGE {
        for n in from..=to {
            push_new(found, make(n));
        }
    } else {
        push_new(found, make(from));
        push_new(found, make(to));
    }
}

fn push_new<T: PartialEq>(found: &mut Vec<T>, item: T) {
    if !found.contains(&item) {
        found.push(item);
    }
}

/// The EP id at byte `at` of `text`, and the byte after it: `EP-`, digits, and one lowercase
/// letter that no other letter or digit follows. The caller checks what precedes `at`.
pub(crate) fn scan_ep_id(text: &str, at: usize) -> Option<(EpId, usize)> {
    let (number, mut end) = scan_number(text, at, "EP-")?;
    let bytes = text.as_bytes();
    let mut suffix = None;
    if let Some(&c) = bytes.get(end).filter(|c| c.is_ascii_lowercase())
        && !bytes.get(end + 1).is_some_and(u8::is_ascii_alphanumeric)
    {
        suffix = Some(char::from(c));
        end += 1;
    }
    if bytes.get(end).is_some_and(u8::is_ascii_alphanumeric) {
        return None;
    }
    Some((EpId { number, suffix }, end))
}

/// `prefix` and digits at byte `at`: the number and the byte after its last digit.
fn scan_number(text: &str, at: usize, prefix: &str) -> Option<(u32, usize)> {
    let digits_at = at + prefix.len();
    if !text.get(at..)?.starts_with(prefix) {
        return None;
    }
    let digits = text[digits_at..]
        .bytes()
        .take_while(u8::is_ascii_digit)
        .count();
    let number = text.get(digits_at..digits_at + digits)?.parse().ok()?;
    Some((number, digits_at + digits))
}

/// Whether the reference at byte `at` starts a word: no letter, digit, `-` or `_` before it.
pub(crate) fn starts_word(text: &str, at: usize) -> bool {
    text[..at]
        .chars()
        .next_back()
        .is_none_or(|c| !(c.is_alphanumeric() || c == '-' || c == '_'))
}

/// Each reference `prefix<number>` in `text` with the far end of a range when one follows: an
/// ellipsis (`…` or `...`) or an en dash, then the prefix again or a bare number.
fn references(text: &str, prefix: &str, suffixes: bool) -> Vec<(EpId, Option<EpId>)> {
    let scan = |at: usize| -> Option<(EpId, usize)> {
        if suffixes {
            scan_ep_id(text, at)
        } else {
            let (number, end) = scan_number(text, at, prefix)?;
            let next = text[end..].chars().next();
            (!next.is_some_and(char::is_alphanumeric)).then_some((EpId::new(number), end))
        }
    };
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(offset) = text[at..].find(prefix) {
        let start = at + offset;
        at = start + prefix.len();
        if !starts_word(text, start) {
            continue;
        }
        let Some((from, end)) = scan(start) else {
            continue;
        };
        at = end;
        let rest = text[end..].trim_start();
        let after_dash = ["…", "...", "–"]
            .iter()
            .find_map(|dash| rest.strip_prefix(dash))
            .map(str::trim_start);
        let to = after_dash.and_then(|after| {
            let after_at = text.len() - after.len();
            let (to, to_end) = if after.starts_with(prefix) {
                scan(after_at)?
            } else {
                let digits = after.bytes().take_while(u8::is_ascii_digit).count();
                let number = after.get(..digits)?.parse().ok()?;
                let next = after[digits..].chars().next();
                if next.is_some_and(char::is_alphanumeric) {
                    return None;
                }
                (EpId::new(number), after_at + digits)
            };
            at = to_end;
            Some(to)
        });
        found.push((from, to));
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep(number: u32) -> EpId {
        EpId::new(number)
    }

    #[test]
    fn reads_and_writes_ep_ids() {
        assert_eq!("EP-12".parse(), Ok(ep(12)));
        assert_eq!(
            "EP-4a".parse(),
            Ok(EpId {
                number: 4,
                suffix: Some('a')
            })
        );
        assert_eq!("EP-007".parse::<EpId>().unwrap().to_string(), "EP-7");
        for bad in [
            "EP-", "EP-x", "ep-3", "EP-3 ", " EP-3", "EP-4ab", "EP-4A", "EP-3.", "D-3",
        ] {
            assert!(bad.parse::<EpId>().is_err(), "{bad:?}");
        }
        let split = EpId {
            number: 4,
            suffix: Some('b'),
        };
        assert_eq!(split.to_string(), "EP-4b");
        assert!(ep(4) < split && split < ep(5));
    }

    #[test]
    fn a_size_cell_ends_on_its_last_size() {
        for (cell, size) in [
            ("M", Some(Size::M)),
            (" S ", Some(Size::S)),
            ("S → L (re-scoped 2026-10-29)", Some(Size::L)),
            ("S→M (owner escalated retirements)", Some(Size::M)),
            ("S/M", Some(Size::M)),
            ("L (an M before it was split)", Some(Size::L)),
            ("XL", None),
            ("", None),
            ("1 session", None),
        ] {
            assert_eq!(Size::last_in(cell), size, "{cell:?}");
        }
        assert_eq!(Size::L.to_string(), "L");
    }

    #[test]
    fn finds_ep_ids_and_expands_ranges() {
        let ids = |text| ep_ids(text).iter().map(EpId::to_string).collect::<Vec<_>>();
        assert_eq!(ids("EP-1, EP-2"), ["EP-1", "EP-2"]);
        assert_eq!(
            ids("EP-6 … EP-9, EP-14"),
            ["EP-6", "EP-7", "EP-8", "EP-9", "EP-14"]
        );
        assert_eq!(ids("EP-0...EP-2"), ["EP-0", "EP-1", "EP-2"]);
        assert_eq!(ids("EP-26–28"), ["EP-26", "EP-27", "EP-28"]);
        assert_eq!(ids("EP-1–EP-3 cover M0–M2"), ["EP-1", "EP-2", "EP-3"]);
        // Split packets, repeats, and words that only look like ids.
        assert_eq!(
            ids("EP-9 (checkpoint), EP-4b, EP-9; EP-4a … EP-4b"),
            ["EP-9", "EP-4b", "EP-4a"]
        );
        assert_eq!(ids("STEP-3, EP-3xy, xEP-4, EP-5"), ["EP-5"]);
        assert_eq!(ids("everything"), Vec::<String>::new());
        // A range written backwards or too long keeps its two ends.
        assert_eq!(ids("EP-9 … EP-7"), ["EP-9", "EP-7"]);
        assert_eq!(ids("EP-1 … EP-999"), ["EP-1", "EP-999"]);
        // An em dash separates clauses; it is not a range.
        assert_eq!(ids("EP-8 — EP-10"), ["EP-8", "EP-10"]);
    }

    #[test]
    fn finds_decision_references() {
        assert_eq!(decisions("(D-2, D-14, D-15, D-29)"), [2, 14, 15, 29]);
        assert_eq!(decisions("(D-1 … D-4)"), [1, 2, 3, 4]);
        assert_eq!(decisions("D-8 … D-10; D-9"), [8, 9, 10]);
        // Domain ids, words ending in D and addenda are not decision numbers.
        assert_eq!(decisions("D-PROD-001, CMD-3, AND-4, D-7's addendum"), [7]);
        assert!(decisions("no decisions here").is_empty());
    }

    #[test]
    fn finds_the_first_date() {
        assert_eq!(
            first_date("Completion note (2026-10-31; opus/max; ≈ 50 min)"),
            Some("2026-10-31")
        );
        assert_eq!(first_date("2026-10-31"), Some("2026-10-31"));
        assert_eq!(
            first_date("filled 2026-08-23 after 2026-08-22"),
            Some("2026-08-23")
        );
        for none in [
            "",
            "no date",
            "12026-10-31",
            "2026-10-311",
            "2026-1-31",
            "YYYY-MM-DD",
        ] {
            assert_eq!(first_date(none), None, "{none:?}");
        }
    }

    #[test]
    fn the_readme_lives_in_the_roadmap_folder() {
        assert_eq!(
            readme(Path::new(r"C:\Data\alpha")),
            Path::new(r"C:\Data\alpha\roadmap\README.md")
        );
    }
}
