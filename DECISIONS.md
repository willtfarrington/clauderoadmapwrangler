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

> **Addendum (2026-11-04, EP-16).** P2 measured as P1 was: EP-9 … EP-15c took 15 h 28 min of
> session against 13 h 30 min planned (115 %), 10 h 27 min of it active (77 %); the other five
> hours were the owner's answers and trials of the real window. An M took 37 to 76 minutes of
> active work (52 on average), the two L's 62 and 53, the three S's 24 to 45; every L so far
> (EP-6, EP-9, EP-14) has taken about an hour of active work. The sizes stay the planning unit (the
> owner's choice): they hold for active time, and the owner's trials add wall time they do not
> count, which D-22's caps meet (EP-19's note). P2 grew by five rows, four at the owner's request
> (EP-14a, EP-15a, EP-15b, EP-15d) and one by a split at pickup (EP-15c), and this re-plan adds
> EP-35 (S): v1 is 41 briefs, 11 S · 24 M · 6 L ≈ 41.5 session-hours, 18 of them open once EP-16
> is checked. *Alternatives.* The three remaining L's (EP-19, EP-24, EP-30) re-cut to M.

**D-5 Dogfooding.** This project's own roadmap is the first registered project: the read-only dashboard
shows its next EP from P1, and **from the P2 re-plan (EP-16) every further EP of this project is started
from the tool**. `CLAUDE.md` records the rule. *Why.* The tool is only trustworthy if it runs the process it
manages. *Alternatives.* Managing this project only after v1; no rule.

> **Addendum (2026-11-04, EP-16).** The switch is made at P2's re-plan, as planned. From EP-17 on,
> every EP of this project starts from the window: its row's Start (or S) opens the launch dialog
> with the generated opener, and Open in VS Code hands it to a new Claude Code tab in this
> project's window (Visual, the default, D-16 addendum), where the owner chooses the model and
> effort and sends it; Start headless serves a brief that builds no release, since a headless run
> is the window's child and the release build needs the window quit. A session checks at its
> start that its prompt is the tool's opener and finds its row in the ledger (`crw sessions`: the
> mode, the EP, its session id), which is the record that it came from the tool; a session
> started by hand says why in its brief. EP-17's visual launch is the first, its row read at
> EP-16's close. `CLAUDE.md` rule 11 says so. *Alternatives.* The rule from EP-19, with the
> autopilot; the first self-launched session checked by its own brief instead of the re-plan.

> **Addendum (2026-11-08, planning session).** The caveat on headless starts ends with EP-37: the
> exe runs from `%LOCALAPPDATA%\clauderoadmapwrangler\bin\`, so a release build no longer needs the
> window quit, and the window restarts itself from the new exe between runs; from then this
> project's briefs may also run as chained headless runs under `auto` (the owner's choice; D-22
> addendum of the same date). Visual stays the way the owner starts a brief by hand, and rule 11
> says both. *Alternatives.* The exclusion kept through v1; the choice left to EP-23.

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

> **Addendum (2026-11-02, EP-33).** A dismissal is kept on its project as `dismissed = { … }`,
> one key per kind of proposal holding the fact it rested on: `all_rows_checked` the roadmap's
> row count, `pickup_gate` the gated brief's id, `no_commit_since` the last commit's time. A newer
> dismissal of a kind replaces the older, and the proposal returns once its fact differs (rows
> added at a re-plan, the next brief, a new commit). The row's menu and the Detail pane's proposal
> frame dismiss, `crw dismiss` too; Restore and `crw undismiss` take a dismissal back; `crw scan`
> and the Detail pane show the dismissed apart. *Alternatives.* A dated list of every dismissal;
> dismissals that expire after a set time; no undo.

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

> **Addendum (2026-11-02, EP-12).** A running session is a reason of its own, `InProgress::Session`
> in the session's words, which the window adds to what the files say: its own headless runs, and
> the sessions the hooks and the poll of live sessions report in the project's folder or one inside
> it. `crw roadmap`, which runs no window, reads the files only. *Alternatives.* The snapshot asking
> for the live sessions itself; a column of its own for sessions.

> **Addendum (2026-11-02, EP-13).** A session open with no prompt yet (the hooks saw it start and it
> has not worked since) is no reason: it edits nothing, so the row's Session cell, the Detail pane and
> the launch dialog's warning leave it out until its first sign of work (the owner's choice). The
> Detail pane says the reasons for every project, whether or not its roadmap has a next row, as the
> list's Session cell did already (the owner's finding after OT-6). *Alternatives.* Any open tab a
> reason at once, as EP-12 had it.

> **Addendum (2026-11-10, EP-21).** The migration's prompt is generated
> (`formalise::migration_prompt`, `crw formalise <project> --migration`) for a roadmap whose first
> phase table is the five-column or the work-packet form, and launches nothing: the owner hands it
> to a headless session in that project, from the launch dialog once EP-21a builds Migrate…, or
> from the chat (EP-24). The
> session reads the project and runs `crw roadmap` first, puts its plan to the owner, converts the
> tables to the house header (ids, order, brief, size and depends-on cells kept; a Core column;
> `[x] hash` → `☑` and the hash backticked; `[~]` → `☐`, named in the record), moves each done
> packet's handoff status into a completion note of its brief (the handoff sections and files
> kept), keeps every file and the old briefs' wording, checks with `crw roadmap` that the next EP
> and the counts are unchanged, records the migration in the project's own decision log, and
> stops before committing. *Alternatives.* The tool rewriting the tables itself (D-11 forbids
> it); a prompt that also writes the formalise step's files.

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

> **Addendum (2026-11-08, planning session).** The cadence of this project's own tasks changes (the
> owner's choice, after sixteen briefs in a row, OT-4 … OT-19, each ending with one task gated before
> the next EP, a practice the P1 and P2 re-plans had kept because the owner's looks found what the
> fake portfolio hides). From EP-19b a session writes an owner task for what needs the owner's hands
> only: a write into the owner's user settings or into another project, a migration an older build
> refuses, the first real run of a new route; it gates the task `[before EP-m]` only when EP-m builds
> on it, since the autopilot's gate refuses a chain into EP-m while such a task is open
> (`prompt::due`), which is unchanged. A look at a changed surface is no owner task: the brief's
> Verification section ends with a **Residual risk** paragraph naming what the session could not
> exercise and why it expects it to hold, and each phase's re-plan (EP-23, EP-29, EP-32, grown from S
> to M) opens with the **walk**: the steps compiled from those paragraphs, taken by the owner live with
> the window beside VS Code, each finding a row, an addendum or a v2 note at once; no task is written
> for it. OT-19, the last task of the old practice, lost its gate the same day, since EP-19b does
> not build on it. The other projects learn the cadence through the house style: its sentence went
> to the environment repository's inbox (the owner's choice), never written from here into another
> project's rules (rule 4). *Why.* A task between every two briefs slowed the owner more than it found, and the gate
> turns each gated one into a stopped chain; the owner accepts some debugging where the session's own
> checks (D-29 addendum of the same date) make success likely. *Alternatives.* Per-EP tasks kept but
> never gated; a second, mid-phase walk; a checklist task walked before the re-plan; a task after each
> change to a surface the owner uses daily.

**D-11 Write-back.** Inside a managed project the tool writes **only files it owns**: `owner-tasks.md`
ticks, `inbox.md`, the adoption files (a `CLAUDE.md` rule block, the two new files, the brief template) and
the `.code-workspace` file. It **never edits `roadmap/README.md`, briefs or `DECISIONS.md`**; sessions do,
and commits are made by sessions (D-22). *Why.* The roadmap files are the sessions' and the owner's record;
two writers would corrupt it. *Alternatives.* Ticking Done and recording the hash from the tool; read-only
with ticks in the registry.

> **Addendum (2026-11-04, EP-15b).** A revert is the tool's first write into a project beyond the
> files it owns: it puts the project back as it was when a session started, which is the project's
> own earlier state. It runs only on the owner's word after its preview (`crw revert <project>
> [<id>] --yes`; the window's Revert is EP-15c), never by the autopilot or a rule, and never while
> a session works or waits there. It never fetches or pushes, and a commit the remote has refuses
> it (v2 notes). Before anything changes, the work is copied out of the project into the backups
> folder, under `<project folder>\reverts\<stamp>\`: the commits as a bundle, the uncommitted
> changes as a binary patch, and the untracked files it removes or overwrites. Then come
> `git reset --hard` to the start, the session's untracked files removed (never an ignored file:
> no `git clean`), the start's uncommitted files put back from their copies, and the tool's own
> ticks since the start redone. For that, each session's start lists the project's uncommitted
> files and copies them into the backups folder, under `<project folder>\starts\<session id>\`,
> writing nothing into the project. A start copies at most 200 files and 20 MB; above that it
> keeps the list alone, and a revert leaves those files as they are. The registry's
> `revert = false` turns both off for one project (`crw set <project> revert yes|no`, on by
> default; the owner turns it off on the data repositories). *Alternatives.* Claude Code's
> checkpoints, which miss what Bash changed and every commit; `git stash` or other objects written
> into the project's `.git`; undoing pushed commits by `git revert` or a force-push; starts recorded
> in the data repositories only when turned on.

> **Addendum (2026-11-04, EP-15c).** The window's Revert runs on the same engine, behind the same
> preview. Besides the engine's refusals, the window refuses what `crw revert` cannot see: its own
> run in the project while it works, stops or waits its turn, and an ended run deferred with a
> request open; a visual launch on its way there, or a session's start being read; a session the
> hooks or the poll report at work or waiting there; and, while the poll of live sessions has not
> answered or fails, any revert, since a session could then work there unseen. At the owner's word
> the plan is read again, with the window as it stands then, and the revert runs only when that
> plan is the one the owner saw (the same target, commits, files, start, ticks and sessions, and no
> new refusal); otherwise the dialog shows the new plan and nothing changes, so that work made
> after the preview is never left uncopied. Quitting waits while a revert runs. A screenshot run's
> plan is canned, and its Revert reverts nothing. *Alternatives.* The plan reverted as it was
> previewed; only the refusals checked again at the click; `claude agents --json` asked again by
> the window at the click.

> **Addendum (2026-11-10, EP-21).** The adoption files are written by the formalise step
> (`crw_core::formalise`, `crw formalise <project|folder> [--yes]`), split at pickup (the owner's
> choice): the window's Formalise… dialog and Migrate… launch are EP-21a's.
>
> - **The files.** The rule block in the project's `CLAUDE.md` (the root's, else
>   `.claude\CLAUDE.md`, where the opener reads them; a new root file with a title when there is
>   none), between the lines `<!-- roadmap-wrangler:begin -->` and `<!-- roadmap-wrangler:end -->`;
>   `roadmap/owner-tasks.md` and `roadmap/inbox.md`, each with a header whose example line stays in
>   backticks (the inbox's is the mailman's, now citing `clauderoadmapwrangler D-13` as the owner's
>   hand-carried inboxes do); `roadmap/_TEMPLATE.md`, the brief template.
> - **The block** says the house's rules in short (the owner's choice, though the owner's
>   user-level rules carry the method too, so that a project formalised elsewhere has them): read
>   first, inbox triage, the brief from the template with its front matter, the completion note
>   with model, effort and duration and the `Handed on` blocks after it, the owner-task line, the
>   house table. A project's own rules outside it win; its `Read first` item is the one the opener
>   quotes when the project has none of its own.
> - **The preview and the write.** The plan reads the files by path only and asks git nothing:
>   each file is created, takes the block at its end (after a blank line, every byte kept, its
>   line endings followed), has the block replaced between the markers, or is kept; a file that is
>   there is never replaced, and markers that cannot be read (an end without a begin, two blocks),
>   a linked `CLAUDE.md` or a folder that is not there refuse it; markers inside a code fence are
>   text. The write reads every file again and refuses when one changed since the preview,
>   copies `CLAUDE.md` into the backups folder (`<project folder>\CLAUDE.<date>.md`) before changing
>   it, creates an inbox under the process's inbox lock, and is refused while a session works or
>   waits in the project; it is logged as an `event` row of kind `formalise`. A registered project's
>   state stays the owner's (D-6).
>
> *Alternatives.* The tool's forms only in the block, with a pointer to the owner's house-style
> rules; the window's dialog in the same brief; a write that also replaces an older header.

> **Addendum (2026-11-10, EP-22).** `crw formalise --yes` leaves the session that runs it out of
> its session check, known by the `CLAUDE_CODE_SESSION_ID` Claude Code gives the commands its tools
> run (2.1.316), so that the plugin's formalise skill can write from a session in the project:
> that session's turn waits for the write. Any other session at work or waiting there still
> refuses it, and `crw revert` still counts every session. Found by EP-22's review: the skill's
> route had always been refused by the session itself. *Alternatives.* The skill handing the
> owner the `--yes` command to run after its turn; the session found by its process id.

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

> **Addendum (2026-11-10, EP-20).** The mailman is built (`crw_core::inbox::mailman`, the window's
> `app::mail`). Split at pickup (the owner's choice): the compose dialog, the urgent flag, ntfy and
> the finished popup's gist are EP-20a's, and the `@<project>` prompt hook is EP-20b's.
>
> - **A letter** is a `Handed on (to <project>)` block of a registered target, its words without
>   the label, up to a later paragraph of its quote that opens with a bold label of its own. Its
>   key is its brief's id and a hash of its receiver and its words, so a letter goes out once, and
>   a block whose words change is a new letter.
> - **The delivery** appends `- [ ] IN-n (from <sender> EP-k, <date>) <text>` to the receiver's
>   `roadmap/inbox.md`, the next free id, wrapped at 100 columns. The file as it was is first
>   copied into the backups folder, and a missing file is created with a header in the house
>   form. A message already there from the same sender and brief with the same words, carried by
>   hand or delivered before, counts as delivered. The write runs in the process's turn, as a
>   session's start does, so a starting session holds the message in its copy or starts before
>   it; one lock serialises the process's inbox writes (the window's tick, the delivery).
> - **When.** A letter goes out once its brief is committed (git reports no change to the brief),
>   so that a draft of a completion note is never sent, and only while no session works or waits
>   in the receiver (the window's runs, the hooks, the poll; an idle tab does not count), so that
>   a running session is never disturbed. Otherwise it waits for the next round. The window runs a
>   round once the poll of live sessions has answered and the files are read, then after each of
>   its runs ends, when a session starts or ends, and every five minutes; only the window that
>   holds the loopback port, where the hooks arrive, runs them (a second window would miss a
>   session at work and could send a letter twice), and a screenshot run never does.
> - **The first reading counts as delivered.** The owner carried every message by hand until now,
>   so the mailman's first reading of a project, the first round or the first after its adoption,
>   counts the blocks of its committed briefs as delivered and sends none. A brief a session is
>   still writing then is no past: its blocks go out once it is committed. Mail to a project read
>   for the first time (adopted later, or its folder back) is its past too.
> - **The log** is the ledger's `event` rows of kind `mail`, under the sender: the first reading,
>   each block counted then, each delivery with the message's id; `crw mail` prints it. No schema
>   change. The Detail pane says each block's state.
> - **A revert keeps it** (EP-15b's rule for the tool's own writes). Each delivery is also a `tick`
>   row of the receiver, `delivered IN-n · (<meta>) <text>`, which a revert of a session there
>   redoes after the restore, unless the message is there. A row cut at the ledger's cap of 1,000
>   characters holds no whole message and is named instead.
> - **The gate leaves `roadmap/inbox.md` out of its clean-tree condition** (D-22), so a message
>   delivered between two chained runs ends no chain: the next brief triages it first.
>
> *Alternatives.* Delivering at once whatever the receiver does; delivering a completion note
> found in the working tree; sending the history at the first round; keeping a delivered message's
> key by its place in the brief; the mailman waiting between a chain's runs instead of the gate's
> exception; a table of deliveries of its own (a schema 6).

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

> **Addendum (2026-11-02, EP-12).** The loopback server is axum with its `http1`, `json` and `tokio`
> features only, on a thread with a current-thread runtime of its own (tokio's `net` joins the
> pinned features), handing what it hears to the frame loop over a channel. The relays post with a
> hand-written HTTP/1.1 request over a plain socket, no client crate, so `crw hook` starts and ends
> in milliseconds. serde_json keeps object keys in their order (`preserve_order`, with the
> `indexmap` the lock held), so the owner's Claude Code settings keep their layout when the tool
> writes into them. *Alternatives.* axum's default features; a client crate in `crw`; an ordered
> JSON value of the tool's own.

> **Addendum (2026-11-03, EP-14a).** The window installs a panic hook at start. A release build has
> no console, so a panic ended the window without a word (one did on 2026-11-03, and its message is
> lost). Each panic, on any thread, now appends a report to `crash.log` in the local folder
> (`CRW_HOME` honoured): crw's version, the UTC time, the thread, the place, the message and a
> backtrace. A log past 1 MB moves to `crash.old.log`. The hook then hands on to the one before it.
> The next start names the last report once in the status bar's error line (`crash.seen` keeps
> which). A screenshot run installs no hook and reads only its own home's log. *Alternatives.* A
> dialog at the moment of the crash (a Windows dependency of its own); `panic = "abort"` with
> Windows Error Reporting's dumps (the session's thread would no longer record the run's end); the
> report in the ledger (a panicking thread should not open SQLite).

> **Addendum (2026-11-04, EP-17).** The usage meter's one HTTPS request goes through WinHTTP,
> Windows' own HTTP stack, called through the `windows` crate the lock already pinned (its
> `Win32_Networking_WinHttp` feature), so no client crate joins the lock, and TLS, the
> certificate store and the proxy settings are the system's (the owner's choice at EP-17's
> pickup). A loopback host, the tests' listener, is reached over plain HTTP without a proxy.
> *Alternatives.* ureq with native-tls; reqwest with rustls; `curl.exe` from System32 at each
> poll, the token on its standard input.

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

> **Addendum (2026-11-02, EP-10).** The launch dialog's models come from the catalog Claude Code
> caches in `~\.claude\cache\model-catalog\` (format 2; the CLI's surface preferred, then the latest
> fetched; the files' names carry account identifiers, which the tool never keeps or shows): the
> main section first, each model with its effort levels and the oldest CLI it runs on, the aliases
> resolved; a built-in list of the four main models stands in when no file reads. The dialog offers
> the effort levels both the model and the CLI's help list, ultracode only when the help lists it,
> and the help's permission modes, and says when a model needs a newer CLI than the one asked.
> *Alternatives.* The built-in list alone; the CLI's current model setting as the default.

> **Addendum (2026-11-02, EP-11).** A `Code.exe` the visual launch starts also withholds the
> `ELECTRON_*` and `VSCODE_*` variables, which the tool inherits when it runs under VS Code (a
> session's shell holds `ELECTRON_RUN_AS_NODE=1`, with which `Code.exe` starts as Node), besides the
> API key and the session markers: VS Code hands the asking `Code.exe`'s environment to the windows
> it opens, where a marker would stop the extension's own CLI. `windows` names `Win32_Foundation`
> beside `Win32_System_Threading`, both compiled in through process-wrap already (the lock is
> unchanged), to ask whether a lock file's process still runs. *Alternatives.* A fixed list of VS
> Code's variables; `code.cmd`, a batch file that reads a link's `%xx` as its own arguments.

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

> **Addendum (2026-11-02, EP-10).** Until the sessions tile (EP-13) and the decisions tile (EP-14),
> the window holds each headless session it starts (the owner's choice): the row's Session column
> marks it in progress, the Sessions pane lists it in plain lines (running, waiting for an answer,
> stopping, or how it ended; its last line of text; the wishes the CLI dropped) with Stop, Kill and
> Dismiss, and the Decisions pane lists what it waits on, read-only. A request waits until the
> session is stopped, and closing the window kills whatever still runs. A start runs on a worker
> thread, which waits for the turn and the spacing (D-29); a screenshot run starts nothing.
> *Alternatives.* Bare answers in the Decisions pane at once (part of EP-14); every request denied
> at once, the session listing its open decisions in its final message.

> **Addendum (2026-11-02, EP-11).** Visual is built. The dialog's Visual | Headless switch resolves
> like the permission mode, the project's `[project.launch] mode` over the tool's default, which is
> Visual until EP-14 lets a headless session answer the questions a brief ends with (the owner's
> choice; EP-16 revisits it). Open in VS Code writes the project's own `<name>.code-workspace` when
> it is missing (EP-33's writer) and opens it with `Code.exe`, which brings the project's window to
> the front or opens it, since the extension's link goes to whichever window has the focus; it
> waits up to 60 s for the extension's lock file (`~\.claude\ide\<port>.lock`, its process running,
> the project its first folder; the auth token is never kept), then hands
> `vscode://anthropic.claude-code/open?prompt=…` to `Code.exe --open-url --`, as Windows' handler
> does. VS Code decodes the link once and the extension's `URLSearchParams` again, so `%`, `&` and
> `+` are escaped twice: one supervised launch at this project's window, on the owner's yes, filled
> a test prompt holding all three into a new tab exactly, unsent. A link over 32,000 characters is
> refused (a Windows command line holds 32,767). The launch is a `visual` ledger row with no session
> id, model or effort; a Cancel while it waits hands nothing over; `Code.exe` comes from a `vscode`
> setting, the user install, the system install or the install of `code.cmd` on PATH.
> *Alternatives.* The link fired at once when a window lists the project (it reaches the focused
> window, maybe another project's); `code.cmd` or the shell's `start`; a fixed delay instead of the
> lock file; Headless as the default.

> **Addendum (2026-11-02, EP-13).** Resume is built: an ended headless session of the window's runs
> again with `--resume <its id>` in place of `--session-id` (the CLI keeps the id unless asked to fork
> it), with the launch's model, effort, permission mode and neighbours and the message the owner types
> in its tab as the prompt; the launch is refused when the CLI's help lists no `--resume`. Each run is
> a ledger row of its own under the same id, and the session's tab keeps one transcript across its
> runs. A visual or manual session is not resumed from the tool, nor a headless one after the window
> restarted, whose launch's choices lived in the window (v2 notes). *Alternatives.* The session's row
> reopened for each run; a tab per run; resume from the ledger's rows.

> **Addendum (2026-11-04, EP-16).** Visual stays the default launch mode (the owner's choice), for
> reasons other than EP-11's, since a headless run answers a brief's questions from EP-14 on. With
> no allow rule in this project's settings or the user's, a headless run under `acceptEdits` waits
> for the owner in the Decisions tile at every shell command it runs. This project's own briefs
> end with a release build that needs the window quit, which ends the window's headless runs. And
> the owner's briefs close with rounds of questions and trials, which a VS Code tab carries. Headless
> stays a switch in the dialog, a project's `[project.launch] mode` can choose it, and the
> autopilot (EP-19) runs headless whatever the default. A run that a crash or a quit window cut
> short, and a visual or manual session, are resumed from their ledger rows in the project's VS
> Code window through the extension's `session=<id>` link (EP-35, before EP-19); the headless
> resume after a restart stays in the v2 notes. *Alternatives.* Headless as the default, with
> Visual written for this project in the registry; Headless everywhere, each release build an
> owner task; the resume after a crash folded into EP-19, or left to `claude --resume` by hand.

> **Addendum (2026-11-04, EP-18).** In Visual mode the launch dialog greys out the model, effort
> and ultracode pickers, as it does the permission mode, with "headless only: choose it in the new
> tab's panel" (the owner's choice). The extension's link carries the prompt alone: at EP-16 an
> effort chosen in the dialog did not reach the tab. The rules' choice, the Fable downshift's
> among them (D-23 addendum), stays in view as the suggestion the opener names, and switching to
> Headless enables the pickers. *Alternatives.* The pickers left enabled, with a plainer line.

> **Addendum (2026-11-05, EP-35).** Resume in VS Code is built. A ledger card in the Sessions
> pane's All tab whose row has a session id shows it beside Revert…. It opens the project's own
> workspace file, waits for the extension's lock file as a visual launch does, then hands
> `vscode://anthropic.claude-code/open?session=<id>` to `Code.exe --open-url --`. Nothing is sent:
> the owner writes the next message in the tab. The extension takes a UUID only and drops any other
> id without a word, so the tool refuses one first. A session that a tab already holds comes to the
> front; any other opens in a new tab. One supervised check at this project's window, on the
> owner's yes, found that a visual session's id opens with its conversation, while a headless
> (`-p`) run's opens as an empty tab: the extension's list of past sessions leaves out the SDK's,
> and its tab loads none. So a headless card's button is disabled, and its hover text says that
> `claude --resume <id>` in a terminal takes the run up; each card now shows its session's full
> id. The headless resume after a restart stays in the v2 notes, as the window's only way back to
> such a run. The button is also disabled, its reason on hover, for a project that is not
> registered or whose folder is missing, for the window's own session or one that runs elsewhere,
> while the list of live sessions has not answered or fails, and while another launch is on its
> way. *Alternatives.* The resume offered on headless rows too, opening an empty tab; a prompt
> handed with the resume; the button enabled while the list of live sessions is unknown.

**D-17 The launch prompt.** A generated house-style opener, the same for Visual and Headless, shown in an
editor before launch and overridable by a per-project template: project name and path; the project's own
read-first order; the next EP row (id, title, size, depends-on) and its brief-notes bullet; open inbox
messages and owner tasks with the triage rule; the rules to write the brief first (front matter), end with
the completion note in the new form, and stop before committing (or commit and record the hash when the
autopilot holds the standing yes, D-22). *Why.* The prompt is the one place the tool can carry the method
into a session without editing `CLAUDE.md` files. *Alternatives.* "Execute EP-n" alone; templates only.

> **Addendum (2026-11-02, EP-10).** The opener (`core::prompt`) names the project and its folder and
> the date, model and effort the session runs with, for its front matter and its completion note;
> quotes the read-first rule of the project's `CLAUDE.md` (the list item or paragraph that opens with
> **Read first**, or a section headed read order, load order or read first), else the house
> default; gives the next row, its brief notes verbatim, its brief when one is written (picked up,
> not rewritten) and the last completed brief; lists the open owner tasks and inbox messages as
> written, names the tasks gated at or before this brief as due, and adds the triage rule while a
> message is open; and ends on five rules: the brief first, the project's `CLAUDE.md`, owner
> decisions through `AskUserQuestion`, the completion note with owner tasks and hand-offs, and stop
> before committing. A template in `[project.launch] template` uses the opener's parts as
> placeholders (`{ep}`, `{note}`, … and `{opener}` for the whole), `{{` and `}}` for braces; a name
> the tool does not fill stays as written and is reported. The dialog regenerates the prompt as
> the choices change until the owner edits it, and copies it; `crw prompt <project>` prints it with
> the resolution. *Alternatives.* The older forms' owner items in the prompt; a template file per
> project; one global template.

> **Addendum (2026-11-02, EP-11).** A visual launch's opener says the session was launched into VS
> Code, whose Claude Code panel chooses its model and effort, names the launch's suggestion and asks
> the session to record the ones it runs with; the dialog marks the model, effort and ultracode as
> that suggestion and the permission mode and the neighbours as headless only. `crw prompt
> <project> [--visual|--headless]` prints either wording, by default the project's mode.
> *Alternatives.* One wording for both launches; no model sentence in a visual opener.

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

> **Addendum (2026-11-09, EP-19b).** The window sets the per-project override, and guards the
> push it allows.
>
> - **The offer.** Turning a project's autopilot on where its headless runs would wait at each
>   shell command (`autopilot::would_wait` over the project's two settings files and the user's)
>   offers `auto` first. The dialog says why the runs would wait and what `auto` means (Claude
>   Code's permission-modes page, read 2026-11-09: a classifier model reviews each action that is
>   neither a read nor an edit in the working folder and blocks what goes beyond the task or looks
>   risky; explicit ask and deny rules still apply). It also says what `auto` means for the
>   project's commits. It offers Set auto and turn on (one write of the registry), Turn on as it
>   is, and Cancel.
> - **Keep for this project.** A pin beside the launch dialog's Permissions picker opens a menu
>   (the owner's choice, over checkboxes written at Start and one Keep row under the pickers).
>   Keep for this project writes `[project.launch] permission_mode` at once: the tool's inline
>   form `launch = { permission_mode = "auto" }` when the project has no `launch`, and a table the
>   owner wrote is kept with its comments. Back to the default takes the key out, as keeping
>   `acceptEdits` does. `crw set <project> permission_mode <mode|default>` does the same.
> - **The push guard** (the owner's choice). `auto` allows a plain push to a branch of the working
>   repository by default. So every headless run of a project whose autopilot is on, chained or
>   started from the dialog, gets the deny rule `Bash(git push *)` through `--settings`, beside
>   the neighbours' `Edit` rules. The rule refuses `git push` as written, inside compound commands
>   and under `auto`. It does not catch a push written another way (`git -C . push`), so it guards
>   the usual case and is no boundary. A CLI without `--settings` drops the rule and says so; the
>   prompt's "never push" stays.
>
> *Alternatives.* The prompt's "never push" alone; the deny rule for every headless run of the
> window; a control for the mode in the Autopilot section; allow rules written by the tool.

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

> **Addendum (2026-11-02, EP-14).** Built. A `-p` run takes the stream-json host's answers only
> when it is started with `--permission-prompt-tool stdio` (the Agent SDK's form; 2.1.316 names the
> flag only inside `--permission-prompts`' description): without it, as the runner had run since
> EP-9, a headless run offers no `AskUserQuestion` and denies whatever would prompt, so D-18's
> requests never reached the window. The runner passes it whenever the help names it, else says it
> could not. The Decisions tile answers a window's headless session while it waits: a permission
> request with Allow or Deny, its tool, call, reason and input shown, a reason optional; an
> `AskUserQuestion` with each question's options, one or several, and the owner's own words, sent
> (a question left blank goes unanswered) or declined with a message. A project's
> `answer_timeout` in minutes (`crw set <project> answer_timeout <minutes|none>`, none by default)
> defers a run whose request waits that long: the runner refuses it with the turn interrupted, the
> run's outcome is `deferred`, and the owner's later answer resumes the session with the answer in
> its message. The defer route is chosen by `settings.decisions = "defer"` (`crw setting decisions
> defer|protocol`), or by the window for the later runs of a CLI and version whose handshake it saw
> refused, since a running process takes no new hooks: `--settings` carries a `PreToolUse` hook on
> `AskUserQuestion` running `crw.exe hook --defer`, the run ends with `tool_deferred`, and the
> answer resumes the session with no prompt (a prompt would drop the deferred call, Claude Code's
> hooks page) and a one-shot `crw.exe hook --answer <file>` that allows that call with the
> questions and their answers as `updatedInput` and defers any later question; the file lives in
> the tool's local folder while the resume runs. A session Claude Code reports waiting elsewhere is
> listed with Focus when it waits in VS Code (`Code.exe` opens the project's own workspace file,
> else its folder); the tool never answers it. Verified on the owner's yes by a supervised probe
> (the cheapest model at low effort in a scratch folder, its throwaway driver deleted and its
> transcripts purged): a first round of three runs found the missing host; with the flag a second
> round answered a question through the protocol, deferred a run at a 5-second limit (the session
> reads the CLI's own words for an interrupted refusal, not the tool's message), and ended a run at
> its question on the defer route and resumed it, without a prompt, to the answer. *Alternatives.*
> A plain refusal at the time limit, the session going on without the answer; one limit for every
> project; a defer hook that asks the window at run time; restarting a run whose handshake is
> refused.

**D-20 Signals.** The tool installs **user-level hooks** (Notification, Stop, SessionStart, SessionEnd) in
`~/.claude/settings.json` that run its `cli` binary, which posts the event to the loopback server and exits
silently when the tool is not running; it installs a **status-line command** the same way (D-23); and it
polls `claude agents --json` as reconciliation. The hooks see visual sessions too, since the extension fires
the same events. *Why.* Hooks are the documented event source; polling covers sessions started without the
tool. *Alternatives.* Polling only; hooks only.

> **Addendum (2026-11-02, EP-12).** Built, and installed on the owner's yes. `crw signals install`
> writes a group per event into the user settings that runs the release `crw.exe` directly with
> `hook` (Claude Code's exec form, `command` and `args`: no shell, no quoting) with a 5 s timeout,
> and the status line (D-23 addendum), after a copy of the file in the tool's backups folder; the
> file's other keys stay byte for byte. With `--inbox` the start hook prints the project's open
> inbox messages as written, which replaced the owner's stopgap PowerShell hook (IN-2; its script
> stays), and a later install keeps it so. The entries are known by the program they run, so a
> second install changes nothing and `crw signals remove` takes them out; another status line is
> said, never replaced; a debug build is refused. The relay keeps the session's id, folder, event
> and its source, reason or notification type, and the session's surface from
> `CLAUDE_CODE_ENTRYPOINT` (an SDK's marks another program's `-p` run), never a transcript's text,
> and posts within 500 ms, of which the connection gets 50: Windows refuses a closed loopback port
> only after its own retries, so with no window a hook costs about 75 ms. The window polls `claude
> agents --json` every 5 s while it has the focus and every 30 s otherwise (the owner's choice; a
> listing costs about 0.2 s of a core), the session markers withheld. The state model
> (`crw_core::sessions`) gives each session the registered project whose folder holds its own, a
> mode (visual when it claims a launch's row, headless, manual) and a status (busy, waiting with
> what for, idle, done, failed); a session the poll listed and then leaves out twice has ended, and
> an ended one leaves the window after ten minutes. *Alternatives.* Shell-form commands through Git
> Bash; `type: "http"` hooks, whose refused connection is a hook error the session shows; a 5 s
> poll whatever the focus; the stopgap kept beside.

> **Addendum (2026-11-02, EP-13).** The `UserPromptSubmit` hook waits for EP-20, which adds its
> `@<project>` text to it (the owner's choice): busy comes from the poll, which asks at once when the
> window gains the focus, and the state model reads the event as busy and as work when it arrives. A
> session that ends without having worked leaves the window at once, not after ten minutes.
> *Alternatives.* The prompt hook installed now, for busy at once whatever the window's focus.

**D-21 Notifications.** Windows toasts (WinRT through notify-rust, attributed to the tool's Start-menu
entry); a tray icon with a badge count and a menu (Show, Start next for a pinned project, Pause autopilot,
Quit); the Claude app's own push for the phone; **ntfy over the tailnet** (a topic configured in the
registry, shared with the owner's other project's planned container). Toast at once for "waiting for
input", on finished or failed, and at 80 % and 95 % of weekly or Fable usage; badges only for a new
unadopted folder, an orphan or mismatched remote, a stale backup and inbox arrivals; quiet hours
configurable, default none. *Why.* Attention is the scarce resource; the phone already gets Remote Control
pushes. *Alternatives.* Toasts and tray without ntfy; only waiting and failed; every event.

> **Addendum (2026-11-03, EP-15).** Built in the window; ntfy waits for EP-20 and the usage
> notifications for EP-17, each adding its moment to the rule table (`crw_core::notify`).
>
> - **A popup in place of Windows toasts** (the owner's choice: Windows toasts are off on their
>   machine). The tool draws a small window of its own at the screen's lower right. It is always
>   on top, takes no keyboard focus when it appears and has no taskbar button. It lists the
>   latest notifications, the newest first (three, with a count of the rest and Dismiss all),
>   each with Show and Dismiss, until the owner dismisses them or acts on one. It is drawn while
>   the window is hidden in the tray too. The Windows-toast path built first, through
>   tauri-winrt-notification 0.8.1, came out with its crate.
> - **What pops up.** A wait: a window's run asks, a run is deferred at its time limit or by the
>   defer route, a session elsewhere waits for a permission prompt, an MCP server's question or
>   input. An end: a window's run completes, a session elsewhere ends a turn of 2 minutes' work
>   or more, a background session is done. A failure: a window's run or a background session.
>   A stopped run, a session closed by hand and a session that never worked show nothing.
> - **Held back**: in the quiet hours (`settings.quiet_hours`, `HH:MM-HH:MM` in local time,
>   past midnight if need be, set with `crw setting quiet_hours`); while the window is in front
>   with the focus, where the status bar says it instead; a session elsewhere's second wait
>   within 2 minutes. One session's moments of one pass make one notification. Show brings the
>   window forward on the Decisions pane or the session's tab, or a VS Code session's window.
> - **The badges** (an unadopted folder; an orphan or missing repository, or an origin a
>   current head check finds diverged or not fetched; a stale, failing or uncovered backup; an
>   open inbox message) are new until the window is next in front with the focus; the first
>   reading at the start counts as seen.
> - **The tray icon** (tray-icon 0.26.1) is drawn by the tool until EP-28 brings the app's own: a
>   red count of what waits for the owner (the Decisions tab's count) and an amber dot while a
>   badge is new, both said in its tooltip. A left click shows the window; the menu says what
>   waits and what is new, then Show, Start next for each pinned project (the launch dialog,
>   D-17), Pause autopilot (disabled until EP-19) and Quit.
> - **The log**: an `event` row of kind `notification` for each notification and what became of
>   it (popup, quiet hours, in front, a repeat), each one opened with Show and each badge; `crw
>   notifications` reads it. File, Send a test notification puts one in the popup whatever the
>   hours.
>
> *Alternatives.* Windows toasts, as D-21 had them (built, then taken out: off on the owner's
> machine), through notify-rust, whose click holds a thread waiting per toast, or through the
> crate it wraps; a popup that fades like a toast; the main window brought forward instead;
> notifications while the window is in front; one count for the waits and the badges; the tray's
> menu rebuilt at each change, which could destroy it while it is open.

> **Addendum (2026-11-04, EP-15d).** The popup is a deferred viewport, no longer an immediate one.
> On 2026-11-03 the window closed on a panic in the popup ("egui backend is implemented incorrectly
> - the user callback was never called"). A notification had come while the window was in the
> tray. Windows sends a hidden window no redraw, so eframe 0.37 paints it from
> `check_redraw_requests`, and from its `new_events` handler that runs without the thread-local
> event loop through which an immediate viewport is given a new window. The popup got none, its
> callback never ran, and egui panicked. A deferred viewport is given its window at the start of
> every pass with the event loop in hand. The window's pass keeps the popup open while it holds
> toasts and wakes it for a new one; the popup's own pass draws them and applies the owner's choice
> there and then, over state the two share. The File menu also sends a test notification in ten
> seconds, time to close the window to the tray, which reaches that path. *Alternatives.* Moving
> eframe, which waits for a re-plan (D-15); keeping the immediate popup's window open at all times;
> catching the panic.

> **Addendum (2026-11-06, EP-19a).** A chain's moment is a kind of its own, `chain`, a toast
> (the owner's choice: every change of a chain's state): a brief chained, the chain held (once
> per change of its words) and the chain ended (a refusal, a cap, a start the runner refused, a
> brief left out of the plan). Its title is the project's name and the chain log's line up to
> its first colon, its text the rest; its Show brings the Sessions pane's All tab forward, where
> the chain log is; a hold or an end is said in amber, and the popup marks the kind in the
> autopilot's violet. The quiet hours and a window in front hold it back like every toast, the
> status bar saying it instead, and the notifications' log keeps it as `chain`, which `crw
> notifications` prints. *Alternatives.* Ends only, or ends and holds, the chained start left to
> the status bar; the chain's moments as the project's `finished` and `failed`.

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

> **Addendum (2026-11-02, EP-10).** The rule table is `[[rule]]` (global) and `[[project.rule]]`
> (per project) in `registry.toml`: `match` by `size`, by `phase` (the heading's id) and by `title`
> keywords, any of which the title holds, case aside (the owner's choice over a regular expression,
> which would add a crate), every condition given holding; `set` of `model`, `effort` or a relative
> `effort_step`, and `ultracode`; an optional `name` as its reason. A launch resolves in layers,
> each value keeping its reason: the tool's default (`fable` at `xhigh`), the project's
> `[project.launch]` model, effort and permission mode, the first global rule that matches, the
> first project rule that matches. While the registry lists no global rule, a built-in table stands
> in: an S brief one effort step lower; it is compiled in and never written into the owner's
> registry (CLAUDE.md rule 13). A step moves along the CLI's effort levels and holds at their ends;
> ultracode asked of a CLI that does not list it is turned off, with that reason. *Alternatives.*
> Title regular expressions; the defaults written into the registry at first use; a project's rules
> replacing the global table rather than layering over it.

> **Addendum (2026-11-05, EP-19).** Built (`crw_core::autopilot`, `app::autopilot`), split at
> pickup (the owner's choice): the owner's model and effort for each brief before a chain starts,
> and the chain's own notification when it stops, are EP-19a's.
>
> - **The switch** is `[project.autopilot] enabled`, written as `autopilot = { enabled = true }`
>   and taken out when off; the list's box, the A key and `crw autopilot <project> on|off` turn
>   it. On, it is the owner's standing yes to the project's commits: every launch's last rule, a
>   visual one's too, commits in the house pattern, records the hash and never pushes. Off, the
>   session stops before committing, and its ended tab offers Commit, a resume with the same words.
> - **The gate** is read after each headless run of the window's whose project has it on, or that
>   the chain started, on a worker thread from a fresh reading (the files, git's status, the
>   ledger's day, the settings files), and each condition that fails is logged: the autopilot on
>   and not paused by the owner; the run completed; its brief has a completion note and its row is
>   checked; a next brief, not a re-plan, no larger than `max_size` (M by default), with no owner
>   task gated before it; weekly and Fable under `autopilot_pause_percent` (90); a clean tree; the
>   day's chained runs under the project's `daily` (2) and `autopilot_daily_total` (4); no other
>   session open in the project; not the tool's own project (the owner's choice: the one whose
>   folder holds the window's exe, its briefs ending with a release build that needs the window
>   quit); runs that would not wait at each shell command (the owner's choice: acceptEdits, manual
>   or plan with no allow rule in the project's `.claude` settings or the user's refuses, and the
>   owner opts a project in with `permission_mode = "auto"` or allow rules).
> - **The figures.** A weekly or Fable figure at or over the mark counts until its reset, whatever
>   its age: the automatic pause. One under it counts within 15 minutes of its reading. One older,
>   or read before its window reset, holds; one never read (Fable while the endpoint is off)
>   refuses.
> - **Held or ended.** A chain the pause alone refuses (the owner's Pause, a figure at the mark, a
>   figure too old) is held in the window, read again every 5 minutes and at the Resume, and goes
>   on when the gate passes; any other refusal ends it. A held chain does not outlive the window.
>   A run deferred with a request open ends the chain until the answer's resume, whose end weighs
>   the gate again.
> - **The chained start** is drafted as the dialog drafts one (the rules' model and effort, the
>   downshift included) and started headless with the project's permission mode, route, hooks,
>   revert start, answer timeout, neighbours and `max_budget_usd` (none by default; passed when
>   the CLI lists the flag). The registry's pause and switch and the project's other sessions are
>   looked at again just before.
> - **Caps.** Every headless run of a project whose autopilot is on has a cap by size (S 45 min,
>   M 90 min, L 3 h; a brief with no size takes L's), counted in working time: a request waiting
>   for the owner stops the clock (the owner's choice). At the cap the run is stopped gracefully,
>   its work left in the tree, which ends the chain; the autopilot never reverts. A revert leaves
>   the switch as it is, since nothing chains until a run of the owner's completes.
> - **Pause.** The owner's Pause (the Sessions pane, the tray, `crw autopilot pause`) is
>   `settings.autopilot_paused`, written only as `true`; it holds chains and never stops a run
>   (D-13).
> - **The log** is the ledger's `event` rows of kind `autopilot`, no schema change, each line's
>   words beginning with what happened; the day's chained runs are counted from its `chained`
>   lines, in local time. The Sessions pane's All tab opens with its latest lines (D-25), and
>   `crw autopilot` prints them.
>
> *Alternatives.* Caps on the wall clock; chaining this project with the release build left to an
> owner task; chaining whatever the permission mode, the runs waiting in the Decisions pane; every
> chained run under `auto`; ending a chain at any refusal, the pause's too; the log as a table of
> its own (a schema 6).

> **Addendum (2026-11-06, EP-19a).** The chain plan, on the owner's choices at pickup, after the
> first real chain (2026-11-06) ran its second brief at the rules' choice where the owner had
> wanted another model and effort.
>
> - **The plan** is the owner's word on the briefs a chain may run after the one they start
>   headless. The launch dialog of a project whose autopilot is on lists, under the prompt and
>   the choices, the next unchecked rows in table order after the launched brief, up to the
>   project's `daily`, each with a Chain box (ticked), its size and title, and the Model and
>   Effort pickers and the Ultracode box as the main choices offer them, the rule table's
>   resolution over its row the default (the downshift included) and its reason on hover. The
>   list stops at a row the gate would refuse on its own account (a re-plan, a size over
>   `max_size`, no size) and says so, and names the owner tasks due before a row. In Visual the
>   section is one line, since a chain follows a headless run only.
> - **The chain takes it.** Start headless keeps the plan in the window, by project, as a held
>   chain is kept (the owner's choice: a chain does not outlive the window either, and no
>   registry key is added). When the gate drafts a brief, a row the owner touched runs with the
>   pickers as they left them, a row left alone with the rules' choice resolved then (the
>   downshift judged then), a row unticked ends the chain before it ("left out of your plan",
>   which gives the number of runs, the owner's choice over a count), and a brief the plan does
>   not list (a row added since) runs with the rules' choice, the log saying so; the chained
>   line names whose choice it was. The plan goes when the chain ends, when the project's
>   autopilot is turned off, when a start the runner refuses leaves no run, or when a new
>   headless launch of the project replaces it; a held chain keeps it. The Sessions pane's
>   Autopilot section shows each plan.
> - **Every chained draft is headless**: its opener says the model and effort the run is given.
>   Before, the chain took the project's launch mode's wording (Visual by default), so a chained
>   run read that VS Code's panel chooses its model and effort.
> - **The chain's moment** is D-21's: a notification of the chain's own kind at each change of
>   its state.
>
> *Alternatives.* The plan in the registry (`[project.autopilot]`), consumed at each chained
> start and printed by `crw autopilot`, stale after a restart and refused by the builds before;
> a count of runs beside the plan; the plan-time resolution kept for a row left alone.

> **Addendum (2026-11-08, planning session).** This project chains (the owner's choice, reversing the
> EP-19 pickup's), through EP-37: the install step copies the release exe to
> `%LOCALAPPDATA%\clauderoadmapwrangler\bin\` (the running one renamed first, which Windows allows)
> and the shortcut, the hooks and the status line point there, so a build never meets the running
> window or a hook; the window restarts itself from the installed exe when no headless run of its own
> works, at the gate's moment or at the owner's word, its held chains and chain plans carried through
> a handover file in its own folder (the v2 items of EP-19 and EP-19a picked up); the gate's
> own-project condition goes. Its runs pass the waiting condition under `permission_mode = "auto"`,
> set once through EP-19b's "keep for this project" (the owner's choice over allow rules): a chained
> run runs its shell commands unasked, commits in the house pattern and never pushes, so the owner
> reviews the diff before each push, and rule 3's word check runs before each commit. The gate's
> owner-task condition is unchanged: a task gated before the next brief still refuses the chain, and
> the D-10 addendum of the same date makes such a task rare. *Alternatives.* Never chaining this
> project (EP-19); the release build left to an owner task after the chain; allow rules for the build
> and git commands under `acceptEdits`; the handover kept in the registry.

> **Addendum (2026-11-09, EP-19b).** The autopilot's settings are set in the window.
>
> - **The section.** The Sessions pane's Autopilot section has a line for each project whose
>   autopilot is on. It shows the project's permission mode, and whether that mode is the
>   project's own or the default. It shows the gate's verdict on the conditions that stand between
>   runs (runs that would wait, the tool's own project), read with the log every 30 seconds, and
>   the sessions open there that would end a chain now. It shows the limits as pickers that write
>   at once: the largest size (S, M or L), the runs a day, and a budget from presets. `crw set
>   <project> max_size|daily|max_budget_usd` takes any figure. A default is taken out of the
>   registry, as `set_autopilot`'s writers do.
> - **Idle sessions** (the owner's choice, after a real chain ended at two idle VS Code tabs on
>   2026-11-09). The gate's "no other session open" condition counts the window's own runs, a
>   chained start on its way and the sessions seen busy or waiting. A session whose turn is over
>   no longer refuses; the chained line names it ("open there and idle, not counted"). Typing into
>   it while a chain runs makes two sessions edit one tree, and that is the owner's own act.
> - **The split** (the owner's choice). The standing model and effort set from the dialog are
>   EP-19c's. There, "keep for this project" writes a `[[project.rule]]` that matches every brief,
>   after any phase rule, so a kept effort holds for an S brief too. This was not the recommended
>   option, which wrote `[project.launch]` as the base that the built-in S rule steps down from;
>   `[project.launch]` model and effort stay hand-written.
>
> *Alternatives.* Idle sessions refusing as before, with the section naming them; the question
> parked for the P3 re-plan; the limits in the Detail pane; figures typed into the section.

> **Addendum (2026-11-09, EP-19c).** A standing model and effort, kept from the launch dialog.
>
> - **Rules combine key by key** (the owner's choice at pickup). Every rule that matches a brief
>   applies. Within the global table (or the built-in one standing in for it), and then within
>   the project's, the earliest matching rule that sets a value wins. An `effort_step` steps from
>   what the later rules and the layers below give. Before, each table applied only its first
>   matching rule (the EP-10 addendum). A phase rule placed before a rule for every brief would
>   then have hidden the latter from that phase's briefs. The owner's registry held no rule, so no
>   launch changed.
> - **The Keep menus.** A pin beside the dialog's Model and Effort pickers, in the form of
>   EP-19b's pin, opens:
>   - Keep for this project, which writes a `[[project.rule]]` with no condition, named "kept for
>     this project", after every other project rule, so that a kept effort holds for an S brief
>     too;
>   - Keep for phase Pn, when the brief's phase has an id, which writes one with
>     `match = { phase = "Pn" }`, named "kept for phase Pn", before the rule for every brief;
>   - Back to the rules, for each of the two that keeps a value.
>
>   Each item writes at once. Each picker writes its own key of the rule's `set`, an effort
>   replacing an `effort_step`. Back takes the key out, and the rule with it once its `set` is
>   empty. The tool finds its rules by their match, so a rule the owner wrote in that form is the
>   one it writes into. `crw set <project> model <id|default>` and `effort <level|default>` write
>   the rule for every brief.
> - **The dialog follows.** The brief is resolved again from the registry, and the picker takes
>   the new value, its reason naming the project's rule. When an earlier rule or the downshift
>   stands over the kept value for this brief, the picker keeps the value shown and the notice
>   names what stands over it. The chain plan's rows that the owner left alone are resolved again,
>   and the rows they chose are kept. The downshift stays the last layer (D-23), so it may still
>   propose Opus 5.5 for an S brief over a kept Fable model.
>
> *Alternatives.* First match kept, with a phase keep copying the project's kept values into its
> rule, or with a warning in the notice; key by key in the project's table alone; the window
> writing `[project.launch]` model and effort (EP-19b's recommended option).

> **Addendum (2026-11-10, EP-37a).** The gate's moment and the tool's own project.
>
> - **The own-project condition is gone.** `Refusal::OwnProject`, `Facts::own` and `is_own` are
>   removed, and the Autopilot section's line for it with them. The tool's own project chains like
>   any other once its runs would not wait (`permission_mode = "auto"`, set by the owner through
>   EP-19b's control).
> - **The gate's moment.** A gate that passes while a newer build is installed
>   (`Standing::Installed`, looked at again then) holds the chain "for the window's restart into
>   the installed build" and restarts the window. The build must be another than the window runs,
>   so a gate never restarts the window into its own build; the move out of `target\release`
>   stays the owner's. The handover (D-29 addendum of this date) carries the held chain and the
>   plans, and the new window's gate starts the next brief, in the new build. A gate that holds
>   (the pause) takes the restart too, the held chain carried. Once the restart is decided, the
>   old window reads no gate again.
> - **What it leaves alone.**
>   - The gate's moment installs nothing: only a build `crw install` installed is taken, never a
>     build found in `target\release`.
>   - A restart refused then (another headless run works, a dialog is open) lets the chain go on in
>     the old window. The log says why the build waits.
>   - A chain that ends restarts nothing, the owner's choice at pickup: its popup stays in front of
>     the owner, and the chip says that a newer build waits.
> - **The opener** of the tool's own project's runs says that the release build runs with the
>   window up and that the session's last step is the install (`pwsh -File
>   scripts\install-shortcut.ps1`). A headless run is told that its install's restart is refused
>   while it works: the window restarts at the gate's moment when a chain goes on or is held after
>   it, and otherwise File > Install and restart takes it. A visual one is told that the window
>   restarts once nothing stands in the way. Templates name it `{build}`.
>
> *Alternatives.* A chain's end restarting the window too, the handover carrying the end's words
> for the new window's popup, or the popup lost; the gate's moment installing a settled build
> found in `target\release`; holding a chain until the restart can go.

> **Addendum (2026-11-10, EP-22).** **What a re-plan is, to the gate.** The gate took EP-22 for a
> re-plan and ended the chain after EP-21: its test matched `re-plan` or `replan` anywhere in a
> title, and EP-22's names the plugin's replan skill. The test still fails safe, since a wrong yes
> only ends a chain: the word (`re-plan`, `replan`, `re-planning`) counts wherever it stands, but
> for an item inside a comma list (`skills plan, replan, formalise`) and a mention of another
> re-plan, the word within two words after an article or a preposition (`Found at the P3 re-plan:
> …`, `notes from the last re-plan`). It always counts as the first word, before a phase's id
> (`re-plan P5`, `Re-plan R0`, `re-plan phase 2`) and after a leading one (`P3 re-plan: …`), the
> forms every re-plan row read across the portfolio's roadmaps takes, the combined ones
> (`Tooling, a template, re-plan P3`) among them (`autopilot::is_replan`, its test of 25 titles).
> *Alternatives.* The title's first clause (it misses the combined rows); the word only as the
> first or last word or before a phase's id (it missed a re-plan with a trailing note or markup,
> found by the review); a marker in the row.

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

> **Addendum (2026-11-02, EP-12).** The status line is `<crw.exe> statusline` with
> `refreshInterval` 30, the path in forward slashes for Git Bash (a status line has no exec form);
> it posts the session's model, cost, context and `rate_limits` to the window and prints nothing.
> The VS Code extension (2.1.314) names `statusLine` only in its settings schema and shows plugins'
> statuses rather than the user's command, so the feed most likely comes from terminal sessions
> only, where a configured status line also hides most of the footer's key hints; the owner had it
> installed all the same, and EP-17's degraded meter inherits the finding. *Alternatives.* The
> hooks only until EP-17; a status line that prints the figures.

> **Addendum (2026-11-04, EP-17).** Built (`crw_core::usage`, `app::meter`), on WinHTTP (D-14
> addendum).
>
> - **The token** is read from Claude Code's `.credentials.json` at each poll into a value that
>   never prints and is overwritten when it drops. An expired one is not sent. The request goes to
>   `api.anthropic.com` alone, with the beta header and the tool's own user agent.
> - **The schedule.** The first poll comes 5 s after the window starts, then one every 5 min, and
>   one as soon as a launched session ends, though no sooner than a minute after the last poll
>   (a visual or headless session's end as the hooks report it, or a window's run's). After a
>   failure of the endpoint
>   the meter waits 1, 2, 4, 8 and 16 min, then 30, and honours a 429's `Retry-After` up to a day.
>   A problem on this machine (no credentials, an expired token) is looked at again each minute.
> - **The reading** keeps `limits[]` (weekly_all; the scoped limits by name, Fable's among them),
>   `five_hour`, `seven_day` when no weekly_all is given, the breakdown by surface and extra
>   usage. A `limits[]` kind it does not know is kept under its own name and shown in the pane,
>   so a limit the endpoint adds or brings back still shows. Each reading also keeps the answer's
>   shape, its keys and value kinds with the limits' kinds by name and never a number or a text,
>   which `crw usage` prints. An answer with no weekly or scoped limit is a schema change, named
>   by that shape.
> - **The merge.** While the last poll succeeded and is under 15 min old, the endpoint's figures
>   stand. Otherwise each figure is the newest of its last reading and the status lines' (Fable
>   the reading's alone), and a failed poll marks the strip and the pane degraded, with the reason.
> - **The records.** Each successful poll is a `usage_sample` row of source `endpoint`, and the
>   cache the next start shows first (`badge_cache`, project `""`, source `usage`). A session's
>   status line writes a row of source `statusline` when its figures change. Each change of the
>   meter's state is an `event` of kind `usage`; `crw usage` reads them all, and only the window
>   polls.
> - **The notifications** are a toast kind of their own (`notify::Kind::Usage`), of no project,
>   never merged with another in a pass, whose Show brings the Usage pane forward. Each of the
>   80 % and 95 % marks of the weekly and Fable limits is sent once per limit window, the window
>   known by its reset time (a share back under 80 % starts a new one); a limit that reaches 95 %
>   at once sends that mark alone. What was sent is kept in the cache (source `usage-alerts`), so a
>   restart does not repeat it.
> - **The switch.** `settings.usage_endpoint = false` (`crw setting usage_endpoint off`) keeps the
>   meter on the status lines alone. `on` takes the key out, since a build before EP-17 refuses
>   it.
>
> *Alternatives.* The token kept in memory between polls; a poll from `crw`; a mark notified at
> each crossing whatever the window; egui_plot for the sparkline (it would move the egui family,
> D-15).

> **Addendum (2026-11-04, EP-18).** The downshift is built as the resolver's last layer, after the
> project's own rules: `rules::resolve` takes a `rules::Usage`, the meter's Fable figure with the
> time it was read.
>
> - **What it weighs.** An S brief whose model so far is a Fable model (the alias or a Fable id,
>   whether the default, the project or a rule chose it) is proposed `claude-opus-5-5` at the
>   same effort while the figure is above `settings.fable_downshift_percent`. That is a whole
>   percent, 60 by default (`crw setting fable_downshift_percent <n>`, or `default`, which takes
>   the key out). M and L briefs are not weighed, and the owner's choice in the dialog stands over
>   it.
> - **The figure's age.** A figure above the mark counts whatever its age, since a window's share
>   only grows. One whose window has ended counts for nothing. One under the mark and older than
>   15 minutes is said to be possibly higher now.
> - **Where it shows.** The judgement is kept in words for every S brief on Fable. The launch
>   dialog shows it under the model, and `crw prompt` prints it as its `downshift` line, from the
>   reading the window cached in the ledger, without the credentials or the network.
> - **The endpoint's `is_active`** is no sign of a window. On 2026-11-05 it gave the weekly limit
>   `true` and the Fable-scoped one `false`, both resetting three days later. So a figure counts
>   until its reset (`Limit::before_reset`), for the downshift and for the 80 % and 95 %
>   notifications. Until EP-18 the notifications skipped a limit the endpoint called inactive,
>   and so would have missed Fable's marks. `is_active` still hides a five-hour figure.
> - **The attribution and the usage-share badge** moved to EP-18a when EP-18 was split at pickup
>   (the owner's choice).
>
> *Alternatives.* The downshift before the rules, so that a rule's model stands over it; a figure
> older than the meter's 15 minutes ignored; a rule's opt-out.

> **Addendum (2026-11-05, EP-18a).** The attribution is built as a split of each poll (the
> owner's choice at pickup).
>
> - **The split.** Each reading of the endpoint's, set against the window's previous one, gives
>   each figure's rise: the difference within one limit window, or the new figure after a reset.
>   The rise is split equally among the sessions that worked in between, and each part goes to
>   the session's latest ledger row. The window's first reading after its start splits nothing,
>   and the cached reading a start shows is never the one before.
> - **Who worked.** A session worked when the window heard its turn end (`Stop`) or a wait for
>   the owner begin (a `Notification`), or saw it busy at a pass (a hook's or the poll's
>   status), or when it is a headless run of the window's that runs. A steady wait is no work.
> - **Manual sessions count** (the owner's choice), so "tool-launched" now reads "every session
>   the window sees work in a registered project". Sessions outside the registry, and those
>   without a row, take their parts too, which stay unattributed. Use the window cannot see
>   (claude.ai, the phone, Cowork) falls to whoever worked in the interval.
> - **The figures** are whole percents (all 14 samples at the pickup), so a part is often a half
>   or a third of a point. A session's share is coarse, and a week's sum per project is the
>   steadier figure.
> - **Shown.** The Sessions pane's ledger card and `crw sessions` give a row's share with the
>   figures around it (`week +0.5 % (27 → 28 %)`). `crw sessions` also sums each project's over
>   the weekly window: the sessions started since the reset less seven days. The badge is
>   EP-18b's, split off at pickup.
>
> *Alternatives.* The figures before and after each session, at its last turn or at its row's
> end, whose sums count parallel use twice; launched sessions only; a split weighted by busy time.

> **Addendum (2026-11-05, EP-18b).** The usage-share badge is built (D-25 addendum).
>
> - **Its figure** is the project's weekly sum this week in points of the weekly limit, the unit
>   of the meter's own figure (the owner's choice). So it holds while other projects work, and
>   the projects' figures never add up to more than the account's. The project's part of all
>   that was attributed joins its sentence. Fable's sum is in the sentence alone, also the
>   owner's choice.
> - **The window's reading.** The meter reads the sums from the ledger without a write
>   (`meter::Week`), for the week that the weekly figure's reset closes. It reads them again when
>   that week moves on and after a split wrote shares, and tries a failed read again after 30 s.
>   At each pass the board puts each registered project's sums into its row by name, case aside,
>   since a files read replaces the rows.
>
> *Alternatives.* The part of all that was attributed as the figure, which sums to 100 % but
> moves whenever any project works; both figures in the cell; a read every half minute, like
> the Sessions pane's.

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

> **Addendum (2026-11-02, EP-13).** The sessions tile is a tab strip in the Sessions pane: All, then a
> tab per headless session of the window's. A session's tab has its header (the state chip, model,
> effort, start and runs; Stop and Kill while it runs, Dismiss after) and its transcript as it streams:
> the prompt folded, the assistant's text with its partial text appended, each tool call in one line
> with its result folded under it (the first lines only), what it asked the host, the refusals, the
> notices, the result; once it has ended, Resume with a message (D-16 addendum). All lists the
> window's sessions, those started elsewhere and the ledger's latest 200 rows, filtered by project,
> mode and running only, the filter saved with the view. Busy is blue, waiting amber, an ending green
> or red, idle grey (busy and idle shared the grey before, the owner's finding). In a narrow pane a
> chip has a row of its own and no row is wider than the pane: one wider row widens everything after
> it past the pane's edge, and a folding header's text never wraps. The transcript lives in the
> window's memory only, 2,000 entries at most (D-3, D-26). *Alternatives.* An egui_tiles pane per
> session, which the saved layout would keep; the list in a window of its own; a transcript file per
> session.

> **Addendum (2026-11-03, EP-15a).** The Detail pane sets the next brief's notes for reading. A
> segment that opens with a bold label ending in a colon (`**From EP-7:**`) starts a paragraph under
> its label, which stands on a line of its own; numbered parts `(1) … (2) …` in sequence become an
> indented list; blank lines stay; paragraphs are spaced apart; in a wide pane the lines hold at
> about a hundred characters. The runs take three colours the window already uses, each covering
> more than one kind (the owner's choice at the trial, over a first set of seven): a label and a
> `[before EP-m]` gate in the amber of a waiting session's words; the ids `EP-n`, `D-n`, `OT-n` and
> `IN-n` and a link's text in the violet of the unformalised chip; a path or a command in the code
> in the green of a passing run, a name in the code in the code face's own ink; struck text weak.
> No segment folds (the owner's choice). An owner item, an open inbox message and a handed-on
> block's text share the renderer: a long one (more than two sentences, or over 240 characters) is
> set a sentence to a bullet, and an item's gate is an amber chip beside its id, repeated neither at
> the text's end nor in its meta line (the owner's choices). Nothing is reworded or dropped; a form these
> rules do not know is one paragraph; the launch dialog's prompt stays source text, and a row's
> title keeps its plain marks. A row's labels sense hover only, so the row takes a click, a right
> click and the hover anywhere on it, and each label keeps its hover text. The group headers, the
> folder rows and a window session's card in the Sessions pane are alike, and their text can no
> longer be selected; the Detail pane's can. The red of failures has a shade per theme, and a
> session waiting elsewhere reads "waits for you: …" on its row and card, as the popup says it.
> *Alternatives.* Segments that fold, open at first or folded to their labels; owner items left
> whole; a colour per kind (a teal accent for labels, the link blue, an orange command, a gold
> name), which read off the window's theme; colours in the list rows' titles; references and links
> that open what they name (v2).

> **Addendum (2026-11-04, EP-15c).** A session's Revert has three places (D-11 addendum): Revert…
> at the end of an ended session's footer, after Resume and its message; a small Revert… on a
> ledger card's chip line for each row with a session id, on a line of its own when the pane is too
> narrow for both, so that a run whose tab a crash took keeps it; and "Revert the latest
> session…" at the foot of a project's row menu. Each is disabled, its reason on hover, when the
> project's `revert` is off or its folder is missing. The preview is a dialog like the Quit
> question, at most 700 points wide: a spinner while the plan is read, then `Plan::lines()` in a
> scroll area of at most 55 % of the window's height (the first line strong, the commits and files
> indented in the code face, a refusal in the window's red, the sessions to close in its amber),
> then Cancel and Revert in the red, or Close alone; Escape and Cancel close it, a click beside it
> does not, and nothing does while the revert runs. Afterwards it shows what the revert did, and the
> status bar says where the work is kept. No new colour. *Alternatives.* The plan's parts as a
> table; Revert in the tab's header beside Dismiss; a card's Revert shown on hover only.

> **Addendum (2026-11-05, EP-18b).** The usage share is a badge column of its own, Usage, after
> Backup and in both of the badges' forms. The give-way order is otherwise as before.
>
> - **The cell.** It shows the project's points of the weekly limit this week (`+6 %`, D-23
>   addendum) in the text's own colour, or `+0 %` weak when its sessions worked through no rise.
>   It shows nothing without a share or a weekly reading. The figure is written whole or to a
>   tenth, and Fable's sum stays in the sentence. No new colour.
> - **The key and the Detail pane.** The legend's Usage group sits between Backup and Row. The
>   Status section's sixth subsection, Usage, gives the badge's sentence, or why there is none,
>   and a line on the method.
> - **The sort.** Usage share ranks the weekly sums as the cell writes them, the largest first,
>   equal figures by name and the projects without one last. Every key of D-25 now has its data,
>   so the menu greys none out.
>
> *Alternatives.* Both figures in the cell (`+1 % · F +0 %`); the column among the hosting
> badges, where D-25 lists the usage share; the raw sums as the sort's key, which orders figures
> the cell writes alike.

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

> **Addendum (2026-11-02, EP-12).** The hooks complete the ledger: a session's start claims the
> newest open visual launch of its project from the last 30 minutes (EP-11's row gains its session
> id), else opens a `manual` row for an interactive session of a registered project; its end
> closes a visual or manual row as completed. A headless row stays the runner's; another program's
> `-p` run, a session outside the registry and a session seen only by the poll (its start and end
> unknown) get none. *Alternatives.* Rows for every session the poll lists; no manual rows.

> **Addendum (2026-11-02, EP-13).** A session the hooks report earns its row at its first sign of
> work (a finished turn, a wait for the owner, a prompt, or the poll listing it busy or waiting), and
> only when the hooks saw it start; the row starts at the session's own start, and a compaction's
> start is no start (the owner's choice after OT-6, where opening a tab wrote three rows: a helper of
> the extension's, a tab the window restored and the owner's session). A visual launch is taken up
> only by a session that starts as a new conversation within its 30 minutes and works before they run
> out; a launch no session took up lapses, closed by the window at its launch plus 30 minutes with no
> session and no outcome, which a run length leaves out (the owner's choice: the two launches recorded
> before the hooks close at the window's next start). Schema 3 adds `result`, a run's final text kept
> to 8,000 characters, and `context_percent`, which an end keeps with the cost from the session's last
> status line. `crw sessions` and the Sessions pane read the ledger without a write
> (`Ledger::open_read_only`, an older schema read with the columns it has), and a migration step takes
> the write lock before it reads the version, since the window's loader and its first frame open the
> ledger at the same moment. The rows written before EP-13 stay (rule 13). *Alternatives.* A row only
> when a transcript is on disk at the end, which a restored tab has; a row at every start, as EP-12
> had it; lapsed launches shown but never closed; a longer claim window.

> **Addendum (2026-11-02, EP-14).** The runner writes `questions` (a question the defer route's
> hook ended the run at counts) and `answer_latency_s`, there since V1: the mean seconds from each
> request the owner answered in the run to the answer, a deferred request's answer counted on the
> resume that carries it, from the deferral to the resume; a refusal at the time limit is no
> answer. The All tab and `crw sessions` say both ("2 questions, answered in 1 min"); no schema
> change. *Alternatives.* The total wait; the median; the latency kept on the run that asked.

> **Addendum (2026-11-04, EP-15b).** Schema 4 adds `session.start`, the project at the session's
> start as JSON: its HEAD and branch, its uncommitted files each as changed, untracked or deleted,
> where their copies are, and whether all were copied. It is written once, into the session's first
> row. The runner reads it just before a headless run's child spawns; a resume reads none and keeps
> its first run's. The window reads it at a VS Code or terminal session's start hook, keeps it until
> the session earns its row (EP-13), and drops it with its copies when the session ends without
> working. A session with no recorded start is reverted to the newest `docs(roadmap): record EP-n
> commit hash` commit made before it started, and the preview says that the uncommitted changes
> from before cannot be told from the session's own. The `event` table gains two kinds. A `revert`
> row names the session, its target, the counts and where the work is kept. A `tick` row is each
> tick the window or `crw tick`/`untick` makes (`ticked OT-3`, `cleared OT-3`, `ticked IN-2 ·
> <disposition>`), which a revert redoes. `crw sessions` prints each session's short id.
> *Alternatives.* A table of starts of its own; the copies kept in the ledger; the HEAD at the
> session's start guessed from the commit times.

> **Addendum (2026-11-05, EP-18a).** Schema 5 adds `session.weekly_share` and `fable_share`: the
> session's part of each usage figure in points of the limit, added up poll by poll on the latest
> row of its session id, so a resumed run takes the parts from then on (D-23 addendum). The same
> write fills V1's `weekly_before`/`after` and `fable_before`/`after` with the figures before the
> first interval that counted and after the latest. A ledger opened to read at an older schema
> reads none, and a build before EP-18a refuses the ledger. *Alternatives.* The parts in a table
> of their own; the shares in the event log.

> **Addendum (2026-11-05, EP-35).** A resumed run has a row of its own (the owner's choice). The
> hooks see a session start with the source `resume`; at its first work, if the session's latest
> row has ended or is a headless run's (a row the runner keeps), a new row starts under the same
> id at the resume's start, with the latest row's EP. The new row is `visual` when the window
> handed the session to VS Code within the 30 minutes before, else `manual`; the handover is a
> `resume` event whose words begin with the session's id. Its end hook closes the row, and its
> usage share goes to it, as the session's latest row (EP-18a). A session whose latest row never
> got its end goes on in that row. Revert still goes back to the session's first start (EP-15b).
> In the watch's model, a session that starts again after an end the window heard begins anew, so
> a run resumed within ten minutes of that end earns its row too. No schema change.
> *Alternatives.* The old row reopened, its end moved to the new run's; no row for a resumed run,
> as before.

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

> **Addendum (2026-11-10, EP-22).** The plugin is built, split at pickup (the owner's choice):
> loading it into the tool's own headless runs with `--plugin-dir` is EP-22a's.
>
> - **The plugin**, `plugins/roadmap-wrangler/`: `.claude-plugin/plugin.json` (the name
>   `roadmap-wrangler`, no `version`, so each push that changes it is a release, versioned by
>   Claude Code from the commit); the skills `plan`, `replan`, `formalise` and `status`, invoked as
>   `/roadmap-wrangler:<skill>`; and `reference/house-style.md`, the schema in full, which quotes
>   the formalise engine's texts byte for byte (a structural test, `crates/core/tests/plugin.rs`,
>   fails when they differ). The formalise skill writes through `crw formalise` and its preview,
>   from a session in the project it writes into or by the owner's hand; never by hand where the
>   tool is there, and without it the reference's texts only on the owner's word, under the
>   engine's rules (D-11 addendum of this date).
> - **The marketplace**: this repository is one, `.claude-plugin/marketplace.json` at its root
>   (the name `clauderoadmapwrangler`, one entry with the source `./plugins/roadmap-wrangler`).
> - **The owner's own sessions** install it from GitHub after a push (the owner's choice): `claude
>   plugin install roadmap-wrangler --marketplace willtfarrington/clauderoadmapwrangler`, later
>   `claude plugin update`. A local clone added by its path loads in place from the working tree
>   instead, unfinished edits included, which is why GitHub leads. A `--plugin-dir` copy replaces
>   an installed plugin of the same name for its session, so the runs EP-22a gives the flag use the
>   build's copy.
> - `claude plugin validate` passes the plugin and the marketplace with two warnings (no
>   `version`, no listing icon), run by hand: tests never run the CLI (D-29).
>
> *Alternatives.* The plugin and `--plugin-dir` in one brief; a `version` bumped by hand at each
> change; the local folder as the install route; skill files written into `~/.claude/skills`.

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

> **Addendum (2026-11-02, EP-12).** The server listens on `127.0.0.1` at the registry's `port`
> (47831 by default, `crw setting port`) with `POST /hook`, `POST /statusline` and `GET /health`,
> and answers only a request without an `Origin` whose `Host` names the loopback, with a JSON body
> of at most 256 KiB, so a web page can neither post to it nor reach it through a name pointed at
> 127.0.0.1. A second window finds the port taken and says so in its status bar. The phone's
> routes (EP-30) take the identity check in the loopback guard's place. *Alternatives.* No guard
> beyond the loopback bind; a shared secret in the hook entries.

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

> **Addendum (2026-11-03, EP-15).** Closing the window leaves it in the tray: hidden, with its
> headless runs, the loopback server and the poll going on, since eframe 0.37 runs `App::logic`
> while the window is hidden (on Windows a hidden window still gets eframe's pass at the pace of
> its repaint requests; it draws nothing then). The first close says so once, in the popup. Quit is
> in the tray's menu and in the window's new File menu: at once when no headless run would stop,
> else after a question that names the runs it stops. Without a tray icon (one that failed to
> start, or a screenshot run) closing quits, with the same question. *Alternatives.* Closing
> quits and Minimize alone keeps the window; minimizing to the tray rather than hiding.

> **Addendum (2026-11-08, planning session).** What a session verifies itself, in place of the owner's
> look after each brief (the D-10 addendum of the same date; the owner chose all four): (1) the
> **field check** at session start, the real window's records since the last brief's record commit
> read through `crash.log`, `crw notifications`, `crw sessions` and `crw autopilot`, anything odd
> triaged like an inbox message; (2) **screenshot runs over a copy of the real portfolio** (EP-36): a
> run reads the roadmap files the session copied into its scratchpad (`CRW_SCREENSHOT_ROOT`), under
> the run's guards (no port, no poll, no session, no tick, no GitHub), so a layout is checked on real
> content, the pictures never entering git; (3) a **review of the diff** for an M or L brief before the
> release build, by the `/code-review` skill at medium or a review subagent given the diff and not the
> session's conclusions; (4) the **residual-risk paragraph** closing the brief's Verification section,
> from which the phase walk is compiled. Tests still never run the real CLI, and no session starts the
> window on the owner's files; the one supervised probe on the owner's yes stays for a brief that
> drives the real CLI. *Alternatives.* The owner's look after each brief (the practice to EP-19a); an
> opt-in live smoke test (rejected at planning); a review pass for every brief.

> **Addendum (2026-11-09, EP-36).** Part 2 is built. A screenshot run reads the data root named by
> `CRW_SCREENSHOT_ROOT=<folder>` or the setup flag `root=<folder>` in place of the tracked fixtures.
> The flag wins, and a relative root is resolved against the run's working folder, since a
> registry's data root is a full path (D-8). Every guard of the EP-6 addendum holds, because
> `Board::fixtures` is set for every run: nothing is written into the root, no session starts,
> nothing opens, and the run binds no port, polls no live session and asks no network. A root that
> holds every folder of the fake portfolio reads with the fake registry. Any other root reads with
> a registry written from its folders, each active with a roadmap and unformalised without, at
> priority 3 with no pin. A home keeps the root its registry was first written for and refuses a
> run that names another. The copies are no repositories, so a copied portfolio's projects take the
> fakes' canned readings by their place in the list, under their own names. `scripts\copy-roadmap.ps1`
> copies what a row reads of one project: the roadmap folder's own files, the decision log, the ADRs
> and handoffs, `CLAUDE.md`, the license, and the stack's manifests, with lock files written empty.
> The README's pictures name a neutral root: a copy of the fixtures in a folder whose path names
> nothing of the machine, `C:\Users\Public\Projects`, which any user may create and a session may
> remove (Claude Code refuses to remove a folder directly under the drive root, such as
> `C:\Projects`). It replaces EP-8's `subst` drive and junction.
> *Alternatives.* A relative root kept as written, which would have shown `portfolio\harbor` but is
> refused by the registry's full-path rule; a run over the real data root, which would read the
> data repositories and run git in real repositories; one registry from the root's folders for
> every root, which would cost the fake portfolio's copies their unadopted folder and missing
> project; bare readings for a copy's projects, which would leave their badge columns empty.

> **Addendum (2026-11-10, EP-37).** The installed copy and the window's restart. Split at pickup
> (the owner's choice): the handover of held chains and plans, the restart at the gate's moment,
> the end of the own-project condition and this project's chaining are EP-37a's.
>
> - **The install.** `crw install` (`crw_core::install`) copies the release build of the
>   repository it was compiled in (`target\release`: the window's exe and `crw.exe`) into
>   `%LOCALAPPDATA%\clauderoadmapwrangler\bin` (`CRW_HOME\bin` in a trial). Each exe that differs
>   from the build is first copied beside its place under a staging name
>   (`crw.installing.exe`), so that a copy that fails changes nothing. Then the installed exe is
>   renamed to a dated previous copy (`crw.previous-20261110-081500.exe`), since Windows lets a
>   running exe be renamed but not replaced, and the staged copy takes its name; a rename that
>   fails puts the exe back and says what was installed. An earlier install's previous copies are
>   removed, and one that still runs is left for the next install. A copy keeps its build's last
>   write, so an exe installed as built is left alone. `crw install` does not build, since a build
>   would replace its own running exe: the script builds, runs `crw install`, writes the icon
>   into the folder and points both shortcuts at the installed window, started in that folder. A
>   trial (`CRW_HOME`) asks no window to restart, and the script refuses one that names no
>   scratch shortcut folders (rule 13).
> - **The hooks** (D-20) run the installed `crw.exe` once `crw signals install` writes it, which
>   it does whenever the copy exists (the owner's to run, rule 7). `crw signals` and `crw doctor`
>   name this exe, the installed copy and the exe the entries run.
> - **The restart.** File > Install and restart, the tray's item and `POST /restart` (`crw
>   install`'s request, answered at the window's next pass, within six seconds) first install a
>   build newer than the copy, once its exes have been left alone for 5 s (a linker still writing
>   one keeps it fresh). Then, when the installed window is newer than the build the window runs,
>   or the same build where the window does not run from, they start it with `--restarted <pid>`.
>   The window closes 0.4 s later, once the answer has gone out, unless the new process has ended
>   already, and the new one waits up to 10 s for the old process to end before it opens, so the
>   layout is saved and the port free. A restart is refused, with every reason, while anything
>   would be lost: a headless run of the window's that works, waits or is being stopped (D-13), a
>   session starting, a visual launch on its way, a revert, a gate read or a chained start, a held
>   chain or a kept plan (until EP-37a), a session's start not yet in the ledger (EP-15b), a
>   disposition being typed, an open dialog; and in a screenshot run. A refused request is not
>   remembered: the owner restarts later from the menu or the tray. A request that installed into
>   another folder than the window's is refused, and one whose caller stopped waiting (the
>   server's 504) is dropped. An amber chip in the status bar says when a newer build is built or
>   installed, looked at every 30 s.
> - **The own-project condition** reads the build's repository (`paths::repository()`), since
>   the window no longer runs from the project's folder. EP-37a drops it.
>
> *Alternatives.* `crw install` building too, which would replace its own running exe; the new
> window started at the old one's exit, where a failed start would leave no window; the held
> chains and plans carried now (more than an M, so split); the restarted window opening hidden
> when the old one was.

> **Addendum (2026-11-10, EP-37a).** The handover.
>
> - **The file.** Before it starts the installed window, the window writes `handover.json` in the
>   local folder (`crw_core::handover`; `CRW_HOME` in a trial) through a temporary name. It holds
>   a version, the writer's process id, the time, the held chains, the chain plans (each row with
>   its resolution) and whether the new window opens hidden. Nothing is written when there is
>   nothing to carry. A window with something to carry and no local folder does not restart. A
>   start that fails removes the file, and so does a new window that ends at once; the old window
>   keeps what it holds, its held chains are read again, and the gate's moment leaves that build
>   alone until the owner's own restart or a newer install.
> - **The take.** The window started with `--restarted <pid>` takes its old window's file at its
>   start: it reads and deletes it. The held chains go on, each gate read once the first poll of
>   live sessions has answered (or failed), so that a session open in the project counts, and the
>   first files read has landed, which gives the gate the project's folder; or after two minutes.
>   The plans are kept, and the start's notice says what was carried. A file of another version,
>   one that does not read, or one whose writer no longer runs is dropped at any start. The
>   notice and the chain log say what it held: an old chain never comes back. Another restart's
>   file whose old window still runs is left for its own new window, since a window started by
>   hand meanwhile (no single instance until EP-28) would otherwise take it. A delete that
>   security software refuses is tried again for a moment.
> - **The window's visibility.** A restart at the gate's moment from a window hidden in the tray
>   opens the new one hidden; the owner's own restart opens it shown, as before.
> - **The form.** Serde forms were added for the plan, its rows and stop, `Resolution` and its
>   parts, `Size` and `EpId` (by its text). The old build writes and the new one reads, so a brief
>   that changes these types keeps the old form readable or raises the version.
> - **What refuses a restart now.**
>   - A held chain and a kept plan no longer do.
>   - A session open with no prompt yet now refuses in every registered project, whatever its
>     revert switch. Only the window that saw it open gives it its ledger row at its first prompt,
>     and the review found that a restart left such a session in a project with revert off
>     without one.
>   - A disposition being typed and the dialogs still refuse.
>   - A restart at the gate's moment also waits while an ended run's tab offers Commit, which a
>     restart nobody asked for would drop.
>
>   The popup's notices and the other ended runs' tabs are not carried; their ledger rows remain.
> - **A session's mode after a restart.** A window learns a session's mode from the ledger at the
>   session's start hook. A session it first meets through the poll or a later hook now takes the
>   mode of its latest ledger row, looked up once, when that row is open and not a headless run's,
>   as the start hook's rule has it. Before, any window started while a visual session was open
>   called that session manual (EP-37a's field check).
>
> *Alternatives.* The handover kept in the registry (the 2026-11-08 addendum's rejected
> alternative); a plan carried without its resolution, made again in the new window; any start
> taking a recent file left by a dead window; the new window started hidden through the viewport
> builder rather than hidden at its start.

**D-30 Planning record.** This planning session is EP-0. Its record is `roadmap/planning-2026-10-31.md`
(verified state, research pass, domain map, interview record) with the portfolio survey in the git-ignored
`private/portfolio-2026-10-31.md`. The interview ran 18 rounds and 72 questions on 2026-10-31; every
recommended option was taken; the owner added the cross-project inbox (D-13), YAML front matter (D-12),
ntfy (D-21) and health badges (D-25), and parked the unpicked items (usage-over-time graph, portfolio
report, free chat from the phone, `--bg` launch mode). *Why.* The owner treats the interview as the design
step and wants the written documents to match it without a second pass. *Alternatives.* A shorter
interview; writing the roadmap from the brief alone.
