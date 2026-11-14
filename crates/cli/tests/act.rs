//! `crw act` on a scratch home (EP-34): the preview names the command and the refusals and runs
//! nothing. The home and its registry are a scratch `CRW_HOME`, never the owner's (rule 13), and
//! no test reaches an action whose plan asks `gh` (create-private, delete): tests never run `gh`
//! or the network (D-29).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CRW: &str = env!("CARGO_BIN_EXE_crw");

/// A scratch home with a registry of one project, alpha, over a scratch data root.
struct Home(PathBuf);

impl Home {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("crw-cli-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("data").join("alpha")).unwrap();
        let registry = format!(
            "format = 1\n\n[settings]\ndata_root = '{}'\n\n\
             [[project]]\nname = \"alpha\"\npath = \"alpha\"\nstate = \"active\"\n",
            dir.join("data").display(),
        );
        fs::write(dir.join("registry.toml"), registry).unwrap();
        Self(dir)
    }

    fn project(&self) -> PathBuf {
        self.0.join("data").join("alpha")
    }
}

impl Drop for Home {
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

#[test]
fn the_preview_names_the_command_and_runs_nothing() {
    let home = Home::new("act-remove");
    let git = home.project().join(".git");
    fs::create_dir_all(&git).unwrap();
    let config = "[remote \"origin\"]\n\turl = https://github.com/example/alpha.git\n";
    fs::write(git.join("config"), config).unwrap();
    let (code, printed) = crw(&home.0, &["act", "alpha", "remove-remote"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.contains("  question  Remove alpha's remote?"),
        "{printed}"
    );
    assert!(
        printed.contains(&format!(
            "  runs      git -C \"{}\" remote remove origin",
            home.project().display()
        )) || printed.contains(&format!(
            "  runs      git -C {} remote remove origin",
            home.project().display()
        )),
        "{printed}"
    );
    assert!(printed.contains("github    no list cached"), "{printed}");
    assert!(printed.contains("previewed only"), "{printed}");
    assert_eq!(
        fs::read_to_string(git.join("config")).unwrap(),
        config,
        "the origin is still there"
    );
}

#[test]
fn the_preview_says_each_refusal() {
    let home = Home::new("act-refused");
    let (code, printed) = crw(&home.0, &["act", "alpha", "make-public"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.contains("refused   only a repository the account lists changes its visibility"),
        "{printed}"
    );
    assert!(!printed.contains("  runs "), "{printed}");
    let (code, printed) = crw(&home.0, &["act", "backup"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.contains("refused   no backup job is set"),
        "{printed}"
    );
    let (code, printed) = crw(&home.0, &["act", "alpha", "archive"]);
    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("refused   only an orphan"), "{printed}");
}

#[test]
fn the_arguments_are_checked() {
    let home = Home::new("act-arguments");
    let (code, printed) = crw(&home.0, &["act", "alpha", "publish"]);
    assert_eq!(code, 2, "{printed}");
    assert!(printed.contains("no action called publish"), "{printed}");
    let (code, printed) = crw(&home.0, &["act", "alpha", "backup"]);
    assert_eq!(code, 2, "{printed}");
    assert!(printed.contains("act backup names no project"), "{printed}");
    let (code, printed) = crw(&home.0, &["act", "make-public"]);
    assert_eq!(code, 2, "{printed}");
    assert!(printed.contains("name the project"), "{printed}");
    let (code, printed) = crw(&home.0, &["act", "nowhere", "archive"]);
    assert_eq!(code, 1, "{printed}");
    assert!(
        printed.contains("nowhere is not a registered project"),
        "{printed}"
    );
}
