---
ep: 30a
title: Phone pages, a project and the decisions — the project page with the next brief's note, owner tasks and inbox messages ticked, a message composed; the decisions page answering questions and permission requests
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-30a — Phone pages, a project and the decisions

**Size:** M · **Depends on:** EP-30 (the pages' base), EP-14 (the decisions tile), EP-20c (compose)
· **Decisions:** D-28, D-10, D-13, D-19 (and D-14, D-15, D-29, D-3 as standing)

## Context

EP-30 gave the window's server the phone's base: the published snapshot (`app::published`), the
`/m/` sub-router behind `crw_core::remote`'s identity check, the remote-action log, askama's
layout and the read-only list. Nothing on a page changes anything yet, and the server has no way
to ask the window to act: the snapshot's ask is an `AtomicBool` and a wake. This brief adds the
two pages that tick and answer (D-28 addendum of 2026-11-15: tick an owner task, tick an inbox
message with its disposition, compose a message, answer a question or a permission request), and
the request channel every form goes through, since every write and every answer runs on the
window's own thread.

**Session start (2026-11-16, 19:57 local time).** A chained headless run from the window: the
chain log's "chained EP-30a after EP-30: claude-opus-5-5 at max, the rules' choice, M's cap of
1 h 30 min; it holds at 80 % of the session (five-hour) window, your mark for this chain" at
19:56, after the window's restart into EP-30's build. The prompt is the tool's opener, and `crw
sessions clauderoadmapwrangler` lists the row (`c304fa80`, headless, EP-30a, claude-opus-5-5/max,
open). No owner task and no inbox message is open. The field check read, since EP-30's record
commit (2026-11-16, 19:54):

- `crash.log`: the one report of 2026-11-03, nothing newer.
- `crw notifications`: EP-30's question at 19:26 (IN-14's triage), its finish at 19:56:24, and
  this chain's start at 19:56:57.
- `crw sessions`: this row; EP-30's, completed in 58 min with its one question answered in 8 min.
- `crw autopilot`: not paused; on for this project and five others, all in mode `auto`; the
  newest lines are this chain's hold for the restart and its start.
- The rolling log: 19:55:36, EP-30's install asked for a restart, refused while its run worked;
  19:56:24, the run's finish; 19:56:25, the restart into the build of 19:55, carrying one held
  chain and one chain plan, as EP-37a built it.

Nothing odd to triage.

## In scope

1. **The request channel** (the brief notes' "first form builds it", on `RestartRequest`'s
   pattern).
   - `server::PageRequest`: an `Order`, the caller's identity in the window's words
     (`remote::Caller::identity`) and a `tokio::sync::oneshot` reply carrying the outcome's one
     line. `Order` is one of: tick an owner task, tick an inbox message with its disposition,
     send a message (text, urgent), answer a request (the session's key, the request's id and
     the form as sent).
   - Sent through `Shared` by the form's handler, which wakes the window and waits at most ten
     seconds for the answer. `Server::take_orders`, `Watch::take_orders`.
   - Drained in `Board::pass`, shown or hidden, before the queue runs. An order whose page
     stopped waiting is dropped unrun, as a restart request's is. A write reads the files again.
   - Each outcome is said in the status bar and the rolling log with the identity
     (`Board::notify`), so the desk sees what the phone did.
2. **The pages' parts in the snapshot** (`app::published`). The version rises with any part's
   content, and every time in them is a clock time, never an age, so an idle window publishes no
   new version.
   - `Project`, one per project: its name and state; the next brief (id, size, title) and its
     note's text (`Roadmap::note`, `Note::text`, the bullet's own label left out as the Detail
     pane leaves it); what waits on the owner (`Snapshot::waiting`: the open items, each with
     its id, gate, text, meta line and whether it is a tickable task line, and how many are
     resolved); the older forms uncounted (`Snapshot::uncounted`, open ones); the inbox
     (`snapshot.inbox`: the open messages with id, sender, date, urgency and text; open of
     total; none when the file is not there); the sessions' words (the list line's); and a
     session at work there or a revert open, in the compose dialog's words
     (`Board::busy_projects`).
   - `Decisions`: each request a window's session waits on (key, the session's title and
     project, the request, when it came, the project's time limit), each deferred run's open
     request (when and by which route it was deferred), and each session Claude Code reports
     waiting elsewhere (project, mode, where, its words). A chat's questions stay the Chat
     pane's (`Sessions::waiting` leaves them out). `sessions::Waiting` gains the clock time its
     request came, beside its `Instant`.
3. **The routes**, on the guarded sub-router.
   - `GET /m/p/{project}`: the project page. Its state and marks; the next brief with its size,
     title and note drawn from `markup::blocks` and `markup::spans` (labels, numbered parts,
     paragraphs; a long item or message a sentence to a line); what waits, each open task line
     with a Tick form and each older form marked read-only with its file; "Risks and older
     forms" uncounted; the inbox with a disposition field and Tick per open message; the
     compose form (text, urgent) with a session's words where one works there; the sessions'
     words. An unknown project is a page not found.
   - `POST /m/p/{project}/tick` (`id`), `/m/p/{project}/message` (`id`, `disposition`),
     `/m/p/{project}/send` (`text`, `urgent`).
   - `GET /m/decisions`: each waiting request as a form, built by `decisions::Draft`'s rules (a
     radio per option for one answer, a box per option for several, the owner's own words, a
     reason for a refusal or a decline); the deferred runs, whose answer resumes them; the
     sessions waiting elsewhere, read-only.
   - `POST /m/decisions/answer` (`key`, `request`, `do` = allow, deny, answer or decline,
     `reason`, `pick<q>` once or more, `other<q>`).
   - The list page links each project to its page and names the decisions page with its count.
   - Each POST answers a 303 to its page with `?done=<n>`: one line kept by the server for a
     few minutes (a small store of at most 32), so the page never draws text a link carries.
4. **The orders on the window's side.**
   - **Tick an owner task:** `tasks::tick` then `Board::log_tick`, as the Detail pane's box
     does, refactored to return its words.
   - **Tick a message:** `inbox::tick` with the form's own disposition, which refuses an empty
     one (`checklist::Error::NoDisposition`); `log_tick` with it.
   - **Send a message:** `mailman::send_from_owner` on a worker thread, as the compose dialog
     runs it, the answer given once written and the files read again. The dialog's refusals
     hold: a screenshot run's projects, a revert open there, an empty message (the flag
     alone). A session at work there is said, as the dialog says it.
   - **Answer:** the window finds the request by key and id among the waiting ones, or as a
     deferred run's open request, builds the `Response` from the form with `Draft`'s rules (at
     least one answer to send; a decline needs its reason; a denial's reason optional), and
     sends it through `Sessions::respond`, or resumes the deferred run through
     `Board::answer_deferred`'s path. A request no longer waiting answers "already answered";
     the words afterwards are `decisions::answer_words`.
   - A screenshot run refuses every order, as it refuses the desk's.
5. **Forms without a new crate.** axum keeps `http1`, `json` and `tokio`: the body is taken as
   bytes and decoded as `application/x-www-form-urlencoded` by a few lines of the tool's own,
   and a link's project name is percent-encoded the same way; no askama `urlencode`, no axum
   `form`, nothing new in the lock (D-15 addendum).
6. **The screenshot feature's `pages=<file.html>`** also writes each project's page and the
   decisions page beside the file (`<stem>-<project>.html`, `<stem>-decisions.html`), the way to
   read the pages on real content (rule 5).
7. **The records.** Addenda to D-28 (the pages that tick and answer as built), D-19 (answers
   from the phone) and D-13 (compose from the phone); the README's status sentence; this brief
   and its completion note.

## Out of scope

- Start, stop, pause and resume from the phone, the usage strip, the autopilot block, the event
  stream, the script and `crw remote` (EP-30b); IN-14's fix (parked in EP-30b).
- The sessions' pages, the notifications page and the doctor's page (EP-31); Enable remote view
  (EP-31a).
- Untick, the ticked tasks and messages folded, Focus for a session waiting in VS Code, a chat's
  questions: the desk's (D-28 lists no untick; Focus opens a window on the machine).
- Setting `owner_login` in the owner's registry (EP-31a's Enable).

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings` and
  `cargo fmt --check` pass (rule 6). Being an M brief, the diff is read by a review subagent given
  the diff and not this session's reasoning, before the release build.
- **Tests:**
  - the form decoding and the link encoding, UTF-8 and `+` included;
  - the published parts from the fake portfolio: a project's items, messages and note; the
    decisions from canned sessions, a live request, a deferred one and a chat's left out; the
    version unchanged by a second build of the same content;
  - the pages rendered: escaping, the forms' actions and fields, a radio for one answer and
    boxes for several, the note's blocks, read-only older forms, the flash line;
  - the server over HTTP: the project page and the decisions page served locally; an unknown
    project a 404; a form without its origin refused; a tick's order reaching the channel, its
    answer redirected with `?done=` and drawn on the next GET; an owner's form through Serve
    logged before its order; a page that stops waiting leaves its order dropped;
  - the board: an owner task and a message ticked in a scratch project with the ledger's tick
    rows; an empty disposition refused; a message sent into a scratch inbox; an answer to a
    request no longer waiting "already answered"; a live request answered; a screenshot run
    refusing every order.
- A screenshot run with `pages=` over the fixtures and over a copy of real roadmaps (rule 5),
  each page read in the scratchpad; the brief names what was checked, never the projects.
- Rule 3's word check prints nothing for the staged diff. Then two commits in the house pattern,
  not pushed, and the release build and the install last.

**Residual risk.** What the session could not exercise, and why it expects it to hold.

1. **A real browser's forms.** No test drives one (D-29). The forms are plain HTML posted as
   `application/x-www-form-urlencoded`, checked as text and over HTTP. A browser sends `Origin`
   on a POST unless the referrer policy is `no-referrer`, which these pages never set, and the
   Content-Security-Policy's `form-action 'self'` covers the form and its same-origin 303. The
   owner may try the pages on the machine itself at `http://127.0.0.1:47831/m/`, where the
   loopback counts as local; what a phone's browser does with them waits for EP-31a's Enable.
2. **The orders in the real window's hidden pass.** The tests run the board's orders with no
   frame. The real window runs `logic` while hidden when a repaint is asked (eframe 0.37), and
   the server's wake asks for one, as EP-30's snapshot ask already relies on.
3. **A real session's answer.** The tests route a valid answer to `Sessions::respond` with a
   canned session that has no process, so the call says it has ended. The send itself is the
   Decisions tile's own call, which EP-14 tested against the stub. A deferred run's resume from
   the phone is not run in a test, since it asks the CLI's preflight; it is the tile's Answer and
   resume, reached through the same `Board::resume_deferred`.
4. **The ten seconds.** A form the window has not taken in ten seconds is let go and never run;
   one it took and is still at (a message whose write waits for the process's turn) is said to
   be under way, and the same words are not written twice within five minutes. The hand-over is
   tested on the window's side and the words over HTTP for a form answered at once; no test
   waits the ten seconds through the server.

> **Completion note (2026-11-16; claude-opus-5-5/max; about 1 h 10 min).** A chained headless
> run from the window (ledger row `c304fa80`, from 19:57), the work commit and the record commit
> after it, then the install. No question was put to the owner: the brief kept to its scope,
> added no crate, changed no setting and nothing outward-facing.
>
> **Built.**
>
> 1. **The request channel** (`app::server`, `app::phone`).
>    - `Order` (tick a task, tick a message, send, answer), `AnswerForm`, `PageRequest` with a
>      `oneshot` reply of a `Said`, and `Hand`, which the window and the page each move once.
>    - `Server::take_orders` and `Watch::take_orders`; `Board::obey_pages` in `Board::pass`,
>      shown or hidden, before the queue.
>    - `Shared::order`: the order handed over, at most ten seconds of wait, then at most three
>      more for the snapshot to show a write; `Shared::back`, the 303 with `?done=<n>`;
>      `Outcomes`, five minutes and 32 kept, by number and page, numbered from the process's
>      start.
> 2. **The window's side** (`app::phone`): the ticks through `Board::tick_in` and
>    `Board::tick_message_in`, the message through the compose dialog's write on a worker
>    thread, the answers through `Draft::from_form` and `Draft::response` then
>    `Board::answer_live` or `Board::resume_deferred`. Also: "already answered", a chat's left
>    out, a revert open there refusing the writes, the same words refused within five minutes,
>    a screenshot run refusing everything, and each outcome said with the caller's identity.
> 3. **The snapshot** (`app::published`): `Parts` (the list, `Project` per project, `Decisions`),
>    compared whole; `sessions::Waiting::at`; `Publisher::refresh` when the files land and after
>    a form ran; `Reader::version` and `Reader::newer_than`.
> 4. **The pages** (`app::pages`): `project`, `decisions`, `note_html` (the Detail pane's blocks
>    with every run escaped), the list's links and its count of the decisions; the templates
>    `project.html`, `decisions.html` and `ask.html`; the stylesheet's forms.
> 5. **The rest.** `app::form` (a body decoded, a segment encoded); the board's refactors
>    returning words (`writable_folder`, `NoResume`); a restart refused while a phone's message
>    is written; the screenshot feature's `pages=` writing every page.
>
> Also: addenda to D-28, D-19 and D-13; the README's words; pickup segments in EP-30b's and
> EP-31's brief notes.
>
> **The review (rule 6).** A subagent was given the diff and this brief, not the session's
> reasoning. It reported after about fifteen minutes with six points:
>
> 1. *Fixed.* **The page after a write could be stale or wait three seconds.** The wait only
>    waited, and a hidden window could build before the files' re-read and not after it. Now
>    the files' landing and every form that ran ask the publisher for a build at once.
> 2. *Fixed.* **A message could be written twice.** The same words to the same project are now
>    refused while being written or within five minutes. A page whose form the window took
>    says it is under way, rather than "most likely nothing was done".
> 3. *Fixed.* **A revert open in the project did not refuse the phone's writes**, though at the
>    desk its dialog is modal. Now the window refuses them, and the page shows the revert's
>    words in place of its forms.
> 4. *Fixed.* **`?done=` numbers restarted at 1** in each process, so a page reloaded after a
>    restart could draw another form's line. Now they start from the process's start, and a
>    line is drawn only on its own page.
> 5. *Partly fixed.* **Tests that missed their claim.**
>    - Fixed: the log's test reads once with no wait; the "no age" test backdates the requests'
>      clocks.
>    - Logged: a real session's answer, and the ten seconds through the server (residual risks
>      3 and 4).
> 6. *Fixed.* **Scope.** The project page shows the list's marks, and a form posted to a project
>    the window does not list keeps its line on the 404. The addenda were written meanwhile.
>
> What it checked and found sound: every `|safe` value is escaped before it is marked; every
> action is a POST behind the guard's origin rule; the redirects are internal paths; the
> refactors keep their return values and their warn, fail and silent cases (one wording
> changed, for an unregistered project); the answers are checked by key and id, chats and
> screenshot runs refused; the snapshot holds no age.
>
> **Verification.**
>
> - **The gate**, run again after the review's fixes:
>   - `cargo test`: 220 in `app` (206 before), 447 in `core` (unchanged) and the CLI's, all
>     passing;
>   - `cargo test --features screenshot`: 228 (214 before);
>   - both clippy runs with `-D warnings` and `cargo fmt --check`: clean.
> - **Screenshot runs with `pages=`**, kept in `target\tmp\ep30a\`, out of git.
>   - Over the fixtures with the canned decisions: two requests waiting, a deferred run, a
>     session waiting in VS Code, and nine project pages.
>   - Over a copy of three projects' roadmaps (this one and two others).
>   - Each page was read as text and its tags checked for balance with the standard library's
>     parser: all balanced, no outside address, no script.
>   - What they show: the notes in blocks with their labels and numbered parts, the open items
>     with their gates, an older form read-only, one project's eighteen uncounted older forms,
>     the inbox counts, the forms' fields and actions, the list's marks.
> - **Rule 3.** The word check over the changed and new files found nothing, and so did the
>   check on the staged diff.
>
> **Owner decisions.** None was put to the owner. The commits are the autopilot's standing yes
> (D-22); the push is the owner's.
>
> **Routine choices (the session's call, logged).**
>
> 1. **The outcome's line is kept by the server**, a number in the redirect. Its words in the
>    query would let a crafted link put words on the owner's page.
> 2. **Forms decoded by hand**, about forty lines with tests. axum's `form` feature and askama's
>    `urlencode` would each be a feature change for so little (D-15 addendum).
> 3. **The window builds the response** from the form against the request it holds, so a request
>    answered at the desk meanwhile is never answered twice.
> 4. **The note's HTML is made in Rust** from `markup::blocks`, every run escaped, and the
>    templates print it with `|safe`; the templates stay plain.
> 5. **No untick from the phone**, which D-28's list does not name; the ticked tasks and
>    messages are not listed on the page.
> 6. **A redirect after a write waits up to three seconds** for the snapshot to change, so the
>    page shows the write.
> 7. **A restart waits** while a message from the phone is being written.
>
> No owner task, and no message for another project. **Next: EP-30b**, the actions and the live
> updates.
