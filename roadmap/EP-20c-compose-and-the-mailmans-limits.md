---
ep: 20c
title: Compose and the mailman's limits
size: M
model: fable
effort: xhigh
status: in progress
---

# EP-20c — Compose and the mailman's limits

**Size:** M · **Depends on:** EP-20a · **Blocks:** EP-23

## Context

D-13 makes the tool the mailman between projects and lets the owner compose messages in it.
EP-20 built the mailman (`crw_core::inbox::mailman`, the window's `app::mail`): a committed
brief's `Handed on (to <project>)` blocks delivered into the receiver's `roadmap/inbox.md` while
no session works there, the first reading counted, each delivery a `tick` row the revert redoes.
EP-20a built the notifications half: the urgent popup, the `[urgent]` chip and ntfy. This brief
is the owner's side of the mail, split off at EP-20a's pickup: the **compose dialog**, and the
**five limitations** EP-20's review parked, (a) … (e), fixed where they fit, plus the handover
point EP-20a's review left here.

**Session start (rule 1).** This session is the chained headless run of EP-20c after EP-20a
(ledger row `addd8994`, fable at xhigh, M's cap of 90 minutes of working time), started by the
window's gate at 18:55 after its restart into EP-20a's build. The field check read:
`crash.log` (the 2026-11-04 report of EP-15d's popup panic, nothing newer); `crw notifications`
(the chain's and EP-20a's toasts, nothing odd); `crw sessions` (EP-21, EP-22, EP-22a and EP-20a
completed today, this run open); `crw autopilot` (the chain held after EP-20a for the restart,
then EP-20c chained; the project's `daily` is 8, which the owner raised); `crw mail` (the first
reading of every registered project at 13:53 with 0 blocks counted, which is right: no roadmap in
the portfolio holds a `Handed on` block to a registered project, only to the owner). No open
inbox message; three open owner tasks (OT-21, OT-23, OT-24), none gated.

**What exists.**

- `mailman::append(folder, meta, text, backups, today)` writes a message with the next free id
  after a dated copy, creating the file with `mailman::HEADER`; `mailman::line` wraps it at 100
  columns; `redeliver` appends again unless `already_there` finds the message by its `from` and
  words; `Tick::delivery` is the revert's row of the receiver, redone in `revert::redo_ticks`.
- `mailman::round` reads the log (`Log`), counts a sender's committed letters at its first
  reading, and sends an unknown letter when its brief is committed (`committed`: git's status
  lists no change to the file) and the receiver is free (`Round::busy`, the window's
  `Board::busy_projects`).
- `autopilot::gate` leaves `inbox::TREE_PATH` out of the clean-tree condition unconditionally.
- `toasts::urgent_messages` and `urgent_arrivals`: the first look taken as seen, the owner's own
  messages left out (`is_owner`); `handover.json` (v1, frozen by a test) carries held chains,
  plans and `hidden`.
- The Detail pane's Inbox section draws each open message through `markup::note` with
  `Look { sentences: true, .. }`, the trailing `[urgent]` left to the chip; the row's context
  menu holds Pin, Priority, State, the dismissals and Revert.
- `crw tick` and `crw formalise` write into a project from the command line, the latter refused
  while `claude agents --json` lists a session at work there (`sessions_in`).

## In scope

1. **The compose dialog** (`app::compose`): opened from the Detail pane's Inbox section
   (Compose…) and the row's context menu (Send a message…); the receiver named, a multiline
   text, an Urgent box (its hover: the flag notifies the owner, not the session, D-13; written by
   the owner it makes no popup and no push), a preview of the message as the Detail pane will
   draw it (`markup::note`, sentences, the chip in place of the trailing flag) and the line as
   it will be written (`mailman::line` with the next free id of the row's inbox and
   `from the owner, <today>`); Send and Cancel. Send writes through `mailman::append` on a worker
   thread in the process's turn (as the mailman's send does), logs a `tick` row of the receiver
   (`Tick::delivery`, so a revert there redoes it) and a `mail` row (`written IN-n from the
   owner`, under the receiver, which `crw mail` prints), says it in the status bar with the copy's
   path, and reads the files again so the row shows the message. A screenshot run's Send is
   refused as every write is (`folder_for_write`); `compose=<project>[:urgent]` opens the dialog
   with canned text for the pictures. While a session works or waits in the receiver the dialog
   says so (the pickup question decides whether Send waits, as the mailman does, or goes with the
   note).
2. **`crw send <project|folder> <text…> [--urgent]`**, D-14's "inbox send": the same write from
   the command line, with the same rows; refused while `claude agents --json` lists a session at
   work there only if the dialog is (the same rule in both places).
3. **(c)** The gate's exception for `roadmap/inbox.md` holds only when the ledger shows a
   delivery or an owner's message written into the project since the run started (`Facts` gains
   it, read from the receiver's `tick` rows of the delivery kind); otherwise a changed inbox is a
   dirty tree like any other file, since the run left its own triage uncommitted.
4. **(d)** `redo_ticks` keeps the id `redeliver` returns: a later tick of the old id in the same
   redo ticks the new one, and the done lines say `delivered IN-4 as IN-6`.
5. **(e)** A sender's first reading waits while one of its briefs does not read (the snapshot's
   problems name it), so no block is missed; the mark says which file.
6. **The handover's seen set**: `handover.json` gains an optional field with the urgent messages
   the old window had seen (project and id), kept in its frozen v1 form; the restarted window
   starts from it, so an urgent letter delivered in the seconds between a run's end and the
   restart is toasted by the new window instead of taken as seen.
7. **(a) and (b)** where they fit this M (the pickup question): (a) committed block by block from
   `git show HEAD:<brief>`, so a block of an old brief whose file is uncommitted at the first
   reading is counted, not sent later; (b) a reworded block, sent again by D-13's rule, is said
   as such in the Detail pane and the status bar (a letter of the same brief and receiver was
   delivered before as IN-k).
8. The records: a D-13 addendum, the README's mail notes, the roadmap's row and notes, EP-23's
   notes for what moves, the completion note; the install last (rule 6).

## Out of scope

- The prompt hook (EP-20b); Formalise… in the window (EP-21a); the chat's compose delegation
  (EP-24); the phone's compose (EP-30).
- A new ledger schema: the rows stay `event` rows of the kinds that exist.
- Re-keying delivered letters by place (D-13's addendum chose the words' hash).
- An urgent toast for the owner's own message (EP-20a's rule stands).

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs and `cargo fmt --check`
  clean; new tests: the dialog's write (a scratch project, the line appended with the next id,
  the urgent tail, the tick and mail rows), the refusal in a screenshot run, the gate's inbox
  exception with and without a delivery since the run, the redo's id map, the first reading held
  by an unreadable brief, the handover's seen set in and out of the frozen form, `crw send`'s
  parse.
- A screenshot run with `compose=lantern:urgent` over the fixtures, both themes, looked at; a
  `--features screenshot` test drives the dialog's Send on a scratch copy (`CRW_HOME`).
- A review subagent given the diff alone (rule 6), its findings fixed or logged.
- Rule 3's word check before each commit; the release built with the window up, then the install.

**Residual risk.** The session cannot press the dialog in the real window, send a real message
into a real project, or restart the real window across a handover with an urgent letter in
flight (rules 5 and 13); it expects them to hold because the write is the mailman's own
`append`, exercised by EP-20's and this brief's tests on scratch folders, the dialog's egui path
is driven by a test through `testing::Screen`, and the handover's new field is covered by the
frozen-form test in both directions. The first real compose from the window is a look at a
changed surface for the P3 walk, not an owner task.

> **Completion note (2026-11-10; fable/xhigh; about 1 h 25 min of wall time, two questions at
> pickup).** The chained headless run of EP-20c after EP-20a (ledger row `addd8994`, from
> 18:55), under M's cap of 90 minutes of working time; the window's gate started it after the
> restart into EP-20a's build.
>
> **Built.**
>
> - `mailman::send_from_owner` (the owner's message appended with the next free id after a dated
>   copy, in the process's turn; a `tick` row of the receiver and a `mail` row `written IN-n
>   from the owner`), `compose_text` (`[urgent]` at the end when flagged), `owner_meta`,
>   `next_id`; `redeliver` returns the message's id now and whether it was appended.
> - `app::compose`: the dialog (text, Urgent, the preview through `markup::note` with the chip
>   in place of the flag and the line as written, Send on a worker thread, Cancel; the amber
>   note while a session works or waits there), `Board::open_compose`, `send_message`,
>   `compose_pass`, `compose_dialog`; `Action::{OpenCompose, Send, CloseCompose}`; the Detail
>   pane's Compose… beside the Inbox title (creating the file when there is none) and the row
>   menu's Send a message…; the keys held while the dialog is open; the screenshot flag
>   `compose=<project>[:urgent]`.
> - `crw send <project|folder> <words…> [--urgent]`, with a note when `claude agents --json`
>   lists a session at work there.
> - (c) `Facts::mail_since_run` and `autopilot::mail_since_run` (the receiver's delivery ticks
>   after the run's session row's start); the gate's inbox exception holds only then, and a
>   refusal of the inbox alone says that its triage is not committed.
> - (d) `revert::redo_ticks` keeps the id a redelivery got and carries a later tick of the old
>   id to it, the done lines saying `(as IN-k)`.
> - (e) `Snapshot::unread_briefs`, `Sender::unread`: a project's first reading waits while one
>   of its briefs does not read, its letters and mail to it waiting with it, the file named.
> - `handover::UrgentSeen` and the optional `urgent_seen` field (the frozen v1 form reads as
>   before); `Toasts::urgent_seen` and `carry_urgent_seen`; the restart writes and takes it.
> - The records: the D-13 addendum, the README (the command list, the mailman paragraph, the
>   Inbox bullet), the roadmap's EP-20d row and note, the P3 ordering note, EP-23's note and
>   depends-on.
>
> **The review (rule 6).** A subagent was given the code diff alone (every changed file and
> the new module, with the existing callers to check) and asked for verified findings with a
> failure scenario each; its report came back fifteen minutes after the work was committed
> (`8a1245c`), with the cap drawing near, so the fixes are a commit of their own, named beside
> the first in the roadmap's row. It found no API misuse and no test that cannot fail, and
> eight points. Fixed and tested:
>
> 1. **A message composed while a revert is open in the project** could be written between the
>    revert's reset and its write-back of the start's files and then put back with them, its
>    tick row logged after the plan's ticks were read: Send is refused while a revert is open
>    there (the dialog says so in place of the busy note), as the mailman holds a delivery.
> 2. **The gate's inbox condition counted deliveries only**, so the owner's own tick of a
>    message or a task from the window or `crw tick` between two chained runs ended the chain,
>    and the refusal's words blamed the run: any tick row of the tool's since the run counts
>    now, and the words say that nothing was written into the inbox by the tool, so a triage
>    of the run's own or a hand edit is not committed.
> 3. **The dialog's Send and the write disagreed on an empty message** (`[urgent]` typed with
>    the box unticked): the dialog refuses it too.
>
> Logged, not fixed (each rare, or a design the note records): a letter held by a receiver's
> unread brief is counted as that receiver's past once the hold lifts, though its mark said
> that it waits (routine choice 5; EP-20d's note takes it up); a message over the ledger's
> 1,000-character cap is not redone by a revert and nothing says so at write time; a flag
> typed before punctuation (`[urgent].`) is doubled; `crw send --urgent <folder> …` takes the
> flag for the target; the preview's id and date are those of the dialog's opening (routine
> choice 3). The lesson, for the memory: launch the review subagent as soon as the code builds,
> before the test gate, and wait for it with foreground work.
>
> **Verification.**
>
> - **Tests.** `cargo test --workspace` passed (345 in `core`, 137 in `app`, the rest in the
>   fixtures and the CLI), and with `--features screenshot` 145 in `app`: EP-20a's counts plus
>   8 new in `core` (the owner's send with its rows and refusals, the redelivery's id, the
>   first reading held, the gate's inbox condition and `mail_since_run`, the redo under another
>   id, the handover's field in and out of the frozen form) and 2 in `app` (the dialog's
>   buttons through `testing::Screen`, the window's write into a scratch project and the
>   screenshot run's refusal). Both clippy runs and `cargo fmt --check` are clean.
> - **Screenshot runs** (the fixtures, a scratch `CRW_HOME`, 1600 wide): `compose=lantern:urgent`
>   in the light theme (the amber note, Urgent ticked, the preview's chip, the id IN-2) and
>   `compose=harbor` in the dark (IN-3, no chip), each looked at and kept in the session's
>   scratch folder, never in git.
> - **The release.** Built with the window up; the install runs last, after the commits, and
>   the window takes it at the gate's moment or from File > Install and restart.
> - **Rule 3's word check** found nothing in the added lines or the brief.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-10).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Splitting the brief (about 3 h of work against M's 90-minute cap) | (a) and (b) to EP-20d | EP-20d (S) after EP-20b, its note written from this brief; EP-23's depends-on gains it |
> | 2 | Send while a session works in the receiver | Write now, with a note | The dialog and `crw send` say the session has read its inbox and write at once; the mailman's own hold stands |
> | 3 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The write runs on a worker thread** in the process's turn, as a delivery does, so a git
>    pass holding the turn never freezes the window; the dialog shows Sending… meanwhile and
>    nothing closes it until the answer comes.
> 2. **The owner's rows reuse the kinds that exist**: the tick row is a delivery's
>    (`Tick::delivery`, so `redo_ticks` redoes it unchanged), and the mail row's verb `written`
>    is one `Log::from_events` passes over; no schema change.
> 3. **The preview's id is the inbox's next free id when the dialog opened**; the write takes
>    the next free id then, so a delivery in between shifts it by one, said in the status bar.
> 4. **An urgent flag typed into the text is not doubled**: `compose_text` adds `[urgent]` only
>    when the words do not end with it already.
> 5. **(e) holds mail to the project too** while its first reading waits, since the alternative
>    (the fresh-receiver rule counting it as the project's past at once) would count a letter
>    written after the hold began; once the brief reads, mail to the project is its past, as
>    for any project read for the first time, and the mark says so.
> 6. **(c) fails safe**: without the run's session row or a ledger that reads, a changed inbox
>    counts as dirty and the chain ends with that said, rather than chaining over an
>    uncommitted triage.
> 7. **No owner task**: the first real compose, the first chain that meets a changed inbox
>    without mail, and the first restart with an urgent letter in flight are looks at changed
>    surfaces, listed in EP-23's note for the walk (D-10 addendum).
>
> No messages for other projects. **Next: EP-20b**, which a chain may run if the gate passes.
