---
ep: 14
title: Decisions tile
size: L
model: claude-opus-5-5
effort: max
status: done
---

# EP-14 — Decisions tile

**Size:** L · **Depends on:** EP-13 · **Blocks:** EP-16 (the default launch mode is revisited once a
headless session can answer the questions a brief ends with), EP-19 (the autopilot's runs ask through
this tile), EP-24 (the chat's questions), EP-30 (the phone's decisions page answers the same requests)

## Context

Until now a headless session that asked the host waited in the Decisions pane, listed read-only, until
the owner stopped it (D-16's EP-10 addendum). This brief answers it there (D-19): a permission request
with Allow or Deny, an `AskUserQuestion` with its options, one or several, or the owner's own words,
the session going on at once. A project may give its requests a time limit, after which the tool
defers the run instead of letting it wait. When a CLI release breaks the control protocol, the
documented defer-and-resume route takes over: a `PreToolUse` hook ends the run at the question, and
the answer resumes it. Visual sessions that wait show "waiting for you in VS Code" with a Focus button
that brings their window forward; the tool never answers for them. The ledger keeps how long the
owner took to answer.

**Session start (2026-11-02, 22:20).** Started from `DATA\clauderoadmapwrangler.code-workspace`
in the VS Code extension. OT-7 is ticked by the owner in the working tree and is committed with this
brief's records, as EP-13 did with OT-6; no inbox message is open. The model is Opus 5.5; the
session's environment gives the effort as max.

**Verified at pickup.**

- `crw doctor`: claude 2.1.316 at the launcher, logged in, headless ready.
- Claude Code's hooks page (read 2026-11-02): `permissionDecision: "defer"` is honoured only with
  `-p`; the turn ends with a result carrying `stop_reason: "tool_deferred"` and `deferred_tool_use`
  (`id`, `name`, `input`); the host resumes with `claude -p --resume <id>` with the same permission
  host and **no prompt**, and the same call fires `PreToolUse` again (a prompt sent with or after
  the resume is answered without the tool's result, and the deferred call is dropped); for
  `AskUserQuestion` the hook must return `allow` with `updatedInput` holding the original
  `questions` and an `answers` map keyed by each question's text ("allow" alone is not enough); a
  resume without a prompt may defer again; `defer` is ignored with a warning when the turn makes
  several tool calls at once (the call then takes the normal permission flow); there is no timeout;
  `-p --resume` does not restore the permission mode, so it is passed again. Hooks have an exec form
  (`command` and `args`, no shell), which EP-12's entries use.
- The changelog: "Added `"defer"` permission decision to `PreToolUse` hooks — headless sessions can
  pause at a tool call and resume with `-p --resume` to have the hook re-evaluate" and "PreToolUse
  hooks can now satisfy `AskUserQuestion` by returning `updatedInput` alongside
  `permissionDecision: "allow"`".
- `ask_user_question_unanswered_v1`, among 2.1.316's init capabilities, is described neither in the
  changelog nor on the hooks, headless and Agent SDK pages; the SDK's user-input page says a
  question left out of `answers` is unanswered. The tool leaves out a question the owner did not
  answer, as the protocol crate does, and relies on nothing more.
- The adapter answers a request with Allow, Deny or the answers (the crate keys them by question
  text and keeps the questions), but `Request` carries no input to show a permission by, and
  `Finish::deferred` keeps the deferred tool's name only; nothing measures how long a request
  waited, and the ledger's `answer_latency_s` (there since V1) is never written.
- The window lists `Live::waiting` read-only (`sessions::request_card`); a seen session's card
  (`sessions::seen_card`) has no Focus; `visual::open` starts `Code.exe` on the project's own
  workspace file one child at a time with the session markers and VS Code's variables withheld.

## In scope

1. **Answers** (D-19): the Decisions tile answers each request a window's session waits on. A
   permission shows its tool, the call in one line, why it asks and its input, with Allow and Deny
   (a reason optional). An `AskUserQuestion` shows each question with its header, its options (one
   or several, each with its description) and an Other field for the owner's own words, with Send
   answers, or Decline with a message. The answer goes at once through the session's handle; the
   transcript shows it under the request.
2. **A time limit per project**: `answer_timeout` in the project's registry entry (minutes; none by
   default, D-19), set with `crw set <project> answer_timeout <minutes|none>`. The runner answers a
   request that has waited that long with a denial that ends the turn and says the run is deferred,
   and the run's outcome is `deferred`; the card counts the time left.
3. **The defer route** (D-19's fallback): chosen by `settings.decisions = "defer"` (`crw setting
   decisions defer|protocol`), or by the window for the later runs of a CLI whose handshake it saw
   refused (a running process cannot take new hooks). A headless run then gets, through
   `--settings`, a `PreToolUse` hook on `AskUserQuestion` that runs `crw.exe hook --defer`; the run
   ends with `tool_deferred`. Answering resumes the session with `--resume <id>`, no prompt, and a
   one-shot `crw.exe hook --answer <file>` that allows that tool use with the answers (and defers a
   later question, so the round trip repeats).
4. **Deferred runs in the tile**: a run deferred by the time limit or by the hook keeps its open
   question or permission, shown with the same form and an Answer and resume button: by the hook's
   route without a prompt, else with the answers as the resume's message.
5. **Sessions that wait elsewhere**: the tile lists the sessions Claude Code reports waiting:
   "waiting for you in VS Code" with Focus (the project's own workspace file, else its folder,
   opened with `Code.exe`, which brings its window forward), or where else it waits. The tool never
   answers for them. The Sessions tile's card of such a session gets Focus too.
6. **Refusals**: the tool uses each session had refused without asking (a neighbour's deny rule, D-18
   addendum) listed with the session, read-only.
7. **The ledger**: `answer_latency_s` holds the mean seconds from a request to the owner's answer in
   the run (a deferred question's answer counts on the run that carries it); `questions` counts a
   deferred question too; the All tab and `crw sessions` show both.
8. **The launch dialog** says, for a headless launch, how its questions are answered: here at once,
   or the run deferred after the time limit, or the defer route.
9. **Screenshot runs**: `decisions` adds canned waiting requests (a pair of questions and a
   permission), a deferred run, a visual session waiting and refusals, with the Decisions pane in
   front.
10. **Owner decisions** through `AskUserQuestion` (rule 7): a supervised real probe, the default
    launch mode, the addenda, the notes, the owner task, the commit.

## Out of scope

- Toasts and the tray's badge (EP-15); the phone's decisions page (EP-30).
- Answering a visual or manual session (D-19: never), or reading its transcript (D-26).
- "Allow always" (the CLI's permission suggestions written as rules) and a free reply in place of
  the answers (`response`): v2 notes.
- Changing a running process's hooks when its handshake is refused: the switch serves the runs after.
- A waiting or deferred question kept across a restart of the window, whose launch choices live in
  the window (D-16's EP-13 addendum): v2 notes.
- Tests that run the real CLI (D-29): the stub plays the timeout, the deferred result and the
  resume; the hooks' answers are tested on their own.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`, `cargo
  build --release` with the app closed, then the install script.
- Unit tests: the adapter (a permission's input, capped; the denial that ends the turn; a deferred
  tool use kept whole with its questions; the answered input keyed by question text); the command
  line (the defer route's hook in `--settings` beside the neighbours' deny rules, refused without
  `--settings` or `crw.exe`; an answer's resume with `--resume`, the answer hook and its matcher);
  the hooks (`--defer` defers; `--answer` allows the matching tool use with its input, defers
  another question, leaves other tools alone, defers when the file is missing); the ledger (the
  latency written and read, old schemas read); the registry (`decisions` and `answer_timeout` read,
  set and checked); the tile's answer drafts (one, several, Other, unanswered) and the route chosen
  after a refused handshake.
- The runner against the stub: a request that outlives the limit is denied with the interrupt and the
  run ends deferred; an answer's latency recorded; a deferred result's tool use kept and counted; a
  deferred resume sends no prompt, passes the answer hook and removes its file.
- Screenshot runs: the Decisions tile light at 1600 × 1000 and dark in a narrow window, a deferred
  card, a waiting visual session with Focus, the dialog's line on questions; each picture looked at.
- The commit's added lines pass rule 3's word check; the owner has reviewed every decision.

> **Completion note (2026-11-02; opus/max; ≈ 1 h 10 min, about 1 h active).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`, from 22:20 to the commits: the reading at pickup with
> OT-7 ticked and no inbox message, this brief, the code and its tests, the pictures, a supervised
> probe on the owner's yes (two rounds), the owner's round of four (answered within minutes), the
> records and the commits. Against L ≈ 2 h. The model is Opus 5.5; the session's environment gives
> the effort as max.
>
> **What was built.** `session::protocol`: a permission request's input (indented JSON, cut to 4,000
> characters), `Deferred` (the deferred call kept whole: id, tool, summary, questions, input) in
> `Finish::deferred`, `Protocol::time_out` (a refusal with the turn interrupted) and
> `answered_input` (the questions with `answers` keyed by question text). `session::defer`: the
> defer route's decisions that `crw hook --defer` and `crw hook --answer <file>` print (the answer
> for its own tool use, a deferral for any other question, nothing for another tool; a missing file
> defers again) and the `--settings` hooks block in exec form. `session::launch`: `Route`,
> `Launch::{route, hooks, answer_timeout, answer, answered_after}`, one `--settings` with the
> neighbours' deny rules and the hooks, and `--permission-prompt-tool stdio` whenever the help names
> it (`Flags::prompt_tool` finds it inside `--permission-prompts`' description), else a dropped
> wish. The runner: a request's time limit (`Event::TimedOut`, the outcome deferred), the answers'
> latency, a deferred question counted, no prompt for an answer's resume, the answer's file written
> before the start and removed before the exit is reported. The ledger writes `answer_latency_s`
> (no schema change), worded by `ledger::asked`. The registry: `settings.decisions` and a project's
> `answer_timeout` (1 to 1,440 minutes), set with `crw setting decisions` and `crw set <project>
> answer_timeout <minutes|none>`; `crw sessions` gains a column for the questions. `visual::focus`.
> In the window: `decisions`, the tile (the answer forms with `Draft`, the deferred runs with Answer
> and resume, Waiting elsewhere with Focus, the refusals); `Sessions::{respond, answer_open, route,
> route_for, deferred}`; each request's answer under it in the transcript and a deferred result's
> line; Focus on the seen card; the dialog's Questions line; `Action::{Respond, AnswerDeferred,
> Focus}`; the switch to the defer route after a refused handshake, said once; and the screenshot
> run's `decisions` flag with harbor's requests under a 30-minute limit and a deferred quarry run.
>
> **The probe (the owner's yes).** A throwaway `crates/core/examples/probe_decisions.rs` ran three
> headless sessions with Haiku at low effort in a scratch folder under the session's scratchpad, with
> no ledger: the protocol's answer, a 5-second limit, and the defer route with its answer's resume.
> In the first round every session said "AskUserQuestion is disabled for this session": the runner
> had passed `--permission-prompts host` without naming itself the host, which the Agent SDK does
> with `--permission-prompt-tool stdio` (the CLI reference: a `-p` run denies what needs approval
> unless something approves it), so since EP-9 a real headless run's permission requests were
> denied rather than sent to the window. With the flag (the owner's option said a surprise would be
> fixed before the commit; the second round stayed within its few-cents envelope) the question came
> as a request and the answer "Grey" came back as the session's reply; the limit refused the request
> after 5 s, the turn ended (`error_during_execution`, exit 1) and the run was deferred, the session
> reading the CLI's own words for an interrupted refusal rather than the tool's message; the defer
> route's hook (the debug `crw.exe`) deferred the question (`tool_deferred`), and the resume with no
> prompt ran the call again before the session's init, the answer's hook allowed it, and "Blue" came
> back. The driver was deleted and the transcripts purged (`claude purge --dry-run`, then `-y`, on
> the scratch folder alone: one item).
>
> **Verification.** `cargo test` 292 passed (41 + 1 + 1 + 212 + 2 + 7 + 6 + 11 + 11), 294 with
> `--features screenshot`; both clippy runs and `cargo fmt --check` clean; `cargo build --release`
> with the app closed, then the install script; `crw signals` says every entry runs the new exe, and
> `crw setting decisions` and `crw sessions` read the owner's registry and ledger without a write.
> Twenty-one new tests: the adapter (a question's empty input and a long input cut, the limit's
> refusal, a deferred call kept whole, the answered input), the defer route's decisions (three), the
> command line (the defer route beside the neighbours' rules; an answer's resume and its matcher),
> the help's prompt tool, the registry's route and time limit, `visual::focus`, the runner (a
> request past its limit; a deferred call and its answer's resume without a prompt), `crw hook
> --defer|--answer`, the tile's drafts and words, a request left open by the limit or the hook, the
> route after a refused handshake, and the transcript's answers; the whole run's test now checks the
> latency and the prompt tool. Screenshot runs, each looked at: the Decisions tile light at 1600 ×
> 1000 (a wait line that wrapped mid-phrase, given a row of its own), light at 1700 × 1400, dark
> 1300 wide as tall as the screen allows, light at 2500 × 1420 (the deferred card, and Waiting
> elsewhere with Focus, whose "waits … waiting for you" now says it once), harbor's transcript with
> each request's state under it, and the dialog's Questions line, dark. Rule 3's word list found
> nothing in the added lines or the new files.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-02),** two rounds, every answer the recommended
> option:
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | A supervised real probe of the routes | Run the probe | Two rounds: the first found the missing permission host, fixed; the second verified the three routes; the driver deleted, the transcripts purged |
> | 2 | The default launch mode, now that headless sessions answer | Leave it for EP-16 | Unchanged; EP-16's notes say so |
> | 3 | The D-19 and D-26 addenda | Accept as written | As drafted |
> | 4 | README, roadmap notes, risk 2, v2 notes, OT-8 | Keep as written, OT-8 gated | As drafted; OT-8 gated before EP-15 |
> | 5 | Commit and push | Commit and push | Two commits in the house pattern, pushed; the first carries OT-7's tick |
>
> **Routine choices (the session's call, logged).** (1) The limit refuses with the turn interrupted,
> so that the session stops rather than guess. (2) The route is chosen at launch; a refused
> handshake switches the window's later runs of that CLI and version, said once in the status bar.
> (3) The answer reaches its hook through a file in the tool's local folder, which the runner writes
> before the start and removes before the exit is reported, rather than through the hook's
> arguments. (4) `crw hook --defer|--answer` beside the relay, printing a decision and never
> failing. (5) An answer hook's matcher adds the deferred call's own tool when its name is plain.
> (6) The latency is the mean over the run's answered requests; a refusal at the limit is none.
> (7) A question left blank goes unanswered; Send answers needs one answer. (8) A single-answer
> question's Other replaces the pick; several picks and Other are joined with ", ". (9) Focus for a
> session that waits in VS Code (its surface, or a visual launch's); a terminal's says where it
> waits. (10) The latest 20 refusals kept per session. (11) The tile's countdown from each
> request's arrival in the window; the runner keeps its own clock for the limit. (12) The tile in a
> module of its own (`app::decisions`). (13) The prompt tool passed when the help names it anywhere,
> else dropped with a note, never a refusal.
>
> **Handed on.** Nothing: the user settings are unchanged, and the environment's memory of crw's
> hooks still holds.
>
> **Next: EP-15** (Notifications), after OT-8.
