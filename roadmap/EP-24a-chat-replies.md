---
ep: EP-24a
title: "Chat, the replies: Markdown drawn with egui_commonmark, the conversation list, the model picker, the session's questions answered in the tile"
size: M
model: fable (claude-fable-5-1)
effort: xhigh
status: done
---

# EP-24a — Chat, the replies

**Session start (2026-11-11).** A headless run chained by the autopilot after EP-24 (ledger row
`196590e6`, `crw sessions clauderoadmapwrangler`: headless, fable at xhigh, M's cap of 90 min,
started 19:48 through the window's restart into EP-24's build), the prompt the opener. Read
first: `roadmap/README.md` (EP-24a is the first unchecked row of P4; its brief note, EP-24's and
EP-24b's, which says what this brief leaves to it), `DECISIONS.md` (D-27 with its EP-23 and
EP-24 addenda, D-15's EP-23 addendum, which clears `egui_commonmark` 0.26.0, D-26, D-18, D-19),
EP-24's completion note, `roadmap/owner-tasks.md` (OT-26 open, the first real conversation, not
gated) and `roadmap/inbox.md` (no open message). The field check: `crash.log` is EP-15d's old
report, seen already (`crash.seen` beside it); `crw notifications` shows EP-24's wait, its finish
and this chain's start, nothing odd; `crw sessions clauderoadmapwrangler` this row open and
EP-24's completed; `crw autopilot` the chain held after EP-24 for the restart and this run
started at 19:48; `crw chats` no conversation yet (OT-26 waits for the owner). The code read
before the brief: `app::chat` (the tile, the exchange as plain labels), `core::chat` (the launch,
the digest, `Turn`), `app::decisions` (`Draft`, `question_form`, `request_card`), `app::sessions`
(`Live::take`'s chat arm, `Sessions::chat`, `respond`, `History::refresh`), `app::launch` (the
model and effort pickers, `efforts_of`, `model_text`), `core::ledger` (`conversations`,
`touch_conversation`), `app::screenshot` (the `chat` flag), `app::toasts` (`Target`); the
window opens no URL anywhere today.

## Context

EP-24 built the hosted session and a plain tile: one read-only Claude Code session per
conversation, its replies drawn as plain labels while they stream, Opus 5.5 at medium as
constants, the conversation rows in the ledger (schema 6) with no way to pick one up again from
the window, and a chat's questions answered in the Decisions tile as any run's. This brief, cut
from EP-24 at EP-23 (D-27 addendum of 2026-11-11), takes the replies: Markdown drawn through
`egui_commonmark` 0.26.0 (cleared by D-15's EP-23 addendum: built against egui 0.37,
egui_extras 0.37 and pulldown-cmark 0.13, all of which the lock holds), the conversation list,
the model and effort picker per conversation, and the session's own questions answered in the
tile with the Decisions form. EP-24b takes the delegated actions, the `crw` commands the session
may run and the chat's mentions.

What the code gives: `Entry::Text` and `Transcript::streaming` hold a reply's Markdown as text;
`Ledger::conversations` lists the rows with their turns and cost; `Turn::Resume` with a session
id and `Sessions::chat` with no live start a resumed conversation's run; `decisions::Draft` and
`question_form` are the form, `Sessions::respond` the answer's path, `Action::AnswerDeferred` a
deferred question's; the launch dialog's `model_picker` and `effort_picker` take the catalog
Claude Code caches and the CLI's effort levels; `Live::take` refuses a chat's permission
requests, so only questions can wait on a chat.

## In scope

1. **The crate.** `egui_commonmark = "=0.26.0"` joins `[workspace.dependencies]` with its
   default features off and the pulldown-cmark backend on, no syntax highlighting and no image
   loading (D-15 addendum; the lock's additions named in the completion note); `app` opts in.
2. **The replies as Markdown.** `app::chat::exchange` draws each reply (`Entry::Text`) and the
   text streaming in through `CommonMarkViewer` with a `CommonMarkCache` kept in the tile's
   state: headings, paragraphs with inline marks, lists, fenced and indented code, tables,
   quotes, rules, links. In the theme's colours where the crate allows: its headings, code and
   quotes take egui's visuals; a link's text takes the violet the notes give a reference and a
   link (`style::reference`), set on a scoped `Ui`. A link's click opens nothing: the window
   opens no URL today, and a reply is a session's text; the target goes to the status bar in a
   notice (put to the owner; the alternative opens `http(s)` targets in the browser). The
   notes' `markup` stays as it is, and the Sessions pane's transcripts stay plain.
3. **The conversation list.** `chat::Conversations` reads the ledger's latest conversations as
   `History` reads the rows (on the pane's draw, stale after a start or an end, else every half
   minute; a screenshot run's canned): each with its title, its last activity, its turns, its
   model at its effort and its cost. The list stands in the tile where the placeholder stood
   while no conversation is open, and folds under a `Conversations · N` header above the
   exchange while one is; the open one is marked. A row's click picks the conversation up: the
   tile takes its title, its session id, its model and effort, and a note says the earlier turns
   are the session's own record (the tool keeps no transcript, D-26), the next Send resuming it
   with `--resume` (`Turn::Resume`, a fresh live). Refused while a reply streams.
4. **The model and effort picker.** Model and Effort combo boxes in the tile's header (the
   launch dialog's pickers, shared), the catalog read from Claude Code's cache once per window
   run, the efforts those the model and the CLI's help list; the choice is the conversation's
   and goes with its next turn; a new conversation starts at Opus 5.5 at medium (D-27); a
   conversation picked up from the list starts at its row's. The row's model and effort follow
   its latest turn (`touch_conversation` updates them with the last activity; no schema step).
   Greyed while a reply streams or a turn starts.
5. **The questions in the tile.** A chat live's waiting requests are drawn in the exchange as
   the Decisions form's cards (`decisions::request_card`, shared), each answered with
   `Action::Respond`; a chat deferred with a question open (the defer route) shows the same
   card with Answer and resume (`Action::AnswerDeferred`). The Decisions tile and its count
   leave chat lives out; the Chat tab says `Chat · asks` while one waits; a chat's "waits for
   you" toast leads to the Chat pane (`Target::Chat`). Permission requests never reach the
   tile: the window refuses them (EP-24).
6. **Screenshot flags and tests.** `chat` cans a conversation whose first reply holds every
   block form and a canned conversation list; `chat=asks` the same with the second turn waiting
   on a question, its form in the tile. Tests draw both through `testing::Screen`: the
   Markdown's texts found, the list's rows, a click on a row picking it up, the form's Send
   answers reaching `Action::Respond`, the pickers' texts; the ledger's upsert of the model.
7. **Docs.** A D-27 addendum (the replies, the list, the picker, the questions, the link rule)
   and a D-15 addendum (the pin as made, its features); the README's status paragraph; OT-26's
   wording notes that the reply draws as Markdown from this build.

## Out of scope

Action chips, the `crw` commands the session may run, the chat's `@<project>` mentions
(EP-24b); syntax highlighting and images in replies (D-15 addendum: no such feature unless a
brief needs one); the Sessions pane's transcripts as Markdown; the notes' `markup`; a
conversation's earlier turns read back from Claude Code's transcripts (D-26); a chat from the
phone (P5).

## Verification / acceptance

- `cargo test --workspace`, `cargo test --features screenshot`, both clippy runs with
  `-D warnings`, `cargo fmt --check`; rule 3's word check on the staged diff prints nothing.
- `app`: the tile draws a canned reply's headings, list items, code and table cells through
  `testing::Screen`; the conversation list's rows and a click picking one up (the title, the
  id, the model and effort taken, Send then a resume); the question form in the tile and its
  Send answers reaching `Action::Respond`; the Decisions tile leaving a chat's question out
  and the Chat tab's title saying it; the pickers changing the next turn's launch.
- `core`: `touch_conversation` moving a row's model and effort on with its last activity.
- Screenshot runs with `chat` and `chat=asks` in both themes on the fake portfolio
  (`target\tmp\ep24a\`, never git), at 1600 and at 1200 points, the second for the narrow pane.
  The pane shows no roadmap content of its own (the list reads the ledger, the replies are
  canned), so no run over the copy of the real roadmaps is made, as at EP-24.
- A diff review by a subagent given the diff alone, launched once the code compiles (rule 6;
  EP-20c's lesson), its findings fixed or logged.

**Residual risk.** No real chat session runs in this brief (D-29): how a real reply's Markdown
reads at the pane's width (a long fenced block, a fence cut mid-stream, a list nested deep)
is seen on canned text only, and whether the questions of a real `manual` run reach the tile
rests on EP-24's reading of the permission-modes page. A table wider than the pane is cut at
its edge, seen on the canned reply and accepted by the owner (question 2): the pane is widened
for it. A deferred question's resume from the tile (the defer route, which this project's runs
take only when the control protocol's handshake is refused) is exercised by tests on a canned
live, not by a real run. The first real conversation is the owner's (OT-26), whose notes go to
the inbox; the pickers' real model list comes from the catalog Claude Code caches, which the
launch dialog has read since EP-10.

> **Completion note (2026-11-11; claude-fable-5-1/xhigh; about 50 min of wall time, a
> headless run under the autopilot, two questions answered in the Decisions tile).** Ledger
> row `196590e6`, chained after EP-24 at 19:48 through the window's restart into EP-24's build
> (an M's cap of 90 min); the prompt was the opener.
>
> **Built.**
>
> 1. **The crate** (D-15 addendum): `egui_commonmark = "=0.26.0"` in the workspace with its
>    default features off and `pulldown_cmark` on, `app` opting in; the lock gained seven
>    packages (`egui_commonmark`, `egui_commonmark_backend` 0.26.0, `egui_extras` 0.37.0 and
>    `pulldown-cmark` 0.13.6, pinned since EP-1 but used by no member until now, `enum-map`
>    3.1.0, `enum-map-derive` 1.0.1, `unicase` 2.10.0).
> 2. **The replies as Markdown** (D-27 addendum): `chat::markdown` draws each `Entry::Text` and
>    the text streaming in through `CommonMarkViewer::show_with_id` under a `push_id` of its
>    own, the link colour the notes' violet on the scoped `Ui`, the URL in the hover; a link's
>    click opens nothing: `take_opened` drains egui's `OpenUrl` commands after the exchange and
>    the status bar says the target (the owner's choice, question 1). The notes' `markup` and
>    the Sessions pane's transcripts are unchanged.
> 3. **The conversation list**: `chat::Conversations` reads `Ledger::conversations` as
>    `History` reads the rows (stale through `Sessions::conversations_stale` after a start or
>    an end, else every half minute), drawn in the exchange's place while no conversation is
>    open and under a `Conversations · n` fold while one is; a row's click is
>    `Action::ChatPickUp`, and `chat_pick_up` takes the row's title, id, model and effort, the
>    note over the exchange saying the earlier turns are the session's own record; `chat_send`
>    then resumes under `Turn::Resume` with no live. `Sessions::conversation_mut`.
> 4. **The pickers**: `chat::Pick` (`core`; `chat::launch` takes it), the launch dialog's
>    `model_picker`, `effort_picker`, `efforts_of` and `cli_efforts` shared (`pub(crate)`),
>    `Board::read_catalog` (the two inline reads folded in), `Sessions::preflight_ready` (the
>    effort levels without asking the CLI), each picker on a row of its own sized to the rest
>    of the row (`picker_width`); `touch_conversation` moves the row's model and effort with
>    its last activity; `Sessions::set_chat_pick` and `started` keeping a chat's whole launch,
>    so a deferred question's resume runs on the picker's choice.
> 5. **The questions in the tile**: `decisions::request_card` and `Wait` shared, `questions`
>    drawing the waiting requests (`Action::Respond`) and a deferred one (`Action::AnswerDeferred`,
>    `answer_deferred` marking the chat's start), `Sessions::waiting` and `deferred` leaving
>    chat lives out and both Decisions loops too, `Board::chat_asks` for the tab's `Chat · n`,
>    `Target::Chat` for the "waits for you" toast.
> 6. **Screenshot flags and tests**: `chat` with the first reply holding every block form and
>    three canned conversation rows, `chat=asks` (the second turn waiting on a question),
>    `chat=list` (the fold open); `testing::text_containing`; seven tests in `app::chat` (the
>    digest, the exchange and Send's gates, the Markdown and the link's click, the list's
>    pick-up, a question answered, a deferred question let go) and the core's launch and
>    upsert tests grown.
> 7. **Docs**: D-27 and D-15 addenda, the README's status paragraph, OT-26's wording.
>
> **The pictures** (`target\tmp\ep24a\`, never git): the Chat pane with `chat` at 1600 light
> and 1200 dark, `chat=asks` at both, `chat=list` at 1600, and the pane with no conversation.
> The first 1200 picture showed the model picker widening the narrow pane past its edge
> (EP-13's trap, a 170-point combo in a 160-point pane): each picker now takes a row of its own
> sized to the row's rest, and the pictures were taken again. A table wider than the pane is
> cut at its edge (question 2). No run over the copy of the real roadmaps: the pane draws no
> roadmap content of its own, as at EP-24.
>
> **The review (rule 6).** A subagent given the diff alone, launched once the code compiled
> and before the test gate (EP-20c's lesson), reported after eleven minutes: three confirmed
> findings, two plausible and eight nits. Fixed: (1, confirmed) New conversation and a row's
> pick-up stayed enabled while a deferred question's resume was on its way, which would have
> dismissed the live and left a running chat no pane shows: `answer_deferred` now marks the
> chat's start, and both refuse while any start is on its way, their buttons and rows greyed;
> (2, confirmed) a deferred question's resume ran on the conversation's first-turn model and
> effort, not the picker's, and wrote them back into the row: `set_chat_pick` before the
> resume, and `started` keeps a chat's whole launch; (3, confirmed) two replies with a table
> shared one egui grid id under `ui.scope`: `push_id` of the reply's id; (4, plausible) the
> link test asserted on a fresh frame: `click` returns the release pass's output; (5,
> plausible) a deferred question dropped silently by New conversation: the notice says it
> went. Nits taken: the manifest's wording, `questions`' parameter name, the rows' enabling
> aligned, the canned rows' "today" set against the run's clock. Logged, not changed: the
> fold's count is the latest twenty (`CONVERSATIONS_SHOWN`), the catalog is read once per
> window life, `row_words` builds a throwaway `Pick`.
>
> **Verification.** `cargo test --workspace`: 157 passed in `app`, 365 in `core`, 13 in the
> other suites; with the feature 165 in `app`; both clippy runs and `cargo fmt --check`
> clean; rule 3's word check on the diff printed nothing. The release build and the install
> (`scripts\install-shortcut.ps1`) ran after the record commit, the opener's last step; the
> restart is refused while this run works and taken at the gate's moment, or from File >
> Install and restart, the tray's, or the launch dialog's.
>
> **Owner decisions** (asked once the build stood, answered within minutes while the test
> gate and the review ran; both took the recommended option).
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | A reply's link: a click opens nothing and the status bar says the target, or `http(s)` targets open in the browser | Open nothing (recommended) | As built; D-27 addendum |
> | 2 | A table wider than the pane is cut at its edge: accept and widen the pane, or park a renderer of the tool's own | Accept (recommended) | D-27 addendum and the residual risk; no v2 note |
> | 3 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The list's place**: in the exchange's place while no conversation is open, a fold under
>    the header while one is, rather than a pane of its own.
> 2. **The row's model and effort follow the latest turn** (the upsert), so the list says what
>    a picked-up conversation resumes with; no schema step.
> 3. **The questions moved wholesale**: the Decisions tile lists a chat live's deny-rule
>    refusals still (a listing, not a count), nothing else of a chat's.
> 4. **`manual` runs have no time limit**, so a chat's question waits in the tile for good
>    unless the defer route's hook deferred it; the tile answers both.
> 5. **The scratchpad** is `target\tmp\ep24a\` (ignored): the pictures, the logs and the diff
>    the review read.
>
> **Owner task:** none new; OT-26's wording says the reply draws as Markdown from this build.
> No message for another project. **Next: EP-24b** (the delegated actions, the `crw` commands
> the session may run, the chat's mentions), chained by the autopilot when its gate allows,
> else from the window.
