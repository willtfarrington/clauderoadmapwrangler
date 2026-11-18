---
ep: 32
title: Re-plan P5 and v1 close-out, opening with the P5 walk; timings, README "State of the app", addenda, v2-notes compiled, v2 direction
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-32 — Re-plan P5 and v1 close-out, opening with the P5 walk

**Size:** M · **Depends on:** EP-41, EP-42, EP-30, EP-30a, EP-30b, EP-31, EP-31b, EP-31a (the eight
briefs of P5) · **Blocks:** v2 (its direction is set here)

## Context

P5 closed with EP-31a on 2026-11-17, and with it the last phase of v1 (D-1). Since the P4 re-plan
the window holds a chain at a five-hour mark of its own, aligns the expanded rows' sparklines and
writes the daily export when the day turns, keeping fourteen days and each month's first (EP-41);
it reads a project's whole history against a word list before Make public, and the doctor says an
autopilot run carries the push deny rule (EP-42). Its loopback server answers the phone's pages
under `/m/`: the list with the usage strip and the autopilot block, a project's page that ticks
and composes, the decisions, a start page, Stop, Pause and Resume, the sessions, a run's own page
with its transcript as it streams, the notifications and the doctor's report with Check again,
one event stream and one small script of the tool's own, every action through Serve logged and
printed by `crw remote` (EP-30, EP-30a, EP-30b, EP-31). The Autopilot section shows and sets the
overall daily cap (EP-31b). Enable remote view mounts the phone's path alone on the tailnet with
Tailscale Serve after a confirmation, writes the owner's login, gives the link, makes pushes open
their page, and Disable takes the mount off (EP-31a).

This brief closes P5 (D-4) and v1. It opens with the **walk** (D-10 addendum, 2026-11-08), the
third and last of v1: the residual-risk paragraphs of P5's eight briefs and the points the P4 walk
carried to this one by name, settled from the records where the records can say and taken with
the owner where only their eyes, hands or judgement can. Then the retro, for P5 and for every EP of
v1; the addenda; the README's "State of the app" and its pictures; the pins; `v2-notes.md`
compiled into themes; and the v2 direction. The plugin's `replan` skill is the method.

**Session start (2026-11-17, 18:03 local time).** Started from the window (Start, Visual, Open in
VS Code; the rolling log's line of 18:03:29); the prompt is the tool's opener, and `crw sessions
clauderoadmapwrangler` lists the row (`67843292`, visual, EP-32, open). The launch suggested
claude-opus-5-5 at max; the panel runs claude-opus-5-5 at max, read from the `model` and `effort`
fields of this session's own transcript entries (no text read). No inbox message is open. One owner
task is: OT-29, the first real Enable remote view, ungated. The tree is clean at `80db114`, and
`main` is seventeen commits ahead of `origin/main`. The field check read, since EP-31a's record
commit (14:00):

- `crash.log`: the one report of 2026-11-03, nothing newer.
- `crw notifications`: nothing after 13:55, when EP-31a's closing question waited in VS Code.
- `crw sessions`: this row; EP-31a's visual row still open (its tab), and a manual row of
  2026-11-15 13:23 still open, a tab the chain log has counted as idle since.
- `crw autopilot`: not paused; on for this project and five others, all in mode `auto`, briefs up
  to M; the last line is the chain's end after EP-31b at 11:58, "EP-31a is left out of your plan".
- `crw remote`: none yet. **OT-29 has not run**: no Enable, no request through Serve, so no
  arrival line for the walk to read.
- The rolling log: at 14:01 EP-31a's install restarted the window into its build, no run
  working; at 14:08 a full read; at 18:03 this launch. At midnight the day turned and the export
  was written (EP-41's day's turn, seen in the real window).

Nothing odd to triage. OT-29's absence is the one gap: the walk's phone points have no record yet,
so the walk's first round offers its steps (the `replan` skill's step 2 asks for steps a later
session can rely on when the record lacks a first run; the owner may take them live).

## In scope

1. **The walk.** The steps compiled from the residual-risk paragraphs of EP-41, EP-42, EP-30,
   EP-30a, EP-30b, EP-31, EP-31b and EP-31a, the review points those briefs logged for this walk,
   and the points the P4 walk carried here by name (this row's brief note). Each is settled from
   the records where they can say (the ledger's `session` and `event` rows through a read-only
   connection, `crw autopilot`, `crw mail`, `crw notifications`, `crw remote`, `crw doctor`, `crw
   signals`, `crw history`, the rolling log, the export folder's names), and put to the owner
   where only they can: what they saw, a live step in the window or on the phone, or a judgement a
   note left to the walk. OT-29's steps are offered live in the first round. Each finding becomes
   a row, an addendum or a v2 note at once. This is v1's last walk, so a point nothing has
   exercised is carried into v2's plan by name, or closed as unexercised on the owner's word.
2. **The retro.** P5's eight briefs: the planned size against the active and the wall time from
   the session transcripts (the first entry, or the previous EP's record commit, to the EP's record
   commit; gaps over five minutes left out; the subagents' own transcripts counted; timestamps and
   file paths only), the ledger's rows set beside them, and the phase's chains. Then **v1 as a
   whole**: every EP's time, from the retro tables of EP-8, EP-16, EP-23, EP-29 and this one, the
   four earlier re-plans' own sessions timed from their transcripts; sums per phase and for v1
   against the plan.
3. **Addenda** to the decisions P5 changed or the walk settles, dated 2026-11-17 and citing EP-32;
   nothing rewritten.
4. **The README.** Its long "Status" paragraph rewritten as "State of the app" for v1 as closed,
   by area rather than by brief; its two pictures retaken from a neutral copy of the fixtures
   (EP-36's recipe, this row's note) after P4 and P5, so the Chat tab and the graphs show; whether
   a phone page is pictured too is put to the owner (a screenshot run binds no port).
5. **The pins** looked at again (D-15): the egui family, `egui_commonmark`, `egui_plot`, axum,
   askama, `futures-util`, the protocol crate against the standalone CLI and the extension's
   bundled one; egui#8470.
6. **`v2-notes.md`** grown with what P5 named and did not build, each with its source; then
   compiled into themes (launch modes, analytics, chat, phone, hosting, conventions, and the
   sections it already has), the items v1 built marked so, and a **v2 direction** section with
   the candidates the owner ranks (the row's first four: `--bg` background sessions with Attach,
   worktree-per-session, usage over time, free chat from the phone).
7. **How v2 continues**: what the roadmap holds after P5 (a v2 planning row, a drafted phase, or
   nothing until the owner asks), the owner's choice.
8. **The records**: the roadmap's header paragraph (v1 closed), the P5 table (this row linked and
   checked), the P5 section's dated sentences, the risks list (dated lines; each risk closed or
   carried into v2), `CLAUDE.md` where a rule's wording follows from a decision here, this brief's
   completion note; a review of the diff by a subagent given the diff alone (rule 6, an M); rule
   3's word check before each commit.

## Out of scope

- Code. A finding that needs code becomes a row where the owner places it; this brief changes
  records and pictures, so the release build and the install at the close are put to the owner
  (EP-23 skipped both for a records-only re-plan).
- The v2 plan itself (its decisions, phases and briefs): a planning interview of its own if the
  owner chooses one (item 7).
- Moving the egui family or the protocol crate, unless the owner asks at the pins question (D-15).
- Any write into another project, the owner's registry or their user settings by the session
  (rules 4, 7 and 13): a live step of the walk is the owner's own hand in the window or on the
  phone.
- Running `tailscale serve` or driving a browser (D-29): OT-29's steps are the owner's.
- Reading a transcript's text (D-3, D-26): the timings read timestamps and file paths, and this
  session's own model and effort fields.
- The push: the owner's.

## Verification / acceptance

- Every segment of this row's brief note has an answer: a row, an addendum, a v2 note, or a line
  in the walk's table saying what the records showed.
- The retro's tables sum to the totals stated; the roadmap's totals (briefs, S, M and L,
  session-hours, open rows) are recounted from the tables after the edits and checked by the
  review subagent against the diff.
- `crw roadmap .` reads the roadmap after the edits with no new problem and the next EP as the
  close-out leaves it; no new row's title reads as a re-plan to the gate unless it is one.
- `cargo test` passes as at EP-31a (no code changes).
- The README's pictures show the fake portfolio at a neutral path and nothing of the machine,
  each looked at before it is staged.
- Rule 3's word check prints nothing for the staged diff; commits in the house pattern, not pushed.

**Residual risk.** This brief changed records and pictures, so its risk is in what they promise.
(1) **EP-43's cause is unknown.** Its note names candidates read from the code, none confirmed;
the measurement it adds may show a cause the note does not name, and the brief is re-scoped at
pickup if so. (2) **The carried routes rest on the field check and v2's first walk.** The rare
ones may stay unmet well into v2; a break there would show first in the crash log, a
notification or the ledger's rows the field check reads. (3) **Remote view stays on** after the
walk (Disable, then Enable again, the owner's choice): the guards held for the owner's own
requests, and a phone lost while on the tailnet reaches the pages until Disable or its removal
from the tailnet (risk 18). (4) **The README's "State of the app" was checked against the old
paragraph by reading**: each topic it carried moved into a detail section; a sentence lost would
be a slip in the text, not in the tool. (5) **The v1 table mixes measures**: P1's wall figure is
the session time EP-8 recorded, the re-plans' own times are this session's readings of their
transcripts, and the subagents' transcripts were counted for P4 and P5 alone. (6) **The phone
picture is the owner's capture** of a page rendered from the fixtures; a later change to the
pages leaves it behind until a re-plan retakes it with the desktop pictures.

> **Completion note (2026-11-17; claude-opus-5-5/max; about 1 h 35 min of wall time,
> twenty-one questions in eight rounds).** A visual session started from the window (ledger row
> `67843292`, from 18:03), following the plugin's `replan` skill. No subagent read for it; one
> reviewed the diff (below).
>
> **Item 1, the walk.** The residual-risk paragraphs of P5's eight briefs, the review points they
> logged for this walk, the routes the P4 walk carried here by name and the field check's
> findings gave the points below. "Records" means settled from the ledger, the rolling log and
> the tool's own reads; "live" a step the owner took during the session (OT-29 on the phone and
> a look at the window); "owner" the owner's answer; "OT-29" a phone route its steps could not
> reach; "v2" a logged point parked.
>
> | From | Point | How | Outcome |
> |---|---|---|---|
> | EP-41 | A real chain at its own mark | records | Closed: EP-30b chained at 54 % of the session window under the chain's 80 %, where the setting's 50 % would have held it; after it the chain held at 91 and 92 % until the 23:50 reset and chained EP-31 at 23:55 |
> | EP-41 | The day's turn in the real window | records | Closed: "the day turned" and the export at midnight on 2026-11-16 and 2026-11-17 |
> | EP-41 | The keep's first removal on the real folder | records | Not due: the folders of the 13th, 15th, 16th and 17th stand, the first removal falls on 2026-11-29; carried |
> | EP-41 | The pickers under the mouse | records | Closed: three lines of 2026-11-16, 18:56 to 18:57, "your change" and "your choice in its plan" |
> | EP-41 | The sparklines on real content | live | Closed: one left edge, this project's step at the P4 re-plan readable |
> | EP-41 | The review's logged point (the start's mark line from the open dialog's project) | v2 | v2 §1 |
> | EP-42 | A real Make public | records | Not met (no hosting row in the ledger); carried |
> | EP-42 | The real list | records | As built (read by its count); carried with Make public |
> | EP-42 | The push guard in the real window | records | Closed: `crw doctor` says "push guard ok" today; the refusal stays with risk 17 (decision 4) |
> | EP-42 | The review's three logged points | v2 | v2 §5 |
> | EP-30 | A request as Serve really passes it | live | Closed: the arrival line, `Host` this machine's tailnet name with a port, the login and the forwarding headers with it |
> | EP-30 | A real browser | live | Closed: the list on the owner's phone through Serve |
> | EP-30 | The hidden pass in the real window | OT-29 | Not met: the window stayed shown through OT-29 (the tray's Pause, from its menu, was taken at once); carried |
> | EP-30a | A real browser's forms | live | Closed: Resume and Check again posted from the phone |
> | EP-30a | The orders in the real window's pass | live | Resume taken at once; the first Check again taken 11 s after its post and dropped as designed: EP-43 (decision 13) |
> | EP-30a | A real session's answer from the phone | OT-29 | Not met (no run waited); carried |
> | EP-30a | The ten seconds | live | Met for real: the page let the form go and said to send it again, and the window dropped it unrun, as built |
> | EP-30b | The script in a real browser | live | Closed: the tray's Pause showed on the phone's open page within seconds, no reload |
> | EP-30b | A real start from the phone | OT-29 | Not met; carried |
> | EP-30b | The stream through Serve | live | Closed with the script's step |
> | EP-30b | IN-14's restart | records | Not made again on purpose; carried |
> | EP-30b | Two refusals tested by reading alone | code | As built |
> | EP-31 | The script's append mode on a real phone | OT-29 | Not met (no headless run streamed during OT-29); carried |
> | EP-31 | A real run's transcript on its page | OT-29 | Not met; carried |
> | EP-31 | The phone's Check again in the real window | live | Ran at the second try (the first is EP-43's) |
> | EP-31 | The window's reads for the phone | records | No sign of a slow pass beside the eleven seconds, whose candidates EP-43's note names, these reads among them |
> | EP-31 | Stale pages after a restart | OT-29 | Not met; carried |
> | EP-31b | The picker on the owner's registry | live | The day line read "1 of [8]"; the picker itself not moved |
> | EP-31b | The window's count beside the gate's | live | Closed: one chained start today, as the gate counts it |
> | EP-31b | The roll-in in the real window | records | In force since the restart of 12:27 into EP-31b's build; no chain since to show it |
> | EP-31b | The phone's line in a real browser | live | Closed: the autopilot block's day line on the phone |
> | EP-31a | The real Serve | live | Closed: Enable, Disable's `off` and Enable again ran; Serve's other three HTTPS entries stood, Funnel off |
> | EP-31a | How `Host` arrives through Serve | live | Closed: this machine's tailnet name with its port |
> | EP-31a | A push's link on the phone | live | Closed: the test push's tap opened the list, as did the link's push |
> | EP-31a | The narrowed guard on the owner's tailnet | live | Closed: the owner's requests passed with the registry's name |
> | EP-31a | The local rule after the arrival | owner | Both readings kept (decision 12) |
> | EP-31a | The review's logged points | v2 | v2 §4 |
> | P4 | The start at login as it looks; File > Start at login | records | One start at login (2026-11-15), none since; the File menu's box ticked (live); the look carried |
> | P4 | The launch dialog's Install and restart with a newer build | records | Not met: the restarts since came from File, the tray and the gate (the 12:27 notice carried nothing); carried |
> | P4 | A chat's question in its tile, a deferred one resumed | records | Not met (13 chat runs, none asked); carried |
> | P4 | The first hosting action from the window | records | Not met (no hosting row; one backup, 2026-11-15); carried |
> | P4 | The Keep pins on the real registry | records | Not met (no kept rule); carried |
> | P4 | A delivery while a session idles in the receiver | records | Not met (no delivery since 2026-11-13); carried |
> | P4 | A chain meeting a changed inbox with no mail; a restart with an urgent letter in flight | records | Not met; carried |
> | P4 | The first reworded block | records | Not met; carried |
> | P4 | A revert that redoes a formalise | records | Not met (the one revert, 2026-11-13, had none to redo); carried |
> | P4 | `/roadmap-wrangler:formalise` from a project's session; the skills' wording | records | Not met (no formalise row since 2026-11-11); the `replan` skill ran this re-plan as written; carried |
> | P4 | The push guard's refusal (risk 17) | owner | Kept open into v2 (decision 4) |
> | P4 | The sparklines after a phase crossing | live | Closed: this project's step at the P4 re-plan read |
> | P4 | The pins | owner | Kept (decision 11) |
> | P4 | egui#8470 | records | Open; the upstream fix of 2026-11-11 is in no release yet |
> | P4 | A phone page in the README | owner | Pictured by the owner from the fake portfolio (decision 16) |
> | Field | The five-hour hold's words ("until the pause lifts" beside the chain's mark) | records | v2 §1 |
> | Field | The re-read's words when the registry does not read ("the registry does not list it") | records | v2 §1 |
> | Field | EP-31a's visual row and a manual row of 2026-11-15 still open | records | As built: their tabs are open; the gate counts the idle one as idle |
>
> Fifty-five points: the records answered 24 (ten closed, fourteen routes found still unmet),
> live steps closed 17 and the owner's answers 4; six of the phone's routes OT-29 could not
> reach, three logged review points were parked and one point the code answers. One fix: EP-43.
> Twenty points are carried into v2's first walk; EP-44's note lists their routes by the brief
> each comes from.
>
> **Item 2, the retro.** From the session transcripts (each session's first entry to its EP's
> record commit; active time leaves out gaps over five minutes; timestamps and file paths only),
> in minutes unless hours are written:
>
> | EP | Size | Launch | Active | With subagents | Wall | What bit |
> |---|---|---|---|---|---|---|
> | EP-41 | M | headless, by the owner | 43 | 43 | 43 | No question; the review's seven points |
> | EP-42 | M | headless, chained | 40 | 48 | 1 h 54 | One question waited 1 h 18; the gate after it could not read the registry (IN-14) |
> | EP-30 | L → M | headless, by the owner | 53 | 59 | 59 | One question answered in 8 minutes (IN-14's triage) |
> | EP-30a | M | headless, chained | 51 | 51 | 51 | No question |
> | EP-30b | M | headless, chained | 1 h 04 | 1 h 04 | 1 h 04 | No question; a five-hour hold of two hours after it |
> | EP-31 | M | headless, chained | 44 | 57 | 11 h 35 | Two questions in one request, answered overnight |
> | EP-31b | S | headless, chained | 23 | 23 | 23 | No question; the field check's roll-in |
> | EP-31a | M | visual | 1 h 09 | 1 h 25 | 1 h 30 | Two closing questions; a subagent built the core module |
>
> In all 6 h 25 min of active work against 7 h 30 min planned (86 %), 7 h 09 min with the
> subagents' own transcripts, and 18 h 59 min of wall time by the script's seconds (the rows,
> each rounded to its minute, sum to 6 h 27, 7 h 10 and 18 h 59), 8 h 20 min without EP-31's overnight
> wait. The S took 23 minutes and the seven M 40 to 69 (52 on average). The ledger's run lengths
> agree with the transcripts once the install after a record commit is allowed for. By the
> ledger's shares the eight runs took 18 points of the weekly limit and none of Fable's, all on
> Opus 5.5 at max by the owner's rules; one M took 28 to 37 points of a five-hour window.
>
> **v1 by phase**, from the retro tables of EP-8, EP-16, EP-23 and EP-29 and this one, each
> phase's briefs without its re-plan, and the re-plans timed here from their transcripts:
>
> | Phase | Briefs | Planned | Active | Wall | Its re-plan, active and wall |
> |---|---|---|---|---|---|
> | P0 | EP-0 | 2 h | 1 h 20 | 2 h 14 | — |
> | P1 | 8 | 9 h | 5 h 37 | 6 h 17 | EP-8: 34 min, 34 min |
> | P2 | 13 | 13 h 30 | 10 h 27 | 15 h 28 | EP-16: 34 min, 34 min |
> | P3 | 21 | 18 h | 15 h 58 | 31 h 27 | EP-23: 44 min (59 with subagents), 2 h 19 |
> | P4 | 14 | 13 h | 10 h 22 | 30 h 04 | EP-29: 48 min (57), 2 h 37 |
> | P5 | 8 | 7 h 30 | 6 h 25 | 18 h 59 | EP-32: this note's head |
> | v1 | 64 | 61 h | 48 h 49 | 102 h 15 | the four before EP-32: 2 h 40 against 3 h, 6 h 04 |
>
> Active work ran at 80 % of the plan over v1 (P1 62 %, P2 77 %, P3 89 %, P4 80 %, P5 86 %); an
> M about 50 minutes, an S 20 to 35. v1 grew from 32 briefs to 69: splits at pickup and cuts made
> beforehand, the walks' fixes and the owner's requests.
>
> **P5's chains** (2026-11-15 … 2026-11-17; D-22 addendum): five chained starts, all this
> project's; four restarts into a new build at the gate's moment, each carrying the held chain;
> one five-hour hold of two hours at the chain's own mark of 80 %; one chain ended silently by a
> registry the running build could not read (EP-42's, IN-14's case, fixed by EP-30b); one end at
> a row the owner left out of the plan; one gate restart held by a chat's ended run (EP-31b's
> roll-in). No run was capped, no chain ran into a re-plan, and no run pushed.
>
> **Item 3, addenda.** Ten, dated 2026-11-17 and citing EP-32: D-1 (v1 closed, v2's direction),
> D-4 (P5's and v1's pace, the sizes kept), D-10 (the third walk, the routes carried, the
> cadence), D-13 (the inbox's routes unmet), D-14 (the pages' stack on a real phone), D-15 (the
> pins), D-21 (the pushes' links), D-22 (P5's chains, risk 17), D-28 (the first real use, the
> arrival, the form dropped), D-30 (v2 planned in an interview).
>
> **Item 4, the README.** The long Status paragraph is now "State of the app": v1's close and
> v2's start, then what the tool does by area, the detail left to the sections below; the
> details only that paragraph carried (the revert, the chat, history and analytics, the
> exports, the hosting and backup actions, the diagnostics and the crash log, the phone's
> pages) moved into those sections, none dropped. The two pictures were retaken from a neutral
> copy of the fixtures with the Chat pane and the graphs in view (the light one) and the
> Decisions pane, the expanded row's sparkline and the Detail pane's three graphs (the dark one),
> and the phone view's section gained the phone's list page (`docs/phone-list.png`), captured by
> the owner from a page rendered from the fixtures.
>
> **Item 5, the pins.** Nothing moved (D-15 addendum): every pin is at its newest but the
> protocol crate, which stays with the standalone CLI at 2.1.316; egui#8470 is open, its upstream
> fix of 2026-11-11 in no release yet.
>
> **Item 6, the parked notes.** `v2-notes.md` compiled in place: the v2 direction first, each of
> the nine themes opening with what v1 built and what stays parked, eight items marked *(EP-32)*
> from P5's briefs, the walk and the field check, and the four leads marked so; nothing taken
> out.
>
> **Item 7, P6.** v2's first phase: EP-43 (S, the phone's form wait) and EP-44 (L, the v2
> planning interview), with their ordering rationale, standing decisions and brief notes; the
> sizes paragraph and risks 3, 9, 16, 17 and 18 gained a dated line. Counted from the tables, v1
> is 69 briefs, 16 S · 49 M · 4 L ≈ 65 session-hours, all done once this row is checked; with P6,
> the roadmap holds 71 briefs, 17 S · 49 M · 5 L ≈ 67.5 session-hours, 2 of them open.
>
> **The review (rule 6).** A subagent given the diff and the files, not the session's reasoning,
> reported after about seventeen minutes. It reproduced the tables' counts (rows per phase 9, 14,
> 22, 15, 9 and 2; v1 69 briefs, 16 S · 49 M · 4 L; 71 with P6), the retro's sums and the v1
> table against the earlier notes, the walk's 55 rows and the D-10 addendum's counts, and the ten
> addenda's places, the log's diff only adding; `autopilot::is_replan` reads neither new row as a
> re-plan, and every identifier of EP-43's note but one matched the code. Its points, all taken:
>
> 1. Sums the note left out or explained short: the with-subagents rows sum to 7 h 10, the
>    re-plans' wall time to 6 h 04, and EP-29's wall time differs from its note's.
> 2. "The records settled 24": fourteen of those found a route still unmet; now "answered", ten
>    closed and fourteen carried, in the note and the D-10 addendum.
> 3. "Seventeen routes" matched no count: the table carried 19 points (20 after point 4), and
>    EP-44's note groups their routes by brief; the number is gone.
> 4. The hidden pass read as closed, though the window stayed shown through OT-29: now not met,
>    carried, and in EP-44's list.
> 5. The D-30 addendum miscounted the parked items that now lead v2: three of the four.
> 6. P6's standing decisions gave D-15 a route it does not allow ("a brief of its own").
> 7. egui#8470 "unchanged since 2026-09-10" left out EP-29's finding, an upstream fix of
>    2026-11-11 in no release.
> 8. EP-44's note cited the D-4 addendum for an S's range only this note gives; EP-43's note
>    named `Ledger::open`, where the phone's reads use `Ledger::open_read_only`.
> 9. `v2-notes.md` §1 listed the five-hour extras as parked though they lead v2; this project's
>    chaining cited EP-37 alone; a double blank line.
> 10. The README's State section said notifications of three figures where only the weekly and
>     Fable limits notify, "eight actions" against the seven the phone section names (now listed),
>     and "each kept in the ledger" where a session gets a row only at its first work.
> 11. Facts of the old Status paragraph found nowhere in the new README: CI, the docked panes, a
>     run's whole process tree ended by Stop and Kill, each session's weekly share in the Sessions
>     pane and `crw sessions`; each restored in its section (the phases' dates stay in the
>     roadmap).
> 12. The roadmap's header gave v1's totals but not the roadmap's with P6.
> 13. Nits: "minutes before" for under a minute; the `replan` skill's step 2 asks for steps, not
>     a live run; "Disable's form" read like a phone form.
>
> **Verification.** `cargo test --workspace`: 253 in `app` and 476 in `core`, the CLI's and the
> rest, all passing, as at EP-31a (no code changed). `crw roadmap .` reads 72 rows (EP-0 among
> them) in seven phases, P6 as two, the next EP this one until its row is checked and EP-43
> after, and no problem. The three pictures were looked at before staging: the fake portfolio, a
> neutral path, nothing of the machine. Rule 3's word check printed nothing for the staged diff,
> for the new brief and for the added lines joined. No release build and no install (decision
> 21).
>
> **Owner decisions (`AskUserQuestion`, 2026-11-17).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | OT-29 in the session | Live now, step by step | Its six steps taken in the walk; OT-29 ticked (decision 20) |
> | 2 | A look at the window (live) | All as described | EP-41's sparklines, EP-26a's phase crossing, EP-31b's day line and File > Start at login closed |
> | 3 | The routes v1 never met | Carry them into v2's first walk (not the recommended closing with a watch list) | EP-44's note; D-10 and D-13 addenda |
> | 4 | Risk 17 | Keep it open into v2 | Risk 17's dated line; D-22 addendum |
> | 5 | OT-29, Enable and the link (live) | Enabled; the list shows on the phone | The arrival line read; D-28 addendum |
> | 6 | OT-29, the phone's actions (live) | Pause and Resume worked; Check again timed out at ten seconds once, then ran | EP-43 |
> | 7 | OT-29, a push's link (live) | The tap opened the list | D-21 addendum |
> | 8 | OT-29, Disable | Disable, then Enable again | Remote view on; risk 18's line |
> | 9 | Where the window was at the dropped form | Shown, perhaps behind another application's window | EP-43's note |
> | 10 | The sizes | Keep them | D-4 addendum |
> | 11 | The pins | Keep every pin | D-15 addendum |
> | 12 | The guard's loopback reading | Keep both readings, record it | D-28 addendum; v2 §4 |
> | 13 | The form's wait | v2's first fix, an S row | EP-43 |
> | 14 | After EP-32 | P6: the fix, then a v2 planning interview | EP-43, EP-44; D-30 addendum |
> | 15 | v2's lead | All four candidates | D-1 addendum; `v2-notes.md`'s direction |
> | 16 | A phone page in the README | Yes, now | `docs/phone-list.png` in the phone view's section |
> | 17 | The phone picture's capture | Captured at a phone's size in the browser's device toolbar | Read, then put into `docs/` |
> | 18 | The README's State of the app and its pictures | Signed off as drafted | Committed with the review's fixes |
> | 19 | The drafted records | Signed off as drafted | Committed with the review's fixes |
> | 20 | OT-29's tick | Tick it with this brief's commit | Its box `[x]` in the first commit |
> | 21 | The release build and the install | Skip both | The installed build stays EP-31a's |
> | 22 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The model and effort** were read from the `model` and `effort` fields of this session's
>    own transcript entries rather than asked.
> 2. **The records** were read by stdlib scripts in the scratchpad: the ledger's `event`,
>    `session` and `usage_sample` rows through a read-only connection (rule 13), each transcript's
>    timestamps and Write or Edit paths (D-26). Serve's configuration and the doctor's remote view
>    line were read with the machine's name, the login and the link masked; nothing of them is
>    recorded here (rule 3).
> 3. **The pictures' setups** added the canned usage, analytics and Chat pane (the light one) and
>    the analytics of every project and the Decisions pane (the dark one) to EP-36's two setups, so
>    that the panes P4 filled show; a fresh home per picture, the neutral copy removed after.
> 4. **The phone page made standalone** by a small script in the scratchpad (the stylesheet
>    inlined, the script dropped), opened and captured by the owner, and read before it went into
>    `docs/`; the README shows it 320 pixels wide.
> 5. **The README's details moved, not rewritten**: the paragraphs only the old Status carried
>    went into the detail sections as they stood, a clause on the remote view's line added to the
>    diagnostics' list.
> 6. **The carried list** in EP-44's note takes, beside the routes the owner's answer named, the
>    phone routes OT-29 could not reach and the export folder's first removal, which are of the
>    same kind.
> 7. **The re-plans' own times** for the v1 table were read from their transcripts by the same
>    script (EP-8 and EP-16 read 34 minutes each, against their notes' 31 and about 33 to the
>    last answer; EP-29 2 h 37 of wall time against its note's about 2 h 50, which ran on past
>    its record commit to the install).
> 8. **Figures in questions that were off**: the sizes question said the phases' briefs were 63
>    and P5's wall time 18 h 58 min, where the tables give 64 and 18 h 59 (1,138.5 minutes); the
>    pins question said egui#8470 was unchanged since September, where an upstream fix was
>    committed on 2026-11-11, in no release (EP-29's addendum); the records' sign-off spoke of
>    seventeen carried routes, where the walk carries twenty points, the hidden pass among them
>    once the review showed the window had stayed shown.
> 9. **No owner task**: OT-29 ran in the session, and nothing else needs the owner's hands only.
>    No message for another project.
>
> The push is the owner's. **Next: EP-43**, the phone's form wait, the owner's to start (the
> autopilot never chains after a visual session); then EP-44, the v2 planning interview.
