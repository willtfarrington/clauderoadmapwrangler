---
ep: 12
title: Loopback server and signals
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-12 — Loopback server and signals

**Size:** M · **Depends on:** EP-9 · **Blocks:** EP-13 (the sessions tile reads the state model
and the status-line figures), EP-14 (Focus and "waiting for you in VS Code" for a visual session),
EP-15 (toasts on waiting, finished and failed), EP-17 (the status-line feed), EP-20 (a visual
session's completion seen through its hooks), EP-30 (the server's phone routes)

## Context

The window knows only the headless sessions it starts itself. D-20 gives it the rest: user-level
hooks (Notification, Stop, SessionStart, SessionEnd) that run the tool's `crw`, which posts the
event to a loopback server in the window and exits silently when no window runs; a status-line
command installed the same way, which posts each session's documented `rate_limits`,
`context_window` and `cost` (D-23); and a poll of `claude agents --json` as reconciliation for the
sessions started without the tool. D-28 binds the server to 127.0.0.1 only, trusts loopback
callers, and has the hook and status-line commands path-bound to the tool's `cli` and failing
silently. The installation writes into the owner's user settings, so it waits for the owner's yes
(rule 7), after a dated copy of the file. A running session then joins the snapshot's
in-progress reasons (D-9's EP-4 addendum left it to this brief), and a visual launch's ledger row
gains its session id when the session starts (EP-11's note).

**Session start (2026-11-02, 17:27).** Started from `DATA\clauderoadmapwrangler.code-workspace`
in the VS Code extension. OT-5 is ticked by the owner (in the working tree); no inbox message is
open, so nothing is triaged. The model is Opus 5.5 (`model = opus` in the user settings); the
session's environment gives the effort as max.

**Verified at pickup.**

- The user settings hold no status line and one hook: the owner's stopgap `SessionStart` entry
  (matcher `startup|resume`) that runs `pwsh -File …\.claude\hooks\session-start-inbox.ps1`, which
  prints the project's open `roadmap/inbox.md` lines into the session (IN-2). The file is two-space
  JSON with a final newline, as Claude Code writes it.
- Claude Code's hooks page (read 2026-11-02): settings hold `hooks.<Event>[]` matcher groups of
  handlers; a `command` handler may give `args`, and then "`command` is resolved as an executable
  and spawned directly … with no shell involved" (the changelog: since 2.1.139); otherwise the
  command runs in Git Bash on Windows, or PowerShell without it. `timeout` defaults to 600 s. Exit 0
  is success, and only `SessionStart`, `UserPromptSubmit` and two others add stdout to the
  session's context; another exit code is a non-blocking error the session shows as
  "`<hook> hook error`". Hooks of an event run in parallel; the same handler in two settings files
  runs once. Common input: `session_id`, `transcript_path`, `cwd`, `permission_mode`, `effort`,
  `hook_event_name`; `SessionStart` matchers `startup`, `resume`, `clear`, `compact`, `fork`;
  `SessionEnd` reasons `clear`, `resume`, `logout`, `prompt_input_exit`, `other`; Notification
  types `permission_prompt`, `idle_prompt`, `elicitation_dialog`, `agent_needs_input`,
  `agent_completed` and more. `type: "http"` hooks post the input to a URL themselves (since
  2.1.63), but a refused connection is a non-blocking error, so a closed window would show an
  error at every event: the command relay stays.
- The status-line page: `statusLine` takes `type: "command"`, `command`, `padding`,
  `refreshInterval` (seconds, since 2.1.97) and `hideVimModeIndicator`; no `args`, so the command
  runs in Git Bash, which wants forward slashes. It runs at session start, after each assistant
  message (debounced 300 ms, an in-flight run cancelled), on the interval and at a rate-limit
  window's reset; stdin carries `session_id`, `cwd`, `workspace.project_dir`, `model.id` and
  `display_name`, `cost.total_cost_usd`, `context_window.used_percentage`,
  `rate_limits.five_hour|seven_day.{used_percentage, resets_at}` (Pro, Max and Ultra, after the
  first response), `effort.level`, `version`, `session_name`. A configured status line hides most
  of the terminal footer's key hints. The VS Code extension (2.1.314) names `statusLine` only in
  its settings schema, and its panel shows plugins' statuses, not the user's command: the feed is
  most likely a terminal sessions' feed only, which EP-17's degraded meter inherits.
- `claude agents --json` (2.1.316; since 2.1.145, `waitingFor` 2.1.162, `state` and `--all`
  2.1.169) prints the live sessions as an array of `pid`, `cwd`, `kind`, `startedAt` (ms),
  `sessionId`, `name`, `status` (`busy`, `idle`, `waiting`), and for background sessions `state`
  and `waitingFor`; it lists sessions of every folder, so the tool keeps the unregistered ones out
  of sight. One call takes about 270 ms and 0.2 s of CPU: a 5 s poll costs about 4 % of a core.
- A session's children inherit `CLAUDE_CODE_ENTRYPOINT` (`claude-vscode` here; the terminal and
  `-p` runs carry their own), so the relay can tell an interactive session from another tool's
  `claude -p` run in a project's work folder, whose hooks fire too.
- A refused connection to a closed loopback port fails at once on this machine (curl, under
  0.1 s with its start-up), so a relay with no window to talk to costs a process start.
- axum 0.8.9 is pinned in the workspace since EP-1 and used by no crate yet; the lock holds none of
  its stack (hyper, http, tower). tokio lacks its `net` feature. serde_json keeps object keys
  sorted unless its `preserve_order` feature is on; the lock already holds `indexmap`.

## In scope

1. **The signals** (`core::signals`): the narrow forms a hook's and a status line's input are
   read into (the session id, its folder, the event, its source, reason or notification type;
   the model, cost, context and rate limits), never a transcript's text; `crw hook` and `crw
   statusline` read stdin, keep those fields with the session's entry point, post them to
   `127.0.0.1:<port>` within 500 ms over a plain HTTP/1.1 request, print nothing and exit 0 on any
   error. With `--inbox`, `crw hook` also prints a project's open inbox messages on
   `SessionStart` (startup or resume), as the stopgap hook does, so the installation can replace
   it on the owner's yes.
2. **The port**: a registry setting `port` (default 47831), read by the window and the relays;
   `crw setting port <n>`.
3. **The server** (`app::server`): axum on `127.0.0.1:<port>` on a thread of its own, `POST /hook`,
   `POST /statusline`, `GET /health`; JSON bodies only, a request with an `Origin` header or a
   `Host` other than the loopback refused (a web page cannot reach it), each signal handed to the
   window with a repaint; a port in use said in the status bar.
4. **The installation** (`core::signals::install`, `crw signals [status|install|remove]`): the four
   hooks in exec form (`command` the full path of the release `crw.exe`, `args` `["hook"]`, a 5 s
   timeout) and the status line (`<crw.exe> statusline`, forward slashes, `refreshInterval` 30),
   written after a dated copy of the settings file in the tool's backups folder, key order and
   layout kept; never over another status line (said, not chained); refused from a debug build;
   `status` says what is installed and whether it runs the current exe; `remove` takes the tool's
   entries out and nothing else. The stopgap entry is taken out only when the owner chooses
   `--inbox`.
5. **The state model** (`core::sessions`): sessions by id from the hooks, the status lines, the
   window's own headless runs and the `claude agents --json` poll (every 5 s while the window has
   the focus, 30 s otherwise, one child under the turn, the session markers withheld); the folder
   reconciled to a registered project (the folder or one inside it, case aside); mode visual
   (a visual launch's row claimed), headless (the window's own, or another tool's `-p` run) or
   manual; status busy, waiting (with what for), idle, done or failed; a session the poll no longer
   lists and no hook mentions is done; ended sessions leave after ten minutes.
6. **The ledger**: `SessionStart` completes the newest open visual row of the project (its session
   id), else opens a `manual` row for an interactive session of a registered project;
   `SessionEnd` closes a visual or manual row; the runner keeps its own rows.
7. **The window**: the Sessions pane lists the sessions seen besides its own (project, mode,
   status, what it waits for, since when, the status line's cost and context) and counts the ones
   outside the registry; the row's Session cell, the Detail pane and the launch dialog's warning
   take a running session as an in-progress reason (`InProgress::Session`, D-9); the status bar
   says where the server listens and whether the signals are installed. A screenshot run binds no
   port and polls nothing; a setup flag adds canned sessions.
8. **Owner decisions** through `AskUserQuestion` (rule 7): installing into the user settings, the
   stopgap hook, the `@<project>` prompt hook's place, the addenda, the edits, the commit.

## Out of scope

- The `UserPromptSubmit` hook that injects a mentioned project's rules and next row (IN-2's
  wish): proposed for EP-20 with the mailman's registry lookups, the owner deciding.
- Chaining another status line (none is configured), the usage samples and the meter (EP-17),
  transcripts and Resume (EP-13), answering and Focus (EP-14), toasts and the tray (EP-15),
  Diagnostics' check of the entries and single instance by `/health` (EP-28), the phone routes
  (EP-30).
- Rows for sessions seen only by the poll (no start or end time the tool can vouch for).
- Tests that run the real CLI or bind the owner's port: the poll's runner is injected, the server
  binds an ephemeral port, the installer works on scratch files (D-29).

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`, `cargo
  build --release` with the app closed, then the install script.
- Unit tests: the narrow forms read from synthetic hook and status-line inputs (unknown fields,
  missing ones, a transcript's text never kept); the relay against a stand-in listener (the
  request it writes, the 500 ms budget, a closed port, silence) and against the real server on an
  ephemeral port (the window receives the signal; an `Origin` or a foreign `Host` refused); the
  inbox lines on `SessionStart`; the installer on scratch settings files (an empty file, the
  owner's form with a stopgap entry, a foreign status line, a second install changing nothing,
  remove leaving the rest byte for byte, the copy made); the agents output parsed; the state model
  through each feed and the reconciliation; the ledger's claim, manual row and close.
- `crw signals status` on the owner's settings, read only; `crw hook` and `crw statusline` timed
  with no window listening.
- Screenshot runs with the canned sessions: the Sessions pane and a row's Session cell, light at
  1600 × 1000 and dark at a narrow window, each picture looked at.
- On the owner's yes, `crw signals install` on the owner's settings, `status` after it, and an
  owner task to watch a real session arrive in the window.
- The commit's added lines pass rule 3's word check; the owner has reviewed every decision.

> **Completion note (2026-11-02; opus/max; ≈ 1 h 10 min, about 1 h active).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`, from 17:27 to the commits: the reading and the
> checks at pickup, this brief, the code and its tests, the pictures, the owner's first round (8
> min), the install into the user settings, the records, the closing round (under five minutes) and
> the commits. Against M ≈ 1 h. The model is Opus 5.5 (`model = opus` in the user settings); the
> session's environment gives the effort as max.
>
> **What was built.** `core::signals`: the narrow forms of a hook's and a status line's input
> (`Hook`, `StatusLine`), never a transcript's text; the relays `relay_hook` and
> `relay_status_line` (stdin read, the session's surface from `CLAUDE_CODE_ENTRYPOINT`, one HTTP/1.1
> POST over a plain socket within 500 ms, of which the connection gets 50); the text a start is
> given with `--inbox` (`inbox_context`, over `inbox::open_as_written` and `checklist::Item::written`);
> `signals::install`: the user settings' entries (exec-form hooks for SessionStart, Notification,
> Stop and SessionEnd with a 5 s timeout, the status line every 30 s), `status`, `preview`,
> `install` after a dated copy, `remove`, the stopgap taken out with `--inbox`, another status line
> kept and said, a debug build refused. `core::sessions`: the state model (`Sessions`, `Seen`,
> `Mode`, `Status`, `project_of`, `interactive`), `agents` (the `claude agents --json` listing, asked
> in its turn with the session markers withheld) and `record`, which keeps the ledger's rows
> (`Ledger::mode_of`, `claim_visual`, `end_seen`). The registry's `port` (`Registry::port`, `crw
> setting port`; `Settings::get` gives text now); `InProgress::Session`; `crw hook [--inbox]`, `crw
> statusline` and `crw signals [status|install [--inbox|--no-inbox] [--no-status-line]|remove]`. In
> the window: `app::server` (axum on 127.0.0.1 with `POST /hook`, `POST /statusline`, `GET /health`
> and the loopback guard), `app::watch` (the server, the poll at 5 s with the focus and 30 s
> without, the model, the ledger's rows, the entries read at the start and on F5), started from
> `main` only; the Sessions pane's "Started elsewhere" cards and the count of sessions outside the
> registry, the tab's count, the rows' Session cell, the Detail pane and the launch dialog's
> warning taking a running session, the status bar's signals line; a screenshot run's `sessions`
> flag.
>
> **Verification.** `cargo test` 258 passed (31 + 193 + 2 + 7 + 6 + 11 + 8), 260 with `--features
> screenshot`; both clippy runs and `cargo fmt --check` clean; `cargo build --release` with the app
> closed, then the install script. Twenty new tests: the hook's and the status line's forms (other
> fields left behind, a transcript's text never sent, missing fields); the relay against a stand-in
> listener (the request it writes, a listener that never answers costing the budget, a closed port
> costing the connection's share, input that is not a hook's posted nothing) and against the real
> server on a free port (both signals heard, `/health`; an `Origin`, a foreign `Host` and a body
> that is not JSON refused; a second server told the port is taken); the inbox text (open messages
> as written, a continuation line kept, only a startup or a resume, printed with no window); the
> installer on the owner's form (read back byte for byte, an install and a removal leaving the rest
> as it was, a second install changing nothing, the stopgap replaced, a moved repository's entries
> replaced, another status line kept, entries written by hand as shell commands known, the copy
> made, a file that is not JSON left alone, a debug build refused); `claude agents --json` read; the
> state model through each feed (a session's life through its hooks, an SDK's run headless, the
> poll's reconciliation and its two missed listings, the expiry); the ledger's claim, manual row and
> close; the port setting. Screenshot runs, each looked at: the Sessions pane with the canned
> sessions at 1600 × 1000 light (twice: the first showed a three-line chip and a sentence with two
> colons, reworded into the status alone and "waits for a permission prompt"), at 1100 × 760 dark
> (twice: a chip squeezed into a column of letters, the card then laid out in two rows), and the
> launch dialog's warning on lantern. `crw signals` read the owner's settings before the install:
> the stopgap and no entry, the file read back byte for byte; a dry run on a scratch copy
> (`CLAUDE_CONFIG_DIR`) changed only the entries and a second run nothing, the copy deleted. The
> relays timed with no window listening took 525 ms each, Windows refusing a closed loopback port
> only after its own retries, so the connection's share became 50 ms and they took 76 ms. Rule 3's
> word list found nothing in the added lines or the new files.
>
> **The install** (decisions 1 and 2). `target\release\crw.exe signals install --inbox` on the
> owner's settings: four hook groups and the status line added, the stopgap entry taken out, the
> file as it was kept as `%LOCALAPPDATA%\clauderoadmapwrangler\backups\claude-settings\settings.2026-11-02.json`;
> `crw signals` then said every entry runs this exe, with no window listening. It also showed a
> flaw, fixed at once: its preview took the inbox choice from the stopgap, gone by then, so a later
> install would have stopped the inbox print; an install now keeps `--inbox` once the start hook
> prints the inbox, and `--no-inbox` turns it off.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-02).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Install the hooks and the status line into the user settings now | Hooks and status line | `crw signals install --inbox`; D-20 and D-23 addenda |
> | 2 | The stopgap SessionStart inbox hook | Replace it | Its entry out, `crw hook --inbox` prints the inbox; the script left for the environment repository |
> | 3 | IN-2's `@<project>` prompt hook | EP-20's brief notes | A "From EP-12" note under EP-20 |
> | 4 | The poll's pace | 5 s with the focus, 30 s without | As built; D-20 addendum |
> | 5 | The six addenda | Accept as written | D-9, D-14, D-20, D-23, D-26, D-28 |
> | 6 | README, roadmap notes, risks, v2 notes, OT-6 | Keep as written | As drafted; OT-6 gated before EP-13 |
> | 7 | The hand-on to the environment repository | Inbox line and memory note | IN-2 in its `roadmap/inbox.md` (the question said IN-5; its inbox held IN-1 only) and one memory file with its index line, uncommitted there |
> | 8 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).** (1) The hooks in exec form (`command` and
> `args`, since Claude Code 2.1.139): no shell, so no quoting and no Git Bash start; the status line
> has no exec form, so its path is written with forward slashes. (2) The relays post narrow forms of
> their own rather than the raw input: a Stop hook's input holds the turn's last message (D-3). (3)
> A hand-written HTTP/1.1 request over a plain socket instead of a client crate, and the connection
> given 50 ms of the 500. (4) The port read from the registry at each relay (`CRW_HOME`'s in a
> trial), 47831 when it does not read. (5) A 5 s timeout on each hook entry as a backstop for
> Claude Code's 600 s default; no `async`, whose timeout is not enforced. (6) `type: "http"` hooks
> passed over: a refused connection is a hook error the session shows. (7) The server's guard (no
> `Origin`, a loopback `Host`, JSON of at most 256 KiB) beside the loopback bind. (8) serde_json's
> `preserve_order` for the whole build, so the settings keep their order; `Signal::StatusLine`
> boxed (clippy). (9) A session's mode from its surface: an SDK's (`sdk-…`) is headless, another
> program's `-p` run included, and gets no ledger row. (10) A visual launch claimed within 30
> minutes of its row; manual rows for the other interactive sessions of registered projects;
> nothing for sessions only the poll saw. (11) A session the poll listed and then leaves out twice
> has ended; ended ones leave the window after ten minutes, silent ones after six hours. (12) The
> server and the poll start from `main` only: tests and screenshot runs never bind a port or run
> the CLI. (13) Sessions outside the registry counted, never named, in the window. (14) An install
> refused from a `target\debug` exe, which every test run replaces. (15) The seen card in two rows,
> so a narrow pane never squeezes a chip.
>
> **Handed on (to the owner).** For the owner's environment repository (its D-7 stopgap and the
> brief it waits on): crw's EP-12 installed its signals into `~\.claude\settings.json` on
> 2026-11-02 with `crw signals install --inbox`, the file as it was kept in crw's backups folder;
> the SessionStart, Notification, Stop and SessionEnd hooks run the release `crw.exe` directly
> (the start hook with `--inbox`), and a status line runs it every 30 s; the stopgap `SessionStart`
> entry is out, `crw hook --inbox` printing the same text on a startup or a resume, so one inbox
> print remains; the script is called by nothing and left in place; each hook costs about 75 ms
> while no window runs. Delivered at the owner's choice as that repository's IN-2, with a memory
> file and its index line, uncommitted there.
>
> **Next: EP-13** (Sessions tile), after OT-6.
