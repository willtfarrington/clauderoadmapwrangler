//! `crw sessions` on a scratch ledger (EP-13): the rows the newest first, one project's alone, a
//! visual launch past its window said lapsed, and the ledger left as it was: never made,
//! migrated or written by a read. The ledger lives in a scratch `CRW_HOME`, never the owner's
//! (rule 13).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{DateTime, TimeDelta, Utc};
use crw_core::ledger::{Finished, Ledger, NewSession};

const CRW: &str = env!("CARGO_BIN_EXE_crw");

/// A scratch home, removed when dropped.
struct Home(PathBuf);

impl Home {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("crw-cli-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn ledger(&self) -> PathBuf {
        self.0.join("ledger.sqlite")
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `crw` with `args` in `home`: its exit code and what it printed.
fn crw(home: &Path, args: &[&str]) -> (i32, String) {
    let output = Command::new(CRW)
        .args(args)
        .env("CRW_HOME", home)
        .output()
        .unwrap();
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

fn session<'a>(
    project: &'a str,
    ep: Option<&'a str>,
    mode: &'a str,
    at: DateTime<Utc>,
) -> NewSession<'a> {
    NewSession {
        session_id: (mode != "visual").then_some("0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b"),
        project,
        ep,
        mode,
        model: (mode == "headless").then_some("claude-opus-5-5"),
        effort: (mode == "headless").then_some("high"),
        ultracode: false,
        started_at: at,
    }
}

#[test]
fn sessions_reads_the_ledger_without_writing_it() {
    let home = Home::new("sessions");
    // No ledger yet: said, and none made.
    let (code, out) = crw(&home.0, &["sessions"]);
    assert_eq!(code, 0);
    assert!(out.contains("No ledger yet"), "{out}");
    assert!(!home.ledger().exists());

    let now = Utc::now();
    let ledger = Ledger::open(&home.ledger()).unwrap();
    let run = ledger
        .start_session(&session(
            "alpha",
            Some("EP-3"),
            "headless",
            now - TimeDelta::hours(5),
        ))
        .unwrap();
    ledger
        .finish_session(
            run,
            &Finished {
                at: now - TimeDelta::hours(5) + TimeDelta::minutes(52),
                outcome: "completed",
                cost_usd: Some(3.1),
                questions: 1,
                result: Some("EP-3 is done: the parser reads the five-column tables."),
            },
        )
        .unwrap();
    ledger
        .start_session(&session(
            "beta",
            Some("EP-2"),
            "visual",
            now - TimeDelta::hours(2),
        ))
        .unwrap();
    ledger
        .start_session(&session(
            "alpha",
            None,
            "manual",
            now - TimeDelta::minutes(5),
        ))
        .unwrap();
    drop(ledger);
    let before = fs::read(home.ledger()).unwrap();

    let (code, out) = crw(&home.0, &["sessions"]);
    assert_eq!(code, 0, "{out}");
    let rows: Vec<&str> = out.lines().filter(|line| line.starts_with("  ")).collect();
    assert_eq!(rows.len(), 4, "{out}");
    assert!(out.contains("3 of 3 for every project"), "{out}");
    // The newest first: the manual session, open; the visual launch no session took up; the run
    // with its result's line under it.
    assert!(
        rows[0].contains("alpha") && rows[0].contains("open: no end recorded"),
        "{out}"
    );
    assert!(
        rows[1].contains("beta EP-2") && rows[1].contains("lapsed"),
        "{out}"
    );
    assert!(
        rows[2].contains("alpha EP-3")
            && rows[2].contains("claude-opus-5-5/high")
            && rows[2].contains("52 min")
            && rows[2].contains("completed")
            && rows[2].contains("$3.10"),
        "{out}"
    );
    assert!(rows[3].contains("EP-3 is done"), "{out}");

    // One project's alone.
    let (_, out) = crw(&home.0, &["sessions", "BETA"]);
    assert!(
        out.contains("1 of 1 for BETA") && !out.contains("alpha"),
        "{out}"
    );

    // A read leaves the file as it was; a wrong flag is a usage error.
    assert_eq!(fs::read(home.ledger()).unwrap(), before);
    let (code, _) = crw(&home.0, &["sessions", "--bogus"]);
    assert_eq!(code, 2);
}
