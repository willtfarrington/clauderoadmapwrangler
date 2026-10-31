---
ep: 0
title: The v1 planning interview
size: L
model: claude-fable-5-1
effort: xhigh
status: done
---

# EP-0 — The v1 planning interview

**Size:** L · **Depends on:** — · **Blocks:** EP-1 … EP-32 (every v1 brief is written from the roadmap
this EP produces)

## Context

The owner wants a Rust desktop application that keeps track of their development projects — the git
repositories under one `DATA\` folder, most of them run as session-sized roadmap briefs executed by Claude
Code — and mirrors their Claude Code workflow: one view of every project's next brief with its size, a way
to pick model and effort, a button that starts the session visually (the VS Code extension) or headless,
a checkable pane of inter-session owner tasks, a toggle that chains the next brief automatically when
nothing needs the owner, discovery of new projects into the single VS Code workspace, GitHub remote and
backup status with actions, coloured badges, sorting, collapsible heatmaps and completion graphs, a chat
window to Claude Code for portfolio-wide questions and project formalisation, an in-session decisions
pane, an auto-updating usage meter, and eventually a mobile-friendly view over the tailnet. The brief
asked for an analysis of what exists, proposed additional domains of consideration, and then "as many
clarifying questions as possible" before the roadmap is composed.

The repository was an empty folder, already added to the owner's `.code-workspace`. No code changes.

## In scope

1. A survey of the machine and the portfolio before any question was asked: sixteen folders, eleven
   roadmaps (eight in the current house style, three in the older work-packet style), git remotes and
   visibilities, the backup job, the notification helpers, Claude Code's installed versions and settings,
   the session state Claude Code keeps on disk, the VS Code extension's URI handler, a probe of the
   account usage endpoint, Tailscale's current Serve mappings, the Rust toolchain and the owner's egui
   pins (`roadmap/planning-2026-10-31.md` §1–2.5; the per-project table in the git-ignored
   `private/portfolio-2026-10-31.md`).
2. Three delegated research passes: the documented Claude Code surface (print mode, the Agent SDK's
   control protocol, hooks, the status line, sessions on disk, the IDE, Remote Control, cloud sessions,
   model and effort, settings); a read-only survey of how the eleven roadmaps vary (headers, checkboxes,
   sizes, row order, in-progress states, completion notes, decision logs, owner to-dos, commit patterns,
   existing checkers); and the Rust GUI stack options for "desktop now, phone later", the relevant crates,
   and the landscape of existing orchestrators and usage meters (§2.6–2.9).
3. A domain map of fourteen areas derived from the brief, plus one added by the owner during the
   interview (the cross-project inbox), written to the repository first (§3).
4. The interview: 18 rounds, 72 questions, four per round, the recommended option first, multi-select
   for feature menus, a close-out round confirming unpicked items (§4).
5. The outputs written from the record: `DECISIONS.md` (D-1 … D-30), `roadmap/README.md` (five phases,
   32 briefs, brief notes for P1–P4, charters for P5, risks), `CLAUDE.md`, `roadmap/v2-notes.md`,
   `README.md`, `.gitignore`, this brief; git initialised on `main`; one commit proposed to the owner.
6. The first message of the cross-project inbox convention, written by hand into another project's new
   `roadmap/inbox.md` with the owner's yes (a flag that project's runner decision relies on is missing from
   the installed CLI).

## Out of scope

- Any code, the Cargo workspace, the GitHub remote (EP-1).
- Migration of the three older-format projects (a chat-pane task per project after EP-21).
- Decisions about other projects beyond the one inbox message.

## Verification / acceptance

- Every question's answer appears in the planning record's §4 with its round, and every decision in
  `DECISIONS.md` cites the facts it rests on; the roadmap's phase tables parse under the house style
  described in this project's own parser notes (six columns, `☐`, `EP-n` ids, `**Brief notes.**` bullets).
- The owner signed off on breadth of function and the security model (round 18).
- `git status` shows `private/` untracked and ignored; no tracked file names a private project.

## The interview (2026-10-31)

Rounds and their outcomes are recorded in `planning-2026-10-31.md` §4; in brief: 1 scope and identity ·
2 purpose and cadence · 3 project model (owner note: the inbox) · 4 conventions (owner added YAML front
matter) · 5 the cross-project inbox · 6 architecture · 7 launch and runtime · 8 decisions and attention
(owner added ntfy) · 9 autopilot · 10 usage meter · 11 hosting and backup · 12 views (owner added health
badges) · 13 history and analytics · 14 chat and method · 15 remote access and security · 16 operations
and quality · 17 shape of v1 (unpicked items parked: "none of the above for now") · 18 close-out (one
commit, no remote yet; the inbox message to the other project; EP-1 next in a window on this folder;
sign-off). Every recommended option was taken.

> **Completion note (2026-10-31; fable/xhigh; ≈ 2 h 45 min from the survey to the documents).** Executed
> in one session as planning only: no code. Owner decisions (`AskUserQuestion`, rounds 1–18): all 72
> answers recorded in `planning-2026-10-31.md` §4 and distilled into D-1 … D-30; four additions beyond the
> recommended options (the cross-project inbox → D-13; YAML front matter → D-12; ntfy → D-21; health badges
> → D-25); four unpicked items parked in `v2-notes.md` (usage-over-time graph, portfolio report, free chat
> from the phone, `--bg` launch mode). Close-out: one commit in this session with no remote (the private
> remote is EP-1's, D-2); the orphan private GitHub repository of a wholly-private project left as is and
> flagged (D-24); the first inbox message written by hand into the other project's `roadmap/inbox.md`
> (D-13); EP-1 next, started by hand in a VS Code window opened on this folder alone (D-16). Routine
> choices made by the session: `main` as the default branch (the GitHub repositories use it), `.gitignore`
> with `/target/`, `/private/` and editor noise, the planning record split into a tracked generic file and a
> private survey, the port 47831 and the HTTPS port 9443 as defaults to be confirmed at EP-12 and EP-31.
> Corrections made during the session: an early "remote ahead of local" reading was wrong (heads match;
> push dates differ), and the stack of one older project was mis-read as Python before the survey showed
> TypeScript — both fixed in the private survey. Facts to re-verify at implementation: whether the status
> line runs in `-p` sessions (EP-12), the exact `claude agents --json` fields for background sessions
> (EP-12), the hook timeout behaviour on a hard kill (EP-12), the backup job's run-record format (EP-7),
> the Tailscale Serve port rules on Windows (EP-31).

> **Handed on (to the owner).** EP-1 is started by hand: open VS Code on `clauderoadmapwrangler` alone and
> begin with the house-style opener (rule 1 of `CLAUDE.md`); the `gh repo create … --private` step waits
> for the owner's yes inside EP-1.
