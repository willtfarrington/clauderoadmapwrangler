---
ep: 5
title: Owner tasks and inbox files
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-5 — Owner tasks and inbox files

**Size:** M · **Depends on:** EP-4 · **Blocks:** EP-6, EP-10, EP-20, EP-21

## Context

EP-3 and EP-4 read a project's roadmap, briefs and decision log. What the dashboard still cannot
show is what waits on the owner and what other projects have sent: the owner's to-dos between
sessions (D-10) and the cross-project inbox (D-13). This EP reads `roadmap/owner-tasks.md` and
`roadmap/inbox.md`, ticks their lines in place (the tool's first write into a managed project,
D-11), extracts the `> **Handed on (to <project>).**` blocks a session leaves for another project
with their targets resolved against the registry (EP-20 delivers them), and reads the older forms
of the owner's to-dos read-only into one `OwnerItem`, so EP-6's detail tile asks one question.

**Session start (2026-11-01).** Started from `DATA\clauderoadmapwrangler.code-workspace`. The inbox
holds IN-1 and IN-2, both triaged and ticked at EP-4: nothing open. `roadmap/owner-tasks.md` does
not exist yet (EP-4 had no owner tasks).

**Verified at pickup (read-only survey of the thirteen `roadmap/` folders under the data root, and
of one project's `docs/handoffs/`).** What the readers must cope with:

- **`owner-tasks.md`:** none anywhere yet; the file arrives with the sessions that need it and with
  EP-21's formalise step.
- **`inbox.md`:** seven projects, this one included, each opening with `# Inbox — messages from
  other projects` and a triage paragraph; messages in two of them (four in all, three ticked). A
  message is a top-level list item `- [ ] IN-n (from <sender>, date) text` whose text wraps onto
  indented continuation lines; the sender is a project and its EP (`from <project> EP-0,
  2026-10-31`) or the owner with a context (`from the owner, workspace split, 2026-10-31`); the
  disposition is a bold run at the end of the item (`**Triaged at EP-4 (2026-10-31):** …`) or on a
  continuation line of its own (`**Disposition (2026-10-31, EP-6): rolled into EP-6** …`).
- **Risks sections** in seven READMEs: `## Risks / open items` with numbered items (five) and
  `## Risks (struck through when resolved)` or `## Risks / open items` with bullets (two). An item is
  resolved by a strike-through at its start and a bold `**Resolved by EP-n (date)**` after it, and
  partly resolved when only a clause is struck (`Resolver traps: ~~…~~ **Resolved by EP-1** …;
  pygam …`); one list carries ids, `- **R-n** …`, with the strike around the id (`~~**R-3** …~~`);
  one numbered list nests sub-bullets of open items under a struck item.
- **Open items for the owner** (one project, in maintenance): a paragraph opening with those words,
  then a bullet list; gates are words and dates at an item's start (`Now:`, `On or after
  2026-11-02, once …:`, `At the next logon:`, `Optional:`); done items are deleted, not struck.
- **Owner actions outside any brief** (one project): one bold-labelled paragraph above the first
  phase whose actions follow the colon, separated by semicolons, each with its decision references
  and sometimes a gate in parentheses (`(D-25, before EP-21)`).
- **`owner-decisions.md`** (one project): a summary table `| # | Ruling needed | Raised by | Class |
  Ruling lands in | Done |` of 18 `OD-n` rows, Done cells `☐ **open — owner only**`, `☐ pushed …;
  **not yet observed** …` (open) and `☑ date — …` (ruled), `### OD-n` sections and dated
  resolutions below; the `pickup-gate.md` beside it is already a fact (EP-4).
- **`open-questions.md`** (one project): `| ID | Item | Owner | Latest responsible decision point |
  Effect if unresolved |`, 9 `OQ-x` rows with letter ids; a closed row says so in words
  (`**Resolved 2026-09-02.**`, `**Closed 2026-09-03 …**`, `**Decided 2026-09-02 at EP-8a: …**`, a
  decision point `— (closed)`). This repository's packet fixture writes `| # | Question | Owner |
  Decision point |` with numeric ids.
- **Handoff notes** (one project): `## Risks / notes for the next session` bullets in its 12
  `docs/handoffs/EP-N.md` files, beside `## Next eligible EPs` (information, not a to-do); another
  project's briefs carry `## Risks, rollback, stop condition`, a planning section, not handoff notes.
- **Blocks in briefs** (EP-4 reads them): `> **Handed on.**` 10, `> **Parked and handed on.**` 4,
  `> **Parked.**` 7 and `> **Parked at execution (date).**` 1, each written by the session that
  completed the brief, for the sessions after it; the only `(to …)` block is this repository's EP-0
  `> **Handed on (to the owner).**`, so no project has yet handed a message on to another in the
  D-13 form.

## In scope

1. **Checklist files** (`core::checklist`): the line form both files share, `- [ ] XX-n (…) text`,
   read from `roadmap::markdown`'s list items with their byte offsets (no pulldown-cmark, as EP-3's
   note foresaw): checked or not, the id, the parenthesis, the text with its continuation lines, the
   line; a line that opens with a box but has no id of the file's prefix, and an id written twice,
   are problems with their line numbers. A tick changes the one byte inside `[ ]` (and back, for an
   owner task) and may append text at the end of the item's last line; every other byte stays (line
   endings, a byte-order mark, the rest of the file). Before the write, the file as it was is copied
   to the tool's backups folder (`<backups>\<project folder>\<name>.<date>.md`, never overwritten);
   the new text is written atomically (D-11, D-29). An unknown or doubled id, a line already in the
   asked state, or an empty disposition writes nothing.
2. **Owner tasks** (`core::tasks`): `roadmap/owner-tasks.md` lines `- [ ] OT-n (EP-k, date) text
   [before EP-m]` → a `Task` (id, done, the EP that raised it, date, text, the `before EP-m` gate,
   line); tick and untick.
3. **Inbox** (`core::inbox`): `roadmap/inbox.md` lines `- [ ] IN-n (from <sender> [EP-k], date) text
   [urgent]` → a `Message` (id, done, sender, the sender's EP, date, text, urgent, the disposition
   when one is written, line); tick with a disposition, required, appended as ` **Disposition
   (date):** text`.
4. **Handed on** (`core::inbox::outgoing`): every `> **Handed on (to <target>).**` block of every
   brief, with its brief, line and text, the target resolved against the registry's names (case
   aside) to a registered project, or unknown (shown, not delivered, D-13); a block to the owner is
   an owner item, not mail.
5. **The older forms, read-only** (`core::tasks::legacy`, D-10): a README's `## Risks …` and `## Open
   items …` sections (numbered or bulleted, `**R-n**` ids), its `Open items for the owner` list
   (`On or after <date>` gates), its `**Owner actions outside any brief**` paragraph split at
   top-level semicolons (`before EP-n` gates); `owner-decisions.md` and `open-questions.md` rows by
   column name (`OD-n`, `OQ-x`; done by `☑`, or by `Resolved`, `Closed`, `Decided`, `(closed)`;
   the decision point as a gate); the `Risks / notes` bullets of the latest completed EP's handoff
   file; and the `Parked`, `Handed on` and `Handed on (to the owner)` blocks of the latest completed
   brief. One `OwnerItem` for every form: form, file and line, id, text, gate (`before EP-m`, `on or
   after` a date, or a decision point in words), state (open, partly resolved, resolved: a
   strike-through at the start resolves, one inside only partly), raised-by EP and date; tickable
   only when it comes from `owner-tasks.md`.
6. **Snapshot** (EP-4's): owner tasks, inbox and the older forms read with the rest of the working
   tree (file reads only, so `crw scan` stays cheap); `Snapshot::owner_items()`; the latest
   completed brief chosen by its completion date, then table order.
7. **`crw tasks <project|folder>`** prints the owner tasks, the inbox, the handed-on blocks with
   their recipients and the open items of the older forms (the resolved ones counted);
   **`crw tick <project|folder> <OT-n|IN-n> [disposition]`** ticks one line (a disposition
   required for `IN-n`) and **`crw untick <project|folder> <OT-n>`** clears an owner task's tick;
   both print where the copy went. `CRW_HOME` keeps the copies in a scratch folder.
8. **Fixtures and tests:** the four EP-4 fixture folders grow the new files, synthetic names only
   (D-3): `house` an `owner-tasks.md` (open, done, gated, wrapped, one line without an id), an
   `inbox.md` (one-line, wrapped, urgent, ticked with a disposition), an owner-actions paragraph, a
   `## Risks` list with `**R-n**` ids, and handed-on and parked blocks in its latest completed
   brief; `house5` a numbered risks list (resolved with open sub-items, open, partly resolved) and
   a maintenance section with open items; `prewritten` an `owner-decisions.md`; `packet` more rows
   in its `open-questions.md` and handoff notes. Unit tests per form; tick round trips on scratch
   copies (CRLF, a byte-order mark, a wrapped item, a missing, doubled or already-ticked id, the
   copy in the backups folder, an untouched file after a refused tick).

## Out of scope

- Delivering handed-on blocks to the receiver's inbox, the delivery log, composing messages,
  `urgent` notifications and the next free `IN-n` (EP-20); the dashboard's tiles (EP-6); the launch
  prompt's inbox and tasks section (EP-10); creating `owner-tasks.md` or `inbox.md` in a project
  (EP-21); appending owner tasks from the chat (EP-24).
- Writing any legacy form, any roadmap README, brief or decision log (D-10, D-11); migrating a
  project's legacy items into `owner-tasks.md` (D-10 chose not to).
- `> **Owner decisions (…).**` blocks (decisions already taken, not to-dos), `## Next eligible EPs`
  sections (information for the next session), `pickup-gate.md` (EP-4's fact) and brief sections
  such as `## Risks, rollback, stop condition` (planning, not hand-offs).
- Registry rows mirroring the inbox (D-13's mirror is the phone's, EP-30).

## Verification / acceptance

- The gate passes with no warnings: `cargo test`, `cargo clippy --all-targets --features screenshot`,
  `cargo fmt --check`, `cargo build --release`.
- Unit tests cover every surveyed form: owner-task and inbox lines (one-line, wrapped, gated, urgent,
  with each disposition form), each legacy form with its states and gates, the handed-on resolution;
  the tick tests show the only changed bytes are the box (and the appended disposition), and that a
  refused tick leaves the file and the backups folder as they were.
- The four fixture folders read to their expected owner items, inbox, outgoing blocks and problems.
- `crw tasks` on every project folder under the real data root, with `CRW_HOME` on a scratch folder
  (nothing written anywhere): risk, open-item, owner-action, register and inbox counts equal `grep`'s
  and the survey's, and the resolved and open states match a read of the files. `crw tick` and
  `crw untick` exercised only on a scratch copy of a fixture folder with `CRW_HOME` on a scratch
  folder: the box changes, the copy appears under the scratch backups folder, nothing else moves.
- No private project name in the tracked files (`git grep` for the names kept in `private/`).

> **Completion note (2026-11-01; opus/max; ≈ 30 min to the owner's questions).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`; the inbox held nothing open. Owner decisions
> (`AskUserQuestion`, every one the recommended option): **commit**: both commits in the house pattern,
> pushed to the private remote; **tick copy**: the copy goes to the tool's backups folder
> (`<backups>\<project folder>\<name>.<date>.md`), not to the `.bak` beside the file the roadmap's note
> named, so a tick leaves no new file in the project (D-11); **addenda**: the D-10 and D-13 addenda
> recorded as written; **CLI ticks**: `crw tick` and `crw untick` kept, beyond the note's scope. Routine
> choices made by the session: one `checklist` module for the line form both files share, read from
> `roadmap::markdown`'s items (no pulldown-cmark), an item's first line sliced at its content column
> because `dedent` keeps a list marker; ids matched in any case, a doubled id refused rather than
> guessed; the disposition appended at the end of the item's last line, before its trailing spaces and
> its line break, line breaks in it folded into spaces; the message text kept as written but for its
> disposition, `[urgent]` read from it; the sender is the parenthesis up to its first comma or EP id; a
> target the registry does not know whole is tried by its first word (`beta EP-6` → `beta`); the older
> forms' state by strike-through only, so a struck item keeps the open items nested under it inside its
> text (one surveyed list does that: it reads resolved with them inside); gates only where a form
> defines one (`before EP-m`, `On or after <date>`, a register's decision point); register rows only
> from tables whose first column is `#` or `ID`; the latest completed brief by its completion date (the
> marker's, a done status's, a filled payload's or the handoff file's), then table position; checklist
> problems kept on `Tasks` and `Inbox`, not in `Snapshot::problems`, so a malformed line does not read
> as an unreadable file; `crw tasks` lists the older forms' open items and counts the resolved ones.
> Verified: `cargo test` 114 passed (21 new: 14 unit, 7 over the fixture folders and scratch copies),
> clippy with `-D warnings` and `cargo fmt --check` clean, the release build in 8 s through
> `install-shortcut.ps1`, which refreshed both shortcuts; `crw tasks` on all 13 project folders under
> the data root with `CRW_HOME` on a scratch folder that stayed empty: the risk items equal an `awk`
> count of each Risks section in the seven READMEs (14, 10, 7, 6, 18, 15, 14), those struck at the
> start read resolved (3, 0, 2, 2, 3, 2, 0) and four read partly resolved in one list; the
> maintenance project's 5 open items with their two `On or after` gates; one project's 5 owner actions
> with their `before EP-21` and `before EP-30` gates; 18 `OD-n` rows with the 2 open ones; 9 `OQ-x`
> rows with the 3 closed ones; 6 handoff notes from the latest handoff file; one `Parked and handed on`
> block from one project's latest brief; 4 inbox messages in two files, 1 open, every ticked one with
> its disposition; no `(to <project>)` block anywhere yet, as the survey found. `crw tick` and
> `crw untick` ran only on a scratch copy of the house fixture: the box alone changed, the untick
> restored the file's bytes, the disposition landed on its line, four refusals wrote nothing, three
> copies sat under the scratch backups folder and no other file appeared in the project's `roadmap\`;
> `crw tasks clauderoadmapwrangler` and `crw scan` on the owner's registry (reads only): 15 registered,
> no proposals. No private project name in the tracked files. No owner tasks; no messages for other
> projects. Next: EP-6 (the dashboard shell), with EP-5's pickup notes in its brief notes and in those
> of EP-10, EP-20 and EP-21.
