//! The mailman (D-13; EP-20): a session's `> **Handed on (to <project>).**` blocks, delivered
//! into the receiver's `roadmap/inbox.md` as `- [ ] IN-n (from <sender> EP-k, <date>) <text>`.
//!
//! A block of a registered target is a [`Letter`], known by a key made of its brief and a hash of
//! its receiver and words, so that a letter goes out once and a block whose words change is a new
//! one. A letter goes out only once its brief is committed, so that a draft of a completion note
//! is never sent, and only while no session works or waits in the receiver, which is never
//! disturbed (D-13); otherwise it waits for the next round. The mailman's first reading of a
//! project counts its blocks as delivered, since the owner carried the messages by hand until
//! then. The log is the ledger's `event` rows of kind [`EVENT_KIND`], under the sender; each
//! delivery is also a `tick` row of the receiver, which a revert of a session there redoes
//! (EP-15b's rule for the tool's own writes).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use super::{Outgoing, PREFIX, Recipient, parse, path as inbox_path};
use crate::checklist;
use crate::files;
use crate::ledger::{EventRow, Ledger};
use crate::revert::{self, Tick};
use crate::roadmap::EpId;

/// The `event` rows of the mail: the first reading, the letters counted then, the deliveries.
pub const EVENT_KIND: &str = "mail";

/// The column a delivered message wraps at.
const WIDTH: usize = 100;

/// The header of an `inbox.md` the mailman or the formalise step creates, in the house form
/// (D-13), citing the decision as the owner's hand-carried inboxes do, since a bare number would
/// name the receiver's own (EP-21).
pub const HEADER: &str = "# Inbox — messages from other projects

Read right after the roadmap at the start of a session. Triage rule (clauderoadmapwrangler D-13):
for each open message, roll it into this session if it fits the brief and is at most S-sized;
otherwise add it to the brief notes of a named later EP or propose a new row in the right phase,
confirming the choice with the owner; or decline with a reason. Write the disposition on the
message's line and tick its box. One message per line,
`- [ ] IN-n (from <project> EP-k, date) text`, delivered by the tool.
";

/// A `Handed on (to <project>)` block of a registered target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Letter {
    pub sender: String,
    pub brief: EpId,
    /// The brief's file.
    pub path: PathBuf,
    /// The block's first line in the brief.
    pub line: usize,
    /// The receiver's registry name.
    pub receiver: String,
    /// The block's words without its label, its lines joined.
    pub text: String,
    pub key: String,
}

impl Letter {
    /// The parenthesis of the delivered line: `from alpha EP-6, 2026-11-10`.
    pub fn meta(&self, today: &str) -> String {
        format!("from {} {}, {today}", self.sender, self.brief)
    }
}

/// The letters among `outgoing`, the blocks of `sender`'s briefs; a block to a target the
/// registry does not know is no letter (shown, not delivered, D-13).
pub fn letters(sender: &str, outgoing: &[Outgoing]) -> Vec<Letter> {
    outgoing
        .iter()
        .filter_map(|block| {
            let Recipient::Project(receiver) = &block.recipient else {
                return None;
            };
            let text = body(&block.text);
            Some(Letter {
                sender: sender.to_owned(),
                brief: block.brief,
                path: block.path.clone(),
                line: block.line,
                key: key(block.brief, receiver, &text),
                receiver: receiver.clone(),
                text,
            })
        })
        .filter(|letter| !letter.text.is_empty())
        .collect()
}

/// A block's words without its bold label (`**Handed on (to …).**`, `**Parked and handed on (to
/// …).**`), on one line. The letter ends where a later paragraph opens with a bold label of its own
/// (`**Residual risk.**`, `**Notes for later sessions.**`), which the brief's parser keeps in the
/// same quote (EP-20's review).
pub fn body(text: &str) -> String {
    let text = text.trim_start();
    let text = text
        .strip_prefix("**")
        .and_then(|rest| rest.find("**").map(|end| &rest[end + 2..]))
        .unwrap_or(text)
        .trim_start_matches(['.', ':'])
        .trim_start();
    let mut kept = Vec::new();
    for (n, paragraph) in text.split("\n\n").enumerate() {
        if n > 0 && paragraph.trim_start().starts_with("**") {
            break;
        }
        kept.push(paragraph);
    }
    words(&kept.join(" "))
}

/// `text` with its whitespace runs made single spaces.
fn words(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A letter's key: its brief and a hash of its receiver and words (`EP-6:9f3a…`).
pub fn key(brief: EpId, receiver: &str, text: &str) -> String {
    format!(
        "{brief}:{:016x}",
        fnv(&format!("{}\n{}", receiver.to_lowercase(), words(text)))
    )
}

/// FNV-1a over the bytes: stable from one build to the next, as a stored key needs.
fn fnv(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The delivered message: `- [ ] IN-<id> (<meta>) <text>`, wrapped at [`WIDTH`] columns onto
/// continuation lines indented by two spaces. A word that would open a markdown block at a line's
/// start (a list marker, a quote, a heading) stays on the line before.
pub fn line(id: u32, meta: &str, text: &str) -> String {
    let mut lines = vec![format!("- [ ] {PREFIX}-{id} ({meta})")];
    let mut placed = false;
    for word in text.split_whitespace() {
        let current = lines.last_mut().expect("the first line is there");
        let fits = current.chars().count() + 1 + word.chars().count() <= WIDTH;
        // The first word stays beside the parenthesis, where the parser reads the text from.
        if fits || !placed || opens_block(word) {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(format!("  {word}"));
        }
        placed = true;
    }
    lines.join("\n")
}

/// Whether `word` at a line's start would open a markdown block of its own.
fn opens_block(word: &str) -> bool {
    matches!(word, "-" | "*" | "+")
        || word.starts_with(['>', '#', '|'])
        || word
            .strip_suffix(['.', ')'])
            .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// The number of an `IN-n` id.
fn number(id: &str) -> Option<u32> {
    id.get(PREFIX.len() + 1..)?.parse().ok()
}

/// A message appended to an inbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Appended {
    /// `IN-4`.
    pub id: String,
    /// The line as written, from its box to its end.
    pub written: String,
    /// The copy of the file as it was; none when the file was created.
    pub copy: Option<PathBuf>,
}

/// Appends a message (`meta`, `text`) to the inbox of the project in `folder` with the next free
/// id: the file as it was copied to `<backups>\<project folder>\inbox.<date>.md`, the new text
/// written atomically, a missing file created with [`HEADER`].
pub fn append(
    folder: &Path,
    meta: &str,
    text: &str,
    backups: &Path,
    today: &str,
) -> io::Result<Appended> {
    let path = inbox_path(folder);
    let _writing = super::writing();
    let old = match fs::read_to_string(&path) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };
    let base = old.as_deref().unwrap_or(HEADER);
    let (messages, _) = parse(base);
    let next = messages
        .iter()
        .filter_map(|message| number(&message.id))
        .max()
        .unwrap_or(0)
        + 1;
    let eol = if base.contains("\r\n") { "\r\n" } else { "\n" };
    let written = line(next, meta, text);
    let mut new = base.trim_end().to_owned();
    new.push_str(eol);
    // Straight after the list when the file ends in it; after a blank line otherwise (a header
    // alone, or a paragraph after the list), so the message opens a list of its own.
    let last = new.trim_end().lines().last().unwrap_or_default();
    let in_list = !messages.is_empty() && (last.starts_with("- ") || last.starts_with("  "));
    if !in_list {
        new.push_str(eol);
    }
    new.push_str(&written.replace('\n', eol));
    new.push_str(eol);
    let copy = match &old {
        Some(old) => Some(files::dated_copy(
            old.as_bytes(),
            &checklist::copies_dir(&path, backups),
            "inbox",
            today,
            ".md",
        )?),
        None => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            None
        }
    };
    files::write_atomic(&path, new.as_bytes())?;
    Ok(Appended {
        id: format!("{PREFIX}-{next}"),
        written,
        copy,
    })
}

/// The id of a message of `text` (an inbox's) from `meta`'s sender and brief with these words,
/// open or ticked: a letter that is there already, delivered or carried by hand.
fn already_there(text: &str, meta: &str, words_of: &str) -> Option<String> {
    let from = meta_from(meta);
    let wanted = words(words_of);
    parse(text).0.into_iter().find_map(|message| {
        let same_from = message
            .from
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case(&from));
        (same_from && words(&message.text) == wanted).then_some(message.id)
    })
}

/// `from alpha EP-6, 2026-11-10` → `alpha EP-6`, as `inbox::parse` reads a message's `from`.
fn meta_from(meta: &str) -> String {
    let body = meta.trim();
    let body = body.strip_prefix("from ").unwrap_or(body);
    let body = body.rsplit_once(',').map_or(body, |(before, _)| before);
    body.trim().to_owned()
}

/// Redoes a delivery after a revert (EP-15b): `words` as the `tick` row has them, `(<meta>)
/// <text>`, appended again unless the message is there. `Ok(None)` when it was.
pub fn redeliver(
    folder: &Path,
    words_logged: &str,
    backups: &Path,
    today: &str,
) -> Result<Option<String>, String> {
    let rest = words_logged
        .trim()
        .strip_prefix('(')
        .ok_or("the log does not hold the message's parenthesis")?;
    let close = rest
        .find(')')
        .ok_or("the log does not hold the message's parenthesis")?;
    let (meta, text) = (&rest[..close], rest[close + 1..].trim());
    let inbox = fs::read_to_string(inbox_path(folder)).unwrap_or_default();
    if already_there(&inbox, meta, text).is_some() {
        return Ok(None);
    }
    append(folder, meta, text, backups, today)
        .map(|appended| Some(appended.id))
        .map_err(|e| e.to_string())
}

/// What the mail log knows of a letter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Known {
    /// Counted as delivered at the mailman's first reading of its sender.
    Counted,
    Delivered {
        receiver: String,
        id: String,
    },
}

/// The mail log, read from the ledger: the senders read once, and each letter's key.
#[derive(Debug, Clone, Default)]
pub struct Log {
    read: HashSet<String>,
    known: HashMap<(String, String), Known>,
}

impl Log {
    /// The log from the `mail` rows, any order.
    pub fn from_events(events: &[EventRow]) -> Self {
        let mut log = Self::default();
        for event in events {
            let (Some(project), Some(detail)) = (&event.project, &event.detail) else {
                continue;
            };
            let sender = project.to_lowercase();
            if detail.starts_with(FIRST_READING) {
                log.read.insert(sender);
                continue;
            }
            let mut parts = detail.split_whitespace();
            let (Some(verb), Some(key)) = (parts.next(), parts.next()) else {
                continue;
            };
            let known = match verb {
                "counted" => Known::Counted,
                "delivered" => {
                    // `delivered <key> to <receiver> as IN-n`
                    let rest: Vec<&str> = parts.collect();
                    let id = rest.last().copied().unwrap_or_default().to_owned();
                    let receiver = rest
                        .iter()
                        .skip(1)
                        .take(rest.len().saturating_sub(3))
                        .copied()
                        .collect::<Vec<_>>()
                        .join(" ");
                    Known::Delivered { receiver, id }
                }
                _ => continue,
            };
            log.known.insert((sender, key.to_owned()), known);
        }
        log
    }

    pub fn read(ledger: &Ledger) -> Result<Self, String> {
        ledger
            .events(EVENT_KIND, None)
            .map(|events| Self::from_events(&events))
            .map_err(|e| format!("the mail log did not read: {e}"))
    }

    /// Whether the mailman has read `sender` once.
    pub fn has_read(&self, sender: &str) -> bool {
        self.read.contains(&sender.to_lowercase())
    }

    pub fn known(&self, letter: &Letter) -> Option<&Known> {
        self.known
            .get(&(letter.sender.to_lowercase(), letter.key.clone()))
    }
}

/// The words a first reading's row opens with.
const FIRST_READING: &str = "first reading";

/// What became of a letter in a round, or of a block that is no letter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Counted,
    Delivered {
        receiver: String,
        id: String,
    },
    /// Not sent yet, and why.
    Waits(String),
    /// Its target is not registered: shown, not delivered.
    Unknown,
}

impl State {
    /// The Detail pane's and `crw mail`'s words.
    pub fn words(&self) -> String {
        match self {
            State::Counted => "counted as delivered at the mailman's first reading".to_owned(),
            State::Delivered { receiver, id } => format!("delivered to {receiver} as {id}"),
            State::Waits(why) => format!("waits: {why}"),
            State::Unknown => "not registered: shown, not delivered".to_owned(),
        }
    }
}

/// A block's state, by its brief's file and its line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mark {
    pub sender: String,
    pub path: PathBuf,
    pub line: usize,
    pub state: State,
}

/// A project whose briefs may hand mail on.
pub struct Sender<'a> {
    pub name: &'a str,
    pub folder: &'a Path,
    pub outgoing: &'a [Outgoing],
}

/// What a round needs besides the senders.
pub struct Round<'a> {
    /// A registered project's folder, by name.
    pub folder_of: &'a dyn Fn(&str) -> Option<PathBuf>,
    /// The changed paths of a sender's working tree (git's status, `/` separated), or why git
    /// could not say: asked only of a sender with a letter to send.
    pub changes: &'a mut dyn FnMut(&Path) -> Result<Vec<String>, String>,
    /// Why a receiver must wait: a session works or waits there.
    pub busy: &'a dyn Fn(&str) -> Option<String>,
    pub backups: &'a Path,
    pub today: &'a str,
    pub now: DateTime<Utc>,
}

/// What a round did.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub marks: Vec<Mark>,
    /// The deliveries' words, for the status bar.
    pub delivered: Vec<String>,
    /// What went wrong, in words.
    pub problems: Vec<String>,
}

/// One round of the mailman over `senders`, its log the ledger's: a first reading counted, each
/// letter not yet in the log sent when its brief is committed and its receiver free.
pub fn round(ledger: &Ledger, senders: &[Sender<'_>], cx: &mut Round<'_>) -> Report {
    let mut report = Report::default();
    let log = match Log::read(ledger) {
        Ok(log) => log,
        Err(why) => {
            report.problems.push(why);
            return report;
        }
    };
    // The projects read for the first time this round. Mail to one of them written before (it
    // was registered later, or its folder came back) is its past too (EP-20's review).
    let fresh_receivers: HashSet<String> = senders
        .iter()
        .filter(|sender| !log.has_read(sender.name))
        .map(|sender| sender.name.to_lowercase())
        .collect();
    for sender in senders {
        for block in sender
            .outgoing
            .iter()
            .filter(|b| b.recipient == Recipient::Unknown)
        {
            report.marks.push(Mark {
                sender: sender.name.to_owned(),
                path: block.path.clone(),
                line: block.line,
                state: State::Unknown,
            });
        }
        let letters = letters(sender.name, sender.outgoing);
        if !log.has_read(sender.name) {
            // Only a committed brief's blocks are the past: a brief a session is still writing
            // hands its letters on later, once committed.
            let changes = if letters.is_empty() {
                Ok(Vec::new())
            } else {
                (cx.changes)(sender.folder)
            };
            if let Err(why) = &changes {
                // Read again next round, when git may answer.
                for letter in &letters {
                    report.marks.push(Mark {
                        sender: sender.name.to_owned(),
                        path: letter.path.clone(),
                        line: letter.line,
                        state: State::Waits(format!("git could not say what is committed: {why}")),
                    });
                }
                continue;
            }
            let (past, fresh): (Vec<Letter>, Vec<Letter>) = letters
                .into_iter()
                .partition(|letter| committed(sender, letter, &changes).is_ok());
            first_reading(ledger, sender.name, &past, cx.now, &mut report);
            for letter in fresh {
                report.marks.push(Mark {
                    sender: sender.name.to_owned(),
                    path: letter.path,
                    line: letter.line,
                    state: State::Waits("its brief is not committed yet".to_owned()),
                });
            }
            continue;
        }
        let mut changes: Option<Result<Vec<String>, String>> = None;
        for letter in &letters {
            let state = match log.known(letter) {
                Some(Known::Counted) => State::Counted,
                Some(Known::Delivered { receiver, id }) => State::Delivered {
                    receiver: receiver.clone(),
                    id: id.clone(),
                },
                None if fresh_receivers.contains(&letter.receiver.to_lowercase()) => {
                    let row = format!("counted {}", letter.key);
                    match ledger.add_event(cx.now, EVENT_KIND, Some(&letter.sender), &row) {
                        Ok(_) => State::Counted,
                        Err(e) => {
                            let why = format!("the mail log did not keep a count: {e}");
                            report.problems.push(why.clone());
                            State::Waits(why)
                        }
                    }
                }
                None => {
                    let changed = changes.get_or_insert_with(|| (cx.changes)(sender.folder));
                    send(ledger, sender, letter, changed, cx, &mut report)
                }
            };
            report.marks.push(Mark {
                sender: sender.name.to_owned(),
                path: letter.path.clone(),
                line: letter.line,
                state,
            });
        }
    }
    report
}

/// The first reading of `sender`: its letters counted as delivered, none sent.
fn first_reading(
    ledger: &Ledger,
    sender: &str,
    letters: &[Letter],
    now: DateTime<Utc>,
    report: &mut Report,
) {
    let mut rows = vec![format!(
        "{FIRST_READING}: {} block{} counted as delivered",
        letters.len(),
        if letters.len() == 1 { "" } else { "s" }
    )];
    rows.extend(
        letters
            .iter()
            .map(|letter| format!("counted {}", letter.key)),
    );
    // The counted letters first, so that a failure halfway reads the sender again next round.
    rows.rotate_left(1);
    for row in rows {
        if let Err(e) = ledger.add_event(now, EVENT_KIND, Some(sender), &row) {
            report.problems.push(format!(
                "the mail log did not keep {sender}'s first reading: {e}"
            ));
            return;
        }
    }
    for letter in letters {
        report.marks.push(Mark {
            sender: sender.to_owned(),
            path: letter.path.clone(),
            line: letter.line,
            state: State::Counted,
        });
    }
}

/// Whether `letter`'s brief is committed by git's `changes` of its sender, or why it waits.
fn committed(
    sender: &Sender<'_>,
    letter: &Letter,
    changes: &Result<Vec<String>, String>,
) -> Result<(), State> {
    let changed = changes.as_ref().map_err(|why| {
        State::Waits(format!(
            "git could not say whether its brief is committed: {why}"
        ))
    })?;
    let Ok(relative) = letter.path.strip_prefix(sender.folder) else {
        return Err(State::Waits(
            "its brief lies outside the project's folder".to_owned(),
        ));
    };
    let relative = relative.to_string_lossy().replace('\\', "/");
    // A path git lists, or an untracked folder (`roadmap/`) that holds the brief.
    let uncommitted = changed.iter().any(|path| {
        *path == relative || path.ends_with('/') && relative.starts_with(path.as_str())
    });
    if uncommitted {
        return Err(State::Waits("its brief is not committed yet".to_owned()));
    }
    Ok(())
}

/// Sends `letter` when its brief is committed and its receiver free.
fn send(
    ledger: &Ledger,
    sender: &Sender<'_>,
    letter: &Letter,
    changes: &Result<Vec<String>, String>,
    cx: &Round<'_>,
    report: &mut Report,
) -> State {
    if let Err(waits) = committed(sender, letter, changes) {
        return waits;
    }
    let Some(folder) = (cx.folder_of)(&letter.receiver) else {
        return State::Waits(format!("{}'s folder is missing", letter.receiver));
    };
    if let Some(why) = (cx.busy)(&letter.receiver) {
        return State::Waits(why);
    }
    let meta = letter.meta(cx.today);
    // In the process's turn, the time taken there (EP-20's review): a session's start, which
    // copies the project's files and takes its time in a turn too, then holds the message in its
    // copy or comes before the tick, so a revert of that session keeps it either way.
    let _turn = crate::process::turn();
    let inbox = fs::read_to_string(inbox_path(&folder)).unwrap_or_default();
    let (id, written) = match already_there(&inbox, &meta, &letter.text) {
        Some(id) => (id, None),
        None => match append(&folder, &meta, &letter.text, cx.backups, cx.today) {
            Ok(appended) => (appended.id.clone(), Some(appended)),
            Err(e) => {
                let why = format!("{}'s inbox could not be written: {e}", letter.receiver);
                report.problems.push(why.clone());
                return State::Waits(why);
            }
        },
    };
    let at = Utc::now();
    let row = format!("delivered {} to {} as {id}", letter.key, letter.receiver);
    if let Err(e) = ledger.add_event(at, EVENT_KIND, Some(&letter.sender), &row) {
        report
            .problems
            .push(format!("the mail log did not keep a delivery: {e}"));
    }
    if let Some(appended) = &written {
        // The tool's own write into the receiver, which a revert there redoes (EP-15b).
        let tick = Tick::delivery(at, &appended.id, &format!("({meta}) {}", letter.text));
        if let Err(e) = ledger.add_event(
            tick.at,
            revert::TICK_KIND,
            Some(&letter.receiver),
            &tick.detail(),
        ) {
            report.problems.push(format!(
                "the ledger did not log a delivery for a revert: {e}"
            ));
        }
        report.delivered.push(format!(
            "{id} to {}, from {} {}",
            letter.receiver, letter.sender, letter.brief
        ));
    }
    State::Delivered {
        receiver: letter.receiver.clone(),
        id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;
    use crate::roadmap::snapshot;

    const BRIEF: &str = "\
# EP-6 — Export

> **Completion note (2026-11-10).** One session.

> **Handed on (to beta).** The export gains a column; read it
> before your EP-3.
> **Handed on (to delta).** Nobody by that name.
";

    /// A project `alpha` whose EP-6 hands a letter to `beta`, and `beta` with an inbox.
    fn portfolio(scratch: &Scratch) -> (PathBuf, PathBuf) {
        let alpha = scratch.0.join("alpha");
        let beta = scratch.0.join("beta");
        fs::create_dir_all(alpha.join("roadmap")).unwrap();
        fs::create_dir_all(beta.join("roadmap")).unwrap();
        fs::write(alpha.join("roadmap").join("EP-6-export.md"), BRIEF).unwrap();
        fs::write(
            inbox_path(&beta),
            "# Inbox\n\nOne message per line.\n\n- [x] IN-2 (from the owner, 2026-11-01) Old. **Disposition (2026-11-02):** done.\n",
        )
        .unwrap();
        (alpha, beta)
    }

    fn outgoing_of(folder: &Path) -> Vec<Outgoing> {
        let snapshot = snapshot::read_tree(folder);
        super::super::outgoing(&snapshot.briefs, |name| {
            ["alpha", "beta"]
                .iter()
                .find(|known| known.eq_ignore_ascii_case(name))
                .map(|known| (*known).to_owned())
        })
    }

    #[test]
    fn a_letter_is_the_block_without_its_label_on_one_line() {
        let scratch = Scratch::new("mail-letter");
        let (alpha, _) = portfolio(&scratch);
        let letters = letters("alpha", &outgoing_of(&alpha));
        assert_eq!(letters.len(), 1, "the unknown target is no letter");
        let letter = &letters[0];
        assert_eq!(letter.receiver, "beta");
        assert_eq!(
            letter.text,
            "The export gains a column; read it before your EP-3."
        );
        assert_eq!(letter.meta("2026-11-10"), "from alpha EP-6, 2026-11-10");
        assert_eq!(
            letter.key,
            key(
                EpId::new(6),
                "Beta",
                " The export gains a column;\nread it before your EP-3."
            )
        );
        assert_ne!(letter.key, key(EpId::new(6), "gamma", &letter.text));
        assert!(letter.key.starts_with("EP-6:"));
    }

    #[test]
    fn a_long_message_wraps_and_never_opens_a_block() {
        // The first line is 40 columns; five words of nine and one of eight bring it to 99, so
        // the list marker and the numbered item after it fall exactly where it wraps.
        let text = "abcdefghi abcdefghi abcdefghi abcdefghi abcdefghi abcdefgh - 1. next words";
        let written = line(4, "from alpha EP-6, 2026-11-10", text);
        let lines: Vec<&str> = written.lines().collect();
        assert_eq!(lines.len(), 2, "{written}");
        assert!(lines[0].starts_with("- [ ] IN-4 (from alpha EP-6, 2026-11-10) abcdefghi"));
        assert!(lines[0].ends_with("abcdefgh - 1."), "{written}");
        assert_eq!(lines[1], "  next words");
        let (messages, problems) = parse(&written);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(messages.len(), 1);
        assert_eq!(words(&messages[0].text), text);
    }

    #[test]
    fn a_letter_ends_at_a_labelled_paragraph_and_drops_any_label() {
        assert_eq!(
            body(
                "**Handed on (to beta).** One note\nover two lines.\n\n**Residual risk.** Not mail."
            ),
            "One note over two lines."
        );
        assert_eq!(
            body("**Parked and handed on (to beta).** A parked note.\n\nIts second paragraph."),
            "A parked note. Its second paragraph."
        );
        assert_eq!(
            body("**Handed on (to beta)**. After the label."),
            "After the label."
        );
    }

    #[test]
    fn mail_to_a_project_read_for_the_first_time_is_its_past() {
        let scratch = Scratch::new("mail-fresh-receiver");
        let (alpha, _) = portfolio(&scratch);
        let gamma = scratch.0.join("gamma");
        fs::create_dir_all(gamma.join("roadmap")).unwrap();
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        ledger
            .add_event(
                Utc::now(),
                EVENT_KIND,
                Some("alpha"),
                "first reading: 0 blocks counted as delivered",
            )
            .unwrap();
        // alpha, read before, wrote to gamma while gamma was not registered.
        let outgoing = vec![Outgoing {
            brief: EpId::new(6),
            path: alpha.join("roadmap").join("EP-6-export.md"),
            line: 9,
            to: "gamma".to_owned(),
            recipient: Recipient::Project("gamma".to_owned()),
            text: "**Handed on (to gamma).** An old note.".to_owned(),
        }];
        let backups = scratch.0.join("backups");
        let folders = |name: &str| (name == "gamma").then(|| gamma.clone());
        let mut changes = |_: &Path| Ok::<_, String>(Vec::new());
        let free = |_: &str| None;
        let mut cx = Round {
            folder_of: &folders,
            changes: &mut changes,
            busy: &free,
            backups: &backups,
            today: "2026-11-10",
            now: Utc::now(),
        };
        let senders = [
            Sender {
                name: "alpha",
                folder: &alpha,
                outgoing: &outgoing,
            },
            Sender {
                name: "gamma",
                folder: &gamma,
                outgoing: &[],
            },
        ];
        let report = round(&ledger, &senders, &mut cx);
        assert!(report.delivered.is_empty(), "{:?}", report.delivered);
        assert_eq!(report.marks[0].state, State::Counted);
        assert!(!inbox_path(&gamma).exists(), "nothing written into gamma");
        assert!(Log::read(&ledger).unwrap().has_read("gamma"));
    }

    #[test]
    fn an_append_takes_the_next_id_and_keeps_a_copy() {
        let scratch = Scratch::new("mail-append");
        let (_, beta) = portfolio(&scratch);
        let backups = scratch.0.join("backups");
        let appended = append(
            &beta,
            "from alpha EP-6, 2026-11-10",
            "A note.",
            &backups,
            "2026-11-10",
        )
        .unwrap();
        assert_eq!(appended.id, "IN-3");
        assert_eq!(
            appended.copy,
            Some(backups.join("beta").join("inbox.2026-11-10.md"))
        );
        let text = fs::read_to_string(inbox_path(&beta)).unwrap();
        assert!(
            text.ends_with("done.\n- [ ] IN-3 (from alpha EP-6, 2026-11-10) A note.\n"),
            "{text}"
        );
        // A project with no inbox gets one with the header.
        let gamma = scratch.0.join("gamma");
        let first = append(
            &gamma,
            "from alpha EP-6, 2026-11-10",
            "Hello.",
            &backups,
            "2026-11-10",
        )
        .unwrap();
        assert_eq!((first.id.as_str(), first.copy), ("IN-1", None));
        let text = fs::read_to_string(inbox_path(&gamma)).unwrap();
        assert!(text.starts_with(HEADER.trim_end()));
        assert!(
            text.ends_with("tool.\n\n- [ ] IN-1 (from alpha EP-6, 2026-11-10) Hello.\n"),
            "{text}"
        );
        let (messages, problems) = parse(&text);
        assert!(problems.is_empty() && messages.len() == 1, "{problems:?}");
        // Redone after a revert: not twice.
        assert_eq!(
            redeliver(
                &gamma,
                "(from alpha EP-6, 2026-11-10) Hello.",
                &backups,
                "2026-11-11"
            )
            .unwrap(),
            None
        );
        fs::write(inbox_path(&gamma), HEADER).unwrap();
        assert_eq!(
            redeliver(
                &gamma,
                "(from alpha EP-6, 2026-11-10) Hello.",
                &backups,
                "2026-11-11"
            )
            .unwrap(),
            Some("IN-1".to_owned())
        );
    }

    #[test]
    fn a_round_counts_the_first_reading_then_delivers_once() {
        let scratch = Scratch::new("mail-round");
        let (alpha, beta) = portfolio(&scratch);
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let backups = scratch.0.join("backups");
        let folders = |name: &str| match name {
            "alpha" => Some(scratch.0.join("alpha")),
            "beta" => Some(scratch.0.join("beta")),
            _ => None,
        };
        let now = Utc::now();
        let run = |busy: Option<&str>, changed: Vec<String>| {
            let outgoing = outgoing_of(&alpha);
            let mut asked = 0;
            let mut changes = |_: &Path| {
                asked += 1;
                Ok::<_, String>(changed.clone())
            };
            let busy_words = busy.map(str::to_owned);
            let busy_of = |name: &str| busy_words.clone().filter(|_| name == "beta");
            let mut cx = Round {
                folder_of: &folders,
                changes: &mut changes,
                busy: &busy_of,
                backups: &backups,
                today: "2026-11-10",
                now,
            };
            let senders = [Sender {
                name: "alpha",
                folder: &alpha,
                outgoing: &outgoing,
            }];
            let report = round(&ledger, &senders, &mut cx);
            (report, asked)
        };
        // A brief a session is still writing when the mailman first reads the project.
        let brief = alpha.join("roadmap").join("EP-7-more.md");
        fs::write(&brief, "# EP-7\n\n> **Completion note (2026-11-10).** Done.\n\n> **Handed on (to beta).** A second note.\n").unwrap();
        // The first reading counts the committed brief's letter and sends nothing; the brief
        // still being written is no past: its letter waits.
        let (report, asked) = run(None, vec!["roadmap/EP-7-more.md".to_owned()]);
        assert_eq!(asked, 1, "git asked once for the sender");
        let states: Vec<&State> = report.marks.iter().map(|m| &m.state).collect();
        let waits = State::Waits("its brief is not committed yet".to_owned());
        assert_eq!(states, [&State::Unknown, &State::Counted, &waits]);
        assert!(report.delivered.is_empty());
        // Still uncommitted at the next round: it waits.
        let (report, _) = run(None, vec!["roadmap/EP-7-more.md".to_owned()]);
        assert_eq!(report.marks.last().unwrap().state, waits);
        // A session works in the receiver: it waits.
        let (report, _) = run(Some("a session works there"), Vec::new());
        assert_eq!(
            report.marks.last().unwrap().state,
            State::Waits("a session works there".to_owned())
        );
        // Committed and free: delivered once, logged, and a tick for a revert.
        let (report, _) = run(None, Vec::new());
        assert_eq!(
            report.marks.last().unwrap().state,
            State::Delivered {
                receiver: "beta".to_owned(),
                id: "IN-3".to_owned()
            }
        );
        assert_eq!(report.delivered, ["IN-3 to beta, from alpha EP-7"]);
        let text = fs::read_to_string(inbox_path(&beta)).unwrap();
        assert!(
            text.ends_with("- [ ] IN-3 (from alpha EP-7, 2026-11-10) A second note.\n"),
            "{text}"
        );
        let ticks =
            revert::ticks_since(&ledger, "beta", now - chrono::TimeDelta::seconds(1)).unwrap();
        assert_eq!(ticks.len(), 1);
        assert_eq!(
            ticks[0].detail(),
            "delivered IN-3 · (from alpha EP-7, 2026-11-10) A second note."
        );
        // The next round sends nothing again and asks git nothing.
        let (report, asked) = run(None, Vec::new());
        assert_eq!(asked, 0);
        assert!(
            report.delivered.is_empty() && report.problems.is_empty(),
            "{:?}",
            report.problems
        );
        assert_eq!(
            report.marks.last().unwrap().state.words(),
            "delivered to beta as IN-3"
        );
        assert_eq!(fs::read_to_string(inbox_path(&beta)).unwrap(), text);
        let log = Log::read(&ledger).unwrap();
        assert!(log.has_read("Alpha"));
    }
}
