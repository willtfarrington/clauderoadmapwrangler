---
ep: 33
title: Per-project workspace files
size: S
model: claude-opus-5-5
effort: xhigh
status: done
---

# EP-33 — Per-project workspace files

**Size:** S · **Depends on:** EP-2 · **Blocks:** EP-11 (Visual launch opens the project's own
workspace file and writes a missing one through this brief's writer)

## Context

Since 2026-10-31 the owner opens each project from its own single-folder `<name>.code-workspace`
beside the project folders (IN-1); the aggregate file, which lists every folder, stays for
browsing and is what the registry's `workspace_file` names. Adopt (EP-2, D-7) only appends to the
aggregate; a registry that does not exist yet names a workspace file only when the data root holds
exactly one, which no longer happens (it holds seventeen); and `crw scan` says only whether the
aggregate lists a folder. Separately, a proposed state (D-6) can only be confirmed, so a project
the owner keeps active against a proposal shows the bulb for good. The EP-8 addenda to D-6 and D-7
decided all four changes; EP-11 needs the first.

**Session start (2026-11-02, 13:13).** Started from `DATA\clauderoadmapwrangler.code-workspace`.
The owner ticked OT-4 (the look at the launch dialog) before the session, uncommitted; it rides
with this brief's commit, as earlier ticks did. No inbox message is open, so nothing is triaged.

**Verified at pickup.**

- The owner's per-project form (this project's own file): tabs, LF, a final newline; one
  `folders` entry with `name` and `path` both the folder's name, then `"settings": {}`.
- `discovery::adopt` appends to the aggregate first, then registers, so a failure in between
  leaves a folder listed but not registered; `workspace::append` checks that its result reads
  back. `registry::only_workspace_file` wants exactly one `.code-workspace` in the data root.
- A screenshot run's data root is the tracked `crates/app/fixtures/portfolio`, which a run never
  writes into, and its registry names no aggregate; `adopt=<folder>` adopts through the window's
  own action, so a file written beside the folder would land in the fixtures.
- The proposals (`proposal::propose`) come from three facts: every row checked, a pickup gate on
  the next brief (both from the parse, `Snapshot::fill`), and the last commit (git). Confirming is
  `Set <state>` in the Detail pane's proposal frame and the row's menu, whose State submenu sets
  the state chip; `crw scan` prints the proposals and the `crw set` line that confirms one.

## In scope

1. **Adopt writes the folder's own workspace file** (D-7 addendum): `<folder>.code-workspace`
   beside the adopted folder, in the owner's form, created new so that an existing file of that
   name is never overwritten; then the aggregate as before, then the registry. The aggregate's
   new text is worked out first, so a file Adopt cannot read stops it before anything is written.
   The writer is public, for EP-11. A screenshot run writes no file beside the folder. `crw adopt`
   and the window's notice say what happened to the file: written, kept, or not written.
2. **A fresh registry's `workspace_file`** (D-7 addendum): the one `.code-workspace` of the data
   root that lists more than one folder, else none; files that do not read are passed over.
   `crw scan` says which file a default registry chose, or why none is set, with the `crw setting
   workspace_file` line when a candidate exists.
3. **`crw scan` lists the own files:** each registered project's and unadopted folder's line says
   whether its `<name>.code-workspace` exists beside it, next to the aggregate's column, and a line
   under the workspace file counts them.
4. **Dismissing a proposal** (D-6 addendum): each proposal carries the fact it rests on (the
   roadmap's row count for every row checked, the next brief's id for a pickup gate, the last
   commit's time for no commit in 60 days); the project's `dismissed` key in the registry keeps,
   per kind, the fact of the proposal dismissed, read strictly like every other key and replaced
   by a newer dismissal of the same kind; a dismissed proposal stays out of the row's bulb and the
   Detail pane's frames while its fact is the one kept, and returns when it changes. Dismiss from
   the row's menu (beside State) and from the proposal's frame in the Detail pane; `crw dismiss
   <project> [<proposal>]`; `crw scan` lists the dismissed proposals apart. An undo: Restore beside
   a dismissed proposal in the Detail pane and `crw undismiss <project> [<proposal>]`.
5. **Owner decisions** through `AskUserQuestion` (rule 7): the addenda, the edits, the commit.

## Out of scope

- Writing the own files of projects registered before this brief: every project has one (EP-8),
  and EP-11 writes a missing one at launch.
- Choosing or moving the aggregate of an existing registry: `workspace_file` stays the owner's
  setting (`crw setting`).
- Proposals that expire, and a list of every dismissal across projects.
- The own workspace file in the window beyond Adopt's notice (EP-11's dialog opens it).
- Any change to the owner's registry or workspace files: tests in temp folders with synthetic
  names, trials with `CRW_HOME` on a scratch data root (rule 13); `crw scan` on the owner's
  registry only reads.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`, `cargo
  build --release` with the app closed, then the install script.
- Unit tests: the own file's text reads back and matches the owner's form; an existing file is
  kept byte for byte; an unreadable aggregate stops Adopt with nothing written; the aggregate rule
  (none, one, several, unreadable, single-folder files only); each proposal's fact, its dismissal
  written into the registry in place and read back, the proposal's return when the fact changes,
  bad values refused with their line, Restore.
- A `CRW_HOME` trial on a scratch data root with synthetic names: `crw scan` on a default registry,
  `crw adopt` writing the own file and keeping an existing one, `crw dismiss` and `crw undismiss`;
  `crw scan` on the owner's registry, which only reads.
- Screenshot runs: `select=quarry` with its proposal, then again after `crw dismiss quarry` on the
  run's own `CRW_HOME` (the frame gone, the dismissed line with Restore), each picture looked at.
- The commit's added lines pass rule 3's word check; the owner has reviewed every decision.

> **Completion note (2026-11-02; opus/xhigh; ≈ 40 min, about 33 active).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`, from 13:15: the reading, this brief, the code and
> its tests, the trial, the pictures, the records, then the owner's one round, the only wait over
> five minutes (7 min). Against S ≈ 30 min. The model and effort are the extension's selection as
> the model catalog cached it at the session's start.
>
> **What was built.** `workspace::{own_file, single_folder, aggregates}` (the own file's place and
> its text in the owner's form; the data root's files that list more than one folder);
> `files::create_new` (a new file, never over one); `discovery::write_own_file`, which EP-11 reuses,
> and Adopt in a new order: the aggregate's new text worked out first, then the own files
> (`OwnFile::Written`, `Kept`, or `Skipped` in a screenshot run through `OwnFiles::Skip`), then the
> aggregate, then the registry; `Folder::own_workspace` and `discovery::facts_of`.
> `proposal::Reason` carries its fact; `Kind`, `Dismissed` and `Reason::returns_when` are new, and
> `propose` marks the dismissed; `Snapshot::fill` gives the row count and the next EP. The
> registry's `dismissed` key on `Project` with `Changes::dismiss` and `undismiss` (the tool's
> inline form laid out afresh, a hand-written `[project.dismissed]` table kept as written); a
> fresh registry's `workspace_file` from `workspace::aggregates`, in place of
> `only_workspace_file`. `crw scan` says why no workspace file is set and which setting would set
> one, counts the own files, shows `aggregate` and `own-file` columns, tags each proposal with its
> kind and lists the dismissed apart; `crw adopt` reports the own file; `crw dismiss` and `crw
> undismiss`. In the window: Dismiss in the row's menu and in the proposal frame, the dismissed
> line with Restore, the bulb for open proposals only, Adopt's notice and the unadopted folder's
> text.
>
> **Verification.** `cargo test` 226 passed (27 + 165 + 2 + 7 + 6 + 11 + 8), 228 with
> `--features screenshot`; both clippy runs and `cargo fmt --check` clean; `cargo build --release`
> with the app closed, then the install script. New tests: the own file's place, text and
> read-back; the aggregate rule (single-folder files, an unreadable file, a folder of that name,
> one, two); `create_new` never replacing a file; Adopt keeping the owner's file byte for byte and
> writing a missing one, writing none in a screenshot run, and stopped before any write by an
> aggregate it cannot read; the facts' row count and next EP; each proposal's fact, the dismissed
> marked and returning once the fact changes, the kinds by key; the `dismissed` key written in
> place with its comment, replaced, taken back to nothing, refused when malformed (an id, a
> negative count, a date without a time, an unknown key), and a hand-written table kept; the fake
> portfolio's quarry losing its bulb once dismissed. A `CRW_HOME` trial on a scratch data root with
> synthetic names: `crw scan` naming the one aggregate and its `crw setting` line; `crw adopt`
> keeping a hand-written own file and writing a missing one; `crw dismiss` by name, for every
> proposal, and refusing an unknown name; the dismissed listed apart; `crw undismiss`; a pickup
> gate returning once the next brief moved on. Read only, on real data: the owner's registry
> (every registered project has its own file) and a default registry over the real data root,
> which chose the one aggregate among the root's workspace files and wrote nothing. Screenshot
> runs, each looked at: `select=quarry` at 1600 × 1000 light (the frame with Set paused and
> Dismiss); after `crw dismiss quarry` on the run's home, at 1100 × 760 dark (no bulb, the
> dismissed line with Restore); `adopt=sketchbook` (registered, no file in the fixtures, the
> notice saying so). `crw roadmap .` reads this brief's front matter and 30 decisions with 31
> addenda, no problem. Rule 3's word list found nothing in the added lines or the new brief.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-02).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The D-6 addendum and the registry form of a dismissal | Accept as written | D-6 addendum; `dismissed = { … }` with its comment |
> | 2 | The undo beyond the brief note (Restore, `crw undismiss`) | Keep it | Shipped, a scope addition |
> | 3 | README, roadmap and v2-notes edits | Keep as written | As drafted |
> | 4 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).** (1) The screenshot setup's `dismiss=` flag in
> the first draft of this brief gave way to `crw dismiss` on the run's own `CRW_HOME`, which also
> tries the console's write into the window's reading. (2) The fact behind "every row is checked"
> is the row count, so a re-plan that removed one row and added another would keep the dismissal:
> unlikely, and harmless. (3) The tool's inline `dismissed` table is laid out afresh at each change
> (`InlineTable::fmt`), since a removed member takes the space before `}` with it; a table the
> owner writes by hand keeps its layout. (4) `crw dismiss` asks git for the project's last commit
> as `crw scan` does, so its facts are the window's. (5) Builds older than this one refuse a
> registry holding `dismissed` (strict reading, D-8); the release exe and the shortcuts are
> rebuilt, and the format stays 1, since nothing needs migrating. (6) `crw scan`'s column for the
> aggregate now reads `aggregate` (it read `workspace`), beside the new `own-file`. (7) Adopt looks
> only at the own file's name: a file there in any form is kept, even one that does not read. (8)
> No owner task: the screenshot runs show both states, and Dismiss appears only where a proposal
> waits.
>
> No messages for other projects. **Next: EP-11** (Visual launch).
