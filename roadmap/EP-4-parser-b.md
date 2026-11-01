---
ep: 4
title: Parser B — briefs, decision logs, format profiles and the snapshot
size: M
model: claude-fable-5-1
effort: high
status: done
---

# EP-4 — Parser B: briefs, decision logs, format profiles and the snapshot

**Size:** M · **Depends on:** EP-3 · **Blocks:** EP-5, EP-6, EP-25

## Context

EP-3 reads a roadmap README: phases, rows, brief notes, the next EP and the counts. What it cannot
tell is whether a session is at work on the next EP, which needs the brief files (D-9: a brief
without a completion note, an untracked brief) and a snapshot that separates what is committed from
what a running session is editing (`git show HEAD:` against the working tree). This EP adds the
second half of the parser: the brief files with their front matter (D-12) and completion notes, the
decision log, the format profiles' completion evidence (`house`, `house5`, `packet`), the snapshot,
the "session in progress" derivation, and the two facts `crw scan` has been waiting for since EP-2
(`all_rows_checked`, `pickup_gate`). EP-5 adds the owner-item files, EP-6 shows the result.

**Session start (2026-10-31, 19:20).** The first session opened from
`DATA\clauderoadmapwrangler.code-workspace` after the owner's workspace split (the environment
repository's D-1 … D-8).
Checked at the start, as the request asked: the cwd is this folder; the user-level `SessionStart`
hook printed IN-1 and IN-2; the four earlier transcripts (EP-0 … EP-3, identified by the brief each
one edited) sit under `~\.claude\projects\c--Users-willi-Documents-DATA-clauderoadmapwrangler\`,
where EP-0 … EP-3 had recorded the filing deviation. Nothing to correct.

**Verified at pickup (read-only survey of the twelve `roadmap/` folders under the data root).** What
the brief parser must cope with, all of it named in the planning record §2.6:

- **368 brief files**, every one named `EP-<n>[a-z]?-<slug>.md`; one project keeps
  `EP-9-completion-handoff.md` and `EP-9-completion-report.md` beside its EP-9 brief; three
  `_TEMPLATE.md`; other files in the folder (`inbox.md`, `v2-notes.md`, charters, retro files,
  `owner-decisions.md`, `open-questions.md`, `pickup-gate.md`) are not briefs.
- **Front matter** (D-12) only in this repository's four briefs so far; a `---` elsewhere is a
  thematic break, never on line 1.
- **Header line:** `**Size:** M · **Depends on:** EP-1 · **Blocks:** …` in the house style; other
  projects add `**Tier:**`, `**Mode:**`, `**Type:**`, `**Core/Stretch:**`, `**Milestone:**`,
  `**Effort:**`, and one project wraps the line (it ends with ` ·` and continues on the next).
- **Completion markers:** 125 `> **Completion note (…).**` blocks: `(date)` 114, `(date, EP-n
  verification)` 5, the new `(date; model/effort; duration)` form 4 (this repository and one other),
  `(date, appended by EP-n — …)` 2; two briefs carry two notes; one `> **☑ COMPLETE (date).**`; one
  `## Completion note (date)` heading; one `**Status:** DONE (executed date, …)` line. The packet
  style: `## Handoff payload (filled date)` 17 (and 2 unfilled `(fill at session end)`), a header
  `**Status:** [x] hash (done date) · **Milestone:** M2 · …`, `**Status:** split date into [EP-5a](…)
  and [EP-5b](…)`, `**Status:** deferred — …`; and 12 `docs/handoffs/EP-N.md` files opening with
  `**Status:** complete · **Date:** date · **Brief:** roadmap/EP-N-….md` and a `**Commits:**` line.
- **Other blocks:** `> **Charter.**` 120, `> **EP-n pickup note (date).**` 22, `> **Handed on.**`
  10 and `> **Handed on (to …).**`, `> **Parked.**` 7, `> **Parked and handed on.**` 4, owner-decision
  blocks in several wordings, and one `> **PICKUP GATE — owner input required before this brief
  starts (EP-8, date).**`.
- **Decision logs:** eleven `DECISIONS.md` (nine at the root, two one level down in the package
  folder); 261 bold `**D-n …**` openers of which 234 end in `.**` (the others are range labels such as
  `**D-9 … D-22**` and titles with a code span); 241 addenda: `(date, EP-n)` 162, `(date,
  maintenance)` 9, `(date, planning)` 5, `(date)` 4, `(date, EP-n close-out)` 3, `(date, owner —
  recorded by EP-n; …)` 4 and a few more; two logs carry an index table (`| # | Title | Decides |`,
  `| D-n | Title (short) | State | Last addendum |`); ADR folders in two projects (`roadmap/adr`,
  `docs/adr`); one roadmap README carries addenda of its own (`(date, from <project> EP-4)`).
- **Working trees:** no other project has a dirty `roadmap/` file at the moment; this repository
  will, as soon as this brief is written.

**Inbox and owner tasks.** `roadmap/owner-tasks.md` does not exist yet. `roadmap/inbox.md` holds
IN-1 and IN-2 (both from the owner, 2026-10-31); their triage is below the completion note.

## In scope

1. **Blockquotes in the scanner** (`roadmap::markdown`): a `Quote` block of consecutive `>` lines
   with the marker stripped, so completion notes and the other labelled blocks are read; the table
   parser keeps ignoring them.
2. **Briefs** (`core::roadmap::brief`), read-only:
   - discovery in `roadmap/`: `EP-<n>[a-z]?-<slug>.md` (never `_TEMPLATE.md`; an
     `EP-n-completion-*.md` is a companion, not the brief); one brief per id, the companions kept
     aside;
   - YAML front matter on line 1 (`ep`, `title`, `size`, `model`, `effort`, `status`, read as
     `key: value` lines, quotes stripped; no YAML crate), the `# ` title, the header line's bold
     fields (`**Size:**`, `**Depends on:**`, `**Blocks:**`, `**Status:**`, and whatever else it
     carries, wrapped lines joined), with the size, the dependency ids and the blocked ids extracted;
   - labelled blockquote blocks, each with its label, kind, date, text and line: completion note
     (date; model and effort; duration when the new form is used), `☑ COMPLETE`, handed on (with its
     target), parked, pickup note (with its EP), charter, pickup gate, owner decisions, other; two
     notes in one brief are two blocks;
   - the heading-form markers: `## Completion note`, `## Handoff payload (filled date)` or `(fill at
     session end)`, `**Status:** DONE` / `complete` / `[x] hash` / `split` / `deferred`;
   - **completion** as one answer with its evidence: a completion note, `☑ COMPLETE`, a completion
     heading, a done status, a filled handoff payload, or front matter `status: done`;
   - the packet profile's `docs/handoffs/EP-N.md` with its `**Status:** … · **Date:** … ·
     **Commits:**` line, read as the same header-line form;
   - **the pickup gate** (`PICKUP GATE` block) as a fact (D-6).
3. **Decision logs** (`core::decisions`): `DECISIONS.md` at the project root, else one level down
   (not under `.git`, `target`, `node_modules`, `.venv`); `**D-n Title.**` entries with their number,
   title and line (a range label or a title without the closing `.**` is not an entry); addenda
   `> **Addendum (date, source).**` with date and source, the count and the last date; index-table
   rows and ADR files (`roadmap/adr`, `docs/adr`) counted only.
4. **Format profiles**: the completion evidence per profile — `house` and `house5` from the brief's
   markers; `packet` also from `[x] hash` cells (EP-3 reads them), the filled handoff payload and the
   handoff file — behind one `Project`-level reading so EP-6 asks one question.
5. **Snapshot** (`core::roadmap::snapshot`): per project folder, the README from `git show
   HEAD:roadmap/README.md` and from the working tree, both parsed and both kept; `git status
   --porcelain -z -- roadmap` for the dirty README, untracked and modified briefs (git children one
   at a time with the EP-2 spacing; no git → working tree only); the briefs and the decision log
   read from the working tree; **session in progress** with its reasons: the README is dirty, a brief
   is untracked or modified, a row is `[~]`, or the next EP's brief exists without completion while it
   is the only open row with a brief (a project that pre-writes every brief shows in progress only
   from the working tree); and the facts for `proposal::Facts`: `all_rows_checked` from
   `Counts::all_checked` of the working-tree reading (HEAD when the tree does not parse) and
   `pickup_gate` from the next EP's brief.
6. **`crw scan`** fills the two facts from the parse (file reads only, no further git children) so
   it proposes done or maintenance and paused from them; **`crw roadmap <project|folder>`** prints
   the snapshot (both readings, the briefs found, the next EP's brief with its front matter and
   completion, the decision log, the in-progress reasons); `crw roadmap <brief file>` prints a
   brief's parse; a README path prints the table reading as before.
7. **Fixtures and tests**: four synthetic project folders under `crates/core/fixtures/projects/`
   (`house` with front matter, the new note form, two notes, handed-on and parked blocks, a charter
   brief and a companion file; `house5` with the heading, `☑ COMPLETE` and `**Status:** DONE` forms
   and no decision log; `packet` with `[x]` cells, handoff payloads, split packets and
   `docs/handoffs`; `prewritten` with every brief written, a `PICKUP GATE` and a nested decision log
   with an index table); unit tests per form and per profile; an integration test over the fixture
   folders and one over a scratch git repository (clean, dirty README, untracked brief, committed
   brief without a note) using git only (D-29).

## Out of scope

- `owner-tasks.md`, `inbox.md`, `Handed on` delivery and the legacy owner-item forms (EP-5); the
  handed-on blocks are read here only as labelled blocks.
- The dashboard (EP-6) and daily `snapshot` rows in the ledger (EP-25).
- Writing any roadmap file (D-11); migrating any project (D-9, v2 notes).
- Running sessions as an in-progress source (EP-12's state model).
- Parsing a brief's body sections (Context, In scope …) beyond the title, the header line, the
  headings and the labelled blocks; YAML beyond flat `key: value` lines.
- IN-1 and IN-2's own work: parked under EP-8, EP-11, EP-12 and EP-21 (see the triage below).

## Verification / acceptance

- The gate passes with no warnings: `cargo test`, `cargo clippy --all-targets --features screenshot`,
  `cargo fmt --check`, `cargo build --release`.
- Unit tests cover every surveyed form: brief file names, front matter, the header line in its
  five shapes, every completion marker, the labelled blocks, the handoff payload and the handoff
  file, the decision entries and addendum labels, the nested log, the in-progress rules and the
  facts.
- The four fixture folders parse to their expected briefs, completion, decisions, next EP and
  in-progress reasons; the scratch-repository test shows HEAD and tree readings diverging and the
  three in-progress reasons appearing and clearing.
- `crw roadmap` on every project folder under the real data root, with `CRW_HOME` on a scratch
  folder (nothing written anywhere): the briefs found equal `ls`'s count, the completion markers of
  the next EP's brief match a read of the file, the decision counts equal `grep`'s, no project but
  this one reads as in progress, and this one does (an untracked brief). `crw scan` under `CRW_HOME`
  with a scratch registry proposes done or maintenance for the all-checked projects and paused for the
  pickup-gated one.
- No private project name in the tracked files (`git grep` for the names kept in `private/`).

> **Completion note (2026-10-31; fable/high; ≈ 40 min to the owner's questions).** The first session
> from `DATA\clauderoadmapwrangler.code-workspace`: the three checks the request asked for passed (the
> session-start paragraph above), so the transcript-filing deviation EP-0 … EP-3 recorded is closed.
> Owner decisions (`AskUserQuestion`, every one the recommended option): **commit**: both commits in the
> house pattern, pushed to the private remote; **D-9 addendum**: recorded — with git, in progress = a
> changed README, an untracked or changed brief, or a `[~]` row; the brief-without-note rule only outside
> a repository and only for the next EP when it is the only open brief; **rule 11**: updated to name the
> workspace file and the read-only grant; **inbox triage**: the parking map kept (IN-1 and IN-2 ticked;
> sentences in the brief notes of EP-8, EP-11, EP-12 and EP-21). Routine choices made by the session: a
> `Quote` block in the scanner, with lazy continuation lines; labels read from the rest of their block,
> because 19 addenda in one log wrap onto a second quoted line; front matter as flat `key: value` lines on
> line 1 only (no YAML crate; a `---` later is a thematic break); header fields found by `**Label:**`
> tokens, so the wrapped form and a `**Status:**` on the next line read, and only above the first `##`; a
> label of a known kind starts a new block, so two notes in one quote are two blocks and `**After the
> commit:**` stays inside its note; a decision entry needs the closing `.**` (range labels and bold
> mentions are not entries), domain ids are counted only; completion evidence in one order (note, `☑
> COMPLETE`, heading, status, filled payload, front matter `done`); `View::facts()` reads the working
> tree only, with no extra git children, while the full snapshot with its two git calls is `crw
> roadmap`'s and EP-6's; `crw roadmap` prints a project, a README or a brief by what the path is; `git
> status` is asked with `-z`. Verified: `cargo test` 93 passed (23 new: 17 unit, 6 over the fixture
> folders and a scratch repository), clippy with `-D warnings` and `cargo fmt --check` clean, the release
> build in 5 s through `install-shortcut.ps1`, which refreshed both shortcuts; `crw roadmap` on all 13
> project folders under the data root with `CRW_HOME` on an empty scratch folder that stayed empty: brief
> counts equal `ls` in all 13 (368 files, two of them companions), decision entries equal `grep` in all 11
> logs, addenda equal `grep` outside code fences (85 in the nested log once wrapped labels read; the one
> difference in another log is a template inside a fence), 78 domain-id entries and 10 ADR files in the
> domain-id project and 8 ADR files in the packet project; the in-progress derivation fired for this
> repository alone (the untracked brief, then the changed README) and, after the D-9 refinement, no
> longer for the two projects whose next brief is pre-written; `crw scan` on the owner's registry (reads
> only) proposed nothing, which is right: the all-checked projects are already maintenance or done and
> the gated one already paused. Found on the way: the domain-id project writes its decisions as list
> items with an em dash, so they are a count, not entries. No owner tasks; no messages for other
> projects. Next: EP-5 (owner tasks and inbox files), from this workspace file.
