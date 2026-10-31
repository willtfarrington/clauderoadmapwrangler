//! Where the tool keeps its files (D-3, D-8): the registry in the roaming profile, the ledger and
//! the backups in the local one, never in a repository. `CRW_HOME=<folder>` puts them all in one
//! scratch folder, so a session can try the commands without touching the owner's files.

use std::env;
use std::path::{Component, Path, PathBuf};

/// The folder under `%APPDATA%` and `%LOCALAPPDATA%` (D-2).
pub const APP_DIR: &str = "clauderoadmapwrangler";

/// When set, the registry, the ledger and the backups all live in this folder instead.
pub const HOME_VAR: &str = "CRW_HOME";

/// `%APPDATA%\clauderoadmapwrangler`: the registry and its migration backups.
pub fn config_dir() -> Option<PathBuf> {
    home().or_else(|| under("APPDATA"))
}

/// `%LOCALAPPDATA%\clauderoadmapwrangler`: the ledger and the copies made before changes.
pub fn local_dir() -> Option<PathBuf> {
    home().or_else(|| under("LOCALAPPDATA"))
}

pub fn registry_file() -> Option<PathBuf> {
    Some(config_dir()?.join("registry.toml"))
}

pub fn ledger_file() -> Option<PathBuf> {
    Some(local_dir()?.join("ledger.sqlite"))
}

pub fn backups_dir() -> Option<PathBuf> {
    Some(local_dir()?.join("backups"))
}

fn home() -> Option<PathBuf> {
    env::var_os(HOME_VAR)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn under(var: &str) -> Option<PathBuf> {
    env::var_os(var)
        .filter(|value| !value.is_empty())
        .map(|base| PathBuf::from(base).join(APP_DIR))
}

/// The repository this build comes from: two folders above this crate's manifest.
pub fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

/// The default data root: the folder that holds this repository (D-6).
pub fn default_data_root() -> PathBuf {
    repository()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

/// `path` with `.` and `..` resolved and trailing separators gone, without asking the file system.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other),
        }
    }
    out
}

/// Whether two paths name the same place, compared as Windows does: case and separators aside.
pub fn same(a: &Path, b: &Path) -> bool {
    let key = |p: &Path| {
        normalize(p)
            .to_string_lossy()
            .replace('/', "\\")
            .to_lowercase()
    };
    key(a) == key(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_repository_holds_this_crate() {
        let repo = repository();
        assert!(
            repo.join("crates")
                .join("core")
                .join("Cargo.toml")
                .is_file()
        );
        assert_eq!(default_data_root(), repo.parent().unwrap());
    }

    #[test]
    fn compares_paths_as_windows_does() {
        let root = Path::new(r"C:\Data");
        assert!(same(&root.join("alpha"), Path::new(r"c:\data\ALPHA")));
        assert!(same(&root.join("./alpha/"), Path::new(r"C:\Data\alpha")));
        assert!(same(
            &root.join("beta/../alpha"),
            Path::new(r"C:\Data\alpha")
        ));
        assert!(same(
            &root.join("C:/Data/alpha"),
            Path::new(r"C:\Data\alpha")
        ));
        assert!(same(&root.join("../Vault"), Path::new(r"C:\Vault")));
        assert!(!same(&root.join("alpha"), Path::new(r"C:\Data\alphabet")));
    }
}
