---
ep: 37
title: The installed exe and the window's restart
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-37 — The installed exe and the window's restart

**Size:** M · **Depends on:** EP-19a, EP-19b · **Blocks:** EP-23

## Context

The window and `crw.exe` run from `target\release`. A release build therefore waits for the owner to
quit the window from the tray, and a hook that runs `crw.exe` at the moment of the link makes the
build fail with access denied. The window's headless runs are its children, so quitting it ends
them, and its held chains and chain plans live only in the window (the D-22 addenda of EP-19 and
EP-19a). This is why the gate never chains this project (the tool's own, `Refusal::OwnProject`).
The planning session of 2026-11-08 added this row to end both limits (the D-5 and D-22 addenda of
that date): the exes run from an installed copy in `%LOCALAPPDATA%\clauderoadmapwrangler\bin\`, the
window restarts itself into a newer copy when no headless run of its own works, and its held chains
and chain plans cross the restart in a handover file. This project's briefs may then run as chained
headless runs under `auto`, each ending with its own release build and install.

**What exists.**

- `scripts\install-shortcut.ps1` builds and points the desktop shortcut and the Start-menu entry at
  `target\release\clauderoadmapwrangler.exe`, with the icon written beside it.
- The owner's user settings run `target\release\crw.exe` for the four hooks and the status line
  (EP-12). `signals::install` knows the tool's entries by the program's file name, so an install
  from another folder replaces them. `crw signals` names "this exe" and what an install from it
  would change. `crw doctor` names the tools' versions.
- `board::crw_exe()` is the `crw.exe` beside the window's own exe, which the defer route's hooks
  and the launches' hooks run. `Watch::read_entries` compares the entries with it.
- `paths::repository()` names the repository a build was compiled in, so a build knows its own
  `target\release`.
- `Sessions::stoppable` names the headless runs that quitting would stop. `Board::quit` lets the
  close through, after a question when runs would stop, and waits for a revert.
- `Pilot::held` and `Pilot::plans` hold the chains and the plans. A held chain is read again every
  5 minutes or at the Resume (`read_held`). The gate's result is taken in `Board::gated`, which
  queues the chained start.
- The gate's `Facts::own` comes from `autopilot::is_own(exe, folder)`. `autopilot::standing` shows
  the same condition in the Autopilot section.
- The loopback server (`server.rs`) takes `POST /hook` and `POST /statusline` and answers
  `GET /health`. It answers at once; the window takes the signals at its next pass.
- eframe 0.37 saves the window's state, then calls `App::on_exit`. There is no single instance yet
  (EP-28): a second window fails to bind the port.

**Session start (2026-11-10, 08:11 local time, 13:11 UTC).** Launched from the window as a visual
launch (rule 11): ledger row `45cc1e3b`, visual, EP-37, open (`crw sessions
clauderoadmapwrangler`). The opener suggested Fable at xhigh, and the session runs at Opus 5.5 with
max effort, chosen in the tab's panel. The tree is clean at `33fed21`. No owner task is open and the
inbox is empty.

**Field check.** I read the records back to EP-36's record commit (`33fed21`, 06:02):

- `crash.log` was last written on 2026-11-03, read before;
- `crw notifications`: the newest is from 2026-11-09 21:58, EP-36's own;
- `crw sessions`: this session's row, and the rows of EP-36 and EP-19c still open with no end
  recorded (their tabs were left open, the known gap of risk 15);
- `crw autopilot`: not paused, one private project on, its last line from 2026-11-09 16:59, read
  by EP-36's field check.

Nothing is new, so there is nothing to triage.

**Taken over (2026-11-10, 10:20 local time).** That session ended its turn at 10:19 with the code,
the tests and the records written, waiting for a review subagent of the diff, its context at
434k tokens. The owner then started a second visual launch of EP-37 from the window: ledger row
`c64b956c`, visual, EP-37, open, on Opus 5.5 at max effort chosen in the tab's panel. Its review
subagent stopped at 10:20 with its tab and reported nothing. Asked which session finishes, the
owner chose this one and closed the first tab. This session read the brief, the diff and the two
new modules. It started a fresh review subagent, given only the diff, and ran its own field
check:

- `crash.log` unchanged since 2026-11-03;
- `crw notifications`: the first session's turn over at 10:19, then this session's question;
- `crw sessions`: both EP-37 rows open, and EP-36's and EP-19c's rows ended at 10:20;
- `crw autopilot`: unchanged since 2026-11-09 16:59.

Nothing to triage.

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-10)

**The split.** Read against the code, the note's four parts are more than an M, and the note says
to split at pickup if the handover makes it so. Auto-compaction is off, so a session that fills its
context stops. The owner chose the recommended split: this brief keeps the install and the restart,
and a new M row, **EP-37a**, right after it, takes the handover file, the restart at the gate's
moment, the gate's own-project condition, the opener's words for this project's runs and this
project chained under `auto`. Until EP-37a a restart is refused while a chain is held or a plan is
kept, since nothing carries them yet. The other options were the install alone in this brief, and
everything in one session re-scoped to L.

## In scope

1. **The install** (`crw_core::install`). `crw install` copies the release build of the
   repository it was compiled in (`target\release`: the window's exe and `crw.exe`) into
   `%LOCALAPPDATA%\clauderoadmapwrangler\bin`, or `CRW_HOME\bin` in a trial. An installed exe that
   differs from the build is renamed first to a dated previous copy, which Windows allows for a
   running exe. The previous copies of an earlier install are removed, and one that still runs is
   kept for the next install. An exe installed as built (same size and last write) is left alone.
   Then `crw install` asks the window over the loopback to restart and prints its answer, unless
   `--no-restart` or a trial. *(After the review: each copy is staged beside its place before any
   rename, so a copy that fails changes nothing.)*
2. **The script** builds, runs `crw install`, writes the icon into the install folder and points
   both shortcuts at the installed window, started in that folder.
3. **The hooks' exe.** `crw signals install` writes the installed `crw.exe` once it exists.
   `crw signals` and `crw doctor` name this exe, the installed copy and the exe the entries run.
4. **The restart** (`app::restart`). File > Install and restart, the tray's item and
   `POST /restart` install a build of the window's repository newer than the copy, then start the
   installed window with `--restarted <pid>` when it is newer than the build the window runs (or
   the same build, where the window runs from `target\release`). The window closes a moment later,
   once the loopback's answer has gone out. The new window waits for the old process to end, so the
   layout is saved and the port free. A restart is refused, with every reason, while anything would
   be lost:
   - a headless run of the window's that works or waits (D-13), or a session starting;
   - a visual launch on its way, or a revert;
   - the autopilot reading a gate or starting a chained brief;
   - a held chain or a kept plan (until EP-37a);
   - an open dialog;
   - a screenshot run;
   - *(after the review)* a run being stopped, a session's start not yet in the ledger, a
     disposition being typed.

   A refused request is not remembered: the owner restarts later from the menu or the tray.
5. **The status bar** shows an amber chip while a newer build is built or installed, looked at
   every 30 seconds and every second while the File menu is open. The menu item is disabled
   otherwise, its hover naming why.
6. **The own-project condition** reads the build's repository (`paths::repository()`), not the
   folder that holds the exe, since the exe now runs from the install folder. This keeps the
   gate's behaviour until EP-37a drops the condition.
7. **The records**: rules 6 and 11, the README, a D-29 addendum, the roadmap's rows (EP-37's link,
   the new EP-37a, EP-23's dependencies) and notes, and the module documentation.

## Out of scope

- **EP-37a's parts** (above).
- **Single instance and the start-at-login entry** (EP-28). A restart only waits for the old
  window's end.
- **Carrying anything else across a restart**: a waiting request, a run, the popup's notices, the
  dialogs, the window's visibility. A restart is refused while anything would stop.
- **The app's own icon** (EP-28).
- **Writing the owner's user settings or registry.** `crw signals install` from the installed copy
  is the owner's (rule 7).

## Verification / acceptance

- **Unit tests:**
  - the install's rename and copy on a scratch folder, an exe installed as built left alone, the
    previous copies removed at the next install, one held open kept, a missing build refused;
  - a newer build, a restart due (a newer copy, or the same build elsewhere, never an older copy),
    the `--restarted` flag;
  - the window's standing toward the build and the copy, and the restart's refusals;
  - the loopback's restart request answered by the window, a page's request refused;
  - the tray's item, and the gate's own-project condition by the build's repository.
- **The checks:** `cargo test` with and without `--features screenshot`, both clippy runs,
  `cargo fmt --check`, and a review of the diff (an M brief, rule 6).
- **Trials on scratch folders:** `crw install --no-restart` into a scratch `CRW_HOME`, twice
  (copied, then installed as built), and `crw signals` naming the scratch copy; a copy of the
  script run against scratch shortcut folders.
- **A screenshot run** of the restart's refusal.
- **The release:** `cargo build --release` once the owner has quit the window (it runs from
  `target\release` this once), then `scripts\install-shortcut.ps1`, the first real install.
- **Rule 3's word check** on the staged diff prints nothing.
- **Residual risk.** The session never acts on the owner's window. Rule 13 keeps it off the
  registry and the ledger, and a restart started by the session would be one it could not watch.
  Five limits:
  1. **The real restart.** The tests cover each step up to the start of the new process: the
     refusals, the standing, the loopback request and its answer, the flag and the wait. Four
     things are not exercised:
     - the old window's close let through while it hides in the tray;
     - eframe saving the layout before the process ends;
     - the new window binding the port once the wait is over;
     - the tray icon passing from the old process to the new one.
     The close takes the path Quit has taken at every brief since EP-15 (`quitting`, then
     `ViewportCommand::Close`), and the 10 s wait covers the save and the port. *(Closing round:
     the owner made the first real restart, the move from `target\release` into the copy, and the
     window came back as described: all four were exercised once.)*
  2. **A running exe renamed.** Windows allows the rename of a running image, which the install
     relies on. The tests rename exes that do not run, and hold a previous copy open to stand in
     for one that runs. The first install over the running installed window comes at the next
     release build after this one.
  3. **The hooks.** They ran `target\release\crw.exe` until the owner ran `crw signals install`
     from the copy in the closing round (rule 7); they now run the installed copy. A hook or the
     status line that starts in the instant between an installed exe's rename and its staged
     copy's rename finds no `crw.exe` and fails once, which Claude Code logs and passes over.
  4. **The first start from the install folder.** The window starts in `bin` (the shortcut's
     folder) and finds the repository by the path it was compiled with. Security software may
     look at a new exe location the first time. A taskbar pin made before today still starts
     `target\release`; a window started there offers its move into the copy (the chip).
  5. **Antivirus on the copy.** The install copies two exes and renames two, far from the burst
     loops that two AV products watch for, and the script ran once against scratch folders.

> **Completion note (2026-11-10; claude-opus-5-5/max; two sessions, about 2 h 10 min and 1 h of
> wall time, each with one round of the owner's).** Two visual sessions in the VS Code extension,
> both launched from the window (ledger rows `45cc1e3b` and `c64b956c`, visual, EP-37).
>
> - **The first**, from 08:11 local time, covered the reading and the field check, the split at
>   pickup, this brief, the code and its tests, the records, the scratch trials and a screenshot
>   run. It ended its turn at 10:19 waiting for its review subagent, with 434k tokens of context.
> - **The second**, from 10:20, took over on the owner's choice. It ran a fresh review of the
>   diff, fixed the review's seven findings, repeated the checks, the trials and the screenshot
>   run, and held the closing round and the release.
>
> **Built.**
>
> - `crw_core::install`:
>   - the staged copy, the rename to a dated previous copy and the removal at the next install;
>   - the restart's due test, the `--restarted` flag and the wait for the old window;
>   - the settle of a build the window takes on its own.
> - `app::restart`: the standing and the status bar's chip, the refusals, File > Install and
>   restart and the tray's item, the loopback's request answered at the window's pass.
> - The loopback route `POST /restart` (`server.rs`, `signals::ask_restart`), which carries the
>   install folder, and `Watch::take_restarts`.
> - The console:
>   - `crw install`;
>   - `crw signals` and `crw doctor` naming this exe, the installed copy and which exe the entries
>     run;
>   - `crw signals install` writing the installed copy once it exists.
> - The gate's own-project condition by the build's repository (`autopilot::is_own`).
> - `process::running`, shared by the visual launch's lock files and the restart's wait.
> - `scripts\install-shortcut.ps1`: the build, the install, and the icon and both shortcuts in
>   the install folder. A trial must name scratch shortcut folders.
> - The records:
>   - rules 6 and 11, the README and the D-29 addendum;
>   - the roadmap's rows: EP-37's link, the new EP-37a and EP-23's dependencies;
>   - the notes of EP-37a and EP-23, and the module documentation.
>
> **The review (rule 6).** A review subagent was given only the diff, since the first session's
> reviewer had stopped with its tab. It found nothing that breaks the main path, and seven
> findings, all fixed:
>
> 1. **A run being stopped.** Such a stop lasts up to 30 s; an autopilot cap's is one. The run
>    did not refuse a restart, whose exit would have killed it before its end was recorded and
>    the gate read it. `Sessions::stopping` now refuses.
> 2. **A trial reached the owner's window.** `crw install` under `CRW_HOME` asked the window on
>    the default port, which would then have installed into the owner's folder and restarted.
>    Three guards now:
>    - a trial asks no window;
>    - the request names the folder it installed into, and the window refuses any other than its
>      own;
>    - the script refuses a trial that names no scratch shortcut folders.
> 3. **A late request.** One the server had answered with a 504 was still acted on at the
>    window's next pass. A request whose caller stopped waiting is now dropped.
> 4. **State kept only in memory.** Two things were lost to a restart: a session's start not yet
>    in the ledger (EP-15b), whose revert would then go back to the last recorded brief, and a
>    disposition being typed. Both now refuse, and EP-37a's note says the handover could carry
>    them.
> 5. **A half install.** An install that failed half way could leave mismatched exes, and a
>    failed putting-back no `crw.exe` at all, silently. Each copy is now staged beside its place
>    before any rename, so a copy that fails changes nothing. A rename that fails puts the exe
>    back and says what was installed, including a putting-back that failed.
> 6. **A build still being linked.** The window judged a build by its last write alone, so it
>    could take one still being linked. It now takes a build only once the exes have been left
>    alone for 5 s. A decided restart whose new process has ended at once keeps the old window.
> 7. **A promised retry.** The help text, the script, rules 6 and 11 and the README promised a
>    restart once the runs end, which nothing does. A refused request is not remembered, and the
>    words now say so; the files are looked at again at each request. EP-37a's note says where a
>    chained brief's install is taken.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed 471, and with `--features screenshot` 479: EP-36's 451 and
>   459, plus 20 new in each run. They cover:
>   - the install on a scratch folder: copy, rename and removal; an exe installed as built; a
>     held previous copy; a missing build; a copy and a rename that fail; the settle;
>   - the restart's due test, its flag and the wait;
>   - the standing and the chip;
>   - the refusals: a held chain, a plan, a dialog, a screenshot run, a start not yet recorded, a
>     typed disposition, a run being stopped;
>   - the loopback's request answered and a page's refused, a late and a foreign request;
>   - the tray's item, and the gate's own-project condition by the repository.
>
>   Both clippy runs and `cargo fmt --check` are clean.
> - **Trials on scratch folders**, under a scratch `CRW_HOME`:
>   - `crw install --no-restart` copied, then found both exes installed as built;
>   - with the copies aged, it renamed them to dated previous copies, which the next install
>     removed;
>   - without the flag it printed that a trial asks no window;
>   - `crw signals` and `crw doctor` named the scratch copy and only read the real settings;
>   - a copy of the script, running the debug `crw`, refused a trial without scratch shortcut
>     folders, then pointed both scratch shortcuts at the installed window, started in `bin`, with
>     the icon there.
> - **A screenshot run** of `restart` at 1600×1000, before and after the fixes. The status bar
>   says "Not restarted: a headless run of the window's works or waits: harbor EP-4 (waiting for
>   an answer since …); a screenshot run restarts nothing". Kept in the session's scratchpad.
> - **The release.** Built once the owner had quit the window, which ran from `target\release`
>   this once. Then `scripts\install-shortcut.ps1` made the first real install: both exes copied
>   into `%LOCALAPPDATA%\clauderoadmapwrangler\bin`, no window to ask, the icon and both
>   shortcuts there. `crw doctor` named the copy, built 10:59.
> - **The first real restart** (the owner's hands, decision 6). The window was started from
>   `target\release`, and File > Install and restart moved it into the copy; it came back as
>   described. Afterwards the window's process runs from the install folder and listens on the
>   loopback port.
> - **The hooks** (the owner's hands, decision 7). `crw signals install` from the copy; `crw
>   signals` now prints each hook and the status line running the installed copy, `--inbox`
>   kept.
> - **Rule 3's word check** found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-10).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The split at pickup (first session) | EP-37 keeps the install and the restart; a new M row takes the rest | EP-37a's row and note |
> | 2 | Which session finishes, the first having stopped mid-review | This session; the first tab closed | The takeover above |
> | 3 | The release build and the first install | Quit the window, build and install | Built and installed |
> | 4 | The cross-project memory on crw's hooks | Update the file and its index line | The environment repository's memory file on the hooks and its index line, left uncommitted there |
> | 5 | Commit and push | Commit and push | Two commits, pushed |
> | 6 | The first real restart | Done now, as described | No owner task |
> | 7 | The hooks pointed at the installed copy | Done now | No owner task |
>
> **Routine choices (the sessions' call, logged).**
>
> 1. **`crw install` does not build**, since a build would replace its own running exe; the
>    script builds.
> 2. **The timing.** The old window closes 0.4 s after deciding, once the loopback's answer has
>    gone out. The new one waits up to 10 s for the old process to end.
> 3. **A previous copy that cannot be removed is kept** for the next install, and the report
>    names it.
> 4. **The window takes a build after 5 s of quiet.** `crw install`, which the script runs once
>    the build has ended, does not wait.
> 5. **A refused request is not remembered.** A restart that came by itself later could surprise
>    the owner in the middle of a dialog. EP-37a takes a chained brief's install at the gate's
>    moment.
> 6. **A start not yet in the ledger refuses a restart**, rather than being dropped, so a tab
>    never prompted holds the restart until its first prompt or its close.
> 7. **Two guards for a trial.** It asks no window, and the window also refuses a request that
>    names another folder, in case an older `crw` sends one.
> 8. **The chip is amber**, from the window's palette (EP-15a), its hover saying what Install
>    and restart does.
>
> **Notes for later sessions.**
>
> - The release build now runs with the window up, since the window runs from the install
>   folder. A visual session's script asks the window to restart: the window refuses while a
>   headless run of its own works, a dialog is open or a session's start awaits its first prompt,
>   and File > Install and restart takes it later.
> - `crw doctor` names the installed copy and which exe the hooks run. The previous copies
>   (`*.previous-*.exe`) in the install folder are removed at the next install.
> - EP-37a picks up the handover, the gate's moment, the own-project condition, the opener's
>   words and this project's permissions. Its note now lists what this brief found.
>
> No messages for other projects. Decision 4 wrote into the environment repository's memory on
> the owner's yes, and
> no owner task remains: the owner made the first restart and pointed the hooks during the
> round. **Next: EP-37a** (the handover and the chained restart), launched from the window.
