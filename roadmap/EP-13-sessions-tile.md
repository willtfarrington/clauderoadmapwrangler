---
ep: 13
title: Sessions tile
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-13 — Sessions tile

**Size:** M · **Depends on:** EP-12 · **Blocks:** EP-14 (the decisions tile answers the tile's
sessions; its fallback resumes a deferred run through this brief's Resume), EP-16 (the first
self-launched session verified in the ledger), EP-19 (the Commit button resumes a finished run),
EP-20 (a session's completion seen in the ledger)

## Context

Until now the window held its headless sessions in plain lines (D-16's EP-10 addendum) and listed
the sessions started elsewhere read-only (EP-12). This brief makes the Sessions pane a tile: one
tab per session the window runs, with its transcript as it streams (the assistant's text, each
tool call in one line with its result collapsed under it), Stop and Kill, Resume by the session's
id with a follow-up message (D-16's EP-9 addendum left it here), and a list of every session with
filters, the ledger's recent rows among them. A headless row gains its result text, and a visual or
manual row the status line's cost and context when the session sent them (D-26).

The owner's notes after OT-6 (2026-11-02, carried in the request at pickup) bring five findings
from the real ledger and window, which this brief settles:

1. **Rows that are not working sessions.** Opening a project's Claude Code tab produced three
   manual rows within seven seconds: a helper the extension starts (two seconds, no transcript), an
   older session the window restored (started and ended within ten seconds), and the owner's test;
   another project shows three rows in the same second. Each also sat in the Sessions pane for ten
   minutes. What earns a row is decided here.
2. **Visual rows that never close.** Two visual launches were recorded before the hooks were
   installed; no session will claim them, so they stay open with no session id. A rule for a
   visual row nothing claims within its 30 minutes is needed.
3. **A compaction's row.** A session compacted after the install got a manual row of its own: the
   start hook fires at a compaction too.
4. **The Detail pane hides a running session** when the project has no roadmap, every row is
   checked or no phase table was read: the "Session in progress" line is never reached, while the
   list's Session column shows the session.
5. **Busy looks like idle**: the status chip is grey for both.

Also from the notes: the end hook works from VS Code (every manual row has an end time); busy
comes from the poll (every 30 s while the window is in the background), and the prompt hook
planned for EP-20 could report it at once, which is the owner's call.

**Session start (2026-11-02, 21:12).** Started from `DATA\clauderoadmapwrangler.code-workspace`
in the VS Code extension. OT-6 is ticked by the owner in the working tree and is committed with
this brief's records, as EP-12 did with OT-5; no inbox message is open. The owner's environment
repository finished its EP-2 at pickup: the stopgap inbox script is deleted, so the copy of the
user settings crw kept before its install (EP-12) would now restore a hook that runs a missing
script. The model is Opus 5.5 (`model = opus` in the user settings); the session's environment
gives the effort as max.

**Verified at pickup.**

- The adapter (`session::protocol`) reads no user line, so the tool results a run prints
  (`ClaudeOutput::User` holding `ContentBlock::ToolResult { tool_use_id, content, is_error }`, the
  content a string or a list of blocks) never reach the window; a subagent's lines carry its
  parent's `parent_tool_use_id`.
- CLI 2.1.316's help: `-r, --resume [value]` resumes a conversation by session id or name;
  `--fork-session` creates "a new session ID instead of reusing the original", so a resume keeps
  the session's id; `--session-id` names a new conversation.
- The ledger is at schema 2: `finish_session` keeps the end, the outcome, the cost and the
  questions, no result text; `end_seen` closes a visual or manual row with no figures; nothing
  closes a visual row that no session claims.
- The state model (`crw_core::sessions`) opens a row at every interactive `SessionStart` of a
  registered project whatever its source (`startup`, `resume`, `clear`, `compact`, `fork`), and an
  ended session stays ten minutes. Busy comes from the poll alone, which asks at once when the
  window gains the focus (the pace drops to 5 s and the last listing is older).
- `detail::next_brief` returns before the in-progress line for a project without a roadmap or a
  next row; `sessions::seen_card` colours busy and idle with the weak text colour.

## In scope

1. **Tool results** (`session::protocol`): `Event::ToolResult` for the main agent's results: the
   tool use it answers, whether it failed, its line count and its first lines, capped; held in the
   window's memory only, never written (D-3, D-26).
2. **Resume** (`session::Launch::resume`): `--resume <id>` in place of `--session-id`, the launch
   refused when the help lacks the flag; the run keeps the session's id and is a ledger row of its
   own; an ended headless session of the window's resumes in its own tab with a message the owner
   types.
3. **The ledger** (schema 3): the columns `result` (a run's final text, capped) and
   `context_percent`; the runner records the result text; an end hook records the session's last
   status-line cost and context; a visual row that no session has taken up 30 minutes after its
   launch lapses, closed at that moment with no session and no outcome; `recent_sessions`.
4. **What earns a row** (`crw_core::sessions`): a session seen through the hooks gets its row at
   its first sign of work (a finished turn, a wait for the owner, or the poll's busy or waiting),
   and only when the hooks saw it start; a compaction's start is no start. A visual launch is taken
   up only by a session that starts as a new conversation (`startup`) within its 30 minutes and
   works before it lapses. A session that has not worked is listed as new, is no session in
   progress, and leaves the window at once when it ends unused. A `UserPromptSubmit` hook reads as
   busy and work when one arrives (EP-20 installs the entry, or this brief on the owner's yes).
5. **The tile** (`app::sessions`): a tab strip, All and then one tab per session of the window's.
   A session's tab: its header with Stop, Kill and Dismiss; the transcript as it streams (text,
   the partial text appended, a tool call in one line with its result collapsed under it, the
   refusals, the requests that wait for EP-14, the notices, the result); Resume with a message once
   it has ended. All: the window's sessions, those started elsewhere and the ledger's recent rows,
   with filters by project, mode and running only, saved with the view. Busy has a colour of its
   own.
6. **The Detail pane**: the in-progress line for every project, whatever its roadmap holds.
7. **`crw sessions [<project>] [--all]`**: the ledger's rows, the newest first, read only (a visual
   row past its 30 minutes said lapsed before a window closes it).
8. **Screenshot runs**: `sessions` also adds a canned headless session of the window's with a
   transcript and canned ledger rows; `transcript` brings that session's tab forward.
9. **Owner decisions** through `AskUserQuestion` (rule 7): the row rule, the lapse, the prompt
   hook, the addenda, the notes, the commit.

## Out of scope

- Answering requests and Focus (EP-14); toasts (EP-15); usage samples (EP-17); the `@<project>`
  prompt hook (EP-20).
- Transcripts of the sessions started elsewhere: the tool reads no transcript file (D-26); their
  cards show what the hooks, the poll and the status line say.
- Resuming a visual or manual session through the extension's `session=` link, and a headless one
  after the window restarted (its launch's choices live in the window): v2 notes, or EP-14 for a
  deferred run.
- The rows the ledger holds from before this brief (rule 13): changed only on the owner's ask.
- Tests that run the real CLI (D-29): the stub plays a resume and a tool result.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`, `cargo
  build --release` with the app closed, then the install script.
- Unit tests: the adapter's tool results (a string, blocks, an error, a subagent's left out, the
  cap); a resume's command line (`--resume <id>` and no `--session-id`, refused without the flag);
  the ledger's migration from schema 2 with its rows kept, the result text, an end's figures, the
  lapse and the recent rows; the state model's row rule through the owner's cases (a helper that
  starts and ends, a restored tab, a compaction, a first turn, the poll's busy, a wait, the claim
  by a new conversation only, an unused session gone at once); the tile's transcript from a run's
  events; `crw sessions` on a scratch ledger.
- The runner against the stub: a tool result read; a resume keeps the session's id and writes a
  second row.
- Screenshot runs: the All tab and a transcript, light at 1600 × 1000 and dark at a narrow window;
  the Detail pane of a project with every row checked and a session running; each picture looked
  at.
- `crw sessions` on the owner's ledger, read only.
- The commit's added lines pass rule 3's word check; the owner has reviewed every decision.

> **Completion note (2026-11-02; opus/max; ≈ 1 h, nearly all active).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`, from 21:12 to the commits: the reading at pickup with
> the owner's notes after OT-6, this brief, the code and its tests, the pictures, the owner's two
> rounds (each answered within minutes), the records and the commits. Against M ≈ 1 h. The model is
> Opus 5.5 (`model = opus` in the user settings); the session's environment gives the effort as max.
>
> **What was built.** `session::protocol`: `Event::ToolResult` from the main agent's user lines
> (the tool use it answers, whether it failed, its line count, its first 12 lines cut to 1,500
> characters). `session::Launch::resume`: `--resume <id>` in place of `--session-id`, refused when
> the help lacks it, the run keeping the session's id; the runner records a run's result text.
> `ledger`: schema 3 (`result`, kept to 8,000 characters, and `context_percent`), `Finished`,
> `end_seen` with the status line's last cost and context, `lapse_visual`, `recent_sessions`,
> `open_read_only` (never creates or migrates; an older schema read with its columns),
> `SessionRow::{state, took, lapsed}` with `RowState` and `length`, and a migration step that takes
> the write lock before it reads the version. `sessions`: `Seen::{opened, worked, is_new}`, a row
> earned at a session's first sign of work when the hooks saw it start (`Change::Started` with
> `claims`, a new conversation's only), a compaction's start no start, `UserPromptSubmit` read as
> busy and work, the poll's busy or waiting as work (`poll` returns the rows it starts), a session
> ended unused gone at once, `Change::Ended` with the figures. In the window: `transcript` (the
> entries, the 2,000 kept, the rendering), the Sessions tile (`sessions`: a tab strip, a session's
> tab with Stop, Kill, Dismiss and Resume with a message continuing the same tab, the All tab over
> the window's sessions, those started elsewhere and the ledger's latest 200 rows with filters saved
> in the view), `Action::ResumeSession` and `Board::resume_session`, the watch recording the poll's
> rows and lapsing unclaimed launches once a minute, unused sessions left out of the in-progress
> reasons, the Detail pane's in-progress line for every project, busy in blue (`style::blue`), and a
> screenshot run's canned transcript, rows, a new tab of beacon's, orchard's busy session and the
> `transcript` flag. `crw sessions [<project>] [--all]`, read only.
>
> **Verification.** `cargo test` 271 passed (36 + 1 + 199 + 2 + 7 + 6 + 11 + 9), 273 with
> `--features screenshot`; both clippy runs and `cargo fmt --check` clean; `cargo build --release`
> with the app closed, then the install script; `crw signals` says every entry runs the new exe.
> Thirteen new tests: the adapter's tool results (a string, blocks, an error, the caps, a
> subagent's and the prompt's echo left out); a resume's command line; the ledger read at schema 2
> before and after its migration, the lapse with the rows' states, four openers of one ledger at
> once, the result text and its cap, an end's figures; the owner's cases (a helper and a restored tab
> that start and end with no row and leave at once, the poll's busy starting a row from the
> session's start, a compaction, a resumed tab never taking up a launch); the runner reading a tool
> result and resuming under the same id with a second row; the transcript from a run's events and
> its cap; the resume's gate, the filter and the run lengths; `crw sessions` on a scratch ledger,
> the file left byte for byte. Screenshot runs, each looked at: the All tab light at 1600 × 1000
> (the first showed "table schema_version already exists": the loader and the lapse migrating a new
> ledger at once, a race older than this brief, fixed by the write lock; and the ledger card's chip
> squeezed into a column of letters, then given a row of its own), the transcript light (a caret
> the font lacks, now an ellipsis; "1 tool uses", now singular), the transcript dark at 1100 × 760
> (three times: the text ran past the pane's edge, widened by a title-and-chip row and by the folded
> prompt's header, whose text never wraps; rows of their own and a short header put it inside), the
> All tab dark at 1100 × 760, orchard's Detail pane (every row checked, the busy session said), and
> the All tab at 2300 × 1500 (a lapsed launch showed 30 min as if a run; it has no length now).
> `crw sessions --all` on the owner's ledger, read only: ten rows at schema 2 (the owner's test and
> its two companions at 19:47, another project's three at 19:00, the compaction's row still open, the
> two launches of 17:25 read as lapsed), the file's size and time unchanged. Rule 3's word list found
> nothing in the added lines or the new files.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-02),** two rounds, every answer the recommended
> option:
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | What earns a ledger row | First sign of work | As built; D-26 addendum; the rows before EP-13 stay (rule 13) |
> | 2 | A visual launch no session takes up | Lapse after 30 min | As built; the two launches from before the hooks close at the window's next start (OT-7) |
> | 3 | The `UserPromptSubmit` hook now | Leave it for EP-20 | No change to the user settings; D-20 addendum and EP-20's note |
> | 4 | A tab open with no prompt yet | In progress only once it works | As built; D-9 addendum |
> | 5 | The five addenda | Accept as written | D-9, D-16, D-20, D-25, D-26 |
> | 6 | README, roadmap notes, risk 4, v2 notes, OT-7 | Keep as written | As drafted; OT-7 gated before EP-14 |
> | 7 | Commit and push | Commit and push | Two commits in the house pattern, pushed; the first carries OT-6's tick |
>
> **Routine choices (the session's call, logged).** (1) Tool results read from the main agent's
> user lines, their first lines only, held in memory. (2) A resume continues its session's tab, the
> message shown as a folded entry after a separator, the runs counted. (3) The transcript keeps 2,000
> entries; notices live in it rather than in the header. (4) The All tab reads the ledger without a
> write every 30 s while the pane shows and at once after a change, 200 rows, leaving out the rows
> whose session shows above. (5) The lapse runs in the window once a minute and at its first frame;
> `crw sessions` says lapsed without writing. (6) Schema 3 by `ADD COLUMN`, no table rebuilt; a
> lapsed launch keeps D-26's four outcomes by having none. (7) The migration's race fixed here, the
> lapse having exposed it. (8) The poll's busy or waiting is work; an idle prompt is not. (9) A
> resumed or forked conversation never takes up a launch; a start that names no source may. (10) Busy
> in a fixed blue for each theme. (11) In a narrow pane each chip has a row of its own, and a folding
> header's text stays short ("Prompt", "Resumed", the time in its body). (12) The canned busy session
> moved from quarry to orchard, whose rows are all checked, so a picture shows the Detail pane's
> fix. (13) `crw sessions` prints the latest 20 unless `--all`, a project's by its name, case aside.
>
> **Handed on.** Nothing: the user settings are unchanged, and the environment's memory of crw's
> hooks still holds.
>
> **Next: EP-14** (Decisions tile), after OT-7.
