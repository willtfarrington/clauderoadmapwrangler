---
ep: 58a
title: A bare project name read as the registered project's before a folder of the working directory, in every crw command that takes a project or a folder; `.` read as its folder's registered project; the formalise skill's wording
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-58a — A bare project name read as the registered project's before a folder of the working directory; `.` read as its folder's registered project; the formalise skill's wording

**Size:** S · **Depends on:** EP-58 · **Blocks:** EP-59 (the P6 re-plan, which takes it into its
retro) · **Decisions:** D-11, D-9 (and D-3, D-29 as standing)

## Context

IN-17 and IN-18 (the sweep's sessions, 2026-11-25) met one bug: `roadmap_target`
(`crates/cli/src/main.rs`), which `project_folder` wraps, reads an existing file, then an existing
folder, both relative to the working directory, and a registered project's name only last. From a
project's root that holds a subfolder of the project's own name (`<name>/<name>`), a bare name
therefore reads the subfolder: `crw formalise <name>` previews the house files there and `--yes`
would write them there, `crw roadmap` and `crw tasks` find no roadmap, `crw status` prints the
registry's name and state beside the subfolder's readings, and `crw send <name>` would create
`roadmap/inbox.md` in the subfolder under the folder's own name. A `.` reads the root, but
`status` shows an empty name and no state, and `send` and `tick` log an empty name (they match
the registry by the folder as given, never made absolute). `crw history` already asks the
registry first, with a lookup of its own. The brief note under the P6 table settles the order
(the owner's choice at the triage of 2026-11-25).

**Session start (2026-11-25).** A headless run the owner started from the window at 18:16 (ledger
row `26fe2810`, EP-58a, headless, claude-opus-5-5/max; the chain log's 18:16 line sets the chain's
mark at 80 % of the session window, the owner's choice in its plan); the prompt is the tool's
opener, so no hand start to explain. S's cap of 1 h 15 min of working time from 18:16. Read
first: the roadmap's P6 table and EP-58a's brief note, `DECISIONS.md` (D-9, D-11), EP-58's
completion note, `roadmap/owner-tasks.md` (OT-40 … OT-43 open, none gated before EP-58a) and
`roadmap/inbox.md` (no open message). Field check since the triage's record commit (`bac7ddc`,
18:10): `crash.log` last written 2026-11-23 04:00, nothing new; the rolling log: the reads again
after the HEAD change (18:10), at the triage session's turn's end (18:11) and as the launch
dialog opened (18:15), then this run's start (18:16); `crw notifications`: the triage session's
wait and finish, nothing odd; `crw sessions`: this run open, the two manual sessions of the
afternoon with no end recorded (VS Code tabs left open); `crw autopilot`: not paused, six
projects on. Nothing to triage.

**Judged to fit S's cap, so not split.** One resolver of about eighty lines and its tests in
`crates/cli/src/main.rs`, the ten commands' lookups replaced by its answer, one paragraph of the
help, one paragraph of the formalise skill, a section of `docs/manual-changes.md`; no window
code, no list or pane, so no screenshot run (rule 5). If time runs short, `crw revert` keeps its
own lookup (its order is already the registry's first) and is proposed with the rest as a row.

## In scope

1. **One resolver**, `resolve_target(target, cwd, registry)`, the working directory passed in so a
   test never changes the process's own, giving the target (a folder, a README or a brief) and
   the registered project together:
   - an argument **not written as a path** (no `/` or `\`, not `.` or `..`, no drive or root) is
     a registered project's name first (the registry's own case-blind match); a file or folder of
     that name in the working directory is read only when the registry holds no such project;
     when both exist and are not the same folder, a `note:` line says that the registered
     project was read and how to name the other (`.\<name>`);
   - **a path** is made absolute against the working directory (`.` and `..` resolved) and a
     folder is matched to the registry by its place (`paths::same`), so `.` from a project's root
     reads that project with its name and state;
   - **a file** stays a brief (by its name) or a README, as today; a folder that a registered
     project lives in under another name is that project's.
2. **The commands take its answer** instead of looking the project up again: `roadmap`, `tasks`,
   `tick`, `untick`, `send`, `formalise`, `manual`, `status`, `prompt`, `history` (its own lookup
   goes), and `revert`'s fallback; `registered_name` and `roadmap_target` go.
3. **The formalise skill** (`plugins/roadmap-wrangler/skills/formalise/SKILL.md`, section 1): a
   session in the project may name it by its registered name or by its folder (`.` or the path),
   both reading the same project; "it is not on PATH, so call it by its full path" goes, stale
   since EP-28a's `crw path`. The plan and status skills' `crw … <folder>` lines are checked
   against the change.
4. **The help** says the order once, under the commands; `docs/manual-changes.md` gains the
   section (rule 14).

## Out of scope

- The window, which takes no argument; the registry's names; the commands that take a registered
  name alone (`set`, `sessions`, `autopilot`, `act`, `export csv`, `dismiss`).
- The manual's own text (EP-59's fold).

## Verification / acceptance

- Tests of the resolver over a temp working directory and a temp data root with synthetic names:
  a folder named like a registered project in the working directory (the registry read, the note
  naming `.\<name>`); the same name read from the data root itself (no note); `.` from a project's
  root (its registered name); a path to an unregistered folder (no name); an unregistered bare
  name with a folder, a README and a brief; a name that is nothing (refused); no registry (the
  working directory alone).
- `cargo test`, `cargo test --features screenshot`, both clippy runs and `cargo fmt --check`.
- A try under `CRW_HOME` with a scratch registry and synthetic project names: `crw status`,
  `crw roadmap`, `crw tasks` and `crw formalise` (preview only) with a bare name from a root
  holding a same-named subfolder, and with `.`.
- The word check of rule 3 on the staged diff prints nothing.
- The release build and the install run last, as the opener asks.
- **Residual risk.** No real session in a managed project runs the new build here (rule 5): the
  sweep's sessions meet it once the install lands and, for the skill's wording, once the owner
  updates the plugin after the push (an owner task). The resolver's rules are exercised by the
  tests and the `CRW_HOME` try on synthetic folders laid out as the sweep's projects are.

> **Completion note (2026-11-25; claude-opus-5-5/max headless; about 20 minutes of working
> time, 18:16 to the commits at about 18:36, the install after them).**
>
> **What was done.** Judged at the brief to fit S's cap, so not split; the fallback (`revert`
> left on its own lookup) was not needed. `crates/cli/src/main.rs` has one resolver,
> `resolve_target(target, cwd, registry)`, behind `resolve` (the process's working directory and
> the registry, `CRW_HOME`'s in a trial, the note printed) and `project_folder` (a folder and its
> registered project, a file refused): a word alone (one plain component, no `/` or `\`) is the
> registry's name first, by its case-blind match, and a file or folder of the working directory
> only when no project is registered under it; a path is joined to the working directory and
> normalised, and a folder is matched to the registry by its place (`paths::same`); a file is a
> brief by its name, else a README, as before. The eleven commands take its answer: `roadmap`,
> `tasks`, `tick`, `untick`, `send`, `formalise`, `manual`, `status`, `prompt`, `history` (its own
> lookup gone) and `revert`; `roadmap_target` and `registered_name` are gone, and a small
> `project_name` gives the registry's name or the folder's. The help says the order in one
> paragraph under the commands. The formalise skill's section 1 says that a project is named by
> its registered name from any folder or by its folder (`.` or the path), both reading the same
> project, that a name is the registry's before a same-named folder (with the note), and that
> `crw path on` puts `crw` on PATH, the stale "it is not on PATH" gone. `docs/manual-changes.md`
> has EP-58a's section (rule 14). OT-45 asks for the plugin's update after the push.
>
> **Owner decisions.** None arose: the order and the note were the owner's choices at the triage
> of 2026-11-25 (the brief note), and the choices below are the session's, technical and
> reversible. The commits are the autopilot's standing yes (crw's D-22), on in this project; the
> push, which publishes the skill's change and is the release of the plugin, is the owner's.
>
> **Session's calls.**
>
> 1. **The registry passed in.** The resolver takes the working directory and the registry as
>    values, so the unit test builds a registry over a temp data root (synthetic names: harbor,
>    lantern at `lantern-app`, meadow) instead of setting `CRW_HOME` in the test process; the
>    `CRW_HOME` route is exercised by a new end-to-end test in `crates/cli/tests/formalise.rs`,
>    which runs the binary from a registered project's root that holds a subfolder of its name
>    (the child's own working directory, never the test's) and checks the note, the registered
>    name in the commands printed, `roadmap .` and `roadmap .\beta`.
> 2. **The note** is a `note:` line on stdout, as `send`'s notes are, printed only when the file
>    or folder passed over is not the project's own folder, so a bare name from the data root
>    prints none. It names the project, its folder, the place passed over and `.\<name>`.
> 3. **The words of a refusal**: a name that is nothing is "not a registered project, nor a file
>    or a folder in <the working directory>", with "(the registry did not read: …)" when it did
>    not; a path that is nothing is "not a file or a folder (<the full path>)"; an empty argument
>    names nothing. `revert` keeps "is not a registered project".
> 4. **`revert` on the resolver too**, though its usage says `<project>`: its order was already
>    the registry's first, and `.` from a project's root now reads there as well (before, `.`
>    never matched, the folder compared as given).
> 5. **Folders printed in full**: `crw roadmap .` and `crw history .` print the absolute folder
>    where they printed `.`.
> 6. **The plan and status skills' `crw roadmap <folder>`, `crw tasks <folder>` and `crw
>    formalise <folder>` lines** were read against the change: a folder by its path reads the
>    same place as before, now with its registered name, so they stay as they are.
> 7. **No screenshot run**: no list or pane changed (rule 5). **No review subagent**: an S brief
>    (rule 6 asks one of an M); the diff was read through by the session before the checks.
> 8. **The checks**: `cargo test` (903 passed, the three new tests among them) and `cargo test
>    --features screenshot` (911), both clippy runs and `cargo fmt --check` clean; the release
>    build. The `CRW_HOME` try with the release `crw` on a scratch data root holding a synthetic
>    project laid out as `harbor\harbor`, run from its root: `roadmap`, `tasks` and `formalise`
>    (the preview) with the bare name printed the note and read the project, the preview's
>    commands naming it `harbor`; `roadmap .` and `status --offline .` read it with its name and
>    state (`harbor  (active)`, where `status .` showed an empty name before); `send .` wrote IN-1
>    into its own inbox under its name; `roadmap .\harbor` read the subfolder; a name that is
>    nothing was refused in its words; `crw help` shows the paragraph. The scratch folder was
>    removed after. The word check of rule 3 on the staged diff printed nothing.
>
> **Not handed on.** IN-17 and IN-18 came from the sweep's sessions, whose projects keep naming
> themselves by their absolute paths until the install and OT-45; the owner's sweep tasks
> (OT-40 … OT-43) meet the fix there, so no letter goes back.
