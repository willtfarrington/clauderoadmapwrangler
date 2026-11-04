---
ep: 15d
title: The popup's crash
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-15d — The popup's crash

**Size:** S · **Depends on:** EP-15 · **Blocks:** EP-15c, EP-16

## Context

On 2026-11-03 at 19:34 the window closed on a panic, and EP-14a's crash log named it at the next
start: "egui backend is implemented incorrectly - the user callback was never called"
(`egui-0.37.0/src/context.rs:4714`). The backtrace runs from winit's `call_new_events` through
eframe's `check_redraw_requests` and `run_ui_and_paint` into `Dashboard::ui`, `Toasts::chosen`,
`Popup::ui` and `Context::show_viewport_immediate`. The window was hidden in the tray, and a
notification had just arrived. No headless run was open, so nothing else was lost.

**The cause (eframe 0.37.0's source, read 2026-11-04).**

- Windows sends no `RedrawRequested` to a hidden or minimized window. So eframe's
  `check_redraw_requests` paints such a window directly (`native/run.rs`, egui issue 5229).
- One of the handlers that call it, `new_events` (`run.rs`, lines 401 to 407), is the only one
  not wrapped in `with_event_loop_context`. Every other handler sets that thread-local.
- An immediate viewport whose window does not exist yet is given one through that thread-local
  (`wgpu_integration.rs`, `render_immediate_viewport`). Without it, eframe returns without
  running the viewport's callback, and egui's `expect` panics.
- So the first notification after the popup was empty kills the window, when the hidden window's
  pass happens to come from `new_events`.
- A deferred viewport gets its window from `initialized_all_windows(event_loop)` at the start of
  every `run_ui_and_paint`, with the event loop passed in. That is safe on every path.

**The owner's choice (2026-11-04).** A fix brief now, an S before EP-15c. Until it lands, the
window stays shown, not in the tray or minimized.

## In scope

1. **The popup as a deferred viewport** (`popup`, D-21 addendum).
   - Its toasts sit behind a shared lock.
   - Its own pass draws them and applies the owner's choice there and then: Dismiss and Dismiss
     all remove toasts, and Show removes its toast and queues it for the window's pass, which the
     popup wakes. Nothing then refers to a toast by an index that a new toast could shift.
   - The window's pass keeps the viewport open while the popup holds toasts, places it at the
     screen's lower right and sizes it to the height the popup's own pass measured.
   - The screenshot run's drawing inside the window stays as it is.
2. **A test that reaches the path.** The File menu's test notification comes while the window is
   in front, which never takes the hidden window's path. A second item, "Send a test notification
   in 10 seconds", leaves time to close the window to the tray first. The window's pass sends it
   when it is due, shown or hidden.
3. **Records.** The module's documentation; a D-21 addendum on why the popup is deferred; the
   owner task; the completion note.

## Out of scope

- Moving eframe or egui, which happens only in a re-plan brief (D-15).
- Reporting the defect upstream (the owner's choice, noted for the re-plan).
- The popup's look and its rules, which stay as EP-15 built them.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`,
  `cargo build --release` with the window quit from the tray, then the install script.
- Unit tests: Show, Dismiss and Dismiss all, applied by the popup's own pass, take the toast that
  was drawn even when a new one arrives before the window's pass; the drawing inside a bare
  context, as before.
- A screenshot run with `popup`, its picture looked at: the popup unchanged.
- The real popup can be judged only on the owner's screen (rule 5): the delayed test
  notification with the window closed to the tray, several times over, then an ordinary
  notification while it is hidden.
- Rule 3's word check prints nothing on the staged diff.

> **Completion note (2026-11-04; claude-opus-5-5/max; about 20 min of wall time with the owner's
> check).** The popup is a deferred viewport, which eframe gives a window on every path, so a
> notification that comes while the window is in the tray no longer closes it. The File menu
> sends a test notification in ten seconds to reach that path, and on the owner's screen it
> worked every time.
>
> **Built.**
>
> - `popup`: the toasts, the shown ones and the measured height live in `Shared` behind an
>   `Arc<Mutex<…>>`. The window's pass shows the popup with `show_viewport_deferred` while it
>   holds toasts, sized and placed as before, and wakes it (`request_repaint_of`) when a toast
>   came, since showing a deferred viewport again repaints nothing. The popup's own pass draws
>   them, applies Show, Dismiss or Dismiss all there and then (`Shared::apply`), and wakes the
>   window's pass when something changed or its height did. A screenshot run still draws it
>   inside the window.
> - `board`: `Action::TestToastLater`, the File menu's "Send a test notification in 10 seconds",
>   due in `attend`, which runs whether the window shows or hides.
>
> **Verification.**
>
> - `cargo test`: 350 tests; with the screenshot feature, 352; 2 new. Both clippy runs and
>   `cargo fmt --check` are clean.
> - The tests: a choice takes the toast that was drawn even when a new one arrives before the
>   window's pass; the popup shown as a deferred viewport, woken for a new toast, its own pass
>   run as eframe runs it (drawing and measuring), and left out once empty.
> - A dark screenshot run with `popup`, looked at: the popup as before.
> - The release build and the install script ran with the window quit. The owner closed the
>   window to the tray before each delayed test notification, three or four times: the popup came
>   each time and the window stayed.
>
> **Owner decisions.**
>
> | Decision | Outcome |
> | --- | --- |
> | How to handle the crash | A fix brief now, an S before EP-15c (recommended). |
> | Quitting the window for the release build | Done by the owner from the tray. |
> | The on-screen check | It worked every time (recommended). |
> | Commit and push | Committed and pushed (recommended); the row's hash is recorded in a commit of its own. |
>
> **Routine choices.**
>
> - The choice is applied in the popup's own pass, under the lock, so that a toast pushed in
>   between never shifts the index of the one chosen. Show queues its toast for the window's pass.
> - The lock is never held while eframe has the viewport. In a screenshot run, where egui would
>   run a deferred callback inline, the popup is drawn inside the window instead.
> - Moving eframe waits for a re-plan (D-15); EP-16's note says no other immediate viewport should
>   be added until then.
>
> **Handed on.** Nothing. **Next:** EP-15c.
