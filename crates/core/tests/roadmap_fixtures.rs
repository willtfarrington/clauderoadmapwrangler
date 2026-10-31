//! The roadmap parser against one synthetic roadmap per variant the survey of the owner's roadmaps
//! found (D-9, D-29): invented project names and titles in the shapes those roadmaps take. Each
//! fixture is read with LF and with CRLF line endings, which must read alike.

use crw_core::roadmap::table::{self, Counts, Done, Profile, Roadmap, Track};
use crw_core::roadmap::{EpId, Size};

fn ep(number: u32) -> EpId {
    EpId::new(number)
}

fn split(number: u32, suffix: char) -> EpId {
    EpId {
        number,
        suffix: Some(suffix),
    }
}

/// Reads a fixture with LF and with CRLF line endings (a Windows checkout may hold either), checks
/// that both read alike, and returns the LF reading.
fn parse(text: &str) -> Roadmap {
    let lf = text.replace("\r\n", "\n");
    let roadmap = table::parse(&lf);
    let crlf = table::parse(&lf.replace('\n', "\r\n"));
    assert_eq!(summary(&crlf), summary(&roadmap), "CRLF reads differently");
    roadmap
}

/// Everything but the byte ranges, which CRLF shifts.
fn summary(roadmap: &Roadmap) -> String {
    let phases: Vec<_> = roadmap
        .phases
        .iter()
        .map(|phase| {
            let rows: Vec<_> = phase
                .rows
                .iter()
                .map(|row| {
                    let cells = (&row.brief, row.size, &row.size_text, &row.depends);
                    (row.id, cells, row.track, &row.done, row.line)
                })
                .collect();
            (&phase.heading, phase.profile, rows)
        })
        .collect();
    format!("{phases:?} {:?} {:?}", roadmap.notes, roadmap.problems)
}

fn no_problems(roadmap: &Roadmap) {
    assert!(roadmap.problems.is_empty(), "{:?}", roadmap.problems);
}

fn ids(roadmap: &Roadmap) -> Vec<String> {
    roadmap.rows().map(|row| row.id.to_string()).collect()
}

/// Each phase's id, name, status and number of rows.
fn phases(roadmap: &Roadmap) -> Vec<(Option<&str>, &str, Option<&str>, usize)> {
    roadmap
        .phases
        .iter()
        .map(|phase| {
            let heading = &phase.heading;
            let (id, status) = (heading.id.as_deref(), heading.status.as_deref());
            (id, heading.name.as_str(), status, phase.rows.len())
        })
        .collect()
}

fn counts(done: usize, total: usize, stretch: usize) -> Counts {
    Counts {
        done,
        total,
        stretch,
    }
}

fn hashes(hashes: &[&str]) -> Done {
    Done::Checked(hashes.iter().map(|hash| (*hash).to_owned()).collect())
}

#[test]
fn alpha_the_house_style() {
    let roadmap = parse(include_str!("../fixtures/alpha.md"));
    no_problems(&roadmap);
    assert_eq!(roadmap.profile(), Some(Profile::House));
    assert_eq!(
        phases(&roadmap),
        [
            (Some("P0"), "Planning", Some("done 2026-10-01"), 1),
            (
                Some("P1"),
                "Foundations and the read-only view",
                Some("planned 2026-10-01"),
                5
            ),
            (Some("P2"), "Sessions", Some("planned 2026-10-01"), 3),
            (
                Some("P3"),
                "Phone view",
                Some("charter briefs; upgraded at EP-8"),
                2
            ),
        ]
    );
    assert_eq!(roadmap.counts(), counts(3, 11, 0));
    // The examples in a code block, a blockquote and a comment are not rows.
    assert!(roadmap.rows().all(|row| row.id.number < 90));

    let next = roadmap.next().unwrap();
    assert_eq!((next.id, next.size), (ep(3), Some(Size::M)));
    assert_eq!(next.depends.ids, [ep(1)]);
    assert_eq!(
        next.brief.title,
        "Gizmo parser: tables, rows, sizes, `☑` cells of every form; next EP in table order"
    );
    assert_eq!(next.brief.link, None);
    assert_eq!(next.brief.decisions, [9]);
    assert_eq!(
        roadmap.note(next.id).unwrap().text,
        "**EP-3.** `core::gizmo`: find the tables by their header row; rows are single lines; \
         cells:\nid, title, size, depends-on, core, done.\n- a nested list item stays with its \
         bullet\n- so does this one"
    );

    let plan = roadmap.row(ep(0)).unwrap();
    assert_eq!(
        plan.brief.title,
        "The planning interview: 10 rounds, 40 questions; D-1 … D-12; phases P1 … P3"
    );
    assert_eq!(
        plan.brief.link.as_deref(),
        Some("EP-0-planning-interview.md")
    );
    assert_eq!(plan.brief.decisions, (1..=12).collect::<Vec<_>>());
    let toolchain = roadmap.row(ep(1)).unwrap();
    assert_eq!(
        toolchain.brief.title,
        "Toolchain bootstrap: workspace (`core`, `app`, `cli`), pins, gate, CI"
    );
    assert_eq!(toolchain.brief.decisions, [2, 5]);
    assert_eq!(roadmap.row(ep(2)).unwrap().brief.decisions, [6, 7, 8]);
    assert_eq!(
        roadmap.row(ep(5)).unwrap().depends.ids,
        [ep(1), ep(2), ep(3), ep(4)]
    );
    // Every brief after the plan has notes; the charter phase's come under its own label.
    assert_eq!(roadmap.notes.len(), 10);
    assert!(
        roadmap
            .rows()
            .filter(|row| row.id != ep(0))
            .all(|row| roadmap.note(row.id).is_some())
    );
    assert_eq!(
        roadmap.note(ep(10)).unwrap().text,
        "**EP-10.** Timings for every EP, the README's state of the app."
    );
}

#[test]
fn beta_decisions_inside_the_link_and_rows_linked_at_completion() {
    let roadmap = parse(include_str!("../fixtures/beta.md"));
    no_problems(&roadmap);
    assert_eq!(
        phases(&roadmap),
        [
            (Some("P0"), "The v1 plan", Some("planned 2026-10-01"), 1),
            (
                Some("P1"),
                "Foundation: client, checklist, scope",
                Some("planned 2026-10-01"),
                4
            ),
            (Some("P2"), "The review loop", Some("planned 2026-10-01"), 5),
        ]
    );
    assert_eq!(roadmap.counts(), counts(5, 10, 2));
    let next = roadmap.next().unwrap();
    assert_eq!(next.id, ep(5));
    assert_eq!(next.brief.link, None, "linked only at completion");
    assert!(
        roadmap
            .note(ep(5))
            .unwrap()
            .text
            .ends_with("**From EP-4:** the client's run log names each request.")
    );
    let toolchain = roadmap.row(ep(1)).unwrap();
    assert_eq!(
        toolchain.brief.title,
        "Toolchain bootstrap: uv project, pinned dependencies, `poe check`"
    );
    assert_eq!(toolchain.brief.decisions, [3, 15]);
    let client = roadmap.row(ep(2)).unwrap();
    assert_eq!(client.size, Some(Size::L));
    assert_eq!(client.size_text, "S → L (re-scoped 2026-10-02)");
    assert_eq!(client.done, hashes(&["f6a7b8c", "0a1b2c3"]));
    assert_eq!(
        roadmap.row(ep(4)).unwrap().depends.ids,
        [ep(1), ep(2), ep(3)]
    );
    assert_eq!(
        roadmap.row(ep(9)).unwrap().depends.ids,
        [ep(5), ep(6), ep(7), ep(8)]
    );
    let stretch: Vec<_> = roadmap
        .rows()
        .filter(|row| row.track == Some(Track::Stretch))
        .map(|row| row.id)
        .collect();
    assert_eq!(stretch, [ep(7), ep(8)]);
}

#[test]
fn gamma_a_re_cut_order_label_variants_and_a_phase_without_a_table() {
    let text = include_str!("../fixtures/gamma.md");
    let roadmap = parse(text);
    no_problems(&roadmap);
    assert_eq!(
        phases(&roadmap),
        [
            (
                Some("P0"),
                "Inventory design",
                Some("planned 2026-10-01; done 2026-10-03"),
                5
            ),
            (
                Some("P1"),
                "The inventory built: shelves, labels, reports",
                Some("planned 2026-10-01; re-cut at EP-4, 2026-10-03"),
                5
            ),
            (
                Some("P2"),
                "Integrations",
                Some("charter briefs; planned 2026-10-01"),
                3
            ),
            (
                Some("P3"),
                "Garage ingestion",
                Some("placeholder; not planned"),
                0
            ),
        ]
    );
    assert_eq!(roadmap.phases[3].profile, None);
    assert_eq!(roadmap.counts(), counts(6, 13, 1));
    assert_eq!(
        ids(&roadmap)[5..10],
        ["EP-5", "EP-6", "EP-8", "EP-7", "EP-9"]
    );
    assert_eq!(roadmap.next().unwrap().id, ep(6));
    // With EP-6 checked the next EP is EP-8, which the re-cut put before EP-7.
    let checked = text.replace(
        "(D-12) | M | EP-5 | core | ☐ |",
        "(D-12) | M | EP-5 | core | ☑ `9c0d1e2` |",
    );
    assert_eq!(table::parse(&checked).next().unwrap().id, ep(8));
    assert_eq!(roadmap.row(ep(8)).unwrap().depends.ids, [ep(6), ep(4)]);
    assert_eq!(
        roadmap.note(ep(8)).unwrap().text,
        "**EP-8** (runs right after EP-6, EP-4). The weekly template; the Monday report links \
         the\nweek's open items."
    );
    assert_eq!(
        roadmap.note(ep(7)).unwrap().text,
        "**EP-7** (runs after EP-8, EP-4). `gamma.items`: read and split the header with the\n\
         description kept byte for byte."
    );
    // The bullets under `**Charters.** Each is …`, a label with text on its own line.
    assert!(roadmap.note(ep(12)).is_some());
    assert_eq!(roadmap.notes.len(), 7);
}

#[test]
fn delta_prose_scope_sketches_and_phases_without_a_status() {
    let roadmap = parse(include_str!("../fixtures/delta.md"));
    no_problems(&roadmap);
    assert_eq!(
        phases(&roadmap),
        [
            (
                Some("P0"),
                "Foundations: hygiene, toolchain, test instance",
                None,
                4
            ),
            (
                Some("P1"),
                "Garden: beds, plantings, harvest log",
                Some("charter"),
                3
            ),
        ]
    );
    assert_eq!(roadmap.counts(), counts(1, 7, 0));
    assert_eq!(roadmap.next().unwrap().id, ep(1));
    let notes: Vec<_> = roadmap
        .notes
        .iter()
        .map(|note| (note.id.number, note.line, note.phase))
        .collect();
    assert_eq!(
        notes,
        [
            (1, 21, Some(0)),
            (2, 22, Some(0)),
            (3, 23, Some(0)),
            (4, 33, Some(1)),
            (5, 35, Some(1)),
            (6, 36, Some(1)),
        ]
    );
    // `EP-3:` inside the paragraph names another phase's row, so it does not start a segment.
    assert_eq!(
        roadmap.note(ep(4)).unwrap().text,
        "EP-4: a bed per row typed from the sketch, filed under\n`beds/<name>/`, as in EP-3: the \
         archive keeps it; frost dates show on the calendar panel (EP-2)."
    );
    assert_eq!(
        roadmap.note(ep(6)).unwrap().text,
        "EP-6: retro, addenda, the P2 charters upgraded."
    );
    assert_eq!(roadmap.row(ep(6)).unwrap().depends.ids, [ep(4), ep(5)]);
}

#[test]
fn epsilon_every_row_checked_unlinked_rows_and_the_longest_row() {
    let roadmap = parse(include_str!("../fixtures/epsilon.md"));
    no_problems(&roadmap);
    assert_eq!(
        phases(&roadmap),
        [
            (
                Some("P0"),
                "Import and tagging charter",
                Some("done 2026-10-01"),
                2
            ),
            (Some("P1"), "Protect, then sort", Some("done 2026-10-02"), 3),
            (
                Some("P2"),
                "Triage",
                Some("session list; refined at EP-4; done 2026-10-03"),
                4
            ),
        ]
    );
    assert_eq!(roadmap.next(), None);
    assert_eq!(roadmap.counts(), counts(9, 9, 0));
    assert!(roadmap.counts().all_checked());
    assert!(roadmap.rows().all(|row| row.brief.link.is_none()));
    assert_eq!(ids(&roadmap)[5..], ["EP-5", "EP-6", "EP-14", "EP-7"]);
    let long = roadmap.row(ep(5)).unwrap();
    assert!(long.span.len() > 860, "{}", long.span.len());
    assert_eq!(long.brief.decisions, [16, 17]);
    assert!(long.brief.title.ends_with("one shelf at a time"));
    let import = roadmap.row(ep(0)).unwrap();
    assert_eq!(import.brief.title, "Recipe import + manifest runner");
    assert_eq!(import.brief.decisions, (1..=7).collect::<Vec<_>>());
}

#[test]
fn zeta_glossaries_inserted_rows_eight_hashes_and_two_digit_phases() {
    let roadmap = parse(include_str!("../fixtures/zeta.md"));
    no_problems(&roadmap);
    let planned = |words: &str| format!("{words}; planned 2026-08-16");
    let statuses = [
        planned("template step 1; full briefs"),
        planned("template steps 2–3; full briefs"),
        planned("template step 4a; full briefs"),
        planned("template step 7d; charter briefs"),
        planned("template step 9; charter briefs"),
    ];
    assert_eq!(
        phases(&roadmap),
        [
            (
                Some("P0"),
                "Charter, rules & toolchain",
                Some(statuses[0].as_str()),
                3
            ),
            (
                Some("P1"),
                "Inventory & knowledge pack",
                Some(statuses[1].as_str()),
                4
            ),
            (
                Some("P2"),
                "Staging: lake, runner, catalog",
                Some(statuses[2].as_str()),
                3
            ),
            (
                Some("P10"),
                "Free text (optional; gated at EP-60; stretch track)",
                Some(statuses[3].as_str()),
                2
            ),
            (
                Some("P11"),
                "Showcase & release",
                Some(statuses[4].as_str()),
                2
            ),
        ]
    );
    assert_eq!(roadmap.counts(), counts(8, 14, 2));
    assert_eq!(
        roadmap.next().unwrap().id,
        ep(5),
        "EP-92 before it is checked"
    );
    assert!(roadmap.notes.is_empty(), "every brief is pre-written");
    let Done::Checked(eight) = &roadmap.row(ep(2)).unwrap().done else {
        panic!("EP-2 is checked");
    };
    assert_eq!(eight.len(), 8);
    assert_eq!(
        roadmap.row(ep(3)).unwrap().brief.title,
        "File inventory manifest ⏱"
    );
    let closeout = roadmap.row(ep(63)).unwrap();
    assert_eq!(closeout.depends.text, "all UI EPs");
    assert!(closeout.depends.ids.is_empty());
}

#[test]
fn eta_five_columns_numeric_phases_and_annotated_dependencies() {
    let roadmap = parse(include_str!("../fixtures/eta.md"));
    no_problems(&roadmap);
    assert_eq!(roadmap.profile(), Some(Profile::House5));
    assert_eq!(
        phases(&roadmap),
        [
            (Some("1"), "single-yard refit", Some("complete"), 4),
            (
                Some("2"),
                "dashboards & flow",
                Some("planned 2026-08-12"),
                8
            ),
        ]
    );
    assert_eq!(roadmap.counts(), counts(11, 12, 0));
    assert!(roadmap.rows().all(|row| row.track.is_none()));
    assert_eq!(
        ids(&roadmap)[4..],
        [
            "EP-5", "EP-6", "EP-7", "EP-4", "EP-10", "EP-9", "EP-11", "EP-8"
        ]
    );
    let next = roadmap.next().unwrap();
    assert_eq!(
        (next.id, next.size, next.size_text.as_str()),
        (ep(8), Some(Size::M), "S/M")
    );
    let rulings = roadmap.row(ep(10)).unwrap();
    assert_eq!(rulings.size, Some(Size::M));
    assert_eq!(rulings.size_text, "S→M (owner escalated retirements)");
    assert_eq!(
        rulings.depends.ids,
        [ep(5)],
        "run before EP-11/EP-9 is not a dependency"
    );
    assert_eq!(roadmap.row(ep(3)).unwrap().depends.ids, [ep(1)]);
    assert_eq!(
        roadmap.row(ep(6)).unwrap().done,
        hashes(&["1a2b3c4", "2b3c4d5"])
    );
}

#[test]
fn theta_release_headings_bare_hashes_and_pre_written_briefs() {
    let roadmap = parse(include_str!("../fixtures/theta.md"));
    no_problems(&roadmap);
    assert_eq!(roadmap.profile(), Some(Profile::House5));
    assert_eq!(
        phases(&roadmap),
        [
            (Some("R0"), "Foundation & governance", None, 3),
            (Some("R1"), "Vertical slice → v0.1", None, 4),
            (Some("R2"), "Depth & breadth", None, 2),
            (Some("R3"), "Hardening → v1.0", None, 1),
        ]
    );
    assert_eq!(roadmap.counts(), counts(4, 10, 0));
    assert_eq!(roadmap.next().unwrap().id, ep(5));
    assert_eq!(roadmap.row(ep(1)).unwrap().done, hashes(&["0a1b2c3"]));
    assert_eq!(
        roadmap.row(ep(3)).unwrap().done,
        hashes(&["2c3d4e5", "3d4e5f6"])
    );
    assert_eq!(roadmap.row(ep(9)).unwrap().depends.ids, [ep(20)]);
    assert!(roadmap.row(ep(10)).unwrap().depends.ids.is_empty());
    assert!(roadmap.notes.is_empty());
}

#[test]
fn iota_bare_hashes_a_held_row_and_a_file_table() {
    let roadmap = parse(include_str!("../fixtures/iota.md"));
    no_problems(&roadmap);
    assert_eq!(
        phases(&roadmap),
        [
            (
                Some("P0"),
                "Foundation and governance",
                Some("full briefs"),
                4
            ),
            (Some("P1"), "Content model", Some("full briefs"), 5),
            (
                Some("P2"),
                "Evaluation",
                Some("charter briefs; upgraded by EP-8, the P1 re-plan"),
                2
            ),
        ]
    );
    // `◐` is done but not landed: not counted as done, and the next EP until it lands.
    assert_eq!(roadmap.counts(), counts(6, 11, 1));
    let next = roadmap.next().unwrap();
    assert_eq!((next.id, &next.done), (ep(6), &Done::NotLanded));
    assert_eq!(
        roadmap.row(ep(1)).unwrap().done,
        hashes(&["4b4b4b4", "5c5c5c5"])
    );
    assert!(roadmap.notes.is_empty());
}

#[test]
fn kappa_the_packet_style() {
    let roadmap = parse(include_str!("../fixtures/kappa.md"));
    no_problems(&roadmap);
    assert_eq!(roadmap.profile(), Some(Profile::Packet));
    assert_eq!(
        phases(&roadmap),
        [
            (Some("M0"), "Governance bootstrap", Some("[x] 1f1f1f1"), 2),
            (
                Some("M1"),
                "Pipeline skeleton + fixture",
                Some("[x] 4c4c4c4"),
                3
            ),
            (None, "Checkpoints", Some("recurring"), 2),
            (
                Some("M2"),
                "Base layer + first source",
                Some("[x] 9a9a9a9"),
                4
            ),
            (Some("M3"), "Route spike", Some("[x] 2d2d2d2"), 3),
            (Some("M4–M8"), "", Some("[ ] refinement gates pending"), 0),
        ]
    );
    let marks: Vec<_> = roadmap
        .phases
        .iter()
        .map(|phase| phase.heading.mark.clone())
        .collect();
    assert_eq!(marks[0], Some(hashes(&["1f1f1f1"])));
    assert_eq!(marks[2], None);
    assert_eq!(marks[5], Some(Done::Open));
    assert_eq!(roadmap.counts(), counts(13, 14, 0));
    // The checkpoint table comes before M2 and M3 in the document, so its open packet is next.
    let next = roadmap.next().unwrap();
    assert_eq!((next.id, &next.done), (ep(12), &Done::Open));
    assert_eq!(next.depends.ids, [ep(11)]);
    assert_eq!(
        roadmap.row(split(5, 'a')).unwrap().depends.ids,
        [ep(8), split(4, 'b')]
    );
    assert_eq!(roadmap.row(ep(6)).unwrap().depends.ids, [ep(5)]);
    assert_eq!(
        roadmap.row(split(4, 'b')).unwrap().brief.link.as_deref(),
        Some("EP-4b-stage-runner.md")
    );
    assert!(roadmap.rows().all(|row| row.track.is_none()));
    assert!(roadmap.notes.is_empty());
}

/// This repository's own roadmap (D-5): read without a problem, and every brief not yet done has
/// its brief notes (D-12).
#[test]
fn this_repositorys_roadmap() {
    let roadmap = parse(include_str!("../../../roadmap/README.md"));
    no_problems(&roadmap);
    assert_eq!(roadmap.profile(), Some(Profile::House));
    assert!(roadmap.counts().total >= 33);
    assert!(roadmap.row(ep(0)).unwrap().done.is_checked());
    for row in roadmap.rows().filter(|row| !row.done.is_checked()) {
        assert!(
            roadmap.note(row.id).is_some(),
            "{} has no brief notes",
            row.id
        );
    }
}
