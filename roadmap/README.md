# clauderoadmapwrangler — Roadmap

Master roadmap for the Rust desktop dashboard and session launcher that keeps track of the owner's
Claude Code projects, under the decisions in [`../DECISIONS.md`](../DECISIONS.md) (D-1 … D-30 settled with
the project owner on 2026-10-31 in the v1 planning interview, EP-0; the record is
[`planning-2026-10-31.md`](planning-2026-10-31.md)).

**Planned 2026-10-31 (v1).** v1 is one window that shows every managed project's next brief with its
size and badges, launches sessions visually or headless with a chosen model and effort, routes a headless
session's questions to a decisions tile, keeps the owner's inter-session tasks and the cross-project inbox,
chains briefs on autopilot within caps, meters the account's usage, shows hosting and backup status,
history graphs and a hosted chat, ships the roadmap method as a plugin, and ends with a phone view over
the tailnet (D-1). It is **32 session briefs in five phases** (EP-1 … EP-32; 8 S · 18 M · 6 L ≈ 34
session-hours; no calendar estimate, by the owner's rule, D-4). Phase P0 (EP-0) is this plan. Everything
named and not built in v1 is parked in [`v2-notes.md`](v2-notes.md), which every re-plan grows. P1–P4 have
full brief notes; P5's briefs are charters upgraded at the P4 re-plan (EP-29).

## How to use this roadmap

Hand one EP to one session. The session writes its `EP-N-*.md` brief first, from its row and the **brief
notes** under the phase table, the decisions it cites and the current code; the brief opens with YAML
front matter (`ep`, `title`, `size`, `model`, `effort`, `status`; D-12). It does the work, reviews open
decisions with the owner (`AskUserQuestion`), appends a `> **Completion note (date; model/effort;
duration).**` to the brief, appends any owner task to `owner-tasks.md` and any message for another project
as `> **Handed on (to <project>).**`. Commit, then check the box here with one backticked hash. **The next
EP is the first unchecked row in table order** (D-9). Depends on lists immediate prerequisites only; the
order below supplies the rest. The schema these files follow, which the parser expects, is described in
the top-level [`README.md`](../README.md#the-roadmap-house-style) (moved there at EP-1).

**Sizes:** S ≈ 30 min, M ≈ 1 h, L ≈ 2 h of one Claude session with the owner supervising; anything larger
is split at pickup. L is reserved for six engines: the tile shell (EP-6), the session runner (EP-9), the
decisions tile (EP-14), the autopilot (EP-19), the chat tile (EP-24) and the web pages (EP-30).

**Standing rules for every session:** the published repository never describes the managed projects —
names, paths and states stay in the git-ignored `private/` or in the runtime registry under the user
profile; tracked files use generic or already-public names (D-3). Managed projects are read-only to this
repository's sessions except the files the tool owns (D-11). Tests never run the real `claude` CLI and
never spend usage (D-29). The egui family and the protocol crate move only in re-plan briefs (D-15).
Sessions for this project run with cwd = this folder, and from EP-16 they are launched from the tool
(D-5, D-16). Commits are the owner's call, written through a scratch file, no AI attribution trailers.

## Phase P0 — Planning (done 2026-10-31)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-0 | [The v1 planning interview: portfolio survey, research pass, domain map, 18 rounds and 72 questions; D-1 … D-30; phases P1 … P5; v2 notes; CLAUDE.md](EP-0-planning-interview.md) | L | — | core | ☑ `c767a00` |

## Phase P1 — Foundations and the read-only dashboard (planned 2026-10-31)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | [Toolchain bootstrap: Cargo workspace (`core`, `app`, `cli`), pins, cargo gate, LICENSE, README, install script skeleton, private GitHub remote, CI](EP-1-toolchain-bootstrap.md) (D-2, D-14, D-15, D-29) | M | EP-0 | core | ☑ `64d6c6a` |
| EP-2 | [Registry and discovery: `registry.toml` with backups, DATA scan, adopt and ignore, workspace-file append, states, priority, pins, ledger schema; this project registered first](EP-2-registry-and-discovery.md) (D-5, D-6, D-7, D-8) | M | EP-1 | core | ☑ `8071475` |
| EP-3 | [Parser A: house-style phase tables, rows, sizes, Done cells of every form, depends-on, brief notes, phase headings; next EP in table order; fixtures from the survey](EP-3-parser-a.md) (D-9) | M | EP-1 | core | ☑ `ecca039` |
| EP-4 | [Parser B: briefs with front matter and completion notes, decision logs, format profiles for the five-column and packet styles, HEAD-plus-status snapshots, in-progress derivation](EP-4-parser-b.md) (D-9, D-12) | M | EP-3 | core | ☑ `9603196` |
| EP-5 | [Owner tasks and inbox files: `owner-tasks.md` and `inbox.md` read and tick by byte range, `Handed on (to …)` extraction, the seven legacy forms read-only](EP-5-owner-tasks-and-inbox.md) (D-10, D-11, D-13) | M | EP-4 | core | ☑ `7bbf169` |
| EP-6 | [Dashboard shell: egui_tiles layout, dense project list, detail tile, groups and sort keys, filters, density, theme, layout persistence, 4K typography, keys, screenshot feature flag with fake projects](EP-6-dashboard-shell.md) (D-25) | L | EP-2, EP-4 | core | ☑ `1b5bfca` |
| EP-7 | Badges and status: stack chips, progress, activity from git, hosting from `gh` with head comparison and orphan detection, backup age from the Documents job, health (D-24, D-25) | M | EP-6 | core | ☐ |
| EP-8 | Re-plan P1: retro and timings, addenda, README with self-screenshots, the remote flips public, P2 confirmed with pickup notes (D-2) | S | EP-1, EP-2, EP-3, EP-4, EP-5, EP-6, EP-7 | core | ☐ |

Ordering rationale: the dashboard is useful read-only from EP-6 and shows this project's own next EP
(D-5); the parser (EP-3, EP-4) is the risk and comes before any UI; badges need git and `gh` plumbing that
EP-9 reuses.

Standing decisions for phase P1: D-3 (nothing in tracked files names a private project; fixtures use
synthetic names), D-9 (snapshot reads; next EP = first unchecked in table order), D-11 (the tool writes
only its own files inside a managed project).

**Brief notes.**

- **EP-1.** Workspace with `crates/core`, `crates/app`, `crates/cli`; edition 2024; Rust 1.98;
  `[workspace.dependencies]` pinned exactly (eframe 0.37.0 with `persistence`, egui_tiles 0.18.0,
  egui_extras 0.37.0, tokio 1.54.1, process-wrap 10.0.1, claude-codes 2.1.316, pulldown-cmark 0.13.6,
  notify 8.2.1, rusqlite 0.41.1 `bundled`, toml 1.1.8, serde, serde_json, axum 0.8.9, askama 0.16.1, chrono,
  anyhow, thiserror, tracing); `cargo test`, `cargo clippy --all-targets --features screenshot`, `cargo fmt
  --check` green on an empty skeleton; `LICENSE` (MIT); `README.md` with the one-paragraph description,
  the house-style section the parser expects (moved from this file's "How to use"), the AI-assistance and
  "drives your own CLI and login" notes; `scripts/install-shortcut.ps1` skeleton (release build + desktop
  shortcut + Start-menu entry with an AppUserModelID); `.github/workflows/ci.yml` on windows-latest running
  the gate; the short alias decided and recorded as a D-2 addendum; `gh repo create
  willtfarrington/clauderoadmapwrangler --private --source . --push` after the owner's yes
  (`AskUserQuestion`); `cli doctor` prints versions of `claude`, `gh`, `tailscale`, `code` and the
  extension. Minimum Claude Code version recorded in `core` as 2.1.261 until a brief needs more (D-15).
- **EP-2.** `core::registry`: `registry.toml` at `%APPDATA%\clauderoadmapwrangler\registry.toml` with
  `format = 1`, `[[project]]` tables (`name`, `path`, `state`, `priority`, `pinned`, `launch` defaults,
  `autopilot` rules placeholder, `ignore` list), a dated backup `registry.<date>.toml` before any migration
  (pattern of the owner's other app), atomic writes; `core::discovery`: list top-level folders of the DATA
  root (the root is a setting; the default is the parent of this repository), classify git repo / non-git,
  diff against the registry → unadopted entries; Adopt appends `{ "path": "<name>" }` to the
  `.code-workspace` file (setting: path; parse-preserve order and the `name` keys that exist; no
  reformatting beyond the appended entry; backup before write); Ignore remembered; state transitions
  proposed (every row checked → done or maintenance; a `PICKUP GATE` → paused; no commit for 60 days →
  paused) and confirmed by the owner; `core::ledger`: SQLite at
  `%LOCALAPPDATA%\clauderoadmapwrangler\ledger.sqlite` with `schema_version`, tables `session`,
  `usage_sample`, `snapshot`, `event`; `cli scan` prints the registry view. This repository is the first
  registered project (D-5).
- **EP-3.** `core::roadmap::table`: find phase tables by their header row (`# | Brief | Size | Depends on |
  Core | Done`, the five-column variant, `# | Packet | Size | Depends on | Status`), never by the `| #`
  prefix; rows are single lines; cells: id (`EP-N`, `EP-Na`), Brief (link optional, D-refs inside or after
  the link, title text), Size (`S`/`M`/`L`, re-scopes such as `S → L (…)` kept as text with the final
  size), Depends on (free text; ids extracted, ranges with `…` or `–` expanded), Core/Stretch, Done
  (`☑ \`hash\``, bare `☑ hash`, several hashes joined by ` + `, `◐`, `☐`, `[x] hash`, `[~]`, `[ ]`);
  phase headings `## Phase P0 —`, `## Phase 1 —`, `## R0 —`, `### M0 — … · [x] hash` with status in
  parentheses; `**Brief notes.**` and variants followed by `- **EP-n.** …` bullets (the detail per EP),
  prose `**Scope sketch …** EP-n: …` segments for charters; next EP = first unchecked row in document order;
  counts done/total/stretch; fixtures: one synthetic roadmap per surveyed variant in `crates/core/fixtures/`
  (names like `alpha`, `beta`), including the 867-character row, a two-digit phase, a phase without a
  table, and a glossary table that starts with `| #`.
- **EP-4.** `core::roadmap::brief`: `EP-N-<slug>.md` discovery (ignore `_TEMPLATE.md`,
  `EP-9-completion-*.md` shapes), YAML front matter (D-12) when present, the `**Size:** … · **Depends on:**
  … · **Blocks:** …` header line, completion notes (`> **Completion note (date).**`, the new form with
  model/effort/duration, suffix variants, two notes per brief, `## Completion note`, `> **☑ COMPLETE**`,
  `**Status:** DONE`), `> **Handed on.**` and `> **Parked.**` blocks, `> **EP-n pickup note**` and `>
  **Charter.**` blocks; `core::decisions`: `DECISIONS.md` at the root or nested, `**D-n Title.**` count,
  addenda count and last date, the domain-id and ADR-folder variants counted only; **format profiles**
  (`house`, `house5`, `packet`): the packet profile reads `[x] hash` cells, `## Handoff payload (filled
  date)` and `docs/handoffs/EP-N.md` `**Status:** … · **Date:**` lines; **snapshot**: parse `git show
  HEAD:<path>` and the working tree, keep both, mark a project "session in progress" when a brief exists
  without a completion note, a brief is untracked, or the roadmap is dirty; unit tests per profile.
  **From EP-2:** fill `proposal::Facts` from the parse — `all_rows_checked` from EP-3's counts and
  `pickup_gate` when the next brief carries a `PICKUP GATE` block — so `crw scan` proposes done or
  maintenance and paused from them as it already does for 60 days without a commit.
  **From EP-3:** `roadmap::table::parse` takes text, so the snapshot parses `git show
  HEAD:roadmap/README.md` and the working tree each; `Counts::all_checked` is the `all_rows_checked`
  fact; a linked row's `brief.link` names its brief file; `roadmap::markdown` drops blockquotes today, so
  completion notes need a blockquote block there; `crw roadmap` can print both readings.
- **EP-5.** `core::tasks`: `roadmap/owner-tasks.md` lines `- [ ] OT-n (EP-k, date) text [before EP-m]`,
  tick written back by byte range through pulldown-cmark's offset iterator with an atomic write and a
  `.bak` copy; `roadmap/inbox.md` lines `- [ ] IN-n (from <project> EP-k, date) text [urgent]` with the
  disposition appended on the line when ticked; `> **Handed on (to <project>).**` blocks extracted from
  completion notes with the target resolved against the registry (unknown target → shown, not delivered);
  legacy forms read-only: `## Risks / open items` numbered lists with `~~…~~` resolution, `> **Parked.**`,
  `> **Handed on.**`, `owner-decisions.md` (`OD-n` rows, `☐ **open — owner only**`), `open-questions.md`
  (`OQ-x` rows), "Open items for the owner" lists, the "Owner actions outside any brief" paragraph split on
  semicolons with its "before EP-n" gates, handoff "Risks / notes" sections; a unified `OwnerItem` with
  source, text, gate and whether it is tickable. **From EP-3:** `roadmap::markdown` already gives each
  line its number and byte offset and each list item its continuation lines, which may make
  pulldown-cmark unnecessary for the tick write-back; `roadmap::ep_ids` reads a `before EP-m` gate.
- **EP-6.** `crates/app`: eframe with `persistence`; egui_tiles tree — list (left, wide), detail (centre),
  decisions / sessions / chat / usage tiles as placeholders that later EPs fill; the list: one row per
  project (name, state chip, next EP id and one-line summary with size, badges from EP-7 as empty slots,
  session status, autopilot toggle disabled until EP-19, Start disabled until EP-10), collapsible groups by
  state, pinned first, sort menu (priority, completion, remaining, last activity, hosting, usage share,
  name), filters (stack, hosting), inline expansion placeholder; detail: the brief-notes bullet of the next
  EP, open owner items, inbox, counts; density toggle; theme System / Light / Dark; layout and window state
  saved in `%APPDATA%` with Reset layout; typography and spacing for a 4K-class display with an icon font
  (egui_material_icons 0.9); keys (arrows, Enter, S, A); status bar for notices; a `screenshot` feature:
  with `CRW_SCREENSHOT=<file.png>` the app loads a fixture registry of fake projects, saves its own frame
  and exits (the pattern of the owner's other app; never a screen capture); `CRW_SCREENSHOT_SETUP` for
  injected keys. **From EP-2:** the unadopted group from `discovery::view` with Adopt and Ignore
  (`discovery::adopt`, `discovery::ignore`), proposal chips with Confirm (`proposal::propose`, then
  `set_state`), the facts gathered off the UI thread (git one child at a time: about 5 s for 14
  repositories), and a `notify` watcher on the data root's top level that rescans when a folder
  appears or goes (D-7); the fake registry through `registry::Store::new` or `CRW_HOME`, never the
  owner's (rule 13). **From EP-3:** the tiles read `Roadmap::next`, `note`, `phase_of` and
  `Phase::counts`; `brief.title` is the row's one-liner and `Note::text` the bullet verbatim;
  `Roadmap::problems` go in the detail tile. **From EP-5:** the detail tile's owner items are
  `Snapshot::owner_items()` (`is_open()`, `is_tickable()`, `State::Partly` for an item with a clause
  struck through, `Gate` for `before EP-m`, `on or after` a date or a decision point in words), the
  inbox is `snapshot.inbox` and the outgoing mail `inbox::outgoing(&snapshot.briefs, …)` resolved
  through `Registry::project`; ticks go through `tasks::tick` / `untick` and `inbox::tick` (a
  disposition required) with the store's backups folder, and the copy's path belongs in the status
  bar; a project's checklist problems (`Tasks::problems`, `Inbox::problems`) go in the detail tile.
- **EP-7.** `core::badges`: stack chips from manifests (`Cargo.toml` → Rust and the GUI family,
  `pyproject.toml` → Python and the version pin, `package.json` → TypeScript/Node and the manager,
  `docker-compose.yml`, `*.ps1` → PowerShell, `index.html` → static site); progress from the parse (done,
  total, stretch, percent); activity from `git status --porcelain=v2 --branch` (dirty count, ahead, behind)
  and `git log -1 --format=%ct`; hosting from `git remote -v` plus `gh repo view <owner>/<name> --json
  visibility,isArchived,pushedAt,defaultBranchRef` and `git ls-remote origin HEAD` compared to local
  HEAD; orphan detection from `gh repo list --json name,visibility` against local-only projects; health
  from `gh run list --limit 1 --json conclusion` for repositories with workflows and a `LICENSE*` file
  check; backup age from the Documents job's settings file and run records (path settings; the job writes
  one archive per unit — read the newest archive's timestamp per project); refresh daily and on demand;
  results cached in the ledger with their fetch time. **From EP-6:** a row keeps the badges' place
  between the counts and the session status (`list::Columns`); the hosting sort key and the stack and
  hosting filters exist, disabled (`SortKey::arrives_at`, the Filters menu); the git pass
  (`portfolio::Loader::read_git`, one repository after another under one lock) is where the activity
  and hosting children join, and `ProjectRow::facts.last_commit` already comes from it; F5 is the
  on-demand refresh; the fake portfolio has no repository, so badge tests make a scratch one as
  `portfolio`'s git test does.
- **EP-8.** Retro (planned vs measured durations per EP), addenda to D-1 … D-30, `README.md` with two
  self-screenshots of fake projects, the public flip: `gh repo edit --visibility public
  --accept-visibility-change-consequences` after the owner's yes, the P2 rows confirmed with pickup notes,
  `v2-notes.md` grown. Before the flip, a pre-flip audit (added at EP-1 with the owner's yes): `git grep`
  over every commit for the private project names kept in `private/`, and a re-read of the tracked
  planning record for personal machine details (it names the `D:` volume's label, the VPN and the
  account's plan tier); anything found goes to the owner before the flip. **From IN-1 and IN-2
  (2026-10-31, triaged at EP-4):** the owner replaced the one multi-root workspace with one
  single-folder `<name>.code-workspace` per project beside the project folders (the aggregate file
  stays for browsing and is what the registry's `workspace_file` names), and grants the other
  projects to every session through the user-level `permissions.additionalDirectories`. Addenda to
  write: D-7 (Adopt still appends to the aggregate file and should also create `<name>.code-workspace`
  from the same template; the first-run default "exactly one `.code-workspace` in the data root" no
  longer holds, so a fresh registry needs another rule for `workspace_file`; `crw scan`'s workspace
  listing should read the per-project files), D-16 "no extra directories" and D-18 (Claude Code's
  documentation says a settings-file grant gives file access only, while a folder passed at launch
  with `--add-dir` also loads its `CLAUDE.md` and `.claude/rules` when
  `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1` is set, so the registry should hold each project's
  neighbour list and the headless launcher pass it as `--add-dir` with that variable). The re-plan
  decides whether Adopt's per-project file, the scan listing and the neighbour list are an S row in
  P2 or done at the re-plan, and writes the pickup notes for EP-9 and EP-10 accordingly. **From
  EP-6:** the README's pictures come from `CRW_SCREENSHOT=<file.png>` with `--features screenshot`
  over the fake portfolio (`CRW_SCREENSHOT_SETUP`: `light` or `dark`, `select=`, `expand=`,
  `size=<w>x<h>`; the default window is 1600 × 1000 points, a 2400 × 1500 picture at 150 %).

## Phase P2 — Sessions (planned 2026-10-31)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-9 | Session runner core: CLI discovery and preflight, minimum version, `-p` stream-json child with job-object kill, `claude-codes` adapter, capability detection, pre-assigned session id, ledger rows, stub `claude` for tests (D-14, D-15, D-16) | L | EP-2 | core | ☐ |
| EP-10 | Prompt generator and launch dialog: the house-style opener, per-project template, model and effort picker from the catalog cache, rule table resolver, permission mode, Headless launch (D-16, D-17, D-18, D-22) | M | EP-5, EP-9 | core | ☐ |
| EP-11 | Visual launch: the extension URI with the prompt pre-filled, per-project window or workspace file when the running window's first folder differs (D-16) | S | EP-10 | core | ☐ |
| EP-12 | Loopback server and signals: axum on 127.0.0.1, `cli hook` and `cli statusline` relays, installation of user-level hooks and the status-line entry, `claude agents --json` polling, session state model (D-20, D-23) | M | EP-9 | core | ☐ |
| EP-13 | Sessions tile: live transcript from stream-json, stop, resume by id, ledger rows with outcome and cost estimate (D-16, D-26) | M | EP-12 | core | ☐ |
| EP-14 | Decisions tile: control-protocol answers to `AskUserQuestion` and permission requests, optional timeout, defer-and-resume fallback, Focus VS Code for visual sessions, answer latency (D-19) | L | EP-13 | core | ☐ |
| EP-15 | Notifications: WinRT toasts, tray icon with badge and menu, event rules, quiet hours (D-21) | M | EP-12 | core | ☐ |
| EP-16 | Re-plan P2 and the dogfooding switch: retro, addenda, the CLAUDE.md rule that this project's EPs are launched from the tool, first self-launched session verified (D-5) | S | EP-9, EP-10, EP-11, EP-12, EP-13, EP-14, EP-15 | core | ☐ |

Ordering rationale: the runner (EP-9) is the second engine and the base of everything unattended; the
prompt and dialog make Headless useful at once; Visual is small once the dialog exists; the loopback
server (EP-12) is needed by the sessions tile, the decisions tile, the notifications and later the usage
feed; the dogfooding switch closes the phase.

Standing decisions for phase P2: D-15 (never `--bare`; `ANTHROPIC_API_KEY` removed from the child
environment; features detected, not assumed), D-18 (acceptEdits with prompts routed to the host), D-19
(the protocol first, defer as fallback), D-3 (recorded streams for tests use fake project names and
contain no real transcript text).

**Brief notes.**

- **EP-9.** `core::session`: locate `%USERPROFILE%\.local\bin\claude.exe` (setting overrides), run
  `claude --version` and `claude auth status` (exit code and JSON `loggedIn`, `subscriptionType`) as a
  preflight cached for an hour; refuse below the minimum version; spawn `claude -p --verbose
  --output-format stream-json --input-format stream-json --include-partial-messages
  --replay-user-messages --session-id <uuid> --name "<project> EP-n" --model <id> --effort <level>
  --permission-mode <mode> --permission-prompts host [--effort ultracode] [--max-budget-usd <n> when
  advertised]` in the project folder with `ANTHROPIC_API_KEY` removed, wrapped in a process-wrap job object
  (kill on close); parse lines through `claude-codes` behind `core::session::protocol` (an adapter that
  exposes only the events the tool uses: init with capabilities and version, assistant text and tool-use
  summaries, control requests, result); graceful stop = close stdin, wait up to 30 s, then kill the job;
  exit code and the result JSON parsed even on a non-zero exit; a `session` ledger row (project, EP, mode,
  model, effort, ultracode, start, end, outcome, session id, cost estimate); `crates/core/tests/stub-claude`:
  a small binary that replays a recorded stream-json file (with fake names) and honours stdin
  control responses, used by all runner tests; spacing of spawns ≥ 2 s (D-29).
- **EP-10.** `core::prompt`: the opener assembled from the parse — project name and path; the read-first
  order taken from the project's `CLAUDE.md` when it has one (else the house default); the next row (id,
  title, size, depends-on, core) and its brief-notes bullet verbatim; open inbox messages and owner tasks
  with the triage rule (D-13); the rules: write `roadmap/EP-N-<slug>.md` first with front matter, follow
  the project's `CLAUDE.md`, end with `> **Completion note (date; model/effort; duration).**`, append owner
  tasks and `Handed on (to …)` blocks, **stop before committing** (the autopilot variant replaces the last
  rule in EP-19); per-project template override in the registry with placeholders; the launch dialog in
  `app`: prompt editor, model picker from `~/.claude/cache/model-catalog/*.json` (fallback: a built-in
  list), effort picker (low … max, ultracode toggle), permission mode (default from the project's
  registry entry), the rule table's resolution shown with a reason; `core::rules`: `[[rule]]` entries in
  `registry.toml` with `match` (size, phase, title regex) and `set` (model, effort, ultracode), global then
  per-project, first match wins; Headless launch button → EP-9. **From EP-5:** the open owner tasks
  and inbox messages are `snapshot.tasks` and `snapshot.inbox` (`open()`), each with its id, the
  parenthesis's fields and its text as written; the older forms' items (`snapshot.legacy`) are the
  project session's own reading and need not be in the prompt. **From EP-6:** the S key and each
  row's Start button exist and answer `Action::NotYet`; the dialog takes their place, and the
  selected project is `Board::view.selected`.
- **EP-11.** Visual launch: build `vscode://anthropic.claude-code/open?prompt=<url-encoded opener>`;
  read `~/.claude/ide/*.lock` to learn the running window's `workspaceFolders`; if the first folder is the
  project, run `Code.exe --open-url "<uri>"`; otherwise open `Code.exe <project folder>` (or a generated
  `<project>.code-workspace` in `%APPDATA%\clauderoadmapwrangler\workspaces\` when the project wants extra
  folders), wait for its lock file, then fire the URI; the session is recorded as `visual` with no process
  handle; a note in the dialog that the prompt is pre-filled and must be sent by hand. **From IN-1
  (2026-10-31, triaged at EP-4):** open the owner's existing `<data root>\<name>.code-workspace` beside
  the project folder instead of generating one under `%APPDATA%` (the D-16 addendum written at EP-8);
  generate one only when no such file exists.
- **EP-12.** `app::server`: axum on `127.0.0.1:<port>` (port in the registry, default 47831), routes
  `POST /hook`, `POST /statusline`, `GET /health`; `cli hook` reads the hook JSON from stdin and posts it
  with a 500 ms timeout, exiting 0 silently on any error; `cli statusline` does the same for the
  status-line JSON and prints nothing; installation (owner's yes through `AskUserQuestion` at first run)
  writes into `~/.claude/settings.json` a `hooks` block for Notification, Stop, SessionStart and SessionEnd
  running `"<exe path>" hook` and a `statusLine` block running `"<exe path>" statusline` with
  `refreshInterval` 30, after a dated backup of the settings file, and only if no other status line is
  configured (otherwise the tool offers to chain it); Diagnostics (EP-28) later checks that the paths match
  the current exe; `core::sessions::state`: a model fed by hooks, the runner and a `claude agents --json`
  poll every 5 s (`pid`, `cwd`, `kind`, `status`, `state`, `waitingFor`, `name`), reconciling cwd to a
  project; statuses busy / waiting / idle / done / failed; mode visual / headless / manual.
  **From IN-2 (2026-10-31, triaged at EP-4):** the owner runs a stopgap user-level `SessionStart`
  hook (`~/.claude/hooks/session-start-inbox.ps1`) that prints the project's open inbox lines into
  the session; the installation replaces it (removed or chained, after the owner's yes). A
  `UserPromptSubmit` hook that recognises `@<project>` in a prompt and injects that project's rules
  and next-EP row gives on-demand cross-project context (the owner's wish; here, or at EP-20 if it
  needs the mailman's registry lookups).
- **EP-13.** The sessions tile: one tab per live or recent session; transcript rendering from the stream
  (assistant text, tool calls as one-line summaries, tool results collapsed, partial messages appended);
  Stop (graceful) and Kill; Resume by session id for a headless session that ended with a deferred tool use
  or a stop; the ledger row completed with outcome (completed, stopped, failed, deferred), end time,
  result text, `total_cost_usd` estimate and the status-line `cost`/`context_window` figures received for
  that session id; a session list with filters.
- **EP-14.** The decisions tile: `control_request` → `can_use_tool` handling in the adapter; for
  `AskUserQuestion` the questions are rendered with their options (single or multi-select, free text),
  the answer sent as `updatedInput` with `answers`; for permission requests Allow / Deny with the tool name
  and input shown; an optional per-project timeout after which the tool defers the run (sends deny with a
  reason and marks the session `deferred`); the **fallback**: a PreToolUse hook passed with `--settings`
  that returns `permissionDecision: "defer"` for `AskUserQuestion`, the run exiting with `stop_reason:
  tool_deferred`, the tile showing `deferred_tool_use`, and Resume re-launching with `--resume <id>` and a
  one-shot hook that returns `allow` with the answer; the fallback is selected automatically when the
  protocol handshake fails or by a setting; visual sessions in `waiting` show "waiting for you in VS Code"
  with Focus (bring the VS Code window to front); answer latency recorded.
- **EP-15.** Toasts through notify-rust (WinRT) with the AppUserModelID of the Start-menu entry from
  `install-shortcut.ps1` (text: project, EP, event, one line of detail; click focuses the window); the tray
  icon (tray-icon 0.26) with a badge count of sessions waiting, a menu (Show, Start next for each pinned
  project, Pause autopilot, Enable remote view when EP-31 exists, Quit) and the close-to-tray behaviour;
  event rules per D-21 (toast: waiting, finished, failed, usage 80 % and 95 %; badge: unadopted, orphan or
  mismatched remote, stale backup, inbox arrival); quiet hours in the registry; a notification log in the
  ledger.
- **EP-16.** Retro, addenda, `CLAUDE.md` rule 11 switched to "launched from the tool", the first
  self-launched session (EP-17's) started from the dashboard with the generated opener and verified in the
  ledger, P3 rows confirmed with pickup notes, `v2-notes.md` grown.

## Phase P3 — Automation, usage, inbox, method (planned 2026-10-31)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-17 | Usage meter: endpoint poller with per-poll token read, status-line feed merge, degrade mode, strip and tile with countdowns and surface breakdown, samples, 80 and 95 percent toasts (D-23) | M | EP-12 | core | ☐ |
| EP-18 | Usage coupling: Fable-share downshift rule, before-and-after delta attribution per session, usage-share badge (D-23) | S | EP-10, EP-17 | core | ☐ |
| EP-19 | Autopilot: per-project toggle, the gate, the commit variant of the prompt, Commit button resume, per-run and per-day caps, pause at 90 percent, global Pause, chain log (D-22) | L | EP-14, EP-17 | core | ☐ |
| EP-20 | Inbox delivery: `Handed on (to …)` parsed at completion, append to the receiver's inbox, delivery log, compose dialog, urgent flag, ntfy publisher (D-13, D-21) | M | EP-5, EP-13 | core | ☐ |
| EP-21 | Adoption and formalise files: CLAUDE.md rule block, `owner-tasks.md`, `inbox.md`, the brief template with front matter, the migration prompt for packet-style projects (D-10, D-12, D-13) | M | EP-5, EP-10 | core | ☐ |
| EP-22 | Roadmap-wrangler plugin: skills plan, replan, formalise, status and the house-style reference; loaded with `--plugin-dir`; install notes (D-27) | M | EP-21 | core | ☐ |
| EP-23 | Re-plan P3 | S | EP-17, EP-18, EP-19, EP-20, EP-21, EP-22 | core | ☐ |

Ordering rationale: the meter precedes the autopilot because the gate reads it; the inbox mailman needs
completion-note parsing from EP-5 and session completion from EP-13; adoption files and the plugin make the
method installable before the chat (P4) delegates to it.

Standing decisions for phase P3: D-22 (the gate, the commit policy, the caps), D-23 (the endpoint is
undocumented and breakable; the token is read per poll and never stored or logged), D-13 (never interrupt
a running session).

**Brief notes.**

- **EP-17.** `core::usage`: read `~/.claude/.credentials.json` → `claudeAiOauth.accessToken` at poll
  time only (never persisted, never logged, redacted from debug output), `GET
  https://api.anthropic.com/api/oauth/usage` with `Authorization: Bearer` and `anthropic-beta:
  oauth-2025-04-20`, parse `limits[]` (`kind` weekly_all / weekly_scoped with `scope.model.display_name`,
  `percent`, `resets_at`, `is_active`), `five_hour`, `seven_day_breakdown.rows[]`, `extra_usage`; a
  `usage_sample` row per poll; poll every 5 min and after each tool-launched session ends; merge the
  status-line feed (`rate_limits.five_hour|seven_day.used_percentage|resets_at` per session) as a second
  series; when the endpoint fails (HTTP error, schema change, expired token) show the status-line figures
  with a "degraded" marker and retry with backoff; the strip in the status bar (weekly, Fable, five-hour
  when active, each a bar with the percent) and the usage tile (bars with reset countdowns, surface
  breakdown, a 7-day sparkline of samples); toasts at 80 % and 95 % of weekly or Fable (once per crossing
  per window).
- **EP-18.** The resolver (EP-10) gains the downshift rule: when the Fable-scoped percent exceeds the
  registry's `fable_downshift_percent` (default 60) and the rule's size is S, propose `claude-opus-5-5` at
  the same effort with the reason shown; the autopilot applies it; attribution: the weekly and Fable
  percents sampled before launch and after the result are stored on the session row, the delta shown per
  session and summed per project for a usage-share badge (this week) in the list and the detail tile;
  sessions the owner starts get the status-line cost estimate only.
- **EP-19.** `core::autopilot`: per-project toggle in the list and the registry; after a headless run's
  result the gate is evaluated and logged (every failed condition named): autopilot on; outcome completed
  and a completion note found in the parsed brief; no open owner task with `before EP-<next>`; next EP not
  a re-plan (title match `Re-plan`); size within `autopilot.max_size` (default M); weekly and Fable below
  `pause_percent` (default 90); `git status` clean after the session's commit; `chained_today` <
  `per_project_daily` (default 2) and `chained_today_total` < `daily_total` (default 4); the prompt variant
  replaces "stop before committing" with "commit in the house pattern (`<Title>: <summary> (EP-n, D-x)`
  then `docs(roadmap): record EP-n commit hash`) and record the hash in the roadmap"; with autopilot off the
  Commit button resumes the finished session with that instruction; per-run wall-clock caps by size (S 45
  min, M 90 min, L 3 h) stop the session gracefully; `--max-budget-usd` passed when advertised by the CLI;
  a global Pause (tray, window, and automatic at the pause threshold with resume after `resets_at`); a chain
  log tile section (what ran, why it stopped).
- **EP-20.** `core::inbox::mailman`: after a session completes (headless result, or a hook Stop/SessionEnd
  for visual sessions followed by a re-parse), scan the brief's completion note for `> **Handed on (to
  <project>).**` blocks, resolve the target in the registry, append `- [ ] IN-n (from <project> EP-k, date)
  text` to the target's `roadmap/inbox.md` (create the file with its header when absent), record a
  delivery row, show a badge on the receiver; compose dialog in the tool (target, text, urgent) writing the
  same line with `from owner`; `urgent` → toast and ntfy; `core::notify::ntfy`: POST to
  `https://<ntfy host>/<topic>` over the tailnet with title and priority (host and topic in the registry;
  disabled when empty); the inbox section of the detail tile with tick (disposition text required).
  **From EP-5:** `inbox::outgoing` already resolves each `Handed on (to …)` block against the registry
  (`Recipient::Project` or `Unknown`) and leaves out the blocks to the owner, which are owner items;
  the append to the receiver's `inbox.md` can reuse `checklist`'s copy-then-atomic-write, the next
  free id is the highest `IN-n` of `inbox::parse` plus one, and the tick with a disposition exists
  (`inbox::tick`, `crw tick <project> IN-n <disposition>`).
- **EP-21.** `core::adopt`: for a project being formalised or adopted into the house style, write (after
  a preview and the owner's yes) a `CLAUDE.md` rule block between markers `<!-- roadmap-wrangler:begin -->`
  / `<!-- roadmap-wrangler:end -->` (read-first order, owner-tasks, inbox and triage rules, the completion
  note form, the front-matter rule), `roadmap/owner-tasks.md` and `roadmap/inbox.md` with headers, and
  `roadmap/_TEMPLATE.md` (front matter, Context, In scope, Out of scope, Verification / acceptance,
  Completion note placeholder); for a `packet` or `house5` profile project, generate the migration prompt
  (convert tables to the six-column header, keep ids and hashes, move handoff status into completion
  notes, keep every file) for a headless session the owner launches from the chat or the dialog; never
  overwrite an existing file outside the markers. **From IN-2 (2026-10-31, triaged at EP-4):** every
  active project already has `roadmap/inbox.md` in the D-13 form (hand-carried by the owner), so the
  formalise step creates the file only where it is missing and adds the triage line to rule 1; the
  owner's local-only environment repository under the data root (environment decisions and the shared
  memory) is adopted by the owner with `crw adopt` (rule 13), never by the tool on its own.
  **From EP-5:** in the files the formalise step writes, every line of work stays a top-level list
  item `- [ ] OT-n (…) text` or `- [ ] IN-n (…) text` below a header paragraph: `checklist::parse`
  reports any other box as a problem, and the header's example line belongs in backticks, as the
  inbox files already write it.
- **EP-22.** `plugins/roadmap-wrangler/` with `.claude-plugin/plugin.json`, `skills/plan/SKILL.md` (the
  interview method as practised in this project's EP-0: research pass, domain map, rounds of four with the
  recommended option first, the planning file written to the repo first and filled per round, close-out
  with unpicked items), `skills/replan/SKILL.md` (retro, addenda, re-cut, pickup notes, v2 notes),
  `skills/formalise/SKILL.md` (the house-style bootstrap from EP-21's files and a first roadmap from a
  brief), `skills/status/SKILL.md` (read a project's roadmap the way the parser does and report next EP,
  counts, open items), and `reference/house-style.md` (the schema the parser expects, the forms of D-12 and
  D-13); tool-launched sessions get `--plugin-dir <repo>/plugins/roadmap-wrangler`; README notes for
  installing it into the owner's own sessions; `cli` verifies the plugin loads (`claude --plugin-dir …
  --help` smoke in CI is out; a structural test checks the files).
- **EP-23.** Retro, addenda, P4 rows confirmed with pickup notes, `v2-notes.md` grown; the first week of
  autopilot runs reviewed (how many chains, how many gate refusals and why).

## Phase P4 — Chat, analytics, exports, diagnostics (planned 2026-10-31)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-24 | Chat tile: hosted read-only portfolio session in DATA, digest system prompt, resumable conversations, delegation of adopt, formalise and compose actions, Opus default (D-27) | L | EP-14, EP-22 | core | ☐ |
| EP-25 | History backfill and snapshots: git import per project with Re-import, completion-note cross-check, daily parse snapshots (D-26) | M | EP-2, EP-4 | core | ☐ |
| EP-26 | Analytics: activity heatmap, burn-up per project, velocity and projection, inline row expansion, Expand all and Collapse all (D-26) | M | EP-25 | core | ☐ |
| EP-27 | Exports: site `projects.json` and HTML snippet, daily JSON export into `private/`, CSV per project (D-8, D-24) | S | EP-25 | core | ☐ |
| EP-28 | Operations: `doctor` and the Diagnostics tile, rolling log, install script final, start-at-login entry, single instance, AV hints, start-up reconciliation (D-29) | M | EP-15 | core | ☐ |
| EP-29 | Re-plan P4; P5 charters upgraded to full briefs | S | EP-24, EP-25, EP-26, EP-27, EP-28 | core | ☐ |

Ordering rationale: the chat needs the decisions tile (its questions) and the plugin (its method); the
backfill feeds the graphs and the exports; operations polish lands before the phone view exposes the tool
beyond the desk.

Standing decisions for phase P4: D-27 (read-only chat session; writes delegated), D-26 (no transcript
parsing), D-3 (exports into `private/`; the site export names public projects only).

**Brief notes.**

- **EP-24.** `core::chat`: a session per conversation spawned like EP-9 but with cwd = the DATA root,
  `--tools "Read,Glob,Grep,Bash"` narrowed by `--allowedTools` to read-only shell commands plus the tool's
  own `cli` (`cli scan`, `cli inbox send`, `cli tasks add`, `cli adopt`), `--permission-mode dontAsk`,
  `--append-system-prompt-file` with the generated digest (every project: state, next EP and size, open
  tasks and inbox counts, session state, usage figures, the house-style reference), `--resume` for
  continued conversations, Opus 5.5 at medium by default with a picker; the chat tile: conversation list,
  streaming replies, an input box, action chips when the session proposes an action (Adopt X, Formalise
  Y → opens the EP-21 preview or the launch dialog with a generated prompt), history in the ledger.
- **EP-25.** `core::history`: `git log --format=%H%x09%ct%x09%s` per project; EP ids from `record EP-n
  commit hash`, `(EP-n` or `(EP-n,` suffixes, `EP-n:` prefixes; completion dates per EP = the record
  commit's date, else the EP commit's date, cross-checked with the brief's completion-note date (the note
  wins when both exist and differ by more than a day, with a flag); hupsim-style single-commit histories
  use note dates only; `snapshot` rows (project, date, done, total) written daily and at every parse that
  changes the counts; Re-import per project from the detail tile.
- **EP-26.** `app::analytics` with egui_plot 0.38: a calendar heatmap (sessions and completions per day,
  per project and for the portfolio), a burn-up per project (done and total over time; re-plans visible as
  total jumps), velocity (EPs per week over the last 8 weeks) and projection (remaining session-hours from
  the size mix × measured median duration per size; projected finish at the measured weekly pace, shown as
  a range) in the detail tile and as the inline row expansion in the list; Expand all / Collapse all; the
  model and effort per EP from the new completion-note form shown in the history list.
- **EP-27.** `core::export`: `projects.json` (name, badge text, done, total, percent, next one or two
  undone rows, `as_of`) and `projects-cards.html` for projects whose remote is public, written into the site
  repository's `assets/data/` after the owner's yes (the site's own session wires it in; a `Handed on (to
  <site>)` message is created for that); the daily JSON export of registry and ledger into this repository's
  `private/export/<date>/`; CSV per project (sessions, snapshots) on demand.
- **EP-28.** `cli doctor` and the Diagnostics tile with the checks of D-29 (Claude Code version vs
  minimum, auth status, extension version from the extensions folder, hooks and status-line entries present
  and pointing at the current exe, Tailscale state via `tailscale status --json`, `D:` mounted with the
  expected label, `gh auth status`, registry and ledger integrity, last poll times); a rolling log under
  `%LOCALAPPDATA%\clauderoadmapwrangler\log\` with daily rotation and 14 days kept; `install-shortcut.ps1`
  final (release build, shortcut, Start-menu entry with AppUserModelID, start-at-login entry registered
  through the tool's own `--register-autostart` and removable); single instance by probing `/health` and
  focusing the running window; AV hints when a spawn fails or the exe path changed; start-up reconciliation
  of session rows against `claude agents --json`. **From IN-3 (2026-11-01, triaged at EP-6):** the
  script also puts `crw` on PATH with a shim in `%USERPROFILE%\.local\bin` beside `claude.exe` (that
  folder is on PATH already); its form is the brief's choice (a `crw.cmd` that forwards its arguments
  to `target\release\crw.exe`, or a copy of the exe refreshed at each run), Diagnostics checks that
  the shim reaches the current exe, and the hook and status-line entries keep the exe's full path.
- **EP-29.** Retro, addenda, P5 charters upgraded to full briefs with pickup notes (the web pages' exact
  routes and templates decided from a month of desktop use), `v2-notes.md` grown.

## Phase P5 — Phone view (charter briefs; upgraded at EP-29)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-30 | Web pages: askama templates with SSE-driven updates — project list, project page with tasks and inbox tick, decisions answer form, start and pause actions; identity middleware and action log (D-28) | L | EP-12, EP-14, EP-19 | core | ☐ |
| EP-31 | Session page and remote enable: streaming transcript over SSE with an input for answers, Enable remote view action running `tailscale serve`, serve status, QR code, disable (D-28) | M | EP-30 | core | ☐ |
| EP-32 | Re-plan P5 and v1 close-out: timings, README "State of the app", addenda, `v2-notes.md` compiled, v2 direction | S | EP-30, EP-31 | core | ☐ |

**Charters.** Each is upgraded to a full brief by EP-29 or at pickup.

- **EP-30.** The axum server from EP-12 gains routes under `/m/`: list (state groups, next EP, badges,
  session status), project (brief notes, owner tasks with tick, inbox with tick and compose), decisions
  (pending questions with option buttons and free text, permission allow/deny), actions (start the next
  brief as the rules resolve it, stop a headless session, pause/resume autopilot); askama templates with a
  small Datastar (or htmx 2.0.x) layer for SSE-driven updates; middleware that requires
  `Tailscale-User-Login == owner` for every `/m/` request (owner login in the registry), returns 403
  otherwise, and treats loopback callers as local; every remote action appended to an `event` row with
  identity, path and time; mobile layout at phone width.
- **EP-31.** `/m/session/<id>`: the transcript streamed over SSE with an input box for answers and
  messages to a headless session; the Enable remote view action: `tailscale serve --bg --https=9443
  http://127.0.0.1:<port>` after the owner's confirmation, `tailscale serve status --json` parsed into
  the Diagnostics tile, the URL and a QR code shown, a Disable action (`tailscale serve --https=9443 off`);
  a check that NordVPN's state does not block the tailnet (hint only).
- **EP-32.** Timings for every EP of v1, the README's "State of the app" and screenshots refreshed,
  addenda, `v2-notes.md` compiled into themes (launch modes, analytics, chat, phone, hosting, conventions),
  the v2 direction (the first candidates: `--bg` background sessions with Attach, worktree-per-session,
  usage over time, free chat from the phone).

## Risks / open items

1. **The usage endpoint is undocumented** (D-23). A schema change or an auth change blanks the meter;
   the status-line feed is the documented fallback and the meter must say when it is degraded.
2. **The stream-json control protocol is unspecified for non-SDK hosts** (D-19). `claude-codes` tracks it
   release by release; the defer-and-resume fallback is documented; EP-9 feature-detects from
   `system/init`. Every CLI update is a potential break — the Diagnostics tile shows the version against
   the recorded minimum and the last successful handshake.
3. **Two CLI versions on the machine** (standalone 2.1.261, extension-bundled 2.1.314). Visual and
   headless sessions may behave differently; the minimum version applies to the standalone binary.
4. **Hooks and status line in the owner's user settings** (D-20, D-23). Installed only after the owner's
   yes, with a dated backup; the relay must never slow a session (500 ms timeout, silent exit). An
   existing status line would be chained, not replaced.
5. **`--max-budget-usd` is an estimate and `--max-turns` may be absent** (D-22). Wall-clock caps are the
   reliable limit; the budget flag is passed only when advertised.
6. **Concurrent sessions edit roadmaps live** (D-9). The parser snapshots HEAD plus status; a project with
   a dirty roadmap is shown as "session in progress" rather than re-counted every few seconds.
7. **Visual launch cannot submit the prompt** (D-16). The owner presses Enter; the ledger records the
   session as visual without a process handle; completion is seen through hooks.
8. **Remote Control stores transcripts on Anthropic's servers while connected** (documented). It is the
   owner's existing posture (`remoteControlAtStartup: true`) and not changed by this tool; noted for the
   wholly-private projects' own decisions.
9. **Worktree isolation of `--bg` sessions** would break roadmap edits in the main tree; `--bg` is parked
   (v2 notes) and would need `worktree.bgIsolation = none` per project.
10. **Antivirus** may flag the child-process spawner or the stub `claude` used in tests (D-29): spaced
    spawns, no burst loops, and the quarantine hint in Diagnostics.

## Decision record

Decisions live in [`../DECISIONS.md`](../DECISIONS.md) (D-1 … D-30); the planning record with the
verified facts, the research pass and the interview is [`planning-2026-10-31.md`](planning-2026-10-31.md);
the portfolio survey that names the owner's projects is in the git-ignored `../private/`.
