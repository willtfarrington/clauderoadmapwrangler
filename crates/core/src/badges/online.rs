//! The networked readings (D-24): the account's repositories from one `gh repo list`, then for
//! each project with an origin its head check, and for a GitHub project with workflows its
//! latest Actions run; one child at a time with the spacing (D-29). The dashboard keeps each
//! reading in the ledger's cache with its fetch time and asks again after a day or on F5; a
//! reading that failed is shown but not cached, so the last good one survives a day offline.
//! `crw status` asks every time.

use std::path::PathBuf;

use chrono::TimeDelta;

use super::health::{self, Run};
use super::hosting::{self, Check, Listed};
use crate::process;

/// The cache's source names; the account's row has the empty project name.
pub const ACCOUNT: &str = "account";
pub const HEAD: &str = "head";
pub const RUN: &str = "run";

/// A cached reading this young is current: the badges are refreshed daily (D-24).
pub const FRESH_FOR: TimeDelta = TimeDelta::hours(24);

/// A project as the pass needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub folder: PathBuf,
    /// The origin's URL; a project without one is not asked about.
    pub remote: Option<String>,
    pub workflows: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    Account(Result<Vec<Listed>, String>),
    Head {
        project: String,
        check: Check,
    },
    Run {
        project: String,
        run: Result<Option<Run>, String>,
    },
}

impl Reading {
    /// The cache row for a reading worth keeping: project, source and JSON; `None` for a failure.
    pub fn to_cache(&self) -> Option<(String, &'static str, String)> {
        let (project, source, json) = match self {
            Reading::Account(Ok(list)) => (String::new(), ACCOUNT, serde_json::to_string(list)),
            Reading::Head { project, check }
                if !matches!(check.verdict, hosting::Verdict::Failed(_)) =>
            {
                (project.clone(), HEAD, serde_json::to_string(check))
            }
            Reading::Run {
                project,
                run: Ok(run),
            } => (project.clone(), RUN, serde_json::to_string(run)),
            _ => return None,
        };
        Some((project, source, json.ok()?))
    }

    /// A reading back from its cache row; `None` for an unknown source or JSON this build cannot
    /// read.
    pub fn from_cache(project: &str, source: &str, json: &str) -> Option<Reading> {
        Some(match source {
            ACCOUNT => Reading::Account(Ok(serde_json::from_str(json).ok()?)),
            HEAD => Reading::Head {
                project: project.to_owned(),
                check: serde_json::from_str(json).ok()?,
            },
            RUN => Reading::Run {
                project: project.to_owned(),
                run: Ok(serde_json::from_str(json).ok()?),
            },
            _ => return None,
        })
    }
}

/// Asks GitHub and the origins about `projects`, handing each reading to `each` as it arrives;
/// `each` returns `false` to stop the pass (a newer one superseded it).
pub fn fetch(projects: &[Project], mut each: impl FnMut(Reading) -> bool) {
    let account = {
        let _turn = process::turn();
        hosting::account()
    };
    if !each(Reading::Account(account)) {
        return;
    }
    for project in projects {
        let Some(url) = &project.remote else {
            continue;
        };
        let check = hosting::check_head(&project.folder);
        if !each(Reading::Head {
            project: project.name.clone(),
            check,
        }) {
            return;
        }
        if project.workflows
            && let Some(repo) = hosting::github(url)
        {
            let run = {
                let _turn = process::turn();
                health::latest_run(&repo)
            };
            if !each(Reading::Run {
                project: project.name.clone(),
                run,
            }) {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::badges::hosting::{Repo, Verdict, Visibility};

    #[test]
    fn readings_go_through_the_cache_and_failures_stay_out() {
        let listed = Listed {
            repo: Repo {
                owner: "example".to_owned(),
                name: "alpha".to_owned(),
            },
            visibility: Visibility::Private,
            archived: false,
            pushed_at: None,
            default_branch: Some("main".to_owned()),
        };
        let check = Check {
            remote_ref: "refs/heads/main".to_owned(),
            remote: Some("abc".to_owned()),
            local: Some("abc".to_owned()),
            verdict: Verdict::InSync,
        };
        let readings = [
            Reading::Account(Ok(vec![listed])),
            Reading::Head {
                project: "alpha".to_owned(),
                check: check.clone(),
            },
            Reading::Run {
                project: "alpha".to_owned(),
                run: Ok(None),
            },
        ];
        for reading in readings {
            let (project, source, json) = reading.to_cache().unwrap();
            assert_eq!(Reading::from_cache(&project, source, &json), Some(reading));
        }
        let failed = [
            Reading::Account(Err("offline".to_owned())),
            Reading::Head {
                project: "alpha".to_owned(),
                check: Check {
                    verdict: Verdict::Failed("offline".to_owned()),
                    ..check
                },
            },
            Reading::Run {
                project: "alpha".to_owned(),
                run: Err("offline".to_owned()),
            },
        ];
        assert!(failed.iter().all(|reading| reading.to_cache().is_none()));
        assert_eq!(Reading::from_cache("alpha", "usage", "{}"), None);
        assert_eq!(Reading::from_cache("alpha", HEAD, "{\"later\": 1}"), None);
    }
}
