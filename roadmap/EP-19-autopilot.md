---
ep: 19
title: Autopilot
size: L
model: claude-opus-5-5
effort: max
status: done
---

# EP-19 — Autopilot

**Size:** L · **Depends on:** EP-14, EP-17 · **Blocks:** EP-19a, EP-23, EP-30

## Context

D-22: a per-project toggle; after a headless run, a **gate** decides whether the next brief is
chained; with the autopilot on, the prompt tells the session to commit in the house pattern and
record the hash itself (the owner's standing yes); with it off, the session stops before committing
and the tab's **Commit** button resumes it with that instruction; **caps** per run (a cap by size,
the CLI's budget flag) and per day; a **pause** at 90 % of the weekly or Fable limit until the
reset, and a global Pause in the tray and the window; a **chain log**.

**What exists.** The registry's `[project.autopilot]` holds `enabled` only (EP-2), and the list's
autopilot box, the A key and the tray's Pause autopilot wait for this brief, disabled. A headless
launch is `prompt::draft`, then the `session::Launch` that `launch::Dialog::launch` builds, started
by `Board::start_headless` with the route, the hooks, the revert's start and the answer timeout;
the window holds and drains its runs (`app::sessions`), resumes an ended one under its id
(`Sessions::resume`), and stops one gracefully (`Session::stop`, the interrupt, then the launch's
`stop_timeout`). The meter's figures are `usage::Gauge::figures(now)`, each with its `source` and
`at`, counting until its reset (`Limit::before_reset`, never `is_active`). The runner passes
`--max-budget-usd` when `Launch::max_budget_usd` is set and the CLI lists it (2.1.316 does). The
ledger's `event` table takes any kind of line. `receive` and `attend` run in eframe's `logic`, so
the window's passes go on while it hides in the tray.

**Session start (2026-11-05, 23:11 UTC).** Launched from the window as a visual launch (rule 11):
the ledger row `e8fece6e`, visual, EP-19, open (`crw sessions clauderoadmapwrangler`). The opener
suggested Fable at xhigh. The owner runs it at Opus 5.5 with max effort, chosen in the tab's panel.
An earlier attempt at this brief (row `ca6a5fc7`, Fable 5.1, 12 minutes) was stopped by an opaque
safety guard and its work reverted by the owner, so this session starts from the same tree: clean
at `b6bbcf2` but for the owner's tick of OT-17. No owner task and no inbox message is open.

**Decided at pickup (owner, through `AskUserQuestion`).**

1. **Split** (D-4): EP-19 builds the engine and the chain. A new **EP-19a** (S), right after it,
   takes the owner's model and effort for each brief before a chain starts (the launch dialog
   lists the briefs a chain may run next, each with its pickers, the rules' choice the default)
   and the chain's own notification when it stops.
2. **This project is never chained.** The gate refuses the project whose folder holds the running
   window's exe, since its briefs end with a release build that needs the window quit. Its briefs
   run visually as today.
3. **Runs that would wait are refused.** The gate refuses a project whose runs would wait at each
   shell command: its permission mode is acceptEdits, manual or plan, and no allow rule is found
   in its `.claude` settings or the user settings. The owner opts a project in with `permission_mode
   = "auto"` under its `[project.launch]`, or with allow rules.
4. **A cap counts working time**: the time a request waits for the owner is left out; a project's
   `answer_timeout` still defers a long wait.

## In scope

1. **The registry** (`crw_core::registry`). `[project.autopilot]` gains `max_size` (S, M or L;
   M by default), `daily` (chained runs a day, 2 by default) and `max_budget_usd` (none by
   default). `[settings]` gains `autopilot_pause_percent` (90), `autopilot_daily_total` (4) and
   `autopilot_paused`, written only as `true`. The toggle is written as `autopilot = { enabled =
   true }` and taken out when off, so a registry with no autopilot on reads in the builds before.
   `crw setting` takes the three settings.
2. **The gate** (`crw_core::autopilot`), a pure function over the facts, naming every condition
   that fails: the autopilot on; not paused by the owner; the run completed; its brief has a
   completion note; its row is checked (the next brief is another); a next brief; no open owner
   task gated before it; not a re-plan (`Re-plan` in its title); its size within `max_size`;
   weekly and Fable under `autopilot_pause_percent`; a clean tree; the day's chained runs under
   `daily` and `autopilot_daily_total`; no other session open in the project; not the tool's own
   project; runs that would not wait at each command. A figure at or above the mark counts until
   its reset whatever its age; one under it counts only within 15 minutes of its reading; a figure
   never read (Fable while the endpoint is off) refuses.
3. **The commit variant** (`crw_core::prompt`). For a project whose autopilot is on, rule 5 of the
   session rules says to commit in the house pattern and record the hash, and never to push; else
   it stops before committing as today. The Commit button's message is the same instruction.
4. **The chain** in the window. When a window's headless run ends and its project's autopilot is
   on (or the run was chained), the gate is read on a worker thread from a fresh snapshot, git's
   status, the ledger's day and the settings files, then logged. If it passes, the next brief is
   drafted with the rules' model and effort (the downshift included) and started headless as the
   dialog starts one: the project's permission mode, route, hooks, revert start, answer timeout,
   neighbours and budget. If only the pause refuses, the chain is held and goes on once the pause
   lifts and the gate passes again; any other refusal ends it.
5. **Caps.** Every headless run of a project whose autopilot is on at its start has a cap by size
   (S 45 min, M 90 min, L 3 h; a row with no size takes L's), counted in working time (decision 4),
   and is stopped gracefully at it, which ends the chain. `max_budget_usd` is passed when set.
6. **Pause.** The owner's Pause in the window and the tray (`autopilot_paused` in the registry, so
   it holds across restarts); the automatic pause while weekly or Fable is at or over the mark,
   lifted by the reset. A pause holds the next start and never stops a run that works (D-13).
7. **The window.** The list's box and the A key toggle a project's autopilot; the tray's item
   pauses and resumes; an ended tab whose run completed under the stop-before-committing rule has
   Commit; the Sessions pane's All tab opens with an Autopilot section: the state, Pause or Resume,
   and the chain log's latest lines (what ran, why it stopped).
8. **`crw autopilot`**: the state, the settings, the projects with it on and the log;
   `crw autopilot <project> on|off`; `crw autopilot pause|resume`.
9. **The log**: `event` rows of kind `autopilot` in the ledger, no schema change.
10. **Recorded**: a D-22 addendum, the README's paragraph, the roadmap's EP-19 link, the EP-19a
    row and its note, "From EP-19" notes where later briefs need them, an owner task.

## Out of scope

- EP-19a's per-brief model and effort and the chain's own notification (decision 1).
- A chain started from a visual session: the gate follows a window's headless runs only (D-22).
- A held chain across a restart of the window: it lives in the window, and the log says so.
- The headless resume after a restart (v2 notes).
- Pushing: a chained run commits and never pushes.
- Reading transcripts, or any real run of the CLI in tests (D-29).

## Verification / acceptance

- **Unit tests**: each gate condition alone and together, the figures' age rule, the waiting
  rule's settings files, the day's counts, the caps by size and the working clock, the commit
  variant and the Commit message, the registry's keys and writers, the chain's start and its
  hold, the Commit button and the toggle pressed through `testing::Screen`, the tray's entries.
- **Build checks**: `cargo test`, `cargo test --features screenshot`, `cargo clippy --all-targets
  --features screenshot -- -D warnings`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt
  --check`.
- **Screenshot runs** of the Sessions pane's Autopilot section and the list's boxes, both themes.
- **The release**: `cargo build --release` once the owner has quit the window from the tray, then
  `scripts\install-shortcut.ps1`.
- **Rule 3's word check** on the staged diff prints nothing.

> **Completion note (2026-11-05; claude-opus-5-5/max; about 50 min to the commit, with the
> owner's three rounds and one quit of the window).** One session in the VS Code extension from
> 23:11 UTC, launched from the window (ledger row `e8fece6e`, visual, EP-19), after the reverted
> Fable attempt `ca6a5fc7`. It covered the reading, the pickup round, this brief, the code and its
> tests, the screenshot runs, the closing round and the release.
>
> **Built.**
>
> - `crw_core::autopilot`: the gate as a pure function over `Facts`, naming each `Refusal`;
>   `holds` (the pause alone refused); the figures' rule (`weigh`, `usage_pause`); the caps by
>   size and the working `Clock` (`Cap`); the waiting rule (`would_wait` over the project's two
>   `.claude` settings files and the user's); `is_own`; the day's counts from the log
>   (`chained_on`); the log's words and its `event` rows (`log`, `recent`); `read`, the facts
>   read afresh with the next brief's draft; `launch`, the chained run's `Launch`.
> - The registry: `[project.autopilot]` `max_size`, `daily`, `max_budget_usd`;
>   `[settings]` `autopilot_pause_percent`, `autopilot_daily_total`, `autopilot_paused`; the
>   switch's writer (`set_autopilot`) and `crw setting`'s three keys.
> - The prompt: rule 5 commits while the project's autopilot is on (`session_rules`'
>   `commits`, `Parts::commits`), and `commit_message` is the Commit button's yes.
> - The window (`app::autopilot`): the gate after each run that ended, on a worker thread under
>   `process::turn`; held chains read again every 5 minutes and at the Resume; the queued
>   chained start, which waits for the start slot and looks at the pause, the switch and the
>   project's other sessions again; the caps (`Sessions::check_caps`); the list's box and A; the
>   tray's Pause autopilot (`Order::Pause`); the Commit button (`Live::offers_commit`,
>   `Sessions::commit`); the Autopilot section at the top of the Sessions pane's All tab; a
>   capped tab's line in violet; the `autopilot` screenshot flag.
> - `crw autopilot`, `crw autopilot <project> on|off`, `crw autopilot pause|resume`.
> - The records: a D-22 addendum, the README's paragraph, the EP-19 link, the EP-19a row and
>   note, "From EP-19" notes for EP-19a, EP-20, EP-23 and EP-30, five v2 items, OT-18.
>
> **Verification.** `cargo test` 413 passed (98 + 1 + 1 + 2 + 1 + 273 + 2 + 7 + 6 + 11 + 11,
> against EP-35's 400), 415 with `--features screenshot`; both clippy runs and `cargo fmt
> --check` are clean. The new tests: each gate condition alone and together; the figures' age
> rule (at the mark whatever the age, 15 minutes under it, a reset, never read); the waiting
> rule's modes and settings files; `is_own`; the day's counts in local time; the cap's working
> clock; the log's words and its ledger rows; the commit rule and the Commit message; the
> registry's switch (inline, the owner's own table, off back to the file as it was) and its
> limits and settings; the list's box pressed through `testing::Screen`; the Commit offer and a
> run's carry-over; the tray's item. Screenshot runs, kept in the session's scratchpad: light
> `autopilot` (the section with a held chain and the log, harbor's box ticked), dark
> `autopilot,transcript=beacon` (a completed tab's Commit), light `autopilot,transcript=harbor`
> (a chained tab's cap line). The parser reads the roadmap with the new row: 44 briefs, next
> EP-19, no problem. The release was built once the window was quit, then
> `scripts\install-shortcut.ps1` refreshed both shortcuts; `target\release\crw.exe autopilot`
> read the owner's registry and ledger: not paused, no project on, the log empty. Rule 3's word
> check found nothing.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-05).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Split at pickup | Split off EP-19a | EP-19a (S) after EP-19: per-brief model and effort, the chain's notification |
> | 2 | This project and the chain | Never chain it | The gate's `OwnProject` refusal |
> | 3 | Runs that would wait at each command | Refuse and name it | The gate's `Waits` refusal; opt in with `auto` or allow rules |
> | 4 | The cap's clock while a request waits | Pause while a request waits | Working time (`Clock`) |
> | 5 | The release build | Quit the window, build and install | Built and installed |
> | 6 | The commit rule's reach | Every launch of an autopilot-on project | As built; D-22 addendum |
> | 7 | OT-18 | Keep it, before EP-19a | OT-18 |
> | 8 | The addenda, the README and the roadmap notes | Accept as written | Written |
> | 9 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. The limits live in `[project.autopilot]` (`max_size`, `daily`, `max_budget_usd`, none
>    by default) and `[settings]` (`autopilot_pause_percent`, `autopilot_daily_total`,
>    `autopilot_paused`), each written only when it differs from the default.
> 2. Two safety conditions beyond the brief note: the run's own row must be checked (else the
>    next brief would be the same one again), and no other session may be open in the project,
>    looked at again just before the start.
> 3. A figure at or over the mark counts until its reset whatever its age; under it, within 15
>    minutes (the meter's freshness); an older one, or one read before its window reset, holds the
>    chain; one never read ends it.
> 4. Every headless run of a project whose autopilot is on has a cap, the owner's first run of a
>    chain included; a brief with no size takes L's.
> 5. A held chain lives in the window, read again every 5 minutes and at the Resume, and is
>    logged again only when its words change.
> 6. The owner's Pause is a registry setting, so that it holds across restarts and `crw` sets it.
> 7. The log is the `event` table's kind `autopilot` (no schema 6, so older builds read the
>    ledger); the switch's turns are logged too.
> 8. The chain's own start neither closes an open launch dialog nor brings a pane forward; its
>    notices go to the status bar.
> 9. `Action::NotYet` went with the A key, its last use.
> 10. The Sessions pane shows the log's latest 8 lines; `crw autopilot` prints 20.
>
> **Notes for later sessions.**
>
> - A registry that sets `max_size`, `daily`, `max_budget_usd` or one of the three settings is
>   refused by the builds before EP-19; `autopilot = { enabled = true }` alone reads in them.
> - Today no project can chain: none sets a permission mode or holds an allow rule, so the gate
>   refuses each with `Waits` until the owner opts one in (OT-18).
>
> No messages for other projects. Owner task: OT-18, before EP-19a. **Next: EP-19a** (the chain
> plan and the chain's moment), launched from the window.
