//! The ledger (D-8, D-26): one SQLite file in the local profile for what the tool records over
//! time, kept indefinitely: sessions, usage samples, parse snapshots and events, and the cache of
//! the badges' networked readings (EP-7). EP-2 creates the schema; each brief that records rows
//! adds its queries (EP-9 the sessions'), and a numbered migration when it needs a new table or
//! column. Times are RFC 3339 text in UTC.

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, io};

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{Connection, OptionalExtension};

/// The schema this build writes: the number of migrations.
pub const SCHEMA_VERSION: u32 = 2;

/// `MIGRATIONS[i]` brings the schema from version `i` to `i + 1`, each in one transaction.
const MIGRATIONS: [&str; SCHEMA_VERSION as usize] = [V1, V2];

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
    /// `completed`, `stopped`, `failed` or `deferred`; none while it runs.
    pub outcome: Option<String>,
    pub cost_usd: Option<f64>,
    pub questions: u32,
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
        Ok(Self { connection })
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

    /// Records how a session ended: its end time, its outcome, the CLI's cost estimate and the
    /// questions it asked.
    pub fn finish_session(
        &self,
        id: i64,
        ended_at: DateTime<Utc>,
        outcome: &str,
        cost_usd: Option<f64>,
        questions: u32,
    ) -> Result<(), Error> {
        self.connection.execute(
            "UPDATE session SET ended_at = ?2, outcome = ?3, cost_usd = ?4, questions = ?5 \
             WHERE id = ?1",
            (id, rfc3339(ended_at), outcome, cost_usd, questions),
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
    /// (EP-12). A headless row is the runner's to close; `false` when no row was open.
    pub fn end_seen(&self, session_id: &str, at: DateTime<Utc>) -> Result<bool, Error> {
        let changed = self.connection.execute(
            "UPDATE session SET ended_at = ?2, outcome = 'completed' WHERE session_id = ?1 \
             AND mode IN ('visual', 'manual') AND ended_at IS NULL",
            (session_id, rfc3339(at)),
        )?;
        Ok(changed > 0)
    }

    /// Every session, the oldest first.
    pub fn sessions(&self) -> Result<Vec<SessionRow>, Error> {
        let mut statement = self.connection.prepare(
            "SELECT id, session_id, project, ep, mode, model, effort, ultracode, started_at, \
             ended_at, outcome, cost_usd, questions FROM session ORDER BY started_at, id",
        )?;
        let rows = statement.query_map([], |row| {
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
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
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

fn migrate(connection: &mut Connection, migrations: &[&str]) -> Result<(), Error> {
    let found = version(connection)?;
    let supported = migrations.len() as u32;
    if found > supported {
        return Err(Error::Newer { found, supported });
    }
    for (step, sql) in migrations.iter().enumerate().skip(found as usize) {
        let transaction = connection.transaction()?;
        transaction.execute_batch(sql)?;
        transaction.execute("DELETE FROM schema_version", [])?;
        transaction.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [step as u32 + 1],
        )?;
        transaction.commit()?;
    }
    Ok(())
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
        assert_eq!(ledger.schema_version().unwrap(), 2);
        assert!(tables(&ledger).contains(&"badge_cache".to_owned()));
        drop(ledger);

        let next = [V1, V2, "ALTER TABLE event ADD COLUMN source TEXT;"];
        let ledger = Ledger::open_with(&path, &next).unwrap();
        assert_eq!(ledger.schema_version().unwrap(), 3);
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
        // This build knows up to version 2 and leaves the file at version 3.
        assert!(matches!(
            Ledger::open(&path),
            Err(Error::Newer {
                found: 3,
                supported: 2
            })
        ));
        assert_eq!(
            Ledger::open_with(&path, &next)
                .unwrap()
                .schema_version()
                .unwrap(),
            3
        );
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
        ledger
            .finish_session(id, at("2026-11-02T05:40:00Z"), "completed", Some(1.25), 2)
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
        // The schema's vocabulary holds: no outcome outside D-26's four.
        assert!(
            ledger
                .finish_session(id, at("2026-11-02T05:41:00Z"), "vanished", None, 0)
                .is_err()
        );
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
        assert!(ledger.end_seen("s-2", at("2026-11-02T10:40:00Z")).unwrap());
        assert!(!ledger.end_seen("s-2", at("2026-11-02T10:41:00Z")).unwrap());
        assert!(ledger.end_seen("s-1", at("2026-11-02T10:45:00Z")).unwrap());
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
        assert!(!ledger.end_seen("s-3", at("2026-11-02T11:10:00Z")).unwrap());
        let rows = ledger.sessions().unwrap();
        let row = |id: i64| rows.iter().find(|row| row.id == id).unwrap();
        assert_eq!(row(old).session_id, None);
        assert_eq!(row(newer).outcome.as_deref(), Some("completed"));
        assert_eq!(row(newer).ended_at.as_deref(), Some("2026-11-02T10:45:00Z"));
        let manual = rows.iter().find(|r| r.mode == "manual").unwrap();
        assert_eq!(manual.ended_at.as_deref(), Some("2026-11-02T10:40:00Z"));
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
