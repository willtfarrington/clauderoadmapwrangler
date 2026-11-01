//! Health (D-25): a licence file at the project's root, and for a GitHub repository with
//! workflows the latest GitHub Actions run, from one `gh run list`. Code scanning and dependency
//! alerts are parked (v2 notes).

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::hosting::{self, Repo};
use super::rfc3339;
use crate::process;

/// The fields `gh run list` is asked for.
const RUN_FIELDS: &str = "conclusion,status,createdAt,workflowName,headSha";

/// The licence file at the root, by name: `LICENSE`, `LICENSE.md`, `LICENCE`, `COPYING`, ….
pub fn licence(folder: &Path) -> Option<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .ok()?
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| {
            let upper = name.to_ascii_uppercase();
            ["LICENSE", "LICENCE", "COPYING", "UNLICENSE"]
                .iter()
                .any(|stem| upper.starts_with(stem))
        })
        .collect();
    names.sort();
    names.into_iter().next()
}

/// How many workflow files `.github/workflows` holds.
pub fn workflows(folder: &Path) -> usize {
    let Ok(entries) = fs::read_dir(folder.join(".github").join("workflows")) else {
        return 0;
    };
    entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| {
            let lower = name.to_ascii_lowercase();
            lower.ends_with(".yml") || lower.ends_with(".yaml")
        })
        .count()
}

/// The latest Actions run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    pub workflow: String,
    /// `queued`, `in_progress`, `completed`, ….
    pub status: String,
    /// `success`, `failure`, `cancelled`, …; `None` while it runs.
    pub conclusion: Option<String>,
    #[serde(with = "rfc3339")]
    pub created_at: Option<DateTime<Utc>>,
    pub head_sha: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Failed,
    Running,
    /// Cancelled, skipped, neutral and the rest.
    Other,
}

impl Run {
    pub fn outcome(&self) -> Outcome {
        match (self.status.as_str(), self.conclusion.as_deref()) {
            ("completed", Some("success")) => Outcome::Passed,
            ("completed", Some("failure" | "timed_out" | "startup_failure")) => Outcome::Failed,
            ("completed", _) => Outcome::Other,
            _ => Outcome::Running,
        }
    }

    /// `ci: success`, `ci: in_progress`.
    pub fn text(&self) -> String {
        let state = self.conclusion.as_deref().unwrap_or(&self.status);
        format!("{}: {state}", self.workflow)
    }
}

/// `gh run list --json …`, read: the first run, or `None` when the repository has none.
pub fn parse_runs(json: &str) -> Result<Option<Run>, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("gh printed no runs: {e}"))?;
    let runs = value.as_array().ok_or("gh printed no runs")?;
    Ok(runs.first().map(|run| {
        let text = |key: &str| {
            run[key]
                .as_str()
                .filter(|text| !text.is_empty())
                .map(str::to_owned)
        };
        Run {
            workflow: text("workflowName").unwrap_or_else(|| "workflow".to_owned()),
            status: text("status").unwrap_or_default(),
            conclusion: text("conclusion"),
            created_at: run["createdAt"].as_str().and_then(rfc3339::parse),
            head_sha: text("headSha"),
        }
    }))
}

/// The repository's latest run: one networked child.
pub fn latest_run(repo: &Repo) -> Result<Option<Run>, String> {
    let slug = repo.slug();
    let args = [
        "run", "list", "-R", &slug, "--limit", "1", "--json", RUN_FIELDS,
    ];
    let text = process::output_with(&hosting::gh(), args, &hosting::GH_ENV, process::TIMEOUT)
        .map_err(|e| format!("gh run list: {e}"))?;
    parse_runs(&text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    #[test]
    fn finds_the_licence_and_the_workflows() {
        let scratch = Scratch::new("health-files");
        let root = &scratch.0;
        assert_eq!(licence(root), None);
        assert_eq!(workflows(root), 0);
        fs::write(root.join("LICENSE-MIT"), "").unwrap();
        fs::write(root.join("Licence.md"), "").unwrap();
        fs::write(root.join("licensing-notes.txt"), "").unwrap();
        assert_eq!(licence(root).as_deref(), Some("LICENSE-MIT"));
        let flows = root.join(".github").join("workflows");
        fs::create_dir_all(&flows).unwrap();
        for name in ["ci.yml", "pages.YAML", "README.md"] {
            fs::write(flows.join(name), "").unwrap();
        }
        assert_eq!(workflows(root), 2);
    }

    #[test]
    fn reads_the_latest_run() {
        let json = r#"[{"conclusion":"success","createdAt":"2026-11-01T21:31:30Z","headSha":"b1deeeb","status":"completed","workflowName":"ci"}]"#;
        let run = parse_runs(json).unwrap().unwrap();
        assert_eq!(run.outcome(), Outcome::Passed);
        assert_eq!(run.text(), "ci: success");
        assert_eq!(run.head_sha.as_deref(), Some("b1deeeb"));
        let running = parse_runs(
            r#"[{"conclusion":"","createdAt":"2026-11-01T21:31:30Z","headSha":"b","status":"in_progress","workflowName":"ci"}]"#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            (running.outcome(), running.conclusion),
            (Outcome::Running, None)
        );
        let failed = Run {
            conclusion: Some("failure".to_owned()),
            ..run.clone()
        };
        assert_eq!(failed.outcome(), Outcome::Failed);
        let cancelled = Run {
            conclusion: Some("cancelled".to_owned()),
            ..run.clone()
        };
        assert_eq!(cancelled.outcome(), Outcome::Other);
        assert_eq!(parse_runs("[]").unwrap(), None);
        assert!(parse_runs("{").is_err());
        let json = serde_json::to_string(&Some(run.clone())).unwrap();
        assert_eq!(
            serde_json::from_str::<Option<Run>>(&json).unwrap(),
            Some(run)
        );
    }
}
