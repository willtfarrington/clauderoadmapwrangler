# v2 notes — parked at the v1 planning interview (2026-10-31)

Everything named during planning and not built in v1. Every re-plan EP grows this file; the v1 close-out
(EP-32) compiles it into themes with a v2 direction. Grown at EP-8 (2026-11-01, the P1 re-plan) by the
items marked *(EP-8)*, at EP-9 by the one marked *(EP-9)*, at EP-10 by those marked *(EP-10)*, at
EP-33 by the one marked *(EP-33)*, at EP-11 by those marked *(EP-11)*, at EP-12 by those marked
*(EP-12)*, at EP-13 by those marked *(EP-13)*, at EP-14 by those marked *(EP-14)*, at EP-15b by the
revert's, at EP-16 (2026-11-04, the P2 re-plan) by those marked *(EP-16)*, and at EP-35 by those
marked *(EP-35)*.

## 1 Launch modes

- **Claude's own background sessions (`claude --bg`) as a third launch mode** with Attach in a Windows
  Terminal tab, `claude logs`, and the supervisor's `claude agents --json` state; requires `"worktree":
  {"bgIsolation": "none"}` in each managed project's `.claude/settings.json` so roadmap edits stay in the
  main tree; such sessions are interactive, so Remote Control reaches them from the phone (round 7, round
  17: not picked).
- **Worktree per session** and **diff review** inside the tool (recurring in other orchestrators; the
  owner's process edits the main tree and reviews in VS Code).
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
  as EP-35, for a headless run a crash cut short too.)*
- *(EP-13)* Resume of a headless session after the window restarted, from its ledger row: its
  launch's permission mode and neighbours would need keeping in the ledger (D-16 addendum).
  *(EP-35: VS Code's extension opens a headless run's id as an empty tab, so this is the window's
  only way back to such a run; `claude --resume <id>` in a terminal meanwhile.)*
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

## 2 Analytics

- **Usage over time**: weekly and Fable percentages charted from the samples with per-project share
  (round 13, round 17: not picked). The samples are recorded from EP-17, so the graph is cheap later.
- Parsing transcripts for token counts per project (the approach of existing usage tools; the format is
  declared internal and changes between releases) — rejected for v1 (D-23, D-26).
- Transcript copies kept beyond the 30-day sweep (rejected, D-3).

## 3 Chat

- **Portfolio report**: a Markdown report across projects written into `private/` on request (round 14,
  round 17: not picked).
- A full-tool chat session in `DATA` (rejected for blast radius, D-27).

## 4 Phone view

- **Free chat from the phone** (round 15, round 17: not picked; a lost device would reach a Claude Code
  session).
- An eframe-wasm phone view served by the same server (egui 0.36 improved mobile IME; the
  keyboard-covers-field issue #4572 was open on 2026-10-31).
- A PIN for write actions and periodic re-authentication (alternatives to D-28).
- A path mount under the other project's existing hostname instead of a new HTTPS port.

## 5 Hosting and backup

- Deleting the orphan private repository from the tool (needs the `delete_repo` scope; D-24 flags it with
  Archive and Delete actions and leaves the scope refresh to the owner).
- A second backup mechanism (rejected; the Documents job is the one, D-24).
- Health badges beyond the latest Actions run and licence presence (CodeQL, Dependabot alerts).
- *(EP-8)* Rewriting this repository's history to drop the planning record's first wording of the
  owner's machine details (rejected before the public flip: every recorded hash would change; D-3
  addendum).

## 6 Conventions and other projects

- **Migration of the three work-packet projects and the five-column project** to the house style: a
  chat-pane task per project, executed by a session in that project with the prompt EP-21 generates
  (D-9).
- The owner's vault tooling plans a mirror row per DATA repository (checkboxes, decision count, recent
  commits) that overlaps this tool's parser; the vault could consume this tool's daily export instead
  (noted for that project's re-plan).
- Owner-tasks ticking for the legacy forms (they stay read-only, D-10).
- *(EP-8)* Eligibility from the dependency graph, and a split set's name (`EP-5` for `EP-5a` and
  `EP-5b`): the next EP stays the first unchecked row (D-9; left out at EP-3).

## 7 Stack alternatives considered

- Dioxus 0.7 for one UI codebase on desktop and web; Leptos 0.8 with a Tauri shell; Tauri 2 with a
  TypeScript frontend; Slint 1.18; Iced 0.14 (D-14 chose egui plus server-rendered pages).
- Embedding Tailscale (libtailscale) instead of the running client's Serve (not needed).

## 8 Operations

- Documented AV exclusions for the exe (alternative to D-29's spaced spawns and hints).
- A Windows service for the core with a separate GUI (rejected, D-29).
- An opt-in live smoke test against the real CLI with a tiny prompt (rejected: tests spend no usage).
- Scheduled autopilot windows through the Claude desktop app's machine-local scheduled tasks (observed in
  the docs; not needed while the tool runs at login).
- *(EP-8)* A pre-commit hook script that checks a commit's added lines against the word list kept in
  `private/` (the alternative to CLAUDE.md rule 3's check by hand; D-3 addendum).
- *(EP-8)* A `root=` setup flag for screenshot runs that names a neutral data root, instead of the
  substituted drive and junction EP-8 used for the README's pictures. *(2026-11-08: planned as
  EP-36, which also serves a scratch copy of the real portfolio.)*
- *(EP-16)* A dialog at the moment of a crash, beside `crash.log` and the next start's line (it
  would need a Windows dependency of its own; D-14's EP-14a addendum).

## 9 Views

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
