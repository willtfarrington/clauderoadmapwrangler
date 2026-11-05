//! The ledger (D-8, D-26): one SQLite file in the local profile for what the tool records over
//! time, kept indefinitely: sessions, usage samples, parse snapshots and events, and the cache of
//! the badges' networked readings (EP-7). EP-2 creates the schema; each brief that records rows
//! adds its queries (EP-9 the sessions', EP-15 the events'), and a numbered migration when it
//! needs a new table or column. Times are RFC 3339 text in UTC.

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, io};

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior};

/// The schema this build writes: the number of migrations.
pub const SCHEMA_VERSION: u32 = 5;

/// `MIGRATIONS[i]` brings the schema from version `i` to `i + 1`, each in one transaction.
const MIGRATIONS: [&str; SCHEMA_VERSION as usize] = [V1, V2, V3, V4, V5];

/// The most of a run's final text its row keeps, in characters.
const RESULT_CAP: usize = 8000;

/// The most of an event's detail its row keeps, in characters.
const DETAIL_CAP: usize = 1000;

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

const V4: &str = "
-- The project as it stood at the session's start, as JSON: its HEAD, its branch, its uncommitted
-- files and where their copies are (D-26, EP-15b); none when no start was recorded.
ALTER TABLE session ADD COLUMN start TEXT;
";

const V5: &str = "
-- The session's part of the weekly and Fable-scoped figures, in points of the limit: each poll's
-- rise split among the sessions that worked since the poll before (D-23, EP-18a).
ALTER TABLE session ADD COLUMN weekly_share REAL;
ALTER TABLE session ADD COLUMN fable_share REAL;
";

/// The `session` columns a [`SessionRow`] reads, in its order, for a ledger at schema `version`:
/// a column added later reads as none, in a ledger opened to read at an older schema (EP-13).
fn session_columns(version: u32) -> String {
    let since = |added: u32, column: &'static str| if version >= added { column } else { "NULL" };
    [
        "id",
        "session_id",
        "project",
        "ep",
        "mode",
        "model",
        "effort",
        "ultracode",
        "started_at",
        "ended_at",
        "outcome",
        "cost_usd",
        "questions",
        since(3, "result"),
        since(3, "context_percent"),
        "answer_latency_s",
        since(4, "start"),
        "weekly_before",
        "weekly_after",
        "fable_before",
        "fable_after",
        since(5, "weekly_share"),
        since(5, "fable_share"),
    ]
    .join(", ")
}

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
    /// The project at the session's start, as `revert::Start`'s JSON (from EP-15b).
    pub start: Option<String>,
    /// Its share of the weekly and Fable figures, with the figures around it (from EP-18a).
    pub usage: RowUsage,
}

/// A row's usage (D-23; EP-18a): its part of each figure, in points of the limit, added up over
/// the polls it worked between, and the figures before the first of them and after the latest.
/// None before schema 5, and for a session the window never saw work.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RowUsage {
    pub weekly_share: Option<f64>,
    pub fable_share: Option<f64>,
    pub weekly_before: Option<f64>,
    pub weekly_after: Option<f64>,
    pub fable_before: Option<f64>,
    pub fable_after: Option<f64>,
}

/// A session's part of one poll's rise ([`Ledger::add_share`]): its share of each figure, and the
/// figures at the two polls.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Part {
    pub weekly: Option<f64>,
    pub fable: Option<f64>,
    pub weekly_before: Option<f64>,
    pub weekly_after: Option<f64>,
    pub fable_before: Option<f64>,
    pub fable_after: Option<f64>,
}

/// A project's parts over a time (EP-18a): the sums of its sessions' shares, and how many of its
/// sessions have one.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectShare {
    pub project: String,
    pub weekly: Option<f64>,
    pub fable: Option<f64>,
    pub sessions: u32,
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

/// A session's latest row, as [`Ledger::latest_of`] reads it (EP-35).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Latest {
    pub mode: String,
    pub ep: Option<String>,
    /// No end recorded yet.
    pub open: bool,
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

/// A row's share of the usage in words (D-23; EP-18a): `week +0.5 % (27 → 28 %) · Fable +0 %
/// (19 %)`, its part of each figure with the figures around it; none when it has no share.
pub fn shares(row: &SessionRow) -> Option<String> {
    let usage = &row.usage;
    let one = |name: &str, share: Option<f64>, before: Option<f64>, after: Option<f64>| {
        let share = share?;
        let figures = match (before, after) {
            (Some(before), Some(after)) if points(before) != points(after) => {
                format!(" ({} → {} %)", points(before), points(after))
            }
            (_, Some(at)) | (Some(at), None) => format!(" ({} %)", points(at)),
            (None, None) => String::new(),
        };
        Some(format!("{name} +{} %{figures}", points(share)))
    };
    let parts: Vec<String> = [
        one(
            "week",
            usage.weekly_share,
            usage.weekly_before,
            usage.weekly_after,
        ),
        one(
            "Fable",
            usage.fable_share,
            usage.fable_before,
            usage.fable_after,
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// Points of a limit: whole, else to a tenth (`2`, `0.5`, `0.3`).
pub fn points(value: f64) -> String {
    let tenths = (value * 10.0).round();
    if tenths % 10.0 == 0.0 {
        format!("{:.0}", tenths / 10.0)
    } else {
        format!("{:.1}", tenths / 10.0)
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

/// One line of the `event` table (D-26): what happened, to which project, in words; from EP-15
/// the notifications' log (`notify::EVENT_KIND`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRow {
    pub id: i64,
    pub at: String,
    pub kind: String,
    pub project: Option<String>,
    pub detail: Option<String>,
}

impl EventRow {
    pub fn at(&self) -> Option<DateTime<Utc>> {
        parse_time(&self.at)
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

/// One usage reading (D-23; EP-17): an endpoint's poll, or a session's status-line figures.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub taken_at: DateTime<Utc>,
    /// `endpoint` or `statusline`.
    pub source: String,
    /// The session whose status line gave it.
    pub session_id: Option<String>,
    pub weekly_percent: Option<f64>,
    pub weekly_resets_at: Option<DateTime<Utc>>,
    pub fable_percent: Option<f64>,
    pub fable_resets_at: Option<DateTime<Utc>>,
    pub five_hour_percent: Option<f64>,
    pub five_hour_resets_at: Option<DateTime<Utc>>,
}

/// The `usage_sample` columns a [`Sample`] reads, in its order.
const SAMPLE_COLUMNS: &str = "taken_at, source, session_id, weekly_percent, weekly_resets_at, \
    fable_percent, fable_resets_at, five_hour_percent, five_hour_resets_at";

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

    /// Writes a session's start (`revert::Start`'s JSON) into the row `id` when that row has none
    /// (EP-15b); `true` when written.
    pub fn set_start(&self, id: i64, start: &str) -> Result<bool, Error> {
        let changed = self.connection.execute(
            "UPDATE session SET start = ?2 WHERE id = ?1 AND start IS NULL",
            (id, start),
        )?;
        Ok(changed > 0)
    }

    /// Writes a session's start into the first row of `session_id` when no row of it has one: a
    /// start the window read after the session earned its row (EP-15b). `true` when written.
    pub fn set_start_of(&self, session_id: &str, start: &str) -> Result<bool, Error> {
        if self.start_of(session_id)?.is_some() {
            return Ok(false);
        }
        let changed = self.connection.execute(
            "UPDATE session SET start = ?2 WHERE id = \
             (SELECT id FROM session WHERE session_id = ?1 ORDER BY started_at, id LIMIT 1)",
            (session_id, start),
        )?;
        Ok(changed > 0)
    }

    /// The start recorded for `session_id`: the first of its rows that holds one, so that a
    /// resumed session keeps its first run's (EP-15b). None in a ledger older than schema 4.
    pub fn start_of(&self, session_id: &str) -> Result<Option<String>, Error> {
        if self.version < 4 {
            return Ok(None);
        }
        Ok(self
            .connection
            .query_row(
                "SELECT start FROM session WHERE session_id = ?1 AND start IS NOT NULL \
                 ORDER BY started_at, id LIMIT 1",
                [session_id],
                |row| row.get(0),
            )
            .optional()?)
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

    /// The latest row of `session_id`, the one its run goes on in or follows (EP-35): its mode, its
    /// EP and whether it is still open.
    pub fn latest_of(&self, session_id: &str) -> Result<Option<Latest>, Error> {
        Ok(self
            .connection
            .query_row(
                "SELECT mode, ep, ended_at IS NULL FROM session WHERE session_id = ?1 \
                 ORDER BY started_at DESC, id DESC LIMIT 1",
                [session_id],
                |row| {
                    Ok(Latest {
                        mode: row.get(0)?,
                        ep: row.get(1)?,
                        open: row.get(2)?,
                    })
                },
            )
            .optional()?)
    }

    /// Whether an event of `kind` names `session_id` first in its words and happened at `since`
    /// or later: the window's handover of a session's link to VS Code (EP-35).
    pub fn names_session(
        &self,
        kind: &str,
        session_id: &str,
        since: DateTime<Utc>,
    ) -> Result<bool, Error> {
        let found: Option<i64> = self
            .connection
            .query_row(
                "SELECT id FROM event WHERE kind = ?1 AND instr(detail, ?2) = 1 AND at >= ?3 \
                 LIMIT 1",
                (kind, session_id, rfc3339(since)),
                |row| row.get(0),
            )
            .optional()?;
        Ok(found.is_some())
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

    /// Adds a session's part of one poll's rise to the latest row of `session_id` (D-23; EP-18a):
    /// each share added to the row's, the figure before kept from the first part and the one
    /// after from the latest. `false` when no row holds the session, whose part then stays
    /// unattributed, or the ledger is older than schema 5.
    pub fn add_share(&self, session_id: &str, part: &Part) -> Result<bool, Error> {
        if self.version < 5 || session_id.is_empty() {
            return Ok(false);
        }
        let changed = self.connection.execute(
            "UPDATE session SET \
             weekly_share = CASE WHEN ?2 IS NULL THEN weekly_share \
                 ELSE COALESCE(weekly_share, 0) + ?2 END, \
             fable_share = CASE WHEN ?3 IS NULL THEN fable_share \
                 ELSE COALESCE(fable_share, 0) + ?3 END, \
             weekly_before = COALESCE(weekly_before, ?4), \
             weekly_after = COALESCE(?5, weekly_after), \
             fable_before = COALESCE(fable_before, ?6), \
             fable_after = COALESCE(?7, fable_after) \
             WHERE id = (SELECT id FROM session WHERE session_id = ?1 \
                 ORDER BY started_at DESC, id DESC LIMIT 1)",
            (
                session_id,
                part.weekly,
                part.fable,
                part.weekly_before,
                part.weekly_after,
                part.fable_before,
                part.fable_after,
            ),
        )?;
        Ok(changed > 0)
    }

    /// Each project's shares over the sessions started at `since` or later (D-23; EP-18a), the
    /// largest weekly sum first; none before schema 5.
    pub fn shares_since(&self, since: DateTime<Utc>) -> Result<Vec<ProjectShare>, Error> {
        if self.version < 5 {
            return Ok(Vec::new());
        }
        let mut statement = self.connection.prepare(
            "SELECT project, SUM(weekly_share), SUM(fable_share), COUNT(*) FROM session \
             WHERE started_at >= ?1 AND (weekly_share IS NOT NULL OR fable_share IS NOT NULL) \
             GROUP BY project ORDER BY COALESCE(SUM(weekly_share), 0) DESC, project",
        )?;
        let rows = statement.query_map([rfc3339(since)], |row| {
            Ok(ProjectShare {
                project: row.get(0)?,
                weekly: row.get(1)?,
                fable: row.get(2)?,
                sessions: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
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
        let columns = session_columns(self.version);
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
                start: row.get(16)?,
                usage: RowUsage {
                    weekly_before: row.get(17)?,
                    weekly_after: row.get(18)?,
                    fable_before: row.get(19)?,
                    fable_after: row.get(20)?,
                    weekly_share: row.get(21)?,
                    fable_share: row.get(22)?,
                },
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

    /// Keeps one line of the `event` table: what happened at `at`, of `kind`, to `project` when it
    /// concerns one, in words kept to [`DETAIL_CAP`] characters.
    pub fn add_event(
        &self,
        at: DateTime<Utc>,
        kind: &str,
        project: Option<&str>,
        detail: &str,
    ) -> Result<i64, Error> {
        self.connection.execute(
            "INSERT INTO event (at, kind, project, detail) VALUES (?1, ?2, ?3, ?4)",
            (rfc3339(at), kind, project, capped(detail, DETAIL_CAP)),
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    /// The `event` table's lines of `kind`, the newest first: the latest `limit`, or every one.
    pub fn events(&self, kind: &str, limit: Option<usize>) -> Result<Vec<EventRow>, Error> {
        let mut statement = self.connection.prepare(
            "SELECT id, at, kind, project, detail FROM event WHERE kind = ?1 \
             ORDER BY at DESC, id DESC LIMIT ?2",
        )?;
        // SQLite reads a negative limit as none.
        let limit = limit.map_or(-1, |limit| limit as i64);
        let rows = statement.query_map((kind, limit), |row| {
            Ok(EventRow {
                id: row.get(0)?,
                at: row.get(1)?,
                kind: row.get(2)?,
                project: row.get(3)?,
                detail: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// The one cached reading of `project` from `source`, if there is one that reads.
    pub fn cached_one(&self, project: &str, source: &str) -> Result<Option<Cached>, Error> {
        let row = self
            .connection
            .query_row(
                "SELECT fetched_at, data FROM badge_cache WHERE project = ?1 AND source = ?2",
                (project, source),
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        Ok(row.and_then(|(fetched_at, data)| {
            Some(Cached {
                project: project.to_owned(),
                source: source.to_owned(),
                fetched_at: parse_time(&fetched_at)?,
                data,
            })
        }))
    }

    /// Keeps one usage reading (D-23; EP-17).
    pub fn add_sample(&self, sample: &Sample) -> Result<i64, Error> {
        let time = |at: Option<DateTime<Utc>>| at.map(rfc3339);
        self.connection.execute(
            &format!(
                "INSERT INTO usage_sample ({SAMPLE_COLUMNS}) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
            ),
            (
                rfc3339(sample.taken_at),
                &sample.source,
                &sample.session_id,
                sample.weekly_percent,
                time(sample.weekly_resets_at),
                sample.fable_percent,
                time(sample.fable_resets_at),
                sample.five_hour_percent,
                time(sample.five_hour_resets_at),
            ),
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    /// The usage readings taken at `since` or later, the oldest first (the tile's sparkline).
    pub fn samples_since(&self, since: DateTime<Utc>) -> Result<Vec<Sample>, Error> {
        self.samples(
            &format!(
                "SELECT {SAMPLE_COLUMNS} FROM usage_sample WHERE taken_at >= ?1 \
                 ORDER BY taken_at, id"
            ),
            rfc3339(since),
        )
    }

    /// The latest `limit` usage readings, the newest first (`crw usage`).
    pub fn latest_samples(&self, limit: usize) -> Result<Vec<Sample>, Error> {
        self.samples(
            &format!(
                "SELECT {SAMPLE_COLUMNS} FROM usage_sample ORDER BY taken_at DESC, id DESC \
                 LIMIT ?1"
            ),
            i64::try_from(limit).unwrap_or(i64::MAX),
        )
    }

    /// The samples a query of [`SAMPLE_COLUMNS`] gives; a row whose time does not read is left
    /// out.
    fn samples(&self, query: &str, parameter: impl rusqlite::ToSql) -> Result<Vec<Sample>, Error> {
        let mut statement = self.connection.prepare(query)?;
        let rows = statement.query_map([parameter], |row| {
            let time = |at: Option<String>| at.as_deref().and_then(parse_time);
            Ok((
                row.get::<_, String>(0)?,
                Sample {
                    taken_at: DateTime::<Utc>::MIN_UTC,
                    source: row.get(1)?,
                    session_id: row.get(2)?,
                    weekly_percent: row.get(3)?,
                    weekly_resets_at: time(row.get(4)?),
                    fable_percent: row.get(5)?,
                    fable_resets_at: time(row.get(6)?),
                    five_hour_percent: row.get(7)?,
                    five_hour_resets_at: time(row.get(8)?),
                },
            ))
        })?;
        let mut samples = Vec::new();
        for row in rows {
            let (taken_at, mut sample) = row?;
            if let Some(at) = parse_time(&taken_at) {
                sample.taken_at = at;
                samples.push(sample);
            }
        }
        Ok(samples)
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
    fn keeps_events_and_reads_them_back_the_newest_first() {
        let scratch = Scratch::new("ledger-events");
        let path = scratch.0.join("ledger.sqlite");
        let ledger = Ledger::open(&path).unwrap();
        let at = |minute: u32| {
            DateTime::parse_from_rfc3339(&format!("2026-11-03T20:{minute:02}:00Z"))
                .unwrap()
                .with_timezone(&Utc)
        };
        ledger
            .add_event(
                at(1),
                "notification",
                Some("alpha"),
                "toast · waiting · first",
            )
            .unwrap();
        ledger
            .add_event(at(5), "notification", None, "toast · test · second")
            .unwrap();
        ledger
            .add_event(at(3), "revert", Some("alpha"), "another kind")
            .unwrap();
        let long = "x".repeat(DETAIL_CAP + 50);
        ledger
            .add_event(at(4), "notification", Some("beta"), &long)
            .unwrap();
        drop(ledger);
        let read = Ledger::open_read_only(&path).unwrap();
        let all = read.events("notification", None).unwrap();
        let details: Vec<&str> = all.iter().filter_map(|e| e.detail.as_deref()).collect();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0], "toast · test · second");
        assert_eq!(details[1].chars().count(), DETAIL_CAP);
        assert!(details[1].ends_with('…'));
        assert_eq!(details[2], "toast · waiting · first");
        assert_eq!(all[0].project, None);
        assert_eq!(all[2].project.as_deref(), Some("alpha"));
        assert_eq!(all[0].at(), Some(at(5)));
        let latest = read.events("notification", Some(1)).unwrap();
        assert_eq!(latest, all[..1]);
        assert!(read.events("delivery", None).unwrap().is_empty());
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

        let next = [
            V1,
            V2,
            V3,
            V4,
            V5,
            "ALTER TABLE event ADD COLUMN source TEXT;",
        ];
        let ledger = Ledger::open_with(&path, &next).unwrap();
        assert_eq!(ledger.schema_version().unwrap(), 6);
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
        // This build knows up to version 5 and leaves the file at version 6.
        assert!(matches!(
            Ledger::open(&path),
            Err(Error::Newer {
                found: 6,
                supported: 5
            })
        ));
        assert!(matches!(
            Ledger::open_read_only(&path),
            Err(Error::Newer { found: 6, .. })
        ));
        assert_eq!(
            Ledger::open_with(&path, &next)
                .unwrap()
                .schema_version()
                .unwrap(),
            6
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
        // The window's open brings it to this build's schema with the row kept.
        let ledger = Ledger::open(&path).unwrap();
        assert_eq!(ledger.schema_version().unwrap(), SCHEMA_VERSION);
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

    /// A session's start (EP-15b): written once, into its first row, and read from there; a
    /// schema 3 ledger reads with none.
    #[test]
    fn a_sessions_start_is_kept_once_in_its_first_row() {
        let scratch = Scratch::new("ledger-start");
        let path = scratch.0.join("ledger.sqlite");
        let older = Ledger::open_with(&path, &[V1, V2, V3]).unwrap();
        older
            .start_session(&NewSession {
                session_id: Some("s-1"),
                ..launch("alpha", "headless", "2026-11-03T10:00:00Z")
            })
            .unwrap();
        drop(older);
        let read = Ledger::open_read_only(&path).unwrap();
        assert_eq!(read.start_of("s-1").unwrap(), None);
        assert_eq!(read.recent_sessions(1).unwrap()[0].start, None);
        drop(read);

        let ledger = Ledger::open(&path).unwrap();
        let first = ledger.sessions().unwrap()[0].id;
        // A resume's row, later, of the same session.
        let resumed = ledger
            .start_session(&NewSession {
                session_id: Some("s-1"),
                ..launch("alpha", "headless", "2026-11-03T11:00:00Z")
            })
            .unwrap();
        assert!(ledger.set_start_of("s-1", r#"{"first":1}"#).unwrap());
        assert!(!ledger.set_start_of("s-1", r#"{"second":2}"#).unwrap());
        assert!(!ledger.set_start(first, r#"{"third":3}"#).unwrap());
        assert!(ledger.set_start(resumed, r#"{"own":4}"#).unwrap());
        assert_eq!(
            ledger.start_of("s-1").unwrap().as_deref(),
            Some(r#"{"first":1}"#)
        );
        let rows = ledger.sessions().unwrap();
        assert_eq!(rows[0].start.as_deref(), Some(r#"{"first":1}"#));
        assert_eq!(ledger.start_of("s-2").unwrap(), None);
        assert!(!ledger.set_start_of("s-2", "{}").unwrap(), "no row of it");
    }

    /// A session's usage (EP-18a): each poll's part adds up on its latest row, the figure before
    /// kept from the first part and the one after from the latest; a project's sums count the
    /// sessions started since the window's start; a schema 4 ledger reads none and takes none.
    #[test]
    fn a_sessions_parts_add_up_on_its_latest_row() {
        let scratch = Scratch::new("ledger-share");
        let path = scratch.0.join("ledger.sqlite");
        let started = |ledger: &Ledger, id: &'static str, project, mode, at| {
            ledger
                .start_session(&NewSession {
                    session_id: Some(id),
                    ..launch(project, mode, at)
                })
                .unwrap()
        };
        let part = Part {
            weekly: Some(0.5),
            weekly_before: Some(27.0),
            weekly_after: Some(28.0),
            ..Part::default()
        };
        let older = Ledger::open_with(&path, &[V1, V2, V3, V4]).unwrap();
        started(&older, "s-1", "alpha", "visual", "2026-11-03T10:00:00Z");
        assert!(!older.add_share("s-1", &part).unwrap(), "schema 4 has none");
        drop(older);
        let read = Ledger::open_read_only(&path).unwrap();
        assert_eq!(read.sessions().unwrap()[0].usage, RowUsage::default());
        assert!(
            read.shares_since(at("2026-11-01T00:00:00Z"))
                .unwrap()
                .is_empty()
        );
        drop(read);

        let ledger = Ledger::open(&path).unwrap();
        assert!(ledger.add_share("s-1", &part).unwrap());
        let later = Part {
            weekly: Some(1.0),
            fable: Some(0.0),
            weekly_before: Some(28.0),
            weekly_after: Some(29.0),
            fable_before: Some(19.0),
            fable_after: Some(19.0),
        };
        assert!(ledger.add_share("s-1", &later).unwrap());
        assert!(!ledger.add_share("s-2", &later).unwrap(), "no row of it");
        // A resume's row takes the parts from then on.
        started(&ledger, "s-1", "alpha", "headless", "2026-11-03T12:00:00Z");
        assert!(ledger.add_share("s-1", &part).unwrap());
        started(&ledger, "s-3", "beta", "manual", "2026-11-03T11:00:00Z");
        let two = Part {
            weekly: Some(2.0),
            ..Part::default()
        };
        assert!(ledger.add_share("s-3", &two).unwrap());
        // Started before the window: not summed.
        started(&ledger, "s-4", "beta", "manual", "2026-10-30T11:00:00Z");
        assert!(ledger.add_share("s-4", &two).unwrap());

        let rows = ledger.sessions().unwrap();
        let first = &rows[1].usage;
        assert_eq!(
            (first.weekly_share, first.fable_share),
            (Some(1.5), Some(0.0))
        );
        assert_eq!(
            (first.weekly_before, first.weekly_after),
            (Some(27.0), Some(29.0))
        );
        assert_eq!(
            (first.fable_before, first.fable_after),
            (Some(19.0), Some(19.0))
        );
        assert_eq!(rows[3].usage.weekly_share, Some(0.5));
        assert_eq!(
            shares(&rows[1]).as_deref(),
            Some("week +1.5 % (27 → 29 %) · Fable +0 % (19 %)")
        );
        assert_eq!(shares(&rows[2]).as_deref(), Some("week +2 %"));
        assert_eq!(shares(&rows[0]).as_deref(), Some("week +2 %"));
        let none = Ledger::open(&path).unwrap();
        started(&none, "s-5", "gamma", "manual", "2026-11-03T13:00:00Z");
        assert_eq!(shares(&none.sessions().unwrap()[4]), None);
        let project = |project: &str, weekly, fable, sessions| ProjectShare {
            project: project.to_owned(),
            weekly,
            fable,
            sessions,
        };
        assert_eq!(
            ledger.shares_since(at("2026-10-31T17:00:00Z")).unwrap(),
            [
                project("alpha", Some(2.0), Some(0.0), 2),
                project("beta", Some(2.0), None, 1)
            ]
        );
    }

    #[test]
    fn points_are_whole_or_to_a_tenth() {
        assert_eq!(points(2.0), "2");
        assert_eq!(points(0.5), "0.5");
        assert_eq!(points(1.0 / 3.0), "0.3");
        assert_eq!(points(0.96), "1");
        assert_eq!(points(0.0), "0");
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
        assert_eq!(
            ledger.cached_one("alpha", "head").unwrap(),
            Some(rows[1].clone())
        );
        assert_eq!(ledger.cached_one("", "usage").unwrap(), None);
    }

    #[test]
    fn usage_samples_are_kept_and_read_back_by_time() {
        let scratch = Scratch::new("ledger-samples");
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let sample = |taken: &str, source: &str, weekly: f64| Sample {
            taken_at: at(taken),
            source: source.to_owned(),
            session_id: (source == "statusline").then(|| "s-1".to_owned()),
            weekly_percent: Some(weekly),
            weekly_resets_at: Some(at("2026-11-07T19:00:00Z")),
            fable_percent: (source == "endpoint").then_some(60.0),
            fable_resets_at: None,
            five_hour_percent: None,
            five_hour_resets_at: None,
        };
        let samples = [
            sample("2026-11-03T12:00:00Z", "endpoint", 40.0),
            sample("2026-11-04T12:05:00Z", "statusline", 42.0),
            sample("2026-11-04T12:00:00Z", "endpoint", 41.0),
        ];
        for sample in &samples {
            ledger.add_sample(sample).unwrap();
        }
        let since = ledger.samples_since(at("2026-11-04T00:00:00Z")).unwrap();
        assert_eq!(since, [samples[2].clone(), samples[1].clone()]);
        let latest = ledger.latest_samples(2).unwrap();
        assert_eq!(latest, [samples[1].clone(), samples[2].clone()]);
        let wrong = Sample {
            source: "guess".to_owned(),
            ..samples[0].clone()
        };
        assert!(ledger.add_sample(&wrong).is_err(), "the source is checked");
    }
}
