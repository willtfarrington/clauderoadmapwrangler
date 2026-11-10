//! `crw formalise` on scratch folders (EP-21): the preview writes nothing, `--migration` prints
//! the prompt for an older roadmap and refuses a house one, and the flags are checked. The home is
//! a scratch `CRW_HOME`, never the owner's (rule 13); `--yes` is left to the engine's tests, since
//! it asks the real CLI whether a session works in the project, which tests never run (rule 5).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CRW: &str = env!("CARGO_BIN_EXE_crw");

const PACKET: &str = "# alpha — Roadmap\n\n## Phase 1 — Start\n\n\
| # | Packet | Size | Depends on | Status |\n|---|---|---|---|---|\n\
| EP-1 | Scaffold | S | — | [x] 9bcb7b2 |\n| EP-2 | Parser | M | EP-1 | [ ] |\n";

const HOUSE: &str = "# beta — Roadmap\n\n## Phase P1 — Start\n\n\
| # | Brief | Size | Depends on | Core | Done |\n|---|---|---|---|---|---|\n\
| EP-1 | Scaffold | S | — | core | ☐ |\n";

/// A scratch folder with a home and a data root, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("crw-cli-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("home")).unwrap();
        Self(dir)
    }

    fn home(&self) -> PathBuf {
        self.0.join("home")
    }

    /// A project folder of the data root with a rule file and a roadmap README.
    fn project(&self, name: &str, rules: &str, readme: &str) -> PathBuf {
        let folder = self.0.join("data").join(name);
        fs::create_dir_all(folder.join("roadmap")).unwrap();
        fs::write(folder.join("CLAUDE.md"), rules).unwrap();
        fs::write(folder.join("roadmap").join("README.md"), readme).unwrap();
        folder
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

#[test]
fn the_preview_shows_each_file_and_writes_nothing() {
    let scratch = Scratch::new("formalise-preview");
    let folder = scratch.project("alpha", "# alpha\n", PACKET);
    let (code, printed) = crw(&scratch.home(), &["formalise", folder.to_str().unwrap()]);
    assert_eq!(code, 0, "{printed}");
    // A folder that is not registered is named by its path in the commands printed.
    let target = format!("\"{}\"", folder.display());
    for words in [
        "CLAUDE.md               the rule block added at the end, the file's 1 line kept"
            .to_owned(),
        "roadmap/owner-tasks.md  created with its header".to_owned(),
        "roadmap/inbox.md        created with its header".to_owned(),
        "roadmap/_TEMPLATE.md    created".to_owned(),
        format!("the work-packet tables, read-only: `crw formalise {target} --migration`"),
        "    <!-- roadmap-wrangler:begin -->".to_owned(),
        format!("Nothing changed. `crw formalise {target} --yes` writes."),
    ] {
        assert!(printed.contains(&words), "{words}\n{printed}");
    }
    assert_eq!(
        fs::read_to_string(folder.join("CLAUDE.md")).unwrap(),
        "# alpha\n"
    );
    let mut names: Vec<String> = fs::read_dir(folder.join("roadmap"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["README.md"]);
}

#[test]
fn the_migration_prompt_is_for_an_older_roadmap_only() {
    let scratch = Scratch::new("formalise-migration");
    let alpha = scratch.project("alpha", "# alpha\n", PACKET);
    let (code, printed) = crw(
        &scratch.home(),
        &["formalise", alpha.to_str().unwrap(), "--migration"],
    );
    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("Migrate the roadmap of alpha to the house style"));
    assert!(printed.contains("The Packet column becomes Brief"));
    assert!(!alpha.join("roadmap").join("inbox.md").exists());

    let beta = scratch.project("beta", "# beta\n", HOUSE);
    let (code, printed) = crw(
        &scratch.home(),
        &["formalise", beta.to_str().unwrap(), "--migration"],
    );
    assert_eq!(code, 1, "{printed}");
    assert!(printed.contains("beta: no migration to prompt for: its tables are the house's"));
}

#[test]
fn the_flags_are_checked() {
    let scratch = Scratch::new("formalise-flags");
    let folder = scratch.project("gamma", "# gamma\n", HOUSE);
    let target = folder.to_str().unwrap();
    for (args, words) in [
        (vec!["formalise"], "formalise takes one project or folder"),
        (
            vec!["formalise", target, "--yes", "--migration"],
            "formalise takes --yes or --migration, not both",
        ),
        (
            vec!["formalise", target, "--force"],
            "formalise takes --yes or --migration, not --force",
        ),
    ] {
        let (code, printed) = crw(&scratch.home(), &args);
        assert_eq!(code, 2, "{args:?}: {printed}");
        assert!(printed.contains(words), "{args:?}: {printed}");
    }
    assert_eq!(
        fs::read_to_string(folder.join("CLAUDE.md")).unwrap(),
        "# gamma\n"
    );
}
