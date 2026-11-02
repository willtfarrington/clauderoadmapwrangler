//! Writing the files the tool changes (D-11, D-29): atomically, through a temp file and a rename,
//! with dated copies made first. The copies follow the owner's other app: one before a format
//! migration, kept beside the file, and one before the day's first change, of which the newest few
//! are kept.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Today's date in the owner's time zone, as the copies are named.
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Replaces `path` with `bytes` through `<name>.tmp` beside it, so an interrupted write never
/// leaves half a file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut name = path.file_name().unwrap_or_default().to_owned();
    name.push(".tmp");
    let tmp = path.with_file_name(name);
    let written = File::create(&tmp).and_then(|mut file| {
        file.write_all(bytes)?;
        file.sync_all()
    });
    match written.and_then(|()| fs::rename(&tmp, path)) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            Err(e)
        }
    }
}

/// Writes `bytes` to a new file at `path`, failing with `AlreadyExists` when anything is there,
/// which is left untouched; a write that fails takes its half-written file away.
pub fn create_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    if written.is_err() {
        drop(file);
        let _ = fs::remove_file(path);
    }
    written
}

/// Writes `bytes` to `<dir>\<stem>.<date><ext>`. A copy already there is never overwritten: one
/// holding the same bytes is reused, any other moves this one on to `<stem>.<date>.2<ext>`, `.3`,
/// and so on.
pub fn dated_copy(
    bytes: &[u8],
    dir: &Path,
    stem: &str,
    date: &str,
    ext: &str,
) -> io::Result<PathBuf> {
    fs::create_dir_all(dir)?;
    for n in 1..100 {
        let name = match n {
            1 => format!("{stem}.{date}{ext}"),
            n => format!("{stem}.{date}.{n}{ext}"),
        };
        let candidate = dir.join(name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                file.write_all(bytes)?;
                file.sync_all()?;
                return Ok(candidate);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                if fs::read(&candidate).is_ok_and(|existing| existing == bytes) {
                    return Ok(candidate);
                }
            }
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other("every copy name for today is taken"))
}

/// Before the first change of `date`: copies `path` as it stands to `<dir>\<stem>.<date>.<ext>`,
/// then deletes all but the newest `keep` of those copies. Returns the new copy, or `None` when
/// today's exists already or there is no file yet.
pub fn daily_copy(path: &Path, dir: &Path, date: &str, keep: usize) -> io::Result<Option<PathBuf>> {
    let (stem, ext) = stem_and_extension(path);
    let target = dir.join(format!("{stem}.{date}.{ext}"));
    if target.exists() || !path.exists() {
        return Ok(None);
    }
    fs::create_dir_all(dir)?;
    // Through a temp file, so an interrupted copy never passes for today's.
    let tmp = dir.join(format!("{stem}.{date}.{ext}.tmp"));
    fs::copy(path, &tmp)?;
    fs::rename(&tmp, &target)?;
    let mut copies: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| is_daily_copy(p, &stem, &ext))
        .collect();
    // `<stem>.YYYY-MM-DD.<ext>` sorts by date.
    copies.sort();
    let excess = copies.len().saturating_sub(keep);
    for old in &copies[..excess] {
        fs::remove_file(old)?;
    }
    Ok(Some(target))
}

fn stem_and_extension(path: &Path) -> (String, String) {
    let lossy = |s: Option<&std::ffi::OsStr>| s.map(|s| s.to_string_lossy().into_owned());
    (
        lossy(path.file_stem()).unwrap_or_default(),
        lossy(path.extension()).unwrap_or_default(),
    )
}

/// Whether `path` is named `<stem>.YYYY-MM-DD.<ext>`; the rotation touches nothing else.
fn is_daily_copy(path: &Path, stem: &str, ext: &str) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    name.strip_prefix(stem)
        .and_then(|rest| rest.strip_prefix('.'))
        .and_then(|rest| rest.strip_suffix(ext))
        .and_then(|rest| rest.strip_suffix('.'))
        .is_some_and(|date| {
            date.len() == 10 && chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
        })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A fresh folder under the system temp folder, removed when dropped (never the project root
    /// or the owner's profile folders).
    pub(crate) struct Scratch(pub PathBuf);

    impl Scratch {
        pub(crate) fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("crw-test-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        pub(crate) fn names(&self, dir: &Path) -> Vec<String> {
            let mut names: Vec<String> = fs::read_dir(self.0.join(dir))
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn an_atomic_write_replaces_the_file_and_leaves_no_temp_file() {
        let scratch = Scratch::new("atomic");
        let path = scratch.0.join("registry.toml");
        write_atomic(&path, b"first").unwrap();
        write_atomic(&path, b"second").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second");
        assert_eq!(scratch.names(Path::new("")), ["registry.toml"]);
    }

    #[test]
    fn a_new_file_never_replaces_one() {
        let scratch = Scratch::new("create-new");
        let path = scratch.0.join("alpha.code-workspace");
        create_new(&path, b"first").unwrap();
        let again = create_new(&path, b"second").unwrap_err();
        assert_eq!(again.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&path).unwrap(), b"first");
    }

    #[test]
    fn a_dated_copy_is_never_overwritten() {
        let scratch = Scratch::new("dated");
        let dir = scratch.0.join("backups");
        let first = dated_copy(b"one", &dir, "data", "2026-10-31", ".code-workspace").unwrap();
        assert_eq!(first, dir.join("data.2026-10-31.code-workspace"));
        // The same bytes reuse their copy; other bytes the same day take the next name.
        assert_eq!(
            dated_copy(b"one", &dir, "data", "2026-10-31", ".code-workspace").unwrap(),
            first
        );
        let second = dated_copy(b"two", &dir, "data", "2026-10-31", ".code-workspace").unwrap();
        assert_eq!(second, dir.join("data.2026-10-31.2.code-workspace"));
        assert_eq!(fs::read(&first).unwrap(), b"one");
        assert_eq!(fs::read(&second).unwrap(), b"two");
    }

    #[test]
    fn a_daily_copy_keeps_the_state_before_the_days_first_change() {
        let scratch = Scratch::new("daily");
        let path = scratch.0.join("registry.toml");
        let dir = scratch.0.join("backups");
        // Nothing written yet: nothing to copy, and no folder made.
        assert_eq!(daily_copy(&path, &dir, "2026-10-30", 14).unwrap(), None);
        assert!(!dir.exists());

        fs::write(&path, "state on the evening of the 29th").unwrap();
        let copy = daily_copy(&path, &dir, "2026-10-30", 14).unwrap().unwrap();
        assert_eq!(copy, dir.join("registry.2026-10-30.toml"));
        fs::write(&path, "after the first change on the 30th").unwrap();
        assert_eq!(daily_copy(&path, &dir, "2026-10-30", 14).unwrap(), None);
        assert_eq!(
            fs::read_to_string(&copy).unwrap(),
            "state on the evening of the 29th"
        );
    }

    #[test]
    fn only_the_newest_daily_copies_are_kept() {
        let scratch = Scratch::new("rotate");
        let path = scratch.0.join("registry.toml");
        let dir = scratch.0.join("backups");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, "format = 1").unwrap();
        for day in 1..=20 {
            fs::write(dir.join(format!("registry.2026-09-{day:02}.toml")), "").unwrap();
        }
        // Files the rotation must not touch.
        let others = [
            "data.2026-09-01.code-workspace",
            "registry.v1.2026-09-01.toml",
            "notes.txt",
        ];
        for name in others {
            fs::write(dir.join(name), "keep").unwrap();
        }
        daily_copy(&path, &dir, "2026-10-30", 14).unwrap().unwrap();
        let kept: Vec<String> = scratch
            .names(Path::new("backups"))
            .into_iter()
            .filter(|name| !others.contains(&name.as_str()))
            .collect();
        assert_eq!(kept.len(), 14);
        assert_eq!(kept.first().unwrap(), "registry.2026-09-08.toml");
        assert_eq!(kept.last().unwrap(), "registry.2026-10-30.toml");
        for name in others {
            assert!(dir.join(name).exists(), "{name}");
        }
    }
}
