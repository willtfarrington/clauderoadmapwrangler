---
ep: 57a
title: The manual in the window, built when stale and opened from a row's menu and the Detail pane, its date beside the last phase's close; the window's old size words
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-57a — The manual in the window: built when stale and opened from a row's menu and the Detail pane, its date beside the last phase's close; the window's old size words

**Size:** M · **Depends on:** EP-57 · **Blocks:** EP-58 (this project's own manual, which
describes these entries) · **Decisions:** D-25, D-12 addendum of 2026-11-25 (and D-3, D-29 as
standing)

## Context

EP-57 built the engine: `crw_core::manual::build(&Job, &dyn Runner)` turns a project's
`docs/manual.md` (or `docs/manual-changes.md`) into a PDF in the local folder's `manuals`,
`Built::Fresh` asking no child at all when the PDF is newer than its source, its pictures and the
exe; its refusals are words ready for a hover ("owes no manual", the winget line). The window
offers none of it yet: this brief gives it the two entries, the build on a worker thread, the
first open-a-file action the window has, the manual's date in the Status section, and the old
size and commit words EP-54 parked (moved here at EP-56, the owner's choice).

**Session start (2026-11-25).** A chained headless run: the window chained this brief at 10:07
after EP-57 (ledger row `3c828ec3`, EP-57a, headless, claude-opus-5-5/max, the chain's mark 70 %
of the session window); the prompt is the tool's opener, so no hand start to explain. M's cap of
1 h 30 min of working time from 10:06. **Judged to fit, so not split:** the build is EP-57's call
on a worker thread in the pattern of `visual.rs`, the open one child through `process`, the
entries two items in an existing menu and two links on an existing line, the Status line one
subsection, the words five strings. If time runs short after the entries, the Status line (the
manual's date and the last phase's close) is proposed as a row EP-57b (S) in the completion note.
Read first: the roadmap's P6 table and EP-57a's brief note, `DECISIONS.md` (D-12's addendum of
EP-56, D-25), EP-57's completion note, `roadmap/owner-tasks.md` (OT-32 … OT-43 open, none
gated) and `roadmap/inbox.md` (no open message). Field check since EP-57's record commit
(`ea9e6b4`, 10:04): `crash.log` last written 2026-11-23 04:00, nothing new; `crw notifications`:
EP-57's finish at 10:06 and this chain at 10:07, nothing odd; `crw sessions`: this run open, EP-57
completed in 50 min; `crw autopilot`: not paused, six projects on, the chain log's last lines
EP-57's held restart and this chain. Nothing to triage.

**What the code says** (read at pickup). The row's menu is `row_menu` (`crates/app/src/list.rs`),
"Refresh this project" its item after State; the Detail pane's head is `project`
(`crates/app/src/detail.rs`), the path a weak label under the heading; the Status section is
`status` there, its subsections named by `SUBSECTIONS` and laid out by `Columns`. A worker thread
whose answer the board takes in each frame is `visual.rs`'s pattern (`mpsc`, `try_recv`, a repaint
asked while it waits); `process::start_alone` starts a child with nothing piped that goes on by
itself. The parked words: the Next column's hover (`list.rs`, "M about an hour, L about two
hours"), `size_hint` (`list.rs`), the `Size` doc comment (`crates/core/src/roadmap/mod.rs`, "S ≈
30 min, M ≈ 1 h, L ≈ 2 h of one session"), the Commit button's hover (`sessions.rs`, "the house
pattern's two commits") and the launch's words on the phone (`phone.rs`, "It commits itself in the
house pattern").

## In scope

1. **What a project holds** (`crw_core::manual`): whether its `docs/manual.md` and
   `docs/manual-changes.md` are there, read with the files (two looks, no child) into the
   snapshot; the manual's last commit time asked by the git pass only when the file is there.
2. **The build and the open:** one build at a time on a worker thread, EP-57's `build` with the
   real runner, `force` off and the window's exe time as the template's, then the PDF opened with
   the system's viewer: `explorer.exe <pdf>` through `process` in its turn, not waited for (a
   fresh PDF opens at once, with no child but the viewer's). What it did in the status bar (opened,
   built now or up to date, its date, the count of warnings, the warnings whole in the rolling log)
   or why not (the engine's refusal or the open's failure). A screenshot run builds and opens
   nothing. Never a link from the chat.
3. **The entries:** **Manual** and **Changes since the last phase** on a row's menu after "Refresh
   this project", and as two links on the Detail pane's path line; greyed with the reason on hover
   when the folder is missing, the project owes no manual (no `docs/manual.md`, D-12), the changes
   file is missing, Typst is not found (the winget line; looked for at the window's start and
   after each build), or a build is on its way.
4. **The Status line:** a Manual subsection in the Status section: the manual's date (its last
   commit; the file's own time while it is not committed) beside the last phase's close (the
   completion date of the last row of the last phase whose rows are all done), amber when that
   phase closed after the manual's date; "owes no manual" for a project without one.
5. **The phone:** its project page links no PDF (the file is on this PC; serving it is a new
   route behind the guard, not cheap): a v2 note.
6. **EP-54's parked words:** an S about 30 minutes of active work and an M about 45, an L still
   read in older rows (the Next column's hover, `size_hint`, the `Size` doc comment); "the
   project's own pattern when its `CLAUDE.md` states one" beside the house pattern (the Commit
   button's hover, the phone's launch words).

## Out of scope

- This project's own manual and its changes file (EP-58); rule 14's changes section, owed only
  once `docs/manual.md` exists here.
- A Force build from the window (`crw manual --force` stays the way); the converter's warnings
  shown in a pane of their own (the rolling log holds them).
- The phone's link to the PDF (v2 note); the status bar's notice cut by width (EP-59's addendum).

## Verification / acceptance

- Tests on temp folders: the two files read into the snapshot; the git pass's manual date only
  when the file is there; the entries' availability and words (each greyed reason); the menu's
  and the Detail pane's entries and their actions; the build-and-open through a fake runner and a
  fake opener (fresh: opened with no child; built: opened; refused: nothing opened; the open's
  failure said); the Status line's words and its amber; the last phase's close from a synthetic
  roadmap. No test opens a viewer or runs `typst` (rule 5).
- `cargo test`, `cargo test --features screenshot`, both clippy runs and `cargo fmt --check`; a
  review subagent given the diff (an M brief, rule 6).
- Screenshot runs at the three sizes showing the Detail pane's links and the Status line over the
  fake portfolio (one fake project with a manual), and one over a copy of the real portfolio's
  roadmap files (rule 5), the brief naming what was checked, never the projects.
- The word check of rule 3 on the staged diff prints nothing.
- The release build and the install run last, as the opener asks.
- **Residual risk.** No session opens the owner's PDF viewer, so the open itself is unexercised:
  `explorer.exe "<pdf>"` hands a file to the program its type opens with, the most common way a
  Windows program opens a document, and the path goes quoted (Explorer splits at commas) with
  `\` alone (it reads `/` as a switch); Explorer's exit code is never read, so a viewer that does
  not come shows only as nothing opening, which OT-44 watches for. No project owes a manual yet,
  so the window's entries are greyed everywhere until EP-58 writes this project's, and the first
  real build from the window is OT-44's; the build itself is EP-57's, which the real Typst ran in
  its trials. The entries, the links, the greyed reasons and the Status line were pictured over
  the fake portfolio and a scratch copy of it (the amber line by a file dated back) and over a
  copy of two real roadmaps (greyed links on real names and paths); the right-click menu is
  tested, not pictured, since a screenshot run cannot open it. The manual's date reads the git
  pass's third child, asked of a project that keeps `docs/manual.md` alone; an edit outside a
  session is dated by the file's own time from the next read of the files (F5, a session's end,
  a change under `roadmap/`), since the watcher looks at `roadmap/` and `HEAD` only.

> **Completion note (2026-11-25; claude-opus-5-5/max; about 50 minutes of working time from the
> start at 10:06 to the install at about 10:55, no wait on the owner).** A chained headless run
> (ledger row `3c828ec3`), judged at the brief to fit M's cap, so not split; the code compiled by
> 10:17 and the fallback (the Status line as an EP-57b) was not needed. Built: `manual::Present`
> (the two files read with the others into the snapshot) and `manual::open` (Explorer, the path
> quoted through the new `process::start_quoted` and rebuilt with `\`); the git pass's third
> child for the manual's last commit (`GitReading::manual`, a `ManualCommit` holding the file's
> own time beside the commit's) and `Snapshot::last_close`; the window's `manuals.rs` (the
> entries' availability and words, the build and open on a worker thread named `manual`, one at a
> time, the Status line's words); **Manual** and **Changes since the last phase** on a row's menu
> and as links beside the Detail pane's path; the Status section's Manual subsection, amber when
> the last phase closed after the manual's date; `Action::OpenManual`; EP-54's parked words; a
> synthetic manual in the fake portfolio's harbor; the copy script's two lines; the phone's v2
> note; EP-58's pickup note; OT-44.
>
> **Owner decisions.** None arose: the choices below are technical and reversible, logged as the
> session's calls. The commits are the autopilot's standing yes (crw's D-22); the push is the
> owner's.
>
> **Session's calls.**
>
> 1. The open is `explorer.exe "<pdf>"` through `process` in the children's turn, not waited for
>    (Explorer exits at once and its code means nothing); `ShellExecuteW` would need a new feature
>    of the pinned `windows` crate and is no child, which the note asked for.
> 2. A fresh PDF opens with no child but the viewer's; the window's exe time is the template's,
>    as `crw manual` does with its own, so a new build of the window builds each PDF again once.
> 3. Typst is looked for (a PATH scan, no child) at the window's start, after each build, on F5
>    and on a row's Refresh this project, and its hover says "then press F5"; a fresh PDF is still
>    greyed while Typst is missing, as the note asked.
> 4. The last phase's close is the completion date of the last row of the last phase whose rows
>    are all checked, else the date of a heading that says done (the fake portfolio's and older
>    roadmaps' `(done 2026-09-14)`), never a "planned" one; amber when that date is later than the
>    manual's, so a manual folded on the re-plan's own day is not amber.
> 5. The manual's date is its last commit while the file is as git saw it (the file's own time
>    kept beside the commit's), else the file's own time; a repository the git pass has not
>    reached says so, and a time that could not be read says that rather than 1970.
> 6. Tests and screenshot runs never build or open (`Manuals::canned`, true under `cfg!(test)`
>    and set by `arrange`); the worker's work is `build_and_open`, tested through a fake runner
>    and a fake opener.
> 7. No Force from the window (`crw manual --force` stays the way); the converter's and Typst's
>    warnings follow the status bar's line into the rolling log.
> 8. The analytics' planned M, the medians' last fallback, is now 45 minutes (EP-53's measured
>    size): the Detail pane's "M 1 h 00 min (planned)" showed in the first pictures, the same
>    family as the parked words.
> 9. The phone's project page links no PDF: not cheap (a new route serving a file behind the
>    guard), so a v2 note in §4.
> 10. No `docs/manual-changes.md` section: this project's manual starts at EP-58 (rule 14).
> 11. The checks: `cargo test` (900 passed) and `cargo test --features screenshot` (908), both
>     clippy runs and `cargo fmt --check` clean. A review subagent given the diff and the row's
>     note found six points and some nits, all taken: (1) nothing stopped a future Board test from
>     a real build and open (now canned under `cfg!(test)`, and the start and answer tested); (2)
>     a Typst installed after the start stayed greyed until a restart (now F5 and Refresh this
>     project look again); (3) Explorer splits an unquoted path at commas and reads `/` as a
>     switch (now quoted and rebuilt); (4) a hand edit after the git pass kept the commit's date,
>     and a failed status read as "not committed" (now the file times are compared and the words
>     claim no more than they know); (5) the amber words said the re-plan folds the changes in,
>     the opposite of what amber means (now "the changes since are not folded in"); (6) the copy
>     script left the manual's files out (now copied); the nits: two doc comments counting git's
>     children, refusal words saying "not opened" twice, an unreadable time shown as 1970, the
>     Commit hover's two commits under a project's own pattern. Not taken: a phase with a `◐` or
>     an unchecked stretch row is not closed (as the note's "phase closed" reads). Screenshot runs
>     in `%TEMP%\crw-ep57a`, each looked at: the fake portfolio's harbor at 1600 × 1000 (the links
>     beside the path, wrapping with it) and 2400 × 1400 in Light; a scratch copy of the fake
>     orchard with a synthetic manual at 1100 × 900 (the greyed Changes link dimmer beside the live
>     one) and at full height (the Status section's Manual line in amber, the file dated back);
>     a copy of two real non-data projects' roadmaps at 1600 × 1000 (the links greyed on a long
>     name and path). Then the word check of rule 3 on the staged diff, the release build and the
>     install.
