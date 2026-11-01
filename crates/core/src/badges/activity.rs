//! Activity (D-25): how lively a repository is, from the git pass's two children: the working
//! tree's status (the changed paths, the branch against its upstream) and the last commit's
//! time, which also sets the staleness colour.

use chrono::{DateTime, TimeDelta, Utc};

use crate::git;
use crate::proposal::STALE_DAYS;

/// A last commit younger than this is fresh.
pub const FRESH_DAYS: i64 = 7;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Activity {
    /// The commit `HEAD` names, which a head check is set against.
    pub oid: Option<String>,
    pub last_commit: Option<DateTime<Utc>>,
    /// Paths changed, staged, in conflict or untracked.
    pub changed: usize,
    pub branch: Option<String>,
    pub upstream: Option<String>,
    /// Ahead of and behind the upstream as git last saw it (at the last fetch or push).
    pub ahead_behind: Option<(u32, u32)>,
}

/// How long the repository has been quiet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Staleness {
    /// A commit within [`FRESH_DAYS`].
    Fresh,
    /// Older, but within D-6's [`STALE_DAYS`].
    Quiet,
    /// No commit for [`STALE_DAYS`] or more: an active project is proposed for pausing.
    Stale,
}

impl Activity {
    pub fn read(status: &git::Status, last_commit: Option<DateTime<Utc>>) -> Self {
        Self {
            oid: status.oid.clone(),
            last_commit,
            changed: status.changes.len(),
            branch: status.branch.clone(),
            upstream: status.upstream.clone(),
            ahead_behind: status.ahead_behind,
        }
    }

    /// `None` before the first commit.
    pub fn staleness(&self, now: DateTime<Utc>) -> Option<Staleness> {
        let age = now - self.last_commit?;
        Some(if age < TimeDelta::days(FRESH_DAYS) {
            Staleness::Fresh
        } else if age < TimeDelta::days(STALE_DAYS) {
            Staleness::Quiet
        } else {
            Staleness::Stale
        })
    }

    /// `main → origin/main, 2 ahead`, `main, no upstream`, `detached`.
    pub fn branch_text(&self) -> String {
        let Some(branch) = &self.branch else {
            return "detached HEAD".to_owned();
        };
        let Some(upstream) = &self.upstream else {
            return format!("{branch}, no upstream");
        };
        let counts = match self.ahead_behind {
            Some((0, 0)) => "level".to_owned(),
            Some((ahead, 0)) => format!("{ahead} ahead"),
            Some((0, behind)) => format!("{behind} behind"),
            Some((ahead, behind)) => format!("{ahead} ahead, {behind} behind"),
            None => "the upstream is gone".to_owned(),
        };
        format!("{branch} → {upstream}, {counts}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_status_and_ages_the_last_commit() {
        let status = git::parse_status(
            "# branch.oid abc\0# branch.head main\0# branch.upstream origin/main\0# branch.ab +2 -0\0\
             1 .M N... 100644 100644 100644 a a x.rs\0? y.rs\0",
        );
        let now = Utc::now();
        let mut activity = Activity::read(&status, Some(now - TimeDelta::days(3)));
        assert_eq!(activity.changed, 2);
        assert_eq!(activity.branch_text(), "main → origin/main, 2 ahead");
        assert_eq!(activity.staleness(now), Some(Staleness::Fresh));
        activity.last_commit = Some(now - TimeDelta::days(FRESH_DAYS));
        assert_eq!(activity.staleness(now), Some(Staleness::Quiet));
        activity.last_commit = Some(now - TimeDelta::days(STALE_DAYS));
        assert_eq!(activity.staleness(now), Some(Staleness::Stale));
        activity.last_commit = None;
        assert_eq!(activity.staleness(now), None);

        activity.ahead_behind = Some((1, 4));
        assert_eq!(
            activity.branch_text(),
            "main → origin/main, 1 ahead, 4 behind"
        );
        activity.ahead_behind = None;
        assert_eq!(
            activity.branch_text(),
            "main → origin/main, the upstream is gone"
        );
        activity.upstream = None;
        assert_eq!(activity.branch_text(), "main, no upstream");
        activity.branch = None;
        assert_eq!(activity.branch_text(), "detached HEAD");
    }
}
