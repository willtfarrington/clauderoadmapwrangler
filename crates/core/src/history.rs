//! A project's history from git (D-26; EP-25): the commits whose subjects record an EP give the
//! EP's completion date, the brief's completion note cross-checks it, and the roadmap's Done
//! cells name the work commits. One `git log` per import, in its turn (D-29); no transcript is
//! read. The import is kept in the ledger per project and run again when `HEAD` moves or on
//! Re-import; the graphs (EP-26) and the exports (EP-27) read it from there.

use std::fmt;
use std::path::Path;

use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};

use crate::git;
use crate::roadmap::snapshot::{self, Snapshot};
use crate::roadmap::table::{Done, Roadmap};
use crate::roadmap::{EpId, first_date, scan_ep_id};

/// One commit of the log: its full hash, its committer time and its subject line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub hash: String,
    pub at: DateTime<Utc>,
    pub subject: String,
}

/// `git log --format=%H%x09%ct%x09%s` over the whole history of `folder`, the newest first; one
/// child. An error is git's own words (not a repository, no commit yet).
pub fn log(folder: &Path) -> Result<Vec<Commit>, String> {
    git::run(folder, &["log", "--format=%H%x09%ct%x09%s"]).map(|text| parse_log(&text))
}

/// Reads the log's lines: hash, tab, epoch seconds, tab, subject. A line that does not read is
/// left out.
pub fn parse_log(text: &str) -> Vec<Commit> {
    text.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\t');
            let hash = parts.next()?.trim();
            let at = parts.next()?.trim().parse::<i64>().ok()?;
            let subject = parts.next().unwrap_or_default().trim();
            if hash.is_empty() {
                return None;
            }
            Some(Commit {
                hash: hash.to_owned(),
                at: DateTime::from_timestamp(at, 0)?,
                subject: subject.to_owned(),
            })
        })
        .collect()
}

/// What a commit's subject says of an EP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// The commit that marks the EP done in the roadmap: `docs(roadmap): record EP-n commit
    /// hash`, with any suffix or a word between (`record EP-20c review-fix commit hash`), or the
    /// packet style's `EP-n: mark done at <hash>`.
    Record,
    /// The EP's own work: a subject ending in a parenthesis that opens with the id (`… (EP-9,
    /// D-14)`, `… (EP-0, D-1 … D-30)`), or starting with it before a colon (`EP-15: …`, `EP-20c
    /// review fixes: …`).
    Work,
}

/// The EP a subject records or works on; `None` for a subject that only mentions one (`record
/// EP-8's CI run`, `docs(roadmap): EP-7a, list and detail polish`).
pub fn classify(subject: &str) -> Option<(EpId, Marker)> {
    let subject = subject.trim();
    if let Some(id) = recorded(subject) {
        return Some((id, Marker::Record));
    }
    if let Some(id) = worked(subject) {
        return Some((id, Marker::Work));
    }
    None
}

/// `record EP-n … commit hash`, or `EP-n: mark done …`.
fn recorded(subject: &str) -> Option<EpId> {
    if let Some(at) = subject.find("record EP-")
        && let Some((id, end)) = scan_ep_id(subject, at + "record ".len())
        && subject[end..].contains("commit hash")
    {
        return Some(id);
    }
    let (id, end) = scan_ep_id(subject, 0)?;
    subject[end..]
        .strip_prefix(':')
        .map(str::trim_start)
        .is_some_and(|rest| rest.starts_with("mark done"))
        .then_some(id)
}

/// `… (EP-n, …)` or `… (EP-n)` at the end, else `EP-n:` or `EP-n <words>:` at the start.
fn worked(subject: &str) -> Option<EpId> {
    if let Some(open) = subject.rfind('(')
        && subject.ends_with(')')
        && let Some((id, end)) = scan_ep_id(subject, open + 1)
        && subject[end..].starts_with([',', ')'])
    {
        return Some(id);
    }
    let (id, end) = scan_ep_id(subject, 0)?;
    let rest = &subject[end..];
    let head = rest.split(['(', ')']).next().unwrap_or_default();
    (rest.starts_with(':') || (rest.starts_with(' ') && head.contains(':'))).then_some(id)
}

/// How an EP's date was settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// Git and the note within a day of each other: git's date stands.
    Agreed,
    /// No completion note, or no note date: git's date.
    GitOnly,
    /// No commit carries the EP (or the history is a single commit): the note's date.
    NoteOnly,
    /// Git and the note more than a day apart: the note wins, flagged for a look.
    NoteWins,
}

impl Flag {
    pub fn as_str(self) -> &'static str {
        match self {
            Flag::Agreed => "agreed",
            Flag::GitOnly => "git",
            Flag::NoteOnly => "note",
            Flag::NoteWins => "note-wins",
        }
    }

    pub fn parse(text: &str) -> Option<Flag> {
        match text {
            "agreed" => Some(Flag::Agreed),
            "git" => Some(Flag::GitOnly),
            "note" => Some(Flag::NoteOnly),
            "note-wins" => Some(Flag::NoteWins),
            _ => None,
        }
    }

    /// The EP's date deserves a look: the sources disagree.
    pub fn is_flagged(self) -> bool {
        self == Flag::NoteWins
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Flag::Agreed => "git and the note agree",
            Flag::GitOnly => "git only",
            Flag::NoteOnly => "the note only",
            Flag::NoteWins => "the note wins over git",
        })
    }
}

/// One EP's completion as the import settles it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub ep: EpId,
    pub completed_on: NaiveDate,
    /// The earliest record commit's local date, else the latest work commit's.
    pub git_date: Option<NaiveDate>,
    /// The brief's completion-note date.
    pub note_date: Option<NaiveDate>,
    /// The record commits' hashes, the earliest first.
    pub record: Vec<String>,
    /// The work commits' hashes (the markers' and the Done cell's), the earliest first.
    pub work: Vec<String>,
    pub flag: Flag,
}

/// What an import read and settled.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Import {
    /// By EP.
    pub completions: Vec<Completion>,
    /// The commits the log held.
    pub commits: usize,
    /// A history of one commit (or none): the notes' dates alone were used.
    pub single: bool,
}

impl Import {
    pub fn flagged(&self) -> impl Iterator<Item = &Completion> {
        self.completions.iter().filter(|c| c.flag.is_flagged())
    }

    /// The earliest and the latest completion dates.
    pub fn span(&self) -> Option<(NaiveDate, NaiveDate)> {
        let dates = self.completions.iter().map(|c| c.completed_on);
        Some((dates.clone().min()?, dates.max()?))
    }
}

/// An import as the ledger keeps it (EP-25): when it ran, the `HEAD` it read, and its result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub imported_at: DateTime<Utc>,
    pub head: Option<String>,
    pub import: Import,
}

/// The rows' Done hashes by EP, from the current reading.
pub fn rows_of(roadmap: &Roadmap) -> Vec<(EpId, Vec<String>)> {
    roadmap
        .rows()
        .map(|row| {
            let hashes = match &row.done {
                Done::Checked(hashes) => hashes.clone(),
                _ => Vec::new(),
            };
            (row.id, hashes)
        })
        .collect()
}

/// The briefs' completion dates by EP, from the files.
pub fn notes_of(snapshot: &Snapshot) -> Vec<(EpId, NaiveDate)> {
    snapshot
        .briefs
        .iter()
        .filter_map(|entry| {
            let date = entry.completed_on()?;
            let date = first_date(&date)?.parse().ok()?;
            Some((entry.file.id, date))
        })
        .collect()
}

/// Settles one completion per EP from the log, the rows and the notes; each commit's day in
/// `zone` at the commit's own instant (`Local` follows the clock change, a fixed offset does
/// not). An EP no commit, no cell and no note carries has none.
pub fn import<Tz: TimeZone>(
    commits: &[Commit],
    rows: &[(EpId, Vec<String>)],
    notes: &[(EpId, NaiveDate)],
    zone: &Tz,
) -> Import {
    let single = commits.len() <= 1;
    let day = |commit: &Commit| commit.at.with_timezone(zone).date_naive();
    let mut ids: Vec<EpId> = Vec::new();
    let mut push = |id: EpId| {
        if !ids.contains(&id) {
            ids.push(id);
        }
    };
    // The oldest first, so each EP's hashes come in time order.
    let oldest_first: Vec<&Commit> = commits.iter().rev().collect();
    let mut records: Vec<(EpId, &Commit)> = Vec::new();
    let mut works: Vec<(EpId, &Commit)> = Vec::new();
    if !single {
        for commit in &oldest_first {
            match classify(&commit.subject) {
                Some((id, Marker::Record)) => {
                    records.push((id, commit));
                    push(id);
                }
                Some((id, Marker::Work)) => {
                    works.push((id, commit));
                    push(id);
                }
                None => {}
            }
        }
        for (id, hashes) in rows {
            for cell in hashes {
                let cell = cell.trim().to_ascii_lowercase();
                if cell.len() < 7 {
                    continue;
                }
                if let Some(commit) = oldest_first
                    .iter()
                    .find(|commit| commit.hash.to_ascii_lowercase().starts_with(&cell))
                    && !works
                        .iter()
                        .any(|(w_id, w)| *w_id == *id && w.hash == commit.hash)
                {
                    works.push((*id, commit));
                    push(*id);
                }
            }
        }
    }
    for (id, _) in notes {
        push(*id);
    }
    ids.sort();
    let completions = ids
        .into_iter()
        .filter_map(|ep| {
            let mut record: Vec<&Commit> = records
                .iter()
                .filter(|(id, _)| *id == ep)
                .map(|(_, c)| *c)
                .collect();
            let mut work: Vec<&Commit> = works
                .iter()
                .filter(|(id, _)| *id == ep)
                .map(|(_, c)| *c)
                .collect();
            record.sort_by_key(|c| c.at);
            work.sort_by_key(|c| c.at);
            let git_date = record
                .first()
                .map(|c| day(c))
                .or_else(|| work.last().map(|c| day(c)));
            let note_date = notes.iter().find(|(id, _)| *id == ep).map(|(_, d)| *d);
            let (completed_on, flag) = match (git_date, note_date) {
                (Some(git), Some(note)) if (note - git).num_days().abs() > 1 => {
                    (note, Flag::NoteWins)
                }
                (Some(git), Some(_)) => (git, Flag::Agreed),
                (Some(git), None) => (git, Flag::GitOnly),
                (None, Some(note)) => (note, Flag::NoteOnly),
                (None, None) => return None,
            };
            Some(Completion {
                ep,
                completed_on,
                git_date,
                note_date,
                record: record.iter().map(|c| c.hash.clone()).collect(),
                work: work.iter().map(|c| c.hash.clone()).collect(),
                flag,
            })
        })
        .collect();
    Import {
        completions,
        commits: commits.len(),
        single,
    }
}

/// Reads `folder` whole: its log (one child, the caller holding the turn) and its files, settled
/// in the machine's local day. A folder with no log (not a repository, no commit) imports from
/// the notes alone.
pub fn read(folder: &Path) -> Import {
    let commits = log(folder).unwrap_or_default();
    let snapshot = snapshot::read_tree(folder);
    let rows = snapshot.current().map(rows_of).unwrap_or_default();
    let notes = notes_of(&snapshot);
    import(&commits, &rows, &notes, &Local)
}

/// A hash as the roadmap writes it: its first seven characters.
pub fn short(hash: &str) -> &str {
    hash.get(..7).unwrap_or(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(text: &str) -> EpId {
        text.parse().unwrap()
    }

    fn date(text: &str) -> NaiveDate {
        text.parse().unwrap()
    }

    fn commit(hash: &str, at: &str, subject: &str) -> Commit {
        Commit {
            hash: hash.to_owned(),
            at: chrono::NaiveDateTime::parse_from_str(at, "%Y-%m-%d %H:%M")
                .unwrap()
                .and_utc(),
            subject: subject.to_owned(),
        }
    }

    const UTC: chrono::FixedOffset = chrono::FixedOffset::east_opt(0).unwrap();

    #[test]
    fn classifies_every_subject_form_and_the_noise() {
        let record = |s: &str| classify(s).map(|(id, m)| (id.to_string(), m));
        assert_eq!(
            record("docs(roadmap): record EP-24b commit hash"),
            Some(("EP-24b".to_owned(), Marker::Record))
        );
        assert_eq!(
            record(
                "docs(roadmap): record EP-20b commit hash (and EP-20c's front matter set to done)"
            ),
            Some(("EP-20b".to_owned(), Marker::Record))
        );
        assert_eq!(
            record("docs(roadmap): record EP-20c review-fix commit hash"),
            Some(("EP-20c".to_owned(), Marker::Record))
        );
        assert_eq!(
            record("EP-15: mark done at 9638bd2; the hand check tallied 34 of 40"),
            Some(("EP-15".to_owned(), Marker::Record))
        );
        assert_eq!(
            record("Chat, the hosted session: one read-only session (EP-24, D-27, D-26, D-18)"),
            Some(("EP-24".to_owned(), Marker::Work))
        );
        assert_eq!(
            record(
                "Planning: interview, research pass, roadmap and decision log (EP-0, D-1 … D-30)"
            ),
            Some(("EP-0".to_owned(), Marker::Work))
        );
        assert_eq!(
            record("The installed exe and the window's restart: crw install (EP-37)"),
            Some(("EP-37".to_owned(), Marker::Work))
        );
        assert_eq!(
            record("EP-15: the concordance's Spearman rho as the Pearson correlation"),
            Some(("EP-15".to_owned(), Marker::Work))
        );
        assert_eq!(
            record("EP-20c review fixes: Send refused while a revert is open"),
            Some(("EP-20c".to_owned(), Marker::Work))
        );
        // Noise: a mention is no marker.
        assert_eq!(record("docs(roadmap): record EP-8's CI run"), None);
        assert_eq!(record("docs(roadmap): record EP-7a's CI run"), None);
        assert_eq!(
            record("docs(roadmap): EP-7a, list and detail polish, from the owner's notes (IN-4)"),
            None
        );
        assert_eq!(
            record("docs(roadmap): record EP-16's first self-launched session and its finds"),
            None
        );
        assert_eq!(
            record("docs(inbox): IN-2, neighbour folders by --add-dir"),
            None
        );
        assert_eq!(
            record("docs(roadmap): EP-16 note: an additional directory"),
            None
        );
        assert_eq!(
            record("memory: facts from clauderoadmapwrangler EP-11 and EP-12 (D-4)"),
            None
        );
        assert_eq!(record("EP-16 was long"), None);
        assert_eq!(record(""), None);
    }

    #[test]
    fn reads_the_logs_lines() {
        let text = "aaaaaaa1\t1761900000\tdocs(roadmap): record EP-1 commit hash\n\
                    bbbbbbb2\t1761890000\tToolchain bootstrap: the workspace (EP-1, D-2)\n\
                    bad line\n\
                    ccccccc3\t1761880000\t\n";
        let commits = parse_log(text);
        assert_eq!(commits.len(), 3);
        assert_eq!(commits[0].hash, "aaaaaaa1");
        assert_eq!(commits[0].at.timestamp(), 1761900000);
        assert_eq!(
            commits[1].subject,
            "Toolchain bootstrap: the workspace (EP-1, D-2)"
        );
        assert_eq!(commits[2].subject, "");
    }

    /// A synthetic history, the newest first as git prints it.
    fn history() -> Vec<Commit> {
        vec![
            commit(
                "f000000f",
                "2026-11-10 23:50",
                "docs(roadmap): record EP-20c review-fix commit hash",
            ),
            commit(
                "e000000e",
                "2026-11-10 23:40",
                "EP-20c review fixes: Send refused while a revert is open",
            ),
            commit(
                "d000000d",
                "2026-11-10 22:00",
                "docs(roadmap): record EP-20c commit hash",
            ),
            commit(
                "c000000c",
                "2026-11-10 21:30",
                "Compose and the mailman's limits (EP-20c, D-13)",
            ),
            commit(
                "b000000b",
                "2026-11-08 20:00",
                "docs(roadmap): owner-task cadence, the session's own checks (planning)",
            ),
            commit(
                "a000000a",
                "2026-11-05 10:00",
                "docs(roadmap): record EP-19 commit hash",
            ),
            commit(
                "9000000a",
                "2026-11-04 23:30",
                "Autopilot: a project's switch in the list (EP-19, D-22)",
            ),
            commit(
                "80000008",
                "2026-11-02 09:00",
                "Session runner core (EP-9, D-14)",
            ),
            commit(
                "70000007",
                "2026-11-01 09:00",
                "docs(roadmap): record EP-4 commit hash",
            ),
            commit(
                "60000006",
                "2026-10-31 23:00",
                "Parser B: briefs (EP-4, D-9)",
            ),
            commit(
                "50000005",
                "2026-10-31 20:00",
                "Registry and discovery, no marker here",
            ),
            commit(
                "40000004",
                "2026-10-31 19:00",
                "docs(roadmap): record EP-0 commit hash",
            ),
            commit(
                "30000003",
                "2026-10-31 18:00",
                "Planning: interview (EP-0, D-1 … D-30)",
            ),
        ]
    }

    #[test]
    fn settles_each_eps_date_from_the_record_commit_the_work_commit_or_the_note() {
        let rows = vec![
            (id("EP-0"), vec!["30000003".to_owned()]),
            (id("EP-2"), vec!["5000000".to_owned()]),
            (id("EP-4"), vec!["60000006".to_owned()]),
            (id("EP-9"), vec!["80000008".to_owned()]),
            (id("EP-19"), vec!["9000000a".to_owned()]),
            (
                id("EP-20c"),
                vec!["c000000c".to_owned(), "e000000e".to_owned()],
            ),
            (id("EP-21"), Vec::new()),
        ];
        let notes = vec![
            (id("EP-0"), date("2026-10-31")),
            (id("EP-2"), date("2026-11-03")),
            (id("EP-4"), date("2026-10-31")),
            (id("EP-19"), date("2026-11-05")),
            (id("EP-21"), date("2026-11-10")),
        ];
        let import = import(&history(), &rows, &notes, &UTC);
        assert!(!import.single);
        assert_eq!(import.commits, 13);
        let by_id = |ep: &str| {
            import
                .completions
                .iter()
                .find(|c| c.ep == id(ep))
                .unwrap_or_else(|| panic!("{ep} is settled"))
        };
        // The record commit's date, the note agreeing.
        let ep0 = by_id("EP-0");
        assert_eq!(ep0.completed_on, date("2026-10-31"));
        assert_eq!(
            (ep0.flag, &ep0.record[..], &ep0.work[..]),
            (
                Flag::Agreed,
                &["40000004".to_owned()][..],
                &["30000003".to_owned()][..]
            )
        );
        // A Done cell's hash gives the work commit where the subject carries no marker; the
        // note three days off wins, flagged.
        let ep2 = by_id("EP-2");
        assert_eq!(ep2.work, ["50000005"]);
        assert_eq!(ep2.git_date, Some(date("2026-10-31")));
        assert_eq!(
            (ep2.completed_on, ep2.flag),
            (date("2026-11-03"), Flag::NoteWins)
        );
        // The record commit a day after the note: git's date stands, agreed.
        let ep4 = by_id("EP-4");
        assert_eq!(
            (ep4.completed_on, ep4.flag),
            (date("2026-11-01"), Flag::Agreed)
        );
        // A work commit alone, no note.
        let ep9 = by_id("EP-9");
        assert_eq!(
            (ep9.completed_on, ep9.flag),
            (date("2026-11-02"), Flag::GitOnly)
        );
        assert!(ep9.record.is_empty());
        // Two record commits: the earliest dates it; both hashes kept, the cell's work commit
        // not listed twice.
        let ep20c = by_id("EP-20c");
        assert_eq!(ep20c.record, ["d000000d", "f000000f"]);
        assert_eq!(ep20c.work, ["c000000c", "e000000e"]);
        assert_eq!(
            (ep20c.completed_on, ep20c.flag),
            (date("2026-11-10"), Flag::GitOnly)
        );
        // A note alone.
        let ep21 = by_id("EP-21");
        assert_eq!(
            (ep21.completed_on, ep21.flag),
            (date("2026-11-10"), Flag::NoteOnly)
        );
        assert!(ep21.git_date.is_none());
        // The planning commit carries no EP and no cell names it; an open row with nothing has
        // no completion.
        assert_eq!(
            import
                .completions
                .iter()
                .map(|c| c.ep.to_string())
                .collect::<Vec<_>>(),
            ["EP-0", "EP-2", "EP-4", "EP-9", "EP-19", "EP-20c", "EP-21"]
        );
        assert_eq!(import.flagged().count(), 1);
        assert_eq!(
            import.span(),
            Some((date("2026-10-31"), date("2026-11-10")))
        );
    }

    #[test]
    fn a_single_commit_history_uses_the_notes_alone() {
        let one = vec![commit(
            "10000001",
            "2026-08-18 12:00",
            "alpha: initial public release",
        )];
        let rows = vec![(id("EP-1"), vec!["10000001".to_owned()])];
        let notes = vec![
            (id("EP-1"), date("2026-07-01")),
            (id("EP-2"), date("2026-07-09")),
        ];
        let import = import(&one, &rows, &notes, &UTC);
        assert!(import.single);
        assert_eq!(import.completions.len(), 2);
        assert!(
            import
                .completions
                .iter()
                .all(|c| c.flag == Flag::NoteOnly && c.work.is_empty())
        );
        assert_eq!(import.completions[1].completed_on, date("2026-07-09"));
        // No commit at all reads the same way.
        assert!(super::import(&[], &rows, &notes, &UTC).single);
    }

    #[test]
    fn the_local_day_follows_the_offset() {
        let commits = vec![
            commit(
                "20000002",
                "2026-11-02 03:30",
                "docs(roadmap): record EP-1 commit hash",
            ),
            commit("10000001", "2026-11-02 03:00", "Bootstrap (EP-1)"),
        ];
        let east = chrono::FixedOffset::east_opt(5 * 3600).unwrap();
        let west = chrono::FixedOffset::west_opt(5 * 3600).unwrap();
        assert_eq!(
            import(&commits, &[], &[], &east).completions[0].completed_on,
            date("2026-11-02")
        );
        assert_eq!(
            import(&commits, &[], &[], &west).completions[0].completed_on,
            date("2026-11-01")
        );
        // A zone, not a fixed offset: each commit's own offset at its instant, so the clock
        // change does not move a day (the review's finding).
        let zone = chrono_tz_free_zone();
        assert_eq!(
            import(&commits, &[], &[], &zone).completions[0].completed_on,
            date("2026-11-02")
        );
    }

    /// `Utc` as a `TimeZone` that is not a `FixedOffset`, which the generic bound must take.
    fn chrono_tz_free_zone() -> Utc {
        Utc
    }

    /// Runs git on this repository, never the `claude` CLI (D-29): the import checked against
    /// the retros' facts (EP-8, EP-16, EP-23) and the roadmap's Done cells.
    #[test]
    fn imports_this_repositorys_history() {
        let repo = crate::paths::repository();
        let _turn = crate::process::turn();
        // CI checks the repository out shallow (one grafted commit): nothing to check there.
        let shallow = git::run(&repo, &["rev-parse", "--is-shallow-repository"])
            .map(|text| text.trim() == "true")
            .unwrap_or(true);
        if shallow {
            eprintln!("a shallow clone: the import is not checked against the log");
            return;
        }
        std::thread::sleep(crate::process::SPACING);
        let commits = log(&repo).expect("git answers in this repository");
        assert!(commits.len() > 100, "{} commits", commits.len());
        let import = import(&commits, &[], &[], &UTC);
        let by_id = |ep: &str| import.completions.iter().find(|c| c.ep == id(ep)).unwrap();
        let ep1 = by_id("EP-1");
        assert_eq!(short(&ep1.record[0]), "74540b4");
        assert_eq!(short(&ep1.work[0]), "64d6c6a");
        assert_eq!(ep1.completed_on, date("2026-10-31"));
        let ep20c = by_id("EP-20c");
        assert_eq!(
            ep20c.record.iter().map(|h| short(h)).collect::<Vec<_>>(),
            ["1282256", "471f469"]
        );
        // The work commit, the docs commit that parked its review (`… (EP-20c)`), the fixes;
        // a later commit naming EP-20c would follow them.
        let work: Vec<&str> = ep20c.work.iter().map(|h| short(h)).collect();
        assert_eq!(&work[..3], ["8a1245c", "a8e97e6", "438caf0"]);
        assert_eq!(short(&by_id("EP-37").work[0]), "a8078a5");
        assert_eq!(by_id("EP-0").completed_on, date("2026-10-31"));
        // The planning session's commit carries no EP.
        let planning = commits
            .iter()
            .find(|c| c.hash.starts_with("811d9a6"))
            .unwrap();
        assert_eq!(classify(&planning.subject), None);
    }
}
