---
ep: 20a
title: Urgent and ntfy
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-20a — Urgent and ntfy

**Size:** M · **Depends on:** EP-20 · **Blocks:** EP-20c (the compose dialog's urgent box), EP-23

## Context

D-21 planned three channels for what needs the owner: the window's popup (built at EP-15 in
place of Windows toasts), the tray's badges, and **ntfy over the tailnet** for the phone, which
waited for the inbox brief. D-13 adds an `urgent` flag on a message that notifies the owner, not
the session. EP-20 built the mailman and split the owner's side of the mail off to this row; at
this brief's pickup the owner split it again (below), so this brief is the notifications half:
the popup's text for a finished run, the urgent message's popup and chip, and the ntfy publisher.

**What exists.**

- `crw_core::notify` keeps the rule table (`Kind::channel`: toasts for a wait, an end, a failure,
  a usage mark and a chain's moment; badges for the rest), the quiet hours and the log's words
  (`notify::line`, `Fate`).
- `app::toasts` turns the moments into `Toast`s and `Toasts::deliver` shows them in the popup or
  holds them back (the quiet hours, the window in front, a repeat), each logged as an `event` row.
  The badge events (`toasts::marks`, `Attention`) include each open inbox message, new until the
  window is next in front, the first reading taken as seen.
- A finished run's toast takes its text from `sessions::ended_line`, which is the result's last
  line with anything in it (`last_line`): in the field a list item, a link or a bold-labelled
  paragraph (below).
- `inbox::parse` sets `Message::urgent` for `[urgent]` anywhere in a message's text; the Detail
  pane draws a red `urgent` chip beside an open message's id and still shows `[urgent]` in its
  text, while an owner item's `[before EP-m]` is dropped from its text when its chip says it
  (`detail::without_gate`).
- The meter's one request goes through WinHTTP (`usage::http::get`, D-14's EP-17 addendum): a GET
  with a header block held as a `Secret`, tested against a loopback listener.
- The registry's `[settings]` take `crw setting <key> <value>` through `Settings::KEYS`, `get` and
  `Edit::set_setting`, which refuses an unknown key (and a build that does not know a key refuses
  the registry, `deny_unknown_fields`).

**Session start (2026-11-10, 17:44 local time).** Launched from the window as a **headless** run
from the launch dialog (rule 11): ledger row `b8ab9279`, headless, EP-20a, `claude-opus-5-5/max`,
open (`crw sessions clauderoadmapwrangler`). No chain started it: the chain log ends at 17:39
("the chain ends after EP-22a: 2 chained runs today in the project, its cap is 2"), after which
the owner raised the project's cap to 8 and pressed Start. The autopilot is on, so the run is
under M's cap of 90 minutes of working time (`board.rs`, `Run::capped`). The prompt is the
window's opener in its headless wording. The tree is clean at `2a011c1`; the inbox has no open
message; OT-21 and OT-23 are open and not gated.

**Field check.** Read back to EP-22a's record commit (`2a011c1`, 17:39):

- `crash.log`: last written 2026-11-03, seen before.
- `crw notifications`: EP-22a's finished popup (17:39) said `**Not exercised.** No session has
  yet started with the flag. …`, EP-22's (17:06) `- **The window is still on an older build.** …`
  and EP-21's (14:37) `- **Push:** four unpushed commits, …`: the last line of each result, as
  part (1) below describes. Nothing else odd.
- `crw autopilot`: this project on, `auto`, briefs up to M, 8 chained runs a day; one other
  project on; not paused.
- `crw mail`: only the fifteen first readings of 13:53, 0 blocks each (no project hands mail on in
  the D-13 form yet).
- `crw sessions`: this run's row; EP-22a's completed.
- `crw doctor`: the installed copy built 17:19 (EP-22a's), its plugin copy matching, the hooks
  running it. **But the window runs EP-22's build**: its process started at 17:06:40, the restart
  into EP-22's build, and EP-22a's install at 17:39 was refused while EP-22a worked; the chain's
  end at 17:39 is no gate's moment for a restart (EP-37a: never at a chain's end), and the owner
  started this run from that window. So this run has no `--plugin-dir`, and its session lists no
  `roadmap-wrangler:` skills: EP-22a's residual risk 1 is not exercised by it. Not S-sized for
  this brief; parked in EP-23's notes (a hand start while a newer build is installed could offer
  File > Install and restart first), and the next run after a restart exercises the flag.

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-10)

1. **Split** (D-4): the brief note holds about two M's with the five mailman limitations EP-20's
   review parked here, and a headless run stopped at its cap leaves its work uncommitted.
   - **EP-20a** (this brief): the finished popup's gist, the `[urgent]` chip text, the urgent
     message's popup and the ntfy publisher.
   - **EP-20c** (M, a new row right after EP-20a): the compose dialog (target, text, urgent,
     written `from the owner`) and the review's limitations (a) … (e).
2. **The gist is the result's first prose paragraph's first sentence** (reversing the choice made
   at EP-19b's field check, the last paragraph's): on the ten latest real headless results in the
   ledger, four of this project and six of another, the first paragraph's first sentence is the
   run's own summary in all ten ("EP-17 is done and committed (…)"), while the last paragraph's
   gives lines such as "No new owner task." or "Install: it ran last.".
3. **ntfy carries what the popup shows**: every toast the popup shows (a run waiting, finished or
   failed, a usage mark, a chain's moment, an urgent message), held back exactly as the popup is
   (the quiet hours; nothing while the window is in front with the focus); priority `urgent` for
   an urgent message, `high` for a wait, a failure or a usage mark, `default` for the rest; File >
   Send a test notification pushes too.

## In scope

1. **The gist** (`markup::gist`): a run's result text reduced to the first sentence of its first
   prose paragraph, a list item, a bold heading alone on its line and a line of links skipped,
   the markdown dropped (bold and code marks gone, a link kept as its text), cut to the toast's
   length. A finished run's toast says it (`sessions::ended_line`), and so do the ledger card's
   result line and the ended tab's line, which read the same text.
2. **The `[urgent]` chip text**: an open message's trailing `[urgent]` is dropped from its text in
   the Detail pane while its chip says it, as an owner item's gate is; an `[urgent]` inside the
   text stays as written.
3. **The urgent message's popup**: `notify::Kind::Urgent`, one more toast in the rule table. A
   message flagged `[urgent]` that arrives open in a registered project's inbox and is not the
   owner's own (`from the owner`) makes a toast once ("harbor: IN-4 is urgent", from whom and the
   text), its Show bringing the window forward on the project's Detail pane, where its inbox is.
   The window's first reading is taken as seen, as the badges' is. The toast is logged and held
   back like any other.
4. **The ntfy publisher** (`crw_core::notify::ntfy`): `ntfy_host` and `ntfy_topic` in the
   registry's `[settings]` (`crw setting`, `none` takes one out), off while either is missing; the
   host a name or an address with an optional port, `https://` unless written `http://`; the
   topic as ntfy takes it (`[-_A-Za-z0-9]`, up to 64). A push is a POST of the toast's text (its
   title when it has none) to `<host>/<topic>` with `Title` (RFC 2047 when it is not ASCII) and
   `Priority` headers, through WinHTTP: `usage::http` gains a request with a method and a body,
   the meter's GET unchanged. The window pushes each toast the popup shows (decision 3) on a
   worker thread of its own, one at a time, and logs each push as a `notification` row
   (`ntfy` or `ntfy failed` with the reason); the topic is never logged.
5. **The records**: a D-21 addendum; the README's command notes; the roadmap's EP-20a row and
   EP-20c's new row with its notes (the compose part and (a) … (e) moved there), EP-23's row and
   its notes (the field check); an owner task for the first real push.

## Out of scope

- **EP-20c**: the compose dialog and the mailman's limitations (a) … (e).
- **EP-20b**: the `@<project>` prompt hook.
- An ntfy access token or user: the tailnet is the boundary (D-21); a setting can join later.
- ntfy for the badges, and a window control for the two settings (`crw setting` sets them).
- Delivering urgent messages to the receiving session (D-13: the owner is notified, not the
  session).

## Verification / acceptance

- **Tests:** the gist over results of the shapes the ledger holds (a summary first, then labelled
  paragraphs, lists, a sources list of links, a heading alone) and its fallbacks; the chip text;
  the urgent toasts (the first reading settled, a new urgent arrival toasting once, the owner's
  own, a ticked one and a plain one not); the rule table with `urgent`; the ntfy settings
  (missing, host forms, refused topics and hosts), the title's encoding, the priorities, a push to
  a loopback listener (method, path, headers, body) and a refusing answer; the registry's two
  keys set, read back and taken out.
- **The checks:** `cargo test` with and without `--features screenshot`, both clippy runs,
  `cargo fmt --check`, and a review of the diff by a subagent given only the diff (rule 6).
- **A screenshot run** of the popup with an urgent toast and of the Detail pane's urgent message
  (the chip, the text without `[urgent]`) on the fixtures.
- **The release:** `cargo build --release` with the window up, then `pwsh -File
  scripts\install-shortcut.ps1` (rule 6); its restart request is refused while this run works.
- **Rule 3's word check** on the staged diff prints nothing.
- **Residual risk.** The session never ran the window on the owner's files or reached a real
  ntfy server (rules 5 and 13): the push went to a listener on this machine over plain HTTP, and
  the toasts were made from the fake portfolio and synthetic lines. Three things are not
  exercised:
  1. **A push over HTTPS to the owner's server on the tailnet.** The request is the meter's
     WinHTTP route with a method and a body, the meter's GET over HTTPS working at every poll
     since EP-17, and the headers are the ones ntfy's publishing page names (`Title`, RFC 2047
     beyond ASCII; `Priority` by name). The owner's first push is OT-24; a failure is a `ntfy
     failed` line in `crw notifications` with WinHTTP's reason, and the popup is unaffected.
  2. **A real urgent message.** No project has handed one on yet. The tracker reads the same
     `inbox::Message::urgent` the Detail pane's chip has shown since EP-5, after each files read,
     with the badges' cadence; the walk (EP-23) reads the first one in `crw notifications`.
  3. **The gist on the next real results.** The rule was checked on the ledger's ten latest
     results before it was built, and its test holds their shape; the first week's review reads
     the finished popups in `crw notifications`.

> **Completion note (2026-11-10; claude-opus-5-5/max; about 1 h 10 min of wall time, three
> questions at pickup and two in the closing round).** One headless run launched from
> the window's launch dialog (ledger row `b8ab9279`, headless, EP-20a), from 17:44 local time,
> under M's cap of 90 minutes of working time.
>
> **Built.**
>
> - `markup::gist`: the first sentence of a result's first prose paragraph, with list items,
>   headings, rules, a bold label alone, lines of links, tables, fenced code and an intro to a list
>   skipped and the markdown dropped; a first sentence of three words or fewer takes the next one,
>   and a result with no prose says a list's first item, else a heading. `sessions::result_line`
>   gives it to a finished run's toast, the Sessions pane's ledger card and an ended tab's line;
>   the live line stays the last line (`last_line`).
> - `detail::without_urgent`: an open message's trailing `[urgent]` left to its chip, which gained
>   a hover text.
> - `notify::Kind::Urgent` in the rule table, `Kind::priority`, `Fate::Pushed` and
>   `Fate::PushFailed`; `Toast::urgent` with `Target::Project` (its Show opens the project's
>   Detail pane), `urgent_messages` and `urgent_arrivals` (the first look taken as seen, the
>   owner's own messages left out), `Toasts::urgent` in the badges' pass; the popup's red dot.
> - `crw_core::notify::ntfy`: `Ntfy` from the settings (the server checked, the topic in ntfy's
>   form), `Push`, `Priority`, `publish` (a POST with `Title`, an RFC 2047 word beyond ASCII, and
>   `Priority` by name; the text, or the title when there is none, as the body); `usage::http::send`
>   with a method and a body, the meter's `get` one of its calls.
> - The registry's `ntfy_host` and `ntfy_topic` (`Settings::ntfy`; `crw setting`, `none` taking one
>   out, its reply saying whether the publisher is on).
> - The window: each toast the popup shows pushed on a thread of its own (`toasts::Pusher`), one
>   at a time, each push a `notification` row (`ntfy`, or `ntfy failed` with the reason), the topic
>   never logged; File > Send a test notification pushes too; the status bar's hover names the
>   server and the last push.
> - Tests: the gist over a result of the ledger's shape and its fallbacks, the chip text, the
>   urgent arrivals and the fake portfolio's urgent message, a push's priority, the ntfy settings,
>   the title's encoding and base64, a POST to a loopback listener through WinHTTP and a refusing
>   answer, the registry's two keys.
> - The records: the D-21 addendum, the README, the roadmap's EP-20a and EP-20c rows and notes,
>   the ordering note, EP-23's row and note, OT-24.
>
> **The review (rule 6).** A subagent given only the code diff found nothing serious in the WinHTTP
> call, the threading, header injection or panics, and twelve points. Fixed and tested:
>
> 1. **An urgent letter swallowed at a start.** The urgent messages' first look waited for the git
>    and GitHub passes, while the mailman's first round needs only the files read, so a letter a
>    new window delivered during its git pass (after a restart at the gate's moment) was taken as
>    seen. The look now follows the files read alone, once a second at most, and the mailman's
>    first round waits for it.
> 2. **The gist's reading of markdown**: the fence state was lost after a list item, rules (`---`,
>    `===` under a heading) read as prose, and an empty quote stopped the search.
> 3. **The gist's fallback** kept heading marks and fence backticks; it now says a list's first
>    item, else a heading, else any line, outside fenced code.
> 4. **The gist's choices**: a bold sentence alone (`**EP-7 is done and committed.**`) was skipped
>    as a label, and an intro ending in a colon before a list became the gist; the join now takes
>    any first sentence of three words or fewer (`Done.`), bold or not.
> 5. **The topic in the ledger**: a hand-written topic that did not read was quoted in every `ntfy
>    failed` line. The refusal no longer quotes the value (only `crw setting`'s does, to the owner
>    who typed it), and the publisher's `Debug` names the server alone.
> 6. **A redirect counted as a push**: WinHTTP follows a redirect and sends the POST again as a GET,
>    which a web page answers with 200. A push now counts only when the server answers as ntfy does,
>    with a JSON object.
> 7. **A passing read error re-toasted** a project's urgent messages; a project whose inbox did not
>    read keeps what the last look saw there.
> 8. **Two tests that could not fail**: the chip text's char-boundary guard and the default
>    priority's header now have cases that fail without them.
> 9. **Nits**: the pushes' thread is named, says a ledger error in the status bar's hover and starts
>    again after an end; the test's notice no longer says the push went before it did; `none` is
>    read in any case; port 0 is refused and `_` taken in a host name; `without_urgent` moved to
>    `crw_core::inbox`; `header_value` is private.
>
> Logged, not fixed (each rare or by the owner's rules): an urgent letter the old window delivers
> in the seconds between a run's end and a restart at the gate's moment is taken as seen by the new
> window (the seen set is not in `handover.json`); the quiet hours drop an urgent toast and its push
> rather than defer them, as the popup's rules do (decision 3); the gist is worked out each frame
> for each ledger card shown, cheap since it stops at the first prose paragraph; an IPv6 address
> without a port is refused as a host; a message that reuses an id already seen never toasts.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed 530, and with `--features screenshot` 538: EP-22a's 521 and
>   529 plus 9 new in each. Both clippy runs and `cargo fmt --check` are clean.
> - **Screenshot runs** (the fixtures, a scratch home, 1600 wide): the popup with "harbor: IN-4 is
>   urgent" at its head in red, its text without the flag; lantern's Detail pane with the urgent
>   chip and the message ending without `[urgent]`, in both themes. Kept in the session's
>   scratchpad.
> - **The release.** Built with the window up; the install ran last, after the two commits.
> - **Rule 3's word check** found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-10).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Splitting EP-20a at pickup (about two M's under M's cap) | Notifications here | EP-20a the gist, the chip text, the urgent popup and ntfy; EP-20c (M) after it the compose dialog and (a) … (e) |
> | 2 | The gist's rule, on the ledger's ten real results | The first paragraph | Its first prose paragraph's first sentence, reversing EP-19b's choice of the last |
> | 3 | Which moments reach the phone | What the popup shows | Pushes held back as the popup is; `urgent`, `high` or `default` by kind |
> | 4 | Whether an urgent message's push breaks through the quiet hours (closing round, from the review) | Held back like the rest | As built; a send at the hours' end went to the v2 notes |
> | 5 | Where the review's four unfixed points go (closing round) | The handover point to EP-20c, the rest logged | EP-20c's brief notes gained it; the other three stay in this note |
> | 6 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The gist lives in `markup`**, beside the sentence splitter and the spans it uses, and the
>    ledger card and the ended tab say it too, so the window says one thing for a result.
> 2. **A short first sentence takes the next** ("Done. EP-5 is committed …", "Not exercised. No
>    session has …"), as the review's case of a bold label widened; a longer one stands alone.
> 3. **The urgent toast reads the inboxes after each files read**, not the mailman's deliveries,
>    so a message any writer leaves counts; the owner's own (`is_owner` on the sender) makes none,
>    since the flag tells the owner what others send (D-13). A push of it is ntfy's `urgent`, but
>    the quiet hours hold it back like any toast (decision 3).
> 4. **Its Show opens the project's Detail pane**, where the inbox is, and its dot is the red of
>    the Detail pane's urgent chip: no new hue.
> 5. **ntfy's header form** on `<host>/<topic>`, as the note planned, with RFC 2047 for a title
>    beyond ASCII (ntfy's publishing page, read this session), not its JSON form; no tags.
> 6. **The host may say `http://`**, for a server on the tailnet without TLS. `crw setting`
>    refuses a host or a topic that does not read; a hand-written one that does not read leaves
>    the registry readable and is said at each push as `ntfy failed`.
> 7. **One thread of the window's sends the pushes**, so a slow or absent server never holds the
>    window; the tool's word about the tray is not pushed.
> 8. **One owner task**, the first real push (OT-24), the route being new and outward; the urgent
>    popup and the gist are looks at a changed surface, named in the residual-risk paragraph for
>    the walk (D-10 addendum).
>
> No messages for other projects. OT-24 is the owner's and not gated: no brief builds on it.
> **Next: EP-20c**, which a chain may run if the gate passes.
