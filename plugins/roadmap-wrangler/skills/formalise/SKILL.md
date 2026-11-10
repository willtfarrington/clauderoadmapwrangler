---
name: formalise
description: "Bring a project into the house style: preview and write its house-style files (the CLAUDE.md rule block, roadmap/owner-tasks.md, roadmap/inbox.md and the brief template) through Roadmap Wrangler's crw formalise on the owner's word, hand an older roadmap's migration prompt to a session in that project, or write a first roadmap from a brief."
when_to_use: "The owner asks to formalise, adopt into the house style, bootstrap or migrate a project, or a project lacks the house-style files (no owner-tasks.md or inbox.md, no rule block, a five-column or work-packet roadmap)."
argument-hint: "[the project's name or folder]"
---

# Formalise a project

A formalised project carries the house's files: the rule block in its `CLAUDE.md`,
`roadmap/owner-tasks.md` and `roadmap/inbox.md` with their headers, and the brief template
`roadmap/_TEMPLATE.md`. Their texts are Roadmap Wrangler's own, so they are written by its
engine, never retyped. The forms are in `${CLAUDE_PLUGIN_ROOT}/reference/house-style.md`, whose
last section quotes the engine's texts byte for byte.

## 1. Find the tool

Roadmap Wrangler's console is `crw`. On Windows it is
`%LOCALAPPDATA%\clauderoadmapwrangler\bin\crw.exe` once installed, or `target\release\crw.exe` in a
build of the tool's repository; it is not on PATH, so call it by its full path. A registered
project is named by its registry name; any other folder by its path.

Without the tool, say so. The reference's texts may be written by hand only on the owner's word,
under the same rules as the engine: a file that is there is never replaced, and `CLAUDE.md` changes
only between the two `roadmap-wrangler` markers, at its end.

## 2. Where it may run

A formalise writes into the project. Run it from a session whose working folder is that project,
or give the owner the command. A session in another project, or a read-only portfolio chat, never
writes there: it names the command for the owner, or for a session launched in the project.

## 3. Preview

```text
crw formalise <project|folder>
```

It changes nothing. It lists the four files, each `created`, `the rule block added at the end, the
file's n lines kept`, `replaced between the markers`, `kept` or refused with its reason (markers
that cannot be read, a linked `CLAUDE.md`, a folder that is not there), then shows the block and
each new file's text and the roadmap's form. Put the plan to the owner through `AskUserQuestion`,
the files and their changes written into the question.

## 4. Write, on the owner's word

```text
crw formalise <project|folder> --yes
```

It plans again and writes the files as they are at that moment: a file that is there is never
replaced, and the text outside the markers is never touched. It is refused while another session
works or waits in the project; the session that runs the command is left out of that check (the
tool knows it by the `CLAUDE_CODE_SESSION_ID` Claude Code gives its commands). It copies
`CLAUDE.md` to the tool's backups folder before changing it and prints `wrote …` for each file. A
second preview then says `Nothing to write`, and `crw tasks <folder>` reads the new files with no
problem.

## 5. An older roadmap

When the preview names a five-column or work-packet roadmap, the tables need a migration, which is
a session's work in that project, never this one's:

```text
crw formalise <project> --migration
```

prints the migration prompt and launches nothing. The owner hands it to a session in that
project: pasted into Roadmap Wrangler's launch dialog for the project, in place of the opener,
and started headless, or sent in a session they open there. That session reads the project and
`crw roadmap` first, puts its plan to the owner, converts the tables keeping every id, hash and
file, checks that the next EP and the counts are unchanged, and stops before committing.

## 6. No roadmap yet

The preview says when a project has no `roadmap/README.md`. A first roadmap comes from the `plan`
skill's interview. For a small project the owner has already described in a brief, write a first
roadmap from that brief instead, each choice confirmed with the owner: a P0 row for the planning
brief, the first phase's rows with their brief notes, `DECISIONS.md` with `D-1` … from the brief's
choices, all in the reference's forms. Check it with `crw roadmap <folder>` (the next EP and the
counts), then tell the owner that `crw set <project> state active` moves a registered project out
of `unformalised`; the registry is the owner's, so a session never writes it, and an unregistered
folder is the owner's to adopt (`crw adopt <folder>`).
