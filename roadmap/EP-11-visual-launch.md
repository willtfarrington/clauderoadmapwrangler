---
ep: 11
title: Visual launch
size: S
model: claude-opus-5-5
effort: xhigh
status: done
---

# EP-11 — Visual launch

**Size:** S · **Depends on:** EP-10, EP-33 · **Blocks:** EP-14 (Focus VS Code for a visual session
that waits), EP-16 (the dogfooding switch launches this project's briefs from the tool)

## Context

EP-10's launch dialog starts a headless session; its Visual button waits, disabled. D-16 offers a
second surface: the VS Code extension's link `vscode://anthropic.claude-code/open?prompt=…`, which
opens a Claude Code tab with the prompt filled in for the owner to send. Since 2026-10-31 each
project opens from its own `<name>.code-workspace` beside its folder (IN-1), and the D-16 addendum
has Visual open that file, writing it first through EP-33's writer when it is missing. Until EP-14
a headless session cannot answer the questions every brief ends with, so Visual is the launch that
can carry a whole brief from the tool today.

**Session start (2026-11-02, 15:43).** Started from `DATA\clauderoadmapwrangler.code-workspace`. No
owner task and no inbox message is open, so nothing is triaged. The model and effort are the user
settings' (`model = opus`, `modelSettings` giving Opus 5.5 `xhigh`).

**Verified at pickup.**

- Claude Code's VS Code page (read 2026-11-02): `vscode://anthropic.claude-code/open` takes two
  optional parameters, `prompt` (URL-encoded, filled in, never submitted) and `session` (a session
  to resume); "If VS Code is already running, the URL opens in whichever window is currently
  focused." So the project's window must have the focus when the link fires.
- The installed extension (2.1.314): its URI handler reads `new URLSearchParams(uri.query)` from a
  query VS Code has already decoded once, and hands `prompt` and `session` to its own
  `primaryEditor.open` command, with no length limit and no confirmation of its own; so a `%`, `&`
  or `+` in the prompt must be escaped twice, or the prompt is cut at an `&` and loses its plus
  signs. It activates on `onStartupFinished` (no `onUri` event) and does not run in an untrusted
  workspace.
- Windows hands a `vscode://` link to `"<VS Code>\Code.exe" --open-url -- "%1"`, the user install
  under `%LOCALAPPDATA%\Programs\Microsoft VS Code` (its `bin\code.cmd` is on PATH); the brief
  note's `--open-url "<uri>"` lacks the `--`. A batch file such as `code.cmd` would read `%xx` in a
  link as its own arguments, so `Code.exe` is started directly.
- The extension writes one `%USERPROFILE%\.claude\ide\<port>.lock` per window it runs in: `pid`,
  `workspaceFolders`, `ideName`, `transport`, `runningInWindows` and an `authToken`. Of six lock
  files, one belonged to a running process, so stale files are the rule; its `pid` is VS Code's
  main process (`VSCODE_PID`), which every window shares. A single-folder workspace's window lists
  the project folder first.
- A session's shell inherits `ELECTRON_RUN_AS_NODE=1` and VS Code's internal `VSCODE_*` variables
  from the extension host, with which `Code.exe` would start as Node, besides the Claude Code
  session markers the runner already withholds (D-15 addendum); VS Code hands the environment of
  the `Code.exe` that asks to the windows it opens.
- This project's opener is 4,617 characters, about 6,800 once encoded; a Windows command line
  holds 32,767.
- `[project.launch] mode` (`visual` or `headless`) has been read since EP-2 and is used nowhere.

## In scope

1. **`core::visual`**: the link, its prompt escaped for both decodings, a link over 32,000
   characters refused; `Code.exe` located (a `vscode` setting, the user install, the system
   install, the install whose `bin\code.cmd` is on PATH); the extension's windows read from the
   lock files (`pid`, `workspaceFolders`, `ideName` only: the auth token is never kept, logged or
   shown), a file counting while its process runs; the launch: the own workspace
   file written when missing (EP-33's writer, never over a file), `Code.exe <own file>` (which
   opens the project's window or brings it forward), a wait of up to 60 s for a live lock whose
   first folder is the project, then `Code.exe --open-url -- <link>`; children one at a time
   (D-29) without `ANTHROPIC_API_KEY`, the session markers and the `ELECTRON_*` and `VSCODE_*`
   variables; a `visual` row in the ledger with no session id, model or effort; a cancel while it
   waits fires nothing.
2. **The prompt** (D-17): a visual launch's opener says that VS Code's panel chooses the model and
   effort and names the launch's suggestion; `crw prompt` prints either form (`--visual`,
   `--headless`; by default the project's mode).
3. **The dialog**: a Headless | Visual switch, defaulting to the project's `mode`, else the tool's
   default (an owner decision); in Visual the model, effort and ultracode are a suggestion, the
   permission mode and the neighbours headless only, and a VS Code line takes the Claude line's
   place (VS Code found, the extension's version, the own workspace file, whether the project's
   window is open); the footer's note and its one start button follow the mode ("Open in VS
   Code"), with a spinner and Cancel while it waits; the notice says the prompt waits in VS Code
   to be sent by hand.
4. **Screenshot runs**: a canned VS Code setup (no machine path shows), lantern given `mode =
   "visual"` in the fixture registry; a run opens nothing.
5. **Owner decisions** through `AskUserQuestion` (rule 7): the default mode, a supervised real run,
   the addenda, the edits, the commit.

## Out of scope

- Resuming a session by its id (the link's `session`), Focus for a visual session that waits
  (EP-14), seeing a visual session start and end (the hooks, EP-12).
- Bringing a window forward by its handle or VS Code's window id: the workspace file is what
  brings the project's window to the front.
- A model or effort passed to the extension (the link takes none), and a `crw` command that
  launches.
- Tests that start VS Code: the launch's tests inject the process starter, and the one real run
  is the owner's yes (D-29).

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`, `cargo
  build --release` with the app closed, then the install script.
- Unit tests: the link's prompt read back through models of VS Code's decoding and of
  `URLSearchParams` (spaces, new lines, `%`, `&`, `+`, `=`, `#`, `%41`, Unicode); the length limit;
  the lock files (their fields, a dead process, a file of another shape, the auth token never
  kept); where `Code.exe` is looked for; the launch with an injected starter (the calls in order
  with their environment, the own file written or kept, a window open already or appearing, one
  that never appears or a cancel firing nothing, the ledger row); the visual opener; the dialog's
  default mode and its switch.
- Screenshot runs: `launch=lantern` (Visual) and `launch=harbor` (Headless) at 1600 × 1000 light
  and a narrow window dark, each picture looked at.
- A supervised real run on the owner's yes, through a throwaway driver that writes no ledger row,
  at this project's open window.
- The commit's added lines pass rule 3's word check; the owner has reviewed every decision.

> **Completion note (2026-11-02; opus/xhigh; ≈ 1 h 40 min, about 50 active).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`, from 15:43 to the commits: the reading and the
> checks at pickup, this brief, the code and its tests, the pictures, then the owner's first round
> (38 min), the supervised launch, the owner's reading of its tab (14 min), the records, the
> closing round (under five minutes) and the commits. Against S ≈ 30 min. The model and effort are
> the user settings' (`model = opus`, `modelSettings` giving Opus 5.5 `xhigh`).
>
> **What was built.** `core::visual`: the link, its prompt escaped for VS Code's decoding and the
> extension's `URLSearchParams` (`%`, `&` and `+` twice), refused over 32,000 characters; the
> extension's windows from its lock files, a file counting while its process runs, the auth token
> in no field; `Code.exe` located (`settings.vscode`, the user install, the system install, the
> install of `code.cmd` on PATH); `Readiness` for the dialog; the launch (`visual::open`): the own
> workspace file written when missing or kept, `Code.exe <file>`, a wait of up to 60 s for a live
> lock file with the project first, a 500 ms pause, `Code.exe --open-url -- <link>`, a `visual`
> ledger row; a cancel hands nothing over; its children one at a time without the API key, the
> session markers and Electron's and VS Code's variables. `rules::Resolution::mode`, the project's
> `[project.launch] mode` over `DEFAULT_LAUNCH_MODE` (Visual); `prompt::Chosen::visual` and the
> visual opener; the registry's `vscode` setting; `ledger::NewSession::session_id` optional;
> `OwnFile::path`; `crw prompt [--visual|--headless]` with its `launch` line. In the window: the
> dialog's Visual | Headless switch with its reason, the model, effort and ultracode marked as a
> suggestion in Visual, the permission mode and the neighbours headless only, a VS Code line in
> place of the Claude line (looked at again every two seconds), the footer's note and one button
> per mode (Open in VS Code), Cancel calling off a launch that waits; `app::visual` runs the launch
> on a worker thread and words its notice; a screenshot run's canned VS Code, and the fixture
> registry's harbor headless and lantern visual.
>
> **Verification.** `cargo test` 238 passed (29 + 175 + 2 + 7 + 6 + 11 + 8), 240 with `--features
> screenshot`; both clippy runs and `cargo fmt --check` clean; `cargo build --release` with the app
> closed, then the install script. New tests: nine prompts read back through models of VS Code's
> decoding and of `URLSearchParams` (spaces, new lines, `%`, `&`, `+`, `=`, `#`, `%41`, Unicode, the
> empty prompt), the page's own example as it shows, and a link escaped once cut at its `&`; the
> length limit; the lock files (two other shapes passed over, a process gone, the token in no
> field); where `Code.exe` is looked for; the variables a `Code.exe` child withholds (tested
> through `withheld`, as the stand-in starter receives only the arguments); the launch through a
> stand-in starter: the own file written, the two calls in order and the ledger's visual row; an
> open window and the owner's file kept byte for byte; a stale lock file with no window, a cancel,
> a prompt too long, each handing nothing over (the last writing nothing); the readiness; the
> visual opener; the mode's resolution; the `vscode` setting; the dialog's mode from the registry,
> its switch, the visual launch it builds and an edited prompt kept; the notice. Screenshot runs,
> each looked at: `launch=lantern` at 1600 × 1000 light (Visual), `launch=harbor` at 1600 × 1000
> light (Headless: EP-10's dialog under the switch), `launch=lantern` at 1100 × 760 dark. `crw
> prompt` on the fake registry in both wordings, a wrong flag refused, and on this repository read
> only. `crw roadmap .` reads this brief's front matter and 30 decisions with 34 addenda, no
> problem; `crw tasks .` reads OT-5 with its gate. Rule 3's word list found nothing in the added
> lines or the new files. **The supervised launch** (decision 1): a throwaway driver, deleted
> afterwards, ran `visual::open` at this project's open window, with no ledger, on a test prompt
> holding `&`, `+`, `100%`, `%41`, quotes, a new line, Unicode and an emoji; from this session's
> shell, so with `ELECTRON_RUN_AS_NODE` and the session markers to withhold. It found VS Code, the
> extension 2.1.314, the own file (kept) and the window, handed the link over 2.5 s after opening
> the workspace file (5.9 s in all), and the owner read the new tab's prompt exactly as written,
> unsent; no confirmation dialog was reported.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-02).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | One supervised Visual launch at this window, a test prompt, nothing sent | Yes, run it now | Run; the tab held the prompt exactly; the driver deleted |
> | 2 | The tool's launch mode when a project names none | Visual, revisit at EP-16 | `rules::DEFAULT_LAUNCH_MODE`; D-16 addendum; EP-16's brief notes |
> | 3 | The three addenda | Accept as written | D-15, D-16, D-17 |
> | 4 | README, roadmap and v2-notes edits; OT-5 | Keep as written | As drafted; OT-5 gated before EP-12 |
> | 5 | A cross-project memory note in the owner's environment repository | Write it | One memory file on the link and the environment, and its index line, uncommitted there |
> | 6 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).** (1) A Visual | Headless switch at the head of
> the choices rather than a second button in the footer: the prompt's model sentence differs by
> mode, so the editor shows what is sent. (2) The own workspace file is opened even when the
> project's window is open, and the link waits for a live lock file: the link goes to the focused
> window. (3) A lock file's `pid` is VS Code's main process, which every window shares, so a window
> closed while VS Code runs is known only by the extension removing its file; accepted. (4)
> `Code.exe` is started directly with `--open-url --`, as Windows' handler writes it, never
> `code.cmd`. (5) The `Code.exe` that hands its arguments over is waited for up to 10 s under the
> process turn; one still running then is VS Code itself, left to run. (6) A 500 ms pause between
> the window's showing and the link, for the focus. (7) A `vscode` setting beside `claude_cli`,
> since it costs nothing (D-1). (8) The ledger row is written once the link is handed over, its
> model and effort empty. (9) `Win32_Foundation` named among the `windows` features, compiled in
> already through process-wrap (D-15 addendum). (10) The VS Code line is read again every two
> seconds while the dialog shows Visual, with a frame asked as often. (11) The fixture registry
> sets both modes, so the pictures do not hang on the default. (12) The extension's lock files are
> only read, never written or deleted. (13) `crw prompt` takes `--visual` or `--headless` after the
> project; another word is refused.
>
> No messages for other projects. **Next: EP-12** (Loopback server and signals).
