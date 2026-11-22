---
ep: 45
title: Re-cut P6 at the owner's request: the window's daily surfaces, the conventions review, the portfolio's states and sizes, the manuals; the v2 planning interview moves to P7
size: M
model: claude-fable-5-2
effort: not recorded (a session started by hand)
status: done
---

# EP-45 — Re-cut P6 at the owner's request

**Size:** M · **Depends on:** EP-32 · **Blocks:** every row of the new P6

## Context

P6 was planned at EP-32 (2026-11-17) as two rows, both still open: EP-43 (S, the phone's form
wait) and EP-44 (L, the v2 planning interview). On 2026-11-22, before either ran, the owner asked
for the phase to be cut again: the interview moves to P7 or later, and P6 takes ten requests that
came of five days' use of the window across the portfolio.

The window's surfaces:

1. The Projects pane loses its Brief column (the next brief's one-liner), which the Detail pane
   already shows.
2. A project is read again by itself when a roadmap session of it is complete, visual or
   headless, so the next brief is offered without the toolbar's Refresh, which asks git and
   GitHub about every project. The owner nearly started one brief twice.
3. A darker base theme with more contrast, after the dark contrast theme the owner uses in VS
   Code, the accent colours keeping their meanings.
4. The Usage pane loses "The last seven days" spark line; the three limit meters and the
   surfaces stay.
5. The owner tasks of every project in one place, and the inbox messages of every project in
   one place, each perhaps a pane of its own, the Detail pane ending at Status; and a project
   without an owner-tasks file no longer counts its risks, parked blocks and other older forms
   as owner tasks in the list.
6. The Autopilot section leaves the Sessions pane for a pane of its own, its log collapsible.
7. The Chat pane: a message sent after the window has been open a while is often refused with
   words about Claude not having answered yet; and Enter alone should send.

The owner's process, across projects:

8. One or more review sessions over every `CLAUDE.md`, `SKILL.md` and other configuration file
   that spans projects, setting out the conventions and preferences they document, for the owner
   to comment on, edit, remove, add to and consolidate.
9. After that, sessions that take the owner through each paused, maintenance and done project:
   made active in the house style, abandoned, or its place in its state understood. With it the
   sizes S, M and L measured again from every project's session history, and a rule that a brief
   is M or smaller, S preferred, an L only where one whole subtask cannot be cut.
10. A PDF manual for each project that works, reachable from the window: a reference and, while
    full use is still wanting, a walkthrough; brought up to date at each phase's close, with a
    separate PDF for what changed in between.

The owner asked that the new P6 guide them through as much of this as it can with few
interruptions, that EP-43 may be rolled in, and that this session stay small: the early part of
P6 may itself be planning sessions, each at or under the new size goal.

**Session start (2026-11-22, 16:04 local time).** Started by hand: the owner typed the request
above into a Claude Code tab of this project's window with `/roadmap-wrangler:replan`, so the
prompt is not the tool's opener and no row of the roadmap named this session (`crw sessions
clauderoadmapwrangler` lists it as `f4a8824d`, manual). **Field check:** `crash.log` was last
written on 2026-11-03, long before EP-32's record commit, so no panic since;
`crw notifications` holds backup-stale badges of today and nothing odd; `crw autopilot` shows no
chain of this project since EP-32 (the chain log's newest line is another project's, 2026-11-18);
`crw sessions` shows one manual session of today before this one. **Inbox:** IN-15, one open
message, about the `replan` skill's retro rule (see the triage below).

## In scope

- The brief, then the method of the plugin's `replan` skill as far as it applies to a phase no
  brief of which has run: no walk (the points EP-32 carried into v2's first walk stay carried, by
  name, in EP-44's note) and no retro (nothing to time).
- IN-15's triage.
- The interview: the new P6's shape, where the cross-project sessions run and how the window
  guides the owner through them, the order, then the sign-off of the drafted rows and the
  close-out.
- The re-cut: P6's rows with sizes and depends-on, none over M; the early process rows as
  planning briefs whose notes later sessions grow; EP-44 moved to a new P7; the totals recounted;
  a dated sentence under the ordering rationale; pickup notes from a survey of the code.
- Addenda to the decisions this changes (D-4 for the size goal, D-30 for the interview's move,
  others as the interview settles them).
- `v2-notes.md` grown with what the interview names and P6 does not take.

## Out of scope

- Any change to the code, to another project's files, or to the owner's settings.
- The conventions review itself, the measurement of the sizes and the manuals: P6's rows.
- A walk or a retro: P6 has not run.

## Verification / acceptance

- `crw roadmap` reads the new tables: the next EP is P6's first row, the counts match the
  recount in the README's opening.
- Each of the owner's ten requests is in a named row's brief note or in `v2-notes.md`.
- No tracked file names a managed project (rule 3); the word check prints nothing on the staged
  diff.

**Residual risk.** The rows are cut from a reading of the code by a survey, not from a trial: a
row may prove larger at pickup, and each brief note says to split then rather than run long. The
sizes rest on v1's pace (an S 20 to 35 minutes, an M about 50) until the measurement row has
run.

> **Completion note (2026-11-22; claude-fable-5-2, effort not recorded; about 1 h of wall time,
> eleven questions in three rounds).** A session started by hand with the plugin's `replan`
> skill. No walk and no retro, P6 not having run; the routes carried into v2's first walk stay
> in EP-44's note. One subagent surveyed the code for the pickup notes (file, type and function
> names for each of the window's seven requests); no code changed, so no build, no review of a
> diff and no install.
>
> **The owner's decisions.**
>
> 1. *Where the conventions review runs:* in the owner's environment repository, as that
>    project's own sessions; this roadmap keeps one row for what this repository publishes
>    (EP-54). Not picked: here into `private/`; a free session with no roadmap.
> 2. *How the sweep of the projects that are not active is guided:* owner tasks, one per
>    project, written by EP-55 with a message into each project. Not picked: a sweep feature in
>    the window; inbox messages alone.
> 3. *The order:* the window first, the review alongside. Not picked: process first; only the
>    quick fixes first.
> 4. *The manual's form:* Markdown in the repository, the PDF built; EP-56 picks the renderer.
>    Not picked: written straight as a PDF; all of it left to a planning row.
> 5. *The theme:* Dark replaced, Light kept (EP-49). Not picked: a fourth choice; both rebuilt.
> 6. *The list's count:* open `owner-tasks.md` lines alone, a mark for a project that holds
>    only older forms (EP-50). Not picked: two counts; no mark.
> 7. *The Detail pane's rest:* owner tasks with the older forms, and the mail in and out, move
>    to panes of their own; Decisions, House style and Problems stay, folded (EP-50, EP-51).
>    Not picked: ending at Status exactly; a third pane.
> 8. *The cut:* accepted as drafted: P6 with sixteen rows, 6 S and 10 M, and P7 with EP-44.
> 9. *The charter:* sent on the owner's word with `crw send` into the environment repository's
>    inbox (its IN-5; the file as it was is in the backups folder); the text is kept in the
>    git-ignored `private/ep-45-review-charter.txt`. OT-30, gated before EP-54, tells the owner
>    to start that project's session and to tick when the outcome comes back.
> 10. *IN-15's line:* it named a private project and the environment repository in this public
>     repository's inbox; reworded before the commit, and EP-51's pane flags such a message.
> 11. *The commit:* commit and push (the hash is in the roadmap's row).
>
> **The session's own choices.** EP-43 stays a row of its own and opens the phase (an
> investigation does not share an S). The trims, the chat's send and the re-read are three S
> rather than one M, so that each chains and each can be reverted alone. EP-48's reading of the
> request: the re-read that EP-25a built fires when a session ends, and a VS Code tab left open
> after its brief's commit never ends, so the turn's end and the project's own files are the
> triggers. EP-47's reading: the refusal comes when the CLI's hourly preflight has lapsed, which
> fits the owner's "about half the time". IN-15 rolled in (the `replan` skill's step 4). The
> size rule is written as a D-4 addendum and in the roadmap's Sizes paragraph now, its figures
> left to EP-53. Addenda: D-4, D-10, D-25, D-30. `v2-notes.md` §9 holds the alternatives not
> picked. This brief ran as an M of about an hour.
>
> **For the next session.** EP-43 is next, the owner's to start from the window; headless with
> a chain plan, EP-43, EP-46, EP-47 and EP-48 may run on. EP-54 waits for OT-30.
