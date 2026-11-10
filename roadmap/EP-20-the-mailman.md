---
ep: 20
title: The mailman
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-20 — The mailman

**Size:** M · **Depends on:** EP-5, EP-13 · **Blocks:** EP-20a, EP-23

## Context

D-13 makes the tool the mailman: a session writes `> **Handed on (to <project>).**` in its
completion note, and the tool appends the message to the receiver's `roadmap/inbox.md` and logs
the delivery. Until now the owner carried every message by hand. EP-5 built the reading side,
EP-15b the revert, EP-19 the gate. This brief builds the delivery. It was split at pickup (below).

**What exists.**

- `inbox::outgoing(&snapshot.briefs, resolve)` gives each brief's `Handed on (to …)` blocks but
  those to the owner, each resolved against the registry (`Recipient::Project` or `Unknown`); every
  files read fills `ProjectRow::outgoing` with them, and the Detail pane lists them ("delivery
  arrives at EP-20").
- `inbox::parse` reads the receiver's messages; `checklist`'s writes copy the file into the
  backups folder (`files::dated_copy`) and write atomically (`files::write_atomic`).
- An open message is already a badge on the receiver's row and the tray (`toasts::marks`,
  `Kind::Inbox`), new until the window is next in front.
- The ledger's `event` table takes any kind (`Ledger::add_event`, `Ledger::events`). A revert
  redoes the tool's own ticks since the session's start from the `tick` rows
  (`revert::ticks_since`, `redo_ticks`).
- The gate refuses a tree that is not clean after the run's commit (`Refusal::Dirty` over
  `Facts::tree`).
- The window reads a project's files at its start, after its own writes, on F5 and when a folder
  of the data root appears or goes. Nothing reads them again when a session ends.

**Session start (2026-11-10, 13:02 local time).** Launched from the window as a **headless** run
(rule 11): ledger row `fe8ac4fb`, headless, EP-20, `claude-opus-5-5/max`, open (`crw sessions
clauderoadmapwrangler`). It is the first headless run of this project since the owner turned its
autopilot on at 12:58 with permission mode `auto` (the chain log), so it runs under M's cap of
90 minutes of working time, and the chain may follow it (OT-20). The prompt is the window's
opener. The tree is clean at `98fd096`; the inbox is empty; OT-20 is open and not gated.

**Field check.** I read the records back to EP-37a's record commit (`98fd096`, 12:41):

- `crash.log` was last written on 2026-11-03, read before;
- `crw notifications`: one since the commit, at 12:42, EP-37a's visual session ending its turn,
  called visual (EP-37a's mode lookup holding);
- `crw sessions`: this run's row; EP-37a's row completed; EP-37's first row (`45cc1e3b`) still
  open with no end, the known gap of risk 15;
- `crw autopilot`: not paused; this project on since 12:58 with `auto` (OT-20's first steps), a
  private project on; nothing odd.

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-10)

1. **Split in three** (D-4), since the brief note holds about three M's and a headless run that
   reaches its cap is stopped with its work uncommitted:
   - **EP-20** (this brief): the mailman;
   - **EP-20a** (M): the compose dialog, the urgent flag, the ntfy publisher, the `[urgent]` chip
     text and the finished popup's gist;
   - **EP-20b** (S): the `@<project>` `UserPromptSubmit` hook, its install an owner task.
2. **The new rows go after EP-22**, before EP-23, so the chain plan the owner set for this run
   (EP-21, EP-22) runs as set. Neither EP-21 nor EP-22 depends on them.

## In scope

1. **The mailman** (`crw_core::inbox::mailman`, new).
   - A **letter** is a `Handed on (to <project>)` block of a registered target: the sender, the
     brief, the receiver and the text without its label, its lines joined. Its **key** is the
     brief's id and a hash of the receiver and the text, so a letter is delivered once, and a
     block whose words change is a new letter.
   - **Delivery** appends `- [ ] IN-n (from <sender> EP-k, <date>) <text>` to the receiver's
     `roadmap/inbox.md`, wrapped at 100 columns with two-space continuation lines: the next free
     id is the highest `IN-n` plus one. The file as it was is copied into the backups folder
     first, and the new text is written atomically. A missing file is created with a header in
     the house form.
   - **When.** A letter is delivered only once its brief is committed (git reports no change to
     the brief file), so a draft of a completion note is never sent; and only while no session
     works or waits in the receiver, so a running session is never disturbed (D-13). Otherwise it
     waits for the next pass.
   - **The first reading counts as delivered.** The owner carried every message so far by hand,
     so the mailman's first reading of a project counts the blocks of its committed briefs as
     delivered and sends none of them; a brief a session is still writing then sends its blocks
     once committed. A project adopted later is read the same way at its first pass.
   - **The log** is the ledger's `event` rows of kind `mail`, under the sender: `first reading`,
     `counted <key>`, `delivered <key> to <receiver> as IN-n`. No schema change.

   *(After the review: a letter ends at a later labelled paragraph of its quote; mail to a project
   read for the first time is counted; the write runs in the process's turn under one inbox lock;
   rounds run only in the window that holds the loopback port, and git is asked only in a folder
   with its own repository.)*
2. **The window's pass** (`app::mail`, new). On a worker thread, after the first files read, after
   each of the window's runs ends, when the watch records a session's start or end, and every five
   minutes: the files of every registered project are read, git is asked only about a sender with
   a letter not yet in the log (one child, in its turn, D-29), and the letters go out. A delivery
   is said in the status bar, and the files are read again, so the receiver's badge shows the new
   message. The Detail pane says each block's state: delivered as `IN-n`, counted at the first
   reading, waiting and why, or not registered. A screenshot run delivers nothing.
3. **A revert keeps a delivery** (EP-15b's rule for the tool's own writes). Each delivery is also a
   `tick` row of the receiver (`delivered IN-n · <message>`), which a revert of a session there
   redoes after the restore: appended again, with the next free id, unless the message is there.
4. **The gate leaves `roadmap/inbox.md` out of its clean-tree condition**, the brief note's choice:
   a message delivered between two chained runs no longer ends the chain, and the next brief
   triages it first (D-13).
5. **`crw mail`** prints the mail log, the newest first: a safe read of the ledger.
6. **The records:** a D-13 addendum; the roadmap's EP-20 row, EP-20a and EP-20b rows with their
   notes, EP-23's row and note; the README; this project's inbox header; an owner task for the
   first real delivery (a new route into another project, D-10 addendum).

## Out of scope

- **EP-20a:** the compose dialog (`from the owner`), the urgent flag's popup and ntfy
  (`notify::ntfy`, host and topic in the registry), the `[urgent]` chip text in the Detail pane,
  the finished popup's gist line (EP-19b's field check).
- **EP-20b:** the `UserPromptSubmit` hook for `@<project>` and its install.
- Delivering into a project the registry does not know (shown, not delivered, D-13), and
  `Handed on (to the owner)` blocks, which are owner items (D-10).

## Verification / acceptance

- **Unit tests:** a letter's text and key; the wrapped line; an append to an existing inbox (next
  id, dated copy) and to a missing one (header); the pass: the first reading counted, a later
  letter delivered once, a letter waiting while its brief is uncommitted or a session works in the
  receiver, an unknown target never delivered; the log read back; a revert's redo of a delivery; the
  gate ignoring `roadmap/inbox.md`.
- **The checks:** `cargo test` with and without `--features screenshot`, both clippy runs,
  `cargo fmt --check`, and a review of the diff by a subagent given only the diff (rule 6).
- **The release:** `cargo build --release` with the window up, then `pwsh -File
  scripts\install-shortcut.ps1` (rule 6); its restart request is refused while this run works.
- **Rule 3's word check** on the staged diff prints nothing.
- **A screenshot run** of a project whose brief hands mail on, the states canned (a screenshot run
  delivers nothing).
- **Residual risk.** The session never ran the window on the owner's files (rules 5 and 13): the
  rounds ran on scratch folders and a scratch ledger, and a revert redid a delivery on a scratch
  repository. Three things are not exercised:
  1. **The first real round.** The window's first round reads every registered project and counts
     the blocks already written. Its condition (the poll answered, the files read) is the carried
     chains' (EP-37a), which the owner's window meets within seconds. Should it not come, nothing
     is sent, which is the safe side; `crw mail` shows it.
  2. **The first real delivery** into another project, after a session there commits a brief with
     a block: an owner task (OT-21), since it is a new route into another project (D-10 addendum).
     The append is the tick's copy-then-atomic-write, which every tick has used since EP-5.
  3. **A session that edits the receiver's inbox while idle in its tab.** An idle tab does not hold
     the mail, as it does not hold a chain (EP-19b), so a delivery can land under a session that
     read the file before. Claude Code refuses an edit of a file changed since its read, and the
     session reads it again: a delay, not a loss.

> **Completion note (2026-11-10; claude-opus-5-5/max; about 1 h of wall time, two questions at
> pickup and one in the closing round).** One headless run launched from the window (ledger row
> `fe8ac4fb`, headless, EP-20), the first of this project under its autopilot, from 13:02 local
> time.
>
> **Built.**
>
> - `crw_core::inbox::mailman`: the letters and their keys (FNV-1a, stable from build to build),
>   the wrapped line, `append` (the next free id, a dated copy, a header for a new file),
>   `redeliver`, the log (`Log`, `Known`), the round (`round`, `Round`, `Sender`, `Report`, `Mark`,
>   `State`) with the first reading, the committed check and the busy check.
> - `revert::Tick::delivery` and the redo of a delivery after a revert; `ledger::DETAIL_CAP` made
>   public and `checklist::copies_dir` visible in the crate.
> - The gate's clean-tree condition without `roadmap/inbox.md` (`inbox::TREE_PATH`).
> - `app::mail`: the window's rounds (`Board::mail_pass`, `busy_projects`, `Mail::apply`), the
>   Detail pane's states, canned states in a screenshot run.
> - `crw mail`, sharing `crw notifications`' printer (`events`).
> - The records: the D-13 addendum, the README, the roadmap's EP-20, EP-20a and EP-20b rows and
>   notes, EP-23's row and note, this project's inbox header, OT-21.
>
> **The review (rule 6).** A subagent given only the diff found six issues, nits and two tests
> that could not fail. Fixed and tested:
>
> 1. **A delivery lost to a revert.** The tick was dated at the round's start, so a message
>    appended after a session's start had copied the project, under a tick dated before that
>    start, was neither in the start's copy nor redone. The delivery now runs in the process's
>    turn, as a session's start does, its rows dated there; one lock now serialises every inbox
>    write in the process (the window's tick and the delivery); a revert open in the receiver holds
>    the mail.
> 2. **Old mail sent to a project registered later.** A block to an unregistered target is not
>    logged, so after an Adopt every such block would have gone out. Mail to a project read for
>    the first time is now counted as its past.
> 3. **A letter running into the next labelled paragraph** of its quote (`**Residual risk.**`),
>    which the brief's parser keeps in the same quote: the letter now ends there, and any bold
>    label opening the block is dropped (`Parked and handed on`, a label followed by a period).
> 4. **The committed check in a folder without its own repository** (found by the session too):
>    git is asked only where the folder holds one.
> 5. **The registry read at every frame** while it does not read: a round is taken as done first.
> 6. **The two tests**: the wrap test now puts the markers exactly at the wrap, and the window's
>    test proves each condition alone and that a round over the fake portfolio writes nothing.
>
> Logged, not fixed (each rare, none losing a message silently), and parked in EP-20a's notes on
> the owner's choice:
>
> - At a project's first reading the past is told from the new by the brief file's git status, so
>   a block in an old brief that is dirty at that moment goes out once committed; reading `git
>   show HEAD:<brief>` would tell them apart by block.
> - A reworded block is a new letter by design (D-13 addendum); a typo fixed in a committed block
>   sends it again.
> - The gate's exception for `roadmap/inbox.md` is unconditional, so a run that triaged its inbox
>   and did not commit it still chains; the next run commits it with its own work.
> - A redone delivery may take another id than it had, and a later tick of the old id in the same
>   redo names the old one.
> - A brief that fails to read at the first reading is not counted.
>
> The review's point that a failing append would leave a dated copy at every round does not hold:
> `files::dated_copy` reuses a copy holding the same bytes.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed 492, and with `--features screenshot` 500:
>   EP-37a's 483 and 491 plus 9 new in each run. They cover the letter, its end and its key, the
>   wrapped line, the append to an inbox and to none, the round (first reading, a brief still
>   uncommitted then, busy, delivered once, the tick for a revert, mail to a project read for the
>   first time), a revert redoing a delivery on a scratch repository, the gate ignoring the inbox,
>   the window's conditions for a round, a round over the fake portfolio writing nothing, and the
>   marks. Both clippy runs and `cargo fmt --check` are clean.
> - **Screenshot runs** (`select=lantern`, 1600×1700, both themes, a scratch home): the Detail
>   pane's "Handed on to other projects" says "delivered to harbor as IN-3" in the window's green
>   and "not registered: shown, not delivered" in amber. Kept in the session's scratchpad.
> - **The release.** Built with the window up; the install ran last (below).
> - **Rule 3's word check** found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-10).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Splitting EP-20 at pickup (a headless run under M's cap) | Three rows | EP-20 the mailman; EP-20a (M) compose, urgent, ntfy, gist; EP-20b (S) the prompt hook |
> | 2 | Where the new rows go | After EP-22 | Before EP-23; the chain plan's EP-21 and EP-22 run as set |
> | 3 | Where the review's five unfixed limitations go (closing round) | EP-20a's notes | Appended to EP-20a's brief notes as (a) … (e) |
> | 4 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The first reading counts as delivered**, per project, a later adoption too, as the badges'
>    first reading counts as seen (D-21 addendum): no history is sent into another project.
> 2. **Committed means git reports no change to the brief**, asked only of a sender with a letter
>    to send (one child in its turn), so a draft is never sent and the git pass is not repeated.
> 3. **The receiver must be free**: a session busy or waiting there (the hooks, the poll, the
>    window's runs) holds a letter; an idle tab does not, as for a chain (EP-19b). A round waits
>    for the first poll, so a session at work is never missed at the window's start.
> 4. **The key** is the brief and a hash of the receiver and the words, not the block's place:
>    a reworded block is a new letter, and a message already there with the same sender, brief and
>    words counts as delivered (a hand-carried one, or one whose log row was lost).
> 5. **The gate's exception** rather than the mailman waiting between a chain's runs: simpler, and
>    D-13's "the next session triages first".
> 6. **The log as `event` rows** (`mail`, and `tick` for the revert), no schema change.
> 7. **A delivery's line wraps at 100 columns**, a word that would open a markdown block kept on
>    the line before, and CRLF files keep their endings.
> 8. **The Detail pane's states** replace "delivery arrives at EP-20"; a screenshot run cans them.
> 9. **Found while the review ran, fixed and tested:** the first reading counted the blocks of a
>    brief a session was still writing, which would never have gone out; it now counts committed
>    briefs only, a brief in progress handing its letters on once committed. Rounds run only in
>    the window that holds the loopback port, where the hooks arrive, so a second window neither
>    misses a session at work nor sends a letter twice. A message appended after a paragraph that
>    follows the list opens a list of its own.
>
> **Notes for later sessions.**
>
> - `inbox::mailman::append(folder, meta, text, backups, today)` is the one writer of a new
>   message; EP-20a's compose dialog writes `from the owner, <date>` through it.
> - A letter's key must stay stable (`mailman::key`, FNV-1a over the receiver and the words):
>   changing it makes every block a new letter at the next round.
> - The gate no longer counts `roadmap/inbox.md` as dirty (D-22, D-13 addenda).
>
> No messages for other projects. OT-21, the first real delivery, is the owner's; no brief builds
> on it, so it is not gated. **Next: EP-21**, which the chain plan runs if the gate passes.
