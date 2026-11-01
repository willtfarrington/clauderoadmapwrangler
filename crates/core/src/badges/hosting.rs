//! Hosting (D-24): where a project's repository lives and whether that copy is current. The
//! origin's URL is read as a GitHub repository and matched in the account's list, which one
//! `gh repo list` gives for every repository at once; a local-only project whose name the account
//! also holds is an orphan. The head check compares the origin's head (`git ls-remote`) with the
//! local one and counts both sides with `git rev-list` when they differ. The tool never fetches:
//! a fetch writes into the project's `.git` (D-11).

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::rfc3339;
use crate::{git, process};

/// The fields `gh repo list` is asked for.
const LIST_FIELDS: &str = "name,visibility,isArchived,pushedAt,defaultBranchRef,owner";

/// More repositories than the account holds.
const LIST_LIMIT: &str = "200";

/// Keeps `gh` from asking anything.
pub(crate) const GH_ENV: [(&str, &str); 1] = [("GH_PROMPT_DISABLED", "1")];

/// A GitHub repository, `owner/name`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repo {
    pub owner: String,
    pub name: String,
}

impl Repo {
    pub fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn is(&self, other: &Repo) -> bool {
        self.owner.eq_ignore_ascii_case(&other.owner) && self.name.eq_ignore_ascii_case(&other.name)
    }
}

/// The GitHub repository a remote URL names: `https://github.com/<owner>/<name>.git`,
/// `git@github.com:<owner>/<name>.git` or `ssh://git@github.com/<owner>/<name>`.
pub fn github(url: &str) -> Option<Repo> {
    let url = url.trim();
    let scheme = ["https://", "http://", "ssh://", "git://"]
        .iter()
        .find_map(|scheme| url.strip_prefix(scheme));
    let (host, path) = match scheme {
        Some(rest) => rest.split_once('/')?,
        // The scp form, `user@host:path`.
        None => url.split_once(':')?,
    };
    let host = host.rsplit('@').next()?.split(':').next()?;
    if !host.eq_ignore_ascii_case("github.com") && !host.eq_ignore_ascii_case("www.github.com") {
        return None;
    }
    let mut parts = path.trim_matches('/').split('/');
    let owner = parts.next()?;
    let name = parts.next()?;
    let name = name.strip_suffix(".git").unwrap_or(name);
    (!owner.is_empty() && !name.is_empty() && parts.next().is_none()).then(|| Repo {
        owner: owner.to_owned(),
        name: name.to_owned(),
    })
}

/// The host a remote URL names, or the URL itself when it names none (a folder).
fn host_of(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host = rest.split(['/', '\\']).next().unwrap_or(rest);
    let host = host.rsplit('@').next().unwrap_or(host);
    match host.split_once(':') {
        // `C:` is a drive, not a host.
        Some((drive, _)) if drive.len() == 1 => url.to_owned(),
        Some((host, _)) => host.to_owned(),
        None => host.to_owned(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

/// One repository of the account, from `gh repo list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listed {
    pub repo: Repo,
    pub visibility: Visibility,
    pub archived: bool,
    #[serde(with = "rfc3339")]
    pub pushed_at: Option<DateTime<Utc>>,
    pub default_branch: Option<String>,
}

/// `gh repo list --json …`, read; entries without a name, an owner or a known visibility are
/// left out.
pub fn parse_account(json: &str) -> Result<Vec<Listed>, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("gh printed no list: {e}"))?;
    let items = value.as_array().ok_or("gh printed no list")?;
    Ok(items
        .iter()
        .filter_map(|item| {
            let visibility = match item["visibility"].as_str()?.to_ascii_uppercase().as_str() {
                "PUBLIC" => Visibility::Public,
                "PRIVATE" => Visibility::Private,
                "INTERNAL" => Visibility::Internal,
                _ => return None,
            };
            Some(Listed {
                repo: Repo {
                    owner: item["owner"]["login"].as_str()?.to_owned(),
                    name: item["name"].as_str()?.to_owned(),
                },
                visibility,
                archived: item["isArchived"].as_bool().unwrap_or(false),
                pushed_at: item["pushedAt"].as_str().and_then(rfc3339::parse),
                default_branch: item["defaultBranchRef"]["name"]
                    .as_str()
                    .filter(|name| !name.is_empty())
                    .map(str::to_owned),
            })
        })
        .collect())
}

/// The account's repositories: one networked child.
pub fn account() -> Result<Vec<Listed>, String> {
    let args = ["repo", "list", "--limit", LIST_LIMIT, "--json", LIST_FIELDS];
    let text = process::output_with(&gh(), args, &GH_ENV, process::TIMEOUT)
        .map_err(|e| format!("gh repo list: {e}"))?;
    parse_account(&text)
}

/// `gh` from PATH, else where its installer puts it.
pub(crate) fn gh() -> PathBuf {
    let on_path = std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join("gh.exe"))
            .find(|candidate| candidate.is_file())
    });
    let installed = std::env::var_os("ProgramFiles")
        .map(|dir| PathBuf::from(dir).join("GitHub CLI").join("gh.exe"))
        .filter(|candidate| candidate.is_file());
    on_path.or(installed).unwrap_or_else(|| PathBuf::from("gh"))
}

/// The origin's head against the local one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    InSync,
    Ahead(u32),
    Behind(u32),
    Diverged {
        ahead: u32,
        behind: u32,
    },
    /// The origin holds commits this clone has never fetched: it is behind, or diverged.
    NotFetched,
    /// The origin has no such branch.
    NoBranch,
    /// The origin could not be asked.
    Failed(String),
}

/// One head check (D-24).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Check {
    /// The ref asked for: the upstream's branch, or `HEAD` without an upstream.
    pub remote_ref: String,
    pub remote: Option<String>,
    /// The local `HEAD` it was set against.
    pub local: Option<String>,
    pub verdict: Verdict,
}

impl Check {
    /// `in sync`, `2 ahead`, `3 behind`, `diverged (2 ahead, 3 behind)`, …
    pub fn text(&self) -> String {
        match &self.verdict {
            Verdict::InSync => "in sync".to_owned(),
            Verdict::Ahead(n) => format!("{n} ahead"),
            Verdict::Behind(n) => format!("{n} behind"),
            Verdict::Diverged { ahead, behind } => {
                format!("diverged ({ahead} ahead, {behind} behind)")
            }
            Verdict::NotFetched => "the origin has commits not fetched".to_owned(),
            Verdict::NoBranch => format!("the origin has no {}", self.remote_ref),
            Verdict::Failed(reason) => format!("the origin could not be asked: {reason}"),
        }
    }
}

/// Checks the origin's head: the status for the branch and its upstream, `git ls-remote`
/// (networked), and `git rev-list` when the two heads differ; each child in its turn.
pub fn check_head(folder: &Path) -> Check {
    let status = {
        let _turn = process::turn();
        git::status(folder)
    };
    let status = match status {
        Ok(status) => status,
        Err(reason) => {
            return Check {
                remote_ref: "HEAD".to_owned(),
                remote: None,
                local: None,
                verdict: Verdict::Failed(format!("git status: {reason}")),
            };
        }
    };
    let remote_ref = match status.upstream_parts() {
        Some(("origin", branch)) => format!("refs/heads/{branch}"),
        _ => "HEAD".to_owned(),
    };
    let listed = {
        let _turn = process::turn();
        git::run_networked(folder, &["ls-remote", "origin", &remote_ref])
    };
    let remote = match listed {
        Ok(text) => text.lines().find_map(|line| {
            let (sha, name) = line.split_once('\t')?;
            (name.trim() == remote_ref).then(|| sha.to_owned())
        }),
        Err(reason) => {
            return Check {
                remote_ref,
                remote: None,
                local: status.oid,
                verdict: Verdict::Failed(format!("git ls-remote: {reason}")),
            };
        }
    };
    let verdict = compare(status.oid.as_deref(), remote.as_deref(), |remote| {
        let _turn = process::turn();
        counts(folder, remote)
    });
    Check {
        remote_ref,
        remote,
        local: status.oid,
        verdict,
    }
}

/// `git rev-list --left-right --count <remote>...HEAD`: behind, then ahead. Fails when the
/// remote's commit is not in the clone.
fn counts(folder: &Path, remote: &str) -> Result<(u32, u32), String> {
    let range = format!("{remote}...HEAD");
    let text = git::run(folder, &["rev-list", "--left-right", "--count", &range])?;
    let (behind, ahead) = text
        .trim()
        .split_once('\t')
        .ok_or("rev-list printed no counts")?;
    let number = |n: &str| n.trim().parse::<u32>().map_err(|e| e.to_string());
    Ok((number(behind)?, number(ahead)?))
}

/// The verdict from the two heads and, when they differ, the counts of each side.
pub fn compare(
    local: Option<&str>,
    remote: Option<&str>,
    counts: impl FnOnce(&str) -> Result<(u32, u32), String>,
) -> Verdict {
    match (local, remote) {
        (_, None) => Verdict::NoBranch,
        (None, Some(_)) => Verdict::NotFetched,
        (Some(local), Some(remote)) if local == remote => Verdict::InSync,
        (Some(_), Some(remote)) => match counts(remote) {
            Ok((0, 0)) => Verdict::InSync,
            Ok((0, ahead)) => Verdict::Ahead(ahead),
            Ok((behind, 0)) => Verdict::Behind(behind),
            Ok((behind, ahead)) => Verdict::Diverged { ahead, behind },
            Err(_) => Verdict::NotFetched,
        },
    }
}

/// What the hosting badge says (D-24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hosting {
    /// No remote.
    Local,
    /// No remote, and the account holds a repository of the project's name.
    Orphan(Listed),
    /// A GitHub repository the account lists.
    Listed(Listed),
    /// A GitHub repository of the account's owner that the account does not list: deleted or
    /// renamed.
    Missing(Repo),
    /// A GitHub repository not matched: the account's list is not read yet, or the repository
    /// belongs to another owner.
    GitHub(Repo),
    /// A remote somewhere else, by its host.
    Elsewhere(String),
}

/// The hosting filter's classes and the Hosting sort key's order (D-25).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Class {
    Public,
    Private,
    /// A remote of another kind: not listed, missing, or not on GitHub.
    Remote,
    Local,
}

impl Class {
    pub const ALL: [Class; 4] = [Class::Public, Class::Private, Class::Remote, Class::Local];

    pub fn label(self) -> &'static str {
        match self {
            Class::Public => "Public",
            Class::Private => "Private",
            Class::Remote => "Other remote",
            Class::Local => "Local only",
        }
    }
}

/// The hosting of a project known by `names` (its name and its folder's) with `remote` as its
/// origin's URL, against the account's list when it has been read.
pub fn classify(names: &[&str], remote: Option<&str>, account: Option<&[Listed]>) -> Hosting {
    let Some(url) = remote else {
        let orphan = account.and_then(|list| {
            list.iter().find(|listed| {
                names
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&listed.repo.name))
            })
        });
        return orphan.map_or(Hosting::Local, |listed| Hosting::Orphan(listed.clone()));
    };
    let Some(repo) = github(url) else {
        return Hosting::Elsewhere(host_of(url));
    };
    let Some(list) = account else {
        return Hosting::GitHub(repo);
    };
    if let Some(listed) = list.iter().find(|listed| listed.repo.is(&repo)) {
        return Hosting::Listed(listed.clone());
    }
    match list.first() {
        Some(any) if any.repo.owner.eq_ignore_ascii_case(&repo.owner) => Hosting::Missing(repo),
        _ => Hosting::GitHub(repo),
    }
}

impl Hosting {
    /// The badge's word.
    pub fn label(&self) -> &'static str {
        match self {
            Hosting::Local => "local",
            Hosting::Orphan(_) => "orphan",
            Hosting::Listed(listed) if listed.archived => "archived",
            Hosting::Listed(listed) => match listed.visibility {
                Visibility::Public => "public",
                Visibility::Private => "private",
                Visibility::Internal => "internal",
            },
            Hosting::Missing(_) => "missing",
            Hosting::GitHub(_) => "GitHub",
            Hosting::Elsewhere(_) => "remote",
        }
    }

    pub fn class(&self) -> Class {
        match self {
            Hosting::Listed(listed) if listed.visibility == Visibility::Public => Class::Public,
            Hosting::Listed(_) => Class::Private,
            Hosting::Missing(_) | Hosting::GitHub(_) | Hosting::Elsewhere(_) => Class::Remote,
            Hosting::Local | Hosting::Orphan(_) => Class::Local,
        }
    }

    /// An orphan or a missing repository: the badges D-21 raises for the owner.
    pub fn needs_attention(&self) -> bool {
        matches!(self, Hosting::Orphan(_) | Hosting::Missing(_))
    }

    /// The GitHub repository, when there is one.
    pub fn repo(&self) -> Option<&Repo> {
        match self {
            Hosting::Listed(listed) | Hosting::Orphan(listed) => Some(&listed.repo),
            Hosting::Missing(repo) | Hosting::GitHub(repo) => Some(repo),
            Hosting::Local | Hosting::Elsewhere(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCOUNT: &str = r#"[
        {"defaultBranchRef":{"name":"main"},"isArchived":false,"name":"alpha","owner":{"login":"example"},"pushedAt":"2026-10-30T10:00:00Z","visibility":"PUBLIC"},
        {"defaultBranchRef":{"name":"trunk"},"isArchived":true,"name":"beta","owner":{"login":"example"},"pushedAt":"2026-08-01T10:00:00Z","visibility":"PRIVATE"},
        {"defaultBranchRef":null,"isArchived":false,"name":"gamma","owner":{"login":"example"},"pushedAt":null,"visibility":"PRIVATE"},
        {"name":"odd","owner":{"login":"example"},"visibility":"SECRET"}
    ]"#;

    #[test]
    fn reads_github_remote_urls() {
        let repo = |owner: &str, name: &str| {
            Some(Repo {
                owner: owner.to_owned(),
                name: name.to_owned(),
            })
        };
        assert_eq!(
            github("https://github.com/example/alpha.git"),
            repo("example", "alpha")
        );
        assert_eq!(
            github("https://user@github.com/example/alpha"),
            repo("example", "alpha")
        );
        assert_eq!(
            github("git@github.com:example/beta.git"),
            repo("example", "beta")
        );
        assert_eq!(
            github("ssh://git@github.com:22/example/gamma.git"),
            repo("example", "gamma")
        );
        assert_eq!(github("https://gitlab.com/example/alpha.git"), None);
        assert_eq!(github(r"C:\repos\alpha.git"), None);
        assert_eq!(github("https://github.com/example"), None);
        assert_eq!(github("https://github.com/example/alpha/tree/main"), None);
        assert_eq!(
            host_of("https://gitlab.com/example/alpha.git"),
            "gitlab.com"
        );
        assert_eq!(
            host_of("git@codeberg.org:example/alpha.git"),
            "codeberg.org"
        );
        assert_eq!(host_of(r"C:\repos\alpha.git"), r"C:\repos\alpha.git");
    }

    #[test]
    fn reads_the_accounts_list() {
        let list = parse_account(ACCOUNT).unwrap();
        assert_eq!(list.len(), 3, "the unknown visibility is left out");
        assert_eq!(list[0].repo.slug(), "example/alpha");
        assert_eq!(list[0].visibility, Visibility::Public);
        assert_eq!(list[0].default_branch.as_deref(), Some("main"));
        assert!(list[1].archived);
        assert_eq!(list[2].pushed_at, None);
        assert_eq!(list[2].default_branch, None);
        assert!(parse_account("not json").is_err());
        assert!(parse_account("{}").is_err());
        // The cache keeps it as JSON and reads it back.
        let json = serde_json::to_string(&list).unwrap();
        assert_eq!(serde_json::from_str::<Vec<Listed>>(&json).unwrap(), list);
    }

    #[test]
    fn classifies_the_hosting() {
        let list = parse_account(ACCOUNT).unwrap();
        let account = Some(list.as_slice());
        let labels = |names: &[&str], remote: Option<&str>| {
            let hosting = classify(names, remote, account);
            (hosting.label(), hosting.class())
        };
        assert_eq!(
            labels(&["alpha"], Some("https://github.com/example/alpha.git")),
            ("public", Class::Public)
        );
        assert_eq!(
            labels(&["beta"], Some("git@github.com:Example/Beta.git")),
            ("archived", Class::Private)
        );
        assert_eq!(
            labels(&["delta"], Some("https://github.com/example/delta.git")),
            ("missing", Class::Remote)
        );
        assert_eq!(
            labels(&["delta"], Some("https://github.com/someone/delta.git")),
            ("GitHub", Class::Remote)
        );
        assert_eq!(
            labels(&["delta"], Some("https://gitlab.com/example/delta.git")),
            ("remote", Class::Remote)
        );
        assert_eq!(labels(&["delta"], None), ("local", Class::Local));
        // A local-only project with an account repository of its name, by either name.
        let orphan = classify(&["Gamma project", "gamma"], None, account);
        assert_eq!(orphan.label(), "orphan");
        assert!(orphan.needs_attention());
        assert_eq!(
            orphan.repo().map(Repo::slug).as_deref(),
            Some("example/gamma")
        );
        // Before the list is read nothing is an orphan, and nothing is missing.
        assert_eq!(classify(&["gamma"], None, None), Hosting::Local);
        assert_eq!(
            classify(
                &["delta"],
                Some("https://github.com/example/delta.git"),
                None
            )
            .label(),
            "GitHub"
        );
    }

    #[test]
    fn compares_the_heads() {
        let never = |_: &str| -> Result<(u32, u32), String> { panic!("no counts needed") };
        assert_eq!(compare(Some("a"), Some("a"), never), Verdict::InSync);
        assert_eq!(compare(Some("a"), None, never), Verdict::NoBranch);
        assert_eq!(compare(None, Some("b"), never), Verdict::NotFetched);
        assert_eq!(
            compare(Some("a"), Some("b"), |_| Ok((0, 2))),
            Verdict::Ahead(2)
        );
        assert_eq!(
            compare(Some("a"), Some("b"), |_| Ok((3, 0))),
            Verdict::Behind(3)
        );
        assert_eq!(
            compare(Some("a"), Some("b"), |_| Ok((3, 2))),
            Verdict::Diverged {
                ahead: 2,
                behind: 3
            }
        );
        assert_eq!(
            compare(Some("a"), Some("b"), |_| Err("bad object".to_owned())),
            Verdict::NotFetched
        );
        let check = Check {
            remote_ref: "refs/heads/main".to_owned(),
            remote: None,
            local: None,
            verdict: Verdict::NoBranch,
        };
        assert_eq!(check.text(), "the origin has no refs/heads/main");
        let json = serde_json::to_string(&check).unwrap();
        assert_eq!(serde_json::from_str::<Check>(&json).unwrap(), check);
    }
}
