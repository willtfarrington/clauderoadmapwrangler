---
ep: 41
title: Fixes from the P4 walk, the daily surfaces — a chain's own five-hour mark, the expanded rows' sparklines on one left edge, the daily export at the day's turn with 14 days and each month's first kept
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-41 — Fixes from the P4 walk, the daily surfaces

**Size:** M · **Depends on:** EP-40 (the five-hour mark), EP-26a (the expanded rows' sparklines),
EP-27 (the daily export) · **Decisions:** D-22, D-25, D-8 (their addenda of 2026-11-15)

## Context

The P4 walk (EP-29) found three things wrong or missing on surfaces the owner uses every day,
and the owner chose each fix there:

1. **One five-hour mark holds every chain.** `settings.autopilot_five_hour_percent` (default 50)
   is the only mark the gate weighs the session (five-hour) window against. The owner wants to
   set it higher for one chain, for instance one left to run overnight, without changing the
   standing setting (D-22 addendum).
2. **The expanded rows' sparklines start raggedly.** Each sparkline sits beside its row's phases
   block, which has a shared minimum width (`PHASES_AT_LEAST`, 360) and grows with a long phase
   name, so under Expand all the sparklines start at different places (D-25 addendum).
3. **The daily export skips days and keeps everything.** It is written at the first full read of
   the files in a day, so on 2026-11-14 the window stayed up without one and wrote no export; and
   every day's folder, a whole copy of the ledger, is kept for ever (D-8 addendum). EP-25's daily
   counts share the gap: they are kept at a full read and at a one-project re-read, so a project
   nobody worked in on such a day has no count for it.

**Session start (2026-11-15, 17:42 local time).** Started from the window as a headless run (the
rolling log's "Started clauderoadmapwrangler EP-41 headless with claude-opus-5-5 at max effort" at
17:42:52); the prompt is the tool's opener, and `crw sessions clauderoadmapwrangler` lists the row
(`8814ff81`, headless, EP-41, claude-opus-5-5/max, open). The autopilot is on for this project, so
the run has M's cap of 1 h 30 min of working time, a question's wait left out. No owner task and no
inbox message is open. The field check read, since EP-29's record commit (16:12):

- `crash.log`: the one report of 2026-11-03, nothing newer.
- `crw notifications`: EP-29's waits and its finish at 16:13, nothing since.
- `crw sessions`: this row; the manual row of 13:23 still open beside it (the idle tab EP-29
  named; the next start's reconciliation closes it once no listing names it).
- `crw autopilot`: not paused, on for this project and five others, all in mode `auto`; the last
  line is the chain's end after EP-28a ("EP-29 is a re-plan, the owner's to run").
- The rolling log: EP-29's install asked for the restart at 16:13:41 and the window took it at
  once (its visual session's turn was over); at 17:42:13 this project was read again after
  EP-29's session ended; this run started at 17:42:52.

Nothing odd to triage.

## In scope

1. **A chain's own five-hour mark** (D-22 addendum).
   - `crw_core::autopilot::Plan` gains `five_hour_mark: Option<u8>`, the chain's own mark (none:
     the setting's, read when the gate weighs), with `#[serde(default)]` so that the handover's
     frozen version-1 form still reads and the mark travels across the window's restarts with the
     plan; `Plan::mark(setting)`; `Plan::words` names an own mark.
   - The gate weighs a chain's own mark: `Ask` gains the mark the chain holds at, `read` puts it
     into `Facts::five_hour_percent` with `Facts::five_hour_own`, and the session window's
     `Refusal::Usage` says "at or over this chain's mark at n %" instead of "at or over the pause
     at n %", so the hold's words in the chain log name the chain's mark.
   - The launch dialog's Chain plan: "Hold this chain at [n] % of the session window", a picker
     filled from the setting (its entry marked "the setting"), kept with the plan at Start
     headless and gone when the chain ends, as the plan is. The dialog's five-hour notice weighs
     the mark the dialog shows.
   - The Sessions pane's Autopilot section: each kept plan's line shows its chain's mark with the
     same picker; a change is the chain's from then on, a held chain's gate is read again at once,
     and the change is a line of the chain log.
   - The chain log names an own mark: a line when a run whose plan holds its own mark starts, the
     chained line's end, the hold's words. `crw autopilot` prints the log, so it prints the mark,
     and its first line says that a chain's own mark wins.
   - A chain with no plan, or a plan left at the setting, holds at the setting, which stays the
     default.
2. **The sparklines on one left edge** (D-25 addendum). `list::expansion` gives the phases block
   the row's width less the sparkline's, so each sparkline is set from the row's right and every
   expanded row's starts at one place; `PHASES_AT_LEAST` goes. A test draws two expanded rows, one
   with a long phase name, and compares the captions' places; a screenshot run over a copy of real
   roadmaps with every row expanded checks it on real content (rule 5).
3. **The daily export's day and its keep** (D-8 addendum).
   - The window notes the local day; when it turns while the window is up (or the machine wakes in
     a new day), the next pass reads every project's files again, which keeps the day's counts for
     every project and writes the day's export: one moment serves both, so EP-25's gap closes too.
   - After each write, the window's and `crw export daily`'s, `export::prune` removes the tool's
     own dated folders under the export folder (named as `export::day_folder` names them) older
     than 14 days, the earliest folder of each month kept; in a removed folder only the files the
     tool writes there (`registry.toml`, `ledger.json`, the CSVs), then the folder itself if it is
     empty; one folder at a time with a pause between folders, never a burst (rule 9); nothing else
     in the export folder touched.
   - A failed write or removal is a line of the rolling log (`crw_core::log::line`), where EP-27
     left it silent; the export runs after the files read is handed to the window.

## Out of scope

- The five-hour notifications, a five-hour line in the usage sparkline and a predictive gate:
  parked by the owner at the walk (v2 notes).
- A per-chain hold in the tray's line and the Autopilot section's header chip: both keep saying
  what the setting holds; a plan's own line says what its chain holds at.
- An editor for the standing setting in the window (`crw setting autopilot_five_hour_percent`
  serves), and any new registry key: the chain's mark lives in the plan, never in the registry.
- The exports in the menus (the owner's choice at the walk: the commands serve).
- The README's pictures (EP-32's close-out) and the push (the owner's).

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings` and
  `cargo fmt --check` pass (rule 6); a review subagent given the diff and not this session's
  reasoning reads it before the release build (an M brief).
- Core tests: a plan's own mark reads from a handover written without it and round-trips with
  it; the gate holds at the chain's mark and passes under it where the setting would hold, its
  words naming the chain's mark; the prune keeps 14 days, each month's first and anything not a
  dated folder, and leaves a folder holding a file of the owner's.
- App tests: the dialog's picker sets the plan's mark and Start headless keeps it; the Autopilot
  section's picker changes a kept plan's mark and writes the log line; the gate reads the plan's
  mark; the day's turn starts a files read; two expanded rows' sparklines start at one place.
- Screenshot runs (rule 5): the launch dialog's Chain plan with the mark picker and the Autopilot
  section with a kept plan, over the fixtures; the list with `expand=all` and `analytics=all` over
  a copy of the real roadmaps in the scratchpad (the copy and the pictures never enter git).
- Rule 3's word check prints nothing for the staged diff; two commits in the house pattern, not
  pushed; then the release build and the install, last.

**Residual risk.** What the session could not exercise, and why it expects it to hold.
(1) **A real chain at its own mark.** No test or screenshot run can hold a real chain on the
real session window: the gate's weighing of a chain's mark is tested in core over synthetic
figures, the window's wiring (the plan's mark into `Ask`, the chained line, the start's line,
the section's change) in the app's tests, and the handover's carrying of the mark over the
frozen version-1 file. The first real use is the owner's next chain started with a mark of its
own; its log lines (`the chain's mark: …`, the chained line's end, a hold's words) say whether it
held where it should. (2) **The day's turn in the real window.** The turn is a date comparison
in a pass that runs at least every ten minutes, shown or hidden, and the read it starts is the
one every start makes; the first real one is tonight's midnight with the window up, which the
rolling log's `the day turned` and `export` lines of 2026-11-16 will show (or their absence:
the window may be quit or asleep). (3) **The keep on the real folder** first removes something
on 2026-11-29 (the 15th's folder; the 13th is November's first and stays). Its rule is tested
on a scratch tree with every case the brief names, and it removes only the files the tool
writes, then an empty folder, so a mistake would keep a folder rather than lose one of the
owner's files. (4) **The pickers under the mouse.** The dialog's and the section's pickers were
seen in screenshot runs and their effect tested below the egui layer; choosing an entry with
the mouse in the real window is egui's own ComboBox, as the section's other pickers are.
(5) **The sparklines on real content** were checked in a screenshot run over a copy of the
portfolio's roadmaps with every row expanded: one left edge for every row; a narrower pane puts
each sparkline under its phases, as before.

> **Completion note (2026-11-15; claude-opus-5-5/max; about 55 minutes).** A headless run the
> owner started from the window (ledger row `8814ff81`, from 17:42), under the autopilot's
> standing yes and M's cap; no question was put, since no decision here was the owner's.
>
> **Built.**
>
> 1. **A chain's own five-hour mark.** `autopilot::Plan::five_hour_mark` (none: the setting's),
>    carried by the handover in version 1's form and kept by `Plan::keep_choices`;
>    `Ask::five_hour_mark`, `Facts::five_hour_own` and `Refusal::Usage::own`, so a hold's words
>    say "at or over this chain's mark at n %"; `mark_words`, `own_mark_words` and `MARKED` for
>    the log; `session_notice` weighs the dialog's own mark. In the window: one `mark_picker`
>    (the setting's entry first, then 30 to 90 %) in the launch dialog's Chain plan ("Hold this
>    chain at [n] of the session (five-hour) window") and under each kept plan in the Autopilot
>    section (`Action::SetChainMark`, `Board::set_chain_mark`); the gate's read takes the plan's
>    mark; the chained line ends with it; `Board::started_by_hand` logs it when a run whose plan
>    holds one starts. `crw autopilot`'s first line says a chain's own mark wins and the log
>    names it.
> 2. **The sparklines on one left edge.** `list::expansion` gives the phases block the row's
>    width less the sparkline's when the two sit side by side; `PHASES_AT_LEAST` is gone.
> 3. **The daily export's day and its keep.** `Loader::day_turned` and a read of every
>    project's files from `Board::receive` when the local day turns (the counts and the export
>    of the new day at one moment, so EP-25's gap closes too); the export runs after the read
>    reaches the window, one at a time; `export::prune`, `due_for_removal` and `remove_day` keep
>    the last 14 days, today's included, and each month's earliest folder, and remove only the
>    tool's own files (a half-written `.tmp` included), then the folder, one folder at a time
>    with half a second between two; `crw export daily` prunes too; each write, removal and
>    failure is a line of kind `export` in the rolling log.
>
> Addenda to D-8, D-22 and D-25 (as built); the README's two sentences and the CLI's usage line.
>
> **The review (rule 6).** A subagent given the diff and this brief, not the session's
> reasoning, reported after about sixteen minutes, with seven points:
>
> 1. *Fixed.* A chain held in a project was still listed when the owner started that project's
>    next brief by hand from the dialog: its next read counted the new run as a busy session,
>    ended the chain and dropped the new plan, its own mark with it (the root predates EP-41).
>    Now the held chain gives way when the owner's run starts, logged as its end ("you started
>    EP-n headless yourself"), and the new plan stays.
> 2. *Taken in part.* Tests for the board's day-turn branch and for the section's action
>    through `Board::run` (now `pub(crate)`) were added. The plan's mark reaching a real gate
>    read is not tested in the window, since a test reaching `read_gate` would run the CLI's
>    preflight (rule 5); the core tests cover the gate's weighing and the field is a required
>    one of `Ask`. The pickers' mouse interaction is egui's ComboBox (residual risk 4).
> 3. *Fixed.* `set_chain_mark` returned `true`, which tells `run_actions` a file was written and
>    starts a read of every project's files; it writes none, so it returns `false`.
> 4. *Fixed.* A mark changed while that project's gate was being read could wait five minutes
>    for the next read; such a change now asks for a read after the result (`recheck_after`). A
>    chain that passed at the old mark in that second or two still starts (logged here).
> 5. *Fixed.* A half-written `.tmp` of the tool's kept its folder for ever; it counts as the
>    tool's now, and a folder holding anything else is left untouched whole, looked through
>    before any file in it is removed.
> 6. *Fixed.* The Autopilot section's "The chains hold: …" was untrue for a chain with a higher
>    mark of its own; the session window's line now reads "Chains without a mark of their own
>    hold: …". The header's chip and the tray keep saying what the setting holds (out of scope).
> 7. *Logged.* The brief's In scope said `Plan::words` would name an own mark; it does not,
>    since the section's picker sits under the plan's line and says it.
>
> A minor point was logged too: the start's mark line takes the open dialog's project, so a
> resumed run starting while another project's dialog is open would log that project's chain
> mark (the dialog closes then, as it did before).
>
> **Verification.** `cargo test --workspace`: 193 in `app`, 430 in `core` and the CLI's, all
> passing; `cargo test --features screenshot`: 201; both clippy runs with `-D warnings` and
> `cargo fmt --check` clean. Screenshot runs, kept in the scratchpad `target\tmp\ep41\` and out
> of git: the list over a copy of the twelve non-data projects' roadmaps with every row expanded
> and synthetic analytics (one left edge for every sparkline, long phase names wrapping in their
> block), the launch dialog's Chain plan with the mark picker, and the Autopilot section with a
> kept plan at 70 %. `crw roadmap .` reads the brief, no problem. Rule 3's word check printed
> nothing for the staged diff. Then the release build and the install, last.
>
> **Owner decisions.** None arose. The commit is the autopilot's standing yes (D-22); the push
> is the owner's.
>
> **Routine choices (the session's call, logged).**
>
> 1. **The sparkline set from the row's right** over the widest phases block measured each pass:
>    the brief allowed both, and with real phase names the two put the edge in the same place,
>    while a measuring pass is where egui's wrapped labels have misled before (EP-26a).
> 2. **Fourteen days, today's included,** as the rolling log counts its days; a folder of the
>    15th is removed on the 29th.
> 3. **The mark as `Option<u8>`**: a plan left alone follows the setting even if the setting
>    changes while its chain runs, as a brief left alone follows the rules.
> 4. **The marks offered**: the setting's entry, then 30 to 90 % by tens; a mark set another
>    way is offered as it stands.
> 5. **`crw autopilot` reads the mark from the log**, since the plan lives in the window: no new
>    record and no inference from the log's lines.
> 6. **The day's turn as a full read** of the files, which every start makes, rather than an
>    export alone: the counts and the export then share one moment.
> 7. **A ledger that does not exist yet** writes no export and logs nothing.
>
> No owner task, and no message for another project. **Next: EP-42**, the two guards.
