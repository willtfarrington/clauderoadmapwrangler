//! Proposed state changes (D-6): facts about a project suggest a state, and the owner confirms
//! it; a state never changes on its own. The last commit comes from git here; whether every row
//! is checked and whether the next brief has a pickup gate come from the parsers (EP-3, EP-4).

use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::process;
use crate::registry::State;

/// An active project with no commit for this many days is proposed for pausing.
pub const STALE_DAYS: i64 = 60;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    /// Every row of the roadmap is checked; `None` until the table parser reads it (EP-3).
    pub all_rows_checked: Option<bool>,
    /// The next brief has a pickup gate; `None` until the brief parser reads it (EP-4).
    pub pickup_gate: Option<bool>,
    /// The time of the newest commit; `None` outside git and before the first commit.
    pub last_commit: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    AllRowsChecked,
    PickupGate,
    NoCommitFor { days: i64 },
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reason::AllRowsChecked => f.write_str("every row is checked"),
            Reason::PickupGate => f.write_str("the next brief has a pickup gate"),
            Reason::NoCommitFor { days } => write!(f, "no commit for {days} days"),
        }
    }
}

/// A state the owner may confirm, and why it is proposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    /// The states to choose from.
    pub to: &'static [State],
    pub reason: Reason,
}

/// The proposals for a project in `state` (D-6): every row checked → done or maintenance; a pickup
/// gate on the next brief, or no commit for [`STALE_DAYS`] days → paused, for active projects.
pub fn propose(state: State, facts: &Facts, now: DateTime<Utc>) -> Vec<Proposal> {
    let mut proposals = Vec::new();
    if facts.all_rows_checked == Some(true) && matches!(state, State::Active | State::Paused) {
        proposals.push(Proposal {
            to: &[State::Done, State::Maintenance],
            reason: Reason::AllRowsChecked,
        });
    }
    if state != State::Active {
        return proposals;
    }
    if facts.pickup_gate == Some(true) {
        proposals.push(Proposal {
            to: &[State::Paused],
            reason: Reason::PickupGate,
        });
    }
    if let Some(days) = facts
        .last_commit
        .map(|at| (now - at).num_days())
        .filter(|&days| days >= STALE_DAYS)
    {
        proposals.push(Proposal {
            to: &[State::Paused],
            reason: Reason::NoCommitFor { days },
        });
    }
    proposals
}

/// The time of the newest commit in `folder`, from `git log -1 --format=%ct`; `None` when git is
/// missing or fails, or there is no commit yet.
pub fn last_commit(folder: &Path) -> Option<DateTime<Utc>> {
    let args = [
        OsStr::new("-C"),
        folder.as_os_str(),
        OsStr::new("log"),
        OsStr::new("-1"),
        OsStr::new("--format=%ct"),
    ];
    let text = process::output_of(Path::new("git"), args, process::TIMEOUT).ok()?;
    DateTime::from_timestamp(text.trim().parse().ok()?, 0)
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;
    use crate::files::tests::Scratch;

    #[test]
    fn proposes_from_the_facts() {
        let now = Utc::now();
        let ago = |days| Some(now - TimeDelta::days(days));
        let stale = Facts {
            last_commit: ago(STALE_DAYS),
            ..Facts::default()
        };
        assert_eq!(
            propose(State::Active, &stale, now),
            [Proposal {
                to: &[State::Paused],
                reason: Reason::NoCommitFor { days: 60 },
            }]
        );
        let recent = Facts {
            last_commit: ago(STALE_DAYS - 1),
            ..Facts::default()
        };
        assert!(propose(State::Active, &recent, now).is_empty());
        // Quiet maintenance and paused projects are not nagged.
        assert!(propose(State::Maintenance, &stale, now).is_empty());
        assert!(propose(State::Paused, &stale, now).is_empty());

        let finished = Facts {
            all_rows_checked: Some(true),
            pickup_gate: Some(true),
            last_commit: ago(90),
        };
        let reasons: Vec<_> = propose(State::Active, &finished, now)
            .into_iter()
            .map(|p| (p.to, p.reason))
            .collect();
        assert_eq!(
            reasons,
            [
                (
                    &[State::Done, State::Maintenance][..],
                    Reason::AllRowsChecked
                ),
                (&[State::Paused][..], Reason::PickupGate),
                (&[State::Paused][..], Reason::NoCommitFor { days: 90 }),
            ]
        );
        assert_eq!(propose(State::Paused, &finished, now).len(), 1);
        assert!(propose(State::Done, &finished, now).is_empty());
        // Unknown facts propose nothing.
        assert!(propose(State::Active, &Facts::default(), now).is_empty());
        assert_eq!(
            Reason::NoCommitFor { days: 61 }.to_string(),
            "no commit for 61 days"
        );
    }

    /// Runs git, never the `claude` CLI (D-29): on this repository, and on a folder that is not a
    /// repository.
    #[test]
    fn reads_the_last_commit_from_git() {
        let at = last_commit(&crate::paths::repository()).expect("this repository has commits");
        assert!(at <= Utc::now());
        let scratch = Scratch::new("proposal-no-git");
        assert_eq!(last_commit(&scratch.0), None);
    }
}
