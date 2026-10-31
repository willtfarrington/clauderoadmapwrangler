---
ep: 2
title: Registry and discovery
size: M
model: claude-opus-5-5
effort: xhigh
status: done
---

# EP-2 — Registry and discovery

**Size:** M · **Depends on:** EP-1 · **Blocks:** EP-6, EP-9 (and through them every later brief)

## Context

EP-1 left the Cargo workspace (`crw-core`, the desktop crate, `crw-cli`), `crw doctor`, the gate and CI on
the private remote. This EP gives the tool its memory: the human-editable registry of managed projects and
settings (D-8), discovery of the folders under the data root with Adopt and Ignore (D-6, D-7), the hand-set
states with proposals from facts (D-6), the SQLite ledger's schema (D-8, D-26) and `crw scan`, which prints
the registry view. This repository is the first registered project (D-5).

**Verified at pickup (2026-10-31, 15:30).** The data root holds seventeen top-level folders (this
repository among them) and the owner's one `.code-workspace` file: three folders are not git repositories,
one git repository is not in the workspace file, one repository's `roadmap/README.md` is empty, and the
workspace file lists one folder outside the data root (`../<vault>`, not a project, D-6). The workspace file is tab-indented with LF line endings and no final newline; its older entries carry
`name` keys, its newer ones only `path`. No registry exists yet: `%APPDATA%\clauderoadmapwrangler` holds only
eframe's `data\app.ron` from EP-1's smoke run, and `%LOCALAPPDATA%\clauderoadmapwrangler` does not exist.
`toml_edit` 0.25.17 is already in `Cargo.lock` (through the GUI family), so format-preserving writes cost no
new crate; `toml` 1.1.8, `rusqlite` 0.41.1 and `chrono` 0.4.45 are pinned since EP-1 and unused until now.

**Inbox and owner tasks.** Neither `roadmap/inbox.md` nor `roadmap/owner-tasks.md` exists yet: nothing to
triage (rule 1).

**Session note.** Started again from another project's VS Code window with an `@clauderoadmapwrangler`
request, so the transcript files under that project's folder slug (rule 11 and D-16 ask for a window on this
folder alone; EP-0 and EP-1 ran the same way).

## In scope

1. **Locations** (`core::paths`). The registry and its migration backups in `%APPDATA%\clauderoadmapwrangler`;
   the ledger and the backups of changed files in `%LOCALAPPDATA%\clauderoadmapwrangler`; `CRW_HOME=<folder>`
   puts all three in one scratch folder, so a session can try the commands without touching the owner's
   files. The repository the tool was built from, and the default data root (its parent folder).
2. **Registry** (`core::registry`). `registry.toml`, format 1: `format`, `ignore` (folder names),
   `[settings]` (`data_root`, `workspace_file`), and `[[project]]` tables with `name`, `path` (a folder of
   the data root, or absolute), `state`, `priority` (1–5, default 3), `pinned`, `[project.launch]` defaults
   (`mode`, `model`, `effort`, `permission_mode`; EP-10 resolves them) and the `[project.autopilot]`
   placeholder (`enabled`; EP-19). Read strictly through serde (`toml`): an unknown key, a bad value or a
   duplicate name is an error that names the line, and nothing is written over a file the tool cannot read.
   Changed surgically through `toml_edit`, so the owner's comments, order and layout survive every change;
   each change re-reads the file first, checks that the result reads back, and writes atomically (temp file,
   then rename). Before the day's first change a copy goes to the local `backups\` folder (14 kept, the
   pattern of the owner's other app). A newer format is refused; an older one is copied byte for byte to
   `registry.v<n>.<date>.toml` beside the file before its migration steps run (none exist at format 1; the
   tests inject one). A missing file reads as the default registry, this repository its first project
   (D-5), and is first written by the first change.
3. **Discovery** (`core::discovery`). The data root's top-level folders (dot folders and files skipped),
   each classified: git repository (a `.git` folder or file) or not, with a roadmap (a non-empty
   `roadmap/README.md`) or not, listed in the workspace file or not; diffed against the registry into
   registered (found or missing), unadopted and ignored.
4. **Adopt and Ignore.** Adopt registers a folder of the data root (state `active` for a git repository
   with a roadmap, else `unformalised`; an ignored folder leaves the ignore list), then, when the workspace
   file does not list it yet, appends `{ "path": "<name>" }` to its `folders` after a dated copy in the local
   `backups\`. A small JSONC scanner (strings, both comment forms, trailing commas) finds the array; the new
   entry copies the last entry's line breaks and indentation and goes in right after it; every other byte
   stays, and the result is scanned again before the atomic write. Ignore adds the name to `ignore`.
5. **States and proposals.** The six stored states (active, paused, maintenance, done, archived,
   unformalised; *unadopted* is a discovery result and never stored); `set` changes state, priority and pin;
   `propose` turns facts into proposals the owner confirms: every row checked → done or maintenance; a
   pickup gate on the next brief → paused; no commit for 60 days → paused (the last two for active projects
   only). The last commit comes from `git log -1 --format=%ct`, one child at a time with the doctor's
   spacing, through the doctor's runner moved into a shared module; the other two facts need the parsers.
6. **Ledger** (`core::ledger`). `ledger.sqlite`: `schema_version` and the tables `session`, `usage_sample`,
   `snapshot` and `event` with the columns D-23, D-26 and the P2–P4 brief notes name; numbered migrations in
   a transaction; a newer schema refused. `open` creates the file; no rows are written before EP-9.
7. **CLI.** `crw scan` prints the registry view and never writes (the chat's read-only session runs it,
   EP-24); `crw adopt <folder>…`, `crw ignore <folder>…` and `crw set <project> state|priority|pinned
   <value>` are the console side of the core actions until the dashboard (EP-6).

## Out of scope

- Reading roadmaps (EP-3, EP-4), so the "every row checked" and "pickup gate" facts stay unknown here.
- The dashboard with its Adopt, Ignore and Confirm buttons, and watching the data root for new folders
  while the app runs (D-7) (EP-6); dismissing a proposal.
- The registry's later keys: the rule table (EP-10), the loopback port (EP-12), quiet hours (EP-15), the
  usage thresholds (EP-18, EP-19), ntfy (EP-20), the site export (EP-27), the owner's tailnet login (EP-30).
- Ledger rows (EP-9 on), the daily JSON export into `private/` (EP-27), doctor's integrity checks (EP-28),
  badges and activity (EP-7).

## Verification / acceptance

- The gate passes with no warnings: `cargo test`, `cargo clippy --all-targets --features screenshot`,
  `cargo fmt --check`, `cargo build --release`.
- Tests in temp folders only, with synthetic names, cover: the default registry; comments and layout kept
  through changes; atomic writes; the migration backup; a newer format refused and left untouched; the daily
  copy and its rotation; strict reading; discovery and the diff; the workspace append in the owner's layout
  and in the JSONC edge cases, byte for byte; listed-folder detection; Adopt end to end; the proposals; the
  ledger's schema and a newer schema refused.
- `crw scan` with `CRW_HOME` set to a scratch folder lists the real data root's folders, classified, with
  this repository registered and the rest unadopted, and writes nothing anywhere.
- `crw adopt`, `ignore` and `set` against a scratch data root and workspace file: the workspace diff is
  exactly the new entry, and its backup exists.
- No private project name in the tracked files (`git grep` for the names kept in `private/`).

> **Completion note (2026-10-31; opus/xhigh; ≈ 30 min to the commit).** Executed in one session from 15:29,
> started from another project's window (see the session note). Owner decisions (`AskUserQuestion`, every
> one the recommended option): **commit**: both commits in the house pattern, pushed to the private remote;
> **registry**: created on this machine at once — the thirteen other git repositories adopted with one
> `crw adopt` (one registry write; the workspace file gained the one repository it did not list, after a
> copy in the local `backups\`), the three non-git folders ignored, and the states of the morning's survey
> set with `crw set` (one maintenance, five paused, one done), after which `crw scan` showed fourteen
> registered, none unadopted and no proposals; **CLAUDE.md**: rule 13, the registry and the ledger are the
> owner's records and sessions try the tool with `CRW_HOME`; **hand-offs**: "From EP-2" sentences in the
> brief notes of EP-4 (the two parse facts) and EP-6 (the unadopted group, Adopt, Ignore, Confirm, the
> facts gathered off the UI thread, the data-root watcher of D-7). Routine choices made by the session:
> `toml` for the strict read with `deny_unknown_fields` on every table, and `toml_edit` 0.25.17 (already
> locked) for the writes, with the parsers' errors boxed for clippy; the layout `format` and `ignore` at
> the root, `[settings]` (`data_root`, `workspace_file` relative to it), `[[project]]` with the folder name
> as `path`, priority 1 (highest) to 5 with 3 by default, and `[project.launch]` (`mode` checked as visual
> or headless, the rest strings for EP-10) and `[project.autopilot]` optional and not written by default;
> `format` required; the migration copy named `registry.v<n>.<date>.toml` beside the file, the other app's
> pattern, since the notes' `registry.<date>.toml` names the daily copies (14 kept, an addition) in the
> local `backups\`; the default registry's first project and data root taken from the build's repository
> path; `crw scan` read-only, so the file is first written by the first change; `CRW_HOME` as the scratch
> override; the doctor's runner moved to `core::process` and reused for `git log -1 --format=%ct`, 250 ms
> apart; a roadmap counted only when `roadmap/README.md` is not empty; dot folders and files skipped;
> folder names matched in any case and stored as spelled on disk; Ignore refuses a registered project
> (archive it instead); Adopt writes the workspace file before the registry, so adopting again finishes an
> interrupted adoption; a JSONC scanner (strings, both comment forms, trailing commas, BOM, CRLF) whose new
> entry copies the last entry's line break, indentation and colon, and a dated copy before every workspace
> write; proposals of `paused` for active projects only, `done or maintenance` for active or paused ones;
> the ledger with a `schema_version` table, times as RFC 3339 text in UTC, a 5 s busy timeout and no WAL,
> its file not created by any command yet; `crw adopt`, `ignore` and `set` added beside `scan` as the
> console side until EP-6 (EP-24 plans `cli adopt`); the README's status, a `scan` line and a paragraph on
> where the tool keeps its files. Verified: `cargo test` 40 passed (30 new); clippy with `-D warnings` and
> `cargo fmt --check` clean; the release build in 34 s through `install-shortcut.ps1`, which refreshed both
> shortcuts; `crw scan` with `CRW_HOME` on an empty scratch folder read the real data root in 0.6 s
> (seventeen folders, this repository registered, sixteen unadopted) and wrote nothing there or in the
> profile folders; `adopt`, `ignore` and `set` on a scratch data root with synthetic names kept the
> hand-written registry's comments, changed the workspace file by exactly the new entry and refused a
> second adoption, an unknown state and ignoring a registered project; on the real registry, the
> workspace diff against its copy was exactly one entry and `crw scan` took 4.8 s, most of it the spacing
> between thirteen git children; ripgrep over this EP's files found no private project name, local path
> or machine detail. Found on the way: one repository's `roadmap/README.md` is empty, so a present file is
> not yet a roadmap; the pickup count was corrected to seventeen folders. No owner tasks; no messages for
> other projects. Next: EP-3 (Parser A), in a VS Code window opened on this folder alone (D-16).
