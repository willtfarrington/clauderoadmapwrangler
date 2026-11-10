---
ep: 22a
title: The plugin in the tool's runs
size: S
model: fable
effort: high
status: done
---

# EP-22a — The plugin in the tool's runs

**Size:** S · **Depends on:** EP-22 · **Blocks:** EP-24 (the chat reuses the flag)

## Context

D-27's plugin (`plugins/roadmap-wrangler/`: the manifest, four skills, the house-style reference)
is built and tested since EP-22, and the owner installs it into their own sessions from GitHub
(OT-23). The tool's own headless runs do not load it yet: a run sees the plugin only where the
owner's install reaches, and a visual launch cannot pass a flag at all. This brief, split from
EP-22 at its pickup (the owner's choice, D-4), gives every headless run of the window the build's
own copy of the plugin through `--plugin-dir`, so that the method a run follows is the one the
build that starts it carries, whatever the working tree holds and whether or not the owner has
installed the plugin.

**What exists.**

- `session::Launch` (`crates/core/src/session/launch.rs`) is what a headless run is started with,
  and `command_line` builds the arguments from it and from the flags `claude --help` lists
  (`session::cli::Flags`, D-15): a wish the CLI cannot take is left out and named in
  `CommandLine::dropped`, which the Sessions tile shows in amber under the run.
- Every headless start of the window funnels through `Sessions::begin` (`crates/app/src/sessions.rs`):
  the dialog's Start (`board.rs`), the autopilot's chained start (`autopilot.rs`), a resume and
  the answer to a deferred request (`resume_with`, `answer_open`), which clone the run's first
  launch. The CLI starts no session.
- The local folder (`paths::local_dir`: `%LOCALAPPDATA%\clauderoadmapwrangler`, or `CRW_HOME` in
  a trial) holds the ledger, the backups and, since EP-37, `bin\` with the installed exes; `crw
  install` copies the build there and `crw doctor` prints the copy (`print_exes`).
- Claude Code 2.1.316's help lists `--plugin-dir <path>` (read this session): a plugin loaded for
  one session, repeatable; a `--plugin-dir` copy replaces an installed plugin of the same name for
  its session (EP-22's reading of the loading reference).
- `crates/core/tests/plugin.rs` holds the plugin's structure to the engine's texts.

**Session start (2026-11-10, 17:07 local time).** The chain's run: ledger row `ca2b1d42`,
headless, EP-22a, `fable/high`, open, chained after EP-22 at the gate's moment after the window's
restart into EP-22's build (the chain log: `held after EP-22 for the window's restart into the
installed build: EP-22a starts in the new window`, then `chained EP-22a after EP-22: fable at
high, not in your plan, the rules' choice, S's cap of 45 min`). The prompt is the window's opener
in its headless wording. The tree is clean; the inbox has no open message; OT-21 and OT-23 are
open and not gated.

**Field check.** Read back to EP-22's record commit (`fd86276`):

- `crash.log`: last written 2026-11-03, seen before (`crash.seen` is newer);
- `crw notifications`: EP-22's finished popup (17:06) and this chain's notice (17:07); nothing
  odd;
- `crw autopilot`: this project and one other on, `auto`, 2 chained runs a day; the chain held
  for the restart and went on, as EP-37a built it: the first chained restart worked (OT-20);
- `crw mail`: nothing since the first readings at 13:53;
- `crw sessions`: this run's row and EP-22's completed one;
- `crw doctor`: the installed copy built 2026-11-10 17:03 (EP-22's), the hooks run it.

Nothing to triage.

## In scope

1. **The plugin's copy** (`crw_core::plugin`): one table of the plugin's files, each carried by
   `include_str!` from `plugins/roadmap-wrangler/` (the manifest, the four skills, the reference);
   `plugin::dir()`, the copy's folder `<local folder>\plugin\roadmap-wrangler`; `plugin::write`,
   which creates the folder, writes each file that is missing or differs (atomically, through
   `files::write_atomic`), leaves one that matches, removes a file under the folder the build no
   longer has and the folders left empty, and reports what it did; `plugin::status`, whether the
   copy matches the build (matches; differs, with the files; none yet). A structural test beside
   `crates/core/tests/plugin.rs` checks that the table lists every file of the folder and nothing
   else, byte for byte.
2. **The flag**: `Launch` gains `plugin_dir: Option<PathBuf>`; `command_line` passes `--plugin-dir
   <folder>` when the help lists it, else names the wish it dropped (D-15), and a folder that is
   not there is dropped and said like a missing neighbour. `Sessions::begin` writes the copy and
   sets the folder on its worker thread before every start of the window, resumes and answers
   included, so each run gets the plugin of the build that starts it; a copy that cannot be
   written is said in the run's dropped notes and the run starts without it.
3. **Saying so**: the launch dialog's Plugin field, between Neighbours and Questions, names the
   plugin and the copy's folder a headless run gets, and says when the CLI lists no `--plugin-dir`
   (for a visual launch: that a visual session has the plugin the owner installed). `crw doctor`
   prints a `plugin` line (the copy's folder and whether it matches the build) and `--plugin-dir`
   among the headless line's features. `crw install` also writes the copy, so that the installed
   build and the copy agree from the install on (the write is the same function; a trial under
   `CRW_HOME` writes under it).
4. **The records**: a D-27 addendum; the README's command notes and plugin section; the roadmap's
   EP-22a row and this brief's completion note.

## Out of scope

- The chat's use of the flag (EP-24, D-27): it sets the same field.
- A `claude --plugin-dir … --help` smoke in CI (the note leaves it out; tests never run the CLI).
- A visual launch loading the plugin: the extension's link takes only a prompt and a session
  (EP-16); the owner's install (OT-23) is what reaches those sessions.
- Removing the copy when the plugin is withdrawn from the build: a build without the table has no
  copy to write; nothing else reads the folder.

## Verification / acceptance

- **Tests:** the table against the folder (structural); `plugin::write` on a scratch folder (a
  fresh copy, an unchanged one, a changed file rewritten, a stray file and its empty folder
  removed, the report) and `plugin::status`; `command_line` passing the flag, dropping it for a
  help without it and for a folder that is not there, a resume carrying it.
- **The checks:** `cargo test` with and without `--features screenshot`, both clippy runs, `cargo
  fmt --check` (rule 6; an S brief needs no review pass).
- **By hand, spending nothing:** `CRW_HOME=<scratch> target\release\crw.exe install --no-restart`
  writes the copy under the scratch folder; `claude plugin validate <scratch>\plugin\roadmap-wrangler`
  passes it as EP-22's folder passed; `CRW_HOME=<scratch> crw doctor` says the copy matches the
  build, and the owner's `crw doctor` says none is written yet until the install.
- **The release:** `cargo build --release` with the window up, then `pwsh -File
  scripts\install-shortcut.ps1` (rule 6); its restart request is refused while this run works.
- **Rule 3's word check** on the staged diff prints nothing.
- **Residual risk.** No session has started with the flag: tests never run the CLI (rule 5), and
  the window's starts are real runs. Three things are not exercised:
  1. **A headless run loading the copy.** The flag is passed as 2.1.316's help lists it, the copy
     is the folder `claude plugin validate` passes, and Claude Code's loading reference says a
     `--plugin-dir` plugin loads for its session, replacing an installed one of the same name. The
     next chained run of this project (EP-20a) starts with it; its transcript would show the
     `roadmap-wrangler:` skills offered, and the P3 walk (EP-23) reads it.
  2. **The write on the window's worker thread before a real start.** The same function is tested
     on a scratch folder and run by `crw install`; the folder is the tool's own under the local
     folder, where the ledger already lives.
  3. **The dialog's field on the real window**: drawn with the other fields' helper; a screenshot
     run shows it on the fixtures.

> **Completion note (2026-11-10; fable/high; about 30 min of work, one question at the close).**
> One headless run (ledger row `ca2b1d42`, headless, EP-22a), chained after EP-22 at the gate's
> moment, after the window's restart into EP-22's build: the first brief a chained restart
> carried (OT-20's walk).
>
> **Built.**
>
> - `crw_core::plugin`: the table of the plugin's six files by `include_str!`, `dir()`,
>   `write` (a fresh copy, a file that differs rewritten atomically, one the build no longer has
>   removed with its empty folders, a report), `prepare` (what a start sets `plugin_dir` to) and
>   `Status::of` (matches, differs with the files, none yet) with `words()` for `crw doctor`.
> - `session::Launch::plugin_dir`; `command_line` passes `--plugin-dir <folder>` when the help
>   lists it, else drops and says the wish, as for a copy that is not there.
> - `Sessions::begin` (the window's one start funnel: Start, the chained start, a resume, a
>   commit, an answer to a deferred request) writes the copy on its worker thread and sets the
>   folder before each start; a copy that cannot be written is said under the run, which starts
>   without it.
> - The launch dialog's Plugin field (the name, the folder on hover; amber when the CLI lists no
>   flag or there is no local folder; "the session's own" for a visual launch); the screenshot
>   run's canned help lists the flag.
> - `crw doctor`: a `plugin` line and `--plugin-dir` in the headless line; `crw install` writes
>   the copy beside the exes (decision 1).
> - Tests: the table against the folder (`crates/core/tests/plugin.rs`), the write and the
>   status on a scratch folder, a file in the folder's way, the command line's flag.
> - The records: a D-27 addendum, the README's command notes and plugin section, the roadmap's
>   row.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed 521 and with `--features screenshot` 529: EP-22's 516 and 524
>   plus five new in each run. Both clippy runs and `cargo fmt --check` are clean.
> - **By hand, spending nothing.** Under a scratch `CRW_HOME`, `crw install --no-restart` wrote
>   the copy (`6 files written, 0 unchanged`); `claude plugin validate` on that copy passed with
>   the two warnings EP-22's folder gets (no `version`, no icon); `crw doctor` there said
>   `matches the build` and listed `--plugin-dir`; the owner's `crw doctor` says `none yet` until
>   the install below. The scratch folder is removed.
> - **The dialog.** A screenshot run (`launch=harbor:headless` on the fixtures) shows the Plugin
>   field between Neighbours and Questions; the pictures are not kept.
> - **The release.** Built with the window up; the install runs last, after the two commits
>   (rule 6), its request to restart the window refused while this run works, so the window
>   restarts into the new build at the gate's moment or from File > Install and restart.
> - **Rule 3's word check** found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-10).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | `crw install` writing the plugin's copy too, a step beyond the note's wording (closing round) | Keep it | In this brief; the D-27 addendum says so |
> | 2 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The copy is written on the start's worker thread**, in `Sessions::begin`, not in
>    `Session::start`: the core's runner stays what its `Launch` says, the runner tests (the
>    stub) never write under the local folder, and no app test reaches `begin`.
> 2. **A copy that cannot be written does not refuse the run**: the plugin is a help, not a need;
>    the dropped note under the run says it.
> 3. **The dialog shows the folder on hover**, not in the label: a screenshot run's `CRW_HOME` may
>    lie under the user profile, and pictures show no user name in a path (rule 3).
> 4. **The flag's spelling lives in `plugin::FLAG`**, the command line and `crw doctor` reading it
>    from there; no new constant in `session::cli`.
> 5. **No owner task**: the next chained run of this project starts with the flag by itself; the
>    P3 walk (EP-23) reads whether its transcript offered the skills.
>
> No messages for other projects. **Next: EP-20a**, which a chain may run if the gate passes.
