---
ep: 31b
title: The overall daily cap in the window — the Autopilot section says today's chained runs against `autopilot_daily_total` and sets it beside the per-project pickers; the phone's autopilot block says the same
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-31b — The overall daily cap in the window

**Size:** S · **Depends on:** EP-31 (the phone's pages as they stand) · **Decisions:** D-22, D-28
(and D-3, D-10, D-29 as standing)

## Context

The gate refuses a chain at two daily caps (D-22): each project's chained runs a day
(`Limit::Daily`, `[project.autopilot] daily`, 2 by default) and the chained runs a day in all
projects (`settings.autopilot_daily_total`, 4 by default; `Refusal::DailyTotal` over
`Facts::chained_total` in `core/src/autopilot.rs`, counted by `chained_on` from the chain log's
latest 500 lines, the local day). The Sessions pane's Autopilot section (`Board::autopilot_section`,
`app/src/autopilot.rs`) shows and sets the first with each project's pickers; the second only
`crw setting autopilot_daily_total <n|default>` sets and only `crw autopilot`'s first line says.
On 2026-11-16 the owner found from a chat that their chain's real ceiling was the total they could
not see (the per-project pickers already at 8), and raised it from a terminal (EP-31's field
check, finding 1; the row added at EP-31 by the owner's choice).

**Session start (2026-11-17, 11:33 local time).** A chained headless run from the window: the
chain log's "chained EP-31b after EP-31: claude-opus-5-5 at max, not in your plan, the rules'
choice, S's cap of 45 min; it holds at 80 % of the session (five-hour) window, your mark for this
chain" at 11:33 (the row was added after the chain's plan was made, so the rules chose, as the
EP-19a addendum says). The prompt is the tool's opener, and `crw sessions clauderoadmapwrangler`
lists the row (`6a380309`, headless, EP-31b, claude-opus-5-5/max, open). No owner task and no
inbox message is open. The field check read, since EP-31's record commit (2026-11-17, 11:30):

- `crash.log`: the one report of 2026-11-03, nothing newer.
- `crw notifications`: EP-31's finish at 11:33:01 and this chain's start at 11:33:11.
- `crw sessions`: this row; EP-31's, completed in 11 h 37 min, its one question answered in
  10 h 52 min.
- `crw autopilot`: not paused; on for this project and five others, all in mode `auto`; 8
  chained runs a day in all projects. The chain log's 11:33 lines: `the restart into the
  installed build waits: an ended run's tab offers Commit: chat: for @clauderoadmapwrangler, the
  three-session chain paused…`, then this run's chained line.
- The rolling log: 00:00, the day's turn (every project read again, the daily export written);
  11:13, EP-31's answer; 11:31, EP-31's install asked for a restart, refused while its run worked;
  11:33, the project read again after its chained run ended.

One finding, triaged as an inbox message is:

1. **A chat's ended run holds the gate's restart.** `Live::offers_commit` (`app/src/sessions.rs`)
   is true for any completed run that was not told to commit and is not a migration, a chat
   conversation's run included; but a chat's run is the Chat pane's, has no tab in the Sessions
   pane and no Commit button (`Live::is_chat`). `Board::gate_refusals` (`app/src/restart.rs`)
   lists it as "an ended run's tab offers Commit", so the gate's restart into an installed build
   waits for as long as the chat's run stays in the window: the window still runs EP-30b's build
   of 2026-11-16 21:52, and EP-31b was chained by it. Every chain after a chat would go on in an
   old build until the owner's File > Install and restart. The fix is one condition and a test,
   on the gate's moment (D-22): rolled in. It counts only once the window runs this build, so
   this run's own install meets the old build's refusal at the gate's moment.

## In scope

1. **The day's count, as the gate counts it** (core). `autopilot::DAY_LINES` names the chain
   log's 500 latest lines the gate reads, and `autopilot::chained_in_all(events, day)` counts the
   chained starts on a local day in every project; the gate's read uses the same constant.
2. **The window reads it** (`Pilot`). `Board::read_pilot` reads the chain log's `DAY_LINES`
   latest lines once (the section keeps its 8 newest), counts the day's chained starts in all
   projects (`Pilot::chained_today`, none while the log does not read), and takes
   `autopilot_daily_total` from the registry with the Pause and the marks (`Pilot::daily_total`),
   every 30 s and after each line the window writes, hidden too.
3. **The section's line** (D-22). Above the projects' lines: "N of [M] chained runs today in all
   projects", M a picker of its own (0 to 20 offered, the default 4 labelled, a value set
   elsewhere offered as it stands), then words saying both caps hold: a chain ends at this total
   or at its project's own runs a day, whichever it meets first. At or over the total the line is
   amber and says the total is reached for today. The picker writes through
   `Registry::set_setting("autopilot_daily_total", n|default)`, the write `crw setting` makes (the
   default taken out of the registry, as the other pickers do), on the owner's click alone (rule
   13); the status bar says the new total. Each project's runs-a-day picker says on hover that the
   total holds too.
4. **The phone's block** (D-28). `published::Autopilot` carries the count and the total;
   `pages::AutopilotView` says the same line, `templates/list.html` draws it under the projects'
   line, read-only (registry settings are not a phone action), its class `warn` once reached.
5. **The roll-in.** `Live::offers_commit` is false for a chat's run, so the gate's restart no
   longer waits for one.
6. **The screenshot feature.** The `autopilot` setup's canned count, counted from its canned log
   by the same function, so the picture shows the line (rule 5).
7. **The records**: an addendum to D-22; the README's words where they describe the Autopilot
   section; this brief and its completion note.

## Out of scope

- Each project's own count in its line ("n of its runs a day today"): not asked.
- The total from the phone (D-28: registry settings stay the desk's).
- `crw autopilot`'s first line, which keeps the total alone.
- A chain the total ended going on when the total is raised: the gate ends a chain at a daily
  cap rather than holding it (D-22), and that stays.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings` and
  `cargo fmt --check` pass (rule 6). An S brief: no review subagent is required.
- **Tests:**
  - core: the day's count in every project, yesterday's and the chain's other lines left out;
  - the board: the count and the total read from a ledger and the registry; the section's line
    with its picker, and the reached line; the picker's action writes the setting and its default
    takes it out, the window's total following;
  - the snapshot and the page: the block carries the count and the total; the list page says the
    line, `warn` when reached, and a count that did not read;
  - the roll-in: a chat's completed run offers no Commit and holds no gate's restart.
- A screenshot run with `autopilot` over the fixtures and over a copy of one project's roadmap
  files (rule 5), with `pages=` for the phone's list page; each read in the scratchpad, the brief
  naming what was checked, never the projects.
- Rule 3's word check prints nothing for the staged diff. Then two commits in the house pattern,
  not pushed, and the release build and the install last.

**Residual risk.** What the session could not exercise, and why it expects it to hold.

1. **The picker on the owner's registry.** The write is `set_setting`, the one `crw setting` and
   the Pause make, tested on a scratch registry; the owner's registry holds the total of 8 they set
   from a terminal, which the picker shows as its value (a total it does not list would be offered
   as it stands).
2. **The window's count beside the gate's.** Both count with the same function over the same 500
   lines. The window's is at most 30 s old (read every 30 s and after each line it writes), so it
   turns with the local day within half a minute.
3. **The roll-in in the real window.** It takes effect once the window runs this build: until
   then the running build's gate keeps waiting on the chat's run, and File > Install and restart
   (or the tray's) takes the new build.
4. **The phone's line in a real browser.** Drawn by the server inside the autopilot part, which
   the script already swaps whole; no script change.

> **Completion note (2026-11-17; claude-opus-5-5/max; about 35 minutes).** A chained headless
> run from the window (ledger row `6a380309`, from 11:33), the work commit and the record commit
> after it, then the release build and the install, whose restart the window refuses while this
> run works. No question went to the owner: nothing outward-facing, nothing in the owner's
> settings and no change of scope arose, and the field check's finding was at most an S on this
> brief's surface, which D-13's rule rolls in.
>
> **Owner decisions.** None arose. The commits are the autopilot's standing yes (D-22); the push
> is the owner's.
>
> **Built.**
>
> 1. **The day's count as the gate counts it** (core): `autopilot::DAY_LINES`, the gate's 500
>    lines of the chain log, now named, and `autopilot::chained_in_all`, the day's chained starts
>    in every project on the local day.
> 2. **The window's read**: `Pilot::daily_total` from the registry with the Pause and the marks,
>    and `Pilot::chained_today` from one read of the chain log's 500 latest lines (the section's 8
>    kept from it), every 30 s and after each line the window writes, hidden too.
> 3. **The section's day line** above the projects' lines: "N of [M] chained runs today in all
>    projects; a chain ends at this total or at its project's own runs a day, whichever it meets
>    first", the picker offering 0 to 20 (the default 4 labelled, a total set another way offered
>    as it stands); at the total the line turns amber with "The total is reached for today: a
>    chain ends at its next gate." The picker's `Action::SetDailyTotal` runs
>    `Board::set_daily_total`, through `set_setting` as `crw setting` writes, its default taken out
>    of the registry, the new total said in the status bar and the rolling log. Each project's
>    runs-a-day picker says on hover that the total holds too.
> 4. **The phone's block**: `published::Autopilot` carries `daily_total` and `chained_today`;
>    `pages::AutopilotView` says the section's words (`autopilot::day_line`) with the class `warn`
>    once reached; `templates/list.html` draws the line, with no form.
> 5. **The roll-in**: `Live::offers_commit` is false for a chat's run, so the gate's restart no
>    longer waits on one.
> 6. **The screenshot feature**: the `autopilot` setup's canned count, from its canned log.
> 7. **The records**: an addendum to D-22; the README's passages on the phone's block and the
>    Sessions pane's section.
>
> **Verification.**
>
> - **The gate**: `cargo test`: 243 in `app` (242 before), 448 in `core`, the CLI's and the rest,
>   all passing; `cargo test --features screenshot`: 251 in `app` (250 before); both clippy runs
>   with `-D warnings` and `cargo fmt --check`: clean. An S brief, so no review subagent; the
>   session read its own diff once while the gate ran.
> - **Screenshot runs**, kept in `target\tmp\ep31b\`, out of git: the `autopilot` setup over the
>   fixtures and over a copy of one project's roadmap files, each with `pages=`. In both pictures
>   the day line sits under the header and above the projects' lines, wrapping in the narrow
>   Sessions pane ("1 of [4] chained runs today in all projects; …", the canned log holding one
>   chained start today). The phone pages (26 and 10) were checked with the standard library's
>   parser: all balanced, `/m/app.js` the only script, the list page's block saying the same line
>   with its one form, the Pause.
> - **Rule 3.** The word check over the working diff found nothing, and so did the check on the
>   staged diff.
>
> **Routine choices (the session's call, logged).**
>
> 1. **One read of the chain log's 500 lines** every 30 s for the section's lines and the count,
>    rather than a count query of its own: the lines the gate counts, at the cost of a scan the
>    gate already makes after each run.
> 2. **The count and the total kept in the window** (`Pilot`), read with the log, never each
>    frame; the phone's snapshot takes them from there.
> 3. **The total's choices**: 0 to 6, then 8, 10, 12, 16 and 20, since the projects share the
>    total; `crw setting` still takes up to 100.
> 4. **The line shown with the autopilot off everywhere** too, so the total can be set before a
>    chain.
> 5. **No chain-log line for a change of the total**, as the per-project limits have none; the
>    status bar and the rolling log say it.
> 6. **The field check's finding rolled in** (D-13: at most an S, on the gate's moment, D-22): one
>    condition and a test.
>
> **The window's build.** The window still runs EP-30b's build of 2026-11-16 21:52, since the
> chat's ended run held the gate's restart at 11:33. That build reads the gate after this run: if
> it chains EP-31a, the restart into this build waits on the chat's run again and EP-31a runs in
> the old window, as EP-31b did, which nothing in EP-31a needs. File > Install and restart (or the
> tray's), at any moment no headless run works, brings the window onto this build, whose gate no
> longer waits on a chat.
>
> No owner task, and no message for another project. **Next: EP-31a**, Enable remote view.
