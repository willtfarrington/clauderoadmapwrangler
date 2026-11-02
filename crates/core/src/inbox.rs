//! The cross-project inbox (D-13): `roadmap/inbox.md`, one line per message from another project
//! or from the owner, `- [ ] IN-n (from <project> EP-k, date) text`, optionally `[urgent]`, which
//! the receiving session triages and the owner may tick in the tool, the disposition written on
//! the message's line; and the other side of the mail, the `> **Handed on (to <project>).**`
//! blocks a session leaves in its brief for another project, with the target resolved against the
//! registry. Delivering them is EP-20's.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::checklist;
use crate::roadmap::brief::Kind;
use crate::roadmap::snapshot::Entry;
use crate::roadmap::table::Problem;
use crate::roadmap::{EpId, ep_ids, first_date};
use crate::tasks::legacy::is_owner;

pub const FILE: &str = "inbox.md";

/// The letters of a message's id: `IN-3`.
pub const PREFIX: &str = "IN";

/// `<folder>\roadmap\inbox.md`.
pub fn path(folder: &Path) -> PathBuf {
    folder.join("roadmap").join(FILE)
}

/// A line of `inbox.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: String,
    pub done: bool,
    /// The parenthesis without `from` and the date: `beta EP-4`, `the owner, workspace split`.
    pub from: Option<String>,
    /// Who sent it: a project's name, or `the owner`.
    pub sender: Option<String>,
    /// The sender's EP, when the parenthesis names one.
    pub ep: Option<EpId>,
    pub date: Option<String>,
    /// The message without its disposition.
    pub text: String,
    /// `[urgent]` in the text: the owner is notified, the session is not (D-13).
    pub urgent: bool,
    /// The disposition written when the message was triaged, from its bold label on
    /// (`**Disposition (date):** …`, `**Triaged at EP-4 (date):** …`).
    pub disposition: Option<String>,
    pub line: usize,
}

/// `inbox.md` as the parser reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inbox {
    pub path: PathBuf,
    pub messages: Vec<Message>,
    /// Boxes without an `IN-n` id and ids written twice.
    pub problems: Vec<Problem>,
}

impl Inbox {
    pub fn open(&self) -> impl Iterator<Item = &Message> {
        self.messages.iter().filter(|message| !message.done)
    }
}

/// The bold labels a disposition opens with.
const DISPOSITIONS: [&str; 2] = ["**Disposition", "**Triaged"];

/// Reads the text of `inbox.md`.
pub fn parse(text: &str) -> (Vec<Message>, Vec<Problem>) {
    let list = checklist::parse(text, PREFIX);
    let messages = list
        .items
        .into_iter()
        .map(|item| {
            let (body, disposition) = match DISPOSITIONS
                .iter()
                .filter_map(|label| item.text.find(label))
                .min()
            {
                Some(at) => (
                    item.text[..at].trim_end(),
                    Some(item.text[at..].trim().to_owned()),
                ),
                None => (item.text.as_str(), None),
            };
            let meta = item.meta.as_deref().unwrap_or_default();
            let from = from_of(meta);
            Message {
                sender: from.as_deref().map(sender_of),
                ep: from
                    .as_deref()
                    .and_then(|from| ep_ids(from).first().copied()),
                date: first_date(meta).map(str::to_owned),
                urgent: body.to_ascii_lowercase().contains("[urgent]"),
                text: body.to_owned(),
                from,
                disposition,
                done: item.checked,
                id: item.id,
                line: item.line,
            }
        })
        .collect();
    (messages, list.problems)
}

/// The open messages of `text` as the file writes them, each from its box to its last line: what
/// a session start is given (EP-12, the stopgap hook's job in the tool).
pub fn open_as_written(text: &str) -> Vec<&str> {
    checklist::parse(text, PREFIX)
        .items
        .iter()
        .filter(|item| !item.checked)
        .map(|item| item.written(text))
        .collect()
}

/// `from beta EP-4, 2026-10-02` → `beta EP-4`: the parenthesis without `from` and its date.
fn from_of(meta: &str) -> Option<String> {
    let body = meta.trim();
    let body = body
        .get(..5)
        .filter(|start| start.eq_ignore_ascii_case("from "))
        .map_or(body, |_| &body[5..]);
    let body = match first_date(body) {
        Some(date) if body.trim_end().ends_with(date) => {
            body.trim_end()[..body.trim_end().len() - date.len()].trim_end()
        }
        _ => body,
    };
    let body = body.trim_end_matches([',', ';']).trim();
    (!body.is_empty()).then(|| body.to_owned())
}

/// The sender in a `from`: up to the first comma or EP id (`beta EP-4` → `beta`, `the owner,
/// workspace split` → `the owner`).
fn sender_of(from: &str) -> String {
    let end = [from.find(','), from.find(" EP-")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(from.len());
    from[..end].trim().to_owned()
}

/// Reads a project's `inbox.md`; `None` when it has none.
pub fn read(folder: &Path) -> io::Result<Option<Inbox>> {
    let path = path(folder);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let (messages, problems) = parse(&text);
    Ok(Some(Inbox {
        path,
        messages,
        problems,
    }))
}

/// Ticks message `id` of the project in `folder` with `disposition` (required, D-13), written on
/// the message's line as ` **Disposition (<today>):** <disposition>`, after a copy of the file in
/// `backups`. Returns the copy.
pub fn tick(
    folder: &Path,
    id: &str,
    disposition: &str,
    backups: &Path,
    today: &str,
) -> Result<PathBuf, checklist::Error> {
    // One line: a line break would end the list item.
    let words = disposition.split_whitespace().collect::<Vec<_>>().join(" ");
    if words.is_empty() {
        return Err(checklist::Error::NoDisposition(id.to_owned()));
    }
    let written = format!("**Disposition ({today}):** {words}");
    checklist::tick(
        &path(folder),
        PREFIX,
        id,
        true,
        Some(&written),
        backups,
        today,
    )
}

/// A `> **Handed on (to <target>).**` block in a brief: a message for another project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outgoing {
    /// The brief it is written in.
    pub brief: EpId,
    pub path: PathBuf,
    pub line: usize,
    /// The target as written.
    pub to: String,
    pub recipient: Recipient,
    /// The block's text, its label included.
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recipient {
    /// A registered project, by its registry name.
    Project(String),
    /// A name the registry does not know: shown, not delivered (D-13).
    Unknown,
}

/// Every `Handed on (to <target>)` block of `briefs` but those to the owner (owner items, D-10),
/// each target resolved by `resolve` (the registry's name, case aside), whole or by its first
/// word (`beta EP-6` → `beta`).
pub fn outgoing(briefs: &[Entry], resolve: impl Fn(&str) -> Option<String>) -> Vec<Outgoing> {
    let mut found = Vec::new();
    for entry in briefs {
        for quote in entry.brief.handed_on() {
            let Kind::HandedOn { to: Some(to) } = &quote.kind else {
                continue;
            };
            if is_owner(to) {
                continue;
            }
            let first_word = to.split([' ', ',']).next().unwrap_or_default();
            let recipient = resolve(to)
                .or_else(|| resolve(first_word))
                .map_or(Recipient::Unknown, Recipient::Project);
            found.push(Outgoing {
                brief: entry.file.id,
                path: entry.file.path.clone(),
                line: quote.line,
                to: to.clone(),
                recipient,
                text: quote.text.clone(),
            });
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;
    use crate::roadmap::brief::{self, BriefFile};

    fn ep(number: u32) -> EpId {
        EpId::new(number)
    }

    const INBOX: &str = "\
# Inbox — messages from other projects

Read right after the roadmap. One message per line: `- [ ] IN-n (from <project> EP-k, date) text`.

- [x] IN-1 (from beta EP-4, 2026-10-02) The shared fixture names changed. **Disposition (2026-10-03):** rolled into EP-2.
- [ ] IN-2 (from the owner, workspace split, 2026-10-04) Sessions start from the project's own
  workspace file now; nothing to build.
- [ ] IN-3 (from gamma EP-7, 2026-10-05) The export gains a column. [Urgent]
- [x] IN-4 (from the owner, environment repository EP-0, 2026-10-06) A longer note that wraps
  onto a second line.
  **Triaged at EP-3 (2026-10-07):** parked in EP-5's brief notes.
- [ ] IN-5 No parenthesis at all.
";

    #[test]
    fn reads_messages_their_senders_and_dispositions() {
        let (messages, problems) = parse(INBOX);
        assert!(problems.is_empty(), "{problems:?}");
        let summary: Vec<_> = messages
            .iter()
            .map(|m| {
                (
                    m.id.as_str(),
                    m.done,
                    m.sender.as_deref(),
                    m.ep,
                    m.date.as_deref(),
                    m.urgent,
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                (
                    "IN-1",
                    true,
                    Some("beta"),
                    Some(ep(4)),
                    Some("2026-10-02"),
                    false
                ),
                (
                    "IN-2",
                    false,
                    Some("the owner"),
                    None,
                    Some("2026-10-04"),
                    false
                ),
                (
                    "IN-3",
                    false,
                    Some("gamma"),
                    Some(ep(7)),
                    Some("2026-10-05"),
                    true
                ),
                (
                    "IN-4",
                    true,
                    Some("the owner"),
                    Some(ep(0)),
                    Some("2026-10-06"),
                    false
                ),
                ("IN-5", false, None, None, None, false),
            ]
        );
        assert_eq!(messages[0].from.as_deref(), Some("beta EP-4"));
        assert_eq!(messages[0].text, "The shared fixture names changed.");
        assert_eq!(
            messages[0].disposition.as_deref(),
            Some("**Disposition (2026-10-03):** rolled into EP-2.")
        );
        assert_eq!(
            messages[1].from.as_deref(),
            Some("the owner, workspace split")
        );
        assert_eq!(
            messages[1].text,
            "Sessions start from the project's own\nworkspace file now; nothing to build."
        );
        assert_eq!(messages[1].disposition, None);
        assert_eq!(
            messages[3].text,
            "A longer note that wraps\nonto a second line."
        );
        assert_eq!(
            messages[3].disposition.as_deref(),
            Some("**Triaged at EP-3 (2026-10-07):** parked in EP-5's brief notes.")
        );
        assert_eq!(messages[4].text, "No parenthesis at all.");
        assert_eq!(from_of("from beta, 2026-10-02"), Some("beta".to_owned()));
        assert_eq!(from_of("2026-10-02"), None);
        assert_eq!(from_of("urgent"), Some("urgent".to_owned()));
    }

    #[test]
    fn a_tick_writes_the_disposition_on_the_line() {
        let scratch = Scratch::new("inbox-tick");
        let folder = scratch.0.join("alpha");
        fs::create_dir_all(folder.join("roadmap")).unwrap();
        fs::write(path(&folder), INBOX).unwrap();
        let backups = scratch.0.join("backups");
        assert!(matches!(
            tick(&folder, "IN-2", " \n ", &backups, "2026-11-01"),
            Err(checklist::Error::NoDisposition(_))
        ));
        assert!(!backups.exists(), "a refused tick copies nothing");
        let copy = tick(
            &folder,
            "in-2",
            "declined:\nnothing to build",
            &backups,
            "2026-11-01",
        )
        .unwrap();
        assert_eq!(copy, backups.join("alpha").join("inbox.2026-11-01.md"));
        let text = fs::read_to_string(path(&folder)).unwrap();
        assert_eq!(
            text,
            INBOX.replace(
                "- [ ] IN-2 (from the owner, workspace split, 2026-10-04) Sessions start from the project's own\n  workspace file now; nothing to build.",
                "- [x] IN-2 (from the owner, workspace split, 2026-10-04) Sessions start from the project's own\n  workspace file now; nothing to build. **Disposition (2026-11-01):** declined: nothing to build"
            )
        );
        let inbox = read(&folder).unwrap().unwrap();
        let ticked = &inbox.messages[1];
        assert!(ticked.done);
        assert_eq!(
            ticked.disposition.as_deref(),
            Some("**Disposition (2026-11-01):** declined: nothing to build")
        );
        assert_eq!(inbox.open().count(), 2);
        assert!(matches!(
            tick(&folder, "IN-2", "again", &backups, "2026-11-01"),
            Err(checklist::Error::AlreadyTicked(_))
        ));
        assert_eq!(read(&scratch.0).unwrap(), None);
    }

    #[test]
    fn handed_on_blocks_resolve_against_the_registry() {
        let text = "\
# EP-2 — Registry

> **Completion note (2026-10-03).** One session.

> **Handed on (to Beta).** The registry's format is 2 from here.
> **Handed on (to beta EP-6).** Read it before your EP-6.
> **Handed on (to delta).** A project the registry does not know.
> **Handed on (to the owner).** An owner task in the old form.
> **Handed on.** For the next session here.
";
        let entry = Entry {
            file: BriefFile {
                id: ep(2),
                name: "EP-2-registry.md".to_owned(),
                path: PathBuf::from("EP-2-registry.md"),
            },
            brief: brief::parse(text),
            handoff: None,
        };
        let registered = ["alpha", "beta"];
        let resolve = |name: &str| {
            registered
                .iter()
                .find(|known| known.eq_ignore_ascii_case(name))
                .map(|known| (*known).to_owned())
        };
        let found = outgoing(&[entry], resolve);
        let summary: Vec<_> = found
            .iter()
            .map(|o| (o.brief, o.line, o.to.as_str(), o.recipient.clone()))
            .collect();
        assert_eq!(
            summary,
            [
                (ep(2), 5, "Beta", Recipient::Project("beta".to_owned())),
                (ep(2), 6, "beta EP-6", Recipient::Project("beta".to_owned())),
                (ep(2), 7, "delta", Recipient::Unknown),
            ]
        );
        assert!(found[0].text.starts_with("**Handed on (to Beta).**"));
    }
}
