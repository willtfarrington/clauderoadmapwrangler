---
name: status
description: "Report where a house-style project's roadmap stands, read the way Roadmap Wrangler's parser reads it: the next EP with its size, phase and brief note, the counts per phase, the open owner tasks and inbox messages, and whether a session is in progress. Read-only."
when_to_use: "The owner asks what is next in a project, how far a roadmap has come, what waits on them, or for a project's status; before picking up a brief in an unfamiliar project."
argument-hint: "[the project's name or folder; the working folder when none is given]"
---

# Report a roadmap's status

Read only; never edit, tick or commit. The schema is in
`${CLAUDE_PLUGIN_ROOT}/reference/house-style.md`.

## 1. The folder

The argument names the project: a path, or a name found as a folder beside the working folder's
project (the data root that holds the portfolio). With no argument, the working folder. Read only
the files named below, by their paths: never list a project's data folders.

## 2. With Roadmap Wrangler

When its console `crw` is there (on Windows `%LOCALAPPDATA%\clauderoadmapwrangler\bin\crw.exe`,
or `target\release\crw.exe` in a build of the tool's repository), its reading is the parser's
own:

- `crw roadmap <folder>`: the next EP, its brief note and brief, the decision log, the counts and
  whether a session is in progress, at git `HEAD` and in the working tree;
- `crw tasks <folder>`: the owner tasks, the inbox, the handed-on blocks and the older forms of the
  owner's to-dos.

Report from their output.

## 3. Without it, read as the parser does

- **The tables**: in `roadmap/README.md`, a phase table's header row opens with `#` and names a
  `Brief` (or `Packet`) column, its columns found by name: `Size` and `Done` (or `Status`) are
  needed, and such a table without one of them is a problem to report; `Depends on` and `Core` may
  be missing. A table whose header opens with `#` but names no Brief column (a glossary) is no
  phase table. Rows are single lines. The phase is the `## Phase …` heading above the table.
- **Done**: `☑` with a hash, or `[x]`, is done; `◐`, `[~]`, `☐`, `[ ]` and anything else are not.
- **The next EP**: the first row not done, in table order (not the lowest number). Its detail is
  its `- **EP-n.** …` bullet under the phase's `**Brief notes.**`; its brief, if written, is
  `roadmap/EP-n-<slug>.md`.
- **Counts**: done and total per phase and overall, `stretch` rows apart.
- **Owner items**: the open `- [ ] OT-n (…)` lines of `roadmap/owner-tasks.md`, each with its
  `[before EP-m]` gate (a gate on the next EP holds it); the open `- [ ] IN-n (…)` lines of
  `roadmap/inbox.md`, which the next session triages.
- **In progress**: `git status --porcelain -- roadmap/` (limited to the roadmap folder, so that
  no other folder is listed) shows the roadmap README changed or a brief untracked or changed, or
  a row is `[~]`; outside a repository, the next EP's brief exists without a completion note.

## 4. The report

One block per project:

- the next EP: id, title, size, depends-on, phase, and the first sentence or two of its note;
- progress: done of total per phase and overall;
- what waits on the owner: open owner tasks (a gate on the next EP first) and open inbox messages;
- whether a session is in progress, and why;
- problems: rows that do not read, a Done cell of an unknown form, a brief link to no file.

Keep it short; quote ids, never paraphrase them.
