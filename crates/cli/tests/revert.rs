//! `crw revert` on a scratch home (EP-15b): the preview of a project's latest session, the
//! refusal while the live sessions cannot be asked (the registry's `claude_cli` names no program,
//! so the real CLI is never run, D-29), the project's switch, and the short id `crw sessions`
//! prints. The registry, the ledger and the project live in a scratch `CRW_HOME` with synthetic
//! names, never the owner's (rule 13). The revert itself is tested in `crw_core::revert`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{TimeDelta, Utc};
use crw_core::ledger::{Ledger, NewSession};
use crw_core::revert::Start;
use crw_core::{git, process};

const CRW: &str = env!("CARGO_BIN_EXE_crw");

/// A scratch home holding a registry, a ledger and a data root with one project, `alpha`.
struct Home(PathBuf);

impl Home {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("crw-cli-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("data").join("alpha")).unwrap();
        let registry = format!(
            "format = 1\n\n[settings]\ndata_root = '{}'\nclaude_cli = '{}'\n\n\
             [[project]]\nname = \"alpha\"\npath = \"alpha\"\nstate = \"active\"\n",
            dir.join("data").display(),
            dir.join("no-such-claude.exe").display()
        );
        fs::write(dir.join("registry.toml"), registry).unwrap();
        Self(dir)
    }

    fn project(&self) -> PathBuf {
        self.0.join("data").join("alpha")
    }

    fn git(&self, args: &[&str]) -> String {
        let _turn = process::turn();
        git::run(&self.project(), args).unwrap()
    }

    fn commit(&self, file: &str, text: &str, message: &str) -> String {
        fs::write(self.project().join(file), text).unwrap();
        self.git(&["add", "-A"]);
        self.git(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "-m",
            message,
        ]);
        self.git(&["rev-parse", "HEAD"]).trim().to_owned()
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
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), text)
}

#[test]
fn a_revert_is_previewed_and_refused_until_the_live_sessions_answer() {
    let home = Home::new("revert");
    home.git(&["init", "--quiet", "-b", "main"]);
    home.git(&["config", "core.autocrlf", "false"]);
    let first = home.commit("a.txt", "one\n", "first");
    // A session started at the first commit, recorded with its start, then committed.
    let started = Utc::now() - TimeDelta::minutes(5);
    let ledger = Ledger::open(&home.0.join("ledger.sqlite")).unwrap();
    let row = ledger
        .start_session(&NewSession {
            session_id: Some("0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b"),
            project: "alpha",
            ep: Some("EP-2"),
            mode: "headless",
            model: Some("claude-opus-5-5"),
            effort: Some("high"),
            ultracode: false,
            started_at: started,
        })
        .unwrap();
    let start = Start {
        at: started,
        head: Some(first.clone()),
        branch: Some("main".to_owned()),
        files: Vec::new(),
        copy: None,
        copied: true,
    };
    ledger.set_start(row, &start.to_json()).unwrap();
    drop(ledger);
    let session = home.commit("a.txt", "two\n", "Session work (EP-2)");

    let (code, out) = crw(&home.0, &["revert", "alpha"]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("Revert alpha's session 0e1d2c3b on EP-2"),
        "{out}"
    );
    assert!(
        out.contains(&format!("Target: {} on main", &first[..8])),
        "{out}"
    );
    assert!(out.contains("Commits undone (1):"), "{out}");
    assert!(
        out.contains(&session[..8]) && out.contains("a.txt: back as it was"),
        "{out}"
    );
    assert!(
        out.contains("Refused: claude agents --json") && out.contains("is not known"),
        "{out}"
    );
    // With --yes it is refused the same, and nothing changes.
    let (code, out) = crw(&home.0, &["revert", "alpha", "0e1d2c3b", "--yes"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(home.git(&["rev-parse", "HEAD"]).trim(), session);
    assert!(
        !home
            .0
            .join("backups")
            .join("alpha")
            .join("reverts")
            .exists()
    );

    // The switch off.
    let (code, out) = crw(&home.0, &["set", "alpha", "revert", "no"]);
    assert_eq!(code, 0, "{out}");
    let (_, out) = crw(&home.0, &["revert", "alpha"]);
    assert!(out.contains("Refused: revert is off for alpha"), "{out}");
    let (code, _) = crw(&home.0, &["set", "alpha", "revert", "maybe"]);
    assert_eq!(code, 2);

    // An unknown project or session, and a stray flag.
    let (code, out) = crw(&home.0, &["revert", "gamma"]);
    assert_eq!(code, 1);
    assert!(out.contains("gamma is not a registered project"), "{out}");
    let (_, out) = crw(&home.0, &["revert", "alpha", "ffffffff"]);
    assert!(out.contains("has the id ffffffff"), "{out}");
    assert_eq!(crw(&home.0, &["revert", "alpha", "--force"]).0, 2);

    // `crw sessions` names the short id `crw revert` takes.
    let (_, out) = crw(&home.0, &["sessions"]);
    assert!(out.contains("0e1d2c3b  alpha EP-2"), "{out}");
}
