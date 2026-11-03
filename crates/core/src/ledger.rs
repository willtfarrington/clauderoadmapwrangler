//! The ledger (D-8, D-26): one SQLite file in the local profile for what the tool records over
//! time, kept indefinitely: sessions, usage samples, parse snapshots and events, and the cache of
//! the badges' networked readings (EP-7). EP-2 creates the schema; each brief that records rows
//! adds its queries (EP-9 the sessions'), and a numbered migration when it needs a new table or
//! column. Times are RFC 3339 text in UTC.

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, io};

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior};

/// The schema this build writes: the number of migrations.
pub const SCHEMA_VERSION: u32 = 3;

/// `MIGRATIONS[i]` brings the schema from version `i` to `i + 1`, each in one transaction.
const MIGRATIONS: [&str; SCHEMA_VERSION as usize] = [V1, V2, V3];

/// The most of a run's final text its row keeps, in characters.
const RESULT_CAP: usize = 8000;

const V1: &str = "
CREATE TABLE schema_version (version INTEGER NOT NULL);

-- One row per Claude Code session the tool starts or sees (D-16, D-26; from EP-9).
CREATE TABLE session (
    id               INTEGER PRIMARY KEY,
    session_id       TEXT,
    project          TEXT NOT NULL,
    ep               TEXT,
    mode             TEXT NOT NULL CHECK (mode IN ('visual', 'headless', 'manual')),
    model            TEXT,
    effort           TEXT,
    ultracode        INTEGER NOT NULL DEFAULT 0,
    started_at       TEXT NOT NULL,
    ended_at         TEXT,
    outcome          TEXT CHECK (outcome IN ('completed', 'stopped', 'failed', 'deferred')),
    -- The weekly and Fable-scoped usage percents sampled around the run (D-23, EP-18).
    weekly_before    REAL,
    weekly_after     REAL,
    fable_before     REAL,
    fable_after      REAL,
    -- The CLI's own estimate, from the result or the status line.
    cost_usd         REAL,
    questions        INTEGER NOT NULL DEFAULT 0,
    answer_latency_s REAL,
    commit_hash      TEXT
);
CREATE INDEX session_by_project ON session (project, started_at);

-- One row per usage reading: a poll of the endpoint or a session's status line (D-23; from EP-17).
CREATE TABLE usage_sample (
    id                  INTEGER PRIMARY KEY,
    taken_at            TEXT NOT NULL,
    source              TEXT NOT NULL CHECK (source IN ('endpoint', 'statusline')),
    session_id          TEXT,
    weekly_percent      REAL,
    weekly_resets_at    TEXT,
    fable_percent       REAL,
    fable_resets_at     TEXT,
    five_hour_percent   REAL,
    five_hour_resets_at TEXT
);
CREATE INDEX usage_sample_by_time ON usage_sample (taken_at);

-- A project's done and total rows when they were parsed (D-26; from EP-25).
CREATE TABLE snapshot (
    project  TEXT NOT NULL,
    taken_at TEXT NOT NULL,
    done     INTEGER NOT NULL,
    total    INTEGER NOT NULL,
    PRIMARY KEY (project, taken_at)
);

-- Everything else worth a line: notifications, deliveries, hosting and remote actions (from EP-15).
CREATE TABLE event (
    id      INTEGER PRIMARY KEY,
    at      TEXT NOT NULL,
    kind    TEXT NOT NULL,
    project TEXT,
    detail  TEXT
);
CREATE INDEX event_by_time ON event (at);
";

const V2: &str = "
-- The badges' networked readings (D-24, EP-7): the account's repositories, each project's head
-- check and latest Actions run, as JSON, with the time they were fetched; one row per project and
-- source, replaced by each fetch.
CREATE TABLE badge_cache (
    project    TEXT NOT NULL,
    source     TEXT NOT NULL,
    fetched_at TEXT NOT NULL,
    data       TEXT NOT NULL,
    PRIMARY KEY (project, source)
);
";

const V3: &str = "
-- A run's final text, capped, and the share of the context window the session's status line last
-- gave (D-26, EP-13).
ALTER TABLE session ADD COLUMN result TEXT;
ALTER TABLE session ADD COLUMN context_percent REAL;
";

/// The `session` columns a [`SessionRow`] reads, in its order; a ledger opened to read at schema 2
/// has no result or context yet.
const SESSION_COLUMNS: &str = "id, session_id, project, ep, mode, model, effort, ultracode, \
    started_at, ended_at, outcome, cost_usd, questions, result, context_percent, answer_latency_s";
const SESSION_COLUMNS_V2: &str = "id, session_id, project, ep, mode, model, effort, ultracode, \
    started_at, ended_at, outcome, cost_usd, questions, NULL, NULL, answer_latency_s";

/// A session as it starts (D-16, D-26; EP-9): the columns known before it runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSession<'a> {
    /// None for a visual launch, whose session VS Code names (EP-11).
    pub session_id: Option<&'a str>,
    pub project: &'a str,
    pub ep: Option<&'a str>,
    /// `visual`, `headless` or `manual`.
    pub mode: &'a str,
    pub model: Option<&'a str>,
    pub effort: Option<&'a str>,
    pub ultracode: bool,
    pub started_at: DateTime<Utc>,
}

/// How a run ended, as the runner records it (EP-9, EP-13).
#[derive(Debug, Clone, PartialEq)]
pub struct Finished<'a> {
    pub at: DateTime<Utc>,
    /// `completed`, `stopped`, `failed` or `deferred`.
    pub outcome: &'a str,
    /// The CLI's own estimate at list prices.
    pub cost_usd: Option<f64>,
    pub questions: u32,
    /// The run's final text, kept to [`RESULT_CAP`] characters.
    pub result: Option<&'a str>,
    /// The mean seconds the owner took to answer the run's requests; none when it carried no
    /// answer (D-26; EP-14).
    pub answer_latency_s: Option<f64>,
}

/// A session's row as the ledger holds it.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionRow {
    pub id: i64,
    pub session_id: Option<String>,
    pub project: String,
    pub ep: Option<String>,
    pub mode: String,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub ultracode: bool,
    pub started_at: String,
    pub ended_at: Option<String>,
    /// `completed`, `stopped`, `failed` or `deferred`; none while it runs, and none for a visual
    /// launch that lapsed.
    pub outcome: Option<String>,
    pub cost_usd: Option<f64>,
    pub questions: u32,
    /// The run's final text, capped (from EP-13).
    pub result: Option<String>,
    /// The share of the context window the session's status line last gave (from EP-13).
    pub context_percent: Option<f64>,
    /// The mean seconds the owner took to answer the run's requests (from EP-14).
    pub answer_latency_s: Option<f64>,
}

impl SessionRow {
    pub fn started(&self) -> Option<DateTime<Utc>> {
        parse_time(&self.started_at)
    }

    pub fn ended(&self) -> Option<DateTime<Utc>> {
        self.ended_at.as_deref().and_then(parse_time)
    }

    /// A visual launch no session took up within its window, as of `now`: closed with no session
    /// and no outcome, or still open past `window` until a window closes it (EP-13).
    pub fn lapsed(&self, now: DateTime<Utc>, window: TimeDelta) -> bool {
        self.mode == "visual"
            && self.session_id.is_none()
            && self.outcome.is_none()
            && (self.ended_at.is_some() || self.started().is_some_and(|at| at + window <= now))
    }

    /// What became of its session as of `now`, a visual launch taken up within `window` (EP-13).
    pub fn state(&self, now: DateTime<Utc>, window: TimeDelta) -> RowState<'_> {
        if self.lapsed(now, window) {
            return RowState::Lapsed;
        }
        match self.outcome.as_deref() {
            Some(outcome) => RowState::Ended(outcome),
            None if self.mode == "visual" && self.session_id.is_none() => RowState::Launched,
            None => RowState::Open,
        }
    }

    /// How long its run took, once it has ended; a visual launch no session took up had no run.
    pub fn took(&self) -> Option<TimeDelta> {
        if self.mode == "visual" && self.session_id.is_none() {
            return None;
        }
        Some(self.ended()? - self.started()?)
    }
}

/// What became of a row's session (EP-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowState<'a> {
    /// `completed`, `stopped`, `failed` or `deferred`.
    Ended(&'a str),
    /// A visual launch no session took up within its window: closed with no outcome.
    Lapsed,
    /// A visual launch handed to VS Code that no session has taken up yet.
    Launched,
    /// No end recorded: it runs, or its end was never heard.
    Open,
}

/// What a row's run asked the owner, in words: `2 questions, answered in 40 s` (EP-14); none when
/// it asked nothing and carried no answer.
pub fn asked(row: &SessionRow) -> Option<String> {
    let questions = match row.questions {
        0 => None,
        1 => Some("1 question".to_owned()),
        n => Some(format!("{n} questions")),
    };
    let answered = row.answer_latency_s.map(|seconds| {
        format!(
            "answered in {}",
            length(TimeDelta::seconds(seconds.round() as i64))
        )
    });
    match (questions, answered) {
        (Some(questions), Some(answered)) => Some(format!("{questions}, {answered}")),
        (Some(text), None) | (None, Some(text)) => Some(text),
        (None, None) => None,
    }
}

/// A run's length in words: `36 s`, `52 min`, `1 h 04 min`.
pub fn length(took: TimeDelta) -> String {
    match took.num_seconds().max(0) {
        s if s < 60 => format!("{s} s"),
        s if s < 3600 => format!("{} min", s / 60),
        s => format!("{} h {:02} min", s / 3600, (s % 3600) / 60),
    }
}

/// One row of the badges' cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cached {
    pub project: String,
    pub source: String,
    pub fetched_at: DateTime<Utc>,
    pub data: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("ledger: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error(
        "the ledger has schema version {found}; this build knows up to {supported} and leaves it as it is"
    )]
    Newer { found: u32, supported: u32 },
}

pub struct Ledger {
    connection: Connection,
    /// The schema it holds: this build's, or an older one opened to read.
    version: u32,
}

impl Ledger {
    /// Opens the ledger at `path`, creating it and its folder when missing and bringing an older
    /// schema up to date; a newer schema is refused untouched.
    pub fn open(path: &Path) -> Result<Self, Error> {
        Self::open_with(path, &MIGRATIONS)
    }

    fn open_with(path: &Path, migrations: &[&str]) -> Result<Self, Error> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|source| Error::Io {
                path: dir.to_owned(),
                source,
            })?;
        }
        let mut connection = Connection::open(path)?;
        // The desktop app and `crw` may both have it open.
        connection.busy_timeout(Duration::from_secs(5))?;
        migrate(&mut connection, migrations)?;
        Ok(Self {
            connection,
            version: migrations.len() as u32,
        })
    }

    /// Opens the ledger at `path` to read only: never created, migrated or written, so `crw` and
    /// the Sessions pane's list read the owner's records as they stand (EP-13). An older schema
    /// reads with the columns it has; a newer one is refused.
    pub fn open_read_only(path: &Path) -> Result<Self, Error> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.busy_timeout(Duration::from_secs(5))?;
        let found = version(&connection)?;
        if found > SCHEMA_VERSION {
            return Err(Error::Newer {
                found,
                supported: SCHEMA_VERSION,
            });
        }
        Ok(Self {
            connection,
            version: found,
        })
    }

    pub fn schema_version(&self) -> Result<u32, Error> {
        Ok(version(&self.connection)?)
    }

    /// Keeps one networked reading, replacing the project's earlier one from the same source.
    pub fn cache(
        &self,
        project: &str,
        source: &str,
        fetched_at: DateTime<Utc>,
        data: &str,
    ) -> Result<(), Error> {
        self.connection.execute(
            "INSERT OR REPLACE INTO badge_cache (project, source, fetched_at, data) \
             VALUES (?1, ?2, ?3, ?4)",
            (project, source, rfc3339(fetched_at), data),
        )?;
        Ok(())
    }

    /// Records a session's start and returns its row's id (EP-9).
    pub fn start_session(&self, session: &NewSession<'_>) -> Result<i64, Error> {
        self.connection.execute(
            "INSERT INTO session (session_id, project, ep, mode, model, effort, ultracode, \
             started_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                session.session_id,
                session.project,
                session.ep,
                session.mode,
                session.model,
                session.effort,
                session.ultracode,
                rfc3339(session.started_at),
            ),
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    /// Records how a run ended: its end time, its outcome, the CLI's cost estimate, the questions
    /// it asked, its final text and how long the owner took to answer.
    pub fn finish_session(&self, id: i64, finished: &Finished<'_>) -> Result<(), Error> {
        self.connection.execute(
            "UPDATE session SET ended_at = ?2, outcome = ?3, cost_usd = ?4, questions = ?5, \
             result = ?6, answer_latency_s = ?7 WHERE id = ?1",
            (
                id,
                rfc3339(finished.at),
                finished.outcome,
                finished.cost_usd,
                finished.questions,
                finished.result.map(|text| capped(text, RESULT_CAP)),
                finished.answer_latency_s,
            ),
        )?;
        Ok(())
    }

    /// The mode of the row that holds `session_id`, if one does.
    pub fn mode_of(&self, session_id: &str) -> Result<Option<String>, Error> {
        Ok(self
            .connection
            .query_row(
                "SELECT mode FROM session WHERE session_id = ?1 ORDER BY id DESC LIMIT 1",
                [session_id],
                |row| row.get(0),
            )
            .optional()?)
    }

    /// Gives `session_id` to the newest visual launch of `project` that has none yet and is
    /// open, started at `since` or later: a visual launch is recorded before VS Code names its
    /// session (EP-11), and the session's start hook names it (EP-12). The row's id, when one is
    /// claimed.
    pub fn claim_visual(
        &self,
        project: &str,
        session_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Option<i64>, Error> {
        let found: Option<i64> = self
            .connection
            .query_row(
                "SELECT id FROM session WHERE mode = 'visual' AND session_id IS NULL \
                 AND ended_at IS NULL AND project = ?1 COLLATE NOCASE AND started_at >= ?2 \
                 ORDER BY started_at DESC, id DESC LIMIT 1",
                (project, rfc3339(since)),
                |row| row.get(0),
            )
            .optional()?;
        let Some(id) = found else {
            return Ok(None);
        };
        self.connection.execute(
            "UPDATE session SET session_id = ?2 WHERE id = ?1",
            (id, session_id),
        )?;
        Ok(Some(id))
    }

    /// Closes the open visual or manual row of `session_id` at `at`, as completed: its end hook
    /// (EP-12), with the cost estimate and the context share its status line last gave when it
    /// sent one (EP-13). A headless row is the runner's to close; `false` when no row was open.
    pub fn end_seen(
        &self,
        session_id: &str,
        at: DateTime<Utc>,
        cost_usd: Option<f64>,
        context_percent: Option<f64>,
    ) -> Result<bool, Error> {
        let changed = self.connection.execute(
            "UPDATE session SET ended_at = ?2, outcome = 'completed', \
             cost_usd = COALESCE(?3, cost_usd), context_percent = COALESCE(?4, context_percent) \
             WHERE session_id = ?1 AND mode IN ('visual', 'manual') AND ended_at IS NULL",
            (session_id, rfc3339(at), cost_usd, context_percent),
        )?;
        Ok(changed > 0)
    }

    /// Closes the visual launches no session took up within `window` of their launch (EP-13):
    /// each ends at its launch plus `window`, with no session and no outcome. How many closed.
    pub fn lapse_visual(&self, now: DateTime<Utc>, window: TimeDelta) -> Result<usize, Error> {
        let mut statement = self.connection.prepare(
            "SELECT id, started_at FROM session WHERE mode = 'visual' AND session_id IS NULL \
             AND ended_at IS NULL AND started_at <= ?1",
        )?;
        let open: Vec<(i64, String)> = statement
            .query_map([rfc3339(now - window)], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<_, _>>()?;
        let mut closed = 0;
        for (id, started_at) in open {
            let Some(started) = parse_time(&started_at) else {
                continue;
            };
            closed += self.connection.execute(
                "UPDATE session SET ended_at = ?2 WHERE id = ?1 AND ended_at IS NULL",
                (id, rfc3339(started + window)),
            )?;
        }
        Ok(closed)
    }

    /// Every session, the oldest first.
    pub fn sessions(&self) -> Result<Vec<SessionRow>, Error> {
        self.session_rows("ORDER BY started_at, id", None)
    }

    /// The latest `limit` sessions, the newest first (EP-13).
    pub fn recent_sessions(&self, limit: usize) -> Result<Vec<SessionRow>, Error> {
        self.session_rows("ORDER BY started_at DESC, id DESC LIMIT ?1", Some(limit))
    }

    fn session_rows(&self, order: &str, limit: Option<usize>) -> Result<Vec<SessionRow>, Error> {
        let columns = if self.version >= 3 {
            SESSION_COLUMNS
        } else {
            SESSION_COLUMNS_V2
        };
        let mut statement = self
            .connection
            .prepare(&format!("SELECT {columns} FROM session {order}"))?;
        let read = |row: &rusqlite::Row<'_>| {
            Ok(SessionRow {
                id: row.get(0)?,
                session_id: row.get(1)?,
                project: row.get(2)?,
                ep: row.get(3)?,
                mode: row.get(4)?,
                model: row.get(5)?,
                effort: row.get(6)?,
                ultracode: row.get(7)?,
                started_at: row.get(8)?,
                ended_at: row.get(9)?,
                outcome: row.get(10)?,
                cost_usd: row.get(11)?,
                questions: row.get(12)?,
                result: row.get(13)?,
                context_percent: row.get(14)?,
                answer_latency_s: row.get(15)?,
            })
        };
        let rows = match limit {
            Some(limit) => statement
                .query_map([limit as i64], read)?
                .collect::<Result<_, _>>()?,
            None => statement.query_map([], read)?.collect::<Result<_, _>>()?,
        };
        Ok(rows)
    }

    /// Every cached reading; a row whose time cannot be read is left out.
    pub fn cached(&self) -> Result<Vec<Cached>, Error> {
        let mut statement = self.connection.prepare(
            "SELECT project, source, fetched_at, data FROM badge_cache ORDER BY project, source",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        let mut cached = Vec::new();
        for row in rows {
            let (project, source, fetched_at, data) = row?;
            if let Ok(at) = DateTime::parse_from_rfc3339(&fetched_at) {
                cached.push(Cached {
                    project,
                    source,
                    fetched_at: at.with_timezone(&Utc),
                    data,
                });
            }
        }
        Ok(cached)
    }
}

/// A time as the ledger writes it: RFC 3339 in UTC, to the second.
fn rfc3339(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// A time the ledger wrote; `None` for text that does not read as one.
fn parse_time(text: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|at| at.with_timezone(&Utc))
}

/// `text` cut to `max` characters, the last one an ellipsis when it was longer.
fn capped(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut short: String = text.chars().take(max - 1).collect();
    short.push('…');
    short
}

/// The schema version written in the file; 0 for a new file.
fn version(connection: &Connection) -> rusqlite::Result<u32> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_version')",
        [],
        |row| row.get(0),
    )?;
    if !exists {
        return Ok(0);
    }
    connection.query_row("SELECT version FROM schema_version", [], |row| row.get(0))
}

/// Brings the schema up to `migrations`, one step per transaction. Each step takes the write lock
/// before it reads the version again, so that two openers at once (the window's threads, `crw`)
/// never run the same step twice (EP-13).
fn migrate(connection: &mut Connection, migrations: &[&str]) -> Result<(), Error> {
    let supported = migrations.len() as u32;
    loop {
        let found = version(connection)?;
        if found > supported {
            return Err(Error::Newer { found, supported });
        }
        if found == supported {
            return Ok(());
        }
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let step = version(&transaction)?;
        if step != found {
            // Another opener ran it meanwhile: read again.
            continue;
        }
        transaction.execute_batch(migrations[step as usize])?;
        transaction.execute("DELETE FROM schema_version", [])?;
        transaction.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [step + 1],
        )?;
        transaction.commit()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    fn tables(ledger: &Ledger) -> Vec<String> {
        let mut statement = ledger
            .connection
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    #[test]
    fn creates_the_schema_once() {
        let scratch = Scratch::new("ledger-create");
        let path = scratch.0.join("local").join("ledger.sqlite");
        let ledger = Ledger::open(&path).unwrap();
        assert_eq!(ledger.schema_version().unwrap(), SCHEMA_VERSION);
        assert_eq!(
            tables(&ledger),
            [
                "badge_cache",
                "event",
                "schema_version",
                "session",
                "snapshot",
                "usage_sample"
            ]
        );
        ledger
            .connection
            .execute(
                "INSERT INTO session (project, mode, started_at) VALUES ('alpha', 'headless', '2026-10-31T19:30:00Z')",
                [],
            )
            .unwrap();
        // The checks hold the vocabularies of D-16 and D-26.
        assert!(
            ledger
                .connection
                .execute(
                    "INSERT INTO session (project, mode, started_at) VALUES ('alpha', 'bogus', 'x')",
                    [],
                )
                .is_err()
        );
        drop(ledger);
        // Opening again keeps the rows and runs no migration.
        let ledger = Ledger::open(&path).unwrap();
        let rows: u32 = ledger
            .connection
            .query_row("SELECT COUNT(*) FROM session", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rows, 1);
    }

    #[test]
    fn migrates_an_older_schema_and_refuses_a_newer_one() {
        let scratch = Scratch::new("ledger-migrate");
        let path = scratch.0.join("ledger.sqlite");
        // A ledger EP-2 wrote, at version 1, with a row in it.
        let first = Ledger::open_with(&path, &[V1]).unwrap();
        first
            .connection
            .execute(
                "INSERT INTO event (at, kind) VALUES ('2026-10-31T19:30:00Z', 'kept')",
                [],
            )
            .unwrap();
        drop(first);
        let ledger = Ledger::open(&path).unwrap();
        assert_eq!(ledger.schema_version().unwrap(), SCHEMA_VERSION);
        assert!(tables(&ledger).contains(&"badge_cache".to_owned()));
        drop(ledger);

        let next = [V1, V2, V3, "ALTER TABLE event ADD COLUMN source TEXT;"];
        let ledger = Ledger::open_with(&path, &next).unwrap();
        assert_eq!(ledger.schema_version().unwrap(), 4);
        ledger
            .connection
            .execute(
                "INSERT INTO event (at, kind, source) VALUES ('t', 'k', 's')",
                [],
            )
            .unwrap();
        let rows: u32 = ledger
            .connection
            .query_row("SELECT COUNT(*) FROM event", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rows, 2, "the migrations kept the earlier row");
        drop(ledger);
        // This build knows up to version 3 and leaves the file at version 4.
        assert!(matches!(
            Ledger::open(&path),
            Err(Error::Newer {
                found: 4,
                supported: 3
            })
        ));
        assert!(matches!(
            Ledger::open_read_only(&path),
            Err(Error::Newer { found: 4, .. })
        ));
        assert_eq!(
            Ledger::open_with(&path, &next)
                .unwrap()
                .schema_version()
                .unwrap(),
            4
        );
    }

    /// The window's loader thread and its first frame both open the ledger at its start: a new
    /// file, or one a migration is due on, takes every opener (EP-13).
    #[test]
    fn openers_at_once_migrate_once() {
        let scratch = Scratch::new("ledger-race");
        for round in 0..4 {
            let path = scratch.0.join(format!("ledger-{round}.sqlite"));
            if round % 2 == 1 {
                // A ledger an older build left at schema 2.
                drop(Ledger::open_with(&path, &[V1, V2]).unwrap());
            }
            let start = std::sync::Arc::new(std::sync::Barrier::new(4));
            let openers: Vec<_> = (0..4)
                .map(|_| {
                    let (path, start) = (path.clone(), std::sync::Arc::clone(&start));
                    std::thread::spawn(move || {
                        start.wait();
                        Ledger::open(&path).map(|ledger| ledger.schema_version().unwrap())
                    })
                })
                .collect();
            for opener in openers {
                assert_eq!(opener.join().unwrap().unwrap(), SCHEMA_VERSION);
            }
        }
    }

    fn at(text: &str) -> DateTime<Utc> {
        parse_time(text).unwrap()
    }

    fn launch<'a>(project: &'a str, mode: &'a str, started: &str) -> NewSession<'a> {
        NewSession {
            session_id: None,
            project,
            ep: Some("EP-4"),
            mode,
            model: None,
            effort: None,
            ultracode: false,
            started_at: at(started),
        }
    }

    #[test]
    fn schema_2_rows_read_before_and_after_the_migration() {
        let scratch = Scratch::new("ledger-v2");
        let path = scratch.0.join("ledger.sqlite");
        // A ledger EP-12's build wrote: schema 2, a manual row.
        let older = Ledger::open_with(&path, &[V1, V2]).unwrap();
        older
            .connection
            .execute(
                "INSERT INTO session (session_id, project, mode, started_at) \
                 VALUES ('s-1', 'alpha', 'manual', '2026-11-02T10:00:00Z')",
                [],
            )
            .unwrap();
        drop(older);
        // Read as it stands: nothing migrated, no result yet.
        let read = Ledger::open_read_only(&path).unwrap();
        assert_eq!(read.schema_version().unwrap(), 2);
        let rows = read.recent_sessions(10).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (rows[0].result.as_deref(), rows[0].context_percent),
            (None, None)
        );
        drop(read);
        assert_eq!(
            Ledger::open_read_only(&path)
                .unwrap()
                .schema_version()
                .unwrap(),
            2
        );
        // The window's open brings it to schema 3 with the row kept.
        let ledger = Ledger::open(&path).unwrap();
        assert_eq!(ledger.schema_version().unwrap(), 3);
        assert_eq!(
            ledger.sessions().unwrap()[0].session_id.as_deref(),
            Some("s-1")
        );
        // A ledger that is not there is not made by a read.
        assert!(Ledger::open_read_only(&scratch.0.join("none.sqlite")).is_err());
        assert!(!scratch.0.join("none.sqlite").exists());
    }

    #[test]
    fn a_visual_launch_no_session_takes_up_lapses() {
        let scratch = Scratch::new("ledger-lapse");
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let window = TimeDelta::minutes(30);
        let old = ledger
            .start_session(&launch("alpha", "visual", "2026-11-02T08:00:00Z"))
            .unwrap();
        let recent = ledger
            .start_session(&launch("alpha", "visual", "2026-11-02T09:50:00Z"))
            .unwrap();
        let claimed = ledger
            .start_session(&launch("beta", "visual", "2026-11-02T08:10:00Z"))
            .unwrap();
        ledger
            .claim_visual("beta", "s-9", at("2026-11-02T07:45:00Z"))
            .unwrap();
        let now = at("2026-11-02T10:00:00Z");
        let rows = ledger.recent_sessions(10).unwrap();
        let row = |id: i64| rows.iter().find(|row| row.id == id).unwrap().clone();
        // Before a window closes it, a launch past its window reads as lapsed already.
        assert!(row(old).lapsed(now, window));
        assert!(!row(recent).lapsed(now, window));
        assert!(!row(claimed).lapsed(now, window));
        assert_eq!(row(old).state(now, window), RowState::Lapsed);
        assert_eq!(row(recent).state(now, window), RowState::Launched);
        assert_eq!(row(claimed).state(now, window), RowState::Open);
        assert_eq!(row(old).took(), None);
        assert_eq!(ledger.lapse_visual(now, window).unwrap(), 1);
        assert_eq!(ledger.lapse_visual(now, window).unwrap(), 0, "closed once");
        let rows = ledger.recent_sessions(10).unwrap();
        let lapsed = rows.iter().find(|row| row.id == old).unwrap();
        assert_eq!(lapsed.ended_at.as_deref(), Some("2026-11-02T08:30:00Z"));
        assert_eq!(
            (lapsed.outcome.as_deref(), lapsed.session_id.as_deref()),
            (None, None)
        );
        assert!(lapsed.lapsed(now, window));
        assert_eq!(
            lapsed.took(),
            None,
            "a launch no session took up had no run"
        );
        assert_eq!(length(window), "30 min");
        assert_eq!(length(TimeDelta::seconds(36)), "36 s");
        assert_eq!(length(TimeDelta::minutes(64)), "1 h 04 min");
        // A lapsed launch is claimed by no session.
        assert_eq!(
            ledger
                .claim_visual("alpha", "s-1", at("2026-11-02T07:00:00Z"))
                .unwrap(),
            Some(recent)
        );
        // The newest first.
        assert_eq!(
            rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            [recent, claimed, old]
        );
        assert_eq!(ledger.recent_sessions(1).unwrap().len(), 1);
    }

    #[test]
    fn a_session_row_is_started_then_finished() {
        let scratch = Scratch::new("ledger-session");
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let at = |text: &str| {
            DateTime::parse_from_rfc3339(text)
                .unwrap()
                .with_timezone(&Utc)
        };
        let id = ledger
            .start_session(&NewSession {
                session_id: Some("0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b"),
                project: "alpha",
                ep: Some("EP-3"),
                mode: "headless",
                model: Some("claude-fable-5-1"),
                effort: Some("xhigh"),
                ultracode: false,
                started_at: at("2026-11-02T05:00:00Z"),
            })
            .unwrap();
        let running = &ledger.sessions().unwrap()[0];
        assert_eq!(running.id, id);
        assert_eq!(running.outcome, None);
        assert_eq!(running.started_at, "2026-11-02T05:00:00Z");
        let finished = |at_text: &str, outcome: &'static str| Finished {
            at: at(at_text),
            outcome,
            cost_usd: Some(1.25),
            questions: 2,
            result: Some("EP-3 is done: the parser reads the five-column tables."),
            answer_latency_s: Some(42.5),
        };
        ledger
            .finish_session(id, &finished("2026-11-02T05:40:00Z", "completed"))
            .unwrap();
        let row = &ledger.sessions().unwrap()[0];
        assert_eq!(
            (
                row.session_id.as_deref(),
                row.project.as_str(),
                row.ep.as_deref(),
                row.mode.as_str(),
                row.model.as_deref(),
                row.effort.as_deref(),
                row.ultracode,
            ),
            (
                Some("0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b"),
                "alpha",
                Some("EP-3"),
                "headless",
                Some("claude-fable-5-1"),
                Some("xhigh"),
                false
            )
        );
        assert_eq!(row.ended_at.as_deref(), Some("2026-11-02T05:40:00Z"));
        assert_eq!(row.outcome.as_deref(), Some("completed"));
        assert_eq!((row.cost_usd, row.questions), (Some(1.25), 2));
        assert_eq!(row.answer_latency_s, Some(42.5));
        assert_eq!(
            row.result.as_deref(),
            Some("EP-3 is done: the parser reads the five-column tables.")
        );
        // The schema's vocabulary holds: no outcome outside D-26's four.
        assert!(
            ledger
                .finish_session(id, &finished("2026-11-02T05:41:00Z", "vanished"))
                .is_err()
        );
        // A long final text is kept to its cap.
        let long = "z".repeat(RESULT_CAP + 50);
        ledger
            .finish_session(
                id,
                &Finished {
                    result: Some(&long),
                    ..finished("2026-11-02T05:42:00Z", "completed")
                },
            )
            .unwrap();
        let kept = ledger.sessions().unwrap()[0].result.clone().unwrap();
        assert_eq!(kept.chars().count(), RESULT_CAP);
        assert!(kept.ends_with('…'));
    }

    #[test]
    fn a_seen_session_claims_its_visual_launch_or_has_a_row_of_its_own() {
        let scratch = Scratch::new("ledger-seen");
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let at = |text: &str| {
            DateTime::parse_from_rfc3339(text)
                .unwrap()
                .with_timezone(&Utc)
        };
        let launch = |project, started: &str| NewSession {
            session_id: None,
            project,
            ep: Some("EP-4"),
            mode: "visual",
            model: None,
            effort: None,
            ultracode: false,
            started_at: at(started),
        };
        let old = ledger
            .start_session(&launch("alpha", "2026-11-02T08:00:00Z"))
            .unwrap();
        let newer = ledger
            .start_session(&launch("alpha", "2026-11-02T09:50:00Z"))
            .unwrap();
        ledger
            .start_session(&launch("beta", "2026-11-02T09:55:00Z"))
            .unwrap();
        let since = at("2026-11-02T09:30:00Z");
        // The newest open launch of the project within the window, in any case of its name.
        assert_eq!(
            ledger.claim_visual("ALPHA", "s-1", since).unwrap(),
            Some(newer)
        );
        assert_eq!(ledger.mode_of("s-1").unwrap().as_deref(), Some("visual"));
        // The older one is out of the window, and a claimed row is not claimed twice.
        assert_eq!(ledger.claim_visual("alpha", "s-2", since).unwrap(), None);
        assert_eq!(ledger.mode_of("s-2").unwrap(), None);
        // A session with no launch to claim has a manual row; its end closes it, once.
        ledger
            .start_session(&NewSession {
                session_id: Some("s-2"),
                project: "alpha",
                ep: None,
                mode: "manual",
                model: None,
                effort: None,
                ultracode: false,
                started_at: at("2026-11-02T10:00:00Z"),
            })
            .unwrap();
        // The end keeps what the status line last gave, when it gave anything.
        assert!(
            ledger
                .end_seen("s-2", at("2026-11-02T10:40:00Z"), Some(0.87), Some(41.0))
                .unwrap()
        );
        assert!(
            !ledger
                .end_seen("s-2", at("2026-11-02T10:41:00Z"), None, None)
                .unwrap()
        );
        assert!(
            ledger
                .end_seen("s-1", at("2026-11-02T10:45:00Z"), None, None)
                .unwrap()
        );
        // A headless row is the runner's.
        ledger
            .start_session(&NewSession {
                session_id: Some("s-3"),
                project: "beta",
                ep: Some("EP-2"),
                mode: "headless",
                model: Some("claude-opus-5-5"),
                effort: Some("high"),
                ultracode: false,
                started_at: at("2026-11-02T11:00:00Z"),
            })
            .unwrap();
        assert!(
            !ledger
                .end_seen("s-3", at("2026-11-02T11:10:00Z"), Some(1.0), None)
                .unwrap()
        );
        let rows = ledger.sessions().unwrap();
        let row = |id: i64| rows.iter().find(|row| row.id == id).unwrap();
        assert_eq!(row(old).session_id, None);
        assert_eq!(row(newer).outcome.as_deref(), Some("completed"));
        assert_eq!(row(newer).ended_at.as_deref(), Some("2026-11-02T10:45:00Z"));
        assert_eq!(
            (row(newer).cost_usd, row(newer).context_percent),
            (None, None)
        );
        let manual = rows.iter().find(|r| r.mode == "manual").unwrap();
        assert_eq!(manual.ended_at.as_deref(), Some("2026-11-02T10:40:00Z"));
        assert_eq!(
            (manual.cost_usd, manual.context_percent),
            (Some(0.87), Some(41.0))
        );
        assert_eq!(
            rows.iter().find(|r| r.mode == "headless").unwrap().outcome,
            None
        );
    }

    #[test]
    fn the_cache_keeps_one_reading_per_project_and_source() {
        let scratch = Scratch::new("ledger-cache");
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let at = |text: &str| {
            DateTime::parse_from_rfc3339(text)
                .unwrap()
                .with_timezone(&Utc)
        };
        ledger
            .cache("", "account", at("2026-11-01T10:00:00Z"), "[]")
            .unwrap();
        ledger
            .cache("alpha", "head", at("2026-11-01T10:00:05Z"), "{\"a\":1}")
            .unwrap();
        ledger
            .cache(
                "alpha",
                "head",
                at("2026-11-01T11:00:00-01:00"),
                "{\"a\":2}",
            )
            .unwrap();
        let rows = ledger.cached().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].source, "account");
        assert_eq!(
            rows[1],
            Cached {
                project: "alpha".to_owned(),
                source: "head".to_owned(),
                fetched_at: at("2026-11-01T12:00:00Z"),
                data: "{\"a\":2}".to_owned(),
            }
        );
    }
}
