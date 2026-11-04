//! `crw usage` on a scratch ledger (EP-17): the endpoint's cached reading, the marks notified,
//! the latest samples and the meter's changes of state, read without a write and without the
//! credentials or the network. The ledger lives in a scratch `CRW_HOME`, never the owner's
//! (rule 13), and its readings are synthetic.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{TimeDelta, Utc};
use crw_core::ledger::Ledger;
use crw_core::usage;

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

const BODY: &str = r#"{
    "five_hour": {"utilization": 23.0, "resets_at": "2099-01-01T03:00:00+00:00"},
    "limits": [
        {"kind": "weekly_all", "percent": 41.0, "resets_at": "2099-01-03T19:00:00+00:00"},
        {"kind": "weekly_scoped", "percent": 82.0, "resets_at": "2099-01-03T19:00:00+00:00",
         "scope": {"model": {"display_name": "Fable"}}}
    ],
    "seven_day_breakdown": {"rows": [{"label": "Claude Code", "percent": 35.0}]},
    "extra_usage": {"is_enabled": false}
}"#;

#[test]
fn usage_reads_the_meters_records_without_writing_them() {
    let home = Home::new("usage");
    let (code, out) = crw(&home.0, &["usage"]);
    assert_eq!(code, 0);
    assert!(out.contains("No ledger yet"), "{out}");
    assert!(!home.ledger().exists());

    let now = Utc::now();
    let ledger = Ledger::open(&home.ledger()).unwrap();
    let (code, out) = crw(&home.0, &["usage"]);
    assert_eq!(code, 0);
    assert!(out.contains("no reading of the endpoint yet"), "{out}");
    assert!(out.contains("no samples yet"), "{out}");

    let reading = usage::parse(BODY.as_bytes()).unwrap();
    let at = now - TimeDelta::minutes(3);
    ledger
        .cache("", usage::CACHE_SOURCE, at, &reading.to_json())
        .unwrap();
    ledger
        .add_sample(&usage::endpoint_sample(&reading, at))
        .unwrap();
    let mut alerts = usage::Alerts::default();
    alerts.check(usage::Which::Fable, reading.fable().unwrap());
    ledger
        .cache("", usage::ALERTS_SOURCE, at, &alerts.to_json())
        .unwrap();
    ledger
        .add_event(
            now - TimeDelta::minutes(10),
            usage::EVENT_KIND,
            None,
            "degraded · the endpoint did not answer: timed out",
        )
        .unwrap();
    drop(ledger);
    let before = fs::read(home.ledger()).unwrap();

    let (code, out) = crw(&home.0, &["usage"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("the endpoint's last reading"), "{out}");
    assert!(out.contains("3 min ago"), "{out}");
    let weekly = out
        .lines()
        .find(|line| line.trim_start().starts_with("weekly"))
        .unwrap();
    assert!(
        weekly.contains("41 %") && weekly.contains("resets 2099-01-0"),
        "{out}"
    );
    assert!(out.contains("Fable"), "{out}");
    assert!(out.contains("by surface: Claude Code 35 %"), "{out}");
    assert!(out.contains("extra usage: off"), "{out}");
    assert!(
        out.contains(
            "the answer's shape: the limits' kinds: weekly_all, weekly_scoped; \
             {five_hour: {utilization: a number, resets_at: a string}"
        ),
        "{out}"
    );
    assert!(
        out.contains("notified in this window: weekly none, Fable 80 %"),
        "{out}"
    );
    assert!(out.contains("the latest sample:"), "{out}");
    assert!(
        out.contains("endpoint    weekly 41 %  Fable 82 %  5 h 23 %"),
        "{out}"
    );
    assert!(
        out.contains("degraded · the endpoint did not answer: timed out"),
        "{out}"
    );
    assert_eq!(
        fs::read(home.ledger()).unwrap(),
        before,
        "a read leaves the ledger as it was"
    );

    let (code, _) = crw(&home.0, &["usage", "--poll"]);
    assert_ne!(code, 0, "crw never polls");
}
