---
name: plan
description: "Plan a project's roadmap in the house style through a multi-round interview with its owner: a verified survey and a research pass, a domain map written to the repository first, rounds of four questions with the recommended option first, then the decision log, the roadmap, the session rules and the parked notes written from the record."
when_to_use: "The owner asks to plan a new project or its roadmap through an interview, to run a planning interview, or for as many questions as possible before a roadmap is written. A first roadmap written straight from a brief, without an interview, is the formalise skill's."
argument-hint: "[the brief, or the path of a file that holds it]"
---

# Plan a project through an interview

A house-style roadmap is written from an interview with the project's owner, never guessed: first
verified facts, then a map of what to decide, then rounds of questions whose answers are recorded
as they come, then the documents written from that record. Read
`${CLAUDE_PLUGIN_ROOT}/reference/house-style.md` before writing any of them: it is the schema they
follow. This planning brief is the project's EP-0.

## 1. The brief and the rules

Read the brief (the argument, or the file the owner names) twice, and the rules this session
already carries (the owner's user-level rules, the folder's `CLAUDE.md` if any). Note each feature
the brief asks for: the domain map marks them **(brief)**.

## 2. A verified survey, before any question

Check what the plan will rest on, with commands, not memory: the toolchain's versions
(`<tool> --version`), what the folder holds, the neighbouring projects whose code or conventions
the brief reuses, and earlier house-style roadmaps that set a precedent. Write down what was
checked and how. An option offered to the owner later states only checked facts; a fact that
could not be checked is said to be unchecked.

## 3. A research pass

For each tool, service or library the brief depends on: its documented surface (from its own
documentation, read now), current versions, and the existing tools that already do part of the
job. When the passes are broad and independent, delegate them to subagents (for example the
documentation, a survey of the precedents, the stack and the landscape) and keep their
conclusions, not their transcripts.

## 4. The brief and the planning file, written first

As in every house-style session, the brief comes before any other file: write
`roadmap/EP-0-<slug>.md` (front matter; Context: the brief and what the survey found; In scope:
the interview and the documents it yields; Out of scope: code; Verification), and close it with
its completion note at the end. Then create `roadmap/planning-<YYYY-MM-DD>.md` before the first
question, with four sections:

1. **State** — the survey's verified facts.
2. **Research pass** — what the research found, with versions and the date read.
3. **Domain map** — lettered domains (purpose and identity, the data model, conventions, runtime,
   views, automation, operations and quality, security, and whatever the brief adds), each a short
   list of what must be decided; the brief's own features marked **(brief)**, everything else
   proposed for the owner's consideration.
4. **Interview record** — empty; filled after every round.

If the repository is or will become public, keep private facts (other projects' names, machine
details, personal data) out of tracked files: a git-ignored file holds them, and the planning file
says so at its top.

## 5. The interview

- **Rounds of up to four questions** through `AskUserQuestion`, one domain per round. Round 1
  confirms the scope (which domains are in) and the identity (names, visibility, licence); then a
  round per domain; a domain the owner adds (often as a free-text note) gets a round of its own.
- **Every question stands alone**: the owner sees only the dialog, so it carries the fact it rests
  on, each option's consequence, and the paths or commands involved.
- **The recommended option first**, labelled "(Recommended)", with two to four options;
  multi-select for menus of features.
- **Record each round at once**, before the next: a paragraph in the interview record,
  `**Round 3 — the project model.** <each answer> (R).`, where `(R)` marks the recommended option
  and an owner's note is quoted. The record must survive a session that stops mid-interview.
- **A correction is a question**: when an earlier question rested on a wrong fact, say so in the
  next round and ask again; never patch an answer silently.
- **Prefer theory and synthetic acceptance** until the product is semi-final: a task that asks the
  owner for real use before a chain of briefs is a last resort, and real-use measurement belongs to
  a closing phase.
- Ask as many rounds as the domains need; owners of this method usually want them exhaustive.

## 6. The shape of v1

A round on the phases and their briefs: P0 is this plan; each phase closes with a re-plan brief;
each brief is sized S (about 30 minutes), M (about 1 hour) or L (about 2 hours) of one session, L
kept for the few engines; the totals in briefs and session-hours, with no calendar estimate unless
the owner asks; what "v1" includes; which phases get full brief notes now and which get charters
upgraded at a later re-plan.

## 7. The close-out round

The items not picked in any round (each parked in `roadmap/v2-notes.md`, or dropped on the owner's
word), the commit, the next EP and how it starts, and a sign-off on the breadth and on any security
model.

## 8. The documents, written from the record

- `DECISIONS.md`: `D-1` … `D-n`, each `**D-n Title.** Decision. *Why.* *Alternatives.*`, citing
  the facts it rests on.
- `roadmap/README.md`: what the plan is and how to use it, the sizes, the standing rules, one table
  per phase with its brief notes (`- **EP-n.** …`), the ordering rationale and the phase's standing
  decisions, and the risks.
- `CLAUDE.md`: the session rules (read first, the brief before any file, owner decisions through
  `AskUserQuestion`, commits as the owner's call, the build and verify commands).
- `roadmap/v2-notes.md`: everything named and not built.
- The completion note of `roadmap/EP-0-<slug>.md`: the rounds and questions, the owner's
  additions, the unpicked items, the routine choices.

When Roadmap Wrangler is installed, `crw formalise <folder>` previews the rule block, the
owner-tasks and inbox files and the brief template, and with `--yes` writes them on the owner's
word (the formalise skill); `crw roadmap <folder>` confirms that the tables parse and names the
next EP.

## 9. The commit

Only on the owner's word: `<Title>: <summary> (EP-0, D-x)`, the message passed through a file,
without attribution trailers unless the owner's rules ask for them; never a push unless asked.
