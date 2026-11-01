# clauderoadmapwrangler

A Rust desktop dashboard and session launcher for a portfolio of software projects developed with
Claude Code as session-sized roadmap briefs. One window shows every project's next brief with its size
and badges, starts a session visually (the VS Code extension) or headless with a chosen model and effort,
routes a headless session's questions to a decisions pane, keeps the owner's inter-session tasks and a
cross-project inbox, chains briefs on autopilot within caps, meters the account's usage, shows hosting and
backup status and history graphs, hosts a chat to Claude Code, ships the roadmap method as a plugin, and
ends with a phone view over a Tailscale tailnet.

**Status:** early development. The Cargo workspace, `crw doctor` and CI (EP-1), the registry of managed
projects with discovery and `crw scan` (EP-2), the roadmap parser with `crw roadmap` (EP-3 the phase
tables, EP-4 the briefs, the decision log and the HEAD-plus-working-tree snapshot that tells a session in
progress), and the owner's tasks, the cross-project inbox and the older forms of the owner's to-dos with
`crw tasks` and `crw tick` (EP-5) exist; the read-only dashboard arrives at EP-6. The roadmap is
[`roadmap/README.md`](roadmap/README.md), the decisions [`DECISIONS.md`](DECISIONS.md), the session rules
[`CLAUDE.md`](CLAUDE.md).

## Building

Windows 11 with Rust 1.98 (MSVC) and, for the features that drive them, the Claude Code CLI, the GitHub
CLI `gh`, VS Code with the Claude Code extension, and Tailscale.

```powershell
cargo build --release                   # target\release\clauderoadmapwrangler.exe and crw.exe
pwsh -File scripts\install-shortcut.ps1 # rebuilds, then a desktop shortcut and a Start-menu entry
target\release\crw.exe doctor           # versions of claude, gh, tailscale, code and the extension
target\release\crw.exe scan             # the registry: projects, unadopted folders, proposals
target\release\crw.exe roadmap <folder> # a project's snapshot: the next EP, its brief notes and brief,
                                        # the decision log, whether a session is in progress
target\release\crw.exe tasks <folder>   # what waits on the owner: tasks, inbox, handed-on blocks
target\release\crw.exe tick <folder> OT-n         # tick an owner task
target\release\crw.exe tick <folder> IN-n <text>  # tick an inbox message with its disposition
```

The tool keeps its registry, a hand-editable `registry.toml`, in `%APPDATA%\clauderoadmapwrangler`, and
its ledger and the copies it makes before changing a file in `%LOCALAPPDATA%\clauderoadmapwrangler`;
nothing is kept in a project. The registry lists the folders of one data root (by default the folder
that holds this repository); `crw adopt` registers a folder and adds it to the VS Code workspace file,
`crw ignore` hides one, and `crw set` changes a project's state, priority or pin. Set `CRW_HOME` to a
scratch folder to try the commands without touching those folders.

The workspace has three crates: `crates/core` (the model and engines, no UI types), `crates/app` (the
desktop window, `clauderoadmapwrangler.exe`) and `crates/cli` (`crw.exe`, the console commands and, later,
the hook and status-line relays). The checks every change passes: `cargo test`, `cargo clippy
--all-targets --features screenshot`, `cargo fmt --check`. Tests never run the real `claude` CLI and never
spend usage.

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
the owner. Licence: [MIT](LICENSE).
