---
ep: 60a
title: Two fixes from the inbox, the Owner tasks and Inbox panes' This project only switch no longer kept across restarts and said plainly while on (IN-22), and a brief still carrying the template's placeholder completion note no longer read as complete (IN-23)
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-60a — The panes' "This project only" switch not kept; the template's placeholder note not read as complete

**Size:** S · **Depends on:** EP-59 · **Blocks:** — · **Decisions:** D-25, D-9 (and D-3, D-11,
D-12, D-29 as standing)

## Context

Two messages delivered during EP-59 (2026-11-26), placed in this row on the owner's choices
(EP-59's note).

**The panes' switch (IN-22; the D-25 addendum of 2026-11-26).** The Owner tasks pane
(`owner_tasks.rs`, EP-50) and the Inbox pane (`mailbox.rs`, EP-51) each have a "This project
only" switch (`View::tasks_one`, `View::inbox_one`, `board.rs`). The Detail pane's Open in Owner
tasks and Open in Inbox, and an urgent message's Show (`Board::lead`, `Target::Inbox`), turn it on
(`Action::OpenTasks`, `Action::OpenInbox`); the view, which eframe saves with the layout
(`VIEW_KEY`), keeps it across restarts; and while it is on, only a weak "Shown: <project> alone."
at the end of the pane's summary says so. Another project's session found the owner missing that
project's five owner tasks this way. The owner's choice: the switch is not kept, each start
showing every project; the leads select the project and scroll to its group without turning the
switch on; while it is on, the pane's head says plainly how many items of other projects it hides,
with Show all; a saved view that holds it on reads as off.

**The placeholder note (IN-23, read in the code by another project's session).** `kind_of`
(`crates/core/src/roadmap/brief.rs`) reads any label opening "Completion note" as a completion
note, dated or not, and `Brief::completion` returns the first, so a brief that still carries the
template's closing placeholder, `> **Completion note (YYYY-MM-DD; model/effort; duration).**`
(`formalise::TEMPLATE`, which `crw formalise` writes as `roadmap/_TEMPLATE.md`), reads as
complete: the gate's condition that the brief has a completion note (`snapshot::Entry::is_complete`)
would pass, and the Detail pane's History would take the placeholder's "model/effort" and
"duration" as the figures of a later note. **`House::wants`, checked at pickup:** the formalise
plan creates `_TEMPLATE.md` only where it is missing (`Change::Keep` when it is there, never
replaced), and only the rule block is compared byte for byte, so a changed template makes no
formalised project's row offer Formalise… again; the projects formalised so far keep the template
they were given, which the parser fix makes harmless. The plugin's reference quotes the template
byte for byte (`crates/core/tests/plugin.rs`), so it changes with it.

**Session start (2026-11-27, 21:44 local time).** Started from the window (Start, Visual, Open in
VS Code); the prompt is the tool's opener, and `crw sessions clauderoadmapwrangler` lists the row
(`4a7598fa`, visual, EP-60a, open). The launch suggested claude-opus-5-5 at max; the panel runs
claude-opus-5-5 at max, read from the `model` and `effort` fields of this session's own transcript
entries (no text read). Read first: the roadmap's P7 table and notes; `DECISIONS.md`'s D-9 and
D-25 with their latest addenda; EP-60's completion note; the code each item names.
**Owner tasks:** none open. **Inbox:** none open. **Field check** since EP-60's record commit
(`d2e218f`, 21:32):

- `crash.log`: last written 2026-11-23 04:00, nothing new.
- `crw notifications`: the latest twenty, 21:17 to 21:42: turns' ends and waits of visual sessions
  in other projects and two turn ends of EP-60's own session, and one inbox badge for a delivery
  between other projects. Nothing odd.
- `crw sessions`: this row, and EP-60's (`8479e6a2`) still open, its tab not closed.
- `crw autopilot`: not paused; fourteen projects on; the chain log unchanged since 2026-11-26
  22:05.
- The rolling log: EP-60's install at 21:32 asked the window to restart, which it refused (a
  session open with no prompt yet in four projects' tabs), so the window still runs EP-58b's
  build, as EP-60's residual risk (4) foresaw; the re-reads and one delivery between other
  projects; no other warning, no error.

## In scope

1. **The panes' switch** (D-25):
   - `View::tasks_one` and `View::inbox_one` no longer saved: a view an older build saved with
     them on reads with both off, its other fields kept;
   - `Action::OpenTasks` and `Action::OpenInbox` select the project and bring the pane forward,
     leaving the switch as it is, and leave the pane a lead to the project, which it takes at its
     next drawing by scrolling the project's group to its top, once; a project the pane does not
     list (nothing open in it) is named under the summary instead, with what shows it;
   - while the switch is on, a line under the summary in the window's amber says how many open
     tasks (messages, and how many of them urgent) of other projects it hides, with **Show all**,
     which turns it off; with no project selected, every one is hidden;
   - the switches' and the leads' hover texts say so;
   - tests on a bare context: the switch after a saved view, each lead's scroll (the group's name
     drawn at the pane's top after it, the switch left off), the head's words and Show all, the
     tray's Show unchanged;
   - a screenshot run with each switch on over the fake portfolio, and one over a copy of the real
     portfolio's roadmaps (rule 5).
2. **The placeholder note** (D-9):
   - a completion note whose label still reads `YYYY-MM-DD` (in any case), the template's
     placeholder, is no note: `kind_of` gives it no completion kind, and a `## Completion note`
     heading that reads so is no evidence either;
   - the template's closing placeholder becomes an instruction in an HTML comment, which the
     parser skips whole: the note is written at the close, with the real date, model, effort and
     duration; the plugin's reference quotes the new template and says the placeholder is no
     note;
   - tests over synthetic briefs: a placeholder alone (not complete), a placeholder then a real
     note (complete by the real one, whose figures the History takes), a real note alone
     (unchanged); the new template read as a brief is not complete and holds no labelled block.
3. **The records:** a D-25 addendum for (1) and a D-9 addendum for (2);
   `docs/manual-changes.md`'s section for both; the roadmap's row linked and checked; this brief's
   completion note; rule 3's word check; the commits in the house pattern; the release build and
   the install, the session's last step.

## Out of scope

- The Ticked in the last week switches: kept with the view as before; the owner's choice names
  the This project only switch.
- The `_TEMPLATE.md` files the formalise step has already written into projects: never replaced
  (D-11); the parser fix covers them.
- The rule block's text (`formalise::RULES`): unchanged, since a change would make every
  formalised project offer Formalise… again.
- Other spellings of a placeholder (`<date>`, `date`): not caught; a real note carries its date.
- The status bar's notice and IN-24's pictures: EP-60b's.
- Any write into another project, the owner's registry or their user settings (rules 4, 7, 13).
- The push: the owner's.

## Verification / acceptance

- The tests above pass in `cargo test --workspace`; `cargo test --workspace --features
  screenshot`, both clippy runs with `-D warnings` and `cargo fmt --check` are clean.
- The screenshot runs: the Owner tasks pane and the Inbox pane with the switch on over the fake
  portfolio, and the panes over a copy of the real portfolio's roadmaps (rule 5), the pictures in
  the scratchpad and never tracked; this brief names what was checked, never the projects.
- `crw roadmap .` reads the roadmap with no new problem and the next EP as EP-60b once this row is
  checked.
- Rule 3's word check prints nothing for the staged diff; the commits in the house pattern, not
  pushed; the release build and the install last.

**Residual risk.** (1) **The lead's scroll in the real window**: the tests scroll a short pane in
a bare context; in the window the pane comes forward at the end of the frame that ran the lead,
and takes the lead at its next drawing, where egui animates the scroll over about a third of a
second. It is expected to hold because the frames run in that order for every pane a lead brings
forward (`run_actions`), and the list scrolls to a selected row the same way, inside its own
scroll area. It can first be seen when the owner next presses Open in Owner tasks or Open in
Inbox on a project lower in the pane. (2) **The window runs EP-58b's build** until it restarts
into this one (EP-60's install was refused at 21:32 while tabs awaited a first prompt; this
install asks again, File > Install and restart otherwise): until then its panes keep the switch
as saved and lead with it on, and its parser reads a placeholder note as complete. The view it
saves meanwhile still holds the switch, which the new build reads as off. (3) **The placeholder
rule reads labels as written**: a placeholder spelled otherwise (`<date>`, `date`) is not caught,
and a real note is passed over only if its label holds `YYYY-MM-DD`, which no real date does.
(4) **The templates already in projects** keep the blockquote placeholder; the parser passes over
it, so a brief that copies one stays open until its real note is written, which is what the gate
needs. The screenshot runs saw the panes over the fake portfolio and a copy of real roadmaps, not
the owner's own window.

> **Completion note (2026-11-27; claude-opus-5-5/max; about 38 minutes of active work and of
> wall time, 21:44 to the work commit `9c2f31c` at 22:22 and the record commit after it; no
> question put to the owner).** A visual session started from the window (ledger row `4a7598fa`); both
> fixes built as the row's note asked, on the owner's choices of EP-59.
>
> **Item 1, the panes' switch** (D-25). `View::tasks_one` and `View::inbox_one` are no longer
> saved (`#[serde(skip)]`), so each start shows every project, and a view an older build saved
> with them on reads them off, the RON reader passing over the fields it no longer knows.
> `Action::OpenTasks` and `Action::OpenInbox` (the Detail pane's leads, and an urgent message's
> Show through `Board::lead`) select the project, bring the pane forward and leave it a lead
> (`Board::tasks_lead`, `Board::inbox_lead`), which the pane takes at its next drawing, scrolling
> to the project's group (`scroll_to_cursor` with the top alignment), the switch as it was. While
> the switch is on, `owner_tasks::switch_line` says under the summary what the pane shows and how
> many open tasks (messages, with the urgent ones) of other projects it hides (`hidden_tasks`,
> `hidden_messages`, counted as the tabs count), in amber while it hides any, with Show all; off,
> `owner_tasks::unlisted` names a selected project the pane does not list. The hovers say so.
> Tests: the view saved without the switches and an older one read with them off
> (`the_this_project_only_switches_are_not_kept_across_restarts`); each lead's scroll in a short
> pane, the switch left off, and the heads' words with Show all
> (`a_lead_scrolls_the_pane_to_its_project_and_leaves_the_switch_off`,
> `while_the_switch_is_on_the_head_says_what_it_hides`,
> `a_lead_scrolls_to_its_receiver_and_the_head_says_what_the_switch_hides`); the three tests that
> expected a lead to turn the switch on now expect it off, the tray's Show test among them.
>
> **Item 2, the placeholder note** (D-9). `brief::is_placeholder`: a completion note whose label
> still reads `YYYY-MM-DD`, in any case, is `Kind::Other`, and such a `## Completion note` heading
> is no evidence. `formalise::TEMPLATE` ends with the note as an instruction in an HTML comment,
> which the parser skips whole; the plugin's reference quotes the new template, and its
> completion-note paragraph and the README's briefs bullet say that the placeholder is no note.
> **`House::wants`:** no formalised project is offered Formalise… again, since the template is
> created only where missing and the rule block is unchanged. Tests: a placeholder alone, a real
> note after it (in a quote of its own, and in the same quote below or above it), a real note
> alone, and the heading forms (`the_templates_placeholder_note_is_no_note`); the template read as
> a brief is open, holds no labelled block, and a real note closes it
> (`the_template_read_as_a_brief_is_open`).
>
> **Verification.** `cargo test --workspace`: 917 passed in 19 binaries (911 at EP-60, and the six
> new tests), run again after `cargo fmt` rewrapped lines of the new tests;
> `cargo test --workspace --features screenshot`: 925 passed; both clippy runs with `-D warnings`
> and `cargo fmt --check` clean. Screenshot runs, the pictures and the copy in the scratchpad and
> never tracked: the Owner tasks pane with the switch on over the fake portfolio (dark), the Inbox
> pane alike (light), the Owner tasks pane at 1100 × 900 with a selected project it does not list;
> over a copy of five non-data projects' roadmaps, the Owner tasks pane with the switch on and a
> project without tasks selected (the head naming four open tasks hidden in four other projects,
> IN-22's case), and the Inbox pane at 1100 × 900 with a selected project that has no open
> message; every line wraps inside its pane. `crw roadmap .` reads 99 rows in eight phases with no
> problem. Rule 3's word check printed nothing for the staged diff. The release build and the
> install are the last step, after the record commit.
>
> **Owner decisions.** None was put this session: the row's choices were the owner's at EP-59
> (the D-25 addendum of 2026-11-26, IN-23's triage), and the commits are the autopilot's standing
> yes (crw's D-22), on in this project. The push is the owner's.
>
> **Routine choices (the session's call, logged).**
>
> 1. **The model and effort** were read from the `model` and `effort` fields of this session's
>    own transcript entries rather than asked.
> 2. **A lead to a project the pane does not list** (nothing open in it) scrolls nowhere; the weak
>    line under the summary names the project and what shows it, rather than an empty group listed
>    for it or the switch turned on.
> 3. **The switch's line counts as the tabs do**: open tasks, and open messages with the urgent
>    ones; the older forms, counted nowhere, are not in it.
> 4. **Amber while the line hides something**, the window's colour for what waits on the owner;
>    the text's own colour when it hides nothing.
> 5. **The placeholder is caught by `YYYY-MM-DD` alone**, in the label and in the heading form; an
>    undated `Completion note.` of the older form stays a note.
> 6. **The template's instruction is an HTML comment**: the parser skips it whole, a forgotten one
>    renders as nothing, and it keeps the `YYYY-MM-DD` spelling, so a copy of its label left
>    unfilled is still no note.
> 7. **The rule block is unchanged**, so no formalised project is offered Formalise… again.
> 8. **No review subagent**: an S, for which rule 6 asks none; the diff was read through.
> 9. **OT-47**, the plugin's update after the push, as OT-32 and OT-45 were; nothing waits on it.
> 10. **No message to the senders**: neither fix needs anything of them.
> 11. **A slip, logged:** to choose the copy's projects, the open lines of five non-data projects'
>     task and inbox files were counted with a search rooted at the data root and a glob naming
>     those files, so ripgrep may have walked the data repositories' folders on its way (no file
>     there was read, and nothing of them was printed); every later search named its folders, as
>     the house style's rule 5 asks.
>
> The push is the owner's. **Next: EP-60b.**
