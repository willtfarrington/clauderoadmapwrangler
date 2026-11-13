---
ep: 26
title: Analytics, the graphs
size: M
model: claude-fable-5-1
effort: high
status: done
---

# EP-26 — Analytics, the graphs

**Size:** M · **Depends on:** EP-25 · **Blocks:** EP-26a, EP-29 · **Decisions:** D-25, D-26, D-15

## Context

D-26 promised three graphs over the history the backfill (EP-25) and the ledger keep: an activity
heatmap (sessions and completions per day, per project and for the portfolio), a burn-up per
project (done and total over time, a re-plan a jump in the total) and velocity with a projection
(EPs per week, the remaining session-hours from the size mix and the measured durations, the
finish at the measured pace). The P3 re-plan (EP-23) cut the original EP-26 in two: this brief
keeps the graphs in the Detail pane, pins `egui_plot` 0.38.0 (cleared in the D-15 addendum of
2026-11-11) and decides the measure of a brief's time; EP-26a takes the row expansion, the
sparkline and the cards. The brief note also asks that the Progress section wrap its long phase
names and that the history list show each EP's model and effort from the new completion-note form.

**Session start (2026-11-12).** The session's prompt was this project's opener, launched from
clauderoadmapwrangler with fable at high effort as a chained headless run after EP-25a; `crw
sessions clauderoadmapwrangler` lists its row (`909387f9`, headless, EP-26, started 18:58, M's cap
of 1 h 30 min). Read: the roadmap's P4 table and EP-26's and EP-26a's brief notes, D-25, D-26 and
D-15 with their addenda, EP-25a's completion note, `owner-tasks.md` (no open task) and the inbox
(no open message). The field check: `crash.log` is still EP-14a's of 2026-11-03, seen; `crw
notifications` shows EP-25's finish and the chain's end at the day's cap (22:55 yesterday) and
nothing since; `crw sessions` shows EP-25a's run completed (19 min) and this run open; `crw
autopilot` shows six projects on and the chain log holding EP-26 after EP-25a's install, then
chaining it in the new window. Nothing odd. The working tree was clean at the start.

**The code as found.** The ledger (schema 7) keeps `session` rows (project, EP, mode, model,
effort, start, end, outcome, questions and the mean answer latency), `snapshot` rows (done and
total per project once a day and at each change, since EP-25), `completion` rows (one per EP with
its date, read into `history::Summary` on each `ProjectRow::history`) and `event` rows, among them
EP-15b's `revert` events whose detail opens with `session <short id> reverted`. The Detail pane
(`detail::project`) draws its sections in order, the History section after Progress; the Progress
section is an egui `Grid`, which does not wrap. The meter's `Week` reads the ledger on the UI
thread with `Ledger::open_read_only` when its inputs move, and its sparkline is painted by hand.
The sizes of a roadmap's rows are `Row::size` (S, M, L); the brief parser reads a completion note's
model, effort and duration (`brief::Completion`). A screenshot run cans a history for harbor with
the flag `history`. The cargo cache holds no `egui_plot`: the build fetches 0.38.0 once. Read
after the fetch: `egui_plot` 0.38 has a `Heatmap` item, but its hover gives a cell's value
alone, not its day, and its `Line` aggregates points that share an x into a band unless
`allow_band(false)` is set (a stepped line's two points per step do).

## In scope

1. **The pin**: `egui_plot = "=0.38.0"` in the workspace (D-15 addendum of 2026-11-11), the app
   crate using it; `Cargo.lock` updated by the fetch alone, the family otherwise untouched.
2. **`app::analytics`**, the data: `Loaded` for one project read from the ledger
   (`Ledger::open_read_only`: every `session` row, the project's `snapshot` rows and the `revert`
   events), kept on the board and read again when the selection changes, when a minute has
   passed, or after a read of the files; a screenshot run's is canned. From it and the row's
   history: the **days** of the last 16 weeks (sessions per local day, grouped by session id so a
   resumed run counts once, D-26 addendum of EP-35; EPs completed per day from the history), for
   the project and for the portfolio; the **burn-up** series (done from the completions'
   cumulative count, total from the snapshots, flat before the first snapshot and stepped between
   rows, both carried to today); the **velocity** (EPs per week over the last 8 weeks, weeks from
   Monday, local); the **measure of a brief's time** and the **projection**.
3. **The measure** (the brief's decision, from the note of EP-16 and EP-23): one sample per
   session id (a session has several rows: a visual launch's, its own, a resume's; the review's
   finding), from its earliest row's start to its latest row's end, when an EP is named, every
   row has ended, the latest ended `completed` and the session was not reverted (no `revert`
   event of the project naming its short id). A headless session's time takes the owner's waits
   off (`questions × answer_latency_s` per row, about: the row keeps a mean). A session with a
   visual or manual row counts only when under 3 hours (one and a half times an L's planned
   time), since a tab left open runs its row for a day; it counts its waits in and is marked as
   wall time. The size of each session's EP is the roadmap row's; the median per size is the
   project's own when it has three measured sessions of that size, else the portfolio's, else
   the plan's S 30, M 60 and L 120 minutes, and the words say which. The retros' figures (an S 34
   min, an M 42 to 54, an L about an hour of active work) are the check: this project's EP-25a
   (19 min) and EP-25 (40 min) read within them.
4. **The projection**: the open rows' size mix × the median per size gives the remaining
   session-hours; the pace is EPs per week over the last 8 weeks, the finish shown as a range from
   the pace over the active weeks (the early bound) to the pace over all 8 weeks (the late bound),
   as dates; with no completed EP in the 8 weeks, the words say the pace is unmeasured.
5. **The Detail pane**: a new **Analytics** section after History with the heatmap (16 weeks in
   columns, the weekdays in rows, a cell's fill by its sessions, a mark for a day with a completion,
   each cell's hover text saying the day, its sessions and its EPs; a switch between this project
   and the portfolio), the burn-up (an `egui_plot` plot: two stepped lines, done and total, a
   legend, no drag or zoom), the velocity (an `egui_plot` bar chart of the 8 weeks) and the
   projection's sentences (remaining hours from the mix, the medians and their source, the finish
   range). A project without a history or a ledger says so in one weak line. The heatmap is
   painted (a grid of cells whose hover text names the day, which `egui_plot`'s `Heatmap`
   item cannot: its hover gives the value alone); the two plots are `egui_plot`'s.
6. **The history list**: under the History section's lines, a collapsible list of the dated EPs,
   newest first, each with its date, the completion note's model, effort and duration where the
   new form gives them, and the ledger's measured time where a row gives it.
7. **The Progress section** rebuilt without a `Grid`: each phase a line whose name wraps under
   itself in a narrow pane.
8. **The list's expanded row** says the sparkline arrives at EP-26a (the heatmap is in the pane).
9. **Tests**: `analytics` unit tests (the days' grouping by session id, the measure with a
   headless row's waits taken off, a reverted row and a long visual row left out, the medians'
   fallbacks, the burn-up series from completions and snapshots, the velocity's weeks, the
   projection's range) and a Detail pane test (the section with canned data shows the projection's
   words and the EPs list shows a model and effort). The screenshot flag `analytics` cans the
   data for harbor; a run with `select=harbor,tile=detail,history,analytics` checked by eye.
10. **Docs**: a D-26 addendum (the measure and the graphs as built), a D-15 line (the pin taken),
    the README's Detail pane paragraph, the roadmap row linked, the completion note.

## Out of scope

- The inline row expansion's sparkline, Expand all / Collapse all and the cards (EP-26a).
- The usage sparkline moved to `egui_plot` (the pins stay; a re-plan's call).
- Waits on the owner in visual sessions counted from the watch's busy and idle (the ledger keeps
  none; the rows' answer latency covers headless runs): a later brief, noted in the D-26 addendum.
- A usage-over-time graph (parked in v2, D-26).
- A CLI print of the analytics (`crw history` prints the dates already).

## Verification / acceptance

1. `cargo test --workspace` and `cargo test --features screenshot` pass; both clippy runs and
   `cargo fmt --check` are clean; rule 3's word check on the staged diff prints nothing.
2. The tests of item 9 pass; the measure's unit test puts a headless row with two questions at
   a 60 s latency at its run length less 2 minutes.
3. A screenshot run over the fixtures with `select=harbor,tile=detail,history,analytics` shows
   the Analytics section's heatmap, both plots and the projection's words, checked by eye at 1600
   light; a run over a copy of a real roadmap (`CRW_SCREENSHOT_ROOT`, rule 5) shows the Progress
   section's phase names wrapping in a narrow pane.
4. A review of the diff by a subagent given the diff alone (rule 6, an M brief), its findings fixed
   or logged.
5. The release build and the install run last (`scripts\install-shortcut.ps1`); the restart is
   refused while this run works and taken at the gate's moment or from File > Install and restart.

**Residual risk.** The real ledger's rows run back to EP-9 (2026-11-01) with the modes and
outcomes of their day: the visual rows of P2 left open for a day drop out of the measure by the
3-hour ceiling, as meant, and a headless row from before EP-14 has no latency and counts whole.
The measure is checked against the retros' figures on this project's rows alone; the P4 walk looks
at the medians the pane shows for the other projects, and at the heatmap's portfolio switch over
the real ledger, which no test reads. The session expects the graphs to hold because the series
are built from the ledger's types in unit tests and drawn from the same types in the pane. The
Progress section's wrap was seen on the fixtures' picture (short names, the same wrapping
`Label`); over the real-roadmap copy a narrow pane could not reach the section in a picture,
since this project's brief notes fill 2160 points of height and a screenshot run cannot scroll:
the walk looks at it in the window.

> **Completion note (2026-11-12; claude-fable-5-1/high; about 50 min of wall time, a chained
> headless run from the window, no question asked).** Ledger row `909387f9`, started 18:58; the
> prompt was the opener.
>
> **Built.**
>
> 1. **The pin**: `egui_plot = "=0.38.0"` in the workspace and the app crate; the lock gained
>    the one crate (D-15 addendum).
> 2. **`app::analytics`**: `Loaded` (every session row, the project's snapshots, the `revert`
>    events' short ids) read with `Ledger::open_read_only`; `Analytics` on the board (refreshed
>    at a selection change, a minute later, after `Action::Refresh` and a one-project read's
>    notice; a screenshot run's `put`); `days` (16 weeks from Monday, sessions grouped by id,
>    the chat's turns and lapsed launches left out), `burn_up` (done from the completions, total
>    from the snapshots, flat before the first and carried to today), `velocity` (8 weeks),
>    `measure` (one sample per session id, see below), `medians` (own with 3 rows, else the
>    portfolio's, else planned) and `Projection` (the open rows `☐` and `[~]` by size at the
>    medians; the finish from the active weeks' pace to all 8 weeks' pace); the section: the
>    painted heatmap with hover text per day and the project/portfolio switch, the burn-up and
>    velocity `egui_plot` plots (no drag, zoom or scroll; `allow_band(false)` on the stepped
>    lines, which egui_plot otherwise draws as bands), the projection's sentences and a small
>    line saying the measure.
> 3. **The Detail pane**: the Analytics section after History; the History section's folded
>    list of the dated EPs with the completion note's model, effort and duration (the last note
>    giving them) and the ledger's time (`19 min (the ledger)`, `wall time`); the Progress
>    section laid out by hand with its names wrapping.
> 4. **The list's expanded row** says the sparkline arrives at EP-26a; the screenshot flag
>    `analytics` (canned rows for harbor over six weeks, lantern's for the portfolio switch,
>    counts that jump).
> 5. **Docs**: the D-26 and D-15 addenda, the README's paragraph, the roadmap row linked.
>    Tests: six in `analytics` (the days, the burn-up, the velocity, the measure with the
>    medians, the reverted id, the projection, the read's minute) and one in `detail` (the
>    section's words and the EPs list's measured time).
>
> **The measure, as decided.** One sample per session id from the earliest row's start to the
> latest row's end: the review found that a session has two or three rows (a visual launch's,
> its own, a resume's), each closed `completed` together, so one brief gave two or three
> samples; grouped now. A headless session's waits come off as `questions × answer_latency_s`
> (about: a mean, a resume's wait inside it); a visual or manual session counts under 3 hours
> as wall time; a reverted session is left out. Checked: this project's EP-25a (19 min, S) and
> EP-25 (40 min less 43 s, M) read within the retros' figures.
>
> **Checked by eye.** `target\tmp\ep26\analytics-tall.png` (`light,history,analytics,select=
> harbor,tile=detail,size=1600x2200`): the heatmap with its month labels and dots, the two
> plots, the EPs list folded; the first picture showed the stepped lines as bands, fixed. Over a
> copy of this project's roadmap (`scripts\copy-roadmap.ps1`, `CRW_SCREENSHOT_ROOT`) at 1200
> wide, the Progress section lies below the brief notes and out of the picture (see the residual
> risk). The copies and the pictures never enter git.
>
> **Review (rule 6).** A subagent given the diff alone found five things: (1) several samples
> per session, fixed by grouping; (2) the waits are a mean × count, not a sum, accepted and
> said as "about" in the caption and the addendum; (3) `◐` counted as open in the projection,
> fixed (`☐` and `[~]` are open); (4) an appended completion note hiding the original's
> figures, fixed (the last note that gives them); (5) the measure and the sizes computed every
> frame, fixed (kept on the board per read). The bucketing, the plots' API use, the painter
> code, the borrows and the panics it checked and found sound.
>
> **Verification.** `cargo test --workspace --features screenshot`: 182 in `app`, 375 in `core`,
> the other suites as before; both clippy runs and `cargo fmt --check` clean; rule 3's word
> check on the staged diff printed nothing. The release build and the install
> (`scripts\install-shortcut.ps1`) ran after the record commit, the opener's last step; the
> restart is refused while this run works and taken at the gate's moment, or from File >
> Install and restart, the tray's, or the launch dialog's.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> No other owner decision arose: no scope change, nothing outward-facing, no inbox message, no
> owner task. The pin was cleared at EP-23 (D-15 addendum).
>
> **Routine choices (the session's call, logged).**
>
> 1. **The heatmap is painted**, not `egui_plot`'s `Heatmap` item, whose hover gives a value
>    alone; the brief said "with egui_plot" for the graphs, and the two plots are its.
> 2. **The ledger is read on the UI thread**, as the meter's week is: three small queries at a
>    selection change or once a minute.
> 3. **The portfolio switch** moves the heatmap alone; the burn-up, the velocity and the
>    projection stay the project's, as the brief note frames them.
> 4. **The EPs list is folded** by default: the pane is long already.
> 5. **The ceiling is 3 hours** (one and a half times an L's plan), under which every retro's
>    session falls and over which the P2 tabs left open fall.
> 6. **The scratchpad** is `target\tmp\ep26\` (ignored): the pictures, the copy, the logs, the
>    commit messages.
>
> **Owner task:** none. No message for another project. **Next: EP-26a** (analytics in the
> list), chained by the autopilot when its gate allows, else from the window.
