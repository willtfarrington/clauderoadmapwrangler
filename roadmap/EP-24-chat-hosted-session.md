---
ep: EP-24
title: "Chat, the hosted session: a read-only portfolio session per conversation, the digest system prompt, conversations kept in the ledger and resumed, a plain tile with streaming replies and an input, Opus default"
size: M (L → M, cut beforehand at EP-23)
model: fable (claude-fable-5-1)
effort: xhigh
status: done
---

# EP-24 — Chat, the hosted session

**Session start (2026-11-11).** A headless run chained by the autopilot after EP-39 (ledger row
`1ab31a65`, `crw sessions clauderoadmapwrangler`: headless, fable at xhigh, M's cap of 90 min),
the prompt the opener; the window runs EP-39's installed build. Read first: `roadmap/README.md`
(EP-24 is the first unchecked row of P4, its brief note and EP-24a's and EP-24b's, which say what
this brief leaves to them), `DECISIONS.md` (D-27 with its three addenda, D-16, D-18, D-19, D-26),
EP-39's completion note, `roadmap/owner-tasks.md` (no open task) and `roadmap/inbox.md` (no open
message). The field check: `crash.log` is EP-15d's old report, seen already (`crash.seen` beside
it); `crw notifications` shows EP-39's finish and this chain's start; `crw autopilot` the chain
log with this run held and started at 18:46; nothing odd. Two facts checked by hand before the
brief, spending no usage: `claude --help` (2.1.316) lists `--restricted` ("confines the file tools
to the working directories (--add-dir included)", removes Bash and WebFetch unless `--tools` names
them, ignores the user, project and local settings files), `--strict-mcp-config`, `--tools`,
`--append-system-prompt <prompt>` (the `-file` form is named only inside `--bare`'s description,
so it is no option the help lists) and `--permission-mode`'s choices `acceptEdits, auto,
bypassPermissions, manual, dontAsk, plan`; and Claude Code's permission-modes page (read
2026-11-11) says `dontAsk` "denies the built-in AskUserQuestion tool even if your allow rules match
it", while a `-p` run with no mode set starts in Manual mode since 2.1.302, its calls that need
approval reaching the host through `--permission-prompt-tool`.

## Context

D-27 plans one hosted Claude Code session per conversation, read-only over the portfolio, with a
generated digest as its appended system prompt, resumable and kept in the ledger, Opus 5.5 at
medium. EP-23 cut the L into three M briefs (D-27 addendum of 2026-11-11): this one builds the
session and a plain tile; EP-24a draws the replies as Markdown, lists the conversations and adds
the picker and the questions answered in the tile; EP-24b adds the delegated actions, the `crw`
commands the session may run and the chat's own `@<project>` mentions. The brief note's first
question at pickup is what the session may read: its planned cwd is the data root, under which
the data repositories lie beside the projects, and the registry has no mark for one.

What the code gives: the runner (`core::session`) starts a `-p` stream-json run from a `Launch`
and records it in the ledger; the window's one start funnel (`Sessions::begin`) writes the
plugin's copy and passes `--plugin-dir`; the transcript (`app::transcript`) appends partial text
while it streams; the ledger's `session` table takes a mode of `visual`, `headless` or `manual`
and a project; the Decisions tile answers a window's session's questions; the usage split
attributes each poll's rise to the window's running sessions by their session id.

## In scope

1. **`core::chat`.** The conversation's launch: cwd the tool's own `chat` folder under the local
   folder (`%LOCALAPPDATA%\clauderoadmapwrangler\chat`, under `CRW_HOME` in a trial), every
   registered project that is not marked a data repository joined read-only with `--add-dir`,
   and the plugin copy's folder too, so the house-style reference reads; `--restricted` and
   `--strict-mcp-config`, both required (the boundary: the file tools confined to those folders,
   no shell, no web, none of the owner's MCP servers, no user or project settings);
   `--tools Read,Glob,Grep,Skill,AskUserQuestion`, required; the permission mode `manual` when
   the help lists it (else no mode flag, said), with `--permission-prompts host` and
   `--permission-prompt-tool stdio` as every run, so that questions reach the window and the
   window answers every permission request itself with a refusal (the chat reads, nothing
   else); the digest passed with `--append-system-prompt-file` when the help lists it, else
   `--append-system-prompt <text>`, else dropped and said; `--name "chat: <title>"`; no
   `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD` (eleven `CLAUDE.md` files would load);
   `--resume` for the conversation's later turns, each a run of the same session id (EP-13's
   way). Opus 5.5 at medium, constants until EP-24a's picker. The ledger's rows for a
   conversation's runs carry the project `chat` and the mode `headless` (a `-p` run is one), so
   the usage split, `crw sessions` and the All tab need no new mode.
2. **The digest.** `chat::Digest` built by the window from its own readings (rule: never a
   `crw status` over the registry, EP-7's note): the date, each project's name, state, folder,
   roadmap counts, next EP with its title and size, open owner tasks and inbox messages (urgent
   counted), the sessions at work there (the window's and those Claude Code reports), the last
   commit's date, the hosting class and this week's usage share; the account's weekly and Fable
   figures; the projects the session cannot read and why; the house-style reference's path in
   the plugin copy and the `/roadmap-wrangler:status` skill. `chat::digest_text` sets it as the
   appended system prompt with a framing paragraph: a read-only chat over the owner's portfolio,
   the files the truth, writes delegated to the window's own surfaces (named), plain words for
   what a brief plans (EP-15a's answer). Capped at 12,000 characters at a project's boundary.
   Written to `chat\digest-<session id>.md` for the record at each turn, fresh at each turn
   (`--system-prompt-snapshot off` when the help lists it, so a resumed turn gets the current
   digest; else said).
3. **The registry mark.** `data = true` on a project marks a data repository: the chat never
   adds its folder and the digest says so. `crw set <project> data yes|no`; written only as
   `true`, so a registry without the mark reads in older builds (as `revert` is).
4. **The ledger.** Schema 6 adds the `conversation` table (session id, title, model, effort,
   started, last activity), written by the runner beside the session row when the launch names a
   conversation; `Ledger::conversations` joins each with its runs' count and cost sum; a ledger
   read at an older schema has none. `crw chats` prints them (a safe read). Older builds refuse
   the migrated ledger, as at every schema step.
5. **The Chat tile.** `app::chat`: one conversation at a time; the header says the model and
   effort, what the session reads and leaves out; the exchange drawn from the run's transcript as
   plain text (the owner's messages, the replies as they stream, the tool calls as small lines,
   refusals, the result's errors); a multiline input with Send (Ctrl+Enter), Stop while a reply
   streams, New conversation. Send on an ended conversation resumes it with the message. A chat
   run goes through `Sessions::begin`; the Sessions pane's tabs and its All tab leave chat runs
   out (the ledger's rows show them), the Decisions tile takes a chat's questions as any run's
   (EP-24a moves them into the tile), a chat's end raises no popup, and a chat that works refuses
   a restart and is listed by Quit's question as any run. A screenshot flag `chat` cans a
   conversation; `tile=chat` brings the pane forward.
6. **Decisions.** A D-27 addendum (the cwd, the boundary, `manual` in place of `dontAsk`, the
   mark, the ledger's form); D-26 and D-18 addenda for the schema step and the mode. The README's
   status paragraph gains two generic sentences.

## Out of scope

Markdown in the replies, the conversation list and resuming an older conversation after a
restart, the model and effort picker, questions answered in the tile (EP-24a); action chips, the
`crw` commands the session may run, Bash of any kind, the chat's `@<project>` mentions (EP-24b);
a `data` box in the window's Adopt (parked in EP-24b's notes); a chat from the phone (P5).

## Verification / acceptance

- `cargo test --workspace`, `cargo test --features screenshot`, both clippy runs with
  `-D warnings`, `cargo fmt --check`; rule 3's word check on the staged diff prints nothing.
- `core`: the chat's command line (the flags, their order, the refusals when `--restricted`,
  `--strict-mcp-config` or `--tools` is missing, the digest inline or by file, the env var left
  unset, the add-dirs with their Edit rules, the resume); `digest_text` on synthetic projects and
  its cap; the schema 6 migration on a schema 5 ledger with rows, the conversation rows and their
  sums, a read-only older schema giving none; the registry's `data` mark written, removed and
  read.
- `app`: a chat live refuses a permission request itself and raises no end news; the tab strip
  and the All tab leave it out; the tile draws a canned conversation through `testing::Screen`
  with Send disabled while the reply streams; the digest built from a scratch board.
- A screenshot run with `chat` in both themes on the fake portfolio (`target\tmp\ep24\`, never
  git). The pane shows no roadmap content of its own, so no run over the copy of the real
  roadmaps is made; the digest over the real portfolio is read from the file the first real
  conversation writes (OT-26).
- A trial of `crw chats` and `crw set <project> data yes` on a scratch `CRW_HOME` registry.

**Residual risk.** No real chat session runs in this brief (D-29: tests never run the CLI, and a
probe spends usage): whether `--tools` takes `Skill` and `AskUserQuestion` by those names, whether
`--restricted` confines Grep as the help says, and whether the questions of a `manual` run reach
the host are read from the help and the permission-modes page, not seen. The first real
conversation is the owner's (OT-26): the tile shows the Init's tool list and every refusal, and
the owner asks the session what it can read. The brief's plan also rests on `claude --help` of
2.1.316; a CLI without the three boundary flags refuses the chat and says why.

> **Completion note (2026-11-11; claude-fable-5-1/xhigh; about 1 h 05 min of wall time, a
> headless run under the autopilot, three questions answered in the Decisions tile).** Ledger
> row `1ab31a65`, chained after EP-39 at 18:46 through the window's restart into EP-39's build
> (an M's cap of 90 min); the prompt was the opener.
>
> **Built.**
>
> 1. **`core::chat`** (D-27 addendum): the constants (`PROJECT` `chat`, Opus 5.5 at medium,
>    the tools `Read,Glob,Grep,Skill,AskUserQuestion`, the mode `manual`, the cap of 12,000
>    characters), `title_of` (the first line of the first prompt, cut to 60 characters),
>    `Digest`/`ProjectDigest`/`digest_text` (the framing, one line per project, the rest's names
>    when the room runs out, what is left out and why, the account's figures, the reference's
>    path and the status skill), `folders` (each project not marked `data = true` whose folder
>    is there, then the plugin copy's), `Turn` (a first turn under a chosen id, or a resume) and
>    `launch`. The `Launch` gained `conversation`, `restricted`, `tools`, `appended`
>    (`Appended { file, text }`), `neighbour_claude_md`, `fresh_system_prompt` and `session_id`;
>    `command_line` passes `--restricted` and `--strict-mcp-config` (required), `--tools`
>    (required), the mode `manual` when listed (else no mode flag, said), the digest by
>    `--append-system-prompt-file` when listed and the file is there, else inline by
>    `--append-system-prompt`, else dropped and said, and `--system-prompt-snapshot off` when
>    listed; the neighbours' `CLAUDE.md` variable is set only when the launch wants it; the
>    session is named `chat: <title>`. The runner takes a chosen id for a new session and writes
>    the conversation's row beside the first run's.
> 2. **The ledger** (D-26 addendum): schema 6, the `conversation` table, `touch_conversation`
>    (new, or its last activity moved on) and `conversations` (the latest first, each with its
>    runs counted and their costs summed from the session rows of its id; none at an older
>    schema). The registry's `data` mark (`set_data`, written only as true). `crw chats` and
>    `crw set <project> data yes|no`; a trial on a scratch `CRW_HOME` registry wrote and removed
>    the mark, refused `maybe`, and `crw chats` said the ledger was not there yet.
> 3. **The window.** `app::chat`: the tile (the title, the state chip with the session's Init on
>    hover, the reads line with the names and the digest's file on hover, New conversation and
>    Stop, the exchange drawn from the run's transcript as plain text with the tool calls in
>    small lines and the refusals in red, a multiline input with Send and Ctrl+Enter, the short
>    session id under it), `chat_digest` from the portfolio's rows, the window's runs, the
>    sessions Claude Code reports, the week's shares and the meter's figures, `chat_send` (the
>    digest written to `chat\digest-<id>.md`, the launch through `Sessions::chat`, a later turn
>    resuming the conversation's session with a fresh digest), `chat_started`, `chat_stop`,
>    `chat_new`. In `sessions`, a chat run refuses each permission request itself at once with
>    `chat::REFUSAL` and says so in its transcript, raises no end toast, keeps its questions for
>    the Decisions tile, is left out of the tab strip, the All tab's list and the Sessions tab's
>    count, and is named `chat: <title>` where Quit's question and a restart's refusals list it.
>    The Chat pane replaced its placeholder; the screenshot flag `chat` cans a conversation of
>    two turns, the second's reply streaming.
> 4. **Docs.** D-27, D-26 and D-18 addenda; the README's status paragraph; EP-24b's brief note
>    (the Adopt chip to offer the mark, the shell's commands); OT-26, the first real
>    conversation.
>
> **The pictures** (`target\tmp\ep24\`, never git): the fake portfolio's Chat pane at 1600
> points in the light theme and at 1200 in the dark, the second showing the narrow pane's
> lesson taken (EP-13's): the chip on its own row, the buttons wrapped, the session id short, so
> that the reply wraps at the pane's edge. No run over the copy of the real roadmaps: the pane
> draws no roadmap content of its own; the first real digest is OT-26's to read.
>
> **The review (rule 6).** A subagent given the diff alone, launched once the code compiled,
> reported after seventeen minutes: one confirmed bug, seven plausible points and six nits.
> Fixed: (1, confirmed) the typed message was cleared before the start's outcome was known and
> lost on a refusal: the id, the title and the message now stay pending (`chat::Pending`) until
> the start comes back, and a refusal keeps the message typed; (2) a start thread that could
> not be spawned left the tile "starting" for good: the flag now follows
> `Sessions::is_starting()` after the hand-over; (3) the autopilot's gate read the registry
> after every chat turn: `weigh_run` returns for a chat live first; (4) a registered project
> named `chat` would mix with the conversations' rows: Send refuses while one exists, and
> `Sessions::summary` and `at_work` leave chat lives out of every project's row; (5) a deferred
> chat question resumed from the Decisions tile took the generic start arm, which closed an
> open launch dialog and brought the Sessions pane forward: `Sessions::last_chat` marks the
> start that last came back, and the Chat tile takes it; (6) one registered project off a
> drive letter would have refused every turn (no `Edit` rule for it): `chat::folders` leaves
> such a folder out and says why; (7) a typed chat message now refuses a restart, as a typed
> disposition does; (8) the conversation's row is written before the run's, so a failure leaves
> no run open with no end to come; (9) the digest's tail (what is left out, the usage) now counts
> against the cap, the project lines stop short by the room the rest's names take, and an empty
> cut reads "…"; (11) the chip and the footer say "starting" only for this conversation's turn,
> another project's start holding Send back alone; (12) the plugin's copy is written before the
> first digest, so a fresh profile's first conversation names the reference; (13) an unused
> parameter; (14) New conversation clears the header's figures. Logged, not changed: (12b) the
> tile's digest test reads whether the real plugin copy's reference exists (a read of a path,
> no write); the tail of the digest is bounded by the cap only together with the lines, a
> portfolio whose left-out names alone exceed it being no case of the owner's.
>
> **Verification.** `cargo test --workspace`: 365 passed in `core`, 153 in `app`, 13 in the
> other suites; with the feature 161 in `app`; both clippy runs and `cargo fmt --check` clean
> (the digest cap's last fix, in `core` alone, was followed by the core's tests and clippy
> again); rule 3's word check on the staged diff printed nothing. The release build and
> the install (`scripts\install-shortcut.ps1`) ran after the record commit, the opener's last
> step; the restart is refused while this run works and taken at the gate's moment, or from
> File > Install and restart, the tray's, or the launch dialog's.
>
> **Owner decisions** (asked once the build stood, so that the answers could be applied in the
> same run; all three took the recommended option, answered within a few minutes while the
> test gate and the review ran).
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The session's boundary: the tool's own folder as cwd with every readable project joined by `--add-dir` under `--restricted`, or the data root as cwd with a `Read` deny rule per data repository | The tool folder and the allow-list (recommended) | As built; D-27 addendum |
> | 2 | The registry's mark: `data = true` as a deny-list, or `chat = true` as an allow-list | `data = true` (recommended) | As built; OT-26 asks for the marks before the first conversation; EP-24b's Adopt chip to offer it |
> | 3 | Whether the digest names the projects left out, with why | Name them (recommended) | As built |
> | 4 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The chat's runs as session rows** under the project `chat` and the mode `headless`, so
>    the usage split, `crw sessions` and the All tab need no new mode; a mode of its own would
>    have rebuilt the session table for its check.
> 2. **The questions asked at the end**, not at pickup: a question blocks the run, and the
>    recommended answers were the ones built; the alternatives were costed in the question.
> 3. **`manual` by name**, dropped and said by a help without it, rather than no mode flag at
>    all, so that the run's mode is the one the addendum names where the CLI lists it.
> 4. **The digest's file is kept** after the run, one per conversation, overwritten at each
>    turn: a few kilobytes, and the record OT-26 reads.
> 5. **A deferred chat question** (the defer route, or a time limit, which a chat has none of)
>    resumes with the launch kept in the live, so with the digest of the turn that asked; a
>    fresh digest comes with the owner's next message.
> 6. **The conversation does not survive a restart** in the window: its live is dropped like
>    any, and the ledger's row waits for EP-24a's list to resume it; a chat that works refuses
>    the restart as any run.
> 7. **`Live::canned` compiles for tests too**, so the tile's test draws a running conversation.
> 8. **The scratchpad** is `target\tmp\ep24\` (ignored): the pictures, the logs, the trial's
>    registry and the diff the review read.
>
> **Owner task:** OT-26, the first real conversation, with the data repositories marked first.
> No message for another project. **Next: EP-24a** (the replies as Markdown, the conversation
> list, the picker, the questions in the tile), chained by the autopilot when its gate allows,
> else from the window.
