---
ep: 58
title: This project's own manual, the reference chapters and the walkthrough of the routes no real use has met, built to PDF
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-58 — This project's own manual: the reference chapters and the walkthrough of the routes no real use has met, built to PDF

**Size:** M · **Depends on:** EP-57a, EP-52 · **Blocks:** EP-59 (the P6 re-plan, which folds
the first changes into it) · **Decisions:** D-1, D-12 addendum of 2026-11-25 (and D-3, D-29 as
standing)

## Context

EP-56 settled the convention (D-12 addendum: `docs/manual.md`, whose presence is the mark, and
`docs/manual-changes.md`; the outline in the plugin's reference, section "The manual"), EP-57
built `crw manual` and EP-57a the window's Manual entries, greyed everywhere until a project owes
a manual. This brief writes the first: this project's own, the convention's proof, the owner's
request 10 at EP-45. With it, CLAUDE.md rule 14 holds for every later brief, and OT-44's first
real open from the window can run.

**Session start (2026-11-25).** A chained headless run: the window restarted into EP-57a's build
at 10:47 carrying the chain and chained this brief at 10:47 (ledger row `0d3f0cb7`, EP-58,
headless, claude-opus-5-5/max, the chain's mark 90 % of the session window); the prompt is the
tool's opener, so no hand start to explain. M's cap of 1 h 30 min of working time from 10:47.
Read first: the roadmap's P6 table and EP-58's brief note, EP-44's note (its third input names
the routes carried into v2's first walk), `DECISIONS.md` (D-12's addendum of EP-56), the
reference's section "The manual", EP-57a's completion note, `roadmap/owner-tasks.md` (OT-32 …
OT-44 open, none gated before EP-58) and `roadmap/inbox.md` (no open message). Field check since
EP-57a's record commit (`5ab72bd`, 10:45): `crash.log` last written 2026-11-23 04:00, nothing
new; the rolling log: the install's restart refused at 10:46 while EP-57a worked, then taken at
10:47 at the gate's moment with one held chain and one plan carried, as EP-37a intends; `crw
notifications`: EP-57a's finish and this chain, nothing odd; `crw sessions`: this run open,
EP-57a completed in 40 min; `crw autopilot`: not paused, six projects on. Nothing to triage.

**Closed by hand (2026-11-25, from 14:32).** The run hit the account's five-hour limit at 11:12
with the review still owed, and the chain ended there. The owner asked a session started by hand
in VS Code (ledger row `dfe5414c`, manual; rule 11) whether the run's work had to be reverted
and redone or could be wrapped up; the completion note says what that session found and did.

**Judged to fit M's cap, so not split.** The work is one Markdown file drawn from the
repository README's detailed sections (which already describe every surface as built) and
re-ordered for a user, a walkthrough whose routes EP-44's note names one by one, an empty
changes file, a handful of screenshot runs over the fake portfolio, a PDF build of a few
seconds and a read-through. No code changes. If the time runs short after the manual's text,
the pictures beyond the main window's are proposed as a row EP-58a (S) in the completion note.

## In scope

1. **`docs/manual.md`**, from the reference's outline: the title and a paragraph on what the
   tool is and for whom, naming the phase it describes (P6, as EP-58 leaves it); the reference
   chapters: what it does; install and start; the window pane by pane (Projects, Detail, Owner
   tasks, Inbox, Autopilot, Decisions, Sessions, Chat, Usage, Diagnostics) with the menus, the
   tray and the keys; launching a session (Visual, Headless, the rules, the downshift, the pins);
   the decisions; the autopilot and its gate; notifications; the mail; formalise and migrate;
   the chat; the usage meter; history and analytics; hosting, backup and exports; the manuals
   (the window's entries from EP-57a); the phone view; `crw`'s commands; the files the tool keeps
   and where; when something goes wrong (the doctor, the logs, the crash log, File > Install and
   restart); then **the walkthrough**: one section per route EP-44's note carries into v2's
   first walk, each with its steps, what to look at and what to note down.
2. **`docs/manual-changes.md`**, created with it, empty: its header and a `Since` line naming
   EP-58, from which rule 14 holds.
3. **Pictures** from screenshot runs over the fake portfolio only (rule 3), into `docs/`, named
   by relative paths; the README's two existing pictures kept as they are.
4. **The README's "State of the app"** points to `docs/manual.md` (never to the PDF).
5. **The build:** `crw manual clauderoadmapwrangler` (and `--changes`), the PDF read through.

## Out of scope

- Any code change; a picture the screenshot feature cannot arrange today (a menu, a dialog it
  does not open) is described in words instead.
- The fold of later changes and the walkthrough's shortening (EP-59, the `replan` skill's step 8).
- The first open from the window (OT-44, the owner's).

## Verification / acceptance

- `crw manual clauderoadmapwrangler` builds the PDF with no warning; `--changes` builds the
  changes file; the PDF is read through page by page (the title page names the phase, the
  contents, each chapter on a new page, the pictures in place, the walkthrough last).
- The manual's statements checked against the code by a review subagent given the manual and
  the tree, not the session's conclusions (an M brief, rule 6); findings fixed or logged.
- `cargo test`, `cargo test --features screenshot`, both clippy runs and `cargo fmt --check`
  (nothing in code changes, so they confirm the tree; the fixtures' and the copy script's manual
  files untouched).
- No private name, path or machine detail in the manual or its pictures: the word check of
  rule 3 on the staged diff prints nothing.
- The release build and the install run last, as the opener asks.
- **Residual risk.** No session opens the real window, the owner's PDF viewer or GitHub's page
  (rules 5 and 4): the manual describes the window from the code, the README's long sections and
  pictures of the fake portfolio, and a review checked its statements against the code; what no
  screenshot run can arrange (a right-click menu, the tray's menu, the popup, most dialogs) is
  described from the code alone, so a word may differ from the real window's until the P6 walk
  reads it against the window. GitHub's rendering of `docs/manual.md` is unseen: the pictures
  are named by paths relative to the file (`manual/<name>.png`), the form GitHub resolves
  against the file's folder, and the file holds no raw HTML (the converter warns on any, and its
  last build warned on none). The walkthrough's routes are by definition unexercised: their
  steps rest on each route's brief and its residual risk, and the walk that uses them is the
  check. The first open of this manual from the window is OT-44's.

> **Completion note (2026-11-25; claude-opus-5-5/max headless, closed by hand in a
> claude-opus-5-5/max session; about 25 minutes of the run's work, 10:47 to 11:12, and about
> 40 minutes by hand, 14:32 to the install at about 15:12).**
>
> **How it ended.** The chained headless run (ledger row `0d3f0cb7`) hit the account's
> five-hour limit at 11:12, 25 minutes into its work, the chain having started under its own
> mark of 90 % of that window. By then everything below was written and staged, the PDF built
> and read through, the five checks passed and the word check clean. Owed were the review,
> whose subagent the same limit stopped before it had reported anything, this note's duration,
> the commits and the install; the window's chain log ended the chain there. The owner asked a
> session started by hand (ledger row `dfe5414c`) whether the work had to be reverted and redone
> or could be wrapped up. It was whole and staged, so that session wrapped it up as EP-43 was
> closed: it ran the review again, fixed its findings, rebuilt the PDF, checked the tree again
> and closed the brief.
>
> **What was written.** Judged at the brief to fit M's cap, so not split; the fallback (the
> pictures beyond the main window's as an EP-58a) was not needed. `docs/manual.md`, the
> first manual under the convention (D-12 addendum of EP-56): a title page naming phase P6, then
> eighteen reference chapters (what it does, with a table of the words that recur; install and
> start; the window pane by pane with the menus, the keys, the tray and the status bar; launching
> a session; answering a session; the autopilot and its gate, its eight conditions numbered;
> notifications; mail; formalise and migrate; the chat; the usage meter; history, analytics and
> exports; hosting and backup; the manuals, with the window's entries from EP-57a; the phone view;
> `crw`'s commands by group; the files the tool keeps, a table; when something goes wrong), and
> chapter 19, the walkthrough: one section for each of the seventeen routes EP-44's note carries
> into v2's first walk, each with its steps, what to look at and what to note down, so that the
> manual is the walk's script; `docs/manual-changes.md`, its header and a `Since` line naming
> EP-58, from which CLAUDE.md rule 14 holds; nine pictures in `docs/manual/`; the README's "State
> of the app" pointing to both files; EP-59's pickup addendum. Built with `crw manual
> clauderoadmapwrangler` (34 pages after the review's fixes, about 2.9 MB) and `--changes` (one
> page), neither with a warning at the last build, and read through by the run, then on the pages
> the fixes touched by the hand session.
>
> **Owner decisions.** (1) The run's work after the limit: wrapped up by hand, not reverted
> (the owner's question at the hand session's start, answered with the state above). No other
> arose: the choices below are the sessions', technical and reversible. The commits are the
> autopilot's standing yes (crw's D-22), on in this project; the push, which publishes the
> manual with the repository, is the owner's.
>
> **Session's calls.**
>
> 1. **The pictures**: ten screenshot runs over a copy of the fake portfolio at the neutral root
>    `C:\Users\Public\Projects` (EP-36's method, a fresh home each, the light theme for print),
>    3 to 4 s each; each looked at (the neutral path, the canned tool paths, nothing of the
>    machine). The template sets every picture at the text's width, so a whole window printed its
>    panes small: the window's picture stays whole, the eight others are cropped to the panes they
>    show (the launch dialog to itself), which about doubles their printed text and brought the
>    nine to about 3 MB. The Decisions picture was dropped as a near-copy of the Autopilot one,
>    whose right column shows the pane; the Decisions chapter points to it.
> 2. **The walkthrough holds EP-44's seventeen routes alone**: P6's own unexercised routes are
>    the re-plan's to add from its walk (the reference: each re-plan adds the new ones), and
>    OT-44 already scripts the first open from the window.
> 3. **The title** is the window's name, "Roadmap Wrangler — manual", the folder's name in the
>    opening paragraph; each picture's alt text is its caption in the PDF, so each says what the
>    picture shows and which pane sits beside the one it is about.
> 4. **The converter's one warning**, `<proposal>` read as raw HTML (which GitHub would hide as
>    well), was reworded; a code block's line that wrapped in the PDF was shortened; the manual
>    holds no footnote or task box.
> 5. **No code change**, so the checks confirm the tree; the README's state paragraph took the
>    pointer only, its words at v1's close left to EP-59 (the addendum says so).
> 6. **The review**, run again by the hand session after the limit stopped the run's: three
>    read-only subagents given the manual and the tree, not the sessions' conclusions, one per
>    part (the chapters up to Answering a session; the autopilot up to When something goes wrong;
>    the walkthrough), each finding checked against the code before it was applied. All
>    thirty-five were real and are fixed. The reference chapters' first part: the list's columns
>    (the autopilot box and Start at the row's right end); every folder under the data root
>    listed, not only repositories, and what Adopt does (its first state, its own workspace file
>    unless one is there, the aggregate workspace file); the registry written at the first change,
>    not the first start; the rules' order (a project's rules override the global ones; the
>    built-in S step-down; ultracode); the pins (Model and Effort as rules for the project or the
>    phase, Permissions as the project's mode); the reads again (a session's end at once and said,
>    a turn's end and the file changes quiet and five seconds apart); Visual greying the permission
>    picker too and showing a VS Code line; Kill as a red cross, Stop's interrupt, an ended tab's
>    Dismiss; the Detail pane's Open in Owner tasks and Open in Inbox; the newer-build chip for a
>    settled release build too; a deferred run's "… and resume" buttons and Decline's reason;
>    Ticked in the last week as the tool's ticks from anywhere; a run from the dialog capped only
>    with the autopilot on; the install renaming each installed exe that differs. The second part:
>    a phone page's "send it again" (the form not taken, nothing done) beside its "still at it";
>    a link with a scheme kept live in the PDF; the cap for every headless run of a project on
>    autopilot, L's for a row without a size, none for a migration; `crash.log` appended and moved
>    to `crash.old.log` past 1 MB; a usage figure too old to trust holding a chain; the daily
>    export's `registry.toml` and `ledger.json`; a refused restart not remembered; the site export
>    and the owner's actions beside the files the tool owns; and six wordings (the login item's
>    full name, Show and Dismiss all, a refusal logged once a minute an identity, the degraded
>    meter's newest figures, the chat's wait on a running reply, an orphan as a remote-less
>    project whose name the account holds). The walkthrough, whose seventeen routes match EP-44's
>    note in order: the Keep pins need Headless; the Formalise… item stays while a project is
>    unformalised; a real sign-in, then the quiet yield by the entry's command by hand; the urgent
>    letter's popup held while the window is in front or in the quiet hours, its time read in
>    `crw notifications`; the launch dialog reopening afresh; Make public's preview before the
>    action, the paths in its word check and the tick when the history did not read; the row's
>    Revert the latest session… only while it is the latest. Logged, not changed: the formalise
>    skill's "it is not on PATH" (`plugins/roadmap-wrangler/skills/formalise/SKILL.md`), stale
>    since EP-28a's `crw path` though its full-path advice still works, parked in EP-59's addendum
>    since this brief changes no plugin file.
> 7. **The checks**: `cargo test` (900 passed) and `cargo test --features screenshot` (908), both
>    clippy runs and `cargo fmt --check` clean, as at EP-57a, since no code changed; run by the
>    run, then again by the hand session on the final tree. The word check of rule 3 on the staged
>    diff printed nothing, at the run and again before the commit; the pictures were read by eye,
>    since it cannot read them.
> 8. **The hand session's calls**: the review cut into three parts run side by side (12 to 16
>    minutes each); the PDF built again after the fixes and the pages they touched rendered and
>    looked at; the install run last although no code changed, as every brief here ends.
