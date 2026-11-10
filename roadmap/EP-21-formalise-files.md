---
ep: 21
title: Formalise files
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-21 — Formalise files

**Size:** M · **Depends on:** EP-5, EP-10 · **Blocks:** EP-22, EP-21a

## Context

D-10, D-12 and D-13 give every house-style project the same files: the owner tasks in
`roadmap/owner-tasks.md`, the inbox in `roadmap/inbox.md`, briefs opened with YAML front matter and
closed by a completion note with model, effort and duration; D-11 lets the tool write these
"adoption files" into a project (a `CLAUDE.md` rule block, the two files, the brief template), and
D-9 leaves migrating an older roadmap to a session in that project. Until now a project took the
files by hand. This brief builds the formalise step. It was split at pickup (below).

**What exists.**

- Adopt (`discovery::adopt`) registers a folder and writes its own workspace file and the
  aggregate's entry (EP-33); a folder without a roadmap starts as `unformalised` (D-6).
- `checklist::parse` reads the two checkbox files: every line of work is a top-level
  `- [ ] OT-n (…)` or `- [ ] IN-n (…)` item, and any other box is a problem (EP-5's note), so a
  header's example line stays in backticks.
- `inbox::mailman::append` creates a missing inbox with `mailman::HEADER` (EP-20) under the
  process's inbox lock; `files::create_new` never replaces a file, `files::dated_copy` and
  `files::write_atomic` are the tick's copy-then-write.
- `table::Profile` tells the house, five-column (`house5`) and work-packet (`packet`) tables apart;
  the brief discovery ignores `_TEMPLATE.md` (EP-4).
- `prompt::read_first` quotes the first `Read first` item or paragraph of a project's `CLAUDE.md`
  into the opener, else the house default.
- Since the brief note was written, the house method also lives in the owner's user-level rule
  `~\.claude\rules\house-style.md`, loaded in every session on this machine.

**Session start (2026-11-10, 13:53 local time).** Launched by the window as a **chained headless
run** (rule 11): the chain log says `held after EP-20 for the window's restart into the installed
build: EP-21 starts in the new window`, then `chained EP-21 after EP-20: claude-opus-5-5 at max,
your choice, M's cap of 1 h 30 min`; ledger row `89829f26`, headless, EP-21,
`claude-opus-5-5/max`, open (`crw sessions clauderoadmapwrangler`). The prompt is the window's
opener in its headless wording. The tree is clean at `b65d21e`; the inbox has no open message;
OT-20 and OT-21 are open and not gated.

**Field check.** I read the records back to EP-20's record commit (`b65d21e`):

- `crash.log` was last written on 2026-11-03, read before;
- `crw notifications`: EP-20's finished popup (13:53:08) and the chain's toast (13:53:29);
- `crw autopilot`: the two lines above, the **first real chained restart** (EP-37a's route): the
  gate held the chain, the window restarted into EP-20's installed build and the carried chain
  started this brief within the minute, which is what OT-20 expects of the log;
- `crw mail`: the mailman's **first reading** at 13:53:12 in all 15 registered projects, each
  `0 blocks counted as delivered`, and no delivery. That is right: no committed brief in the
  portfolio holds a `Handed on (to <project>)` block to another project yet (this project's are
  all `to the owner`, and the neighbours read by path have none), so EP-20's residual risk 1 held
  and OT-21's first half is met; its second half waits for a real block.
- `crw sessions`: this run's row; EP-20's row completed; EP-37's first row still open with no end
  (the known gap). Nothing odd to triage.

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-10)

1. **Split in two** (D-4), since the note holds about two M's and a run stopped at its cap leaves
   its work uncommitted:
   - **EP-21** (this brief): the formalise engine, `crw formalise` with its preview and `--yes`,
     and the migration prompt (`crw formalise --migration`);
   - **EP-21a** (M): the window's Formalise… dialog with the preview and Write (the row menu and
     the Detail pane), and Migrate… opening the launch dialog in Headless with the migration
     prompt.
2. **EP-21a goes after EP-22**, after EP-20a and EP-20b and before EP-23, so the chain plan's
   EP-21 → EP-22 runs as set; EP-22's formalise skill needs the engine's texts, not the window.
3. **The rule block is self-contained and short**: the note's list in about a dozen lines (read
   order with the triage rule, the brief from the template with its front matter, the completion
   note's form, `Handed on` blocks, the owner-task and inbox line forms, the house table), so that
   it works in a clone or for another user; a project's own rules outside the block win.

## In scope

1. **The engine** (`crw_core::formalise`, new; the note's `core::adopt`, named for what it does
   since `discovery::adopt` is Adopt).
   - **The files.** The rule block between `<!-- roadmap-wrangler:begin -->` and
     `<!-- roadmap-wrangler:end -->` in the project's `CLAUDE.md` (the root's, else
     `.claude\CLAUDE.md`, as the opener reads it; a new root `CLAUDE.md` when there is none);
     `roadmap/owner-tasks.md` and `roadmap/inbox.md` with their headers (the inbox's is the
     mailman's, one header for both writers); `roadmap/_TEMPLATE.md` (front matter, Context, In
     scope, Out of scope, Verification / acceptance with its residual-risk line, the completion
     note's placeholder).
   - **The plan** (`plan`) says what each file gets: created, the block added at the end of the
     file, the block replaced between the markers, or kept (a file there is never replaced; a
     block as written is left); markers that cannot be read (an end without a begin, two blocks)
     refuse the plan, and nothing outside them is ever touched. It reads files by path only and
     asks git nothing.
   - **The write** (`apply`) reads every file again and refuses when one changed since the plan
     (the revert's rule, EP-15c), creates the new files without replacing anything, and changes
     `CLAUDE.md` after a copy of it to the backups folder; the inbox is written under the
     process's inbox lock, as the mailman's deliveries are.
   - **The migration prompt** (`migration_prompt`) for a `packet` or `house5` roadmap: convert the
     tables to the six-column header, keep ids and hashes, move each handoff's status into its
     brief's completion note, keep every file, check with `crw roadmap` that the next EP and the
     counts stay, record the migration, stop before committing.
2. **`crw formalise <project|folder> [--yes]`** prints the plan (each file and its change, the
   block's text, the profile, a migration hint for an older roadmap); `--yes` writes, refused
   while a session works or waits in the project (`claude agents --json`, as `crw revert`), and
   logs a `formalise` event under the project. **`crw formalise <project|folder> --migration`**
   prints the migration prompt; it launches nothing.
3. **The inbox header** cites `clauderoadmapwrangler D-13`, the form the owner's hand-carried
   inboxes use, since a bare `D-13` names another project's decision there.
4. **The records:** a D-11 addendum (the formalise step's writes) and a D-9 addendum (the
   migration prompt); the roadmap's EP-21 row, the EP-21a row and note, EP-22's and EP-23's
   notes; the README's house-style section and the CLI's usage; an owner task for the first real
   formalise (a new route into another project, D-10 addendum).

## Out of scope

- **EP-21a:** the window's Formalise… dialog and Migrate… launch.
- The chat's delegation of formalise and migrate (EP-24, D-27), and the plugin's formalise skill
  (EP-22), which use this engine.
- Writing a first roadmap README or `DECISIONS.md` (the plan skill's and the formalise skill's,
  EP-22); changing a project's registry state (the owner's, D-6).
- Adopting the owner's environment repository: the owner's own `crw adopt` (IN-2, rule 13).

## Verification / acceptance

- **Unit tests:** the block's text (markers, the read-first item `prompt::read_first` quotes, a
  project's own rule above it winning, 100 columns); the plan of a bare folder (four creations),
  of a formalised one (all kept), of a `CLAUDE.md` without markers (block added, every byte
  before it kept), with an older block (replaced, the text around it kept), with broken markers
  (refused), of `.claude\CLAUDE.md`; the headers parse with no item and no problem; `apply`
  writes the plan, keeps a dated copy of `CLAUDE.md`, refuses a file changed since the plan and
  never replaces a file; CRLF endings kept; the migration prompt for `packet` and `house5`, none
  for `house`.
- **The checks:** `cargo test` with and without `--features screenshot`, both clippy runs, `cargo
  fmt --check`, and a review of the diff by a subagent given only the diff (rule 6).
- **A trial** of `crw formalise` on a scratch folder (`CRW_HOME` and a scratch data root): the
  preview, `--yes`, the preview again showing every file kept; and the preview, read-only, on a
  few real non-data projects.
- **The release:** `cargo build --release` with the window up, then `pwsh -File
  scripts\install-shortcut.ps1` (rule 6); its restart request is refused while this run works.
- **Rule 3's word check** on the staged diff prints nothing.
- **Residual risk.** The session wrote only into scratch folders (rules 4 and 13) and previewed
  four real non-data projects without `--yes`. Three things are not exercised:
  1. **The first real write into a project** (OT-22, a new route into another project, D-10
     addendum). The change to `CLAUDE.md` is the tick's copy-then-atomic-write, used since EP-5,
     and the new files go through `files::create_new`, which Adopt's own workspace file has used
     since EP-33, so a file there is never replaced.
  2. **The migration prompt in a real session.** No session has run it. Its first two steps read
     the project and `crw roadmap` and put the plan to the owner before any edit, and its check
     compares the next EP and the counts before and after, so a wrong step shows before anything
     is committed (the session stops before committing).
  3. **The block's words in the owner's sessions.** In this machine's sessions the block repeats
     part of the user-level house-style rule; a project's own rules outside the block win, and its
     `Read first` item is quoted only where a project has none of its own (tested). The walk weighs
     the wording on the first project formalised.

> **Completion note (2026-11-10; claude-opus-5-5/max; about 45 min of wall time, three questions
> at pickup and two in the closing round).** One chained headless run
> (ledger row `89829f26`, headless, EP-21), started by the window at 13:53 local time after its
> restart into EP-20's installed build: the first brief a chain carried across a restart.
>
> **Built.**
>
> - `crw_core::formalise`: the rule block (`BEGIN`, `END`, `block`), the owner tasks' header
>   (`OWNER_TASKS_HEADER`), the brief template (`TEMPLATE`), the plan (`plan`, `Plan`, `Step`,
>   `Change`, `Readme`, `File`), the write (`apply`, `Applied`, `Written`, `Error`) and the
>   migration prompt (`migration_prompt`); `mailman::HEADER`, now shared by both writers, cites
>   `clauderoadmapwrangler D-13`.
> - `crw formalise <project|folder> [--yes | --migration]`, its usage text.
> - The records: the D-9 and D-11 addenda, the roadmap's EP-21 row, the EP-21a row, its note and
>   the ordering paragraph, EP-22's and EP-23's notes, the README's house-style section, OT-22.
>
> **The review (rule 6).** A subagent given only the diff found eight issues and a design note.
> Fixed and tested:
>
> 1. **The block stood over a project's own read-order section** in the opener's quote:
>    `prompt::read_first` took the first `Read first` item anywhere before looking at headings,
>    so a project whose rule is a `## Read order` section (two of the owner's are) would have had
>    the block quoted. It now reads the text outside the block first, then the block
>    (`formalise::without_block`).
> 2. **A registered folder that moved would have been made again** with the four files in its
>    old place: the plan refuses a folder that is not there.
> 3. **The changed-file test could not catch a check made step by step**: it now proves the rule
>    file and the files before the last step untouched, and names the path.
> 4. **A linked `CLAUDE.md` would have become a copy** holding the block: the plan refuses a link
>    (the test returns early where the shell may not make one).
> 5. **Markers inside a code fence were taken as the block**, and a byte-order mark before the
>    first marker hid it: fenced lines are skipped, and the mark stays outside the block.
> 6. **A file that could not be read showed as kept** above the refusal: it is a refused step with
>    its error, and an error at the second reading is an I/O error, not a change.
> 7. **The printed commands named an unregistered folder by its name**, which resolves only from
>    its parent folder: they name its path.
> 8. **A file appearing during the write** (the mailman's inbox, from the window's process, which
>    the lock does not reach) no longer stops it: the file is kept, as the plan keeps one, and the
>    write goes on.
>
> The design note, a revert that does not redo a formalise (routine choice 6), is parked in
> EP-21a's notes (the owner's choice). Found by the session while the review ran: the preview said
> `the file's 1 lines kept`, now `1 line`; and a console test (`crates/cli/tests/formalise.rs`)
> covers the preview, `--migration` and the flags.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed 510, and with `--features screenshot` 518: EP-20's 492 and
>   500 plus 18 new in each run (15 in `formalise`, 3 console tests), covering the block (the
>   read-first item the opener quotes, a project's own rule item or read-order section winning,
>   100 columns), the headers (no item, no problem), a bare folder (four files, then all kept), a
>   block added with every byte kept and its copy, an older block replaced in a CRLF file, broken
>   markers refused with nothing written, markers in a fence and a byte-order mark, the nested
>   rule file, a linked one refused, a missing folder refused and never made, a file changed or
>   appeared since the plan stopping the write before anything is written, the README's form,
>   the migration prompt for each form, and the console's preview, `--migration` and flags. Both
>   clippy runs and `cargo fmt --check` are clean.
> - **The trial** (`CRW_HOME` and a scratch data root): the preview of a work-packet project with
>   a rule file, `--migration`'s prompt, `--yes` (the session check asked `claude agents --json`,
>   which found none there), then the preview again saying `Nothing to write`, and `crw tasks`
>   reading the new files with no problem. The preview, without `--yes`, of four real non-data
>   projects read as expected: two with the block to add and the template to create, one with a
>   README the parser reads no table in, one with no roadmap yet.
> - **The release.** Built with the window up, again after the review's fixes; the install runs
>   last, after the two commits (rule 6), its request to restart the window refused while this
>   run works, so the window restarts into the new build at the gate's moment or from File >
>   Install and restart.
> - **Rule 3's word check** found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-10).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Splitting EP-21 at pickup (a chained headless run under M's cap) | Split in two | EP-21 the engine, `crw formalise` and the migration prompt; EP-21a (M) the window's Formalise… and Migrate… |
> | 2 | Where EP-21a goes | After EP-22 | After EP-20b, before EP-23; the chain plan's EP-22 follows EP-21 as set |
> | 3 | What the rule block says, now that the owner's user-level rules carry the method | Self-contained, short | The house's rules in about a dozen lines; a project's own rules outside it win |
> | 4 | Ticking OT-20, whose expected chain-log lines the field check found (closing round) | Tick it | Ticked with `crw tick` (the file's copy in the backups folder, the tick logged) |
> | 5 | Where the review's design note goes, a revert not redoing a formalise (closing round) | EP-21a's notes | Appended to EP-21a's brief note |
> | 6 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The module is `formalise`**, not the note's `core::adopt`: Adopt is `discovery::adopt`
>    already, and the step formalises.
> 2. **The rule file** is the root's `CLAUDE.md`, else `.claude\CLAUDE.md`, the order the opener
>    reads them in; a project with neither gets a root file with a title line and the block.
> 3. **Reads by path only, no git**: the plan never lists a folder or asks git, which keeps the
>    house rule for data repositories (read by path only, never `git status` there) wherever it
>    runs.
> 4. **Markers are whole lines outside code fences**: a marker quoted inside a line (in backticks)
>    or in a fence is no marker, so a file that documents the block is not taken for one.
> 5. **The write reads every file again** and refuses when one changed since the plan, the
>    revert's rule (EP-15c), and a failure after the first write says what was written: running it
>    again finishes the job, as Adopt's does.
> 6. **Refused while a session works or waits** in the project, as `crw revert` is; an idle tab
>    does not count, as for the mailman. A revert of a session that started before a formalise
>    takes its new files and its block back (they are the project's uncommitted changes then):
>    formalising again restores them. A revert redoes ticks and deliveries only (EP-15b, EP-20);
>    the review's design note, parked in EP-21a's notes (decision 5).
> 7. **Logged as an `event` row** of kind `formalise` under the project, no schema change.
> 8. **The inbox header cites `clauderoadmapwrangler D-13`**, the form of the owner's
>    hand-carried inboxes, since a bare `D-13` in another project names that project's decision;
>    the mailman creates inboxes with it from now on.
> 9. **The migration prompt names the installed `crw.exe`** (else the running one) with its full
>    path, since `crw` is not on PATH (IN-3, EP-28), and numbers its steps for the form: the
>    handoff step is the work-packet form's only.
> 10. **A registered project's state stays** (D-6): the preview of an unformalised project says
>     how to set it once its roadmap is written.
> 11. **No console test of `--yes`**: it asks the real CLI's session listing, which tests never
>     run (rule 5); the engine's tests cover every write.
>
> **Notes for later sessions.**
>
> - `formalise::plan(folder, name)` is the preview, `formalise::apply(&plan, backups, today)` the
>   write, `Step::shown` what to show of a file, `Plan::migrates` an older form,
>   `formalise::migration_prompt(name, folder, profile, crw, date)` the prompt (EP-21a, EP-22).
> - The block's text is a constant (`formalise::block`): rewording it makes every formalised
>   project's preview say `replaced between the markers` at its next formalise, which is the
>   markers' purpose.
>
> No messages for other projects. OT-22, the first real formalise, is the owner's; no brief builds
> on it, so it is not gated. **Next: EP-22**, which the chain plan runs if the gate passes.
