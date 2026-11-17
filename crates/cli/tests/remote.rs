//! `crw remote` on a scratch ledger (EP-30b): the phone's action log, the newest first, read
//! without a write, as `crw mail` reads the mailman's. The ledger lives in a scratch `CRW_HOME`,
//! never the owner's (rule 13), and its lines name the fixtures' projects and
//! `owner@example.com` (D-3).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{TimeDelta, Utc};
use crw_core::ledger::Ledger;

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
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

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

#[test]
fn remote_prints_the_phones_action_log_newest_first() {
    let home = Home::new("remote");
    let (code, out) = crw(&home.0, &["remote"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.starts_with("No ledger yet at "), "{out}");
    let ledger = Ledger::open(&home.0.join("ledger.sqlite")).unwrap();
    let now = Utc::now();
    ledger
        .add_event(
            now - TimeDelta::minutes(2),
            crw_core::remote::EVENT_KIND,
            None,
            "someone@example.org · GET /m/ · refused: not the owner's login",
        )
        .unwrap();
    ledger
        .add_event(
            now,
            crw_core::remote::EVENT_KIND,
            None,
            "owner@example.com · POST /m/p/harbor/start · allowed",
        )
        .unwrap();
    // Another kind's row is not the phone's.
    ledger
        .add_event(now, "autopilot", Some("harbor"), "chained EP-4 after EP-3")
        .unwrap();
    drop(ledger);
    let (code, out) = crw(&home.0, &["remote"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains(" remote: the latest 2, the newest first "),
        "{out}"
    );
    let allowed = out
        .find("POST /m/p/harbor/start · allowed")
        .expect("the action");
    let refused = out.find("GET /m/ · refused").expect("the refusal");
    assert!(allowed < refused, "the newest first: {out}");
    assert!(!out.contains("chained EP-4"), "{out}");
    let (code, out) = crw(&home.0, &["remote", "--all"]);
    assert_eq!(code, 0);
    assert!(out.contains(" remote: all 2, the newest first "), "{out}");
    let (code, _) = crw(&home.0, &["remote", "everything"]);
    assert_eq!(code, 2, "a wrong argument is a usage error");
}
