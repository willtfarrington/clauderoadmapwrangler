---
ep: 25a
title: Per-project refresh
size: S
model: claude-fable-5-1
effort: high
status: done
---

# EP-25a — Per-project refresh

**Size:** S · **Depends on:** EP-12, EP-19 · **Blocks:** EP-29 · **Decisions:** D-9, D-20

## Context

A session changes the files the row reads: it ticks a roadmap row, writes a completion note, ticks
an owner task or an inbox line, and commits. Until now the window learned of that only at F5
(`Action::Refresh`, which reads every project's files, the backup job, git and GitHub again) or at
the next full pass. IN-13 (2026-11-11) asked that the project a session just worked on read afresh
on its own, and EP-25's triage placed it here as an S row after the history backfill. This brief
gives `portfolio::Loader` a one-project read and runs it at three moments: when a headless run of
the window ends (a chained one included, whose gate is read right after), when the hooks say a
visual or manual session ended, and from the row's menu.

**Session start (2026-11-12).** The session's prompt was this project's opener, launched from
clauderoadmapwrangler with fable at high effort; `crw sessions clauderoadmapwrangler` lists its
row (`97701680`, headless, EP-25a, started 18:39). Read: the roadmap's P4 table and EP-25a's brief
note, D-9 and D-20 with their addenda, EP-25's completion note, `owner-tasks.md` (no open task)
and the inbox (no open message). The field check: no new `crash.log` (the one present is EP-14a's
of 2026-11-03); `crw notifications` shows EP-25's finish and the chain's end at the day's cap
(22:55 yesterday) and nothing since; `crw sessions` shows yesterday's chain EP-38 → EP-25, each
run completed, and this run open; `crw autopilot` shows six projects on and the chain log ending
with EP-25's. Nothing odd. The working tree was clean at the start.

**The code as found.** `Loader::read_files` reads the whole registry on a worker thread and
`Loader::receive` swaps the portfolio in, adding back each row's kept git reading and history;
`Loader::read_git` walks every repository in its turn, importing the history when `HEAD` moved
(`import_into`). A headless run's end is drained in `Sessions::receive` (`Live::drain` returning
true pushes the key into `ended`), which `Board::autopilot_pass` takes with `take_ended` to weigh
the gate (`weigh_run`): the chained run's gate and the run's end are one moment there. A session's
end the hooks or the poll saw is a `Change::Ended` that `Watch::record` keeps in the ledger, where
the seen session's project and mode are known. The row's context menu is in `list::row`, and the
Detail pane's Status section is `detail::status`.

## In scope

1. **`Loader::read_project(name, moment)`** (brief note item 1): on a worker thread, the registry
   loaded, that project's `Registered` built (`Folder::inspect` on its folder), its `ProjectRow`
   read by `row()` as the files pass reads it, its counts recorded (EP-25's daily snapshot), and
   the row sent as `Message::Project`; then, for a repository, `read_git(folder)` in one turn as
   `Message::ProjectGit`, and the history check as the git pass does it (an import when `HEAD`
   moved, else the stored summary) as `Message::History`; last, `Message::Reread` with the moment
   and what was read. `receive` swaps the row in place, keeping the other rows untouched and
   adding back the kept git reading, the history, the backup, GitHub and hosting readings
   (`apply`) and the mail marks (the mail pass re-applies each frame); a full read of the files
   started after the one-project read supersedes it (the files generation is checked). The
   project read is noted on the row (`ProjectRow::reread`: when, the moment, what) and a notice
   is left for the status bar.
2. **The moments** (item 2): `Board::autopilot_pass` reads the project of each ended run before
   weighing its gate, a chat's turn and a migration's run left out, the words saying "chained"
   when the run was; `Watch::record` keeps the project of each `Change::Ended` whose session is
   visual or manual (a headless one is the window's own, read at the drain) in `ended_projects`,
   which the board takes each frame; a row's menu gains **Refresh this project**
   (`Action::RefreshProject`). The reads of one frame are deduplicated by project.
3. **The words** (item 3): the status bar's notice `harbor read again after its run ended: the
   files, git and the history`, and the Detail pane's Status section a line `Read again 2 min
   ago after its run ended: the files, git and the history` under Activity, gone at the next full
   read of the files.
4. **Tests** on the fake portfolio: a one-project read through the loader leaves the other rows'
   git, facts and history readings as they were and swaps the named row; a canned session's end
   (`Live::canned` with an `Event::Exited`) makes the board ask for the project's read, and a
   chat's end does not; the watch's `Change::Ended` of a manual session names its project; the
   menu item pushes its action.
5. **Docs**: a D-20 addendum (the end hook's third use: the project's re-read), the README's
   line on F5, the roadmap row linked to this brief, the completion note.

## Out of scope

- The GitHub pass for one project: the daily cache and F5 stay as they are (D-24).
- The backup job's reading for one project: read at each full git pass.
- A re-read on every hook event (a Stop, a prompt): the files change at a session's end.
- The gate reading from the refreshed row: `autopilot::read` reads the files from disk already.

## Verification / acceptance

1. `cargo test --workspace` and `cargo test --features screenshot` pass; both clippy runs and
   `cargo fmt --check` are clean; rule 3's word check on the staged diff prints nothing.
2. The tests of item 4 pass.
3. A screenshot run with `select=harbor,tile=detail` shows the Status section's line after a
   canned re-read (the flag `reread`), checked by eye at 1600 light.
4. The release build and the install run last (`scripts\install-shortcut.ps1`); the restart is
   refused while this run works and taken at the gate's moment or from File > Install and restart.

**Residual risk.** The three moments run in the real window only: a headless run's end is
exercised by this very run (the installed window reads this project again when it ends, once the
new build runs, so the first real re-read is the one after the next brief's run); the hook's
`Change::Ended` for a visual session, and the menu item on a real row, are the P4 walk's items
for this brief. A one-project read while a full git pass walks the same repository starts a
second git child in its turn, after the spacing: slower by a second, never wrong. The session
expects the moments to hold because each reuses the full passes' code on one row and the tests
drive the same message path the threads use.

> **Completion note (2026-11-12; claude-fable-5-1/high; about 25 min of wall time, a headless
> run from the window, no question asked).** Ledger row `97701680`, started 18:39; the prompt
> was the opener.
>
> **Built.**
>
> 1. **`portfolio`**: `read_project(store, name)` (one `Registered` built from the registry and
>    `Folder::inspect`, its row by `row()`), `Moment` (`RunEnded { chained }`, `SessionEnded`,
>    `Menu`) and `Reread` (`at`, `moment`, `what`; `line` for the pane, `notice` for the status
>    bar) on `ProjectRow::reread`; `Loader::read_project(name, moment)` on a worker thread
>    (`Message::Project` under the files generation it started in, `Message::ProjectGit`,
>    `Message::History` through the shared `history_after`, `Message::Reread` last), `rereading`
>    (a second ask while one runs is dropped), `reread_notice`; `receive` swaps the row in place
>    with the kept git reading, history, note and mail marks added back and `apply` run; a read
>    of every project's files clears the notes and supersedes a one-project read's row and note.
>    `record_counts` and `Git::head` factored out of the two passes.
> 2. **The moments**: `Board::reread_after_run(key)` before `weigh_run` in `autopilot_pass` (a
>    chat's turn, a run going on and a screenshot run read nothing); `Watch::record` keeps the
>    project of each `Change::Ended` whose session is visual or manual in `ended_projects`,
>    taken by `take_ended_projects` each frame; `Action::RefreshProject` from the row's menu's
>    **Refresh this project** (disabled for a missing folder).
> 3. **The words**: the status bar's notice (`harbor read again after its chained run ended: the
>    files, git and the history (imported again)`, or why nothing was read) and the Status
>    section's line under Activity (`Read again 2 min ago …`, `just now` under two minutes); the
>    screenshot flag `reread`.
> 4. **Docs**: a D-20 addendum, the README's paragraph, the roadmap row linked; tests: one in
>    `portfolio` (the swap, the other rows untouched, the dropped second ask, the superseding
>    full read, a name the registry lacks), one in `autopilot` (a canned run's end asks for its
>    project, a chat's and a run going on do not, the note on the row), one in `watch` (a manual
>    session's end names its project once, a headless one's and one outside the registry do
>    not), one in `detail` (the line), the list's row-menu test extended to click the item.
>
> **Checked by eye.** `target\tmp\ep25a\reread-light.png` (`light,reread,select=harbor,
> tile=detail,size=1600x1000`): the line under Activity on harbor's Status section; the dark
> 1200×900 picture does not scroll to Status. The pictures never enter git.
>
> **Verification.** `cargo test --workspace`: 375 in `core`, 166 in `app`, the other suites as
> before; with the feature 174 in `app`; both clippy runs and `cargo fmt --check` clean; rule
> 3's word check on the diff printed nothing. No review subagent: an S brief (rule 6). The
> release build and the install (`scripts\install-shortcut.ps1`) ran after the record commit,
> the opener's last step; the restart is refused while this run works and taken at the gate's
> moment, or from File > Install and restart, the tray's, or the launch dialog's.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> No other owner decision arose: no scope change, nothing outward-facing, no inbox message, no
> owner task.
>
> **Routine choices (the session's call, logged).**
>
> 1. **The gate's moment and the run's end are one call site**: `take_ended` gives both, and
>    the read goes first, so the row is fresh while the gate reads the files for itself; the
>    words say "chained" when the run was.
> 2. **A migration's run is read** (it writes the roadmap); only a chat's turn is left out.
> 3. **The watch leaves headless sessions out**: the window's own run is read at its drain, so
>    the end hook of that run does not read it twice; a session the watch first meets at its
>    end counts as manual and is read, harmlessly.
> 4. **A one-project read never re-reads the backup job or GitHub**: those stay with the full
>    passes (D-24's daily cache).
> 5. **The note is dropped at a full read** rather than kept: a row's "read again 3 h ago"
>    after a later F5 would mislead.
> 6. **The scratchpad** is `target\tmp\ep25a\` (ignored): the pictures, the logs, the commit
>    messages.
>
> **Owner task:** none. No message for another project. **Next: EP-26** (analytics, the
> graphs), chained by the autopilot when its gate allows, else from the window.
