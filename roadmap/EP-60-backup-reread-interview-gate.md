---
ep: 60
title: The P6 walk's fixes, split at pickup, the backup job read again on its own when its run log changes and at the day's turn, the gate holding a planning interview for the owner
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-60 — The backup job read again on its own; the gate holding a planning interview

**Size:** S · **Depends on:** EP-59 · **Blocks:** EP-44 · **Decisions:** D-24, D-22 (and D-3,
D-11, D-29 as standing; D-25 moves with item 2 to EP-60b)

## Context

EP-59's walk and field check left three fixes for P7's first row, the owner's choices (EP-59's
note). **The backup badges' false "stale".** The window reads the backup job
(`read_backup_job`, `crates/app/src/portfolio.rs`) only at the git pass's start (the window's own
start, F5, after a revert) and after Back up now (`Loader::read_backups`), and the row's badge
and the tray's marks weigh that cached reading against the clock (`Backup::state`, `STALE_AFTER`
of 48 hours). A window up past two of the job's daily runs without a full read therefore sees
every unit's last check pass 48 hours at the job's usual time: on 2026-11-20, 11-22 and 11-26
every project's badge turned "stale" between 12:30 and 12:32 while the job ran each day, cleared
only by F5. **The status bar's notice** is cut by characters, not by the width the right-hand
items leave. **The gate and a planning interview.** `autopilot::is_replan` names a re-plan as the
owner's to run, and the gate (`Refusal::Replan`) and the chain plan use it; a title naming a
planning interview (P7's EP-44, EP-44a and EP-44b) is not caught, so a chain reaching one could
start it headless.

**Session start (2026-11-27, 20:56 local time).** Started from the window (Start, Visual, Open in
VS Code); the prompt is the tool's opener, and `crw sessions clauderoadmapwrangler` lists the row
(`8479e6a2`, visual, EP-60, open). The launch suggested claude-opus-5-5 at max; the panel runs
claude-opus-5-5 at max, read from the `model` and `effort` fields of this session's own transcript
entries (no text read). Read first: the roadmap's P7 table and notes; `DECISIONS.md`'s D-22, D-24
and D-25 with their latest addenda and D-10's of 2026-11-26; EP-59a's completion note and EP-52's
parked line on the status bar; the code each item names. **Owner tasks:** none open. **Inbox:**
IN-24 (from the owner, the site's two pictures), triaged with the owner (below). **Field check**
since EP-59a's record commit (`fc17d63`, 2026-11-27 11:24):

- `crash.log`: last written 2026-11-23 04:00, nothing new.
- `crw notifications`: the latest twenty, 13:25 to 20:45: turns' ends and waits of visual and
  manual sessions in other projects, one inbox badge for a delivery between other projects, and
  three new folders of the data root marked unadopted. Nothing odd.
- `crw sessions`: this row, and EP-59a's ended at 13:29; a manual row of 2026-11-24 closed by the
  window's start at login from its transcript's last write.
- `crw autopilot`: not paused; fourteen projects on; the chain log unchanged since 2026-11-26
  22:05.
- The rolling log: the owner's Install and restart at 14:22, into the installed build of
  2026-11-25 19:52 (EP-58b's), which the window now runs; a start at login at 14:31; two second
  starts at 14:22 and 14:35 that ended at once, handing over to the running window; full reads at
  13:30 and at the start of 14:31 (so no false "stale" today); the re-reads and deliveries; no
  warning, no error, no `slow` line.
- **The portfolio's titles**, read for item 3's forms (named projects' roadmaps only, no project
  named here): five done rows name a planning interview at their head or after its first colon
  ("Planning interview: …", "The v1 planning interview: …", "…charter: owner interview on …",
  "Re-plan P0 as the v1 planning interview: …"); **one unchecked M row of a project whose
  autopilot is on** names one after a colon ("Plan the second …: a planning interview through
  `/roadmap-wrangler:plan`, …"), which a chain reaching it would start headless today; two
  unchecked M rows of another project with the autopilot on name an interview as that project's
  own subject, a feature and not a planning session, which the rule must leave alone; and this
  roadmap names the interview in passing (EP-45's "…; the v2 planning interview moves to P7",
  this row's "the v2 interview's sessions", EP-61's "as the interview wrote it").

**Against the size: split at pickup** (the owner's choice, EP-59's note: "If the three outgrow
the S at pickup, (2) is split off there"). The three came to about an hour by the session's
estimate: item 2's two screenshot runs, one over a copy of the real portfolio (rule 5), are its
larger part. Item 2 moves to a new S row, **EP-60b**, after EP-60a and before EP-44, and
**IN-24**'s two site pictures join it, the owner's choice: this project's v1 closed at EP-32
(2026-11-17), so they are due now, and that row makes screenshot runs anyway. This row keeps
items 1 and 3, neither of which changes a list's or a pane's layout.

## In scope

1. **The backup job read again on its own** (D-24):
   - its run log, the registry's `backup_log`, watched by the projects' poll watcher (notify's
     `PollWatcher` every `PROJECTS_POLL`, holding no handle between scans), its path taken from
     each read of the job, so a new path is watched and an old one let go;
   - a change of the log owes the job a read (`read_backups()`: the settings, the log's tail and
     each unit's newest archive, file reads only, no child) once its writes have settled, so a
     run's burst of lines is read once after it, not at every scan;
   - the day's turn reads the job too, beside every project's files, so a log the watcher cannot
     see (none set, or on a drive not mounted when it was watched) still leaves the reading at
     most a day old, under the 48 hours of `STALE_AFTER`;
   - tests on a scratch job whose log gains a newer `unchanged` line: the watcher names the log,
     the read lands, the unit's last check moves, its state goes from stale to fresh and the
     tray's marks lose its "backup stale"; the settle reads a burst once; the day's turn asks for
     the job's read.
2. **The gate and a planning interview** (D-22):
   - `autopilot::is_interview` (core): a title naming a planning interview, the owner's to run:
     "planning interview", "owner interview" or "owner's interview", "interview with the owner",
     "planning session" or "planning round", wherever it stands, failing safe as `is_replan` does
     (a wrong yes only ends a chain), unless named as another session's after a preposition just
     before it ("found at the planning interview", "fixes from the planning interview"); an
     interview with none of those words is a project's own subject, never caught;
   - `Refusal::Interview`, worded "EP-n is a planning interview, the owner's to run", in the gate
     after the re-plan's (a title naming both refuses as a re-plan) and in the chain plan's stop;
   - tests over the title forms the portfolio writes, as synthetic titles of the same shapes: the
     planning sessions (the interview at the head, after a colon, in a session of three, an owner
     interview), the interviews named in passing (this roadmap's three) and a project's own
     interviews; the gate's refusal and the plan's stop with their words.
3. **The records**: a D-24 addendum for item 1 and a D-22 addendum for item 3;
   `docs/manual-changes.md`'s section; the roadmap: this row linked and checked, EP-60b's row and
   brief note (item 2 from this row's note, IN-24's pictures with the real-or-mock question), the
   split noted in the header and P7's ordering rationale; IN-24's disposition written and ticked;
   this brief's completion note; rule 3's word check; the commits in the house pattern; the
   release build and the install, the session's last step.

## Out of scope

- **Item 2, the status bar's notice**, and its screenshot runs: EP-60b's, with IN-24's pictures.
- The window's own restart refusals written to the rolling log as the install's are (v2 §8).
- `STALE_AFTER`, the backup job and its script: unchanged, never run here.
- A screenshot run: no list's or pane's layout changes (the badges' state and the gate's words
  only), so rule 5's run is not owed.
- Any write into another project, the owner's registry or their user settings (rules 4, 7, 13).
- The push: the owner's.

## Verification / acceptance

- The tests above pass in `cargo test --workspace`; `cargo test --features screenshot`, both
  clippy runs with `-D warnings` and `cargo fmt --check` are clean.
- `is_replan`'s tests stand unchanged, and every title of P7's three interview rows reads as a
  planning interview while this row's and EP-61's do not.
- `crw roadmap .` reads the roadmap with no new problem and the next EP as EP-60a once this row
  is checked, EP-60b in P7's table after it.
- Rule 3's word check prints nothing for the staged diff; the commits in the house pattern, not
  pushed; the release build and the install last.

**Residual risk.** (1) **The re-read is exercised on a scratch log, not on the real job**: the
tests move a scratch log's time and watch the badge and the mark change, while the real job writes
its lines over a minute or two at its usual time, so in the real window the read should land
about ten seconds after a run's last line. It is expected to hold because the same poll watcher
has followed every project's roadmap and `HEAD` log in the real window since EP-48, and notify's
poll keeps a watch on a file that goes and comes back. It can first be seen two days after the
window starts on this build: every badge still fresh after the job's usual time with no F5, which
the next field checks read in the rolling log and the badges. (2) **A log the watcher cannot see**
(none set, or on a drive not mounted when the job was read) is watched only from the job's next
read; the day's turn reads the job meanwhile, so its reading is never more than a day old.
(3) **The interview rule reads titles not yet written**: a planning session named in other words
(a survey and rounds with no "planning" or "owner" beside "interview") would pass, and a re-cut
that names the interview as an item reads as one. The `plan` skill's rows name a planning
interview as P7's do, and a wrong yes only ends a chain. (4) **The window runs EP-58b's build**
until it restarts into this one (the install at the close asks it to, File > Install and restart
otherwise). Until then its gate does not hold the interview's rows, and EP-60a and EP-60b come
before EP-44 in any case.

> **Completion note (2026-11-27; claude-opus-5-5/max; about 36 minutes of active work and of
> wall time, 20:56 to the work commit `ce4687d` at 21:32 and the record commit after it; one
> round of two questions, answered within a few minutes).** A visual session started from the window (ledger row `8479e6a2`), split at
> pickup on the owner's word: items 1 and 3 built here, item 2 moved to EP-60b with IN-24.
>
> **Item 1, the backup job read again on its own.** The projects' poll watcher now watches the
> backup job's run log too (`Loader::watch_backup_log`, its path taken from each read of the job,
> which now carries it in `Message::Backups`), and the watcher's start is shared
> (`Loader::poll_watcher`). A change of the log owes the job a read once the log has gone
> `BACKUP_SETTLE` (10 s) without another (`read_backups_due`, asked in the window's pass beside
> the projects' owed reads, hidden too), so a run's lines are read once after its last; the read
> is `read_backups()`, file reads only. The day's turn reads the job beside every project's files
> (`Loader::read_for_the_day`), its rolling-log line ending "…, and the backup job". The test
> (`a_change_of_the_backup_log_reads_the_job_again_once_it_settles`) runs a scratch job whose log
> checked a unit 50 hours ago: the badge reads stale and the tray marks it; a newer line owes a
> read that waits for the settle, then lands, the badge fresh and the mark gone; then a newer line
> read by the day's turn alone, with no settled read asked.
>
> **Item 3, the gate and a planning interview.** `autopilot::is_interview` and
> `Refusal::Interview` ("EP-n is a planning interview, the owner's to run") in the gate after the
> re-plan's condition and in the chain plan's stop. The rule's words came from the portfolio's
> titles, read at pickup: "planning interview", "owner interview", "owner's interview",
> "interview with the owner", "planning session", "planning round", anywhere in the title, a
> mention left out after one of seven prepositions among the three words before; the word
> "interview" alone is not enough, which keeps two unchecked M rows of another project, whose
> subject is an interview, chainable, and the one unchecked M row elsewhere that `is_replan`
> missed is now held. Tests: the gate's refusal (and a title naming both refusing once as a
> re-plan), fifteen titles caught and fourteen not, the refusal's and the chain log's words, and a
> chain plan in P7's shape stopping at the interview's first session.
>
> **Verification.** `cargo test --workspace`: 911 passed in 19 binaries (908 at EP-59a, and the
> three new tests); `cargo test --workspace --features screenshot` passed (330 in the app, 523 in
> core); both clippy runs with `-D warnings` and `cargo fmt --check` clean; `is_replan`'s tests
> unchanged. The workspace tests and the fmt check were run again after the last two edits (the
> kept watch, the test titles), and `cargo build --release` passed. `crw roadmap .` reads 99 rows
> in eight phases with no problem, P7 at 0 of 7 with EP-60b after EP-60a. Rule 3's word check
> printed nothing for the staged diff. The install is the last step, after the record commit.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-27).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | EP-60 at pickup, about an hour against an S: split item 2 (the status bar's notice and its two screenshot runs) off? | Split (2) off as EP-60b | EP-60b (S) after EP-60a and before EP-44, with its brief note; this row keeps items 1 and 3; the split noted in the roadmap's header and P7's rationale |
> | 2 | IN-24, the site's two pictures, due now since v1 closed at EP-32: which row makes them? | With the status bar's row | In EP-60b's note, the real-or-mock call asked there; IN-24's line triaged and ticked |
>
> The commits are the autopilot's standing yes (crw's D-22), on in this project; the release
> build and the install are the session's last step, as the opener says (rule 6); the push is
> the owner's.
>
> **Routine choices (the session's call, logged).**
>
> 1. **The model and effort** were read from the `model` and `effort` fields of this session's
>    own transcript entries rather than asked.
> 2. **The watcher over a file-time check on the pass**: the poll that already follows the
>    roadmaps scans the log in its own thread, with no second clock in the pass; the settle of
>    10 s is longer than a scan, so a run's lines are read once.
> 3. **A watched log that goes stays watched**: notify 8.2.1's poll keeps a watch whose file goes
>    and says when it is back, but adds no watch for a missing path and returns no error, so a new
>    path is watched only once it exists, tried again at the job's next read.
> 4. **The interview rule fails safe**: a re-cut naming the interview as an item reads as one
>    (a re-cut is the owner's as a re-plan is); this row's title keeps "the v2 interview's
>    sessions", which the rule leaves alone, so that the row describing the rule does not read as
>    an interview.
> 5. **No review subagent**: an S, for which rule 6 asks none; the diff was read through, which
>    found the watch to keep when the log goes (choice 3).
> 6. **No screenshot run**: no list's or pane's layout changed; the badges and the gate's words
>    change in place.
> 7. **The test titles are synthetic**, of the portfolio's shapes, none copied from another
>    project's row.
> 8. **No owner task and no message**: the re-read is automatic and later field checks read it;
>    IN-24's reply to prime is EP-60b's, with the pictures' paths.
>
> The push is the owner's. **Next: EP-60a**, then EP-60b.
