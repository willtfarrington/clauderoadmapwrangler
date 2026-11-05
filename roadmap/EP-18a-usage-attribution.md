---
ep: 18a
title: Usage attribution
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-18a — Usage attribution

**Size:** S · **Depends on:** EP-18 · **Blocks:** EP-18b, EP-19

## Context

D-23's attribution: the weekly-percent change around each session, attributed to its project,
with a usage-share badge. EP-17 built the meter: a poll of the endpoint every 5 minutes, one a
minute after a launched session ends (`Meter::session_ended`), each reading a `usage_sample` row
of source `endpoint`. EP-18 split this brief off and left four facts for its first question: a
visual row ends at its tab's close, hours after its last turn; the owner runs sessions in
parallel; the figures are coarse; and D-23 attributes launched sessions only, while most of the
other projects' sessions are manual.

What the pickup checked (read only, from a script in the scratchpad):

- The endpoint gives whole percents. All 14 samples since 23:02 UTC are integers.
- The weekly figure moved once in those 85 minutes, 27 to 28 %, while EP-18 ran.
- The ledger has no `statusline` sample, which fits D-23's EP-12 addendum: the VS Code extension
  never runs the status line.

The sessions model (`crw_core::sessions`) hears a `Stop` at each turn's end and a `Notification`
when a session waits for the owner. The poll of `claude agents --json` lists each session busy,
waiting or idle. The window's own headless runs are in `app::sessions`. The `session` table has
V1's `weekly_before`/`after` and `fable_before`/`after`, never written so far.

**Session start (2026-11-05, 00:25 UTC).** Launched from the window as a visual launch (rule 11,
OT-14): the ledger row `d7542e51`, visual, EP-18a, open (`crw sessions clauderoadmapwrangler`).
The opener suggested Fable at high effort. The owner chose Opus 5.5 at max in the tab's panel and
says that, until the tool can set a visual tab's model and effort, they weigh the suggestion and
choose there. The tree was clean at `4d642ea` but for the owner's tick of OT-14. No owner task
and no inbox message is open.

**Decided at pickup (owner, through `AskUserQuestion`).**

1. **The method: split each poll.** Each reading's change since the window's previous reading is
   split equally among the sessions that worked in between. A tab's idle hours and the parallel
   sessions' use stay out, and the projects' sums add up to no more than the account used. The
   shares need ledger schema 5. The alternatives were the figures before and after each session
   (its last turn as the mark, or its row's end), whose sums count parallel use twice.
2. **Manual sessions count too** (a D-23 addendum). The alternative was launched sessions only,
   which leaves the projects started by hand near empty.
3. **Split.** The usage-share badge (its list column, the Detail line, the legend's key, the sort
   key) becomes EP-18b, a new S row after this one. The alternative was the whole brief as an M,
   which risks the session's context with auto-compaction off.
4. **The session's model and effort**: Opus 5.5 at max, chosen in the tab's panel.

## In scope

1. **Schema 5.** `session.weekly_share` and `fable_share`: the session's part of the weekly and
   Fable figures, in points of the limit. `SessionRow` reads them, with V1's
   `weekly_before`/`after` and `fable_before`/`after`. An older ledger reads them as none.
2. **The split** (`crw_core::usage::share`, no I/O). Between two readings of the same window's
   run, a figure's rise is the difference within one limit window, or the new figure after a
   reset, and none when either reading lacks it. Each session that worked between them takes an
   equal part. The window's first reading after its start splits nothing, since nothing watched
   before it.
3. **Who worked** (`app::watch`). A session worked between two readings when the window heard its
   `Stop`, or a `Notification` that waits for the owner, or saw it busy (a hook's or the poll's
   status) at a pass, or when it is a headless run of the window's that runs. A steady wait counts
   for nothing. Sessions outside the registry, or without a row, take their part as well, and it
   stays unattributed, so that the registered projects' sums never absorb it.
4. **The rows** (`Ledger::add_share`). Each worker's latest row by session id adds its part,
   whatever the row's mode (manual included, decision 2). It keeps the figures before the first
   interval that counted and after the latest. A session the ledger has no row for adds nothing.
5. **Shown.**
   - The words: `ledger::shares(row)`, `week +0.5 % (27 → 28 %) · Fable +0 % (19 %)`.
   - The Sessions pane's ledger card shows them in a line.
   - `crw sessions` shows them in each row's line. It also prints each project's sum over the
     weekly window, the sessions started since the reset less seven days, with the reset taken
     from the window's cached reading (read only).
6. **The split recorded.** EP-18a's row is retitled. EP-18b gets a row and brief notes, and
   EP-23's depends-on and totals follow. "From EP-18a" notes go where later briefs need them,
   with D-23 and D-26 addenda.

## Out of scope

- **The usage-share badge**: its column in both of `GIVE_WAY`'s forms, the Detail pane's Status
  line, the legend's key, and `SortKey::UsageShare` (EP-18b).
- An unattributed remainder shown anywhere, and a split weighted by busy time.
- The chart of D-23 and D-26 (P4's analytics), and the five-hour figure.
- Use the window cannot see (claude.ai, the phone, Cowork): it falls to whoever worked in the
  interval. The breakdown by surface shows Claude Code at 100 % this week.

## Verification / acceptance

- **Unit tests**:
  - the split: a rise within a window, a reset between readings, a missing figure, no worker,
    three workers;
  - the ledger: schema 5's migration, shares that add up with the figures kept, the weekly sums
    by project, a schema 4 ledger read;
  - the words;
  - the watch's workers: a `Stop`, a busy listing, a steady wait.
- **`crw sessions`** in the CLI's tests: a row's share and a project's sum.
- **Build checks:** `cargo test`, `cargo test --features screenshot`, `cargo clippy --all-targets
  --features screenshot -- -D warnings`, `cargo clippy --all-targets -- -D warnings` and `cargo
  fmt --check`.
- **Screenshot runs** of the Sessions pane's All tab in both themes, with canned shares on the
  ledger cards.
- **`crw sessions clauderoadmapwrangler`** on the real ledger, read only, after the release.
- **The release.** `cargo build --release` once the owner has quit the window from the tray, then
  `scripts\install-shortcut.ps1`.
- **Rule 3's word check** on the staged diff prints nothing.

> **Completion note (2026-11-05; claude-opus-5-5/max; about 40 min to the commit, with the
> owner's two rounds and one quit of the window).** One session in the VS Code extension from
> 00:25 UTC, launched from the window (ledger row `d7542e51`, visual, EP-18a; OT-14 done). The
> reading, the pickup round, this brief, the code and its tests, the screenshot runs, the closing
> round and the release.
>
> **Built.**
>
> - `ledger`: schema 5 (`weekly_share`, `fable_share`). `SessionRow::usage` (`RowUsage`) reads
>   them with V1's `weekly_before`/`after` and `fable_before`/`after`, and `session_columns`
>   replaces the three column lists. New: `Part`, `Ledger::add_share` (onto the latest row of a
>   session id), `Ledger::shares_since` (a `ProjectShare` per project), `shares` (the words) and
>   `points`.
> - `usage::share`: `rise`, `part` and `week_start`, without I/O.
> - The window:
>   - the watch keeps the sessions that worked (`Watch::take_worked`: a `Stop`, a `Notification`
>     that waits, a busy status at a pass);
>   - the meter hands over each reading with this run's one before (`Meter::take_readings`);
>   - the board splits each rise among those sessions and the window's running headless runs
>     (`Board::split_usage`), then marks the Sessions pane's history stale.
> - The Sessions pane's ledger card and `crw sessions` give a row's share. `crw sessions` also
>   prints each project's sum over the week, from the cached reading's reset.
> - The split: EP-18a's row retitled, EP-18b's row and brief note, EP-23's depends-on, totals and
>   order note, the D-23 and D-26 addenda, the README, and OT-15.
>
> **Verification.** `cargo test` 392 passed (91 + 1 + 1 + 2 + 1 + 259 + 2 + 7 + 6 + 11 + 11,
> against EP-18's 385), 394 with `--features screenshot`; both clippy runs and `cargo fmt
> --check` are clean. Screenshot runs, kept in the session's scratchpad:
>
> - the Sessions pane's All tab filtered to harbor, dark: a headless card's `week +2 % (21 → 23
>   %) · Fable +0 % (19 %)`;
> - filtered to lantern, light: a visual card's `week +1.5 % (24 → 26 %) · Fable +1.5 % (19 → 21
>   %)`.
>
> The release was built with the window quit from the tray, through
> `scripts\install-shortcut.ps1`. `crw sessions clauderoadmapwrangler` on the real ledger, read
> only and still at schema 4, printed "the week since 2026-10-31 18:00: no session has a share
> of the usage yet". That is the reset's instant in local time, an hour off 17:00 because
> daylight time ended between. Rule 3's word check found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-05).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The attribution's method, at pickup | Split each poll among the sessions that worked | As built; schema 5; D-23 and D-26 addenda |
> | 2 | Manual sessions, at pickup | Count them too | As built; D-23 addendum |
> | 3 | EP-18a's scope, at pickup | Split off the badge | EP-18b's row and brief note; 43 briefs |
> | 4 | The session's model and effort | Opus 5.5 at max, chosen in the tab's panel | The front matter |
> | 5 | The release build | Quit the window, build and install | Built and installed |
> | 6 | The addenda, the README and the roadmap notes | Accept as written | Written |
> | 7 | OT-15 | Add it | OT-15, before EP-18b |
> | 8 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. The split is equal among the sessions that worked, not weighted by busy time. Two readings
>    taken in one pass share one set of them.
> 2. A session that waits for the owner works only when its wait begins, so a question left open
>    for hours takes no part.
> 3. A part goes to the latest row of its session id, so a resumed run takes the parts from its
>    start. The figure before comes from the first part, the one after from the latest.
> 4. After a reset between two readings, the rise is the new figure. A figure that falls within a
>    window rises by nothing.
> 5. The week's sums count the sessions started since the window opened, as the brief notes say,
>    so a session that spans a reset brings its earlier parts along.
> 6. A share is written whole or to a tenth (`ledger::points`), and the figures around it once
>    when they did not change.
> 7. The pickup's look at the samples opened the ledger read only, from a script in the
>    scratchpad, and read the samples' times and figures alone.
>
> **Notes for later sessions.**
>
> - Unchecked: whether `claude agents --json` lists a session's subagents or the extension's
>   helpers as sessions of their own. A busy one would take a part that stays unattributed, so
>   the sums would run low; OT-15's first real shares may show it.
> - The screenshot setup has no flag for the Sessions pane's All tab:
>   `transcript=<a project with no run>` brings the pane forward on it.
> - This session's work before the release ran under EP-18's build, which keeps no shares.
>
> No messages for other projects. Owner task: OT-15, before EP-18b. **Next: EP-18b** (the
> usage-share badge), launched from the window.
