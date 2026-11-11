//! The owner tasks, the inbox, the handed-on blocks and the older forms of the owner's to-dos
//! against the four synthetic project folders (D-10, D-13), and the ticks against scratch copies
//! of them: a fixture itself is never written.

use std::fs;
use std::path::{Path, PathBuf};

use crw_core::checklist;
use crw_core::inbox::{self, Recipient};
use crw_core::roadmap::EpId;
use crw_core::roadmap::snapshot::{self, Snapshot};
use crw_core::tasks::{self, Form, Gate, OwnerItem, State};

fn ep(number: u32) -> EpId {
    EpId::new(number)
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("projects")
        .join(name)
}

/// Each item's form, id, state and line, and the file it is in by name.
fn summary(items: &[OwnerItem]) -> Vec<(Form, Option<&str>, State, &str, usize)> {
    items
        .iter()
        .map(|item| {
            let file = item.path.file_name().unwrap().to_str().unwrap();
            (item.form, item.id.as_deref(), item.state, file, item.line)
        })
        .collect()
}

fn take(name: &str) -> Snapshot {
    let snapshot = snapshot::read_tree(&fixture(name));
    assert!(snapshot.problems.is_empty(), "{:?}", snapshot.problems);
    snapshot
}

#[test]
fn house_owner_tasks_inbox_owner_actions_risks_and_the_latest_briefs_blocks() {
    let snapshot = take("house");
    let tasks = snapshot.tasks.as_ref().unwrap();
    assert_eq!(
        tasks.path,
        fixture("house").join("roadmap").join("owner-tasks.md")
    );
    let problems: Vec<String> = tasks.problems.iter().map(ToString::to_string).collect();
    assert_eq!(problems, ["line 10: a box without an OT-n id"]);
    assert_eq!(tasks.open().count(), 2);
    let gated = &tasks.tasks[1];
    assert_eq!(
        (gated.id.as_str(), gated.raised, gated.gate),
        ("OT-2", Some(ep(1)), Some(ep(3)))
    );
    assert_eq!(
        tasks.tasks[2].text,
        "Look at the backups folder after a week and say whether fourteen\ndaily copies are enough."
    );

    let inbox = snapshot.inbox.as_ref().unwrap();
    assert!(inbox.problems.is_empty(), "{:?}", inbox.problems);
    let messages: Vec<_> = inbox
        .messages
        .iter()
        .map(|m| (m.id.as_str(), m.done, m.sender.as_deref(), m.urgent, m.line))
        .collect();
    assert_eq!(
        messages,
        [
            ("IN-1", true, Some("beta"), false, 9),
            ("IN-2", false, Some("the owner"), false, 10),
            ("IN-3", false, Some("gamma"), true, 12),
            ("IN-4", true, Some("beta"), false, 13),
        ]
    );
    assert!(
        inbox
            .messages
            .iter()
            .filter(|m| m.done)
            .all(|m| m.disposition.is_some())
    );

    let latest = snapshot.latest_complete().unwrap();
    assert_eq!(latest.file.id, ep(2));
    assert_eq!(latest.completed_on().as_deref(), Some("2026-10-03"));
    let items = snapshot.owner_items();
    assert_eq!(
        summary(&items),
        [
            (
                Form::Task,
                Some("OT-1"),
                State::Resolved,
                "owner-tasks.md",
                6
            ),
            (Form::Task, Some("OT-2"), State::Open, "owner-tasks.md", 7),
            (Form::Task, Some("OT-3"), State::Open, "owner-tasks.md", 8),
            (Form::OwnerAction, None, State::Open, "README.md", 8),
            (Form::OwnerAction, None, State::Open, "README.md", 9),
            (Form::OwnerAction, None, State::Resolved, "README.md", 9),
            (Form::OwnerAction, None, State::Open, "README.md", 10),
            (Form::Risk, Some("R-1"), State::Open, "README.md", 36),
            (Form::Risk, Some("R-2"), State::Resolved, "README.md", 37),
            (Form::Risk, Some("R-3"), State::Partly, "README.md", 39),
            // The latest brief's parked block; EP-1's and EP-0's are older and read already.
            (Form::Parked, None, State::Open, "EP-2-registry.md", 26),
        ]
    );
    assert_eq!(items[1].gate, Some(Gate::Before(ep(3))));
    assert_eq!(items[4].gate, Some(Gate::Before(ep(4))));
    assert_eq!(items[10].raised, Some(ep(2)));
    assert_eq!(items.iter().filter(|item| item.is_tickable()).count(), 3);
    assert_eq!(items.iter().filter(|item| item.is_open()).count(), 8);
    // The tasks file is kept, so its lines alone count as waiting; the older forms are listed
    // uncounted (D-10 addendum of 2026-11-11, EP-38).
    assert!(!snapshot.older_forms_wait());
    assert_eq!(summary(&snapshot.waiting()), summary(&items[..3]));
    assert_eq!(snapshot.open_waiting(), 2);
    assert_eq!(summary(snapshot.uncounted()), summary(&items[3..]));

    // EP-0's block to the owner is an owner item, not mail; the registry knows beta only.
    let outgoing = inbox::outgoing(&snapshot.briefs, |name| {
        name.eq_ignore_ascii_case("beta").then(|| "beta".to_owned())
    });
    let mail: Vec<_> = outgoing
        .iter()
        .map(|o| (o.brief, o.line, o.to.as_str(), o.recipient.clone()))
        .collect();
    assert_eq!(
        mail,
        [
            (ep(2), 24, "beta", Recipient::Project("beta".to_owned())),
            (ep(2), 25, "delta", Recipient::Unknown),
        ]
    );
}

#[test]
fn house5_open_items_with_a_date_gate_and_a_numbered_risks_list() {
    let snapshot = take("house5");
    assert_eq!(snapshot.tasks, None);
    assert_eq!(snapshot.inbox, None);
    assert_eq!(snapshot.latest_complete().unwrap().file.id, ep(3));
    let items = snapshot.owner_items();
    assert_eq!(
        summary(&items),
        [
            (Form::OpenItem, None, State::Open, "README.md", 28),
            (Form::OpenItem, None, State::Open, "README.md", 29),
            (Form::OpenItem, None, State::Open, "README.md", 30),
            (Form::Risk, Some("1"), State::Resolved, "README.md", 36),
            (Form::Risk, Some("2"), State::Open, "README.md", 40),
            (Form::Risk, Some("3"), State::Partly, "README.md", 41),
        ]
    );
    assert_eq!(
        items[1].gate,
        Some(Gate::OnOrAfter("2026-11-02".to_owned()))
    );
    assert_eq!(items[0].gate, None);
    // No tasks file: the older forms are what waits, and none is listed uncounted (EP-38).
    assert!(snapshot.older_forms_wait());
    assert_eq!(summary(&snapshot.waiting()), summary(&items));
    assert_eq!(snapshot.open_waiting(), 5);
    assert!(snapshot.uncounted().is_empty());
    assert!(
        items[3].text.contains("- **Bed count**"),
        "a struck item keeps the open items nested under it: {}",
        items[3].text
    );
}

#[test]
fn prewritten_an_owner_decisions_register() {
    let snapshot = take("prewritten");
    assert_eq!(snapshot.latest_complete().unwrap().file.id, ep(5));
    let items = snapshot.owner_items();
    assert_eq!(
        summary(&items),
        [
            (
                Form::OwnerDecision,
                Some("OD-1"),
                State::Resolved,
                "owner-decisions.md",
                10
            ),
            (
                Form::OwnerDecision,
                Some("OD-2"),
                State::Open,
                "owner-decisions.md",
                11
            ),
            (
                Form::OwnerDecision,
                Some("OD-3"),
                State::Open,
                "owner-decisions.md",
                12
            ),
        ]
    );
    let raised: Vec<_> = items.iter().map(|item| item.raised).collect();
    assert_eq!(raised, [Some(ep(1)), Some(ep(2)), Some(ep(2))]);
    assert_eq!(
        items[1].text,
        "Does the authored block match the approved draft?"
    );
}

#[test]
fn packet_open_questions_and_the_latest_handoff_notes() {
    let snapshot = take("packet");
    let latest = snapshot.latest_complete().unwrap();
    assert_eq!(latest.file.id, ep(2), "complete by its handoff file");
    assert_eq!(latest.completed_on().as_deref(), Some("2026-08-24"));
    let items = snapshot.owner_items();
    assert_eq!(
        summary(&items),
        [
            (
                Form::OpenQuestion,
                Some("OQ-1"),
                State::Open,
                "open-questions.md",
                5
            ),
            (
                Form::OpenQuestion,
                Some("OQ-2"),
                State::Resolved,
                "open-questions.md",
                6
            ),
            (
                Form::OpenQuestion,
                Some("OQ-3"),
                State::Open,
                "open-questions.md",
                7
            ),
            (Form::HandoffNote, None, State::Open, "EP-2.md", 12),
            (Form::HandoffNote, None, State::Resolved, "EP-2.md", 13),
        ]
    );
    let gates: Vec<_> = items[..3].iter().map(|item| item.gate.clone()).collect();
    assert_eq!(
        gates,
        [
            Some(Gate::Before("EP-4b".parse().unwrap())),
            None,
            Some(Gate::When("M4 acceptance".to_owned())),
        ]
    );
}

/// This repository's own inbox follows D-13: every ticked message carries its disposition.
#[test]
fn this_repositorys_inbox_and_risks_read() {
    let snapshot = snapshot::read_tree(&crw_core::paths::repository());
    assert!(snapshot.problems.is_empty(), "{:?}", snapshot.problems);
    let inbox = snapshot.inbox.as_ref().unwrap();
    assert!(inbox.problems.is_empty(), "{:?}", inbox.problems);
    assert!(!inbox.messages.is_empty());
    for message in inbox.messages.iter().filter(|m| m.done) {
        assert!(
            message.disposition.is_some(),
            "{} has no disposition",
            message.id
        );
    }
    if let Some(tasks) = &snapshot.tasks {
        assert!(tasks.problems.is_empty(), "{:?}", tasks.problems);
    }
    assert!(
        snapshot
            .owner_items()
            .iter()
            .any(|item| item.form == Form::Risk)
    );
}

// --- Ticks on scratch copies ----------------------------------------------------------------------

struct Scratch(PathBuf);

impl Scratch {
    /// A copy of the house fixture's `roadmap/` folder in a fresh folder named `house`.
    fn house(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("crw-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let roadmap = dir.join("house").join("roadmap");
        fs::create_dir_all(&roadmap).unwrap();
        for file in ["owner-tasks.md", "inbox.md", "README.md"] {
            fs::copy(
                fixture("house").join("roadmap").join(file),
                roadmap.join(file),
            )
            .unwrap();
        }
        Self(dir)
    }

    fn folder(&self) -> PathBuf {
        self.0.join("house")
    }

    fn backups(&self) -> PathBuf {
        self.0.join("backups")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_tick_changes_one_box_and_keeps_a_copy_outside_the_project() {
    let scratch = Scratch::house("owner-tick");
    let folder = scratch.folder();
    let original = fs::read_to_string(tasks::path(&folder)).unwrap();

    let copy = tasks::tick(&folder, "OT-2", &scratch.backups(), "2026-11-01").unwrap();
    assert_eq!(
        copy,
        scratch
            .backups()
            .join("house")
            .join("owner-tasks.2026-11-01.md")
    );
    assert_eq!(fs::read_to_string(&copy).unwrap(), original);
    let ticked = fs::read_to_string(tasks::path(&folder)).unwrap();
    assert_eq!(
        ticked,
        original.replacen("- [ ] OT-2 (EP-1", "- [x] OT-2 (EP-1", 1)
    );
    let mut names: Vec<String> = fs::read_dir(folder.join("roadmap"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["README.md", "inbox.md", "owner-tasks.md"]);

    // Refused ticks write nothing.
    for (id, refusal) in [("OT-1", "ticked already"), ("OT-9", "not in the file")] {
        let error = tasks::tick(&folder, id, &scratch.backups(), "2026-11-01").unwrap_err();
        assert!(error.to_string().contains(refusal), "{error}");
    }
    assert_eq!(fs::read_to_string(tasks::path(&folder)).unwrap(), ticked);

    tasks::untick(&folder, "ot-2", &scratch.backups(), "2026-11-01").unwrap();
    assert_eq!(fs::read_to_string(tasks::path(&folder)).unwrap(), original);
}

#[test]
fn an_inbox_tick_needs_a_disposition_and_writes_it_on_the_line() {
    let scratch = Scratch::house("inbox-tick");
    let folder = scratch.folder();
    let original = fs::read_to_string(inbox::path(&folder)).unwrap();
    assert!(matches!(
        inbox::tick(&folder, "IN-3", "", &scratch.backups(), "2026-11-01"),
        Err(checklist::Error::NoDisposition(_))
    ));
    assert!(!scratch.backups().exists());

    inbox::tick(
        &folder,
        "IN-3",
        "rolled into EP-3",
        &scratch.backups(),
        "2026-11-01",
    )
    .unwrap();
    let text = fs::read_to_string(inbox::path(&folder)).unwrap();
    assert_eq!(
        text,
        original.replacen(
            "- [ ] IN-3 (from gamma EP-7, 2026-10-05) The export gains a column. [urgent]",
            "- [x] IN-3 (from gamma EP-7, 2026-10-05) The export gains a column. [urgent] **Disposition (2026-11-01):** rolled into EP-3",
            1
        )
    );
    let read = inbox::read(&folder).unwrap().unwrap();
    let message = read.messages.iter().find(|m| m.id == "IN-3").unwrap();
    assert!(message.done && message.urgent);
    assert_eq!(message.text, "The export gains a column. [urgent]");
    assert_eq!(
        message.disposition.as_deref(),
        Some("**Disposition (2026-11-01):** rolled into EP-3")
    );
    assert_eq!(read.open().count(), 1);
}
