---
ep: 28a
title: Operations, the start
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-28a — Operations, the start

**Size:** M · **Depends on:** EP-28, EP-37a · **Blocks:** EP-29 · **Decisions:** D-29, D-26, D-21,
D-8, D-3, D-15

## Context

D-29 wants the tool to register its own start-at-login entry, keep a single instance, and reconcile
its sessions from `claude agents --json` when it starts. EP-23 cut the operations brief in two (the
owner's choice). EP-28 built `crw doctor`, the Diagnostics pane, the rolling log and the AV hints,
and EP-40 built the single-instance guard. This brief takes the rest of the start:

- the start-at-login entry, which starts the installed window hidden in the tray;
- the start-up reconciliation of the ledger's open rows;
- `crw` on PATH (IN-3);
- the app's own icon in the tray, the window and the shortcuts;
- `install-shortcut.ps1` in its final form.

The brief note adds where these belong. The Exes group of `crw_core::doctor::report` takes the
PATH line and the start-at-login line. `Board::window_lines` says what the reconciliation closed.
The rolling log records the window's starts.

**Session start (2026-11-14).** The prompt is this project's opener. `crw sessions
clauderoadmapwrangler` lists this run (`242b1d4e`, headless, EP-28a, claude-opus-5-5/max, started
2026-11-13 23:57). `crw autopilot`'s chain log says the autopilot chained it after EP-28 under M's cap
of 1 h 30 min, once the window had restarted into EP-28's build. Read: the P4 table with EP-28a's
brief note and EP-28's, D-29 with its addenda, EP-28's completion note, `owner-tasks.md` (OT-27 and
OT-28 open; both need the owner's hands and neither gates this brief), the inbox (no open message)
and the v2 note on a headless row a crash left open.

The field check found nothing odd:

- `crash.log` is still EP-14a's report of 2026-11-03, seen.
- The rolling log's first day file holds three lines: the restarted window's start at 23:57:16, the
  notice that it carried one held chain and one plan, and this chain.
- `crw notifications` since EP-28's record commit shows EP-28's finish and this chain.
- `crw autopilot`: six projects on, not paused.

The ledger holds three open rows in all: this run's; EP-37's first session (`45cc1e3b`, visual, its
tab closed on 2026-11-10 with no end heard); and a manual row of 2026-11-02 (`a026d3…`, EP-12's
own session). The owner has no taskbar pin of the tool, so the note's pin made before EP-37 does
not arise here. The working tree was clean.

**The code as found.**

- **The tray's icon.** `app::tray::draw(&Look)` paints the tray's icon: a blue tile with a
  roadmap's three white rows, a red count and an amber dot.
- **The window's icon.** The window keeps eframe's default icon, egui's logo. The script copies
  that logo into the shortcuts' `.ico`, found through `cargo metadata` and drawn with
  System.Drawing.
- **Start at login.** Nothing registers it. `main` knows only `--restarted <pid>`.
- **The guard.** `signals::ask_show` asks `/health`, then `POST /show`.
- **Open rows.** `Ledger::lapse_visual` closes only the visual launches no session took up
  (EP-13). No code closes a row whose end no window heard.
- **The install.** `crw install` copies the two exes into the install folder through staged
  copies and previous copies (`install::install`).

## In scope

1. **The app's own icon (`crw_core::icon`).**
   - The tray's drawing moves from `app::tray` into core: `Look`, the canvas, the glyphs and
     `draw`. The tray keeps drawing its badges with it.
   - `icon::plain(size)` gives the bare tile.
   - `icon::ico(sizes)` writes a Windows `.ico` with one 32-bit image a size, 16 to 256.
   - The window takes the bare tile at 256 px as its icon (`ViewportBuilder::with_icon`). eframe
     scales it for the title bar, Alt+Tab and the taskbar, in place of egui's logo.
   - `crw install` writes the `.ico` beside the exes when it differs, so the script needs
     neither `cargo metadata` nor System.Drawing.
2. **The start-at-login entry (`crw_core::autostart`).**
   - The entry is the value `Roadmap Wrangler` under
     `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, holding
     `"<install folder>\clauderoadmapwrangler.exe" --autostart`. It is written through the pinned
     `windows` crate's registry functions: a feature added, no new crate, so `Cargo.lock` is
     unchanged.
   - In a trial (`CRW_HOME`) and in tests, a file `autostart.txt` in the home stands in for the
     value, so a trial never writes the owner's registry (rule 13).
   - Functions: `status` (absent, or the command, and whether it starts the installed window),
     `register` and `remove`, with their words.
   - The switches are `crw autostart [on|off]` and the window's File > Start at login, a checkbox
     that a screenshot run refuses.
   - With `--autostart` the window starts hidden in the tray (`Board::hide_at_start`), or shown
     when the tray failed to start. A start at login that finds another window holding the port
     yields without showing it: the guard asks `/health` alone.
3. **The start-up reconciliation (`crw_core::sessions::reconcile`).**
   - When it runs: once per window, at the first listing of `claude agents --json` that answers
     while the window holds the port.
   - What it reads: the ledger's open rows that started before the window did
     (`Ledger::open_rows_before`), whose session no listing names. A visual launch that no
     session took up stays the lapse's (EP-13).
   - Visual or manual rows close as `completed`. A headless row, whose runner died with its
     window in a crash or a kill, closes as `stopped`.
   - The end time is the transcript's last write (`<config>\projects\*\<id>.jsonl`'s modified
     time; its contents are never read, D-26), else the row's start.
   - Each closed row's result reads "No end was heard; closed when a window started on <date>",
     and an `event` row of kind `reconcile` names it.
   - The window's lines (the Diagnostics pane) and the rolling log say what was closed, and the
     Sessions pane reads again.
4. **`crw` on PATH (IN-3).**
   - A copy of `crw.exe` in `%USERPROFILE%\.local\bin`, beside `claude.exe` in a folder already on
     PATH. The form is the owner's decision, asked at the close. This is the recommended form,
     built first.
   - `crw path on` places the copy and `crw path off` removes it. Every `crw install` refreshes it
     while it is there, staged and renamed as the install does, since a running copy can be
     renamed but not replaced.
   - A trial uses a folder in `CRW_HOME`.
5. **The doctor's Exes group** gains two lines:
   - "start at login": not registered, or registered and starting the installed window, or
     starting something else.
   - "crw on PATH": what PATH finds for `crw`, and whether it is the installed copy or a copy as
     built. This is IN-3's check.
6. **`install-shortcut.ps1` final.** It builds, then runs `crw install` (the exes, the icon, the
   PATH copy refreshed). It points both shortcuts at the installed window with the tool's own
   icon, and at the end prints the start-at-login entry's state and the PATH copy's. The header
   says what each switch does.
7. **Tests**, never touching the real `claude` (rule 5), the owner's registry or the owner's PATH
   folder:
   - the `.ico`'s header and entries, and the tile's pixels;
   - the entry over a scratch file (register, status, remove, words);
   - the reconciliation's choice (live or not, headless or not, before the start or not, the
     last write or the start) and its ledger writes;
   - the PATH copy placed, refreshed and removed in scratch folders;
   - the doctor's two lines;
   - the `--autostart` flag read;
   - the File menu's item in a screenshot run.
8. **Docs:** a D-29 addendum, the README's install words, the CLI's usage text, the roadmap row
   linked and the completion note.

## Out of scope

- **The icon inside the exes as a resource.** It needs a resource compiler as a build dependency,
  a stack change for a re-plan, so Explorer keeps showing the exes' default icon.
- **The other forms of `crw` on PATH.** The `bin` folder on the user's PATH and a `.cmd` shim are
  for the owner's choice at the close.
- **A delay at sign-in**, which the owner's start-up manager sets (D-29).
- **A reconciliation during the window's life.** The state model's poll and hooks keep doing that
  work.
- **The window's own resume of a crashed headless run** (v2 notes).
- **A taskbar pin.** None exists; the shortcuts take the new icon at the next install.

## Verification / acceptance

1. `cargo test --workspace` and `cargo test --workspace --features screenshot` pass. Both clippy
   runs and `cargo fmt --check` are clean. Rule 3's word check on the staged diff prints nothing.
2. A review subagent is given the diff (an M brief), launched at the first clean compile. Its
   findings are fixed or logged.
3. `crw autostart on|off` and `crw path on|off` run under a scratch `CRW_HOME` and write only
   there. `crw doctor` reads the owner's files and writes nothing (rule 13).
4. Screenshot runs show the File menu and the Diagnostics pane, in both themes, on the fixtures
   and on a copy of the real roadmaps (rule 5). The pictures stay in the scratchpad.
5. The release build and the install run last (`scripts\install-shortcut.ps1`).

**Residual risk.**

- **The registry calls.** Tests and the trial go through the scratch `autostart.txt`, so the
  `windows` crate's registry calls behind the Run value run only when the owner's switch or command
  writes it. They follow the documented functions the doctor's volume label already uses in the
  same crate. A start at login itself (Windows starting the installed window hidden at sign-in, and
  the quiet yield to a window already running) can be seen only at the owner's next sign-in.
- **The reconciliation's first real run.** It comes at the window's first start in this build. The
  chain ends at EP-29's re-plan row, so that start comes with File > Install and restart, or the
  tray's. It should close the two old rows the field check found, each at its transcript's last
  write. The run was tested on scratch ledgers with canned listings. The risk is a live session that
  the listing does not name: its row would close as completed and its next turn would start a new
  row (EP-35's row per run).
- **The icons.** The window's icon comes from eframe, which scales the 256-pixel tile for the
  title bar and the taskbar. The shortcuts' `.ico` follows the documented format its test checks.
  A screenshot run captures neither, since it saves the app's frame alone.
- **File > Start at login.** A screenshot run cannot open a menu, so the item was seen in no
  picture. It calls the same functions as `crw autostart`.
- **The window at login.** eframe 0.37 shows its root window after the first painted frame,
  before the queued hide applies. So a start at login may flash the window and take the focus
  for a moment before it goes to the tray, as EP-37a's hidden restart does (the review's
  finding, left as it is).

> **Completion note (2026-11-14; claude-opus-5-5/max; about 1 h of work inside about 15 h of
> wall time; a headless run chained by the autopilot).** Ledger row `242b1d4e`, started
> 2026-11-13 23:57 under M's cap of 1 h 30 min of working time; the prompt was the opener. The
> owner answered the two questions after about 13 h 50 min, which the cap does not count. Built
> as scoped. The core and CLI half went to a subagent with an exact API (EP-19b's method), while
> the session built the reconciliation, the window and the script.
>
> **Built.**
>
> 1. **The app's own icon** (`crw_core::icon`).
>    - `Look`, the canvas, the glyphs and `draw` moved from `app::tray`, whose local function is
>      now `drawn`.
>    - `plain(size)` gives the bare tile, and `ico(sizes)` writes one 32-bit image a size, 16 to
>      256. `ICO_FILE` is `app.ico`.
>    - The window takes `plain(256)` as its icon.
>    - `install::write_icon` and `install::after_install` (the icon, then the PATH copy) run in
>      `crw install` and in the window's own Install and restart. `Restart::path_dir` is none in
>      tests, which therefore never reach the owner's folder.
> 2. **The start-at-login entry** (`crw_core::autostart`).
>    - `Place` is the Run key, or a trial's `autostart.txt`. Functions: `command`, `exe_of`,
>      `read`, `register`, `remove`, `status` with `Status::words`, and `started_at_login`.
>    - The Run key goes through `RegGetValueW`, `RegSetKeyValueW` and `RegDeleteKeyValueW`: the
>      `windows` crate's `Win32_System_Registry` feature, with no change to `Cargo.lock`.
>    - The switches are `crw autostart [on|off]` and File > Start at login (`Action::StartAtLogin`,
>      `Board::start_at_login`, a notice and an `autostart` log line). A screenshot run refuses
>      the item.
>    - `--autostart` starts hidden in the tray and keeps the owner's first-close notice. It
>      yields quietly to a running window: `signals::ask_show(port, show)`, `Yield::Present`,
>      `Watch::quiet`. The log's start line says "at login".
> 3. **The start-up reconciliation** (`crw_core::sessions::reconcile`).
>    - `choose`, `last_write`, `apply`, `words`, `Closing`, and `EVENT_KIND` (`reconcile`).
>    - `Ledger::open_rows`, `close_unheard` and `in_transaction`.
>    - `agents::parse_counting` and `ask_counting`, which count the entries a listing left out.
>    - `Watch::reconcile` runs at the second whole listing in a row while the port is held, and
>      `Watch::reconcile_line` is the Diagnostics pane's "reconciliation" line.
> 4. **`crw` on PATH.**
>    - `install::path_dir` is `%USERPROFILE%\.local\bin`, or a trial's `CRW_HOME\path`.
>    - `place_path_copy`, `remove_path_copy`, `PathCopy` and `path_copy`. `previous_copies` now
>      takes the exes' names, so a pass in that folder touches crw's files only.
>    - The switch is `crw path [on|off]`.
> 5. **The doctor's Exes group** gains "start at login" and "crw on PATH" (`Places::autostart`,
>    `Places::path_dir`, `doctor::crw_on_path`). The pane's canned report shows both.
> 6. **`install-shortcut.ps1` final.**
>    - The icon now comes from `crw install`: the egui logo's extraction is gone, and the earlier
>      `clauderoadmapwrangler.ico` is removed.
>    - The script ends by printing the entry's and the copy's state, and its header is rewritten.
> 7. **Tests.**
>    - Core: the icon 2 (with the tray's moved test), the entry 4, the install 6 (another
>      program's `crw.exe` among them) and an extended one, the doctor 2, the reconciliation 3,
>      the listing's count in an existing test.
>    - App: the watch's reconciliation, the guard's quiet probe (in the server's test), the tray's
>      look, and the pane's line names.
>    - CLI: `start.rs` with 2.
> 8. **Docs:** the D-29 addendum, the README's words and command block, the CLI's usage text, and
>    the roadmap row linked.
>
> **Checked.**
>
> - **Screenshot runs** of the Diagnostics pane: dark on the fixtures, light on a copy of two real
>   non-data roadmaps (rule 5). Both show the reconciliation line and the new Exes lines.
> - **`crw doctor`** from the debug build over the owner's files (reads, rule 13): before the
>   session's writes, it said the entry was not registered and `crw` was not on PATH.
> - **A trial.** The builder ran `crw autostart` and `crw path` under a scratch `CRW_HOME` and
>   removed both entries afterwards.
>
> The pictures, the copy, the diffs and a sample `.ico` stay in `target\tmp\ep28a\`.
>
> **Review.** Two subagents, each given one half's diff and the brief, not the session's
> conclusions.
>
> The window half's reviewer (17 min, launched at the first clean compile) made 8 points:
>
> 1. **One listing could close a live session's row**, and a CLI whose entries changed form reads
>    as no session at all. Now the reconciliation:
>    - waits for two listings in a row that left no entry out;
>    - counts a session in either listing as live;
>    - skips a visual or manual row whose transcript was written since the window started.
> 2. **The ledger keeps whole seconds**, so a row from the window's first second counted as older
>    than the window. The cutoff is now the start's whole second.
> 3. **Headless rows.** A headless row's session is listed only when the owner resumed it by hand.
>    The listings no longer count for headless rows, which end at the window's start at the latest.
> 4. **Partial closes.** `apply` now runs in one transaction, so a failure closes nothing and a
>    later try names every row. A failure's words no longer blame the ledger's opening.
> 5. **The menu's read.** The menu read the entry's exe on the disk at every frame. It now reads
>    the entry alone.
> 6. **The first-close notice.** A start at login used up that notice for good. It is kept now.
> 7. **The flash at login** is left as it is and named in the residual risk.
> 8. **The icon cache.** Explorer's icon cache could keep egui's logo under the old file name. The
>    icon is now `app.ico`, and the script removes the old file.
>
> The builder also noted that the window's own Install and restart wrote no icon and left the
> PATH copy stale. That is taken (`after_install` in `restart.rs`).
>
> The core and CLI half's reviewer (13 min) found no blocking bug and made 5 points. Three are
> taken:
>
> - **The doctor's advice for a missing installed window.** It pointed at `crw autostart on`,
>   which refuses while the window is missing. It now says to install again.
> - **Another program's `crw.exe` in `.local\bin`.** One that uv or pipx put there would have
>   been renamed and removed. A copy now counts as the tool's only when its bytes name the
>   tool's folder, as every build does. Placing, removing and an install's refresh leave another
>   program's file alone and say so (`PathCopy::Foreign`).
> - **The `.ico`'s AND mask** now marks the clear pixels, so a reader that falls back on the mask
>   keeps the corners clear.
>
> Two are logged rather than fixed:
>
> - **An entry turned off in Task Manager's Startup apps.** Windows keeps that flag beside the
>   Run value, which the tool does not read, so the doctor and the File menu would still call
>   the entry registered.
> - **Leftover previous copies on PATH.** When `crw path off` runs from the PATH copy itself, or
>   after a refresh, `crw.previous-<stamp>.exe` files can stay in `.local\bin` until the next
>   pass. PATH never finds them as `crw`.
>
> Both go to EP-29's walk.
>
> **Verification.** After the review's fixes:
>
> - `cargo test --workspace` passes (189 in `app`, 427 in `core`, the CLI's suites), and so does
>   `cargo test --workspace --features screenshot` (197 in `app`).
> - Both clippy runs and `cargo fmt --check` are clean, and `Cargo.lock` is unchanged.
> - Rule 3's word check on the staged diff printed nothing.
>
> After the record commit, the session's last steps are: the release build and the install,
> then `crw path on` and `crw autostart on` from the installed copy.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | `crw` on PATH (IN-3): the form, and whether the session places it | A copy in `%USERPROFILE%\.local\bin`, placed now (the recommended option) | The session runs the installed `crw.exe path on` after the install. Every later install refreshes the copy; `crw path off` undoes it |
> | 2 | The start-at-login entry: registered by the session now | Yes, now (the recommended option) | The session runs the installed `crw.exe autostart on` after the install. File > Start at login, unticked, or `crw autostart off` undoes it |
> | 3 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The icon is the tray's tile without its badges.** It adds no new hue (EP-15a), and one
>    drawing serves the tray, the window and the shortcuts.
> 2. **The Run value, not a Startup-folder shortcut.** Task Manager's Startup apps and the
>    owner's start-up manager list it, and the tool reads it back for the doctor.
> 3. **The reconciliation's end is the transcript's last write**, the file's time alone (D-26),
>    rather than the window's start, which would have made a tab closed four days ago a run four
>    days long. Visual and manual rows close as completed and headless ones as stopped, as the
>    end hook and a quit would have recorded them.
> 4. **The quiet yield.** A start at login that finds a window running quits without showing
>    it, since nobody asked for a window at sign-in.
> 5. **The split of the work.** The core and CLI half went to a subagent with an exact API, and
>    the review was split by half.
> 6. **No picture of the File menu**, which a screenshot run cannot open. The item calls what
>    `crw autostart` calls.
> 7. **The scratchpad** is `target\tmp\ep28a\` (ignored).
>
> **No owner task.** The two writes into the owner's environment are the session's, on the
> owner's yes. Two first real runs remain for EP-29's walk, as the residual risk says:
>
> - the reconciliation, at the window's first start in this build: File > Install and restart,
>   since the chain ends at EP-29's re-plan row;
> - the start at login, at the next sign-in.
>
> **Next: EP-29** (the P4 re-plan, opening with the P4 walk), from the window. The work commit
> and the record commit follow.
