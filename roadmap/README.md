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

**Re-planned 2026-11-01 (EP-8, the P1 re-plan).** P1 took 6 h 17 min of session against 9 h planned
(about 70 %; an M about 45 minutes), and the sizes stay the planning unit (D-4 addendum). EP-7a joined
P1 at the owner's request (IN-4); this re-plan added EP-33 (per-project workspace files, in P2 before
EP-11) and EP-34 (D-24's hosting and backup actions, in P4 before EP-28), folded D-25's cards view into
EP-26 and gave P2 its pickup notes. v1 is now **35 briefs, 9 S · 20 M · 6 L ≈ 36.5 session-hours**;
P2–P5 hold 27 of them, about 19 at P1's pace. The repository is public from EP-8 (D-2, D-3 addenda).

**Re-planned 2026-11-04 (EP-16, the P2 re-plan).** P2 took 15 h 28 min of session against 13 h 30
min planned (115 %), 10 h 27 min of it active (77 %), the rest the owner's answers and trials of the
real window; an M about 52 minutes of active work and each L about an hour, and the sizes stay the
planning unit (D-4 addendum). Five rows joined P2 (EP-14a, EP-15a, EP-15b and EP-15d at the owner's
request, EP-15c by a split at pickup), and this re-plan adds EP-35 (Resume in VS Code from a ledger
row, in P3 before EP-19). v1 is now **41 briefs, 11 S · 24 M · 6 L ≈ 41.5 session-hours**, 18 of
them open. From EP-17 on, every EP of this project starts from the tool, visually by default (D-5,
D-16 addenda).

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
Sessions for this project run with cwd = this folder, and from EP-17 on they are launched from the
tool (D-5, D-16 addenda). Commits are the owner's call, written through a scratch file, no AI
attribution trailers.

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
| EP-7 | [Badges and status: stack chips, progress, activity from git, hosting from `gh` with head comparison and orphan detection, backup age from the Documents job, health](EP-7-badges-and-status.md) (D-24, D-25) | M | EP-6 | core | ☑ `47a2199` |
| EP-7a | [List and detail polish: columns sized to their content under a header row, a toggle for the badges' legend, the Status section laid out by subsection, one spelling of license](EP-7a-list-and-detail-polish.md) (D-25) | M | EP-7 | core | ☑ `6308fd9` |
| EP-8 | [Re-plan P1: retro and timings, addenda, README with self-screenshots, the remote flips public, P2 confirmed with pickup notes](EP-8-replan-p1.md) (D-2) | S | EP-1, EP-2, EP-3, EP-4, EP-5, EP-6, EP-7, EP-7a | core | ☑ `5f3aa64` |

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
- **EP-7a.** Added on 2026-11-01 at the owner's request (IN-4, written after OT-2), before the README's
  pictures and the public flip. (1) **Columns sized to their content** (the owner's choice): the
  Projects pane measures each column against the widest value the portfolio holds (the name, the
  state chip with its proposal mark, the next EP and size, the progress pill, the counts, and each of
  the badge column's own columns: the stack chip, the hosting, the sync, the age with the dirty mark,
  the CI run with the license warning, the backup), adds padding, and names the columns in a slim
  header row; the title takes what is left, a wide window shares its spare width instead of packing
  the badges tight, and a narrow one keeps D-25's give-way order (EP-7 addendum); no chip is clipped
  (`RS+1`, `PY+2`, `DKR`, `WEB`). Today's fixed widths are `list::{NAME, STATE, NEXT, PROGRESS, ITEMS,
  FULL, COMPACT}`; a text's width can be measured each frame with the context's fonts (about fifteen
  rows). Drag-resizable columns stay out (v2 notes). (2) **A legend toggle** (the owner's choice): a
  Legend button in the Projects pane's toolbar shows or hides a compact key below the list, with
  every icon and mark of the badges and its meaning, grouped as the badges are (stack, hosting, sync,
  activity, CI and license, backup), and the row's other marks (the pin, the proposal's bulb, a
  session in progress, git not asked yet); built from the same definitions as `badges.rs`'s marks so
  the key cannot drift from the row, its open state saved with the view. (3) **One spelling**: the
  window's text and `crw`'s output say license (the owner's choice), matching the LICENSE file name;
  DECISIONS.md keeps its wording. (4) **The Status section by subsection**: each subsection's name
  aligned with its first line rather than centred against its lines, a theme-coloured divider or
  partial border between subsections, and the same spacing at every window size and in both themes.
  Verification: screenshot runs at 1600 × 1000, 2300 × 1000 and a narrow 1100-point window, in both
  themes, with the legend open and closed, each picture looked at (a `legend` setup flag may help).
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
  **From EP-7 (the owner's answer at its close):** place D-24's actions, which no row holds: Create
  private remote, Make public, Make private and Remove remote (each behind a confirmation dialog and
  logged, `gh repo edit --visibility … --accept-visibility-change-consequences` repeating `gh`'s
  warning), Archive and Delete for an orphan (Delete needs the `delete_repo` scope the token lacks),
  and Back up now (the job's script with `-Force`); their state is already read
  (`badges::hosting::Hosting`, `badges::backup::Reading`), probably an M row in P2 or P4. The row's
  full badge form needs a wide window (`size=2300x1000`; at 1600 × 1000 the compact form shows and
  the counts give way), and the setup gained `sort=`, `stack=` and `hosting=` for the pictures.
  **From IN-4 (2026-11-01):** EP-7a joined P1 at the owner's request, so the plan's totals above
  (32 briefs, 18 M) are recounted here, and the README's pictures show EP-7a's layout.
  **From EP-7a:** the screenshot setup gained `legend` (the key open at the foot of the Projects
  pane); at 1600 × 1000 the compact badges show with the counts given way and the key takes two
  lines, at 2300 × 1000 every column is named and the whole key fits on one line, and at 1100 the
  optional columns are gone and the key scrolls; OT-3, the owner's look at the real portfolio, is
  gated before EP-8 so that its notes reach the pictures.

## Phase P2 — Sessions (planned 2026-10-31)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-9 | [Session runner core: CLI discovery and preflight, minimum version, `-p` stream-json child with job-object kill, `claude-codes` adapter, capability detection, pre-assigned session id, read-only neighbour folders by `--add-dir`, ledger rows, stub `claude` for tests](EP-9-session-runner-core.md) (D-14, D-15, D-16, D-18) | L | EP-2 | core | ☑ `71b2076` |
| EP-10 | [Prompt generator and launch dialog: the house-style opener, per-project template, model and effort picker from the catalog cache, rule table resolver, permission mode, Headless launch](EP-10-prompt-and-launch-dialog.md) (D-16, D-17, D-18, D-22) | M | EP-5, EP-9 | core | ☑ `7984abe` |
| EP-33 | [Per-project workspace files: Adopt writes `<name>.code-workspace` beside the folder, a rule for a fresh registry's `workspace_file`, `crw scan` lists the per-project files, a proposed state can be dismissed](EP-33-per-project-workspace-files.md) (D-6, D-7) | S | EP-2 | core | ☑ `4a25323` |
| EP-11 | [Visual launch: the extension URI with the prompt pre-filled, per-project window or workspace file when the running window's first folder differs](EP-11-visual-launch.md) (D-16) | S | EP-10, EP-33 | core | ☑ `0100629` |
| EP-12 | [Loopback server and signals: axum on 127.0.0.1, `cli hook` and `cli statusline` relays, installation of user-level hooks and the status-line entry, `claude agents --json` polling, session state model](EP-12-loopback-server-and-signals.md) (D-20, D-23) | M | EP-9 | core | ☑ `4a27db1` |
| EP-13 | [Sessions tile: a tab per session with its live transcript, Stop and Kill, Resume by id with a message, a list with filters over the ledger's rows; a row earned by a session's first work, unclaimed visual launches lapsed, `crw sessions`](EP-13-sessions-tile.md) (D-9, D-16, D-20, D-25, D-26) | M | EP-12 | core | ☑ `76a3482` |
| EP-14 | [Decisions tile: answers to `AskUserQuestion` and permission requests through the control protocol, the host named with `--permission-prompt-tool stdio`, a per-project time limit that defers the run, the defer-and-resume route, Focus for sessions waiting in VS Code, answer latency](EP-14-decisions-tile.md) (D-19, D-26) | L | EP-13 | core | ☑ `bcd386b` |
| EP-14a | [Crash log and the notice's time: a panic hook writing `crash.log`, the next start naming the last report once, the status bar's notice read once so its 8 s cannot underflow, notices cut to one line](EP-14a-crash-log.md) (D-14) | M | EP-14 | core | ☑ `daf2de9` |
| EP-15 | [Notifications: a popup of the window's own in place of Windows toasts, the tray icon with its count and menu, the event rules, quiet hours, closing to the tray and Quit, the log in the ledger](EP-15-notifications.md) (D-21, D-29) | M | EP-12 | core | ☑ `df99ba4` |
| EP-15a | [Readable notes and whole-row clicks: the Detail pane's brief notes, owner items and inbox set as paragraphs, their parts, code and references in the theme's colours; rows and cards that answer hover and click over their text and icons](EP-15a-readable-notes-and-row-clicks.md) (D-25) | M | EP-7a, EP-13 | core | ☑ `77c61b5` |
| EP-15b | [Revert a session: each session's start recorded (the HEAD, the branch, the uncommitted files copied to the backups folder), the revert in `crw_core` behind a preview of every commit and file it undoes, the work copied out first, refused while a session works there or once a commit is pushed, the tool's ticks redone, `crw revert`](EP-15b-revert-a-session.md) (D-11, D-26) | M | EP-13 | core | ☑ `61de8f6` |
| EP-15d | [The popup's crash: the popup a deferred viewport, since eframe 0.37 cannot open an immediate one while the window is hidden; a delayed test notification that reaches that path](EP-15d-popup-crash.md) (D-21) | S | EP-15 | core | ☑ `3446eeb` |
| EP-15c | [Revert in the window: an ended session's tab, the Sessions pane's ledger rows and a project's row menu, the preview as a dialog read off the UI thread, the window's own refusals](EP-15c-revert-in-the-window.md) (D-11, D-25, D-26) | M | EP-15b | core | ☑ `d23bb3c` |
| EP-16 | [Re-plan P2 and the dogfooding switch: retro, addenda, the CLAUDE.md rule that this project's EPs are launched from the tool, first self-launched session verified](EP-16-replan-p2.md) (D-5) | S | EP-9, EP-10, EP-33, EP-11, EP-12, EP-13, EP-14, EP-14a, EP-15, EP-15a, EP-15b, EP-15d, EP-15c | core | ☑ `2b1db41` |

Ordering rationale: the runner (EP-9) is the second engine and the base of everything unattended; the
prompt and dialog make Headless useful at once; Visual is small once the dialog exists; the loopback
server (EP-12) is needed by the sessions tile, the decisions tile, the notifications and later the usage
feed; the dogfooding switch closes the phase. *(EP-8, 2026-11-01.)* EP-33 comes before EP-11, which
opens the per-project workspace file EP-33 teaches Adopt to write; the order is otherwise kept.
*(2026-11-03, the owner's requests.)* EP-15a and EP-15b follow EP-15 and precede the re-plan, so the
window reads and clicks well and a session can be reverted before EP-16 makes the tool the way this
project's sessions start. *(2026-11-04, EP-15b split at pickup, the owner's choice.)* EP-15c puts
EP-15b's revert in the window, before the re-plan. *(2026-11-04, the owner's choice.)* EP-15d fixes
the popup's crash first, since a crash takes the window and its runs with it.

Standing decisions for phase P2: D-15 (never `--bare`; `ANTHROPIC_API_KEY` removed from the child
environment; features detected, not assumed), D-18 (acceptEdits with prompts routed to the host), D-19
(the protocol first, defer as fallback), D-3 (recorded streams for tests use fake project names and
contain no real transcript text). *(EP-8, 2026-11-01.)* D-16 and D-18 addenda: the project's neighbour
folders join a headless run by `--add-dir`, read-only through an `Edit` deny rule each; D-3 addendum:
every commit's added lines are checked before it is made, the repository being public.

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
  control responses, used by all runner tests; spacing of spawns ≥ 2 s (D-29). **From EP-7:**
  `process::turn()` is the one lock the threads that start children wait on (the git and GitHub
  passes hold it per child), and `process::output_with` passes extra environment variables, as
  `git::run_networked` does to keep credential prompts away. **From EP-8 (re-plan):** `crw doctor`
  found claude 2.1.316 at `%USERPROFILE%\.local\bin\claude.exe` (it updates itself; the extension
  bundles 2.1.314), the version of the `claude-codes` pin, which is the crate's newest; its `--help`
  has `--permission-prompts host|none`, `--session-id`, `--name`, `--add-dir`, `--settings`,
  `--plugin-dir` and `--max-budget-usd`, still no `--max-turns`, and lists the effort levels low …
  max without `ultracode`, so ultracode is passed only when detected; the minimum stays 2.1.261
  unless the adapter needs more (D-15). Each project gains `neighbours` in the registry (folders
  relative to the data root, or absolute); the spawn passes `--add-dir <folder>` for each,
  `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1` in the child's environment, and through
  `--settings` a `permissions.deny` rule `Edit(//<drive>/<path>/**)` per neighbour (Windows paths
  match in POSIX form), so `acceptEdits` cannot edit there (D-16, D-18 addenda). Whether that deny
  also stops the `mkdir`, `touch`, `mv` and `cp` that `acceptEdits` accepts in an added folder is not
  documented: the brief finds out (the documentation, or one supervised run on the owner's yes, since
  tests never run the CLI) and puts any gap to the owner. The ledger's `session` table (schema V1)
  already has the columns the runner fills.
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
  selected project is `Board::view.selected`. **From EP-8:** the dialog shows the project's
  neighbour folders (EP-9's registry key; D-16 addendum) beside the rule table's resolution.
  **From EP-9:** Headless is `session::Session::start(&preflight, launch)` with
  `Launch::new(project, folder, model, prompt)` and its fields (`ep`, `effort`, `ultracode`,
  `permission_mode`, `neighbours`, `max_budget_usd`, `ledger` = `paths::ledger_file()`); a
  `session::cli::Cache` keeps the preflight of `cli::locate(settings.claude_cli)` for an hour, its
  `refusals()` disable Headless with the reasons, and its `Flags` give the effort levels to offer,
  `ultracode()` and the permission modes; `Session::dropped` lists the wishes the CLI could not take,
  for the dialog to say. The neighbours are `Project::neighbour_folders(&data_root)`, empty in every
  registry entry today (written by hand, or by the tool on the owner's ask, rule 13). A start waits
  up to two seconds after the last (D-29), so it runs off the UI thread.
- **EP-33.** Added at EP-8 (2026-11-01) from IN-1 and EP-2's out-of-scope list (D-6, D-7 addenda).
  (1) `discovery::adopt` also writes `<data root>\<name>.code-workspace` beside the adopted folder
  when none exists, in the owner's form (one folder with its `name` and `path`, an empty
  `settings`), and still appends the folder to the aggregate file `workspace_file` names; it never
  overwrites a file. (2) A fresh registry's `workspace_file` is the one `.code-workspace` in the
  data root that lists more than one folder, else none, said in `crw scan`
  (`registry::only_workspace_file` wants exactly one file today, and the data root holds seventeen).
  (3) `crw scan` lists per project whether its own workspace file exists, beside the aggregate's
  listing. (4) A proposed state (D-6: every row checked, a pickup gate, 60 days without a commit)
  can be dismissed from the chip's menu and from `crw`, remembered in the registry with the facts
  it rested on, so it returns only when those change. Tests in temp folders with synthetic names;
  no test or trial touches the owner's registry or workspace files (rule 13).
- **EP-11.** Visual launch: build `vscode://anthropic.claude-code/open?prompt=<url-encoded opener>`;
  read `~/.claude/ide/*.lock` to learn the running window's `workspaceFolders`; if the first folder is the
  project, run `Code.exe --open-url "<uri>"`; otherwise open `Code.exe <project folder>` (or a generated
  `<project>.code-workspace` in `%APPDATA%\clauderoadmapwrangler\workspaces\` when the project wants extra
  folders), wait for its lock file, then fire the URI; the session is recorded as `visual` with no process
  handle; a note in the dialog that the prompt is pre-filled and must be sent by hand. **From IN-1
  (2026-10-31, triaged at EP-4):** open the owner's existing `<data root>\<name>.code-workspace` beside
  the project folder instead of generating one under `%APPDATA%` (the D-16 addendum written at EP-8);
  generate one only when no such file exists. **From EP-8:** every project has its file today (the
  data root holds seventeen: each project's, the vault's and the aggregate), in the form EP-33 writes
  for an adopted folder; a missing one is generated through EP-33's writer. **From EP-10:** the
  dialog's Visual button waits, disabled, in its footer (`app::launch::Dialog`); the text to send is
  `Dialog::prompt` as the owner left it, the same for both launches, and `prompt::draft` with
  `Dialog::new` opens the dialog; the opener says "launched from clauderoadmapwrangler … with
  <model> at <effort>", which a visual launch chooses in VS Code instead, so the brief words that
  sentence for it. Until then `crw prompt <project>` prints the opener for pasting by hand.
  **From EP-33:** the project's own file is `workspace::own_file(folder)` (`<folder>.code-workspace`
  beside the folder) and `Folder::own_workspace` says whether it is there; a missing one is
  written by `discovery::write_own_file(folder)` in the owner's form, created new so that it never
  replaces a file (`OwnFile::Written` or `Kept`); a screenshot run writes none, its data root being
  the tracked fixtures (Adopt passes `OwnFiles::Skip` when `board.fixtures`). `crw scan` found
  every registered project with its own file (2026-11-02).
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
  needs the mailman's registry lookups). **From EP-8:** a running session joins the snapshot's
  in-progress reasons (`roadmap::snapshot`) as the source D-9's EP-4 addendum left to this brief.
  **From EP-10:** the sessions the window starts already mark their rows in progress
  (`app::sessions::Sessions::summary`, joined into the Session cell's reasons), so the state model
  takes those from `board.sessions` and adds the ones the hooks and `claude agents --json` report.
  **From EP-11:** a visual launch is a `session` row of mode `visual` with no session id, model or
  effort, written when the link is handed over (`visual::open`); the SessionStart hook's session
  id and cwd can complete the newest open visual row of that project, and SessionEnd close it. The
  extension's lock files are read by `visual::windows` (`pid`, the window's folders, `ideName`; the
  auth token never kept), and `alive` asks whether VS Code's main process runs, which every window
  shares.
- **EP-13.** The sessions tile: one tab per live or recent session; transcript rendering from the stream
  (assistant text, tool calls as one-line summaries, tool results collapsed, partial messages appended);
  Stop (graceful) and Kill; Resume by session id for a headless session that ended with a deferred tool use
  or a stop; the ledger row completed with outcome (completed, stopped, failed, deferred), end time,
  result text, `total_cost_usd` estimate and the status-line `cost`/`context_window` figures received for
  that session id; a session list with filters. **From EP-8:** the Sessions tab is EP-6's placeholder
  in the right-hand group, and the `session` table's columns (`outcome`, `cost_usd`, `questions`,
  `answer_latency_s`, `commit_hash`) exist since the ledger's V1. **From EP-9:** the tile reads
  `Session::events()`: `TextDelta` while text streams, then the whole `Text`, `ToolUse` in one line,
  `Request`, `Withdrawn`, `Denied`, `Notice`, `Unreadable` (a diagnostics line, not transcript) and
  last `Exited(Ended)` with the outcome, the exit code, the result and stderr's last lines; Stop and
  Kill are `Session::stop()` and `kill()`. A run closes its stdin at its result (D-16 addendum), so
  Resume is a new launch with `--resume <id>`, a `Launch` field this brief adds; the runner starts
  and finishes the ledger row (`Ledger::sessions()` reads them). **From EP-10:** the Sessions pane
  lists the window's sessions in plain lines until this tile (`app::sessions`: a `Live` keeps the
  handle, the last line, the waiting requests and the notices; `Sessions::receive` drains every
  session's events each frame and asks for a frame every 250 ms while one runs; a start comes back
  as `Started` from a worker thread); the tile replaces `session_card` with the transcript and
  keeps Stop, Kill and Dismiss. **From EP-11:** the extension's link also takes `session=<id>`,
  which resumes a session in the project's VS Code window; `visual::link` writes the `prompt`
  parameter only today, and a visual row has no session id until EP-12's hooks give one.
  **From EP-12:** the sessions started elsewhere are `crw_core::sessions::Seen` in the window's
  `watch.model` (mode, status, `waiting_for`, name, surface, start, the latest `signals::StatusLine`),
  listed read-only under the window's own cards (`sessions::seen_card`, `Board::seen_sessions`); a
  visual row gains its session id at the session's start (`Ledger::claim_visual`, within 30 minutes
  of the launch) and closes at its end (`Ledger::end_seen`), and an interactive session of a
  registered project started by hand has a `manual` row. The status line's `cost` and
  `context_window` arrive per session id, but most likely from terminal sessions only (D-23
  addendum), so a headless row's cost stays the result's `total_cost_usd`.
- **EP-14.** The decisions tile: `control_request` → `can_use_tool` handling in the adapter; for
  `AskUserQuestion` the questions are rendered with their options (single or multi-select, free text),
  the answer sent as `updatedInput` with `answers`; for permission requests Allow / Deny with the tool name
  and input shown; an optional per-project timeout after which the tool defers the run (sends deny with a
  reason and marks the session `deferred`); the **fallback**: a PreToolUse hook passed with `--settings`
  that returns `permissionDecision: "defer"` for `AskUserQuestion`, the run exiting with `stop_reason:
  tool_deferred`, the tile showing `deferred_tool_use`, and Resume re-launching with `--resume <id>` and a
  one-shot hook that returns `allow` with the answer; the fallback is selected automatically when the
  protocol handshake fails or by a setting; visual sessions in `waiting` show "waiting for you in VS Code"
  with Focus (bring the VS Code window to front); answer latency recorded. **From EP-8:** the
  Decisions tab is EP-6's placeholder; an edit a neighbour's deny rule refuses (D-18 addendum) comes
  back as a denied tool use, not a question, and the tile lists it with the session. **From EP-9:**
  answers go through `Session::respond(id, Response::{Allow, Deny(message), Answers(one per
  question)})`, the adapter keeping the questions and keying the answers by question text;
  `Event::Handshake(Err(_))` is the protocol failure that selects the fallback; a refused tool use is
  `Event::Denied` with the CLI's reason; `Outcome::Deferred` and `Finish::is_deferred()` exist; CLI
  2.1.316 names `ask_user_question_unanswered_v1` among its capabilities, which this brief reads up
  on. **From EP-10:** the Decisions pane lists each waiting request read-only (`sessions::request_card`
  over `Live::waiting`, the tab's title counting them), so the tile adds the answers there, through
  `Session::respond` on the `Live`'s handle. **From EP-11:** Focus is the first half of a visual
  launch: `Code.exe <own workspace file>` brings the project's window to the front
  (`workspace::own_file`; `visual::Readiness::window_open` says whether one runs the extension).
  **From EP-12:** a session started elsewhere that waits is `sessions::Status::Waiting` with what it
  waits for (`Seen::waiting_for`: a Notification hook's `permission_prompt`, `elicitation_dialog`
  or `agent_needs_input` in words, or the poll's `waitingFor`), and its project's row says so
  (`InProgress::Session`); "waiting for you in VS Code" with Focus belongs on that card.
  **From EP-13:** a window's session keeps its `Launch` and resumes through `Sessions::resume(key,
  preflight, message)` (`--resume <id>`, the same id, a ledger row per run, the same tab), which the
  fallback's Resume can reuse with its one-shot hook; the transcript shows each request as
  `transcript::Entry::Asked`, and the Decisions pane is still `sessions::request_card` over
  `Live::waiting`. A tab open with no prompt yet is `Seen::is_new()`, no session in progress; the
  seen card (`sessions::seen_card`) is where Focus goes.
- **EP-14a.** Added on 2026-11-03 at the owner's request, before EP-15. The window vanished without
  a word as another project's app closed, a few seconds after a headless run took the owner's long
  answer. The runner killed the run and recorded it as `stopped`, and Windows kept no crash record:
  a Rust panic, which a console-less release build prints nowhere. Its message is lost. The fix: a
  panic hook that writes `crash.log` in the local folder, and the next start naming the last report
  once. Also the status bar's definite panic of the same kind (`NOTICE_FOR - at.elapsed()` read a
  second time after its check) and notices cut to one line. AccessKit stays: turning it off was
  considered and declined.
- **EP-15.** Toasts through notify-rust (WinRT) with the AppUserModelID of the Start-menu entry from
  `install-shortcut.ps1` (text: project, EP, event, one line of detail; click focuses the window); the tray
  icon (tray-icon 0.26) with a badge count of sessions waiting, a menu (Show, Start next for each pinned
  project, Pause autopilot, Enable remote view when EP-31 exists, Quit) and the close-to-tray behaviour;
  event rules per D-21 (toast: waiting, finished, failed, usage 80 % and 95 %; badge: unadopted, orphan or
  mismatched remote, stale backup, inbox arrival); quiet hours in the registry; a notification log in the
  ledger. **From EP-7:** the badge events have their readings: `Hosting::needs_attention()` (an orphan
  or a missing repository), `Verdict::{Diverged, NotFetched}` of a current head check, and
  `backup::State::{Stale, Failing, NotCovered}`; they reach the window through `portfolio::Loader`.
  **From EP-10 (the owner's answer at its close):** closing the window kills every headless session
  it started (the runner kills a session whose handle is dropped, D-16 addendum), and only the
  dialog's footer and the start's notice say so; close-to-tray must keep the window's sessions
  (`board.sessions`) alive, and Quit says how many it stops. **From EP-12:** waiting, finished and
  failed reach the window at once through the hooks (`watch::Watch::receive` applies each `Signal`
  to `sessions::Sessions`, whose `Change::Started` and `Ended` it also records), and the poll of
  `claude agents --json` covers what they miss; close-to-tray keeps the server and the poll, both
  threads of the window's (`app::server`, `watch`). **From EP-13:** `Change::Started` now marks a
  session's first sign of work, not its start, and a session that ends without working leaves the
  window at once, so neither a helper of the extension's nor a restored tab should toast; a window's
  run has ended when `Live::drain` returns true (its `Event::Exited`). **From EP-14:** what waits
  for the owner is `Board::decisions_count()` (the requests the window's sessions wait on, the
  runs deferred with a request open, the sessions waiting elsewhere), which the Decisions tab's
  title already counts and the tray's badge can show; the moments for a "waiting" toast are an
  `Event::Request` taken in `Live::take`, an `Event::TimedOut` (a project's time limit deferred the
  run) and a result whose `Finish::deferred` names a call (the defer route), and a toast's click
  can bring `Pane::Decisions` forward.
- **EP-15a.** Added on 2026-11-03 at the owner's request, with EP-15b, after EP-15 and before the
  re-plan (the owner's choice): two window fixes in one M, as EP-7a took IN-4's notes, in place
  before EP-16 makes the window the way this project's sessions start. (1) **Readable notes.** The
  Detail pane draws the next brief's notes, the items waiting on the owner and the inbox through
  `markup`, whose `reflow` joins every continuation line into one paragraph and whose `spans` mark
  bold, code and struck text only, so a bullet such as EP-15's above is one block of some 2,500
  characters in which the owner loses their place. The renderer learns the notes' shape: a segment
  that opens with a bold label ending in a colon (`**From EP-7:**`, `**From EP-10 (the owner's
  answer at its close):**`, `**From IN-1 (…):**`) starts a paragraph under its label, numbered parts
  `(1) … (2) …` become an indented list, blank lines stay, and lines keep a reading measure (about
  100 characters) in a wide pane; the runs take colours from `style` that hold in both themes:
  inline code in the code face with its kind told apart where it is plain (a path, a command such as
  `crw …` or `git …`, a name in the code such as `list::cell`), the ids `EP-n`, `D-n`, `OT-n` and
  `IN-n` in a reference colour, bold labels in the accent, a `before EP-m` gate in amber, struck
  text weak, a link as its text in the link colour. An owner item gets the same renderer, a long one
  split at its sentences or steps (`Optional, spends usage:`, `Notes go to …`), its gate a chip
  beside its id; the inbox and the mail share it. Nothing is reworded or dropped, and a form the
  rules do not know (another project's) draws as today; the launch dialog's prompt stays source
  text, being what the session is sent. Which of these the owner wants, and whether a segment
  folds, is the brief's first question. (2) **The whole row answers** (the owner's finding). A row
  takes its rect with `Sense::click()` before its cells, and egui 0.37 makes every label selectable
  by default (`style.interaction.selectable_labels`): a selectable label senses click and drag, so
  over the name, a chip, the title, the counts, a badge or an icon the label takes the pointer, the
  row is neither hovered nor clicked, and a right click misses the row's menu (`egui-0.37.0`'s
  `widgets/label.rs` and `interaction.rs`, read 2026-11-03). The fix: a row's labels are not
  selectable, so they sense hover only, keep their hover texts and leave the click to the row, while
  the expander, Start and the autopilot box keep their own. Every cell is drawn through
  `list::cell`, one place for the rule, beside the progress pill's `ui.put`; the folder rows and the
  group headers are made alike, and the Sessions pane's cards, which add their click to a frame
  drawn under their labels (`sessions.rs`), are checked too. A row's text can then no longer be
  selected for copying; the Detail pane's stays selectable. A test drives an `egui::Context` with the
  pointer over a row's name and expects the row hovered and clicked. Verification: screenshot runs
  at 1600 × 1000, 2300 × 1000 and 1100 wide, both themes, on a fake project whose next brief has a
  long note in every form (the fake portfolio may need one), each picture looked at; a D-25 addendum.
  **From EP-15:** `style::red` is egui's own `error_fg_color`, a pure red that reads dim on a
  button in the dark theme (the Decisions pane's Deny, the Quit question's `quit` picture), worth
  its look beside the notes' colours; the menu bar now opens with a File menu. A session the poll
  reports waiting reads "waits for input needed" on its card and row (`Seen::doing` puts Claude
  Code's own `waitingFor` words after "waits for"), which the popup words "waits for you: input
  needed" instead.
- **EP-15b.** Added on 2026-11-03 at the owner's request, after EP-15a and before the re-plan (the
  owner's choice): a reliable way in the window to put a project back as it was when a session
  started, in the house style the end of the last completed brief. On 2026-11-03 the owner reverted
  a run's work by hand after the window vanished (EP-14a). Claude Code's checkpoints (`/rewind`) are
  no substitute: they track its file tools' edits only (a file only Bash changed stays, a subagent's
  edits mostly stay, a commit is untouched) and are swept after about 30 days (its checkpointing
  page, read 2026-11-03); the revert is git's, to a start the tool records. (1) **The start** (the
  owner's choice): the project's HEAD, its branch and every file changed or untracked then, those
  files copied into the backups folder (`paths::backups_dir()`, as a tick's copy, D-10 addendum),
  nothing written into the project; read at a headless launch, and at the start hook of a visual or
  manual session (before its first prompt), kept by the window until the session earns its row
  (EP-13) and then written with it (schema 4); a resumed session keeps its first run's start. With
  none recorded (no window ran at the start, or before this brief), the target is the commit that
  recorded the last checked row's hash (`docs(roadmap): record EP-n commit hash`), and the preview
  says that changes uncommitted before the session cannot be told from its own. (2) **Where:** an
  ended session's tab and its row in the Sessions pane's All list (a run whose tab a crash took keeps
  it; EP-14a's note asks EP-16 for a Resume there too), the project's row menu for its latest
  session, and `crw revert <project> [<session id>]`, which previews and reverts only with `--yes`.
  (3) **The preview**, one confirmation: the target (recorded or derived), each commit and each file
  changed, created or deleted since it, what stays (ignored files such as `target\`, the start's own
  uncommitted files), an idle session still open there, and where the copy goes. (4) **Nothing
  lost:** the work is first copied out of the project into the backups folder (the brief's choice of
  form: a `git bundle` of the commits undone, a binary patch, the new files); then `git reset --hard
  <start>`, the files the session created removed (never an ignored file, no `git clean -x`) and the
  start's files put back, one child at a time under `process::turn()` (D-29); an `event` row of kind
  `revert`; the snapshot read again. (5) **Commits** (the owner's choice): those the remote lacks are
  undone with the rest; one the remote has, read from the remote-tracking ref a push moves, without a
  fetch (D-24 addendum), refuses the revert, and the tool never force-pushes (v2 notes). (6)
  **Refused** too while a session works or waits there (the window's, the hooks', the poll's), when
  the branch changed or the start is no ancestor of HEAD, and in a screenshot run. (7) **The tool's
  own writes** during the session (a tick; from EP-20 a delivered message) are kept, re-applied after
  the restore, or named in the preview to redo where that cannot be exact (the brief's choice).
  Records: a D-11 addendum (the tool's first write into a project beyond its own files, restoring
  the project's own earlier state: only on the owner's click after the preview, never by the
  autopilot or a rule, never while a session runs there, never a push) and a D-26 addendum (the
  start in the ledger, the `revert` event). Tests on scratch repositories in temp folders, as EP-7's
  git tests make them, never the owner's projects (rule 13); a trial with `CRW_HOME` on a scratch
  project; if it outgrows its M it splits at pickup (D-4). **From EP-15:** the `event` table has
  its writer and reader (`Ledger::add_event`, `Ledger::events` by kind), so the `revert` event is
  a row beside the notifications' (kind `notification`). **From EP-15a:** a card or a row of the
  Sessions pane that takes a click makes its labels hover-only with `style::unselectable`, as a
  window session's card does (its click already reached it over its text, the frame lying on top
  of its labels, but a selectable label took the drag and the double click); `testing::Screen`,
  with `testing::moved`, `button` and `fills`, drives a bare context pass by pass, so a test can
  press Revert or a row and see what the press reaches.
- **EP-15d.** Added on 2026-11-04 at the owner's request, after the window closed on a panic in the
  popup at 19:34 on 2026-11-03 ("egui backend is implemented incorrectly - the user callback was
  never called", `egui-0.37.0/src/context.rs:4714`). A hidden or minimized window gets no
  `RedrawRequested` on Windows, so eframe 0.37 paints it from `check_redraw_requests`. When that
  runs from `new_events`, the one handler not wrapped in `with_event_loop_context`, an immediate
  viewport that needs a new window gets none, its callback never runs, and egui panics. The first
  notification after the popup was empty does exactly that while the window is in the tray. The
  fix: the popup as a deferred viewport, whose window eframe creates on every path
  (`initialized_all_windows(event_loop)`), its toasts behind a shared lock, the owner's choice
  applied in the popup's own pass; and a test notification that waits ten seconds, so that the
  window can be closed to the tray before it comes. The egui family stays (D-15).
- **EP-15c.** Added on 2026-11-04 when EP-15b was split at pickup (the owner's choice, D-4): EP-15b's
  places (2) and preview (3) in the window, on its engine. (1) **The three places**: an ended
  session's tab, beside Resume (`Live::session_id`); a ledger card of the Sessions pane's All tab
  whose row has a session id (`ledger_line`), so that a run whose tab a crash took keeps its
  Revert; and a project's row menu for its latest session (`list.rs`'s `context_menu`). (2) **The
  preview** is a dialog like the Quit question (`Board::quit_dialog`): `Plan::lines()` in a
  scroll area, then Revert in the window's red and Cancel, or Close alone when it is refused. A
  plan asks git about six children, each in its turn (about two seconds), and a revert about five
  more, so both run on a worker thread, as the visual launch does, the dialog showing a spinner.
  (3) **The window's own refusals**, which `crw` cannot see: its own runs in the project, a start
  on its way, the sessions the hooks and the poll report working or waiting there, and a
  screenshot run. (4) **After a revert**, the files and git are read again, the All tab's rows go
  stale, and the status bar says where the work is kept. A test drives a bare context to press
  Revert. Screenshot runs show the dialog over the fake portfolio with a canned plan, in both
  themes. **From EP-15b:** `crw_core::revert` holds `choose(rows, project, id)` (the session by
  its id or the latest, its start, the later sessions), `plan(&chosen, &Context)` and
  `revert(&plan, ledger, backups, today)`. `Context` takes what the window knows: its switch
  (`row.project.revert`), the sessions at work there in words (`working`, each a refusal), those
  open idle (`idle`), the ticks since the start (`ticks_since`), the backups folder
  (`Store::backups`). `Plan::lines()` and `Done::lines()` are the words `crw revert` prints. A
  session's start is in the ledger from EP-15b on (`SessionRow::start`, `Ledger::start_of`): the
  runner records it for a headless run, and the window's `watch` at a VS Code session's start
  hook. A session from before EP-15b, or one that started while no window ran, has none and goes
  back to the last recorded brief. The window logs its ticks (`revert::log_tick`), and a revert
  redoes them.
- **EP-16.** Retro, addenda, `CLAUDE.md` rule 11 switched to "launched from the tool", the first
  self-launched session (EP-17's) started from the dashboard with the generated opener and verified in the
  ledger, P3 rows confirmed with pickup notes, `v2-notes.md` grown. **From EP-8:** time the briefs as
  EP-8 did: each session's transcript under this project's slug, found by the Write or Edit of its
  own `roadmap\EP-n-*.md` (the fixtures hold briefs of the same names, so match this repository's
  path), from its first timestamp to the EP's last commit, with gaps over five minutes left out as
  waits on the owner; timestamps and file paths only. **From EP-11 (the owner's answer at its
  close):** revisit the tool's default launch mode, `rules::DEFAULT_LAUNCH_MODE`, Visual while a
  headless session cannot answer the questions a brief ends with (D-16 addendum); the first
  self-launched session may be visual, its ledger row then of mode `visual`. **From EP-12:** with the
  hooks installed (2026-11-02), a visual launch's row gains its session id when the session starts
  and its end when it ends, which is how the ledger verifies the self-launched session; OT-6, the
  owner's look at a real session in the Sessions pane, comes first. **From EP-13:** the claim comes
  at the session's first sign of work, by a session that started as a new conversation within 30
  minutes of the launch, and an unclaimed launch lapses after 30 (D-26 addendum), so the
  self-launched session's row shows its id once its first turn has run or the poll has seen it busy;
  `crw sessions <project>` prints the rows with each run's length, read only. OT-7, the owner's look
  at the tile on the real ledger, comes first. **From EP-14 (the owner's answer at its close):** a
  headless session's questions and permission requests are now answered in the Decisions tile
  (verified on claude 2.1.316 by a supervised probe, D-19 addendum), which removes the reason
  Visual became the default; the owner left `rules::DEFAULT_LAUNCH_MODE` for this re-plan to decide
  with the first self-launched session in view. A project's `answer_timeout` and the defer route
  (`crw setting decisions`) are the owner's to set; OT-8, the owner's look at the tile, comes before
  EP-15. **From the owner's requests (2026-11-03):** EP-14a, EP-15a and EP-15b joined P2 at the
  owner's request, so the plan's totals above (35 briefs, 20 M) are recounted here: 38 briefs, 9 S ·
  23 M · 6 L ≈ 39.5 session-hours, 21 briefs still open. The retro times the three with the rest, and
  the first self-launched session can be reverted from the Sessions pane (EP-15b) should it go
  astray. **From EP-15:** a headless run's questions and its end now pop up while the window is
  not in front, and closing the window keeps its runs in the tray, which bears on the default
  launch mode the re-plan decides; a visual session's waits pop up too, and its long turns' ends.
  **From EP-15a:** the README's two pictures (`docs/dashboard-light.png` and `-dark.png`, taken at
  EP-8) show the Detail pane before its notes were set for reading. A trailing `[urgent]` stays in
  an inbox message's text while its chip says it, which the gate's rule could also cover. In a
  narrow window the Progress section's phase names run past the pane's edge (an egui `Grid` does
  not wrap), seen at 1100 wide. **From EP-15b (split at pickup, the owner's choice):** EP-15c
  joined P2, so the totals above are 39 briefs, 9 S · 24 M · 6 L ≈ 40.5 session-hours, with 19
  still open once EP-15b is checked. The first self-launched session can be reverted with
  `crw revert` (EP-15b) or from the window (EP-15c). The retro times EP-15b and EP-15c with the
  rest. **From EP-15d (the owner's request):** with its S row the totals are 40 briefs, 10 S · 24 M
  · 6 L ≈ 41 session-hours, 19 still open once EP-15d is checked. eframe 0.37 cannot give an
  immediate viewport a window while the window is hidden (D-21 addendum): a re-plan that moves the
  egui family can look for the fix upstream, and no other immediate viewport should be added
  until then. **From EP-15c:** the window's Revert is built (D-11, D-25 addenda): an ended
  session's tab, a ledger card of the All tab and a project's row menu open the preview, which is
  read on a worker thread and read again at the owner's word, so the first self-launched session
  can be reverted from the window should it go astray. The preview refuses while the poll of live
  sessions has not answered, so a window just opened needs its first poll (a few seconds) first.
  OT-13, the owner's look at the dialog on the real portfolio, comes before this brief; 18 briefs
  are still open once EP-15c is checked.

## Phase P3 — Automation, usage, inbox, method (planned 2026-10-31)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-17 | [Usage meter: endpoint poller with per-poll token read, status-line feed merge, degrade mode, strip and tile with countdowns and surface breakdown, samples, 80 and 95 percent toasts (D-23)](EP-17-usage-meter.md) | M | EP-12 | core | ☑ `3cf6dcf` |
| EP-18 | [Downshift rule: the Fable-share downshift as the resolver's last layer with its registry mark, its judgement in the launch dialog and `crw prompt`, the dialog's Visual pickers greyed out for the tab's panel](EP-18-downshift-rule.md) (D-23, D-16) | S | EP-10, EP-17 | core | ☑ `8014ad6` |
| EP-18a | Usage attribution: weekly and Fable percents sampled around each launched session on its ledger row, the delta per session, the usage-share badge with its column, Detail line and sort key (D-23) | S | EP-18 | core | ☐ |
| EP-35 | Resume in VS Code: a ledger row's session reopened in the project's own window through the extension's `session=<id>` link, for a run a crash or a quit window cut short and for visual and manual sessions (D-16, D-26) | S | EP-11, EP-13 | core | ☐ |
| EP-19 | Autopilot: per-project toggle, the gate, the commit variant of the prompt, Commit button resume, per-run and per-day caps, pause at 90 percent, global Pause, chain log (D-22) | L | EP-14, EP-17 | core | ☐ |
| EP-20 | Inbox delivery: `Handed on (to …)` parsed at completion, append to the receiver's inbox, delivery log, compose dialog, urgent flag, ntfy publisher (D-13, D-21) | M | EP-5, EP-13 | core | ☐ |
| EP-21 | Adoption and formalise files: CLAUDE.md rule block, `owner-tasks.md`, `inbox.md`, the brief template with front matter, the migration prompt for packet-style projects (D-10, D-12, D-13) | M | EP-5, EP-10 | core | ☐ |
| EP-22 | Roadmap-wrangler plugin: skills plan, replan, formalise, status and the house-style reference; loaded with `--plugin-dir`; install notes (D-27) | M | EP-21 | core | ☐ |
| EP-23 | Re-plan P3 | S | EP-17, EP-18, EP-18a, EP-35, EP-19, EP-20, EP-21, EP-22 | core | ☐ |

Ordering rationale: the meter precedes the autopilot because the gate reads it; the inbox mailman needs
completion-note parsing from EP-5 and session completion from EP-13; adoption files and the plugin make the
method installable before the chat (P4) delegates to it. *(EP-16, 2026-11-04.)* The order is kept (the
owner's choice). EP-35 follows the usage briefs and comes before the autopilot: from the switch on
(D-5) the window is how this project's sessions start, so a run a crash cuts short should be picked
up from the window before chains begin; EP-17 stays the first self-launched session.
*(2026-11-04, EP-18 split at pickup, the owner's choice.)* EP-18a takes EP-18's attribution and
usage-share badge, right after it and before EP-35.

Standing decisions for phase P3: D-22 (the gate, the commit policy, the caps), D-23 (the endpoint is
undocumented and breakable; the token is read per poll and never stored or logged), D-13 (never interrupt
a running session). *(EP-16, 2026-11-04.)* D-5 addendum: every EP of this project starts from the
window; D-16 addendum: Visual stays the default, and the autopilot runs headless whatever it is.

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
  per window). **From EP-7:** the ledger is at schema 2 (`badge_cache` added) and the window creates it
  at its first start; the usage rows go into V1's `usage_sample`, and the endpoint's poll can follow
  the GitHub pass's pattern (cache first, a failure shown and not stored). **From EP-12:** the
  status-line feed exists: `crw statusline` posts `signals::StatusLine` (`rate_limits.five_hour` and
  `seven_day` with `used_percentage` and `resets_at`, cost, context, model) to the window, where
  `watch.model` keeps each session's latest (`Seen::line`); the installed status line prints
  nothing and runs every 30 s. The VS Code extension most likely never runs it (D-23 addendum), so
  the feed comes from terminal sessions and the endpoint is the meter's real source; a `usage_sample`
  row of source `statusline` is written from `watch::Watch::receive`. **From EP-13:** a session's end
  keeps its last status line's cost and context share in its row (`Ledger::end_seen`, schema 3's
  `context_percent`), so the samples need not carry them. **From EP-15:** the usage notifications
  join `crw_core::notify`'s rule table as a kind of their own (`Kind`, `Channel::Toast`), each made
  a `toasts::Toast` and handed to `Toasts::deliver`, which shows it in the window's popup, holds it
  back in the quiet hours and while the window is in front, and logs it; "once per crossing per
  window" is the meter's to keep. **From EP-16:** EP-17 is the first self-launched session (D-5
  addendum): the owner starts it from this project's row as a visual launch, and its session start
  records its ledger row. The user settings deny Claude's own tools a read of
  `~\.claude\.credentials.json`, which the poller reads in the tool's own code at poll time (D-23),
  so the session that builds it never reads the real file: its tests use a synthetic credentials
  file in a temp folder and never reach the endpoint, and the first real poll is the window's,
  after the owner's yes. The ledger needs no migration for the meter: `usage_sample` is there since
  V1, and the session row's `weekly_before`/`after` and `fable_before`/`after` wait for EP-18
  (schema 4).
- **EP-18.** The resolver (EP-10) gains the downshift rule: when the Fable-scoped percent exceeds the
  registry's `fable_downshift_percent` (default 60) and the rule's size is S, propose `claude-opus-5-5` at
  the same effort with the reason shown; the autopilot applies it; attribution: the weekly and Fable
  percents sampled before launch and after the result are stored on the session row, the delta shown per
  session and summed per project for a usage-share badge (this week) in the list and the detail tile;
  sessions the owner starts get the status-line cost estimate only. **From EP-7:** the usage-share
  badge joins the row's badge column as one more of its own columns (`list::FULL`, `list::COMPACT`),
  a line of the Detail pane's Status section, and the sort key `SortKey::UsageShare`, which waits
  for it (`arrives_at`). **From EP-10:** the resolver is `rules::resolve` in layers (the default,
  the project's launch defaults, the first global rule, the first project rule), each value a
  `Choice` with its `Source`; the downshift is one more layer with a `Source` of its own, its place
  among the others this brief's call, and the dialog shows `Choice::reason()` as it is.
  **From EP-16:** `rules::DEFAULT_LAUNCH_MODE` stays Visual for the D-16 addendum's reasons, and its
  comment still gives EP-14's: this brief, which edits `rules.rs` for the downshift, words it anew.
  **From EP-16 (the owner's finding at the first self-launch):** in Visual mode the dialog offers
  the model, effort and ultracode pickers, each marked "a suggestion: VS Code's panel chooses", yet
  the owner set max there and found the new tab at VS Code's own effort. The extension's link
  takes only `prompt` and `session` (2.1.318's `/open` handler, read 2026-11-04), and the panel's
  effort is a preference the extension stores for itself, so no launch can set it. This brief,
  whose downshift is also only a suggestion for a visual launch, makes the dialog plain about it:
  in Visual mode the pickers serve Headless only, or they say that the tab's own panel is where
  the owner chooses, once it opens. **From EP-17:** the figures are `usage::Gauge::figures(now)`
  (the window's `meter.gauge`), each with its `source` and the time it was read. Fable is the
  endpoint's alone: absent before the first reading, and the last reading's while the endpoint
  fails (`Figures::degraded` says why), so the downshift reads it with its age. The session row's
  `weekly_before`/`after` and `fable_before`/`after` are V1's columns and need no migration. The
  meter polls as soon as a launched session ends (no sooner than a minute after the last poll),
  so the "after" figures can be the first `endpoint` sample after the row's end and the "before"
  the last one before its start (`Ledger::samples_since`); a `statusline` sample carries its
  session's id.
- **EP-18a.** Added on 2026-11-04 when EP-18 was split at pickup (the owner's choice, D-4): EP-18's
  attribution and badge, on EP-17's samples. (1) **The figures around a session**: the weekly and
  Fable percents before and after each launched session (a headless run of the window's, a visual
  launch), on the row's `weekly_before`/`after` and `fable_before`/`after` (V1's columns, no
  migration); the delta shown per session (the Sessions pane's ledger card, `crw sessions`) and
  summed per project over the weekly window (the sessions since its reset less seven days).
  (2) **The usage-share badge**: one more column of the row's badges (`list::Column`, in both of
  `GIVE_WAY`'s forms with the badges), a line of the Detail pane's Status section, the legend's
  key, and the sort key `SortKey::UsageShare` (its `arrives_at` says EP-18). (3) Sessions the
  owner starts by hand keep the status-line cost estimate only (D-23), which `Ledger::end_seen`
  already writes. **From EP-18 (what the pickup found):** four facts for the brief's first
  question to the owner. (a) A visual row ends at its tab's `SessionEnd`, which can come hours
  after its last turn: EP-17's row was still open at EP-18's start, its commit hours old. So an
  "after" read at the row's end takes in whatever ran meanwhile, and the end of the session's last
  turn (its `Stop` hook, which the window hears and the ledger does not keep) is the better mark.
  (b) The owner runs sessions in parallel, so a delta over one session's span holds the others'
  use. Either each delta is shown with the sessions that overlapped it (the ledger's spans), or
  each poll's delta is split among the sessions that worked since the last one (the watch knows
  who was busy: the poll's `busy`, a `Stop`, a window's run), which also leaves out a tab's idle
  hours. (c) `crw usage` showed the weekly and Fable figures unchanged over five polls in half an
  hour of EP-18's reading, so an S session may move the weekly figure by a point or two at most:
  per-session deltas are coarse, and the week's sum per project is the steadier figure. (d) D-23
  attributes tool-launched sessions only; the other projects' sessions are mostly manual today, so
  a badge of launched sessions alone leaves them near empty, and counting manual sessions changes
  D-23, the owner's call. The meter polls a minute after a launched session ends
  (`Meter::session_ended`), `Ledger::samples_since` reads the samples, and OT-14 comes first.
- **EP-35.** Added at EP-16 (2026-11-04) for EP-14a's and EP-15b's notes (D-16 addendum): a run a
  crash or a quit window cut short keeps its ledger row but has no Resume in the next window,
  since its launch's choices lived in the old one, and `claude --resume` in a terminal is today's
  way back. (1) **Resume in VS Code…** on each ledger card of the Sessions pane's All tab whose
  row has a session id, beside its Revert… (`ledger_line`): a headless run whose tab is gone, a
  visual or a manual session. It opens the project's own workspace file and hands the extension
  `vscode://anthropic.claude-code/open?session=<id>` as a visual launch hands `prompt=`
  (`visual::link`, `visual::open` with its wait for the lock file), escaped as EP-11 found.
  (2) **The first check**, on the owner's yes, by one supervised launch through a throwaway driver
  (D-29): whether the extension opens a headless (`-p`) run's id, whose transcript lies under the
  project's slug as any session's does, or only the sessions its own list offers; where it
  refuses, the card offers the resume for visual and manual rows only and says why on a headless
  one. (3) **The ledger**: a visual launch's row is taken up only by a new conversation (D-26's
  EP-13 addendum), so a resumed session needs a rule of its own (a row per run under the same id,
  as a headless resume writes, or none), the brief's choice. (4) Disabled, its reason on hover,
  when the folder is missing or the session still runs, in the window or seen working elsewhere.
  Tests inject the process starter as EP-11's do; screenshot runs show the cards with both
  buttons in both themes; a D-16 addendum. The headless resume after a restart (the launch's
  choices kept in a schema 5) stays in the v2 notes. **From EP-15a:** a card that takes a click
  keeps `style::unselectable` on its `Ui`, and `testing::Screen` presses a card's button.
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
  log tile section (what ran, why it stopped). **From EP-9:** a cap stops a run with
  `Session::stop()` (the interrupt, then the launch's `stop_timeout` before the kill); CLI 2.1.316
  lists `--max-budget-usd`, passed when `Launch::max_budget_usd` is set. **From EP-10:** the commit
  variant replaces rule 5 of `prompt::session_rules` ("Stop before committing"); a chained run is
  drafted and launched as the dialog does it (`prompt::draft`, then the `session::Launch` that
  `launch::Dialog::launch` builds), and the window already holds and drains its sessions
  (`app::sessions`). **From EP-11:** a chained run is headless whatever the project's launch mode
  (a visual launch waits for the owner to send it), so the chain sets the mode itself rather than
  take `Resolution::mode`, whose default is Visual. **From EP-13:** the Commit button is a Resume
  whose message is the commit instruction: `Sessions::resume(key, preflight, message)` runs the ended
  session again under its id from the `Launch` it kept, as a ledger row of its own, and
  `Board::resume_session` already says the preflight's refusals. **From EP-14:** a chained run's
  questions wait in the Decisions tile like any headless run's; a project's `answer_timeout`
  (minutes, `crw set <project> answer_timeout`) defers a run whose request waits that long, its
  outcome `deferred`, so the gate's "outcome completed" fails and the chain stops there until the
  owner's answer resumes the run (`Sessions::answer_open`); `Sessions::resume` now takes the route
  (`Sessions::route_for`), and a chained start sets `Launch::route` and `Launch::hooks` as
  `Board::start_headless` does. **From the owner's requests (2026-11-03):** a chained run that
  fails or that a cap stops ends the chain (D-22) and leaves its work in the tree; EP-15b's Revert
  takes the project back to that run's start on the owner's click, and the autopilot never reverts
  on its own (EP-15b's D-11 addendum). The brief decides whether a revert also turns the project's
  autopilot off, since the next chained brief would start from the restored state. **From
  EP-15:** the tray's menu holds Pause autopilot, disabled (`tray::entries`, id `pause`), for an
  `Order` of its own to carry out; a chain's run toasts its waits and its end as any window's run
  does, so a chain that stops (a gate refused, a cap) may want a moment of its own. **From EP-16:**
  three facts the gate and the caps meet. (1) A headless run is the window's child, and this
  project's own briefs end with a release build that needs the window quit (CLAUDE.md rule 6),
  which ends the run: a chained run here cannot finish its brief as a visual session does, so the
  brief decides with the owner how a chain of this project ends (the release build and the install
  script left to an owner task after the chain, the gate refusing a brief that changes the tool's
  own code, or a release build the running window does not hold, EP-28's install script). (2)
  Neither this project nor the user settings hold an allow rule, so under `acceptEdits` a headless
  run waits for the owner at its first shell command: a chain needs a project's `permission_mode`
  of `auto` (D-18's override; the CLI's help lists it) or the project's own allow rules, and the
  gate can name a project whose runs would wait. (3) P2's M's took 37 to 76 minutes of active work
  but up to 2 h 45 min of wall time with the owner's answers and trials (D-4 addendum); the caps
  count the wall clock, so a run that waits in the Decisions tile can reach its cap while it
  waits, and the brief decides whether a cap pauses while a request is open. EP-35's Resume in VS
  Code picks up a chained run that a crash cut short. **From EP-17:** the gate's weekly and Fable
  figures are `usage::Gauge::figures(now)`, each with its `at` and `source`. While the endpoint
  fails (`Figures::degraded`), weekly may come from a status line and Fable is the last reading's,
  so the brief decides how old a figure the gate trusts and whether a missing Fable refuses the
  chain. A limit's `resets_at` is the pause's resume time. The notifications' marks stay 80 % and
  95 % (`usage::WARN`, `usage::ALARM`); the pause at 90 % is the chain's own rule. **From EP-18:**
  the downshift is the resolver's last layer: `prompt::draft` takes a `rules::Usage`, which the
  window builds from its meter (`rules::Usage::of(&meter.gauge.figures(now), now)`). So a chained
  run drafted that way is downshifted with no code of its own, and its log can say so from
  `Resolution::downshift`. The mark is the registry's `fable_downshift_percent` (60 by default);
  a figure above it counts whatever its age. A figure counts until its reset
  (`Limit::before_reset`): the endpoint calls the Fable-scoped limit inactive while its window
  runs, so the pause must not read `is_active` (D-23 addendum).
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
  (`inbox::tick`, `crw tick <project> IN-n <disposition>`). **From EP-12 (the owner's answer at its
  close):** IN-2's `UserPromptSubmit` hook belongs here: it recognises `@<project>` in a prompt and
  adds that project's `CLAUDE.md` rules and next-EP row to the session (registry lookups, as the
  mailman's). `crw hook` is the relay to extend (exec form; `--inbox` already prints a start's open
  messages as written, `inbox::open_as_written`, in place of the owner's stopgap hook), and
  `signals::install` the installer to add the event to (the user settings' entries are known by
  the program they run). A visual session's end reaches the window as `SessionEnd` through
  `watch`, which can start the re-parse. **From EP-13 (the owner's answer at its close):** the
  prompt hook is this brief's to install, on the owner's yes after a dated copy; the state model
  already reads a `UserPromptSubmit` hook as busy and as work, and a headless run's row keeps its
  result text (`SessionRow::result`). **From the owner's requests (2026-11-03):** a message
  delivered into a project while one of its sessions runs is the tool's write, not the session's,
  so a revert of that session (EP-15b) must keep it: the delivery follows the rule EP-15b sets for
  the tool's own writes during a session (re-applied after the restore, or named in its preview).
  **From EP-15:** an open inbox message is a badge on the tray icon already (`toasts::marks`,
  `Kind::Inbox`), new until the window is next in front; an `urgent` one in the popup is one more
  rule of `crw_core::notify`'s table, and ntfy joins that module (`notify::ntfy`, as planned) with
  the same quiet hours; a delivery is an `event` row of its own kind (`Ledger::add_event`).
  **From EP-15a:** the Detail pane draws an open message and a handed-on block's text through
  `markup::note` with `Look { sentences: true, .. }` (a long one a sentence to a line, numbered
  parts as a list), which the compose dialog's preview can reuse. **From EP-16:** an open message's
  trailing `[urgent]` stays in its text in the Detail pane while its chip says it (EP-15a's note,
  placed here by the owner): the pane drops it from the text, as it drops an owner item's
  `[before EP-m]` gate.
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
  inbox files already write it. **From EP-33:** Adopt (`discovery::adopt`) already writes the
  folder's own workspace file and the aggregate's entry before it registers the folder, so the
  formalise step writes the house-style files only and its preview can show Adopt's as done.
- **EP-22.** `plugins/roadmap-wrangler/` with `.claude-plugin/plugin.json`, `skills/plan/SKILL.md` (the
  interview method as practised in this project's EP-0: research pass, domain map, rounds of four with the
  recommended option first, the planning file written to the repo first and filled per round, close-out
  with unpicked items), `skills/replan/SKILL.md` (retro, addenda, re-cut, pickup notes, v2 notes),
  `skills/formalise/SKILL.md` (the house-style bootstrap from EP-21's files and a first roadmap from a
  brief), `skills/status/SKILL.md` (read a project's roadmap the way the parser does and report next EP,
  counts, open items), and `reference/house-style.md` (the schema the parser expects, the forms of D-12 and
  D-13); tool-launched sessions get `--plugin-dir <repo>/plugins/roadmap-wrangler`; README notes for
  installing it into the owner's own sessions; `cli` verifies the plugin loads (`claude --plugin-dir …
  --help` smoke in CI is out; a structural test checks the files). **From EP-16:** a visual launch
  cannot pass `--plugin-dir` (the extension's link takes `prompt` and `session` only, EP-11), and
  Visual stays the default (D-16 addendum), so the plugin reaches most of the owner's sessions only
  once it is installed into their own Claude Code, which the README's install notes explain;
  `--plugin-dir` serves the headless runs and the chat. The extension's link also has an
  `/install-plugin` path (`vscode://anthropic.claude-code/install-plugin?plugin=<name>&marketplace=<url>`,
  2.1.318's handler, read at EP-16), which opens its plugins dialog in the chat for the owner to
  confirm: one way to bring the plugin into the owner's VS Code sessions.
- **EP-23.** Retro, addenda, P4 rows confirmed with pickup notes, `v2-notes.md` grown; the first week of
  autopilot runs reviewed (how many chains, how many gate refusals and why). **From EP-16:** time
  the briefs as EP-8 and EP-16 did: each session's transcript under this project's slug, found by
  the Write or Edit of its own `roadmap\EP-n-*.md`, from its first entry (or the previous EP's
  record commit, when one session did two) to the EP's record commit, gaps over five minutes left
  out; timestamps and file paths only. From EP-17 each self-launched session also has its ledger
  row (`crw sessions`), worth setting beside the transcripts, though a row runs to its tab's close
  and is no brief's time. Retake the README's two pictures of the fake portfolio with EP-8's method
  (EP-32's charter; the owner's choice at EP-16), once P3 has changed the window. Check egui#8470
  (open since 2026-08-29: an immediate viewport made while the window is minimized panics) and
  eframe's releases with the pins (D-15; the D-21 addendum's rule against immediate viewports), and
  claude-codes (2.1.318 at EP-16, against the pin 2.1.316) beside the CLI's version. **From EP-18
  (split at pickup, the owner's choice):** EP-18a joined P3, so the totals are 42 briefs, 12 S ·
  24 M · 6 L ≈ 42 session-hours, 17 still open once EP-18 is checked; the retro times EP-18 and
  EP-18a with the rest. EP-18 was started by hand, out of habit, and OT-14 walks the owner through
  the window's launch so that EP-18a starts from it.

## Phase P4 — Chat, analytics, exports, diagnostics (planned 2026-10-31)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-24 | Chat tile: hosted read-only portfolio session in DATA, digest system prompt, resumable conversations, delegation of adopt, formalise and compose actions, Opus default (D-27) | L | EP-14, EP-22 | core | ☐ |
| EP-25 | History backfill and snapshots: git import per project with Re-import, completion-note cross-check, daily parse snapshots (D-26) | M | EP-2, EP-4 | core | ☐ |
| EP-26 | Analytics: activity heatmap, burn-up per project, velocity and projection, inline row expansion, Expand all and Collapse all, the cards view (D-25, D-26) | M | EP-25 | core | ☐ |
| EP-27 | Exports: site `projects.json` and HTML snippet, daily JSON export into `private/`, CSV per project (D-8, D-24) | S | EP-25 | core | ☐ |
| EP-34 | Hosting and backup actions: Create private remote, Make public, Make private, Remove remote, Archive and Delete an orphan, Back up now, each behind a confirmation and logged (D-24) | M | EP-7 | core | ☐ |
| EP-28 | Operations: `doctor` and the Diagnostics tile, rolling log, install script final, start-at-login entry, single instance, AV hints, start-up reconciliation (D-29) | M | EP-15 | core | ☐ |
| EP-29 | Re-plan P4; P5 charters upgraded to full briefs | S | EP-24, EP-25, EP-26, EP-27, EP-34, EP-28 | core | ☐ |

Ordering rationale: the chat needs the decisions tile (its questions) and the plugin (its method); the
backfill feeds the graphs and the exports; operations polish lands before the phone view exposes the tool
beyond the desk. *(EP-8, 2026-11-01.)* EP-34 follows the exports, the other brief that writes outward,
and comes before operations.

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
  **From EP-7:** `crw status` prints counts and stack names, never paths, but a session runs it on
  named non-data projects only (D-24 addendum), so the digest takes the badges from the window's own
  readings rather than from a `crw status` over the whole registry. **From EP-14:** a chat run is a
  `-p` session like the runner's, so it needs `--permission-prompt-tool stdio` for its questions to
  reach the host (EP-14's probe: without it a `-p` run offers no `AskUserQuestion` and denies what
  would prompt); its questions can use the tile's form (`decisions::Draft`, `Sessions::respond`).
  **From EP-15a:** `markup` reads the house style's inline Markdown (bold, code told by its kind,
  struck text, links, ids, gates) and sets blocks (labels, numbered parts, blank lines) at a
  reading measure (`markup::note`, `markup::measure`); a reply's full Markdown (headings, lists,
  fenced code) is beyond it, so the tile needs a renderer of its own or a crate, whose pin is a
  re-plan's (D-15). **From EP-15a (the owner's answer at its close):** a brief's notes, written for
  the sessions that carry them out, are hard to read for what the brief plans (its references,
  its pickup notes); the owner left them so for the sessions' sake and expects to ask the chat
  tile what a brief plans, in plain words.
- **EP-25.** `core::history`: `git log --format=%H%x09%ct%x09%s` per project; EP ids from `record EP-n
  commit hash`, `(EP-n` or `(EP-n,` suffixes, `EP-n:` prefixes; completion dates per EP = the record
  commit's date, else the EP commit's date, cross-checked with the brief's completion-note date (the note
  wins when both exist and differ by more than a day, with a flag); hupsim-style single-commit histories
  use note dates only; `snapshot` rows (project, date, done, total) written daily and at every parse that
  changes the counts; Re-import per project from the detail tile. **From EP-13:** the ledger's rows
  before EP-13 hold sessions that never worked (a helper of the extension's, a restored tab: seconds
  long, several at the same second); from EP-13 a hooks-seen row is a session that worked. A lapsed
  visual launch has no session id, no outcome and no run length (`SessionRow::took` is `None`,
  `RowState::Lapsed`), and each run of a resumed headless session is a row of its own under one id.
  **From EP-14:** the runner writes `questions` (a question the defer route's hook ended the run
  at counts) and `answer_latency_s`, the mean seconds from a request to the owner's answer in the
  run, a deferred request's answer counted on the resume that carries it (`SessionRow`'s fields,
  worded by `ledger::asked`); rows before EP-14 have no latency.
- **EP-26.** `app::analytics` with egui_plot 0.38: a calendar heatmap (sessions and completions per day,
  per project and for the portfolio), a burn-up per project (done and total over time; re-plans visible as
  total jumps), velocity (EPs per week over the last 8 weeks) and projection (remaining session-hours from
  the size mix × measured median duration per size; projected finish at the measured weekly pace, shown as
  a range) in the detail tile and as the inline row expansion in the list; Expand all / Collapse all; the
  model and effort per EP from the new completion-note form shown in the history list. **From EP-8:**
  the cards view of D-25, which EP-6 left out with no destination (D-25 addendum): a Cards switch in
  the Projects toolbar, saved with the view, shows one card per project with its name, state chip,
  next brief and size, progress pill, badges and the burn-up sparkline, in the list's groups and
  order; if the brief outgrows its M it splits at pickup (D-4). **From the owner's requests
  (2026-11-03):** the cards and the inline expansion follow EP-15a's rule that a row's or a card's
  text never takes its hover and click; the measured duration per size leaves out, or marks, a
  session whose work was reverted (EP-15b's `revert` events). **From EP-15a:** that rule is
  `style::unselectable` on the card's or the row's `Ui`, as `list::cell` applies it to every cell
  of a row, and `list::tests::the_whole_row_answers_over_its_text` shows how to test it.
  **From EP-16:** the measured duration per size cannot be a ledger row's run length: a visual or
  manual row runs from its session's start to the tab's close (EP-15b's and EP-15c's read 20 h 50
  min and 18 h 8 min), and only the sessions that started while a window ran have a row. A brief's
  time runs from its session's start to the EP's record commit (EP-25's history), less the waits
  on the owner, which the state model's busy and idle (the hooks, the poll) can count without
  reading a transcript (D-26); the brief decides, and the re-plans' retros give figures to check
  it against. In a narrow pane the Progress section's long phase names run past the pane's edge,
  since an egui `Grid` does not wrap its cells (EP-15a's note, placed here by the owner): the
  rebuilt section wraps them. **From EP-17:** the usage samples are `usage_sample` rows (source
  `endpoint` every 5 minutes while a window runs, `statusline` when a session's figures change),
  read with `Ledger::samples_since`; the Usage pane's 7-day sparkline is drawn with egui's
  painter (`meter::sparkline`), which egui_plot may replace once the pins move.
- **EP-27.** `core::export`: `projects.json` (name, badge text, done, total, percent, next one or two
  undone rows, `as_of`) and `projects-cards.html` for projects whose remote is public, written into the site
  repository's `assets/data/` after the owner's yes (the site's own session wires it in; a `Handed on (to
  <site>)` message is created for that); the daily JSON export of registry and ledger into this repository's
  `private/export/<date>/`; CSV per project (sessions, snapshots) on demand. **From EP-7:** "whose
  remote is public" is `Hosting::class() == Class::Public`, from the account's list cached in the
  ledger's `badge_cache`.
- **EP-34.** Added at EP-8 (2026-11-01) for the actions D-24 decided and EP-7 parked (D-24
  addendum): Create private remote (`gh repo create <owner>/<name> --private --source <folder>
  --push`), Make public and Make private (`gh repo edit --visibility …
  --accept-visibility-change-consequences`, the dialog repeating `gh`'s warning), Remove remote
  (`git remote remove origin`; the GitHub repository is left alone), Archive and Delete for an
  orphan (`gh repo archive`, `gh repo delete`; Delete needs the `delete_repo` scope the token lacks,
  so the dialog says how to add it, `gh auth refresh -s delete_repo`), and Back up now (the job's
  script with `-Force`); each behind one confirmation dialog that names the command, run one child at
  a time under `process::turn()`, logged as an `event` row of the ledger (`kind`, `project`,
  `detail`, there since V1) and followed by a refresh of that project's badges; a failure is shown,
  never retried silently. Tests use an injected runner, never `gh` or the network (D-29). **From
  EP-7:** the state is read already (`badges::hosting::Hosting`, `badges::backup::Reading`).
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
  **From EP-7:** the backup job's settings name the target drive's label (`VolumeLabel`, not read
  yet), which the `D:` check can set against the drive's own; the last poll times include the
  GitHub cache's `fetched_at` (`Ledger::cached`). **From EP-9:** `crw doctor` prints the
  preflight already (logged in or not, headless ready or the reasons it is refused); the tile can
  show `session::Preflight` and the last handshake's answer. **From EP-11:** the extension leaves
  its lock files in `~\.claude\ide` when VS Code is killed (five of six were stale on 2026-11-02):
  Diagnostics may count them and never deletes them, being the extension's; it can also show where
  the visual launch finds `Code.exe` (`visual::locate`) and the extension's version. **From EP-12:**
  the entries' check is `signals::install::status` (`Status::summary(exe)`, `hooks_run`,
  `prints_inbox`), which `crw signals` prints with the window's `/health`; single instance probes
  that route. The hooks run `target\release\crw.exe`, so a release build that replaces it while a
  hook runs it is refused (access denied, rare: a hook lives about 20 ms); the shim IN-3 asks for
  decides what the entries run, and an install from the shim's path moves them (they are known by
  the program's name). The poll of `claude agents --json` is the start-up reconciliation's source.
  **From EP-13:** a manual or visual row whose end no window heard stays open (`RowState::Open`: "no
  end seen" in the tile, "open: no end recorded" in `crw sessions`; one from EP-12's own session at
  the time of writing), which the reconciliation closes. A migration step takes the ledger's write
  lock before it reads the version, so the window's threads and `crw` may open it at once; reads go
  through `Ledger::open_read_only`, which never creates or migrates the file. **From EP-14:** the
  tile can show the route the window's headless runs take (`Sessions::route_for`: the registry's
  `settings.decisions`, or a handshake refused in the window) and whether the preflight's help names
  `--permission-prompt-tool` (`Flags::prompt_tool`; without it a run's questions are disabled, as a
  dropped wish says); the defer route's hooks run the `crw.exe` beside the window
  (`board::crw_exe`), whose absence refuses that route, and an answer's file lives in
  `%LOCALAPPDATA%\clauderoadmapwrangler\answers\` only while its resume runs. **From EP-15:** the
  app's own icon replaces the tile `tray::draw` paints in the tray, and the window's default
  (eframe's, egui's logo), which `install-shortcut.ps1` already writes into the shortcuts' `.ico`
  beside the exe (the owner's ask, 2026-11-05); the status bar's hover already
  says how the notifications and the tray stand (`Toasts::line`, `Tray::line`), for Diagnostics to
  show; a start at login may open hidden in the tray (`Board::hidden`), and single instance can
  show the running window as the tray's Show does; the popup's corner assumes a taskbar at the
  bottom of the screen (`popup::BOTTOM`). **From EP-16:** the ledger has rows only for the
  sessions that started while a window ran (none for EP-13 … EP-15a's), which a start at login
  makes the rule rather than luck.
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
  identity, path and time; mobile layout at phone width. **From EP-12:** the routes join
  `app::server`'s router (axum with `http1`, `json` and `tokio` only); its guard refuses a request
  with an `Origin` or a `Host` other than the loopback, as a phone's request through Tailscale
  Serve may well carry, so the `/m/` routes take the identity check of D-28 in its place.
  **From EP-14:** the decisions page answers through the window's own calls
  (`Sessions::respond(key, request, Response)`, and `Sessions::answer_open` for a deferred run), its
  forms by `decisions::Draft`'s rules (one answer or several, the owner's own words, a refusal's
  reason) and its words by `decisions::answer_words`. **From the owner's requests (2026-11-03):**
  EP-15b's Revert is a desktop action; D-28's phone may read, answer, tick, start and pause, and
  whether it may also revert is EP-29's question for the owner.
- **EP-31.** `/m/session/<id>`: the transcript streamed over SSE with an input box for answers and
  messages to a headless session; the Enable remote view action: `tailscale serve --bg --https=9443
  http://127.0.0.1:<port>` after the owner's confirmation, `tailscale serve status --json` parsed into
  the Diagnostics tile, the URL and a QR code shown, a Disable action (`tailscale serve --https=9443 off`);
  a check that the phone's VPN does not block the tailnet (hint only). **From EP-15:** Enable
  remote view joins the tray's menu (`tray::entries`), as D-21 planned.
- **EP-32.** Timings for every EP of v1, the README's "State of the app" and screenshots refreshed,
  addenda, `v2-notes.md` compiled into themes (launch modes, analytics, chat, phone, hosting, conventions),
  the v2 direction (the first candidates: `--bg` background sessions with Attach, worktree-per-session,
  usage over time, free chat from the phone). **From EP-8:** the README's pictures come from
  screenshot runs whose home holds a registry naming a neutral data root (`data_root = "P:\\Projects"`,
  the letter mapped by `subst` onto a folder with a junction `Projects` to `crates/app/fixtures/portfolio`),
  so no personal path shows in the Detail pane; drop the mapping and the junction (`rmdir`, never a
  recursive delete) afterwards.

## Risks / open items

1. **The usage endpoint is undocumented** (D-23). A schema change or an auth change blanks the meter;
   the status-line feed is the documented fallback and the meter must say when it is degraded.
   *(EP-12, 2026-11-02.)* The feed is installed, but the VS Code extension most likely never runs
   the status line (its code names it only in its settings schema), so the fallback holds the
   figures of terminal sessions only.
2. **The stream-json control protocol is unspecified for non-SDK hosts** (D-19). `claude-codes` tracks it
   release by release; the defer-and-resume fallback is documented; EP-9 feature-detects from
   `system/init`. Every CLI update is a potential break — the Diagnostics tile shows the version against
   the recorded minimum and the last successful handshake. *(EP-9, 2026-11-02.)* CLI 2.1.316 answered
   the `initialize` handshake, named its capabilities and ran a supervised probe through the runner to
   its result; a line the adapter cannot read is an event, and a request it cannot read is refused,
   so a protocol change shows as events rather than a hung session. *(EP-14, 2026-11-02.)* A
   supervised probe found that the runner had never named itself the run's permission host: without
   `--permission-prompt-tool stdio` (the Agent SDK's form, named in 2.1.316's help only inside
   `--permission-prompts`' description) a `-p` run offers no `AskUserQuestion` and denies whatever
   would prompt. With it, the probe's question was answered through the protocol, a time limit
   deferred a run, and the defer route ended a run at its question and resumed it without a prompt;
   a CLI whose help stops naming the flag runs without a host, which the launch says.
3. **Two CLI versions on the machine** (standalone 2.1.261, extension-bundled 2.1.314). Visual and
   headless sessions may behave differently; the minimum version applies to the standalone binary.
   *(EP-8, 2026-11-01.)* The standalone binary updates itself (2.1.316 at EP-8, the version of the
   `claude-codes` pin) while the extension's stays at 2.1.314, so the gap moves either way: the minimum
   is a floor, and the version is read at every preflight. *(EP-16, 2026-11-04.)* It moved the other
   way: the extension carries 2.1.318 and the standalone binary is still 2.1.316, so a visual
   session may meet a release the runner has not.
4. **Hooks and status line in the owner's user settings** (D-20, D-23). Installed only after the owner's
   yes, with a dated backup; the relay must never slow a session (500 ms timeout, silent exit). An
   existing status line would be chained, not replaced. *(EP-12, 2026-11-02.)* Installed on the
   owner's yes with `crw signals install --inbox` (the stopgap start hook replaced), the file as it
   was kept in the backups folder; `crw signals remove` takes the entries out. A hook costs about
   75 ms while no window runs (Windows refuses a closed loopback port late, so the relay gives the
   connection 50 ms of its half second) and a few milliseconds while one does. Another status line
   is left in place and said, not chained (none was configured). *(EP-13, 2026-11-02.)* The owner's
   environment repository retired the stopgap script at its EP-2, so the copy of the settings from
   before the install would now restore a hook that runs a missing script: the install is undone with
   `crw signals remove`, never by restoring that copy.
5. **`--max-budget-usd` is an estimate and `--max-turns` may be absent** (D-22). Wall-clock caps are the
   reliable limit; the budget flag is passed only when advertised. *(EP-8.)* Still absent from
   2.1.316's `--help`.
6. **Concurrent sessions edit roadmaps live** (D-9). The parser snapshots HEAD plus status; a project with
   a dirty roadmap is shown as "session in progress" rather than re-counted every few seconds.
7. **Visual launch cannot submit the prompt** (D-16). The owner presses Enter; the ledger records the
   session as visual without a process handle; completion is seen through hooks. *(EP-11,
   2026-11-02.)* The link goes to the focused window, so Visual opens the project's own workspace
   file first and waits for the extension's lock file; a window opened from the folder rather than
   from its workspace file gets a second window beside it, and VS Code may ask once whether Claude
   Code may open such links.
8. **Remote Control stores transcripts on Anthropic's servers while connected** (documented). It is the
   owner's existing posture (`remoteControlAtStartup: true`) and not changed by this tool; noted for the
   wholly-private projects' own decisions.
9. **Worktree isolation of `--bg` sessions** would break roadmap edits in the main tree; `--bg` is parked
   (v2 notes) and would need `worktree.bgIsolation = none` per project.
10. **Antivirus** may flag the child-process spawner or the stub `claude` used in tests (D-29): spaced
    spawns, no burst loops, and the quarantine hint in Diagnostics.
11. **The repository is public** (D-2, D-3 addenda; EP-8). A push publishes at once: a private name or
    a machine detail in a commit cannot be taken back without rewriting history, and the commits
    before EP-8 keep the planning record's first wording. The guard is rule 3's check of each
    commit's added lines against the word list kept in `private/`.
12. **A revert discards a session's work** (EP-15b; D-11). Putting a project back to a session's
    start removes what the session did, and a wrong start would remove more. The guards: the start
    recorded by the tool with copies of the files uncommitted then, a preview of every commit and
    file before the owner's click, the work copied out of the project first, a refusal while a
    session runs there or once a commit is on the remote, and never a revert by the autopilot.
13. **The tray is Windows' to show** (D-21; EP-15). Windows 11 puts a new tray icon in the
    overflow by the clock until the owner pins it, and Windows' own toasts may be off (they are on
    the owner's machine), which is why the notifications are a popup of the tool's own. The popup
    sits at the lower right of the screen the window is on, above a taskbar at the bottom, and
    File, Send a test notification shows where it lands.
14. **A headless run lives in the window** (D-16, D-29; EP-16). Quitting the window, which this
    project's release build needs, or a crash (EP-14a) ends its headless runs, recorded as
    stopped. That is why Visual stays the default; EP-35's Resume in VS Code picks such a run up
    from its ledger row, and the autopilot (EP-19) decides how a chain of this project's own briefs
    ends.
15. **The ledger knows what a running window heard** (D-26; EP-16). A session that starts while no
    window runs gets no row, and a VS Code tab left open runs its row on for hours, so the ledger
    cannot yet time a brief: the re-plans time them from the transcripts, EP-26 decides the
    measure, and EP-28's start at login closes the gap in the rows.

## Decision record

Decisions live in [`../DECISIONS.md`](../DECISIONS.md) (D-1 … D-30); the planning record with the
verified facts, the research pass and the interview is [`planning-2026-10-31.md`](planning-2026-10-31.md);
the portfolio survey that names the owner's projects is in the git-ignored `../private/`.
