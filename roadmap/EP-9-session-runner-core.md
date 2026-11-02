---
ep: 9
title: Session runner core
size: L
model: claude-opus-5-5
effort: max
status: done
---

# EP-9 — Session runner core

**Size:** L · **Depends on:** EP-2 · **Blocks:** EP-10 (the launch dialog and its Headless button),
EP-12, EP-13, EP-14, EP-17 … EP-19, EP-24

## Context

P2 opens with the second engine (D-4): the runner that starts a headless Claude Code session in a
project, reads what it says, answers what it asks, stops it, and records it. Nothing launches a
session from the window yet (EP-10 adds the dialog), so this brief builds the engine and proves it
against a stub `claude` that replays recorded stream-json (D-29); the real CLI is never run by a
test. The runner passes a project's neighbour folders read-only (the D-16 and D-18 addenda of
EP-8) and finds out what the `Edit` deny rule behind that promise covers.

**Session start (2026-11-01, 23:49).** Started from `DATA\clauderoadmapwrangler.code-workspace`. No
owner task is open. The inbox holds IN-6 (the untracked `.claude\settings.local.json` grants the
whole data root, so the data repositories are among this project's working directories), which
suggests this brief as its home: it fits the neighbour-folder brief and is S-sized, so it is rolled
in (scope 9).

**Verified at pickup.**

- `claude --version`: 2.1.316 at `%USERPROFILE%\.local\bin\claude.exe`. Its `--help` lists `-p`,
  `--verbose`, `--output-format` and `--input-format` with `stream-json`,
  `--include-partial-messages`, `--replay-user-messages`, `--session-id <uuid>`, `-n, --name`,
  `--model`, `--effort <level>` with `(low, medium, high, xhigh, max)` and no `ultracode`,
  `--permission-mode` with six choices, `--permission-prompts host|none` (default `host`),
  `--add-dir <directories...>`, `--settings <file-or-json>` and `--max-budget-usd`; still no
  `--max-turns`. The help wraps at 80 columns when piped, so a flag's description spans lines.
- `claude-codes` 2.1.316, the pin, compiled with `default-features = false, features = ["types"]`
  brings only its serde models (with chrono, serde_json, thiserror and uuid): `ClaudeOutput` with
  `system/init` (`capabilities`, `claude_code_version`, `model`, `effort`, `permissionMode`,
  `tools`), `assistant`, `stream_event`, `control_request` (`can_use_tool` with
  `ToolPermissionRequest::{allow, deny, answer_questions}`, the last keying answers by the question's
  text and keeping `questions`), `control_cancel_request` and `result` (`total_cost_usd`,
  `stop_reason`, `deferred_tool_use`, `permission_denials`); `ControlRequestMessage::{initialize,
  interrupt}`. Its clients send `initialize` before anything else and wait for the matching
  `control_response`: the handshake whose failure selects EP-14's fallback. An unknown top-level
  `type` fails to parse, so the adapter must tolerate one. The 2.1.316 wire schema has
  `system/permission_denied` (`tool_name`, `message`, `decision_reason`), which is how a refused
  edit reaches the host.
- process-wrap 10.0.1: the std flavour's `JobObject` never sets kill-on-close; the tokio flavour
  sets `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` when `KillOnDrop` is wrapped as well, which is what D-14
  ("process plumbing with tokio and process-wrap job objects") and the research pass ("`JobObject`,
  kill-on-close") planned. `JobObject` replaces a command's creation flags with its own, so the
  no-window flag goes through the `CreationFlags` wrapper, whose type comes from the `windows` crate
  0.62.2 (locked already through process-wrap). Its `wait` returns soon after the main process
  exits, and dropping the child closes the job, so a grandchild left behind dies with the session.
  tokio 1.54.1 is locked with no features; uuid 1.29.0 is locked (the pre-assigned id needs `v4`).
- The ledger's `session` table (schema V1) has every column the runner fills. `process::turn()` and
  `process::output_with` (no window, `ANTHROPIC_API_KEY` removed) serve the preflight's children.
- Claude Code's permissions page (read 2026-11-02): "Read and Edit deny rules apply to Claude's
  built-in file tools, to file commands Claude Code recognizes in Bash, such as `cat`, `head`,
  `tail`, `sed`, and `tee`, and to the targets of Bash redirections such as `> file` and `< file`.
  They don't apply to a command that reads files without naming them … or to arbitrary subprocesses
  that read or write files indirectly, like a Python or Node script that opens files itself";
  `acceptEdits` accepts "common filesystem commands such as `mkdir`, `touch`, `mv`, and `cp` for
  paths in the working directory or `additionalDirectories`"; `Edit` rules cover every built-in
  editing tool; Windows paths match in POSIX form (`C:\Users\alice` becomes `/c/Users/alice`, an
  absolute pattern starts `//`); `--add-dir` folders load their `CLAUDE.md` and rules only with
  `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1`; an untracked local settings file needs no trust
  step. Whether `mkdir`, `touch`, `mv` and `cp` are among the recognised commands is not stated.
- IN-6: `.claude\settings.local.json` holds the data root's grant and nothing else.

## In scope

1. **Discovery and preflight** (`core::session::cli`): the CLI located from `settings.claude_cli`,
   else `~\.local\bin\claude.exe`, else PATH; `claude --version`, `claude auth status` (its exit
   code, `loggedIn`, `authMethod`, `subscriptionType`) and `claude --help` (the flags it lists, the
   effort levels, the permission modes), one child at a time under `process::turn()`; a cache at
   most an hour old; the reasons a headless launch is refused: no CLI, a version below
   `MIN_CLAUDE_CODE`, not logged in, a flag the runner needs missing.
2. **The adapter** (`core::session::protocol`), the one module that names `claude-codes`' types. It
   reads a stdout line into the events the tool uses (the init with version, model, effort, mode,
   tools and capabilities; the handshake's answer; text as it streams; an assistant message's text;
   each tool use in one line; a request, either a permission with the tool and its reason or an
   `AskUserQuestion` with its questions and options; a withdrawn request; a refused tool use; the
   result) and writes the stdin lines (the handshake, the prompt as a user message, the answers:
   allow, deny with a message, or the chosen options keyed by question text; the interrupt). An
   unreadable or unknown line becomes an event, never an error; a control request the tool does not
   handle is answered with an error so the session never waits on it.
3. **The runner** (`core::session`): a `Launch` (project, folder, EP, model, effort, ultracode,
   permission mode, neighbour folders, prompt, budget, stop timeout, ledger); the command line `-p
   --verbose --output-format stream-json --input-format stream-json --include-partial-messages
   --replay-user-messages --session-id <uuid> --name "<project> EP-n" --model <id> --effort <level>
   --permission-mode <mode> --permission-prompts host`, an `--add-dir` per neighbour with
   `--settings` carrying an `Edit(//<drive>/<path>/**)` deny rule per neighbour, `--effort
   ultracode` and `--max-budget-usd` only when the CLI advertises them (a dropped wish is reported);
   the child started in the project folder without `ANTHROPIC_API_KEY`, with
   `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1` when it has neighbours, without a console
   window, inside a job object that is killed when its handle closes; at least 2 s between session
   spawns (D-29); one thread with a small tokio runtime per session; events on a channel; respond,
   stop (the interrupt, stdin closed, up to 30 s, then the job killed), kill, and the kill when the
   handle is dropped; stdin closed after the result, so a run is one prompt and a follow-up resumes
   by id (EP-13); the outcome (completed, stopped, failed, deferred) read from the result even on a
   non-zero exit, with the exit code and the last lines of stderr.
4. **The ledger**: a `session` row at the start (project, EP, mode `headless`, model, effort,
   ultracode, start time, session id) completed at the exit (end time, outcome, the CLI's cost
   estimate, the questions asked).
5. **The registry**: `neighbours` per project (folders of the data root or full paths; never the
   project's own folder or a network path) and `settings.claude_cli`.
6. **The stub** `stub-claude`, a binary target of `crw-core` used only by tests (its source under
   `crates/core/tests/stub_claude/`): it plays a script from its working folder, stream-json lines
   with fake names between directives (answer the handshake, take the prompt, wait for a given
   control response, wait for the end of stdin, hang, exit with a code, start a grandchild that
   beats a heartbeat file), records its arguments, chosen environment variables and every line it
   reads, and answers `--version`, `auth status` and `--help` with synthetic text. Every runner test
   drives it (D-29).
7. **The deny question** (D-18 addendum): the documentation's answer recorded; on the owner's yes,
   one supervised probe, a real headless run through the runner (the cheapest model at low effort,
   in scratch folders, the host denying every request) that tries `mkdir`, `touch`, `cp`, `mv` and
   the Write tool in a neighbour under its deny rule, with the working folder as the control; any
   gap goes to the owner.
8. **`crw doctor`** prints the preflight: logged in or not, and the features found (no usage
   spent).
9. **IN-6**: the data root's entry in `.claude\settings.local.json` dropped or narrowed as the owner
   chooses, rule 11's sentence brought along.
10. **Owner decisions** through `AskUserQuestion` (rule 7): the probe, IN-6, any gap the deny
    question leaves, the commit.

## Out of scope

- Launching from the window or from `crw` (EP-10's dialog and Headless button), the prompt
  generator (EP-10), the sessions tile and Resume (EP-13), the decisions tile, its timeout and the
  defer-and-resume hook (EP-14), the hooks, the loopback server and the state model (EP-12), the
  autopilot's caps (EP-19), usage attribution (EP-18).
- Raising `MIN_CLAUDE_CODE` unless the adapter needs a newer CLI; moving a pin (D-15).
- The owner's registry and ledger (rule 13): trials use `CRW_HOME` or temp folders.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`, `cargo
  build --release` with the app closed, then the install script.
- Runner tests against the stub only: a whole run (handshake, init, streamed text, a tool use, a
  question answered, a permission denied, the result, outcome completed, the ledger row and the
  recorded command line); the failures (an error on stderr at start, an error result with a non-zero
  exit); the stops (graceful, and the kill after the timeout taking the grandchild with it); the
  kill when the handle is dropped; unreadable lines tolerated; a deferred result.
- Unit tests: the help and auth readings, the command line and its environment (no API key), the
  POSIX deny patterns, the adapter's reading and writing, the outcomes, the ledger's queries, the
  registry's new keys.
- No test or trial runs the real CLI; the probe, if the owner says yes, is a supervised run by this
  session, never a test, and leaves no code behind.
- The commit's added lines pass rule 3's word check; the owner has reviewed every decision.

> **Completion note (2026-11-02; opus/max; ≈ 1 h 5 min, all active).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`, from 23:49 on 2026-11-01: the reading, this brief, the
> owner's first two answers (the probe, IN-6), the code and its tests, the probe, the records, then
> the owner's second round. No wait passed five minutes. Against L ≈ 2 h.
>
> **What was built.** `core::session` (the runner: `Launch`, `Session::start`, `respond`, `stop`,
> `kill`, the events, `Ended` and `Outcome`), `session::cli` (the preflight and its cache, the help's
> and the login's readings), `session::launch` (the command line, the environment, the deny rules)
> and `session::protocol` (the adapter, the only module that names `claude-codes`); the ledger's
> session queries; the registry's `neighbours` and `settings.claude_cli`; `crw doctor`'s preflight;
> `stub-claude` with six scripts in `crates/core/tests/streams/`; IN-6.
>
> **The probe** (decision 1). One headless run through the runner with a throwaway driver, in two
> scratch folders under the session's temp folder: CLI 2.1.316, the cheapest model at low effort,
> the host denying every request. The preflight refused nothing, the handshake was answered, the init
> named fourteen capabilities and no line went unread; the control `mkdir` in the working folder ran,
> while `mkdir` by an absolute and by a relative path, `touch`, `cp`, `mv` and the Write tool in the
> neighbour were refused by its deny rule without a prompt (six denials, the neighbour left empty);
> the result came in 8 turns and 19 s at an estimated $0.012, the outcome completed, exit 0, and the
> scratch ledger row was whole. The driver files were deleted and the probe's transcript removed with
> `claude purge -y`; nothing of it is in the repository. The documentation's remaining gap (a program
> that opens files itself) went to the owner (decision 3).
>
> **Verification.** `cargo test` 198 passed (22 + 143 + 2 + 7 + 6 + 11 + 7), 200 with `--features
> screenshot`; both clippy runs and `cargo fmt --check` clean; `cargo build --release` with the app
> closed, then the install script. The runner's seven tests drive `stub-claude` only: a whole run (the
> handshake, the init, streamed text, a tool use, a question answered by its text, a permission
> denied, a refusal, the result, the ledger row, the command line and the environment the stub saw),
> a refused command line and an error result with a non-zero exit, a graceful stop, a stop that runs
> out of time and kills the stub's grandchild with it, the kill when the handle is dropped, odd lines
> with a refused handshake and a deferred result; about 14 s, the spacing. `crw doctor` with
> `CRW_HOME` on a scratch folder read the real CLI's login and help (headless ready; effort low to
> max; no ultracode; `--max-budget-usd`). `crw roadmap .` reads 6 phases, 36 rows, 10 done, next
> EP-9, 30 decisions with 26 addenda and no problem. Rule 3's word list found nothing in the added
> lines or the new files.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-02).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | One supervised probe of the neighbour deny rule | Run the probe | As above; D-18 addendum |
> | 2 | IN-6, the data root's grant in the local settings file | Drop it | The file (holding nothing else) deleted; rule 11's sentence; IN-6 ticked |
> | 3 | The gap the deny rule leaves (a program that opens files itself) | Accept; park a sandbox | Stated in the D-18 addendum; v2 notes §1 |
> | 4 | The six addenda | Accept as written | D-14, D-15, D-16, D-18, D-19, D-29 |
> | 5 | README, roadmap, CLAUDE.md and inbox edits | Accept as written | As drafted |
> | 6 | Commit and push | Commit and push | Two commits in the house pattern, pushed; the CI run recorded in a third |
>
> **Routine choices (the session's call, logged).** (1) A thread with a single-threaded tokio
> runtime per session, and process-wrap's tokio flavour, since only its job object is killed when its
> handle closes (D-14 as planned, addendum). (2) `claude-codes` with `types` only; `uuid` (v4) and
> `windows` pinned at the versions the lock held. (3) The child withholds a parent session's markers
> as well as the API key, the machine's lesson about nested `claude -p` (no such variable is set for
> the user or the machine, so every one is a marker). (4) A run is one prompt: stdin closes at the
> result. (5) The stop sends the protocol's interrupt before it closes stdin, so the turn ends with a
> result rather than running on for the 30 s. (6) Once the main process exits, its job is killed
> before the rest of the output is read, so a leftover process neither lingers nor holds the pipes.
> (7) A `user` line the crate cannot read is no event (the prompt's echo); the stub's echo carries the
> session as the CLI's does. (8) `stub-claude` is a binary target of the core crate under
> `tests/stub_claude/`, out of Cargo's test discovery; a release build makes it too, unused. (9) `crw
> doctor` prints the preflight (scope 8), so it can be checked on the real CLI without usage. (10) A
> refusal carries the CLI's reason (`Denied::reason`, added after the probe). (11) A neighbour that
> holds the project's folder or sits on a network share refuses the launch; a missing one is left out
> and said. (12) `settings.claude_cli` must be a full path. (13) The effort and the permission mode are
> checked against the help's lists when it gives them.
>
> No owner tasks; no messages for other projects. **Next: EP-10** (the prompt generator and the
> launch dialog).
