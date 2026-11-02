//! The documented list of live sessions (D-20): `claude agents --json`, an array with each
//! session's `pid`, `cwd`, `kind`, `startedAt` (milliseconds), `sessionId`, `name` and `status`
//! (`busy`, `waiting`, `idle`), and for a background session its `state` and `waitingFor`. It
//! lists the sessions of every folder; the model keeps the ones outside the registry out of sight.
//! Asked one child at a time like the other questions (D-29), with the session markers withheld
//! as for every Claude Code child (D-15 addendum); never by the tests, which read synthetic lists.

use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

use super::Status;
use crate::{process, session};

/// A listing that has not come back by then is given up.
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// One live session as `claude agents --json` lists it.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Agent {
    pub pid: Option<u32>,
    pub cwd: Option<PathBuf>,
    /// `interactive` or `background`.
    pub kind: Option<String>,
    /// Milliseconds since the epoch.
    pub started_at: Option<f64>,
    pub session_id: Option<String>,
    pub name: Option<String>,
    /// `busy`, `waiting` or `idle`.
    pub status: Option<String>,
    /// A background session's `working`, `blocked`, `done`, `failed` or `stopped`.
    pub state: Option<String>,
    /// What a waiting session is blocked on, in Claude Code's words.
    pub waiting_for: Option<Value>,
}

impl Agent {
    pub fn started(&self) -> Option<DateTime<Utc>> {
        self.started_at
            .and_then(|ms| DateTime::from_timestamp_millis(ms as i64))
    }

    pub fn folder(&self) -> &Path {
        self.cwd.as_deref().unwrap_or(Path::new(""))
    }

    fn waits_for(&self) -> Option<String> {
        self.waiting_for
            .as_ref()
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    }

    /// The status the listing gives, with what a waiting session waits for: a background
    /// session's state first, then the status every session has.
    pub fn status_now(&self) -> Option<(Status, Option<String>)> {
        match self.state.as_deref() {
            Some("done" | "stopped") => return Some((Status::Done, None)),
            Some("failed") => return Some((Status::Failed, None)),
            Some("blocked") => return Some((Status::Waiting, self.waits_for())),
            _ => {}
        }
        match self.status.as_deref()? {
            "busy" => Some((Status::Busy, None)),
            "waiting" => Some((Status::Waiting, self.waits_for())),
            "idle" => Some((Status::Idle, None)),
            _ => None,
        }
    }
}

/// The listing's text read; an entry that does not read as a session is passed over.
pub fn parse(text: &str) -> Result<Vec<Agent>, String> {
    let entries: Vec<Value> = serde_json::from_str(text.trim())
        .map_err(|e| format!("claude agents --json printed no list: {e}"))?;
    Ok(entries
        .into_iter()
        .filter_map(|entry| serde_json::from_value(entry).ok())
        .collect())
}

/// Asks `program` (the Claude Code CLI) for its live sessions: one child, in its turn (D-29).
pub fn ask(program: &Path) -> Result<Vec<Agent>, String> {
    let remove = session::markers(env::vars_os().map(|(name, _)| name));
    let _turn = process::turn();
    let text = process::output_without(program, ["agents", "--json"], &remove, TIMEOUT)
        .map_err(|e| format!("claude agents --json: {e}"))?;
    parse(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_live_sessions_and_their_status() {
        let agents = parse(
            r#"[
              {"pid": 4100, "cwd": "C:\\Data\\alpha", "kind": "interactive", "startedAt": 1793658411113,
               "sessionId": "a0", "name": "alpha-b2", "status": "busy"},
              {"pid": 4200, "cwd": "c:\\data\\beta", "kind": "interactive", "startedAt": 1793638904473,
               "sessionId": "b0", "name": "A title", "status": "waiting", "waitingFor": "permission prompt"},
              {"pid": 4300, "cwd": "C:\\Data\\gamma", "kind": "background", "id": "bg-1", "sessionId": "c0",
               "status": "idle", "state": "failed"},
              "not a session",
              {"pid": "not a number"}
            ]"#,
        )
        .unwrap();
        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0].session_id.as_deref(), Some("a0"));
        assert_eq!(
            agents[0].started().unwrap().to_rfc3339(),
            "2026-11-02T22:26:51.113+00:00"
        );
        assert_eq!(agents[0].status_now(), Some((Status::Busy, None)));
        assert_eq!(
            agents[1].status_now(),
            Some((Status::Waiting, Some("permission prompt".to_owned())))
        );
        assert_eq!(agents[2].status_now(), Some((Status::Failed, None)));
        assert_eq!(agents[1].folder(), Path::new(r"c:\data\beta"));
        assert_eq!(parse("[]").unwrap(), []);
        assert!(parse("Usage: claude agents").is_err());
    }
}
