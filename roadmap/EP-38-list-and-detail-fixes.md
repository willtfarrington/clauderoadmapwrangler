---
ep: 38
title: List and Detail fixes from the P3 walk
size: M
model: claude-fable-5-1
effort: xhigh
status: done
---

# EP-38 — List and Detail fixes from the P3 walk

**Size:** M · **Depends on:** EP-7a, EP-36, EP-21a · **Blocks:** EP-29

## Context

The P3 walk (EP-23, 2026-11-11) settled three findings of EP-36's copy runs against the surfaces
the owner uses daily, each an owner's choice recorded as an addendum of that date: the list's
Brief title gives way before the badge columns (D-25), a project that keeps
`roadmap/owner-tasks.md` counts its tasks and its mail as waiting and not its risks list (D-10),
and the Detail pane's House style section shows only when it matters (D-11). This brief builds
the three, and checks them on a copy of the real roadmaps before its close (rule 5).

**Session start (2026-11-11, 17:27 local time).** A headless run started from the window under
this project's autopilot; its prompt is the tool's opener, and `crw sessions
clauderoadmapwrangler` lists its row (`d09da718`, headless, EP-38, fable/xhigh, open). The field
check read, since EP-23's record commit (`9d456f1`): `crash.log` (the 2026-11-03 report, nothing
newer); `crw notifications` (IN-11's `urgent` popup at 13:48:02 with its ntfy line and the inbox
badge, the first real urgent letter; two finished turns of EP-23's visual session at 13:35 and
13:47; the migration run's waits and its finish at 13:31 in the project migrated at the walk);
`crw mail` (`delivered EP-23:3cee3099e762a81f to clauderoadmapwrangler as IN-11` at 13:48:01,
once EP-23 was committed and its tab idle, as the brief note expected; IN-10 written by the owner
at 12:40); `crw sessions` (this run; the migration run, completed in 14 min; EP-23's visual
session, completed after 2 h 20 min); `crw autopilot` (not paused; six projects on under `auto`,
four of them turned on by the owner at 15:19 and 15:20 today; the last chain ended after EP-20b
on 2026-11-10 at the day's cap). Nothing odd to triage. The tree carries IN-11's delivered line
in `roadmap/inbox.md`, uncommitted, which the window's mailman wrote. No open owner task.

**Inbox triage (D-13).** IN-11, the test letter EP-23 handed on to this project's own inbox
flagged `[urgent]`: **declined as the test it is** and ticked. The log showed what the note
asked for: the delivery row at 13:48:01, then at 13:48:02 the `badge · inbox` row, the `popup ·
urgent` row ("clauderoadmapwrangler: IN-11 is urgent", the letter's first words as its gist)
and the `ntfy · urgent` row, so the first real urgent letter reached the popup and the phone.

**What exists.**

- `list::Plan::new` chooses among `Column::GIVE_WAY`'s five choices the first whose least leaves
  the title `TITLE_AT_LEAST` (160 points), and with none left the title keeps `TITLE_MIN` (40);
  the title is drawn `truncate()`d, and egui's `Label` shows an elided text whole on hover by
  default (`show_tooltip_when_elided`, 0.37). `list::tests::a_narrowing_row_gives_way_in_d25s_order`
  walks the choices.
- `Snapshot::owner_items()` chains the lines of `owner-tasks.md` and the older forms
  (`tasks::legacy`: a README's risks and open items, the registers, the latest brief's blocks and
  handoff notes); `ProjectRow::open_items()` counts the open ones of both for the Waiting cell
  (`list::counts_cell`), the status bar's sum (`board.rs`) and the Detail pane's "Waiting on the
  owner" section (`detail::owner_items`); `crw tasks` (`cli::waiting`) prints the tasks, the
  inbox, the handed-on blocks and the older forms' open items. The gate's `prompt::due` reads the
  tasks file alone.
- `portfolio::House` (`read`, `wants`, `writes`, `refused`, `migrates`, `writes_words`) is read
  by the files pass from `formalise::plan`; `detail::house_style` shows its section for a project
  with `wants()`, in the unformalised state or with an older roadmap, and the row menu
  (`list::project_row`) offers Formalise… and Migrate the roadmap… on the same test.
- Screenshot runs: `CRW_SCREENSHOT`, `CRW_SCREENSHOT_SETUP` (`light`/`dark`, `select=`, `tile=`,
  `size=<w>x<h>`, `root=`), `CRW_SCREENSHOT_ROOT` and `scripts\copy-roadmap.ps1` (EP-36).

## In scope

1. **The title gives way first** (D-25 addendum, 2026-11-11). `Plan::new` keeps `TITLE_AT_LEAST`
   for the full form alone (the badges compact first, as today); for the compact forms (the
   compact badges with the counts, without them, the progress alone) the title's least is a new
   `TITLE_FLOOR`, so the title, cut with an ellipsis and whole on hover and in the Detail pane,
   gives way down to the floor before the counts leave and before the compact badges leave; with
   every optional column gone it keeps `TITLE_MIN`. The spare width is shared only beyond
   `TITLE_AT_LEAST`, as today, so the gaps stay at their least while the title is squeezed. The
   order among the optional columns stays D-25's. The floor is set from pictures of the copy at
   1440 and 1600 points: the candidates (120, 100, 80) judged against the column sums the
   pictures' runs measure, pictures taken at 100 and at the floor set; the one that keeps the
   compact badges at both widths, and if none does, the pictures' choice goes to the owner. *(As
   it went: at the default layout none does, and the owner chose 80; the completion note has the
   numbers.)*
2. **Waiting counts tasks and mail** (D-10 addendum, 2026-11-11). `Snapshot` gains
   `older_forms_count()` (no `owner-tasks.md`), `waiting()` and `open_waiting()` (the tasks where
   the file is kept, else the older forms) and `uncounted()` (the older forms of a project that
   keeps the file; none elsewhere). `ProjectRow::open_items()` counts `open_waiting()`, so the
   Waiting cell and the status bar's sum follow; the Detail pane's "Waiting on the owner" section
   lists `waiting()`, and a new section, "Risks and older forms", lists `uncounted()` under a
   heading of its own with the count and "not counted", read-only with the lock, after the
   handed-on mail and before the decision log. The Waiting header's hover text says the rule.
   `crw tasks` prints the same split: its `older` line and its heading say whether the older forms
   count and why.
3. **The House style section only when it matters** (D-11 addendum, 2026-11-11).
   `House::matters()`: `roadmap/owner-tasks.md` or `roadmap/inbox.md` among the files to write, a
   refusal, or an older form. `detail::house_style` shows the section for `matters()` or the
   unformalised state; the row menu keeps today's test (`wants()`, or the unformalised state).
4. **Tests.** The list's plan: the badges compact before the title gives way, the title at the
   floor before the counts and the compact badges leave, D-25's order kept. The split: the `house`
   fixture (tasks file and a risks list) counts its tasks and lists its forms uncounted, a fixture
   without the file counts its forms; `harbor` and `orchard` keep their counts. The section:
   `harbor` (lacking only `CLAUDE.md` and the template) shows the menu item and no section;
   `lantern` (lacking `owner-tasks.md` too) shows it.
5. **The records.** Addenda to D-25 (the floor as set), D-10 and D-11 (as built); the README's
   sentences on the Waiting column, the Detail pane and the House style section; this brief.

## Out of scope

- **Drag-resizable columns** and the names cut first (v2 notes; D-25 addendum's alternatives).
- **A sort by the Waiting count** (the review of EP-23 found none exists; not asked).
- **Counting the older forms by kind** (a parked block counted, a risk not): the file decides.
- **The gate.** `prompt::due` already reads the tasks file alone.
- **The chat's mentions and EP-39's fixes.**

## Verification / acceptance

- `cargo test --workspace`, `cargo test --features screenshot`, both clippy runs and `cargo fmt
  --check` clean.
- A review subagent given the diff alone (rule 6), launched as soon as the code compiles, its
  findings fixed or logged.
- **Screenshot runs**, kept in the scratchpad (`target\tmp\ep38\`), never in git:
  - the fake portfolio in both themes, the Detail pane on a project with the section and on one
    without;
  - the copy of the twelve non-data projects (`scripts\copy-roadmap.ps1`, one call each) at 1440
    and 1600 points at a floor of 100 and at the floor set, both themes at the floor set; the
    Detail pane of a copy that keeps a tasks file and a risks list, its risks listed uncounted
    (this project's pane runs longer than the screen, so another project's copy showed the
    section, this project's the sections above it).
- `crw tasks .` prints the split on this folder: the tasks and the inbox counted, the risks not.
- The release build and the install (`pwsh -File scripts\install-shortcut.ps1`) with the window
  up; rule 3's word check on each staged diff prints nothing.

**Residual risk.** The floor is chosen from pictures of today's copy, so a longer project name
or a wider State chip tomorrow could still cost the counts a column at 1440 points; the title
then keeps the floor and the next narrower choice applies in D-25's order, which the tests pin.
The hover text that shows a cut title whole is egui's own and was not pictured, since a
screenshot run feeds keys and not the pointer. The risks list drawn uncounted was pictured on
one real copy, whose list is short; this project's, seventeen items long, is covered by the unit
test and by `crw tasks .`, since its Detail pane runs past the screen's height in a screenshot
run, which cannot scroll a pane.

> **Completion note (2026-11-11; claude-fable-5-1/xhigh; about 45 min of wall time, a headless
> run under the autopilot, one question).** Ledger row `d09da718`, started from the window at
> 17:27 under this project's autopilot (an M's cap of 90 min); the prompt was the opener.
>
> **Built.**
>
> 1. **The title gives way first.** `list::TITLE_FLOOR` (80) is the title's least for the compact
>    forms and `TITLE_AT_LEAST` (160) the full form's, `Plan::title_least` choosing by the choice's
>    index; the gaps still widen only beyond 160, so a squeezed title never pays for them. Tests:
>    `the_title_gives_way_before_the_counts_and_the_compact_badges` (the badges compact first;
>    from 159 down to the floor the title alone gives way, the compact badges and the counts
>    staying and the row filled to its edge; a point under the floor the counts leave, then the
>    badges once the title is at the floor again) and `a_narrowing_row_gives_way_in_d25s_order`
>    with the per-choice least; a `const` assertion keeps 40 < floor < 160.
> 2. **Waiting counts tasks and mail.** `Snapshot::older_forms_wait`, `waiting`, `open_waiting`
>    and `uncounted`; `ProjectRow::open_items` counts `open_waiting`, so the Waiting cell and the
>    status bar's sum follow; the Detail pane's "Waiting on the owner" lists `waiting()` through
>    the shared `owner_item` drawer, and the new `older_forms` section ("Risks and older forms: n
>    open, m resolved, not counted", a weak line saying why, after the handed-on mail) lists
>    `uncounted()` with the lock and no tick; the Waiting header's hover says the rule; `crw tasks`
>    says under its `older` line whether the older forms count and why, and in its open-items
>    heading. On this folder: 25 tasks, 0 open; 11 messages, 0 open; 17 older items, not counted.
>    The copy's status bar sums 42 owner items where EP-36's runs counted about 78.
> 3. **The House style section only when it matters.** `House::matters` (the tasks file or the
>    inbox among the files to write, a refusal, an older form); `detail::house_style` shows for
>    `matters()` or the unformalised state; the row menu's test is unchanged. Tests: lantern (no
>    tasks file) shows the section and its button, harbor (only `CLAUDE.md` and the template
>    missing) the menu's test and no section, unformalised harbor the section, a refusal and an
>    older form each the section; `House::matters` on built values.
>
> **The pictures** (`target\tmp\ep38\`, never git). The copy of the twelve non-data projects
> (ten active with a roadmap, two unformalised), at the window's default layout (the Projects
> pane at 55 % of the width): the row measures 766 points in a 1440-point window and 854 in a
> 1600-point one, while the columns alone take 935 for the full badges, 842 for the compact
> badges with the counts, 765 without them and 551 for the progress alone, the longest name 153
> and the counts 67. So at 1440 no floor keeps a badge column (the progress alone, the title at
> 215), and at 1600 a floor of 80 or less keeps the compact badges without the counts, the title
> cut at 89, which the pictures at 80 show in both themes; at 100 the progress alone at both
> widths. The fake portfolio in both themes with the Detail pane: lantern's House style section
> and harbor's none; a real copy's Detail pane with "Risks and older forms: 4 open, 2 resolved,
> not counted" under an empty "Waiting on the owner" and its House style section for an older
> form. The pictures of the decisions pane, the launch dialog and the compose dialog were not
> retaken: nothing of theirs changed.
>
> **The review (rule 6).** A subagent given the diff alone, launched once the code compiled,
> reported after eight minutes: no confirmed bug and four plausible points. (1) The test's
> hard-coded `\` in a meta line: fixed with `Path::join`. (2) `older_forms_count` read as a
> count: renamed `older_forms_wait`. (3) A parked block or a handoff note of a project with the
> file is now uncounted: intended, the D-10 addendum says so. (4) `text_at(&output, "1")` is a
> weak but exact match: left. Clippy then refused a constant assertion in the test
> (`assertions_on_constants`): moved to a `const _` beside the constants.
>
> **Verification.** `cargo test --workspace`: 557 passed (354 in `core`, 148 in `app`, 55 in the
> other suites); with the feature 565 (156 in `app`); both clippy runs and `cargo fmt --check`
> clean, the whole gate run again after the floor was set. The copy runs as the Verification
> section says, `crw tasks .` as above. Rule 3's word check on the staged diff printed nothing.
> The release build and the install (`scripts\install-shortcut.ps1`) run after the record commit,
> the opener's last step; the restart is refused while this run works and taken at the gate's
> moment, or from File > Install and restart.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-11, answered in the decisions tile).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The title's floor, none keeping the compact badges at both widths at the default layout | 80 points (the recommended option) | `TITLE_FLOOR = 80.0`; the D-25 addendum with the numbers |
>
> The commits are the autopilot's standing yes (D-22), not pushed.
>
> **Routine choices (the session's call, logged).**
>
> 1. **One floor for the compact forms** rather than one per step; the full form keeps 160, so
>    the badges compact before the title gives way, as the brief note's order says.
> 2. **The file decides what counts, not the form** (the D-10 addendum): a project that keeps
>    `owner-tasks.md` counts no older form, since the house style writes what needs the owner's
>    hands to the file.
> 3. **The uncounted section** sits after the handed-on mail and before the decision log, open
>    (EP-15a: no segment folds), with a weak line saying why it is not counted; the empty waiting
>    line reads "Nothing: no open owner task." where the file is kept.
> 4. **`House::matters`** tests the plan's step names by the file names `tasks::FILE` and
>    `inbox::FILE`, as `formalise::plan` builds them.
> 5. **The floor judged from measured sums.** A temporary print in the pictures' runs gave each
>    choice's least (removed before the commit), so the candidates were settled at once and the
>    pictures taken at 100 and at 80 rather than at three floors.
> 6. **The scratchpad** is `target\tmp\ep38\` (ignored): the copy, the home folders, the
>    pictures, the logs and the diff the review read.
>
> No owner task; no message for another project. **Next: EP-39** (the launch dialog and the
> prompts), chained by the autopilot when its gate allows, else from the window.
