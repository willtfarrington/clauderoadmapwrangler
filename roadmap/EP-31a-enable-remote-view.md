---
ep: 31a
title: Enable remote view — `tailscale serve` on the phone's path alone behind a confirmation, the owner's login read and confirmed, Serve's status in Diagnostics, the link shown and pushed through ntfy, pushes that open their page, Disable, the tray's entry
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-31a — Enable remote view

**Size:** M · **Depends on:** EP-31 (the phone's pages as they stand), EP-20a (ntfy) ·
**Decisions:** D-28, D-21 (and D-3, D-10, D-29 as standing)

## Context

The phone's pages exist (EP-30 … EP-31b) and answer only the machine itself: the window's server
binds the loopback, and a request through Tailscale Serve passes the guard
(`crw_core::remote::check`) only with the registry's `owner_login`, which nothing sets yet. This
brief is the one that touches Tailscale, placed last by the owner's choice at EP-29: the action
that mounts the phone's path on the tailnet, the login it confirms, the link, the pushes that open
their page, Disable, and the tray's entry. Its brief notes (roadmap, P5) give the eight parts and
what EP-30 and EP-31 left for it.

**Session start (2026-11-17, 12:29 local time).** A visual launch from the window: the prompt is
the tool's opener, and `crw sessions clauderoadmapwrangler` lists the row (`d82a3e26`, visual,
EP-31a, open). The tab runs claude-opus-5-5 at max effort (the panel's choice; the transcript
records `"effort":"max"`). No owner task and no inbox message is open. The field check read, since
EP-31b's record commit (2026-11-17, 11:56):

- `crash.log`: the one report of 2026-11-03, nothing newer.
- `crw notifications`: EP-31b's finish at 11:58:29 and the chain's end at 11:58:30 ("EP-31a is
  left out of your plan"), each pushed to ntfy.
- `crw sessions`: this row; EP-31b's, completed in 25 minutes.
- `crw autopilot`: not paused; on for this project and five others, all in mode `auto`; 8 chained
  runs a day in all projects. The chain log's last line is the 11:58 end: the chain's plan,
  made before EP-31b's row existed, stopped short of EP-31a, as the EP-19a addendum says.
- The rolling log: 11:57, EP-31b's install asked for a restart, refused while its run worked;
  11:58, the project read again; 12:27, the owner's restart into the build of 11:57, so the
  window now runs EP-31b's build with its fix for a chat's run at the gate; 12:29, this launch.

Nothing odd: no finding to triage.

**What the installed client says.** Tailscale 1.104.7's `tailscale serve --help` lists `--bg`,
`--https <port>`, `--set-path <path>` ("appends the specified path to the base URL for accessing
the underlying service"), `--tcp`, `--tls-terminated-tcp`, `--http` and `--yes`, and a target that
may be "a full URL including a path (e.g., http://localhost:3000/foo)". So the brief note's form
stands: `tailscale serve --bg --https=9443 --set-path=/m http://127.0.0.1:<port>/m`. Its usage
lines do not name `off`, which Tailscale's source takes as the last argument to remove a mount:
Disable is `tailscale serve --https=9443 --set-path=/m off`, unexercised until the owner's first
Disable. No `--yes`: a prompt the tool did not see is never answered yes, and the child's input is
empty, so a prompt ends the command with its words. `tailscale serve status --json` on the owner's
node (read, names masked) holds three HTTPS ports, each with one `/` mount proxying to a loopback
port; the port D-28 names is free. `tailscale status --json` gives this machine's user as
`User[<Self.UserID>].LoginName`, its name as `Self.DNSName` (with a final dot), no `Self.Tags` on
an untagged machine, and one `CertDomains` entry when the tailnet has HTTPS certificates on.

## In scope

1. **Serve, read and run** (core, a module of its own on `actions`' runner).
   - `tailscale serve status --json` read into the node's TCP ports (HTTPS, HTTP, a TCP forward or
     a TLS-terminated one, with its target) and web mounts (port, host, path, proxy target).
   - The phone's mount weighed against the window's port: ours (the HTTPS port 9443, path `/m`,
     proxying to `http://127.0.0.1:<port>/m` or `localhost`); 9443 held otherwise (another mount
     there, or a forward) or `/m` held by another target, said in Serve's terms; and the window's
     port exposed elsewhere (a forward to it, which passes raw requests with no login and no proxy
     header, as EP-30's review found, or another mount proxying to it, which would put the relays'
     routes on the tailnet).
   - `tailscale status --json` read narrowly, a second read beside the doctor's: the backend's
     state, this machine's login, its name without the final dot, whether it is tagged, whether
     the tailnet gives it a certificate domain. Nothing of the peers is kept.
   - **Enable's plan**: the command above, shown before it runs; the login shown with what the
     registry holds now; the URL `https://<name>:9443/m/`; what it does and what it leaves alone
     (the node's other Serve entries; never `tailscale serve reset`). Refused when Tailscale does
     not run, the machine is tagged or names no login or name, the tailnet gives it no
     certificate domain, 9443 or `/m` is held, the window's port is exposed elsewhere, or the
     window's server does not listen. With our mount there already, no command: Enable writes the
     registry only.
   - **Disable's plan**: `tailscale serve --https=9443 --set-path=/m off`; with no mount of ours,
     no command: Disable takes the registry's record out only.
   - **The run**: the plan read again at the owner's word and run only when it is the plan shown;
     Serve's words shown whole; a minute's timeout in the children's turn. On success Enable
     writes `owner_login` and `remote_view` in one registry change (the owner's click, rule 13),
     Disable takes `remote_view` out and keeps the login. Each run is an `event` row of kind
     `remote` with the verb, the outcome and the command, never the tailnet name.
2. **The registry's `remote_view`**: this machine's tailnet name while remote view is on, written
   by Enable, taken out by Disable, `crw setting remote_view <name|none>` by hand. A build before
   this one refuses a registry that holds it (`deny_unknown_fields`): the install at this brief's
   close replaces the installed copy first.
3. **The guard** (`remote::check`): while `remote_view` names this machine, a request through
   Serve passes only with that name or the loopback in its `Host` (EP-30's note: Enable may narrow
   the check to the machine's own name); without it, any tailnet name, as today. Both ways Serve
   may pass `Host` stay tested by their headers (part 6 of the note). **The arrival**: the first
   request through Serve a window allows is a line of the remote-action log naming the `Host`'s
   form (this machine's name, another tailnet name, the loopback) and the proxy headers by name,
   never a value beyond the login, so the owner task's first step can say which reading the real
   request showed.
4. **The doctor's "remote view" line** (Network group): off; on at its URL; on in the registry
   while Serve says otherwise; Serve's mount with no record in the registry; the window's port
   exposed elsewhere; Serve's status unread. The probe asks `tailscale serve status --json` after
   `tailscale status --json`, in its turn. `crw doctor` prints it; the phone's doctor's page shows
   it with the rest of the report.
5. **The dialog** (`app`), in the form of EP-34's: the command named whole before it runs, the
   login shown and written only at the click, the URL with Copy and "Send the link to my phone"
   (ntfy, when it is set, the push's click opening the page), the result's words shown whole,
   Cancel and Close. A fake runner in tests and a canned one in screenshot runs: nothing real runs
   (rule 5). The restart is refused while it is open and Quit waits while it runs.
6. **Where it opens**: the tray's "Enable remote view…" or "Disable remote view…" before Install
   and restart (`Order::RemoteView`, handled where the tray's orders are, the window shown); the
   Diagnostics pane's Network group: Enable or Disable remote view…, Copy the link, Send the link
   to my phone. The window keeps the registry's record, read every half minute and after a run,
   for the tray and the pane.
7. **Pushes that open their page** (D-21): `Push` gains a click link, sent as ntfy's `Click`
   header. While the registry's `remote_view` is set, each push carries its toast's page: a
   waiting request the decisions page (a session waiting in VS Code too, which it lists
   read-only); a finished or failed run of the window its session's page, `Target::Session`
   carrying the run's session id beside its key (EP-31's note); a session elsewhere ended the
   sessions page; a chain's moment, a usage mark and the test the list; an urgent letter the
   project's page. A chat's question carries none (the chat is not on the phone). With it unset
   the pushes stay plain.
8. **The README**: the phone view's section and its security words in generic terms (D-3: no
   login, no tailnet name; a phone whose other VPN displaces the tailnet cannot reach the page,
   said as a hint).
9. **The owner task**, ungated: the first real Enable on the owner's tailnet, the link opened on
   the phone, one action of each kind, Disable; its first step reads the arrival line.
10. **The records**: addenda to D-28 and D-21; risk 18's; the roadmap's row; this brief and its
    completion note.

## Out of scope

- Funnel, a QR code, the whole port proxied, `tailscale serve reset`, or any change to a Serve
  entry beside the phone's mount.
- Enable or Disable from the phone (D-28: registry settings stay the desk's) or from `crw`
  (`crw doctor` shows the line; `crw setting remote_view none` takes the record out by hand).
- A check that the phone's other VPN displaces the tailnet: a hint in the README and the dialog.
- The local rule narrowed by what the first real request shows: EP-32's walk, with the arrival
  line, decides whether a loopback `Host` with the login may still pass.
- A machine renamed while remote view is on: the doctor's line shows Serve and the registry
  apart; Disable and Enable again.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings` and
  `cargo fmt --check` pass (rule 6). An M brief: a review subagent given the diff, launched at the
  first clean compile, its findings fixed or logged in the completion note.
- **Tests:**
  - core: the status read (an empty configuration, the owner's shape with synthetic names, ours,
    9443 held by a forward or another mount, `/m` held, a forward or another mount to the
    window's port, Funnel's flags); the node read (login, name, tagged, certificate domains, a
    stopped backend, no peer kept); Enable's and Disable's plans through a fake runner (the
    commands asked, each refusal, ours already there); the run (the registry written on success
    and not on failure, the login kept by Disable, the event row without the name); the doctor's
    line in each state; the registry key; the guard narrowed and the arrival line; the `Click`
    header;
  - app: the toast's page for each target and kind, a push with its link only while remote view
    is on; the tray's entry and its order; the dialog through a fake runner from the preview to
    the registry written; the pane's controls; the restart refused while the dialog is open; the
    server logging the first arrival once and narrowing to the registry's name.
- Screenshot runs (rule 5): the dialog for Enable and for Disable and the Diagnostics pane's
  Network group, over the fixtures and over a copy of one project's roadmap files; each read in
  the scratchpad, the brief naming what was checked, never the projects.
- Rule 3's word check prints nothing for the staged diff (and over new files before staging).
  Then two commits in the house pattern, not pushed, and the release build and the install last.

**Residual risk.** What the session could not exercise, and why it expects it to hold.

1. **The real Serve** (no session runs `tailscale serve`, rule 5): the command's form comes from
   1.104.7's own help, `off` from Tailscale's source; the first real Enable and Disable are the
   owner task's. A refusal by Serve is shown in its own words and changes no registry setting.
2. **How `Host` arrives through Serve**: both readings pass the guard, and the arrival line says
   which one the real request showed.
3. **A push's link on the phone**: ntfy's `Click` header is its documented form; whether the iOS
   app opens it while the tailnet is up is the owner task's.
4. **The narrowed guard on the owner's tailnet**: the name compared is the one Enable read from
   `Self.DNSName`, ASCII case aside, with or without the port; a mismatch refuses, which the
   arrival line and the doctor's line would show.

> **Completion note (2026-11-17; claude-opus-5-5/max; about 1 h 40 min).** A visual session
> launched from the window (ledger row `d82a3e26`, from 12:29); the opener and the row were
> checked at its start. The work commit and the record commit, then the release build and the
> install, whose restart the window takes, no headless run of its own working.
>
> **Owner decisions.**
>
> 1. **The first real Enable** (OT-29): later, as the owner task (the recommended option),
>    rather than at once with the session watching. OT-29 is open and ungated; EP-32's walk, the
>    phone's, takes what it showed.
> 2. **The word list**: the tailnet's suffix and the Tailscale login added to the git-ignored
>    `private/public-check.txt` (the recommended option), read from `tailscale status --json`
>    on this machine and never printed. The list already covered the machine's own name, and
>    neither new word appears in a tracked file.
>
> The commits are the autopilot's standing yes (D-22); the push is the owner's.
>
> **Built.**
>
> 1. **`crw_core::serve`**, which a subagent built to an exact API while the session built the
>    window: Serve's status read (each TCP port's kind, the web mounts, foreground sessions
>    merged, Funnel by port, `null` read as empty); the node read (state, login, name, tags,
>    certificate domain, no peer kept); `Status::standing` (the mount on the exact key `/m`,
>    what holds 9443 or `/m`, a forward or another mount to the window's port, a mount shaped
>    so to another loopback port, Funnel on 9443; `Standing::earlier_of`); Enable's and
>    Disable's plans and their run (Serve read again after a command that exits 0, then the
>    registry's one change, then an `event` row of kind `remote` without the name); the
>    doctor's "remote view" line and its probe; `actions::said` made crate-visible.
> 2. **The registry's `remote_view`** (`Settings::remote_view`, `crw setting remote_view
>    <name|none>`, `remote::check_machine`).
> 3. **The guard** (`remote::check` reads a `remote::Owner`): narrowed to the registry's
>    machine name in `Host`, or in `X-Forwarded-Host` behind a loopback `Host`
>    (`Refusal::OtherMachine`, `remote::names_machine`); `remote::arrival_line`, written once
>    a window by the server's guard (`Shared::arrived`).
> 4. **ntfy's `Click`** (`Push::click`, sent only as a printable `http(s)` address).
> 5. **The window** (`app::remote_view`): the dialog in the hosting actions' form (`Dialog`,
>    `Stage`, a `Canned` runner); `RemoteView` (the dialog, the record read every half minute
>    in the pass, hidden too, and after a run, a test's runner); `Board::open_`, `confirm_`,
>    `close_` and `receive_remote_view`, `copy_remote_link`, `send_remote_link`
>    (`Toasts::send_link`), `remote_view_buttons` under the Diagnostics pane's Network group,
>    the report gathered again after a run; `Action`s `OpenRemoteView`, `ConfirmRemoteView`,
>    `CloseRemoteView`, `CopyRemoteLink`, `SendRemoteLink`; the tray's entry before Install and
>    restart (`Order::RemoteView`, `tray::State::remote`); the restart refused while the dialog
>    is open, Quit waiting while it runs.
> 6. **The pushes' links**: `Target::Session { key, id }`, `Toast::page`, `Channels` (the quiet
>    hours, ntfy and the record, read at a delivery); the test's push links the list.
> 7. **The screenshot feature's** `remote=enable|disable|done|on`.
> 8. **The records**: the README's section "The phone view", its intro's sentence and three
>    commands; addenda to D-28 and D-21; risk 18's note; OT-29.
>
> **Verification.**
>
> - **The gate**: `cargo test`: 253 in `app` (243 before), 476 in `core` (448 before), the
>   CLI's and the rest, all passing; `cargo test --features screenshot`: 261 in `app` (251
>   before); both clippy runs with `-D warnings` and `cargo fmt --check`: clean, before and
>   after the review's fixes.
> - **The review** (a subagent given the diff alone, about 19 minutes) found no way past the
>   guard, and ten points:
>   1. Funnel on port 9443 went unnoticed. Fixed: Enable is refused, the doctor's line fails,
>      and the dialog's "never on Funnel" warning stays for Funnel on another port.
>   2. A mount left at an earlier window's port was no mount Disable would take off. Fixed with
>      a narrowing of the session's own: it is the tool's when the registry's record names its
>      host, and Disable takes it off; without the record it may be another service's, holds
>      the port and is left alone.
>   3. A `/m/` key counted as the mount though `--set-path=/m off` cannot reach it. Fixed: the
>      mount is the exact key `/m`.
>   4. Behind a loopback `Host`, the narrowing ignored `X-Forwarded-Host`. Fixed.
>   5. A command that exits 0 with nothing changed was taken at its word. Fixed: Serve is read
>      again first. Logged: a timeout keeps none of the command's printed words
>      (`process::finish`, shared code); the certificate check makes Serve's waiting flow
>      unlikely.
>   6. The registry written from the worker thread without a lock. Logged: `Store::change`
>      reads, changes and writes again, as a `crw setting` beside the window does; the window's
>      moment is the second after the click.
>   7. A refused plan showed the login and the link; the end stage showed the link with the
>      registry unwritten; the Diagnostics report was not gathered again after a run. Fixed.
>   8. Serve's `Services` read as a flag alone. Logged: a service's host is a tagged device,
>      which Enable refuses.
>   9. Test gaps. Fixed: the changed-plan path (`a_plan_changed_since_the_preview_runs_nothing`),
>      a dead assertion, and tests of Funnel, the earlier mount, the `/m/` key, the loopback
>      `Host` and a change that changed nothing.
>   10. The doctor's ok line names the link in `crw doctor`. Kept, as the brief asks; `crw
>       doctor`'s output is never pasted into a tracked file, and the word list now holds the
>       tailnet's suffix (the owner's choice above).
> - **Screenshot runs**, kept in the session's scratchpad, out of git: `remote=enable`,
>   `remote=disable` (dark), `remote=done` and `remote=on` with `tile=diagnostics` at
>   2300 × 1400 over the fixtures, and `remote=enable` and the pane over a copy of one project's
>   roadmap files. The dialog names the command whole, the login, the link and the VPN hint;
>   the end stage shows Serve's words, then the link with Copy and Send the link to my phone;
>   the Network group shows the line on, then the three buttons, wrapping in the narrow pane.
>   A `pages=` run wrote the phone's doctor's page with the line, escaped.
> - **Rule 3.** The word check over the working diff and the new files found nothing, and so
>   did the check on the staged diff, with the list's two new words.
>
> **Routine choices (the session's call, logged).**
>
> 1. A module of its own on the hosting actions' runner, rather than two more `actions::Verb`s:
>    its plan carries a login, a link and a registry write the hosting plans have no place for.
> 2. The record is the machine's name, from which the links are built, and the guard narrows
>    to it (EP-30's note allowed it).
> 3. The arrival line for the first request through Serve each window allows, a read too, so
>    that the owner task's first step can read how Serve passed `Host`.
> 4. No `--yes`: the child's input is empty, so an unseen prompt is never answered yes.
> 5. Refusals beyond the note: no certificate domain, a tagged machine, a login or a name the
>    registry would refuse, another web mount to the window's port, Funnel on 9443.
> 6. Disable keeps `owner_login`.
> 7. The pages of the pushes the note does not name: a session elsewhere's end the sessions
>    page, a usage mark and the test the list, a VS Code session's wait the decisions page; a
>    chat's question none.
> 8. The tray and the Diagnostics pane open the dialog; no File menu entry.
> 9. Copy sits among the pane's buttons, not on the report's line: the report is the doctor's,
>    the buttons the window's.
> 10. The record read every half minute in the pass, never at each frame (EP-28's trap).
>
> The four residual risks above stand; OT-29 and EP-32's walk take them. No message for another
> project. **Next: EP-32**, the P5 re-plan and the v1 close-out, opening with the P5 walk; the
> autopilot never chains a re-plan, so it is the owner's to start.
