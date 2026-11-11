---
ep: 20d
title: The mailman's keys
size: S → M (re-scoped at pickup)
model: claude-opus-5-5
effort: max
status: done
---

# EP-20d — The mailman's keys

**Size:** S → M (re-scoped at pickup, the owner's choice) · **Depends on:** EP-20c · **Blocks:**
EP-23

## Context

EP-20 built the mailman (D-13): a committed brief's `Handed on (to <project>)` blocks delivered
into the receiver's `roadmap/inbox.md` while no session works there, a project's first reading
counting the blocks already there. EP-20's review parked five rare limitations; EP-20c fixed
three and split two off to this brief: (a) the past is told from the new by git's status of the
brief's *file*, so a block of an old brief whose file happens to be uncommitted at a project's
first reading goes out later as news; (b) a block whose words change is a new letter and is sent
again (the key hashes the words, D-13's EP-20 addendum, kept), with nothing saying why the
receiver got it twice. EP-20c's review logged four more points for here: (1) `compose_text`
doubles a flag typed before punctuation (`Do it [urgent].` → `… [urgent]. [urgent]`); (2) `crw
send --urgent <project> …` takes the flag for the target; (3) a message over the ledger's
1,000-character cap is not redone by a revert, and the dialog has no length hint; (4) a letter
to a receiver whose first reading waits for an unread brief is marked as waiting, then counted as
that receiver's past once the brief reads (`fresh_receivers`), never written, even when it was
written during the hold.

**Session start (rule 1).** Started from the window (Start, Visual, Open in VS Code); its prompt
is the tool's opener, and `crw sessions clauderoadmapwrangler` lists its row (`3b34ec3d`, visual,
EP-20d, open). The launch suggested fable at high; the VS Code panel runs claude-opus-5-5 at
max (the owner's answer). The field check read: `crash.log` (the 2026-11-03
report of EP-15d's popup panic, nothing newer); `crw notifications` (EP-20b finished at 20:41 and
the chain ended at the day's cap of four chained runs; at 21:34 a headless run under EP-20b's
label, `b5beb495`, stopped after 24 s and the chain ended on its stop, which the session takes as
the owner's own start and Stop, the window having read EP-20d by 22:05, when it launched this
session); `crw sessions` (EP-20b completed, a manual session at 21:50 and 22:01, this one);
`crw autopilot` (on, `auto`, eight a day, nothing chained since); `crw mail` (the first readings
of 13:53 and nothing since); `crw doctor` (the installed copy built at 20:37, EP-20b's; the hooks
run it). No open inbox message; three open owner tasks (OT-21, OT-24, OT-25), none gated; OT-23
ticked by the owner since the last commit, left uncommitted in the tree.

**What exists.** `mailman::round` reads the log (`Log`), counts a sender's committed letters at
its first reading, and sends an unknown letter once `committed` passes (git's status of the
sender lists no change to the brief's file, `Round::changes`, asked once per sender with a
letter) and the receiver is free (`Round::busy`). `inbox::outgoing` resolves each block's target
through the registry, whole or by its first word; a letter's key is `<brief>:<hash of receiver
and words>`. `Known::Delivered { receiver, id }` is read from `delivered <key> to <receiver> as
IN-n` rows. `process::output_with` discards a child's standard error, so a failure says only its
exit code. `snapshot::read_git` already reads `git show HEAD:roadmap/README.md`.

## In scope

1. **(a) Committed block by block.** A letter is committed when its key is among the letters of
   `git show HEAD:<brief>`, the text parsed through `brief::parse` and its blocks resolved as
   `inbox::outgoing` resolves them (`inbox::outgoing_of`, one brief's blocks, shared). `Round`'s
   `changes` seam becomes `head` (one brief's text at a sender's `HEAD`, `None` when `HEAD` holds
   none of it) and `resolve` (the registry's name for a target); HEAD's copy is asked once a round
   per brief that holds a letter to send. `git::show_at_head` runs `git show HEAD:<path>` with
   its standard error kept (`process::output_with_stderr`, under `LC_ALL=C`): `does not exist in
   'HEAD'`, `exists on disk, but not in 'HEAD'` and `invalid object name 'HEAD'` (no commit yet)
   mean no committed block; any other failure waits for the next round, and at a first reading
   holds the whole reading, as a status error did. A letter waits with `its brief is not
   committed yet` or `its words are not committed yet`. `app::mail::round` supplies the seam in
   the process's turn, spaced (D-29).
2. **(b) A rewording said.** When a letter goes out, the log's `Known::Delivered` letters of the
   same sender, brief and receiver whose words are gone from the sender's briefs are what it
   rewords: the mark says `delivered to beta as IN-9, rewording IN-4`, and the status bar
   `Delivered IN-9 to beta, from alpha EP-6, rewording IN-4.`; later rounds say it again from the
   log.
3. **(1)** `compose_text` adds the flag only when the words hold none, as `inbox::parse` reads
   one anywhere; a message of the flag and punctuation alone is empty for the write and the
   dialog.
4. **(2)** `crw send` takes `--urgent` anywhere; the first other argument is the target.
5. **(3)** The compose dialog warns, under the preview, when the line's tick row would pass the
   ledger's 1,000 characters (a revert there would name the message, not put it back);
   `mailman::Sent` says so, and `crw send` and the window's status bar repeat it.
6. **(4)** Mail to a project whose own first reading waits for an unread brief is not held: the
   first round counts it as that project's past, as for any project read for the first time, and
   records that it did (a `mail` row under the project), so a letter written to it later is news,
   delivered while its own reading still waits.
7. **The owner's test (both routes, the owner's choice).** `crw send` writes a test message into
   the environment repository's inbox now (its name is on rule 3's word list, so no tracked block
   names it); and this brief's completion note hands a test block on to this project's own inbox,
   which the window's mailman delivers after the commit (OT-21's path and (a)'s check).
8. **The records.** A D-13 addendum, the README's mailman sentences, the roadmap's row and EP-23's
   notes, the completion note; the review subagent (rule 6, an M now); the install last.

## Out of scope

- Re-keying letters by place, or naming a rewording of a block counted at the first reading
  (its `counted` row holds no receiver).
- Keeping a long message whole for a revert (the ledger's cap stands; (3) only says it).
- Delivering a block found only in the working tree (D-13's EP-20 addendum stands).
- EP-21a's Formalise… and Migrate…; the phone's compose (EP-30).

## Verification / acceptance

- New tests: a first reading over a scratch repository whose old brief has an uncommitted edit
  (its block counted, not sent later), a new block of a committed brief file (waits, then goes out
  once committed), a reworded block (waits, then goes out saying `rewording IN-n`), a CRLF working
  copy against HEAD's LF, `git::show_at_head`'s three answers and a failure; the held receiver's
  mail counted at once and a later letter delivered during the hold; `compose_text`'s flag before
  punctuation; `crw send`'s arguments; the dialog's length warning.
- `cargo test`, `cargo test --features screenshot`, both clippy runs and `cargo fmt --check`
  clean; a review subagent given the diff alone, its findings fixed or logged.
- Rule 3's word check before each commit; the release built with the window up, then the install.

**Residual risk.** The session cannot watch the real window deliver (rules 5 and 13): the
mailman's round runs there only, after the poll answers and while no session works in the
receiver. It expects (a) to hold because the round's HEAD seam is driven by tests over real
scratch repositories through the same `git::show_at_head` the window calls, git's three messages
were read from git itself (2.x on this machine) and are matched under `LC_ALL=C`, and a failure
of any other kind only delays a letter. The owner's test block (route 2) is the first real look:
`crw mail` should list `delivered EP-20d:… to clauderoadmapwrangler as IN-8` once this session is
idle, which the P3 walk takes with OT-21.

> **Completion note (2026-11-10; claude-opus-5-5/max; about 1 h of wall time, four questions).**
> A visual session started from the window (ledger row `3b34ec3d`, from 22:05), re-scoped S → M
> at pickup (the owner's choice).
>
> **Built.**
>
> - (a) `git::show_at_head`: `git cat-file blob HEAD:<path>` under `LC_ALL=C`, git's standard
>   error kept by `process::output_with_stderr`, `None` for git's three words for a path `HEAD`
>   does not hold (`does not exist in 'HEAD'`, `exists on disk, but not in 'HEAD'`, `invalid
>   object name 'HEAD'`). `inbox::outgoing_of`, one brief's blocks, shared with `outgoing`.
>   `mailman::Round`'s `head` (answering `Shown`) and `resolve` in place of `changes`;
>   `committed` keys `HEAD`'s copy of the letter's brief once a round (`Heads`) and waits with
>   `its brief is not committed yet` or `its words are not committed yet`; a git failure holds
>   a first reading whole; `app::mail::round` supplies the seam in the process's turn, spaced.
> - (b) `State::Delivered`'s `rewording`; `Log::reworded` (the delivered letters of the same
>   sender, brief and receiver whose words are gone, less any another reworded already), the
>   `rewording <key> of IN-n` row and `Log::rewording_of`; `delivered` writes the row, the
>   status bar's words and the state.
> - (1) `compose_text` adds the flag only when the words hold none; `is_empty_message` for the
>   write and the dialog. (2) `crw send`'s `send_args`, the flag anywhere. (3)
>   `mailman::revert_keeps` and `Sent::revert_keeps`; the dialog's amber warning under the
>   preview, the status bar's and `crw send`'s note. (4) `Log::has_received` and the `mail
>   counted as its past` row: a project whose own first reading waits is a fresh receiver in
>   its first round, and EP-20c's hold of its mail is gone.
> - The records: the D-13 addendum, the README's mailman sentences, the roadmap's row and
>   EP-23's note.
>
> **The review (rule 6).** A subagent was given the code diff alone, launched as soon as the
> code compiled; it reported after about fifteen minutes, having run the core tests, with no
> serious bug and six low points. Fixed and tested:
>
> 1. **A project whose own first reading waits on a git failure** (a folder without its own
>    repository, a dubious-ownership refusal) stayed a fresh receiver every round, so mail to it
>    was counted and never delivered: it now records its past mail once, as a held project does.
> 2. **`git show` reads its argument as a pathspec** when `HEAD` holds nothing there, so a brief
>    whose name held a glob character read as an empty copy, or before the first commit as an
>    error: `git cat-file blob` answers with git's three words alone (checked on git 2.56).
> 3. **A third version of a block named both earlier ones**: a version another reworded already
>    is left out, so it says `rewording IN-4`, not `IN-3 and IN-4`.
> 4. **A draft another project was writing to a project read for the first time** was counted as
>    that project's past (EP-20's rule, kept by the old code): only a committed letter is now,
>    and the draft goes out once committed.
>
> Logged, not fixed: an ignored brief, whose letters the old status check sent, now never counts
> as committed (no registered project ignores `roadmap/`, the reviewer checked); the past-mail
> row's count leaves out letters counted through a new sender's own first reading (its words now
> say `from projects read before`); two blocks to one receiver reworded at once each name both;
> a delivery a revert redid under another id is still named by its first id, as the Detail pane
> already names it (EP-20c's (d) writes no mail row).
>
> **Verification.**
>
> - **Tests.** `cargo test --workspace` passed (353 in `core`, 138 in `app`, the rest in the
>   fixtures and the CLI), and with `--features screenshot` 146 in `app`: EP-20b's counts plus 3
>   new in `core` (`show_at_head`'s answers over a scratch repository; a round over a real
>   scratch repository through `show_at_head`: an old brief edited since its commit, with CRLF
>   ends against `HEAD`'s LF, counted at the first reading, a new brief waiting and then
>   delivered, a reworded block waiting and then sent again with `rewording IN-3`, a third
>   version with `rewording IN-4`; a project whose git cannot say), 1 in `app` (the dialog's
>   length warning drawn, the flag with punctuation no message), 1 in the CLI (`send_args`), and
>   three rewritten (the first reading over the `HEAD` seam, git asked once per brief; a held
>   project's mail counted at once and a later letter delivered during the hold; a fresh
>   receiver's draft waiting, then delivered). Both clippy runs and `cargo fmt --check` are clean.
> - **The release** built with the window up; the install runs last, after the commits.
> - **Rule 3's word check** found nothing in the added lines or the brief.
>
> **The owner's test (both routes, the owner's choice).** Route 1: `crw send` wrote the owner's
> test message into the environment repository's inbox at 22:20 as its IN-4, after a dated copy
> in the backups folder; `crw mail` lists `written IN-4 from the owner` under it. Route 2: the
> block below hands a test message on to this project's own inbox; once this session is idle the
> window's mailman should deliver it as IN-8 (`crw mail`: `delivered EP-20d:… to
> clauderoadmapwrangler as IN-8`, the status bar `Delivered IN-8 to clauderoadmapwrangler, from
> clauderoadmapwrangler EP-20d.`), and EP-21a's session declines it.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-10).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The brief's scope: (a), (b) and the four points EP-20c's review left | All of it, S → M | Re-scoped at pickup; the review subagent ran (rule 6) |
> | 2 | The test message asked for (OT-21), the environment repository's name being on rule 3's list | Both routes | `crw send` into that repository at once; a test block handed on to this project's own inbox |
> | 3 | The panel's effort (a fact for the record) | max | The front matter and this note |
> | 4 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **Git's standard error is kept for this child alone** (`process::output_with_stderr`), under
>    `LC_ALL=C`, rather than for every child, whose error words would all change.
> 2. **The rewording is recorded at delivery** in a row of its own, which older builds pass over,
>    rather than put into the `delivered` row (they would misread its receiver) or recomputed at
>    each round (a later deletion would misname it).
> 3. **A git failure at a fresh receiver's letter counts it**, as before, rather than holding it:
>    the receiver is read that round, and the letter would otherwise go out later as news.
> 4. **The plugin's house-style reference keeps "once the brief is committed"**, which stays true
>    for a session that commits its brief; only an edit after the commit differs.
> 5. **OT-23's tick**, the owner's, uncommitted at the session's start, is committed with the
>    work, as earlier work commits carried the owner's ticks.
> 6. **No owner task**: the first real delivery is OT-21's, which route 2 now exercises, and the
>    walk takes it (EP-23's note).
>
> **Next: EP-21a**, the window's Formalise… and Migrate….

> **Handed on (to clauderoadmapwrangler).** A test message from EP-20d, at the owner's request,
> for OT-21's check of the mailman's delivery between projects: nothing to build; decline it as
> a test and tick it.
