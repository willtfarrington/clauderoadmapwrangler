# clauderoadmapwrangler

A Rust desktop dashboard and session launcher for a portfolio of software projects developed with
Claude Code as session-sized roadmap briefs. One window shows every project's next brief with its size
and badges, starts a session visually (the VS Code extension) or headless with a chosen model and effort,
routes a headless session's questions to a decisions pane, keeps the owner's inter-session tasks and a
cross-project inbox, chains briefs on autopilot within caps, meters the account's usage, shows hosting and
backup status and history graphs, hosts a chat to Claude Code, ships the roadmap method as a plugin, and
ends with a phone view over a Tailscale tailnet.

**Status:** phases P1, the read-only dashboard, and P2, sessions, are done (2026-11-01 and
2026-11-04). The Cargo workspace, `crw doctor`
and CI (EP-1), the registry of managed projects with discovery and `crw scan` (EP-2), the roadmap parser
with `crw roadmap` (EP-3 the phase tables, EP-4 the briefs, the decision log and the HEAD-plus-working-tree
snapshot that tells a session in progress), the owner's tasks, the cross-project inbox and the older forms
of the owner's to-dos with `crw tasks` and `crw tick` (EP-5), the dashboard's shell (EP-6: the docking
layout, the project list grouped by state, the detail pane, ticks and Adopt from the window) and its
badges with `crw status` (EP-7: stack, progress, activity, hosting with a check of the origin's head, CI
and license, backup age; EP-7a: columns sized to their content under a header row, and a legend of every
mark) exist; the P1 re-plan (EP-8) measured the phase and opened this repository. In phase P2, the
session runner (EP-9) starts a headless Claude Code session in a project's folder with
the project's neighbour folders read-only, reads its stream-json, answers its questions and permission
requests, stops it with its whole process tree and records it in the ledger; the launch dialog (EP-10)
generates the house-style opener for a project's next brief, resolves its model, effort and permission
mode through a rule table with the reason for each, and starts the session from the window, which
holds it. Adopt writes a new project's own
single-folder VS Code workspace file beside it, and a proposed state can be dismissed until the fact
it rests on changes (EP-33). The dialog launches visually too (EP-11): it opens the project's own
workspace file in VS Code and fills the prompt into a new Claude Code tab of that window, for the
owner to send. The window listens on the loopback for Claude Code's hooks and status line, which
`crw signals install` sets up in the user settings, and asks Claude Code for its live sessions, so
the sessions started elsewhere show beside its own (EP-12). The Sessions pane is a tile (EP-13): a tab
per headless session with its transcript as it streams, Stop and Kill, Resume with a message, and a
list of every session with the ledger's rows, which `crw sessions` prints too. The Decisions pane is
a tile too (EP-14): a headless session's questions and permission requests are answered there while
it waits, a project may give them a time limit after which the run is deferred until the answer
resumes it, a documented defer-and-resume route stands in when a CLI release breaks the control
protocol, and a session waiting in VS Code is listed with Focus. A panic leaves its report in
`crash.log` in the local folder, and the next start names it in the status bar (EP-14a). A popup of
the window's own tells the owner when a session waits for them, has finished or has failed, a tray
icon counts what waits, and closing the window leaves it in the tray with its sessions going on
(EP-15). Each session's start is recorded: the project's HEAD, its branch and a copy of its
uncommitted files, kept in the backups folder. `crw revert` puts a project back as it was when a
session started, after copying the session's work out; it never pushes, and it is refused while
a session works there or when the remote has one of the session's commits (EP-15b). The window
previews the same revert in a dialog, from an ended session's tab, a ledger row of the Sessions
pane or a project's row menu, and reads the plan again before it reverts (EP-15c). The P2 re-plan
(EP-16) measured the phase and made the switch planned from the start: from EP-17 on, this
project's own briefs are started from the tool, as visual launches by default. The roadmap is
[`roadmap/README.md`](roadmap/README.md), the decisions
[`DECISIONS.md`](DECISIONS.md), the session rules [`CLAUDE.md`](CLAUDE.md).

![The dashboard in the light theme on a wide window: the projects grouped by state, each row with its next brief, progress, owner items and badge columns under a header row, and the Detail pane of the selected project with its brief notes, status, owner tasks and inbox](docs/dashboard-light.png)

![The dashboard in the dark theme: a row expanded to its phases, the legend of every mark open under the list, and the Detail pane of the selected project](docs/dashboard-dark.png)

*Both pictures show the fake portfolio tracked in `crates/app/fixtures/portfolio`, taken by the app of
its own window (`--features screenshot`, below).*

## Building

Windows 11 with Rust 1.98 (MSVC) and, for the features that drive them, the Claude Code CLI, the GitHub
CLI `gh`, VS Code with the Claude Code extension, and Tailscale.

```powershell
cargo build --release                   # target\release\clauderoadmapwrangler.exe and crw.exe
pwsh -File scripts\install-shortcut.ps1 # rebuilds, then a desktop shortcut and a Start-menu entry
target\release\crw.exe doctor           # versions of claude, gh, tailscale, code and the extension,
                                        # and whether claude would start a headless session
target\release\crw.exe scan             # the registry: projects, unadopted folders, proposals
target\release\crw.exe dismiss <project>          # leave its proposals out until their facts change
target\release\crw.exe roadmap <folder> # a project's snapshot: the next EP, its brief notes and brief,
                                        # the decision log, whether a session is in progress
target\release\crw.exe tasks <folder>   # what waits on the owner: tasks, inbox, handed-on blocks
target\release\crw.exe tick <folder> OT-n         # tick an owner task
target\release\crw.exe tick <folder> IN-n <text>  # tick an inbox message with its disposition
target\release\crw.exe status [--offline]         # every project's badges; asks git, gh, the origins
target\release\crw.exe setting <key> <value>      # set a registry setting, such as backup_settings
target\release\crw.exe prompt <folder> [--visual|--headless]
                                        # what a launch of the next brief starts from: the launch
                                        # mode, model, effort and permission mode with their
                                        # reasons, the prompt worded for the mode
target\release\crw.exe sessions [<folder>] [--all]  # the ledger's sessions, the newest first; reads only
target\release\crw.exe revert <project> [<id>]    # preview undoing its latest session, or the one named
target\release\crw.exe revert <project> <id> --yes  # copy the work out, then put the project back
target\release\crw.exe set <project> revert no    # record no session starts there, and allow no revert
target\release\crw.exe notifications [--all]      # the window's notifications and badges; reads only
target\release\crw.exe setting quiet_hours 22:00-07:00  # hold notifications back at night; none undoes it
target\release\crw.exe signals          # the hooks and status line in Claude Code's user settings,
                                        # whether a window listens, what an install would change
target\release\crw.exe signals install [--inbox]  # write them, after a dated copy of the file
target\release\crw.exe signals remove   # take them out again
```

The tool keeps its registry, a hand-editable `registry.toml`, in `%APPDATA%\clauderoadmapwrangler`, and
its ledger and the copies it makes before changing a file in `%LOCALAPPDATA%\clauderoadmapwrangler`;
nothing is kept in a project. The registry lists the folders of one data root (by default the folder
that holds this repository); `crw adopt` registers a folder, writes its own single-folder
`<folder>.code-workspace` beside it unless one is there, and adds it to the aggregate VS Code workspace
file; `crw ignore` hides one, `crw set` changes a project's state, priority, pin or time limit for
its sessions' requests, and `crw dismiss`
leaves a proposed state out until the fact it rests on changes (`crw undismiss` brings it back). Set
`CRW_HOME` to a scratch folder to try the commands without touching those folders.

The window (`clauderoadmapwrangler.exe`, the "Roadmap Wrangler" shortcut) shows the registry's projects
grouped by state, each with its next brief, its progress, what waits on the owner, its badges and
whether a session is in progress, in columns as wide as their widest value under a header row; the
Legend switch shows a key to every mark; a click anywhere on a row selects it and a right click opens
its menu. The detail pane shows the selected project's brief notes, status, owner items, inbox,
handed-on mail and problems, the notes set for reading: each `From …:` segment under its label,
numbered parts as a list, lines of about a hundred characters, and labels, references, gates and
runnable code in three colours of the window's, a long owner item a sentence to a bullet with its
gate as a chip beside its id.
It reads each project's files at once and adds what git says one
repository at a time; GitHub (through `gh`) and each origin's head are asked daily and on F5, the
answers cached in the ledger, and nothing is ever fetched. The backup badge reads an existing backup
job's settings file and run log, named by the `backup_settings` and `backup_log` settings. It writes
only what the tool owns: the registry (Adopt, Ignore, a confirmed or dismissed proposal, pin,
priority), the workspace files Adopt writes (and a visual launch, when a project's own one is
missing), the ledger's rows and the ticks in a project's `owner-tasks.md` and `inbox.md`. S,
or a row's Start button, opens the launch dialog on the selected project's next brief: the
generated prompt in an editor, a Visual | Headless switch, and the model (from
the catalog Claude Code caches), the effort, ultracode and the permission mode, each with the reason
the rule table gave it. A project's `[project.launch]` in the registry sets its defaults (`mode`
among them) and a prompt `template`; `[[rule]]` tables, global or per project, match a brief by
size, phase or title words and set its model, effort or an effort step. Open in VS Code, for
Visual (the default), opens the project's own workspace file, which brings its window to the front
or opens it, and fills the prompt into a new Claude Code tab there, where the owner chooses the
model and effort and sends it. Start headless runs the session and gives it a tab in the Sessions
pane: its transcript as it streams (the assistant's text, each tool call in one line with its result
collapsed under it, what it asked, what was refused, the result), Stop and Kill while it runs, and,
once it has ended, Resume, which runs it again under its session id with the message typed there.
The pane's All tab lists every session, the window's, those started elsewhere and the ledger's
latest rows, filtered by project, mode and running only. The transcript stays in the window's
memory: the tool writes none and reads none.

The Decisions pane answers what a headless session asks while it waits: a permission request with
Allow or Deny (its tool, its call, why it asks and its input shown; a reason optional), a question
with its options, one or several, or the owner's own words, sent or declined. A project's
`answer_timeout` (`crw set <project> answer_timeout <minutes|none>`, none by default) defers a run
whose request waits that long: the request is refused, the turn ends, and the answer given later
in the pane resumes the session with it. With `crw setting decisions defer`, or by itself for the
later runs of a CLI that refused the control protocol's handshake, the window takes Claude Code's
documented defer-and-resume route instead: a `PreToolUse` hook (`crw.exe hook --defer`) ends a run
at each question, and the answer resumes the session without a prompt through a one-shot hook
(`crw.exe hook --answer <file>`). A session Claude Code reports waiting elsewhere is listed too,
never answered: Focus brings its VS Code window forward. The ledger keeps how many questions a run
asked and how long the answers took.

The window also listens on `127.0.0.1` (the registry's `port`, 47831 by default) for Claude Code's
hooks (SessionStart, Notification, Stop, SessionEnd) and its status line, which `crw signals install`
writes into the user settings to run `crw.exe hook` and `crw.exe statusline`: each reads the
session's input, keeps the session's id, folder and event (or the status line's model, cost, context
and rate limits, never a transcript's text) and posts them within half a second, printing nothing
and exiting quietly when no window listens. With `--inbox` the start hook also prints the project's
open inbox messages into the session. The window asks `claude agents --json` for the live sessions
every 5 s while it has the focus (every 30 s otherwise), matches each session's folder to a
registered project, and lists the sessions started elsewhere (visual, headless or by hand; busy,
waiting, idle or ended) in the Sessions pane and as a session in progress in the project's row. A
session gets its ledger row at its first sign of work (a finished turn, a wait for the owner, or the
poll seeing it busy), so a tab opened and left unused, a helper the extension starts and a tab a
window restores leave none; a visual launch no session takes up within 30 minutes lapses.

The window calls the owner with a popup of its own: a small window at the screen's lower right,
always on top, that takes no keyboard focus and has no taskbar button (Windows' toasts can be off,
and the popup does not depend on them). A notification comes when a session waits for an answer or
a permission (one of the window's headless runs, a run deferred at its time limit, or a session
Claude Code reports waiting in VS Code or a terminal), when a run finishes or fails, and when a
session elsewhere ends a turn of two minutes' work or more. The popup lists the latest, the newest
first, each with Show, which brings the window forward on the pane that matters or the VS Code
window a session waits in, and Dismiss; they stay until dismissed. Notifications are held back in
the registry's quiet hours (`crw setting quiet_hours 22:00-07:00`, none by default) and while the
window is in front, where the status bar says them instead; File, Send a test notification puts one
in the popup. The tray icon counts what waits for the owner and marks what is new since the window
was last in front: an unadopted folder, a remote or a backup to look at, an inbox message. A left
click on it shows the window; its menu shows the window too, opens the launch dialog on a pinned
project's next brief, or quits. Closing the window leaves it in the tray with its headless sessions
going on, and Quit asks first when it would stop them. Each notification and what became of it,
each Show and each badge is a line in the ledger, which `crw notifications` prints.

Built with `--features
screenshot` and run with `CRW_SCREENSHOT=<file.png>`, it shows a fake portfolio
(`crates/app/fixtures/portfolio`), saves a picture of its own window and exits;
`CRW_SCREENSHOT_SETUP` arranges the view (see `crates/app/src/screenshot.rs`).

The workspace has three crates: `crates/core` (the model and engines, no UI types), `crates/app` (the
desktop window, `clauderoadmapwrangler.exe`, with its loopback server) and `crates/cli` (`crw.exe`, the
console commands and the hook and status-line relays). The checks every change passes: `cargo test`
with and without `--features screenshot`, `cargo clippy --all-targets` with and without it, `cargo fmt
--check`. Tests never run the real `claude` CLI and never spend usage: the session runner's tests start
`stub-claude`, a stand-in built with the core crate that replays recorded stream-json with fake names,
and the signals' tests bind a free loopback port and write scratch settings files.

## The roadmap house style

The tool reads each project's roadmap files in the shape below; this repository's own `roadmap/` folder
is a complete example. Two older shapes (a five-column table and a "work packet" style) are read through
format profiles, read-only. `crw roadmap <folder>` prints what the parser reads in one roadmap.

- **Phase tables.** `## Phase P1 — <name> (<status>)` headings, each followed by a table with the header
  `| # | Brief | Size | Depends on | Core | Done |`. One row per brief: an `EP-n` id; a one-line summary,
  optionally a link to the brief file, with the decisions it rests on as `D-n`; the size `S`, `M` or `L`
  (about 30 minutes, 1 hour and 2 hours of one session; a re-scope is written in the cell, such as
  `S → L (…)`); the ids it depends on; `core` or `stretch`; and the Done cell, `☐` or `☑` with one
  backticked commit hash.
- **The next brief** is the first unchecked row in table order, not the lowest number.
- **Brief notes.** Under each table, `**Brief notes.**` and one `- **EP-n.** …` bullet per brief: the
  detail a session starts from.
- **Briefs.** `roadmap/EP-n-<slug>.md`, opened with YAML front matter (`ep`, `title`, `size`, `model`,
  `effort`, `status`), then Context, In scope, Out of scope and Verification / acceptance, and closed by
  `> **Completion note (date; model/effort; duration).**`, which records each owner decision and its
  outcome. The tool reads the roadmap at git `HEAD` and in the working tree and keeps both: a changed
  README, an untracked or changed brief, or the next brief without a completion note (when it is the only
  open brief) means a session is in progress.
- **Decisions.** `DECISIONS.md` entries `**D-n Title.** Decision. *Why.* *Alternatives.*`, refined by
  dated addenda (`> **Addendum (date, EP-n).** …`) and never rewritten.
- **Owner tasks.** `roadmap/owner-tasks.md`, one line per task only the owner can do:
  `- [ ] OT-n (EP-k, date) text`, optionally `before EP-m`. The owner ticks a task in the tool, which
  changes the box and nothing else in the project (a copy of the file goes to the tool's backups folder
  first). Older forms of the owner's to-dos (a README's `## Risks / open items` list struck through when
  resolved, an `Open items for the owner` list, an `**Owner actions outside any brief**` paragraph,
  `owner-decisions.md` and `open-questions.md` registers, handoff notes, and the `Parked` and
  `Handed on` blocks of the latest completed brief) are read and shown, never rewritten.
- **Inbox.** `roadmap/inbox.md`, one line per message from another project:
  `- [ ] IN-n (from <project> EP-k, date) text`, optionally `[urgent]`. A sending session writes
  `> **Handed on (to <project>).** …` after its completion note and the tool delivers it; the receiving
  session triages each open message after reading the roadmap and writes the disposition on the line.
  The owner may tick a message in the tool, which writes `**Disposition (date):** …` on its line.
- **Re-plans and parked work.** A re-plan brief closes each phase; work named and not built is parked in
  `roadmap/v2-notes.md`.
- **Commits.** `<Title>: <summary> (EP-n, D-x)`, then `docs(roadmap): record EP-n commit hash`.

## Notes

Built for one developer's workflow: one user, one house style, configurability only where it costs
nothing. The tool drives the user's own installed Claude Code CLI and claude.ai login; it offers neither,
and it never stores the login's token. Developed with AI assistance (Claude Code), reviewed and directed by
the owner. License: [MIT](LICENSE).
