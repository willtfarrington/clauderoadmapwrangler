---
ep: 35
title: Resume in VS Code
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-35 — Resume in VS Code

**Size:** S · **Depends on:** EP-11, EP-13 · **Blocks:** EP-23

## Context

The D-16 addendum written at EP-16: a run that a crash or a quit window cut short, and a visual or
manual session, are resumed from their ledger rows in the project's VS Code window through the
extension's `session=<id>` link. The window's own Resume (EP-13) takes only its own headless runs,
from their tabs, and a run's launch choices die with the window that started it. Today the way back
is `claude --resume` in a terminal.

**The extension's link** (its 2.1.318 `/open` handler, read at pickup). It reads `session` and
`prompt`. An id that is not a UUID is dropped without a word. Otherwise `primaryEditor.open` calls
`createPanel(session, prompt)`:

- a session already open in a tab is brought forward, and a prompt is not applied ("Session is
  already open. Your prompt was not applied — enter it manually.");
- one the sidebar holds is shown there, and a remembered tab is revived;
- any other id gets a new tab bound to it.

The extension's own list of past sessions leaves out the SDK's sessions (`entrypoint` `sdk-cli`,
`sdk-ts` or `sdk-py`) and daemons. So whether a headless (`-p`) run's id opens in a tab is what the
first check finds.

**What exists.**

- `visual::open` (EP-11) opens the project's own workspace file, waits up to 60 s for the
  extension's lock file to show the project's window, then hands the link to `Code.exe --open-url
  --`. The link escapes `%`, `&` and `+` twice.
- The Sessions pane's All tab lists the ledger's recent rows as cards (`sessions::ledger_line`). A
  card whose row has a session id has its Revert… (EP-15c). A row whose session the window holds,
  or which the hooks or the poll show open, is not listed again.
- The ledger (D-26's EP-12 and EP-13 addenda). A session earns its row at its first work, and only
  when the hooks saw it start. A visual launch is taken up only by a new conversation. And
  `sessions::record` leaves the row of a session id the ledger knows as it is.

So a resumed visual or manual session writes nothing today. Its run's time is not recorded, its end
finds no open row, and its usage share (EP-18a) is added to the old row. And a session resumed within
ten minutes of its end is still in the watch's model, with its first start and its work: its new run
gives no sign of work at all.

**Session start (2026-11-05, 22:19 UTC).** Launched from the window as a visual launch (rule 11):
the ledger row `8a379a7c`, visual, EP-35, open (`crw sessions clauderoadmapwrangler`). The opener
suggested Fable at high effort. The owner runs it at Opus 5.5 with max effort, chosen in the tab's
panel. The tree was clean at `929ff17` but for the owner's tick of OT-16. No owner task and no inbox
message is open.

**Decided at pickup (owner, through `AskUserQuestion`).**

1. **A row per run.** A resumed run earns a row of its own under the same session id at its first
   work, with the EP of the session's latest row. Its mode is visual when the window handed over
   its link in the 30 minutes before its start, else manual. Its end hook closes it, and its usage
   share goes to it. Revert still goes back to the session's first start (EP-15b). A session whose
   latest row never got its end goes on in that open row. The alternatives were reopening the old
   row, so that the gap between the runs counts as session time, and no row, as today.
2. **The check, with both ids**: a throwaway headless run's and EP-18a's visual session's, at this
   project's window, once the code is built. The throwaway's transcript is then deleted by its id,
   since `claude purge` takes a whole project.
3. **The session's model and effort**: Opus 5.5 at max, chosen in the tab's panel.

## In scope

1. **The link** (`crw_core::visual`). `session_link(id)` gives
   `vscode://anthropic.claude-code/open?session=<id>`. It refuses an id that is not a UUID, which
   the extension would drop without a word. A `Launch` hands over a prompt or a session, the same
   way: the workspace file, the wait for the lock file, then the link. A resume writes no session
   row. It writes a `resume` event (the project, the session, the EP), which the resumed run's row
   reads (4).
2. **The button.** "Resume in VS Code" on each ledger card whose row has a session id, beside
   Revert…. Both buttons sit at the end of the chips' line when they fit, else on a line of their
   own. A press is `Action::ResumeInCode`, carried out off the UI thread like a visual launch, and
   the status bar says where the session waits. A screenshot run opens nothing.
3. **Disabled, with the reason on hover**, when:
   - the project is not registered, or its folder is missing;
   - the id is not a UUID;
   - a launch to VS Code is already on its way;
   - the session runs in the window, or the hooks or the poll show it open (such rows are not
     listed today, but the rule is the button's own);
   - the list of live sessions has not answered or fails, so whether it runs elsewhere is not known,
     as Revert refuses;
   - the row is a headless run's, if the check finds that the extension refuses those ids.
4. **The ledger's rule for a resumed run** (the owner's choice at pickup).
5. **The watch's model.** A start (not a compaction's) of a session that has ended starts it again:
   its start, its source and its work begin anew, so a run resumed soon after its end earns its
   row too.
6. **The first check**, on the owner's yes: one supervised launch through a throwaway driver (D-29).
7. **Screenshot runs**: the ledger cards with both buttons, in both themes.
8. **Recorded**: D-16 and D-26 addenda, the README's paragraph on the window, the roadmap row's
   link, and "From EP-35" notes where later briefs need them.

## Out of scope

- The headless resume after a restart, with the launch's choices kept in the ledger (v2 notes).
- A prompt handed with the resume: the owner types the next message in the tab, and an open tab
  would not take one.
- A `crw` command that resumes in VS Code.
- Closing a crashed headless run's open row: no hook closes a row of the runner's.
- A session whose working folder was a subfolder of the project. Its transcript lies under the
  subfolder's slug, which the project's window may not look in.

## Verification / acceptance

- **Unit tests**:
  - the link: UUIDs only, and the id read back through the models of VS Code's decoding and of
    `URLSearchParams`;
  - a resume through the injected starter: the workspace file, the wait, then `--open-url -- <the
    session link>`, a `resume` event and no session row, nothing after a cancel;
  - each reason the button is disabled for;
  - the ledger's rule, on scratch ledgers;
  - the model: a start after an end begins the session anew;
  - the button pressed through `testing::Screen`, and a disabled one not pressed.
- **Build checks:** `cargo test`, `cargo test --features screenshot`, `cargo clippy --all-targets
  --features screenshot -- -D warnings`, `cargo clippy --all-targets -- -D warnings` and `cargo
  fmt --check`.
- **Screenshot runs** of the Sessions pane's All tab with both buttons, in both themes.
- **The check**, on the owner's yes, supervised.
- **The release.** `cargo build --release` once the owner has quit the window from the tray, then
  `scripts\install-shortcut.ps1`.
- **Rule 3's word check** on the staged diff prints nothing.

> **Completion note (2026-11-05; claude-opus-5-5/max; about 40 min to the commit, with the
> owner's three rounds, the supervised check and one quit of the window).** One session in the
> VS Code extension from 22:19 UTC, launched from the window (ledger row `8a379a7c`, visual,
> EP-35). It covered the reading, the pickup round, this brief, the code and its tests, the
> supervised check, the screenshot runs, the closing round and the release.
>
> **Built.**
>
> - `crw_core::visual`: `session_link` and `is_uuid`, since the extension's link takes a UUID
>   only. `Launch::resume` and `Launch::link`, so that `open` hands over a session as it hands a
>   prompt. A resume writes a `resume` event (`RESUME_KIND`, its words the session's id and EP)
>   in place of a session row.
> - `crw_core::ledger`: `latest_of` (a session's latest row: its mode, its EP, whether it is
>   open) and `names_session` (an event of a kind whose words begin with an id, since a time).
> - `crw_core::sessions`: `Change::Started` carries `resumed`. `record` gives a resumed run a
>   row of its own when the latest row has ended or is a headless run's: visual after a handover
>   within `VISUAL_CLAIM`, else manual, with the latest row's EP. In the model, a start after a
>   heard end begins the session anew.
> - The window: `Action::ResumeInCode` and `Board::resume_in_code`, off the UI thread through
>   `visual::Visual`, with a notice of its own (`Outcome::Resumed`). The ledger card's Resume in
>   VS Code sits beside Revert… (`sessions::card_buttons`). `resume_availability` disables it
>   with its reason (`Resumes`: the window's sessions, those open, the live list unknown, a launch
>   on its way; and, after the check, a headless run). Each card's figures end with its session's
>   full id. A screenshot run's watch counts as polled.
> - The records: D-16 and D-26 addenda, three README sentences, the EP-35 row's link, "From
>   EP-35" notes for EP-19, EP-23 and EP-26, four v2 items and a note on the EP-13 item, OT-17.
>
> **The check (the owner's yes).** A throwaway `crates/core/examples/probe_resume.rs` ran one
> headless session through the runner, with Haiku at low effort under `dontAsk`, in this
> project's folder and with no ledger. It answered "ready" as `00135d0c`. The new resume code then
> handed its id to this project's window (already open, 2.5 s), and the owner saw a new tab,
> **empty**. The same for EP-18a's visual session `d7542e51` gave a new tab **holding its
> conversation**. So the extension opens a headless run's id without its conversation, as its
> list of past sessions leaves the SDK's sessions out (its 2.1.318 `/open` handler and session
> reader, read at pickup). Afterwards the driver and its build outputs were deleted. The
> throwaway's transcript, its security-warnings state and its empty session-env folder were
> deleted by its id, since `claude purge` takes a whole project. `crw sessions
> clauderoadmapwrangler` still listed the same 11 rows.
>
> **Verification.** `cargo test` 400 passed (96 + 1 + 1 + 2 + 1 + 262 + 2 + 7 + 6 + 11 + 11,
> against EP-18b's 395), 402 with `--features screenshot`; both clippy runs and `cargo fmt
> --check` are clean. The new tests:
>
> - the session link: UUIDs only, read back through the models of VS Code's decoding and of
>   `URLSearchParams`;
> - a resume through the injected starter: the workspace file, the wait, the session link, an
>   event and no row, and a bad id stopping it before anything starts;
> - the ledger's rule on a scratch ledger: a resume five minutes after the end, still in the
>   model; the handover's visual row; a manual resume an hour on; an open row going on; a crashed
>   headless run's resume; the usage share on the new row;
> - the resume's notice, the card's two buttons pressed through `testing::Screen`, and each
>   reason the button is refused for.
>
> Screenshot runs, kept in the session's scratchpad: light with `sessions=lantern` (lantern's
> visual card with Resume in VS Code and Revert… enabled on a line of their own, its lapsed
> launch with neither), and dark with `sessions=harbor` (both headless cards with Resume greyed
> out beside Revert…). The release was built once the window was quit, then
> `scripts\install-shortcut.ps1` refreshed both shortcuts. Rule 3's word check found nothing in
> the staged lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-05).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The ledger's rule for a resumed run, at pickup | A row per run | As built; D-26 addendum |
> | 2 | The supervised check, at pickup | Run it, both ids | Run; headless cards' buttons disabled; D-16 addendum |
> | 3 | The session's model and effort | Opus 5.5 at max, chosen in the tab's panel | The front matter |
> | 4 | What the headless id opened | A tab, but empty | A headless run is not resumed in VS Code |
> | 5 | What EP-18a's id opened | A tab with EP-18a's conversation | Visual and manual runs are |
> | 6 | The release build | Quit the window, build and install | Built and installed |
> | 7 | OT-17 | Add it, before EP-19 | OT-17 |
> | 8 | The addenda, the README and the roadmap notes | Accept as written | Written |
> | 9 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. The button reads "Resume in VS Code", without the brief's ellipsis: it acts at once and
>    asks nothing, unlike Revert…, whose preview follows.
> 2. Each ledger card's figures end with its session's full id, selectable, for `claude
>    --resume` and `crw revert`; the seen cards and the tabs showed theirs already.
> 3. The handover is an event, not a row: a row is earned by work (D-26's EP-13 addendum), so a
>    tab opened and closed unused leaves none.
> 4. The button is refused while the list of live sessions has not answered or fails, as
>    Revert's preview is: a session opened twice would run twice.
> 5. A launch with no ledger, such as the check's driver, records nothing, so the owner's
>    ledger keeps no trace of the check.
> 6. A screenshot run's watch counts as polled, so its cards show the button as a real window's
>    do.
>
> **Notes for later sessions.**
>
> - The CLI's help no longer lists a `default` permission mode: it gives acceptEdits, auto,
>   bypassPermissions, manual, dontAsk and plan, and the runner refuses any other. The tool's
>   default is acceptEdits, so nothing changes, but a project's `permission_mode` of `default`
>   in the registry would now be refused.
> - A session whose latest row never got its end (EP-18b's `6fa87918` is one) goes on in that
>   row when resumed, so the row spans both runs and the time between them.
>
> No messages for other projects. Owner task: OT-17, before EP-19. **Next: EP-19** (the
> autopilot), launched from the window.
