---
ep: 29
title: Re-plan P4, opening with the P4 walk; P5 charters upgraded to full briefs
size: M
model: claude-fable-5-2
effort: high
status: done
---

# EP-29 — Re-plan P4, opening with the P4 walk; P5 charters upgraded to full briefs

**Size:** M · **Depends on:** EP-38 … EP-28a (the 14 briefs of P4) · **Blocks:** EP-30 … EP-32
(P5 re-cut from its charters, with pickup notes)

## Context

P4 closed with EP-28a on 2026-11-14. Since the P3 re-plan the window hosts a read-only chat over
the portfolio whose replies draw as Markdown and end in buttons that open the window's own
dialogs, imports each project's history from git and keeps a daily count, draws a heatmap, a
burn-up and a velocity with a projection, shows them in the list as sparklines and as cards,
re-reads one project when its session or its chain ends, writes the site's cards, a daily export
and a CSV, runs the hosting and backup actions behind a confirmation, and reports on itself in
`crw doctor` and a Diagnostics pane with a rolling log. It holds a chain at the five-hour limit
again, yields to a window already running, starts at login hidden in the tray, closes the ledger
rows no window heard end, and puts `crw` on PATH. This re-plan closes the phase (D-4). It opens
with the **walk** (D-10 addendum, 2026-11-08), the second one: the residual-risk paragraphs of
P4's fourteen briefs and the segments parked in this row's note, settled from the records where
the records can say and taken with the owner where only their eyes, hands or judgement can. Then
the retro, the addenda, **P5's three charters upgraded to full briefs** with pickup notes (the
plan left the web pages' routes and templates to this session), and `v2-notes.md` grown. The
plugin's `replan` skill is the method, from the owner's GitHub install.

**Session start (2026-11-15, 13:34 local time).** Started from the window (Start, Visual, Open in
VS Code); the prompt is the tool's opener, and `crw sessions clauderoadmapwrangler` lists the row
(`8a16b96b`, visual, EP-29, open). The launch suggested claude-opus-5-5 at max; the panel runs
claude-fable-5-2 at high, read from the `model` and `effort` fields of this session's own
transcript entries (no text read). No owner task and no inbox message is open: the owner ticked
OT-27 and OT-28 in the window at 13:31 and 13:33, which is the tree's one uncommitted change at
`23aab24` and is committed with this brief. `main` is ten commits ahead of `origin/main`. The
field check read, since EP-28a's record commit:

- `crash.log`: the one report of 2026-11-03, nothing newer.
- `crw notifications`: EP-28a's finish and the chain's end at the re-plan row (2026-11-14 14:54),
  then one manual session's turn today at 13:25, with its push.
- `crw sessions`: this row; a manual row of 13:23 open beside it (a tab opened by hand, idle);
  one chat run of 13 s on 2026-11-14.
- `crw autopilot`: not paused, on for this project and five others, all in mode `auto`; the last
  line is the chain's end after EP-28a, "EP-29 is a re-plan, the owner's to run".
- `crw mail`: nothing since EP-27's letter was delivered into the site's inbox on 2026-11-13.
- `crw doctor`: every line ok or a note. The installed build is EP-28a's (2026-11-14 14:52), the
  hooks and the status line run it, the start-at-login entry and the PATH copy are in place, the
  backup ran minutes before, and the VS Code extension now bundles 2.1.325 against the standalone
  CLI's 2.1.316.
- `crw signals`: the five events and the status line run the installed copy; the window listens.
- The rolling log (three day files): the install at EP-28a's close asked for a restart and was
  refused, the run still working; the window then reached EP-28a's build by the **first start at
  login**, today at 12:40:17 ("at login"), and its **first reconciliation** at 12:40:52 closed the
  two old rows the note names, each at its transcript's last write. A second start at 12:50:10
  ended at 12:50:12, which is the single-instance guard's yield as built (EP-40).

Nothing odd to triage: the two first real runs EP-28a left for this walk both happened and both
read as built, and the walk asks the owner only what the log cannot say (what the start at login
looked like).

## In scope

1. **The walk.** The steps compiled from the residual-risk paragraphs of EP-38, EP-39, EP-24,
   EP-24a, EP-24b, EP-25, EP-25a, EP-26, EP-40, EP-26a, EP-27, EP-34, EP-28 and EP-28a and from
   the segments parked in this row's brief note, the points carried from the P3 walk among them.
   Each is settled from the records where they can say (the ledger's rows and its `event` rows
   through a read-only connection, `crw autopilot`, `crw mail`, `crw notifications --all`, `crw
   doctor`, `crw signals`, the rolling log), and put to the owner where only they can: what they
   saw, a live step in the window, or a judgement a note left to the walk. Each finding becomes a
   row, an addendum or a v2 note at once.
2. **The retro.** EP-38 … EP-28a, the planned size against the active and the wall time from the
   session transcripts (the first entry, or the previous EP's record commit, to the EP's record
   commit; gaps over five minutes left out; timestamps and file paths only), the ledger's rows
   set beside them, and the phase's chains: starts, ends and their reasons, the holds, the first
   five-hour holds.
3. **Addenda** to the decisions P4 changed or the walk settles, dated 2026-11-15 and citing
   EP-29; nothing rewritten.
4. **P5 upgraded from charters to full briefs:** the web pages' routes, templates and update
   mechanism decided with the owner; EP-30 (the one open L) cut beforehand or kept, as P4's large
   rows were; the identity check, the remote actions and their log; the charter's open question
   whether the phone may revert; the rows with sizes and depends-on, the totals recounted, a
   dated sentence under an ordering rationale, standing decisions for the phase, and pickup notes
   as `**From EP-29:**` segments naming code, files and commands found by a survey of the tree.
   Any fix the walk finds joins as a row where the owner places it.
5. **The pins** looked at again (D-15): the egui family, `egui_commonmark`, `egui_plot`, the
   protocol crate against the CLI the runner meets, the web crates P5 needs, and egui#8470.
6. **`v2-notes.md` grown** with what P4 named and did not build, each with its source, and its
   stale marks brought up to date.
7. **The records:** the roadmap's header paragraph, the P4 table (this row linked and checked),
   the P5 section, the risks list, `CLAUDE.md` where a rule's wording follows from a decision
   here, this brief's completion note; a review of the diff by a subagent given the diff alone
   (rule 6, an M); rule 3's word check before each commit.

## Out of scope

- Code. A finding that needs code becomes a row; this brief changes records only, so there is no
  release build to make and the install at the close is put to the owner as a question (EP-23's
  decision 30 skipped both). *(At the walk the owner chose to add the `replan` skill's missing
  sentence here, text the exe carries, so the close does build and install: decisions 18 and
  38.)*
- Moving the egui family or the protocol crate, unless the owner asks for it at the pins question
  (D-15 leaves it to a re-plan).
- Any write into another project or into the owner's registry or user settings by the session
  (rules 4, 7 and 13): a live step of the walk is the owner's own hand in the window.
- Reading a transcript's text (D-3, D-26): the timings read timestamps and file paths, and this
  session's own model and effort fields.
- The README's two pictures: the P3 re-plan left their next retake to the close-out (EP-32),
  after P5.
- The push: the owner's.

## Verification / acceptance

- Every segment of this row's brief note has an answer: a row, an addendum, a v2 note, or a line
  in the walk's table saying what the records showed.
- The retro's table sums to the totals stated; the roadmap's totals (briefs, S, M and L,
  session-hours, open rows) are recounted from the tables after the re-cut and checked by the
  review subagent against the diff.
- `crw roadmap .` reads the roadmap after the edits with the next EP as the re-cut names it and
  no new problem; no new row's title reads as a re-plan to the gate unless it is one.
- `cargo test` passes as at EP-28a (no code changes).
- Rule 3's word check prints nothing for the staged diff; two commits in the house pattern, not
  pushed.

**Residual risk.** This brief changed records and one sentence of a skill, so its risk is in what
the records now promise. (1) **P5's notes name code by reading, not by building:** each
identifier came from a survey subagent's reading of the tree and was checked again by the review
subagent; a brief that finds one moved says so. (2) **The phone view's design rests on
documentation for the two things no test can see:** how a request arrives through Tailscale
Serve (its `Host`, the identity header, the `--set-path` form) was read on Tailscale's pages on
2026-11-15, not tried, and the tool's own script has no browser to run in before the owner's
phone; EP-31a's owner task and EP-32's walk are where both are first seen (risk 18). (3) **The
event stream may need a crate** the lock does not hold directly; EP-30b's note names the
fallback. (4) **The walk took the owner's word for surfaces they looked at quickly,** and one of
its questions described a card's double click wrongly; the owner's answer exposed it, and the
D-10 addendum now says where a question's description comes from. (5) **The missing export day
is explained from EP-27's own words** (the export runs at a full read of the files), not from a
trace of 2026-11-14; EP-41 finds out whether the day's turn is the whole cause. (6) **The
timings count a gap over five minutes as a wait,** which in a headless run can be a build or a
subagent; with the subagents' own transcripts counted the phase's total moves by five minutes.
(7) **The `replan` skill's new sentence reaches the owner's own sessions only after a push** and
a plugin update; the tool's headless runs get it with this brief's install.

> **Completion note (2026-11-15; claude-fable-5-2/high; about 2 h 50 min of wall time,
> thirty-eight questions in ten rounds).** A visual session started from the window (ledger row `8a16b96b`, from
> 13:34), following the plugin's `replan` skill from the owner's GitHub install. Two subagents
> read for it: one collected the fourteen briefs' findings and the v2 candidates, one surveyed
> the tree for P5's pickup notes; a third reviewed the diff.
>
> **Item 1, the walk.** The fourteen residual-risk paragraphs and the segments parked in this
> row's note gave the points below. "Records" means settled from the ledger, the rolling log and
> the tool's own reads; "owner" the owner's answer; "live" a step the owner took in the window
> during the session.
>
> | From | Point | How | Outcome |
> |---|---|---|---|
> | EP-38 | The title's floor at the owner's usual width | live | As built: the title stops at its floor of 80 before the CI column goes; kept |
> | EP-38 | A cut title's hover, the long risks list uncounted | live | Closed with the list's look, which named neither |
> | EP-39 | The dialog's Install and restart, the real round trip | owner | Never came up: carried to EP-32's walk |
> | EP-39 | The hook's new context in a real session | owner | Closed: works as wanted |
> | EP-39 | The joined quote on a real rule | records | Closed: this session's own opener quotes rule 1 with its lines joined |
> | EP-24 … EP-24b | The first real conversation: the boundary flags, the commands, the buttons, a mention | records | Closed (OT-26; IN-12's three findings fixed at EP-25); ten chat runs in the ledger |
> | EP-24a | Real Markdown, a chat's question in its tile, a deferred question's resume | owner | Fine, barely used: the two routes carried to EP-32's walk |
> | EP-22a | A headless run loading the build's copy of the plugin | live | Closed: asked which skills it holds, a chat run named the four |
> | EP-25 | The import over real logs | records | Closed: `crw history` over this project and the nine others that are no data repository and keep a roadmap, every EP dated, none flagged |
> | EP-25 | The daily counts and the import on a moved HEAD in the real window | records | The import runs (the log's "the history (imported again)"); a project not read on a day gets no count for it, the gap EP-41 (3) weighs |
> | EP-25, EP-26 | History, Progress and the graphs over the real ledger | live | Closed: they read right |
> | EP-25a | The re-read at a chained run's end and at a session's end | records | Closed: the log's lines of 2026-11-14 14:54 and 19:32 and of today |
> | EP-25a | The row menu's item | live | Closed: "Refresh this project" works |
> | EP-26a | The cards and the expanded rows under the mouse | live | They work; under Expand all the sparklines start raggedly: EP-41 (2). A double click is as built (decision 9) |
> | EP-26a | The sparklines over the real ledger | live | Too little change across phases to judge: carried to EP-32's walk |
> | EP-40 | A second start yields | records | Closed: the log's start of 12:50:10 that ended at 12:50:12 |
> | EP-40 | The first real five-hour hold | records | Closed: three hold rows at 57 % against 50 %, EP-27 chained at 17:04 after the reset |
> | EP-40 | The policy's rules in a chained run | records | Closed: four chained starts read "claude-opus-5-5 at max, the rules' choice" |
> | EP-40 | The merged session row in the Usage pane | owner | Closed: reads right |
> | EP-40 | The guard's two gaps, the parked extras | owner | v2 notes; the owner asked for a mark per chain: EP-41 (1) |
> | EP-27 | The first site export | records | Closed (OT-27): both files written on 2026-11-15 at 13:30; the site's inbox message is its own session's |
> | EP-27 | The window's daily write | records | Ran on the 13th and the 15th, skipped the 14th: EP-41 (3) |
> | EP-27 | A retention rule; the exports in the menus | owner | 14 days and each month's first, EP-41 (3); the commands serve, v2 notes |
> | EP-34 | The first real action | records | Closed (OT-28): Back up now on 2026-11-15 at 13:32, a `backup` row "backed up"; no hosting action yet, carried |
> | EP-34 | A scan before Make public | owner | EP-42 (1); D-24 addendum |
> | EP-28 | The pane in the real window, the fifth tab | live | Closed: as expected |
> | EP-28 | The rolling log's real lines | records | Closed: three day files hold the starts, the notices, a restart, the reconciliation and a quit |
> | EP-28 | A refusal by security software | — | None met; stays with risk 10 |
> | EP-28a | The first start at login | records, owner | It ran at 12:40:17 ("at login"); how it looked the owner did not notice: carried, with File > Start at login |
> | EP-28a | The first reconciliation | records | Closed: two rows closed at 12:40:52, each at its transcript's last write; `crw sessions --all` reads both completed |
> | EP-28a | The icons | live | Closed: the tile in all four places |
> | EP-28a | The review's two logged points | owner | Parked in the v2 notes; no leftover copy was in the folder today |
> | P3 | The push guard (risk 17) | owner | EP-42 (2), a doctor's line; the refusal itself carried |
> | P3 | The Keep pins | owner | Carried once more |
> | P3 | A delivery from one project into another | records | Closed: EP-27's letter delivered as the site's IN-1 on 2026-11-13 at 17:29 |
> | P3 | Five routes unmet (EP-20, EP-20c, EP-20d, EP-21a) | records, owner | None has happened; carried to EP-32's walk. The first real revert did (2026-11-12: seven files back, the work kept) |
> | P3 | The first migration's tables | owner | Closed: reads well, committed |
> | P3 | The `replan` skill's missing sentence | owner | Added in this brief |
> | P3 | `/roadmap-wrangler:formalise` from a session in the project, the skills' wording | — | Not raised: carried to EP-32's walk |
> | P3 | The pins, egui#8470 | records, owner | Kept; D-15 addendum |
> | Field | The tray's Pause queued while the window is hidden | code, owner | Found by the survey for P5: EP-30 runs the queue in the hidden pass |
>
> Forty-one points: fourteen settled from the records alone; most of the others took the
> owner's answers, nine of them from five live looks (the Diagnostics pane, the list, the
> graphs, the icons, a chat run), and two were never met or raised; nine carry something to
> EP-32's walk. Six fixes: five in EP-41 and EP-42, one in EP-30.
>
> **Item 2, the retro.** From the session transcripts (each session's first entry to its EP's
> record commit; active time leaves out gaps over five minutes), in minutes unless hours are
> written:
>
> | EP | Size | Launch | Active | Wall | What bit |
> |---|---|---|---|---|---|
> | EP-38 | M | headless | 39 | 39 | The chain's first run; one question answered in 35 s |
> | EP-39 | M | headless, chained | 35 | 35 | No question |
> | EP-24 | L → M | headless, chained | 1 h 01 | 1 h 01 | Three questions answered in the Decisions pane within minutes |
> | EP-24a | M | headless, chained | 42 | 42 | |
> | EP-24b | M | headless, chained | 44 | 44 | The day's cap of four ended the chain |
> | EP-25 | M | headless | 39 | 39 | IN-12 rolled in; IN-13 became EP-25a |
> | EP-25a | S | headless | 18 | 18 | No question, no review (S) |
> | EP-26 | M | headless, chained | 31 | 31 | The shortest M |
> | EP-40 | M | by hand, in VS Code | 47 | 2 h 20 | The one brief not started from the tool; three waits, eleven decisions |
> | EP-26a | M | headless, two runs | 1 h 09 | 1 h 09 | The first, chained, hit the five-hour limit after 19 min and was reverted; the second took 50 |
> | EP-27 | S | headless, chained | 22 | 22 | Chained once the five-hour hold lifted |
> | EP-34 | M | headless, chained | 50 | 5 h 16 | One question waited 4 h 20 |
> | EP-28 | M | headless, chained | 55 | 55 | |
> | EP-28a | M | headless, chained | 1 h 10 | 14 h 55 | One pair of questions waited 13 h 39 |
>
> In all 10 h 22 min of active work against 13 h planned (80 %; P3 89 %, P2 77 %, P1 62 %) and
> 30 h 04 min of wall time by the script's seconds (the rows above, each rounded to its minute,
> sum to 30 h 06), 12 h 05 min without the two long waits for the owner's answers (EP-34's
> through an evening, EP-28a's overnight). The two S took 18 and
> 22 minutes, the twelve M 31 to 70 (48 on average; 54 in P3). With each session's subagents'
> own transcripts counted, the total is 10 h 27 min (EP-28a alone moves, by five minutes). The
> ledger's run lengths agree with the transcripts once the build and the install after a record
> commit are allowed for (EP-39's row reads 38 minutes against 35). By the ledger's shares the
> fourteen headless runs took 36 points of the weekly limit across two weekly windows, and the
> nine on Fable 49 points of the Fable limit, which is what moved the owner's policy at EP-40 (S
> and M on Opus 5.5 at max, L on Fable at high); EP-40's session by hand took 5 and 11. P4 grew
> from 13 rows to 15 (EP-25a, EP-40), and no row split at pickup.
>
> **The phase's chains** (2026-11-11 … 2026-11-14; D-22 addendum): ten chained starts, all this
> project's; four ends (two at the day's cap of four, one for the failed run, one before this
> row); nine holds for the window's restart, all taken within about twenty seconds, and three
> rows for one five-hour hold; one restart asked by an install refused while its own run worked;
> no gate refusal for a tree or a task; no run capped; no chain into a re-plan.
>
> **Item 3, addenda.** Fourteen, dated 2026-11-15 and citing EP-29: D-4 (P4's pace, the cuts,
> the totals and the short count), D-8 (the daily export's day and its keep, the menus), D-9
> (the migration read), D-10 (the second walk, the cadence), D-13 (the first delivery between
> projects, the routes carried), D-14 (the phone pages' stack), D-15 (the pins), D-20 (the hook
> and the re-read seen), D-22 (the chains, the first hold, the mark per chain, the push guard),
> D-24 (the first action, the scan), D-25 (the list under the owner's hands), D-27 (the chat,
> the plugin's copy, the skill's sentence), D-28 (the phone view's shape), D-29 (the start, the
> reconciliation, the tray's Pause).
>
> **Item 4, P5.** Eight rows, all M: EP-41 and EP-42 (the walk's five fixes) open it; EP-30
> becomes EP-30, EP-30a and EP-30b; EP-31 becomes EP-31 and EP-31a; EP-32 stays. The section
> gained its ordering rationale and standing decisions; the three charters stand as written,
> each followed by a `**From EP-29:**` segment, and the five new rows have notes of their own.
> Counted from the tables, v1 is 68 briefs, 15 S · 49 M · 4 L ≈ 64.5 session-hours, 8 open once
> this row is checked; the roadmap's line of EP-40 had been one S short. Risks 3, 16 and 17
> gained a dated line and risk 18 (the phone view) is new. `CLAUDE.md` rule 5 gained the phone
> pages' test rule.
>
> **Item 5, the pins.** Nothing moved (D-15 addendum): the egui family, axum and askama are at
> their newest; the protocol crate stays at 2.1.316 with the standalone CLI, 2.1.325 being the
> crate's newest and the extension's; egui#8470 is open.
>
> **Item 6, the parked notes.** `v2-notes.md` grew by fourteen items marked *(EP-29)*, most
> with the brief they come from, across all nine sections, and five older items gained a dated
> mark. A reading subagent
> collected the candidates from the fourteen briefs; the session curated them.
>
> **The skill.** `plugins/roadmap-wrangler/skills/replan/SKILL.md`, step 3: settle from the
> records first, take to the owner only what needs their eyes, hands or judgement, carry by
> name what nothing has exercised.
>
> **The review (rule 6).** A subagent given the diff and the brief, not the session's reasoning,
> reported after about eleven minutes. It reproduced the tables' counts (69 rows, 15 S · 49 M ·
> 4 L after EP-0, 8 open; 63 briefs at HEAD where the header said 62), the retro's active sum,
> the walk's 41 rows and 14 records rows, and the fourteen addenda; it found the identifiers
> the notes cite where they say they are, the eight of point 4 aside, and read only EP-32's
> title as a re-plan by `autopilot::is_replan`. Its points, all fixed:
>
> 1. EP-30's bullet line stood twice, so `Roadmap::note` would have returned a one-line stub
>    for EP-30 and the tool's opener would have carried it.
> 2. Three addenda (D-8, D-13, D-15, each the last decision of its section) sat under the next
>    section's heading: moved above it.
> 3. Counts in this note: the v2 notes grew by fourteen items and five marks, not sixteen and
>    six; the wall column's rounded rows sum to 30 h 06 against the script's 30 h 04.
> 4. Notes that described the code wrongly: the doctor has no Headless group; the action's id
>    is `make-public`; `Board::start_queued` starts what `Board::gated` drafted and resolves
>    nothing itself; the sparklines sit beside the phases block, which already has a shared
>    minimum width (`PHASES_AT_LEAST`), not after the title; the daily counts are also written
>    at a one-project re-read; `Settings::is_path` lists the keys that are not paths; the
>    server tests' helpers send no headers and read no body; `futures-util` is in the lock
>    through more than axum.
> 5. The phone pages' test rule was cited to different decisions in three places and written
>    in no addendum: the D-29 addendum now states it, and "a browser on the machine itself" is
>    said to be the owner's look, never a session's.
> 6. A fresh doctor's report asked from the phone stood in the v2 notes as left out by the
>    owner while EP-31's note makes it that brief's first question: the v2 line is gone.
> 7. Smaller: one five-hour hold written three times, not three holds; EP-34's wait fell in an
>    evening, not overnight; the icons were a live look; risk 18 named the wrong brief and left
>    out answering; the D-28 addendum's pages lacked the sessions list and the start's page;
>    EP-14 belongs to EP-30a's depends-on; "the first three charters"; an unfinished reference
>    to a decisions table not yet written, and an out-of-scope line the skill's sentence had
>    overtaken.
>
> **Verification.** `cargo test --workspace`: 676 passed (189 in `app`, 427 in `core`), as at
> EP-28a; the plugin's tests hold the edited skill, which the build carries by `include_str!`.
> `crw roadmap .` reads 69 rows in six phases, P5 as 8, the next EP this one until its row is
> checked and EP-41 after, and no problem. Rule 3's word check printed nothing for the staged
> diff. Then the release build and the install (decision 38).
>
> **Owner decisions (`AskUserQuestion`, 2026-11-15).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The start at login as seen | Did not notice | Carried to EP-32's walk |
> | 2 | The icon in four places | The tile in all four | EP-28a's risk closed |
> | 3 | The Diagnostics pane (live) | As expected | EP-28's risks closed |
> | 4 | EP-28a's two logged points | Park both | v2 notes |
> | 5 | The list (live) | Works, with findings in their own words | Ragged sparklines to EP-41; the rest as built (decision 9) |
> | 6 | The Detail pane's graphs (live) | They read right | EP-25's and EP-26's risks closed |
> | 7 | The Usage row; the tray's Pause while hidden | Fix the tray's Pause | EP-30; D-29 addendum |
> | 8 | The chat | Fine, barely used | Two routes carried |
> | 9 | The list's fixes | Align the sparklines only | EP-41 (2); the floor and the double click kept; D-25 addendum |
> | 10 | The launch dialog's Install and restart | Never came up | Carried |
> | 11 | The exports in the menus | The CLI serves | v2 notes |
> | 12 | The daily export's rule | 14 days and each month's first, no day skipped | EP-41 (3); D-8 addendum |
> | 13 | A scan before Make public | Scan, then confirm | EP-42 (1); D-24 addendum |
> | 14 | The push guard | A Diagnostics check | EP-42 (2); risk 17 keeps the refusal |
> | 15 | The Keep pins | Carry once more | EP-32's note |
> | 16 | Five routes never met | Carry to the P5 walk | EP-32's note; D-13 addendum |
> | 17 | The first migration as read | Reads well, committed | D-9 addendum |
> | 18 | The `replan` skill's sentence | Add it now | The skill's step 3; D-27 addendum |
> | 19 | The prompt hook's context | Works as wanted | EP-39's risk closed |
> | 20 | The five-hour extras | Park the three; a mark that can be set higher for a chain (their own words) | v2 notes; EP-41 (1) |
> | 21 | A headless run's plugin copy (live) | It named the four | EP-22a's point closed |
> | 22 | The protocol crate's pin | Keep 2.1.316 | D-15 addendum; EP-32 looks again |
> | 23 | The sizes | They stay | D-4 addendum |
> | 24 | The cadence | Keep it | D-10 addendum |
> | 25 | EP-30's cut | Three M briefs | EP-30, EP-30a, EP-30b |
> | 26 | The pages' update layer | Plain forms, own small script | D-14 addendum |
> | 27 | P5's order | Enable last | The ordering rationale |
> | 28 | The five-hour mark's form | Per chain, in the Chain plan | EP-41 (1); D-22 addendum |
> | 29 | The phone's actions | The charter's list | D-28 addendum |
> | 30 | A revert from the phone | No, desktop only | D-28 addendum |
> | 31 | The session page's input | Answers only | D-28 addendum; EP-31 |
> | 32 | Extra views | Usage, Autopilot, Notifications, Diagnostics | EP-30b (4), EP-31 (3) and (4) |
> | 33 | The owner's login | Read at Enable, confirmed | EP-31a (2); `owner_login` |
> | 34 | The link | The URL and an ntfy push | No QR crate; D-28 addendum |
> | 35 | Pushes that open their page | Yes, while remote view is on | EP-31a (4) |
> | 36 | The walk's five fixes | Two M rows opening P5 | EP-41, EP-42 |
> | 37 | The drafted records, EP-31's cut among them | Signed off | As drafted, with the review's corrections |
> | 38 | The release build and the install | Build and install | Run after the two commits |
> | 39 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The model and effort** were read from the `model` and `effort` fields of this session's
>    own transcript entries rather than asked (EP-23 asked): the panel ran claude-fable-5-2 at
>    high where the launch suggested claude-opus-5-5 at max.
> 2. **The ledger and the transcripts** were read by two stdlib scripts in the scratchpad: the
>    `event` and `session` rows through a read-only SQLite connection (rule 13), and each
>    transcript's timestamps and Write or Edit paths (D-26). `crw history` was run over the
>    projects that are no data repository; no name is recorded here (rule 3).
> 3. **Two questions were wrong in a detail**, said in decision 9 and here: the list question
>    described a card's double click as opening the launch dialog, and the retro question said
>    both long waits were overnight (EP-34's was an evening) and that ten of the fourteen
>    briefs ran chained (ten chained starts, nine briefs completed in a chained run). The
>    cadence question's counts were a rough tally made before the walk's table (41 points, 14
>    from the records).
> 4. **EP-31's cut** into EP-31 and EP-31a was the session's proposal, made after the owner
>    added four views and the push links, and put to them at the sign-off rather than at the
>    shape round.
> 5. **Numbers.** EP-41 and EP-42 take the next free numbers and sit where they are needed;
>    the cut rows take letters; EP-30's Size cell reads `L → M (cut beforehand at EP-29)`.
> 6. **The phone's path alone proxied** (`--set-path=/m`), the form's own origin checked, and
>    the routes' names are the session's design within D-28, written into its addendum for the
>    briefs to verify against the installed client.
> 7. **OT-27's and OT-28's ticks**, the owner's in the window minutes before this session, are
>    committed with this brief, the tree's one change at its start.
> 8. **No owner task:** nothing here needs the owner's hands only. The first real Enable is
>    EP-31a's to write; what the walk carried is in EP-32's note (D-10 addendum).
> 9. **The README's pictures** were left for the close-out, as EP-23 planned.
>
> No message for another project. The push is the owner's; their own sessions get the skill's
> new sentence after it, with `claude plugin update roadmap-wrangler@clauderoadmapwrangler`.
> **Next: EP-41**, the fixes to the daily surfaces.
