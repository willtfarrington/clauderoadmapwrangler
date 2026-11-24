---
ep: 51a
title: A private word flagged and a long item folded
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-51a — A private word flagged and a long item folded

**Size:** S · **Depends on:** EP-51 · **Decisions:** D-3, D-25 (and D-13, D-29 as standing)

## Context

The two parts EP-51's split left (2026-11-24, the owner's choice there).

1. **IN-15's flag** (from EP-45, the owner's choice): the mailman once wrote a private project's
   name into this public repository's inbox, which rule 3's word check caught only at the commit.
   The Inbox pane flags such a message as it arrives, so it is reworded before a commit carries it,
   and `crw tasks` says the same.
2. **The long items' fold** (EP-50's addendum, the owner's choice there): over the copy of the real
   roadmaps one owner task, a single `owner-tasks.md` line of several paragraphs, filled most of
   the Owner tasks pane's height at 1600 × 1000, and messages of three or four paragraphs filled
   most of the Inbox pane's at EP-51's run.

**What the code says** (read at pickup).

- **The word list:** `crw_core::wordcheck::List` (EP-42), its `Debug` the count alone, `hits` a
  case-blind fixed-string match; the registry's `private_words()` names the file (relative to the
  data root). Make public's dialog (`act.rs`) and `crw act … make-public` read it; nothing else does.
- **The hosting class:** `Hosting::class` (`crates/core/src/badges/hosting.rs`): `Public` for a
  listed public repository, `Private` for a listed private or internal one, `Remote` for a GitHub
  remote the account's list does not match (the list not read yet, another owner's, or missing) or a
  remote elsewhere, `Local` without a remote. The window classifies each row in
  `Online::apply` (`portfolio.rs`) from `row.badges.remote` and the account's list (the ledger's
  cache at once, the network daily). The CLI already reads that cache for `crw act`
  (`ledger.cached_one("", online::ACCOUNT)`, twice in `main.rs`), and `git::remote_url` reads a
  folder's origin from `.git/config` with no child.
- **A message** (`crw_core::inbox::Message`) holds `from`, `text` and `disposition` apart; the
  mailman writes the sender into `from`, which is where IN-15's name was.
- **The panes' renderer:** `markup::note` sets an item in blocks (labels, parts, sentence steps)
  at the reading measure; `owner_tasks::owner_item`, `mailbox::message_ui`, `mailbox::letter_ui`
  and `mailbox::every_ticked` draw each item in full through it.

**Session start (2026-11-24, 12:27 local time).** A headless run the window chained after EP-51
(the chain log, 12:28: "chained EP-51a after EP-51: claude-opus-5-5 at max, not in your plan, the
rules' choice, S's cap of 1 h 15 min"); `crw sessions clauderoadmapwrangler` lists the row
(`13c29a8c`, headless, EP-51a, open), and the prompt is the tool's opener. The field check read,
since EP-51's record commit (12:26:43): `crash.log`, unchanged since the panic of 2026-11-23 04:00
that EP-43a read; the rolling log, EP-51's install restart refused while it worked, then the
restart into the 12:26 build at the gate's moment carrying the chain and its plan; `crw
notifications` and `crw autopilot` (this chain's mark at 70 % of the session window, the owner's
choice in its plan; "not in your plan" is the plan naming no model for a row added after it, the
rules choosing): nothing odd. No inbox message is open; OT-31 (the phone picture) is open and gates
nothing.

**Against the cap.** S's cap is 1 h 15 min of working time from 12:27, so about 13:42, and the
reading took to about 12:32. Two parts, each a few functions with their tests, the screenshot runs
over the fixtures and one over a copy, the checks, the release build, the records and the install;
an S brief asks for no review. **Judged at about 12:33: it fits;** no split.

## In scope

1. **The flag, read with the files.** `inbox::Message::hits(&List)`: whether its sender, its text
   or its disposition holds a listed word. Each files pass (`read_files`, and `read_project` for
   one project) reads the registry's list once, and each row keeps the ids of its messages that
   hit (`ProjectRow::word_hits`); the words themselves are never kept on a row, shown, logged or
   counted. A list that is not named or does not read flags nothing.
2. **Which projects:** `Hosting::exposure`, the reason a project's words matter: its remote is
   public (`Class::Public`), or its visibility is not known (`Class::Remote`: a GitHub remote the
   account's list does not match, the list not read yet, or a remote elsewhere), which is checked
   as if public, since a missed flag costs a leak and a false one a glance (the brief's call, as
   its notes left it); a listed private repository and a project without a remote are not checked.
3. **The Inbox pane:** an amber chip "private word" on the message's header line, open
   (`message_ui`), ticked in the last week (`ticked`) and in the fold of every ticked message
   (`every_ticked`), its hover saying why (the reason above) and what to do: reword the line
   before a commit carries it (rule 3, D-3); the word itself is never shown.
4. **`crw tasks`:** a `words` line in the head saying whether the messages were checked (no list
   named, the list did not read, the remote private or none, or how many messages hit and why);
   under each message that hits, a line saying so, as the disposition's line is printed. The
   cached account list is read as `crw act` reads it (one helper for the three uses).
5. **The fold:** `markup::long_note`, used by the four item drawers above: an item whose blocks
   lay out to more than six lines at the pane's width shows its first sentence and a "more…"
   toggle; open, the whole item and a "less" toggle; each item's fold kept in egui's memory for
   the window's life (temporary data, never saved), by project and item id. A text of six lines or
   fewer draws as `note` draws it today.
6. **Screenshot runs** (rule 5): a synthetic word list in the fake registry (`private_words`
   naming a fixture file of invented words), the Inbox pane and the Owner tasks pane at 1600 ×
   1000, a fold opened, and one run over a copy of the real portfolio's roadmaps (no word list
   there), each looked at. *(Corrected at the close: no picture opens a fold, since no setup flag
   presses "more…", and the unit test opens and folds one; the Owner tasks pane was pictured over
   the copy, whose tasks are long, the fake portfolio's being short.)*
7. **The records:** the README's paragraph on the panes; a D-3 addendum (the flag) and a D-25
   addendum (the fold).

## Out of scope

- Flagging owner tasks, briefs or the decision log: the flag is IN-15's, the inbox's.
- Rewording a message from the window: the owner or the next session rewords the line.
- The phone's pages, which keep their lists without the flag or the fold.
- Keeping a fold across restarts: for the window's life, as the notes say.

## Verification / acceptance

- Unit tests: `Message::hits` over sender, text and disposition; `Hosting::exposure` for each
  class; a files pass over a scratch portfolio whose registry names a scratch list giving a row's
  word hits; the chip drawn for a public project's hit and not for a private one's, and its hover;
  `crw tasks`' words line and the message's line through the CLI's own function; the fold's line
  count (a short text unfolded, a long one folded to its first sentence, opened by its memory).
- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings`, `cargo
  fmt --check`.
- Screenshot runs as in scope (6), kept in the git-ignored `target\ep51a-shots`; the copy and its
  pictures never enter git, and this brief names what was checked, never the projects (rule 3).
- The release build, and the install last.
- No tracked file names a managed project, the owner's login or the machine (rule 3); the word
  check prints nothing on the staged diff.

**Residual risk.** The flag's pictures read the fake registry's synthetic list (two invented
names) against the fake portfolio, whose public projects the screenshot feature's canned account
lists; the real window reads the owner's list, which no picture and no test touches, against
remotes the real GitHub pass classifies. So the first real flag, or its absence, is seen in the
window: this repository's own inbox, all of whose messages were reworded at triage, should show
none, and `crw tasks` on it, run once with the release build, says whether the messages were
checked without saying a word. A project whose remote the account's list does not match (another
owner's repository, or a window that has not read the list yet) is checked as if public, so a
private project of that kind may show a flag the owner judges harmless. The fold's line count is
egui's own wrapping of each block at the pane's width, measured in the tests on a bare context and
looked at in the pictures; a pane resized narrower folds more items, and an item's fold lasts the
window's life, not a restart. No screenshot run presses "more…": the pictures show the folded
state, and the opening, "less" and the fold kept per item are exercised by the unit test on a bare
context, not by the owner's hand.

> **Completion note (2026-11-24; claude-opus-5-5/max, a headless run the window chained after
> EP-51; about 30 minutes of working time from the start at 12:27 to the work commit, no question
> asked).**
> Chained by the window after EP-51 at 12:28 (ledger row `13c29a8c`) under S's cap of 1 h 15
> min. Judged at the brief (about 12:33) to fit, so not split. The code compiled clean at 12:36,
> the full tests passed by 12:44, the screenshot-feature tests, both clippy runs and the format
> check by 12:48, the pictures over the fixtures and over a copy by 12:50, the release build at
> 12:50, then the records, the work commit, the record commit and the install.
>
> **Owner decisions.** None was asked: the brief's notes left the one open call (a project of
> unknown visibility) to the brief, and the rest are routine choices, below. The commits are the
> autopilot's standing yes (D-22); the push is the owner's.
>
> **Built.**
>
> 1. **The flag** (IN-15): `inbox::Message::hits` (its sender, text or disposition against a
>    `wordcheck::List`); `Hosting::exposure`, the reason a project is checked (public, or of
>    unknown visibility); each files pass (`portfolio::read_files`, and `read_project` for one
>    project) reads the registry's list once and keeps each row's hits as message ids
>    (`ProjectRow::word_hits`), never the words; the Inbox pane's amber "private word" chip
>    (`mailbox::private_word`, `private_chip`) on an open message, on one ticked in the last
>    week and in the fold of every ticked one, its hover saying why and asking for the line to be
>    reworded before a commit carries it, the word never said.
> 2. **`crw tasks`:** a `words` line in its head (no list named, the list did not read, the remote
>    private or none, or how many messages hit and why) and a `private word:` line under each
>    message that hits; `cached_account`, the window's cached listing read once for the three
>    uses in the CLI; the usage text says so.
> 3. **The fold** (EP-50's addendum): `markup::long_note`, with `lines` (each block's rows as
>    egui wraps it at the pane's width) and `folded` (the first sentence, cut at a word near 280
>    characters); over six lines an item shows its first sentence and a small **more…** button
>    under it, open the whole item and **less**; each item's fold kept in egui's memory for the
>    window's life by project and id. Used by `owner_tasks::owner_item`, `mailbox::message_ui`,
>    `letter_ui` and `every_ticked`; `markup::note` and its other callers draw in full as before.
> 4. **The fixtures:** the fake registry names a synthetic list,
>    `crates/app/fixtures/portfolio/private-words.txt` (two of the fake portfolio's own invented
>    names), so a fake-portfolio picture shows harbor's open IN-2 flagged.
> 5. **The records:** the README's paragraph on the panes; a D-3 addendum (the flag) and a D-25
>    addendum (the fold).
>
> **Verification.** `cargo test`: 311 in `app`, 481 in `core`, the CLI's and the integration
> suites, 854 in all, passing; `cargo test --features screenshot`: 319 in `app`; both clippy runs
> with `-D warnings` (one needless `mut` in the new test fixed) and `cargo fmt --check` clean, all
> on the final tree. The new tests: a message's hits in its sender, text and disposition and not
> in its date or id; the exposure of each hosting class; the fake registry's list giving harbor
> its two hits through a files pass, the chip drawn for a public remote and for an unknown one
> (each reason said, the word never) and not for a private remote or none; `crw tasks`' words line
> in its five cases, the list's size never said; the fold's first sentence and its cut; a long
> item folded, opened by "more…", folded again by "less", another item kept shut. Screenshot
> runs, kept in the git-ignored `target\ep51a-shots`, each looked at: the Inbox pane at 1600 ×
> 1000 in Dark (harbor's IN-2 with the chip; lantern's long message, three sentences and two
> parts, folded to its first sentence) and harbor alone at 1100 × 900 in Light (the chip legible). Over a copy of the real
> portfolio's twelve non-data projects (`scripts\copy-roadmap.ps1`, one at a time, into a scratch
> folder outside the repository, removed after with its run's home and pictures), no word list
> there: the Owner tasks pane, whose four open tasks each folded to one or two lines so that the
> pane fits its height, and the Inbox pane, whose six open messages each folded to a first
> sentence of at most four lines with its ellipsis, every receiver now on screen at 1600 × 1000.
> With the release build, `crw tasks` on this repository read the owner's list in its process and
> said "no message holds a private word; this project's remote is public"; the same read over the
> other projects found three public ones, none of whose messages hits, and the rest private or
> without a remote, so the real window should show no flag today. The release build passed; the
> word check printed nothing.
>
> **Session's calls.**
>
> 1. A project of unknown visibility (a GitHub remote the account's list does not match, that
>    list not read yet, or a remote elsewhere) is checked as if public, its hover saying so: a
>    missed flag costs a leak, a false one a glance.
> 2. A list that is not named or does not read flags nothing in the window, silently; `crw tasks`
>    says which, and Make public's dialog says why a list does not read.
> 3. The sender's parenthesis is read with the text and the disposition, since the mailman writes
>    the sending project's name there, where IN-15's name was; the date and the id are not.
> 4. The chip sits on the message's own line, not on the receiver's.
> 5. The toggle is a small button under the first sentence, as Compose… is: egui's link widget
>    did not take a click in the test harness, and the button's own line keeps the folded label
>    selectable.
> 6. A folded first sentence is cut at a word near 280 characters, so a fold is at most about
>    four lines in a narrow pane.
> 7. An owner task's fold is keyed by its id where it has one (by its line otherwise), so a line
>    added above keeps it; a handed-on block's by its sender, file and line.
> 8. The synthetic list holds two of the fake portfolio's own names, so every picture of the fake
>    inbox shows the flag on harbor's IN-2.
> 9. The brief's times corrected from the clock read at the time (the first draft put the
>    judgement at 12:37).
>
> No owner task and no message to another project.
