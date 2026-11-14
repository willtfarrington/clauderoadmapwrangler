---
ep: 34
title: Hosting and backup actions
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-34 — Hosting and backup actions

**Size:** M · **Depends on:** EP-7 · **Blocks:** EP-29 · **Decisions:** D-24, D-11, D-29, D-3

## Context

D-24 decided the hosting badges and four actions behind a confirmation dialog, logged and never
automatic (Create private remote, Make public repeating `gh`'s consequences warning, Make
private, Remove remote), the orphan's Archive and Delete (Delete needs the `delete_repo` scope
the token lacks), and Back up now (the backup job's script with `-Force`, no second mechanism).
EP-7 built the readings and parked the actions; EP-8 placed them here (D-24 addendum): one
confirmation dialog that names the command, one child at a time under `process::turn()`, each
action an `event` row of the ledger, then a refresh of the project's badges; a failure is shown,
never retried silently; tests use an injected runner, never `gh` or the network (D-29). From
EP-23: the dialogs follow the Revert's and the formalise's pattern (the plan read on a worker
thread with the refusals of `Board::work_view(project, verb)`, read again at the click and run
only when it is the plan shown), a restart is refused while one is open, Quit waits for a
running action, each dialog has a screenshot flag, and Make public, the one action that
publishes, says what rule 3 says of a push.

**Session start (2026-11-13).** The prompt is this project's opener; `crw sessions
clauderoadmapwrangler` lists this run (`51ca30e2`, headless, EP-34, started 17:42), and `crw
autopilot`'s chain log says the autopilot chained it after EP-27 with claude-opus-5-5 at max
(the rules' choice) under M's cap of 1 h 30 min, once the window had restarted into EP-27's
build. Read: the P4 table and EP-34's brief note, D-24 with its three addenda, D-3, D-11, D-29,
EP-27's completion note, `owner-tasks.md` (OT-27 open, the owner's hands) and the inbox (no open
message). The field check: `crash.log` is still EP-14a's of 2026-11-03, seen; `crw
notifications` since EP-27's record commit (17:26) shows the mailman's delivery of EP-27's
letter into the site's inbox at 17:29 (prime IN-1, the first brief's `Handed on` block delivered
into another project: EP-20's residual risk, for EP-29's walk), EP-27's finish at 17:41 and this
chain at 17:42; `crw sessions` shows this run open; `crw autopilot` six projects on, not paused.
Nothing odd to triage. The working tree was clean at the start.

**The code as found.** `badges::hosting` classifies a project (`Hosting::{Local, Orphan,
Listed, Missing, GitHub, Elsewhere}`) from its origin's URL (read from `.git/config`) and the
account's list (`gh repo list`, cached in the ledger, refreshed daily and on F5); `gh()` finds
the CLI and `GH_ENV` keeps it from prompting. `badges::backup` reads the job's settings and run
log, never runs it; the registry names the settings and the log, not the script. The job's
script sits beside its settings under the settings' name with `.ps1`, defaults to a dry run, and
is run by its scheduled task as `-Mode Run`; `-Force` skips its minimum hours between runs; a
run took one and a half to two minutes this week and exits 0 (ok or skipped), 1 (an error) or 2
(a warning, such as a target not ready). `process::output_with_stderr` keeps a child's words for
its failure; `process::TIMEOUT` is 20 s. The dialogs' pattern is `app::formalise` (worker
thread, `Stage`, `confirm` reading the plan again), `app::revert::View` and `Board::work_view`;
`Board::restart_refusals` names each open dialog; `Board::quit` waits while a revert runs or a
formalise writes. The local `gh` is 2.104.0: `gh repo edit` prints the consequences paragraph
the dialog repeats; `gh repo delete` names the scope and `gh auth refresh -s delete_repo`, and
refuses a private repository with forks without `--accept-cascading-deletion-of-private-forks`
when not interactive; `gh auth status --active --hostname github.com --json hosts` gives the
login and the token's scopes (no `delete_repo` today).

## In scope

1. **`core::actions`**: `Verb` (Create private remote, Make public, Make private, Remove remote,
   Archive, Delete, Back up now), `Facts` (what the window knows: the project, its folder, the
   data mark, the origin, the hosting from the cached list, `HEAD`, the work view's refusals),
   `Plan` (the command as program, arguments and folder; what it does in words; the warnings;
   the refusals), `plan(facts, runner)`, a `Runner` seam (`Real`: `gh` and `git` under
   `process::turn()` with their words kept; the backup's script started under the turn, its
   end waited on outside it), `run(plan, runner)` and `log(ledger, …)` (an `event` row of kind
   `hosting` or `backup` with the command and its outcome). Refusals: Create for a project
   with a remote, an orphan's name taken, no repository, no commit, the login unread; the
   visibility changes for a repository not listed, archived, or already of that visibility;
   Remove remote without an origin; Archive and Delete for anything but an orphan (Archive also
   for one already archived); Delete while the token lacks `delete_repo` (the dialog names `gh
   auth refresh -s delete_repo`); Back up now with no job, its script missing or its target not
   mounted; Create and Make public for a data repository.
2. **`app::act`**: the confirmation dialog on the formalise's pattern (Reading, Ready, Running,
   Done, Failed), the plan read on a worker thread and again at the click; the window's
   refusals from `Board::work_view(project, verb)`; opened from the row's menu (the actions the
   row's hosting allows), the Detail pane's Hosting and Backup subsections and the File menu's
   Back up now…; a restart refused while it is open, Quit waiting while it runs, the window's
   keys taken; after an action, the project read again with its GitHub reading asked afresh
   (the account's list and its head check), or the backup job read again; a screenshot flag
   `act=<project>:<verb>` (and `act=backup`) with a canned runner; tests never reach `gh`.
3. **`crw act <project> <verb>` and `crw act backup`**: the plan previewed (a read: the cached
   list, plus `gh auth status` where the plan needs the login or the scopes); no `--yes`, the
   window's dialog being the confirmation D-24 asks for.
4. **Tests**: each verb's plan and refusals over synthetic facts, the probe's reading, the
   script found beside its settings, a run through a fake runner with its event row, the
   dialog's stages (a plan changed since the preview runs nothing), the board's refusal in a
   screenshot run.
5. **Docs**: a D-24 addendum, the README's words, the roadmap row linked, the completion note
   and an owner task for the first real action.

## Out of scope

- Running any real action in this session (rule 5; the first is the owner's).
- Scanning a project's history for private words before Make public (parked for EP-29's walk).
- `gh auth refresh` run by the tool (it opens a browser: the owner's terminal), and
  `--accept-cascading-deletion-of-private-forks`, which the window never passes.
- Connecting an orphan as a project's origin, renaming, descriptions and topics.
- A CLI that runs the actions.

## Verification / acceptance

1. `cargo test --workspace` and `cargo test --workspace --features screenshot` pass; both clippy
   runs and `cargo fmt --check` are clean; rule 3's word check on the staged diff prints nothing.
2. A review subagent given the diff (an M brief), launched at the first clean compile; its
   findings fixed or logged.
3. Screenshot runs of the dialogs (Make public with `gh`'s warning and rule 3's words, Delete
   refused for the scope, Back up now) and of the row's menu and the Detail pane's buttons, on
   the fixtures and on a copy of the real roadmaps (rule 5); the pictures stay in the
   scratchpad.
4. `crw act` previews over the owner's registry and cache (reads): a local project's Create, a
   public one's Make private, the orphan's Archive and Delete, Back up now.
5. The release build and the install run last (`scripts\install-shortcut.ps1`).

**Residual risk.** No real action ran in this session (rule 5; the first is the owner's,
OT-28): `gh repo create --push`, `gh repo edit --visibility`, `gh repo archive` and `delete` and
`git remote remove` were exercised through a fake runner only, so their real words on success
and on failure are `gh`'s and git's own, shown whole and never retried. The session expects them
to hold because each command and flag was read from the installed `gh` 2.104.0's help, and the
previews over the owner's registry named the real login (`gh auth status` asked for real), the
real orphan with its Delete refused for the missing scope, and the real job's script with its
target mounted. The backup job's run from the window (about two minutes from its script's
folder, its exit code read, the Backup lines read again, its dialog closed while it runs) was
exercised through a fake runner only, nor was the refresh after an action seen in a running
window (the project read again, then F5's own GitHub pass), whose paths are the files pass's
and the GitHub pass's. The dialogs, the row's menu and the Detail pane's buttons were seen in
screenshot runs and pointer tests, not in the owner's window.

> **Completion note (2026-11-13; claude-opus-5-5/max; about 5 h 25 min of wall time, of which
> about 4 h 25 min was the owner's answer to the one question, so about an hour of work; a
> headless run chained by the autopilot).** Ledger row `51ca30e2`, started 17:42 under M's cap
> of 1 h 30 min of working time; the prompt was the opener. Built as scoped.
>
> **Built.**
>
> 1. **`core::actions`**: `Verb` (the seven, with `id`, `parse`, `label`, `question`, `done`,
>    `kind`, `offered`), `Target` (`read` by path: the repository and the origin), `Facts`,
>    `Command` (its timeout and folder, `line`), `Plan`, the `Runner` seam with `Real` (`gh` and
>    `git` in their turn, prompts off; the job started in its turn from its script's folder and
>    waited on outside it), `parse_auth` over `gh auth status --json` (Create's login, Delete's
>    scope), `job_script`, `plan`, `backup_plan`, `run` (`Ran`, `End`) and `log` (event rows of
>    kind `hosting` or `backup`). `process::{Exit, exit_of, start_alone, wait_alone}` over a
>    shared `finish`, the older callers' behaviour kept.
> 2. **`app::act`**: the dialog (Reading, Ready, Running, Done, Failed; the plan read again at
>    the press), `offered(row)`, `hint`, the `Canned` runner of screenshot runs and tests, a
>    running backup's dialog that may close (`Detached`, `Board::act_detached`), and the board's
>    `act_asked`, `open_act`, `confirm_act`, `close_act`, `receive_act`, `act_dialog` and
>    `acting`; `Action::{OpenAct, ConfirmAct, CloseAct}`; the row menu's group, the Detail pane's
>    button lines under Hosting and Backup, File's Back up now…; Quit and the window's close
>    wait while an action runs, a restart is refused while its dialog is open or a backup it
>    started runs, and the keys are taken; after a run the project is read again
>    (`Moment::Acted`) with F5's GitHub pass, or the job again (`Loader::read_backups`);
>    `Git::has_commit`; screenshot flags `act=<project>:<verb>` and `act=backup`.
> 3. **`crw act <project> <action>` and `crw act backup`**: the preview, running nothing.
> 4. **Tests**: 9 in core, 5 in `app::act`, one each in `list` and `detail`, 3 in the CLI.
> 5. **Docs**: the D-24 addendum, the README's words, the roadmap row linked, EP-29's note (the
>    first delivery between projects, the walk's two EP-34 items), OT-28.
>
> **Checked.** Screenshot runs on the fixtures: Make public (the command framed, `gh`'s
> consequences paragraph and rule 3's word in amber, the red button), the orphan's Delete
> refused for the scope, Back up now with a neutral script, and a short Detail pane's Backup
> button; on a copy of five real non-data roadmaps (rule 5): a private project's Hosting
> buttons and Back up now in its Detail pane, and Create on a local project, its long command
> wrapping in the frame. The first picture found the flag opening before the canned readings
> (fixed). `crw act` previews over the owner's registry and cache (reads, never in a data
> repository): Make private on a public project and Make public refused there, Create on a
> local project with the real login from `gh auth status`, the orphan's Archive planned and its
> Delete refused for the missing scope, Back up now with the job's real script found beside its
> settings and its target mounted. No action ran.
>
> **Review.** A subagent given the diff and the brief (17 min, launched at the first clean
> compile) made 11 points, all taken: (1) the one-project GitHub pass superseded a running F5
> and re-stamped the account's cache, postponing every other project's daily look: F5's pass
> runs instead; (2) the screenshot flag's order, which the first picture had found already; (3)
> `gh auth status` under the shared turn for up to 5 min: a timeout per command (20 s for the
> status, a minute for a change, 5 min for Create's push); (4) the window held behind the
> backup's dialog for its two minutes, its words promising a run that goes on while Quit
> waited: the dialog may close, the board still reads and logs its end, and a second run and a
> restart are refused meanwhile; (5) a failed `git status` read as "no commit": unknown now, in
> the window and the CLI; (6) the job run from the window's folder: from its script's now; (7)
> wording (Create pushes the current branch only; the orphan only under the project's or its
> folder's name; a failure's sentence); (8) a failure's command named from its arguments, not a
> re-split line; (9) the error reader joined only for a failure, as before; (10) the spinner
> rows wrapping; (11) two doc comments put back in place.
>
> **Verification.** `cargo test --workspace` (187 in `app`, 395 in `core`, the CLI's suites)
> and `cargo test --workspace --features screenshot` (195 in `app`) pass; both clippy runs and
> `cargo fmt --check` are clean; rule 3's word check on the staged diff printed nothing. The
> release build and the install run after the record commit, the opener's last step.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Refuse Create a private remote and Make public for a project marked a data repository | Keep the refusal (the recommended option) | As built: the dialog says so and offers only Close |
> | 2 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The job's script found beside its settings** under their name with `.ps1`, not a new
>    registry setting: no write into the owner's registry (rule 13).
> 2. **`-Mode Run` beside the note's `-Force`**: the job's default mode is a dry run; it runs as
>    its scheduled task runs it.
> 3. **The job started in its turn and waited on outside it**, never killed at its 30-minute
>    wait: the git and GitHub passes go on while it runs.
> 4. **The login and the scope read from `gh auth status --json` at the plan**, so Delete is
>    refused before the press rather than failing on a 403 after it.
> 5. **A refused plan names no command**; the dialog says why and offers Close.
> 6. **The window's refusals for every project action** (the brief note's `work_view`), and for
>    Back up now only a run it started that still goes on.
> 7. **`crw act` previews only**: the window's dialog is D-24's confirmation.
> 8. **Make public and Delete in the window's red**, as the Revert's button is.
> 9. **The scratchpad** is `target\tmp\ep34\` (ignored): the logs, the homes, the copy of the
>    roadmaps, the pictures, the diff.
>
> **Owner task:** OT-28, the first real action. **Next: EP-28** (operations, Diagnostics),
> chained by the autopilot when its gate allows, else from the window.
