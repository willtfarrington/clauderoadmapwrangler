---
ep: 37a
title: The handover and the chained restart
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-37a — The handover and the chained restart

**Size:** M · **Depends on:** EP-37 · **Blocks:** EP-23

## Context

EP-37 installed the exes into `%LOCALAPPDATA%\clauderoadmapwrangler\bin` and taught the window to
restart into a newer installed build (File > Install and restart, the tray's item, `crw install`'s
`POST /restart`). Its restart is refused while anything would be lost, a held chain or a kept
chain plan among them, since nothing carried them yet. It was split at pickup: this brief takes
the parts that let a chain of this project's briefs survive its own release build (the D-22 and
D-5 addenda of 2026-11-08).

**What exists.**

- `Pilot::held` (`app::autopilot::Held`: project, the brief whose run completed, the log's words)
  and `Pilot::plans` (`crw_core::autopilot::Plan`, whose rows carry a `rules::Resolution`) live in
  the window only. `read_held` reads a held chain's gate again every 5 minutes or at the Resume;
  `Board::gated` takes a gate's result: queued for the start, held, or ended.
- `Board::install_and_restart` installs a settled build newer than the copy, starts the installed
  window with `--restarted <pid>` and closes 0.4 s later; `main` waits for the old process to end
  before the window opens. `restart::Standing::Installed` says the copy is one to restart into.
- A headless run of the window's refuses the restart, so the install step of a chained run's brief
  is always refused: the gate's moment is where it can be taken.
- The gate refuses the tool's own project (`Refusal::OwnProject`, `Facts::own`,
  `autopilot::is_own` against `paths::repository()`), and the Autopilot section shows that verdict
  (`autopilot::standing`).
- The opener (`crw_core::prompt`) says nothing of this project's build.
- No serde form exists for `Plan`, `Planned`, `Stop`, `Picked`, `Resolution` and its parts, `EpId`
  or `Size`; core has `serde` with derive and `serde_json`; chrono has no serde feature.

**Session start (2026-11-10, 11:25 local time, 16:25 UTC).** Launched from the window as a visual
launch (rule 11): ledger row `5e9b0834`, visual, EP-37a, open (`crw sessions
clauderoadmapwrangler`). The opener suggested Fable at xhigh, and the session runs at Opus 5.5
with max effort, chosen in the tab's panel. The tree is clean at `75ec010`. No owner task is open
and the inbox is empty.

**Field check.** I read the records back to EP-37's record commit (`75ec010`, 11:15):

- `crash.log` was last written on 2026-11-03, read before;
- `crw notifications`: since the commit, one at 11:17, "a manual session worked 7 min and waits for
  a prompt"; with the two at 11:09 it was EP-37's own visual session (row `c64b956c`), which the
  window had restarted under. **Odd:** the window named a visual session "manual";
- `crw sessions`: this session's row; EP-37's second row completed; its first row (`45cc1e3b`)
  still open with no end recorded, the known gap of risk 15;
- `crw autopilot`: not paused, one private project on, its last line from 2026-11-09 16:59, read
  by EP-37's field check.

**Triage of the odd line.** A window learns a session's mode from the ledger only at the session's
start hook (`Watch::record`, `Sessions::set_mode`). A session it first meets through the poll or a
later hook keeps the default, manual, although its ledger row says visual. Every window started
while a session is open does this, and a restart at the gate's moment would do it after each
chained run, in the notifications and in the gate's line for idle sessions. The fix is at most S
and fits this brief, which is about what a window loses across a restart, so it is rolled in
(part 6 below).

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-10)

**A chain's end does not restart the window.** When a chain ends for good while a newer build is
installed, the window stays, its popup in front of the owner, and the amber chip stays until File >
Install and restart or the tray's item takes it. The other options were a restart at the end with
the handover carrying the end's words for the new window's popup, and a restart with the popup
lost.

## In scope

1. **The handover** (`crw_core::handover`, new). A JSON file, `handover.json` in the local folder
   (`CRW_HOME` in a trial), with a version, the writer's process id, the time written, the held
   chains, the chain plans and whether the new window opens hidden. It is written to a temporary
   name and then renamed into place. `take` reads and deletes it:
   - the window started with `--restarted <pid>` takes a file its old window wrote;
   - any other file is deleted too, and the window names what was lost: another writer, another
     version, or one that does not read.

   Serde forms are added for `Plan`, `Planned`, `Stop`, `Picked`, `Resolution`, `Choice`,
   `Source`, `Downshift` and `LaunchMode`, for `Size` by its variant, and for `EpId` by its text.
   `Held` moves into `crw_core::autopilot` beside `Plan`. *(After the review: another restart's
   file whose old window still runs is left for its own new window; a refused delete is tried
   again for a moment; a frozen version 1 file is a test.)*
2. **The restart carries it** (`app::restart`).
   - The refusals for a held chain and a kept plan go.
   - Before the installed window starts, the window writes the handover. When the start fails or
     the new window ends at once, the file is removed and the window keeps what it holds.
   - The restarted window takes the file at its start. Its held chains go on: each gate is read
     again once the first poll of live sessions has answered (or failed), so that an open session
     counts, or after two minutes without an answer. Its plans are kept. The start's notice says
     what was carried.
   - A restart at the gate's moment from a window hidden in the tray opens the new window hidden.
     A restart the owner asks for opens it shown, as EP-37's does.
   - *(After the review.)* A carried gate also waits for the first files read, which gives it the
     project's folder; a chain the missing folder ends lets its held entry go with its plan. A
     dropped handover is said in the start's notice and the chain log. A session open with no
     prompt yet refuses a restart in every registered project, not only those whose revert is on.
3. **The gate's moment** (`app::autopilot`). A gate that passes or holds while the installed copy is
   one to restart into (`Standing::Installed`, looked at again then) restarts the window when
   nothing else stands in the way. A passing chain is first held "for the restart" and logged, and
   the new window's gate starts the next brief. Two limits:
   - The gate's moment installs nothing: only a build installed by `crw install` is taken.
   - When the restart is refused (another headless run works, a dialog is open, …), the chain goes
     on in this window as before and the chip stays.

   A new window that ends at once keeps the old one, which then chains without the restart until
   the owner's own restart or a newer install. A chain that ends does not restart the window (the
   owner's choice). *(After the review: only another build than the window runs is taken; once
   the restart is decided the old window reads no gate again; an ended run's tab that offers
   Commit holds a gate's restart.)*
4. **The gate without the own-project condition.** `Refusal::OwnProject`, `Facts::own`,
   `Ask::repository`, `autopilot::is_own` and `standing`'s repository parameter go, and the
   Autopilot section's verdict line goes with them. The tests are updated.
5. **The opener's words for the tool's own project** (`crw_core::prompt`). When the project's folder
   is the repository the build was compiled in, the opener adds a paragraph before the rules. It
   says that the release build runs with the window up, since the window runs from its installed
   copy, and that the brief's last step is the install, `pwsh -File
   scripts\install-shortcut.ps1`. How the restart comes depends on the mode:
   - **Headless:** the window refuses the install's request while the run works, and takes the
     restart at the gate's moment after the run's end.
   - **Visual:** the install asks the window to restart, which it refuses while one of its own
     headless runs works, a dialog is open or a session's start awaits its first prompt; File >
     Install and restart takes it later.
6. **A session's mode from the ledger** (rolled in from the field check). A session the window
   first meets through the poll or a hook other than its start takes the mode of its latest ledger
   row. Each session is looked up once. *(After the review: only an open row that is not a
   headless run's, as the start hook's rule has it.)*
7. **The records**:
   - rules 6 and 11 of `CLAUDE.md`;
   - the README;
   - a D-22 addendum (the gate's moment, the own-project condition gone) and a D-29 addendum (the
     handover);
   - the roadmap's row and notes, EP-23's note, and the module documentation.

## Out of scope

- **Carrying anything else across a restart.** A session's start not yet in the ledger and a
  disposition being typed still refuse the restart, as do the dialogs. The handover could carry
  the first two later. The popup's notices and the ended runs' tabs are not carried: their ledger
  rows remain.
- **This project's permission mode and autopilot switch.** `auto` is set by the owner through
  EP-19b's control (the offer when the box is ticked, the pin beside Permissions, or `crw set
  clauderoadmapwrangler permission_mode auto`), rule 13. The session asks in the closing round.
- **The first real chained restart.** It is a hands-only task for the owner (the brief note's
  acceptance).
- **Single instance and the start-at-login entry** (EP-28).

## Verification / acceptance

- **Unit tests:**
  - the handover: write and take, the file deleted; another writer's file, another version and a
    file that does not read, each deleted and named; a plan with its resolution and a held chain
    read back as written;
  - the restart: no refusal for a held chain or a plan; the handover written at a restart and
    removed when the start fails; the restarted window taking the file, its held chains waiting for
    the first poll, its plans kept;
  - the gate's moment: a passing gate with a pending install holds the chain for the restart and
    restarts; a refused restart queues the start as before; a chain's end restarts nothing;
  - the gate and the standing without the own-project condition;
  - the opener's paragraph for the tool's own project in both modes, and none for another project;
  - a session first seen through a later hook takes its ledger row's mode.
- **The checks:** `cargo test` with and without `--features screenshot`, both clippy runs,
  `cargo fmt --check`, and a review of the diff by a subagent given only the diff (an M brief,
  rule 6).
- **A screenshot run** of the Autopilot section with a held chain and a plan, showing no
  own-project line, and a run of the restart's refusal without the held-chain reason.
- **The release:** `cargo build --release` with the window up, then `pwsh -File
  scripts\install-shortcut.ps1`, the install asking the owner's window to restart (rule 6).
- **Rule 3's word check** on the staged diff prints nothing.
- **Residual risk.** The session never acts on the owner's window beyond rule 6's install. Rule 13
  keeps it off the registry and the ledger, and the gate's moment comes only after a headless run
  of the window's, which this visual session is not. Six limits:
  1. **The real chained restart.** The tests cover each step with stand-ins for the process start:
     the hold, the handover written, the window's close decided, the take, and the carried
     chain's wait for the first poll. Four things are not exercised:
     - the old window's close at the gate's moment;
     - the new window taking the file after its wait for the old one's end;
     - its first poll, then the carried gate read and the chained start, which asks the CLI;
     - the run starting in the new build.

     The close and the wait are EP-37's, which the owner's first restart exercised once. The
     take runs before the window's first frame. The first real chained restart is the owner's
     task.
  2. **The hidden start.** A gate's restart from a window hidden in the tray hides the new one
     through the tested hide path, but from the creation closure, before the first frame. The new
     window may show for a frame, or eframe's first-frame show may win and leave it shown. Either
     is harmless: the window is only less quiet.
  3. **Two builds, one form.** The first real handover is written by this build or a later one
     and read by the next. A brief that changes `Held`, `Plan`, `Planned`, `Resolution` or their
     parts must keep the old form readable or raise the version (the module's documentation and
     the D-29 addendum say so). A file that does not read is deleted and named, so a chain is
     lost loudly, never taken up wrong.
  4. **A restart under the owner's hands.** Only the dialogs, a typed disposition, a start not
     yet recorded and the runs refuse a restart. A gate's restart while the owner reads the
     window, a menu open or the filter being typed in, closes and reopens it within a few
     seconds, the layout saved. It comes only between two runs of a chain with a newer build
     installed, which today means a chain of this project's briefs.
  5. **What a restart drops.** The ended runs' tabs and the popup's notices are not carried; their
     ledger rows and the notifications' log remain. EP-23's note asks the walk whether the owner
     misses the tabs.
  6. **The mode lookup.** It is tested on a scratch ledger. The real case is this session's own
     window after the install's restart: its notifications should call this session visual.
     *(Closing round: they did, at 12:26:42, 46 s after the restart.)*

> **Completion note (2026-11-10; claude-opus-5-5/max; about 1 h 20 min of wall time, a question
> at pickup and one closing round).** One visual session in the VS Code extension, launched from
> the window (ledger row `5e9b0834`, visual, EP-37a), from 11:25 local time.
>
> **Built.**
>
> - `crw_core::handover`: the file, its write through a temporary name, its take (read and
>   delete, another restart's file left while its old window runs, a refused delete tried again),
>   its words. Serde forms for the plan, its rows and stop, `Resolution` and its parts, `Size`
>   and `EpId`; `Held` moved into `crw_core::autopilot`.
> - `app::restart`: the handover written before the installed window starts and removed when the
>   start fails or the new window ends at once; `Asked` (the owner's restart installs a settled
>   build and opens shown, the gate's takes only an installed build and keeps a hidden window
>   hidden); `restart_at_gate`, `pending_install` (another build than the window runs, not one
>   whose window ended at once); the gate-only refusal for an offered Commit; `take_handover`,
>   whose notice says what was carried or lost.
> - `app::autopilot`: the gate's moment in `gated` (a passing chain held for the restart, a
>   holding one carried, an ending one left alone, a refused restart logged and the chain going
>   on); the carried chains' wait for the first poll and the first files read; no gate read once
>   a restart is decided; a chain the missing folder ends lets its held entry go.
> - The own-project condition removed from the gate, the standing, the CLI and the section.
> - `crw_core::prompt`: the paragraph on the tool's own build in both modes (`Parts::own`,
>   `build`, the `{build}` placeholder, `is_tool`).
> - `Watch::learn_modes` and `sessions::recorded_mode`, and `unrecorded_starts` widened to every
>   registered project.
> - The records: rules 6 and 11, the README, the D-22 and D-29 addenda, the roadmap's row and
>   EP-23's note, the v2 notes, OT-20 and the module documentation.
>
> **The review (rule 6).** A subagent given only the diff found eight issues and two minor
> ones, all fixed and tested:
>
> 1. **A carried chain ended falsely.** A poll answering before the first files read let the gate
>    find no folder, end the chain and drop its plan while the held entry stayed. The carried
>    wait now waits for the files read too, and such an end lets the held entry go.
> 2. **A lost handover said in a notice overwritten at once.** The start's notice now says what
>    was carried or lost in one line, and a lost one is a chain-log line.
> 3. **Gaps that an unasked restart makes matter.**
>    - A session open with no prompt yet in a project whose revert is off lost its ledger row: it
>      now refuses a restart, as a revert project's start did.
>    - An ended run's offered Commit now holds a gate's restart.
> 4. **A second gate read in the 400 ms before the close.** None is read once a restart is
>    decided.
> 5. **The take's edge cases.** A file that does not read is deleted too, a refused delete is tried
>    again, and the false words of a later take are gone.
> 6. **A window started by hand during a restart's wait deleted the handover.** Another restart's
>    file whose old window still runs is now left alone.
> 7. **The mode lookup took a closed or headless row.** It now takes only an open row that is not
>    a headless run's, as the start hook does.
> 8. **Tests that could not fail.** Two were removed, and a frozen version 1 file is now a test.
>
> The two minor ones: the headless paragraph promised a restart after every run, and now names
> when it comes; and a gate never restarts the window into the build it runs, which closes a loop
> a differently spelled exe path could open.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed 483, and with `--features screenshot` 491: EP-37's 471 and
>   479, plus 12 new in each run. They cover the handover's write, take and deletion, the frozen
>   file, the restart's handover and its removal, the gate's moment (passing, refused, ending,
>   holding), the carried wait, the opener's paragraph, the refusals and the mode lookup. Both
>   clippy runs and `cargo fmt --check` are clean.
> - **A screenshot run** (`autopilot,restart` at 1600×1000, a scratch home). The Autopilot
>   section shows a held chain and a plan with no own-project line, and the status bar says "Not
>   restarted: a headless run of the window's works or waits: harbor EP-4 (…); a screenshot run
>   restarts nothing". Kept in the session's scratchpad.
> - **The release.** Built with the window up. `scripts\install-shortcut.ps1` renamed both
>   running installed exes to dated previous copies and copied the new build in, which exercised
>   EP-37's second residual risk. The window answered "restarting into the build of 2026-11-10
>   12:25" and came back at 12:25:56 from the install folder. `crw doctor` names the copy and the
>   hooks running it.
> - **The mode lookup, live.** At 12:26:42 the restarted window called this session "a visual
>   session"; EP-37's restart had called it manual.
> - **Rule 3's word check** found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-10).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | A chain's end with a newer build installed (pickup) | No restart at a chain's end | The window stays, its popup in front; the chip says the build waits |
> | 2 | Commit and push | Commit and push | Two commits, pushed |
> | 3 | This project's permission mode `auto` | Leave it to the box's offer | Nothing written; OT-20 walks through it with the first real chained restart |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The field check's odd line was rolled in** (D-13's rule: it fits a brief about what a
>    window loses across a restart, and it is at most S).
> 2. **The form.** The handover is JSON (`serde_json` is already a dependency; chrono has no serde
>    feature, so the time is text). A plan carries its resolution as it was, for the section's
>    words. A row left alone is resolved again at the gate anyway (EP-19a).
> 3. **Nothing is written when nothing is carried**, and a window with something to carry and no
>    local folder does not restart.
> 4. **The hidden start** uses the tested hide path at the window's creation, not a viewport
>    built invisible, whose first frame eframe may treat differently.
> 5. **The carried wait gives up after two minutes**, as the toasts' settling does, so a broken
>    poll never holds a chain forever.
> 6. **A refused restart at a passing gate is logged once**; one at a held chain's rereading every
>    five minutes is not, so the log does not fill.
> 7. **The opener's paragraph stands before the rules**, worded at each choice of mode, so the
>    dialog's switch and a chained draft both get the right words; templates name it `{build}`.
> 8. **The widened refusal applies to the owner's restart too**, since its loss is silent; an
>    offered Commit holds only the gate's, since the owner restarting sees the tab.
>
> **Notes for later sessions.**
>
> - A brief that changes `Held`, `Plan`, `Planned`, `Resolution`, `Choice`, `Source`,
>   `Downshift`, `Picked`, `Size` or `EpId` keeps the handover's version 1 readable (the frozen
>   test fails otherwise) or raises `handover::VERSION`.
> - The gate's moment restarts the window only into another build than it runs, only between
>   runs, and never at a chain's end; its log lines are `held … for the window's restart into the
>   installed build` and `the restart into the installed build waits: …`.
> - A session open with no prompt yet, in any registered project, now holds the owner's restart
>   too: a tab left unprompted is closed or prompted first.
>
> No messages for other projects. OT-20, the first real chained restart, is the owner's; no brief
> builds on it, so it is not gated. **Next: EP-20** (inbox delivery), launched from the window.
