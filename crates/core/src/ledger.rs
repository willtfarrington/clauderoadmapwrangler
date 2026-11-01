//! The ledger (D-8, D-26): one SQLite file in the local profile for what the tool records over
//! time, kept indefinitely: sessions, usage samples, parse snapshots and events, and the cache of
//! the badges' networked readings (EP-7). EP-2 creates the schema; each brief that records rows
//! adds its queries, and a numbered migration when it needs a new table or column. Times are
//! RFC 3339 text in UTC.

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, io};

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::Connection;

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
            (
                project,
                source,
                fetched_at.to_rfc3339_opts(SecondsFormat::Secs, true),
                data,
            ),
        )?;
        Ok(())
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
