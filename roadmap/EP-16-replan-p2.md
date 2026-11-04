---
ep: 16
title: Re-plan P2 and the dogfooding switch
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-16 — Re-plan P2 and the dogfooding switch

**Size:** S · **Depends on:** EP-9 … EP-15c · **Blocks:** EP-17 … EP-23 (P3 confirmed with pickup
notes) and every later EP of this project, which starts from the tool (D-5)

## Context

P2 closed with EP-15c on 2026-11-04. The window starts headless runs and visual launches, routes a
headless run's questions to its Decisions tile, shows every session in its Sessions pane, pops up
when a session waits, ends or fails, sits in the tray, and reverts a session behind a preview;
`crw` reads and prints the ledger. This re-plan closes the phase (D-4): the briefs against plan,
the parked items, the addenda P2 left to it, P3 confirmed with pickup notes, `v2-notes.md` grown,
and the dogfooding switch decided at EP-0 (D-5). From here on this project's briefs start from
the tool: rule 11 says so, the default launch mode is decided with that in view (EP-11's and
EP-14's notes), and the first self-launched session, EP-17's, is started from the dashboard with
the generated opener and found in the ledger.

**Session start (2026-11-04, 21:18 UTC).** Started by hand in the VS Code extension, cwd this
folder; the ledger's row for it is `manual` (`1e09593c`), so this session is not the self-launched
one (D-5 asks that of every EP after this one). The owner ticked OT-13 in the working tree; no other
owner task is open, and the inbox holds nothing open. The window runs (since 04:18 UTC, after
EP-15c's commit).

**Verified at pickup.**

- `cargo test`: 357 passed (85 + 1 + 1 + 1 + 232 + 2 + 7 + 6 + 11 + 11), as at EP-15c.
- History: 28 commits since EP-8 (two per brief for EP-9 … EP-15c, IN-6's inbox line `c946986`,
  and the rows EP-15a and EP-15b added at the owner's request in `d8c166a`), every one pushed;
  `main` only.
- The transcripts of EP-9 … EP-15c are filed under this project's slug, one session per brief,
  except EP-14 with EP-14a (one session across a night) and EP-15b with EP-15d (one session).
- The ledger (`crw sessions clauderoadmapwrangler --all`, read only) holds six rows for this
  project: four `manual` sessions, one short one, and one lapsed `visual` launch of EP-12 at
  22:25 UTC on 2026-11-02, two minutes before EP-12's session started, from before the hooks
  that claim a launch. The sessions of EP-13 … EP-15a started while no window ran and left no row.
  A VS Code tab left open overnight reads 18 h and 20 h long (EP-15c's and EP-15b's sessions).
- `crw doctor`: claude 2.1.316, still the standalone binary's version, while the extension now
  carries 2.1.318; gh 2.103.3, tailscale 1.104.4, code 1.144.1; headless ready with the effort
  levels low … max, no ultracode, and `--max-budget-usd`. The help's permission modes are
  `acceptEdits`, `auto`, `bypassPermissions`, `manual`, `dontAsk` and `plan`.
- crates.io: eframe 0.37.0, egui 0.37.0, egui_extras 0.37.0 and egui_tiles 0.18.0 are the newest
  releases and the pins; egui_plot's newest is 0.38.0, EP-26's; claude-codes' newest is 2.1.318
  against the pin 2.1.316 (D-15).
- Upstream: egui#8470 ("Showing viewports from App::logic doesn't work with deferred and can crash
  with immediate", open since 2026-08-29, labelled bug) is EP-15d's defect reported on eframe
  0.36.1: an immediate viewport made while the window is minimized panics with "the user callback
  was never called". The window already asks a minimized window to paint
  (`RequestPaintWhileHidden`, `board.rs`), so the issue's deferred-viewport case does not reach it.
- Permissions: neither this project nor the user settings hold an allow rule, so a headless run
  under `acceptEdits` sends each shell command it runs (every `cargo` and `git` call) to the
  Decisions tile; the user settings deny Claude's own tools a read of
  `~\.claude\.credentials.json`, which EP-17's poller reads in the tool's code.
- A headless run is the window's child: quitting the window, which rule 6's release build needs
  (a running exe cannot be replaced), kills the run (D-16's EP-9 addendum, D-29's EP-15 addendum).
- `rules::DEFAULT_LAUNCH_MODE` is `Visual`, its comment still giving EP-14's reason; a project's
  `[project.launch] mode` is written by hand in the registry (`crw set` does not change it).
- The ledger's `session` table already has the columns P3 fills: `weekly_before`/`after` and
  `fable_before`/`after` (EP-18), `cost_usd`, `commit_hash` (EP-19); `usage_sample` (EP-17) is
  there since V1. The schema is 4.

## In scope

1. **Retro.** A table *EP · size planned · actual · what bit* for EP-9 … EP-15c from the
   completion notes, the commit times and the session transcripts (timestamps and the brief each
   session wrote, nothing else; each session's first entry, or the previous EP's record commit
   when one session did two, to the EP's record commit; active time leaves out gaps over five
   minutes, the waits on the owner); the phase total against plan; the planning session that added
   EP-15a and EP-15b reported apart; lessons.
2. **Addenda** dated 2026-11-04, EP-16, appended and never rewritten: D-4 (the measured pace and
   the totals), D-5 (the switch), D-16 (the default launch mode), and whichever the owner's
   answers call for; no new number unless the owner approves one.
3. **Parked items.** Every out-of-scope item of EP-9 … EP-15c gets a disposition: a later brief, a
   v2 note, done, or dropped with the reason (a table in the completion note); EP-15a's three notes
   for this brief (the README's pictures, an inbox message's trailing `[urgent]`, the Progress
   section's phase names past a narrow pane's edge) and EP-15d's upstream report among them.
4. **The dogfooding switch** (D-5): rule 11 in `CLAUDE.md` in the present tense, with what a
   session started by hand does; the default launch mode decided; after the commits, EP-17 started
   from the dashboard by the owner with the generated opener and its row found in the ledger
   (`crw sessions`, read only).
5. **Roadmap**: the header's totals and a re-plan paragraph; the EP-16 row linked and, after the
   commit, ticked; P3 confirmed (order, sizes, dependencies) with pickup notes for EP-17 … EP-23 and
   the later notes a P2 fact changes; rows added where the owner places them, each with the next
   free number and a brief-notes bullet; the risks brought up to date; `v2-notes.md` grown.
6. **README**: the status paragraph brought to P2's end; the pictures as the owner decides.
7. **Owner decisions** through `AskUserQuestion` (rule 7): the sizes after the retro; the pins;
   the default launch mode and the first self-launch; the homes of the parked items; sign-offs of
   the addenda, the roadmap edits, the v2 notes, the README and `CLAUDE.md`; the commits and the
   push.

## Out of scope

- Code, dependency, pin or build changes, beyond what the owner decides for the launch mode or a
  pin (D-15); the added rows are planned here, not built.
- Writing P3's briefs, or an added row's brief (each session writes its own, rule 2).
- Rewriting `DECISIONS.md` text or a completed brief's note (addenda and dated notes only).
- The owner's registry and ledger (rule 13): `crw sessions` and `crw doctor` only read, and a
  project's `[project.launch]` keys are the owner's to write by hand.
- Pressing Start, Open in VS Code or Revert in the real window: the owner makes the self-launch.
- Another project's files (rule 4).

## Verification / acceptance

- `cargo test` green at pickup; if no code changes, nothing is built.
- The retro covers EP-9 … EP-15c with planned and measured times and the phase total; every figure
  is checked, not recalled.
- Every addendum is dated and cites EP-16; numbering stays D-1 … D-30 unless the owner approves a
  new decision; no decision text is rewritten.
- Every parked item of P2 has a disposition; no dangling D or EP reference in `roadmap/README.md`;
  every added row has six cells and a brief-notes bullet; `crw roadmap .` (with `CRW_HOME` on a
  scratch folder) reads the roadmap with no problem.
- Rule 3's word check prints nothing on the staged diff.
- After the commits: EP-17's session, started from the dashboard, has its ledger row with its EP
  and session id.
- The owner has reviewed every decision interactively.

> **Completion note (2026-11-04; claude-opus-5-5/max; about 33 min to the owner's last answer, all
> active).** One session in the VS Code extension from 21:18 UTC: the reading, the baseline `cargo
> test`, the transcripts' timings, the pins, the ledger and the upstream issue, this brief, then
> the owner's five rounds and 18 questions, with the drafts written between the third and the
> fourth. No code, dependency or build file changed. Every answer took the recommended option.
>
> **Item 1, retro.** From the completion notes, the commit times and the session transcripts
> (timestamps and the brief each session wrote, nothing else; each session's first entry, or the
> previous EP's record commit where one session did two, to the EP's record commit; active time
> leaves out gaps over five minutes, the waits on the owner):
>
> | EP | size planned | actual | what bit |
> |---|---|---|---|
> | EP-9 | L ≈ 2 h | 1 h 2 min, all active | process-wrap's std job object never kills on close (tokio's with `KillOnDrop` does); a nested `claude -p` needs the session markers withheld; one supervised probe of the neighbours' deny rule |
> | EP-10 | M ≈ 1 h | 46 min, all active | an egui `Modal`'s separator stretched the dialog off the screen; the app's tests cannot start a session, the stub being the core crate's |
> | EP-33 | S ≈ 30 min | 37 min, 29 active | a screenshot run's data root is the tracked fixtures, so every write the window makes checks it; toml_edit's inline-table removal left a stray space |
> | EP-11 | S ≈ 30 min | 1 h 37 min, 45 active | the extension decodes its link twice (`%`, `&` and `+` escaped twice), found by one supervised launch; the word check must run on the staged diff after the last edit |
> | EP-12 | M ≈ 1 h | 1 h 3 min, 55 active | Windows refuses a closed loopback port only after its own retries (the relay's connection gets 50 ms); the extension shows no user status line |
> | EP-13 | M ≈ 1 h | 53 min, all active | the ledger's migration raced between the loader and the first frame; one row wider than a narrow pane widens all after it; the owner's rule after OT-6, a row at a session's first work |
> | EP-14 | L ≈ 2 h | 1 h 4 min, 53 active | a probe found that headless runs had had no permission host since EP-9 (`--permission-prompt-tool stdio`): their questions disabled, their permissions denied without a word |
> | EP-14a | M ≈ 1 h | 56 min, 49 active, from the owner's return to EP-14's session the next afternoon | the window had vanished on a panic that a release build prints nowhere; its message was lost, and the investigation took 35 min |
> | EP-15 | M ≈ 1 h | 2 h 45 min, 1 h 16 min active | Windows toasts are off on the owner's machine: the WinRT path was built, then replaced by the window's own popup at the close |
> | EP-15a | M ≈ 1 h | 1 h 53 min, 48 active | the first colours read off the window's theme (seven hues, then three of its own); a selectable label takes a row's click |
> | EP-15b | M ≈ 1 h | 1 h 21 min, 50 active | split at pickup, the window's Revert becoming EP-15c; a trial needs a session row that nothing in `crw` writes |
> | EP-15d | S ≈ 30 min | 24 min, all active, after EP-15b's record commit in the same session | eframe 0.37 gives an immediate viewport no window while the window is hidden; the crash log named the place |
> | EP-15c | M ≈ 1 h | 1 h 7 min, 37 active | a preview gone stale could leave work made after it uncopied, so the plan is read again at Revert |
>
> EP-9 … EP-15c took 15 h 28 min of session against 13 h 30 min planned (115 %), 10 h 27 min of it
> active (77 %); the waits, about 5 h, were the owner's answers and trials (EP-11's launch, EP-15's
> popup, EP-15a's colours, EP-15b's and EP-15c's revert). An M took 37 to 76 minutes active (52 on
> average), the L's 62 and 53, the S's 24 to 45. Apart from the briefs, a planning session of 19
> minutes wrote the rows EP-15a and EP-15b at the owner's request (`d8c166a`). P1 had run 6 h 17
> min of session against 9 h. **Four lessons.** (1) The sizes hold for active work, and the
> owner's trials add wall time they do not count: every L so far took about an hour, an M about
> 52 minutes, while an M with trials ran up to 2 h 45 min; the sizes stay (decision 1), and the
> autopilot's caps, which count the wall clock, meet this at EP-19. (2) A supervised probe finds
> what the stub cannot: EP-9's deny rule, EP-11's twice-decoded link, and EP-14's missing
> permission host, which had disabled every headless question since EP-9 without a sign. A brief
> that drives the real CLI keeps one probe on the owner's yes. (3) The owner's use of the real
> window keeps finding what the fake portfolio hides, and P2 grew from 8 rows to 13 by it: OT-6's
> three rows for one tab became EP-13's rule, toasts that were off became the popup, a vanished
> window became EP-14a and then EP-15d, a revert made by hand became EP-15b and EP-15c. The owner
> task after each brief that changes the window stays the practice. (4) The ledger cannot time a
> brief yet: a session that starts while no window runs gets no row (EP-13 … EP-15a's have none),
> and a VS Code tab left open runs its row on for hours (EP-15b's and EP-15c's read 20 h 50 min and
> 18 h 8 min). The transcripts stay the re-plans' source, and EP-26 decides the measure.
>
> **Item 2, addenda.** Three, dated 2026-11-04 and citing EP-16: D-4 (P2's pace, the totals with
> EP-35), D-5 (the switch: every EP of this project from the window, the session's check of its
> opener and its row), D-16 (Visual stays the default for new reasons; EP-35 resumes a run a crash
> cut short). No new number.
>
> **Item 3, parked items.**
>
> | Brief | Parked item | Disposition |
> |---|---|---|
> | EP-9 | The launch, the prompt, the tiles, the hooks; the caps; usage attribution | EP-10 … EP-15, built; EP-19 and EP-18 ahead, as planned |
> | EP-9 | Raising `MIN_CLAUDE_CODE`; moving a pin | the minimum stays 2.1.261; the pins kept (decision 2) |
> | EP-10 | Launch defaults, rules and templates written from the dialog; a `regex` crate | v2 §1, since EP-10 |
> | EP-33 | Proposals that expire; one list of every dismissal | v2 §9, since EP-33 |
> | EP-11 | Resuming by the link's `session`; Focus; a visual session's start and end | Focus and the hooks built (EP-14, EP-12); the resume is EP-35 (decision 8) |
> | EP-11 | A window brought forward by its handle; a model or effort passed to the extension; a `crw` command that launches | v2 §1, since EP-11 |
> | EP-12 | The `@<project>` prompt hook | EP-20, as planned (D-20 addendum) |
> | EP-12 | Chaining another status line; rows for sessions seen only by the poll | v2 §1, since EP-12; EP-28's start at login narrows the second |
> | EP-13 | Transcripts of the sessions started elsewhere | dropped: the tool reads no transcript (D-26) |
> | EP-13 | Resuming a visual or manual session by `session=`; a headless one after a restart | the first is EP-35; the second stays in v2 §1 (decision 8) |
> | EP-14 | "Allow always"; a free reply; a request kept across a restart; a refused handshake restarted on the defer route | v2 §1, since EP-14 |
> | EP-14 | Answering a visual or manual session | dropped: never (D-19) |
> | EP-14a | Finding the lost panic of 2026-11-03 | done: the log named the next one, the popup's (EP-15d) |
> | EP-14a | A dialog at the moment of a crash | v2 §8 |
> | EP-14a | A headless run kept alive past the window | dropped: D-29 ends it on purpose; EP-35 resumes it in VS Code |
> | EP-14a | AccessKit turned off | dropped: declined by the owner at EP-14a |
> | EP-15 | ntfy and an urgent message's popup; the usage notifications | EP-20, EP-17, as planned |
> | EP-15 | The app's icon, start at login, single instance; Pause autopilot; Enable remote view | EP-28, EP-19, EP-31, as planned |
> | EP-15 | Windows toasts | dropped: off on the owner's machine (D-21 addendum) |
> | EP-15 | A question answered in the popup; a notification from `crw`; a switch per kind | v2 §9 |
> | EP-15 | A notification for a session closed by hand or a stopped run | dropped by design (D-21 addendum) |
> | EP-15a | Folding segments | dropped: the owner's choice at EP-15a |
> | EP-15a | References and links that open what they name | v2 §9 |
> | EP-15a | Rewording notes; the prompt, transcript and question texts set as notes; a row's text selectable | dropped by design (D-25 addendum) |
> | EP-15b | The window's Revert | done at EP-15c |
> | EP-15b, EP-15c | Undoing a commit the remote has | v2 §1, since EP-15b |
> | EP-15b | The start's index restored exactly | v2 §1 |
> | EP-15b | Starts for sessions that run while no window listens | done by the derived target (D-26 addendum); start at login (EP-28) narrows it |
> | EP-15d | Moving eframe | nothing to move: 0.37.0 is the newest (decision 2) |
> | EP-15d | Reporting the defect upstream | it is egui#8470, watched with no comment (decision 3); EP-23's note |
> | EP-15c | A Revert for a session with no session id | dropped: a lapsed launch or an unregistered session has no start to return to |
> | EP-15c | The copy's folder opened from the dialog | v2 §1 |
> | EP-14a, EP-15b (to EP-16) | A Resume where a crash took the tab | EP-35 (decision 8) |
> | EP-15a (to EP-16) | The README's pictures; a trailing `[urgent]`; the Progress section past a narrow pane's edge | EP-23, EP-20 and EP-26 (decisions 9, 10, 11) |
> | EP-11, EP-14 (to EP-16) | The default launch mode | Visual kept (decision 5) |
>
> **Item 4, the switch.** Rule 11 of `CLAUDE.md` says every EP is launched from the tool from EP-17
> on, how (Start, then Open in VS Code), why Start headless suits only a brief that builds no
> release, and what a session checks at its start (its opener, its row) or says when started by
> hand. `rules::DEFAULT_LAUNCH_MODE` stays Visual; its comment, which still gives EP-14's reason,
> is EP-18's to reword. EP-17's launch is recorded below.
>
> **Item 5, roadmap.** A re-plan paragraph in the header (41 briefs, 11 S · 24 M · 6 L ≈ 41.5
> session-hours, 18 open); the EP-16 row linked; EP-35 in P3 after EP-18, with its bullet, and
> EP-23 depending on it; P3's rationale and standing decisions with dated lines; pickup notes under
> EP-17, EP-18, EP-19, EP-20, EP-22 and EP-23, and the later notes P2 changes under EP-26 and EP-28;
> the standing rules' "from EP-16" brought to EP-17; risk 3 brought up to date and risks 14 (a
> headless run lives in the window) and 15 (the ledger knows what a running window heard) added.
> `v2-notes.md` gains five entries holding eight items and marks EP-13's resume line as EP-35. The
> README's status paragraph closes P2.
>
> **Verification.** `cargo test` 357 passed at pickup (85 + 1 + 1 + 1 + 232 + 2 + 7 + 6 + 11 + 11);
> no code, dependency or build file changed, so nothing was built. `crw roadmap .` with `CRW_HOME`
> on a scratch folder: 6 phases, 42 rows, 23 done, next EP-16, 30 decisions with 59 addenda, no
> problem; every EP reference in the roadmap resolves to a row (EP-0 … EP-35 with the lettered
> ones) and none to a decision passes D-30. The word list found nothing in the working diff's added
> lines or in this brief.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-04).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Re-cut the sizes after the retro | Keep the sizes | D-4 addendum with P2's pace |
> | 2 | The pins | Keep every pin | No dependency change; EP-23 checks claude-codes 2.1.318 again |
> | 3 | The upstream report of the popup's crash | Watch egui#8470, no comment | EP-23's note |
> | 4 | P3's order | Keep the order | EP-17 → EP-18 → EP-35 → EP-19 → EP-20 → EP-21 → EP-22 → EP-23 |
> | 5 | The default launch mode | Keep Visual | D-16 addendum; EP-18 rewords the constant's comment |
> | 6 | The first self-launched session's mode | Visual | D-5 addendum; rule 11 |
> | 7 | When it starts, and who checks its row | Right after the push, checked here | Recorded below |
> | 8 | Resume after a crash | A new S row, EP-35, before EP-19 | EP-35's row and bullet; D-16 addendum; v2 §1 marked |
> | 9 | The README's pictures | Retaken at EP-23 | EP-23's note |
> | 10 | An inbox message's trailing `[urgent]` | Fold into EP-20 | EP-20's note |
> | 11 | The Progress section in a narrow pane | Fold into EP-26 | EP-26's note |
> | 12 | The three addenda | Accept as written | Written |
> | 13 | The roadmap edits | Accept as written | Item 5 |
> | 14 | The v2 notes | Accept as written | Five entries, eight items (the question said six; corrected in the next round) |
> | 15 | Rule 11 and the README's status | Accept as written | As drafted |
> | 16 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
> | 17 | The owner's uncommitted tick of OT-13 | Commit it with EP-16 | In the first commit |
> | 18 | The line recording EP-17's ledger row | A third commit, pushed once the row is found | Below |
> | 19 | The owner's finding at the self-launch: the dialog's effort did not reach the tab | EP-18's notes | EP-18's bullet |
> | 20 | Recording the self-launch's finds | Notes in the roadmap too | EP-18's and EP-22's bullets, in the third commit |
>
> **Routine choices (the session's call, logged).** (1) EP-35 takes the next free number, the
> house rule for an added brief, and sits after EP-18 so that EP-17 stays the first self-launched
> session. (2) EP-14a is timed from the owner's return to EP-14's session the next afternoon
> (17:57 UTC), the transcript holding EP-14, a 55-minute pause and a night before it; EP-15d from
> EP-15b's record commit in the same session. (3) The planning session that wrote EP-15a's and
> EP-15b's rows is reported apart, as EP-8 reported EP-7's IN-4 tail. (4) The constant's comment
> is left to EP-18, which edits `rules.rs`, so that this re-plan changes no code. (5) The v2 notes'
> header now names EP-15b's revert item, which carried no marker. (6) The timing scripts (the
> standard library over the transcripts' timestamps and brief paths) stayed in the session's
> scratchpad.
>
> No owner tasks; no messages for other projects. **Next: EP-17** (the usage meter), the first
> self-launched session.
>
> **After the commits (≈ 46 min in all).** `2b1db41` and the record commit `3c25d9e` were pushed
> together at 21:52 UTC, and CI run 41920131724 on windows-latest passed. The owner then launched
> EP-17 from the window, the first self-launched session (D-5): F5, S on this project's row, Visual,
> Open in VS Code, and the opener sent from the new Claude Code tab in this project's window. Its
> ledger row reads `visual`, EP-17, session `66483ca0`, started 21:57 UTC, open while it runs
> (`crw sessions clauderoadmapwrangler`, read only, at 22:00 UTC): the launch was taken up at the
> session's first work, as D-26's EP-13 addendum has it. The owner's finding at the launch: the
> effort chosen in the dialog (max) did not reach the tab, which opened at VS Code's own (xhigh).
> The extension's `/open` link reads only `prompt` and `session` (its 2.1.318 handler, read here),
> and the panel's effort is a preference it stores for itself, so the dialog's model and effort are
> only the opener's suggestion in Visual mode; the dialog's wording is EP-18's (decision 19). The
> same handler has an `/install-plugin` path, noted for EP-22 (decision 20). This paragraph and
> those two notes are a third commit, on the owner's yes, touching neither EP-17's files nor its
> work.
