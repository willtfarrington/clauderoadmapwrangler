---
ep: 42
title: Fixes from the P4 walk, the two guards — Make public scans the project's history against a word list before its confirmation; the doctor says an autopilot run carries the push deny rule
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-42 — Fixes from the P4 walk, the two guards

**Size:** M · **Depends on:** EP-34 (the hosting actions), EP-28 (the doctor and the Diagnostics
pane) · **Decisions:** D-24, D-22, D-3 (their addenda of 2026-11-15)

## Context

The P4 walk (EP-29) found two guards too thin, and the owner chose a fix for each there:

1. **Make public trusts the owner's memory.** It flips a repository at once and cannot be taken
   back. The dialog repeats rule 3's word on private names in the history, and nothing reads the
   history (D-24 addendum). Rule 3's word check covers one staged diff at a time, so the history of
   a project that was never checked, or was checked against an older list, goes out unread.
2. **The push guard is unseen.** Every headless run of an autopilot project is passed the deny
   rule `Bash(git push *)` through `--settings` (D-22 addendum of EP-19b). No run is known to have
   met it (risk 17), and nothing outside the tests shows the rule is still passed. A later change
   could drop it without anyone noticing.

**Session start (2026-11-15, 18:26 local time).** Chained from the window as a headless run: the
chain log's "chained EP-42 after EP-41: claude-opus-5-5 at max, the rules' choice, M's cap of 1 h
30 min" at 18:26. The prompt is the tool's opener, and `crw sessions clauderoadmapwrangler` lists
the row (`7bd1f110`, headless, EP-42, claude-opus-5-5/max, open). No owner task and no inbox
message is open. The field check read, since EP-41's record commit (18:24):

- `crash.log`: the one report of 2026-11-03, nothing newer.
- `crw notifications`: EP-41's finish at 18:26:31 and this chain at 18:26:42, nothing else.
- `crw sessions`: this row; the manual row of 13:23 still open (the idle tab EP-29 named).
- `crw autopilot`: not paused, on for this project and five others, all in mode `auto`; the
  last lines are the hold for the window's restart after EP-41 and this chain.
- The rolling log: EP-41's install asked for the restart at 18:24:52, refused while its run
  worked. The window restarted into the build of 18:24 at 18:26:32 at the gate's moment,
  carrying the held chain and its plan, and chained this run at 18:26:42.

Nothing odd to triage.

## In scope

1. **A scan before Make public** (D-24 addendum).
   - **The list.** A new registry setting `private_words`: a path, relative to the data root or
     full, as `backup_settings` is (this project's list is `private/public-check.txt`).
     `crw setting private_words <file>` sets it and `none` takes it out. `crw_core::wordcheck`
     reads the file: one fixed string a line, matched with case ignored, as rule 3's
     `grep -iFf` takes them. A line ending's `\r` and a leading byte-order mark are dropped,
     and blank lines are skipped, since an empty pattern would match every line. The list's
     `Debug` prints its count alone.
   - **The pass.** One read-only git child in its turn through the actions' `Runner`:
     `git log --branches --tags --remotes -p -U0 --cc` with a fixed format, the history GitHub
     holds as last fetched and what a later push would add, never the stash. It is given two
     minutes, where this repository's 141 commits take two seconds. Each commit's message and
     its diff's added lines are read. Added lines are those rule 3's `grep '^+'` takes, the
     `+++ b/<path>` line included, so a file's path counts too. A merge's dense combined diff
     counts a line with a `+` in its columns. A line counts once however many words it holds.
   - **The plan.** `actions::Facts` gains the list as the window read it. For Make public, when
     nothing else refuses the plan, `actions::plan` runs the pass after the refusals, so a data
     repository's history is never read. `Plan::check` holds one of three outcomes:
     - no list (unset, its file missing or unreadable, or naming no word): the dialog says so
       and goes on as today;
     - the history did not read (git failed or timed out): the second tick is asked;
     - the history read: the commits read, the words' count and each commit that hit, by short
       hash and subject with its count of lines. A subject that holds a word is replaced by
       "(its subject holds a listed word)".
     The press reads the plan again, as every action does, so the pass runs again then, and a
     commit made since the preview changes the plan, which then runs nothing.
   - **The dialog.** It lists the commits that hit, or says how many were read and that none
     hit. When any hit, or the history did not read, a second tick ("Make it public anyway: I
     have looked at these commits") is needed before the Make public button works. The board's
     confirm checks the tick too, and a changed plan clears it. The preview's words say the
     scan reads only.
   - **`crw act <project> make-public`** prints the same check under the plan's lines.
   - **D-3.** The words, the matched lines and the list's contents are never shown beyond a
     count, never logged, never written to the ledger and never passed to a child: the pass
     reads git's output in the process. Tests use a scratch repository and a scratch list. A
     screenshot run's canned runner answers the log with a synthetic history, and its list is
     synthetic.
2. **The push guard's line in the doctor** (D-22 addendum; risk 17).
   - `crw_core::doctor::push_guard(flags)` builds the launch the autopilot builds for a chained
     start: `autopilot::launch` on a stand-in project whose autopilot is on and a stand-in
     draft, with the effort left to the CLI. It then builds that launch's command line with the
     CLI's flags (`session::launch::command_line`) and reads the deny rules back from the
     `--settings` it would pass. Its levels:
     - ok when `Bash(git push *)` is among the deny rules;
     - warn when the CLI has no `--settings` and drops the rule, with the runner's words;
     - warn when the command line does not build, with its reason;
     - fail when no rule is passed and nothing says why, which is a later change dropping it.
     The words say the rule is passed and that Claude Code's honouring of it is not checked.
   - `crw doctor` prints it under Headless beside the preflight it prints last, and its fail
     fails the command. The Diagnostics pane shows it among the window's own lines after the
     headless route's line, from the window's own preflight (a note before that is read).
3. **The records.** Addenda to D-24 (the scan as built), D-22 (the doctor's line) and D-3 (the
   history scan beside rule 3's check). The public README's words on Make public, the setting
   and the doctor. The CLI's usage line for `crw setting`. This brief and its completion note.

## Out of scope

- A scan before Create a private remote, a pre-commit or pre-push hook, or a scan of the
  working tree's files: Make public is the step that publishes (v2 notes keep the hook).
- Showing which word or which line hit, in any surface (D-3).
- Rewriting a history that hits: the owner's, by hand, after the dialog names the commits.
- A supervised probe of whether Claude Code honours the deny rule (risk 17's other half).
- The dialog's own start of an autopilot project (`Board::start_headless`, which sets the guard
  itself): the doctor builds the chained start's launch, and the README's pictures and the push
  stay the owner's.
- Setting `private_words` in the owner's registry: rule 13 makes it theirs, and the closing
  question offers the command. `Settings` refuses unknown fields (`deny_unknown_fields`), so a
  build before this one refuses a registry that holds the key. This brief's install replaces
  the installed copy, and the window running now still reads with the old build until it
  restarts.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings` and
  `cargo fmt --check` pass (rule 6); a review subagent given the diff and not this session's
  reasoning reads it before the release build (an M brief).
- Core tests:
  - the list's reading (CRLF, a BOM, blank lines, case) and a count-only `Debug`;
  - the log's parse over synthetic text with a message hit, a path hit, a deleted file, a
    binary file and a combined diff;
  - the pass over a scratch repository with real git: a commit that adds a word, a commit that
    removes it, which does not hit, a merge whose resolution adds one, and a subject that holds
    one, masked;
  - the plan: no list, an unread list, an empty list, a refused plan that never scans, hits that
    ask for the tick, and a failed pass;
  - the setting's key, path and `none`;
  - the doctor's line ok with a full help, warn without `--settings`, and a fail when the rules
    hold no push rule.
- App tests: the dialog lists the hits, refuses the confirm until the tick, and clears the tick
  when the plan changes; the window's lines include the push guard after the headless line.
- A screenshot run of the Make public dialog with hits over the fixtures (rule 5; the copy and
  the pictures never enter git).
- Rule 3's word check prints nothing for the staged diff; two commits in the house pattern, not
  pushed; then the release build and the install, last.

**Residual risk.** What the session could not exercise, and why it expects it to hold.
(1) **A real Make public.** No test runs `gh` (D-29), and the owner has made nothing public
from the window yet; the scan's pass is tested with real git on scratch repositories, and the
dialog's states and tick below the egui layer and in a screenshot run. The first real use is the
owner's next Make public with `private_words` set. Its dialog should then name the history's
commit count and either no hit or the commits to look at. (2) **The real list.** The session
never prints the list; it reads only its line count, its blank lines and its line endings. A
list written for `grep -iFf` reads the same here, with blank lines skipped where grep would
match everything. (3) **The push guard in the real window.** The pane's line comes from the
window's preflight, which the launch dialog reads first; the line is tested with a recorded
help, and `crw doctor` prints it from the real CLI at this session's close. It shows the rule is
passed, not that Claude Code honours it (risk 17 keeps that half).

> **Completion note (2026-11-15; claude-opus-5-5/max; about 2 hours of wall time).** A chained
> headless run (ledger row `7bd1f110`, from 18:26; the work commit `05cd5e2` at 20:19, the record
> commit at 20:21, then the install), under the autopilot's standing yes and M's cap; one
> question was put to the owner, answered while the tests ran.
>
> **Built.**
>
> 1. **The scan before Make public.**
>    - `crw_core::wordcheck`: `List` (one fixed string a line, case ignored; a byte-order mark
>      and `\r` dropped, blank lines skipped, UTF-16 read by its mark; `Debug` prints the count
>      alone), `command` (`git log --branches --tags --remotes --root -p -U0 --cc
>      --no-renames` with a fixed format, two minutes), `read_log` and `history` (one child
>      through the actions' runner in its turn), `Scan` and `Hit` (short hash, the subject or
>      `MASKED`, the count of lines).
>    - `actions::Facts::words`, `actions::Check` (`NoList`, `Failed`, `Read`) and
>      `Plan::check`, read after the refusals for a Make public that nothing else refuses;
>      `Plan::asks_tick`.
>    - The dialog: the check's line (amber unless nothing hit), the commits that hit (the first
>      100, then a count), the second tick (`Action::TickActAnyway`, `Dialog::tick`) without
>      which the button and `confirmable` refuse, cleared by a plan shown afresh.
>    - The registry's `private_words` (`Settings::KEYS`, `Registry::private_words`, `none` takes
>      it out); `crw setting` words it with the list's count; `crw act <project> make-public`
>      prints the check and its hits. A screenshot run's canned runner answers `git log` with a
>      synthetic history, read against a synthetic list.
> 2. **The push guard's line.** `doctor::push_guard` and `PUSH_GUARD`: the stand-in's chained
>    launch from `autopilot::launch`, its command line from `session::launch::command_line`, and
>    the deny rules read back from its `--settings` (ok, a warning without `--settings` or when
>    the line does not build, a failure when the rule is gone unsaid). `crw doctor` prints it
>    under Headless and fails on its failure; the Diagnostics pane shows it after the headless
>    line.
>
> Addenda to D-3, D-22 and D-24; the README's two passages; the CLI's usage lines for `doctor`,
> `setting` and `act`.
>
> **The review (rule 6).** A subagent given the diff and this brief, not the session's
> reasoning, reported after about fourteen minutes, with nine points:
>
> 1. *Fixed.* A failed pass was worded through `actions::said`, which falls back to git's
>    output when its errors are empty, so a crashed git could have put the history's lines
>    into the dialog. It is now worded from git's errors alone, a line holding a word left out,
>    at most 300 characters (`wordcheck::failure`).
> 2. *Fixed.* A path showed only through a `+++` line, which a binary file, an empty file and a
>    mode change lack. Each file's `diff --git` (or `diff --cc`) header now counts too, so
>    `art/<word>.png` or `<word>/.gitkeep` hits.
> 3. *Fixed.* A UTF-16 list (Windows PowerShell 5.1's `>`, Notepad's "Unicode") read as no
>    match at all. It is read by its mark now, and a file with NUL bytes and no mark is refused
>    as unread, which the dialog says.
> 4. *Fixed.* A NUL inside a file git diffs as text split a record, which could drop its lines
>    or show file content as a hash. A record now starts only at 4 to 64 hex digits, and any
>    other segment rejoins the record before it with its NUL.
> 5. *Logged.* The pass's output is buffered whole, and Cancel while it reads leaves git
>    holding the children's turn until it ends (two minutes at most); a history of gigabytes
>    would be read into memory. The owner's projects are far smaller, and a data repository is
>    never read. Left for EP-32's walk and the v2 notes.
> 6. *Fixed.* `--root`, so a `log.showRoot = false` setting cannot hide the first tree.
> 7. *Logged.* The doctor's stand-in has no neighbour, no defer route and no plugin, so the
>    rule's place beside the `Edit` rules and the hook is not built there; the launch's own
>    tests build all three in one `--settings`, and the line's words claim no more than the rule.
> 8. *Logged.* The registry and the list are read on the UI thread at the dialog's opening and
>    at the press, as the backup's settings already are; a list on a sleeping drive would stall
>    the window that long, and a registry that does not read refuses the dialog with its reason.
> 9. *Taken.* Tests for 1 to 4, and a press whose unchanged plan runs, which shows the pass is
>    stable across the two reads; "1 word" in `crw setting`; one arm fewer in `Scan::summary`;
>    the disabled button's hover worded for a failed pass as well; the hits capped at 100.
>
> **Verification.** `cargo test --workspace`: 194 in `app`, 436 in `core` and the CLI's, all
> passing; `cargo test --features screenshot`: 202; both clippy runs with `-D warnings` and
> `cargo fmt --check` clean. A screenshot run of the Make public dialog over the fixtures,
> kept in the scratchpad `target\tmp\ep42\` and out of git. The real list, read by line count
> only: 27 lines, none blank, no `\r`, no mark. Rule 3's word check printed nothing for the
> staged diff. Then the release build and the install, last.
>
> **Owner decisions.**
>
> 1. **The `private_words` setting in the owner's registry** (rule 13; the closing question,
>    asked while the tests ran). Offered: an owner task once the window runs this build
>    (recommended), the session setting it now, or not now. *Outcome:* the session sets it now,
>    right after the install: `crw setting private_words
>    clauderoadmapwrangler\private\public-check.txt`. The window running this run's build of
>    18:24 refuses a registry holding the key, and it loads the registry first when this run
>    ends (`weigh_run`). So the gate after EP-42 most likely fails ("The autopilot could not
>    read the registry") and the chain ends there, without the restart a held chain would take.
>    File > Install and restart (or the tray's) then brings the window into this build, which
>    reads the key, and EP-30 starts from the window as usual.
>
> The commits are the autopilot's standing yes (D-22); the push is the owner's.
>
> **Routine choices (the session's call, logged).**
>
> 1. **The messages read too**, beside the added lines rule 3's check reads, since a commit's
>    message is published with it (the pre-flip audit of D-3's addendum read both); a subject
>    that holds a word is masked in the list.
> 2. **The refs read**: the branches, the tags and the remote-tracking branches; the stash and
>    the notes, never pushed, left out.
> 3. **Renames read as a removal and an addition**, so a renamed file's new path is read; the
>    renamed file's lines then count again in that commit, which lists it too.
> 4. **A failed pass asks for the tick**, where a missing list goes on as before: a list named
>    and a history unread is the case the owner did not expect.
> 5. **The pass runs again at the press** with the plan, as every action's plan is read twice;
>    a history changed since the preview runs nothing.
> 6. **The doctor's line beside the headless route's**, not in Readings: the rule's presence
>    depends on the CLI's `--settings`, which only the preflight knows.
> 7. **The key's name `private_words`**: it names what the list holds, so a later guard (a
>    pre-commit check, say) can read the same file.
>
> No owner task, and no message for another project. **Next: EP-30**, the phone pages' base.
