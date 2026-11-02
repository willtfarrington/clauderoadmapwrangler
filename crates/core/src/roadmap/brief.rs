//! A brief file, `roadmap/EP-N-<slug>.md`, read in the house style and the two older ones (D-9,
//! D-12) and never written (D-11): the file names, the YAML front matter, the title, the header
//! line of bold fields (`**Size:** M · **Depends on:** EP-1 · **Blocks:** …`), the labelled
//! blockquote blocks (completion notes, `☑ COMPLETE`, handed on, parked, pickup notes, charters,
//! the pickup gate), the heading-form markers (`## Completion note`, `## Handoff payload`), the
//! packet style's `docs/handoffs/EP-N.md`, and the one question the dashboard asks of a brief:
//! is it complete, and on what evidence. Nothing here panics on a malformed file.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::markdown::{Block, Line, blocks, lines, quoted};
use super::table::{Done, hashes};
use super::{EpId, Size, ep_ids, first_date, scan_ep_id};

// --- Files --------------------------------------------------------------------------------------

/// A brief file by name: `EP-4-parser-b.md` → EP-4 and the slug `parser-b`. `_TEMPLATE.md` and
/// the other files of the folder are not briefs.
pub fn file_id(name: &str) -> Option<(EpId, &str)> {
    let stem = name.strip_suffix(".md")?;
    let (id, end) = scan_ep_id(stem, 0)?;
    let slug = stem.get(end..)?.strip_prefix('-')?;
    let plain = !slug.is_empty()
        && slug
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    plain.then_some((id, slug))
}

/// Whether a brief-shaped name is a companion of the brief rather than the brief itself
/// (`EP-9-completion-handoff.md`, `EP-9-completion-report.md`).
pub fn is_companion(slug: &str) -> bool {
    slug.starts_with("completion-")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BriefFile {
    pub id: EpId,
    pub name: String,
    pub path: PathBuf,
}

/// The brief files of a `roadmap/` folder.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Files {
    /// One brief per id, by id then name.
    pub briefs: Vec<BriefFile>,
    /// Files beside a brief that are not it (`EP-n-completion-*.md`).
    pub companions: Vec<BriefFile>,
    /// A second brief file for an id, hidden by the first.
    pub duplicates: Vec<BriefFile>,
}

impl Files {
    pub fn brief(&self, id: EpId) -> Option<&BriefFile> {
        self.briefs.iter().find(|file| file.id == id)
    }
}

/// The brief files in `roadmap_dir`; an absent folder holds none.
pub fn discover(roadmap_dir: &Path) -> io::Result<Files> {
    let entries = match fs::read_dir(roadmap_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Files::default()),
        Err(e) => return Err(e),
    };
    let mut found: Vec<(BriefFile, bool)> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let (id, slug) = file_id(&name)?;
            let companion = is_companion(slug);
            Some((
                BriefFile {
                    id,
                    path: entry.path(),
                    name,
                },
                companion,
            ))
        })
        .collect();
    found.sort_by(|a, b| a.0.id.cmp(&b.0.id).then_with(|| a.0.name.cmp(&b.0.name)));
    let mut files = Files::default();
    for (file, companion) in found {
        if companion {
            files.companions.push(file);
        } else if files.brief(file.id).is_some() {
            files.duplicates.push(file);
        } else {
            files.briefs.push(file);
        }
    }
    Ok(files)
}

// --- The brief ----------------------------------------------------------------------------------

/// A brief as the parser reads it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Brief {
    pub front: Option<FrontMatter>,
    /// The first `# ` heading.
    pub title: Option<String>,
    /// The header line's bold fields, in order, from every such line above the first `##`.
    pub fields: Vec<Field>,
    pub headings: Vec<Heading>,
    /// The labelled blockquote blocks, in order.
    pub quotes: Vec<Quote>,
    /// A `## Handoff payload` section (the packet style).
    pub handoff: Option<Handoff>,
}

/// YAML front matter (D-12): flat `key: value` lines between `---` fences on line 1.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrontMatter {
    pub fields: Vec<Field>,
    /// The lines it takes, both fences included.
    pub lines: usize,
}

/// A `**Label:** value` field of the header line, or a `key: value` line of the front matter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: usize,
    pub text: String,
    pub line: usize,
}

/// A blockquote block that opens with a bold label, such as `> **Completion note (…).** …`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quote {
    pub kind: Kind,
    /// The label as written, without the asterisks and the closing punctuation.
    pub label: String,
    /// The first date in the label.
    pub date: Option<String>,
    /// The block's text without the `>` markers, the label included.
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Kind {
    /// `**Completion note (date).**`, with model, effort and duration in the new form.
    Completion(Completion),
    /// `**☑ COMPLETE (date).**`
    Complete,
    /// `**Handed on.**`, `**Handed on (to <project>).**`, `**Parked and handed on.**`
    HandedOn { to: Option<String> },
    /// `**Parked.**`
    Parked,
    /// `**EP-n pickup note (date).**`
    PickupNote { ep: Option<EpId> },
    /// `**Charter.**`
    Charter,
    /// `**PICKUP GATE — owner input required … (EP-n, date).**` (D-6's fact).
    PickupGate,
    /// `**Owner decisions (…).**` and its wordings.
    OwnerDecisions,
    /// A block that opens with a bold label of another kind, or with none.
    #[default]
    Other,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Completion(completion) => {
                f.write_str("completion note")?;
                match (&completion.model, &completion.effort) {
                    (Some(model), Some(effort)) => {
                        write!(f, " ({model}/{effort}")?;
                        if let Some(duration) = &completion.duration {
                            write!(f, "; {duration}")?;
                        }
                        f.write_str(")")
                    }
                    _ => match &completion.suffix {
                        Some(suffix) => write!(f, " ({suffix})"),
                        None => Ok(()),
                    },
                }
            }
            Kind::Complete => f.write_str("☑ COMPLETE"),
            Kind::HandedOn { to: Some(to) } => write!(f, "handed on to {to}"),
            Kind::HandedOn { to: None } => f.write_str("handed on"),
            Kind::Parked => f.write_str("parked"),
            Kind::PickupNote { ep: Some(ep) } => write!(f, "{ep} pickup note"),
            Kind::PickupNote { ep: None } => f.write_str("pickup note"),
            Kind::Charter => f.write_str("charter"),
            Kind::PickupGate => f.write_str("pickup gate"),
            Kind::OwnerDecisions => f.write_str("owner decisions"),
            Kind::Other => f.write_str("other"),
        }
    }
}

/// What a completion note's parenthesis says beyond the date: `(2026-10-31; fable/xhigh; 1 h 20
/// min)` in the D-12 form, or a suffix such as `EP-28 verification` in the old one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Completion {
    pub model: Option<String>,
    pub effort: Option<String>,
    pub duration: Option<String>,
    pub suffix: Option<String>,
}

/// A `## Handoff payload (filled date)` or `(fill at session end)` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handoff {
    pub filled: bool,
    pub date: Option<String>,
    pub line: usize,
}

/// Why a brief counts as complete, the strongest evidence first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evidence {
    CompletionNote {
        date: Option<String>,
        line: usize,
    },
    Complete {
        date: Option<String>,
        line: usize,
    },
    CompletionHeading {
        date: Option<String>,
        line: usize,
    },
    /// A `**Status:**` field that reads done: `DONE`, `complete …` or `[x] hash`.
    Status {
        value: String,
    },
    HandoffPayload {
        date: Option<String>,
        line: usize,
    },
    /// Front matter `status: done`.
    FrontMatter {
        status: String,
    },
}

impl fmt::Display for Evidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dated = |f: &mut fmt::Formatter<'_>, what: &str, date: &Option<String>, line| match date
        {
            Some(date) => write!(f, "{what} ({date}, line {line})"),
            None => write!(f, "{what} (line {line})"),
        };
        match self {
            Evidence::CompletionNote { date, line } => dated(f, "completion note", date, line),
            Evidence::Complete { date, line } => dated(f, "☑ COMPLETE", date, line),
            Evidence::CompletionHeading { date, line } => {
                dated(f, "a completion-note heading", date, line)
            }
            Evidence::Status { value } => write!(f, "status {value}"),
            Evidence::HandoffPayload { date, line } => {
                dated(f, "a filled handoff payload", date, line)
            }
            Evidence::FrontMatter { status } => write!(f, "front matter status {status}"),
        }
    }
}

impl FrontMatter {
    /// The value of `key`, in any case.
    pub fn get(&self, key: &str) -> Option<&str> {
        field(&self.fields, key)
    }

    /// The EP the brief is for: `ep: 3`, `ep: 7a` (a lettered row, EP-7a) or `ep: EP-7a`.
    pub fn ep(&self) -> Option<EpId> {
        let value = self.get("ep")?.trim();
        value
            .parse()
            .ok()
            .or_else(|| format!("EP-{value}").parse().ok())
    }

    pub fn size(&self) -> Option<Size> {
        Size::last_in(self.get("size")?)
    }

    pub fn status(&self) -> Option<&str> {
        self.get("status")
    }

    pub fn model(&self) -> Option<&str> {
        self.get("model")
    }

    pub fn effort(&self) -> Option<&str> {
        self.get("effort")
    }
}

fn field<'a>(fields: &'a [Field], label: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|field| field.label.eq_ignore_ascii_case(label))
        .map(|field| field.value.as_str())
}

impl Brief {
    /// The header line's field `label`, in any case.
    pub fn field(&self, label: &str) -> Option<&str> {
        field(&self.fields, label)
    }

    /// The size the front matter gives, else the `**Size:**` field.
    pub fn size(&self) -> Option<Size> {
        self.front
            .as_ref()
            .and_then(FrontMatter::size)
            .or_else(|| Size::last_in(self.field("Size")?))
    }

    /// The ids the `**Depends on:**` field names.
    pub fn depends(&self) -> Vec<EpId> {
        ep_ids(self.field("Depends on").unwrap_or_default())
    }

    /// The ids the `**Blocks:**` field names.
    pub fn blocks(&self) -> Vec<EpId> {
        ep_ids(self.field("Blocks").unwrap_or_default())
    }

    pub fn status(&self) -> Option<&str> {
        self.field("Status")
    }

    pub fn completion_notes(&self) -> impl Iterator<Item = &Quote> {
        self.quotes
            .iter()
            .filter(|quote| matches!(quote.kind, Kind::Completion(_)))
    }

    pub fn handed_on(&self) -> impl Iterator<Item = &Quote> {
        self.quotes
            .iter()
            .filter(|quote| matches!(quote.kind, Kind::HandedOn { .. }))
    }

    pub fn pickup_gate(&self) -> Option<&Quote> {
        self.quotes
            .iter()
            .find(|quote| quote.kind == Kind::PickupGate)
    }

    /// The first heading whose text starts with `prefix`, in any case.
    pub fn heading(&self, prefix: &str) -> Option<&Heading> {
        self.headings
            .iter()
            .find(|heading| starts_with_ignoring_case(&heading.text, prefix))
    }

    /// The evidence that this brief is complete, when there is some: a completion note, `☑
    /// COMPLETE`, a completion-note heading, a done status, a filled handoff payload, or front
    /// matter `status: done`.
    pub fn completion(&self) -> Option<Evidence> {
        if let Some(note) = self.completion_notes().next() {
            return Some(Evidence::CompletionNote {
                date: note.date.clone(),
                line: note.line,
            });
        }
        if let Some(quote) = self.quotes.iter().find(|q| q.kind == Kind::Complete) {
            return Some(Evidence::Complete {
                date: quote.date.clone(),
                line: quote.line,
            });
        }
        if let Some(heading) = self
            .headings
            .iter()
            .find(|h| h.level >= 2 && starts_with_ignoring_case(&h.text, "completion note"))
        {
            return Some(Evidence::CompletionHeading {
                date: first_date(&heading.text).map(str::to_owned),
                line: heading.line,
            });
        }
        if let Some(value) = self.status().filter(|value| status_is_done(value)) {
            return Some(Evidence::Status {
                value: value.to_owned(),
            });
        }
        if let Some(handoff) = self.handoff.as_ref().filter(|h| h.filled) {
            return Some(Evidence::HandoffPayload {
                date: handoff.date.clone(),
                line: handoff.line,
            });
        }
        self.front
            .as_ref()
            .and_then(FrontMatter::status)
            .filter(|status| status.trim().eq_ignore_ascii_case("done"))
            .map(|status| Evidence::FrontMatter {
                status: status.to_owned(),
            })
    }

    pub fn is_complete(&self) -> bool {
        self.completion().is_some()
    }
}

/// Whether a `**Status:**` value says done: `DONE (…)`, `complete — …`, `[x] hash (done date)`;
/// not `split …`, `deferred …` or `[ ]`.
pub fn status_is_done(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    Done::parse(value).is_checked()
        || ["done", "complete", "completed", "finished"]
            .iter()
            .any(|word| word_at_start(&lower, word))
}

fn word_at_start(text: &str, word: &str) -> bool {
    text.strip_prefix(word)
        .is_some_and(|rest| !rest.chars().next().is_some_and(char::is_alphanumeric))
}

fn starts_with_ignoring_case(text: &str, prefix: &str) -> bool {
    text.len() >= prefix.len()
        && text.is_char_boundary(prefix.len())
        && text[..prefix.len()].eq_ignore_ascii_case(prefix)
}

/// Reads a brief file.
pub fn read(path: &Path) -> io::Result<Brief> {
    Ok(parse(&fs::read_to_string(path)?))
}

/// Reads a brief's text.
pub fn parse(text: &str) -> Brief {
    let all = lines(text);
    let (front, skip) = front_matter(&all);
    let mut brief = Brief {
        front,
        ..Brief::default()
    };
    let mut in_body = false;
    for block in blocks(&all[skip..]) {
        match block {
            Block::Heading { level, text, line } => {
                if level == 1 && brief.title.is_none() {
                    brief.title = Some(text.to_owned());
                }
                if level >= 2 {
                    in_body = true;
                }
                if brief.handoff.is_none() && starts_with_ignoring_case(text, "Handoff payload") {
                    let group = group_of(text);
                    let date = first_date(group).map(str::to_owned);
                    brief.handoff = Some(Handoff {
                        filled: !group.to_ascii_lowercase().starts_with("fill at")
                            && (date.is_some() || group.to_ascii_lowercase().contains("filled")),
                        date,
                        line,
                    });
                }
                brief.headings.push(Heading {
                    level,
                    text: text.to_owned(),
                    line,
                });
            }
            Block::Paragraph { lines } if !in_body => {
                let joined: Vec<&str> = lines.iter().map(|line| line.text.trim()).collect();
                brief.fields.extend(fields_of(&joined.join(" ")));
            }
            Block::Quote { lines } => brief.quotes.extend(quotes_of(&lines)),
            _ => {}
        }
    }
    brief
}

/// Front matter at the top of the file: the fields and the number of lines to skip.
fn front_matter(all: &[Line]) -> (Option<FrontMatter>, usize) {
    if all.first().is_none_or(|line| line.text.trim_end() != "---") {
        return (None, 0);
    }
    let Some(close) = all
        .iter()
        .skip(1)
        .take(100)
        .position(|line| matches!(line.text.trim_end(), "---" | "..."))
        .map(|k| k + 1)
    else {
        return (None, 0);
    };
    let fields = all[1..close]
        .iter()
        .filter_map(|line| {
            let text = line.text.trim();
            if text.is_empty() || text.starts_with('#') {
                return None;
            }
            let (key, value) = text.split_once(':')?;
            let key = key.trim();
            if key.is_empty() || key.contains(' ') {
                return None;
            }
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);
            Some(Field {
                label: key.to_owned(),
                value: value.to_owned(),
            })
        })
        .collect();
    (
        Some(FrontMatter {
            fields,
            lines: close + 1,
        }),
        close + 1,
    )
}

/// The bold fields of a header line: each `**Label:**` with the text up to the next one, the
/// ` · ` separators trimmed away. A line of prose that opens with bold text has no fields.
fn fields_of(text: &str) -> Vec<Field> {
    let mut tokens: Vec<(usize, usize, &str)> = Vec::new();
    let mut at = 0;
    while let Some(k) = text[at..].find("**") {
        let open = at + k;
        let inner_at = open + 2;
        match text[inner_at..].find(":**") {
            Some(len)
                if len > 0
                    && len <= 40
                    && !text[inner_at..inner_at + len].contains("**")
                    && !text[inner_at..inner_at + len].contains(['\n', '(', ')']) =>
            {
                tokens.push((open, inner_at + len + 3, &text[inner_at..inner_at + len]));
                at = inner_at + len + 3;
            }
            _ => at = inner_at,
        }
    }
    if tokens
        .first()
        .is_none_or(|&(open, _, _)| !text[..open].trim().is_empty())
    {
        return Vec::new();
    }
    tokens
        .iter()
        .enumerate()
        .map(|(i, &(_, value_at, label))| {
            let end = tokens.get(i + 1).map_or(text.len(), |&(open, _, _)| open);
            let value = text[value_at..end]
                .trim()
                .trim_end_matches('·')
                .trim()
                .to_owned();
            Field {
                label: label.trim().to_owned(),
                value,
            }
        })
        .collect()
}

/// The labelled blocks of one blockquote: a new one starts at the first line and at every line
/// that opens with a label of a known kind, so two completion notes in one quote are two blocks.
fn quotes_of(lines: &[Line]) -> Vec<Quote> {
    let mut found: Vec<Quote> = Vec::new();
    let texts: Vec<&str> = lines.iter().map(|line| quoted(line.text)).collect();
    for (i, line) in lines.iter().enumerate() {
        let text = texts[i];
        // A label may wrap onto the next quoted line, so it is read from the rest of the block.
        let label = text
            .starts_with("**")
            .then(|| label_of(&texts[i..].join(" ")))
            .flatten();
        let starts = i == 0 || label.as_ref().is_some_and(|(_, kind)| *kind != Kind::Other);
        if starts {
            let (label, kind) = label.unwrap_or_default();
            found.push(Quote {
                date: first_date(&label).map(str::to_owned),
                kind,
                label,
                text: text.trim_end().to_owned(),
                line: line.number,
            });
        } else if let Some(last) = found.last_mut() {
            last.text.push('\n');
            last.text.push_str(text.trim_end());
        }
    }
    found
}

/// The bold label a line opens with (`**Completion note (date).**` → `Completion note (date)`)
/// and what kind of block it names.
fn label_of(text: &str) -> Option<(String, Kind)> {
    let rest = text.strip_prefix("**")?;
    let close = rest.find("**")?;
    let label = rest[..close].trim().trim_end_matches(['.', ':']).trim();
    if label.is_empty() {
        return None;
    }
    Some((label.to_owned(), kind_of(label)))
}

fn kind_of(label: &str) -> Kind {
    let lower = label.to_ascii_lowercase();
    let name = lower.split('(').next().unwrap_or_default().trim();
    let group = group_of(label);
    if name.starts_with("completion note") {
        return Kind::Completion(completion_of(group));
    }
    if name.starts_with("☑ complete") || name == "complete" || name.starts_with("complete (") {
        return Kind::Complete;
    }
    if name.starts_with("handed on") || name.starts_with("parked and handed on") {
        let to = group
            .strip_prefix("to ")
            .map(|to| to.trim().trim_end_matches(['.', ';']).to_owned())
            .filter(|to| !to.is_empty());
        return Kind::HandedOn { to };
    }
    if name.starts_with("parked") {
        return Kind::Parked;
    }
    if let Some((id, end)) = scan_ep_id(label, 0)
        && lower[end..].trim_start().starts_with("pickup note")
    {
        return Kind::PickupNote { ep: Some(id) };
    }
    if name.starts_with("pickup note") {
        return Kind::PickupNote { ep: None };
    }
    if name.starts_with("charter") {
        return Kind::Charter;
    }
    if name.starts_with("pickup gate") {
        return Kind::PickupGate;
    }
    if name.starts_with("owner decision") {
        return Kind::OwnerDecisions;
    }
    Kind::Other
}

/// The inside of the first parenthesised group of `text` (up to its last `)`), or nothing.
fn group_of(text: &str) -> &str {
    match (text.find('('), text.rfind(')')) {
        (Some(open), Some(close)) if close > open => text[open + 1..close].trim(),
        _ => "",
    }
}

/// A completion note's parenthesis: `2026-10-31; fable/xhigh; 1 h 20 min` gives model, effort and
/// duration; `2026-10-31, EP-28 verification` gives a suffix; `2026-10-31` alone gives nothing.
fn completion_of(group: &str) -> Completion {
    let mut completion = Completion::default();
    if group.contains(';') {
        let parts: Vec<&str> = group.split(';').map(str::trim).collect();
        let mut rest = parts.iter().skip(1).copied();
        for part in rest.by_ref() {
            if let Some((model, effort)) = part.split_once('/')
                && !model.trim().is_empty()
                && !effort.trim().is_empty()
                && !model.contains(' ')
                && !effort.trim().contains(' ')
            {
                completion.model = Some(model.trim().to_owned());
                completion.effort = Some(effort.trim().to_owned());
                break;
            }
            if completion.suffix.is_none() && !part.is_empty() {
                completion.suffix = Some(part.to_owned());
            }
        }
        let duration: Vec<&str> = rest.filter(|part| !part.is_empty()).collect();
        if !duration.is_empty() {
            completion.duration = Some(duration.join("; "));
        }
    } else if let Some((_, suffix)) = group.split_once(',') {
        let suffix = suffix.trim();
        if !suffix.is_empty() {
            completion.suffix = Some(suffix.to_owned());
        }
    }
    completion
}

// --- The packet style's handoff file ------------------------------------------------------------

/// `docs/handoffs/EP-N.md`: the packet style's record of a finished packet, opening with
/// `**Status:** complete · **Date:** date · **Brief:** …` and a `**Commits:**` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffFile {
    pub path: PathBuf,
    pub fields: Vec<Field>,
}

impl HandoffFile {
    pub fn status(&self) -> Option<&str> {
        field(&self.fields, "Status")
    }

    pub fn date(&self) -> Option<&str> {
        field(&self.fields, "Date").and_then(first_date)
    }

    pub fn commits(&self) -> Vec<String> {
        hashes(field(&self.fields, "Commits").unwrap_or_default())
    }

    pub fn is_complete(&self) -> bool {
        self.status().is_some_and(status_is_done)
    }
}

/// The handoff file of `id` in `folder`, when the project keeps one.
pub fn handoff_file(folder: &Path, id: EpId) -> Option<HandoffFile> {
    let path = folder
        .join("docs")
        .join("handoffs")
        .join(format!("{id}.md"));
    let text = fs::read_to_string(&path).ok()?;
    Some(HandoffFile {
        fields: parse(&text).fields,
        path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep(number: u32) -> EpId {
        EpId::new(number)
    }

    #[test]
    fn reads_brief_file_names() {
        assert_eq!(file_id("EP-4-parser-b.md"), Some((ep(4), "parser-b")));
        assert_eq!(
            file_id("EP-5a-spine-acquisition.md").map(|(id, _)| id.to_string()),
            Some("EP-5a".to_owned())
        );
        assert_eq!(
            file_id("EP-01-zero-padded.md"),
            Some((ep(1), "zero-padded"))
        );
        assert_eq!(
            file_id("EP-130-report-engine-md-html.md"),
            Some((ep(130), "report-engine-md-html"))
        );
        for not in [
            "_TEMPLATE.md",
            "README.md",
            "EP-4.md",
            "EP-4-.md",
            "EP-4-parser b.md",
            "EP-4-parser-b.txt",
            "P0-retro.md",
            "ep-4-lower.md",
        ] {
            assert_eq!(file_id(not), None, "{not:?}");
        }
        assert!(is_companion("completion-handoff"));
        assert!(is_companion("completion-report"));
        assert!(!is_companion("report-engine"));
        assert!(!is_companion("parser-b"));
    }

    const HOUSE: &str = "\
---
ep: 3
title: \"Parser A — tables\"
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-3 — Parser A: tables

**Size:** M · **Depends on:** EP-1 · **Blocks:** EP-4 (and through it EP-5 to EP-8)

## Context

**Verified at pickup (2026-10-31, 16:10).** A read-only survey found 61 tables.

## In scope

1. Tables.

> **Completion note (2026-10-31; opus/max; ≈ 50 min to the commit).** Executed in one session.
> Owner decisions: **commit**: both commits pushed.
> **After the commit (≈ 52 min in all):** CI passed.

> **Handed on (to alpha).** EP-4: the snapshot parses HEAD and the working tree each.
> **Parked.** Below about 450 pt the rows overflow.
";

    #[test]
    fn reads_a_house_style_brief() {
        let brief = parse(HOUSE);
        let front = brief.front.as_ref().unwrap();
        assert_eq!(front.lines, 8);
        assert_eq!(front.ep(), Some(ep(3)));
        // A lettered row's brief, in the house form and with the prefix; nothing else.
        for (value, id) in [("7a", Some("EP-7a")), ("EP-7a", Some("EP-7a")), ("N", None)] {
            let text = HOUSE.replacen("ep: 3", &format!("ep: {value}"), 1);
            let front = parse(&text).front.unwrap();
            assert_eq!(
                front.ep().map(|id| id.to_string()).as_deref(),
                id,
                "{value}"
            );
        }
        assert_eq!(front.get("title"), Some("Parser A — tables"));
        assert_eq!(front.size(), Some(Size::M));
        assert_eq!(front.model(), Some("claude-opus-5-5"));
        assert_eq!(front.effort(), Some("max"));
        assert_eq!(front.status(), Some("done"));
        assert_eq!(brief.title.as_deref(), Some("EP-3 — Parser A: tables"));
        let fields: Vec<(&str, &str)> = brief
            .fields
            .iter()
            .map(|f| (f.label.as_str(), f.value.as_str()))
            .collect();
        assert_eq!(
            fields,
            [
                ("Size", "M"),
                ("Depends on", "EP-1"),
                ("Blocks", "EP-4 (and through it EP-5 to EP-8)"),
            ]
        );
        assert_eq!(brief.size(), Some(Size::M));
        assert_eq!(brief.depends(), [ep(1)]);
        assert_eq!(brief.blocks(), [ep(4), ep(5), ep(8)]);
        assert_eq!(brief.status(), None);
        let headings: Vec<_> = brief.headings.iter().map(|h| h.text.as_str()).collect();
        assert_eq!(headings, ["EP-3 — Parser A: tables", "Context", "In scope"]);

        let kinds: Vec<_> = brief
            .quotes
            .iter()
            .map(|q| (q.line, &q.kind, q.date.as_deref()))
            .collect();
        assert_eq!(
            kinds,
            [
                (
                    22,
                    &Kind::Completion(Completion {
                        model: Some("opus".to_owned()),
                        effort: Some("max".to_owned()),
                        duration: Some("≈ 50 min to the commit".to_owned()),
                        suffix: None,
                    }),
                    Some("2026-10-31")
                ),
                (
                    26,
                    &Kind::HandedOn {
                        to: Some("alpha".to_owned())
                    },
                    None
                ),
                (27, &Kind::Parked, None),
            ]
        );
        let note = brief.completion_notes().next().unwrap();
        assert_eq!(
            note.label,
            "Completion note (2026-10-31; opus/max; ≈ 50 min to the commit)"
        );
        assert!(note.text.starts_with("**Completion note"));
        assert!(
            note.text
                .ends_with("**After the commit (≈ 52 min in all):** CI passed."),
            "a bold line of another kind stays in the note: {:?}",
            note.text
        );
        assert_eq!(brief.handed_on().count(), 1);
        assert_eq!(
            brief.completion(),
            Some(Evidence::CompletionNote {
                date: Some("2026-10-31".to_owned()),
                line: 22
            })
        );
        assert!(brief.is_complete());
    }

    #[test]
    fn a_brief_in_progress_has_no_completion() {
        let open = HOUSE
            .replace("status: done", "status: in progress")
            .split("> **Completion note")
            .next()
            .unwrap()
            .to_owned();
        let brief = parse(&open);
        assert_eq!(brief.completion(), None);
        assert!(!brief.is_complete());
        assert!(brief.quotes.is_empty());
        // Front matter `status: done` alone is evidence; `in progress` is not.
        let done_by_front = parse("---\nep: 2\nstatus: done\n---\n# EP-2 — Two\n");
        assert_eq!(
            done_by_front.completion(),
            Some(Evidence::FrontMatter {
                status: "done".to_owned()
            })
        );
    }

    #[test]
    fn reads_completion_notes_in_every_surveyed_form() {
        let note = |label: &str| {
            let brief = parse(&format!("# EP-1 — One\n\n> **{label}** text\n"));
            brief.quotes.into_iter().next().unwrap()
        };
        let plain = note("Completion note (2026-08-12).");
        assert_eq!(plain.kind, Kind::Completion(Completion::default()));
        assert_eq!(plain.date.as_deref(), Some("2026-08-12"));
        assert_eq!(plain.label, "Completion note (2026-08-12)");

        let verified = note("Completion note (2026-09-20, EP-28 verification).");
        assert_eq!(
            verified.kind,
            Kind::Completion(Completion {
                suffix: Some("EP-28 verification".to_owned()),
                ..Completion::default()
            })
        );
        let appended =
            note("Completion note (2026-09-21, appended by EP-28 — the ⏱ verification).");
        assert!(
            matches!(appended.kind, Kind::Completion(ref c) if c.suffix.as_deref() == Some("appended by EP-28 — the ⏱ verification"))
        );

        let new = note("Completion note (2026-10-31; fable/xhigh; 1 h 20 min).");
        assert_eq!(
            new.kind,
            Kind::Completion(Completion {
                model: Some("fable".to_owned()),
                effort: Some("xhigh".to_owned()),
                duration: Some("1 h 20 min".to_owned()),
                suffix: None,
            })
        );
        let long = note(
            "Completion note (2026-10-31; fable/xhigh; ≈ 2 h 40 min from the survey to the documents).",
        );
        assert!(
            matches!(long.kind, Kind::Completion(ref c) if c.duration.as_deref() == Some("≈ 2 h 40 min from the survey to the documents"))
        );
        let bare = note("Completion note.");
        assert_eq!(bare.kind, Kind::Completion(Completion::default()));
        assert_eq!(bare.date, None);
        let colon = note("Completion note (2026-08-18):");
        assert_eq!(colon.label, "Completion note (2026-08-18)");

        assert_eq!(note("☑ COMPLETE (2026-08-18).").kind, Kind::Complete);
        assert_eq!(note("Handed on.").kind, Kind::HandedOn { to: None });
        assert_eq!(
            note("Handed on (to the owner).").kind,
            Kind::HandedOn {
                to: Some("the owner".to_owned())
            }
        );
        assert_eq!(
            note("Parked and handed on.").kind,
            Kind::HandedOn { to: None }
        );
        assert_eq!(note("Parked.").kind, Kind::Parked);
        let pickup = note("EP-54 pickup note (2026-09-20).");
        assert_eq!(pickup.kind, Kind::PickupNote { ep: Some(ep(54)) });
        assert_eq!(pickup.date.as_deref(), Some("2026-09-20"));
        assert_eq!(note("Charter.").kind, Kind::Charter);
        assert_eq!(
            note("PICKUP GATE — owner input required before this brief starts (EP-8, 2026-09-01).")
                .kind,
            Kind::PickupGate
        );
        assert_eq!(
            note("Owner decisions (`AskUserQuestion`, 2026-10-30).").kind,
            Kind::OwnerDecisions
        );
        assert_eq!(
            note("Owner decisions at close-out (`AskUserQuestion`, 2026-10-30).").kind,
            Kind::OwnerDecisions
        );
        assert_eq!(
            note("Routine choices (the session's call, logged).").kind,
            Kind::Other
        );
        assert_eq!(note("Addendum (2026-10-31, EP-3).").kind, Kind::Other);

        assert_eq!(
            new.kind.to_string(),
            "completion note (fable/xhigh; 1 h 20 min)"
        );
        assert_eq!(
            verified.kind.to_string(),
            "completion note (EP-28 verification)"
        );
        assert_eq!(plain.kind.to_string(), "completion note");
        assert_eq!(pickup.kind.to_string(), "EP-54 pickup note");
        assert_eq!(
            Kind::HandedOn {
                to: Some("alpha".to_owned())
            }
            .to_string(),
            "handed on to alpha"
        );
    }

    #[test]
    fn two_notes_in_one_quote_are_two_blocks_and_a_plain_quote_is_one() {
        let text = "\
# EP-2 — Two

> **Completion note (2026-08-01).** First.
> More of the first.
> **Completion note (2026-08-09, EP-5 verification).** Second.

> A plain quote with **bold** inside.
> **Deviations.** Still the same block.
";
        let brief = parse(text);
        let summary: Vec<_> = brief
            .quotes
            .iter()
            .map(|q| (q.line, q.kind.clone(), q.text.lines().count()))
            .collect();
        assert_eq!(
            summary,
            [
                (3, Kind::Completion(Completion::default()), 2),
                (
                    5,
                    Kind::Completion(Completion {
                        suffix: Some("EP-5 verification".to_owned()),
                        ..Completion::default()
                    }),
                    1
                ),
                (7, Kind::Other, 2),
            ]
        );
        assert_eq!(brief.quotes[2].label, "");
        assert_eq!(brief.completion_notes().count(), 2);

        // A label that wraps onto the next quoted line.
        let wrapped = parse(
            "# EP-3 — Three\n\n> **Completion note (2026-10-31; opus/max; ≈ 50 min to the\n> commit).** Executed in one session.\n",
        );
        let note = wrapped.completion_notes().next().unwrap();
        assert_eq!(
            note.label,
            "Completion note (2026-10-31; opus/max; ≈ 50 min to the commit)"
        );
        assert!(matches!(
            note.kind,
            Kind::Completion(ref c) if c.duration.as_deref() == Some("≈ 50 min to the commit")
        ));
        assert_eq!(note.text.lines().count(), 2);
    }

    #[test]
    fn reads_the_heading_status_and_handoff_markers() {
        let heading = parse(
            "# EP-9 — Nine\n\n**Size:** M · **Depends on:** EP-5 · **Blocks:** —\n\n## Context\n\nx\n\n## Completion note (2026-08-12)\n\nExecuted across two sessions.\n",
        );
        assert_eq!(
            heading.completion(),
            Some(Evidence::CompletionHeading {
                date: Some("2026-08-12".to_owned()),
                line: 9
            })
        );

        let status = parse(
            "# EP-7 — Seven\n\n**Size:** M · **Depends on:** EP-5 · **Blocks:** — (EP-8+ assume the repo is clean)\n**Status:** DONE (executed 2026-08-12, the session that authored this roadmap).\n\n## Context\n",
        );
        assert_eq!(status.depends(), [ep(5)]);
        assert_eq!(status.blocks(), [ep(8)]);
        assert!(
            matches!(status.completion(), Some(Evidence::Status { value }) if value.starts_with("DONE"))
        );

        let packet = parse(
            "# EP-2 — Scaffold\n\n**Status:** [x] 9bcb7b2 · **Milestone:** M0 · **Effort:** M (1–2 sessions, high confidence) · **Parallel with:** —\n\n## Outcome\n\n## Handoff payload (filled 2026-08-23)\n- **Packet:** EP-2 — done at commit `9bcb7b2`.\n",
        );
        assert_eq!(packet.status(), Some("[x] 9bcb7b2"));
        assert_eq!(packet.field("Milestone"), Some("M0"));
        assert_eq!(
            packet.field("Effort"),
            Some("M (1–2 sessions, high confidence)")
        );
        assert_eq!(packet.size(), None, "the packet style has no Size field");
        assert_eq!(
            packet.handoff,
            Some(Handoff {
                filled: true,
                date: Some("2026-08-23".to_owned()),
                line: 7
            })
        );
        assert!(matches!(packet.completion(), Some(Evidence::Status { .. })));

        let unfilled = parse(
            "# EP-6 — Six\n\n**Status:** [ ] · **Milestone:** M1\n\n## Handoff payload (fill at session end)\n- packet ID + status\n",
        );
        assert_eq!(unfilled.handoff.as_ref().map(|h| h.filled), Some(false));
        assert_eq!(unfilled.completion(), None);
        let filled_only = parse("# EP-6 — Six\n\n## Handoff payload (filled 2026-09-01)\n- done\n");
        assert!(matches!(
            filled_only.completion(),
            Some(Evidence::HandoffPayload { .. })
        ));

        for (value, done) in [
            ("DONE", true),
            ("complete — all owner decisions ruled 2026-09-04", true),
            ("[x] e4c2c95 done 2026-08-27", true),
            (
                "split 2026-08-25 into [EP-5a](EP-5a.md) and [EP-5b](EP-5b.md)",
                false,
            ),
            ("deferred — not on the critical path", false),
            ("[ ]", false),
            ("in progress", false),
            ("Donegal", false),
        ] {
            assert_eq!(status_is_done(value), done, "{value:?}");
        }
    }

    #[test]
    fn reads_header_lines_in_their_shapes() {
        // The wrapped form: the line ends with a separator and continues on the next.
        let wrapped = parse(
            "# EP-1 — One\n\n**Size:** M · **Mode:** n/a · **Core/Stretch:** core ·\n**Depends on:** EP-0 (baseline) · **Blocks:** EP-2 (docs), EP-6 (CI)\n\n## Context\n",
        );
        let labels: Vec<_> = wrapped.fields.iter().map(|f| f.label.as_str()).collect();
        assert_eq!(
            labels,
            ["Size", "Mode", "Core/Stretch", "Depends on", "Blocks"]
        );
        assert_eq!(wrapped.field("Core/Stretch"), Some("core"));
        assert_eq!(wrapped.depends(), [ep(0)]);
        assert_eq!(wrapped.blocks(), [ep(2), ep(6)]);

        let tiered = parse(
            "# EP-58 — App\n\n**Size:** M · **Tier:** fixture+dev+full · **Core/Stretch:** core · **Depends on:** EP-57 (App shell A (Streamlit multipage)) · **Blocks:** EP-59\n",
        );
        assert_eq!(tiered.field("Tier"), Some("fixture+dev+full"));
        assert_eq!(tiered.depends(), [ep(57)]);
        assert_eq!(tiered.blocks(), [ep(59)]);
        assert_eq!(
            tiered.field("Depends on"),
            Some("EP-57 (App shell A (Streamlit multipage))")
        );

        // Prose that opens with bold text is not a header line; fields after `##` are not read.
        let prose = parse(
            "# EP-3 — Three\n\n**Verified at pickup (2026-10-31, 16:10).** A survey found **Size:** nothing.\n\n## Context\n\n**Size:** L · **Depends on:** EP-9\n",
        );
        assert!(prose.fields.is_empty(), "{:?}", prose.fields);
        let none = parse("# EP-3 — Three\n\nPlain context.\n");
        assert!(none.fields.is_empty());
        assert_eq!(none.size(), None);
    }

    #[test]
    fn front_matter_is_only_at_the_top_and_only_when_closed() {
        let rule_later = parse("# EP-1 — One\n\n---\n\nsize: L\n\n---\n");
        assert_eq!(rule_later.front, None);
        let unclosed = parse("---\nep: 1\n# EP-1 — One\n");
        assert_eq!(unclosed.front, None);
        assert_eq!(unclosed.title.as_deref(), Some("EP-1 — One"));
        let bom = parse("\u{feff}---\nep: 1\ntitle: One\n---\n# EP-1 — One\n");
        assert_eq!(bom.front.as_ref().and_then(FrontMatter::ep), Some(ep(1)));
        let dots = parse("---\nep: EP-4a\nsize: 'S'\n# a comment\nodd line\n...\n");
        let front = dots.front.unwrap();
        assert_eq!(
            front.ep().map(|id| id.to_string()),
            Some("EP-4a".to_owned())
        );
        assert_eq!(front.size(), Some(Size::S));
        assert_eq!(front.fields.len(), 2);
        assert_eq!(front.lines, 6);
    }

    #[test]
    fn reads_a_handoff_file() {
        let text = "\
# EP-0 handoff — Baseline floor

**Status:** complete · **Date:** 2026-08-23 · **Brief:** roadmap/EP-0-baseline-floor.md

## Completed scope IDs

All six.

**Commits:** `cc5c892` (identity pack), `07f46a4`
";
        let handoff = HandoffFile {
            path: PathBuf::from("docs/handoffs/EP-0.md"),
            fields: parse(text).fields,
        };
        assert_eq!(handoff.status(), Some("complete"));
        assert_eq!(handoff.date(), Some("2026-08-23"));
        assert!(handoff.is_complete());
        // The commits line sits after a heading, so the body is not read for fields; the
        // header line's own fields are.
        assert!(handoff.commits().is_empty());
        let with_commits = HandoffFile {
            path: PathBuf::from("x"),
            fields: parse(
                "# H\n\n**Status:** complete · **Commits:** `cc5c892` (pack), `07f46a4`\n",
            )
            .fields,
        };
        assert_eq!(with_commits.commits(), ["cc5c892", "07f46a4"]);
        assert!(
            !HandoffFile {
                path: PathBuf::new(),
                fields: Vec::new()
            }
            .is_complete()
        );
    }

    #[test]
    fn discovers_briefs_companions_and_duplicates() {
        let scratch = crate::files::tests::Scratch::new("brief-discover");
        let dir = scratch.0.join("roadmap");
        fs::create_dir_all(&dir).unwrap();
        for name in [
            "README.md",
            "_TEMPLATE.md",
            "EP-0-plan.md",
            "EP-2-second.md",
            "EP-1-first.md",
            "EP-1-first-again.md",
            "EP-9-room.md",
            "EP-9-completion-handoff.md",
            "EP-9-completion-report.md",
            "EP-4a-split.md",
            "inbox.md",
        ] {
            fs::write(dir.join(name), "# x\n").unwrap();
        }
        fs::create_dir_all(dir.join("EP-3-a-folder.md")).unwrap();
        let files = discover(&dir).unwrap();
        let names = |list: &[BriefFile]| list.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
        assert_eq!(
            names(&files.briefs),
            [
                "EP-0-plan.md",
                "EP-1-first-again.md",
                "EP-2-second.md",
                "EP-4a-split.md",
                "EP-9-room.md"
            ]
        );
        assert_eq!(names(&files.duplicates), ["EP-1-first.md"]);
        assert_eq!(
            names(&files.companions),
            ["EP-9-completion-handoff.md", "EP-9-completion-report.md"]
        );
        assert_eq!(files.brief(ep(9)).unwrap().path, dir.join("EP-9-room.md"));
        assert_eq!(files.brief(ep(3)), None);
        assert_eq!(
            discover(&scratch.0.join("missing")).unwrap(),
            Files::default()
        );
    }
}
