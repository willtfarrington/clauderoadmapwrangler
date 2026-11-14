//! `crw autostart` and `crw path` on a scratch home (EP-28a): the start-at-login entry and the
//! copy of `crw.exe` on PATH are written, read and removed in the home alone, never in the
//! owner's registry or PATH folder (rule 13). The installed exes are stand-ins of a few bytes,
//! never started.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

    /// Stand-ins for the installed exes in the home's install folder.
    fn install(&self) {
        let bin = self.0.join("bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("clauderoadmapwrangler.exe"), "window").unwrap();
        // A build of the tool names its folder, by which its copy on PATH is known.
        fs::write(bin.join("crw.exe"), "crw of clauderoadmapwrangler").unwrap();
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `crw` with `args` in `home`: its exit code and what it printed, both streams. The home's PATH
/// folder comes first on the child's PATH, so that what PATH finds for `crw` is the home's copy.
fn crw(home: &Path, args: &[&str]) -> (i32, String) {
    let inherited = std::env::var_os("PATH").unwrap_or_default();
    let path = std::env::join_paths(
        std::iter::once(home.join("path")).chain(std::env::split_paths(&inherited)),
    )
    .unwrap();
    let output = Command::new(CRW)
        .args(args)
        .env("CRW_HOME", home)
        .env("PATH", path)
        .output()
        .unwrap();
    let mut printed = String::from_utf8_lossy(&output.stdout).into_owned();
    printed.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), printed)
}

#[test]
fn the_start_at_login_entry_is_written_and_removed_in_the_home() {
    let home = Home::new("autostart");
    let file = home.0.join("autostart.txt");
    let (code, printed) = crw(&home.0, &["autostart"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.contains(&format!("the trial's file {}", file.display())),
        "{printed}"
    );
    assert!(printed.contains("not registered"), "{printed}");
    // Nothing installed: refused, and nothing written.
    let (code, printed) = crw(&home.0, &["autostart", "on"]);
    assert_eq!(code, 1, "{printed}");
    assert!(printed.contains("install-shortcut.ps1"), "{printed}");
    assert!(!file.exists());

    home.install();
    let (code, printed) = crw(&home.0, &["autostart", "on"]);
    assert_eq!(code, 0, "{printed}");
    let window = home.0.join("bin").join("clauderoadmapwrangler.exe");
    let command = format!("\"{}\" --autostart", window.display());
    assert_eq!(fs::read_to_string(&file).unwrap(), command);
    assert!(printed.contains(&format!("Wrote {command}")), "{printed}");
    let (code, printed) = crw(&home.0, &["autostart"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.contains("starts the installed window hidden in the tray at login"),
        "{printed}"
    );

    let (code, printed) = crw(&home.0, &["autostart", "off"]);
    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("is removed from"), "{printed}");
    assert!(!file.exists());
    let (code, printed) = crw(&home.0, &["autostart", "off"]);
    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("nothing changed"), "{printed}");
    let (code, printed) = crw(&home.0, &["autostart", "maybe"]);
    assert_eq!(code, 2, "{printed}");
}

#[test]
fn the_copy_on_path_is_placed_and_removed_in_the_home() {
    let home = Home::new("path");
    let copy = home.0.join("path").join("crw.exe");
    // Nothing installed: refused, and nothing written.
    let (code, printed) = crw(&home.0, &["path", "on"]);
    assert_eq!(code, 1, "{printed}");
    assert!(printed.contains("install first"), "{printed}");
    assert!(!copy.exists());
    let (code, printed) = crw(&home.0, &["path"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.contains("none (crw path on places one)"),
        "{printed}"
    );

    home.install();
    let (code, printed) = crw(&home.0, &["path", "on"]);
    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("crw.exe copied"), "{printed}");
    assert!(
        printed.contains("The folder is on PATH: a new shell finds crw."),
        "{printed}"
    );
    assert_eq!(
        fs::read_to_string(&copy).unwrap(),
        "crw of clauderoadmapwrangler"
    );
    let (code, printed) = crw(&home.0, &["path"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.contains(&format!(
            "PATH finds {}, a copy of the installed build",
            copy.display()
        )),
        "{printed}"
    );
    // Placed again: left as it is.
    let (code, printed) = crw(&home.0, &["path", "on"]);
    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("installed as built already"), "{printed}");

    let (code, printed) = crw(&home.0, &["path", "off"]);
    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("is removed from"), "{printed}");
    assert!(!copy.exists());
    let (code, printed) = crw(&home.0, &["path", "off"]);
    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("nothing changed"), "{printed}");
    let (code, printed) = crw(&home.0, &["path", "elsewhere"]);
    assert_eq!(code, 2, "{printed}");
}
