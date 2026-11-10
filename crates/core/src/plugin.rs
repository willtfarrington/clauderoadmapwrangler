//! The build's copy of the roadmap-wrangler plugin (D-27; EP-22a). The plugin's files travel in
//! the build, one table of them carried by `include_str!` from `plugins/roadmap-wrangler/`, and
//! are written under the tool's local folder before a headless start, so that a run gets the
//! plugin of the build that starts it, whatever the working tree holds and whether or not the
//! owner installed the plugin from GitHub: a `--plugin-dir` copy replaces an installed plugin of
//! the same name for its session (Claude Code's loading reference, read 2026-11-10). A file that
//! matches is left alone, one that differs or is missing is written, and one the build no longer
//! has is removed with the folders it leaves empty. A structural test (`tests/plugin.rs`) holds
//! the table to the folder, file for file and byte for byte.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{files, paths};

/// The plugin's name, its folder's name under [`PLUGIN_DIR`] and the name a skill is invoked by
/// (`/roadmap-wrangler:<skill>`).
pub const NAME: &str = "roadmap-wrangler";

/// The copies' folder under the local folder.
pub const PLUGIN_DIR: &str = "plugin";

/// The flag by which a headless run loads the copy for its session; passed only when `claude
/// --help` lists it (D-15).
pub const FLAG: &str = "--plugin-dir";

/// The plugin's files, by their path under its folder (with `/`), as the build carries them.
pub const FILES: [(&str, &str); 6] = [
    (
        ".claude-plugin/plugin.json",
        include_str!("../../../plugins/roadmap-wrangler/.claude-plugin/plugin.json"),
    ),
    (
        "reference/house-style.md",
        include_str!("../../../plugins/roadmap-wrangler/reference/house-style.md"),
    ),
    (
        "skills/formalise/SKILL.md",
        include_str!("../../../plugins/roadmap-wrangler/skills/formalise/SKILL.md"),
    ),
    (
        "skills/plan/SKILL.md",
        include_str!("../../../plugins/roadmap-wrangler/skills/plan/SKILL.md"),
    ),
    (
        "skills/replan/SKILL.md",
        include_str!("../../../plugins/roadmap-wrangler/skills/replan/SKILL.md"),
    ),
    (
        "skills/status/SKILL.md",
        include_str!("../../../plugins/roadmap-wrangler/skills/status/SKILL.md"),
    ),
];

/// `%LOCALAPPDATA%\clauderoadmapwrangler\plugin\roadmap-wrangler`, or the same under `CRW_HOME`
/// in a trial; none without a local folder.
pub fn dir() -> Option<PathBuf> {
    Some(paths::local_dir()?.join(PLUGIN_DIR).join(NAME))
}

/// What a write did to the copy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// The files written, missing or different before, by their path under the folder.
    pub written: Vec<String>,
    /// The files removed, which the build does not carry, by their path under the folder.
    pub removed: Vec<String>,
    /// How many files matched the build already.
    pub unchanged: usize,
}

impl Report {
    /// `3 files written, 1 removed, 2 unchanged`, or `matches the build already`.
    pub fn words(&self) -> String {
        if self.written.is_empty() && self.removed.is_empty() {
            return format!("matches the build already ({} files)", self.unchanged);
        }
        let mut parts = Vec::new();
        match self.written.len() {
            0 => {}
            1 => parts.push("1 file written".to_owned()),
            n => parts.push(format!("{n} files written")),
        }
        match self.removed.len() {
            0 => {}
            1 => parts.push("1 removed".to_owned()),
            n => parts.push(format!("{n} removed")),
        }
        parts.push(format!("{} unchanged", self.unchanged));
        parts.join(", ")
    }
}

/// Writes the build's copy into `dir`, creating it: each file of [`FILES`] that is missing or
/// differs is written atomically, one that matches is left alone, and any other file under `dir`
/// is removed with the folders it leaves empty. `Err` names the first file that could not be
/// written or removed; the files before it stand.
pub fn write(dir: &Path) -> Result<Report, String> {
    let mut report = Report::default();
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (name, text) in FILES {
        let path = dir.join(name.replace('/', "\\"));
        if fs::read(&path).is_ok_and(|bytes| bytes == text.as_bytes()) {
            report.unchanged += 1;
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        files::write_atomic(&path, text.as_bytes())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        report.written.push(name.to_owned());
    }
    let carried: BTreeSet<&str> = FILES.iter().map(|(name, _)| *name).collect();
    for (name, path) in files_under(dir) {
        if carried.contains(name.as_str()) {
            continue;
        }
        fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        report.removed.push(name);
    }
    remove_empty_folders(dir);
    Ok(report)
}

/// Writes the copy where [`dir`] says and gives its folder: what a headless start sets its
/// launch's `plugin_dir` to. `Err` says why there is no copy to load.
pub fn prepare() -> Result<PathBuf, String> {
    let dir = dir().ok_or_else(|| {
        format!(
            "no local folder for the {NAME} plugin's copy: neither LOCALAPPDATA nor {} is set",
            paths::HOME_VAR
        )
    })?;
    write(&dir).map_err(|e| format!("the {NAME} plugin's copy: {e}"))?;
    Ok(dir)
}

/// How the copy in `dir` stands against the build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// No folder there yet.
    None,
    /// Every file of the build is there as built, and nothing else.
    Matches,
    /// What differs, each in words: `skills/plan/SKILL.md differs`, `… missing`, `… not in the
    /// build`.
    Differs(Vec<String>),
}

impl Status {
    /// The copy in `dir` compared with the build, file for file.
    pub fn of(dir: &Path) -> Self {
        if !dir.is_dir() {
            return Status::None;
        }
        let mut differences = Vec::new();
        for (name, text) in FILES {
            match fs::read(dir.join(name.replace('/', "\\"))) {
                Ok(bytes) if bytes == text.as_bytes() => {}
                Ok(_) => differences.push(format!("{name} differs")),
                Err(_) => differences.push(format!("{name} missing")),
            }
        }
        let carried: BTreeSet<&str> = FILES.iter().map(|(name, _)| *name).collect();
        for (name, _) in files_under(dir) {
            if !carried.contains(name.as_str()) {
                differences.push(format!("{name} not in the build"));
            }
        }
        if differences.is_empty() {
            Status::Matches
        } else {
            Status::Differs(differences)
        }
    }

    /// For `crw doctor`: `matches the build`, or what differs and when it is put right.
    pub fn words(&self) -> String {
        match self {
            Status::None => {
                "none yet: written at the first headless start, or by crw install".to_owned()
            }
            Status::Matches => "matches the build".to_owned(),
            Status::Differs(differences) => format!(
                "differs from the build ({}): rewritten at the next headless start, or by crw \
                 install",
                differences.join(", ")
            ),
        }
    }
}

/// Every file under `dir`, by its path under it (with `/`) and its full path, in order.
fn files_under(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(folder) = pending.pop() {
        let Ok(entries) = fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if let Ok(rest) = path.strip_prefix(dir) {
                let name = rest
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                found.push((name, path));
            }
        }
    }
    found.sort();
    found
}

/// Removes the folders under `dir` that hold nothing, deepest first; `dir` itself stays.
fn remove_empty_folders(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            remove_empty_folders(&path);
            if fs::read_dir(&path).is_ok_and(|mut left| left.next().is_none()) {
                let _ = fs::remove_dir(&path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    #[test]
    fn the_folder_is_under_the_local_one() {
        assert_eq!(
            dir().map(|d| d.file_name().unwrap().to_string_lossy().into_owned()),
            paths::local_dir().map(|_| NAME.to_owned())
        );
        if let Some(dir) = dir() {
            assert_eq!(
                dir.parent().unwrap(),
                paths::local_dir().unwrap().join(PLUGIN_DIR)
            );
        }
    }

    #[test]
    fn writes_the_copy_once_and_puts_right_what_changed() {
        let scratch = Scratch::new("plugin-copy");
        let dir = scratch.0.join("plugin").join(NAME);
        assert_eq!(Status::of(&dir), Status::None);
        assert!(Status::None.words().starts_with("none yet"));

        // A fresh copy: every file written.
        let report = write(&dir).unwrap();
        assert_eq!(report.written.len(), FILES.len());
        assert!(report.removed.is_empty());
        assert_eq!(report.unchanged, 0);
        assert_eq!(
            report.words(),
            format!("{} files written, 0 unchanged", FILES.len())
        );
        for (name, text) in FILES {
            assert_eq!(
                fs::read_to_string(dir.join(name.replace('/', "\\"))).unwrap(),
                text
            );
        }
        assert_eq!(Status::of(&dir), Status::Matches);
        assert_eq!(Status::Matches.words(), "matches the build");

        // Written again: nothing to do.
        let report = write(&dir).unwrap();
        assert_eq!(
            report,
            Report {
                unchanged: FILES.len(),
                ..Report::default()
            }
        );
        assert!(report.words().starts_with("matches the build already"));

        // A changed file, a missing one, a stray file in a folder of its own and one beside the
        // manifest: said by the status, put right by the write, the stray's folder gone.
        let plan = dir.join("skills").join("plan").join("SKILL.md");
        fs::write(&plan, "edited by hand").unwrap();
        let manifest = dir.join(".claude-plugin").join("plugin.json");
        fs::remove_file(&manifest).unwrap();
        let stray_dir = dir.join("skills").join("extra");
        fs::create_dir_all(&stray_dir).unwrap();
        fs::write(stray_dir.join("SKILL.md"), "an old skill").unwrap();
        fs::write(dir.join(".claude-plugin").join("notes.txt"), "stray").unwrap();
        assert_eq!(
            Status::of(&dir),
            Status::Differs(vec![
                ".claude-plugin/plugin.json missing".to_owned(),
                "skills/plan/SKILL.md differs".to_owned(),
                ".claude-plugin/notes.txt not in the build".to_owned(),
                "skills/extra/SKILL.md not in the build".to_owned(),
            ])
        );
        assert!(
            Status::of(&dir)
                .words()
                .starts_with("differs from the build (")
        );
        let report = write(&dir).unwrap();
        assert_eq!(
            report.written,
            [".claude-plugin/plugin.json", "skills/plan/SKILL.md"]
        );
        assert_eq!(
            report.removed,
            [".claude-plugin/notes.txt", "skills/extra/SKILL.md"]
        );
        assert_eq!(report.unchanged, FILES.len() - 2);
        assert_eq!(report.words(), "2 files written, 2 removed, 4 unchanged");
        assert!(!stray_dir.exists());
        assert!(manifest.is_file());
        assert_eq!(fs::read_to_string(&plan).unwrap(), FILES[3].1);
        assert_eq!(Status::of(&dir), Status::Matches);
        // No temp file of the atomic write is left behind.
        assert!(
            files_under(&dir)
                .iter()
                .all(|(name, _)| !name.ends_with(".tmp"))
        );
    }

    #[test]
    fn a_file_in_the_way_of_the_folder_is_said() {
        let scratch = Scratch::new("plugin-blocked");
        let dir = scratch.0.join("plugin");
        fs::write(&dir, "a file where the folder goes").unwrap();
        let why = write(&dir.join(NAME)).unwrap_err();
        assert!(why.contains(NAME), "{why}");
    }
}
