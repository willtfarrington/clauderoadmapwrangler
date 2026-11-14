---
ep: 28
title: Operations, Diagnostics
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-28 — Operations, Diagnostics

**Size:** M · **Depends on:** EP-15, EP-37 · **Blocks:** EP-28a, EP-29 · **Decisions:** D-29, D-3,
D-8, D-15, D-20, D-24

## Context

D-29 asks for a `doctor` command and a Diagnostics tile with its checks (Claude Code's version
against the minimum, the login, the extension's version, the hooks and the status line present and
pointing at the current exe, Tailscale's state, the backup drive mounted, `gh`'s login, the
registry's and the ledger's integrity), a rolling log under the local folder with daily rotation,
and an AV-quarantine hint. EP-23 cut the operations brief beforehand (the owner's choice, D-29
addendum of 2026-11-11): EP-28 keeps `crw doctor`, the Diagnostics tile, the rolling log and the
AV hints; EP-28a takes the start (start at login, the reconciliation of open rows, `crw` on PATH,
the app's own icon, the install script final; single instance came forward to EP-40). The brief
note gathers what earlier briefs left for the tile: the preflight and the last handshake (EP-9),
the stale lock files of `~\.claude\ide` counted and never deleted, and where the visual launch finds
`Code.exe` (EP-11), `signals::install::status` (EP-12), the headless route and
`Flags::prompt_tool`, the defer route's `crw.exe` beside the window (EP-14), the notifications' and
the tray's lines (EP-15), the installed copy and the plugin's copy (EP-37, EP-22a), the standalone
CLI's version beside the extension's and the protocol crate's pin, `crash.log`'s last report, the
ntfy settings with the last push (EP-20a), the mailman's last round and the usage poll's last
answer, the backup job's `VolumeLabel` set against the drive's own (EP-7) and the GitHub cache's
`fetched_at`.

**Session start (2026-11-13).** The prompt is this project's opener; `crw sessions
clauderoadmapwrangler` lists this run (`1f762881`, headless, EP-28, claude-opus-5-5/max, started
22:59), and `crw autopilot`'s chain log says the autopilot chained it after EP-34 with the rules'
claude-opus-5-5 at max under M's cap of 1 h 30 min, once the window had restarted into EP-34's
build. Read: the P4 table and EP-28's brief note (with EP-28a's, which names the cut), D-29 with
its addenda, EP-34's completion note, `owner-tasks.md` (OT-27 and OT-28 open, both the owner's
hands, neither gating this brief) and the inbox (no open message). The field check: `crash.log` is
still EP-14a's of 2026-11-03, seen; `crw notifications` since EP-34's record commit (22:57) shows
EP-34's finish at 22:59 and this chain at 22:59; `crw sessions` shows this run open; `crw
autopilot` six projects on, not paused. Nothing odd to triage. The working tree was clean.

**The code as found.** `core::doctor` probes `claude`, `gh`, `tailscale` and `code` for their
versions and reads the extension's from VS Code's `extensions.json`; the CLI's `doctor` adds this
exe, the installed copy with its build time, the plugin's copy, which `crw.exe` the hooks run
(`print_exes`), the minimum and the preflight (login, headless refusals, flags). `process`'s
children start with `did not start: <error>` on failure and no hint. The window's notices go
through `Board::notify`, `warn`, `fail` and `mention`, and vanish from the status bar; nothing is
logged to a file but `crash.log` (EP-14a), whose parser lives in `app::crash`. The layout's panes
are `layout::Pane` (six), and a saved tree gains a later pane beside the Detail pane.

## In scope

1. **The report (`core::doctor`).** A `Report` of `Line`s (a group, a name, a level of ok, note,
   warn or fail, the words), gathered with the children one at a time in their turn
   (`process::turn()`), shared by `crw doctor` and the tile:
   - **Tools:** `claude`, `gh`, `tailscale`, `code` and the extension, as today; Claude Code
     against the minimum; the extension's version beside the standalone CLI's; the protocol
     crate's pin (a constant held to `Cargo.lock` by a test).
   - **Exes:** this exe, the installed copy and its build time, the plugin's copy, the hooks and
     the status line (`signals::install::status`: each of the five events and the line, which
     exe they run, and whether that exe is there).
   - **Accounts:** `gh auth status` (the login and whether the token has `delete_repo`, EP-34's
     reader); in the CLI the preflight's login as today (the tile shows the window's own
     preflight).
   - **Network:** Tailscale's state from `tailscale status --json` (the backend's state and
     whether this machine is online; peers, names and addresses never kept).
   - **Backup:** the job's target drive mounted, and its label against the job's `VolumeLabel`
     (read from now on), through `GetVolumeInformationW`.
   - **Records:** the registry read (its projects counted, or the error), the ledger read only
     (its schema against the build's and SQLite's `quick_check`), `crash.log`'s last report (the
     parser moved into core, `app::crash` keeping the hook and the status bar's word), the
     rolling log's folder and files.
   - **Readings:** the usage endpoint's last reading, the GitHub list's `fetched_at`, the
     mailman's last logged row, the last ntfy push, the latest status-line sample, each with its
     age; the ntfy settings (the host, and whether a topic is set, never the topic).
   - **VS Code:** where the visual launch finds `Code.exe` (`visual::locate`), and the lock files
     in `~\.claude\ide` counted, live and stale (a stale one's process is gone), never deleted.
2. **`crw doctor`** prints the report by group, then the preflight as today; it fails on a fail
   line, as it fails today on a missing or old `claude` or a refused headless run.
3. **The Diagnostics tile** (`app::diagnostics`, `Pane::Diagnostics` in the right-hand group of
   the default layout and added to a saved layout beside the Detail pane): the report gathered on
   a worker thread when the tile is first shown and at its Check again button, its lines coloured
   by level; above it the window's own lines, read each frame: the build it runs and the installed
   one, the signals (`Watch::line`), the notifications and the tray, the meter's state, the
   mailman's last round, the headless route with the preflight's readiness and
   `--permission-prompt-tool` (`Sessions::route_for`, `Flags::prompt_tool`), the defer route's
   `crw.exe` beside the window, the log's file. A screenshot run shows a canned report and starts
   no child; a flag opens the tile (`tile=diagnostics`, there already).
4. **The rolling log (`core::log`).** `%LOCALAPPDATA%\clauderoadmapwrangler\log\crw-<date>.log`
   (`CRW_HOME\log` in a trial), one file per local day, files older than 14 days removed at the
   window's start and when the day turns, only files of that name. The window writes its start
   (the build, the pid, a restart), every notice, warning and error the status bar shows, its
   quit and its restart, a panic's head line beside `crash.log`, the report's warn and fail lines
   when gathered, and a child that did not start; `crw` and the hooks write nothing (a hook runs
   in about 20 ms, at every event). Screenshot runs and tests write no log.
5. **AV hints.** A child that does not start says why in words that point at security software
   where Windows' error does: the file flagged (errors 225 and 226), access denied (5), the file
   gone (2, 3) for a path that names one; in `process` (every short-lived child, the backup job)
   and the session runner's start. The report says the same of an exe the hooks run that is
   missing, and of the defer route's `crw.exe` missing beside the window.
6. **Tests:** the report's levels and words over synthetic places (scratch registry, ledger,
   settings, lock files, crash log); Tailscale's JSON read without its peers; the label compared;
   the ledger's check; the log's write, its day file and its pruning; the hints by error code; the
   pin against `Cargo.lock`; the pane in the default and a saved layout; the tile drawn from a
   canned report. Never the real `claude` (rule 5).
7. **Docs:** a D-29 addendum, the README's words, the roadmap row linked, the completion note.

## Out of scope

- EP-28a's part: the start-at-login entry and `--register-autostart`, the start-up reconciliation
  of open rows against `claude agents --json`, `crw` on PATH, the app's own icon in the tray and
  the window, `install-shortcut.ps1` final.
- Deleting stale lock files (the extension's) or anything else the report finds; fixing is the
  owner's or a later brief's.
- A log written by `crw` or the hooks; a log viewer in the window (the tile names the file).
- The push guard as a check (risk 17, left a risk at the P3 walk).

## Verification / acceptance

1. `cargo test --workspace` and `cargo test --workspace --features screenshot` pass; both clippy
   runs and `cargo fmt --check` are clean; rule 3's word check on the staged diff prints nothing.
2. A review subagent given the diff (an M brief), launched at the first clean compile; its
   findings fixed or logged.
3. `crw doctor` from the release build over the owner's registry, ledger and settings (reads,
   rule 13): every group answers, nothing written.
4. Screenshot runs of the tile (both themes, a narrow and a wide right-hand group) on the fixtures
   and on a copy of the real roadmaps (rule 5); the pictures stay in the scratchpad.
5. The release build and the install run last (`scripts\install-shortcut.ps1`).

**Residual risk.** The pane's own gathering never ran in a window: tests and screenshot runs start
no child (rule 5), so the worker thread that runs `probe` and `report` in the owner's window (six
children in their turn: four versions, `gh auth status`, `tailscale status --json`) was not
exercised there. The session expects it to hold because `crw doctor` ran the same two functions
over the owner's registry, ledger and settings, every group answering, and the window calls them
on a worker thread as its git and GitHub passes call theirs. The rolling log's real lines (the
window's start, its notices, its restart and its end) were exercised only by tests on scratch
folders (the day file, the turn of the day, the fourteen days, a repeat left out); the window
writes through the same `Log` once `main` starts it. No refusal by security software could be
provoked: the hints are tested by error code and by a real missing program (Windows' error 2),
and a real 225, 226 or 5 takes the same path. The new tab joins the owner's saved layout beside
the Detail pane through the path EP-24's Chat pane took; it was seen in screenshot runs only, and
in the narrow right-hand group the fifth tab's title may sit past the tab bar's scroll arrow.

> **Completion note (2026-11-13; claude-opus-5-5/max; about 1 h, all of it work, no question
> put; a headless run chained by the autopilot).** Ledger row `1f762881`, started 22:59 under M's cap of 1 h 30 min of working time;
> the prompt was the opener. Built as scoped.
>
> **Built.**
>
> 1. **`core::doctor`'s report**: `Level` (ok, note, warn, fail), `Group` (eight, in order),
>    `Line`, `Report` (`group`, `worst`, `problems`, `line`), `Probed` and `probe` (the versions
>    in their turn now, then `gh auth status` through EP-34's reader and `tailscale status
>    --json`, read by `parse_tailscale` for `BackendState` and `Self.Online` alone), `Places`
>    (`real()`, or scratch ones), `report` (files only; its warn and fail lines go to the log),
>    `volume_label` (`GetVolumeInformationW`, the `windows` crate's `Win32_Storage_FileSystem`
>    feature, no new crate), `drive_line`, `PROTOCOL_PIN` with its test against `Cargo.lock`, and
>    the crash log's form and reader (`CRASH_HEAD`, `CRASH_SEP`, `Crash`, `last_crash`) moved from
>    `app::crash`. `backup::Job::label` reads `VolumeLabel`; `backup::drive_root` is public;
>    `Ledger::check` runs `PRAGMA quick_check` on a read-only ledger.
> 2. **`crw doctor`** prints the report by group, then the preflight under Headless; it fails on
>    a fail line. `print_exes` is gone (the Exes group says it).
> 3. **The Diagnostics pane** (`app::diagnostics`, `Pane::Diagnostics`): the report gathered on a
>    worker thread when the tab is first shown and at Check again, only in the window `main`
>    starts; the window's lines (`Board::window_lines`: build, signals, notifications, usage
>    meter, mailman, headless, defer route, log), read at most every 2 s since the
>    notifications' line reads the registry; each line one wrapped text; a canned report in
>    screenshot runs.
> 4. **The rolling log** (`core::log`): `Log` (the day file, the turn of the day pruning, a line
>    written within five minutes left out whatever came between), `start`, `line`, `try_line`
>    (the panic hook's, which never waits on the lock), `folder`, `files`, `prune`; the window
>    starts it in `main` and writes
>    its start, its notices, warnings, errors and held-back notifications (`Board::notify`,
>    `warn`, `fail`, `mention`), its restart, its end, a panic's line and each child that did not
>    start.
> 5. **The AV hints** (`process::hint`, `explained`, `did_not_start`; `FLAGGED`, `DENIED`,
>    `GONE`) in `process`'s children, the backup job, the session runner's start and the visual
>    launch; the report's entry lines and the defer route's line use `GONE`.
> 6. **Tests**: 14 in core (the log 3, the hints 2, the ledger's check, Tailscale, the drive
>    line, the label read, the pin, the crash reader, a push told from a failed push, a bare
>    entry looked up on PATH, the report over scratch places), the pane's draw in the app, and
>    the new pane in the layout's tests (behind Decisions by default, beside the Detail pane in
>    a saved layout, which stays in front).
> 7. **Docs**: the D-29 addendum, the README's words and the CLI's usage text, the roadmap row
>    linked, a pickup note in EP-28a's brief note.
>
> **Checked.** `crw doctor` from the debug build over the owner's registry, ledger and settings
> (reads, rule 13): every group answered, with the hooks running the installed copy, the backup
> drive mounted and labelled as its job expects, the ledger sound at schema 7, the crash log's
> report of 2026-11-03, no log file yet (no window had run this build) and no stale lock file;
> under a scratch `CRW_HOME` it wrote nothing. Screenshot runs of the pane in front: dark on the
> fixtures and light on a copy of three real non-data roadmaps (rule 5); the first picture found
> a line's level word left alone on its row (no real space where a line could break), fixed. The
> pictures, the copy and the logs stay in `target\tmp\ep28\`.
>
> **Review.** A subagent given the diff and the brief (17 min, launched at the first clean
> compile) made 7 points, all taken, and confirmed one the session had fixed meanwhile (the
> window's lines read the registry at every frame through the notifications' line: kept 2 s
> now): (1) a failed ntfy push counted as the last push: the log's two row forms are told apart
> (`ntfy · `, `ntfy failed · `, held to `notify`'s words by a test) and a failure warns with its
> reason; (2) a screenshot run's defer-route line showed the machine's path: a neutral one now
> (rule 3); (3) a `vscode` setting naming a file that is gone read ok: it warns; (4) the backup
> job's last run was a note when it failed and ok however old: a failed or a stale run (past
> the badge's 48 h) warns; (5) the notifications' line was always ok: it warns when the ledger
> refused a notification, ntfy's settings do not read or the last push failed
> (`Toasts::line_warns`); (6) a hand-written bare `crw` entry read as gone: PATH is asked; (7)
> the log's repeat filter remembered only the line before, so a child failing at every poll
> wrote a line a minute: each line is remembered five minutes. Its note on item 3, the
> installed build beside the running one, is taken too: the build line names the installed
> copy's build when the window runs another.
>
> **Verification.** `cargo test --workspace` (188 in `app`, 409 in `core`, the CLI's suites)
> and `cargo test --workspace --features screenshot` (196 in `app`) pass after the review's
> fixes; both clippy runs and `cargo fmt --check` are clean; rule 3's word check on the staged
> diff printed nothing. A last screenshot run showed the defer route's neutral path. The release
> build and the install run after the record commit, the opener's last step.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The checks run when the tab is first shown and at Check again**, not at the window's
>    start: six children the owner did not ask for at every start; a saved layout gains the tab
>    behind the Detail pane, so nothing runs until it is chosen.
> 2. **The window's lines in the pane, the report's in core**: the lines only the running window
>    knows (its build, signals, meter, mailman, route) are read from the board, the rest is one
>    report for both surfaces.
> 3. **The preflight is the window's own** in the pane (shown when the launch dialog has read
>    it); `crw doctor` runs its own as before.
> 4. **The log names its kinds** (`start`, `notice`, `warning`, `error`, `in front`, `restart`,
>    `quit`, `panic`, `spawn`, `doctor`), writes a notice whole where the status bar cuts it, and
>    remembers each line five minutes, so a failure at every poll is a line in five minutes.
> 5. **No log from `crw`** or the hooks, and none from screenshot runs or tests.
> 6. **A missing `code.cmd` is a note**, since Visual launches find `Code.exe` by its install;
>    a missing extension or `gh` warns, a missing or old `claude` fails.
> 7. **The scratchpad** is `target\tmp\ep28\` (ignored).
>
> **No owner task.** Nothing here needs the owner's hands: the pane and the log arrive with the
> install, and the residual risks go to EP-29's walk. **Next: EP-28a** (operations, the start),
> chained by the autopilot when its gate allows, else from the window. The work commit is
> `fef1e59`; the record commit follows.
