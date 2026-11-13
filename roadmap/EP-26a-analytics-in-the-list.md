---
ep: 26a
title: Analytics in the list
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-26a — Analytics in the list

**Size:** M · **Depends on:** EP-26 · **Blocks:** EP-29 · **Decisions:** D-25, D-26

## Context

D-25 promised rows that expand inline to the burn-up sparkline, Expand all and Collapse all, and
a cards view as a toggle; EP-8 folded the cards into EP-26 (D-25 addendum), and the P3 re-plan cut
EP-26 in two (D-4 addendum): EP-26 drew the graphs in the Detail pane, this brief brings them to
the list. An expanded row says today that "the burn-up sparkline arrives at EP-26a". The brief
note asks for the sparkline in the expanded row, Expand all and Collapse all, and the cards view
(a Cards switch in the Projects toolbar, saved with the view, one card per project with its name,
state chip, next brief and size, progress pill, badges and sparkline, in the list's groups and
order), the cards and expanded rows following `style::unselectable` (EP-15a), wrapping in a
narrow pane (EP-26's note from EP-16), and checked on a copy of the real roadmaps (rule 5).

**Session start (2026-11-13).** The session's prompt is this project's opener, launched from
clauderoadmapwrangler with claude-opus-5-5 at max effort; `crw sessions clauderoadmapwrangler`
lists its row (`fbb829f6`, headless, EP-26a, started 14:23), and `crw autopilot`'s log has no
chain line for it, so the owner started it from the launch dialog (Start headless) under M's
cap of 1 h 30 min. Read: the P4 table and EP-26a's brief note, D-25 and D-26 with their
addenda, EP-40's completion note, EP-26's brief, `owner-tasks.md` (no open task) and the inbox
(no open message). The field check: `crash.log` is still EP-14a's of 2026-11-03, seen; `crw
notifications` shows EP-26a's failed chained run of 2026-11-12 19:49 and EP-40's manual session
today, nothing since EP-40's record commit; `crw sessions` shows the same and this run open;
`crw autopilot` shows six projects on and the chain log ending with EP-26a's failed run; `crw
usage` reads the session (five-hour) window at 33 %. Nothing odd to triage. The working tree
was clean at the start. **The first attempt's diff**: the failed run of 2026-11-12 (fable at
high, cut after 19 minutes by the five-hour limit and reverted by the owner) left its diff in
the ignored scratchpad `target\tmp\ep26a\diff.patch`, with a passing `cargo test --features
screenshot`; this session reviews it and takes it as its starting point.

**The code as found.** `list::project_row` draws a row and, when expanded, `expansion` under it
(each phase's id, bar, counts and name in a `ui.horizontal`, which does not wrap, and the
placeholder line); the expansion takes no click. The View menu has "Expand every row" and "Fold
every row"; the Projects toolbar is one `ui.horizontal` line (the filter, Sort, Filters,
Legend). `analytics::Loaded` reads every session row but one project's snapshots, for the
selected project, so the list has no snapshots of the other projects; `burn_up` builds the
series from a project's completions (`ProjectRow::history`) and snapshots. The first attempt's
diff makes the read portfolio-wide (`Ledger::snapshots_all`), adds the painted sparkline
(`analytics::sparkline`), the cards (`project_card`, `folder_card`, `cards_per_line`), the
toolbar's buttons, `View::cards` and the `cards` flag; it uses `#[allow(clippy::too_many_
arguments)]` twice (the crates use none), leaves the toolbar a non-wrapping line (a narrow
pane's trap, EP-13), lets the cards of one line differ in height, and gives the expansion no
click.

## In scope

1. **One read for the graphs.** `Ledger::snapshots_all` (every project's `snapshot` rows by
   project) in core with a test; `analytics::Loaded` holds every project's snapshots and is read
   whatever project is selected, so the Detail pane's section and the list share one read a
   minute (or when invalidated), asked by the Detail pane for a selected project and by the list
   while a row is expanded or the cards show.
2. **The sparkline.** `analytics::sparkline_points` (the burn-up's done and total as stepped
   polylines in a rect, from the series' first day to today, zero at the bottom) and
   `analytics::sparkline` (painted with egui's painter, hover only: the done in green over a
   tint, the total in the text's colour; the hover text says done of total, the first day drawn
   and the total's last change). No `egui_plot` in a row: a plot per row would take the row's
   pointer.
3. **The expanded row.** Each phase on a line whose name wraps under itself, and the burn-up
   sparkline beside the phases when the row is wide enough, else under them, with a small
   caption, or one weak line saying why there is none (no ledger; no EP dated and no daily
   count). The expansion answers as its row does: a click selects, a double click opens, a right
   click opens the row's menu, and the row and its expansion take one hover tint.
4. **Expand all and Collapse all** in the Projects toolbar (icon buttons, their words on hover)
   and in the View menu (renamed from "Expand every row" and "Fold every row"): Expand all
   takes the projects the list shows through its filters, Collapse all folds every row. The
   toolbar wraps in a narrow pane.
5. **The cards view.** A Cards switch in the toolbar and the View menu, saved with the view
   (`View::cards`, read as off from a view saved before). The groups and their order stay; the
   cards of a group fill the line, as many as fit at 280 points, the spare width shared, one
   card as wide as the pane when it is narrower; the cards of one line are as tall as its
   tallest. A project's card: the pin and the name; the state chip with the bulb, the next EP
   and its size; the next brief's title on two lines at most (whole on hover), or why there is
   none; the progress pill; what waits, the badges and the session mark on a wrapping line; the
   sparkline. An unadopted folder's card: its mark, name, chip, what it holds, Adopt and
   Ignore. A card takes the click, the double click and the right click over its text as a row
   does (`style::unselectable`), with the row's menu.
6. **No `#[allow]`**: the drawing context (the view, the scroll, the times, the ledger's read)
   goes to the row and the card as one struct.
7. **Screenshot flags**: `cards`; `expand=all` (every registered project's row expanded);
   `analytics=all` (canned daily counts and history for every registered project, so that a
   copy of the real roadmaps draws sparklines too); `analytics` keeps harbor's and adds
   lantern's counts.
8. **Tests**: the sparkline's points; the read's minute without a project; `snapshots_all`; a
   card answering over its name with the row's menu and drawing its sparkline; an expanded row
   drawing the sparkline or saying why not, and its expansion answering a click; the cards per
   line; Expand all through the filters and Collapse all; a view saved before reading with the
   cards off.
9. **Docs**: a D-25 addendum (the cards and the expansion as built), the README's paragraph on
   the list, the roadmap row linked, the completion note, the project memory.

## Out of scope

- The heatmap in the expanded row (D-25 named it; the Detail pane has it since EP-26, and the
  brief note asks for the sparkline alone).
- The autopilot box and the Start button on a card: the keys A and S act on the selected card,
  and the row's menu is the card's.
- A card's own density or a size switch; drag-resizable cards (v2 notes).
- The usage sparkline moved to `egui_plot`; the five-hour items parked at EP-40 (EP-29).

## Verification / acceptance

1. `cargo test --workspace` and `cargo test --features screenshot` pass; both clippy runs and
   `cargo fmt --check` are clean; rule 3's word check on the staged diff prints nothing.
2. Screenshot runs over the fixtures: `cards,history,analytics` at 1600 light (harbor's and
   lantern's sparklines on cards, the lines of equal height), `cards` at a narrow size in dark
   (one card per line, the text wrapping); `expand=all,history,analytics` (the expanded rows
   with the sparkline beside the phases) and a narrow one (the sparkline under them, the phase
   names wrapping).
3. Screenshot runs over a copy of the real roadmaps (`scripts\copy-roadmap.ps1`, one call per
   non-data project; `CRW_SCREENSHOT_ROOT`, a fresh `CRW_HOME`): `cards,analytics=all` at 1600
   and 1200, `expand=all,analytics=all` at 1200, where names and titles are long; the copies
   and the pictures stay in the scratchpad and out of git, and this brief names what was
   checked, never the projects.
4. A review of the diff by a subagent given the diff alone (rule 6, an M brief), its findings
   fixed or logged.
5. The release build and the install run last (`scripts\install-shortcut.ps1`); the restart is
   refused while this run works and taken from File > Install and restart, the tray's, or the
   launch dialog's.

**Residual risk.** The real window was not exercised: the clicks, double clicks, right clicks
and hover tints of the cards and the expanded rows are held by pointer tests on a bare context
(`testing::Screen`) and seen in pictures, not under the owner's mouse. The sparklines over the
real ledger were not seen either: the pictures draw canned and synthetic series (`analytics`,
`analytics=all`), while the real ones take done from each project's history import and total
from the daily counts kept since EP-25 (2026-11-11), so for most projects the total runs flat
back to the first completion and steps only where a count changed since; the session expects
them to hold because the series come from EP-26's `burn_up`, which the Detail pane draws from
the same read. A card's height settles one pass after its content changes (a resize, a filter,
a read of the files), so one frame may show the cards of a line unequal; a pass follows every
input, so it should not be seen. The portfolio-wide read adds every project's daily counts to
the minute's read (a few rows a project a day), which the session expects to stay cheap.

> **Completion note (2026-11-13; claude-opus-5-5/max; about 50 min of wall time, a headless run
> started from the window, no question asked).** Ledger row `fbb829f6`, started 14:23; the
> prompt was the opener. Built as scoped.
>
> **Built.**
>
> 1. **One read for the graphs**: `Ledger::snapshots_all` (every project's daily counts by
>    project, with a test); `analytics::Loaded` holds them all and no longer names a project,
>    `Analytics::refresh` reads a minute apart or when invalidated, asked by the Detail pane
>    for a selected project and by the list while a row is expanded or the cards show.
> 2. **The sparkline**: `analytics::sparkline_points` (stepped polylines in a rect) and
>    `analytics::sparkline` (painted, hover only; done in green over a tint, total in the text's
>    colour; the hover text says done of total, the first day and the total's last change).
> 3. **The expanded row**: `phases` (each phase on a line, its name wrapping, the whole line
>    wrapping under 300 points), the sparkline beside the phases when the row has room for both,
>    the phases' column at least 360 points so that the sparklines line up, else under them, or
>    a weak line saying why there is none; the expansion in a scope as wide as the row that
>    senses the click under its unselectable labels, so a click selects, a double click opens
>    and a right click opens the row's menu, the row and the expansion under one tint painted
>    once both hovers are known.
> 4. **Expand all and Collapse all** (`View::expand_all`, `collapse_all`, `can_expand_all`):
>    icon buttons in the toolbar and items of the View menu (renamed from "Expand every row"
>    and "Fold every row"); Expand all takes the projects the list shows through its filters.
>    The toolbar wraps in a narrow pane.
> 5. **The cards view**: `View::cards` (saved with the view, off in a view saved before, with a
>    test), the Cards switch in the toolbar and the View menu; `cards_per_line`, `card`,
>    `card_frame`, `project_card`, `folder_card`; a line's height is the tallest of its cards'
>    own heights kept from the pass before in egui's temporary data, a moved height asking for
>    the pass again; the keys → and ← leave the cards view alone.
> 6. **`Drawing`**: the view, the scroll, the clock, the day and the read, passed to the rows and
>    the cards as one, in place of the first attempt's two `#[allow(clippy::too_many_
>    arguments)]`.
> 7. **Screenshot flags** `cards`, `expand=all`, `analytics=all` (`registered_names`,
>    `synthetic_history`, `synthetic_snapshots`); `analytics` cans lantern's counts too.
> 8. **Docs**: the D-25 addendum, the README's words on the list, the roadmap row linked.
>
> **From the first attempt.** The failed run's diff (`target\tmp\ep26a\diff.patch`, kept as
> found) applied cleanly to EP-40's tree and was taken as the starting point after a review:
> its one portfolio-wide read, its painter sparkline and its cards stood; its two `#[allow]`s,
> its non-wrapping toolbar, its cards of unequal heights and its expansion without a click were
> fixed as the brief said, and one of its tests did not compile (a path's stroke is a
> `ColorMode`). The pictures then found four layout faults no test caught, each fixed and
> pictured again: a card's content laid out left to right (its scope took the line's layout:
> now top to bottom); the cards of a line measured in a sizing pass, which aligns differently
> from the drawn card (now each card's own height, kept a pass); `set_min_height` after the
> content, which egui measures from the cursor and so doubled each card (now before); and the
> badges sinking one after another on a card's line, since a child `Ui` per cell advanced the
> wrapped line even for a cell that draws nothing (now the cells go straight into the line,
> the spacing put back after each).
>
> **Checked by eye** (pictures in `target\tmp\ep26a2\`, ignored). Over the fixtures: the cards
> at 1600 light with harbor's and lantern's sparklines, the lines of equal height and the badges
> level; the cards at 1100 dark, the text wrapping, the toolbar on two lines; every row expanded
> at 2000 light, the sparklines beside the phases and lined up, the selected row's tint over its
> expansion; the same at 1100 dark, the phase names wrapping. Over a copy of the real roadmaps
> (twelve non-data projects, `scripts\copy-roadmap.ps1` one call at a time, `CRW_SCREENSHOT_
> ROOT`, a fresh `CRW_HOME` per run): the cards at 1600 with `analytics=all`, the long titles
> cut at two rows with an ellipsis; every row expanded at 1200 dark, the long phase names
> wrapping under themselves and nothing past the pane's edge. `git status -- crates/app/
> fixtures` was clean after the runs.
>
> **Review (rule 6).** A subagent given the diff alone (launched at the first clean compile,
> back in about 24 minutes; it read the tree as it changed) confirmed the three layout faults
> the pictures had found and their fixes, found the clicks, the menus, the card widths, the
> reads and the Detail pane sound, and made seven smaller points, all fixed: (1) a card's height
> settled a pass late with nothing to ask for that pass, so an idle window could keep a line
> unequal: a moved height now asks for the pass again (`request_discard`); (2) the expansion
> answered only over its content's width: it now spans the row; (3) the marks after a cell that
> sets its own spacing crowded: the line's gap is put back after a cell that drew; (4) the card
> test's green line could pass empty: harbor gets a history there and the check wants two points;
> (5) a series without daily counts said "of 0", and a failed read said "no burn-up yet": both
> worded as they are; (6) a phase line could not narrow below about 230 points: under 300 it
> wraps as a whole; (7) → and ← expanded hidden rows in the cards view: they now leave it alone.
> The screenshot doc's `analytics=all` now skips every project with canned counts, as it says.
>
> **Verification.** `cargo test --workspace` (180 in `app`) and `cargo test --workspace
> --features screenshot` (188 in `app`, 378 in `core`, the other suites as before) pass; both
> clippy runs and `cargo fmt --check` are clean; rule 3's word check on the staged diff printed
> nothing. The release build and the install (`scripts\install-shortcut.ps1`) run after the
> record commit, the opener's last step; the restart is refused while this run works and taken
> from File > Install and restart, the tray's, or the launch dialog's.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> No other owner decision arose: the scope is the brief note's, nothing outward-facing, no inbox
> message, no owner task.
>
> **Routine choices (the session's call, logged).**
>
> 1. **The first attempt's diff as the starting point**, reviewed and fixed, rather than a
>    rewrite: it applied cleanly and its design held.
> 2. **One portfolio-wide read** rather than a second read for the list: the snapshot table is
>    small, and the Detail pane's switch of project no longer reads the ledger again.
> 3. **A card's height kept from the pass before** rather than measured first: one draw per
>    card, and the drawn layout is the measured one.
> 4. **Expand all through the filters**: the rows the owner sees; Collapse all folds every row.
> 5. **The sparkline painted**, not an `egui_plot` plot per row, which would take the row's
>    pointer; the heatmap stays in the Detail pane.
> 6. **No autopilot box or Start on a card**: the brief note's list; the keys and the row's menu
>    act on the selected card.
> 7. **The scratchpad** is `target\tmp\ep26a2\` (ignored): the diff, the logs, the copy, the
>    pictures, the commit messages.
>
> **Owner task:** none. No message for another project. **Next: EP-27** (exports), chained by
> the autopilot when its gate allows, else from the window. The work commit is `7f1778b`; the
> record commit follows.
