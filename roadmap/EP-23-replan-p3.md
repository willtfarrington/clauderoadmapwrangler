---
ep: 23
title: Re-plan P3, opening with the P3 walk
size: M
model: claude-fable-5-2
effort: xhigh
status: done
---

# EP-23 — Re-plan P3, opening with the P3 walk

**Size:** M · **Depends on:** EP-17 … EP-21a (the 21 briefs of P3) · **Blocks:** EP-38, EP-39,
EP-24 … EP-29 (P4 re-cut and confirmed with pickup notes)

## Context

P3 closed with EP-21a on 2026-11-11. The window meters the account's usage and gives each
session and project its share, chains briefs on autopilot within caps and restarts itself into
an installed build between two runs, delivers the projects' mail and takes the owner's own
messages, pushes its notifications to the owner's phone, writes the house-style files into a
project, and ships the roadmap method as a plugin; a prompt hook answers an `@<project>` with
that project's rules and next row. This re-plan closes the phase (D-4). It is the first to open
with the **walk** (D-10 addendum, 2026-11-08): the residual-risk paragraphs of EP-19b … EP-21a
compiled into steps, settled from the records where the records can say and taken with the owner
where only their eyes, hands or judgement can. Then the retro, the addenda, P4 re-cut with pickup
notes, and `v2-notes.md` grown. The plugin's `replan` skill (EP-22), installed from GitHub at
OT-23, is the method this session follows, which is its first real use.

**Session start (2026-11-11, 11:27 local time).** Started from the window (Start, Visual, Open
in VS Code); the prompt is the tool's opener, and `crw sessions clauderoadmapwrangler` lists the
row (`3b9f4501`, visual, EP-23, open). The launch suggested fable at xhigh, and the panel runs
claude-fable-5-2 at xhigh (the owner's answer). No owner task and no inbox message is open; the
tree was clean at `9c6793e`. The field check read, since EP-21a's record commit: `crash.log` (the
one report of 2026-11-03, nothing newer); `crw notifications` (EP-21a's own waits and its
finished turn at 00:48, nothing since); `crw mail` (nothing since the owner's messages of
2026-11-10); `crw sessions` (this row; one short manual session of another project at 00:33);
`crw autopilot` (on for this project and one other, both `auto`, nothing chained since the day's
cap ended the chain after EP-20b); `crw doctor` (the installed copy built at 00:36, EP-21a's;
the hooks run it; the plugin's copy matches the build); `crw signals` (the five events and the
status line run the installed copy; the window listens). Nothing odd to triage.

**Verified at pickup.**

- **History.** 48 commits since EP-16's record commit: two per brief for EP-17 … EP-21a (EP-20c
  five: its work, its record, the review parked, the review's fixes and their record), the
  shortcut icon (`4d642ea`, made in EP-18's session), EP-16's own follow-up (`186a38b`) and the
  planning session of 2026-11-08 (`811d9a6`). `origin/main` stands at `cf429a3`, so EP-20d's and
  EP-21a's four commits are local.
- **The transcripts** of EP-17 … EP-21a are filed under this project's slug, one session per
  brief, except EP-37 with two (the first stopped mid-review). The planning session of
  2026-11-08 is there too, with no brief. A stdlib script read each entry's timestamp and each
  Write or Edit's file path, nothing else.
- **The ledger** (read only: `crw sessions`, and the `event` rows through a read-only SQLite
  connection, since `crw autopilot` prints the latest twenty) holds, since EP-16, 15 headless
  rows (8 this project's, 7 another's), 19 visual and 14 manual; 28 `autopilot` rows, 19 `mail`,
  17 `tick`, one `formalise` (OT-22's, on a registered project) and one `resume`.
- **`crw doctor`:** claude 2.1.316, the standalone binary's version still, the extension's
  bundled one now 2.1.322; gh 2.103.3, code 1.145.3; headless ready with the effort levels
  low … max, no ultracode, `--max-budget-usd` and `--plugin-dir`.
- **crates.io:** eframe 0.37.0, egui_extras 0.37.0 and egui_tiles 0.18.0 are the newest releases
  and the pins; egui_plot's newest is 0.38.0 (EP-26's); claude-codes' newest is 2.1.323 against
  the pin 2.1.316 (D-15); egui_commonmark's newest is 0.26.0 (a candidate for the chat tile's
  replies, EP-24).
- **Upstream:** egui#8470 is still open (labelled bug, last touched 2026-09-10), so the D-21
  addendum's rule against immediate viewports stands.
- **The plugin** this session loaded is the GitHub install's copy at `cf429a3` (the user-level
  plugin cache), which is OT-23's route working; `crw doctor` says the build's own copy for the
  tool's headless runs matches the build.

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-11)

1. **The panel's effort:** xhigh.

The walk's answers, the retro's and the re-cut's are recorded in the completion note as they are
given.

## In scope

1. **The walk.** The steps compiled from the residual-risk paragraphs of EP-19b, EP-19c, EP-36,
   EP-37, EP-37a, EP-20, EP-21, EP-22, EP-22a, EP-20a, EP-20c, EP-20b, EP-20d and EP-21a and from
   the segments parked in this row's brief note. Each is settled from the records where they can
   say (the chain log, the mail log, the notifications, `crw doctor` and `crw signals`), and put
   to the owner where only they can: what they saw, a live step with the window beside VS Code,
   or a judgement the note left to the walk. Each finding becomes a row, an addendum or a v2
   note at once.
2. **The retro.** EP-17 … EP-21a, the planned size against the active and the wall time from the
   transcripts (the first entry, or the previous EP's record commit, to the EP's record commit;
   gaps over five minutes left out), the planning session of 2026-11-08 beside them, the ledger
   rows set beside the transcripts, and the first week of autopilot runs: how many chains, how
   many ends and why, the holds, whether a chain ran into a re-plan.
3. **Addenda** to the decisions P3 changed or the walk settles, dated 2026-11-11 and citing
   EP-23; nothing rewritten.
4. **P4 re-cut:** its rows confirmed, split, re-ordered or added with sizes and depends-on, the
   totals recounted, a dated sentence under the ordering rationale, pickup notes as
   `**From EP-23:**` segments naming code, files and commands; the pins checked (D-15).
5. **`v2-notes.md` grown** with what P3 named and did not build, each with its source, and its
   stale marks brought up to date.
6. **The README's two pictures** retaken on the fake portfolio from the neutral root (EP-36's
   method), since P3 changed the window after EP-36 took them.
7. **The records:** the roadmap's header paragraph, the P3 table (EP-20c's row gains its link),
   the risks list, this brief's completion note; a review of the diff by a subagent given the
   diff alone (rule 6, an M); rule 3's word check before each commit.

## Out of scope

- Code. A finding that needs code becomes a row; this brief changes records and pictures only.
- Moving the egui family or the protocol crate, unless the owner asks for it at the pins
  question (D-15 leaves it to a re-plan, and nothing forces it today).
- Any write into another project or into the owner's registry by the session (rules 4 and 13):
  a live step of the walk is the owner's own hand in the window.
- Reading a transcript's text (D-3, D-26): the timings read timestamps and file paths.
- The push: the owner's.

## Verification / acceptance

- Every segment of this row's brief note has an answer: a row, an addendum, a v2 note, or a
  line in the walk's table saying what the records showed.
- The retro's table sums to the totals stated; the roadmap's totals (briefs, S, M and L,
  session-hours, open rows) are recounted from the tables after the re-cut and checked by the
  review subagent against the diff.
- `crw roadmap .` reads the roadmap after the edits with the next EP as the re-cut names it and
  no new problem.
- `cargo test` passes as at EP-21a (no code changes); the two pictures are looked at before they
  replace the README's.
- Rule 3's word check prints nothing for the staged diff; two commits in the house pattern, not
  pushed.

**Residual risk.** This brief changed records and two pictures, so its risk is in what the
records now promise. (1) **The pickup notes name code by reading, not by building:** each
identifier was found with a search and checked again by the review subagent, and a brief that
finds one moved says so. (2) **The two cleared crates were read on crates.io only:**
`egui_commonmark` 0.26.0 and `egui_plot` 0.38.0 declare the pinned egui, but neither has been
resolved against this lock; EP-24a and EP-26 find out at their first build and have their
fallbacks named. (3) **The test letter goes out after this session:** its popup and its push are
unseen until EP-38's field check reads `crw notifications`. (4) **The first migration's result
is the owner's to judge:** the ledger and `crw roadmap` say it completed with the counts held,
not that the converted tables read well. (5) **The timings count a gap over five minutes as a
wait,** which in a headless run can be a long build or a subagent rather than the owner; the
method is EP-8's and EP-16's, kept so that the three retros compare. What the walk carried is
listed in EP-29's note, and risk 17 holds the push guard.

> **Completion note (2026-11-11; claude-fable-5-2/xhigh; about 2 h 20 min of wall time,
> thirty-one questions in eight rounds).** A visual session started from the window (ledger row
> `3b9f4501`, from 11:27), following the plugin's `replan` skill from the owner's GitHub install.
>
> **Item 1, the walk.** Fourteen residual-risk paragraphs and the segments parked in this row's
> note gave the points below. "Records" means settled from the ledger and the tool's own reads;
> "owner" means the owner's answer; "live" a step the owner took in the window during the
> session.
>
> | From | Point | How | Outcome |
> |---|---|---|---|
> | EP-19b | The offer of `auto`, the limits' pickers, the pin | records, owner | Closed: `turned on by the owner, with permission mode auto`; the pickers wrote the limits chosen |
> | EP-19b | The push guard in a real run | owner | Carried as risk 17 |
> | EP-19b | The gate with real hooks, the section's verdict | records | Closed: no chain ended on an idle tab since; `crw autopilot` prints both projects' mode |
> | EP-19c | The Keep pins, a chain with a kept value | owner | Carried to EP-29's walk: unused on the real registry |
> | EP-36 | The list's badge columns at the default width | owner | EP-38 (1): the title gives way first; D-25 addendum |
> | EP-36 | Risks counted as waiting | owner | EP-38 (2): tasks and inbox only; D-10 addendum |
> | EP-36 | The opener's quote | owner | EP-39 (2): the lines joined; D-17 addendum |
> | EP-37 | The running exe renamed, the hooks, the first start | records | Closed: every install since renamed the running exes; `crw signals` and `crw doctor` name the installed copy |
> | EP-37a | The chained restart | records | Closed (OT-20): four holds, each taken about 20 s later, none refused |
> | EP-37a | The hidden start, a restart under the owner's hands | owner | Closed: came back as it was, never in the way |
> | EP-37a | The ended runs' tabs | owner | Not missed: a v2 note |
> | EP-20 | The first round, the first delivery | records | Closed (OT-21): fifteen first readings, no block counted; IN-8 delivered. A delivery between two projects is carried |
> | EP-20a | The push to the phone | records | Closed (OT-24, IN-9): `ntfy · test`, then each popup |
> | EP-20a | The gist on real results | records | Closed: the two finished popups since read as their results' first sentences |
> | EP-20a | A real urgent letter | owner | A test block handed on by this brief; EP-38 reads the log |
> | EP-20a | A hand start on an older window | owner | EP-39 (1): the dialog says so and offers the restart; D-29 addendum |
> | EP-20c | The first compose from the window | live | Closed: IN-10 written at once with the note, declined here as a test |
> | EP-20c | A changed inbox with no mail, a restart with an urgent letter in flight | — | Carried to EP-29's walk: neither has happened |
> | EP-20b | The hook installed, a mention answered | records, owner | Closed (OT-25): `UserPromptSubmit runs the installed copy hook` |
> | EP-20b | What the context holds | owner | EP-39 (3): the row first, the rules in short; D-20 addendum |
> | EP-20d | The test block's delivery | records | Closed: `delivered EP-20d:… as IN-8` |
> | EP-20d | The rewording's two wording points | owner | Parked in the v2 notes until a block is reworded |
> | EP-21 | The first real write | records | Closed (OT-22): a `formalise` row of 2026-11-10 |
> | EP-21, EP-21a | The first Migrate… | live | Run at the walk: the dialog as described, the run completed in 14 min and stopped before committing; D-9 addendum |
> | EP-21a | The first Formalise… from the window | live | Closed: as described, four files created, the `formalise` row |
> | EP-21a | The House style section on most rows | owner | EP-38 (3): only when it matters; D-11 addendum |
> | EP-21a | A revert that redoes a formalise | — | Carried to EP-29's walk |
> | EP-22 | The GitHub install, the skills in a session | records, owner | The install works (this session ran its `replan` skill); the skills barely used yet, their wording carried |
> | EP-22 | The gate's reading of a re-plan | records | Closed: one misreading before the fix (EP-22's own title), none since; no chain ran into a re-plan |
> | EP-22a | A headless run loading the build's copy | — | Carried to EP-24, whose runs load it |
> | EP-35 | The headless resume after a restart | owner | Kept parked in the v2 notes |
>
> **Item 2, the retro.** From the session transcripts (each session's first entry to its EP's
> record commit; active time leaves out gaps over five minutes), in minutes unless hours are
> written:
>
> | EP | Size | Launch | Active | Wall | What bit |
> |---|---|---|---|---|---|
> | EP-17 | M | visual | 1 h 17 | 1 h 35 | The first self-launched session and the longest M |
> | EP-18 | S | visual, by hand | 40 | 40 | Split at pickup (EP-18a); the shortcut icon's commit rode along |
> | EP-18a | S | visual | 32 | 32 | Split at pickup (EP-18b) |
> | EP-18b | S | visual | 39 | 39 | |
> | EP-35 | S | visual | 33 | 33 | |
> | EP-19 | L | visual | 47 | 47 | After an abandoned first attempt of 11 min, reverted; split (EP-19a) |
> | EP-19a | S | visual | 43 | 1 h 16 | One wait of 33 min; IN-7 became EP-19b |
> | EP-19b | M | visual | 55 | 1 h 48 | Split at pickup (EP-19c) |
> | EP-19c | S → M | visual | 42 | 1 h 40 | Re-scoped at pickup; four waits |
> | EP-36 | S | visual | 35 | 1 h 12 | Across a night (8 h 35 with it) |
> | EP-37 | M | visual, two sessions | 1 h 11 | 3 h 03 | The first stopped mid-review; split (EP-37a) |
> | EP-37a | M | visual | 51 | 1 h 16 | |
> | EP-20 | M | headless | 49 | 49 | The first headless brief here; split (EP-20a, EP-20b) |
> | EP-21 | M | headless, chained | 42 | 42 | The first brief a chain carried across a restart; split (EP-21a) |
> | EP-22 | M | headless | 50 | 2 h 10 | Started by hand, the gate misreading its title; one wait of 1 h 20; split (EP-22a) |
> | EP-22a | S | headless, chained | 23 | 32 | |
> | EP-20a | M | headless | 59 | 1 h 10 | Started by hand on an older window; split (EP-20c) |
> | EP-20c | M | headless, chained | 44 | 1 h 21 | The review back after the commit, its fixes a commit of their own; split (EP-20d) |
> | EP-20b | S | headless, chained | 23 | 23 | No question |
> | EP-20d | S → M | visual | 45 | 51 | Re-scoped at pickup |
> | EP-21a | M | visual | 57 | 1 h 04 | |
>
> In all 15 h 58 min of active work against 18 h planned (89 %; P1 62 %, P2 77 %) and 31 h 27
> min of wall time, about 24 h without EP-36's night (134 %). An S took 23 to 43 minutes of
> active work (34 on average), an M 42 to 77 (54), the one L 47. The seven headless briefs took
> 4 h 50 min against 6 h planned, an M 49 minutes on average against 57 for a visual M, and their
> ledger rows' run lengths match the transcripts' wall time within a minute, which a visual
> row's does not (EP-19c's reads 14 h 45 min). The planning session of 2026-11-08 took 22 minutes of
> active work and 30 of wall time. By the ledger's shares, this project's 21 sessions since the
> weekly reset of 2026-11-07 took 13.5 points of the weekly limit, EP-20c alone 3 (the one M on
> fable at xhigh among the headless runs). P3 grew from 8 rows to 22: eleven by a split at
> pickup, one at the owner's request (EP-19b), two from the planning session (EP-36, EP-37).
>
> **The first week of chains** (2026-11-06 … 2026-11-10; D-22 addendum): seven chained starts
> in two projects; seven ends (three daily caps, two before a re-plan of which one was EP-22's
> misread title, one for idle tabs before EP-19b, one run stopped by the owner); four holds, all
> for the window's restart, all taken; no gate refusal for a tree, a task or the usage; no run
> capped; no chain into a re-plan.
>
> **Item 3, addenda.** Twelve, dated 2026-11-11 and citing EP-23: D-4 (P3's pace, the pre-split,
> the totals), D-9 (the first migration), D-10 (the walk as it ran, the cadence kept, what counts
> as waiting), D-11 (the House style section), D-13 (the mail at the walk), D-15 (the pins),
> D-17 (the quote), D-20 (the hook's context), D-22 (the first week), D-25 (the list's width),
> D-27 (the chat in three, the plugin's first use), D-29 (the restart's week, the hand start).
>
> **Item 4, the re-cut.** P4 is thirteen rows: EP-38 and EP-39 (the walk's six fixes, M each)
> open it; EP-24 becomes EP-24, EP-24a and EP-24b, EP-26 becomes EP-26 and EP-26a, EP-28 becomes
> EP-28 and EP-28a, each an M; EP-25, EP-27, EP-34 and EP-29 stay. Each new row has its brief
> note and each kept row a `**From EP-23:**` segment; EP-29's holds the walk's method and what
> it carried. Counted afresh from the tables, v1 is 61 briefs, 14 S · 42 M · 5 L ≈ 59
> session-hours, 16 open once this row is checked; the notes' running totals had been one M
> short since EP-19b joined. Risks 3, 14 and 16 gained a dated line and risk 17 (the push guard)
> is new. EP-20c's row gained the link it lacked. `CLAUDE.md` rule 5 gained the copy run before
> a brief's close.
>
> **Item 5, the parked notes.** `v2-notes.md` grew by twenty items marked *(EP-23, from EP-n)*
> across launch modes, analytics, conventions, stack alternatives, operations and views, and
> three stale marks were brought up to date. A reading subagent collected the candidates from
> the 21 briefs' out-of-scope lists and completion notes; the session curated them.
>
> **Item 6, the pictures.** `docs/dashboard-light.png` and `docs/dashboard-dark.png` retaken on
> the fake portfolio from `C:\Users\Public\Projects` (EP-36's method), both looked at, the copy
> removed.
>
> **The review (rule 6).** A subagent given the diff alone, launched once the records were
> drafted, reported after about fifteen minutes. It recounted the tables (62 rows, 14 S · 42 M ·
> 5 L after EP-0, 16 open; the retro's sums within a minute), found every code name the notes
> cite where they say it is, and read no new row's title as a re-plan by `autopilot::is_replan`.
> Its fifteen points, all fixed:
>
> 1. The `replan` skill has nine steps, not eight (the D-27 addendum; the question put to the
>    owner said eight).
> 2. Risk 16 said EP-36's findings waited five briefs for EP-38: eleven, for EP-38 and EP-39.
> 3. Stale after the cut: "six engines" in the Sizes paragraph and risk 15's "EP-28's start at
>    login" each gained a dated clause; `CLAUDE.md` rule 10 now names `egui_commonmark`, and the
>    D-15 addendum no longer says `egui_plot` joins a rule it was under.
> 4. Figures set beside the wrong measure: P1's 70 % is of session time (62 % of active work,
>    which is what the P3 and P2 figures are; the question put to the owner said 70); an M in P1
>    was 45 minutes of session and about 42 of active work; EP-20 was a headless run, not a
>    chained one; the ledger matches the transcripts within a minute, not to the minute.
> 5. Notes that promised what the code or the records do not hold: the retros' tables give
>    times, not start and record commits (EP-25); no sort reads the Waiting count (EP-38);
>    `Column::GIVE_WAY` has five choices and four steps; the House style section and the menu
>    item both show for the unformalised state today, which EP-38 keeps; EP-26a cited a rule
>    EP-38 does not have.
> 6. The header said "since EP-37" for what EP-37a built; EP-24b lacked EP-39 and D-20; EP-28a's
>    note gained a pointer to the segments EP-28's note keeps; the v2 notes' migration item
>    gained its EP-23 mark; a blank line inside the notes list and four short lines were closed.
>
> **Verification.** `cargo test --workspace`: 555 passed (354 in `core`, 146 in `app`), as at
> EP-21a, no code having changed. `crw roadmap .` reads 62 rows in six phases, P4 as 13, the
> next EP this one until its row is checked and EP-38 after, and no problem. Rule 3's word check
> printed nothing for the staged diff. No release build and no install (decision 30).
>
> **Owner decisions (`AskUserQuestion`, 2026-11-11).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The panel's effort | xhigh | The front matter and this note |
> | 2 | The chained restarts as seen | Came back as it was | EP-37a's risks 2 and 4 closed |
> | 3 | The ended runs' tabs after a restart | Not missed | A v2 note |
> | 4 | A hand start while a newer build is installed | The dialog says so and offers it | EP-39 (1); D-29 addendum |
> | 5 | The first compose from the window (live) | Done, as described | IN-10 declined as the test |
> | 6 | The limits' pickers on the real registry | In the window, as expected | EP-19b's risk 1 closed |
> | 7 | The Keep pins | Carry to P4's walk | EP-29's note |
> | 8 | The push guard | Carry it as a risk | Risk 17 |
> | 9 | The list at its default width | The title first | EP-38 (1); D-25 addendum |
> | 10 | Risks counted as waiting | Tasks and inbox only | EP-38 (2); D-10 addendum |
> | 11 | The opener's quote | Join the lines | EP-39 (2); D-17 addendum |
> | 12 | The House style section | Only when it matters | EP-38 (3); D-11 addendum |
> | 13 | The first Formalise… from the window (live) | Done, as described | EP-21a's risk 1 closed |
> | 14 | The prompt hook's context | The row first, the rules in short | EP-39 (3); D-20 addendum |
> | 15 | The rewording's wording points | Park both | v2 notes |
> | 16 | A test urgent letter | Yes, hand it on | The block below; EP-38's field check |
> | 17 | Migrate… and the revert's redo | Migrate one now | Started as described; the revert's redo carried |
> | 18 | The plugin's skills | Barely used yet | Their wording carried to EP-29's walk |
> | 19 | The headless resume after a restart | Keep it parked | v2 notes |
> | 20 | The protocol crate's pin | Keep 2.1.316 | D-15 addendum; EP-29 looks again |
> | 21 | The sizes | They stay | D-4 addendum |
> | 22 | P4's large rows | Pre-split the three | Seven M briefs; D-4 addendum |
> | 23 | The cadence | Keep it | D-10 addendum; rule 5's sentence |
> | 24 | The walk's six fixes | Two M rows | EP-38, EP-39 |
> | 25 | P4's order | The plan's | The ordering rationale |
> | 26 | The chat's Markdown | `egui_commonmark` 0.26.0 | D-15 addendum; EP-24a |
> | 27 | The migration's end | Still running when asked | The ledger showed it completed minutes later; D-9 addendum |
> | 28 | The README's pictures | Keep both | Committed with this brief |
> | 29 | The drafted records | Signed off | As drafted, with the review's corrections |
> | 30 | The release build and the install | Skip both | No code changed; a deviation from the opener, logged |
> | 31 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The records first.** A point the ledger or the tool's reads could settle was not put to
>    the owner; EP-29's note says so for the next walk, and the `replan` skill's missing sentence
>    is carried there too.
> 2. **The whole chain log** came through a read-only SQLite connection to the ledger, since
>    `crw autopilot` prints the latest twenty rows; nothing was written (rule 13). `crw roadmap`
>    was run once, read-only, on the migrated project to see that its counts held; no name is
>    recorded here (rule 3).
> 3. **Numbers.** EP-38 and EP-39 take the next free numbers, as rows added at a re-plan do
>    (D-4's EP-8 addendum); the pre-split rows take letters, as a split at pickup does. EP-24's
>    Size cell reads `L → M (cut beforehand at EP-23)`, the re-scope form the parser reads.
> 4. **The totals** were counted from the tables with the tool's parser and a count by size; the
>    older notes keep their figures as written, and the D-4 addendum says where they went short.
> 5. **No owner task:** nothing here needs the owner's hands only. What the walk carried is in
>    EP-29's note, not in tasks (D-10 addendum).
> 6. **The chat's two open questions** (what its session may read under the data root, and which
>    writing commands it may run) are written as EP-24's and EP-24b's first questions, not
>    decided here.
> 7. **The timing script** and the ledger reader stayed in the scratchpad.
>
> No message for another project. **Next: EP-38**, the list and Detail fixes.

> **Handed on (to clauderoadmapwrangler).** A test letter from EP-23's P3 walk, at the owner's
> request, for the first real urgent popup and its push: nothing to build; decline it as a test
> and tick it, saying what `crw notifications` showed for it. [urgent]
