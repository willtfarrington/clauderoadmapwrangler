---
ep: 48
title: A project read again when its session's work is complete — at the end of a turn of a visual or manual session and when its roadmap or its HEAD changes, the one project's files and git only
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-48 — A project read again when its session's work is complete

**Size:** M · **Depends on:** EP-45 · **Decisions:** D-9, D-20 (and D-29, D-3 as standing)

## Context

The owner's request 2 at EP-45: a session completes and the row still offers the brief it ran, so
the owner nearly started one twice; only the toolbar's Refresh, or the row's Refresh this project,
helps.

**What the code says** (read at pickup).

- `Loader::read_project(name, Moment)` (`crates/app/src/portfolio.rs`, EP-25a) reads one project on
  a worker thread: its files (the free function `read_project`: the registry,
  `snapshot::read_tree`, the badges and the formalise preview, no child), then for a repository
  `read_git` in one `process::turn()` and `history_after`. It sends `Message::Project` (the row
  swapped in place), `ProjectGit`, `History` and `Reread`, whose note the Detail pane's Status
  section shows and `Board::receive` puts in the status bar and the rolling log. A second ask while
  one runs is dropped (`rereading`); a read of every project's files started later supersedes it
  (`generation`).
- Its callers: `Board::reread_after_run` at a headless run's drain (`Moment::RunEnded`),
  `Watch::take_ended_projects` for a visual or manual session's end (`Moment::SessionEnded`, from
  the SessionEnd hook's or the poll's `Change::Ended` in `Watch::record`), the row's menu
  (`Moment::Menu`) and a hosting action (`Moment::Acted`).
- The session model (`crw_core::sessions::Sessions`, `crates/core/src/sessions/mod.rs`): the Stop
  hook sets the session `Idle` and returns `Change::Nothing` after its first work; the poll
  (`Sessions::poll`, every 5 s with the focus and 30 s without) sets each listed session's status
  as `claude agents --json` gives it, with no change for busy to idle. Nothing tells the window
  that a turn ended.
- The one `notify` watcher (`Loader::watch`) sees the data root's top level for folders created,
  removed or renamed; `Board::receive` settles a burst for `SETTLE` (600 ms), then reads every
  project's files.
- The launch dialog (`Board::open_launch`; `Dialog` in `crates/app/src/launch.rs`) is drafted from
  the row's snapshot as last read (`prompt::draft`), the registry read fresh; Start headless and
  Open in VS Code say why they are greyed (`headless_blocked`, `visual_blocked`).

**Session start (2026-11-23, 21:53 local time).** A chained headless run: the window restarted into
EP-47's build at 21:53:00 and chained EP-48 at 21:53:09 (the rolling log and `crw autopilot`:
claude-opus-5-5 at max, the rules' choice, M's cap of 1 h 30 min, the chain's own mark at 80 % of
the session window); `crw sessions clauderoadmapwrangler` lists the row (`e7f00dc5`, headless,
EP-48, open), and the prompt is the tool's chained opener. The field check read, since EP-47's
record commit (21:52:08): `crash.log`, unchanged since the panic of 04:00 that EP-43a read; the
rolling log, EP-47's install restart refused while it worked, the restart at the gate's moment,
IN-16 delivered at 21:53:06 and this chain; `crw notifications` and `crw autopilot`, nothing odd.
One finding for this brief: twice this evening the owner chose Refresh this project on another
project's row before pressing Start there (20:46 and 21:47, each before a visual brief), the hand
step this brief removes. IN-16 (from the owner's environment repository's EP-8: the conventions
review's outcome, for EP-54) is open and triaged at the close, its sender's name reworded under
rule 3 before a commit carries the line. OT-30 is open, gated before EP-54.

**Against the cap.** Three parts in the window's watch, loader, board and launch dialog, with
their tests, the review an M asks for, a screenshot run and the records: judged at 22:00, seven
minutes in, to fit M's cap only narrowly. So the work goes in the order (1), (3), (2): should (1)
and (3) not compile with their tests by 22:35, or (2) not by 22:55, part (2), the watcher, is split
off as a row of its own (EP-48a) and the rest closed.

## In scope

1. **A turn's end reads the project again.** The Stop hook of a visual or manual session in a
   registered project, or the poll's move of such a session from busy or waiting to idle, owes
   that project a read on its own (`Moment::TurnEnded`, "after a session's turn ended there").
   The loader keeps the owed reads (`Loader::owe`, and `Loader::read_owed` in each pass): one per
   project, started once due and while no read of that project runs, and none within
   `REREAD_GAP` (5 s) of the project's last read on its own; a read asked while one runs is owed,
   not dropped, so a change made during a read is read after it.
2. **A change of a project's roadmap or HEAD reads it again.** One watcher (`notify`,
   non-recursive; polling every 3 s after the review, see the completion note) on each registered
   project's `roadmap` folder and on its `.git/logs/HEAD`, the
   log a commit, a reset or a checkout appends to, set up again at each read of every project's
   files (a project added or gone follows); an event owes that project a read
   (`Moment::Changed`, "after its roadmap or its HEAD changed"), settled for `SETTLE` as the
   top-level watcher's burst is, so a commit made in a terminal counts. Never a data repository's
   other folders, nor `.git` itself, whose index `git status` may write (a loop); no GitHub read.
3. **The launch dialog reads the project as it opens.** `Board::open_launch` owes the project a
   read at once (`Moment::Launch`, no gap); Start headless and Open in VS Code are greyed while it
   runs; when it lands, a different next brief drafts the dialog again from the fresh row (a mode
   the owner chose kept), its first line saying "The row offered EP-a; read again as this dialog
   opened, the project's next brief is EP-b."; no brief left closes the dialog with the reason in
   the status bar. A migration's dialog reads nothing.
4. **Quiet reads.** The three new moments write a line of the rolling log and the Detail pane's
   note, not the status bar's notice, since a session's writes may bring one every few seconds; a
   failure still warns.
5. **A run of the window's own** in the project: a turn's end or a change there owes nothing, and
   the run's drain reads the project as today (EP-25a), so a headless brief's writes start no reads
   ("a re-read while a run of the window's own works there stays as it is").
6. **The records.** A D-20 addendum; the README's paragraph on a project read again.

## Out of scope

- The phone's Start page (EP-30b), which offers the row as the window last read it; the chain's
  own start reads the files for itself already.
- A roadmap's subfolders; a repository whose `.git` is a file (a worktree); a project whose first
  commit or `roadmap` folder came after the last read of every project's files, watched from the
  next one (F5, a folder change, the day's turn).
- The GitHub pass, which F5 and the hourly look keep (D-24).

## Verification / acceptance

- Unit tests, in temp folders with synthetic names: a Stop hook of a visual or manual session
  owes its project a turn's read, a headless run's or a session outside the registry none; the
  poll's busy-to-idle owes one, idle-to-idle none; owed reads wait out the gap and a read under
  way, and a burst settles to one read; a change in a watched `roadmap` folder and an append to
  `.git/logs/HEAD` in a temp repository owe that project a read; the launch dialog greys its start
  while its read runs, drafts the new brief with its line when the row's brief changed, says
  nothing when it did not, and closes with words when no brief is left. No test starts a session
  or the real CLI (rule 5).
- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings`, `cargo
  fmt --check`; a review subagent given the diff (an M brief).
- A screenshot run of the launch dialog with the changed brief's line over the fixtures.
- The release build, and the install last.
- No tracked file names a managed project, the owner's login or the machine (rule 3); the word
  check prints nothing on the staged diff.

**Residual risk.** No test runs a real session, VS Code or the hooks (rule 5): a turn's end is
tested with synthetic Stop hooks and synthetic listings of the poll, and a real VS Code tab's Stop
at each turn's end is expected from D-20's hooks, which have marked each finished turn as work
since EP-13. The watcher is tested in a temp folder with real writes, which its poll sees within
a scan; a real `git commit` appends to `.git/logs/HEAD` the same way, as an amend, a reset, a
checkout or a rebase does, and `git gc` or `reflog expire` rewrite it, which changes its time too.
The poll compares whole seconds (notify's), so a write in the same second as the scan before it
is seen only at the next change; a commit rarely follows the last move of `HEAD` within a second,
and a turn's end and the launch dialog read the project whatever the poll saw. Each scan stats
every registered project's `roadmap` entries and one file, every 3 seconds on a thread of its
own, holding no handle between scans (a rename that meets a scan's moment is refused once); its
cost on the real portfolio is not measured. A repository whose `.git` is a file (a worktree), and a `roadmap`
folder or a first commit made after the last read of every project's files, are not watched
until the next one (F5, a folder change, the day's turn). A session that writes its roadmap often
has its project read every five seconds at most, each read a few git children in their turn under
`process::turn()` and a line of the rolling log; not measured on the real portfolio. The launch
dialog's start waits for its read, a second or so for a repository by EP-25a's account, measured
here only on a folder without git; a slow git holds the button longer, its hover saying why. The
screenshot run shows the changed brief's line canned over the fixtures; the redraft itself is
covered by the board's test, not seen in a real window. While a headless run of the window's own
works in a project, a commit made there in a terminal shows only at the run's end, as before.

> **Completion note (2026-11-23; claude-opus-5-5/max, a chained headless run; 55 minutes from the
> start to the work commit, one owner question).**
> Chained at 21:53 after EP-47 (ledger row `e7f00dc5`) under M's cap of 1 h 30 min. Judged at the
> brief to fit only narrowly, with checkpoints at 22:35 and 22:55 for splitting the watcher off
> as EP-48a; the three parts compiled and their tests passed at 22:09, so nothing was split.
> The work commit `aaf8322` at 22:48, the record commit, then the install.
>
> **Owner decisions** (one `AskUserQuestion` at 22:12, both answered with the recommended
> option).
>
> 1. **IN-16's triage** (the conventions review's outcome, from the owner's environment
>    repository's EP-8): parked, items (1) to (8) in EP-54's brief notes as a dated addendum and
>    item (9) in EP-59's; item (7), this project's own memory note naming a note the environment
>    repository retired, amended with this session's memory update. Not chosen: clearing the
>    leftover copies under `target\tmp` tonight; a row of its own. The line's sender, on the
>    private word list, reworded before the commit (rule 3), as IN-15's was.
> 2. **OT-30** ticked on the owner's word, IN-16 being the message it waited for: EP-54 is no
>    longer gated.
>
> The commits are the autopilot's standing yes (D-22); the push is the owner's.
>
> **Built.**
>
> 1. **A turn's end** (`crates/app/src/watch.rs`): `Watch::turned` notes the project of a
>    visual or manual session at its Stop hook (`take_hook`) and at the poll's move from busy or
>    waiting to idle (`take_listing`, factored out of `receive`); `take_turned_projects`.
> 2. **The owed reads** (`crates/app/src/portfolio.rs`): `Loader::owe`, `read_owed`,
>    `reads_project`; `REREAD_GAP` (5 s from the project's last read on its own, which
>    `read_project` now notes) and `CHANGE_SETTLE` (600 ms); a read asked while one runs is
>    owed, not dropped; `Moment::TurnEnded`, `Changed` and `Launch`, quiet (`is_quiet`: the note
>    goes to `reread_lines` and the rolling log, a failure still to `reread_notice`).
> 3. **The watcher** (`portfolio.rs`): `watch_projects` at each read of every project's files,
>    notify's `PollWatcher` every `PROJECTS_POLL` (3 s) over each project's `roadmap` folder and
>    `.git/logs/HEAD` (`watched_paths`), each without subfolders, a path gone let go and a new one
>    watched, a failure a line of the rolling log; `Message::Changed` mapped to its project
>    (`watched_project`), `take_changed`. A read on its own asked while one runs, by any caller,
>    is owed (`read_project`); a read of every project's files that began before such a read and
>    lands after it has it owed again (`files_started`); a read thread that stops short still
>    ends (`Landing`).
> 4. **The board** (`crates/app/src/board.rs`): each pass owes the turned and changed projects
>    a read unless a run of the window's own works or starts there (`Sessions::summary`), starts
>    the due ones and wakes for the next; `open_launch` is now `draft_launch` and a `Launch` read
>    with `Dialog::checking`, which greys Start headless and Open in VS Code
>    (`crates/app/src/launch.rs`, `CHECKING`) and guards `start_headless` and `start_visual`;
>    `launch_reread` drafts the dialog again when the next brief changed (`changed_brief`'s line
>    first, a mode the owner chose kept) or closes it with words when none is left.
> 5. **The records**: the README's paragraph on a project read again; a D-20 addendum; the
>    screenshot feature's `launch=<project>:changed`.
>
> **Verification.** `cargo test`: 284 in `app`, 479 in `core`, the CLI's and the integration
> suites, all passing; `cargo test --features screenshot`: 292 in `app`; both clippy runs with
> `-D warnings` and `cargo fmt --check` clean; all run again after the review's fixes. The new
> tests: a turn's end from the Stop hook and from the poll, the owed reads (the gap, a read under
> way, the settle, a launch's at once), the watcher's writes in `roadmap` and `HEAD`'s log, a
> watched folder renamed, the dialog's start held, and the board's dialog over a scratch roadmap
> whose brief is done before its read lands, unchanged, and wholly done. The watcher's test
> failed once after the switch to polling, its `HEAD` log written in the same second as the scan
> before the append: the log is back-dated, as a real one is, and three runs passed. A
> screenshot run of `launch=harbor:changed` over the fixtures at 1600 × 1000, kept in the
> git-ignored `target\ep48-shots`: the line first among the dialog's notes, nothing cut. The
> release build passed on the final tree; the word check printed nothing.
>
> **Review** (a subagent given the diff and the brief's scope, about 23 minutes). Nine points:
>
> 1. **The watch held the project folders open.** Checked by a test: with the system's change
>    notifications on `roadmap` and `.git/logs`, renaming the project folder failed with "Access
>    is denied", and the window runs from login, so every registered folder would have refused a
>    rename. Fixed with notify's polling watcher, which holds no handle between its scans; a test
>    renames a watched folder. It compares whole seconds (residual risk).
> 2. **Every other caller's read was still dropped while one ran**, so a run's drain behind a
>    launch dialog's read could leave the dialog on the brief the run had just closed: any read
>    on its own asked while one runs is owed now; EP-25a's test says so.
> 3. **A read of every project's files in flight could overwrite a fresher read on its own**,
>    which ungreyed the dialog on the older brief: such a read is owed again when the full read
>    lands.
> 4. **A read thread that panicked left its project read forever**, the dialog greyed: the read's
>    end is sent on unwind.
> 5. **A launch's read lost its "at once"** when a change merged into it: an owed read keeps the
>    earlier time when either ask is at once, and the weightier moment; a test.
> 6. The dialog waits for git and the history import, which its decision does not need (low):
>    logged, not changed; a second or so, longer after `HEAD` moved.
> 7. Only the brief's id is compared (low): a row re-sized or retitled under the same id at the
>    moment of a launch keeps its old size in the dialog, and a read that lands after the
>    dialog's check is not followed while it stays open: logged, for the walk.
> 8. Watches the system drops by itself are never armed again (low): moot with the polling
>    watcher, which keeps its paths.
> 9. Each quiet read made the analytics read the ledger again on the window's thread (low): only
>    a read that imported the history again does now.
>
> **Session's calls.**
>
> 1. "A re-read while a run of the window's own works there stays as it is" read as: a turn's
>    end or a change in that project owes nothing, its drain reading it (EP-25a), so a headless
>    brief's many writes start no reads; the launch dialog's read goes whatever runs there.
> 2. The three new moments quiet: a line of the rolling log and the Detail pane's note, not the
>    status bar, which a session's writes would otherwise take every few seconds.
> 3. The dialog's start held while its read runs, beyond the brief note's ask, so that a quick
>    Start cannot send the stale brief.
> 4. The gap counted from the start of any read of the project on its own, the launch's
>    excepted, so a turn's end just after a session's end reads once more five seconds later
>    rather than never.
> 5. `.git/logs/HEAD` watched as a file, `.git` itself never, since `git status` may write its
>    index; after the review, the poll every 3 s (a commit seen within a few seconds) rather than
>    a watcher of my own comparing finer times, notify's being pinned already.
> 6. The screenshot run over the fixtures only: the change is one line of a dialog, not a list
>    or a pane (rule 5's run over a copy is for those).
>
> No owner task and no message to another project.
