---
ep: 43
title: The phone's form wait — how long a page's order waits for the window, and what the window's pass did meanwhile, in the rolling log; the eleven-second wait's cause found and fixed; the page's wait lengthened only if a slow pass proves normal
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-43 — The phone's form wait

**Size:** S · **Depends on:** EP-32 · **Decisions:** D-28, D-14 (and D-3, D-10, D-29 as standing)

## Context

On 2026-11-17, with remote view on, the phone posted `POST /m/diagnostics/check` through Serve at
18:31:24 (the guard's line in the ledger). The window took the order at 18:31:35, after the page's
wait (`server::FORM_WAIT`, ten seconds) had let it go, so `Board::obey_orders` dropped it unrun
("A form from … was dropped: its page stopped waiting.") and the page said to send it again. The
second Check again at 18:32:00 was taken at once, as were the tray's Pause at 18:30:28 and the
phone's Resume at 18:30:35. The window was shown, perhaps behind another application's window.

**The path.** `Shared::order` (`crates/app/src/server.rs`) hands a `PageRequest` to the window
through a channel, calls the wake (`ctx.request_repaint()`, set in `Watch::bind`) and awaits its
`oneshot` for `FORM_WAIT`; `Hand::take` and `Hand::let_go` decide which side moved first. The
window drains the channel in `Board::obey_pages` (`phone.rs`), called from `Board::pass`
(`Dashboard::logic`) after `receive`, `read_remote_view` and `attend`, and before `run_queued`
and `publish`.

**What the records say** (read at pickup). The rolling log of 2026-11-17 holds nothing between
18:30:35 and the drop at 18:31:35. A read-only query of the ledger for 18:28 … 18:34 finds the
guard's lines at 18:30:35, 18:31:24 and 18:32:00, the autopilot's pause and resume, and one more
write inside the gap: the usage meter's endpoint sample stamped 18:31:28 (stamped before its
request, written with the cache by the meter's own thread after the answer). No session started
or ended, no history import, no badge reading and no conversation row in the span.

**What the code says** (read at pickup, the candidates of the brief note).

- *A wake that brings no pass:* not in this build. eframe 0.37 (`native/run.rs`) schedules a
  repaint for each `request_repaint`; a shown window is asked to redraw and, when no redraw comes
  within 250 ms (`MISSED_REDRAW_TIMEOUT`), runs `App::logic` alone; a hidden or minimized
  window's pass runs directly. `logic` runs in both, and `obey_pages` drains the channel in every
  pass without a condition. So the window's thread itself was held for about ten seconds.
- *A child or the network on the window's thread:* none. `process::turn()` is taken on worker
  threads only (the git and GitHub passes, the poll of live sessions, the gate, the mailman, the
  doctor's checks); ntfy's pushes and the usage poll have threads of their own.
- *A registry read:* no lock; one small file read and parsed, a few milliseconds.
- *The ledger:* the one mechanism in the code that holds the window's thread for seconds. Its
  file is in SQLite's rollback-journal mode (header bytes 18 and 19 are 1; nothing sets WAL), and
  every opener waits up to five seconds for a lock (`busy_timeout` in `Ledger::open` and
  `Ledger::open_read_only`). While another connection commits (the guard, the meter's thread,
  the pushes' thread, the runner's threads, a `crw` command in a session), or waits to commit
  behind a reader, no new reader gets in; each statement the window runs then waits up to five
  seconds, and an open runs two statements of its own before the work (`version`, through
  `migrate` for a writer). The pass opens the ledger in many places: the watch's records, the
  minute's lapse of visual launches and the modes it learns (`watch.rs`), the share of each
  usage reading (`split_usage`) and the week's sums after it (`Week::refresh`), the status
  lines' samples, the notifications' lines (`toasts.rs`), and `publish`'s two reads that EP-31
  added, the sessions' rows when the ledger changed (`History::refresh`) and the latest
  notifications every 15 s (`Publisher::read_notifications`). `publish` runs every second
  whether or not remote view is on. Which connection held the lock at 18:31 is not recorded.
- *The order's place in the pass:* a form that arrives while the window sleeps waits for the
  pass's whole `receive` and `attend`, with their reads and writes, before `obey_pages` takes it.

**Session start (2026-11-22, 16:25 local time).** A headless run from the window, the owner's
Start: the rolling log's "Started clauderoadmapwrangler EP-43 headless with claude-opus-5-5 at
max effort" at 16:25:09, and the chain log's "the chain's mark: 70 % of the session (five-hour)
window, your choice in its plan" at 16:25. The prompt is the tool's opener, and `crw sessions
clauderoadmapwrangler` lists the row (`56dfab5b`, headless, EP-43, claude-opus-5-5/max, open).
OT-30 is open and gated before EP-54, not this brief's; no inbox message is open. The field
check read, since EP-45's record commit (16:21): `crash.log`, last written on 2026-11-03, so no
panic since; the rolling log, another project's visual launch at 16:24, this project read again
after EP-45's session and this run's start, and earlier today a second window that yielded at
once (EP-40's guard, 15:08) and two starts at login without a quit between (a restart of the
machine); `crw notifications`, EP-45's manual session waiting and ending, the backup-stale
badges of 12:31; `crw autopilot`, not paused, on for this project and five others in mode
`auto`, this chain's mark. Nothing odd to triage.

## In scope

1. **Measure.** The server stamps each order with the moment it hands it over. When the window
   takes or drops an order that waited two seconds or more, the rolling log says how long it
   waited, how long the pass before took and its longest parts, and how long passed between
   that pass and the one that took it, with the frame's drawing in it. A pass that takes two
   seconds or more is logged on its own with its parts, at most one line a minute, the others
   counted in the next line, so that a slow pass is caught when no form waits. The parts are the
   pass's steps and the main steps of `receive`. Shorter waits add nothing to the log.
2. **Find.** The analysis above, completed from the code; a test that holds a scratch ledger's
   lock from a second connection, as a slow commit would, and shows what it does to a pass.
3. **Fix.**
   - The orders are taken at the start of the pass as well, before its reads and writes: a form
     that arrives while the window sleeps or draws no longer waits for them. They are still taken
     after `attend`, so that one arriving during the pass waits only for what is left of it.
   - The pass's three ledger reads (the sessions' rows, the latest notifications, the week's
     sums) wait at most a quarter second for the lock; one that finds the ledger locked keeps
     what it has, says no problem, and tries again a second later. *(Added in the session, from
     the test's measurement: the chain log's read every half minute in `read_pilot`, and the
     minute's lapse of visual launches, a write the next minute makes again, the same way.)*
   - `FORM_WAIT` stays ten seconds, and with it the page's words and the README's phone section:
     nothing shows yet that a slow pass is normal. The measurement ships in this build; a
     later brief lengthens the wait if its lines say so.

## Out of scope

- The window's writes to the ledger (the watch's records, the samples, the shares, the
  notifications' lines, the chain log): they keep their five seconds, since a write must not be
  lost; the measurement names them if they hold a pass.
- The panes' own reads while the window shows (the graphs, the chat's list): the frame's drawing
  in the measurement names them.
- The ledger in WAL mode, which would let readers past a commit: a change to the form of the
  owner's records (rule 13), for the owner to weigh; noted in `v2-notes.md` with the measurement's
  case.
- Any change to the phone's pages, the guard or the `Hand`'s rule (a form not taken in time is
  never run).

## Verification / acceptance

- `Pace` (the passes' timing): unit tests for the wait's line and its threshold, the slow pass's
  line with its parts, and the minute's limit with its count.
- A scratch ledger locked by a second connection: the three reads return within about a second,
  keep their rows, show no problem, and read once the lock is gone; a pass with the ledger
  locked answers an order waiting at its start at once, and none of its parts waits out the
  ledger's five seconds.
- The orders' tests of EP-30a … EP-31 pass unchanged; the HTTP tests of the server are untouched.
- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings`, `cargo
  fmt --check`, a review of the diff by a subagent given the diff alone, `cargo build --release`,
  and the install last.
- No tracked file names a managed project, the owner's login or the machine (rule 3); the word
  check prints nothing on the staged diff.

**Residual risk.** The cause of the 2026-11-17 wait is not confirmed: what held the window's
thread then is not recorded, and the session cannot drive the real window, the phone or Serve
(rule 5). The ledger is the only mechanism in the code that holds a pass for seconds, and the
fix bounds the pass's reads and takes a form before the pass's own work; a form arriving in the
middle of a slow write, or while a pane reads, can still wait. The measurement's lines name the
slow part the next time, in a pass or in a frame, and the next re-plan's walk reads them. A read
that finds the ledger locked says nothing in the panes or the log however long the lock lasts
(the brief's choice): a writer stuck for minutes would show only as rows that do not move, and
as slow-pass lines once its waits add up to two seconds.

> **Completion note (2026-11-22; claude-opus-5-5/max headless, closed by hand in a
> claude-fable-5-2 session; 45 min of the run's work and about 40 min by hand).**
>
> **How it ended.** The headless run was stopped by the autopilot at 44.7 minutes of work, S's
> cap of 45 (D-22), with the code written, both test suites, both clippy runs, the format check,
> a screenshot run and the release build passed, and the review, this note, the commit and the
> install owed; its review subagent was stopped with it and had reported nothing. The owner
> chose to have the work finished by hand in a session started in VS Code, not from the tool
> (rule 11), instead of a revert. That session ran the review again, fixed two of its findings,
> ran the checks again and closed the brief.
>
> **What was built.** As In scope says: `crates/app/src/pace.rs` (the pass's parts, the form's
> wait, the slow lines and their minute's limit), each `PageRequest` stamped at its handover,
> the orders taken at the pass's start as well, and five accesses of the ledger in the pass
> bounded to a quarter second (`Ledger::open_read_only_waiting`, `Ledger::open_waiting`,
> `autopilot::recent_waiting`): the sessions' rows, the latest notifications, the week's sums,
> the chain log, the minute's lapse. The test's measurement: a pass with the ledger locked took
> 16.3 s before and about 3 s after, a form waiting at its start answered at once. `FORM_WAIT`
> stays ten seconds. WAL is parked in `v2-notes.md`.
>
> **The review** (a subagent given the diff and the brief's scope alone; five findings, none of
> them a wrong result). Fixed: (1) the retry "a second later" was never asked for, so an idle
> window would have slept until its next wake, five to thirty seconds or more: the pass's end
> now asks for a wake after `LEDGER_RETRY` when a read set its retry (`pace::retry_after`,
> `pace::retry_wanted`); (2) a lapse that met the lock between two rows left some launches
> closed and the records unread: the records are now read again after a busy lapse. Logged, not
> changed: (3) a lasting lock is silent, in the residual risk above; (4) three of the new tests
> lean on the wall clock (the history's and the notifications' second read within a second of
> the first, the form answered within a second) and could fail on a machine stalled for most of
> a second; (5) a slow frame gets a line of its own and shares the minute's limit with the slow
> passes, a little beyond In scope's words, kept because the context names the panes' reads as
> the other thing that can hold the window's thread.
>
> **Session's calls.** The two accesses added to the fix in the run (the chain log, the lapse),
> from its test's measurement; `rusqlite` as a dev-dependency of the app crate for the test's
> second connection; a static flag for the retry's wake rather than a context handed to four
> readers.
>
> **Owner's decisions (2026-11-22, by `AskUserQuestion`).** (1) The capped run's work: finished
> by hand, not reverted. (2) The cap: a new row EP-43a next (S's cap of 75 minutes, a headless
> run told its cap; D-22 addendum there). (3) Sizes: the rule that an S is a change whose shape
> is known when planned, with EP-47, EP-48 and EP-56 re-sized to M (D-4 addendum, the roadmap's
> Sizes paragraph). (4) The commit: yes, as the work's commit, a second for the sizes and the new row, and the
> hash's record, pushed, then the install.
>
> No owner task and no message to another project.
