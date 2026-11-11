---
ep: 21a
title: Formalise in the window
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-21a — Formalise in the window

**Size:** M · **Depends on:** EP-21 · **Blocks:** EP-23

## Context

EP-21 built the formalise engine (`crw_core::formalise`) and `crw formalise` (the D-9 and D-11
addenda of 2026-11-10). This brief, split off at EP-21's pickup (the owner's choice), is the
window's side: **Formalise…**, the plan's preview with Write, and **Migrate…**, the launch
dialog with the migration prompt of a five-column or work-packet roadmap. EP-21's review parked
one design point here: a formalise is allowed while a tab sits idle in the project, and a revert
of a session that started before it takes the new files and the block back without redoing
them, unlike the tool's ticks, deliveries and the owner's messages, which a revert redoes.

**Session start (2026-11-10, 23:43 local time).** Started from the window (Start, Visual, Open in
VS Code); its prompt is the tool's opener, and `crw sessions clauderoadmapwrangler` lists its row
(`9bfea0ff`, visual, EP-21a, open). The launch suggested fable at xhigh; the VS Code panel runs
claude-opus-5-5 at max (the owner's answer). The field check read, since EP-20d's record commit
(`58d5973`): `crash.log` (the 2026-11-03 report, nothing newer); `crw notifications` (OT-24's
pushes, `ntfy · test` at 23:25 … 23:28; a manual session's waits and its finished turn at 23:36;
the inbox badges of IN-8 at 23:03 and IN-9 at 23:37); `crw mail` (IN-8 **delivered by the
window's mailman at 23:03**, EP-20d's route 2 and the second half of OT-21; IN-9 and a message to
another registered project written by the owner at 23:32); `crw sessions` (EP-20d completed, a
manual session from 23:02, this one); `crw autopilot` (on, `auto`, eight a day; the chain ended
after EP-20b at the day's cap of four, nothing chained since); `crw doctor` (the installed copy
built at 22:55, EP-20d's; the hooks run it; the plugin's copy matches the build). Nothing odd to
triage. Two open inbox messages (IN-8, IN-9, triaged below); no open owner task: OT-21, OT-24 and
OT-25 were ticked by the owner since the last commit and are uncommitted in the tree, with the two
delivered inbox lines.

**What exists.**

- `formalise::plan(folder, name)` reads by path only and asks git nothing; `Plan` (`steps`,
  `readme`, `problems`, compared whole by `PartialEq`), `Step::words` and `Step::shown`,
  `Plan::lines`, `is_refused`, `writes` and `migrates`; `apply(&plan, backups, today)` reads every
  file again (`Error::Changed`), creates the new files without replacing one (the inbox under
  `inbox::writing`) and copies `CLAUDE.md` before its atomic write; a write is an `event` row of
  kind `formalise` (`Applied::detail`); `migration_prompt(name, folder, profile, crw, date)`.
- The window's Revert (EP-15c): `Board::revert_view` (the window's runs and starts there, a
  visual launch on its way, a start being read, the hooks' and the poll's sessions at work or
  waiting, the poll unanswered), the plan read on a worker thread and read again at the owner's
  word; the compose dialog (EP-20c) opened from the row menu and the Detail pane.
- The launch dialog (`launch::Dialog`) is built from `prompt::draft`, one brief's `ep`: the prompt
  follows the choices until edited, Regenerate, the chain plan of an autopilot project, the Keep
  pins. `start_headless` gives an autopilot project's run its cap, budget, push guard and plan;
  `weigh_run` reads the gate after every ended run of an autopilot project; a completed run not
  told to commit offers Commit, whose message records an EP's hash.
- `revert::ticks_since` feeds both the revert's redo and the gate's inbox exception (EP-20c), so a
  formalise row read there would count as a tool tick for the gate too.

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-10)

1. **The panel's model and effort:** claude-opus-5-5 at max.
2. **IN-9 rolled in:** the README's ntfy paragraph gains the loopback advice and the iOS app's
   needs, in generic words (rule 3).
3. **A revert redoes a formalise** made after the reverted session's start, in this brief, as it
   redoes the tool's ticks and deliveries; EP-21a stays one M.

## In scope

1. **Formalise…** (`app::formalise`, new), on the Revert dialog's pattern: the plan read on a
   worker thread with the window's refusals added (`revert_view`'s, worded for a formalise; an
   idle tab is named, never a refusal); the preview over the window (each file and its change,
   the roadmap's form, the block and each new file's text, what refuses it), Write and Cancel, or
   Close alone when refused or when nothing is to be written; at Write the plan read again and
   written only when it equals the one shown, else the new one shown with a note; after the write
   the files read again, the dialog saying each file written and the status bar naming the copy
   of `CLAUDE.md`; the write logged as the `formalise` event row, as `crw formalise --yes` logs it.
   A registered project's state stays the owner's (D-6): the preview says how to set it.
2. **Where it opens.** The row menu's Formalise… for a project the files pass finds missing a
   house-style file or holding unreadable markers, or in the unformalised state; the Detail pane's
   House style section, which says what Formalise… would write and, for an older roadmap, offers
   Migrate…; the dialog's own Migrate… beside an older roadmap's line.
3. **The row's reading.** `ProjectRow::house`, from `formalise::plan` in the files pass (paths
   only, no git, no child): the files a formalise would write, what refuses it, the older form.
4. **Migrate…** (`Board::open_migration`): the launch dialog in Headless with
   `formalise::migration_prompt` in its editor, its `crw` the installed `crw.exe`
   (`install::bin_dir`), else the one beside the window's exe; the prompt fixed (no choice
   rewrites it; Regenerate puts it back after an edit); no chain plan and no Keep pins; the
   ledger row's EP and the session's name say `migration`; the run outside the autopilot (no cap,
   no budget, no gate after it, no Commit button; the push guard kept), its start recorded for a
   revert as any run's is. The owner may still switch it to Visual.
5. **The revert redoes a formalise** (D-11): `revert::formalised_since` reads the project's
   `formalise` rows since the session's start; the plan carries them (`Plan::formalised`), its
   preview names them, `is_as_seen` compares them, and the revert formalises again after the ticks
   (the plan read then, then written), the result in `Done::redone` or `not_redone`; the window's
   Revert and `crw revert` both pass them. `ticks_since` and the gate stay as they are.
6. **A screenshot run** cans both: `formalise=<project>` opens the dialog on the fixture's own
   plan (read by path; Write refused, as every write is), `formalise=<project>:busy` the same
   plan refused by a canned session at work there, and `formalise=<project>:migrate` the launch
   dialog's migration of a canned work-packet form.
7. **The inbox:** IN-8 declined as EP-20d's test (its delivery was the check); IN-9 rolled in.
8. **The records:** a D-11 addendum (the window's formalise and the revert's redo), a D-9
   addendum (Migrate…), the README (Formalise… and Migrate… in the house-style section, the
   revert's redo, the ntfy paragraph), the roadmap's row and EP-23's notes, the completion note;
   the review subagent (rule 6, an M); the release and the install last.

## Out of scope

- The chat's delegation of formalise and migrate (EP-24, D-27); writing a first roadmap (the
  plugin's skills, EP-22).
- Formalising an unadopted folder from the window (Adopt stays first; `crw formalise <folder>`
  takes one).
- A migration chained by the autopilot, a Commit message of its own, or a migration of the tables
  by the tool itself (D-11 forbids it; D-9's session does it).
- A project's state changed by a formalise (the owner's, D-6).
- The gate's reading of the tool's ticks (`ticks_since`), unchanged.

## Verification / acceptance

- **Tests.** The dialog: a scratch project's plan read, the preview drawn, Write writing the plan
  and logging the row, a plan changed since the preview shown again with nothing written, a busy
  project refused, a screenshot run's Write refused; the row's reading (a formalised fixture, a
  bare one, broken markers); the menu item and the Detail pane's section; the migration dialog
  (its prompt fixed through a choice and Regenerate, no plan, the launch's EP `migration`, its run
  neither gated nor offering Commit); the revert's redo over a scratch repository (a formalise
  after the session's start redone after the reset, its preview line, `is_as_seen`), and
  `formalised_since`.
- `cargo test`, `cargo test --features screenshot`, both clippy runs and `cargo fmt --check`
  clean; a review subagent given the diff alone, its findings fixed or logged.
- Screenshot runs of the three flags on the fake portfolio, and of the dialog on a scratch copy of
  the real roadmaps (EP-36), the pictures kept out of git.
- Rule 3's word check before each commit; the release built with the window up, then the install.

**Residual risk.** The session wrote only into scratch folders and the tracked fixtures were only
read (rules 4, 5 and 13), so three things are not exercised. (1) **The first Formalise… on a real
project from the window**: the dialog drives the same engine write `crw formalise --yes` made for
real in OT-22, through the Revert's worker-thread pattern (EP-15c) and its session refusals, and
tests write a scratch project end to end (the plan read, the second reading, the write, the ledger
row); what is new is the real poll's answer at the click and the files pass showing the section
gone after it. (2) **The first Migrate… of a real older roadmap**: no registered project's tables
were checked to be in an older form, and no real session has run the migration prompt (EP-21's
residual risk 2, unchanged); the run goes through the launch dialog's tested headless path, and
the prompt makes the session put its plan to the owner and stop before committing. (3) **The
first revert that redoes a formalise**: tested over a scratch repository through the same
`formalise::apply` as the dialog's; a real one needs a session started before a formalise and
then reverted, which is rare. EP-23's walk takes all three (its notes).

> **Completion note (2026-11-11; claude-opus-5-5/max; about 1 h 15 min of wall time, three
> questions at pickup and two in the closing round).** A visual session started from the window
> (ledger row `9bfea0ff`, from 23:43 on 2026-11-10).
>
> **Built.**
>
> - **Formalise…** (`app::formalise`, new): `Dialog` with Reading, Ready, Writing, Done and
>   Failed; the plan read on a worker thread with the window's refusals among its problems
>   (`Board::work_view`, the Revert's view with the action's verb); the preview (each file and
>   its change, the README's form with Migrate… for an older one, the refusals, an idle tab
>   named, the unformalised state, the rule file's text open and each new file's collapsed);
>   Write reading the plan again and writing only the plan shown (`Answer::Changed` otherwise,
>   an engine `Error::Changed` too); the write logged by `formalise::log`, which `crw formalise
>   --yes` now shares; the files read again and the status bar naming the copy of `CLAUDE.md`.
> - **Where it opens:** the row menu's Formalise… and Migrate the roadmap…, the Detail pane's
>   House style section, both from `ProjectRow::house` (`House::read`, `wants`, `writes_words`),
>   which the files pass reads through `formalise::plan` by path.
> - **Migrate…:** `launch::Dialog::migration`, `label`, `MIGRATION`, `migration_title`; the
>   dialog's `ep` optional, its `migrates` the older form; `Board::open_migration` with
>   `launch_cli` and `launch_warnings` factored out of `open_launch`, and `migration_cli`;
>   `sessions::Run::migration` and `Live::migration` through every resume; `weigh_run` skips a
>   migration, `offers_commit` leaves it out, `Live::finished_migration` holds a gate's restart.
> - **The revert's redo:** `revert::Formalised`, `formalised_since`, `Context::formalised` and
>   `Plan::formalised`, the preview's line, `redo_formalise` after the ticks; the window's Revert
>   and `crw revert` pass the rows; `formalise::form_words` is public.
> - The restart's refusals gain the formalise preview and the compose dialog (found by the
>   session: EP-20c left it out, so a restart could drop a message being written); Quit waits
>   for a formalise's write; the screenshot flag `formalise=<project>[:busy|:migrate]`.
> - **The records:** the D-9 and D-11 addenda, the README (the house-style section, the ntfy
>   paragraph from IN-9), the roadmap's row and EP-23's notes, IN-8 and IN-9 ticked.
>
> **The review (rule 6).** A subagent given the code diff alone, launched as soon as the code
> compiled, reported after about fourteen minutes: no serious bug, eight points. Fixed and
> tested where they could be:
>
> 1. **A restart at the gate's moment would have dropped a finished migration's tab**: the gate
>    kept only tabs offering Commit, which a migration's no longer does. `Live::finished_migration`
>    is now a refusal of its own.
> 2. **Quit did not wait for a formalise's write**, which a tray Quit could cut between two files;
>    it waits now, as for a revert.
> 3. **`crw formalise --yes` on a folder that is not registered** logged its row under the
>    folder's name, which a registered project may share and a revert would then redo there: it
>    logs for a registered project only.
> 4. A doc comment the session displaced in `screenshot.rs`; the Detail section's folder check;
>    one migration title for the rules and the dialog; no dead Migrate… button while the write
>    runs.
>
> Kept: the redo reads the plan after the reset rather than replaying the logged files, so it
> may create a house-style file the first formalise kept, when the reverted session had made it;
> that is the "formalises again" the owner chose at pickup.
>
> **Verification.**
>
> - **Tests.** `cargo test --workspace` passed (354 in `core`, 146 in `app`, the rest in the
>   fixtures and the CLI), and with `--features screenshot` 154 in `app`: EP-20d's counts plus 1
>   in `core` (a formalise after the start redone over a scratch repository, `formalised_since`
>   by project and time, `is_as_seen`, `ticks_since` untouched) and 8 in `app` (the preview
>   writing its plan and logging the row; a plan changed since the preview, or a session at work
>   since, shown again with nothing written; a busy project refused and an idle tab named; a
>   fixture's plan with Write refused; the row's reading and Migrate…, the formalise preview
>   giving way to it; the migration dialog's fixed prompt, Regenerate, label and missing pins; a
>   migration offering no Commit and resuming as one; the House style section), and the gate's
>   restart test extended to a finished migration. Both clippy runs and `cargo fmt --check` are
>   clean.
> - **Screenshot runs** on the fake portfolio (the preview, the refused preview, the migration
>   dialog, the Detail pane's section) and on a scratch copy of two real non-data projects'
>   roadmap files (the preview of a long `CLAUDE.md`); the pictures stayed in the scratchpad.
> - **The release** built with the window up; the install runs last, after the commits.
> - **Rule 3's word check** found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-10 and 2026-11-11).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The panel's effort (a fact for the record) | max | The front matter and this note |
> | 2 | IN-9, outside the brief's subject | Roll it into EP-21a | The README's ntfy paragraph in generic words; IN-9 ticked |
> | 3 | What a revert does with a formalise made since the session's start (EP-21's review) | Redo it, in this brief | `formalised_since` and the redo; the D-11 addendum |
> | 4 | An owner task for the first real Formalise… and Migrate… | No task: EP-23's walk | Both, with the first redone formalise, in EP-23's notes |
> | 5 | The House style section on most projects' Detail panes | Keep it; the walk judges it | As built; EP-23's notes weigh it |
> | 6 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The Revert's dialog pattern**, not the compose's: the write must be the plan shown, so
>    the plan is read on a worker thread and read again at the click.
> 2. **The window's refusals join the plan's `problems`**, so the preview says them, the
>    engine's `apply` refuses them, and the plan's equality catches one new at the click.
> 3. **The row's reading in the files pass**, one more README parse per project per read; no
>    git and no child, which keeps the house rule for data repositories.
> 4. **A screenshot run's formalise sees no session** (it polls none); `:busy` cans one, and
>    `:migrate` cans a work-packet form, since every fixture roadmap is in the house form.
> 5. **Migrate…** keeps the Launch switch (Visual allowed), labels the run `migration` rather
>    than leaving the EP empty, hides the Keep pins, keeps the push guard and records the start
>    for a revert; its prompt names the installed `crw.exe`, a screenshot's a neutral path.
> 6. **The House style section** sits after the decision log and before the readers' problems.
> 7. **The owner's ticks** of OT-21, OT-24 and OT-25 and the two delivered inbox lines,
>    uncommitted at the session's start, are committed with the work, as earlier work commits
>    carried the owner's ticks.
> 8. **The plugin's formalise skill is unchanged**: it names `crw formalise`, which a session
>    can run, and no session can press the window's button.
>
> No messages for other projects, and no owner task (decision 4). **Next: EP-23**, the P3
> re-plan, opening with the walk.
