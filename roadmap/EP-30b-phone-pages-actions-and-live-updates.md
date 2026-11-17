---
ep: 30b
title: Phone pages, the actions and live updates — start the next brief as the rules resolve it, stop a headless run, pause and resume the autopilot; the usage strip and the autopilot block; one event stream and the tool's own script
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-30b — Phone pages, the actions and live updates

**Size:** M · **Depends on:** EP-30a (the pages that tick and answer), EP-40 (the session window
in the gate and the meter) · **Decisions:** D-28, D-22, D-14 (and D-15, D-29, D-3, D-10 as
standing)

## Context

EP-30 gave the window's server the phone's base (the published snapshot, the guarded `/m/`
router, the remote-action log, the read-only list). EP-30a added a project's page and the
decisions page with their forms, and the request channel every form goes through. This brief
adds the three actions that run things (D-28 addendum of 2026-11-15: start a project's next
brief headless with the model and effort the rules resolve, a chain then following the gate as
on the desktop; stop a headless run; pause and resume the autopilot), the two blocks the owner
asked for on the list page (the usage strip and the autopilot block), and the live updates: one
event stream and the tool's own small script (D-14 addendum). It also takes IN-14's fix, parked
here at EP-30 by the owner's choice: the gate's registry read fails softly today.

**Session start (2026-11-16, 20:48 local time).** A chained headless run from the window: the
chain log's "chained EP-30b after EP-30a: claude-opus-5-5 at max, the rules' choice, M's cap of
1 h 30 min; it holds at 80 % of the session (five-hour) window, your mark for this chain" at
20:48, after the window's restart into EP-30a's build. The prompt is the tool's opener, and `crw
sessions clauderoadmapwrangler` lists the row (`5140bb4c`, headless, EP-30b, claude-opus-5-5/max,
open). No owner task and no inbox message is open. The field check read, since EP-30a's record
commit (2026-11-16, 20:46):

- `crash.log`: the one report of 2026-11-03, nothing newer.
- `crw notifications`: EP-30a's finish at 20:48:03 and this chain's start at 20:48:36.
- `crw sessions`: this row; EP-30a's, completed in 51 min with no question.
- `crw autopilot`: not paused; on for this project and five others, all in mode `auto`; the
  newest lines are this chain's hold for the restart and its start.
- The rolling log: 20:47:14, EP-30a's install asked for a restart, refused while its run worked;
  20:48:03, the run's finish; 20:48:04, the restart into the build of 20:47, carrying one held
  chain and one chain plan, as EP-37a built it. The installed `crw.exe` is still EP-30's (19:55):
  EP-30a changed the window's crate alone, so cargo did not link the CLI again.

Nothing odd to triage.

## In scope

1. **Start from the phone** (D-28, D-22). Headless only, the rules' choice, behind a page of its
   own.
   - `GET /m/p/{project}/start`: the confirmation page. The server cannot read the window, and a
     draft reads the project's `CLAUDE.md`, so the page asks the window for a preview through a
     second channel beside the forms' (`server::StartAsk`, a `oneshot` reply), drained in the
     same pass. The preview (`published::Start`) holds the brief (id, size, title), the model,
     the effort, ultracode and the permission mode the rules resolve with their reasons and the
     downshift's words, what the run is (its cap, its commits, the chain plan when the
     autopilot is on, or "stops before committing" when it is off), the notes the launch
     dialog would show (owner tasks due before the brief, the files' evidence of a session in
     progress, sessions open and idle there, the session window's notice, the gate's standing
     verdict from `Pilot.standing`), and what refuses the start now. A project the window does
     not list is a page not found.
   - `POST /m/p/{project}/start` (`ep`, the brief the page showed): an `Order::Start`, run on
     the window's thread. Refused, in words, for: a screenshot run; the window restarting; the
     folder missing; the registry not reading; nothing to launch (`prompt::draft`'s reason); a
     next brief other than the page's (the page is old); a start already on its way in the
     window; the window's own run or a session seen working or waiting in the project, or a
     chained start on its way there (stricter than the dialog, which only warns: from the
     phone nobody sees the other tab, and the gate refuses the same); a revert open or running
     there; the CLI not answered yet (the preview asks it, so a second press goes); the CLI's
     own refusals (`Preflight::refusals`).
   - The launch is the chained start's (`Draft::for_chain(None, …)`, the prompt worded
     headless with the rules' choice; `autopilot::launch`), with what the dialog's Start
     headless adds: the route, the hooks, the revert's start; the cap, the budget, the push
     guard and a chain plan of the rules' choice for each row only when the project's
     autopilot is on, as the dialog does. A held chain in the project gives way to it
     (`Board::started_by_hand`).
   - The start's result comes back in a later pass (`Sessions::receive`): the page waits for
     it within the form's ten seconds, else says the start is under way, as EP-30a's forms do.
     A start the runner refuses lets the plan go and drops the preflight, as the dialog's does.
     The desk's panes are not moved.
2. **Stop from the phone** (D-28). The project page lists the window's own headless runs there
   (a chat's aside), each with its state and start time, and a Stop form for a run that works or
   waits (`key` and `ep`). `POST /m/p/{project}/stop` is an `Order::Stop`: `Sessions::stop`,
   never Kill. The key must name a run of that project and EP, so a page drawn before the
   window's restart (whose keys start again at 1) stops nothing else. The words say that a
   stopped run ends its chain.
3. **Pause and resume** (D-22). The list page's autopilot block has Pause, or Resume when
   paused; `POST /m/autopilot` (`pause` = `on` or `off`, never a toggle, so an old page cannot
   invert it) is an `Order::Pause`, run in the orders' pass, shown or hidden, through the
   desk's own `pause_autopilot` refactored to return its words. A pause already in force is
   said and not written again.
4. **The two blocks on the list page** (D-28 addendum).
   - The usage strip: the meter's weekly, Fable and session (five-hour) figures with their
     resets, a closed session window said, the degraded marker with its reason.
   - The autopilot block: paused by you, held by usage (`autopilot::usage_pause`, worded as the
     Autopilot section words it), off everywhere or on, with the projects it is on in; the held
     chains with their words; the chain plans with their own marks; the chain log's latest lines
     (`Pilot.log`, read every 30 s by `autopilot::recent`).
   - Ages are drawn when the page is drawn (a reset's countdown, a reading's age), never kept in
     the snapshot.
5. **The steady version** (EP-30's and EP-30a's pickups). The list's badge sentences and the
   last commit's age word carry relative times, which raised the version about once a minute
   with nothing new. The publisher now compares the list without what only the clock changes
   (each badge's sentence, the age word marked as such at its source in `badges::words`, the
   usage figures' read times), so an idle window publishes no new version. The snapshot still
   holds those words as of each build, which a page reads fresh.
6. **One event stream and the tool's own script** (D-14 addendum).
   - `GET /m/events`: a server-sent event stream. Each event (`change`) names a part of the
     window's state by the path of the page that shows it: `/m/` (the list, its two blocks),
     `/m/p/<project>` (a project's page, its runs included), `/m/decisions`. It is driven by the
     snapshot's version: a stream holds the snapshot it last saw, awaits the `watch` channel's
     change, and when the version moved names each part that differs by the steady comparison.
     A first event sets the browser's retry to five seconds; a keep-alive comment keeps a proxy
     from closing an idle stream. No stream outlives the window's quit, and the browser's
     `EventSource` tries again.
   - axum's `Sse` takes a `Stream`. `futures-util` 0.3.34, in the lock as axum's own dependency,
     joins the app crate pinned exactly with no default features, for `stream::unfold` and the
     combinators: no package joins the lock (D-15 addendum).
   - `GET /m/app.js`, the tool's own script (about sixty lines, served from the exe): each page
     names the paths it listens to (`data-listen`); on an event for one of them the script
     fetches its own page again and puts in each part (`data-part`) that differs, leaving alone
     a part that holds the focused field or typed text (and fetching again once the focus
     leaves). After the stream comes back it fetches once, since changes may have passed. A page
     works the same without it.
   - The pages' Content-Security-Policy gains `script-src 'self'` and `connect-src 'self'`.
7. **`crw remote`** prints the remote-action log (`event` rows of kind `remote`), as `crw mail`
   prints the mailman's, `--all` for every row.
8. **IN-14: the gate's registry read** (the owner's choice at EP-30). When the registry does not
   read at the gate after a run (`weigh_run`, for a chained run or a project whose autopilot the
   last read showed on), at a held chain's re-read (`read_held`, silent today) or at a chained
   start (`start_queued`, which kept failing at each pass): the chain is held with words naming
   the failure and File > Install and restart, logged and notified once; and when a newer
   installed build waits (`Restart::pending_install`, over `install::restart_due`), the window
   restarts into it with the chain carried, for the new window to weigh the gate.
9. **The screenshot feature's `pages=`** also writes each project's start page beside the file
   (`<stem>-<project>-start.html`), so the pages are read on real content (rule 5).
10. **The records.** Addenda to D-28 (the actions and the live updates as built), D-22 (the
    phone's start and pause; IN-14), D-14 (the script and the stream) and D-15 (`futures-util`);
    the README's words; this brief and its completion note; IN-14's line in the inbox noted.

## Out of scope

- The sessions' pages (`/m/s/<key>`, a transcript streaming over this stream), the sessions
  list, the notifications page and the doctor's page (EP-31); a session's own event path joins
  the stream there. Enable remote view and the pushes' links (EP-31a).
- A start with another model or effort than the rules', a visual start, the chain plan's
  choices, Resume or Commit from the phone: the desk's (D-28's list names none of them). Kill:
  never from the phone.
- Setting `owner_login` in the owner's registry (EP-31a's Enable).

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings` and
  `cargo fmt --check` pass (rule 6). Being an M brief, the diff is read by a review subagent given
  the diff and this brief, not this session's reasoning, before the release build.
- **Tests:**
  - the steady version: a second build with only the clock moved (badge sentences, the age
    word, a reading's time) keeps the version, a real change raises it; the parts that changed
    named by path;
  - the stream: a reader's changes name the list, a project and the decisions as they change,
    nothing for an ask alone, and end when the publisher goes; over HTTP, `GET /m/events`
    answers `text/event-stream` with its first event;
  - the pages: the start page's refusals in place of its button, its form; the list's usage
    strip and autopilot block with Pause or Resume; a project's runs with Stop; every page
    naming its listened paths and loading `/m/app.js`; the script served as JavaScript; the
    policy's two new sources;
  - the server: the start page's preview asked of the window and drawn, a 404 for a project the
    window does not list, the three forms reaching the window as orders;
  - the board: a start refused for each of the window's reasons that a test can set (a
    screenshot run, a page's old brief, a run working there, the CLI not answered); Stop
    refused for a key of another project or EP and taken for the run it names; Pause and Resume
    written once, a pause in force said; IN-14's hold, logged once, with a pending install
    taken at once;
  - `crw remote` over a scratch ledger.
- A screenshot run with `pages=` over the fixtures and over a copy of real roadmaps (rule 5),
  each page read in the scratchpad; the brief names what was checked, never the projects.
- Rule 3's word check prints nothing for the staged diff. Then two commits in the house pattern,
  not pushed, and the release build and the install last.

**Residual risk.** What the session could not exercise, and why it expects it to hold.

1. **The script in a real browser.** No test drives one (D-29). The script is small and uses
   `EventSource`, `fetch`, `DOMParser` and `replaceWith`, all in every current phone browser;
   the server's side (the events, the parts, the paths, the policy) is checked whole. The owner
   may open `http://127.0.0.1:47831/m/` on the machine itself; a phone waits for EP-31a's
   Enable.
2. **A real start from the phone.** Tests never run the real CLI (D-29): the start's refusals
   and its launch are tested on the board, and the start itself is the runner's
   `Sessions::start`, which every chained start uses. The first real Start from a page is the
   owner's, when they want one; nothing gates on it.
3. **The stream through Serve.** Tailscale's proxy flushes `text/event-stream` answers as they
   come (Go's reverse proxy does so for that type), and the keep-alive comment every 15 s keeps
   an idle stream open; unseen until EP-31a's Enable.
4. **IN-14's restart.** The hold and its line are tested with an unreadable scratch registry,
   and the restart through the installed copy is EP-37a's, tested there; the real case, a key a
   new build added during a run, has happened once and is not made again on purpose.
5. **Two refusals tested by reading alone.** "The CLI not answered" and "a gate being read
   there" are not set in a test: the first needs the sessions' asking state, which a test
   could reach only by starting a preflight of the real CLI (rule 5); the second needs the
   autopilot's private reading set. Both are one-line conditions beside the tested ones.

> **Completion note (2026-11-16; claude-opus-5-5/max; about 1 h 10 min).** A chained headless
> run from the window (ledger row `5140bb4c`, from 20:48), the work commit and the record commit
> after it, then the install, whose restart the window refuses while this run works. No question
> was put to the owner. The brief kept to its scope. `futures-util` joined under the D-15
> addendum's standing rule for the event stream's crate, at the version the lock held, so no
> package was added. Nothing outward-facing or in the owner's settings changed.
>
> **Built.**
>
> 1. **Start** (`app::phone`, `app::server`): `GET /m/p/<project>/start`, whose preview comes
>    from the window through `server::StartAsk`; `POST …/start` with the brief it showed
>    (`Order::Start`). `Board::prepare_start` is one reading for the preview and the start:
>    - the draft worded headless with the rules' choice (`Draft::for_chain(None, …)`), the
>      chained start's launch with the dialog's additions (`phone_launch`), the cap, budget,
>      push guard and rules' plan only with the autopilot on;
>    - the dialog's refusals, and also a page's old brief, a session at work there, a gate being
>      read there, a visual launch on its way, a deferred run there and a revert open;
>    - the CLI's preflight asked last.
>
>    The page waits for the runner's answer (`Board::phone_started`, the first arm of
>    `Board::receive`), a held chain giving way to the start.
> 2. **Stop**: the project page's runs (`published::Run`) with their Stop; `POST …/stop` names
>    the run by number and brief; `Sessions::stop`, never Kill.
> 3. **Pause and Resume**: `POST /m/autopilot` with `pause=on|off`, through `set_paused` (the
>    desk's write, refactored to return its words); a pause in force said, not written.
> 4. **The list's blocks**: `published::Usage` and `published::Autopilot`, ages drawn when the page
>    is drawn.
> 5. **The live updates**:
>    - the steady version (`List::steady`, `badges::Word::clocked`), versions counted from the
>      process's start;
>    - `Published::changed_paths`, and `Reader::changes(since)` over the `watch` channel with
>      `stream::unfold`;
>    - `GET /m/events` (server-sent events, `*` first for a page drawn from an older version);
>    - `GET /m/app.js` (`templates/app.js`): parts swapped unless typed in, folds kept, a closed
>      stream opened anew;
>    - every page's `data-listen`, `data-version` and `data-part`; the policy's `script-src` and
>      `connect-src`.
> 6. **`crw remote`** over the `remote` rows.
> 7. **IN-14** (`Board::hold_unread`, `registry::Error::for_newer_build`,
>    `autopilot::unread_words`):
>    - a chain whose registry this build cannot read is held and said once, and taken into a
>      newer installed build;
>    - after a completed run any failure holds the chain; a failed run's chain ends.
> 8. **The rest**: the screenshot feature's start pages; addenda to D-14, D-15, D-22 and D-28;
>    the README's words; IN-14's line noted; EP-31's pickup segment.
>
> **The review (rule 6).** A subagent was given the diff and this brief, not the session's
> reasoning, and reported after about twenty minutes with eight points:
>
> 1. *Fixed.* **A stream closed by an HTTP error never came back**: `EventSource` retries a
>    dropped connection but not a non-stream answer, such as Serve's while the window is down.
>    The script now opens the stream anew, waiting 5 s and doubling up to a minute.
> 2. *Fixed.* **A deferred run was invisible to the start**: a run deferred in the project with
>    its request open now refuses it, since the answer would resume a second session in the
>    same tree. `resume_deferred`'s own check is the desk's and stays.
> 3. *Fixed.* **The start's mark could stick**:
>    - a start whose thread was not made now answers at once;
>    - the phone's arm comes first in `Board::receive`;
>    - a disconnected start's thread now sets `last_chat`.
> 4. *Fixed.* **IN-14's words copied the registry's line**: the TOML error quotes the offending
>    line. The words now carry the parser's message alone (`for_newer_build`), and only such
>    failures hold at a re-read or a chained start. Other failures are tried again as before.
> 5. *Fixed.* **A gate being read or a visual launch on its way** now refuse the start.
> 6. *Fixed.* **`since` could match by chance after a restart**: versions now count from the
>    process's start.
> 7. *Fixed.* **The phone's Pause read every project again**: it no longer does (the desk's
>    Pause still does, out of this brief).
> 8. *Partly fixed.* **Tests**:
>    - fixed: the stream's end is asserted as an end, not a timeout;
>    - logged: "the CLI not answered" untested (residual risk 5).
>
> While the review ran, the session found and fixed one more point: a run's start just after the
> redirect could go unnamed, so the pages carry their version and the stream catches them up.
>
> **Verification.**
>
> - **The gate**, run again after the review's fixes:
>   - `cargo test`: 233 in `app` (220 before), 447 in `core` and the CLI's with `crw remote`'s
>     new test, all passing;
>   - `cargo test --features screenshot`: 241 (228 before);
>   - both clippy runs with `-D warnings` and `cargo fmt --check`: clean.
> - **Screenshot runs with `pages=`**, kept in `target\tmp\ep30b\`, out of git:
>   - over the fixtures with the canned usage and autopilot: the list, nine project pages, nine
>     start pages and the decisions;
>   - over a copy of three projects' roadmaps (this one and two others): seven pages.
>
>   Each page was checked with the standard library's parser: all balanced, no outside
>   address, `/m/app.js` the only script, the parts as listed. What they show: the usage strip
>   with its resets, the autopilot block with Pause, a held chain, a plan and its mark and the
>   chain log; a run's Stop; the Start link; the start page's choices with their reasons, the
>   downshift's words, the run's sentences, the notes, and a screenshot run's refusals in place
>   of the button.
> - **Rule 3.** The word check over the changed and new files found nothing, and so did the
>   check on the staged diff.
>
> **Owner decisions.** None was put to the owner. The commits are the autopilot's standing yes
> (D-22); the push is the owner's.
>
> **Routine choices (the session's call, logged).**
>
> 1. **The preview comes through a channel**, not the snapshot: a draft reads the project's
>    `CLAUDE.md`, too much for every second's build.
> 2. **Stricter than the dialog**: a session at work, a gate being read or a deferred run in the
>    project refuses a start from the phone, where the dialog only warns.
> 3. **The steady comparison** in place of timestamps carried for every badge, which would have
>    refactored the marks the window draws too.
> 4. **Events name page paths.** A session's changes show in its project's part, and EP-31's
>    session page adds its own part and path (its pickup segment says so).
> 5. **The catch-up by version** (`?since=`, `*`) in place of a fetch at every reconnect.
> 6. **The phone's Pause runs in the orders' pass**, not through the action queue, so the page
>    hears its outcome. The pass runs shown or hidden, as the queue's hidden pass does.
>
> No owner task, and no message for another project. **Next: EP-31**, the sessions' pages.
