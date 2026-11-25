---
ep: 54
title: The conventions in this project's own files after the owner's review and the measured sizes
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-54 — The conventions in this project's own files after the owner's review and the measured sizes

**Size:** M · **Depends on:** EP-53 · **Blocks:** EP-55

## Context

The row brings the files this repository publishes for every project in line with the
conventions review the owner ran in their environment repository and with EP-53's sizes. The
review's outcome came in as IN-16 (triaged at EP-48, items (1) to (8) parked in this row's brief
note, item (9) in EP-59's, item (7) done at EP-48); EP-53 settled the sizes (the D-4 addendum of
2026-11-24: an S about 30 minutes of active work, an M about 45, no L; an S where the shape is
known at planning and it touches a handful of files, an M for an investigation, a state machine
or the choice of a tool, or cut into a row that finds and a row that fixes; a row not cut only to
make it smaller; a row that proves larger at pickup split there, never run long). The parser
keeps reading an L, which done rows in every project hold, and the autopilot's caps stay.

What exists: the plugin `plugins/roadmap-wrangler/` (four skills and `reference/house-style.md`,
which quotes the formalise engine's texts byte for byte, held by `crates/core/tests/plugin.rs`;
`plugin.json` names no `version`, so the push that changes it is the release, D-27 addendum of
EP-22); the formalise texts in `crates/core/src/formalise.rs` (`RULES`, `OWNER_TASKS_HEADER`,
`TEMPLATE`); the opener's last rule and commit words in `crates/core/src/prompt.rs`
(`session_rules`, `commit_words`, `commit_message`); the top-level README's house-style section;
this project's `CLAUDE.md`. A changed rule block makes every formalised project's plan
`ReplaceBlock`, so its row's menu offers Formalise… (no Detail section: `House::matters` counts
only the owner-tasks and inbox files, a refusal or an older roadmap) until its next formalise,
which EP-55's sweep names.

**Session start (2026-11-24).** A headless run the owner started from the window at 21:35 with a
chain plan (ledger row `c842a690`, EP-54, claude-opus-5-5/max; the chain's mark 70 % of the
session window); the prompt is the tool's opener. M's cap of 1 h 30 min: judged to fit with its
checks and close (text edits in about ten files, one small code change with its tests, the
review, the build), so not split. Read first: the roadmap's P6 table and EP-54's brief note with
its two addenda, `DECISIONS.md` (D-4, D-12, D-17, D-22, D-27 and their addenda), EP-53's
completion note, `roadmap/owner-tasks.md` (no open task; OT-31's tick, the owner's, uncommitted
in the tree after their manual session `440abbf9` committed the phone picture `98cf56f` at
21:34) and `roadmap/inbox.md` (no open message). Field check since EP-53's record commit
(`30fb61c`, 17:23): `crash.log` last written 2026-11-23 04:00, before it, nothing new;
`crw notifications`: the chain's end after EP-53 and the environment repository's inbox badge for
EP-53's message, nothing odd; `crw autopilot`: not paused, six projects on; `crw sessions`: this
run and the owner's manual session. Nothing to triage.

## In scope

Each change against the item it answers (IN-16's numbers; "sizes" for EP-53's addendum):

1. **(1)** This project's `CLAUDE.md` rule 1 names `DECISIONS.md` without its range of ids.
   **(sizes)** Rule 6's review "for an M or L brief" becomes "for an M brief".
2. **(2)** The `replan` skill's step 4 gains, after "when one session did two briefs, start the
   second at the first's record commit;", the clause "a session resumed after a night or another
   long absence shows it as one gap: report that EP's wall time from the owner's return;".
3. **(3, sizes)** The size rule in the D-4 addendum's words: the `plan` skill's step 6 (in place
   of "S (about 30 minutes), M (about 1 hour) or L (about 2 hours) …, L kept for the few
   engines"); the reference's Size cell (in place of "anything larger is split at pickup"); the
   rule block's roadmap bullet (in place of "sizes S, M or L", as a short Sizes bullet), quoted
   byte for byte in the reference; the `replan` skill's re-cut (rows S or M, the totals recounted
   from the tables); the top-level README's house-style section. The brief template's `size: S`
   stays; the opener names a size only through the run's cap, which stays.
4. **(4)** The `plan` skill's "Ask as many rounds as the domains need; owners of this method
   usually want them exhaustive." becomes "Keep a planning session to a few rounds, the detail
   pushed into brief notes that later sessions grow; a long planning request becomes several
   sessions."
5. **(5)** The opener's rule 5 for an autopilot run says "(crw's D-22)".
6. **(6)** The opener's commit words (also the Commit button's message): "commit as the
   project's `CLAUDE.md` says: the house pattern, `<Title>: <summary> ({ep}, D-x)` then
   `docs(roadmap): record {ep} commit hash`, unless it states its own", the hash recorded in the
   row and never a push as before; the reference's Commits section and the README's commit line
   say a project's own pattern is kept.
7. **(8)** The leftover copies of other projects' `CLAUDE.md` files under the git-ignored
   `target\tmp` removed (36 files, looked at by their first line before). How much more of
   `target\tmp` goes (2,732 files, about 100 MB of old briefs' scratch, about 1,800 of them
   roadmap files, many copied from the portfolio) is put to the owner: a bulk delete is the kind
   the machine's security software flags.
8. **(IN-16's preamble)** The reference aligned where it differs from the rewritten house style:
   the front matter kept true to the session, one brief per session unless the owner asks for
   more, a parked block under a brief note written by the project's own sessions and never by a
   sender, and the inbox line written by a session only for a receiver outside the tool's
   registry.
9. **Records.** A D-12 addendum (the house-style texts: the rule block's sizes, the reference's
   alignment, the commit pattern deferred to a project's own) and a D-27 addendum (the plugin's
   skills; no `version`, so the push is the release and the owner's sessions take it with
   `claude plugin update`); the owner task to update the plugin in their own sessions after the
   push; a pickup note on EP-55's brief note (the replaced block, the updated plugin); EP-54's
   two parked addenda folded in here and removed; the row checked with its hash.

## Out of scope

- Item (9), the history clauses in this `CLAUDE.md`: EP-59's. Item (7): done at EP-48.
- The `formalise` and `status` skills: read, nothing in them states a size, a round count or the
  commit words (the status skill already limits `git status` to `roadmap/`).
- The parser's reading of an L, the autopilot's caps (S 75 minutes, M 90), the registry's
  launch rules by size (the owner's, rule 13).
- Formalising any project again (the owner's, by EP-55's sweep); the user-level house style and
  the environment repository's files (theirs); EP-56's manual convention (that row changes the
  reference and the `replan` skill itself, this row having run).
- A `version` in `plugin.json` (D-27 addendum of EP-22 chose none).

## Verification / acceptance

- `cargo test` (the plugin's structural test holds the reference's quote of the rule block to
  the engine's text; the prompt tests read the new commit words), `cargo test --features
  screenshot`, both clippy runs with `-D warnings`, `cargo fmt --check`.
- A review subagent given the diff (an M brief, rule 6); findings fixed or logged.
- `crw formalise` previewed on a scratch folder under `CRW_HOME` with an old block: "replaced
  between the markers" and the new Sizes bullet shown. No screenshot run: no list or pane
  changes (rule 5).
- The word check of rule 3 on each staged diff prints nothing; then `cargo build --release` and
  the install.
- **Residual risk.** The owner's own sessions keep the old skills until the push and OT-32's
  update, and the window's headless runs take the new copy only from the installed build. Once
  the new build runs, every formalised project's row menu offers Formalise… (the block replaced)
  until the sweep formalises it again, which the owner may take for a fault; EP-55's steps say
  it. A session in a project with its own commit pattern reading the new commit words is
  unexercised here; the words keep the house pattern as the default. The `plan` skill's planning
  split over several sessions has met no real planning request yet.

> **Completion note (2026-11-25; claude-opus-5-5/max; about 75 minutes of working time from the
> start at 21:35 on 2026-11-24, the owner's two answers' waits, about 22:27 to 23:50 and 00:03 to
> the morning, not counted).**
> A headless run the owner started with a chain plan (ledger row `c842a690`), judged at the
> brief to fit M's cap, so not split. Built, by the review's items: (1) `CLAUDE.md` rule 1 names
> `DECISIONS.md` without a range, rule 6's review is "for an M brief"; (2) the `replan` skill's
> step 4 reports a resumed session's wall time from the owner's return; (3, sizes) the rule
> block's Sizes bullet (S about 30 minutes of active work, M about 45, the D-4 addendum's rule),
> quoted byte for byte in the reference, whose Size cell states the full rule, an `L` in older
> rows still read; the `plan` skill's step 6, the `replan` skill's re-cut (rows S or M, totals
> recounted from the tables) and the top-level README's house-style section the same; (4) the
> `plan` skill keeps a session to a few rounds, a long request becoming several sessions with
> briefs of their own, its trigger no longer "as many questions as possible"; (5) the opener
> cites "crw's D-22" in its autopilot rule and "crw's D-13" in its triage rule; (6) the opener's
> commit words and the Commit button's message defer to a project's own pattern, as do the
> reference, the README and the `replan` skill's close; (8) the 36 copies of other projects'
> `CLAUDE.md` files under `target\tmp` deleted (one `find … -delete`, the 14 copies of this
> project's own and one synthetic fixture left). From the review's preamble, the reference states
> one brief per session unless the owner asks for more, the front matter kept true to the
> session, segments and parked blocks written by the project's own sessions only, and an inbox
> line by hand only for a receiver outside the registry. Records: D-12, D-22 and D-27 addenda,
> OT-32 and OT-33, a pickup note on EP-55, a parked addendum on EP-59, EP-54's two addenda folded
> in and removed.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The rest of `target\tmp` (about 2,700 files of EP-24 … EP-42's scratch, about 1,800 of them roadmap files, many copied from the real portfolio), after the 36 copies went | **You delete it** (recommended; the alternatives were leaving it, or a recursive delete from the session's shell, which the machine's security software may stop half way) | OT-33: delete the folder in File Explorer |
> | 2 | The brief note's "plugin's version raised", against D-27's choice of no `version` at EP-22 | **Keep no version** (recommended; the alternative was `1.0.0` now, raised by hand at every later change) | The push is the release; the D-27 addendum; OT-32, the owner's update after the push |
>
> The commits are the autopilot's standing yes (crw's D-22); the push is the owner's.
>
> **Session's calls.**
>
> 1. "(crw's D-13)" in the triage rule and "(crw's D-19)" in the two texts a deferred run reads
>    (`session::deferred_message`, `decisions::resume_message`), by item (5)'s reason; item (5)
>    named only D-22. The review found the D-19 texts.
> 2. The rule block cites D-4 beside D-10 … D-13 and gives the size rule short (no "row that
>    finds and a row that fixes"); the reference's Size cell gives it whole.
> 3. The `formalise` and `status` skills unchanged: nothing in them states a size, a round count
>    or the commit words.
> 4. The review (a subagent given the diff and the review's items) found no blocking defect.
>    Fixed: the `plan` skill's trigger and its round-per-domain bullet against "a few rounds"; two
>    ragged wraps; a D-22 addendum for the commit words, which D-22's text states. Parked in
>    EP-59's notes (rule 5 owes a picture run for a list's or a pane's words): the old sizes in
>    the Projects pane's two hovers and a doc comment, the Commit hover's and the phone's "the
>    house pattern". Put to the owner: the version (decision 2).
> 5. No screenshot run: no list or pane changed. The `crw formalise` preview on a scratch folder
>    was not run for time; the formalise tests cover the replaced block (`ReplaceBlock`) and the
>    new block's text. `claude plugin validate` was not run (rule 5 keeps the real CLI out); the
>    plugin's structural test passed, its skill listings and front matter included.
> 6. OT-31's tick, the owner's, left in the tree by their manual session, is committed with the
>    record. Scratch logs went to a folder under `%TEMP%`, not `target\tmp`.
>
> Checks: `cargo fmt --check`, `cargo test` and `cargo test --features screenshot` (1,720 tests
> together), both clippy runs with `-D warnings`, all clean after the review's fixes; the word
> check of rule 3 on the staged diff; then the release build and the install.
>
> No message to another project.
