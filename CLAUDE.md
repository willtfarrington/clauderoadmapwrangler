# clauderoadmapwrangler — session rules

1. **Read first:** `roadmap/README.md` (pick the next unchecked EP **in table order** and its **brief
   notes** under the phase table), `DECISIONS.md` (D-1 … D-30), the last executed brief's completion note,
   then `roadmap/owner-tasks.md` and `roadmap/inbox.md` (triage rule, D-13: roll a message in if it fits
   the brief and is at most S-sized; otherwise add it to a named later EP's brief notes or propose a new
   row, confirming with the owner through `AskUserQuestion`; or decline with a reason; write the
   disposition on the line and tick it). Parked work lives in `roadmap/v2-notes.md`; re-plans grow it.
2. **Write the brief before touching files.** Each EP session starts by writing
   `roadmap/EP-N-<slug>.md` from its roadmap row: YAML front matter (`ep`, `title`, `size`, `model`,
   `effort`, `status`), context, in scope, out of scope, verification / acceptance. It ends with a
   `> **Completion note (date; model/effort; duration).**` that records each owner decision and its
   outcome, followed by `> **Handed on (to <project>).**` blocks for messages to other projects (the
   tool delivers them from EP-20; before that the owner carries them) and lines appended to
   `roadmap/owner-tasks.md` for anything only the owner can do.
3. **Public repository, private data (D-3).** The published repository never describes the projects
   this tool manages. Names, paths and states of the owner's projects stay in the git-ignored `private/`
   folder or in the runtime registry under the user profile; tracked files use generic names or the names
   of projects that are already public; fixtures and recorded streams use synthetic names; screenshots show
   fake projects; never commit the registry, the ledger, exports, tokens or transcripts. The OAuth token is
   read from `~/.claude/.credentials.json` only by the usage poller at poll time, never stored or logged,
   and sent nowhere but `api.anthropic.com` (D-23). The remote is public since EP-8, so a push publishes
   at once: before each commit, check the staged diff's added lines against the word list in `private/`
   (the private names and the machine details: drive labels and backup paths, the plan's tier, devices
   and VPN, tailnet ports, security software, user names in paths), `git diff --cached -U0 | grep '^+' |
   grep -iFf private/public-check.txt` printing nothing; screenshots name a neutral data root (D-3
   addendum).
4. **Managed projects are read-only from here (D-11).** A session in this repository never edits
   another project's roadmap, briefs or `DECISIONS.md`. The tool itself writes only the files it owns in a
   managed project (`owner-tasks.md` ticks, `inbox.md`, the adoption files, the `.code-workspace` entry).
   Findings for another project go through `> **Handed on (to <project>).**`.
5. **Tests never run the real `claude` CLI and never spend usage (D-29).** The session runner is tested
   against the stub executable replaying recorded stream-json; parser tests use the synthetic fixtures;
   UI checks use the `screenshot` feature (`CRW_SCREENSHOT=<file.png>` saves the app's own frame with fake
   projects and exits; `CRW_SCREENSHOT_SETUP` for injected keys). Never capture the screen.
6. **Build and verify:** `cargo test`, `cargo test --features screenshot`, `cargo clippy
   --all-targets --features screenshot -- -D warnings` and `cargo clippy --all-targets -- -D warnings`
   (code only the feature uses is dead without it, EP-6), `cargo fmt --check`, then `cargo build
   --release` with the app closed (a running exe cannot be replaced). `pwsh -File scripts\install-shortcut.ps1` rebuilds and refreshes the desktop shortcut and
   the Start-menu entry; run it last. `target\release\crw.exe doctor` prints the tool versions.
7. **Owner decisions go through `AskUserQuestion`** before an EP is marked done: commits, scope changes,
   anything written into the owner's user settings (hooks, status line) or into another project, anything
   outward-facing (the remote, visibility, the site export), unresolved risks. Routine, reversible
   technical choices are the session's call and are logged in the completion note.
8. **Commits** are the owner's call (from EP-19 on, autopilot on for this project is their standing yes).
   Messages go through a scratch file (`git commit -F`), with no AI attribution trailers. Pattern:
   `<Title>: <summary> (EP-n, D-x)`, then `docs(roadmap): record EP-n commit hash`.
9. **Windows shell constraints** (see the global `~/.claude/CLAUDE.md`): create and edit files with the
   Write/Edit tools, keep Bash to single lines, no heredocs, no `python -` stdin scripts. Spawn child
   processes one at a time; no burst copy or delete loops (two AV products watch this machine).
10. **Pins (D-15).** `Cargo.lock` pins everything; the egui family (eframe, egui, egui_tiles, egui_extras,
    egui_plot) and the `claude-codes` protocol crate move only in a re-plan brief. The minimum Claude Code
    version lives in `core` and is raised only by a brief that needs a newer flag; features are detected
    from `claude --version` and the `system/init` capabilities, never assumed. The tool never passes
    `--bare` and removes `ANTHROPIC_API_KEY` from every child environment.
11. **Sessions for this project run with cwd = this folder**, started from
    `DATA\clauderoadmapwrangler.code-workspace` (the owner's single-folder workspace file since 2026-10-31,
    IN-1), so transcripts and memory file under this project (D-16); the other projects' roadmaps are
    reachable read-only through `permissions.additionalDirectories` in the untracked
    `.claude\settings.local.json` (rule 4). From EP-16 on, the EPs of this project are launched from the
    tool (D-5).
12. **Generic names.** Modules, types, settings keys and file names describe function (registry, parser,
    runner, mailman, meter), never a managed project.
13. **The registry and the ledger are the owner's records (D-8).** `%APPDATA%\clauderoadmapwrangler\registry.toml`
    (the owner's since 2026-10-31, EP-2) and `%LOCALAPPDATA%\clauderoadmapwrangler` (the ledger and the
    copies of the registry and of the workspace file) change only through the tool and only when the owner
    asks; never reset, regenerate or "clean" them. `crw scan` only reads and may run on them. Tests use
    temp folders. To try `crw` or the app, set `CRW_HOME=<scratch folder>`: the registry, the ledger and
    the backups then live there, and a hand-written scratch registry points `data_root` at a scratch data
    root with synthetic names.
