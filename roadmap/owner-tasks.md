# Owner tasks — what only the owner can do

One line per task (D-10): `- [ ] OT-n (EP-k, date) text`, optionally `before EP-m`. Sessions append
at completion; the owner ticks in the tool (the Detail pane, from EP-6) or here.

- [x] OT-1 (EP-6, 2026-11-01) Open Roadmap Wrangler from its shortcut once and set it beside
  `crw scan`: every registered project in its state group with the next EP `crw roadmap` names, and
  the status bar's "Asking git" counting through the repositories to "Up to date" (about 15 s). The
  session could check the window only on the fake portfolio (rule 5) [before EP-7]
- [x] OT-2 (EP-7, 2026-11-01) Open Roadmap Wrangler once and look at its badges: the first start
  creates the ledger (`%LOCALAPPDATA%\clauderoadmapwrangler\ledger.sqlite`, the GitHub cache) and
  the status bar counts "Asking GitHub: n of 9" after git; the row's badges and the Detail pane's
  Status section should agree with `target\release\crw.exe status`, the backup badge now reading
  the job through the two settings set at EP-7's close. Widen the window to see the full badge form.
  The session checked the window only on the fake portfolio (rule 5)
- [x] OT-3 (EP-7a, 2026-11-01) Open Roadmap Wrangler on the real portfolio and look at EP-7a's
  polish: each column as wide as its widest value under a header row (no chip clipped), the badges
  loosening as the window widens and the optional columns giving way as it narrows, the Legend
  button in the Projects toolbar opening the key at the foot of the pane, and the Detail pane's
  Status section with each name level with its first line and dividers between the subsections.
  Anything to adjust goes to `roadmap/inbox.md` before EP-8 takes the README's pictures; the gaps'
  share of a wide window is one constant (`list.rs`). The session checked only the fake portfolio
  (rule 5) [before EP-8]
- [x] OT-4 (EP-10, 2026-11-02) Open Roadmap Wrangler, select a project and press S (or its row's
  Start button): the launch dialog should show the model, effort, permission mode and prompt that
  `target\release\crw.exe prompt <project>` prints, and its Claude line the installed CLI, logged
  in. A first real headless run is optional and spends usage: Start headless, then watch the
  Sessions pane; a request waits in the Decisions pane until EP-14, Stop ends the session, and the
  ledger keeps its row. Notes for the dialog go to `roadmap/inbox.md`. The session checked the
  dialog only on the fake portfolio and started no real session (rule 5) [before EP-11]
- [x] OT-5 (EP-11, 2026-11-02) Try a Visual launch from Roadmap Wrangler on a project whose VS
  Code window is closed: select it and press S; the dialog opens on Visual, its VS Code line
  saying "no window has it open: one opens"; press Open in VS Code. The project's window should
  open from its own `<name>.code-workspace` and, once the Claude Code extension is up (within a
  minute), a new Claude Code tab should hold the prompt, unsent: choose the model and effort in
  its panel, then send it or close the tab. If VS Code asks "Allow Claude Code extension to open
  this URI?", tick "Do not ask me again" and Open. The session tried only a window already open,
  its own, with a test prompt (rule 5). Notes go to `roadmap/inbox.md` [before EP-12]
- [x] OT-6 (EP-12, 2026-11-02) Open Roadmap Wrangler from its shortcut and hover over "Up to date"
  in the status bar: it should end "Signals: listening on 127.0.0.1:47831; hooks installed, status
  line installed". Then open a new Claude Code tab in a registered project's VS Code window: within
  a few seconds the Sessions pane should list it under "Started elsewhere" as manual and idle, the
  project's row should show a session in progress, and a project with open inbox messages should
  have them printed into the new session as the stopgap hook did. Send a prompt (busy within 5 s,
  from the poll); a permission prompt should show it waiting, in amber; closing the tab ends it
  (done, gone from the pane after ten minutes). The session checked the window only on the fake
  portfolio and the relays on a test port (rule 5). The stopgap script
  `~\.claude\hooks\session-start-inbox.ps1` is no longer run (retiring it is your environment
  repository's next brief); `target\release\crw.exe signals remove` takes the tool's entries out.
  Notes go to `roadmap/inbox.md` [before EP-13]
- [x] OT-7 (EP-13, 2026-11-02) Open Roadmap Wrangler from its shortcut: its first start brings the
  ledger to schema 3 and closes the two visual launches from before the hooks, which
  `target\release\crw.exe sessions` then lists as lapsed (before the window, it already reads them
  so, without writing). In the Sessions pane's All tab, try the project, mode and Running only
  filters over the ledger's rows. Open a new Claude Code tab in a registered project's window: it
  should show as new, with no row and no session in progress; send a prompt, and within 30 s (at
  once if you switch to the window) it turns busy, in blue, the project's row shows it in progress,
  and `crw sessions` shows its row from its start. A real headless run is optional and spends
  usage: Start headless from the launch dialog, watch its tab's transcript, Stop it, then type a
  message and Resume; the ledger keeps a row per run. The session checked the tile only on the fake
  portfolio and the ledger's rule on scratch ledgers (rule 5). Notes go to `roadmap/inbox.md`
  [before EP-14]
- [x] OT-8 (EP-14, 2026-11-02) Open Roadmap Wrangler from its shortcut and look at the Decisions
  tab: with nothing waiting it says so. Optional, spends usage: start a small brief headless (S,
  Start headless) whose work asks you something; its question waits in the Decisions tab with its
  options and an Other field (Send answers, or Decline with a reason), a permission request with
  its input and Allow or Deny, and the session goes on at once, its transcript in the Sessions pane
  showing your answer under each request. To try the time limit, run
  `target\release\crw.exe set <project> answer_timeout 1` first and `... answer_timeout none`
  afterwards: a request left a minute defers the run, and Answer and resume in the Decisions tab
  picks it up. A Claude Code tab in VS Code that waits on a permission prompt shows under Waiting
  elsewhere with Focus, which brings its window forward. The session checked the tile on the fake
  portfolio and the routes with a supervised probe of throwaway sessions (rule 5). Notes go to
  `roadmap/inbox.md` [before EP-15]
- [x] OT-9 (EP-14a, 2026-11-03) Reopen Roadmap Wrangler from its shortcut and use it as before:
  nothing new shows unless a panic happened. Should the window ever vanish again, reopen it at once.
  Its status bar's red line then names the panic once (the time, the message's first line, the file
  and line), and `crash.log` in `%LOCALAPPDATA%\clauderoadmapwrangler` holds the whole report with
  its backtrace. Put the red line into `roadmap/inbox.md`, so the next session starts from the
  cause. A headless run the window held is recorded as stopped. Claude Code's own list of past
  sessions in that project (`claude --resume` there) picks it up
- [x] OT-10 (EP-15, 2026-11-03) Keep Roadmap Wrangler in its tray for a few sessions and note what
  its popup gets wrong. A notification should come at the screen's lower right when a session
  waits for you (a headless run's question or permission while the window is not in front, a VS
  Code session's permission prompt), when a run finishes or fails, and when a VS Code session ends
  a turn of two minutes' work or more; none for a session you close yourself. Show brings the
  window forward on the Decisions pane or the session's tab, or the VS Code window a session waits
  in; notifications stay until dismissed, and the popup should never take your keyboard focus as
  it appears. The icon's red count is the Decisions tab's; its amber dot marks an unadopted folder,
  a remote or a backup to look at, or an inbox message new since the window was last in front.
  `target\release\crw.exe notifications` lists what popped up, what was held back and what was
  opened; `crw setting quiet_hours 22:00-07:00` holds notifications back at night (`none` undoes
  it). Windows 11 puts a new tray icon in the overflow by the clock: drag it onto the taskbar, or
  turn it on under Settings, Personalization, Taskbar, Other system tray icons. Notes go to
  `roadmap/inbox.md` [before EP-16]
- [x] OT-11 (EP-15a, 2026-11-03) Over your next sessions, look at the Detail pane on your other
  projects and note any brief note, owner item or inbox message that it sets wrong. The tests see
  only synthetic notes, and each project writes its own forms. Examples: a label or a numbered part
  split where it should not be, a sentence cut in the middle, a code span in the wrong colour, a
  form drawn as one block where it has parts. Name the project and the item, and say what it
  should look like. Notes go to `roadmap/inbox.md` [before EP-16]
- [x] OT-12 (EP-15b, 2026-11-04) Reopen Roadmap Wrangler from its shortcut and, after your next
  session in a project other than this one, try the preview: `target\release\crw.exe revert
  <project>` prints what undoing that session would do and changes nothing (`--yes` would revert
  it; `crw sessions` shows each session's short id). A session that started while the window ran
  shows "the project's HEAD at the session's start" as its target. Each session start in a
  registered project now records the project's HEAD and copies its uncommitted files that git does
  not ignore into `%LOCALAPPDATA%\clauderoadmapwrangler\backups\<project>\starts\` (at most 200
  files and 20 MB a start). At your word the session turned this off on your three data
  repositories (`crw set <project> revert no`; `yes` turns it on again). The session tried the
  revert on a scratch project only (rule 13). Notes go to `roadmap/inbox.md` [before EP-15c]
- [x] OT-13 (EP-15c, 2026-11-04) Reopen Roadmap Wrangler from its shortcut and, after your next
  session in a project other than this one, right-click that project's row and choose "Revert the
  latest session…". The dialog reads the project for a second or two, then shows what undoing the
  session would do: its target, each commit and file, what stays, and where the work would be
  copied. Press Cancel, and nothing changes. The same preview opens from Revert… on the session's
  card in the Sessions pane's All tab, and from an ended headless session's tab beside Resume. On
  your three data repositories, whose revert is off, these are greyed out and say why on hover.
  While a session works or waits in the project, the preview is refused and says why. Revert, if
  you press it, reads the project again first and runs only if nothing changed since the preview;
  the work is then copied to `%LOCALAPPDATA%\clauderoadmapwrangler\backups\<project>\reverts\`.
  The session checked the dialog only on the fake portfolio and reverted only scratch
  repositories (rules 5 and 13). Notes go to `roadmap/inbox.md` [before EP-16]
- [x] OT-14 (EP-18, 2026-11-04) Start EP-18a from the window, the way every EP of this project
  starts now (rule 11), instead of typing the request into a tab by hand. (1) Open Roadmap
  Wrangler from its shortcut, or from its tray icon by the clock. (2) In the Projects pane,
  select clauderoadmapwrangler's row, whose Next column should read EP-18a, and press S (or the
  Start button at the row's right end). (3) The launch dialog opens on Visual, the default. Its
  prompt begins "Run the next brief of clauderoadmapwrangler: EP-18a", and Model, Effort and
  Ultracode are greyed out, since the new tab's panel chooses them; the Model line names the
  rules' suggestion and, below it, how the Fable downshift judged the brief. (4) Press Open in VS
  Code. This project's VS Code window comes forward, and within a minute a new Claude Code tab
  holds the prompt, unsent. (5) In that tab's panel, choose the model and effort the opener
  suggests, then press Enter to send the prompt as it is. A request typed into a tab opened by
  hand is recorded as manual, and the session cannot show it came from the tool. (6) The
  session's first step finds its row: `target\release\crw.exe sessions clauderoadmapwrangler`
  lists it as visual, EP-18a, with its session id, once its first turn has run. If a step goes
  another way, put what you saw in `roadmap/inbox.md` [before EP-18a]
- [x] OT-15 (EP-18a, 2026-11-05) Reopen Roadmap Wrangler from its shortcut; its first start
  brings the ledger to schema 5, after which an older build refuses it. Keep it running through
  your next session in any registered project, manual or launched, then run
  `target\release\crw.exe sessions <project>`. The session's line should end with its share,
  such as `week +0.5 % (27 → 28 %)`, once the window has polled at least twice while it worked
  (a poll every 5 minutes). The line under the header gives each project's sum over the week
  since the reset, which can stay `+0 %` for a short session, since the endpoint moves in whole
  points. The same share shows on the session's card in the Sessions pane's All tab. The session
  checked this only on scratch ledgers and the fake portfolio (rules 5 and 13). Notes go to
  `roadmap/inbox.md` [before EP-18b]
- [x] OT-16 (EP-18b, 2026-11-05) Reopen Roadmap Wrangler from its shortcut. The Projects pane
  has a Usage column after Backup: each project's points of the weekly limit since the weekly
  reset, such as `+1 %`. A weak `+0 %` marks a project whose sessions worked through no rise, and
  a project without a share this week shows nothing. The figures should match the sums that
  `target\release\crw.exe sessions` prints at its top. Hover a figure for its sentence, which
  adds its Fable sum and its part of all that was attributed. Select the project for the same
  sentence at the foot of the Detail pane's Status section, under Usage. Sort, Usage share orders
  each group by the figures, the largest first. The session checked this only on the fake
  portfolio (rules 5 and 13). Notes go to `roadmap/inbox.md` [before EP-35]
- [x] OT-17 (EP-35, 2026-11-05) Reopen Roadmap Wrangler from its shortcut and open the Sessions
  pane's All tab. Each ledger card of a visual or manual session has Resume in VS Code beside
  Revert…, and the card's figures line ends with the session's full id. Press it on an ended
  session of this project, such as EP-18a's: this project's VS Code window should come to the
  front with a Claude Code tab holding that conversation. Close it without sending, or write in
  it to go on: that run then gets a visual ledger row of its own, which
  `target\release\crw.exe sessions clauderoadmapwrangler` shows. A headless card's button is
  greyed out, and its hover text names `claude --resume <id>` for a terminal. The session checked
  the window only on the fake portfolio (rules 5 and 13). Notes go to `roadmap/inbox.md`
  [before EP-19]
- [x] OT-18 (EP-19, 2026-11-05) Reopen Roadmap Wrangler from its shortcut. The Sessions pane's
  All tab opens with the Autopilot section: off everywhere, not paused, and Pause beside it; the
  tray's menu has Pause autopilot, and `target\release\crw.exe autopilot` prints the same state.
  The list's autopilot box (or A on a selected row) now turns a project's autopilot on, and its
  hover says what that means: the project's sessions then commit themselves, never pushing. To
  try a chain (optional: it spends usage and commits in that project), choose a project you trust
  whose next two briefs are S or M, write `permission_mode = "auto"` under its `[project.launch]`
  in the registry, tick its box, and start its next brief headless from the launch dialog. When
  the run completes, the section's log says whether the gate chained the next brief, or each
  condition that refused it. Untick the box afterwards if you do not want it on. The session
  checked the window only on the fake portfolio (rules 5 and 13). Notes go to
  `roadmap/inbox.md` [before EP-19a]
- [x] OT-19 (EP-19a, 2026-11-06) Reopen Roadmap Wrangler from its shortcut. The next time you
  start a brief headless in a project whose autopilot is on (its box in the list, or A), choose
  Headless in the launch dialog: a Chain plan shows under the prompt, one row per brief the
  chain may run after this one, each with a Chain box, the Model and Effort pickers and the
  Ultracode box, the rules' choice filled in. Set what you want for each row, or untick a row
  to stop the chain before it, then press Start headless. When the run completes and the gate
  passes, the chain starts the next brief with your choice, and the chain log's line ends
  "your choice" (or "the rules' choice" for a row you left alone); the Sessions pane's
  Autopilot section shows the plan while the chain runs, and `target\release\crw.exe autopilot`
  prints the log. Each change of a chain (a brief chained, a hold, an end) now comes as a
  notification of its own in the popup, whose Show opens the Sessions pane's All tab. The
  session checked the dialog, the section and the popup only on the fake portfolio and started
  no real session (rules 5 and 13). Notes go to `roadmap/inbox.md` (the gate `before EP-19b`
  taken off on 2026-11-08 under the D-10 addendum of that date: EP-19b does not build on it)
- [x] OT-20 (EP-37a, 2026-11-10) The first real chained restart, when you next want this
  project's briefs to chain. In Roadmap Wrangler, tick clauderoadmapwrangler's Autopilot box in
  the list (or A on its row). The window offers permission mode auto, since its headless runs
  would otherwise wait at each shell command: accept it. With the box ticked, every session of
  this project, a visual one too, commits in the house pattern by itself and never pushes. Then
  press Start on its row, choose Headless, set the chain plan's rows as you want them, and press
  Start headless. When the brief's run ends with its install, the window should restart by
  itself: the chain log (the Sessions pane's Autopilot section, or `target\release\crw.exe
  autopilot`) says `held after EP-n for the window's restart into the installed build`; the
  window closes and comes back (hidden if it was in the tray), its notice reading "Restarted
  into the installed build of …, carrying 1 held chain …"; and within a minute or two the next
  brief starts in the new window (`chained EP-m after EP-n`). If the restart was refused, the
  log says `the restart into the installed build waits: …` and the chain goes on in the old
  window. The session exercised this only with stand-ins for the process start (rules 5 and
  13). Notes go to `roadmap/inbox.md`
- [x] OT-21 (EP-20, 2026-11-10) The first real delivery between projects: the window now writes
  into another project's `roadmap/inbox.md` by itself. Once the window runs EP-20's build (after
  its restart), `target\release\crw.exe mail` should print a `first reading` line for each
  registered project within a minute of the first poll of live sessions, with a `counted` line
  for each `Handed on (to …)` block already written, and no `delivered` line: nothing written so
  far is sent, so a block you have not carried yet still needs your hands. Then, the next time a session of any project ends with a `> **Handed on (to
  <project>).**` block to a registered project, once its brief is committed and no session works
  in the receiving project, the window's status bar says "Delivered IN-n to <project>, from
  <sender> EP-k", the receiving project's `roadmap/inbox.md` ends with that line (a copy of the
  file as it was is in `%LOCALAPPDATA%\clauderoadmapwrangler\backups\<project>\`), its row shows
  the open message, and the sender's Detail pane says "delivered to <project> as IN-n". The
  session exercised this only on scratch folders (rules 5 and 13). Notes go to `roadmap/inbox.md`
- [x] OT-22 (EP-21, 2026-11-10) The first real formalise, when you next want a project brought
  into the house style (it writes into that project). In a terminal in this folder, run
  `target\release\crw.exe formalise <project>`: it changes nothing and lists the four files, each
  `created`, `the rule block added at the end, the file's n lines kept`, `replaced between the
  markers` or `kept`, then shows the block and each new file's text. If they read as you want, run
  it again with `--yes`, with no session working in that project: it prints `wrote …` for each
  file and where the copy of `CLAUDE.md` went (`%LOCALAPPDATA%\clauderoadmapwrangler\backups\<project
  folder>\`). The project's `CLAUDE.md` should end with the block between the two
  `roadmap-wrangler` markers, every line above it as it was; a third run should say `Nothing to
  write`; and the window's row (after F5) should show no new problem. For a project whose tables
  are the five-column or work-packet form, `--migration` prints the prompt to hand to a session
  there. The session wrote only into scratch folders (rules 4 and 13). Notes go to
  `roadmap/inbox.md`
- [x] OT-23 (EP-22, 2026-11-10) Install the roadmap-wrangler plugin into your own Claude Code
  sessions (it writes your user settings). First push this project's commits (the plugin must be
  on GitHub). Then, in any terminal, run `claude plugin install roadmap-wrangler --marketplace
  willtfarrington/clauderoadmapwrangler`: it should add the marketplace `clauderoadmapwrangler` to
  your user settings and say the plugin is installed; `claude plugin list` should show
  `roadmap-wrangler@clauderoadmapwrangler` enabled. In a new session of another project (VS Code or
  a terminal), typing `/roadmap-wrangler:` should offer `plan`, `replan`, `formalise` and `status`,
  and `/roadmap-wrangler:status` should report that project's next EP, its counts and its open
  items as `target\release\crw.exe roadmap <its folder>` prints them. Later pushes reach your
  sessions with `claude plugin update roadmap-wrangler@clauderoadmapwrangler` (Claude Code does
  not update this marketplace by itself unless you turn its auto-update on under `/plugin`). The
  session checked the plugin with `claude plugin validate` only (rules 5 and 13). Notes go to
  `roadmap/inbox.md`
- [x] OT-24 (EP-20a, 2026-11-10) The first real push to your phone through ntfy, when you want the
  window's notifications there (it sends them to your ntfy server). Once the window runs EP-20a's
  build (File > Install and restart, or the next gate's moment), run in a terminal in this folder
  `target\release\crw.exe setting ntfy_host <your server>` (its name or address on the tailnet,
  with `:port` if it needs one, and `http://` in front if it has no HTTPS), then
  `target\release\crw.exe setting ntfy_topic <topic>` (letters, digits, `-` and `_`): the second
  should print `note: a window that runs pushes each notification its popup shows to <server> …`.
  Subscribe to the topic in the ntfy app on your phone. In the window, File > Send a test
  notification: the popup shows it, the status bar adds `It goes to ntfy at <server> too`, and
  within seconds your phone shows "Roadmap Wrangler" with "A test notification: Show brings the
  window forward."; the status bar's hover then says `ntfy: the last push went at <time>`.
  `target\release\crw.exe notifications` should list `ntfy · test · Roadmap Wrangler`; a `ntfy
  failed · test · Roadmap Wrangler · <why>` line says what to fix.
  From then on each notification the popup shows reaches the phone too, held back in the quiet
  hours and while the window is in front with the focus; `none` for either setting turns it off.
  The session pushed only to a listener on this machine (rules 5 and 13). Notes go to
  `roadmap/inbox.md`
- [x] OT-25 (EP-20b, 2026-11-10) Install the prompt hook (it writes your user settings). Once the
  window runs EP-20b's build (File > Install and restart, or the next gate's moment), the status
  bar's Signals item says `hooks installed but not the prompt hook (crw signals install adds it)`.
  In a terminal in this folder, `target\release\crw.exe signals` should list `UserPromptSubmit not
  installed` and, under what an install would change, `UserPromptSubmit hook added: <the installed
  crw.exe> hook`. Run `target\release\crw.exe signals install`: it should print that one change
  and where the copy of your settings went (`%LOCALAPPDATA%\clauderoadmapwrangler\backups\claude-settings\`),
  after which `crw signals` says `The entries are installed and run the installed copy.` Then, in
  a new VS Code session of any project, a prompt that names another registered project as
  `@<project>` (say, "What is next in @<project>?") should be answered with that project's next EP
  and its rules in view, and the tab should read busy in the window as soon as you send it.
  `crw signals remove` takes the tool's entries out again. The session ran the relay only on a
  scratch registry (rules 5 and 13). Notes go to `roadmap/inbox.md`
- [x] OT-26 (EP-24, 2026-11-11) The first real chat conversation (the first run of a new route). Once
  the window runs EP-24's build (the next gate's moment, or File > Install and restart): in a
  terminal in this folder, mark each data repository with `target\release\crw.exe set <project> data
  yes` (it writes `data = true` into your registry; `crw scan` lists the projects), then in the
  window's Chat pane (the tab beside Sessions on the right) type a question such as "Which project has
  the most open owner tasks?" and press Send (or Ctrl+Enter). The header should say `claude-opus-5-5
  at medium · reads N projects, M left out · read-only` with M the data repositories marked (hover it
  for the names and the digest's file under `%LOCALAPPDATA%\clauderoadmapwrangler\chat`), the
  reply should stream in within a minute as plain text (as Markdown from EP-24a's build, which
  also lists the conversation in the pane and answers the session's questions there), and
  `target\release\crw.exe chats` should
  list the conversation with 1 turn (`crw sessions chat` its run). Then ask it which folders it can
  read and whether it can read a data repository by its path: it should name the projects given and
  nothing else, and a refusal shows in the pane in red. If a reply says a tool is missing (Skill,
  AskUserQuestion) or the session starts in a mode other than manual, note the Init line the pane
  shows on hover of the state chip. From EP-24b's build the session also runs the tool's own
  reading commands (ask it what waits on you in a project: a `crw tasks <project>` line should
  show in the exchange, and a refused command shows in red), a reply may end in buttons under it
  (ask it to formalise a project or to compose a message into one: the button should open the
  window's own dialog with the project filled in, writing nothing until you confirm there), and
  `@<project>` in a message should give it that project's next row and rules (hover the header's
  "reads" line to see the mention named). Notes go to `roadmap/inbox.md`
