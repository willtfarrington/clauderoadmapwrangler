//! `crw export site` on a scratch `CRW_HOME` (EP-58b), never the owner's (rule 13): a data
//! repository's card comes from the file it writes itself in the home's `cards` folder, its own
//! folder never there, and `--yes` writes `projects.json` alone into a scratch site. The account's
//! list is seeded into the scratch ledger as the window's GitHub pass caches it; nothing asks
//! GitHub (D-29).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{Local, Utc};
use crw_core::badges::hosting;
use crw_core::badges::online::Reading;
use crw_core::ledger::Ledger;

const CRW: &str = env!("CARGO_BIN_EXE_crw");

/// A scratch folder with a home and a data root, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("crw-cli-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("home")).unwrap();
        fs::create_dir_all(dir.join("data").join("web")).unwrap();
        Self(dir)
    }

    fn home(&self) -> PathBuf {
        self.0.join("home")
    }

    fn site_data(&self) -> PathBuf {
        self.0.join("data").join("web").join("assets").join("data")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `crw` with `args` in `home`: its exit code and what it printed, both streams.
fn crw(home: &Path, args: &[&str]) -> (i32, String) {
    let output = Command::new(CRW)
        .args(args)
        .env("CRW_HOME", home)
        .output()
        .unwrap();
    let mut printed = String::from_utf8_lossy(&output.stdout).into_owned();
    printed.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), printed)
}

/// The account's list as the window caches it: `quarry-data` with `visibility`.
fn seed_account(home: &Path, visibility: &str) {
    let list = hosting::parse_account(&format!(
        r#"[{{"name": "quarry-data", "owner": {{"login": "someone"}},
             "visibility": "{visibility}", "isArchived": false,
             "pushedAt": "2026-11-01T10:00:00Z"}}]"#
    ))
    .unwrap();
    let (project, source, json) = Reading::Account(Ok(list)).to_cache().unwrap();
    let ledger = Ledger::open(&home.join("ledger.sqlite")).unwrap();
    ledger.cache(&project, source, Utc::now(), &json).unwrap();
}

/// A card file for `quarry` as of `day`.
fn card(day: &str) -> String {
    format!(
        r#"{{"as_of": "{day}", "name": "quarry", "badge": "Python",
            "done": 3, "total": 4, "percent": 75,
            "next": [{{"id": "EP-4", "title": "Loader", "size": "M"}}]}}"#
    )
}

#[test]
fn a_data_repositorys_card_comes_from_its_own_file() {
    let scratch = Scratch::new("export-site");
    let home = scratch.home();
    // The data repository's folder (`quarry-data`) does not exist: nothing there is looked at.
    let registry = format!(
        "format = 1\n\n[settings]\ndata_root = '{}'\nsite = \"web\"\n\n\
         [[project]]\nname = \"web\"\npath = \"web\"\nstate = \"active\"\n\n\
         [[project]]\nname = \"quarry\"\npath = \"quarry-data\"\nstate = \"active\"\ndata = true\n",
        scratch.0.join("data").display(),
    );
    fs::write(home.join("registry.toml"), registry).unwrap();

    // No list cached: not public, the file not opened.
    let (code, printed) = crw(&home, &["export", "site"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.contains("left out: a data repository, not public (no account list cached)"),
        "{printed}"
    );
    assert!(
        printed.contains(&format!("cards     {}", home.join("cards").display())),
        "{printed}"
    );

    // Public, no file yet.
    seed_account(&home, "PUBLIC");
    let (_, printed) = crw(&home, &["export", "site"]);
    assert!(
        printed.contains("left out: a data repository, no card file ("),
        "{printed}"
    );

    // Its file: carried, with its day; a malformed one says why.
    fs::create_dir_all(home.join("cards")).unwrap();
    fs::write(home.join("cards").join("quarry.json"), card("2026-11-20")).unwrap();
    let (code, printed) = crw(&home, &["export", "site"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.contains("3 of 4 (75 %), next EP-4 · Python · its own card of 2026-11-20"),
        "{printed}"
    );
    assert!(
        printed.contains("a preview: crw export site --yes writes projects.json there"),
        "{printed}"
    );
    assert!(!scratch.site_data().exists(), "a preview writes nothing");

    // --yes writes projects.json alone, the card in it, as of the export's day.
    let (code, printed) = crw(&home, &["export", "site", "--yes"]);
    assert_eq!(code, 0, "{printed}");
    let names: Vec<String> = fs::read_dir(scratch.site_data())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["projects.json"]);
    let json = fs::read_to_string(scratch.site_data().join("projects.json")).unwrap();
    assert!(
        json.contains(&format!(
            "\"as_of\": \"{}\"",
            Local::now().format("%Y-%m-%d")
        )),
        "{json}"
    );
    assert!(json.contains("\"name\": \"quarry\""), "{json}");
    assert!(json.contains("\"title\": \"Loader\""), "{json}");

    fs::write(
        home.join("cards").join("quarry.json"),
        card("2026-11-20").replace("\"percent\": 75", "\"percent\": 80"),
    )
    .unwrap();
    let (_, printed) = crw(&home, &["export", "site"]);
    assert!(
        printed.contains("left out: a data repository, its card file is malformed: its percent 80"),
        "{printed}"
    );

    // Private on the account's list: left out, the file not read.
    seed_account(&home, "PRIVATE");
    let (_, printed) = crw(&home, &["export", "site"]);
    assert!(
        printed.contains("not public (private on the account's list)"),
        "{printed}"
    );
}
