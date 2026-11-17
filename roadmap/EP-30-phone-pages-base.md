---
ep: 30
title: Phone pages, the base — the window's state published to the server, queued actions run while the window is hidden (the tray's Pause with them), the /m/ router with the identity check and the remote-action log, askama's layout and the read-only project list
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-30 — Phone pages, the base

**Size:** L → M (cut beforehand at EP-29) · **Depends on:** EP-12 (the loopback server), EP-19
(the autopilot's Pause) · **Decisions:** D-28, D-14 (their addenda of 2026-11-15), D-15, D-29, D-3

## Context

P5 gives the owner's phone a view of the window over the tailnet (D-28). EP-29 cut the web pages
into three M briefs and settled their stack: askama templates rendered by the window's own axum
server, plain HTML forms, and later one small script of the tool's own; no Datastar, no htmx
(D-14 addendum). This brief is the base the two after it build on. Today the server only takes
signals in: `app/src/server.rs` serves `POST /hook`, `POST /statusline`, `GET /health`,
`POST /restart` and `POST /show`, and no handler can read the window's state. Queued actions also
run only in a drawn frame (`Board::run_actions` at the end of `Dashboard::ui`, after the early
return while hidden). So the tray's Pause waits until the window is shown (the P4 walk's finding,
D-29 addendum), and a phone's action would wait the same way.

**Session start (2026-11-16, 18:57 local time).** Started headless from the window by the owner:
the rolling log's "Started clauderoadmapwrangler EP-30 headless with claude-opus-5-5 at max
effort" at 18:57:25, after the chain plan's own mark was set to 80 % of the session window at
18:56 to 18:57 (the chain log). The prompt is the tool's opener, and `crw sessions
clauderoadmapwrangler` lists the row (`22d628f5`, headless, EP-30, claude-opus-5-5/max, open). No
owner task is open. One inbox message is open, IN-14, triaged below. The field check read, since
EP-42's record commit (2026-11-15, 20:20):

- `crash.log`: the one report of 2026-11-03, nothing newer.
- `crw notifications`: EP-42's finish at 20:22:39, then a manual session's turn at 23:13. Nothing
  since.
- `crw sessions`: this row; a manual session of 23:07 to 23:14 (the one that wrote IN-14); the
  manual row of 2026-11-15 13:23, still open (the idle tab EP-29 named).
- `crw autopilot`: not paused; on for this project and five others, all in mode `auto`; the
  newest lines are this chain's own mark.
- The rolling log:
  - 20:21:06: EP-42's install asked for a restart, refused while its run worked.
  - 20:22:39: the gate after EP-42 could not read the registry ("unknown field
    `private_words`"), and the run's re-read said the project was not read again. This is
    IN-14's subject.
  - 23:15:11: the owner's restart into the build of 20:21, carrying one chain plan. The plan
    outlived its chain because the gate never ran (IN-14 again).
  - 00:00:18: EP-41's first export at the day's turn, written as built.
  - 18:57:25: this run's start.

Nothing else odd to triage.

**Inbox triage (D-13).** IN-14 (from the owner, 2026-11-15) asks for the gate's registry read to
fail softly: when a newer installed build waits, take the restart anyway; or at least write the
chain log's line and a notice naming File > Install and restart. It is about an S, and it is low
priority. It does not fit this brief, which does not touch the gate. The owner's own suggested
home is "a brief that touches the gate or the install". EP-30b builds the phone's Start from the
gate's two halves (`Board::gated`, `Board::start_queued`), and `start_queued` is one of the three
places IN-14 names. So the proposal is to park IN-14 in EP-30b's brief notes, confirmed with the
owner (the outcome goes in the completion note). This brief adds a registry key itself
(`owner_login`) and leaves it unset in the owner's registry. That keeps IN-14's convention: the
window first meets the key through EP-31a's Enable, in a build that knows it.

## In scope

1. **The window's state published to the server** (D-14 addendum).
   - **The snapshot.** A new `app::published` module holds the list's data as plain data, with no
     egui type: `Published` (when it was built, a version raised only when the content changes,
     and the list). The list is the state groups in the list's order. Each project row carries:
     - from its `portfolio::ProjectRow`: the name, the state, the pin, a proposal waiting, the
       autopilot switch, `next()` (id, size, title) or why there is none, `counts()`,
       `open_items()`, `open_messages()` and `usage_tenths()`;
     - the session's words: `Board::session_summary(name)`, then `in_progress()`;
     - the badges in words, from the same `badges::Mark`s the row draws: a short word for the
       list and the mark's whole sentence.
     The unadopted folders are carried by name, with the files' problem and whether the files
     are still being read.
   - **The publishing.** A `tokio::sync::watch` channel (`sync` is on). In each pass, shown or
     hidden (`Dashboard::logic`), the window builds the snapshot at most once a second, and at
     once when a page asks for a fresh one. It sends the snapshot when the content changed, or
     when a page asked. The server's handler asks for a fresh snapshot when the one it holds is
     older than two seconds, wakes the window, and waits at most two seconds for the answer. A
     window too busy to answer leaves the page the older snapshot, whose time the page shows.
     The version's change is the event EP-30b's stream will send.
2. **Queued actions run while the window is hidden** (D-29 addendum).
   - `Board::run_queued` runs the queue and reads the files after a write. While the window is
     hidden, `Dashboard::logic` calls it after `attend`, so a queued action no longer waits for a
     drawn frame.
   - A pane brought forward (`Board::bring`, the one use of the tiles' tree) waits for the next
     drawn frame, where `run_actions` applies it.
   - The tray's orders go through `Board::obey`. A test holds the tray's Pause acting in a hidden
     pass: the registry's `autopilot_paused` written and the window's Pause on, with no frame
     drawn.
3. **The `/m/` router with the identity check** (D-28 addendum).
   - **The routes.** `GET /m/` (the list) and `GET /m/app.css` (the stylesheet, served from the
     exe) join `app::server`'s router. `GET /m` redirects to `/m/`. Any other `/m/…` path gets a
     404 page once it passes the check. The relays' five routes keep `Shared::local` unchanged.
   - **The identity check** is the core module `crw_core::remote`. It is pure logic over the
     request's headers, so it is tested without a server. A middleware applies it to every
     `/m/` route.
   - **A request through Serve** is one that carries `Tailscale-User-Login`. It passes only when
     the login equals `settings.owner_login` (ASCII case ignored) and its `Host` names a tailnet
     machine (`*.ts.net`) or the loopback. A page reaching the loopback through a name of its own
     (DNS rebinding) could otherwise send the login itself: the review's finding, fixed in this
     session. The registry is read at that request, so `crw setting` counts at once. A registry
     that does not read, or one that names no owner, refuses every request through Serve.
   - **A request from the machine itself** names the loopback in `Host` (`127.0.0.1:<port>` or
     `localhost:<port>`), carries no login and carries no header a proxy adds (`X-Forwarded-*`,
     `Forwarded`, any other `Tailscale-*`). Such a request is local. A request through Serve from
     a device with no user identity is then refused whichever way Serve passes `Host`.
   - **Anything else** gets a 403 page that names no reason.
   - **The origin.** A request whose `Origin` is not the page's own origin is refused. A form, any
     method but GET and HEAD, must carry its own origin: `http://<the loopback Host>` when local,
     `https://<X-Forwarded-Host or Host>` through Serve (`X-Forwarded-Proto` when present).
   - **The setting.** `owner_login` is new in `Settings`, `Settings::KEYS` and `Settings::get`, and
     `Settings::is_path` lists it as no path. `Changes::set_setting` checks the value (one word, no
     spaces or control characters) and has a `none` arm. `crw setting owner_login <login|none>`
     sets it and says what it does. EP-31a's Enable will write it.
   - **The response headers** on every `/m/` response: `Cache-Control: no-store`,
     `X-Content-Type-Options: nosniff`, `Referrer-Policy: same-origin` (a form then still sends
     its `Origin`), `X-Frame-Options: DENY`, and a Content-Security-Policy that allows the page's
     own stylesheet and forms and nothing else.
4. **The remote-action log** (D-28).
   - Through `Ledger::add_event`, under the new kind `remote` (`remote::EVENT_KIND`).
   - **An action through Serve** (any method but GET and HEAD) is written before its handler
     runs: the identity, the method and path and "allowed". It runs only once the row is
     written; a failed write answers 503. The base has no such route yet: EP-30a's forms are the
     first.
   - **A refused request** is written too, at most once a minute for each identity (a login, or
     "no login"). Its words name the reason and which proxy headers were present, never a
     header's value beyond the login (D-3).
   - **Not written:** reads, and the machine's own requests.
5. **askama's layout and the read-only list** (D-14).
   - `askama` (pinned `=0.16.1` in the workspace since EP-1) joins `crates/app`. Its features are
     set in the workspace entry to `derive` and `std` alone: no `config` (no `askama.toml` and no
     TOML parser of its own) and no `urlencode`. The lock gains askama's own crates, named in the
     note (D-15 addendum).
   - **The templates** are in `crates/app/templates/`: `base.html` (the head, the stylesheet's
     link, the header) and `list.html`. `app.css` sits beside them and is served from the exe.
     The page loads nothing from outside. Both themes follow the phone's own setting
     (`prefers-color-scheme`).
   - **The list page:** the state groups in the window's order and sort, with no filter. Done and
     archived are folded in a `<details>`, as the window folds them. Each project shows:
     - its name, state, pin, proposal and autopilot;
     - its next EP with size and title (the title's markup from `markup::spans`, as the row
       draws it) or why there is none;
     - its progress, as a `<progress>` and in words;
     - what waits (owner items, inbox, urgent);
     - the session's words;
     - the badges as words, with their sentences in a `<details>`.
     The page also shows the unadopted folders by name, the snapshot's time and a Reload link.
     It works at phone width.
   - **The screenshot feature** gains `pages=<file.html>`: once the run's projects are read, it
     writes the list page of its portfolio there. That is a fake portfolio, or a copy of the real
     roadmaps under `root=`, under the run's guards. A run binds no port, so this is how a page is
     checked on real content (rule 5). The copy and the page never enter git.
6. **The records.**
   - Addenda to D-28 (the guard as built), D-14 (askama and the snapshot), D-15 (the lock's
     additions) and D-29 (the hidden pass).
   - The README's status sentence, and the CLI's usage line for `crw setting`.
   - This brief and its completion note.

## Out of scope

- The project and decisions pages, the ticks, compose and answers (EP-30a). The start, stop and
  pause from the phone, the usage strip, the autopilot block, the event stream, the script and
  `crw remote` (EP-30b). The sessions' pages, notifications and diagnostics (EP-31). Enable
  remote view, Serve, the link and the tray's entry (EP-31a). No route of this brief changes
  anything.
- Links from the list to project pages. They come with EP-30a's `GET /m/p/<project>`, so the list
  links to nothing that does not exist yet.
- Setting `owner_login` in the owner's registry. The window running now is on EP-42's build and
  would refuse a registry that holds the key (IN-14). EP-31a's Enable writes it from a window
  that knows it.
- A test through `tower` (not a direct dependency) or a browser (D-29). The tests speak HTTP to
  `Server::start` on a free port.
- IN-14's fix (parked by the triage above).

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings` and
  `cargo fmt --check` pass (rule 6). Being an M brief, the diff is read by a review subagent given
  the diff and not this session's reasoning, before the release build.
- **Core tests:**
  - the setting's key, its check and `none`;
  - the identity rule:
    - local by the loopback `Host`;
    - refused for a foreign `Host` and for a proxy header without a login, whichever way `Host`
      arrives;
    - the owner's login allowed, case aside;
    - the owner's login with a foreign `Host` (DNS rebinding) refused, a forged form too;
    - another login refused;
    - a login while no owner is set refused, and while the registry does not read;
  - the origin rule: a GET with a foreign `Origin`, a form without one, a form with the loopback's
    own, a form through Serve with the forwarded host's, and `null`;
  - the log's words and the minute's throttle per identity.
- **App tests:**
  - the snapshot of the fake portfolio: groups, next EP, counts, waits, session words, badge
    words;
  - the publisher: a change raises the version, an unchanged snapshot does not, an asked one
    renews the time alone;
  - the list page rendered from it: escaped names, the title's code spans, folded groups, no
    external URL;
  - the server over HTTP with a helper that sends its own `Host`, `Origin` and
    `Tailscale-User-Login` and reads the body back:
    - a local `GET /m/` with the list and the headers;
    - `GET /m` redirected;
    - `GET /m/app.css`;
    - 403 for a foreign host, a foreign origin, a stranger's login and an absent owner, with one
      ledger row for each identity within the minute;
    - the owner's login allowed;
    - an owner's form logged before its 405;
    - the relays' routes still refusing an `Origin`;
  - the tray's Pause acting in a hidden pass.
- A screenshot run with `pages=` over the fixtures and over a copy of real roadmaps (rule 5),
  each page read in the scratchpad. The brief names what was checked, never the projects.
- Rule 3's word check prints nothing for the staged diff. Then two commits in the house pattern,
  not pushed, and the release build and the install last.

**Residual risk.** What the session could not exercise, and why it expects it to hold.

1. **A request as Serve really passes it.** No test can run `tailscale` (D-29). Tailscale's
   source sets `X-Forwarded-Host`, `X-Forwarded-For` and the identity headers, and appears to keep
   the original `Host`. The local rule therefore refuses a proxied request with no login however
   `Host` arrives, and both readings are tested by their headers. A request with the login must
   name a tailnet machine or the loopback in `Host` (the review's fix). If Serve handed on some
   other `Host`, the owner's own requests would get a 403 and a refused line; that fails closed,
   and the first Enable would show it at once. The owner's first Enable (EP-31a's task) shows the
   real request.
2. **A real browser.** No test drives one. The page is plain HTML and CSS, checked as text, and
   the forms' origin rule follows the Fetch standard: a POST carries `Origin` unless the referrer
   policy is `no-referrer`, which these pages never set. The owner may open
   `http://127.0.0.1:47831/m/` in a browser on the machine itself, where the loopback counts as
   local.
3. **The hidden pass in the real window.** The test runs the board's pass with no frame. The real
   window runs `logic` while hidden when a repaint is asked (eframe 0.37's documented
   behaviour), which the tray's orders and the server's requests both ask for.

> **Completion note (2026-11-16; claude-opus-5-5/max; about 1 h 5 min of wall time).** Started
> headless from the window by the owner (ledger row `22d628f5`, from 18:57); the work commit at
> about 20:00, the record commit after it, then the install. One question was put to the owner,
> answered within minutes while the first gate ran.
>
> **Built.**
>
> 1. **The snapshot** (`app::published`).
>    - `Published` (`at`, `version`, `list`). `List`: the state groups in the window's order and
>      sort, the unadopted folders by name, the files' problem and `reading`. `Group`.
>    - `Line`: the name, state, pin, proposal and autopilot; `Next`, or why there is none; the
>      counts; owner items, inbox and urgent; the sessions' words; the week's tenths; the
>      badges. `Badge`.
>    - `Publisher`: `due`, and `offer`, which sends a new version when the content changed, a
>      renewed time when a page asked, and nothing otherwise.
>    - `Reader::fresh`: the snapshot held when it is younger than 2 s; otherwise an ask, a wake
>      and at most 2 s of wait.
>    - `badges::words` and `Tone::class` give the badges in words from the row's own marks.
>      `list::no_next`, `missing_folder` and `session_words` give the same words to the window's
>      row and to the phone's line.
> 2. **The hidden pass.**
>    - `Board::pass`: receive, attend, the queue while hidden, publish. `Dashboard::logic` calls
>      it.
>    - `Board::run_queued`: what `run_actions` ran, less the pane brought forward.
>    - `Board::obey`: the tray's orders.
> 3. **The router and the guard.**
>    - `crw_core::remote`: `Asked`, `check`, `Caller`, `Refusal`, `is_loopback`, `check_login`,
>      the log's lines, `Throttle`, `EVENT_KIND`.
>    - `app::server`: the `/m/` routes on a sub-router behind the `guard` middleware;
>      `Server::serve(port, Some(Pages), wake)`; `secured`'s headers.
>    - The relays' `Shared::local` now calls `remote::is_loopback` and is otherwise unchanged.
>    - The registry's `owner_login`, and `crw setting owner_login`.
> 4. **The log.** Both writes go through `spawn_blocking` on the server's runtime:
>    - `Shared::log`, awaited before an action through Serve runs; a failed write answers 503;
>    - `Shared::log_detached`, for a refusal, throttled.
> 5. **The pages.**
>    - askama (`derive`, `std`) in the app crate.
>    - `templates/base.html`, `list.html`, `message.html`, `app.css`.
>    - `app::pages`: `list`, `message`, `STYLESHEET`, the view structs, and the title's runs from
>      `markup::spans`.
>    - The screenshot feature's `pages=<file.html>`.
>
> Also: addenda to D-14, D-15, D-28 and D-29; the README's status sentence; the CLI's usage line
> for `crw setting`; pickup segments in EP-30a's and EP-30b's brief notes, and IN-14's in
> EP-30b's.
>
> **The review (rule 6).** A subagent was given the diff and this brief, not the session's
> reasoning. It reported after about fifteen minutes, with six points:
>
> 1. *Fixed.* **A DNS-rebinding page passed as the owner.** The session had traced the same gap
>    while it waited. A page served from a name of its own that points at 127.0.0.1 is
>    same-origin to the browser. It can send the owner's login (no secret) and forwarding
>    headers of its choosing, with no `Origin` on a GET, and the login path never looked at
>    `Host`. Now a request with the login must name a tailnet machine (`remote::is_tailnet_name`)
>    or the loopback in `Host`. The origin's host is that tailnet `Host`, and `X-Forwarded-Host`
>    counts only behind Serve's loopback `Host`, which a page cannot send. Core and HTTP tests
>    cover a rebinding GET and a forged form. The D-28 addendum says so.
> 2. *Fixed.* **The test of the folded Done group could not fail**: the text before the heading
>    ends the same way open or folded. It now asserts the group's own `<details>` without
>    `open`.
> 3. *Logged for EP-30b.* **The version rises about once a minute with nothing new**, since the
>    badge sentences carry relative ages. Harmless before the event stream exists. EP-30b's
>    notes say to render the ages at drawing time, or to compare without them.
> 4. *Fixed.* **`at` was the last send, not the last build**, so after two seconds of stable
>    content every page asked and waited for a pass. A build with the same content now renews
>    the time without notifying, and the publisher's test holds it.
> 5. *Logged for EP-31a.* **The Serve mode.** A `--tcp` or `--tls-terminated-tcp` forward would
>    pass raw requests with neither login nor forwarding headers, which the guard and the
>    relays' routes would take as local. EP-31a's notes and the D-28 addendum require
>    `--https … --set-path=/m` and a refusal of a TCP forward to the window's port.
> 6. *Taken.* Nits:
>    - the Content-Security-Policy no longer names `img-src` (the brief says nothing else);
>    - a method is kept as sent, so an extension method `get` is a form, not a GET;
>    - the HTTP test now has a bare foreign `Host` and a login with a foreign `Host`;
>    - a test's doc comment named a case of the next test.
>
> What it checked and found sound: the parsing fails closed; the relays' guard is unweakened;
> every `/m` route, its 405 and HEAD pass the guard; askama escapes every value; the watch
> channel holds no borrow across an await and bounds every wait; the ledger writes from the
> server's thread hold; and no action of the queue needs a drawn frame.
>
> **Verification.**
>
> - **The gate**, run again after the review's fixes:
>   - `cargo test`: 206 in `app` (194 before), 447 in `core` (436) and the CLI's, all passing;
>   - `cargo test --features screenshot`: 214;
>   - both clippy runs with `-D warnings` and `cargo fmt --check`: clean.
> - **Screenshot runs with `pages=`**, over the fixtures (with the canned sessions and usage, then
>   again after the fixes) and over a copy of four projects' roadmaps. All were kept in the
>   scratchpad `target\tmp\ep30\` and out of git. Each page was read as text and its tags checked
>   for balance with the standard library's parser. They show:
>   - the groups in the window's order, finished work folded;
>   - the titles' code spans, long titles, a project with every row checked, and a proposal;
>   - the sessions' words, and each badge's word and sentence.
>   The desktop picture of the fixtures is unchanged by the rows' shared helpers.
> - **Rule 3.** An early word check over the changed and new files found one hit in a test
>   literal, now changed. The check on the staged diff printed nothing.
> - Then the release build and the install, last.
>
> **Owner decisions.**
>
> 1. **IN-14's triage** (D-13). Offered: park it in EP-30b's brief notes (recommended), roll it
>    into EP-30, put it in EP-32's v2 notes, or decline it. *Outcome:* parked in EP-30b's notes,
>    and IN-14 ticked with that disposition.
>
> The commits are the autopilot's standing yes (D-22); the push is the owner's.
>
> **Routine choices (the session's call, logged).**
>
> 1. **Local also needs no proxy header**, beyond the loopback `Host` the brief notes named. A
>    request through Serve from a device with no user carries no login, and it would otherwise
>    pass as local if Serve handed on the loopback's `Host`.
> 2. **Reads are not logged**, nor the machine's own requests. "Every remote action" is read as
>    every action through Serve, and EP-30b's fetches would otherwise fill the ledger. A refusal
>    is logged at most once a minute per identity, and for at most 64 identities a minute.
> 3. **The owner's login is read from the registry at the request**, as the quiet hours are read
>    when a toast is due, so `crw setting` counts at once. Only a request that carries a login
>    reads it.
> 4. **No request channel yet.** No route of this brief needs the window's hand beyond the
>    snapshot's ask, which is an `AtomicBool` and the wake. EP-30a's first form builds the
>    channel, as its brief notes now say.
> 5. **The list follows the window's list**: its sort key, no filter, and finished work folded
>    in a `<details>` as the window folds it by default. It has no links to project pages until
>    they exist (EP-30a).
> 6. **`GET /m` answers a 303 to `/m/`**, named by path alone. Any other `/m/…` path gets a 404
>    page once it passes the check.
> 7. **The badges' short words** are made in `badges::words`, beside the signs: `↑3 not pushed`,
>    `last commit 3 h ago`, `backup: within 48 h`, `+2 % of the weekly limit`. The marks'
>    sentences are in a `<details>`.
> 8. **A test literal changed for rule 3.** The early word check found a port of the private list
>    in a test's "another port"; a neighbouring port says the same.
>
> No owner task, and no message for another project. **Next: EP-30a**, the project and
> decisions pages.
