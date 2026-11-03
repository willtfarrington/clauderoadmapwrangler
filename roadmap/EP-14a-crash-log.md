---
ep: 14a
title: Crash log and the notice's time
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-14a — Crash log and the notice's time

**Size:** M · **Depends on:** EP-14 · **Blocks:** EP-15

## Context

On 2026-11-03 the window vanished while it held a headless run of another project that had just
taken the owner's long answer to a question. Three seconds after sending the answer, the owner
closed that project's own desktop app, and the Roadmap Wrangler window went at once. The runner was
still alive: when the window's handle on the run dropped, it killed the run and recorded its end as
`stopped` (D-29). The owner reverted the run's work in its project, to be done again later.

**Verified at pickup.**

- Windows kept no record of a crash: no Application Error or Windows Error Reporting event, no
  report, no dump. A native fault or an abort (a `panic = "abort"`, a stack overflow, running out of
  memory) would have left one. So the window ended on a Rust panic that unwound, which a release
  build (`windows_subsystem = "windows"`, no console) prints nowhere. The window installs no panic
  hook, so the message is lost.
- Nothing outside the window closed it. The two apps have separate taskbar ids, the other app's
  close button did not lie over this window's, and the desktop automation script running has no
  window-closing command. The window's last moments were losing focus to the other app and getting
  it back when that app closed.
- The status bar holds a definite crash of the same kind. `status_bar` checks
  `at.elapsed() < NOTICE_FOR`, lays out the notice, then asks for a repaint after
  `NOTICE_FOR - at.elapsed()`. `Duration` subtraction panics on underflow, so a frame whose second
  reading passes the 8 s mark ends the window. egui makes such a frame likely: it brings a delayed
  repaint forward by the predicted frame time, and frames run back to back up to the mark. A long
  notice widens the gap between the two readings, and the answer's notice carried the whole answer
  (some 1,700 characters on one line). The timestamps rule it out for this crash: the run's end was
  recorded about 2 s before that notice's 8 s were up.
- The other panic sites on the window's thread were checked and are guarded: an `expect` after
  `if let Some`, the time limit's `saturating_sub`, char-safe cuts. egui 0.37 filters a focused id
  without a node out of its AccessKit update. AccessKit is compiled in (eframe's defaults) and stays.

## In scope

1. **Crash log.** At start, the window (not a screenshot run) installs a panic hook. Each panic,
   on any thread, appends a report to `crash.log` in the local folder (`%LOCALAPPDATA%\
   clauderoadmapwrangler`, or `CRW_HOME`): a head line with crw's version, the UTC time, the thread
   and the place, then the message and the backtrace. Past 1 MB the log moves to `crash.old.log`.
   Then the default hook runs as before, so a debug build's console still shows the panic.
2. **Named at the next start.** When the log's last report is newer than the last one the window
   named (its time is kept in `crash.seen` beside the log), the status bar's error line names it
   once: the local time, the message's first line, the file and line, and the log's path.
3. **The notice's time.** The status bar reads a notice's age once and takes
   `NOTICE_FOR.checked_sub(age)`, so no frame can underflow at the 8 s mark.
4. **One-line notices.** Notices, warnings and errors keep their first line, at most 160
   characters, cut by characters with an ellipsis. An answer's words are cut on their own, so
   "the session goes on" still ends the line.
5. **Tests.** A report reads back as the last; the status bar names a report once and a newer one
   again; a full log moves aside; the hook writes a panicking thread's report; a notice's time left
   before, at and past 8 s; the one-line cut on text with multibyte characters.
6. **Records.** The roadmap row and its note, a dated addendum to D-14, the completion note.

## Out of scope

- Finding the panic of 2026-11-03: its message is lost. The log names the next one.
- A dialog at the moment of a crash (it would need a Windows dependency of its own).
- Keeping a headless run alive past the window. D-29 kills it on purpose, so no run goes on
  unattended. Claude Code's own list of past sessions in the project resumes it by hand.
- Turning AccessKit off: considered, and declined by the owner.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, `cargo clippy --all-targets --features
  screenshot -- -D warnings`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
- `cargo build --release` with the window closed, then `pwsh -File scripts\install-shortcut.ps1`.
- Rule 3's word check prints nothing on the staged diff.

> **Completion note (2026-11-03; claude-opus-5-5/max; about 1 h, of which the investigation took
> 35 min).** The window now leaves a report when it panics, and names it at the next start. The
> status bar's notice can no longer underflow, and notices keep to one line. The crash of
> 2026-11-03 is still unexplained.
>
> **How the crash was read.** The evidence came from the run's transcript, the ledger (read-only),
> the other app's saved state and the Windows event log.
>
> - The run took the answer at 17:47:50 UTC, and the other app saved its state as it closed at
>   17:47:53.
> - The run's end was recorded at 17:47:56. That is the runner killing it once the window's handle
>   dropped (D-29). The transcript has no interrupt, so this was no Stop.
> - Windows recorded no crash, no report and no dump.
> - The owner confirmed the order (answer sent, the other app closed three seconds later with its
>   title bar's X, this window gone at once, nothing else pressed).
>
> So the window ended on an unwinding Rust panic when it got focus back. These were ruled out: the
> notice bug (by about 2 s), the answer's own path, the Decisions pane, the session's events, the
> folder watcher, a focused id without an AccessKit node, the taskbar's grouping, a click through to
> this window's close button, and the desktop automation script.
>
> **Built.**
>
> - `crash.rs` (`install`, `unseen`, a report's head line read back by `last`).
> - `Places::crash_log`. The window's own run passes the local folder's log. A screenshot run passes
>   its home's log and installs no hook.
> - `Board::new` puts the unseen report into the error line.
> - `notice_left` reads the notice's age once.
> - `one_line` cuts notice, warning and error text to 160 characters, and an answer's words to 80
>   inside it.
>
> **Verification.**
>
> - `cargo test`: 298 tests. With the screenshot feature: 300. Six of them are new: four in
>   `crash`, two in `board`. The hook's test panics a named thread and restores the hook before it.
> - Both clippy runs and `cargo fmt --check` are clean.
> - `cargo build --release` ran with the window closed (the owner closed it), then
>   `scripts\install-shortcut.ps1`. `crw signals` still shows the hooks running this exe.
> - `crw roadmap .` reports 0 problems and EP-14a next.
> - Rule 3's word check printed nothing.
>
> **Owner decisions.**
>
> | Decision | Outcome |
> | --- | --- |
> | How to fix it before EP-15 | The crash log and the notice fix as EP-14a (recommended). The owner declined a reproduction attempt on a scratch window and turning AccessKit off. |
> | Close the window for the release build | Closed; built and installed. |
> | The records (the D-14 addendum, the EP-14a row, its note and its place among EP-16's dependencies, the README sentence, OT-9) | Kept as written (recommended). |
> | Commit and push | Committed and pushed (recommended); the row's hash is recorded in a commit of its own. |
>
> **Routine choices.**
>
> - The report's head line holds the version, the UTC time, the thread and the place, apart by
>   ` · `, so the next start reads them back without a format of its own.
> - The status bar shows the last report only, once: `crash.seen` keeps its time.
> - Past 1 MB the log moves aside, one old log kept.
> - The hook hands on to the previous one, so debug builds still print to their console.
> - `notice_left` treats the mark itself as past.
> - The cut is by characters with an ellipsis, and a dropped second line is marked ` …`.
> - The answer's words are cut on their own, so "the session goes on" ends the notice.
> - The OT-8 tick rides along. OT-9 says what to do if the window vanishes again.
>
> **For EP-16's re-plan.** A run stopped by a crash has no tab in the next window, so its Resume is
> gone. A Resume on its row in the All list would pick it up. Today Claude Code's own list of past
> sessions in the project (`claude --resume`) does.
>
> **Handed on.** Nothing. **Next:** EP-15.
