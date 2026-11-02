# clauderoadmapwrangler — DECISIONS

Architecture-decision log. Numbered decisions are **settled with the project owner** (they/them) in the
planning session of **2026-10-31** (EP-0) after 72 clarifying questions in 18 rounds across purpose,
project model, conventions, the cross-project inbox, architecture, launching, decisions and attention,
automation, usage, hosting, views, analytics, chat, remote access, operations and the shape of v1, informed
by a survey of the owner's eleven roadmaps and a research pass over Claude Code's documented surface, the
Rust GUI landscape and existing orchestrators (`roadmap/planning-2026-10-31.md`). Later sessions append
addenda (`> **Addendum (date, EP-n).** …`) under the decision they refine and add new numbered decisions at
the end; nothing is rewritten.

Format: **D-n Title.** Decision. *Why.* *Alternatives considered.*

## Purpose and governance

**D-1 Purpose, scope and tie-breaker.** A Rust desktop application that keeps track of the owner's
development projects (the git repositories under one `DATA\` folder, most of them run as session-sized
roadmap briefs executed by Claude Code) and mirrors the owner's Claude Code workflow: one view of every
project's next brief with its size, session launch with a chosen model and effort, a checkable pane of
inter-session owner tasks, optional automatic chaining of briefs, repository and backup status, badges,
sorting, history graphs, a hosted chat to Claude Code, an in-session decisions pane, an account usage meter,
a cross-project inbox, and a phone view over the owner's tailnet. When ergonomics for the owner's own daily
workflow conflicts with generality for other users or portfolio polish, **the owner's workflow wins**: one
user, one house style; configurability only where it costs nothing. *Why.* The tool exists to remove
friction from a working process that already produces results; generality would slow the first useful
version. *Alternatives.* A portfolio showpiece with configurable parsers from the start; deciding each
conflict at its EP.

**D-2 Identity, repository, licence.** Folder, crate, executable and window title are
`clauderoadmapwrangler`; a short alias for the shortcut and CLI is chosen at EP-1. The repository gets a
**private GitHub remote at EP-1** and becomes **public at the first usable release** (the P1 re-plan, once
the dashboard runs with fake projects for the README screenshots). Licence **MIT**. The README states that
the tool is built for one developer's workflow, documents the roadmap house style the parser expects,
discloses AI-assisted development as the owner's other public projects do, and says that the tool drives
the user's own installed Claude Code CLI and login and offers neither. *Why.* The owner's choice; the
remote is a backup from the first commit; the licence matches hupsim and mimicwarehouse. *Alternatives.* A
shorter or product-neutral name; public from EP-1; private until v1; local-only like the recent projects;
Apache-2.0.

> **Addendum (2026-10-31, EP-1).** The short alias is **`crw`**: the console binary `crw.exe` (package
> `crw-cli`), the library crate `crw-core` and the `CRW_*` environment variables. The desktop shortcut and
> the Start-menu entry are called **"Roadmap Wrangler"**, the name Windows toasts will carry, which keeps
> "Claude" out of the display name; both carry the AppUserModelID `willtfarrington.clauderoadmapwrangler`.
> The app's crate and exe and the window title stay `clauderoadmapwrangler`. *Alternatives.* `crw`
> everywhere, the shortcuts included; no alias (`clauderoadmapwrangler-cli.exe`).

> **Addendum (2026-11-01, EP-8).** The repository turns public at the P1 re-plan as planned, after the
> pre-flip audit (D-3 addendum): `gh repo edit willtfarrington/clauderoadmapwrangler --visibility
> public --accept-visibility-change-consequences`, with a one-line description and no topics, as the
> owner's other public repositories have; issues stay on and the wiki off. The README shows two
> pictures of the fake portfolio that the app took of itself (D-29). *Alternatives.* Staying private
> until v1; topics.

**D-3 Private data boundary.** The published repository must not describe the projects the tool handles.
Planning records that name the owner's projects live in the git-ignored `private/` folder; the runtime
**registry lives under `%APPDATA%\clauderoadmapwrangler`** and the **ledger under `%LOCALAPPDATA%`**, never
in the repository; tracked documents use generic names or the names of projects that are already public;
README screenshots show fake projects; session transcripts are never parsed or copied; the OAuth token is
never stored, logged or sent anywhere but `api.anthropic.com`. *Why.* "The online version should exclude
handled projects" (owner, 2026-10-31); several managed projects are wholly private by their own decisions.
*Alternatives.* Tracking the survey (names are public anyway); a settings dotfile in every managed project
instead of a central registry; everything under `%LOCALAPPDATA%` including planning records.

> **Addendum (2026-11-01, EP-8).** The pre-flip audit (EP-1's addition and IN-5) searched every
> commit's files and messages for the names kept in `private/` and found none, and read the inbox
> and owner-task files as generic. It found machine details in the planning record tracked since
> EP-0 (the backup drive's label, path and schedule, the plan's tier, the phone and its VPN,
> Tailscale Serve's ports, the security software and the shell's rights, a start-up manager, the
> probe's figures), the Windows user name in one brief's transcript path and the VPN's name in one
> brief note. On the owner's choice the tracked files now say these in general words and the
> private survey keeps them as written; the commits before EP-8 keep the first wording, since
> rewriting 25 commits would change every recorded hash, and this file keeps its own (D-28, D-29).
> From EP-8, before each commit, the added lines of the staged diff are checked against a word list
> in `private/` (the private names and these details; CLAUDE.md rule 3), and the README's pictures
> name a neutral data root. *Alternatives.* Publishing as written; rewriting the history before the
> flip; a pre-commit hook script (v2 notes).

**D-4 Cadence, horizon and sizes.** Two to three sessions a week, six to nine hours. Sizes S ≈ 30 min,
M ≈ 1 h, L ≈ 2 h of one Claude session with the owner supervising; anything larger is split at pickup; L is
reserved for six engines (the tile shell, the session runner, the decisions tile, the autopilot, the chat
tile, the web pages). v1 is **P1 through P5 including the phone view**, 32 briefs after EP-0 (8 S · 18 M ·
6 L ≈ 34 session-hours); the roadmap records session-hours, never a calendar estimate. A re-plan EP closes
each phase; everything named and not built is parked in `roadmap/v2-notes.md`, which every re-plan grows.
*Why.* The house pattern; measured durations re-cut the plan at each re-plan. *Alternatives.* Splitting
every L into two M (≈ 38 briefs); fewer, larger briefs; v1 without the phone view; a calendar estimate.

> **Addendum (2026-11-01, EP-8).** P1 measured from the session transcripts (each session's first
> entry to its EP's last commit): EP-1 … EP-7a took 6 h 17 min of session, 5 h 37 min without the
> gaps over five minutes, against 9 h planned (about 70 %); an M took 28 to 59 minutes (45 on
> average) and the one L 59. The sizes stay the planning unit (the owner's choice), so D-22's caps
> keep their margin. With EP-7a (M, IN-4) and this re-plan's EP-33 (S) and EP-34 (M), v1 is 35
> briefs, 9 S · 20 M · 6 L ≈ 36.5 session-hours, 27 of them in P2–P5 (about 19 at P1's pace). A row
> added at a re-plan takes the next free number and sits where it is needed (the house rule); EP-7a
> keeps its letter. *Alternatives.* Downsizing the remaining L engines to M; merging the small rows
> (EP-11 into EP-10, EP-18 into EP-17).

**D-5 Dogfooding.** This project's own roadmap is the first registered project: the read-only dashboard
shows its next EP from P1, and **from the P2 re-plan (EP-16) every further EP of this project is started
from the tool**. `CLAUDE.md` records the rule. *Why.* The tool is only trustworthy if it runs the process it
manages. *Alternatives.* Managing this project only after v1; no rule.

## Project model, discovery, storage

**D-6 Project model and states.** A managed project is **every git repository directly under `DATA`**,
roadmap or not; a repository without a parsable roadmap shows as *unformalised* with a Formalise action.
Non-git folders appear once as *unadopted* with Adopt or Ignore remembered. The owner's personal-site
repository is managed as the export target for project cards, not as a roadmap project. The vault folder
outside `DATA` is not a project. States: active, paused, maintenance, done, archived, unformalised,
unadopted — set by hand, with transitions proposed from facts (every row checked, a pickup gate, no commit
for 60 days) and confirmed by the owner; priority 1–5 and pinning by hand; "session in progress" is derived
live from a brief without a completion note or a running session and never stored. *Why.* Hand-set state
with proposals matches how the owner already classifies projects ("maintenance", "paused", "gated").
*Alternatives.* Derived states only; hand-set without proposals; active/inactive plus pinning; the vault as a
project.

> **Addendum (2026-11-01, EP-8).** A proposed state can be dismissed as well as confirmed; the
> registry remembers the dismissal with the facts the proposal rested on, so it returns only when
> they change (EP-33). *Alternatives.* Confirm only (P1's behaviour); proposals that expire.

**D-7 Discovery and the workspace file.** The tool watches `DATA` for new top-level folders and rescans at
start; a new folder appears as unadopted; **Adopt registers it and appends an entry to the owner's
`.code-workspace` file**, preserving the existing order and the named entries; Ignore is remembered. *Why.*
The owner adds projects by creating folders and wants them in the single workspace without hand-editing
JSON. *Alternatives.* Manual refresh only; silent auto-adoption; never touching the workspace file.

> **Addendum (2026-11-01, EP-8).** Since 2026-10-31 the owner opens each project from its own
> single-folder `<name>.code-workspace` beside the folders, and the aggregate file stays for
> browsing (IN-1). Adopt therefore also writes the adopted folder's own file in that form, never over
> an existing one, and still appends to the aggregate; a fresh registry's `workspace_file` is the one
> workspace file of the data root that lists more than one folder, else none; and `crw scan` lists
> which projects have their own file. EP-33 builds this before EP-11, whose Visual launch opens the
> project's own file (D-16 addendum). *Alternatives.* Folding it into EP-11 (S → M); building it at
> the re-plan.

**D-8 Storage.** A human-editable **`registry.toml`** under `%APPDATA%\clauderoadmapwrangler` (projects,
states, priority, pins, launch defaults, autopilot rules, settings) with a format version and a dated backup
before any migration; a **SQLite** database under `%LOCALAPPDATA%\clauderoadmapwrangler` for the session
ledger, usage samples and daily parse snapshots; a daily JSON export of both into this repository's
git-ignored `private/` folder, which the owner's existing Documents backup job already archives; CSV export
per project on demand; the ledger is kept indefinitely. *Why.* The owner's other apps keep human-editable
config with dated backups; `%LOCALAPPDATA%` is not in the backup job's scope, `private/` is.
*Alternatives.* One SQLite database for everything; TOML plus JSONL ledgers; a single JSON document; the
database stored in `private/` directly; a one-year retention.

## Conventions

**D-9 Convention model and format profiles.** The house style is the schema: `# | Brief | Size | Depends
on | Core | Done` phase tables, `☑ \`hash\`` or `☐`, sizes S/M/L with re-scopes in the cell, `**Brief
notes.**` with `- **EP-n.** …` bullets, `EP-N-<slug>.md` briefs with a completion note, `DECISIONS.md` with
`**D-n Title.**` entries and dated addenda, a re-plan EP per phase, parked work in a notes file. **The next
EP is the first unchecked row in table order, not the lowest number.** "In progress" is derived from a
brief file without a completion note, an untracked brief, or a running session. The parser reads a
snapshot (git `HEAD` plus `git status`), never live files alone. The current house style is parsed fully;
the five-column variant and the work-packet style (`# | Packet | Size | Depends on | Status`, `[x] hash`,
handoff payloads, letter-suffixed ids) are parsed **read-only** through format profiles (next unchecked
row, counts, handoff status). Migrating a project to the house style is a chat-pane task the owner
triggers per project, executed by a session in that project. *Why.* The survey of eleven roadmaps found
three header variants, two checkbox families, non-numeric row order, implicit in-progress states and live
edits by concurrent sessions. *Alternatives.* Migrating the three older projects first; supporting the
house style only with a regex fallback.

> **Addendum (2026-10-31, EP-3).** Columns of a phase table are located by name (`#`, `Brief` or `Packet`,
> `Size`, `Depends on`, `Core`, `Done` or `Status`), so an added column does not hide a table, and the
> profile follows from the columns. Only `☑` and `[x]` count as done: a `◐` (done but not landed), `[~]`
> (in progress) or unknown Done cell is not, so the next EP stops at it and the counts leave it out.
> Depends-on ids leave out parenthesised annotations and `before` clauses. *Alternatives.* The three header
> strings only; `◐` counted as done.

> **Addendum (2026-10-31, EP-4).** "Session in progress" is derived from the snapshot's git status when the
> folder is a repository: the roadmap README changed, a brief untracked or changed, or a `[~]` row. The rule
> "a brief without a completion note" applies only where git cannot be asked (a folder that is not a
> repository), and only when that brief belongs to the next EP and is the only open row with a brief: a
> committed brief in a clean tree is a pre-written one, as two surveyed projects showed on 2026-10-31, not a
> session. A running session joins as a source at EP-12. Completion evidence is one rule for the three
> profiles (a checked row, the brief's own markers, or the handoff file), since their forms are disjoint.
> *Alternatives.* The literal rule (two false positives); a separate "brief written" state.

**D-10 Owner tasks.** Each managed project carries **`roadmap/owner-tasks.md`**: one line per task,
`- [ ] OT-n (EP-k, date) text`, optionally `before EP-m`. Sessions append at completion; the owner ticks in
the tool and the tool writes the tick back. The seven existing forms (Risks / open items lists, Parked and
Handed-on blocks, OD and OQ registers, maintenance open items, "owner actions outside any brief" paragraphs,
handoff notes) are parsed and shown read-only and never rewritten. The rule is added to a project's
`CLAUDE.md` at adoption. *Why.* The pane needs one machine-readable home; the old forms carry history that
must not be disturbed. *Alternatives.* Migrating today's open items into the file; parsing the existing
forms only with ticks in the registry; tasks only in the registry.

> **Addendum (2026-11-01, EP-5).** A tick changes the one byte inside the line's box (and back) and
> nothing else in the file; the file as it was is first copied to the tool's backups folder
> (`%LOCALAPPDATA%\clauderoadmapwrangler\backups\<project folder>\owner-tasks.<date>.md`, never
> overwritten), not to a `.bak` beside it, so a tick leaves no new file in the project (D-11). Every form
> reads into one owner item with a state: struck through at its start, resolved; a clause struck inside,
> partly resolved; a register row by its Done cell or by words (`Resolved`, `Closed`, `Decided`,
> `(closed)`). The `Parked` and `Handed on` blocks and the handoff notes are read from the latest
> completed brief only (the newest completion date, then table order): the sessions after a brief have
> read its hand-offs. `> **Owner decisions (…).**` blocks record decisions taken and are not owner
> items. *Alternatives.* A `.bak` copy beside the file; the blocks of every brief.

**D-11 Write-back.** Inside a managed project the tool writes **only files it owns**: `owner-tasks.md`
ticks, `inbox.md`, the adoption files (a `CLAUDE.md` rule block, the two new files, the brief template) and
the `.code-workspace` file. It **never edits `roadmap/README.md`, briefs or `DECISIONS.md`**; sessions do,
and commits are made by sessions (D-22). *Why.* The roadmap files are the sessions' and the owner's record;
two writers would corrupt it. *Alternatives.* Ticking Done and recording the hash from the tool; read-only
with ticks in the registry.

**D-12 House-style amendments.** From the next brief on, sessions in every house-style project (1) record
model, effort and duration in the completion note: `> **Completion note (2026-10-31; fable/xhigh; 1 h 20
min).**`; (2) keep `- **EP-n.** …` under `**Brief notes.**` as the one form for upcoming EPs (the row's
Brief cell is the one-line summary, the bullet is the detail the launch prompt carries); (3) write the Done
cell as one backticked hash; (4) open each brief with **YAML front matter** (`ep`, `title`, `size`, `model`,
`effort`, `status`). The parser accepts the old forms forever; old briefs are not rewritten. *Why.* Model and
effort were recorded nowhere, which the history graphs and effort rules need; the owner asked for front
matter as well. *Alternatives.* Any subset; a model/effort column in the phase table.

> **Addendum (2026-11-01, EP-8).** A lettered row's brief writes its id in the front matter with or
> without the prefix (`ep: 7a` or `ep: EP-7a`); the parser reads both (EP-7a's fix). *Alternatives.*
> The prefix required.

**D-13 Cross-project inbox.** Messages between projects live in the receiver's **`roadmap/inbox.md`**, one
line each, `- [ ] IN-n (from <project> EP-k, date) text`, tracked in git and mirrored in the registry for
the UI and the phone. **The tool is the mailman**: a sending session writes `> **Handed on (to
<project>).** …` in its completion note; the tool parses it, appends to the receiver's inbox and logs the
delivery; the owner can compose messages in the tool and from the phone. **Triage rule** (in each
project's `CLAUDE.md` and in the launch prompt): right after reading the roadmap, for each open message —
roll it into this session if it fits the brief and is at most S-sized; otherwise add it to the brief notes
of a named later EP or propose a new row in the right phase, confirming with the owner through
`AskUserQuestion`; or decline with a reason; the disposition is written on the line and the box ticked. A
running session is **never interrupted**; the tool shows a badge; the autopilot is not blocked because the
next session triages first; an `urgent` flag notifies the owner, not the session. *Why.* The owner runs
several projects in parallel and carried such findings by hand (a vault-tooling session noted other
projects drifting from a contract). *Alternatives.* Registry-only messages injected into prompts; sessions
writing across repositories with `--add-dir`; messages only through the UI; a blocking inbox; injecting
urgent messages into running headless sessions.

> **Addendum (2026-11-01, EP-5).** The owner's tick in the tool requires a disposition and writes it at
> the end of the message's last line as ` **Disposition (date):** text`, on one line, the form one
> project's sessions already use; a session's own `**Triaged at EP-n (date):** …` reads as a disposition
> too. A `> **Handed on (to the owner).**` block is an owner item, not mail; a target the registry does
> not know, whole or by its first word, is shown and not delivered. *Alternatives.* The disposition on a
> continuation line of its own; delivering `(to the owner)` blocks into `owner-tasks.md`.

## Architecture

**D-14 Stack and layout.** A Cargo **workspace**: `core` (model, parsers, registry, session runner, usage,
no UI types), `app` (the desktop in **eframe 0.37 + egui_tiles 0.18**, plus an embedded **axum** server on
a loopback port that later serves a few server-rendered pages — askama templates, SSE — for the phone) and
`cli` (doctor, scan, inbox send, hook and status-line relays, scripting). One binary, one process, no wasm.
Protocol types from **`claude-codes`** behind a thin adapter module; process plumbing with tokio and
process-wrap job objects; Markdown with pulldown-cmark and byte-range edits; `rusqlite` (bundled) and
`toml`; git and GitHub by shelling out to `git` and `gh`. *Why.* The owner knows egui; a server-rendered
phone view has native text input and costs a few templates; the research pass found Dioxus, Leptos and
Tauri all mid-transition and eframe-wasm still weak at mobile text editing. *Alternatives.* Dioxus for both
targets; eframe-wasm for the phone; a server-rendered UI with the desktop as an installed PWA; one crate
with modules; hand-written protocol types; another community client.

> **Addendum (2026-11-02, EP-9).** Each headless session gets a thread of its own with a
> single-threaded tokio runtime, so the window's frame loop and `crw` see a channel of events and a
> handle rather than an async API. `claude-codes` is compiled with its serde models only (`types`):
> the runner spawns and reads the child itself. process-wrap's tokio flavour is the one used, since
> only its job object is killed by the system when the tool lets go of it (`KillOnDrop`); its
> no-window flag takes the `windows` crate's type. `uuid` (v4) and `windows` join the pins at the
> versions the lock already held. *Alternatives.* One runtime shared by every session; std threads
> with process-wrap's std job object, which outlives a crashed tool; the crate's own async client.

**D-15 Pins and version detection.** `Cargo.lock` pins everything; the egui family and the protocol crate
move only in re-plan briefs. A **minimum Claude Code version** is recorded in `core`; at start the tool
reads `claude --version` and `claude auth status`, and on each run the `system/init` capabilities, enabling
or hiding features instead of assuming flags (the installed 2.1.261 lacks `--max-turns` though the docs
list it). The tool never passes `--bare` (API-key only) and removes `ANTHROPIC_API_KEY` from every child
environment so runs bill the subscription. *Why.* CLI flags drift between releases; the extension bundles a
different CLI version than the standalone binary. *Alternatives.* Tracking latest at every brief; freezing
everything.

> **Addendum (2026-11-02, EP-9).** A session's child also withholds the markers of a Claude Code
> session the tool itself may run in (`CLAUDECODE`, `CLAUDE_PID`, `AI_AGENT`,
> `CLAUDE_AGENT_SDK_VERSION` and every `CLAUDE_CODE_*` variable), with which a nested `claude -p`
> does not start; a setting meant for every session belongs in the user settings' `env` block, which
> the child reads itself. The preflight asks `--version`, `auth status` and `--help`, one child after
> the other, and is kept for an hour; the help is where features come from: a wish it does not list
> (`--effort ultracode`, `--max-budget-usd`) is left out and said so, and a flag the run needs
> refuses the launch when it is missing. `crw doctor` prints the preflight. The minimum stays
> 2.1.261: the adapter is known to need nothing newer. *Alternatives.* A fixed list of markers;
> probing flags with a made-up option; raising the minimum to 2.1.316, the one version tried.

## Sessions

**D-16 Launch surfaces and cwd.** v1 offers **Visual** (the VS Code extension's
`vscode://anthropic.claude-code/open?prompt=…` URI, which pre-fills the prompt for the owner to send) and
**Headless** (`claude -p` with stream-json in and out, chosen model and effort, transcript streamed to a
tile and to the phone, decisions routed to the tool, a pre-assigned session id, `--name "<project> EP-n"`).
Claude's own `--bg` background sessions are a later option (v2 notes). **Cwd is always the project
folder**, with no extra directories; for Visual the tool opens VS Code on the project folder (or a
generated per-project workspace file) when the running window's first folder is not that project, then
fires the URI; the owner's `DATA` workspace stays for manual cross-project work. *Why.* Sessions started
from the multi-root workspace run in its first folder, so transcripts and auto-memory of other projects are
misfiled; the URI handler exists in the extension. *Alternatives.* `--bg` instead of `-p`; all three
surfaces in v1; headless only; keeping one window and naming the project in the prompt; rewriting the
workspace file's first folder.

> **Addendum (2026-11-01, EP-8).** IN-1 and IN-2 revise "with no extra directories". Cwd stays the
> project folder. Visual opens the owner's existing `<data root>\<name>.code-workspace` and generates
> one only where none exists (EP-11, through EP-33's writer). Headless runs add the project's
> neighbour folders, a list per project in the registry, with `--add-dir` and
> `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1`, so their `CLAUDE.md` and rules load too, which a
> folder granted in a settings file never does (Claude Code's permissions page, read 2026-11-01);
> EP-9 passes them and EP-10's dialog shows them. *Alternatives.* No neighbours in headless runs;
> neighbours granted through settings files only.

> **Addendum (2026-11-02, EP-9).** A headless run is one prompt: the runner closes the session's
> stdin when its result arrives, so the process exits and its ledger row closes; a follow-up resumes
> the session by its pre-assigned id (EP-13). The session is named `<project> EP-n`. A stop sends the
> protocol's interrupt, closes stdin and waits 30 seconds before the job is killed; dropping the
> session's handle kills it, and whatever process the session left behind dies with it.
> *Alternatives.* Keeping stdin open for further messages to the same process; a stop without the
> interrupt.

**D-17 The launch prompt.** A generated house-style opener, the same for Visual and Headless, shown in an
editor before launch and overridable by a per-project template: project name and path; the project's own
read-first order; the next EP row (id, title, size, depends-on) and its brief-notes bullet; open inbox
messages and owner tasks with the triage rule; the rules to write the brief first (front matter), end with
the completion note in the new form, and stop before committing (or commit and record the hash when the
autopilot holds the standing yes, D-22). *Why.* The prompt is the one place the tool can carry the method
into a session without editing `CLAUDE.md` files. *Alternatives.* "Execute EP-n" alone; templates only.

**D-18 Permissions for headless runs.** `--permission-mode acceptEdits --permission-prompts host`: edits
inside the project pass; shell commands outside the project's allow rules, network and writes elsewhere
reach the decisions tile; each project's own `.claude/settings.json` allow and deny rules still apply; a
per-project override to `auto` or `bypassPermissions` for projects the owner trusts. *Why.* Fail-safe by
default with the owner in the loop, without the terminal. *Alternatives.* `auto` everywhere; bypass; `dontAsk`.

> **Addendum (2026-11-01, EP-8).** Under `acceptEdits`, Claude Code's documentation says file edits
> and the common filesystem commands (`mkdir`, `touch`, `mv`, `cp`) pass without a prompt for paths
> in the working directory or the added directories, so a neighbour (D-16 addendum) would be
> editable. Neighbours are read-only: the launcher passes through `--settings` an `Edit` deny rule
> for each (`Edit(//<drive>/<path>/**)`; Edit rules cover every built-in editing tool, and Windows
> paths match in POSIX form). The documentation does not say whether that deny also stops those
> filesystem commands; EP-9 finds out and puts any gap to the owner. *Alternatives.* Writable
> neighbours, kept read-only only by each project's own rules; no neighbours.

> **Addendum (2026-11-02, EP-9).** Found on the owner's yes with one supervised headless run through
> the runner (CLI 2.1.316, the cheapest model at low effort, scratch folders): under `acceptEdits` the
> `Edit` deny rule refused `mkdir` (by an absolute and by a relative path), `touch`, `cp` and `mv`
> into a neighbour, and the Write tool there, each without a prompt (`system/permission_denied`),
> while `mkdir` in the working folder ran. Claude Code's permissions page says deny rules do not
> reach a program that opens files itself (a script, say); `acceptEdits` does not accept such
> commands, so they reach the host as permission requests unless a project's own allow rules admit
> them, and only an OS-level sandbox would close that. *Alternatives.* Bash deny rules per
> neighbour; a `PreToolUse` hook that checks paths; the sandbox.

**D-19 Decision routing.** The **control protocol over stream-json first**: the tile shows each
`AskUserQuestion` with its options and each permission request; the session blocks until the owner answers
in the tool or on the phone; an optional timeout (default none) defers the run instead. The documented
**defer-and-resume** route (a PreToolUse hook returning `defer`, the run exiting with the pending tool use,
`--resume` with the answer in `updatedInput`) is the fallback when a CLI release breaks the protocol. Visual
sessions show "waiting for you in VS Code" with a Focus button; the tool never answers for them. *Why.* The
protocol is what community tools use and what `claude-codes` types; the docs guarantee only the defer route.
*Alternatives.* Defer-and-resume only; no answering in the tool.

> **Addendum (2026-11-02, EP-9).** The handshake is the `initialize` control request the runner
> sends before the prompt; its answer is an event, and a refusal is what selects EP-14's fallback. A
> control request the tool does not handle (a hook callback, an MCP message, one the protocol crate
> cannot read) is answered with an error at once, so that a session never waits on it; an
> `AskUserQuestion` is answered with its questions kept and the answers keyed by question text. CLI
> 2.1.316 answered the handshake and named fourteen capabilities in its init, among them
> `interrupt_receipt_v1` and `ask_user_question_unanswered_v1`. *Alternatives.* No handshake (the
> CLI asks the host by default); unknown requests left unanswered.

**D-20 Signals.** The tool installs **user-level hooks** (Notification, Stop, SessionStart, SessionEnd) in
`~/.claude/settings.json` that run its `cli` binary, which posts the event to the loopback server and exits
silently when the tool is not running; it installs a **status-line command** the same way (D-23); and it
polls `claude agents --json` as reconciliation. The hooks see visual sessions too, since the extension fires
the same events. *Why.* Hooks are the documented event source; polling covers sessions started without the
tool. *Alternatives.* Polling only; hooks only.

**D-21 Notifications.** Windows toasts (WinRT through notify-rust, attributed to the tool's Start-menu
entry); a tray icon with a badge count and a menu (Show, Start next for a pinned project, Pause autopilot,
Quit); the Claude app's own push for the phone; **ntfy over the tailnet** (a topic configured in the
registry, shared with the owner's other project's planned container). Toast at once for "waiting for
input", on finished or failed, and at 80 % and 95 % of weekly or Fable usage; badges only for a new
unadopted folder, an orphan or mismatched remote, a stale backup and inbox arrivals; quiet hours
configurable, default none. *Why.* Attention is the scarce resource; the phone already gets Remote Control
pushes. *Alternatives.* Toasts and tray without ntfy; only waiting and failed; every event.

## Automation and usage

**D-22 Autopilot.** A per-project toggle. **Gate** for chaining the next brief after a headless run:
autopilot on; the previous run ended cleanly with a completion note; no open owner task gated before the
next EP; the next EP is not a re-plan (re-plans are the owner's); its size is within the project's limit
(default S and M; L needs a click); weekly and Fable usage below the pause threshold; a clean tree after the
commit step; daily caps not reached. **Commits:** autopilot on is the owner's standing yes for that project
— the headless prompt tells the session to commit in the house pattern and record the hash itself, as it
does today after the owner's yes; with autopilot off the session stops before committing and the tile's
Commit button resumes the session with "commit and record the hash", so the tool never edits roadmap files
(D-11). **Rules:** a per-project default model and effort (initially `fable` at `xhigh`) plus a rule table
in `registry.toml` matching size, phase or title keywords to model, effort and `ultracode` (S one effort
step lower, L may add `--effort ultracode`, re-plans owner-run); global rules with per-project overrides;
the launch dialog shows the resolution. **Caps:** per run a wall-clock cap by size (S 45 min, M 90 min,
L 3 h) and the budget-estimate flag when the CLI has it, a failed result stops the chain; per day at most 2
chained runs per project and 4 overall; the autopilot pauses everywhere at 90 % weekly or Fable usage and
resumes after the reset; a global Pause in the tray and the window. *Why.* Unattended work must respect the
owner's commit rule, the owner's attention and the plan's limits. *Alternatives.* Chaining on success
alone; never chaining without a click; the owner committing in VS Code with the autopilot waiting; the tool
committing and recording hashes; one model and effort per project; usage thresholds as the only cap.

**D-23 Usage meter.** Two sources: the **undocumented** `GET https://api.anthropic.com/api/oauth/usage`
(weekly, Fable-scoped weekly, five-hour while active, per-surface breakdown), polled every five minutes and
after each tool-launched session with the access token read from `~/.claude/.credentials.json` at poll time
and never stored, treated as breakable; and the **status-line feed** — a command installed in the user
settings that posts each session's documented `rate_limits`, `context_window` and `cost` to the loopback
server. When the endpoint breaks the meter degrades to the status-line numbers and says so. Display: a
strip in the status bar (weekly, Fable, five-hour when active) and a tile with reset countdowns, the
per-surface breakdown and the last seven days of samples. **Downshift:** above a configurable share of the
Fable-scoped limit (default 60 %) the rule resolver proposes Opus 5.5 for S briefs, shown in the launch
dialog and applied by the autopilot; M and L keep Fable until the pause threshold. **Attribution:** the
weekly-percent delta sampled before and after each tool-launched session is attributed to the project; the
status-line cost estimate is recorded per session for sessions the owner starts; a usage-share badge and a
chart; transcripts are not parsed. *Why.* The probe on 2026-10-31 showed the Fable-scoped limit only on
the endpoint; the status line is the documented source. *Alternatives.* Status-line only; endpoint only;
every minute or on demand; no downshift or downshift for every size; parsing transcripts for tokens.

## Hosting, views, history

**D-24 Hosting, backup and the site.** Remote badges none / private / public, ahead / behind / diverged by
head comparison, orphan (a GitHub repository named like a local-only project), missing; actions Create
private remote, Make public (repeating `gh`'s consequences warning), Make private, Remove remote, each
behind a confirmation dialog and logged, never automatic; refreshed daily and on demand. The orphan
private repository found in the survey stays as it is, flagged with Archive and Delete actions (Delete needs
the `delete_repo` scope the current token lacks). **Backup:** the tool reads the owner's existing Documents
backup job (its settings and run records) for per-project "last archived" and archive age, whether `D:` is
mounted and which projects the job excludes; Back up now runs the job's script with `-Force`; no second
mechanism. **Site:** an Export action writes `projects.json` (and an HTML snippet) with name, badge, done
and total, next items and an "as of" date for public projects into the site repository; a site session
wires it in once; the tool never edits `index.html`. *Why.* Head comparison, not push dates, shows
divergence; the backup job already exists; the site's figures were hand-typed and stale. *Alternatives.*
Status only; actions without confirmation; archiving, deleting or renaming the orphan now; a second
archive mechanism; editing `index.html`.

> **Addendum (2026-11-01, EP-7).** The head comparison asks `git ls-remote origin` for the upstream's
> branch (`HEAD` without an upstream) and sets it against the local `HEAD`; when the two differ,
> `git rev-list --left-right --count` counts both sides: in sync, ahead, behind, diverged, or *not
> fetched* when the origin holds commits the clone has never seen. The tool never fetches, since a
> fetch writes into the project's `.git` (D-11); once a commit makes the check stale, the row shows
> git's own counts against the upstream as last fetched. One `gh repo list` gives every repository
> of the account (the hosting and the orphans), one `gh run list` per repository with workflows its
> latest run; the answers are cached in the ledger (`badge_cache`, schema 2) with their fetch time,
> asked again when a day old (the window looks hourly) or on F5, and a failed answer is shown but
> not cached. The activity badge's `git --no-optional-locks status` covers the whole working tree
> and writes nothing; it runs in every registered repository, the data repositories included, and
> git's output appears only as counts, never as paths; this project's sessions run `crw status` on
> named non-data projects, never over the whole registry (the owner's answer). The backup job is
> read through two registry settings, `backup_settings` (its settings file) and `backup_log` (its
> run log); a project is stale after 48 hours without the job writing or checking its unit. The
> actions above are placed by the P1 re-plan (EP-8). *Alternatives.* Push dates instead of heads;
> a fetch before the comparison; a cache file of its own; asking GitHub at every start; a setting
> for the archive folder instead of the job's settings file; no git child in the data repositories.

> **Addendum (2026-11-01, EP-8).** The actions (Create private remote, Make public, Make private,
> Remove remote, Archive and Delete for an orphan, Back up now) are EP-34, an M row in P4 after the
> exports and before operations: one confirmation dialog that names the command and repeats `gh`'s
> warning, one child at a time, each action an `event` row of the ledger; until then the owner uses
> `gh` and the job by hand. *Alternatives.* An M row early in P2; v2.

**D-25 Views.** Docked egui_tiles: a dense project list (name, state, next-EP one-liner with size, badges,
session status, autopilot toggle, Start), a detail tile (brief notes, owner tasks, inbox, history), a
decisions tile, a sessions tile with live transcripts, the chat tile and the usage tile; layout saved with
Reset; a cards view as a toggle. Badge groups: stack and progress (manifest-detected chips, done over total
with bar and percent, next size), activity (last commit age, dirty tree, session state, autopilot, staleness
colour), hosting and safety (remote, ahead/behind, orphan, last archived, inbox and owner-task counts, usage
share) and **health** (latest GitHub Actions run, licence present). Collapsible groups by state with a
sort-key menu within groups (priority, completion, remaining, last activity, hosting, usage share, name),
pinned first, filters by stack and hosting, saved with the layout. Rows expand inline to heatmap and burn-up
sparkline, Expand all / Collapse all, compact and comfortable density, 4K-class typography with an icon
font, theme System with override, keys (arrows select, Enter opens, S starts, A toggles autopilot). *Why.*
The owner's egui conventions, carried over. *Alternatives.* Cards first; a table only; a flat list; tabs per
state; graphs only in the detail tile or a separate window.

> **Addendum (2026-11-01, EP-6).** The list's groups run active, paused, maintenance, unformalised,
> unadopted folders, done, archived, with done and archived folded at first; inside a group, pinned
> first, then the sort key — priority (1 first), completion (the most complete first), remaining (the
> fewest open rows first), last activity (the latest commit first), name — with the projects that lack
> the key's data last; hosting and usage share wait for EP-7 and EP-18, and a name filter is added.
> Keys: ↑ ↓ select, → ← expand and fold a row, Enter brings the Detail tile forward. A project's files
> show at once; git's readings join as a worker thread gets them, one child at a time (D-29). A right
> click on a row pins it or sets its priority or state (D-6's "by hand"). *Alternatives.* Unadopted
> folders first; Enter expanding the row; pin, priority and state from `crw set` only.

> **Addendum (2026-11-01, EP-7).** The badges have a column between the counts and the session
> status, cut into columns of their own so the marks line up from row to row: in the full form the
> first stack chip (`RS+1` when there are more), the hosting, the sync with the origin, the last
> commit's age in its staleness colour (fresh under 7 days, quiet under D-6's 60, stale after) with a
> dot for a dirty tree, the CI run with a warning for a public repository without a licence, and
> the backup; the compact form drops the chip and the age. As a row narrows the badges turn compact,
> then the counts give way, then the badges, then the progress, now a small pill filled to the
> fraction done. Hosting sorts public, private, other remotes, then local only, a missing folder
> last; the stack and hosting filters keep the projects with any chosen value, are saved with the
> view, and hide the unadopted folders while set. Each badge's sentence is its hover text and a line
> of the Detail pane's Status section. *Alternatives.* The badges giving way before the counts;
> every stack chip in the row; filters that need every chosen value.

> **Addendum (2026-11-01, EP-7a).** Each column of the list is as wide as the widest value the
> portfolio holds, measured every frame by the code that draws it, over every project and folder
> whether folded, filtered out or shown, so the columns stay put; and at least as wide as its name
> in a header row that stays above the scrolling list (Project, State, Next, Brief, Done, Waiting,
> Stack, Hosting, Sync, Age, CI, Backup, Session; the expander, the pin and the two switches go
> unnamed). Beyond the title's 160 points the gaps between the columns take three tenths of the
> spare width, each at most 12 points more, and the title takes the rest; a narrower row keeps the
> give-way order above. A Legend switch in the toolbar, saved with the view, opens a key to every
> mark of the row at the foot of the pane (at most 45 % of its height), grouped as the columns run
> and drawn from the same table of signs the row's marks are made from. The Status section sets
> each subsection's name level with its first line and the marks in a column of their own, so the
> sentences start together, with a divider in the theme's separator colour between subsections.
> The window, `crw` and the README spell license; this file keeps its wording. *Alternatives.*
> Drag-resizable columns with saved widths (v2 notes); headers clipped to their content's width;
> the key as hover texts only, or in a window of its own; fixed gaps.

> **Addendum (2026-11-01, EP-8).** The cards view, which EP-6 left out with no destination, joins
> EP-26: a Cards switch in the Projects toolbar, saved with the view, shows one card per project
> (name, state, next brief, progress, badges and the burn-up sparkline EP-26 draws) in the list's
> groups and order. *Alternatives.* An S row of its own; v2.

**D-26 History and analytics.** **Backfill** per project with Re-import: commits whose messages record an
EP (the `docs(roadmap): record EP-n commit hash` commits, `(EP-n` suffixes, `EP-n:` prefixes) give
completion dates; completion-note dates cross-check; the single-commit published project gets note dates
only; no transcript parsing. **Ledger** per session: project, EP, start and end, launch mode (visual,
headless, manual), model, effort, ultracode, outcome (completed, stopped, failed, deferred), usage delta,
status-line cost estimate, questions asked and the owner's answer latency, resulting commit; usage samples;
daily per-project done and total. **Graphs:** activity heatmap (sessions and completions per day, per
project and portfolio), burn-up per project (re-plans as jumps in the total), velocity and projection (EPs
per week, projected finish and remaining session-hours from the size mix and measured durations). *Why.*
Git already holds the dates; the ledger makes the rest cheap. *Alternatives.* Completion-note dates only;
no backfill; transcript copies; a usage-over-time graph (parked).

## Chat, remote access, operations

**D-27 Chat and the method plugin.** One hosted Claude Code session per conversation (`-p` stream-json,
resumable, history kept), **cwd = `DATA`, read-only tools plus the tool's own `cli`**; questions are
answered directly; anything that writes into a project (formalise, migrate, add a brief) is delegated to a
headless session launched in that project with a generated prompt; default model Opus 5.5 at medium,
switchable per conversation; an appended system prompt carries a generated portfolio digest (states, next
EPs, open tasks and inbox counts, session states, usage) and the house-style reference. The owner's roadmap
method ships as a **Claude Code plugin in this repository** (`plugins/roadmap-wrangler/`): skills plan (the
multi-round interview), replan, formalise (bootstrap a project into the house style: roadmap README
skeleton, `DECISIONS.md`, `CLAUDE.md` with read-first, owner-tasks, inbox and triage rules, a brief template
with front matter), status, and the house-style reference document; tool-launched sessions load it with
`--plugin-dir`; the owner installs it into their own sessions from the repo. Chat actions in v1: adopt a
project, formalise or migrate a project, compose inbox messages and owner tasks. *Why.* A session rooted in
`DATA` can read every project; writes stay with per-project sessions; the method must be the same in the
chat, in launched sessions and in the owner's own sessions. *Alternatives.* A full-tool session in `DATA`; a
VS Code conversation with the question pre-filled; prompts only; skill files written into `~/.claude/skills`;
nothing injected or a full registry dump; a portfolio report (parked).

**D-28 Remote access and security.** axum binds **127.0.0.1 only**; a one-time "Enable remote view"
action runs `tailscale serve --bg --https=9443` to that port after the owner's click (the node's 443 and
8443 belong to another project), tailnet only, never Funnel; the tool checks `tailscale serve status` and
shows the URL and a QR code; disabling is one click. Requests must carry **`Tailscale-User-Login` equal to
the owner's login**; every other identity is denied; loopback callers (the desktop app, hooks, the
status-line command) are trusted as local; every remote action is logged with identity, path and time.
The phone may read everything, answer and tick, start and pause; free chat from the phone is parked.
**Threat model:** a lost phone and a compromised tailnet device are in scope (identity check plus loopback
bind, the action log, device removal from the tailnet, a kill-all for headless sessions from the desktop); a
compromised desktop is out of scope; the OAuth token is handled as in D-3 and D-23; hook and status-line
commands are path-bound to the tool's `cli` and fail silently. *Why.* The same model the owner chose for
their other tailnet service; Serve strips forged identity headers and the backend is unreachable except
through Serve or locally. *Alternatives.* A path mount under the existing hostname; no exposure until v2; a
PIN for write actions; periodic re-authentication; the tailnet as the only boundary.

**D-29 Operations and quality.** A release exe with a desktop shortcut and a Start-menu entry installed by
`scripts/install-shortcut.ps1`; the tool registers its own start-at-login entry (staggered by the owner's
Autorun Organizer); single instance by probing the loopback port and focusing the running window; closing
the window leaves it in the tray. A `doctor` command and a Diagnostics tile (Claude Code version against
the minimum, auth status, extension version, hooks and status-line entries present and pointing at the
current exe, Tailscale state, `D:` mounted, `gh` auth, registry and ledger integrity); a rolling log under
`%LOCALAPPDATA%` with daily rotation; headless launches refused with a reason below the minimum CLI version.
**Tests:** parser unit tests against fixture roadmaps with synthetic names covering every surveyed variant;
tests for the rule engine, prompt generator and registry migration; session-runner tests against a stub
`claude` executable replaying recorded stream-json, never the real CLI and never real usage;
self-screenshots behind a feature flag with fake projects; `cargo test`, `cargo clippy --all-targets`,
`cargo fmt --check` in every brief; GitHub Actions on windows-latest once the remote exists.
**Robustness:** child processes start one at a time with a minimum spacing, no burst file loops, atomic
writes (temp file then rename) with dated backups before registry migrations, an AV-quarantine hint in
Diagnostics, reconciliation of running sessions from `claude agents --json` on start. *Why.* The owner's
machine runs two AV products that flag bursts; the owner's other app established the install and
screenshot patterns. *Alternatives.* A portable exe; a service plus GUI; a log file only; unit tests only;
an opt-in live smoke test; documented AV exclusions.

> **Addendum (2026-11-01, EP-6).** A screenshot run (`CRW_SCREENSHOT=<file.png>`) keeps its registry,
> the window state and the backups in its own home (`CRW_HOME`, else a folder beside the picture),
> reads the tracked fake portfolio in `crates/app/fixtures/portfolio/` and refuses ticks there, so it
> writes nothing into a project or the owner's files; `CRW_HOME` alone also moves the window state, so
> a trial never touches the owner's saved layout. *Alternatives.* A scratch copy of the fake projects
> for each run; screenshots of the real portfolio kept out of git.

> **Addendum (2026-11-02, EP-9).** The stand-in `claude` the runner's tests start is `stub-claude`,
> a binary target of the core crate, so that integration tests find it through Cargo (a release
> build makes it too, and nothing uses it there); it plays scripts of synthetic stream-json from
> `crates/core/tests/streams/` and records what it is sent. Session starts keep two seconds apart in
> tests as anywhere. A real run happens only on the owner's yes, supervised, through a throwaway
> driver that is never committed, its transcript purged afterwards (`claude purge`).
> *Alternatives.* A stub crate of its own, found by path; an example target; an ignored test that
> runs the real CLI.

**D-30 Planning record.** This planning session is EP-0. Its record is `roadmap/planning-2026-10-31.md`
(verified state, research pass, domain map, interview record) with the portfolio survey in the git-ignored
`private/portfolio-2026-10-31.md`. The interview ran 18 rounds and 72 questions on 2026-10-31; every
recommended option was taken; the owner added the cross-project inbox (D-13), YAML front matter (D-12),
ntfy (D-21) and health badges (D-25), and parked the unpicked items (usage-over-time graph, portfolio
report, free chat from the phone, `--bg` launch mode). *Why.* The owner treats the interview as the design
step and wants the written documents to match it without a second pass. *Alternatives.* A shorter
interview; writing the roadmap from the brief alone.
