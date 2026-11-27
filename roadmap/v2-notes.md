# v2 notes — parked at the v1 planning interview (2026-10-31)

Everything named during planning and not built in v1. Every re-plan EP grows this file; the v1 close-out
(EP-32) compiles it into themes with a v2 direction. Grown at EP-8 (2026-11-01, the P1 re-plan) by the
items marked *(EP-8)*, at EP-9 by the one marked *(EP-9)*, at EP-10 by those marked *(EP-10)*, at
EP-33 by the one marked *(EP-33)*, at EP-11 by those marked *(EP-11)*, at EP-12 by those marked
*(EP-12)*, at EP-13 by those marked *(EP-13)*, at EP-14 by those marked *(EP-14)*, at EP-15b by the
revert's, at EP-16 (2026-11-04, the P2 re-plan) by those marked *(EP-16)*, at EP-35 by those
marked *(EP-35)*, at EP-19, EP-19a, EP-19c and EP-20a by those marked so, at EP-23 (2026-11-11,
the P3 re-plan) by those marked *(EP-23)*, and at EP-29 (2026-11-15, the P4 re-plan) by those
marked *(EP-29)*, each with the brief it comes from.

**Compiled at v1's close-out (EP-32, 2026-11-17).** v1 is done (69 briefs; the roadmap, D-1
addendum). Each theme below now opens with what v1 built of it and what stays parked; the items
P5 named are marked *(EP-32)* with the brief they come from; nothing was taken out. The direction
the owner chose for v2 comes first. The routes no real use met in v1, carried into v2's first
walk, are listed in EP-44's brief note in the roadmap, where v2's plan reads them.

**Grown in P6 (2026-11-22 … 11-26).** By the items marked *(EP-43)*, *(EP-43a)*, *(EP-45)* and
*(EP-57a)*, which those briefs parked, and at EP-59, the P6 re-plan, by those marked *(EP-59)*,
each with the brief it comes from; nothing was taken out. The routes P6's walk did not meet are
carried to P7's re-plan (EP-61) and listed in EP-44's brief note.

## The v2 direction (EP-32, 2026-11-17)

The owner's choice at the close-out: all four candidates named at the v1 planning lead v2. The v2
planning interview (EP-44, the `plan` skill; D-30 addendum) orders them and weighs the rest of
this file.

1. **Claude Code's background sessions with Attach** (§1): `claude --bg` as a third launch mode,
   Attach in a Windows Terminal tab, `claude logs` and the supervisor's state. Each managed project
   needs `"worktree": {"bgIsolation": "none"}` so that roadmap edits stay in the main tree (risk
   9); such sessions are interactive, so Remote Control reaches them from the phone.
2. **A worktree per session, with its diff reviewed in the tool** (§1): each run in a git worktree
   of its own, its diff read in the window before it lands in the main tree; weighed against the
   house's process, which edits the main tree and reviews in VS Code.
3. **Usage over time** (§2): the weekly, Fable and five-hour figures charted from the samples kept
   since EP-17, with each project's share; with them the five-hour extras parked at EP-29 (alerts
   at 80 % and 95 % of the session window, a five-hour line in the sparkline, a predictive gate
   that needs each session's five-hour rise recorded).
4. **Free chat from the phone** (§4): a typed message into a session or the chat from the phone,
   which D-28 left out because a lost device would reach a Claude Code session; v2 weighs it
   against that reason and the guards EP-30 … EP-31a built.

Before them, EP-43 fixes the one defect the P5 walk found: a form from the phone the window took
eleven seconds after its post, past the page's wait (D-28 addendum).

## 1 Launch modes

*(EP-32, the theme compiled.)* Built in v1 from this list: Resume in VS Code for visual and manual
sessions (EP-35), a headless run's row closed at the next start (EP-28a), the held chain and the
chain plan carried across the window's own restart (EP-37a), this project chained (EP-37,
EP-37a), the downshift for every size (EP-40). Leading v2: background sessions with Attach and a
worktree per session, and the five-hour extras with usage over time (the direction above).
Parked: other launch surfaces and their hand-offs (a terminal `claude --ide`, a launch from
`crw`, a visual launch's model and effort), the dialog writing defaults and templates into the
registry, a headless run resumed after the window restarts, the Decisions tile's further answers
(Allow always, a free reply, a request kept across a restart), and the autopilot's refinements (a
plan in the registry, a pause per project, the gate read from the command line, a chain after a
visual session).

- **Claude's own background sessions (`claude --bg`) as a third launch mode** with Attach in a Windows
  Terminal tab, `claude logs`, and the supervisor's `claude agents --json` state; requires `"worktree":
  {"bgIsolation": "none"}` in each managed project's `.claude/settings.json` so roadmap edits stay in the
  main tree; such sessions are interactive, so Remote Control reaches them from the phone (round 7, round
  17: not picked). *(EP-32: leads v2's direction.)*
- **Worktree per session** and **diff review** inside the tool (recurring in other orchestrators; the
  owner's process edits the main tree and reviews in VS Code). *(EP-32: leads v2's direction.)*
- A Windows Terminal tab running `claude --ide` as a visible CLI attached to VS Code (an alternative
  visual launch).
- *(EP-9)* Claude Code's OS-level sandbox for headless runs that have neighbour folders: the `Edit`
  deny rule keeps the built-in tools and the file commands Claude Code recognises out of a
  neighbour, but not a program that opens files itself, which a project's own allow rules may admit
  (D-18 addendum; the owner's choice at EP-9).
- *(EP-10)* Rule titles matched by a regular expression, which needs a `regex` pin (the owner chose
  keywords; D-22 addendum).
- *(EP-10)* Launch defaults, rules and templates edited from the launch dialog and written into the
  registry; EP-10's dialog only reads them and the owner edits `registry.toml` by hand (rule 13).
- *(EP-11)* A visual launch's model and effort handed to the extension: its link takes only
  `prompt` and `session`, so VS Code's panel chooses and the prompt names the launch's suggestion
  (D-17 addendum).
- *(EP-11)* The launch mode set by a rule or a global setting; EP-11 resolves it from the project's
  `[project.launch] mode` over the tool's default only (D-16 addendum).
- *(EP-11)* A project's VS Code window brought forward by its handle or by VS Code's window id,
  which would also find a window opened from the folder rather than from its own workspace file;
  EP-11 opens the workspace file, which such a window does not answer to.
- *(EP-11)* A `crw` command that launches visually; `crw prompt --visual` only prints the prompt.
- *(EP-12)* A status line that chains the owner's own: `crw statusline` running another command and
  passing its output through. None was configured at EP-12, so an install leaves another status
  line in place and says so (D-23 addendum).
- *(EP-12)* A status line that prints the session's figures (cost, context, the rate limits) rather
  than nothing, which would give the terminal's status row back a use (D-23 addendum).
- *(EP-12)* A marker file the window writes while it listens, so that a hook with no window to
  talk to costs no connection attempt at all (EP-12 gives the connection 50 ms; D-20 addendum).
- *(EP-12)* Ledger rows for sessions the poll of live sessions lists without a start hook (a session
  started while no window ran), with the start time the listing gives (D-26 addendum).
- *(EP-13)* Resume for a visual or manual session from the tool, through the extension's
  `vscode://anthropic.claude-code/open?session=<id>` link, which opens the session in the project's
  window (EP-13 resumes the window's own headless sessions only; D-16 addendum). *(EP-16: planned
  as EP-35, for a headless run a crash cut short too.)* *(EP-23: built by EP-35 for visual and
  manual sessions; a headless run's id opens an empty tab there, the next item.)*
- *(EP-13)* Resume of a headless session after the window restarted, from its ledger row: its
  launch's permission mode and neighbours would need keeping in the ledger (D-16 addendum).
  *(EP-35: VS Code's extension opens a headless run's id as an empty tab, so this is the window's
  only way back to such a run; `claude --resume <id>` in a terminal meanwhile.)* *(EP-23: weighed
  at the P3 walk against the autopilot's needs and kept parked, the owner's choice: in the first
  week every headless run ended before the window restarted, and a gate's restart comes only
  between two runs.)*
- *(EP-14)* "Allow always" in the Decisions tile: a permission request's suggestions
  (`permission_suggestions`) offered as an allow that writes the rule (`updatedPermissions`);
  EP-14 allows or denies one request at a time (D-19 addendum).
- *(EP-14)* A free reply in place of a question's answers (the Agent SDK's `response`, which the
  session reads as "The user responded: …"); EP-14 sends the answers or a refusal.
- *(EP-14)* A waiting or deferred request kept across a restart of the window: the deferred call
  and the launch's choices would need keeping in the ledger (EP-14's open requests live in the
  window, as its sessions do).
- *(EP-14)* A run whose control-protocol handshake is refused restarted at once on the defer route,
  or a defer hook that asks the window at run time whether to defer; EP-14 switches the window's
  later runs only (D-19 addendum).
- *(EP-14)* One time limit for every project beside each project's own `answer_timeout`.
- Undoing a session whose commits the remote already has, by new commits that reverse them (`git
  revert`) or by a force-push: EP-15b refuses such a revert and the tool never force-pushes (the
  alternative the owner did not pick, 2026-11-03).
- *(EP-16)* A revert that restores the start's index exactly: changes staged at a session's start
  come back unstaged (EP-15b's out-of-scope list); and the revert's copy opened in Explorer from
  the dialog, which names its folder (EP-15c's).
- *(EP-16)* Headless as the default launch mode, with Visual written for this project in the
  registry (the alternative the owner did not pick: a headless run asks at every shell command
  where no allow rule admits it, and this project's release build ends it; D-16 addendum).
- *(EP-35)* A prompt handed with Resume in VS Code: the link takes `prompt` beside `session`, but
  a tab that holds the session already takes none ("Session is already open. Your prompt was not
  applied"), so EP-35 hands the session alone and the owner writes in the tab.
- *(EP-35)* `crw resume <project> [<session>]`, the window's Resume in VS Code from the command
  line.
- *(EP-35)* A headless run's row that a crash left open closed when the window starts again: no
  hook closes a row of the runner's, and a resume in a terminal starts a manual row beside it.
  *(EP-29: built by EP-28a's reconciliation, which closes such a row as stopped at the window's
  next start.)*
- *(EP-35)* Resume in VS Code for a session whose working folder was a subfolder of the project:
  its transcript lies under the subfolder's slug, which the project's window may not look in.
- *(EP-19)* A held chain kept across a restart of the window: EP-19's lives in the window, and
  the log says where it stood (D-22 addendum). *(2026-11-08: planned as EP-37, the handover file
  of the window's own restart. 2026-11-10: built by EP-37a for the window's own restart into a
  newer build; a held chain still goes with a quit or a crash.)*
- *(EP-19)* A chain after a visual session: the gate follows the window's headless runs only, a
  visual tab's end coming hours after its last turn (D-22).
- *(EP-19)* `crw autopilot gate <project>`: the gate read now from the command line, as if the
  project's last run had just completed, to see why it would refuse.
- *(EP-19)* A pause per project beside the global one, and a cap that counts the wall clock as a
  second limit (the owner chose working time alone at pickup).
- *(EP-19)* This project chained, its release build and install script left to an owner task
  after the chain, or built where the running window does not hold it (EP-28's install script);
  the owner chose never to chain it at pickup (D-22 addendum). *(2026-11-08: planned as EP-37, the
  exe installed outside `target`; the owner's choice reversed, D-22 addendum of that date.
  2026-11-10: built by EP-37 and EP-37a; it chains once the owner sets its mode to `auto`.)*
- *(EP-19a)* A chain plan kept in the registry or across a restart of the window: EP-19a's
  lives in the window, as a held chain does, and goes with it (the owner's choice, D-22
  addendum); with it, `crw autopilot` printing the plan, or a plan set from the command line.
  *(2026-11-08: the plan across the window's own restart planned as EP-37 with the held chain; the
  registry form and the command line stay parked. 2026-11-10: the plan across the window's own
  restart built by EP-37a.)*
- *(EP-19a)* A "back to the rules" button on a plan row, dropping the owner's choice; today a
  touched row keeps its pickers as left until the dialog is opened again.
- *(EP-19a)* A plan listing beyond the project's `daily`, greyed where the day's cap would stop
  the chain, and a count of runs beside the boxes (the owner chose a box per brief at pickup).
- *(EP-19c)* A Keep menu beside the chain plan rows' pickers. Today the main pickers' keep reaches
  the rows that the owner left alone, by resolving them again.
- *(EP-19c)* Ultracode kept as a standing choice; `crw set` for a phase's rule; a keep made in
  Visual, whose pickers are greyed out (the owner's choice at EP-19b, since the tab's panel
  chooses there).
- *(EP-23, from EP-18 and EP-19c)* A downshift for M or L briefs, to a model other than Opus 5.5,
  or a rule's opt-out; the downshift lifted over a kept Fable model (it stays the resolver's last
  layer and names itself when it stands over a keep); the effort's "CLI default" kept as a
  standing choice. *(EP-29: EP-40 built the downshift for every size and the owner's policy as
  two rules in the registry; another model, a rule's opt-out and the rest stay parked.)*
- *(EP-23, from EP-19b)* A control for the permission mode in the Autopilot section, which shows
  it (the box's offer, the dialog's pin and `crw set` change it); free figures for the limits in
  the window, whose pickers offer presets (`crw set` takes any).
- *(EP-23, from EP-19 and EP-19b)* A push by a chained run, and a push guard that also refuses a
  push written another way (`git -C . push`): the rule `Bash(git push *)` guards the usual form
  (risk 17). A supervised probe of the guard in a scratch repository, or a Diagnostics check that
  the window's autopilot runs hold the rule, were the alternatives to the risk line at the P3
  walk.
- *(EP-23, from EP-35)* A resumed run that reopens its old ledger row, or writes none (the owner
  chose a row per run).
- *(EP-23, from EP-37 and EP-37a)* More carried across the window's restart: a session's start
  not yet in the ledger, a disposition being typed, an open dialog, the popup's notices, and the
  ended runs' tabs with their drawn transcripts (not missed, the owner's answer at the P3 walk),
  or each ended run's result named in the new window's notice. A refused restart remembered and
  taken once the runs end (not remembered, since a restart that came later could surprise the
  owner). A restart at a chain's end, the end's words carried for the new window's popup (the
  walk chose the launch dialog's offer instead, EP-39). The gate's moment building or installing
  by itself (it takes an installed build only).
- *(EP-23, from EP-22a)* A visual launch that loads the build's copy of the plugin: the
  extension's link takes a prompt and a session only, so those sessions have the owner's install.
- *(EP-23, from EP-20b)* The prompt hook's mentions by folder path, or by a folder name that
  differs from the registry's; a session title from the hook; a hook that blocks or changes a
  prompt; the context as a pointer alone (the alternative to EP-39's order).
- *(EP-29, from EP-39)* A restart by itself when the launch dialog opens on a newer installed
  build (the dialog offers it); the dialog's edits (its prompt, its mode, the chain plan's rows)
  carried across that restart, which carries the project alone.
- *(EP-29, from EP-40)* The five-hour limit's extras, parked at the P4 walk (the owner's choice):
  notifications at 80 % and 95 % of the session window, a five-hour line in the usage sparkline,
  and a predictive gate from the ledger's attribution, which needs the five-hour rise recorded
  for each session (a ledger column). A run that hit the limit resumed after the reset (the
  owner prefers a fresh run). The global mark made editable in the Autopilot section (the
  alternative to EP-41's mark per chain). The single-instance guard's two gaps: a hand-started
  window that binds the port in the moment between a restart's two windows, and the second
  window's quit coming before its meter's first poll by timing, not by a gate.
- *(EP-29, from EP-28a)* The start's parked points. An entry turned off in Task Manager's Startup
  apps, a flag Windows keeps beside the Run value, which the doctor and File > Start at login do
  not read; `crw.previous-<stamp>.exe` files left in the PATH copy's folder until the next pass
  (both parked at the P4 walk, the owner's choice). The window created hidden at a start at
  login, should EP-32's walk find the flash eframe may give it; a delay at sign-in; a
  reconciliation during the window's life; the icon inside the exes as a resource (it needs a
  resource compiler, a stack change); the install folder itself on PATH, or a `crw.cmd` shim
  (the owner chose a copy).
- *(EP-32, from EP-41)* A chain's own five-hour mark shown in the tray's line and the Autopilot
  section's header chip, which say what the setting holds; an editor for the standing setting in
  the window (`crw setting autopilot_five_hour_percent` serves); the start's mark line taken from
  the run's own plan, where today it takes the open dialog's project, so a resumed run starting
  while another project's dialog is open logs that project's mark (EP-41's review, logged).
- *(EP-32, from EP-31b)* Each project's own count in its line ("n of its runs a day today"); `crw
  autopilot`'s first line with the day's count; a chain the daily total ended taken up again when
  the total is raised (the gate ends a chain at a daily cap rather than holding it).
- *(EP-32, from the field check)* The words of a five-hour hold: the chain log says "held after
  EP-n until the pause lifts" for the usage mark, the name the weekly and Fable mark has had since
  EP-19, while the rest of the line names the chain's own mark (EP-41); "until the session window
  resets" would say what lifts it. And the words of a re-read after a run when the registry does
  not read (a key the running build does not know): "was not read again: the registry does not
  list it" (2026-11-15, the case IN-14 fixed for the gate).
- *(EP-43a)* A warning sent into a run that nears its cap, such as "fifteen minutes of your cap
  remain: close with what is done", so that a run which misjudged its row at the brief closes
  before the window stops it. The runner's input stream, which carries the control protocol's
  answers, could carry it; whether a message sent in the middle of a turn reaches the session
  before the turn ends is the CLI's behaviour and unexercised. EP-43a made the run's own judgment
  at its brief the first guard (its opener says the cap and its start) and parked this, as its
  brief note said. With it: a resumed run told the cap of its new run, since each resume has a
  cap of its own and its message is the owner's word or an answer, never an opener.
- *(EP-59, from EP-48)* A repository whose `.git` is a file (a worktree) is not watched, since
  EP-48's watcher follows `.git/logs/HEAD`, and neither is a `roadmap` folder or a first commit
  made after the last full read, until the next one (F5, a folder change, the day's turn): a
  worktree per session, the direction's second lead, needs the watcher to follow each worktree's
  own `HEAD`.

## 2 Analytics

*(EP-32, the theme compiled.)* Built in v1: the per-poll attribution and the Usage badge (EP-18a,
EP-18b), the history import and the daily counts (EP-25, EP-25a), the graphs, the sparklines and
the cards (EP-26, EP-26a), and the projection. Leading v2: usage over time (the direction above).
Parked: the attribution's finer forms, a re-read at every hook event, the analytics printed by
`crw`, the portfolio's burn-up, and transcripts parsed for tokens (rejected for v1, D-23, D-26).

- **Usage over time**: weekly and Fable percentages charted from the samples with per-project share
  (round 13, round 17: not picked). The samples are recorded from EP-17, so the graph is cheap later.
  *(EP-32: leads v2's direction.)*
- Parsing transcripts for token counts per project (the approach of existing usage tools; the format is
  declared internal and changes between releases) — rejected for v1 (D-23, D-26).
- Transcript copies kept beyond the 30-day sweep (rejected, D-3).
- *(EP-23, from EP-17)* A usage poll from `crw`: only the window reads the credentials and polls.
- *(EP-23, from EP-18a)* The attribution's finer forms: an unattributed remainder shown, a split
  weighted by busy time (it is equal among the sessions that worked), the five-hour figure, use
  the window cannot see (claude.ai, the phone) told apart, and the check whether the poll of live
  sessions lists a session's subagents or the extension's helpers as sessions of their own.
  *(EP-29: the five-hour limit is back and the gate weighs its figure since EP-40; its rise
  split among the sessions is still unbuilt, and is what a predictive gate would need.)*
- *(EP-23, from EP-18b)* The Fable sum in the Usage cell with a sort by it; the usage share in
  `crw status` (`crw sessions` prints the week's sums).
- *(EP-29, from EP-25 and EP-25a)* The ledger's rows from before EP-13 cleaned or backfilled;
  Re-import in a row's menu (the Detail pane has it); the latest daily count drawn in the
  History section; record commits worded another way (an unknown form reads as noise); a GitHub
  pass, or the backup job's reading, for one project; a re-read at every hook event rather than
  at a session's end.
- *(EP-29, from EP-26 and EP-26a)* The usage sparkline moved to `egui_plot`; the owner's waits in
  visual sessions counted from the watch's busy and idle; the waits as a true sum (they are a
  mean times a count, worded "about"); the analytics printed by `crw`; a burn-up, a velocity and
  a projection for the whole portfolio (the switch moves the heatmap alone); the ledger read off
  the UI thread; the heatmap in an expanded row; Start and the autopilot box on a card; a card's
  density, or cards resized by hand; the arrow keys in the cards view.

## 3 Chat

*(EP-32, the theme compiled.)* Built in v1: the hosted read-only chat over the portfolio, its
replies as Markdown with action buttons, its questions answered in the pane, `crw` reading
commands under exact rules, `@<project>` mentions (EP-24, EP-24a, EP-24b). Parked: the portfolio
report, writing commands for the session, the replies' and the pane's further forms; a full-tool
chat session stays rejected (D-27).

- **Portfolio report**: a Markdown report across projects written into `private/` on request (round 14,
  round 17: not picked).
- A full-tool chat session in `DATA` (rejected for blast radius, D-27).
- *(EP-29, from EP-24, EP-24a and EP-24b)* The chat's parked points. In a reply: syntax
  highlighting, images (an image reads as its alt text), a search, a link opened in the browser
  (the status bar names its target). The conversations in a pane of their own, or more than the
  latest twenty listed; a conversation's earlier turns read back from Claude Code's transcripts;
  a time limit for a chat's own question, which waits for good; the Sessions pane's transcripts
  drawn as Markdown too. For the session: writing commands admitted (`crw send`, `crw formalise
  --yes`, `crw adopt`; the owner chose none, every write a button), `crw scan` and `crw chats`,
  an owner task added from the chat, a button from the streaming text, commands with a pipe or a
  redirect, and a tool call the host answers (a server of the tool's own) as the proposal's
  channel, should the line form prove fragile. A ledger mode of its own for chat runs; a `data`
  box in the window's own Adopt, as the chat's confirmation has.
- *(EP-59, from EP-47)* An input method's composition that ends with Enter (a dead key, an East
  Asian IME) could send a message where it should only confirm the composition: EP-47's key tests
  drive synthetic presses, and no chat has run since its build.

## 4 Phone view

*(EP-32, the theme compiled.)* Built in v1: the phone's pages under `/m/` with the identity check,
the eight actions and their log (EP-30 … EP-31), and Enable remote view on Tailscale Serve
(EP-31a), first used for real in the P5 walk (OT-29; D-28 addendum). Leading v2: free chat from
the phone (the direction above). Parked: the actions D-28 keeps at the desk, other ways in (a
PIN, a path under another hostname, an eframe-wasm page), and the guard's exact rule.

- **Free chat from the phone** (round 15, round 17: not picked; a lost device would reach a Claude Code
  session). *(EP-32: leads v2's direction.)*
- An eframe-wasm phone view served by the same server (egui 0.36 improved mobile IME; the
  keyboard-covers-field issue #4572 was open on 2026-10-31).
- A PIN for write actions and periodic re-authentication (alternatives to D-28).
- A path mount under the other project's existing hostname instead of a new HTTPS port.
- *(EP-29)* What the P4 re-plan left out of the phone view, each the owner's choice (D-28
  addendum): a revert from the phone, or its preview read-only; a session resumed with a typed
  message; a project's autopilot switch; Kill; the hosting and backup actions; a QR code for
  the link (`qrcode` 0.14.1 was the crate looked at; the link is shown and pushed through
  ntfy); links on the waiting pushes
  alone; Enable remote view built early, each page then watched on the phone, or a gated phone
  check after each page's brief; the whole port proxied as D-28 first wrote it.
- *(EP-32, from EP-30a, EP-30b and EP-31)* What P5's briefs kept at the desk, each by D-28's
  list: untick, and the ticked tasks and messages folded on a project's page; Focus for a session
  waiting in VS Code; a chat's questions; a start with another model or effort than the rules', a
  visual start, the chain plan's choices, Resume or Commit; a typed message into a session; a
  transcript of a session started elsewhere (the tool reads none, D-26); the Diagnostics pane's
  own lines (its build, signals, meter, routes); the daily total set from the phone.
- *(EP-32, from EP-31a)* Enable and Disable from the phone or from `crw`; a check that the phone's
  other VPN displaces the tailnet (a hint in the README and the dialog); the machine renamed while
  remote view is on (Disable, then Enable). The review's logged points: a command that times out
  keeps none of its printed words (`process::finish`, shared code); the registry written from the
  dialog's worker thread without a lock, as a `crw setting` beside the window would; Serve's
  `Services` read as a flag alone (a service's host is a tagged device, which Enable refuses).
- *(EP-32, from the P5 walk)* The guard's exact rule: since Serve passes this machine's tailnet
  name in `Host` (OT-29's arrival line), a request with the owner's login and a loopback `Host`
  refused, though only the machine itself can send one (the owner kept both readings, D-28
  addendum).
- *(EP-57a)* A project's manual on its phone page: not cheap, so not built. The PDF lives in the
  window's local folder on this PC; the phone would need a new route serving the file behind the
  guard (the build too, when stale, on the window's worker thread), the first binary a page sends.
  The window's Manual entries and `crw manual` are the ways in meanwhile.

## 5 Hosting and backup

*(EP-32, the theme compiled.)* Built in v1: the hosting badges and their actions behind a
confirmation, an orphan's Archive and Delete, Back up now (EP-7, EP-34), the word check of the
history before Make public (EP-42), the site's cards, the daily export with its keep and the CSV
(EP-27, EP-41). Parked: the actions from the command line, more of a repository changed from the
tool, the exports in the menus, more guards on the publishing path (a hook, a scan before
Create), and a second backup mechanism (rejected, D-24).

- Deleting the orphan private repository from the tool (needs the `delete_repo` scope; D-24 flags it with
  Archive and Delete actions and leaves the scope refresh to the owner). *(EP-29: EP-34 built
  Archive and Delete for an orphan; Delete is refused until the owner's token has the scope.)*
- A second backup mechanism (rejected; the Documents job is the one, D-24).
- Health badges beyond the latest Actions run and licence presence (CodeQL, Dependabot alerts).
- *(EP-8)* Rewriting this repository's history to drop the planning record's first wording of the
  owner's machine details (rejected before the public flip: every recorded hash would change; D-3
  addendum).
- *(EP-29, from EP-34)* Connecting an orphan repository as a project's origin; a repository's
  name, description and topics changed from the tool; the actions run from the command line
  (`crw act` previews only); a registry setting naming the backup job's script; Create pushing
  more than the current branch; the history scan before Make public as a command's preview
  alone (the alternative to EP-42's dialog).
- *(EP-29, from EP-27)* The exports in the window's menus, a File item for the site export and a
  row-menu item for a project's CSV (the commands serve, the owner's choice at the P4 walk); a
  registry or a ledger restored from an export; a JSON file for each table; the daily folders
  kept 30 days, or all of them (the owner chose 14 days and each month's first, EP-41).
- *(EP-32, from EP-42)* A scan before Create a private remote; a pre-commit or pre-push hook
  reading the same `private_words` list (the alternative to rule 3's check by hand, §8's EP-8
  item); a scan of the working tree's files. The review's logged points: the pass's output is
  read whole, and Cancel while it reads leaves git holding the children's turn until it ends (two
  minutes at most); the doctor's stand-in launch has no neighbour, defer route or plugin, so the
  rule's place beside those is built only in the launch's own tests; the registry and the list
  are read on the window's thread at the dialog's opening and at the press.

## 6 Conventions and other projects

*(EP-32, the theme compiled.)* Built in v1: the house style as the schema with the older forms
read only (D-9), owner tasks and the inbox (EP-5), the mailman between projects (EP-20 …
EP-20d), Formalise and Migrate (EP-21, EP-21a), the plugin with its four skills (EP-22, EP-22a).
Parked: the mail's and the formalise's finer forms, ticking the older forms, a split set's name,
and the conventions the walks named (a sort by the Waiting count, the older forms counted by
kind). The routes no real use met in v1 are carried into v2's first walk (EP-44's note).

- **Migration of the three work-packet projects and the five-column project** to the house style: a
  chat-pane task per project, executed by a session in that project with the prompt EP-21 generates
  (D-9). *(EP-23: Migrate… opens that prompt from the window since EP-21a, and the first older
  roadmap was migrated at the P3 walk (D-9 addendum); the others when the owner wants them.)*
- The owner's vault tooling plans a mirror row per DATA repository (checkboxes, decision count, recent
  commits) that overlaps this tool's parser; the vault could consume this tool's daily export instead
  (noted for that project's re-plan). *(EP-29: the daily export exists since EP-27, a folder a
  day under `private/export/`.)*
- Owner-tasks ticking for the legacy forms (they stay read-only, D-10).
- *(EP-8)* Eligibility from the dependency graph, and a split set's name (`EP-5` for `EP-5a` and
  `EP-5b`): the next EP stays the first unchecked row (D-9; left out at EP-3).
- *(EP-23, from EP-20 … EP-20d)* The mail's parked points. A letter keyed by its place, so that a
  rewording replaces it (the words' hash was chosen, so a reworded block is a new letter); a
  composed message held while a session works in the receiver; a block delivered from the working
  tree; a long message kept whole for a revert (the ledger keeps 1,000 characters); a block for a
  project the registry does not know delivered anyway. `crw mail` saying a rewording on its
  delivery's line, and a rewording of a block counted at a first reading said at all: parked at
  the P3 walk until a block is reworded for real (the owner's choice). The reviews' logged points:
  an ignored brief never counts as committed; the past-mail row's count leaves out letters counted
  through a new sender's own first reading; two blocks to one receiver reworded at once each name
  both; a delivery a revert redid under another id is named by its first; the compose preview's
  id is the one at the dialog's opening.
- *(EP-23, from EP-21 and EP-21a)* A formalise that sets the project's registry state (it stays
  the owner's, D-6); formalising an unadopted folder from the window; a migration chained by the
  autopilot or with a Commit message of its own; a revert's redo that replays the logged files
  rather than formalising again; the House style section as a standing nudge on every project a
  formalise would change, or no section at all (the alternatives to EP-38's rule).
- *(EP-23, from EP-22)* The extension's `/install-plugin` link as a way to bring the plugin into
  VS Code sessions (its parameters unverified); a `version` and a listing icon in `plugin.json`
  (`claude plugin validate` warns without them, and its `--strict` would fail).
- *(EP-23, from the P3 walk)* No project's risks list counted as waiting at all, with or without
  an owner-tasks file (EP-38 leaves a project without the file as it is); the opener pointing at
  a project's read-first rule and quoting nothing, which would shorten every opener; a second
  walk in the middle of a phase.
- *(EP-29, from the P4 walk)* The routes no real use has met, staged once on scratch projects
  under a scratch home, or listed at the close-out and no longer asked (the owner chose to carry
  them to EP-32's walk); the Keep pins closed as unexercised; other wording for the plugin's
  four skills (the `replan` skill gained its sentence on the records at EP-29).
- *(EP-29, from EP-38 and EP-39)* A sort by the Waiting count; the older forms counted by kind (a
  parked block counted, a risk not); the Detail pane's `markup::reflow` shared with core's
  `prompt::joined`, which carries a form it does not know (a fence nested in a list item) as
  written.
- *(EP-59, from EP-57)* The manual's freshness read from file times: a picture replaced by a
  copy that keeps an older time is caught only by `crw manual --force`; a hash of the sources
  kept beside the PDF would catch it.

## 7 Stack alternatives considered

*(EP-32, the theme compiled.)* The choices v1 did not take, kept for v2's interview; none was
reopened at the close-out, and every pin stayed (D-15 addendum of 2026-11-17).

- Dioxus 0.7 for one UI codebase on desktop and web; Leptos 0.8 with a Tauri shell; Tauri 2 with a
  TypeScript frontend; Slint 1.18; Iced 0.14 (D-14 chose egui plus server-rendered pages).
- Embedding Tailscale (libtailscale) instead of the running client's Serve (not needed).
- *(EP-23, from EP-17 and EP-20a)* ureq with native-tls, reqwest with rustls or `curl.exe` for the
  usage poll (WinHTTP was chosen, D-14 addendum); ntfy's JSON publishing form and its tags (the
  header form is used).
- *(EP-23)* A Markdown renderer of the tool's own for the chat's replies, grown from `markup` on
  pulldown-cmark, and replies shown as source text (the owner chose `egui_commonmark`, D-15
  addendum); the protocol crate moved at each of its releases rather than at a re-plan.
- *(EP-29)* For the phone pages: Datastar 1.0.5 vendored, or htmx 2.0.11 with its SSE extension
  (the owner chose plain forms and one script of the tool's own, D-14 addendum); the handlers
  reading the registry and the ledger themselves rather than a snapshot the window publishes.
  The protocol crate moved to 2.1.325 in a row of its own while the standalone CLI stays at
  2.1.316 (kept, D-15 addendum).

## 8 Operations

*(EP-32, the theme compiled.)* Built in v1: the crash log (EP-14a), the installed exe with its
restart and handover (EP-37, EP-37a), screenshot runs over a neutral or copied root (EP-36), the
doctor's report and the rolling log (EP-28), the start at login, the reconciliation and `crw` on
PATH (EP-28a). Parked: hooks and checks that would replace a check by hand (a pre-commit word
check, a CLI smoke in CI), the doctor putting right what it finds, and a log viewer.

- Documented AV exclusions for the exe (alternative to D-29's spaced spawns and hints).
- A Windows service for the core with a separate GUI (rejected, D-29).
- An opt-in live smoke test against the real CLI with a tiny prompt (rejected: tests spend no usage).
- Scheduled autopilot windows through the Claude desktop app's machine-local scheduled tasks (observed in
  the docs; not needed while the tool runs at login).
- *(EP-8)* A pre-commit hook script that checks a commit's added lines against the word list kept in
  `private/` (the alternative to CLAUDE.md rule 3's check by hand; D-3 addendum).
- *(EP-8)* A `root=` setup flag for screenshot runs that names a neutral data root, instead of the
  substituted drive and junction EP-8 used for the README's pictures. *(2026-11-08: planned as
  EP-36, which also serves a scratch copy of the real portfolio.)* *(EP-23: built by EP-36.)*
- *(EP-16)* A dialog at the moment of a crash, beside `crash.log` and the next start's line (it
  would need a Windows dependency of its own; D-14's EP-14a addendum).
- *(EP-23, from EP-22 and EP-22a)* A `claude --plugin-dir … --help` smoke in CI (tests never run
  the CLI); the plugin's written copy removed when a build no longer carries it.
- *(EP-23, from EP-36)* A screenshot run over the owner's registry or over the real data root: a
  copy's states are derived, a data repository is never copied or listed, and a copy run's
  sessions, plans and usage stay the fake projects'. The copy script reading a manifest two
  folders down, or a decision log in a folder it skips.
- *(EP-29, from EP-28)* The extension's stale lock files deleted by the tool, or anything else
  the doctor's report finds put right by it (it reads only); a log written by `crw` and the
  hooks beside the window's; a log viewer in the window; the doctor's checks run at every start
  of the window. For the push guard (risk 17): a supervised probe on a scratch repository with
  a scratch remote, the alternative to EP-42's line. *(EP-32: risk 17 stays open into v2, the
  owner's choice; the probe stays here.)*
- *(EP-32, from EP-30)* A test of the router through `tower` (not a direct dependency) in place
  of HTTP to a free port.
- *(EP-43)* The ledger in SQLite's WAL mode, where a reader never waits for another connection's
  commit and only writers wait for each other: the systemic answer to the window's waits on the
  ledger, which EP-43 bounded for the pass's periodic accesses alone (its rollback journal blocks
  every new reader during a commit, and each of the window's statements waits up to five
  seconds). A change to the form of the owner's records (rule 13, D-8): the file's header, two
  files beside it while a connection is open, and every reader of a copy taking the WAL into
  account. Taken up if the rolling log's `slow` lines name the ledger's writes on the window's
  thread (the watch's records, the shares, the samples, the notifications' lines, the chain
  log), which keep their five seconds; moving those writes to a worker of their own is the
  other way.
- *(EP-59, from the P6 walk)* The window's own restart refusals (File > Install and restart, the
  tray's, the launch dialog's) written to the rolling log as the install's are ("Crw install
  asked for a restart: not restarted: …"), so that a walk reads their words from the records:
  the launch dialog's refusal met live in EP-59's walk reached only the status bar.

## 9 Views

*(EP-32, the theme compiled.)* Built in v1: the docked tiles, the dense list with its measured
columns, the legend, the readable notes, the cards and the expanded rows (EP-6, EP-7a, EP-15a,
EP-26a), the Decisions, Sessions, Chat, Usage and Diagnostics panes. Parked: drag-resizable
columns and a sort by a header's click, a transcript in a tile of its own, references that open
what they name, and the popup's further forms.

- Drag-resizable columns in the project list, their widths saved with the layout (the alternative the
  owner did not pick for EP-7a, which sizes the columns to their content; IN-4, 2026-11-01).
- *(EP-8)* Sorting by a click on a column header (EP-7a kept the Sort menu as the one place).
- *(EP-33)* Proposals that expire after a set time, and one list of every dismissal across the
  projects (EP-33 keeps a dismissal until the fact it rests on changes, shown per project in the
  Detail pane and in `crw scan`; D-6 addendum).
- *(EP-13)* A session's transcript in a tile of its own, dragged out of the Sessions pane as an
  egui_tiles pane (EP-13 keeps one tab strip inside the pane, so the saved layout never holds a
  session; D-25 addendum), and a search through a transcript.
- *(EP-16)* From EP-15's out-of-scope list: a question answered in the popup itself (its Show opens
  the Decisions pane), a notification sent by `crw`, and a switch per kind of notification beside
  the quiet hours and Dismiss all (D-21 addendum).
- *(EP-16)* References and links in the Detail pane's notes that open what they name: a brief, a
  decision, a URL (EP-15a's out-of-scope list; D-25 addendum).
- *(EP-20a)* An urgent message's push held during the quiet hours and sent at their end, a queue
  that outlives the window's pass (the owner kept urgent pushes held back like the rest, the
  alternative not picked at EP-20a's close); an ntfy access token setting for a server outside
  the tailnet; ntfy for the badges; the two ntfy settings set from the window.
- *(EP-23, from EP-20a and EP-20c)* An urgent popup for the owner's own message (they make none,
  EP-20a's rule). EP-20a's logged review points: the gist is worked out each frame for each
  ledger card shown, `ntfy_host` refuses an IPv6 address without a port, and a message that
  reuses an id already seen never toasts.
- *(EP-23, from the P3 walk)* Long project names cut with an ellipsis in the list, the title
  keeping its width (the owner chose the title to give way first, D-25 addendum; the
  drag-resizable columns above stay parked).
- *(EP-29, from the P4 walk)* The Brief title shrinking all the way, cut with an ellipsis, before
  any badge column goes (its floor of 80 points stays, the owner's choice again); a double click
  on a row or a card that opens the launch dialog (it brings the Detail pane forward, as Enter
  does); the Diagnostics tab's place, should its title ever sit past the tab bar's scroll arrow
  in a narrow group (it was easy to find in the owner's layout).
- *(EP-45, from the P6 re-cut)* The alternatives the owner did not pick: a fourth theme choice
  beside the stock Dark, and Light rebuilt on the same family; two counts side by side in the
  list (tasks and older forms); a third pane for a project's notes (Decisions, House style,
  Problems), the Detail pane ending at Status exactly; a "Portfolio review" list built into the
  window for a sweep through the projects that are not active (owner tasks guide it instead,
  EP-55); a manual written straight as a PDF with no Markdown source. Named and not cut into
  P6: the usage over the last days, which leaves the Usage pane at EP-46 and returns only as
  v2's usage over time (§2); the phone pages' dark style on the new base and a manual's link
  on the phone's project page, each built only if cheap (EP-49, EP-57); the cards view's title
  line (EP-46 keeps it); an owner-tasks pane that ticks the older forms (§6 keeps them
  read-only).
- *(EP-59, from EP-46 and EP-51a)* One state worded one way: the list's short words for a
  project with no next brief ("all checked", "no roadmap", "no table") against the long ones the
  phone's list and the cards keep. An item's "more…" fold kept across a restart (EP-51a keeps it
  for the window's life).
