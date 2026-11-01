# v2 notes — parked at the v1 planning interview (2026-10-31)

Everything named during planning and not built in v1. Every re-plan EP grows this file; the v1 close-out
(EP-32) compiles it into themes with a v2 direction.

## 1 Launch modes

- **Claude's own background sessions (`claude --bg`) as a third launch mode** with Attach in a Windows
  Terminal tab, `claude logs`, and the supervisor's `claude agents --json` state; requires `"worktree":
  {"bgIsolation": "none"}` in each managed project's `.claude/settings.json` so roadmap edits stay in the
  main tree; such sessions are interactive, so Remote Control reaches them from the phone (round 7, round
  17: not picked).
- **Worktree per session** and **diff review** inside the tool (recurring in other orchestrators; the
  owner's process edits the main tree and reviews in VS Code).
- A Windows Terminal tab running `claude --ide` as a visible CLI attached to VS Code (an alternative
  visual launch).

## 2 Analytics

- **Usage over time**: weekly and Fable percentages charted from the samples with per-project share
  (round 13, round 17: not picked). The samples are recorded from EP-17, so the graph is cheap later.
- Parsing transcripts for token counts per project (the approach of existing usage tools; the format is
  declared internal and changes between releases) — rejected for v1 (D-23, D-26).
- Transcript copies kept beyond the 30-day sweep (rejected, D-3).

## 3 Chat

- **Portfolio report**: a Markdown report across projects written into `private/` on request (round 14,
  round 17: not picked).
- A full-tool chat session in `DATA` (rejected for blast radius, D-27).

## 4 Phone view

- **Free chat from the phone** (round 15, round 17: not picked; a lost device would reach a Claude Code
  session).
- An eframe-wasm phone view served by the same server (egui 0.36 improved mobile IME; the
  keyboard-covers-field issue #4572 was open on 2026-10-31).
- A PIN for write actions and periodic re-authentication (alternatives to D-28).
- A path mount under the other project's existing hostname instead of a new HTTPS port.

## 5 Hosting and backup

- Deleting the orphan private repository from the tool (needs the `delete_repo` scope; D-24 flags it with
  Archive and Delete actions and leaves the scope refresh to the owner).
- A second backup mechanism (rejected; the Documents job is the one, D-24).
- Health badges beyond the latest Actions run and licence presence (CodeQL, Dependabot alerts).

## 6 Conventions and other projects

- **Migration of the three work-packet projects and the five-column project** to the house style: a
  chat-pane task per project, executed by a session in that project with the prompt EP-21 generates
  (D-9).
- The owner's vault tooling plans a mirror row per DATA repository (checkboxes, decision count, recent
  commits) that overlaps this tool's parser; the vault could consume this tool's daily export instead
  (noted for that project's re-plan).
- Owner-tasks ticking for the legacy forms (they stay read-only, D-10).

## 7 Stack alternatives considered

- Dioxus 0.7 for one UI codebase on desktop and web; Leptos 0.8 with a Tauri shell; Tauri 2 with a
  TypeScript frontend; Slint 1.18; Iced 0.14 (D-14 chose egui plus server-rendered pages).
- Embedding Tailscale (libtailscale) instead of the running client's Serve (not needed).

## 8 Operations

- Documented AV exclusions for the exe (alternative to D-29's spaced spawns and hints).
- A Windows service for the core with a separate GUI (rejected, D-29).
- An opt-in live smoke test against the real CLI with a tiny prompt (rejected: tests spend no usage).
- Scheduled autopilot windows through the Claude desktop app's machine-local scheduled tasks (observed in
  the docs; not needed while the tool runs at login).

## 9 Views

- Drag-resizable columns in the project list, their widths saved with the layout (the alternative the
  owner did not pick for EP-7a, which sizes the columns to their content; IN-4, 2026-11-01).
