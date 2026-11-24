---
ep: 51
title: The Inbox pane, every project's open messages and the outgoing mail in one place with Compose; the Detail pane ends at Status, with Decisions, House style and Problems folded
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-51 — The Inbox pane; the Detail pane ends at Status

**Size:** M · **Depends on:** EP-50 · **Decisions:** D-13, D-25 (and D-3, D-11, D-29 as standing)

## Context

The owner's request 5 at EP-45, its second half, and their choice (D-25 addendum of
2026-11-22): the mail in and out gets a pane of its own over every project, as the owner tasks
did at EP-50, and the Detail pane, grown too long, ends at Status with Decisions, House style
and Problems folded under it.

**What the code says** (read at pickup).

- **The Detail pane** (`crates/app/src/detail.rs`, `project`) runs the proposals, in progress,
  Next brief, Progress, History, Analytics, Status, EP-50's owner-tasks line, then `inbox` (the
  open messages with a disposition box and Tick, `Action::TickMessage` reading the draft from
  `Board::drafts`; Compose… beside the title, `Action::OpenCompose`; the ticked messages folded),
  `handed_on` (each `Handed on` block of the project's briefs, `row.outgoing`, with the
  mailman's mark from `row.mail`: delivered, waits and why, counted at the first reading, or a
  receiver the registry does not know), then `decisions`, `house_style` and `problems`, each a
  plain section.
- **The mailman's log** is the ledger's `event` rows of kind `mail` (`mailman::EVENT_KIND`),
  which `crw mail` prints; the window holds the latest round's marks (`Mail::marks`) and no
  lines of the log. A `Delivered` mark holds no date.
- **A ticked message's date:** a tick from the tool writes ` **Disposition (<date>):** …` on the
  line and a `tick` row in the ledger; a session's triage writes `**Triaged at EP-n (<date>):**`
  or the same Disposition label into the file and nothing in the ledger. So the disposition's
  date dates both, where the ledger dates only the tool's.
- **What leads to the inbox today:** an urgent message's toast (`Toast::urgent`, its Show
  `Target::Project`, the one use of that target) selects the project and brings the Detail pane
  forward, as the popup's hover says; the tray's left click and its Show (`tray::Order::Show`)
  show the window as it stands, its amber dot marking new badge events, an open inbox message
  among them (`toasts::marks`, `Attention::unseen`). **No key** led to the inbox: Enter brings
  the Detail pane forward, and the status bar's keys line names no pane, as EP-50 found for the
  tasks; so the keys stay, and the Detail pane's new line leads on.
- **The layout:** `Pane::ALL`, `default_layout`, `pane_ui`, `complete` (`layout.rs`); EP-50 put
  the Owner tasks pane behind Detail, where `complete` puts a missing pane in a saved layout.

**Session start (2026-11-24, 11:46 local time).** A headless run the window chained after EP-50
(the rolling log, 11:46:31: "chained EP-51 after EP-50: claude-opus-5-5 at max, the rules'
choice, M's cap of 1 h 30 min"); `crw sessions clauderoadmapwrangler` lists the row (`517c9221`,
headless, EP-51, open), and the prompt is the tool's opener. The field check read, since EP-50's
record commit (11:45:51): `crash.log`, unchanged since the panic of 2026-11-23 04:00 that EP-43a
read; the rolling log, EP-50's install restart refused while it worked, its finish, the restart
into the 11:42 build at the gate's moment carrying the chain and its plan, and this start;
`crw notifications` and `crw autopilot` (this chain's mark at 70 % of the session window, the
owner's choice in its plan): nothing odd. No inbox message is open; OT-31 (the phone picture) is
open and gates nothing.

**Against the cap.** M's cap is 1 h 30 min of working time from 11:46, so about 13:16, and the
reading took to about 11:50. The row holds five parts: the pane with its two toggles and the outgoing
mail and the mailman's lines; the Detail pane's line and its three folds; the routes into the
pane; IN-15's flag of a private word in a public project's message, in the pane and in `crw
tasks` (a word list read with the files, a hosting class the CLI does not compute today); and
EP-50's addendum, the long items folded to their first sentence with "more…" in both panes
(the shared renderer, measured per item). With the review an M asks for, the screenshot runs
over the fixtures and a copy, the release build, the records and the install, all five do not
fit. **Judged at about 11:50: split.** This session builds the first three; the flag and the fold go
to a new row, **EP-51a (S)**, proposed in the completion note and put to the owner (D-4
addendum; the addendum's own words allow the fold's split).

## In scope

1. **The pane:** `Pane::Inbox`, "Inbox" (the tab says "Inbox · N" while N messages are open in
   any project), a tab behind Detail beside Owner tasks in the default layout, where `complete`
   puts it in a saved one; a new module `crates/app/src/mailbox.rs`. Every project's open
   messages grouped by receiver in the list's order (its groups and sort key, its filters left
   aside, as EP-50's pane), each group's header its name (a click selects it), its state chip,
   its counts and **Compose…** with the project filled in; each message with its id, an urgent
   chip, from whom and when, its text set by `markup` as the Detail pane set it, and its
   disposition box and Tick (`Action::TickMessage`, the drafts as before, today's refusals).
2. **Two toggles saved with the view,** as EP-50's: **This project only** (the selected project,
   whatever it holds, so Compose… reaches any project) and **Ticked in the last week** (the
   messages ticked whose disposition is dated within seven days, each with its disposition; a
   tick written without a dated disposition is not shown, which the hover says; no Untick, as
   the Detail pane had none for a message).
3. **The outgoing mail** under the messages: each sender's `Handed on` blocks in the list's
   order with the mailman's mark, as `detail::handed_on` drew them (moved here). Those that wait,
   are not read yet or name an unknown receiver show; the delivered and the counted fold shut
   under one header with their count. *(Corrected at the run over the copy: a block the mailman
   has not read yet, which is every block before a window's first round, folds under a header of
   its own.)* **The mailman's last lines:** the ledger's latest `mail` rows, read at most once a
   minute and again after a round, in a fold shut by default whose header says when the newest
   was written. *(Corrected as it was written: the header names the newest line's time, not its
   words, since a folding header's text never wraps, EP-13.)*
4. **The Detail pane:** `inbox` and `handed_on` leave it; in their place one line under EP-50's,
   how many messages are open (and urgent) and how many letters wait, with **Open in Inbox**
   (selects the project, sets This project only, brings the pane forward) and Compose…. After
   Status the three headers, **Decisions**, **House style** and **Problems**, each shown when it
   has something to say, as today; Decisions and House style shut by default, Problems open by
   default (it shows only when the project has a problem); the fold remembered per header in the
   view, saved with the layout.
5. **The routes:** an urgent message's Show selects the project and brings the Inbox pane
   forward on it alone; the tray's Show while an inbox message is among the new badge events
   brings the Inbox pane forward; the popup's hover says so.
6. **Screenshot runs** (rule 5): the setup's `tile=inbox`, `inbox-one`, `inbox-ticked`; runs of
   the pane at 1600 × 1000, 2300 × 1000 and 1100 × 900 and of the Detail pane with its folds,
   and one over a copy of the real portfolio's roadmaps, each looked at.
7. **The records:** the README's paragraph on the panes; D-13 and D-25 addenda; EP-51a proposed.

## Out of scope

- **EP-51a (proposed):** IN-15's amber flag on an open or ticked message of a public project
  whose words hit the registry's `private_words`, in the pane and in `crw tasks`; EP-50's
  addendum, an item longer than about six lines folded to its first sentence with "more…" in the
  Owner tasks and Inbox panes, the fold remembered per item for the window's life.
- The Autopilot pane (EP-52); the phone's pages, which keep their lists.
- Unticking a message, or rewriting its disposition (no such write exists; D-13).
- A key of its own for the pane: none led to the inbox (above).

## Verification / acceptance

- Unit tests: the pane's groups in the list's order with a filter set, This project only, a
  group's Compose… and a message's Tick as actions; the ticked of the week by the disposition's
  date; the outgoing mail split into in flight and delivered; the mailman's lines read from a
  scratch ledger; the Detail pane's line and Open in Inbox; the three folds, their defaults and
  the view's memory; the routes (the urgent toast's target, the tray's Show with an inbox mark);
  the layout's default and a layout saved without the pane.
- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings`, `cargo
  fmt --check`; a review subagent given the diff (an M brief), launched once the code compiles.
- Screenshot runs as in scope (6), kept in the git-ignored `target\ep51-shots`; the copy and its
  pictures never enter git, and this brief names what was checked, never the projects (rule 3).
- The release build, and the install last.
- No tracked file names a managed project, the owner's login or the machine (rule 3); the word
  check prints nothing on the staged diff.

**Residual risk.** The pictures are the app's own frames of the fake portfolio and of a copy of
the real roadmaps; no screenshot run presses Tick, Compose…, a fold's header or a Show, so a tick
from the pane and its refusals (a revert open, a session working there, a data repository), the
compose dialog opened from a receiver, a header's fold kept across a restart, and an urgent
message's Show are exercised by unit tests and by the action paths the Detail pane and the popup
used, not by the owner's hand. The tray's Show is tested through `Board::obey` and the layout,
not through the real tray icon; the popup's Show for an urgent message is driven by no test, and
runs `Action::OpenInbox`, which is tested. The mailman's lines read the real ledger's `mail` rows, which the
tests write into a scratch ledger; a screenshot run makes no round, so the real letters' states
are seen first in the window. The owner's saved layout gains the Inbox pane behind Detail at the
first start of the new build (tested on layouts saved without it), and the saved view gains its
folds empty, so the decision log and the house style are shut there at first. The Detail pane's
lines below Status lie beyond the pictures' height for a project with a long Status, and were
looked at on the one fixture short enough (atlas), shut and open.

> **Completion note (2026-11-24; claude-opus-5-5/max, a headless run the window chained after
> EP-50; about 40 minutes of working time from the start at 11:46 to the work commit, the
> owner's one question answered within a minute).**
> Chained by the window after EP-50 at 11:46 (ledger row `517c9221`) under M's cap of 1 h 30 min
> of working time. Judged at the brief (about 11:50) not to fit whole, so it was split there: the
> pane, the Detail pane's end and the routes into the pane were built here; IN-15's flag and the
> long items' fold went to EP-51a. The code compiled at the first build at 12:00, the review ran
> in the background from then, the screenshot series and the runs over the copy by 12:10, the
> owner's question at 12:12, the first full checks by 12:13, the review back at 12:15 and its
> fixes by 12:20, then the checks again and the records. The work commit `a8b9d8b` at 12:26,
> the record commit, then the install.
>
> **Owner decision** (one `AskUserQuestion` at 12:12, answered with the recommended option).
>
> 1. **Where the split's rest goes:** a new row, EP-51a (S), right after EP-51 in P6 and
>    depending on it, holding IN-15's flag of a private word in a public project's message (in
>    the pane and in `crw tasks`) and EP-50's addendum, the long items folded to their first
>    sentence with "more…" in both panes; its brief notes carry both, and EP-50's addendum left
>    EP-51's notes. Not chosen: both parts as an addendum to EP-52's notes; EP-51a for the fold
>    alone, IN-15's flag parked in `v2-notes.md`.
>
> The commits are the autopilot's standing yes (D-22); the push is the owner's.
>
> **Built.**
>
> 1. **The pane** (`crates/app/src/mailbox.rs`; `Pane::Inbox` in `layout.rs`, a tab behind
>    Detail beside Owner tasks; `View::inbox_one` and `inbox_ticked`; `Board::mail_log`): every
>    project's open messages grouped by receiver in the list's order with its filters left
>    aside, each receiver with its state chip, its counts and Compose…, each message with its
>    id, urgent chip, sender and date, its text, its disposition box and Tick; the ticked of the
>    last week, dated by their dispositions; on one project alone, every message it ticked,
>    folded; the outgoing mail in three standings (shown: it waits or names an unknown receiver;
>    folded: not read by the mailman yet; folded: delivered or counted), a block with no words
>    left out as the mailman leaves it; the mailman's last eight lines from the ledger, in a
>    fold.
> 2. **The Detail pane** (`detail.rs`): `inbox` and `handed_on` gone for
>    `mailbox::detail_line` (Open in Inbox, `Action::OpenInbox`, and Compose…); the decision
>    log, the house style and the problems under `fold`, each kept in `View::folds` by its name
>    (`detail::FOLDS`: shut, shut, open).
> 3. **The routes:** `Target::Inbox` in place of `Target::Project` for an urgent message's Show,
>    its popup hover saying the Inbox pane, its phone link still the project's page; the tray's
>    Show (`Board::obey`, `Order::Show`) brings the pane forward on every project while a new
>    inbox message is among `Attention::unseen`; a round's end marks the mailman's lines stale
>    (`mail.rs`). The restart's refusal (`restart.rs`) counts a typed disposition only while the
>    files do not say its message is closed (`mailbox::closed_message`), and the drafts of
>    messages no longer open go when the files land (`Board::prune_drafts`).
> 4. **The screenshot feature:** `tile=inbox`, `inbox-one`, `inbox-ticked`, `folds-open`.
> 5. **The records:** the README's paragraphs on the panes and on the inbox; D-13 and D-25
>    addenda; the roadmap's EP-51a row, its notes, and a paragraph on the split.
>
> **Verification.** `cargo test`: 308 in `app`, 479 in `core`, the CLI's and the integration
> suites, 848 in all, passing; `cargo test --features screenshot`: 316 in `app`; both clippy
> runs with `-D warnings` and `cargo fmt --check` clean, all on the final tree. The new tests: the receivers in the list's order whatever its
> filters, This project only, and Open in Inbox; a receiver's block (the urgent chip and its
> sender, the ticked message left out, its name selecting it, Compose…, Tick held until a
> disposition is typed and then queued); the ticked of the week by the disposition's date, the
> tool's label and a session's alike; the outgoing mail's three standings, the unread and the
> delivered folded; the mailman's lines from a scratch ledger, read again only when stale or a
> minute old; the Detail line's words and buttons; the folds, their defaults and the view's
> memory, the House style test opening its header; the tray's Show through `obey` and the
> layout, with and without a new message; the default layout and layouts saved before EP-50 and
> before EP-51. Screenshot runs, kept in the git-ignored `target\ep51-shots`, each looked at:
> the pane at 1600 × 1000 in Dark; at 2300 × 1000 in Light with Ticked in the last week; one
> project alone at 1100 × 900; the Detail pane tall on a long project (its tail beyond the frame,
> as EP-50 found) and on the short fixture with its folds shut and open. Over a copy of the real
> portfolio's twelve non-data projects (`scripts\copy-roadmap.ps1`, one at a time, into a scratch
> folder outside the repository, removed after with its run's home and pictures): the pane at
> 1600 and one project alone at 1100. The copy held six open messages in five receivers;
> messages of three or four paragraphs filled most of the pane's height, which EP-51a's fold
> answers; nothing was drawn wider than its pane. The release build passed; the word check
> printed nothing.
>
> **Review** (a subagent given the diff and the brief, about 14 minutes from 12:01). Seven
> points:
>
> 1. **The tray's Show** brought the pane forward still narrowed to one project when an earlier
>    Open in Inbox or urgent message's Show had set This project only, so a new message
>    elsewhere was not on screen: that route now clears This project only; tested.
> 2. **A disposition typed for a message ticked elsewhere** (the phone, `crw`, a session's
>    triage) stayed in the window's drafts and refused File > Install and restart and the gate's
>    restart with no box left to clear, a gap older than this brief that one pane of every
>    project's boxes makes easier to meet: the refusal counts only a draft whose message is still
>    open, and the drafts of messages no longer open go when the files land; tested.
> 3. **The Detail pane's fold of every ticked message** was lost, so a message ticked more than a
>    week ago, or without a dated disposition, showed nowhere: the pane on one project (This
>    project only) now folds every message it ticked, with its disposition; tested and pictured.
> 4. **The brief against the code:** a block the mailman has not read yet folds rather than
>    shows, and the Detail line counts it only after the next round, within five minutes
>    (logged, kept); the mailman's fold names its newest line's time (the brief corrected); a
>    block with no words after its label, which the mailman never sends nor marks, would have
>    stayed "not read yet" for good, and is now left out, as the mailman leaves it.
> 5. **Words:** "No open message in any project." was said twice when nothing is open, and the
>    body now says only what the line above does not; a selected project whose folder is missing
>    now says so; an empty inbox read "No open message of 0" and now reads "No message yet".
> 6. **The week's window** runs from the date seven days back, about the Owner tasks pane's 168
>    hours, and a disposition dated in the future shows: left as it is.
> 7. **Tests:** the pane drawn whole (its receivers, the outgoing header, This project only with
>    and without a selection, the history fold), Open in Inbox bringing the pane forward through
>    the layout, and the tray's route from a narrowed pane: added. The popup's `Target::Inbox`
>    arm is driven by no test (it runs the tested `Action::OpenInbox`), which the residual risk
>    names.
>
> **Session's calls.**
>
> 1. The module named `mailbox.rs`, beside the mailman's `mail.rs` (rule 12: by function).
> 2. A ticked message dated by its disposition, not by the ledger's ticks, so that a session's
>    triage counts as the tool's tick does; the switch's hover says a message ticked without a
>    dated disposition is not shown.
> 3. The problems' header open by default, its fold kept by name like the others, so a header
>    the owner shut stays shut until opened: the brief's reading of "opening itself".
> 4. The mailman's fold names its newest line's time, not its words (EP-13: a folding header's
>    text never wraps).
> 5. No key led to the Detail pane's inbox, so the keys and the status bar's keys line stay.
> 6. A block the mailman has not read yet folds under its own header, found at the run over the
>    copy: without marks, as in a window before its first round, every block ever handed on
>    showed as undelivered.
> 7. The brief's times corrected from its file's timestamp (written at 11:51; the first draft
>    put the judgement at 12:08).
>
> No owner task and no message to another project.
