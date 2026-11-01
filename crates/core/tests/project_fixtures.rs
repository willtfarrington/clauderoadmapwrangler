//! The brief, decision-log and snapshot parsers against four synthetic project folders, one per
//! style the survey found (D-9, D-29), and against a scratch git repository where the working
//! tree moves away from `HEAD` as a session would move it. Git runs here (never the `claude`
//! CLI), one child at a time with the usual spacing.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use crw_core::proposal::Facts;
use crw_core::roadmap::brief::{Evidence, Kind};
use crw_core::roadmap::snapshot::{self, InProgress, Snapshot};
use crw_core::roadmap::table::{Counts, Profile};
use crw_core::roadmap::{EpId, Size};
use crw_core::{process, roadmap};

fn ep(number: u32) -> EpId {
    EpId::new(number)
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("projects")
        .join(name)
}

fn counts(done: usize, total: usize) -> Counts {
    Counts {
        done,
        total,
        stretch: 0,
    }
}

fn facts(snapshot: &Snapshot) -> (Option<bool>, Option<bool>) {
    let mut facts = Facts::default();
    snapshot.fill(&mut facts);
    (facts.all_rows_checked, facts.pickup_gate)
}

fn brief_names(snapshot: &Snapshot) -> Vec<&str> {
    snapshot
        .files
        .briefs
        .iter()
        .map(|file| file.name.as_str())
        .collect()
}

/// The fixtures sit inside this repository, but each folder has no `.git` of its own, so git is
/// not asked and the reading is the working tree's.
#[test]
fn house_a_session_in_progress_on_the_next_brief() {
    let snapshot = snapshot::take(&fixture("house"));
    assert!(snapshot.problems.is_empty(), "{:?}", snapshot.problems);
    assert!(snapshot.head.is_none() && snapshot.git.is_none());
    let roadmap = snapshot.current().unwrap();
    assert_eq!(roadmap.profile(), Some(Profile::House));
    assert_eq!(roadmap.counts(), counts(3, 6));
    assert_eq!(snapshot.next().unwrap().id, ep(3));
    assert_eq!(
        brief_names(&snapshot),
        [
            "EP-0-plan.md",
            "EP-1-toolchain.md",
            "EP-2-registry.md",
            "EP-3-parser.md"
        ]
    );
    assert_eq!(
        snapshot.files.companions[0].name,
        "EP-1-completion-report.md"
    );
    assert!(snapshot.files.duplicates.is_empty());

    let plan = &snapshot.entry(ep(0)).unwrap().brief;
    let front = plan.front.as_ref().unwrap();
    assert_eq!((front.ep(), front.size()), (Some(ep(0)), Some(Size::L)));
    assert_eq!(front.model(), Some("claude-fable-5-1"));
    assert_eq!(front.status(), Some("done"));
    assert!(matches!(
        plan.completion(),
        Some(Evidence::CompletionNote { date: Some(ref d), line: 18 }) if d == "2026-10-01"
    ));
    let note = plan.completion_notes().next().unwrap();
    assert_eq!(
        note.kind.to_string(),
        "completion note (fable/xhigh; ≈ 2 h 40 min from the survey to the documents)"
    );
    assert_eq!(
        plan.handed_on().next().unwrap().kind,
        Kind::HandedOn {
            to: Some("the owner".to_owned())
        }
    );
    assert_eq!(plan.blocks(), [ep(1), ep(2), ep(3), ep(4), ep(5)]);

    let toolchain = &snapshot.entry(ep(1)).unwrap().brief;
    assert_eq!(toolchain.completion_notes().count(), 1);
    assert!(toolchain.quotes.iter().any(|q| q.kind == Kind::Parked));
    assert!(
        toolchain
            .completion_notes()
            .next()
            .unwrap()
            .text
            .contains("**After the commit:**"),
        "a bold line of another kind stays inside the note"
    );

    let registry = &snapshot.entry(ep(2)).unwrap().brief;
    let kinds: Vec<String> = registry.quotes.iter().map(|q| q.kind.to_string()).collect();
    assert_eq!(
        kinds,
        [
            "owner decisions",
            "completion note",
            "completion note (EP-5 verification)",
            "handed on to beta",
            "handed on to delta",
            "parked"
        ]
    );

    let parser = snapshot.entry(ep(3)).unwrap();
    assert_eq!(parser.brief.completion(), None);
    assert!(!parser.is_complete());
    assert!(!snapshot.is_complete(ep(3)));
    assert!(snapshot.is_complete(ep(2)));
    assert_eq!(
        snapshot.in_progress(),
        [InProgress::BriefWithoutCompletion(ep(3))]
    );
    assert_eq!(facts(&snapshot), (Some(false), Some(false)));

    let decisions = &snapshot.decisions;
    assert_eq!(decisions.path, Some(fixture("house").join("DECISIONS.md")));
    assert!(!decisions.nested);
    assert_eq!(decisions.log.highest(), Some(3));
    assert_eq!(decisions.log.addenda.len(), 2);
    assert_eq!(decisions.log.last_addendum(), Some("2026-10-05"));
    assert_eq!(decisions.adr_files, 0);
}

#[test]
fn house5_every_row_checked_in_three_completion_forms() {
    let snapshot = snapshot::take(&fixture("house5"));
    assert!(snapshot.problems.is_empty(), "{:?}", snapshot.problems);
    let roadmap = snapshot.current().unwrap();
    assert_eq!(roadmap.profile(), Some(Profile::House5));
    assert_eq!(roadmap.counts(), counts(4, 4));
    assert_eq!(snapshot.next(), None);
    assert_eq!(facts(&snapshot), (Some(true), Some(false)));
    assert!(snapshot.in_progress().is_empty());

    let evidence = |n: u32| snapshot.entry(ep(n)).unwrap().brief.completion();
    assert!(matches!(
        evidence(1),
        Some(Evidence::CompletionHeading { line: 9, .. })
    ));
    assert!(
        matches!(evidence(2), Some(Evidence::Status { ref value }) if value.starts_with("DONE"))
    );
    assert!(matches!(
        evidence(3),
        Some(Evidence::Complete { line: 9, .. })
    ));
    // A stale status line on a checked row: the row wins for the project, the brief alone says no.
    assert_eq!(evidence(0), None);
    assert!(snapshot.is_complete(ep(0)));
    assert!(!snapshot.entry(ep(0)).unwrap().is_complete());
    assert_eq!(
        snapshot.files.companions[0].name,
        "EP-1-completion-handoff.md"
    );
    assert!(
        snapshot.decisions.is_empty(),
        "decisions inline in the README are not a log"
    );
}

#[test]
fn packet_a_row_in_progress_and_two_handoff_forms() {
    let snapshot = snapshot::take(&fixture("packet"));
    assert!(snapshot.problems.is_empty(), "{:?}", snapshot.problems);
    let roadmap = snapshot.current().unwrap();
    assert_eq!(roadmap.profile(), Some(Profile::Packet));
    assert_eq!(roadmap.counts(), counts(2, 5));
    assert_eq!(snapshot.next().unwrap().id, ep(3));
    let names = brief_names(&snapshot);
    assert_eq!(names.len(), 6, "{names:?}");
    assert!(
        names.contains(&"EP-4-spine.md"),
        "the split set's parent file"
    );
    assert_eq!(snapshot.in_progress(), [InProgress::RowInProgress(ep(3))]);
    assert_eq!(facts(&snapshot), (Some(false), Some(false)));

    let governance = snapshot.entry(ep(1)).unwrap();
    assert!(matches!(
        governance.brief.completion(),
        Some(Evidence::Status { ref value }) if value == "[x] 102af00"
    ));
    assert!(governance.brief.handoff.as_ref().unwrap().filled);
    assert_eq!(governance.handoff, None);

    let scaffold = snapshot.entry(ep(2)).unwrap();
    assert_eq!(scaffold.brief.completion(), None, "the payload is unfilled");
    let handoff = scaffold.handoff.as_ref().unwrap();
    assert_eq!(handoff.date(), Some("2026-08-24"));
    assert_eq!(handoff.commits(), ["9bcb7b2", "0c1d2e3"]);
    assert!(handoff.is_complete());
    assert!(scaffold.is_complete(), "complete by the handoff file");

    let ingest = snapshot.entry(ep(3)).unwrap();
    assert_eq!(ingest.brief.status(), Some("[~]"));
    assert!(!ingest.is_complete());
    let parent = snapshot.files.brief(ep(4)).unwrap();
    assert_eq!(parent.name, "EP-4-spine.md");
    assert!(roadmap.row(ep(4)).is_none());
    let split = snapshot.entry(ep(4)).unwrap();
    assert!(
        split
            .brief
            .status()
            .unwrap()
            .starts_with("split 2026-08-25")
    );
    assert!(!split.is_complete());

    assert_eq!(snapshot.decisions.path, None);
    assert_eq!(snapshot.decisions.adr_files, 2);
    assert!(!snapshot.decisions.is_empty());
}

#[test]
fn prewritten_briefs_mean_nothing_and_the_gate_is_a_fact() {
    let snapshot = snapshot::take(&fixture("prewritten"));
    assert!(snapshot.problems.is_empty(), "{:?}", snapshot.problems);
    let roadmap = snapshot.current().unwrap();
    assert_eq!(roadmap.profile(), Some(Profile::House));
    // The `◐` row is neither done nor the next EP's obstacle here.
    assert_eq!(roadmap.counts(), counts(3, 6));
    assert_eq!(snapshot.next().unwrap().id, ep(3));
    assert_eq!(brief_names(&snapshot).len(), 6);
    assert!(
        snapshot.in_progress().is_empty(),
        "{:?}",
        snapshot.in_progress()
    );
    assert_eq!(facts(&snapshot), (Some(false), Some(true)));
    assert!(snapshot.pickup_gate());

    let registry = &snapshot.entry(ep(3)).unwrap().brief;
    let labels: Vec<_> = registry.fields.iter().map(|f| f.label.as_str()).collect();
    assert_eq!(
        labels,
        ["Size", "Mode", "Core/Stretch", "Depends on", "Blocks"]
    );
    assert_eq!(registry.size(), Some(Size::M));
    assert_eq!(registry.depends(), [ep(2)]);
    let kinds: Vec<_> = registry.quotes.iter().map(|q| q.kind.clone()).collect();
    assert_eq!(
        kinds,
        [Kind::PickupGate, Kind::PickupNote { ep: Some(ep(2)) }]
    );
    assert_eq!(
        registry.pickup_gate().unwrap().date.as_deref(),
        Some("2026-09-01")
    );
    let engine = &snapshot.entry(ep(4)).unwrap().brief;
    assert_eq!(engine.quotes[0].kind, Kind::Charter);
    assert!(snapshot.entry(ep(5)).unwrap().is_complete());

    let decisions = &snapshot.decisions;
    assert_eq!(
        decisions.path,
        Some(fixture("prewritten").join("pkg").join("DECISIONS.md"))
    );
    assert!(decisions.nested);
    assert_eq!(decisions.log.entries.len(), 3);
    assert_eq!(decisions.log.index_rows, 3);
    let sources: Vec<_> = decisions
        .log
        .addenda
        .iter()
        .map(|a| (a.date.as_deref(), a.source.as_deref()))
        .collect();
    assert_eq!(
        sources,
        [
            (Some("2026-09-01"), Some("EP-2 close-out")),
            (Some("2026-09-02"), None)
        ]
    );
}

/// This repository's own roadmap folder reads whole: a brief for every checked row, each with
/// front matter and a completion note, and the decision log at the root.
#[test]
fn this_repository_reads_whole() {
    let snapshot = snapshot::read_tree(&crw_core::paths::repository());
    assert!(snapshot.problems.is_empty(), "{:?}", snapshot.problems);
    let roadmap = snapshot.current().unwrap();
    for row in roadmap.rows().filter(|row| row.done.is_checked()) {
        let entry = snapshot
            .entry(row.id)
            .unwrap_or_else(|| panic!("{} has no brief", row.id));
        assert!(entry.brief.front.is_some(), "{}", row.id);
        assert!(entry.brief.is_complete(), "{}", row.id);
    }
    assert!(snapshot.files.duplicates.is_empty());
    assert!(!snapshot.decisions.nested);
    assert!(snapshot.decisions.log.highest() >= Some(30));
}

// --- A scratch repository -----------------------------------------------------------------------

struct Repo(PathBuf);

impl Repo {
    fn new() -> Self {
        let dir =
            std::env::temp_dir().join(format!("crw-test-{}-snapshot-git", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("roadmap")).unwrap();
        let repo = Self(dir);
        repo.git(&["init", "-q"]);
        repo
    }

    /// Runs git in the repository with a neutral identity and no signing or line-ending
    /// conversion, spaced like every child the tool starts (D-29).
    fn git(&self, args: &[&str]) -> String {
        thread::sleep(process::SPACING);
        let mut all = vec![
            "-C",
            self.0.to_str().unwrap(),
            "-c",
            "user.name=crw-test",
            "-c",
            "user.email=crw-test@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.autocrlf=false",
        ];
        all.extend_from_slice(args);
        process::output_of(Path::new("git"), all, process::TIMEOUT)
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"))
    }

    fn write(&self, relative: &str, text: &str) {
        fs::write(self.0.join(relative), text).unwrap();
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
    }

    fn take(&self) -> Snapshot {
        thread::sleep(process::SPACING);
        snapshot::take(&self.0)
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const README: &str = "\
# scratch — Roadmap

## Phase P1 — Build (planned 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | First | S | — | core | ☐ |
| EP-2 | Second | S | EP-1 | core | ☐ |
";

#[test]
fn a_working_tree_moves_away_from_head_and_back() {
    let repo = Repo::new();
    repo.write("roadmap/README.md", README);
    repo.commit("Planning: the roadmap (EP-0)");

    // Clean: HEAD and the tree read alike, nothing in progress.
    let clean = repo.take();
    assert!(clean.head.is_some() && clean.tree.is_some());
    assert!(!clean.differs());
    assert!(clean.git.as_ref().unwrap().is_clean());
    assert!(clean.in_progress().is_empty());
    assert_eq!(clean.next().unwrap().id, ep(1));

    // A session writes the brief: untracked, and the only open row with a brief.
    repo.write(
        "roadmap/EP-1-first.md",
        "---\nep: 1\nstatus: in progress\n---\n\n# EP-1 — First\n\n## Context\n\nStarted.\n",
    );
    let started = repo.take();
    assert_eq!(
        started.in_progress(),
        [InProgress::UntrackedBrief("EP-1-first.md".to_owned()),]
    );
    assert!(!started.differs());
    // Committed before its note, with a clean tree: a pre-written brief, not a session.
    repo.commit("First: the brief (EP-1)");
    assert!(repo.take().in_progress().is_empty());

    // The session ticks the row: HEAD still says EP-1 is next, the tree says EP-2.
    repo.write(
        "roadmap/README.md",
        &README.replace(
            "| EP-1 | First | S | — | core | ☐ |",
            "| EP-1 | First | S | — | core | ☑ `0123abc` |",
        ),
    );
    let ticked = repo.take();
    assert!(ticked.differs());
    assert_eq!(ticked.head.as_ref().unwrap().next().unwrap().id, ep(1));
    assert_eq!(ticked.next().unwrap().id, ep(2));
    assert_eq!(ticked.in_progress(), [InProgress::ReadmeDirty]);
    let mut facts = Facts::default();
    ticked.fill(&mut facts);
    assert_eq!(facts.all_rows_checked, Some(false));

    // Committed without a completion note: the row is checked, so nothing is in progress.
    repo.commit("First: done (EP-1)");
    let committed = repo.take();
    assert!(!committed.differs());
    assert!(committed.git.as_ref().unwrap().is_clean());
    assert!(committed.in_progress().is_empty());
    assert!(committed.is_complete(ep(1)));
    assert!(!committed.entry(ep(1)).unwrap().is_complete());

    // A later session appends the note: the brief is modified until it commits.
    repo.write(
        "roadmap/EP-1-first.md",
        "---\nep: 1\nstatus: done\n---\n\n# EP-1 — First\n\n## Context\n\nStarted.\n\n> **Completion note (2026-10-02).** Done.\n",
    );
    let appended = repo.take();
    assert_eq!(
        appended.in_progress(),
        [InProgress::ModifiedBrief("EP-1-first.md".to_owned())]
    );
    assert!(appended.entry(ep(1)).unwrap().is_complete());
    assert_eq!(
        roadmap::readme(&repo.0),
        repo.0.join("roadmap").join("README.md")
    );
}
