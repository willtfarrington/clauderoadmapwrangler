---
ep: 25
title: History backfill and snapshots
size: M
model: claude-fable-5-1
effort: high
status: done
---

# EP-25 — History backfill and snapshots

**Size:** M · **Depends on:** EP-2, EP-4 · **Blocks:** EP-26, EP-27 · **Decisions:** D-26, D-8

## Context

D-26 settles that git already holds the dates a project's history graphs need: the commits whose
subjects record an EP give completion dates, the briefs' completion notes cross-check them, and the
ledger keeps a project's done and total counts over time. The `snapshot` table has waited in the
ledger since EP-2 with no writer. This brief builds `core::history`, the import that reads a
repository's log once per project and keeps what it finds in the ledger, with Re-import from the
Detail pane and `crw history`; and the daily parse snapshots. EP-26 draws the graphs from both.

**Session start (2026-11-11).** The session's prompt was this project's opener, launched from
clauderoadmapwrangler with fable at high effort; `crw sessions clauderoadmapwrangler` lists its row
(`e9774ab2`, headless, EP-25, started 22:15). Read: the roadmap's P4 table and EP-25's brief notes,
D-26 with its addenda, EP-24b's completion note, `owner-tasks.md` (no open task) and the inbox (IN-12
and IN-13 open). The field check: no new `crash.log` (the one present is EP-14a's of 2026-11-03);
`crw notifications` shows the two inbox badges of IN-12 and IN-13 (22:12, 22:14) and the manual
session's waits and ends between 21:44 and 22:11; `crw sessions` shows the chain EP-38 → EP-24b of
this afternoon, each run completed; `crw autopilot` shows the chain ending after EP-24b at the day's
cap of four and six projects on. Nothing odd beyond the inbox. The working tree held one uncommitted
change at the start, the owner's own IN-13 line in `roadmap/inbox.md`, which this session's triage
ticks and the record commit carries.

**The subjects as found.** This repository's log (119 commits) holds the forms EP-23's note lists:
`docs(roadmap): record EP-n commit hash`, with a suffix (`record EP-20b commit hash (and EP-20c's
front matter set to done)`), the review-fix form (`record EP-20c review-fix commit hash`), work
commits ending in `(EP-n, D-x, …)` or `(EP-0, D-1 … D-30)`, one prefix form (`EP-20c review fixes:
…`), and noise that names an EP without recording one (`record EP-8's CI run`, `docs(roadmap): EP-7a,
list and detail polish …`, `record EP-16's first self-launched session …`). Other projects' logs add
`EP-15: …` prefixes and `EP-15: mark done at <hash>` as a record form, and one public project has a
single commit, so its dates can come from its notes alone.

**Inbox triage (D-13).** IN-12 (three findings from the first real chat): none fits a history brief,
all three together are S-sized (two sentences of the digest and a line in D-27); put to the owner as
question 1, roll in or park. IN-13 (a project's state re-read on its own when its session or chain
ends, instead of F5 over every project): an S of window work outside this brief's subject; put to the
owner as question 2, a new S row EP-25a after this one (recommended), rolled in, or parked in EP-28a's
notes. The dispositions are written on the lines once answered.

## In scope

1. **`core::history`.** `log(folder)` runs `git log --format=%H%x09%ct%x09%s` (one child, in its
   turn) and parses it; `classify(subject)` reads an EP marker: a **record** commit (`record EP-n …
   commit hash`, or `EP-n: mark done …`) or a **work** commit (a subject ending in a parenthesis that
   opens `(EP-n` followed by `,` or `)`, or starting `EP-n:` or `EP-n <words>:`); anything else is
   noise. `import(commits, rows, notes, offset)` gives one `Completion` per EP: its record commits,
   its work commits (from the markers and from the row's Done cell's hashes resolved against the
   log), its git date (the earliest record commit's local date, else the latest work commit's), its
   note date (the brief's completion-note date), and the date that stands: the note's when both exist
   and differ by more than a day (flagged `NoteWins`), else git's, else whichever exists (`GitOnly`,
   `NoteOnly`). A history with one commit, or none that carries a marker or a Done hash, uses note
   dates only (`Single`).
2. **The ledger (schema 7).** `completion` rows (project, ep, completed_on, git_date, note_date,
   record_hash, work_hash, flag) replaced per project at each import, and `history_import` (project,
   head, imported_at, commits, flagged) saying what the import read; `import_history` writes both in
   one transaction, `completions(project)` and `history_of(project)` read them. `record_counts` writes
   a `snapshot` row when the project has none, when the counts differ from the latest row's, or when
   the latest row is from an earlier local day; `snapshots(project)` reads them. A ledger opened to
   read at schema 6 reads none of the new tables.
3. **The window.** The files pass records every project's counts (the current reading) after each
   read; the git pass, for each repository whose HEAD differs from its stored import (or has none),
   runs the import and sends its summary; the Detail pane gains a **History** section after Progress
   (completions imported, the first and latest dates, the flagged EPs with both dates, the import's
   time and HEAD, the latest snapshot) with a **Re-import** button (`Action::Reimport`), which runs
   the import again on a worker thread whatever HEAD says; disabled for a folder that is no
   repository and in a fixtures run. The ledger is never touched by a test or a screenshot run
   beyond the scratch one `CRW_HOME` names.
4. **`crw history <project|folder>`**: the import computed from git and the files and printed, no
   write (EP, date, source, hashes, flag; the flagged ones marked), then, for a registered project,
   the ledger's stored import and its snapshot rows, read only; `--reimport` writes the import into
   the ledger (the window's Re-import by hand).
5. **Docs**: a D-26 addendum; the README's `crw` line; the roadmap row linked to this brief; the
   completion note; the inbox dispositions.

## Out of scope

- The graphs (EP-26), the exports (EP-27): this brief gives them `completions`, `snapshots` and
  the import's summary.
- Transcript parsing of any kind (D-26).
- A brief's measured time (EP-26's note from EP-16: the session's start to the record commit).
- The ledger's rows before EP-13 (sessions that never worked): untouched, as the notes say.
- Dates for EPs no commit and no note carry (an open row): none.

## Verification / acceptance

1. `cargo test --workspace` and `cargo test --features screenshot` pass; both clippy runs and
   `cargo fmt --check` are clean; rule 3's word check on the staged diff prints nothing.
2. Core tests: `classify` on every subject form above and the noise; `import` on a synthetic log
   (a record and a work commit, a review-fix record, a two-hash Done cell, a planning commit with no
   brief, a note a day off (git wins) and three days off (the note wins, flagged), a note-only EP, a
   single-commit history); a test over this repository's own log checks the import against the
   retros: EP-1's record commit `74540b4` on 2026-10-31, EP-20c's two record commits with `1282256`
   first, EP-37's work commit `a8078a5`, the planning commit `811d9a6` carrying no EP; ledger tests
   for `import_history` replacing rows, `record_counts` writing once a day and at a change, and a
   schema-6 read-only open reading none.
3. App tests: the Detail pane's History section with a canned summary and its Re-import action; a
   screenshot run with `history` showing the section on a fake project.
4. `crw history clauderoadmapwrangler` prints the import of this repository with its flagged EPs,
   checked by eye against the Done cells.
5. The release build and the install run last (`scripts\install-shortcut.ps1`); the restart is
   refused while this run works and taken at the gate's moment.

**Residual risk.** The import has run on this repository's log and on synthetic ones, not on the
other projects' real logs: a subject form none of these hold (a record commit worded otherwise, a
hash written in a Done cell that the log no longer has after a rewrite) reads as noise or as a
missing work commit, which the History section shows as an EP dated by its note alone, never as a
wrong date; the History section of each real row, and `crw history <project>` over each real
project, are the P4 walk's first item for this brief (no owner task: a look at a changed surface
is the walk's, D-10 addendum). The
daily snapshot and the import on a moved HEAD run in the real window's passes, which no test
drives; their writes are one row per project per day and one `git log` per repository per new
commit, so a fault shows in the ledger's size, not in the window's speed. The History section is
not checked on a copy of the real roadmaps (rule 5): a copy is no repository and has no import,
so the section stays out there by design; the real rows show it once the installed window's git
pass has run.

> **Completion note (2026-11-11; claude-fable-5-1/high; about 50 min of wall time, a headless
> run under the autopilot, two triage questions answered in the Decisions tile at once).**
> Ledger row `e9774ab2`, started 22:15 from the window after the chain ended at the day's cap;
> the prompt was the opener.
>
> **Built.**
>
> 1. **`core::history`** (D-26 addendum): `log`, `parse_log`, `classify` (`Marker::Record` for
>    `record EP-n … commit hash` and `EP-n: mark done`, `Marker::Work` for a trailing `(EP-n,
>    …)` group or an `EP-n:` / `EP-n <words>:` prefix, everything else noise), `import` over the
>    commits, the rows' Done hashes and the briefs' note dates in a `TimeZone` (`Local` in the
>    window and `crw`, so each commit's day is its own), `Completion` with its `Flag` (`Agreed`,
>    `GitOnly`, `NoteOnly`, `NoteWins`), `Import` (`flagged`, `span`, `single`), `Summary`,
>    `read(folder)`, `rows_of`, `notes_of`, `short`.
> 2. **The ledger, schema 7**: `completion` and `history_import`; `import_history` (one
>    immediate transaction replacing the project's rows), `history_of`, `imported_head`,
>    `record_counts` (a `snapshot` row when none, at a change of the counts, or on a later local
>    day) and `snapshots`; a ledger read at schema 6 reads none of the new tables.
> 3. **The window**: `ProjectRow::history`; the files pass records every project's counts; the
>    git pass imports a repository when its `HEAD` differs from the stored import, else reads
>    the stored one (`Message::History`, `import_into`); `Loader::reimport` on a worker thread
>    (`Action::Reimport`, `Message::HistoryFailed` to the status bar); the Detail pane's
>    **History** section after Progress (the dated EPs and their span, the flagged ones in
>    amber with both dates, when and at which `HEAD`, Re-import, disabled for a folder that is
>    no repository and refused in a fixtures run); the screenshot flag `history`.
> 4. **`crw history <project|folder> [--reimport]`**: the import printed (EP, date, source,
>    hashes, `LOOK` on a flagged one), then the ledger's stored import and snapshot rows for a
>    registered project; `--reimport` writes it.
> 5. **IN-12 rolled in**: `ProjectDigest::older_forms` and the digest's wording for a project
>    without `owner-tasks.md`; the Compose sentence saying it creates a missing `inbox.md`; the
>    D-27 line on `permissionMode: "default"` being the CLI's name for `manual` (checked
>    against `claude --help`). **IN-13 placed**: EP-25a, an S row after this one, with its brief
>    note and the recount (62 briefs).
> 6. **Docs**: D-26 and D-27 addenda, the README's paragraph, the roadmap row linked, the
>    `crw` help; tests: seven in `history` (every subject form and the noise, the log's lines,
>    the synthetic import, a single-commit history, the zone, this repository's own log checked
>    against the retros and skipped on a shallow clone), one in `ledger`, one in the digest's
>    test extended, one in `detail` (the section, the disabled and the acting Re-import).
>
> **Checked by hand.** `crw history clauderoadmapwrangler` (the debug build, read only): 119
> commits, 51 EPs dated, 0 flagged, every EP's record and work commits matching its Done cell
> (EP-20c's two records and three work commits among them); no import stored yet, since the
> installed window has not run this build. **The pictures** (`target\tmp\ep25\`, never git):
> `history,select=harbor,tile=detail` at 1600 light and 1200 dark, the section under Progress
> with the flagged EP in amber and Re-import below.
>
> **The review (rule 6).** A subagent given the diff alone, launched once the code compiled,
> reported after nine minutes: one confirmed, four plausible, three nits. Fixed: (1, confirmed)
> the test over this repository's log would fail CI's shallow checkout: it returns on a shallow
> clone, and EP-20c's work list is checked by its first three; (2, plausible) a fixed offset
> frozen at import time moved a commit's day across a clock change: `import` and
> `record_counts` take a `TimeZone`, `Local` resolving each instant; (3, plausible) the
> deferred transaction got `SQLITE_BUSY` at once under another connection's commit: immediate,
> as the migration's; (4, nit) "Imported now ago": "just now"; (6, nit) a Done-cell commit
> already attributed to another EP was never added for this row: the dedupe compares the EP
> too; (7, nit) `crw history --reimport <x>` fell through: the flag is read anywhere among the
> arguments; (8, nit) a zero-commit import read as single-commit: worded apart. Logged, not
> changed: (5) the daily snapshot is written at a read of the files, so a day the window never
> reads them gets no row (the D-26 addendum says so); the first import holds the turn through
> its spacing, about half a second per repository once.
>
> **Verification.** `cargo test --workspace`: 375 in `core`, 162 in `app`, the other suites as
> before; with the feature 170 in `app`; both clippy runs and `cargo fmt --check` clean after
> the review's fixes; rule 3's word check on the staged diff printed nothing. The release build
> and the install (`scripts\install-shortcut.ps1`) ran after the record commit, the opener's
> last step; the restart is refused while this run works and taken at the gate's moment, or
> from File > Install and restart, the tray's, or the launch dialog's. The installed window's
> first git pass imports every repository's history (one `git log` each, once) and writes the
> first snapshot rows.
>
> **Owner decisions** (asked right after the read-first pass, answered at once).
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | IN-12's three chat findings: roll in here, park at EP-28, or at EP-29's walk | Roll into EP-25 (recommended) | Built (item 5); the inbox line ticked |
> | 2 | IN-13's per-project refresh: a new S row EP-25a, roll in here, or park at EP-28a | New S row EP-25a after EP-25 (recommended) | The row and its brief note added; the inbox line ticked |
> | 3 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **A row's Done hashes count as work commits**, so a project whose subjects carry no marker
>    is still dated by git; the brief notes named the cells as a check only.
> 2. **Two new tables rather than JSON in `badge_cache`**: EP-26's queries and EP-27's CSV read
>    rows.
> 3. **The import runs when `HEAD` moves**, not at every git pass: one `git log` per repository
>    per new commit; the stored summary is re-sent at each pass so the row keeps it.
> 4. **Re-import lives in the Detail pane only**, not the row's menu; `crw history --reimport` is
>    its command-line twin.
> 5. **The latest snapshot is not drawn in the pane**: EP-26's burn-up reads the rows; the brief's
>    in-scope line said it would, and this note records the cut.
> 6. **The scratchpad** is `target\tmp\ep25\` (ignored): the pictures, the logs, the diff the
>    review read, the commit messages.
>
> **Owner task:** none. No message for another project. **Next: EP-25a** (per-project
> refresh), chained by the autopilot when its gate allows (the day's cap of four chained runs
> was reached at 21:18, so most likely tomorrow), else from the window.
