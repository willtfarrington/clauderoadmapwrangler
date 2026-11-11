//! The mailman (D-13; EP-20): a session's `> **Handed on (to <project>).**` blocks, delivered
//! into the receiver's `roadmap/inbox.md` as `- [ ] IN-n (from <sender> EP-k, <date>) <text>`.
//!
//! A block of a registered target is a [`Letter`], known by a key made of its brief and a hash of
//! its receiver and words, so that a letter goes out once and a block whose words change is a new
//! one, which says what it rewords (EP-20d). A letter goes out only once its words are committed
//! (its key among the letters of `HEAD`'s copy of its brief, EP-20d), so that a draft of a
//! completion note is never sent, and only while no session works or waits in the receiver,
//! which is never disturbed (D-13); otherwise it waits for the next round. The mailman's first
//! reading of a project counts its committed blocks as delivered, since the owner carried the
//! messages by hand until then. The log is the ledger's `event` rows of kind [`EVENT_KIND`], under
//! the sender; each delivery is also a `tick` row of the receiver, which a revert of a session
//! there redoes (EP-15b's rule for the tool's own writes).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use super::{Outgoing, PREFIX, Recipient, outgoing_of, parse, path as inbox_path};
use crate::checklist;
use crate::files;
use crate::ledger::{self, EventRow, Ledger};
use crate::revert::{self, Tick};
use crate::roadmap::{EpId, brief};

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

/// The next free id among `messages`, the highest plus one (1 for an empty inbox).
pub fn next_id(messages: &[super::Message]) -> u32 {
    messages
        .iter()
        .filter_map(|message| number(&message.id))
        .max()
        .unwrap_or(0)
        + 1
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
    let next = next_id(&messages);
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

/// The sender of a message the owner composes (D-13; EP-20c): `from the owner, <date>`.
pub const OWNER: &str = "the owner";

/// The flag at the end of a message that notifies the owner, not the session (D-13).
pub const URGENT: &str = "[urgent]";

/// The owner's message as the inbox line takes it: its words on one line, [`URGENT`] at the
/// end when flagged (where `inbox::parse` reads it and the Detail pane's chip says it), unless
/// the words hold one already, anywhere, as `inbox::parse` reads it, so that a flag typed
/// before punctuation is not doubled (EP-20c's review; EP-20d).
pub fn compose_text(text: &str, urgent: bool) -> String {
    let mut words = words(text);
    if urgent && !words.to_ascii_lowercase().contains(URGENT) {
        if !words.is_empty() {
            words.push(' ');
        }
        words.push_str(URGENT);
    }
    words
}

/// Whether the owner's message holds nothing beyond the flag and punctuation: no message.
pub fn is_empty_message(words: &str) -> bool {
    !words
        .to_ascii_lowercase()
        .replace(URGENT, " ")
        .chars()
        .any(char::is_alphanumeric)
}

/// Whether a revert of a session in the receiver would put the message back (EP-20c's review;
/// EP-20d): its `tick` row, `delivered <id> · (<meta>) <words>`, fits the ledger's cap; a
/// longer one is cut there, so the revert names the message instead.
pub fn revert_keeps(id: &str, meta: &str, words: &str) -> bool {
    let tick = Tick::delivery(Utc::now(), id, &format!("({meta}) {words}"));
    tick.detail().chars().count() < ledger::DETAIL_CAP
}

/// The parenthesis of the owner's message: `from the owner, 2026-11-10`.
pub fn owner_meta(today: &str) -> String {
    format!("from {OWNER}, {today}")
}

/// The ledger's `mail` row of a message the owner composed, under the receiver.
const WRITTEN: &str = "written";

/// A message the owner composed, written (EP-20c).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sent {
    pub id: String,
    /// The line as written.
    pub written: String,
    /// The copy of the inbox as it was; none when the file was created.
    pub copy: Option<PathBuf>,
    /// What the ledger did not keep, when it did not: the write stands all the same.
    pub unlogged: Option<String>,
    /// Whether a revert there would put it back ([`revert_keeps`]); a longer message is named.
    pub revert_keeps: bool,
}

/// Writes the owner's message into the inbox of `receiver` (the project in `folder`), in the
/// process's turn as a delivery is, and logs it in the ledger at `ledger` when there is one: a
/// `tick` row of the receiver, which a revert of a session there redoes (EP-15b's rule for the
/// tool's own writes), and a `mail` row (`written IN-n from the owner`). An empty message
/// ([`is_empty_message`]) is refused.
pub fn send_from_owner(
    folder: &Path,
    receiver: &str,
    text: &str,
    urgent: bool,
    backups: &Path,
    today: &str,
    ledger: Option<&Path>,
) -> Result<Sent, String> {
    let words = compose_text(text, urgent);
    if is_empty_message(&words) {
        return Err("the message is empty".to_owned());
    }
    let meta = owner_meta(today);
    let appended = {
        let _turn = crate::process::turn();
        append(folder, &meta, &words, backups, today)
            .map_err(|e| format!("{}: {e}", inbox_path(folder).display()))?
    };
    let at = Utc::now();
    let unlogged = ledger.and_then(|path| {
        let logged = Ledger::open(path).and_then(|ledger| {
            let tick = Tick::delivery(at, &appended.id, &format!("({meta}) {words}"));
            ledger.add_event(tick.at, revert::TICK_KIND, Some(receiver), &tick.detail())?;
            let row = format!("{WRITTEN} {} from {OWNER}", appended.id);
            ledger.add_event(at, EVENT_KIND, Some(receiver), &row)
        });
        logged
            .err()
            .map(|e| format!("the ledger did not log the message: {e}"))
    });
    Ok(Sent {
        revert_keeps: revert_keeps(&appended.id, &meta, &words),
        id: appended.id,
        written: appended.written,
        copy: appended.copy,
        unlogged,
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

/// A delivery redone after a revert: the message's id now, which may differ from the one the
/// log holds (EP-20c, the review's point (d)), and whether it was appended again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redelivered {
    pub id: String,
    pub appended: bool,
}

/// Redoes a delivery after a revert (EP-15b): `words` as the `tick` row has them, `(<meta>)
/// <text>`, appended again unless the message is there, found by its sender and words.
pub fn redeliver(
    folder: &Path,
    words_logged: &str,
    backups: &Path,
    today: &str,
) -> Result<Redelivered, String> {
    let rest = words_logged
        .trim()
        .strip_prefix('(')
        .ok_or("the log does not hold the message's parenthesis")?;
    let close = rest
        .find(')')
        .ok_or("the log does not hold the message's parenthesis")?;
    let (meta, text) = (&rest[..close], rest[close + 1..].trim());
    let inbox = fs::read_to_string(inbox_path(folder)).unwrap_or_default();
    if let Some(id) = already_there(&inbox, meta, text) {
        return Ok(Redelivered {
            id,
            appended: false,
        });
    }
    append(folder, meta, text, backups, today)
        .map(|appended| Redelivered {
            id: appended.id,
            appended: true,
        })
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

/// The mail log, read from the ledger: the senders read once, the projects whose mail was
/// counted while their own first reading waited, each letter's key and what it reworded.
#[derive(Debug, Clone, Default)]
pub struct Log {
    read: HashSet<String>,
    received: HashSet<String>,
    known: HashMap<(String, String), Known>,
    rewords: HashMap<(String, String), Vec<String>>,
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
            if detail.starts_with(MAIL_PAST) {
                log.received.insert(sender);
                continue;
            }
            let mut parts = detail.split_whitespace();
            let (Some(verb), Some(key)) = (parts.next(), parts.next()) else {
                continue;
            };
            let known = match verb {
                REWORDING => {
                    // `rewording <key> of IN-4, IN-6`
                    let ids = parts
                        .map(|word| word.trim_end_matches(','))
                        .filter(|word| number(word).is_some())
                        .map(str::to_owned)
                        .collect();
                    log.rewords.insert((sender, key.to_owned()), ids);
                    continue;
                }
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

    /// Whether the mail to `project` was counted as its past while its own first reading
    /// waited for a brief that did not read (EP-20d).
    pub fn has_received(&self, project: &str) -> bool {
        self.received.contains(&project.to_lowercase())
    }

    pub fn known(&self, letter: &Letter) -> Option<&Known> {
        self.known
            .get(&(letter.sender.to_lowercase(), letter.key.clone()))
    }

    /// The ids a delivered letter reworded, as its delivery recorded them.
    pub fn rewording_of(&self, letter: &Letter) -> Vec<String> {
        self.rewords
            .get(&(letter.sender.to_lowercase(), letter.key.clone()))
            .cloned()
            .unwrap_or_default()
    }

    /// What `letter` rewords as it goes out (EP-20d): the letters its sender delivered from the
    /// same brief to the same receiver whose words are gone from the sender's briefs (`current`,
    /// the keys of its letters now), by id; one another of them reworded already is left out,
    /// so a third version names the second, not both (the review).
    fn reworded(&self, letter: &Letter, current: &HashSet<String>) -> Vec<String> {
        let sender = letter.sender.to_lowercase();
        let brief = format!("{}:", letter.brief);
        let gone: Vec<(&String, &String)> = self
            .known
            .iter()
            .filter_map(|((from, key), known)| {
                let Known::Delivered { receiver, id } = known else {
                    return None;
                };
                let same = *from == sender
                    && key.starts_with(&brief)
                    && *key != letter.key
                    && !current.contains(key)
                    && receiver.eq_ignore_ascii_case(&letter.receiver);
                same.then_some((key, id))
            })
            .collect();
        let replaced: HashSet<&String> = gone
            .iter()
            .filter_map(|(key, _)| self.rewords.get(&(sender.clone(), (*key).clone())))
            .flatten()
            .collect();
        let mut ids: Vec<String> = gone
            .into_iter()
            .filter(|(_, id)| !replaced.contains(id))
            .map(|(_, id)| id.clone())
            .collect();
        ids.sort_by_key(|id| number(id));
        ids.dedup();
        ids
    }
}

/// The words a first reading's row opens with.
const FIRST_READING: &str = "first reading";

/// The words the row opens with that records a project's mail counted as its past while its own
/// first reading waits (EP-20d).
const MAIL_PAST: &str = "mail counted as its past";

/// The verb of the row that records what a delivered letter reworded (EP-20d).
const REWORDING: &str = "rewording";

/// `, rewording IN-4`, `, rewording IN-4 and IN-6`: what a delivered letter rewords, or nothing.
fn rewording_words(ids: &[String]) -> String {
    match ids {
        [] => String::new(),
        [one] => format!(", rewording {one}"),
        [rest @ .., last] => format!(", rewording {} and {last}", rest.join(", ")),
    }
}

/// What became of a letter in a round, or of a block that is no letter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Counted,
    Delivered {
        receiver: String,
        id: String,
        /// The ids of the letters it rewords (EP-20d).
        rewording: Vec<String>,
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
            State::Delivered {
                receiver,
                id,
                rewording,
            } => format!(
                "delivered to {receiver} as {id}{}",
                rewording_words(rewording)
            ),
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
    /// Its briefs that did not read (the snapshot's problems): while one is there, the sender's
    /// first reading waits, so that no block of it is missed (EP-20c, the review's point (e)).
    pub unread: &'a [PathBuf],
}

impl Sender<'_> {
    /// Why the sender's first reading waits, when a brief did not read.
    fn unread_words(&self) -> Option<String> {
        let names: Vec<String> = self
            .unread
            .iter()
            .map(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string())
            })
            .collect();
        (!names.is_empty()).then(|| {
            format!(
                "{}'s first reading waits: {} did not read",
                self.name,
                names.join(", ")
            )
        })
    }
}

/// What [`Round::head`] answers: the text `HEAD` holds at a brief, `None` when it holds none, or
/// why git could not say.
pub type Shown = Result<Option<String>, String>;

/// What a round needs besides the senders.
pub struct Round<'a> {
    /// A registered project's folder, by name.
    pub folder_of: &'a dyn Fn(&str) -> Option<PathBuf>,
    /// A block's target as the registry names it (`inbox::outgoing`'s resolve), so that the
    /// blocks of `HEAD`'s copy of a brief are keyed as the working tree's are (EP-20d).
    pub resolve: &'a dyn Fn(&str) -> Option<String>,
    /// The text a sender's `HEAD` holds at one of its briefs (the sender's folder, the brief's
    /// path from it with `/` between folders): `None` when `HEAD` holds nothing there, or why
    /// git could not say. Asked once a round per brief that holds a letter to send (EP-20d).
    pub head: &'a mut dyn FnMut(&Path, &str) -> Shown,
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

/// The keys of the letters `HEAD`'s copy of each brief holds, by the brief's file, asked once a
/// round: `None` when `HEAD` holds no copy, or why git could not say (EP-20d).
type Heads = HashMap<PathBuf, Result<Option<HashSet<String>>, String>>;

/// One round of the mailman over `senders`, its log the ledger's: a first reading counted, each
/// letter not yet in the log sent when its words are committed and its receiver free.
pub fn round(ledger: &Ledger, senders: &[Sender<'_>], cx: &mut Round<'_>) -> Report {
    let mut report = Report::default();
    let log = match Log::read(ledger) {
        Ok(log) => log,
        Err(why) => {
            report.problems.push(why);
            return report;
        }
    };
    // The senders whose first reading waits for a brief that did not read (EP-20c).
    let held: HashMap<String, String> = senders
        .iter()
        .filter(|sender| !log.has_read(sender.name))
        .filter_map(|sender| Some((sender.name.to_lowercase(), sender.unread_words()?)))
        .collect();
    // The projects read for the first time this round, as receivers: mail to one of them
    // written before (it was registered later, or its folder came back) and committed is its
    // past too (EP-20's review). A project whose own first reading waits (a brief that did not
    // read, or git that could not say) is one as well, in its first round, which is recorded,
    // so that a letter written to it later is news, delivered while its own reading still
    // waits (EP-20d, EP-20c's review and this brief's).
    let fresh_receivers: HashSet<String> = senders
        .iter()
        .filter(|sender| !log.has_read(sender.name) && !log.has_received(sender.name))
        .map(|sender| sender.name.to_lowercase())
        .collect();
    let mut counted_to: HashMap<String, usize> = HashMap::new();
    let mut uncounted: HashSet<String> = HashSet::new();
    let mut git_held: HashSet<String> = HashSet::new();
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
            // A brief that did not read may hold blocks: the first reading waits for it, so
            // that none is sent as news later (EP-20c).
            if let Some(why) = held.get(&sender.name.to_lowercase()) {
                for letter in &letters {
                    report.marks.push(Mark {
                        sender: sender.name.to_owned(),
                        path: letter.path.clone(),
                        line: letter.line,
                        state: State::Waits(why.clone()),
                    });
                }
                continue;
            }
            // Only a block committed as it reads is the past (EP-20d: block by block, from
            // `HEAD`'s copy of its brief): one a session is still writing, or rewording, goes
            // out later, once committed.
            let mut heads = Heads::new();
            let checked: Vec<(Letter, Result<(), State>)> = letters
                .into_iter()
                .map(|letter| {
                    let committed = committed(sender, &letter, &mut heads, cx);
                    (letter, committed)
                })
                .collect();
            if let Some(why) = heads.values().find_map(|answer| answer.as_ref().err()) {
                // Read again next round, when git may answer: a brief it could not show may
                // hold the past.
                git_held.insert(sender.name.to_lowercase());
                for (letter, _) in &checked {
                    report.marks.push(Mark {
                        sender: sender.name.to_owned(),
                        path: letter.path.clone(),
                        line: letter.line,
                        state: State::Waits(format!("git could not say what is committed: {why}")),
                    });
                }
                continue;
            }
            let mut past = Vec::new();
            let mut fresh = Vec::new();
            for (letter, committed) in checked {
                match committed {
                    Ok(()) => past.push(letter),
                    Err(waits) => fresh.push((letter, waits)),
                }
            }
            first_reading(ledger, sender.name, &past, cx.now, &mut report);
            for (letter, waits) in fresh {
                report.marks.push(Mark {
                    sender: sender.name.to_owned(),
                    path: letter.path,
                    line: letter.line,
                    state: waits,
                });
            }
            continue;
        }
        let current: HashSet<String> = letters.iter().map(|letter| letter.key.clone()).collect();
        let mut heads = Heads::new();
        for letter in &letters {
            let receiver = letter.receiver.to_lowercase();
            let state = match log.known(letter) {
                Some(Known::Counted) => State::Counted,
                Some(Known::Delivered { receiver, id }) => State::Delivered {
                    receiver: receiver.clone(),
                    id: id.clone(),
                    rewording: log.rewording_of(letter),
                },
                None if fresh_receivers.contains(&receiver) => {
                    // Only a letter committed as it reads is the receiver's past: a draft goes
                    // out later, once committed (EP-20d, the review). Where git cannot say, it
                    // is counted, as before.
                    let draft = match committed(sender, letter, &mut heads, cx) {
                        Ok(()) => None,
                        Err(waits) => heads
                            .get(&letter.path)
                            .is_some_and(Result::is_ok)
                            .then_some(waits),
                    };
                    let row = format!("counted {}", letter.key);
                    match draft {
                        Some(waits) => waits,
                        None => {
                            match ledger.add_event(cx.now, EVENT_KIND, Some(&letter.sender), &row) {
                                Ok(_) => {
                                    *counted_to.entry(receiver).or_default() += 1;
                                    State::Counted
                                }
                                Err(e) => {
                                    uncounted.insert(receiver);
                                    let why = format!("the mail log did not keep a count: {e}");
                                    report.problems.push(why.clone());
                                    State::Waits(why)
                                }
                            }
                        }
                    }
                }
                None => match send(ledger, sender, letter, &mut heads, cx, &mut report) {
                    Ok(delivery) => {
                        delivered(ledger, letter, delivery, &log, &current, &mut report)
                    }
                    Err(waits) => waits,
                },
            };
            report.marks.push(Mark {
                sender: sender.name.to_owned(),
                path: letter.path.clone(),
                line: letter.line,
                state,
            });
        }
    }
    // The mail of a project whose own first reading waits, counted as its past this round, is
    // recorded as such, so that a letter written to it later is news (EP-20d). A count the log
    // did not keep is tried again.
    for sender in senders {
        let name = sender.name.to_lowercase();
        let waits = held.contains_key(&name) || git_held.contains(&name);
        if !waits || !fresh_receivers.contains(&name) || uncounted.contains(&name) {
            continue;
        }
        let n = counted_to.get(&name).copied().unwrap_or(0);
        let row = format!(
            "{MAIL_PAST}: {n} block{} to it from projects read before, while its first reading \
             waits",
            if n == 1 { "" } else { "s" }
        );
        if let Err(e) = ledger.add_event(cx.now, EVENT_KIND, Some(sender.name), &row) {
            report.problems.push(format!(
                "the mail log did not keep {}'s past mail: {e}",
                sender.name
            ));
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

/// Whether `letter`'s words are committed: its key among the letters of `HEAD`'s copy of its
/// brief, parsed and resolved as the working tree's (EP-20d), or why it waits. `heads` keeps
/// each brief's answer for the round, so git is asked once per brief.
fn committed(
    sender: &Sender<'_>,
    letter: &Letter,
    heads: &mut Heads,
    cx: &mut Round<'_>,
) -> Result<(), State> {
    let Ok(relative) = letter.path.strip_prefix(sender.folder) else {
        return Err(State::Waits(
            "its brief lies outside the project's folder".to_owned(),
        ));
    };
    let keys = heads.entry(letter.path.clone()).or_insert_with(|| {
        let relative = relative.to_string_lossy().replace('\\', "/");
        let text = (cx.head)(sender.folder, &relative)?;
        Ok(text.map(|text| {
            let blocks = outgoing_of(letter.brief, &letter.path, &brief::parse(&text), cx.resolve);
            letters(sender.name, &blocks)
                .into_iter()
                .map(|committed| committed.key)
                .collect()
        }))
    });
    match keys {
        Ok(Some(keys)) if keys.contains(&letter.key) => Ok(()),
        Ok(Some(_)) => Err(State::Waits("its words are not committed yet".to_owned())),
        Ok(None) => Err(State::Waits("its brief is not committed yet".to_owned())),
        Err(why) => Err(State::Waits(format!(
            "git could not say whether its words are committed: {why}"
        ))),
    }
}

/// A letter sent: the message's id in the receiver's inbox, and whether it was written now
/// (it may be there already, carried by hand or delivered before).
struct Delivery {
    id: String,
    appended: bool,
}

/// Sends `letter` when its words are committed and its receiver free; else why it waits.
fn send(
    ledger: &Ledger,
    sender: &Sender<'_>,
    letter: &Letter,
    heads: &mut Heads,
    cx: &mut Round<'_>,
    report: &mut Report,
) -> Result<Delivery, State> {
    committed(sender, letter, heads, cx)?;
    let Some(folder) = (cx.folder_of)(&letter.receiver) else {
        return Err(State::Waits(format!(
            "{}'s folder is missing",
            letter.receiver
        )));
    };
    if let Some(why) = (cx.busy)(&letter.receiver) {
        return Err(State::Waits(why));
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
                return Err(State::Waits(why));
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
    }
    Ok(Delivery {
        id,
        appended: written.is_some(),
    })
}

/// A letter just sent: what it rewords (EP-20d) recorded in the log, the status bar's words
/// when it was written now, and its state.
fn delivered(
    ledger: &Ledger,
    letter: &Letter,
    delivery: Delivery,
    log: &Log,
    current: &HashSet<String>,
    report: &mut Report,
) -> State {
    let rewording = log.reworded(letter, current);
    if !rewording.is_empty() {
        let row = format!("{REWORDING} {} of {}", letter.key, rewording.join(", "));
        if let Err(e) = ledger.add_event(Utc::now(), EVENT_KIND, Some(&letter.sender), &row) {
            report
                .problems
                .push(format!("the mail log did not keep a rewording: {e}"));
        }
    }
    if delivery.appended {
        report.delivered.push(format!(
            "{} to {}, from {} {}{}",
            delivery.id,
            letter.receiver,
            letter.sender,
            letter.brief,
            rewording_words(&rewording)
        ));
    }
    State::Delivered {
        receiver: letter.receiver.clone(),
        id: delivery.id,
        rewording,
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

    /// The tests' registry, for the blocks of `HEAD`'s copies: alpha, beta and gamma.
    fn resolve(name: &str) -> Option<String> {
        ["alpha", "beta", "gamma"]
            .iter()
            .find(|known| known.eq_ignore_ascii_case(name))
            .map(|known| (*known).to_owned())
    }

    /// A `HEAD` seam over `texts` (a brief's path from its project's folder → its committed
    /// text), counting the briefs it is asked for.
    fn head_over<'t>(
        texts: &'t HashMap<String, String>,
        asked: &'t std::cell::Cell<usize>,
    ) -> impl FnMut(&Path, &str) -> Shown + 't {
        move |_, path| {
            asked.set(asked.get() + 1);
            Ok(texts.get(path).cloned())
        }
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
        // alpha, read before, wrote to gamma while gamma was not registered, and is writing a
        // second note to it, not committed yet.
        let note = |line: usize, words: &str| Outgoing {
            brief: EpId::new(6),
            path: alpha.join("roadmap").join("EP-6-export.md"),
            line,
            to: "gamma".to_owned(),
            recipient: Recipient::Project("gamma".to_owned()),
            text: format!("**Handed on (to gamma).** {words}"),
        };
        let outgoing = vec![note(9, "An old note."), note(11, "A draft note.")];
        let old = "# EP-6\n\n> **Handed on (to gamma).** An old note.\n";
        let both = format!("{old}\n> **Handed on (to gamma).** A draft note.\n");
        let backups = scratch.0.join("backups");
        let folders = |name: &str| (name == "gamma").then(|| gamma.clone());
        let run = |committed: &str| {
            let texts =
                HashMap::from([("roadmap/EP-6-export.md".to_owned(), committed.to_owned())]);
            let asked = std::cell::Cell::new(0);
            let mut head = head_over(&texts, &asked);
            let free = |_: &str| None;
            let mut cx = Round {
                folder_of: &folders,
                resolve: &resolve,
                head: &mut head,
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
                    unread: &[],
                },
                Sender {
                    name: "gamma",
                    folder: &gamma,
                    outgoing: &[],
                    unread: &[],
                },
            ];
            round(&ledger, &senders, &mut cx)
        };
        // The committed note is gamma's past; the draft is not (EP-20d, the review): it waits.
        let report = run(old);
        assert!(report.delivered.is_empty(), "{:?}", report.delivered);
        assert_eq!(report.marks[0].state, State::Counted);
        assert_eq!(
            report.marks[1].state,
            State::Waits("its words are not committed yet".to_owned())
        );
        assert!(!inbox_path(&gamma).exists(), "nothing written into gamma");
        assert!(Log::read(&ledger).unwrap().has_read("gamma"));
        // Committed once gamma is read: news, delivered.
        let report = run(&both);
        assert_eq!(report.delivered, ["IN-1 to gamma, from alpha EP-6"]);
        assert_eq!(report.marks[0].state, State::Counted);
    }

    #[test]
    fn mail_to_a_project_whose_git_cannot_say_goes_on_after_its_first_round() {
        let scratch = Scratch::new("mail-git-held");
        let (alpha, beta) = portfolio(&scratch);
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        ledger
            .add_event(
                Utc::now(),
                EVENT_KIND,
                Some("alpha"),
                "first reading: 0 blocks",
            )
            .unwrap();
        let note = |folder: &Path, to: &str, line: usize, words: &str| Outgoing {
            brief: EpId::new(6),
            path: folder.join("roadmap").join("EP-6-export.md"),
            line,
            to: to.to_owned(),
            recipient: Recipient::Project(to.to_owned()),
            text: format!("**Handed on (to {to}).** {words}"),
        };
        let from_beta = vec![note(&beta, "alpha", 3, "From beta.")];
        let first = "# EP-6\n\n> **Handed on (to beta).** An old note.\n";
        let later = format!("{first}\n> **Handed on (to beta).** A new note.\n");
        let backups = scratch.0.join("backups");
        let folders = |name: &str| (name == "beta").then(|| beta.clone());
        let run = |committed: &str, from_alpha: &[Outgoing]| {
            let texts =
                HashMap::from([("roadmap/EP-6-export.md".to_owned(), committed.to_owned())]);
            // beta's git fails at every round; alpha's answers.
            let mut head = |folder: &Path, path: &str| -> Shown {
                if folder == beta.as_path() {
                    return Err("exited with exit code: 128: fatal: dubious ownership".to_owned());
                }
                Ok(texts.get(path).cloned())
            };
            let free = |_: &str| None;
            let mut cx = Round {
                folder_of: &folders,
                resolve: &resolve,
                head: &mut head,
                busy: &free,
                backups: &backups,
                today: "2026-11-10",
                now: Utc::now(),
            };
            let senders = [
                Sender {
                    name: "alpha",
                    folder: &alpha,
                    outgoing: from_alpha,
                    unread: &[],
                },
                Sender {
                    name: "beta",
                    folder: &beta,
                    outgoing: &from_beta,
                    unread: &[],
                },
            ];
            round(&ledger, &senders, &mut cx)
        };
        // beta's first reading waits for git; alpha's committed note to it is its past, and
        // that is recorded once (the review's point on EP-20d).
        let report = run(first, &[note(&alpha, "beta", 3, "An old note.")]);
        assert_eq!(report.marks[0].state, State::Counted);
        assert!(matches!(&report.marks[1].state, State::Waits(why) if why.contains("dubious")));
        let log = Log::read(&ledger).unwrap();
        assert!(!log.has_read("beta") && log.has_received("beta"));
        // A note alpha writes to beta later is news, delivered while beta's reading waits.
        let both = [
            note(&alpha, "beta", 3, "An old note."),
            note(&alpha, "beta", 5, "A new note."),
        ];
        let report = run(&later, &both);
        assert_eq!(report.delivered, ["IN-3 to beta, from alpha EP-6"]);
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
        // Redone after a revert: not twice, the id of the message there.
        assert_eq!(
            redeliver(
                &gamma,
                "(from alpha EP-6, 2026-11-10) Hello.",
                &backups,
                "2026-11-11"
            )
            .unwrap(),
            Redelivered {
                id: "IN-1".to_owned(),
                appended: false
            }
        );
        // Another message took the id meanwhile: appended under the next (EP-20c, (d)).
        fs::write(
            inbox_path(&gamma),
            format!("{HEADER}\n- [ ] IN-1 (from beta EP-2, 2026-11-11) Another.\n"),
        )
        .unwrap();
        assert_eq!(
            redeliver(
                &gamma,
                "(from alpha EP-6, 2026-11-10) Hello.",
                &backups,
                "2026-11-11"
            )
            .unwrap(),
            Redelivered {
                id: "IN-2".to_owned(),
                appended: true
            }
        );
    }

    #[test]
    fn the_owners_message_is_written_with_its_flag_and_logged_for_a_revert() {
        let scratch = Scratch::new("mail-owner");
        let (_, beta) = portfolio(&scratch);
        let backups = scratch.0.join("backups");
        let ledger_path = scratch.0.join("ledger.sqlite");
        assert_eq!(
            compose_text("  Read  this\nfirst ", false),
            "Read this first"
        );
        assert_eq!(
            compose_text("Read this first", true),
            "Read this first [urgent]"
        );
        assert_eq!(compose_text("Done [Urgent]", true), "Done [Urgent]");
        // A flag typed before punctuation is not doubled (EP-20c's review; EP-20d).
        assert_eq!(compose_text("Do it [urgent].", true), "Do it [urgent].");
        assert_eq!(owner_meta("2026-11-10"), "from the owner, 2026-11-10");
        // The revert keeps a message whose tick row fits the ledger's cap, and names a longer.
        let meta = owner_meta("2026-11-10");
        assert!(revert_keeps("IN-3", &meta, &"word ".repeat(150)));
        assert!(!revert_keeps("IN-3", &meta, &"word ".repeat(200)));
        for empty in ["", " \n ", "[urgent]", "[Urgent] !"] {
            assert_eq!(
                send_from_owner(&beta, "beta", empty, true, &backups, "2026-11-10", None),
                Err("the message is empty".to_owned()),
                "{empty:?}"
            );
        }
        let sent = send_from_owner(
            &beta,
            "beta",
            "Read the export's\nnew column first.",
            true,
            &backups,
            "2026-11-10",
            Some(&ledger_path),
        )
        .unwrap();
        assert_eq!(sent.id, "IN-3");
        assert_eq!(
            sent.written,
            "- [ ] IN-3 (from the owner, 2026-11-10) Read the export's new column first. [urgent]"
        );
        assert_eq!(
            sent.copy,
            Some(backups.join("beta").join("inbox.2026-11-10.md"))
        );
        assert_eq!(sent.unlogged, None);
        assert!(sent.revert_keeps);
        let inbox = super::super::read(&beta).unwrap().unwrap();
        let message = inbox.messages.last().unwrap();
        assert_eq!(message.sender.as_deref(), Some("the owner"));
        assert!(message.urgent && !message.done);
        assert_eq!(message.date.as_deref(), Some("2026-11-10"));
        // A revert of a session in beta redoes it; the mail log names it under beta.
        let ledger = Ledger::open(&ledger_path).unwrap();
        let ticks =
            revert::ticks_since(&ledger, "beta", Utc::now() - chrono::TimeDelta::minutes(1))
                .unwrap();
        assert_eq!(ticks.len(), 1);
        assert!(ticks[0].delivery);
        assert_eq!(
            ticks[0].detail(),
            "delivered IN-3 · (from the owner, 2026-11-10) Read the export's new column first. \
             [urgent]"
        );
        let rows = ledger.events(EVENT_KIND, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].project.as_deref(), Some("beta"));
        assert_eq!(
            rows[0].detail.as_deref(),
            Some("written IN-3 from the owner")
        );
        // The mail log reads past it.
        let log = Log::from_events(&rows);
        assert!(!log.has_read("beta") && log.known.is_empty());
        assert_eq!(
            redeliver(
                &beta,
                ticks[0].disposition.as_deref().unwrap(),
                &backups,
                "2026-11-11"
            )
            .unwrap(),
            Redelivered {
                id: "IN-3".to_owned(),
                appended: false
            }
        );
        // Without a ledger the write stands and says nothing of a log.
        let plain = send_from_owner(
            &beta,
            "beta",
            "Second.",
            false,
            &backups,
            "2026-11-10",
            None,
        )
        .unwrap();
        assert_eq!((plain.id.as_str(), plain.unlogged), ("IN-4", None));
        assert!(plain.written.ends_with(") Second."));
    }

    #[test]
    fn a_first_reading_waits_for_a_brief_that_did_not_read_and_mail_to_it_goes_on() {
        let scratch = Scratch::new("mail-unread");
        let (alpha, beta) = portfolio(&scratch);
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let backups = scratch.0.join("backups");
        let folders = |name: &str| match name {
            "alpha" => Some(scratch.0.join("alpha")),
            "beta" => Some(scratch.0.join("beta")),
            _ => None,
        };
        let outgoing = outgoing_of(&alpha);
        // beta hands letters to alpha too, from a committed brief.
        let note = |line: usize, words: &str| Outgoing {
            brief: EpId::new(2),
            path: beta.join("roadmap").join("EP-2-two.md"),
            line,
            to: "alpha".to_owned(),
            recipient: Recipient::Project("alpha".to_owned()),
            text: format!("**Handed on (to alpha).** {words}"),
        };
        let old = vec![note(3, "A note for alpha.")];
        let later = vec![note(3, "A note for alpha."), note(5, "A later note.")];
        let texts = HashMap::from([
            ("roadmap/EP-6-export.md".to_owned(), BRIEF.to_owned()),
            (
                "roadmap/EP-2-two.md".to_owned(),
                "# EP-2\n\n> **Handed on (to alpha).** A note for alpha.\n\n\
                 > **Handed on (to alpha).** A later note.\n"
                    .to_owned(),
            ),
        ]);
        let unread = vec![alpha.join("roadmap").join("EP-7-broken.md")];
        // beta was read before, so a letter it writes to alpha later is news.
        ledger
            .add_event(
                Utc::now(),
                EVENT_KIND,
                Some("beta"),
                "first reading: 0 blocks counted as delivered",
            )
            .unwrap();
        let run = |unread: &[PathBuf], to_alpha: &[Outgoing]| {
            let asked = std::cell::Cell::new(0);
            let mut head = head_over(&texts, &asked);
            let free = |_: &str| None;
            let mut cx = Round {
                folder_of: &folders,
                resolve: &resolve,
                head: &mut head,
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
                    unread,
                },
                Sender {
                    name: "beta",
                    folder: &beta,
                    outgoing: to_alpha,
                    unread: &[],
                },
            ];
            round(&ledger, &senders, &mut cx)
        };
        let report = run(&unread, &old);
        let waits =
            State::Waits("alpha's first reading waits: EP-7-broken.md did not read".to_owned());
        // alpha's letter waits with its first reading, which is not recorded; beta's letter to
        // alpha, there when alpha is first seen, is alpha's past, counted at once and recorded
        // so (EP-20d).
        assert_eq!(report.marks[1].state, waits);
        assert_eq!(report.marks[2].state, State::Counted);
        assert!(report.delivered.is_empty(), "{:?}", report.delivered);
        let log = Log::read(&ledger).unwrap();
        assert!(!log.has_read("alpha") && log.has_read("beta") && log.has_received("alpha"));
        assert!(!inbox_path(&alpha).exists());
        // A letter beta writes to alpha during the hold is news: delivered while alpha's own
        // first reading still waits.
        let report = run(&unread, &later);
        assert_eq!(report.marks[1].state, waits);
        assert_eq!(report.marks[2].state, State::Counted);
        assert_eq!(
            report.marks[3].state,
            State::Delivered {
                receiver: "alpha".to_owned(),
                id: "IN-1".to_owned(),
                rewording: Vec::new(),
            }
        );
        assert_eq!(report.delivered, ["IN-1 to alpha, from beta EP-2"]);
        // Once alpha's brief reads, its first reading counts its letter; beta's keep theirs.
        let report = run(&[], &later);
        assert_eq!(report.marks[1].state, State::Counted);
        assert_eq!(report.marks[2].state, State::Counted);
        assert!(matches!(report.marks[3].state, State::Delivered { .. }));
        assert!(report.delivered.is_empty());
        assert!(Log::read(&ledger).unwrap().has_read("alpha"));
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
        let run = |busy: Option<&str>, committed: &[&str]| {
            let outgoing = outgoing_of(&alpha);
            // HEAD holds the briefs named, as they are on disk.
            let texts: HashMap<String, String> = committed
                .iter()
                .map(|name| {
                    let path = format!("roadmap/{name}");
                    let text = fs::read_to_string(alpha.join(&path)).unwrap();
                    (path, text)
                })
                .collect();
            let asked = std::cell::Cell::new(0);
            let mut head = head_over(&texts, &asked);
            let busy_words = busy.map(str::to_owned);
            let busy_of = |name: &str| busy_words.clone().filter(|_| name == "beta");
            let mut cx = Round {
                folder_of: &folders,
                resolve: &resolve,
                head: &mut head,
                busy: &busy_of,
                backups: &backups,
                today: "2026-11-10",
                now,
            };
            let senders = [Sender {
                name: "alpha",
                folder: &alpha,
                outgoing: &outgoing,
                unread: &[],
            }];
            let report = round(&ledger, &senders, &mut cx);
            (report, asked.get())
        };
        // A brief a session is still writing when the mailman first reads the project.
        let brief = alpha.join("roadmap").join("EP-7-more.md");
        fs::write(&brief, "# EP-7\n\n> **Completion note (2026-11-10).** Done.\n\n> **Handed on (to beta).** A second note.\n").unwrap();
        // The first reading counts the committed brief's letter and sends nothing; the brief
        // still being written is no past: its letter waits.
        let (report, asked) = run(None, &["EP-6-export.md"]);
        assert_eq!(asked, 2, "git asked once per brief with a letter");
        let states: Vec<&State> = report.marks.iter().map(|m| &m.state).collect();
        let waits = State::Waits("its brief is not committed yet".to_owned());
        assert_eq!(states, [&State::Unknown, &State::Counted, &waits]);
        assert!(report.delivered.is_empty());
        // Still uncommitted at the next round: it waits.
        let (report, _) = run(None, &["EP-6-export.md"]);
        assert_eq!(report.marks.last().unwrap().state, waits);
        let both = ["EP-6-export.md", "EP-7-more.md"];
        // A session works in the receiver: it waits.
        let (report, _) = run(Some("a session works there"), &both);
        assert_eq!(
            report.marks.last().unwrap().state,
            State::Waits("a session works there".to_owned())
        );
        // Committed and free: delivered once, logged, and a tick for a revert.
        let (report, _) = run(None, &both);
        assert_eq!(
            report.marks.last().unwrap().state,
            State::Delivered {
                receiver: "beta".to_owned(),
                id: "IN-3".to_owned(),
                rewording: Vec::new(),
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
        let (report, asked) = run(None, &both);
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

    /// The window's `HEAD` seam over a real repository: `git::show_at_head`, as `app::mail` asks.
    fn real_head(folder: &Path, path: &str) -> Shown {
        let _turn = crate::process::turn();
        crate::git::show_at_head(folder, path)
    }

    #[test]
    fn a_block_is_committed_by_its_words_and_a_rewording_says_so() {
        let repo = crate::revert::testing::Repo::new("mail-head");
        let alpha = repo.0.clone();
        let beta = repo.scratch().join("beta");
        fs::create_dir_all(beta.join("roadmap")).unwrap();
        fs::write(
            inbox_path(&beta),
            "# Inbox\n\n- [ ] IN-2 (from the owner, 2026-11-01) Old.\n",
        )
        .unwrap();
        repo.write("roadmap/EP-6-export.md", BRIEF);
        repo.commit("EP-6");
        // An old brief edited since its commit, here with CRLF line ends against HEAD's LF, and
        // a new brief no commit holds: the first reading counts the old block, whose words are
        // committed, and holds the new one (EP-20d; the file's status made it news before).
        repo.write(
            "roadmap/EP-6-export.md",
            &BRIEF
                .replace("One session.", "One long session.")
                .replace('\n', "\r\n"),
        );
        let seven = "# EP-7\n\n> **Completion note (2026-11-10).** Done.\n\n\
                     > **Handed on (to beta).** A second note.\n";
        repo.write("roadmap/EP-7-more.md", seven);
        let ledger = Ledger::open(&repo.scratch().join("ledger.sqlite")).unwrap();
        let backups = repo.backups();
        let folders = |name: &str| match name {
            "alpha" => Some(alpha.clone()),
            "beta" => Some(beta.clone()),
            _ => None,
        };
        let run = || {
            let outgoing = outgoing_of(&alpha);
            let mut head = real_head;
            let free = |_: &str| None;
            let mut cx = Round {
                folder_of: &folders,
                resolve: &resolve,
                head: &mut head,
                busy: &free,
                backups: &backups,
                today: "2026-11-10",
                now: Utc::now(),
            };
            let senders = [Sender {
                name: "alpha",
                folder: &alpha,
                outgoing: &outgoing,
                unread: &[],
            }];
            round(&ledger, &senders, &mut cx)
        };
        let report = run();
        let states: Vec<&State> = report.marks.iter().map(|m| &m.state).collect();
        let not_committed = State::Waits("its brief is not committed yet".to_owned());
        assert_eq!(states, [&State::Unknown, &State::Counted, &not_committed]);
        // Committed: the new block goes out as news.
        repo.commit("EP-7");
        let report = run();
        assert_eq!(report.delivered, ["IN-3 to beta, from alpha EP-7"]);
        // Reworded in the working tree: a new letter, waiting for its words to be committed.
        repo.write(
            "roadmap/EP-7-more.md",
            &seven.replace("A second note.", "A second note, reworded."),
        );
        let report = run();
        assert_eq!(
            report.marks.last().unwrap().state,
            State::Waits("its words are not committed yet".to_owned())
        );
        assert!(report.delivered.is_empty());
        // Committed: sent again (D-13), saying what it rewords, here and in later rounds.
        repo.commit("EP-7 reworded");
        let report = run();
        assert_eq!(
            report.delivered,
            ["IN-4 to beta, from alpha EP-7, rewording IN-3"]
        );
        let words = "delivered to beta as IN-4, rewording IN-3";
        assert_eq!(report.marks.last().unwrap().state.words(), words);
        let report = run();
        assert!(report.delivered.is_empty() && report.problems.is_empty());
        assert_eq!(report.marks.last().unwrap().state.words(), words);
        let text = fs::read_to_string(inbox_path(&beta)).unwrap();
        assert!(
            text.ends_with(
                "- [ ] IN-3 (from alpha EP-7, 2026-11-10) A second note.\n\
                 - [ ] IN-4 (from alpha EP-7, 2026-11-10) A second note, reworded.\n"
            ),
            "{text}"
        );
        // A third version names the second, which reworded the first already (the review).
        repo.write(
            "roadmap/EP-7-more.md",
            &seven.replace("A second note.", "A second note, reworded twice."),
        );
        repo.commit("EP-7 reworded twice");
        let report = run();
        assert_eq!(
            report.delivered,
            ["IN-5 to beta, from alpha EP-7, rewording IN-4"]
        );
        assert_eq!(
            rewording_words(&["IN-4".to_owned(), "IN-6".to_owned(), "IN-9".to_owned()]),
            ", rewording IN-4, IN-6 and IN-9"
        );
    }
}
