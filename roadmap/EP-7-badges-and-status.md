---
ep: 7
title: Badges and status
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-7 — Badges and status

**Size:** M · **Depends on:** EP-6 · **Blocks:** EP-8

## Context

EP-6 left a slot in each row of the project list, a Hosting sort key and the stack and hosting
filters disabled until this brief. EP-7 fills them (D-24, D-25): what a project is built with, how
far it is, how lively it is, where it is hosted and whether that copy is current, whether its CI
passes and it carries a licence, and how old its last backup is. The readings come from the
project's files, from git, from `gh` and from the owner's existing Documents backup job; the slow,
networked ones are refreshed daily and on demand and cached in the ledger with their fetch time.

**Session start (2026-11-01).** Started from `DATA\clauderoadmapwrangler.code-workspace`. The inbox
holds nothing open (IN-1 … IN-3 are triaged); OT-1 (the window on the real portfolio, `before
EP-7`) is ticked in the working tree by the owner.

**Verified at pickup.**

- git 2.56.0 prints `git status --porcelain=v2 --branch -z` with `# branch.oid`, `branch.head`,
  `branch.upstream` and `branch.ab +a -b` headers before the entries, so one status child gives the
  roadmap's changes (EP-4), the dirty count, the branch and ahead/behind; `--no-optional-locks`
  keeps it from refreshing the index, so the reading writes nothing into the project.
- gh 2.103.3 is logged in (scopes `repo`, `workflow`, `read:org`, `gist`; no `delete_repo`).
  `gh repo list --limit 200 --json name,visibility,isArchived,pushedAt,defaultBranchRef,owner`
  answers for every repository of the account in one child, which serves both the hosting badges
  and the orphan check; `gh run list -R <owner>/<name> --limit 1 --json
  conclusion,status,createdAt,workflowName,headSha` gives the latest Actions run.
- `git ls-remote origin HEAD` answers with the remote's head; this repository's equals its local
  `HEAD`. A remote's URL is in `.git/config` (`[remote "origin"] url = https://github.com/…`).
- The Documents backup job keeps its settings in a PowerShell data file (`Target`, the volume label,
  `Sets` with one unit per folder of the data root through `Each`, `ExcludePaths`) and appends one
  JSON line per event to a run log (`ts`, `run`, `event` of `run-start`, `written`, `unchanged`,
  `warn`, `error`, `run-end` with `status`, `unit`, `archive`); it writes
  `<Target>\<set>\<folder>\<yyyy-MM-dd HHmmss> <folder>.zip` and keeps the newest archive of each
  unit. On 2026-11-01 the log held 239 lines (51 KB), the last run ended at 12:31 with every
  project written, and the target drive was mounted.
- The fake portfolio has no repository and no remote, so the screenshot runs need their git and
  GitHub readings from a fixture, as EP-6 noted.

## In scope

1. **Git plumbing** (`core::git`): the porcelain v2 status read once per repository (branch, its
   upstream, ahead and behind, every changed path), replacing the roadmap-only status child, so the
   snapshot's `GitStatus` (EP-4) is derived from it and the git pass gains no child; the origin's
   URL read from `.git/config`; network children (`ls-remote`) run with `GIT_TERMINAL_PROMPT=0` and
   `GCM_INTERACTIVE=never`, so a missing credential fails instead of prompting.
2. **Stack chips** (`core::badges::stack`) from the manifests at the root and one folder down:
   `Cargo.toml` → Rust (egui when the GUI family is a dependency), `pyproject.toml` → Python with
   the version pin (`requires-python`, else `.python-version`; uv when `uv.lock` exists),
   `package.json` → TypeScript or Node with the manager (`packageManager`, else the lockfile),
   `docker-compose.yml` / `compose.yaml` → Docker, `*.ps1` at the root or in `scripts/` →
   PowerShell, `index.html` at the root → static site.
3. **Progress** from the parse (done, total, stretch, percent) drawn as a small bar in the row.
4. **Activity** (`core::badges::activity`): the changed-path count, the branch against its upstream
   (ahead, behind, as git counts them since the last fetch or push), the last commit's age and a
   staleness tier (fresh under 7 days, quiet under 60, stale after: D-6's 60 days).
5. **Hosting** (`core::badges::hosting`): the origin read as a GitHub repository (https and ssh
   forms) and matched in the account's list: public, private, internal, archived; *missing* when
   the account lacks it; *local* without a remote; *orphan* when a local-only project shares its
   name with a repository of the account; the head check, `git ls-remote origin <upstream branch>`
   against the local `HEAD`, with `git rev-list --left-right --count` when they differ: in sync,
   ahead, behind, diverged, or *not fetched* when the remote holds commits the clone has never seen.
   The tool never fetches (a fetch writes into the project's `.git`, D-11).
6. **Health** (`core::badges::health`): a `LICENSE*` file at the root; for a GitHub repository with
   `.github/workflows/*.yml`, the latest run's workflow, conclusion and time.
7. **Backup** (`core::badges::backup`): two optional registry settings, `backup_settings` (the job's
   settings file) and `backup_log` (its run log), relative to the data root or absolute; a reader
   for the PowerShell data file; the project's unit from the set whose `Each` folder holds it; its
   newest archive on the target (the time in the archive's name), the unit's last `written` or
   `unchanged` event and the last run's end from the log's tail; whether the target drive is
   mounted; the project's excluded paths; stale after 48 hours without a check (the job runs
   daily). Nothing is written to the job's files.
8. **The cache** (ledger migration 2): a `badge_cache` table of the networked readings (the account's
   repository list, each project's head check and latest run) with their fetch time; read at start so
   the badges show at once, refreshed when older than a day (checked at start and every hour while the
   window is open) and on F5, one child at a time with the spacing (D-29).
9. **The window**: a badge column between the counts and the session status in a full and a compact
   form, which gives way as the row narrows (full, then compact, then none, after the counts); a
   Status section in the Detail pane with every reading, its source and age; the Hosting sort key
   (public, private, other remotes, local; unknown last) and the stack and hosting filters, saved
   with the layout; the status bar counting the GitHub pass.
10. **`crw`**: `crw status [<project|folder>]` prints the badges of every registered project (or
    one), reading only, as `crw scan` does; `crw setting <key> [<value>]` prints or sets a registry
    setting (`data_root`, `workspace_file`, `backup_settings`, `backup_log`) through the store.
11. **Screenshots**: manifests and licence files added to the fake portfolio; a fixture of synthetic
    git, GitHub and backup readings, applied relative to the run's own clock in screenshot runs
    only, which never start the GitHub pass.
12. **Tests**: the v2 status, the remote URLs, the account list, the head check (a scratch repository
    with a local bare `origin`, no network), the stack chips, the data-file reader, the log's tail,
    the archive names, staleness, the ledger cache and its migration, the hosting sort and the
    filters.

## Out of scope

- D-24's actions (Create private remote, Make public or private, Remove remote, Archive and
  Delete for an orphan, Back up now): writes outward or into the job, each behind a confirmation;
  not in this brief's note, so their home is put to the owner at the close.
- Notifications for a stale backup or an orphan (EP-15); the usage-share badge and its sort key
  (EP-18); the target drive's volume label and the hook checks (EP-28's Diagnostics).
- Fetching, pulling or pushing; any write into a project or into the backup job's files.

## Verification / acceptance

- The gate of rule 6 passes with no warnings, `install-shortcut.ps1` last.
- Unit and fixture tests as in scope 12; no test calls `gh` or the network, and none runs the
  `claude` CLI (D-29).
- `crw status` read against the owner's registry (it reads only, like `crw scan`) and set beside
  `gh repo list`, `git status` and the backup log: visibility, ahead and behind, the orphan, the
  CI runs and the backup times agree; the private names it prints stay out of tracked files.
- Screenshot runs on the fake portfolio, each picture looked at: the badges in both forms and both
  themes, the Detail pane's Status section, a hosting sort and a stack filter.
- No private project name in the tracked files (`git grep` for the names kept in `private/`).

> **Completion note (2026-11-01; opus/max; ≈ 50 min to the owner's questions).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`; the inbox held nothing open and the owner had ticked
> OT-1. Owner decisions (`AskUserQuestion`, every one the recommended option): **data
> repositories**: the whole-tree status keeps running in every registered repository with git's
> output shown only as counts, and this project's sessions run `crw status` on named non-data
> projects only (D-24 addendum); disclosed in the question: this session had already run `crw status`
> once over the whole registry, which printed counts, stack names and hosting for the three data
> repositories; **D-24's actions**: parked in EP-8's brief notes for the re-plan to place;
> **backup settings**: set in the owner's registry by the session with `crw setting` (two lines
> added, after the day's copy `registry.2026-11-01.toml` in the backups folder); **addenda**: D-24
> and D-25 recorded as written; **commit**: both commits in the house pattern, pushed to the
> private remote. Routine choices made by the session: one porcelain v2 status child per repository
> replaces EP-4's roadmap-only one, the snapshot's `GitStatus` derived from it and
> `--no-optional-locks` keeping it from refreshing the index, so the git pass gained no child; the
> origin read from `.git/config` with no child; one `gh repo list` for the whole account rather than
> a `gh repo view` per repository; the head check as `ls-remote` plus `rev-list`, with *not fetched*
> when the origin's commit is unknown to the clone; `process::turn()` as the one lock every thread's
> children wait on (the app's own git mutex retired); a reader for PowerShell data files
> (`badges::psdata`) rather than a `pwsh` child for the job's settings; the run log read from its
> last 512 KB; times in the cache as RFC 3339 text through a small serde helper rather than chrono's
> `serde` feature, so `Cargo.lock` did not move; failed readings shown and not cached; the window
> looks at the cache's age hourly (an idle window wakes every 10 minutes); the progress drawn as a
> pill, since the first screenshot's bar read as a link's underline; the licence warning only for a
> public repository; the fake portfolio gained manifests and licence files, and `screenshot.rs` a
> table of synthetic git, GitHub and backup readings (a screenshot run starts no GitHub pass and
> keeps its ledger in its home); setup flags `sort=`, `stack=`, `hosting=`; `crw setting` with the
> new keys `backup_settings` and `backup_log`; the README's status and commands brought up to date.
> Verified: `cargo test` 155 passed (30 new), 157 with the feature (30 new), clippy with and without
> the feature and `cargo fmt --check` clean, the release build through `install-shortcut.ps1`, which
> refreshed both shortcuts; `crw status` against the owner's registry (27 s) agreed with `gh repo
> list` (9 repositories, each visibility, the orphan), with git (every origin in sync, this
> repository's uncommitted work counted) and with `gh run list` (this repository's CI at 21:31 UTC);
> the backup reading on a scratch registry of four non-data projects agreed with the job's log (the
> last run ended 12:31 local, ok) and with today's archives on the target; five screenshot runs on
> the fake portfolio, each picture looked at: the compact form dark at 1600 × 1000 (harbor, then
> compass), the full form light at 2300 points (meadow), the hosting sort with the Rust filter dark
> (beacon), quarry light; the first found the underline-like bar and a "1 commits", both fixed. No
> private project name in the tracked or new files (two of the session's own strings that echoed
> the job's layout were made generic; EP-0's planning record still names the drive's label for
> EP-8's audit). Owner task OT-2 (the window on the real portfolio; its first start creates the
> ledger). No messages for other projects. Next: EP-8 (the P1 re-plan), with EP-7's pickup notes in
> its brief notes and in those of EP-9, EP-15, EP-17, EP-18, EP-24, EP-27 and EP-28.
> **After the commit (≈ 60 min in all):** `47a2199` and the record commit `c33cd3c` pushed together
> at 22:50 UTC; CI run 41359159353 on windows-latest passed in 2 min 20 s (format, clippy with and
> without the screenshot feature, the tests without and with it, the head check's bare-origin
> repositories included). This line is a third commit, on the owner's yes, as at EP-5 and EP-6.
