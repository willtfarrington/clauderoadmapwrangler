---
ep: 15b
title: Revert a session
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-15b — Revert a session

**Size:** M · **Depends on:** EP-13 · **Blocks:** EP-15c (Revert in the window), EP-16

## Context

On 2026-11-03 the owner reverted a run's work by hand after the window vanished (EP-14a). This
brief gives the tool a reliable way to put a project back as it was when a session started.
Claude Code's checkpoints (`/rewind`) cannot do it: they track only the edits made by its file
tools and are swept after about 30 days. The revert uses git, back to a start the tool records.

**Session start (2026-11-04, 00:32 UTC).** Started from `DATA\clauderoadmapwrangler.code-workspace`
in the VS Code extension. The tree is clean. No inbox message is open. OT-10 and OT-11 are open,
both gated before EP-16. The model is Opus 5.5 and the environment gives the effort as max.

**The owner's first answers (2026-11-04).**

- **The split (D-4).** The brief notes hold about two M of work, so the brief is split at pickup.
  This brief records the start, builds the revert in `crw_core` and adds `crw revert`. A new M
  row, EP-15c, puts Revert in the window: an ended session's tab, the Sessions pane's ledger
  rows and a project's row menu, with the preview as a dialog run off the UI thread.
- **Every project, with an off switch.** Starts are recorded in every registered project, the
  data repositories included. `crw set <project> revert no` turns recording and Revert off for
  one project, and an owner task asks the owner to do that on the data repositories. A start
  copies at most 200 files and 20 MB. Above that it records only the list, and a revert leaves
  those files as they are.

## In scope

1. **The start** (`revert::start`; the owner's choice at the brief notes).
   - What it records: the project's HEAD, its branch, and every file that is uncommitted and
     not ignored, each as changed, untracked or deleted. The files are listed by one
     `git status --untracked-files=all`.
   - The copies: the changed and untracked files go to the backups folder, at
     `<backups>\<project folder>\starts\<session id>\`. Nothing is written into the project.
     The cap is all or nothing: 200 files and 20 MB.
   - A headless launch records it in the runner (`Launch::starts`), just before the child
     spawns, under the turn the start already holds (D-29). A resume records none: a resumed
     session keeps its first run's start.
   - A visual or manual session records it at its start hook, through a worker thread of the
     window's (`watch`). The window keeps the start until the session earns its row (EP-13)
     and writes it with that row. A start that comes after the row is written into the row,
     and only if the row has none. A session that ends without working drops its start and the
     start's copies.
   - The ledger keeps it as JSON in a new `session.start` column (schema 4).
2. **The target.** The recorded start's HEAD. A session with no recorded start (no window ran
   at its start, or it predates this brief) is reverted to a derived target: the newest
   `docs(roadmap): record EP-n commit hash` commit made before the session started. The
   preview then says that changes left uncommitted before the session cannot be told from the
   session's own.
3. **The preview** (`revert::plan`), read with one git child at a time, the turn taken for
   each. It names:
   - the target, recorded or derived, and the branch;
   - each commit undone;
   - each file changed, created or deleted since the target, with its fate: undone, put back
     as at the start, removed, or kept as it is;
   - what stays: the ignored files, and the start's own uncommitted files;
   - the later sessions recorded in the project since the start, whose work is undone too;
   - an idle session still open there;
   - the tool's ticks to redo;
   - where the copy goes.
4. **Nothing lost** (`revert::revert`).
   - First the work is copied out to `<backups>\<project folder>\reverts\<stamp>\`:
     - `commits.bundle`, the commits undone (`git bundle create`);
     - `uncommitted.patch`, `git diff --binary` against HEAD, written by git;
     - `files\`, every untracked file the revert removes or overwrites;
     - `kept\`, the files it keeps as they are;
     - `revert.txt`, which says what each part holds.
   - Then git and the files, one child at a time:
     1. `git reset --hard <target>`;
     2. the files the session created are removed (never an ignored file: no `git clean`),
        with their parent folders once those are empty;
     3. the start's files are put back from its copies, and the start's deleted files are
        deleted again;
     4. the kept files are written back;
     5. the tool's ticks are redone.
   - An `event` row of kind `revert` records it.
5. **Refusals.** The revert is refused:
   - when the project's switch is off;
   - when a session works or waits in the project (for `crw`: what `claude agents --json`
     lists; when that listing does not answer, the revert is refused too);
   - when the branch changed or HEAD is detached;
   - when the target is not an ancestor of HEAD, or its commit is gone;
   - when the remote has one of the commits, read from the remote-tracking refs without a fetch
     (D-24 addendum); the tool never force-pushes (v2 notes);
   - when a merge or a rebase is in progress;
   - when there is nothing to undo.
   The window's own refusals (its runs, the hooks' sessions, a screenshot run) are EP-15c's.
6. **The tool's own writes** (the brief's choice: redone). Each tick the window or
   `crw tick`/`untick` makes is logged as an `event` row of kind `tick`. A revert redoes, oldest
   first, the ticks made since the start. A tick it cannot redo is named, for example an item
   that the session itself had added.
7. **`crw`.**
   - `crw revert <project> [<session id>] [--yes]` previews the project's latest session, or the
     session named by its id or the id's first eight characters. It reverts only with `--yes`.
   - `crw set <project> revert yes|no` is the switch. Yes, the default, takes the key out, so
     that older builds keep reading the registry.
   - `crw sessions` prints each session's short id.
8. **Records.**
   - A D-11 addendum: the revert is the tool's first write into a project beyond the files it
     owns. It runs only on the owner's word after the preview, never by the autopilot or a
     rule, never while a session works there, and it never pushes.
   - A D-26 addendum: the start in the ledger (schema 4), and the `revert` and `tick` events.
   - The README's commands; EP-15c's row and brief note; the plan's totals; the owner tasks;
     the completion note.

## Out of scope

- The window's Revert: the buttons, the menu item, the dialog and the window's own refusals
  (EP-15c).
- Undoing a commit the remote has, by `git revert` or by a force-push (v2 notes).
- Restoring the start's index exactly: changes staged at the start come back unstaged.
- Starts for sessions that run while no window listens; the derived target covers them.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`,
  `cargo build --release` with the window quit from the tray, then the install script.
- Unit and integration tests, on scratch repositories in temp folders and never the owner's
  projects (rule 13):
  - a clean start and a dirty one, the copies, the cap;
  - the ledger at schema 4, an older ledger read, and a start set once;
  - a revert of a session that committed, edited a file left dirty at the start, created,
    deleted and ignored files: the copy-out, HEAD back at the start, the start's file as it
    was, the new file gone, the ignored file kept;
  - each refusal (a branch changed, a start not an ancestor, a pushed commit to a scratch bare
    remote, nothing to undo);
  - the derived target;
  - ticks redone;
  - a headless run recording its start (the stub, never the CLI);
  - the window's start kept until the row;
  - `crw revert` previewing on a scratch home.
- A trial with `CRW_HOME` on a scratch project with a synthetic name: `crw revert` previews,
  then reverts with `--yes`.
- Rule 3's word check prints nothing on the staged diff.

> **Completion note (2026-11-04; claude-opus-5-5/max; about 1 h 5 min of wall time with the
> owner's answers).** A session's start is recorded in the ledger, and `crw revert` puts a project back as
> it was then. The revert copies the work out first, never pushes, and redoes the tool's own
> ticks. The window's Revert is EP-15c, split off at pickup.
>
> **Built.**
>
> - `revert::start`: `Start` (HEAD, branch, the uncommitted files, where their copies are) and
>   `capture`. One `git status --untracked-files=all` lists the files (`git::status_all`). The
>   changed and untracked ones are copied to `<backups>\<project folder>\starts\<session id>\`,
>   all or none at 200 files and 20 MB.
> - `revert`:
>   - `choose` finds the session by its id, its first eight characters, or the latest.
>   - `plan` reads the target, the commits, each file's fate (undone, removed, put back, deleted
>     again, kept) and the refusals. `Plan::lines` is the preview.
>   - `revert` copies the work out (`commits.bundle`, `uncommitted.patch`, `files\`, `kept\`,
>     `revert.txt`), runs `git reset --hard`, removes the session's untracked files and their
>     emptied folders, puts the start's files back, writes the kept files back, redoes the ticks
>     and logs a `revert` event. `Done::lines` is the result.
>   - `log_tick` and `ticks_since`, the `tick` events.
> - The ledger, schema 4: `session.start`, `set_start`, `set_start_of` (once, into the first row)
>   and `start_of`.
> - The runner records a headless run's start just before the child spawns, under the turn the
>   start holds (`Launch::starts`). A resume records none.
> - The window: `watch` reads a VS Code or terminal session's start at its start hook, on a worker
>   thread, keeps it until the session earns its row, and drops it with its copies when the session
>   ends without working. `Registered::revert` carries the switch. The board sets `Launch::starts`
>   for a headless launch and logs its ticks.
> - The registry: `revert`, true by default (`set_revert`; on takes the key out).
> - `crw`: `revert <project> [<id>] [--yes]`, which asks `claude agents --json` for the sessions
>   at work there; `set <project> revert yes|no`; the short id in `sessions`; `tick` and `untick`
>   log their ticks.
>
> **Verification.**
>
> - `cargo test`: 348 tests; with the screenshot feature, 350. Of these, 14 are new. Both clippy
>   runs and `cargo fmt --check` are clean.
> - The scratch-repository tests revert a session to its recorded start (two commits, the owner's
>   edited draft, new, deleted and ignored files, a tick redone), to a derived target, and from a
>   start above the cap (its files kept). They check each refusal: the switch, a session at work,
>   a commit pushed to a scratch bare remote, a changed branch, a rewritten history, and nothing to
>   undo.
> - The `CRW_HOME` trial on a scratch project named `fernhill`: `crw sessions` printed the short
>   id; `crw revert fernhill` previewed one commit and four files; `--yes` reverted them, kept the
>   ignored file and redid OT-2. The bundle fetched the commit back onto a branch, and the patch
>   applied on it. The trial ran `claude agents --json` (a listing; no usage) for the live
>   sessions.
> - The trial showed "1 commits"; the result's words now count properly.
> - The release build and the install script ran; `crw roadmap .` reports 0 problems.
>
> **Owner decisions.**
>
> | Decision | Outcome |
> | --- | --- |
> | The brief's size (D-4) | Split at pickup (recommended): the start, the engine and `crw revert` here; the window's Revert as EP-15c, an M row before EP-16. |
> | Where starts are recorded | Every project, with an off switch (recommended): `crw set <project> revert no`; at most 200 files and 20 MB a start. |
> | The switch on the data repositories | Turned off now by the session (recommended), on the three registered projects the user-level grant leaves out. The registry's daily copy from before the day's first change stands; `revert yes` takes each key out again. OT-12 asks only for the preview. |
> | The records (the D-11 and D-26 addenda, EP-15c's row and note, EP-16's recount, the README, OT-12) | Kept as written (recommended). |
> | Commit and push | Committed and pushed (recommended); the row's hash is recorded in a commit of its own. |
>
> **Routine choices.**
>
> - The tool's own writes are redone, not just named: each tick is logged as a `tick` event, and a
>   revert redoes those since the start, oldest first. One it cannot redo is named.
> - The derived target is the newest `record EP-n commit hash` commit made before the session's
>   first row, so that a session that recorded its own hash is not its own target.
> - A start's file already as it was then is left out of the preview. The revert writes every
>   start file back after the reset whatever the preview listed.
> - The copies are files at their paths in the backups folder, not an archive (no new crate);
>   `git diff --output` writes the patch, so its bytes are git's.
> - `crw revert` refuses when `claude agents --json` does not answer, since it cannot then tell
>   whether a session works there. The CLI test therefore covers the preview and the refusals,
>   and the revert itself is tested in `crw_core`.
> - The index at the start is not restored: changes staged then come back unstaged.
> - Turning the switch back on takes the `revert` key out, so a registry with no project turned
>   off still reads in older builds.
>
> **For later briefs.** EP-15c's brief note, with its pickup notes from EP-15b; EP-16's note
> (the totals: 39 briefs, 9 S · 24 M · 6 L ≈ 40.5 session-hours, 19 open); OT-12 before EP-15c.
>
> **Handed on.** Nothing. **Next:** EP-15c.
