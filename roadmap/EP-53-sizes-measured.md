---
ep: EP-53
title: "The sizes measured again: every house-style project's briefs timed from its transcripts, S, M and L against the measurement, the rule that a brief is M or smaller"
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-53 — The sizes measured again

**Session start.** A headless run the window chained after EP-52 at 15:54 on 2026-11-24 (ledger
row `8ef2ca96`, `crw sessions clauderoadmapwrangler`: headless, EP-53, claude-opus-5-5/max), under
M's cap of 1 h 30 min of working time; the prompt is the window's opener. Read first: the roadmap's
P6 table and EP-53's brief note, D-4 and its six addenda, EP-52's completion note, the `replan`
skill's step 4 (the timing rule), `roadmap/owner-tasks.md` (OT-31 open, nothing waits on it) and
`roadmap/inbox.md` (no open message). **Field check:** the crash log's newest entry is of
2026-11-23 09:00 UTC, marked seen before EP-52's record commit, so nothing new; the notifications
since that commit are EP-52's finish, the chain held three times on the five-hour window (94 to
95 % against the chain's own mark of 90 %) until its reset at 15:50, and this run's chaining at
15:54; `crw sessions` shows this run open and nothing odd; `crw autopilot` is not paused, six
projects on. Nothing to triage.

**Fit.** Judged at the brief to fit M's cap: no code changes, a stdlib Python script outside the
repository, a table in the git-ignored `private/`, one round of questions, the records. If the
measurement runs long, the split is: the table and the figures here, the sizes' wording, the
D-4 addendum and the message as a row after it.

## Context

The owner's rule since EP-45 (D-4 addendum, 2026-11-22): **a brief is M or smaller, S preferred**;
an L only for one subtask that cannot be divided, with its reason in its row; a row that proves
larger at pickup is split there. The owner expects cutting to add briefs more than in proportion:
a brief carries its read-first, its brief, its build and its close whatever its size. After
EP-43's capped run the owner added a sizing rule by shape (an S is a change whose shape is known
at planning and touches a handful of files; an investigation, a state machine or the choice of a
tool is an M), and EP-43a raised S's cap to 75 minutes, M's and L's caps left until this
measurement. Five re-plans timed this project's briefs (D-4 addenda of EP-8, EP-16, EP-23, EP-29
and EP-32); no one has timed the other projects' briefs. The house-style rule in the owner's
environment repository states the sizes for every project (S about 30 minutes, M about an hour, L
about two hours).

## In scope

1. **The measurement.** A stdlib script in a scratch folder outside the repository, read before it
   runs, one project at a time, over every registered project whose roadmap has executed briefs
   (a checked row with its hash) and whose transcripts remain under `~/.claude/projects/<slug>/`.
   The `replan` skill's step 4 with IN-15's rule: a brief's sessions found by a Write or Edit of
   `<project>/roadmap/EP-n-*.md` (the absolute path, so fixtures never match) before its record
   commit; each session timed from its first entry (or the record commit of a brief it did before)
   to the EP's record commit; a gap over five minutes is the owner's only when the entry that ends
   it is an answered `AskUserQuestion` or a new prompt from the owner; timestamps, entry types and
   file paths only, never a conversation's text. The record commit from `git log -- roadmap/`
   (hashes and times only), the roadmap file and the transcripts being all that is read of a data
   repository, which is never listed. Subagents' own transcripts left out.
2. **The overhead,** from the same entries: the head (the session's first entry to the brief's
   first write: the read-first and the brief) and the tail (the last write of a file outside the
   records to the record commit: the checks, the build, the completion note and the commits),
   which a brief carries whatever its size.
3. **The table** in the git-ignored `private/` (each project's S, M and L: count, median and
   spread of active minutes, the share over the size's minutes and over its cap, what kind of
   brief overran, the overhead); this repository's tracked files get the figures without names.
4. **The sizes,** put to the owner with the measurement: what an S and an M are in minutes of
   active work, whether L remains, the rule's wording, and what the measurement suggests for the
   autopilot's caps and the registry's launch rules by size (the owner's to change, rule 13).
5. **The records:** a D-4 addendum, the roadmap's Sizes paragraph, EP-54's brief note told the
   sizes it carries into the conventions, and the message for the owner's environment repository
   (through `crw send` on the owner's word, since its name is a private word: never a tracked
   Handed on block).

## Out of scope

The caps' code (`crw_core::autopilot::cap`) and the opener's words, which a later row changes if
the owner moves the caps; the registry's launch rules; the plugin's skills, its reference and the
formalise templates (EP-54); re-sizing P6's open rows (EP-59's re-cut); any read of a
conversation's text.

## Verification / acceptance

- The script read before it ran; it prints counts and minutes, never text; its output and the
  table stay outside git (the table in `private/`, the script and its output in the scratch
  folder).
- Each project's brief count reconciles with its roadmap's checked rows less those whose sessions
  left no transcript, the misses counted by reason.
- This project's figures agree with the re-plans' retro tables where they overlap (P5 and P6's
  first rows), within a few minutes.
- The word check on the staged diff prints nothing; no tracked file names a managed project.
- No code changes, so `cargo test`, clippy and fmt are not rerun; the release build and the
  install run last, as the opener asks.

**Results.** The script and its helpers (a probe of the entries' shapes, a gatherer of the table)
were read before each run; they print counts and minutes only, and they and their outputs stay in
the scratch folder `%TEMP%\crw-ep53`. Eleven registered projects have checked rows in the
parser's form, 282 in all: 161 were timed in seven projects, and 121 left no transcript of their
own sessions (three rows in two of the seven, whose briefs no session wrote; every row of two
projects whose one transcript is a later session that touched each brief, a miss and not zero
minutes; every row of two projects with no transcripts). This project's figures agree with the
earlier notes where they overlap: EP-50 51 minutes, EP-51 40 (41 in its note), EP-51a 25, EP-52
28, EP-31b 23 (P5's S), EP-25a 18 and EP-27 22 (P4's S). A visual session runs higher under
IN-15's rule than under the older one, a gap ended by a permission answer now counting as the
session's (EP-31a: 90 minutes here, at most 69 in P5's retro). The figures without names are in
the D-4 addendum; the named table is `private/ep53-sizes-2026-11-24.md`. The word check on the
staged diff printed nothing.

**Residual risk.** The overhead is a floor: the tail starts at the last edit of a work file, so
checks run before a final fix count as work, and EP-43's transcript, read whole, put a code
brief's fixed part at about twenty minutes against the fifteen measured here. The transcripts
begin in late October, so four projects' briefs are unmeasured and this project holds half the
sample (81 of 161), its M's slower than the others' (50 against 34); an M of about 45 sits between
them, and the caps, unchanged, cover every size measured but four briefs. The message sent as
IN-7 quotes the first pass's figures (156 briefs, an M 40 and an L 45), before five re-sized rows
("L → M") were read; the final figures differ by a minute or two and change none of the owner's
choices. Until EP-54, the plugin's skills, its reference, the formalise templates and the opener
still name an L and the old minutes; the parser keeps reading an L, which every project's done
rows hold. The owner's registry rule for L briefs no longer meets a row, which breaks nothing.

> **Completion note (2026-11-24; claude-opus-5-5/max, a headless run the window chained after
> EP-52; about 20 minutes of working time from the start at 15:54 to the work commit, the owner's
> answers' wait of 74 minutes, 16:04 to 17:18, not counted).**
> Chained by the window after EP-52 at 15:54 (ledger row `8ef2ca96`) under M's cap of 1 h 30 min,
> once the five-hour window's reset at 15:50 let the held chain go. Judged at the brief to fit, so
> not split. The measurement ran by about 16:03 and the questions went to the owner at 16:04; the
> answers came at 17:18; the records, a second pass of the script and the commits followed. The
> work commit, the record commit, then the install.
>
> **Owner decisions.**
>
> 1. **What an S and an M are** (the measured medians S 30, M 40 in the first pass; this project's
>    M 49, the others' 34): **S about 30 minutes of active work, M about 45** (recommended; the
>    alternatives were M kept at an hour or set at the median of 40). Applied: the D-4 addendum,
>    the roadmap's Sizes paragraph, EP-54's and EP-59's brief notes.
> 2. **Whether L remains** (18 L's in the first pass, 28 to 80 minutes, median 45, all within M's
>    cap): **no L, a brief is S or M** (recommended; the alternative was L kept as the exception for
>    one indivisible subtask). Applied as in 1; EP-44, P7's one L, is cut into M sessions by EP-59
>    as planned; the registry's L rule, the owner's, is left as it is.
> 3. **The rule's words**, with the measured cost of cutting (about 15 minutes per brief whatever
>    its size): **"A brief is S or M: an S where its shape is known at planning and it touches a
>    handful of files; an M for an investigation, a state machine or the choice of a tool, or cut
>    into a row that finds and a row that fixes. A row is not cut only to make it smaller"**, with
>    a row that proves larger at pickup split there (recommended; the alternatives were "S
>    preferred" as it was, or no preference). Applied as in 1.
> 4. **The message for the owner's environment repository**, whose house-style rule 8 states the
>    sizes for every project: **sent now** (recommended; the alternatives were holding it for
>    EP-54 or none). Sent by `crw send` as IN-7 in that repository's inbox, with the figures
>    without names, the three choices and rule 8's new first sentences; the file as it was is
>    kept in the tool's backups. Its name is a private word, so no Handed on block here.
>
> The commits are the autopilot's standing yes (D-22); the push is the owner's.
>
> **The figures** (the final pass, 161 briefs; without names, as the D-4 addendum has them): S
> 30 minutes of active work (47 briefs, three quarters under 35), M 41 (95, under 51), L 47 (19,
> 28 to 80); a brief's fixed part about 15 minutes (the median code brief: 9 to its brief, 6 from
> its last code edit to its record commit); S's cap covers 45 of 47 S, M's 93 of 95 M; the
> overruns were code briefs and the old S-sized re-plans. The second pass read five re-sized rows
> ("L → M (cut beforehand)") that the first, which the questions and IN-7 quote, had skipped: a
> minute or two on M and L, none of the choices changed.
>
> **Session's calls.**
>
> 1. A brief's end is its record commit, the next commit touching `roadmap/` after the row's work
>    hash (`git log --format='%H %ct' -- roadmap/`, hashes and times only); for the data
>    repository, read by its transcripts and roadmap file alone, the last write of its roadmap
>    README in the brief's session.
> 2. A prompt is the owner's when its origin is human or it is the window's SDK prompt; task
>    notifications, peers' messages, compaction summaries and meta entries never end a wait.
> 3. Entries counted once by their uuid across a project's files, so that a resumed session's
>    copied history is not timed twice; subagents' transcripts left out.
> 4. A brief whose only writes came from a later session (a migration touching every brief) is a
>    miss, not zero minutes.
> 5. A re-sized row is timed at the size after its arrow.
> 6. The overhead measured as head (first entry to the brief's first write) and tail (the last
>    write of a work file, outside the records, scratch folders and commit-message files, to the
>    record commit); kind by the row's title (re-plan, re-cut, planning) and the files written.
> 7. The caps left as they are (S 75, M 90, L's 3 h for a brief with no size), since the
>    measurement asks for no change; the registry untouched (rule 13).
> 8. No owner task: the message is the environment repository's session's to act on, and the
>    registry's L rule harms nothing.
>
> No message to another project in a tracked block (IN-7 went by `crw send`).
