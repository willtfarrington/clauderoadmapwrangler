---
ep: 47
title: The Chat pane's send — Enter alone sends, and a message sent while the CLI's preflight is being asked waits for it and goes by itself
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-47 — The Chat pane's send

**Size:** M · **Depends on:** EP-45 · **Decisions:** D-27 (and D-15, D-29 as standing; D-3, D-10)

## Context

The owner's request 7 at EP-45: the chat's first Send "about half the time" says "claude has not
answered yet: try again in a moment.", and Enter should send.

**What the code says** (read at pickup).

- `chat_send` (`crates/app/src/chat.rs`) refuses with those words when `Sessions::preflight`
  (`crates/app/src/sessions.rs`) returns `None`. `preflight` hands back the answers when they are
  fresh for the program asked, and otherwise starts a worker thread (`Preflight::run`,
  `crates/core/src/session/cli.rs`: `--version`, `auth status` and `--help`, one child after the
  other under `process::turn()`, `SPACING` between) and returns `None` until it answers. The
  answers are trusted for `MAX_AGE`, an hour; nothing asks again before a launch or a Send needs
  them, so the first Send after an hour's quiet always finds none. The state is `Asked`: `Not`,
  `Asking { program, receiver }`, `Ready(preflight)`; `poll_preflight` takes an answer in
  (`Sessions::receive`, each pass), `forget_preflight` drops it after a refused start,
  `preflight_ready` hands back the last answers fresh or not (the Chat pane's effort picker and the
  Diagnostics pane read them).
- `chat_send` also refuses with "A session is starting: try again in a moment." while any start
  is on its way (`Sessions::is_starting`: one start goes at a time) or the chat's own turn is; the
  Send button and Ctrl+Enter are greyed in both cases, so the words show only after a race.
- The input is `egui::TextEdit::multiline` with the hint "Ask about the portfolio (Ctrl+Enter
  sends)"; `sent_by_key` reads `modifiers.command && Enter` while the box has the focus. egui
  0.37's `TextEdit::return_key` names the shortcut that makes a new line (plain Enter by
  default); a shortcut that is not it makes none.
- The launch dialog asks the preflight as it opens (`Board::launch_dialog`, each frame) and shows
  "asking…" while `preflight` is `None` and `is_asking`; its Start headless refuses with "claude
  has not answered yet" as the chat did. Screenshot runs and tests can the answers
  (`sessions.canned`, `launch::canned_preflight`); a registry's `claude_cli` naming no file gives a
  preflight that starts no child (`found: false`), which the tests use.

**Session start (2026-11-23, 21:09 local time).** A chained headless run: the window restarted
into EP-46's build at 21:09:30 and chained EP-47 at 21:09:39 (the rolling log and `crw
autopilot`: claude-opus-5-5 at max, the rules' choice, M's cap of 1 h 30 min, the chain's own mark
at 80 % of the session window); `crw sessions clauderoadmapwrangler` lists the row (`004c2758`,
headless, EP-47, open), and the prompt is the tool's chained opener. OT-30 is open, gated before
EP-54, not this brief's; no inbox message is open. The field check read, since EP-46's record
commit (21:09:01): `crash.log`, unchanged since the panic of 04:00 that EP-43a read; the rolling
log, the install's restart refused while EP-46 worked, the restart at the gate's moment and this
chain; `crw notifications`, EP-46's finish and this chain, and another project's visual session
waiting; `crw autopilot`, not paused, the chain's lines. Nothing odd.

**Against the cap.** Three parts in two files of the window and one constant in core, with their
tests, the review an M asks for, a screenshot run and the records. Judged to fit M's cap with its
checks and its close, read at 21:14 with five minutes spent; should the code not compile by
22:00, part 2 (the renewal) is split off as a row of its own and the rest closed.

## In scope

1. **A message sent before claude has answered waits and goes by itself.** The chat holds a queued
   send, `Chat::queued: Option<Wait>`: `Wait::Answers` when Send finds no answers in hand (the ask
   goes out at that Send, as today), `Wait::Start` when another start is on its way (the "A
   session is starting" case, which the queue can hold: Send is no longer greyed for another
   start, only for the chat's own). The box shows the message, greyed, and under it a spinner, the
   wait in words and Cancel (`Action::ChatCancel`, which leaves the message typed); New
   conversation and the list's rows are greyed while it waits. Each pass, `Board::pass`, after the
   start and the answers are taken in, sends it once what it waits for has come
   (`chat_send_queued`): the answers are in, or the other start came back. A refusal the answers
   themselves give (no CLI, not logged in, below the minimum, a flag missing) is said as today,
   and the message stays typed; an ask that ended without answers drops the queue with words
   rather than asking again, so a queue never starts a second ask by itself (D-29). The queue runs
   in the hidden window's pass too: the owner sent it.
2. **Answers about to lapse are asked again while the window is shown.** In the shown window's
   pass, answers in hand that lapse within five minutes, or have lapsed, are asked again on the
   worker thread (`Sessions::renew_preflight`, the program they were taken of; `Preflight::is_due`
   and `RENEW_BEFORE` in core), one child at a time through `process::turn()` as every preflight;
   the old answers stay in hand meanwhile (`Asked::Asking` keeps them as `previous`, handed out
   while fresh), so neither the chat nor the launch dialog meets a gap; `forget_preflight` drops
   them. The hidden window asks nothing (D-29: no child without a reason), and answers never
   asked are not asked by this (the first Send or launch asks them).
3. **Enter sends.** The box's new-line shortcut becomes Shift+Enter (`return_key`); Enter alone
   and Ctrl+Enter send while Send could be pressed, and do nothing otherwise. The hint under the
   box says so.
4. **The records.** The README's chat paragraph; a D-27 addendum; the screenshot feature's
   `chat=queued` setup, which draws a message waiting.

## Out of scope

- The launch dialog's own refusal at Start headless, and the Commit and Resume buttons' (their
  dialog or tab asks the answers as it shows, and part 2 keeps them fresh while the window is up).
- A message queued while the conversation's reply streams (Send stays greyed then).
- The phone, which has no chat (free chat from the phone is v2's).
- OT-26's line in `owner-tasks.md`, ticked, whose "Send (or Ctrl+Enter)" stays true.

## Verification / acceptance

- Unit tests: a Send with no answers in hand queues the message, asks once, and, when the answers
  come and refuse (a `claude_cli` naming no file), says their refusal with the message kept and
  the queue gone; a queue whose ask ended without answers gives up with words and asks nothing;
  a message queued behind another start waits while it is on its way and goes once it came back;
  Cancel keeps the message; the box greys and shows the wait; Enter and Ctrl+Enter send, Shift+Enter
  makes a new line and sends nothing; answers within five minutes of lapsing are due and those
  fresher are not; a renewal keeps the old answers in hand while it asks, `forget_preflight`
  drops them, and the hidden window renews nothing. No test starts a session or the real CLI
  (rule 5).
- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings`, `cargo
  fmt --check`; a review subagent given the diff (an M brief).
- Screenshot runs of the Chat pane with a message waiting (`chat=queued`) at the default size and
  in a narrow pane, and one over a copy of the real portfolio (rule 5), kept in the scratchpad.
- The release build, and the install last.
- No tracked file names a managed project, the owner's login or the machine (rule 3); the word
  check prints nothing on the staged diff.

**Residual risk.** No test runs the real CLI (rule 5), so the real answers' time is not measured
here: three short children one after the other with the spacing between, a second or two by
EP-9's account, which a Send made then now waits out instead of refusing. The queued message goes
from the pass the answers' thread wakes with its repaint, shown or hidden; a hidden window gets
passes at its repaint requests (EP-15's finding), but the route is not exercised in a real hidden
window. The renewal runs only while the window is shown, so after more than an hour in the tray
the first pass after showing starts it, and a Send within the next second or two waits for it.
While the window stays shown the renewal asks every 55 minutes whether or not the chat or the
launch dialog is used, three short children an hour, as the brief note chose. The key tests
drive egui with synthetic press and release events; an input method's composition that ends with
Enter (a dead key, an East Asian IME) is not tested and could send where it should only confirm.
The screenshot runs show the waiting state as canned (the greyed box, Cancel, the spinner and the
words, fitting a narrow pane), not a real move from waiting to sent, which the unit tests cover
with answers sent by hand and a program that is no file. The launch dialog's own Start headless
still refuses while the answers are asked (out of scope), more rarely now. The app's renewal
tests need answers older than the machine's uptime, so on a machine up for less than an hour (a
fresh CI runner) they skip their due part; core's rule is tested on durations whatever the
uptime.

> **Completion note (2026-11-23; claude-opus-5-5/max, a chained headless run; 44 minutes from the
> start to the work commit, no owner question).** Chained at 21:09 after EP-46 (ledger row
> `004c2758`) under M's cap of 1 h 30 min; the row was judged at the brief to fit with its checks
> and its close, and was not split. The work commit at 21:53, the record commit, then the
> install.
>
> **Owner decisions.** None arose: the brief note settled the behaviour, nothing outward-facing
> changed, and the commits are the autopilot's standing yes (D-22); the push is the owner's.
>
> **Built.**
>
> 1. **The queued send** (`crates/app/src/chat.rs`): `Chat::queued: Option<SendWait>` (`Answers`,
>    `Start`, `Reply`); `chat_send` queues where it refused ("claude has not answered yet", "A
>    session is starting"); `chat_send_queued`, in `Board::pass` after the starts and the answers
>    are taken in, never asks the CLI: it goes on only with fresh answers in hand for the CLI a
>    Send starts (`Sessions::preflight_in_hand`; `board::cli_program`, factored out of
>    `launch_cli`), waits while any ask runs, and otherwise drops the queue with words. Cancel
>    (`Action::ChatCancel`); the box greyed and read-only while it waits, keeping the focus
>    (`draft_box`); Send no longer greyed for another start; New conversation, the list and the
>    question forms held while it waits; the wait's short words, the sentence on hover, and
>    `WAIT_ROOM` in the footer.
> 2. **The renewal** (`crates/app/src/sessions.rs`, `crates/core/src/session/cli.rs`):
>    `RENEW_BEFORE` (5 min) and `Preflight::is_due`; `Asked::Asking { previous }`, handed out while
>    fresh; `Sessions::renew_preflight` in the shown window's pass; `forget_preflight` drops the
>    old answers; `preflight_ready` hands them to the effort picker and the Diagnostics pane.
> 3. **Enter sends** (`draft_box`, `enter_sends`): Shift+Enter is the box's new-line key
>    (`TextEdit::return_key`), Enter alone and Ctrl+Enter send while Send could be pressed, a
>    held key's repeats send nothing; the hint says so; only the sent text leaves the box when
>    the turn starts (`after_sent`).
> 4. **The records**: the README's chat paragraph; a D-27 addendum; the screenshot feature's
>    `chat=queued`.
>
> **Verification.** `cargo test`: 278 in `app`, 479 in `core`, the CLI's and the integration
> suites, all passing; `cargo test --features screenshot`: 286 in `app`; both clippy runs with
> `-D warnings` and `cargo fmt --check` clean, run again after the review's fixes. The feature's
> first test run failed twice: the Enter test pressed Enter twice without a release, which egui
> counts as a repeat (the test now sends press and release, as a keyboard does), and
> `crash::tests::the_hook_writes_a_panicking_threads_report` beside it (EP-43a's finding).
> Screenshot runs, kept in the git-ignored `target\ep47-shots`: the Chat pane with a waiting
> message over the fixtures at 1600 × 1000, whose first picture showed the long wait sentence
> pushing Cancel and the session line past the pane's foot (fixed with short words, Cancel first
> and the footer's room), again after the review's fix (the box greyed and read-only), and over
> a copy of three non-data projects' roadmaps made with `scripts\copy-roadmap.ps1` one at a time,
> at 1100 × 900 with the pane at its narrowest: nothing cut. The release build passed on the final
> tree; the word check printed nothing.
>
> **Review** (a subagent given the diff and the brief's In scope, about 18 minutes). Four points,
> all fixed:
>
> 1. The disabled box dropped the keyboard focus, so keys typed after Enter reached the window's
>    shortcuts (A toggles the selected project's autopilot, S opens the launch dialog): a waiting
>    message is now shown in a read-only box of the same id, which keeps the focus; a test.
> 2. The queue's retry re-entered `chat_send`, whose `launch_cli` asks whenever the CLI differs
>    from the one asked, so two callers naming two CLIs (a launch dialog opened before a registry
>    change, the autopilot's queued start) could replace each other's ask at every pass: the
>    answers' wait now asks nothing (point 1 of Built); a test.
> 3. A start's return cleared the whole box, which with Enter as the send key could wipe what was
>    typed after the message meanwhile: only the sent text leaves; a test.
> 4. Two weak test assertions (one always true, one racing the renewal's thread): replaced by a
>    test of the old answers while an ask runs that starts no thread, and core's rule tested on
>    durations.
>
> **Session's calls.**
>
> 1. The reply wait, found by the session while the review ran: a message queued behind a
>    deferred question's resume met the conversation replying when the start came back, and the
>    queue would have gone unsent without a word; it now waits for the reply and goes as the next
>    turn, or after Stop.
> 2. The renewal also renews lapsed answers, not only those near their lapse: a window shown
>    after more than an hour in the tray asks at its first pass.
> 3. A queued message goes from the hidden window's pass too: the owner sent it.
> 4. OT-26's ticked line left as written: its "Send (or Ctrl+Enter)" stays true.
>
> No owner task and no message to another project.
