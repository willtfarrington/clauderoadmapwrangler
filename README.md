# clauderoadmapwrangler

A Rust desktop dashboard and session launcher for a portfolio of software projects developed with
Claude Code as session-sized roadmap briefs. One window shows every project's next brief with its size
and badges, starts a session visually (the VS Code extension) or headless with a chosen model and effort,
routes a headless session's questions to a decisions pane, keeps the owner's inter-session tasks and a
cross-project inbox, chains briefs on autopilot within caps, meters the account's usage, shows hosting and
backup status and history graphs, hosts a chat to Claude Code, ships the roadmap method as a plugin, and
ends with a phone view over a Tailscale tailnet.

**Status:** planning complete (EP-0, 2026-10-31); implementation starts at EP-1. The roadmap is
[`roadmap/README.md`](roadmap/README.md), the decisions [`DECISIONS.md`](DECISIONS.md), the session rules
[`CLAUDE.md`](CLAUDE.md).

Built for one developer's workflow. The roadmap house style the parser expects (phase tables, brief
notes, completion notes, decision log, owner tasks, inbox) is described in the roadmap's "How to use"
section and will move to `docs/` with the plugin at EP-22. The tool drives the user's own installed Claude
Code CLI and claude.ai login; it offers neither. Developed with AI assistance (Claude Code), reviewed and
directed by the owner. Licence: MIT (the `LICENSE` file is added at EP-1).
