---
ep: 10
title: Prompt generator and launch dialog
size: M
model: claude-opus-5-5
effort: xhigh
status: done
---

# EP-10 — Prompt generator and launch dialog

**Size:** M · **Depends on:** EP-5, EP-9 · **Blocks:** EP-11 (Visual launch from the same dialog),
EP-13, EP-14, EP-18, EP-19

## Context

EP-9 built the runner; nothing starts it yet. This brief makes the launch: the house-style opener
generated from a project's snapshot (D-17), the model, effort and ultracode resolved from a rule
table with the reason for each (D-22), the permission mode from the project (D-18), and a dialog in
the window that shows all of it, lets the owner edit the prompt and choose, and starts a headless
session (D-16). The row's Start button and the S key, which EP-6 left answering "not yet", open it.
The tiles that show a session's transcript (EP-13) and answer its requests (EP-14) come later, so
the window holds what it starts and shows it plainly until then.

**Session start (2026-11-02, 11:58).** Started from `DATA\clauderoadmapwrangler.code-workspace`. No
owner task and no inbox message is open, so nothing is triaged. Two decisions that shape the work
were put to the owner first (`AskUserQuestion`): what the window does with a session it starts
before EP-13 and EP-14 (*hold it, show it, stop it*) and how a rule matches a title (*keywords*, as
D-22 words it, not the brief note's regex, which would add a crate).

**Verified at pickup.**

- `claude --version`: 2.1.316. `session::cli::Preflight` gives the version, the login and the
  help's flags (`efforts` low … max, no `ultracode`; six permission modes); `Session::start(&preflight,
  launch)` waits up to two seconds after the last start (D-29), so it runs off the UI thread, and
  dropping a `Session` kills it.
- The model catalog Claude Code caches in `~\.claude\cache\model-catalog\` holds one file per
  surface (`…-cc.json` for the CLI, `…-ccd.json` for the desktop and the extension; the file names
  carry account identifiers, so a fixture is synthetic): `version` 2, `fetchedAt` and `staleAt` in
  milliseconds, `catalog.surface`, `catalog.config.models[]` (`id`, `name`, `section` main or
  overflow, `runtime.effort_levels`, `runtime.default_effort`, `min_claude_code_version`) and
  `catalog.config.provider_alias_targets` (`fable` → `claude-fable-5-1`, `opus` →
  `claude-opus-5-5`, …). Twelve models today, four in the main section; Opus 4.6 and Sonnet 4.6 lack
  `xhigh`.
- The registry's `[project.launch]` has `mode`, `model`, `effort` and `permission_mode` (EP-2);
  every table is read with `deny_unknown_fields`. No `regex` crate is in the workspace or the lock.
- The projects' `CLAUDE.md` files open their read-first rule in two shapes: a list item `1. **Read
  first:** …` (most of them) or a section headed like "Read order for every working session" or
  "Load order for a cold session"; some projects have no `CLAUDE.md`, so the house default applies
  (the user-level house style, rule 1).
- This repository has no `.claude\settings.json`, so a headless run here asks the host before most
  shell commands: before EP-14 such a run waits at its first request.

## In scope

1. **The opener** (`core::prompt`, D-17), from the snapshot and the registry entry: the project's
   name and folder; the date, model and effort the session runs with (for its completion note); the
   read-first rule quoted from the project's `CLAUDE.md` (the item or paragraph that opens with
   **Read first**, or the section headed read order, load order or read first), else the house
   default; the next row (id, title, size, depends-on, core or stretch, phase) and its brief-notes
   bullet verbatim; its brief file when one exists (pick it up, do not rewrite it); the last
   completed brief; the open owner tasks and inbox messages as written, a task gated before this
   EP flagged, and the triage rule (D-13) when a message is open; the rules: write
   `roadmap/EP-N-<slug>.md` first with front matter, follow `CLAUDE.md`, owner decisions through
   `AskUserQuestion`, end with `> **Completion note (date; model/effort; duration).**` with owner
   tasks appended and `Handed on (to …)` blocks, **stop before committing**. A project with no
   next row has no opener, and says why.
2. **The template** (D-17): `[project.launch] template`, a string with placeholders (`{project}`,
   `{folder}`, `{date}`, `{model}`, `{effort}`, `{ep}`, `{title}`, `{size}`, `{depends}`, `{track}`,
   `{phase}`, `{note}`, `{read_first}`, `{brief}`, `{last_brief}`, `{tasks}`, `{inbox}`, `{triage}`,
   `{rules}`, and `{opener}` for the whole generated text); `{{` and `}}` for braces; an unknown
   placeholder is kept as written and reported.
3. **The rule table** (`core::rules`, D-22): `[[rule]]` at the top level and `[[project.rule]]` per
   project, each with `match` (`size`, `phase` by its id, `title` by keywords; every condition given
   must hold, none matches every brief) and `set` (`model`, `effort` or `effort_step`, `ultracode`),
   and an optional `name` shown as the reason. Resolution in layers, each field with its reason: the
   built-in default (`fable` at `xhigh`, D-22), the project's launch defaults, the first matching
   global rule, the first matching project rule. While the registry lists no global rule, the
   built-in table applies: an S brief one effort step lower (D-22). The permission mode is the
   project's `permission_mode`, else `acceptEdits` (D-18).
4. **The model catalog** (`core::catalog`): the cached files read (the CLI's surface preferred, the
   newest), the main section first, each model's effort levels and minimum CLI version, the aliases
   resolved; a built-in list when no file reads.
5. **The launch dialog** (`app`): opened by the row's Start button, the S key and a Start button in
   the Detail pane's next brief; the prompt editor (regenerated as the choices change until the
   owner edits it, then a Regenerate button) and Copy; the model, effort, ultracode and permission
   mode with the resolution's reason beside each, the efforts the model and the CLI both take, and
   ultracode only when the CLI lists it; the neighbour folders; the preflight, asked off the UI
   thread and kept an hour, with its refusals; warnings for a task gated before this EP, a model
   newer than the CLI and a session in progress; Headless (Visual disabled until EP-11).
6. **The launch and the window's hold on it** (owner decision 1): `Session::start` off the UI thread
   with the launch's EP, effort, ultracode, permission mode, neighbours and the ledger; the window
   keeps the session; its row shows a session in progress; the Sessions pane lists the window's
   sessions in plain lines (running, waiting for an answer, ended with its outcome, the last line of
   text, the wishes the CLI dropped) with Stop, Kill and, once ended, Dismiss; the Decisions pane
   lists a waiting request read-only; closing the window kills what runs (D-16 addendum).
7. **`crw prompt <project|folder>`**: the resolution with its reasons and the opener, printed; it
   reads and never launches.
8. **The screenshot setup** gains `launch=<project>`: the dialog open over a canned preflight (a
   synthetic help) and a tracked synthetic catalog; a screenshot run starts no session.
9. **Owner decisions** through `AskUserQuestion` (rule 7): the two above, the addenda, the edits,
   the commit.

## Out of scope

- Visual launch (EP-11); the sessions tile's transcript and Resume (EP-13); answering a request
  (EP-14); hooks and the loopback server (EP-12); the downshift rule (EP-18); the autopilot and its
  commit variant of the prompt (EP-19).
- Writing launch defaults, rules or templates into the owner's registry: the tool only reads them
  here, and the owner writes them by hand (rule 13).
- A `regex` crate (owner decision 2), or any pin moved (D-15).
- Launching the real CLI from this session: tests drive the stub (D-29), the dialog is checked on
  the fake portfolio (rule 5), and the owner's first launch from the window is an owner task.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`, `cargo
  build --release` with the app closed, then the install script.
- Unit tests: the read-first shapes and the default; the opener over a synthetic project (next row,
  note, brief, last brief, tasks with a gate, inbox with the triage rule, no next row); the template
  (placeholders, braces, unknown names); the rule table (matching, layers, steps at the ladder's
  ends, reasons, the built-in table, the registry's new keys and their refusals); the catalog
  (surface and age, sections, levels, aliases, the fallback); the dialog's launch built from its
  choices and its prompt kept once edited.
- Screenshot runs with `launch=harbor` at 1600 × 1000 in both themes and in a narrow window, each
  picture looked at; `crw prompt` on this repository and on a fixture folder.
- The commit's added lines pass rule 3's word check; the owner has reviewed every decision.

> **Completion note (2026-11-02; opus/xhigh; ≈ 40 min, all active).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`, from 11:58: the reading, the owner's first two answers,
> this brief, the code and its tests, the pictures, the records, then the owner's second round. No
> wait passed five minutes. Against M ≈ 1 h. The model and effort are the extension's selection as
> the model catalog cached it at the session's start.
>
> **What was built.** `core::prompt` (the opener from the snapshot: the read-first rule in its two
> shapes or the house default, the next row and its notes, the brief to pick up, the last completed
> brief, the open tasks with the due ones named, the inbox with the triage rule, five rules; the
> template and its placeholders; `draft`), `core::rules` (the rule table, its checks, the layered
> resolution with a reason per value, the built-in table), `core::catalog` (Claude Code's cached
> model catalog, the built-in list); the registry's `[[rule]]`, `[[project.rule]]` and
> `[project.launch] template`, and `Project::new`; `crw prompt`; in the window, the launch dialog
> (`app::launch`: the editor that follows the choices until edited, Copy, Regenerate, the pickers
> with their reasons, the neighbours, the preflight off the UI thread, the warnings, Start headless)
> opened by S, a row's Start button and the Detail pane's, and `app::sessions` (the window's hold on
> its sessions, their plain lines in the Sessions pane, what they wait on in the Decisions pane);
> the screenshot setup's `launch=`, a canned preflight and a synthetic catalog for every run.
>
> **Verification.** `cargo test` 218 passed (27 + 157 + 2 + 7 + 6 + 11 + 8), 220 with `--features
> screenshot`; both clippy runs and `cargo fmt --check` clean; `cargo build --release` with the app
> closed, then the install script. New tests: the read-first shapes; the opener over the synthetic
> `house` project; the template's placeholders, braces and unknown names; the rule table's layers,
> steps held at the ends, reasons, built-in table and refusals; the registry's new keys; the
> catalog's surface, age, sections and fallback; the dialog's launch, its prompt kept once edited,
> ultracode turned off for a CLI that does not list it; a session's events into its lines; and in
> the runner's suite, a drafted prompt (dashes, ellipsis, backticks) reaching the stub whole, with
> the model, effort and name on its command line and the EP in the ledger. Screenshot runs, each
> looked at: `launch=harbor` at 1600 × 1000 light (the first one showed the dialog taller than the
> window: a separator between the columns stretched to the modal's whole height, so the divider is
> now drawn by hand); `keys=S` on lantern at 1100 × 760 dark (the built-in rule's `high` with its
> reason; the footer's note moved above the buttons); the window at 2300 × 1000 with the Sessions
> pane. `crw prompt` on this repository (its read-first rule quoted, this brief found as written)
> and on the `house` fixture. `crw roadmap .` reads 30 decisions with 30 addenda, no problem. Rule
> 3's word list found nothing in the added lines or the new files.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-02).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | What the window does with a headless session before EP-13 and EP-14 | Hold, show, stop | `app::sessions`; D-16 addendum |
> | 2 | How a rule matches a title | Keywords | No `regex` pin; D-22 addendum; the regex parked in v2 notes §1 |
> | 3 | The four addenda | Accept as written | D-15, D-16, D-17, D-22 |
> | 4 | README, roadmap and v2-notes edits; `crw prompt` and Copy beyond the row | Keep as written | As drafted |
> | 5 | OT-4, the owner's look at the dialog | Keep, gated before EP-11 | `roadmap/owner-tasks.md` |
> | 6 | Closing the window kills the window's sessions | Accept; note it at EP-15 | EP-15's brief notes |
> | 7 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).** (1) Three core modules (`prompt`, `rules`,
> `catalog`) so that `crw` and the window share them; the dialog's state and the launch it builds
> stay in the app. (2) A relative effort is `effort_step` (D-22's "one step lower"), along the
> CLI's levels, held at the ends; a rule sets `effort` or `effort_step`, not both, and sets
> something. (3) The built-in table stands in only while the registry lists no global rule, so a
> written table replaces it whole. (4) The opener tells the session its date, model and effort,
> which its completion note needs. (5) A task is due when its gate is this brief or a row above it in
> table order. (6) The catalog's CLI surface is preferred, then the latest fetched; a file that does
> not read is passed over. (7) The dialog is a modal that Escape and Cancel close and a click
> beside it does not, so an edited prompt is not lost; the window's keys rest while it is open.
> (8) The preflight and the start each run on a worker thread; a refused start drops the preflight
> so the next asks again. (9) `crw prompt` resolves with the built-in effort levels and asks
> nothing of the CLI. (10) Every screenshot run gets the canned preflight and the synthetic catalog,
> so a picture never depends on the machine. (11) The Decisions and Sessions tabs count what they
> list. (12) `Project::new` for a project not in the registry (`crw prompt` on a folder, tests).
>
> No messages for other projects. **Next: EP-33** (per-project workspace files), then EP-11.
