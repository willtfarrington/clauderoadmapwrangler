# The roadmap house style

The schema a house-style project's roadmap files follow: the forms Roadmap Wrangler's parser
reads (`crw roadmap <folder>` prints its reading of one project) and the forms its sessions write.
The decisions behind it are clauderoadmapwrangler's D-4 (sizes), D-9 (the convention), D-10
(owner tasks), D-11 (what the tool writes), D-12 (front matter, the completion note and the
manual) and D-13 (the inbox).
The texts quoted under [The engine's texts](#the-engines-texts) are the ones `crw formalise`
writes, byte for byte.

## The files

| File | What it holds | Who writes it |
|---|---|---|
| `CLAUDE.md` | The project's session rules: read first, brief first, owner decisions, commits, build and verify | Sessions and the owner; `crw formalise` adds the rule block between its markers |
| `DECISIONS.md` | The decision log, `D-1` … `D-n`, with dated addenda | Sessions |
| `roadmap/README.md` | The phases, their tables, brief notes, ordering rationale and standing decisions | Sessions |
| `roadmap/EP-n-<slug>.md` | One brief per EP, closed by its completion note | Sessions |
| `roadmap/owner-tasks.md` | What only the owner can do, one line each | Sessions append; the owner ticks |
| `roadmap/inbox.md` | Messages from other projects, one line each | The tool delivers; sessions triage |
| `roadmap/v2-notes.md` | Work named and not built, grown by every re-plan | Sessions |
| `roadmap/_TEMPLATE.md` | The brief template; never read as a brief | `crw formalise` |
| `roadmap/planning-<date>.md` | The record of a planning interview | The plan session |
| `docs/manual.md` | The manual of a project that owes one: reference chapters, then the walkthrough | Sessions: a stub by the session that settles it with the owner, the first manual by a brief of its own, then each re-plan |
| `docs/manual-changes.md` | What changed for a user since the last phase's close, a section per brief | Sessions: each brief that changes what a user sees or does; the re-plan empties it |

The tool writes only the files it owns (D-11): ticks in `owner-tasks.md` and `inbox.md`,
deliveries into `inbox.md`, the formalise step's files, the `.code-workspace` entry. It never edits
the roadmap README, a brief or `DECISIONS.md`, but for the owner's revert, which puts a project
back as it was when a session started, after copying the session's work out (D-11 addendum).

## The roadmap

**Phases.** Each phase is a heading `## Phase P1 — <name> (<status>)`, such as `(planned
2026-10-31)` or `(done 2026-11-04)`, followed by its table. P0 is the planning brief. Each phase
closes with a re-plan brief.

**The phase table.** The header row locates a table, its columns found by name:

```markdown
| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-7 | [Badges and status: stack chips, progress, activity from git](EP-7-badges-and-status.md) (D-24, D-25) | M | EP-6 | core | ☑ `47a2199` |
| EP-8 | Re-plan P1: retro and timings, addenda, P2 confirmed with pickup notes (D-2) | S | EP-1 … EP-7 | core | ☐ |
```

- **#**: `EP-n`, or a lettered id for a brief added or split later (`EP-7a`).
- **Brief**: a one-line summary, linked to the brief file once it exists, with the decisions it
  rests on as `(D-n, …)`.
- **Size**: `S` (about 30 minutes of active work in one session) or `M` (about 45). A brief is S
  or M: an S where its shape is known at planning and it touches a handful of files; an M for an
  investigation, a state machine or the choice of a tool, or cut into a row that finds and a row
  that fixes. A row is not cut only to make it smaller (each brief carries about 15 minutes of
  read-first, brief, checks and close), and a row that proves larger at pickup is split there,
  never run long. `L` (about 2 hours), which older rows hold, is still read. A re-scope stays in
  the cell with the final size last: `S → M (re-scoped at pickup)`.
- **Depends on**: the immediate prerequisites only, ids or ranges (`EP-1 … EP-7`); the table's
  order supplies the rest.
- **Core**: `core` or `stretch`.
- **Done**: `☐`, or `☑` and one backticked commit hash once the brief is committed.

**The next EP is the first row not done, in table order**, not the lowest number. Only `☑` (and
`[x]` in the older form) counts as done: `◐` (done but not landed), `[~]` (in progress) or any other
cell is not, so the next EP stops at it and the counts leave it out.

**Brief notes.** Under each table, `**Brief notes.**` and one bullet per brief:
`- **EP-n.** …`, the detail a session starts from (the row's Brief cell is the one-liner; the
launch prompt carries the bullet verbatim). A later session adds to a bullet in segments that open
with a bold label: `**From EP-12:** …` for what an executed brief learned, `**From IN-4
(2026-11-01):** …` for a triaged message. A brief note for an EP two or more briefs ahead may also
take a parked block, `> **Addendum (date, from …).** …`, which the session that picks the brief up
folds in. Only the project's own sessions write either (at triage or from an executed brief),
never a sender: another project's word arrives through the inbox.

**Ordering rationale and standing decisions.** Under each table, a paragraph says why the rows
come in that order, and a dated sentence is added for each change (`*(2026-11-10, EP-20 split at
pickup, the owner's choice.)* …`); a second paragraph names the decisions every brief of the
phase obeys.

## Briefs

One brief per session, unless the owner asks for more. A session for EP-n first writes
`roadmap/EP-n-<slug>.md` from its row and its brief note, before touching any other file:

- **Front matter** (D-12): `ep`, `title`, `size`, `model`, `effort`, `status` (`in progress`, then
  `done`), kept true to the session. A lettered id is written with or without its prefix (`ep: 7a`
  or `ep: EP-7a`).
- A title line `# EP-n — <title>` and `**Size:** M · **Depends on:** EP-k · **Blocks:** EP-m`.
- **Context**: what the row and its note ask for and what exists; a **Session start** paragraph
  (how the session was launched, what it read first, the state of the tree); a **Decided at
  pickup** list when the owner answered questions before the work.
- **In scope**, **Out of scope**, **Verification / acceptance**. Verification ends with a
  **Residual risk** paragraph: what the session could not exercise (real sessions, other machines,
  the owner's own windows) and why it expects it to hold. The phase's re-plan walks these
  paragraphs with the owner.
- The brief ends with the **completion note** (D-12):
  `> **Completion note (2026-11-10; claude-opus-5-5/max; 1 h 20 min).**`: what was built, each
  owner decision and its outcome (a table `| # | Decision | Owner's answer | Outcome |` reads well),
  the routine choices the session made, and notes for later sessions. Older forms (`> **Completion
  note (date).**`, `## Completion note`, `> **☑ COMPLETE**`) are read as well. A note whose label
  still reads `YYYY-MM-DD`, the template's placeholder, is no note: the brief stays open until the
  real one is written.
- A message for another project follows the note as its own block,
  `> **Handed on (to <project>).** …`; the tool delivers it into that project's inbox once the
  brief is committed (D-13). Only for a receiver outside the tool's registry does the session
  write the inbox line itself. A block `(to the owner)` is an owner item, not mail.

## Decisions

`DECISIONS.md` holds one entry per decision:

```markdown
**D-7 Discovery and the workspace file.** Decision, in plain sentences. *Why.* The facts it rests
on. *Alternatives.* What was weighed and not taken.
```

An entry is never rewritten. A later change is a dated addendum under it:

```markdown
> **Addendum (2026-11-01, EP-8).** What changed and why. *Alternatives.* What was weighed.
```

## Owner tasks

`roadmap/owner-tasks.md` holds one line per task only the owner can do: a write into the owner's
user settings or into another project, a migration an older build refuses, the first real run of a
new route. A look at a changed surface is no task: the brief's residual-risk paragraph names it.

```markdown
- [ ] OT-n (EP-k, YYYY-MM-DD) What to do, where to look and what to expect. [before EP-m]
```

The `[before EP-m]` gate is written only when EP-m builds on the task. Every line of work is a
top-level item below a header paragraph, whose example line stays in backticks; any other box is a
problem to the parser. The owner ticks a task in the tool, which changes the box and nothing else
(a copy of the file goes to the tool's backups folder first). Older forms of the owner's to-dos (a
`## Risks / open items` list struck through when resolved, `owner-decisions.md` and
`open-questions.md` registers, handoff notes, the `Parked` and `Handed on` blocks of the latest
completed brief) are read and shown, never rewritten.

## Inbox

`roadmap/inbox.md` holds one line per message from another project:

```markdown
- [ ] IN-n (from <project> EP-k, YYYY-MM-DD) The message. [urgent]
```

**Triage**, right after reading the roadmap at a session's start: for each open message, roll it
into this session if it fits the brief and is at most S-sized; otherwise add it to the brief notes
of a named later EP or propose a new row in the right phase, confirming with the owner through
`AskUserQuestion`; or decline it with a reason. Write the disposition on the message's line, ` **Triaged at
EP-n (date):** …` or ` **Disposition (date):** …`, and tick its box. A running session is never
interrupted by a delivery; an `urgent` message notifies the owner, not the session.

## Commits

`<Title>: <summary> (EP-n, D-x)` for the work, then `docs(roadmap): record EP-n commit hash` for
the row's Done cell; a project whose `CLAUDE.md` states its own pattern keeps it. Commits are the
owner's call (with the autopilot on for a project, its standing yes); messages go through a file,
without AI attribution trailers. A session never pushes unless the owner says so.

## Re-plans and parked work

A re-plan brief closes each phase: the walk of the phase's residual-risk paragraphs, a retro of
planned against measured times, dated addenda, the next phase re-cut with pickup notes,
`roadmap/v2-notes.md` grown with what was named and not built, and a manual's changes folded
into it. The plugin's `replan` skill has the steps.

## The manual

A project the owner calls working well enough to owe a manual keeps `docs/manual.md`, and the
file's presence is the mark (D-12): the session that settles it with the owner creates it, a
stub holding the outline below, with an empty `docs/manual-changes.md`, and a brief of its own
writes the first manual; a project without it owes none and writes no changes. The manual is
plain Markdown, its pictures beside it in `docs/` (a public repository's showing synthetic data
only):

```markdown
# <Project> — manual

What the project is and for whom, in a paragraph, and the phase it describes.

## What it does
## Install and start
## <A chapter per surface: its commands, panes or pages, files and settings>
## When something goes wrong
## Walkthrough

### <A route no real use has met>

1. The steps, in order.
2. What to look at.
3. What to note down.
```

The walkthrough stays while the project's full use is still wanting: each re-plan takes out the
routes real use has met (its walk says which) and adds the new ones no use has met, and the
chapter goes once every route has been used. `docs/manual-changes.md` holds what changed for a
user since the last phase's close; each brief that changes what a user sees or does adds its
section before its close, as it writes its completion note:

```markdown
# <Project> — changes since the last phase's close

Since EP-n (YYYY-MM-DD), the last re-plan or the file's creation. Each brief that changes what a
user sees or does adds its section; the next re-plan folds them into `docs/manual.md` and
empties this file.

## EP-k — <title> (YYYY-MM-DD)

- What changed, and where to see it.
```

The phase's re-plan folds the sections into the manual's chapters, empties the file to its
header with a new `Since` line and builds the manual's PDF to read it through (the `replan`
skill's step 8).

**The PDFs** are built, never tracked: `crw manual <project>` (with `--changes`, the changes
file) turns the Markdown into [Typst](https://typst.app/) markup inside one template (a title
page, a contents page, numbered chapters, page numbers, Typst's built-in fonts alone) and runs
the Typst CLI, pinned at 0.15.1, so the same source gives the same bytes, offline, into
`%LOCALAPPDATA%\clauderoadmapwrangler\manuals\<project>-manual.pdf` and
`<project>-manual-changes.pdf`; the tool writes nothing into the project for it. Typst is
installed with `winget install --id Typst.Typst --version 0.15.1`, a portable package that
needs no administrator, and held there with `winget pin add --id Typst.Typst --version 0.15.1`;
`crw manual` refuses another version.

## A session in progress

The tool reads the roadmap at git `HEAD` and in the working tree and keeps both. A session is in
progress when the roadmap README is changed, a brief is untracked or changed, or a row is `[~]`;
in a folder that is not a repository, when the next EP's brief has no completion note and is the
only open brief. A live session in the folder is a reason of its own.

## Older forms

Two older table forms are read, not written: a five-column table (`| # | Brief | Size | Depends on
| Done |`) and the work-packet style (`| # | Packet | Size | Depends on | Status |`, `[x] hash` and
`[~]` cells, handoff payloads, letter-suffixed ids). `crw formalise <project> --migration` prints
the prompt of a session that converts either to the house tables, keeping every id, hash and file.

## The engine's texts

What `crw formalise` writes into a project (clauderoadmapwrangler's `crw_core::formalise` and
`inbox::mailman`), quoted byte for byte; the plugin's structural test fails when they differ.

**The rule block** at the end of the project's `CLAUDE.md` (the root's, else `.claude/CLAUDE.md`).
The tool may rewrite what lies between the markers; the project's own rules outside them win.

```markdown
<!-- roadmap-wrangler:begin -->
## Roadmap house style

Written by Roadmap Wrangler (`crw formalise`; clauderoadmapwrangler D-4, D-10 … D-13) between
these markers, which it may rewrite: edit outside them. This file's own rules win where they
differ.

- **Read first:** `roadmap/README.md` (the next unchecked EP in table order and its brief notes
  under the phase table), `DECISIONS.md`, the last executed brief's completion note,
  `roadmap/owner-tasks.md` and `roadmap/inbox.md`.
- **Inbox triage:** for each open message, `- [ ] IN-n (from <project> EP-k, date) text`, roll it
  into this session if it fits the brief and is at most S-sized; otherwise add it to the brief
  notes of a named later EP or propose a new row in the right phase, confirming with the owner
  through `AskUserQuestion`; or decline it with a reason. Write the disposition on its line and
  tick its box.
- **Brief first:** an EP session starts by writing `roadmap/EP-N-<slug>.md` from
  `roadmap/_TEMPLATE.md`: YAML front matter (`ep`, `title`, `size`, `model`, `effort`, `status`),
  then Context, In scope, Out of scope and Verification / acceptance.
- **Completion note:** the brief ends with
  `> **Completion note (YYYY-MM-DD; model/effort; duration).**`, recording each owner decision and
  its outcome. A message for another project follows as a `> **Handed on (to <project>).**` block,
  which the tool delivers into that project's `roadmap/inbox.md` once the brief is committed.
- **Owner tasks:** what only the owner can do is appended to `roadmap/owner-tasks.md` as
  `- [ ] OT-n (EP-k, date) text`, ending `[before EP-m]` only when EP-m builds on it.
- **The roadmap:** phase tables `| # | Brief | Size | Depends on | Core | Done |`, a done row's
  cell `☑` and one backticked commit hash; each upcoming EP's detail is a `- **EP-n.** …` bullet
  under `**Brief notes.**`.
- **Sizes:** a brief is S (about 30 minutes of active work) or M (about 45): an S where its shape
  is known at planning and it touches a handful of files, an M for an investigation, a state
  machine or the choice of a tool. A row is not cut only to make it smaller, and a row that
  proves larger at pickup is split there, never run long.
- **Manual:** a project the owner calls working well enough to owe one keeps `docs/manual.md`
  (reference chapters, then a walkthrough of the routes no real use has met) and
  `docs/manual-changes.md`, where each brief that changes what a user sees or does adds a
  `## EP-n — <title> (date)` section before its close; the phase's re-plan folds that file into
  the manual and empties it. `crw manual` builds the PDFs, which are never tracked.
<!-- roadmap-wrangler:end -->
```

**The owner tasks' header**, `roadmap/owner-tasks.md`:

```markdown
# Owner tasks — what only the owner can do

One line per task (clauderoadmapwrangler D-10): `- [ ] OT-n (EP-k, date) text`, ending
`[before EP-m]` only when EP-m builds on it. Sessions append at completion; the owner ticks a task
in Roadmap Wrangler or here.
```

**The inbox's header**, `roadmap/inbox.md` (also what the tool writes when it delivers into a
project that has no inbox):

```markdown
# Inbox — messages from other projects

Read right after the roadmap at the start of a session. Triage rule (clauderoadmapwrangler D-13):
for each open message, roll it into this session if it fits the brief and is at most S-sized;
otherwise add it to the brief notes of a named later EP or propose a new row in the right phase,
confirming the choice with the owner; or decline with a reason. Write the disposition on the
message's line and tick its box. One message per line,
`- [ ] IN-n (from <project> EP-k, date) text`, delivered by the tool.
```

**The brief template**, `roadmap/_TEMPLATE.md`:

```markdown
---
ep: N
title: The row's title
size: S
model: the model's id
effort: the effort level
status: in progress
---

# EP-N — The row's title

**Size:** S · **Depends on:** EP-k

## Context

What the row and its brief notes ask for, and what exists. **Session start (YYYY-MM-DD).** How the
session was launched and what it read first.

## In scope

1. …

## Out of scope

- …

## Verification / acceptance

- The tests and the checks the project's rules name.
- **Residual risk.** What the session could not exercise, and why it expects it to hold.

<!-- At the close, and not before: the brief ends with its completion note, a blockquote that
opens `> **Completion note (YYYY-MM-DD; model/effort; duration).**` with the real date, model,
effort and duration in place of those words (a label that still reads YYYY-MM-DD is no note),
then records what was built, each owner decision and its outcome, the routine choices and notes
for later sessions. A message for another project follows it as a
`> **Handed on (to <project>).**` block. Delete this comment. -->
```
