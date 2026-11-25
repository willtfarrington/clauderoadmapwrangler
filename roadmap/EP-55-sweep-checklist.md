---
ep: 55
title: The sweep's checklist, an owner task and a message for each project that is not active
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-55 — The sweep's checklist, an owner task and a message for each project that is not active

**Size:** S · **Depends on:** EP-50, EP-54 · **Blocks:** EP-59 (its walk reads how far the sweep came)

## Context

EP-45 (the owner's request 9 and their choice) planned a sweep through every registered project
that is not active: one session inside each project, which the owner starts and answers, guided
by an owner task in this project's own file, one per project, worded without the project's name
(D-10 addendum of 2026-11-22), and by a message in that project's inbox, sent through the tool on
the owner's word (rule 4). The names and the full steps live in a checklist in the git-ignored
`private/`, which each task points to (rule 3). The registry's states stay the owner's to set
(D-6; the window's state chip, or `crw set`), and EP-59's walk reads how far the sweep came by
the tasks' ticks.

What exists: the Owner tasks pane (EP-50) shows the tasks, folding one longer than about six lines
(EP-51a); `crw send <project> <words…>` writes the owner's message into a project's
`roadmap/inbox.md`, creating the file with the house header where it is missing, and warns past
the 1,000 characters a revert keeps (D-13 addenda of EP-20c and EP-20d); `crw formalise <project>`
previews what Formalise… would write and changes nothing. Since EP-54 (D-12 addendum of
2026-11-24) the rule block holds a Sizes bullet, so the window, which runs EP-54's build since its
restart at 07:35, offers Formalise… on every formalised project's row, "the rule block replaced
between the markers, the text around it kept": the conventions' update, taken on the owner's
word; a project with no block takes the new one whole. A sweep session in VS Code uses the plugin
the owner installed, so it reads best after OT-32 (the push, then `claude plugin update`).

**The survey (read-only, `crw scan`, `crw status --offline` and a `crw formalise` preview per
project).** Fifteen projects are registered; ten are not active: two unformalised (no roadmap),
five paused, two maintenance, one done; two of the ten are data repositories. By the previews:
two have no `CLAUDE.md`, five have one without the rule block, two hold the old block (replaced
by Formalise…), one keeps the five-column tables (Migrate…), one holds a roadmap README with no
phase table the parser reads, and two have no roadmap at all. Names and per-project lines are in
the checklist, never here.

**Session start (2026-11-25).** A chained headless run: the window restarted into EP-54's build
at 07:35 and chained this brief at 07:36 (ledger row `b6e43327`, EP-55, claude-opus-5-5/max, the
chain's mark 70 % of the session window); the prompt is the tool's opener. S's cap of 1 h 15
min: judged to fit with its checks and close (no code; one private checklist; ten short tasks
and ten messages from one template; the interview's waits not counted), so not split. Read
first: the roadmap's P6 table and EP-55's brief note with EP-54's pickup note, `DECISIONS.md`
(D-6, D-10, D-13 and their addenda), EP-54's completion note, `roadmap/owner-tasks.md` (OT-32
and OT-33 open, neither gating) and `roadmap/inbox.md` (no open message). Field check since
EP-54's record commit (`b20bf1a`, 07:34): `crash.log` last written 2026-11-23 04:00, nothing new;
`crw notifications`: EP-54's finish and this chain's start, nothing odd; `crw autopilot`: not
paused, six projects on, the log's last lines the restart's hold and this chain; `crw sessions`:
this run and the owner's idle manual session of last night. Nothing to triage.

## In scope

1. **A short interview** (`AskUserQuestion`, rule 7): which projects the sweep covers, in what
   order (the brief note's: the unformalised and the paused first, then maintenance and done),
   and what the owner already knows they want for any of them.
2. **The checklist** `private/sweep-checklist.md` (git-ignored): per project in the sweep's
   order, its name, state, whether it is a data repository, its hosting, the formalise preview's
   lines, the route for its form (`/roadmap-wrangler:formalise` or Formalise… for missing
   house-style files, Migrate… for five-column or work-packet tables, `/roadmap-wrangler:plan`
   for no roadmap, `/roadmap-wrangler:status` and a state question for a done or maintenance
   project), the session's steps, what it settles, the message's text and its id once written.
3. **The owner tasks**, one per project, in `roadmap/owner-tasks.md`: no project's name, its
   registry state and route saying which, pointing to the checklist's item; ungated (no brief
   builds on them; EP-59's walk reads their ticks).
4. **The messages**, one per project, through `crw send` on the owner's word, one child at a
   time: what its session settles with the owner, namely its state (active; paused with a reason
   and a pickup note; maintenance and what maintenance means there; done; or abandoned and
   archived), the house-style files by its route, the sizes under EP-53's rule, and whether it
   works well enough to owe a manual (EP-56's convention, still to come). Each under the
   revert's 1,000 characters, naming no other project (several land in public repositories,
   IN-15's flag).
5. **Records:** a D-10 addendum (the sweep as written); the row checked with its hash.

## Out of scope

- Setting any project's state (the owner's, D-6, rule 13); running Formalise…, Migrate… or a
  plan in any project (the owner's, in the sweep's sessions); any write into another project but
  the tool's own inbox line (rule 4, D-11).
- The five active projects: their row menus offer Formalise… for the replaced block too, which
  their own sessions or the owner take; one line in the checklist says so.
- The manual's convention (EP-56); the window's old size words (EP-59's parked addendum).
- Code: none changes, so no screenshot run (no list or pane changes, rule 5).

## Verification / acceptance

- The parser reads the new tasks: `crw tasks clauderoadmapwrangler` lists them as open, none
  gated.
- Each `crw send` prints the id it wrote; `crw mail` lists each as `written IN-n from the
  owner`; no warning for length.
- The word check of rule 3 on each staged diff prints nothing (no project's name, path or
  machine detail in the tasks, the brief or the addendum).
- No code changes, so `cargo test`, clippy and fmt are not rerun (as at EP-53); the release
  build and the install run last, as the opener asks.
- **Residual risk.** The sweep's sessions run in other projects under the owner's hands, so
  nothing here exercises how a session there reads its message, a session in a project with no
  next brief picks it up, or the plugin's skills (the owner's copy before OT-32) guide its
  formalise or plan. The tasks' wording is read only as text here: the Owner tasks pane's look
  at them is the owner's (no pane changed). A message to a project whose inbox file is missing
  creates the file with the house header, which the formalise previews then keep; a data
  repository's session runs in its own folder, which this session never lists.

> **Completion note (2026-11-25; claude-opus-5-5/max; about 25 minutes of working time from the
> start at 07:35, the owner's one round of answers not counted).**
> A chained headless run (ledger row `b6e43327`), judged at the brief to fit S's cap, so not
> split. Written: the survey of the ten registered projects that are not active (read-only:
> `crw scan`, `crw status --offline`, a `crw formalise` preview each); the checklist
> `private/sweep-checklist.md` (git-ignored), one item per project in the owner's order with its
> name, state, data flag, hosting, the preview's lines, its route, the common steps and its
> message; ten messages from the owner, one in each project's inbox, written by `crw send` at
> 07:46–07:47 (four inbox files created with the house header; each line under 1,000 characters
> with its id, ASCII only, no word of rule 3's list; `crw mail` lists all ten as `written IN-n
> from the owner`); the owner tasks OT-34 … OT-43, "sweep n of 10", worded by state and route
> without names, ungated; a D-10 addendum; EP-59's brief note naming the sweep's tasks.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Which projects the sweep covers | **All ten** (recommended; the alternatives left out the site, the site and the environment repository, or the three whose rows are all checked) | Ten tasks and ten messages |
> | 2 | The tasks' order | **By state, freshest first** (recommended; the alternative was the least work first) | Unformalised, paused, maintenance, done; within a state the most recent commit first |
> | 3 | A state or a manual already known for any of them | **Nothing settled yet** (recommended; the alternative kept the done and maintenance ones as they are) | Each project's session asks; the messages state no outcome |
> | 4 | Sending the messages | **Send them now** (recommended; the alternative kept them in the checklist for the owner to send from each row) | Sent by `crw send`, one child at a time |
>
> The commits are the autopilot's standing yes (crw's D-22); the push is the owner's.
>
> **Session's calls.**
>
> 1. The sweep's session is opened by hand in each project, not by the row's Start: Start runs a
>    project's next brief, and six of the ten have none (every row checked, or no roadmap the
>    parser reads). The tasks and the checklist say so.
> 2. The messages name no other project and no machine detail (several of the inboxes are in
>    public repositories, IN-15's flag) and are ASCII only (the window's "…" written as "...", so
>    that no encoding passes through the shell); they were sent with `MSYS_NO_PATHCONV=1`, so that
>    Git Bash left the `/roadmap-wrangler:` commands as written.
> 3. The tasks carry the registry state and the route, the checklist the rest (progress,
>    hosting, the data flag, the changed trees), as the brief note reads rule 3; the counts in
>    the brief and the addendum are aggregates.
> 4. Every task is ungated: no brief builds on the sweep, and EP-59's walk reads the ticks. Each
>    says "after OT-32", since a VS Code session takes the skills EP-54 changed only from the
>    updated plugin.
> 5. No code changed, so `cargo test`, clippy and fmt were not rerun (as at EP-53), and no
>    screenshot run was owed (no list or pane changed). The release build and the install run
>    last, as the opener asks.
>
> Checks: `crw tasks clauderoadmapwrangler` (12 open, 0 gated, no problem); `crw mail` (the ten
> `written` rows); the word check of rule 3 on the messages and on the staged diff; then the
> release build and the install.
>
> No `Handed on` block: the ten messages went by `crw send` on the owner's word, since a block in
> a tracked brief would name its receiver (rule 3).
