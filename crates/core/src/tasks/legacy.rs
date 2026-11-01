//! The owner's to-dos in the forms written before `owner-tasks.md` (D-10), read-only and never
//! rewritten: a README's `## Risks …` and `## Open items …` lists (numbered or bulleted, struck
//! through when resolved), its `Open items for the owner` list, its `**Owner actions outside any
//! brief**` paragraph; the `owner-decisions.md` and `open-questions.md` registers; the
//! `## Risks / notes for the next session` bullets of the latest completed EP's handoff file; and
//! the `Parked` and `Handed on` blocks of the latest completed brief. Older briefs and handoff
//! files keep theirs: the sessions after them have read and folded them in.

use std::fs;
use std::io;
use std::path::Path;

use super::{Form, Gate, OwnerItem, State, before};
use crate::roadmap::brief::{Brief, Kind};
use crate::roadmap::markdown::{Block, Line, blocks, cell_texts, dedent, lines};
use crate::roadmap::snapshot::Entry;
use crate::roadmap::table::Done;
use crate::roadmap::{EpId, ep_ids, first_date, readme};

/// The register of rulings only the owner can give (the packet style).
pub const DECISIONS_FILE: &str = "owner-decisions.md";

/// The register of open questions and deferred items (the packet style).
pub const QUESTIONS_FILE: &str = "open-questions.md";

/// The older forms of `folder`: the README's (from `readme_text`), the two registers in
/// `roadmap/`, and the latest completed EP's handoff notes and blocks. Files that exist and
/// cannot be read are returned as problems.
pub fn read(
    folder: &Path,
    readme_text: Option<&str>,
    latest: Option<&Entry>,
) -> (Vec<OwnerItem>, Vec<String>) {
    let mut items = Vec::new();
    let mut problems = Vec::new();
    if let Some(text) = readme_text {
        items.extend(from_readme(text, &readme(folder)));
    }
    for (name, form) in [
        (DECISIONS_FILE, Form::OwnerDecision),
        (QUESTIONS_FILE, Form::OpenQuestion),
    ] {
        let path = folder.join("roadmap").join(name);
        match fs::read_to_string(&path) {
            Ok(text) => items.extend(from_register(&text, &path, form)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => problems.push(format!("{}: {e}", path.display())),
        }
    }
    if let Some(entry) = latest {
        if let Some(handoff) = &entry.handoff {
            match fs::read_to_string(&handoff.path) {
                Ok(text) => items.extend(from_handoff(&text, &handoff.path)),
                Err(e) => problems.push(format!("{}: {e}", handoff.path.display())),
            }
        }
        items.extend(from_brief(&entry.brief, entry.file.id, &entry.file.path));
    }
    (items, problems)
}

/// Whether a `Handed on (to …)` target is the owner rather than a project: such a block is an
/// owner task written before `owner-tasks.md`, not mail.
pub fn is_owner(target: &str) -> bool {
    matches!(
        target.trim().to_ascii_lowercase().as_str(),
        "the owner" | "owner" | "the project owner"
    )
}

/// A README's risks lists, its open-items list and its owner-actions paragraph.
pub fn from_readme(text: &str, path: &Path) -> Vec<OwnerItem> {
    let all = lines(text);
    let mut found = Vec::new();
    // Inside a `## Risks …` or `## Open items …` section, up to the next heading.
    let mut risks = false;
    // Right after an `Open items for the owner` paragraph, up to the next block of another kind.
    let mut open_items = false;
    for block in blocks(&all) {
        match block {
            Block::Heading { text, .. } => {
                let lower = text.to_ascii_lowercase();
                risks = lower.starts_with("risks") || lower.starts_with("open items");
                open_items = false;
            }
            Block::Item { lines, indent } => {
                let (number, content) = item_text(&lines, indent);
                let item = |form| OwnerItem {
                    form,
                    path: path.to_owned(),
                    line: lines[0].number,
                    id: number.clone(),
                    text: content.clone(),
                    gate: None,
                    state: State::struck(&content),
                    raised: None,
                    date: None,
                };
                if open_items {
                    let mut open = item(Form::OpenItem);
                    open.gate = on_or_after(&content);
                    found.push(open);
                } else if risks {
                    let mut risk = item(Form::Risk);
                    if let Some(id) = bold_id(&content) {
                        risk.id = Some(id);
                    }
                    found.push(risk);
                }
            }
            Block::Paragraph { lines } => {
                let lead = lines[0].text.trim_start().trim_start_matches(['*', '_']);
                open_items = starts_with_ignoring_case(lead, "Open items for the owner");
                if lines[0]
                    .text
                    .trim_start()
                    .starts_with("**Owner actions outside any brief")
                {
                    found.extend(owner_actions(&lines, path));
                }
            }
            Block::Table { .. } | Block::Quote { .. } => open_items = false,
        }
    }
    found
}

/// A list item's number (`3` for `3.`, nothing for a bullet) and its text: the first line
/// without its marker, continuation lines without the item's indentation.
fn item_text(lines: &[Line], indent: usize) -> (Option<String>, String) {
    let first = lines[0].text;
    let marker = first
        .trim_start()
        .split([' ', '\t'])
        .next()
        .unwrap_or_default();
    let number = marker
        .strip_suffix(['.', ')'])
        .filter(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
        .map(str::to_owned);
    // The marker and the spaces after it are ASCII, so the column is the byte.
    let mut text = first.get(indent..).unwrap_or_default().trim().to_owned();
    for line in &lines[1..] {
        text.push('\n');
        text.push_str(dedent(line.text, indent).trim_end());
    }
    (number, text)
}

/// The id in bold opening an item, inside a strike-through or not: `**R-3**` → `R-3`.
fn bold_id(text: &str) -> Option<String> {
    let rest = text
        .trim_start()
        .trim_start_matches('~')
        .strip_prefix("**")?;
    let inner = &rest[..rest.find("**")?];
    is_item_id(inner).then(|| inner.to_owned())
}

/// `R-3`, `OD-12`, `OQ-B`: one to four capitals, a dash, then letters or digits.
fn is_item_id(text: &str) -> bool {
    let Some((letters, rest)) = text.split_once('-') else {
        return false;
    };
    (1..=4).contains(&letters.len())
        && letters.bytes().all(|b| b.is_ascii_uppercase())
        && !rest.is_empty()
        && rest.bytes().all(|b| b.is_ascii_alphanumeric())
}

/// `On or after 2026-11-02, once …:` opening an open item.
fn on_or_after(text: &str) -> Option<Gate> {
    if !starts_with_ignoring_case(text, "on or after") {
        return None;
    }
    let lead = text.lines().next().unwrap_or_default();
    first_date(lead).map(|date| Gate::OnOrAfter(date.to_owned()))
}

/// The actions of a `**Owner actions outside any brief** (…): a (D-1); b (D-2, before EP-5).`
/// paragraph: the text after the first colon outside the label and its parentheses, split at
/// each semicolon outside parentheses, each action on the line it starts on.
fn owner_actions(lines: &[Line], path: &Path) -> Vec<OwnerItem> {
    let mut text = String::new();
    let mut starts = Vec::new();
    for line in lines {
        if !text.is_empty() {
            text.push('\n');
        }
        starts.push((text.len(), line.number));
        text.push_str(line.text.trim());
    }
    let line_of = |byte: usize| {
        starts
            .iter()
            .rev()
            .find(|&&(start, _)| start <= byte)
            .map_or(lines[0].number, |&(_, number)| number)
    };
    let label_end = text[2..].find("**").map_or(0, |at| at + 4);
    let mut pieces = Vec::new();
    let mut depth = 0usize;
    // A label may carry its own colon: `**Owner actions outside any brief:** a; b.`
    let mut from = text[..label_end]
        .trim_end_matches('*')
        .ends_with(':')
        .then_some(label_end);
    for (i, c) in text.char_indices().skip_while(|&(i, _)| i < label_end) {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ':' if depth == 0 && from.is_none() => from = Some(i + 1),
            ';' if depth == 0 => {
                if let Some(start) = from {
                    pieces.push(start..i);
                    from = Some(i + 1);
                }
            }
            _ => {}
        }
    }
    if let Some(start) = from {
        pieces.push(start..text.len());
    }
    pieces
        .into_iter()
        .filter_map(|range| {
            let raw = &text[range.clone()];
            let piece = raw.trim().trim_end_matches('.').trim_end();
            let start = range.start + (raw.len() - raw.trim_start().len());
            (!piece.is_empty()).then(|| OwnerItem {
                form: Form::OwnerAction,
                path: path.to_owned(),
                line: line_of(start),
                id: None,
                text: piece.replace('\n', " "),
                gate: before(piece).map(Gate::Before),
                state: State::struck(piece),
                raised: None,
                date: None,
            })
        })
        .collect()
}

/// The rows of a register's tables whose first column is `#` or `ID` and whose first cell is an
/// item id (`OD-3`, `OQ-B`), read by column name: the text from `Ruling needed`, `Item` or
/// `Question`; done from a `Done` cell's `☑`, or from words (`**Resolved`, `**Closed`,
/// `**Decided`, `(closed)`); the gate from the decision point; the raising EP from `Raised by`.
pub fn from_register(text: &str, path: &Path, form: Form) -> Vec<OwnerItem> {
    let all = lines(text);
    let mut found = Vec::new();
    for block in blocks(&all) {
        let Block::Table { header, rows } = block else {
            continue;
        };
        let names: Vec<String> = cell_texts(header.text)
            .iter()
            .map(|name| name.trim_matches(['*', '`']).trim().to_ascii_lowercase())
            .collect();
        if !matches!(names.first().map(String::as_str), Some("#" | "id")) {
            continue;
        }
        let column = |wanted: &[&str]| {
            names
                .iter()
                .position(|name| wanted.iter().any(|w| name.starts_with(w)))
        };
        let text_at = column(&["ruling needed", "item", "question"]).unwrap_or(1);
        let done_at = column(&["done"]);
        let point_at = column(&["latest responsible decision point", "decision point"]);
        let raised_at = column(&["raised by"]);
        for row in rows {
            let cells = cell_texts(row.text);
            let id = cells
                .first()
                .map_or("", |cell| cell.trim_matches(['*', '`']));
            if !is_item_id(id) {
                continue;
            }
            let cell = |at: Option<usize>| at.and_then(|at| cells.get(at)).copied();
            let state = match cell(done_at).map(Done::parse) {
                Some(done) if done.is_checked() => State::Resolved,
                Some(Done::Open) => State::Open,
                _ if cells.iter().any(|cell| closed_in_words(cell)) => State::Resolved,
                _ => State::Open,
            };
            found.push(OwnerItem {
                form,
                path: path.to_owned(),
                line: row.number,
                id: Some(id.to_owned()),
                text: cell(Some(text_at)).unwrap_or_default().to_owned(),
                gate: cell(point_at).and_then(decision_point),
                state,
                raised: cell(raised_at).and_then(|by| ep_ids(by).first().copied()),
                date: None,
            });
        }
    }
    found
}

/// A register's cell that says its row is closed.
fn closed_in_words(cell: &str) -> bool {
    let lower = cell.to_ascii_lowercase();
    ["**resolved", "**closed", "**decided", "(closed)"]
        .iter()
        .any(|word| lower.contains(word))
}

/// A decision-point cell as a gate: an EP (`EP-8`, `before EP-4b`), else the words; nothing for
/// `—` or a closed row.
fn decision_point(cell: &str) -> Option<Gate> {
    let text = cell.trim();
    if text.is_empty() || matches!(text, "—" | "–" | "-") || closed_in_words(text) {
        return None;
    }
    if let Some(id) = before(text) {
        return Some(Gate::Before(id));
    }
    match text.parse::<EpId>() {
        Ok(id) => Some(Gate::Before(id)),
        Err(_) => Some(Gate::When(text.to_owned())),
    }
}

/// The bullets under a handoff file's `## Risks / notes …` heading.
pub fn from_handoff(text: &str, path: &Path) -> Vec<OwnerItem> {
    let all = lines(text);
    let mut found = Vec::new();
    let mut notes = false;
    for block in blocks(&all) {
        match block {
            Block::Heading { text, .. } => {
                notes = starts_with_ignoring_case(text, "Risks / notes");
            }
            Block::Item { lines, indent } if notes => {
                let (_, content) = item_text(&lines, indent);
                found.push(OwnerItem {
                    form: Form::HandoffNote,
                    path: path.to_owned(),
                    line: lines[0].number,
                    id: None,
                    state: State::struck(&content),
                    text: content,
                    gate: None,
                    raised: None,
                    date: None,
                });
            }
            _ => {}
        }
    }
    found
}

/// The `Parked`, `Handed on` and `Handed on (to the owner)` blocks of brief `id`; a block handed
/// on to a project is mail (`inbox::outgoing`), not an owner item.
pub fn from_brief(brief: &Brief, id: EpId, path: &Path) -> Vec<OwnerItem> {
    brief
        .quotes
        .iter()
        .filter_map(|quote| {
            let form = match &quote.kind {
                Kind::Parked => Form::Parked,
                Kind::HandedOn { to: None } => Form::HandedOn,
                Kind::HandedOn { to: Some(to) } if is_owner(to) => Form::HandedOn,
                _ => return None,
            };
            Some(OwnerItem {
                form,
                path: path.to_owned(),
                line: quote.line,
                id: None,
                text: quote.text.clone(),
                gate: None,
                state: State::Open,
                raised: Some(id),
                date: quote.date.clone(),
            })
        })
        .collect()
}

fn starts_with_ignoring_case(text: &str, prefix: &str) -> bool {
    text.get(..prefix.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roadmap::brief;

    fn ep(number: u32) -> EpId {
        EpId::new(number)
    }

    fn summary(items: &[OwnerItem]) -> Vec<(Form, Option<&str>, State, usize)> {
        items
            .iter()
            .map(|item| (item.form, item.id.as_deref(), item.state, item.line))
            .collect()
    }

    const README: &str = "\
# alpha — Roadmap

**Owner actions outside any brief** (the roadmap assumes them; see D-1): a hardware key on the
account that owns the remote (D-2); a second disk for the backups (D-3, before EP-4); ~~the
licence read through~~ (D-1); a monthly look at the logs.

## Phase P1 — Build

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | One | S | — | core | ☐ |

## Maintenance

Every row is checked.

Open items for the owner, with the detail in the last note:

- Now: delete the scratch folder.
- On or after 2026-11-02, once the clocks change: the job starts at 01:00.
- Optional: confirm two records.

Notes from other projects below this line.

- Not an open item: the list above ended at the paragraph.

## Risks / open items

1. ~~The podium overhangs.~~ **Resolved by EP-2** (2026-08-12): the plate follows the wings.
   New open items:
   - **Bed count**: twelve beds modelled.
2. Bridge levels are inferred.
3. Speeds: ~~the pitch shifts~~ **resolved at EP-3**; the long takes drift.

## Risks (struck through when resolved)

- **R-1** A header the parser has never seen.
- ~~**R-2** Backups fill the disk.~~ **Resolved by EP-2 (2026-10-03):** fourteen copies.

## Decision record

1. Not a risk: a numbered decision.
";

    #[test]
    fn reads_a_readmes_risks_open_items_and_owner_actions() {
        let items = from_readme(README, Path::new("README.md"));
        assert_eq!(
            summary(&items),
            [
                (Form::OwnerAction, None, State::Open, 3),
                (Form::OwnerAction, None, State::Open, 4),
                (Form::OwnerAction, None, State::Resolved, 4),
                (Form::OwnerAction, None, State::Open, 5),
                (Form::OpenItem, None, State::Open, 19),
                (Form::OpenItem, None, State::Open, 20),
                (Form::OpenItem, None, State::Open, 21),
                (Form::Risk, Some("1"), State::Resolved, 29),
                (Form::Risk, Some("2"), State::Open, 32),
                (Form::Risk, Some("3"), State::Partly, 33),
                (Form::Risk, Some("R-1"), State::Open, 37),
                (Form::Risk, Some("R-2"), State::Resolved, 38),
            ]
        );
        let texts: Vec<&str> = items[..4].iter().map(|i| i.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "a hardware key on the account that owns the remote (D-2)",
                "a second disk for the backups (D-3, before EP-4)",
                "~~the licence read through~~ (D-1)",
                "a monthly look at the logs",
            ]
        );
        assert_eq!(items[1].gate, Some(Gate::Before(ep(4))));
        assert_eq!(items[0].gate, None);
        assert_eq!(items[4].text, "Now: delete the scratch folder.");
        assert_eq!(
            items[5].gate,
            Some(Gate::OnOrAfter("2026-11-02".to_owned()))
        );
        assert_eq!(
            items[7].text,
            "~~The podium overhangs.~~ **Resolved by EP-2** (2026-08-12): the plate follows the wings.\nNew open items:\n- **Bed count**: twelve beds modelled."
        );
        assert!(items.iter().all(|item| !item.is_tickable()));
    }

    #[test]
    fn reads_the_two_registers_by_column_name() {
        let decisions = "\
# pending owner decisions

| # | Ruling needed | Raised by | Class | Ruling lands in | Done |
|---|---|---|---|---|---|
| OD-1 | Is the board setting **on**? | EP-4 dev. 3 | drift | an addendum | ☑ 2026-09-01 — performed |
| OD-2 | Does the block match the draft? | EP-3 dev. 1 | confirmation | an addendum | ☐ **open — owner only** |
| OD-8 | Confirm the form renders | EP-4 dev. 2 | verification | EP-8 | ☐ pushed 2026-09-01; **not observed** |

### OD-13

| Brief | # | Criterion |
|---|---|---|
| EP-1 | 10 | Reads in one screen |

| File | Reason |
|---|---|
| `SAFETY.md` | the block |
";
        let items = from_register(
            decisions,
            Path::new("owner-decisions.md"),
            Form::OwnerDecision,
        );
        assert_eq!(
            summary(&items),
            [
                (Form::OwnerDecision, Some("OD-1"), State::Resolved, 5),
                (Form::OwnerDecision, Some("OD-2"), State::Open, 6),
                (Form::OwnerDecision, Some("OD-8"), State::Open, 7),
            ]
        );
        assert_eq!(items[0].text, "Is the board setting **on**?");
        assert_eq!(items[0].raised, Some(ep(4)));
        assert_eq!(items[1].gate, None);

        let questions = "\
| ID | Item | Owner | Latest responsible decision point | Effect if unresolved |
|---|---|---|---|---|
| OQ-A | **Resolved 2026-09-02.** The licence reply. | owner | — (closed) | — |
| OQ-B | A colleague's review | owner | M7 harm review | Narrative held out |
| OQ-H | Deploy the page early? | owner | EP-8 | Not deployed. **Decided 2026-09-02 at EP-8a: no** |
| OQ-I | Reliability conventions | author + owner | first M5 packet, before EP-12 | Stand as published |
| Not an id | x | y | z | w |
";
        let items = from_register(
            questions,
            Path::new("open-questions.md"),
            Form::OpenQuestion,
        );
        let gates: Vec<_> = items
            .iter()
            .map(|item| (item.id.as_deref(), item.state, item.gate.clone()))
            .collect();
        assert_eq!(
            gates,
            [
                (Some("OQ-A"), State::Resolved, None),
                (
                    Some("OQ-B"),
                    State::Open,
                    Some(Gate::When("M7 harm review".to_owned()))
                ),
                (Some("OQ-H"), State::Resolved, Some(Gate::Before(ep(8)))),
                (Some("OQ-I"), State::Open, Some(Gate::Before(ep(12)))),
            ]
        );
        assert_eq!(items[1].text, "A colleague's review");
        assert!(is_item_id("OQ-1") && is_item_id("R-12") && !is_item_id("ABCDE-1"));
        assert!(!is_item_id("od-1") && !is_item_id("OD-") && !is_item_id("OD1"));
    }

    #[test]
    fn reads_handoff_notes_and_the_blocks_of_a_brief() {
        let handoff = "\
# EP-4 handoff

**Status:** complete · **Date:** 2026-09-02

## Risks / notes for the next session

- The freeze flag must come from EP-9.
- ~~The binary is end-of-life~~ **settled:** it keeps working.

## Next eligible EPs

- EP-5 is eligible now.
";
        let items = from_handoff(handoff, Path::new("EP-4.md"));
        assert_eq!(
            summary(&items),
            [
                (Form::HandoffNote, None, State::Open, 7),
                (Form::HandoffNote, None, State::Resolved, 8),
            ]
        );

        let text = "\
# EP-2 — Registry

> **Completion note (2026-10-03).** One session.

> **Handed on (to beta).** Mail for a project.
> **Handed on (to the owner).** Start the next EP by hand.
> **Handed on.** For the sessions after this one.
> **Parked (2026-10-04).** A later brief.
";
        let items = from_brief(&brief::parse(text), ep(2), Path::new("EP-2-registry.md"));
        let kinds: Vec<_> = items
            .iter()
            .map(|item| (item.form, item.line, item.raised, item.date.as_deref()))
            .collect();
        assert_eq!(
            kinds,
            [
                (Form::HandedOn, 6, Some(ep(2)), None),
                (Form::HandedOn, 7, Some(ep(2)), None),
                (Form::Parked, 8, Some(ep(2)), Some("2026-10-04")),
            ]
        );
        assert!(items[0].text.starts_with("**Handed on (to the owner).**"));
        assert!(is_owner("The Owner") && !is_owner("beta"));
    }
}
