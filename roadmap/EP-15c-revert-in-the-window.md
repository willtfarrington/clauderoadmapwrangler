---
ep: 15c
title: Revert in the window
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-15c — Revert in the window

**Size:** M · **Depends on:** EP-15b · **Blocks:** EP-16

## Context

EP-15b recorded each session's start in the ledger (schema 4) and built the revert in `crw_core`
with `crw revert`. EP-15b was split at pickup (the owner's choice, D-4), and this brief puts the
revert in the window, on that engine. `revert::choose` picks the session, `plan` reads the project
and says what a revert would do or why it is refused, and `revert` copies the work out and puts
the project back. `Plan::lines()` and `Done::lines()` are the words `crw revert` prints.

**Session start (2026-11-04, 03:16 UTC).** Started from `DATA\clauderoadmapwrangler.code-workspace`
in the VS Code extension. The tree holds the owner's own ticks of OT-10, OT-11 and OT-12 in
`roadmap/owner-tasks.md`, uncommitted. No inbox message is open, and no owner task. The model is
Opus 5.5 and the effort max. The window runs, so the release build waits for the owner to quit it
from the tray.

## In scope

1. **The three places** (`sessions`, `list`). Each opens the preview and changes nothing.
   - An ended session's tab: Revert… beside Resume (`Live::session_id`).
   - A ledger card of the Sessions pane's All tab whose row has a session id (`ledger_line`), so
     that a run whose tab a crash took keeps its Revert.
   - A project's row menu: Revert the latest session… (`list.rs`'s `context_menu`).
   - A project whose `revert` switch is off, or whose folder is missing, has them disabled, and
     the hover says why.
2. **The preview** (`app::revert`), a dialog like the Quit question (`Board::quit_dialog`).
   - A plan asks git about six children, each in its turn (about two seconds), so it is read on
     a worker thread, as the visual launch is, and the dialog shows a spinner meanwhile.
   - `Plan::lines()` in a scroll area: the commits and files in the code face, a refusal in the
     window's red.
   - Then Revert in the window's red and Cancel; or Close alone when the plan is refused or the
     ledger holds no such session.
   - Escape and Cancel close it, a click beside it does not, and nothing closes it while a
     revert runs.
3. **The window's own refusals**, which `crw revert` cannot see:
   - its own runs in the project: one that runs or stops, a start or a resume waiting its turn,
     and an ended run whose deferred request waits for the owner's answer;
   - a start on its way there: a visual launch on its way to VS Code, or a session's start being
     read by the watch;
   - the sessions the hooks and the poll report working or waiting there, with the idle ones
     named for the owner to close; while the poll of live sessions has not answered, or fails,
     the window cannot tell, so it refuses as `crw revert` does;
   - a screenshot run, whose projects are the tracked fixtures: its Revert opens a canned plan,
     and its confirmation reverts nothing.
4. **The revert**, on the worker thread. At the owner's word the plan is read again, with the
   window's refusals as they stand at that moment. The revert runs only when the plan is the one
   the owner saw: the same target, commits, files, start, ticks and sessions, and no new
   refusal. Otherwise the dialog shows the new preview and nothing changes. (This brief's
   addition: a preview gone stale could leave work made after it uncopied.) Quitting waits while
   a revert runs.
5. **After a revert**: the dialog shows `Done::lines()` with Close; the files and git are read
   again; the All tab's rows go stale; and the status bar says where the work is kept.
6. **Records.** The module's documentation; a D-11 addendum (the window's Revert, its refusals,
   the plan read again at the owner's word); a D-25 addendum (the places and the dialog); the
   README's paragraph; EP-16's note; the owner task; the completion note.

## Out of scope

- The engine's rules (EP-15b), beyond what the window needs: `Plan` compared field by field and
  a constructor for a plan that no git was asked for (a picture's or a test's).
- Undoing a commit the remote has (v2 notes).
- A Revert for a session with no session id: a visual launch no session took up, or a session
  outside the registry.
- Opening the copy's folder from the dialog.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`,
  `cargo build --release` with the window quit from the tray, then the install script.
- Unit tests, on scratch repositories in temp folders and never the owner's projects (rule 13):
  - pressing the dialog's Revert in a bare context reaches the confirmation, and a refused
    preview offers Close alone (`testing::Screen`);
  - a ledger card's Revert… reaches the preview, and a disabled one does not;
  - the window's refusals from its runs, its starts, the sessions reported and the poll;
  - the worker's plan and revert of a scratch repository through the window's path, and a
    project changed after the preview refusing the stale plan;
  - a plan read again is as seen until the project changes (`crw_core`).
- Screenshot runs over the fake portfolio with a canned plan, in both themes: the dialog ready,
  refused and done, and the All tab's cards with their Revert…; each picture looked at.
- The real dialog can be judged only on the owner's screen (rule 5): an owner task.
- Rule 3's word check prints nothing on the staged diff.

> **Completion note (2026-11-04; claude-opus-5-5/max; about 1 h of wall time with the owner's
> answers).** The window reverts a session behind its preview. An ended session's tab, a ledger
> card of the Sessions pane and a project's row menu open a dialog that reads the plan on a worker
> thread. Revert reads the plan again and runs only if nothing changed since the preview.
>
> **Built.**
>
> - `app::revert` (new): the `Dialog` (reading, ready, reverting, done, failed) and its worker
>   (`read_plan`, `confirm`); `availability` and `revert_button`; `seen_view`, the sessions the
>   hooks and the poll report; and the board's `revert_view`, `open_revert`, `confirm_revert`,
>   `close_revert`, `receive_revert` and `revert_dialog`.
> - `sessions`: `Sessions::at_work`, the window's runs, starts and deferred runs in words.
>   Revert… sits at the end of an ended tab's footer, and a small Revert… on a ledger card's chip
>   line, or on a line of its own in a narrow pane.
> - `list`: "Revert the latest session…" at the foot of a row's menu.
> - `board`: `Action::{OpenRevert, ConfirmRevert, CloseRevert}`; the dialog over the window; keys
>   off while it is open; `Closing::Wait`, and `quit` waiting while a revert runs.
>   `visual::opening_in` and `watch::reading_start` feed the refusals.
> - `crw_core::revert`: `Plan` derives `PartialEq`; `Plan::new`, a plan no git was asked for;
>   `Plan::is_as_seen`; the start's single uncommitted file worded in the singular.
> - `screenshot`: `revert=<project>[:refused|:done]`, `sessions=<project>` and
>   `transcript=<project>`. `testing::text_at` finds a button by its label.
>
> **Verification.**
>
> - `cargo test`: 357 tests; with the screenshot feature, 359. Of these, 7 are new. Both clippy
>   runs and `cargo fmt --check` are clean.
> - The tests:
>   - the preview's Revert and Cancel pressed in a bare context, and a refused plan with Close
>     alone;
>   - a ledger card's Revert…, enabled and disabled;
>   - the window's runs, starts and deferred runs; the sessions reported, the idle ones named,
>     and the poll unheard or failing;
>   - the worker on a scratch repository: the window's refusals in the plan; a file made after
>     the preview turning the confirmation into a new preview; then the revert, with HEAD back,
>     the new files gone, the work copied out and the ledger's event;
>   - a project with no session, and a plan read again that is as seen until the project
>     changes (`crw_core`).
> - Screenshot runs, each picture looked at: the preview ready (light, and dark at 1100 wide),
>   refused (dark) and done (light); harbor's ledger cards in the narrow pane (Revert… on its own
>   line) and in a 2300-wide window (at the chip line's end); quarry's ended tab with its Revert….
> - The release build and the install script ran with the window quit. `crw roadmap .` reports
>   0 problems. Rule 3's word check printed nothing.
>
> **Owner decisions.**
>
> | Decision | Outcome |
> | --- | --- |
> | Quitting the window for the release build | Done by the owner from the tray. |
> | The four rules beyond the brief notes (the plan read again at Revert, the refusal while the poll has not answered, a deferred run's refusal, quitting waits) and the records (the D-11 and D-25 addenda, the README, EP-16's note) | Kept as written (recommended). |
> | The owner's uncommitted ticks of OT-10, OT-11 and OT-12 | Committed with EP-15c (recommended). |
> | The on-screen check | Left to OT-13, gated before EP-16. |
> | Commit and push | Committed and pushed (recommended); the row's hash is recorded in a commit of its own. |
>
> **Routine choices.**
>
> - The window's refusals reach the plan two ways. A session at work goes to `Context::working`,
>   which the engine follows with "revert once it has ended". The others (a deferred run, a visual
>   launch on its way, a start being read, the poll) are added to `Plan::refusals` as they are
>   said, so the engine's API is unchanged.
> - A plan is as seen when every field but the copy's folder is equal, the ticks and the
>   sessions named included. The previewed plan is the one reverted, so the work goes to the
>   folder the preview named.
> - A screenshot run's Revert opens a canned plan, and its confirmation reverts nothing: its
>   projects are the tracked fixtures, inside this repository.
> - A tab open with no prompt yet is not named among the idle sessions, since its context holds
>   none of the work.
> - The canned plan's paths start `%LOCALAPPDATA%\`, so no path of the machine shows (D-3).
>
> **Handed on.** Nothing. **Next:** EP-16.
