---
name: replan
description: "Run a house-style re-plan that closes a roadmap phase: the walk of the phase's residual risks with the owner, a retro of planned against measured times from the session transcripts, dated addenda to the decisions the phase changed, the next phase re-cut with pickup notes, and the parked notes grown."
when_to_use: "The next brief of a house-style roadmap is a re-plan row (Re-plan P2, …), or the owner asks to close a phase, re-plan, or review a phase's timings and decisions."
argument-hint: "[the project's folder, when it is not the working folder]"
---

# Re-plan a phase

A re-plan brief closes each phase. It measures the phase, settles what the phase taught, and makes
the next phase ready to pick up. The forms of every file it edits are in
`${CLAUDE_PLUGIN_ROOT}/reference/house-style.md`.

## 1. Read

The roadmap README (the phase being closed and the next one), every brief of the phase with its
completion note, `DECISIONS.md`, `roadmap/v2-notes.md`, `roadmap/owner-tasks.md` and
`roadmap/inbox.md` (triage first, as at any session's start). Then the re-plan row's own brief note:
earlier briefs park their findings there as `**From EP-n:** …` segments and dated addenda, and each
one needs an answer in this session. Write the re-plan's brief before touching any other file.

## 2. Check that the measurements have data

If the re-plan is meant to judge something the owner was to try (real use, a first real run,
timings over real work), check the record holds it before the interview. If it does not, offer in
the first round an owner task with steps a later session can rely on, rather than judging nothing.

## 3. The walk, first

Compile the **Residual risk** paragraphs that close the phase's briefs into steps: where to look,
what to do, the expected result. Take them with the owner live, one or a few per question, each
question standing alone (the owner sees only the dialog). Each finding becomes, at once, a new
row, an addendum or a v2 note.

## 4. The retro

For each brief of the phase, the planned size against the measured time. Time a brief from its
session's transcript, not from commits or a session ledger:

- transcripts are `~/.claude/projects/<slug>/<session id>.jsonl`, where the slug is the session's
  working folder with every character other than a letter or a digit replaced by `-`
  (`C:\Work\my_app` becomes `C--Work-my-app`);
- find a brief's session by its Write or Edit of `<project>/roadmap/EP-n-*.md`, anchored to the
  project's own folder (test fixtures may hold briefs of the same names), never by the request's
  words, which other text quotes;
- run from the session's first entry to the EP's record commit (`git log`); when one session did
  two briefs, start the second at the first's record commit;
- every entry's `timestamp` is UTC; gaps over five minutes are waits on the owner: report the
  active time and the wall time;
- read timestamps and file paths only, never the conversation's text.

Sum each phase's active and wall time against its planned hours. The sizes stay the planning unit
unless the owner changes them.

## 5. Addenda

For each decision the phase changed or refined, a dated addendum under its entry in
`DECISIONS.md`: `> **Addendum (YYYY-MM-DD, EP-n).** … *Alternatives.* …`. An entry is never
rewritten.

## 6. The re-cut

The next phase's rows confirmed, split, merged, re-ordered or added, each with its size and
depends-on; the totals recounted (briefs, S, M and L, session-hours). A dated sentence under the
phase's ordering rationale for each change. **Pickup notes** appended to the next phase's brief
notes as `**From EP-n:** …` segments: the code, files, commands and facts a later session needs,
named precisely. A far phase's charter briefs are upgraded to full brief notes when the plan said
so.

## 7. The parked notes

`roadmap/v2-notes.md` grows with everything named in the phase and not built, each with where it
came from.

## 8. The interview

Rounds of up to four questions, the recommended option first: facts and addenda first, then the
next phase's shape, then sign-offs of the edits drafted between rounds (so that each question is
concrete), then findings for other projects, then the close-out (the commit, the next EP).

## 9. The close

The brief's completion note records the timings, each owner decision and its outcome and the
routine choices. A finding for another project is a `> **Handed on (to <project>).**` block after
the note; a task only the owner can do is a line in `roadmap/owner-tasks.md`. Commit in the house
pattern on the owner's word, then record the hash in the row.
