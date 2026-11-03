---
ep: 15
title: Notifications
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-15 — Notifications

**Size:** M · **Depends on:** EP-12 · **Blocks:** EP-17 (the usage notifications at 80 and
95 %), EP-19 (Pause autopilot in the tray), EP-20 (an urgent message's notification, ntfy), EP-28
(the icon, single instance, start at login)

## Context

Until now the owner learns that a session waits, has finished or has failed only by looking at the
window, and closing the window ends every headless run it holds (D-16's EP-10 addendum). D-21 asks
for Windows toasts at once when a session waits for the owner, finishes or fails, a tray icon with a
count of what waits and a menu, badges for what only wants a look, and quiet hours; D-29 asks that
closing the window leave it in the tray. This brief builds them, so that the window can sit in the
tray while its runs go on and call the owner when one needs them.

**Session start (2026-11-03, 19:38 UTC).** Started from `DATA\clauderoadmapwrangler.code-workspace`
in the VS Code extension; the tree is clean, every owner task is ticked (OT-9 among them) and no
inbox message is open. The model is Opus 5.5; the session's environment gives the effort as max.

**Verified at pickup.**

- eframe 0.37 calls `App::logic` before each `App::ui` and also while the window is hidden, when a
  repaint is asked ("the app logic keeps ticking"). On Windows a hidden window gets no
  `RedrawRequested`, so eframe runs its pass itself, at most every 100 ms of repaint requests
  (`native/run.rs`, egui issues 5229 and 7776). winit 0.30 sends no `Occluded` on Windows, so a
  window hidden with `ViewportCommand::Visible(false)` still runs full passes: egui's `visible()`
  means only "not minimized, not occluded". The window's work (`Board::receive`: the sessions'
  events, the server's signals, the poll) runs from `ui` today.
- tray-icon 0.26.1 (the research pass's version) re-exports muda 0.21.2; on Windows both use
  windows-sys 0.60–0.61, of which the lock holds 0.61.2. The menu opens on a left click unless told
  otherwise; the tooltip is copied into a 128-unit buffer with no room left for its terminator; the
  icon is added again when Explorer restarts. Events reach a handler or a global channel. A muda
  `Menu` is an `Rc` handle, destroyed at its last drop.
- notify-rust 4.18.2 wraps tauri-winrt-notification 0.8.1 on Windows (which uses `windows` 0.62,
  the lock's 0.62.2). Its handle reports a click only through a wait that blocks until the toast is
  clicked or dismissed: a parked thread per toast, for good when Windows holds toasts back.
  tauri-winrt-notification's `on_activated` calls back on WinRT's own thread, with no thread waiting.
- The install script's two shortcuts carry the AppUserModelID `willtfarrington.clauderoadmapwrangler`
  (EP-1), the id the toasts must name to show as the tool's.
- The ledger's `event` table (V1) was made for "notifications, deliveries, hosting and remote actions
  (from EP-15)"; nothing writes it yet.
- `Board::decisions_count()` counts what waits for the owner (EP-14). A window's run ends with
  `Event::Exited` (EP-13); a session elsewhere waits, ends a turn or ends in the state model
  (`crw_core::sessions`, EP-12), and one that never worked is `Seen::is_new()`.

## In scope

1. **The popup** (D-21). *Changed at the close, the owner's choice:* the notifications are a popup
   of the window's own, in place of the Windows toasts first built (off on the owner's machine).
   It is a small window at the screen's lower right: always on top, without the keyboard focus
   when it appears, without a taskbar button. It lists the latest notifications, the newest first
   (three, with a count of the rest and Dismiss all), each naming the project, the EP and the
   event, with a line of detail, Show and Dismiss, until dismissed. Show brings the window
   forward with the pane that matters (the Decisions pane for a wait, the session's tab for an
   end), or the VS Code window a session waits in.
2. **The rule table** (`crw_core::notify`, D-21). These pop up at once:
   - Waiting: a window's run asks; a run is deferred at its time limit or by the defer route; a
     session elsewhere waits for a permission prompt, an MCP server's question or input.
   - Finished: a window's run completes; a session elsewhere ends a turn after 2 minutes or more of
     work; a background session is done.
   - Failed: a window's run fails; a background session fails.

   These are badges only, on the tray icon:
   - a new unadopted folder;
   - an orphan or a mismatched remote (a missing repository, or a current head check that is
     diverged or not fetched);
   - a stale, failing or uncovered backup;
   - an inbox arrival.

   A stopped run, a session closed by hand and a session that never worked show nothing. The usage
   notifications are EP-17's, which adds their moment to the table.
3. **Held back**: in quiet hours (`settings.quiet_hours`, local time, wrapping midnight, none by
   default; `crw setting quiet_hours <22:00-07:00|none>`), and while the window is in front with the
   focus, where the status bar says it instead. A session elsewhere shows a second wait only after
   2 minutes, and the moments of one session in one frame make one notification.
4. **The tray icon** (tray-icon 0.26.1, D-21):
   - The icon: drawn by the tool (EP-28 brings the app's own), with a red count of what waits
     (`decisions_count`) and an amber dot while a badge event is new since the window was last in
     front. The tooltip says both.
   - A left click shows the window.
   - The menu: what waits; what is new; Show; Start next for each pinned project (the launch dialog
     on its next brief, D-17); Pause autopilot, disabled until EP-19; Quit.
5. **Close to the tray** (D-29). Closing the window hides it, and its headless runs, the loopback
   server and the poll go on, through `App::logic`. The first close says so once, in the popup.
   - Quit (in the tray, and in a new File menu in the window) quits at once when no headless run is
     going; otherwise it first asks, naming the runs it stops.
   - Without a tray (if it failed to start, or in a screenshot run), closing quits, with the same
     question.
6. **The log**: one `event` row of kind `notification` for each notification shown or held back,
   each Show and each badge arrival. `crw notifications [--all]` prints them and reads only.
7. **A test notification**: File → Send a test notification. It shows whatever the quiet hours and
   the focus, so the owner can see where the popup lands.
8. **The status bar's hover** says how notifications stand: on or off, the quiet hours, how many
   are in the popup, the tray.
9. **Records**: a D-21 addendum and a D-29 addendum; the README; pickup notes for EP-17, EP-19,
   EP-20 and EP-28; the owner task; the completion note.

## Out of scope

- ntfy and the popup of an urgent inbox message (EP-20); the usage notifications (EP-17).
- The app's own icon, start at login and single instance (EP-28); Pause autopilot (EP-19); Enable
  remote view (EP-31).
- Windows toasts (built, then taken out at the owner's choice); answering a question from the
  popup; a notification from `crw`; switches per event (the quiet hours and Dismiss all serve).
- A notification for a session the owner closes by hand, or for a stopped run.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`, `cargo
  build --release` with the window closed, then the install script.
- Unit tests:
  - Quiet hours: read, written, wrapping midnight, refusals.
  - The rule table.
  - The ledger's event rows.
  - The registry's `quiet_hours`.
  - The moments of a window's run: asked, deferred, finished, failed, stopped.
  - The changes of a session elsewhere: a wait and its 2-minute rest, a long turn's end and a short
    one's, ends by hand and in the background, the window's own sessions and new ones left out, the
    sessions present at the start taken as they are.
  - What holds a notification back, and one per session per frame.
  - The badge events: each kind, new against seen, seen once the window is in front.
  - The icon's pixels: the count, the dot, the transparent corner.
  - The tray's tooltip under its limit, and the menu's items.
  - What closing the window does: hide, ask or quit.
  - The popup's list: the newest first, bounded; drawn inside a bare context.
- Screenshot runs of the Quit question and of the popup (`quit`, `popup`), in both themes, looked
  at.
- The real tray and popup can be seen only on the owner's screen (rule 5): the owner tries them
  with the new build at the session's close, or later as an owner task.
- Rule 3's word check prints nothing on the staged diff.

> **Completion note (2026-11-03; claude-opus-5-5/max; about 2 h 45 min of wall time with the
> owner's trials and answers, of which the popup asked for at the close took about 50 min).**
> The window calls the owner with a popup of its own and keeps a tray icon with the count of what
> waits. It closes to the tray with its runs going on, and Quit asks before it stops them. The
> Windows toasts built first came out at the owner's choice: toasts are off on their machine.
>
> **Built.**
>
> - `crw_core::notify`: the rule table (`Kind`, `Channel`), the quiet hours (`QuietHours`,
>   `settings.quiet_hours`) and the log's words (`Fate`, `line`). The ledger gained the `event`
>   table's first writer and reader (`Ledger::add_event`, `Ledger::events`); no migration.
> - `toasts`: the moments of the window's runs (`sessions::News`) and of the sessions elsewhere
>   (a wait and its 2-minute rest, a long turn's end, a background session's end), the badge
>   events (`marks`, `Attention`), what holds a notification back, and the log.
> - `popup`: the tool's own window at the screen's lower right, an immediate viewport that is
>   always on top, inactive when it appears and off the taskbar, with three notifications at
>   once, Show, Dismiss and Dismiss all, drawn while the main window is hidden too.
> - `tray`: the icon the tool draws (a red count, an amber dot), the tooltip, the menu changed in
>   place, and its orders.
> - The board: the window's work moved to `App::logic`, which eframe runs while the window is
>   hidden; closing hides; a File menu (Send a test notification, Close to the tray, Quit); the
>   Quit question; the notifications' line in the status bar's hover.
> - `crw notifications` and `crw setting quiet_hours`. The start notices and the launch dialog
>   now say that quitting, not closing, stops a run. Screenshot flags `quit` and `popup`.
>
> **Verification.**
>
> - `cargo test`: 321 tests; with the screenshot feature, 323. Of these, 23 are new. Both clippy
>   runs and `cargo fmt --check` are clean.
> - Screenshot runs of the Quit question and of the popup in both themes, looked at. The popup's
>   first picture showed a row of buttons taking the whole height; fixed and taken again.
> - The release build and the install script ran four times, the owner quitting the trial's
>   window for one of them. `crw roadmap .` reports 0 problems; `crw signals` shows the hooks
>   running this exe.
> - The owner tried both builds: the tray works (its icon, its menu, closing to it, Quit from
>   it), and the popup appeared at the lower right, its Show, Dismiss and list as described.
> - The real log (`crw notifications`) shows the rules at work on live sessions, this one among
>   them: a wait held back while the window was in front, a repeat within 2 minutes, the test
>   notifications shown and one opened. The Windows toasts of the first build are logged too,
>   though Windows showed none of them.
>
> **Owner decisions.**
>
> | Decision | Outcome |
> | --- | --- |
> | The toast crate, D-21 naming notify-rust | tauri-winrt-notification directly (recommended); superseded when the toasts came out. |
> | Notifications while the window is in front | Held back, the status bar saying them (recommended). |
> | The records (the D-21 and D-29 addenda, the README, the notes, OT-10) | Kept as written (recommended), then brought in line with the popup. |
> | The first build's trial | The tray works; Windows toasts are off on the owner's machine, who asked for a popup window instead: a change of scope. |
> | How long a notification stays in the popup | Until dismissed (recommended). |
> | The Windows-toast code | Popup only: the code and its crate came out (recommended). |
> | Quitting the trial window for the build | Quit (recommended); it exited at once. |
> | The popup's trial | Works as described (recommended). |
> | Commit and push | Committed and pushed (recommended); the row's hash is recorded in a commit of its own. |
>
> **Routine choices.**
>
> - A badge event is new until the window is next in front with the focus; the first reading at
>   the start counts as seen.
> - A session elsewhere's turn shows its end only after 2 minutes' work, and its second wait
>   within 2 minutes is a repeat. The sessions present at the start are taken as they stand until
>   the first poll answers (30 s at most).
> - The tray's menu changes its texts in place; a menu replaced for a new list of pinned projects
>   is kept a minute longer, in case it was open.
> - The icon is drawn at 16 points of the window's scale (24 pixels at 150 %). Its tooltip stays
>   under 128 UTF-16 units, since tray-icon copies 128 with no room for the end.
> - Start next opens the launch dialog (D-17) and never starts a session itself.
> - The popup keeps 50 notifications at most. A minimized window is asked to paint while the
>   popup holds any. A wait reads "waits for you: …", which fits the hooks' words and the
>   poll's alike.
> - The log's words: popup, quiet hours, in front, repeat, opened, badge.
>
> **For later briefs.** The "From EP-15" notes in EP-15a, EP-15b, EP-16, EP-17, EP-19, EP-20,
> EP-28 and EP-31.
>
> **Handed on.** Nothing. **Next:** EP-15a.
