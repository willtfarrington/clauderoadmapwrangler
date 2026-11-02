//! Proposed state changes (D-6): facts about a project suggest a state, and the owner confirms
//! it or dismisses it; a state never changes on its own. The last commit comes from git here;
//! whether every row is checked and whether the next brief has a pickup gate come from the
//! parsers (EP-3, EP-4). A dismissal is kept in the registry with the fact the proposal rested
//! on, so the proposal returns when that fact changes (EP-33).

use std::ffi::OsStr;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

use crate::process;
use crate::registry::State;
use crate::roadmap::EpId;

/// An active project with no commit for this many days is proposed for pausing.
pub const STALE_DAYS: i64 = 60;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    /// Every row of the roadmap is checked; `None` until the table parser reads it (EP-3).
    pub all_rows_checked: Option<bool>,
    /// How many rows the roadmap has: what a dismissed "every row is checked" rests on.
    pub rows: usize,
    /// The next brief has a pickup gate; `None` until the brief parser reads it (EP-4).
    pub pickup_gate: Option<bool>,
    /// The next EP, whose brief a pickup gate is on.
    pub next: Option<EpId>,
    /// The time of the newest commit; `None` outside git and before the first commit.
    pub last_commit: Option<DateTime<Utc>>,
}

/// Why a state is proposed, with the fact the proposal rests on: a dismissal keeps that fact
/// (D-6 addendum).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// Every one of the roadmap's `rows` is checked.
    AllRowsChecked { rows: usize },
    /// The next brief, `ep`, has a pickup gate.
    PickupGate { ep: EpId },
    /// No commit for `days` days; the last one was made at `last`.
    NoCommitFor { days: i64, last: DateTime<Utc> },
}

impl Reason {
    pub fn kind(&self) -> Kind {
        match self {
            Reason::AllRowsChecked { .. } => Kind::AllRowsChecked,
            Reason::PickupGate { .. } => Kind::PickupGate,
            Reason::NoCommitFor { .. } => Kind::NoCommit,
        }
    }

    /// What has to change for a dismissed proposal of this reason to return.
    pub fn returns_when(&self) -> String {
        match self {
            Reason::AllRowsChecked { rows } => {
                format!("the roadmap's row count changes from {rows}")
            }
            Reason::PickupGate { ep } => format!("the next brief is no longer {ep}"),
            Reason::NoCommitFor { last, .. } => format!(
                "a commit is made after the one of {}",
                last.format("%Y-%m-%d")
            ),
        }
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reason::AllRowsChecked { .. } => f.write_str("every row is checked"),
            Reason::PickupGate { ep } => write!(f, "the next brief, {ep}, has a pickup gate"),
            Reason::NoCommitFor { days, .. } => write!(f, "no commit for {days} days"),
        }
    }
}

/// The three kinds of proposal, by the key a dismissal is kept under in the registry, which is
/// also the name `crw dismiss` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    AllRowsChecked,
    PickupGate,
    NoCommit,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::AllRowsChecked, Kind::PickupGate, Kind::NoCommit];

    pub fn key(self) -> &'static str {
        match self {
            Kind::AllRowsChecked => "all_rows_checked",
            Kind::PickupGate => "pickup_gate",
            Kind::NoCommit => "no_commit_since",
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

impl FromStr for Kind {
    type Err = String;

    /// A key, with `-` taken for `_`.
    fn from_str(s: &str) -> Result<Self, String> {
        let key = s.replace('-', "_");
        Kind::ALL
            .into_iter()
            .find(|kind| kind.key() == key)
            .ok_or_else(|| {
                format!("unknown proposal {s:?}: all_rows_checked, pickup_gate or no_commit_since")
            })
    }
}

/// A state the owner may confirm, and why it is proposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    /// The states to choose from.
    pub to: &'static [State],
    pub reason: Reason,
    /// The owner dismissed it and its fact is still the one kept: it is shown apart, with
    /// Restore, never as the bulb (D-6 addendum).
    pub dismissed: bool,
}

/// The proposals the owner dismissed, each kind with the fact it rested on: a project's
/// `dismissed` key in the registry, such as `{ pickup_gate = "EP-5", no_commit_since =
/// 2026-08-15T10:00:00Z }` (D-6 addendum). A proposal is left out while its fact is the one
/// kept; a newer dismissal of the same kind replaces the older.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dismissed {
    /// The roadmap's row count.
    pub all_rows_checked: Option<usize>,
    /// The brief whose pickup gate it was.
    #[serde(default, deserialize_with = "ep_id")]
    pub pickup_gate: Option<EpId>,
    /// The time of the last commit.
    #[serde(default, deserialize_with = "moment")]
    pub no_commit_since: Option<DateTime<Utc>>,
}

impl Dismissed {
    /// Whether `reason` is the proposal dismissed: its kind's fact is the one kept.
    pub fn covers(&self, reason: &Reason) -> bool {
        match *reason {
            Reason::AllRowsChecked { rows } => self.all_rows_checked == Some(rows),
            Reason::PickupGate { ep } => self.pickup_gate == Some(ep),
            Reason::NoCommitFor { last, .. } => self.no_commit_since == Some(last),
        }
    }
}

/// `pickup_gate = "EP-5"`.
fn ep_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<EpId>, D::Error> {
    let text = String::deserialize(deserializer)?;
    text.parse().map(Some).map_err(D::Error::custom)
}

/// `no_commit_since = 2026-08-15T10:00:00Z`: a TOML date and time with its offset.
fn moment<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error> {
    let value = toml::value::Datetime::deserialize(deserializer)?;
    DateTime::parse_from_rfc3339(&value.to_string())
        .map(|at| Some(at.to_utc()))
        .map_err(|_| D::Error::custom(format!("{value} is not a date and a time with its offset")))
}

/// The proposals for a project in `state` (D-6): every row checked → done or maintenance; a pickup
/// gate on the next brief, or no commit for [`STALE_DAYS`] days → paused, for active projects.
/// Each says whether the owner dismissed it (`dismissed`).
pub fn propose(
    state: State,
    facts: &Facts,
    dismissed: &Dismissed,
    now: DateTime<Utc>,
) -> Vec<Proposal> {
    let mut found: Vec<(&'static [State], Reason)> = Vec::new();
    if facts.all_rows_checked == Some(true) && matches!(state, State::Active | State::Paused) {
        found.push((
            &[State::Done, State::Maintenance],
            Reason::AllRowsChecked { rows: facts.rows },
        ));
    }
    if state == State::Active {
        if let (Some(true), Some(ep)) = (facts.pickup_gate, facts.next) {
            found.push((&[State::Paused], Reason::PickupGate { ep }));
        }
        if let Some(last) = facts.last_commit {
            let days = (now - last).num_days();
            if days >= STALE_DAYS {
                found.push((&[State::Paused], Reason::NoCommitFor { days, last }));
            }
        }
    }
    found
        .into_iter()
        .map(|(to, reason)| Proposal {
            to,
            reason,
            dismissed: dismissed.covers(&reason),
        })
        .collect()
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

    fn ep(number: u32) -> EpId {
        EpId::new(number)
    }

    #[test]
    fn proposes_from_the_facts() {
        let now = Utc::now();
        let none = Dismissed::default();
        let stale_since = now - TimeDelta::days(STALE_DAYS);
        let stale = Facts {
            last_commit: Some(stale_since),
            ..Facts::default()
        };
        assert_eq!(
            propose(State::Active, &stale, &none, now),
            [Proposal {
                to: &[State::Paused],
                reason: Reason::NoCommitFor {
                    days: 60,
                    last: stale_since
                },
                dismissed: false,
            }]
        );
        let recent = Facts {
            last_commit: Some(now - TimeDelta::days(STALE_DAYS - 1)),
            ..Facts::default()
        };
        assert!(propose(State::Active, &recent, &none, now).is_empty());
        // Quiet maintenance and paused projects are not nagged.
        assert!(propose(State::Maintenance, &stale, &none, now).is_empty());
        assert!(propose(State::Paused, &stale, &none, now).is_empty());

        let long_ago = now - TimeDelta::days(90);
        let finished = Facts {
            all_rows_checked: Some(true),
            rows: 12,
            pickup_gate: Some(true),
            next: Some(ep(13)),
            last_commit: Some(long_ago),
        };
        let reasons: Vec<_> = propose(State::Active, &finished, &none, now)
            .into_iter()
            .map(|p| (p.to, p.reason))
            .collect();
        assert_eq!(
            reasons,
            [
                (
                    &[State::Done, State::Maintenance][..],
                    Reason::AllRowsChecked { rows: 12 }
                ),
                (&[State::Paused][..], Reason::PickupGate { ep: ep(13) }),
                (
                    &[State::Paused][..],
                    Reason::NoCommitFor {
                        days: 90,
                        last: long_ago
                    }
                ),
            ]
        );
        assert_eq!(propose(State::Paused, &finished, &none, now).len(), 1);
        assert!(propose(State::Done, &finished, &none, now).is_empty());
        // Unknown facts propose nothing.
        assert!(propose(State::Active, &Facts::default(), &none, now).is_empty());
        assert_eq!(
            Reason::NoCommitFor {
                days: 61,
                last: long_ago
            }
            .to_string(),
            "no commit for 61 days"
        );
        assert_eq!(
            Reason::PickupGate { ep: ep(5) }.to_string(),
            "the next brief, EP-5, has a pickup gate"
        );
    }

    #[test]
    fn a_dismissed_proposal_returns_when_its_fact_changes() {
        let now = Utc::now();
        let last = now - TimeDelta::days(70);
        let facts = Facts {
            all_rows_checked: Some(true),
            rows: 30,
            pickup_gate: Some(true),
            next: Some(ep(5)),
            last_commit: Some(last),
        };
        let dismissed = Dismissed {
            all_rows_checked: Some(30),
            pickup_gate: Some(ep(5)),
            no_commit_since: Some(last),
        };
        let open = |facts: &Facts| -> Vec<Kind> {
            propose(State::Active, facts, &dismissed, now)
                .into_iter()
                .filter(|p| !p.dismissed)
                .map(|p| p.reason.kind())
                .collect()
        };
        assert!(open(&facts).is_empty(), "every proposal is dismissed");
        // The days keep counting, but the last commit is the same one.
        let later = now + TimeDelta::days(9);
        assert!(
            propose(State::Active, &facts, &dismissed, later)
                .iter()
                .all(|p| p.dismissed)
        );
        // A re-plan added rows and they are all checked again; the next brief changed; a commit
        // came and went stale.
        let changed = Facts {
            rows: 35,
            next: Some(ep(6)),
            last_commit: Some(last + TimeDelta::days(1)),
            ..facts.clone()
        };
        assert_eq!(
            open(&changed),
            [Kind::AllRowsChecked, Kind::PickupGate, Kind::NoCommit]
        );
        assert_eq!(
            Reason::PickupGate { ep: ep(5) }.returns_when(),
            "the next brief is no longer EP-5"
        );
    }

    #[test]
    fn kinds_read_by_their_keys() {
        for kind in Kind::ALL {
            assert_eq!(kind.key().parse::<Kind>(), Ok(kind));
        }
        assert_eq!("pickup-gate".parse::<Kind>(), Ok(Kind::PickupGate));
        assert!("stale".parse::<Kind>().is_err());
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
