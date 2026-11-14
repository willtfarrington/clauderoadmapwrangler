//! The start-up reconciliation (D-29; EP-28a): the ledger's rows that no window heard end, closed
//! when a window starts. A visual or manual session whose tab closed while no window ran, or whose
//! end hook never came, keeps an open row (`RowState::Open`, "open: no end recorded"); so does a
//! headless run whose window crashed or was killed, since only the runner closes its row and it died
//! with its window. Once a window holds the loopback port and two listings of
//! `claude agents --json` in a row have answered, each leaving no entry out, each open row that
//! started in a second before the window's is weighed. A visual or manual row whose session
//! neither listing names, and whose transcript was not written since the window started, closes as
//! completed, as its end hook would have. A headless row, which no listing names (`-p` runs are
//! not listed) and whose runner died with its window, closes as stopped, as a quit records the
//! runs it ends. A row ends at its transcript's last write, the file's modified time with its
//! contents never read (D-26), a headless row at the window's start at the latest; else at its
//! start. Its result says no end was heard, and an `event` row of kind [`EVENT_KIND`] names it,
//! all in one transaction. A visual launch no session took up is left to the lapse (EP-13).

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use chrono::{DateTime, Local, SubsecRound, Utc};

use crate::ledger::{Error, Ledger, SessionRow};

/// The `event` rows' kind for a row the reconciliation closed.
pub const EVENT_KIND: &str = "reconcile";

/// One row to close, and how.
#[derive(Debug, Clone, PartialEq)]
pub struct Closing {
    pub row: i64,
    pub session_id: Option<String>,
    pub project: String,
    pub mode: String,
    /// `completed` for a visual or manual row, `stopped` for a headless one.
    pub outcome: &'static str,
    pub ended_at: DateTime<Utc>,
    /// Whether the end is the transcript's last write; else the row's start.
    pub from_transcript: bool,
}

impl Closing {
    /// The row's session in short, as the ledger's cards show it: its id's first eight
    /// characters, or the row's number when it has none.
    pub fn short(&self) -> String {
        match &self.session_id {
            Some(id) => id.chars().take(8).collect(),
            None => format!("row {}", self.row),
        }
    }

    /// The result the closed row keeps, which its card shows.
    pub fn note(&self, now: DateTime<Utc>) -> String {
        let end = if self.from_transcript {
            "it ends at its transcript's last write"
        } else {
            "no transcript was found, so it ends at its start"
        };
        format!(
            "No end was heard; closed when a window started on {} ({end}).",
            local(now)
        )
    }

    /// The `event` row's words: the session, its mode and the end it was given.
    pub fn detail(&self) -> String {
        let end = if self.from_transcript {
            "its transcript's last write"
        } else {
            "its start"
        };
        format!(
            "closed {} ({}, {}): no end was heard; ended {} at {end}",
            self.short(),
            self.mode,
            self.outcome,
            local(self.ended_at)
        )
    }
}

/// A time in local words: `2026-11-10 09:02`.
fn local(at: DateTime<Utc>) -> String {
    at.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

/// The open rows to close. Each started in a second before `window_started`'s (the ledger keeps
/// whole seconds), a visual launch with no session left out. A headless row closes whatever the
/// listings say, since `-p` runs are never listed and one listed is the owner's resume of its
/// session by hand (EP-35's row per run); it ends at its transcript's last write, at the window's
/// start at the latest. A visual or manual row closes when no id of `live` names its session and
/// its transcript was not written since the window started (a write since then is a live
/// session's); it ends at that last write. `last_write` gives a session's transcript's last
/// write, taken when it is not before the row's start; without one a row ends at its start.
pub fn choose(
    open: &[SessionRow],
    live: &HashSet<String>,
    window_started: DateTime<Utc>,
    last_write: impl Fn(&str) -> Option<DateTime<Utc>>,
) -> Vec<Closing> {
    let cutoff = window_started.trunc_subsecs(0);
    let mut closings = Vec::new();
    for row in open {
        if row.ended_at.is_some() {
            continue;
        }
        let Some(started) = row.started().filter(|at| *at < cutoff) else {
            continue;
        };
        if row.mode == "visual" && row.session_id.is_none() {
            continue;
        }
        let headless = row.mode == "headless";
        if !headless
            && row
                .session_id
                .as_deref()
                .is_some_and(|id| live.contains(id))
        {
            continue;
        }
        let written = row
            .session_id
            .as_deref()
            .and_then(&last_write)
            .filter(|at| *at >= started);
        let ended_at = match written {
            Some(at) if headless => at.min(window_started),
            Some(at) if at >= window_started => continue,
            Some(at) => at,
            None => started,
        };
        closings.push(Closing {
            row: row.id,
            session_id: row.session_id.clone(),
            project: row.project.clone(),
            mode: row.mode.clone(),
            outcome: if headless { "stopped" } else { "completed" },
            ended_at,
            from_transcript: written.is_some(),
        });
    }
    closings
}

/// The last write of `session_id`'s transcript, `<claude_dir>\projects\<slug>\<id>.jsonl` in any
/// project's folder: the file's modified time alone. None for an id that is no plain id, or when
/// no folder holds the file.
pub fn last_write(claude_dir: &Path, session_id: &str) -> Option<DateTime<Utc>> {
    let plain = !session_id.is_empty()
        && session_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !plain {
        return None;
    }
    let file = format!("{session_id}.jsonl");
    fs::read_dir(claude_dir.join("projects"))
        .ok()?
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| fs::metadata(entry.path().join(&file)).ok())
        .filter(fs::Metadata::is_file)
        .filter_map(|meta| meta.modified().ok())
        .map(DateTime::<Utc>::from)
        .max()
}

/// Closes each row of `closings` in `ledger` with its note and an `event` row, all in one
/// transaction, so that a failure closes none and a later try names them all: the ones closed,
/// a row that had an end by then left out.
pub fn apply(
    ledger: &Ledger,
    closings: &[Closing],
    now: DateTime<Utc>,
) -> Result<Vec<Closing>, Error> {
    ledger.in_transaction(|ledger| {
        let mut closed = Vec::new();
        for closing in closings {
            let note = closing.note(now);
            if ledger.close_unheard(closing.row, closing.ended_at, closing.outcome, &note)? {
                ledger.add_event(now, EVENT_KIND, Some(&closing.project), &closing.detail())?;
                closed.push(closing.clone());
            }
        }
        Ok(closed)
    })
}

/// What a start's reconciliation closed, in words for the window's lines and the log.
pub fn words(closed: &[Closing]) -> String {
    if closed.is_empty() {
        return "nothing to close: every open row's session is live or the window's own".to_owned();
    }
    let rows: Vec<String> = closed
        .iter()
        .map(|closing| {
            let end = if closing.from_transcript {
                "its transcript's last write"
            } else {
                "its start"
            };
            format!(
                "{} ({}, {}, {}, ended {} at {end})",
                closing.short(),
                closing.project,
                closing.mode,
                closing.outcome,
                local(closing.ended_at)
            )
        })
        .collect();
    let count = match closed.len() {
        1 => "1 row".to_owned(),
        n => format!("{n} rows"),
    };
    format!("closed {count} no window heard end: {}", rows.join("; "))
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;
    use crate::files::tests::Scratch;
    use crate::ledger::NewSession;

    fn at(minute: i64) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-10T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
            + TimeDelta::minutes(minute)
    }

    fn live(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|id| (*id).to_owned()).collect()
    }

    fn start(ledger: &Ledger, id: Option<&str>, mode: &str, minute: i64) -> i64 {
        ledger
            .start_session(&NewSession {
                session_id: id,
                project: "alpha",
                ep: Some("EP-4"),
                mode,
                model: None,
                effort: None,
                ultracode: false,
                started_at: at(minute),
            })
            .unwrap()
    }

    #[test]
    fn closes_the_rows_no_window_heard_end_and_leaves_the_live_and_the_new() {
        let scratch = Scratch::new("reconcile-choose");
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let gone_visual = start(&ledger, Some("visual-gone"), "visual", 0);
        start(&ledger, Some("visual-live"), "visual", 1);
        let crashed = start(&ledger, Some("headless-crash"), "headless", 2);
        start(&ledger, None, "visual", 3);
        let bare = start(&ledger, Some("manual-bare"), "manual", 4);
        let ended = start(&ledger, Some("ended"), "manual", 5);
        assert!(ledger.end_seen("ended", at(6), None, None).unwrap());
        start(&ledger, Some("visual-alive"), "visual", 6);
        // The ledger keeps whole seconds: a row of the window's first second is the window's.
        start(&ledger, Some("same-second"), "manual", 25);
        // The window's own: started after it.
        start(&ledger, Some("manual-new"), "manual", 30);

        let open = ledger.open_rows().unwrap();
        assert!(open.iter().all(|row| row.id != ended));
        let window = at(25) + TimeDelta::milliseconds(700);
        let written = |id: &str| match id {
            "visual-gone" => Some(at(9)),
            // Resumed by hand since its window crashed: still the crashed run's row to close,
            // at the window's start at the latest.
            "headless-crash" => Some(at(40)),
            // A write before the row's start is no end of it.
            "manual-bare" => Some(at(1)),
            // Written since the window started: a live session the listings missed.
            "visual-alive" => Some(at(30)),
            _ => None,
        };
        let closings = choose(
            &open,
            &live(&["visual-live", "headless-crash"]),
            window,
            written,
        );
        let rows: Vec<(i64, &str, DateTime<Utc>, bool)> = closings
            .iter()
            .map(|c| (c.row, c.outcome, c.ended_at, c.from_transcript))
            .collect();
        assert_eq!(
            rows,
            [
                (gone_visual, "completed", at(9), true),
                (crashed, "stopped", window, true),
                (bare, "completed", at(4), false),
            ]
        );

        let closed = apply(&ledger, &closings, at(26)).unwrap();
        assert_eq!(closed.len(), 3);
        let still: Vec<Option<String>> = ledger
            .open_rows()
            .unwrap()
            .into_iter()
            .map(|row| row.session_id)
            .collect();
        assert_eq!(
            still,
            [
                Some("visual-live".to_owned()),
                None,
                Some("visual-alive".to_owned()),
                Some("same-second".to_owned()),
                Some("manual-new".to_owned())
            ]
        );
        let rows = ledger.sessions().unwrap();
        let crash = rows.iter().find(|row| row.id == crashed).unwrap();
        assert_eq!(crash.outcome.as_deref(), Some("stopped"));
        assert_eq!(crash.ended(), Some(at(25)));
        assert!(
            crash
                .result
                .as_deref()
                .unwrap()
                .starts_with("No end was heard; closed when a window started on")
        );
        let events = ledger.events(EVENT_KIND, None).unwrap();
        assert_eq!(events.len(), 3);
        assert!(events.iter().any(|event| {
            event.detail.as_deref().is_some_and(|detail| {
                detail.starts_with("closed headless (headless, stopped): no end was heard")
            })
        }));
        // A second pass finds them closed and writes nothing.
        assert!(apply(&ledger, &closings, at(27)).unwrap().is_empty());
        assert_eq!(ledger.events(EVENT_KIND, None).unwrap().len(), 3);
    }

    #[test]
    fn a_transcript_is_found_by_its_id_in_any_project_folder() {
        let scratch = Scratch::new("reconcile-transcript");
        let projects = scratch.0.join("projects");
        fs::create_dir_all(projects.join("C--Data-alpha")).unwrap();
        fs::create_dir_all(projects.join("C--Data-beta")).unwrap();
        fs::write(projects.join("C--Data-beta").join("abc-1.jsonl"), "{}").unwrap();
        assert!(last_write(&scratch.0, "abc-1").is_some());
        assert_eq!(last_write(&scratch.0, "abc-2"), None);
        assert_eq!(last_write(&scratch.0, "..\\abc-1"), None);
        assert_eq!(last_write(&scratch.0, ""), None);
        assert_eq!(last_write(&scratch.0.join("nowhere"), "abc-1"), None);
    }

    #[test]
    fn the_words_say_what_was_closed() {
        assert!(words(&[]).starts_with("nothing to close"));
        let closing = Closing {
            row: 7,
            session_id: Some("45cc1e3b-0000".to_owned()),
            project: "alpha".to_owned(),
            mode: "visual".to_owned(),
            outcome: "completed",
            ended_at: at(0),
            from_transcript: true,
        };
        let text = words(std::slice::from_ref(&closing));
        assert!(text.starts_with(
            "closed 1 row no window heard end: 45cc1e3b (alpha, visual, completed, ended "
        ));
        assert!(text.ends_with("at its transcript's last write)"));
        let bare = Closing {
            session_id: None,
            from_transcript: false,
            ..closing
        };
        assert_eq!(bare.short(), "row 7");
        assert!(bare.detail().ends_with("at its start"));
    }
}
