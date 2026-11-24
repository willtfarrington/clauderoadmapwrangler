---
ep: 50
title: The Owner tasks pane, every project's open tasks in one place and ticked there; the list counts owner-tasks lines alone, the older forms marked and folded, never counted
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-50 — The Owner tasks pane; the list counts owner-tasks lines alone

**Size:** M · **Depends on:** EP-46 · **Decisions:** D-10, D-25 (and D-3, D-11, D-29 as standing)

## Context

The owner's request 5 at EP-45, its first half, and their choices (D-10 and D-25 addenda of
2026-11-22): the list counts open `owner-tasks.md` lines and nothing else; a project without the
file shows no count and a mark whose hover says how many older-form items it holds; every
project's open tasks get one pane of their own, ticked there, and the Detail pane keeps a line
that leads to it.

**What the code says** (read at pickup).

- **The count's one lever.** `Snapshot::open_waiting` (`crates/core/src/roadmap/snapshot.rs`)
  counts the open lines of `owner-tasks.md` when the file is kept, else the open older forms;
  `Snapshot::waiting` gives the same items; `Snapshot::uncounted` the older forms of a project
  that keeps the file, none elsewhere; `older_forms_wait` says which. `ProjectRow::open_items`
  (`crates/app/src/portfolio.rs`) calls `open_waiting`.
- **Its readers, and what each counted before.**
  - The list's Waiting cell, `counts_cell` (`crates/app/src/list.rs`, drawn in the rows and the
    cards), with its header's hover; the status bar's sum "N owner items" (`board.rs`): the
    file's open lines, else the open older forms.
  - `crw tasks` (the CLI's `waiting`): the file's, the inbox's and the older forms' counts, the
    older forms said "counted as waiting" without the file and "not counted" with it.
  - `crw status`: the badges only; it counts no owner items, so nothing changes.
  - The chat's digest (`ProjectDigest::open_tasks` and `older_forms`, `crates/core/src/chat.rs`,
    filled in `crates/app/src/chat.rs`): "N open items (no owner-tasks.md: items in older forms,
    a risks list among them, as crw tasks lists them)" without the file (IN-12's wording, EP-25).
  - The tray: its red disc counts the decisions waiting (`Board::decisions_count`) and its amber
    dot the new badge events (an unadopted folder, remote, backup, an inbox message): neither
    counts owner items, so nothing changes.
  - The phone: the list's count per project (`published::Line::owner_items`) and its sum
    (`pages.rs`, "N owner items"), and the project page's open items and its "Risks and older
    forms … not counted" (`published.rs` from `waiting` and `uncounted`, `project.html`).
  - The daily export: the registry file and the ledger's tables, which hold no such count, so
    nothing changes.
  - The Detail pane: `detail::owner_items` (from `waiting`, ticked there, the ticked folded with
    Untick) and `detail::older_forms` (from `uncounted`).
- **The note's "Waiting sort key and the keys' help"**: no sort key by waiting exists (the keys
  run Priority … Name), and the keys' help, the status bar's line (↑ ↓ select · → ← expand ·
  Enter detail · S start · A autopilot · F5 refresh), names no pane, so it stays; the Waiting
  column's header and the legend are what follow. *(Corrected at the first picture: the brief
  first said no keys' help existed.)*
- **A tick** goes through `Action::Tick` / `Action::Untick` to `Board::tick`, whose
  `folder_for_write` keeps today's refusals (a revert open, a session working there, a data
  repository, a screenshot run's fixtures), then `tasks::tick` with its backup, the status
  bar's line, and `revert::log_tick` into the ledger, which `revert::ticks_since` reads back.
  So the ledger dates the ticks the tool made; a tick written into the file by hand is dated
  nowhere.
- **The layout.** `Pane::ALL`, `default_layout`, `pane_ui`, `complete` (`layout.rs`): a saved
  layout gains a missing pane as a tab beside Detail, so the default puts the new pane there
  too, behind Detail, and the two layouts agree.

**Session start (2026-11-24, 10:54 local time).** A headless run the owner started from the
window (the rolling log, 10:54:59: "Started clauderoadmapwrangler EP-50 headless with
claude-opus-5-5 at max effort"); `crw sessions clauderoadmapwrangler` lists the row (`63120ec4`,
headless, EP-50, open), and the prompt is the tool's opener. The field check read, since EP-49's
record commit (00:27:36): `crash.log`, unchanged since the panic of 2026-11-23 04:00 that EP-43a
read; the rolling log, EP-49's install restart refused while it worked, the restart at the gate's
moment carrying the held chain, the chain's end after EP-49 ("EP-50 is left out of your plan"),
a full read at 10:48, the owner's state change of another project at 10:51, this project read
again as its launch dialog opened, and this start; `crw notifications` and `crw autopilot`
(this run's chain mark at 70 % of the session window, the owner's choice in its plan): nothing
odd. No inbox message is open; OT-31 (the phone picture) is open and gates nothing.

**Against the cap.** M's cap is 1 h 30 min of working time from 10:54, so about 12:24. The count
is one lever in `core` that every reader goes through, plus the words of `crw tasks`, the
chat's digest and the list's mark; the pane is one new module with two toggles, a line in the
Detail pane and a layout entry; then the tests, the review an M asks for (in the background,
launched once the code compiles), the screenshot runs, the release build, the records and the
install. Judged at 11:05 to fit. **Should the code not compile with the pane by 11:45,** the
"ticked in the last week" toggle (the one part that reads the ledger) is split off as EP-50a,
proposed in the completion note, and the rest closes.

## In scope

1. **The count** (`snapshot.rs`): `waiting` gives the file's lines or nothing; `open_waiting` the
   open lines or none; `uncounted` every older form, whether or not the file is kept;
   `older_open`, the open older forms; `keeps_tasks` in place of `older_forms_wait`. Every reader
   above follows from these.
2. **The list** (`list.rs`, `badges.rs`): the Waiting cell counts open owner tasks ("open owner
   tasks"); a project without the file and with open older forms shows no count and a weak
   "older forms" mark, whose hover says how many open items its older forms hold, that they are
   read-only and counted nowhere, shown folded in the Owner tasks pane, and that Formalise…
   gives the project a task file. The legend's Row group gains the mark; the Waiting header's
   hover says the new rule; the status bar says "N owner tasks".
3. **The readers' words:** `crw tasks` says the older forms are never counted (with or without
   the file); the chat's digest gives "N open owner tasks" for a project with the file and, for
   one without, "no owner-tasks.md (N open items in older forms, not counted, as crw tasks lists
   them)"; the phone's "owner items" become "owner tasks".
4. **The pane:** `Pane::OwnerTasks`, "Owner tasks" (the tab says "Owner tasks · N" while N
   tasks are open), a tab behind Detail in the default layout; a new module
   `crates/app/src/owner_tasks.rs`. Every project's open tasks grouped by project in the list's
   order (its groups and sort key, its filters not applied, so a filter never hides a waiting
   task); under each project's name its tasks, each with its box, its id, its gate as a chip
   (amber with a hover saying a chain into the next brief holds on it when it gates the next
   brief or one before it, `prompt::due`'s rule; weak when it gates a later brief), its date and
   its text set by `markup` as the Detail pane's were; a box ticks through `Action::Tick` with
   its backup, the status bar's line and today's refusals. Two toggles saved with the view:
   **This project only** (the selected one) and **Ticked in the last week** (the tasks ticked
   now whose latest tick in the ledger is within seven days, each with Untick and when it was
   ticked; a tick made in the file by hand is not dated, which the toggle's hover says). At each
   project's foot its older forms, folded shut, read-only (the lock), never counted; a project
   without the file shows that fold alone. A name selects its project.
5. **The Detail pane:** `owner_items` and `older_forms` leave it; in their place one line, how
   many tasks are open (and how many gate the next brief), or that the project keeps no task file
   and how many older-form items it holds, with **Open in Owner tasks**, which selects the
   project, sets This project only and brings the pane forward.
6. **Screenshot runs** (rule 5): the setup's `tile=owner tasks` and the toggles' flags; runs of
   the pane, the list with its legend and the Detail pane at 1600 × 1000, 2300 × 1000 and
   1100 × 900, and one over a copy of the real portfolio's roadmaps, each looked at.
7. **The records:** the README's paragraph on the panes; D-10 and D-25 addenda.

## Out of scope

- The Inbox pane and the Detail pane's folds after Status (EP-51); the Autopilot pane (EP-52).
- Ticking or rewriting an older form (D-10: read-only); EP-55's sweep tasks.
- A date for a tick written into the file by hand (the file records none; the ledger only the
  tool's); a sort key by waiting tasks, which the note named and the code never had.
- The phone's own pane for the tasks: its project page keeps its lists, from the same functions.

## Verification / acceptance

- Unit tests: the count over a project with the file and one without (the fixtures'
  `owner_fixtures.rs` too); the list's mark and its hover; the digest's words; the pane's groups
  in the list's order, a gate marked when it gates the next brief, the toggles, the older forms
  folded and uncounted, a box's tick as an action; the Detail pane's line and its button; the
  layout's default and a saved layout completed; the ledger's ticks of the last week.
- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings`, `cargo
  fmt --check`; a review subagent given the diff (an M brief), launched once the code compiles.
- Screenshot runs as in scope (6), kept in the git-ignored `target\ep50-shots`; the copy and its
  pictures never enter git, and this brief names what was checked, never the projects (rule 3).
- The release build, and the install last.
- No tracked file names a managed project, the owner's login or the machine (rule 3); the word
  check prints nothing on the staged diff.

**Residual risk.** The pictures are the app's own frames of the fake portfolio and of a copy of
the real roadmaps; no screenshot run presses a box or opens a fold, so a tick from the pane, its
refusals (a revert open, a session working there, a data repository) and the folded older forms
opened are exercised by unit tests and by the action path the Detail pane used, not by the
owner's hand. The "Ticked in the last week" switch reads the real ledger's `tick` events, which
the tests write into a scratch ledger; how the real ledger's ticks show, and whether a busy
ledger ever leaves the list a minute stale, is the walk's. A saved layout gains the pane behind
Detail through `complete`, tested on a layout saved without it; the owner's own saved layout
meets it at the first start of the new build. The Detail pane's new line sits below Status,
beyond the pictures' height, and is checked by unit tests alone. The phone's pages say "owner
tasks" now, checked as HTML out (rule 5), not in a browser.

> **Completion note (2026-11-24; claude-opus-5-5/max, a headless run the owner started; about
> an hour of working time from the start at 10:54 to the work commit, the owner's one question
> answered within a minute).**
> Started by the owner from the window at 10:54 (ledger row `63120ec4`) under M's cap of
> 1 h 30 min of working time. Judged at the brief to fit, so nothing was split: the count and
> the pane compiled at the first build at 11:09, the review ran in the background from then,
> the screenshot series and the copy run by 11:19, the review's fixes and the checks by 11:41,
> the owner's question at 11:43, then the records, the commits and the install.
>
> **Owner decision** (one `AskUserQuestion` at 11:43, answered with the recommended option).
>
> 1. **A long task's room in the pane:** over the copy of the real roadmaps one project's task,
>    a single `owner-tasks.md` line of several paragraphs, fills most of the pane's height at
>    1600 × 1000. Parked in EP-51's brief notes as a dated addendum: EP-51 folds an item longer
>    than about six lines to its first sentence with "more…" in both the Owner tasks and the
>    Inbox pane, or splits it off at pickup. Not chosen: a new S row after EP-51; carrying it to
>    the P6 walk; leaving it.
>
> The commits are the autopilot's standing yes (D-22); the push is the owner's.
>
> **Built.**
>
> 1. **The count** (`crates/core/src/roadmap/snapshot.rs`): `waiting` and `open_waiting` read
>    the lines of `owner-tasks.md` alone; `uncounted` is every older form; `older_open` and
>    `keeps_tasks` (in place of `older_forms_wait`) are new. The fixtures' test
>    (`crates/core/tests/owner_fixtures.rs`) says both cases.
> 2. **The readers:** the list's Waiting cell counts open owner tasks, and a project without the
>    file shows the weak "older forms" mark (`badges::OLDER_FORMS`, `list::older_forms_words`),
>    in the legend's Row group, the Waiting header's hover saying the rule; the status bar says
>    "owner tasks"; `crw tasks` calls the older forms counted nowhere; the chat's digest says "No
>    owner-tasks.md (N open items in older forms, …, counted nowhere, …)", or "No
>    owner-tasks.md" alone when none is open (`ProjectDigest::older_forms` an `Option<usize>`);
>    the phone's list says "owner tasks", and its project page carries `keeps_tasks` so that its
>    words for the older forms are true of a project without the file.
> 3. **The pane** (`crates/app/src/owner_tasks.rs`; `Pane::OwnerTasks` in `layout.rs`, a tab
>    behind Detail; `View::tasks_one` and `tasks_ticked`; `Action::OpenTasks`; `Board::ticks`):
>    the projects with an open task or, without a task file, open older forms, in the list's
>    order with its filters left aside; each task with its box, id, gate chip (`Hold`: amber
>    for the next brief's, weak for a later brief's, amber with neutral words for an older
>    form's or an unplaced one), text and meta line; the tool's ticks of the last week from the
>    ledger with Untick; the older forms folded; the summary over every project. The phone's
>    tick marks the ticks stale too.
> 4. **The Detail pane** (`detail.rs`): `owner_items` and `older_forms` gone for
>    `owner_tasks::detail_line` with Open in Owner tasks.
> 5. **The screenshot feature:** `tile=owner tasks`, `tasks-one`, `tasks-ticked`.
> 6. **The records:** the README's paragraph on the panes; D-10 and D-25 addenda; EP-51's
>    addendum.
>
> **Verification.** `cargo test`: 297 in `app`, 479 in `core`, the CLI's and the integration
> suites, all passing; `cargo test --features screenshot`: 305 in `app`; both clippy runs with
> `-D warnings` and `cargo fmt --check` clean, all on the final tree. The new tests: the count
> with and without the file (the fixtures and orchard); a project's block (its open tasks, the
> ticked one left out, the older forms folded, its name selecting it); a project without the
> file and the list mark's words; the Detail line and its button; the ledger's ticks of the
> week (the latest of each, a clear undoing a tick, a message's tick and a delivery left out,
> a minute's wait and `stale`); a ticked task with Untick; each gate's hold against the next
> brief; the pane's projects in the list's order with a filter set, This project only, and
> Open in Owner tasks; the summary; the default layout and a layout saved without the pane;
> the phone's project page without the file. Screenshot runs, kept in the git-ignored
> `target\ep50-shots`, each looked at: the pane at 1600 × 1000 in Dark, with the list's legend
> showing the new mark; the pane at 2300 × 1000 in Light; one project with both switches at
> 1100 × 900; the Detail pane, tall, whose new line lies below Status out of frame. Over a copy
> of the real portfolio's twelve non-data projects (`scripts\copy-roadmap.ps1`, one at a time,
> into a scratch folder outside the repository, removed after with its run's home): the pane
> with the legend at 1600 and the pane at 1100. The list counted 4 open tasks there, where the
> old reading counted about 78 items; nothing drawn wider than its pane. The release build
> passed; the word check printed nothing.
>
> **Review** (a subagent given the diff and the brief, about 11 minutes). Seven points:
>
> 1. **The phone's project page** said "the project keeps owner-tasks.md, which is what waits"
>    of every project with older forms, now true only of those with the file: `keeps_tasks`
>    carried to the page, its words branched; tested.
> 2. **An older form's gate chip** said a chain into the next brief holds on it, which no chain
>    reads: the colour and the words now come from `Hold`, an older form's and an unplaced
>    gate's words neutral; tested.
> 3. **`cargo fmt`** had not run when the diff was taken; it had by the review's end.
> 4. **The ticks' read** waited a minute after a busy ledger and missed the phone's ticks: a
>    busy read is asked again after the window's retry (`pace::LEDGER_RETRY`), and the phone's
>    tick marks the ticks stale.
> 5. **Words:** the switch's hover names the tool's ticks (the panes, the phone, `crw tick`);
>    the digest drops "0 open items"; the README and the screenshot's `tile=` list had been
>    brought up to date meanwhile; the summary's "in any project" under This project only was
>    found in the 1100 picture and fixed before the review came back.
> 6. **Which projects the pane lists:** a project that keeps the file with no open task no
>    longer shows for its older forms alone (the brief's reading), which the copy's projects
>    with a risks list would have filled the pane with.
> 7. **Missing tests** of the pane's rows, This project only and Open in Owner tasks: added.
>
> **Session's calls.**
>
> 1. The pane behind Detail in the default layout, where `complete` puts it in a saved one.
> 2. "Ticked in the last week" from the ledger's `tick` events, the one record of a tick's
>    time; a tick written by hand is dated nowhere and the switch's hover says so.
> 3. The sentence on a project without the file on the hover of its "no task file", after the
>    first picture showed it repeated for every such project.
> 4. The brief's sentence on the keys' help corrected at the first picture: the status bar's
>    keys line exists and names no pane, so it stays.
>
> **Found and logged, not changed.** The Detail pane's new line sits below Status, out of the
> pictures' frame at every height the screenshot takes; its words and button are unit-tested.
>
> No owner task and no message to another project.
