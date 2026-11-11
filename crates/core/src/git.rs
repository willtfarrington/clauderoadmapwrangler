//! Git, asked one question at a time (D-29). A working tree's porcelain v2 status gives its
//! branch, the upstream, ahead and behind, and every changed path in one child; with
//! `--no-optional-locks` the status leaves the index alone, so reading a project writes nothing
//! into it (D-11). The remote's URL is read from `.git/config` with no child. Children that reach a
//! remote run with prompts switched off, so a missing credential fails instead of waiting.

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::process;

/// What keeps a networked child from asking for credentials: git's own prompt and Git Credential
/// Manager's dialog.
const NO_PROMPTS: [(&str, &str); 2] = [("GIT_TERMINAL_PROMPT", "0"), ("GCM_INTERACTIVE", "never")];

/// Runs `git -C <folder> <args>` and returns what it prints.
pub fn run(folder: &Path, args: &[&str]) -> Result<String, String> {
    run_with(folder, args, &[])
}

/// [`run`] for a child that reaches a remote: it fails rather than ask for a credential.
pub fn run_networked(folder: &Path, args: &[&str]) -> Result<String, String> {
    run_with(folder, args, &NO_PROMPTS)
}

fn run_with(folder: &Path, args: &[&str], env: &[(&str, &str)]) -> Result<String, String> {
    let mut all: Vec<&OsStr> = vec![OsStr::new("-C"), folder.as_os_str()];
    all.extend(args.iter().map(OsStr::new));
    process::output_with(Path::new("git"), all, env, process::TIMEOUT)
}

/// `git status --porcelain=v2 --branch`, read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Status {
    /// The commit `HEAD` names; `None` before the first commit.
    pub oid: Option<String>,
    /// The branch checked out; `None` when `HEAD` is detached.
    pub branch: Option<String>,
    /// The branch's upstream, such as `origin/main`.
    pub upstream: Option<String>,
    /// Commits ahead of and behind the upstream as git last saw it, at the last fetch or push;
    /// `None` without an upstream, or when the upstream is gone.
    pub ahead_behind: Option<(u32, u32)>,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// The path from the repository's root, with `/` between folders.
    pub path: String,
    pub kind: ChangeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    /// Modified, added, deleted or of another type, staged or not.
    Changed,
    /// Renamed or copied from `from`.
    Renamed {
        from: String,
    },
    /// A merge conflict.
    Unmerged,
    Untracked,
}

impl Status {
    /// The upstream as remote and branch: `origin/main` → `origin`, `main`. A branch name may
    /// hold slashes; the remote is the part before the first one.
    pub fn upstream_parts(&self) -> Option<(&str, &str)> {
        self.upstream.as_deref()?.split_once('/')
    }
}

/// The status of the whole working tree at `folder`: one child.
pub fn status(folder: &Path) -> Result<Status, String> {
    let args = [
        "--no-optional-locks",
        "status",
        "--porcelain=v2",
        "--branch",
        "-z",
    ];
    run(folder, &args).map(|text| parse_status(&text))
}

/// [`status`] with every untracked file listed on its own, never its folder alone: what a
/// session's start copies and a revert removes (EP-15b). Ignored files are never listed.
pub fn status_all(folder: &Path) -> Result<Status, String> {
    let args = [
        "--no-optional-locks",
        "status",
        "--porcelain=v2",
        "--branch",
        "-z",
        "--untracked-files=all",
    ];
    run(folder, &args).map(|text| parse_status(&text))
}

/// What git says when `HEAD` holds nothing at a path: the path is not in its tree, on disk or
/// not, or there is no commit yet. Read under `LC_ALL=C`, so a translated git says them too.
const NOT_AT_HEAD: [&str; 3] = [
    "does not exist in 'HEAD'",
    "exists on disk, but not in 'HEAD'",
    "invalid object name 'HEAD'",
];

/// The text `HEAD` holds at `path` (from the repository's root, `/` between folders), one child,
/// `git cat-file blob HEAD:<path>` (unlike `git show`, it never reads the argument as a pathspec
/// when `HEAD` holds nothing there): `None` when `HEAD` holds nothing there ([`NOT_AT_HEAD`]); an
/// error when git failed for another reason, which a caller takes as "ask again later" (the
/// mailman's keys, EP-20d).
pub fn show_at_head(folder: &Path, path: &str) -> Result<Option<String>, String> {
    let spec = format!("HEAD:{path}");
    let args: [&OsStr; 5] = [
        OsStr::new("-C"),
        folder.as_os_str(),
        OsStr::new("cat-file"),
        OsStr::new("blob"),
        OsStr::new(&spec),
    ];
    let env = [("LC_ALL", "C")];
    match process::output_with_stderr(Path::new("git"), args, &env, process::TIMEOUT) {
        Ok(text) => Ok(Some(text)),
        Err(why) if NOT_AT_HEAD.iter().any(|words| why.contains(words)) => Ok(None),
        Err(why) => Err(why),
    }
}

/// `--porcelain=v2 -z`: `# branch.*` headers, then one record per path, each ending in NUL; a
/// renamed or copied path's record is followed by its old path as a record of its own.
pub fn parse_status(text: &str) -> Status {
    let mut status = Status::default();
    let mut records = text.split('\0').filter(|record| !record.is_empty());
    while let Some(record) = records.next() {
        if let Some(header) = record.strip_prefix("# ") {
            let (key, value) = header.split_once(' ').unwrap_or((header, ""));
            match key {
                "branch.oid" if value != "(initial)" => status.oid = Some(value.to_owned()),
                "branch.head" if value != "(detached)" => status.branch = Some(value.to_owned()),
                "branch.upstream" => status.upstream = Some(value.to_owned()),
                "branch.ab" => status.ahead_behind = ahead_behind(value),
                _ => {}
            }
            continue;
        }
        // The fields before the path: `1 XY sub mH mI mW hH hI`, `2 … Xscore`, `u … h3`, `?`.
        let (kind, fields) = match record.as_bytes().first() {
            Some(b'1') => (ChangeKind::Changed, 8),
            Some(b'2') => (
                ChangeKind::Renamed {
                    from: String::new(),
                },
                9,
            ),
            Some(b'u') => (ChangeKind::Unmerged, 10),
            Some(b'?') => (ChangeKind::Untracked, 1),
            // `!` (ignored, never asked for here) and anything a later git adds.
            _ => continue,
        };
        let Some(path) = record.splitn(fields + 1, ' ').nth(fields) else {
            continue;
        };
        let kind = match kind {
            ChangeKind::Renamed { .. } => ChangeKind::Renamed {
                from: records.next().unwrap_or_default().to_owned(),
            },
            other => other,
        };
        status.changes.push(Change {
            path: path.to_owned(),
            kind,
        });
    }
    status
}

/// `+3 -1` → 3 ahead, 1 behind.
fn ahead_behind(value: &str) -> Option<(u32, u32)> {
    let (ahead, behind) = value.split_once(' ')?;
    Some((
        ahead.strip_prefix('+')?.parse().ok()?,
        behind.strip_prefix('-')?.parse().ok()?,
    ))
}

/// The URL of the remote called `name`, from the repository's `.git/config` with no child;
/// `None` when there is no such remote, or when `.git` is a file (a worktree or a submodule,
/// which a project folder is not).
pub fn remote_url(folder: &Path, name: &str) -> Option<String> {
    let text = fs::read_to_string(folder.join(".git").join("config")).ok()?;
    config_value(&text, "remote", name, "url")
}

/// The first value of `key` in the section `[<section> "<subsection>"]` of a git config file:
/// section and key names case aside, the subsection exactly, as git compares them.
fn config_value(text: &str, section: &str, subsection: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        if let Some(header) = line.strip_prefix('[') {
            let header = header.split(']').next().unwrap_or_default().trim();
            inside = header.split_once(' ').is_some_and(|(name, sub)| {
                name.eq_ignore_ascii_case(section) && sub.trim().trim_matches('"') == subsection
            });
            continue;
        }
        let (name, value) = line.split_once('=').unwrap_or((line, ""));
        if inside && name.trim().eq_ignore_ascii_case(key) {
            let value = value.trim();
            let value = match value.strip_prefix('"').and_then(|v| v.split_once('"')) {
                Some((quoted, _)) => quoted,
                // An unquoted value ends at a comment.
                None => value
                    .split([';', '#'])
                    .next()
                    .unwrap_or_default()
                    .trim_end(),
            };
            return Some(value.to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn head_shows_a_committed_file_and_nothing_where_it_holds_none() {
        let repo = crate::revert::testing::Repo::new("git-show-head");
        let show = |path: &str| {
            let _turn = process::turn();
            show_at_head(&repo.0, path)
        };
        // No commit yet: `invalid object name 'HEAD'`.
        repo.write("roadmap/EP-1-one.md", "# EP-1\n");
        assert_eq!(show("roadmap/EP-1-one.md"), Ok(None));
        repo.commit("one");
        assert_eq!(show("roadmap/EP-1-one.md"), Ok(Some("# EP-1\n".to_owned())));
        // On disk but not in HEAD, and nowhere at all.
        repo.write("roadmap/EP-2-two.md", "# EP-2\n");
        assert_eq!(show("roadmap/EP-2-two.md"), Ok(None));
        assert_eq!(show("roadmap/EP-3-three.md"), Ok(None));
        // Any other failure is an error, with git's words: a caller asks again later.
        let missing = repo.scratch().join("no such folder");
        let _turn = process::turn();
        let failed = show_at_head(&missing, "roadmap/EP-1-one.md").unwrap_err();
        assert!(failed.contains("exited with"), "{failed}");
        assert!(failed.contains("fatal"), "{failed}");
    }

    #[test]
    fn reads_the_porcelain_v2_status() {
        let text = "# branch.oid 1111111111111111111111111111111111111111\0\
                    # branch.head main\0\
                    # branch.upstream origin/main\0\
                    # branch.ab +2 -1\0\
                    # stash 3\0\
                    1 .M N... 100644 100644 100644 aaaa aaaa roadmap/README.md\0\
                    1 A. N... 000000 100644 100644 0000 bbbb src/new file.rs\0\
                    2 R. N... 100644 100644 100644 cccc cccc R100 roadmap/EP-5-new.md\0roadmap/EP-5-old.md\0\
                    u UU N... 100644 100644 100644 100644 dddd eeee ffff notes.md\0\
                    ? roadmap/EP-6-draft.md\0\
                    ! target/\0";
        let status = parse_status(text);
        assert_eq!(
            status.oid.as_deref(),
            Some("1111111111111111111111111111111111111111")
        );
        assert_eq!(status.branch.as_deref(), Some("main"));
        assert_eq!(status.upstream_parts(), Some(("origin", "main")));
        assert_eq!(status.ahead_behind, Some((2, 1)));
        let paths: Vec<(&str, &ChangeKind)> = status
            .changes
            .iter()
            .map(|c| (c.path.as_str(), &c.kind))
            .collect();
        assert_eq!(
            paths,
            [
                ("roadmap/README.md", &ChangeKind::Changed),
                ("src/new file.rs", &ChangeKind::Changed),
                (
                    "roadmap/EP-5-new.md",
                    &ChangeKind::Renamed {
                        from: "roadmap/EP-5-old.md".to_owned()
                    }
                ),
                ("notes.md", &ChangeKind::Unmerged),
                ("roadmap/EP-6-draft.md", &ChangeKind::Untracked),
            ]
        );
    }

    #[test]
    fn a_new_repository_and_a_detached_head_read_as_none() {
        let status = parse_status("# branch.oid (initial)\0# branch.head (detached)\0");
        assert_eq!(status, Status::default());
        assert_eq!(status.upstream_parts(), None);
        // An upstream that is gone has no counts.
        let gone = parse_status("# branch.head main\0# branch.upstream origin/main\0");
        assert_eq!(gone.ahead_behind, None);
        assert_eq!(ahead_behind("+0 -0"), Some((0, 0)));
        assert_eq!(ahead_behind("3 1"), None);
    }

    #[test]
    fn reads_a_remote_from_the_config_text() {
        let text = "[core]\n\tbare = false\n\
                    [Remote \"upstream\"]\n\turl = https://example.invalid/other.git\n\
                    # a comment\n\
                    [remote \"origin\"]\n\
                    \tfetch = +refs/heads/*:refs/remotes/origin/*\n\
                    \tURL = https://github.com/example/alpha.git ; the first push\n\
                    [remote \"quoted\"]\n\turl = \"git@github.com:example/beta.git\"\n";
        assert_eq!(
            config_value(text, "remote", "origin", "url").as_deref(),
            Some("https://github.com/example/alpha.git")
        );
        assert_eq!(
            config_value(text, "REMOTE", "upstream", "url").as_deref(),
            Some("https://example.invalid/other.git")
        );
        assert_eq!(
            config_value(text, "remote", "quoted", "url").as_deref(),
            Some("git@github.com:example/beta.git")
        );
        // The subsection is compared exactly.
        assert_eq!(config_value(text, "remote", "Origin", "url"), None);
        assert_eq!(config_value(text, "remote", "origin", "pushurl"), None);
    }

    /// Runs git on a scratch repository and on this one, never the `claude` CLI (D-29).
    #[test]
    fn reads_this_repositorys_status_and_remote() {
        let repo = crate::paths::repository();
        let status = status(&repo).expect("git answers in this repository");
        assert!(status.oid.is_some());
        let scratch = crate::files::tests::Scratch::new("git-not-a-repo");
        assert_eq!(remote_url(&scratch.0, "origin"), None);
        std::thread::sleep(process::SPACING);
        assert!(super::status(&scratch.0).is_err());
    }
}
