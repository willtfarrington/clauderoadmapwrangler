---
ep: 43a
title: The run's cap as a guard — S's cap of 75 minutes, and a headless run told its cap, to split a row that will not fit it when it writes its brief
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-43a — The run's cap as a guard

**Size:** S · **Depends on:** EP-43 · **Decisions:** D-22, D-4 (and D-3, D-10, D-29 as standing)

## Context

EP-43's headless run was stopped by the autopilot at 44.7 minutes of work under S's cap of 45,
its code written and its checks and release build passed, the review, the completion note, the
commit and the install owed: about ten minutes from its end. The session read its own clock for
the first time at minute 44. The D-4 addendum of 2026-11-22 measured why: a code brief here
carries about twenty minutes whatever its size (read-first, the brief, the checks, the release
build, the close), so an S under that cap had about twenty-five minutes of work.

**What the code says** (read at pickup).

- `crw_core::autopilot::cap` (`crates/core/src/autopilot.rs`) gives S 45 minutes, M 90 and L
  three hours, a brief with no size taking L's. `Cap::new` makes a run's cap and its working
  clock, and `Cap::words` says it through the ledger's `length` ("M's cap of 1 h 30 min"). Every
  surface that names a cap takes those words: the chain log's chained line (`chained_words`) and
  capped line (`capped_words`), the run's tab in the Sessions pane, the phone's start page
  (`phone::run_words`) and its session pages (`published.rs`), and the chain's notifications.
- `Sessions::check_caps` (`crates/app/src/sessions.rs`) ticks each capped run's clock, a request
  waiting for the owner stopping it, and stops the run gracefully at its cap. A run is capped
  (`Run::capped`) when it is headless and its project's autopilot is on: the dialog's Start
  headless (`Board::start_headless`), the phone's start (`phone_launch`) and every chained start
  (`start_queued`); a migration, a visual launch and a chat never are. The clock starts once the
  session has started (`Live::new`), and each resume has a cap of its own.
- The opener (`crw_core::prompt`) is built from `Parts`: `Parts::choose` sets what the session
  runs with and this project's paragraph on its build and install (`build`, EP-37a), and a
  project's template names the parts as placeholders. It says nothing of a cap. A draft is made
  when the launch dialog opens, and made again at each choice until the owner edits the prompt;
  at the gate's moment for a chained start (`Draft::for_chain`), which starts within seconds, or
  about 75 s while an idle tab is confirmed; and at the phone's start, which starts at once.
- The launch dialog names no cap: its Headless note says where the run goes and that the tray
  keeps it. The cap shows in the run's tab only once it works.
- The README's autopilot paragraph says "S 45 min, M 90 min, L 3 h".

**Session start (2026-11-23, 17:23 local time).** A visual launch from the window, the owner's
Start and Open in VS Code: the rolling log's line at 17:23:14, the prompt the tool's opener, and
`crw sessions clauderoadmapwrangler` lists the row (`57b5c79f`, visual, EP-43a, open). The panel
runs claude-opus-5-5 at max effort, as the launch suggested. OT-30 is open, gated before EP-54,
not this brief's; no inbox message is open. The field check read, since EP-43's record commit
(2026-11-22 18:10): `crash.log`, one panic at 04:00 today, egui-wgpu's renderer failing after the
graphics driver's reset, which left the window closed until the owner started it at 14:51; the
owner's commit `6b65b59` (a manual session at 14:52, outside a brief) now starts the window again
after such a panic, installed at 15:08, so nothing is left to triage. The rolling log: the
install's restart into EP-43's build refused at 18:10 while two sessions awaited their first
prompt, and taken at 18:45; the day's export; the panic; the start at 14:51 and the restart at
15:08; another project's visual launches; this launch. No line of EP-43's measurement yet, since
no form has come from the phone. `crw notifications`: another project's visual sessions waiting
and finishing, with their repeats. `crw autopilot`: not paused, six projects on in mode `auto`,
the latest lines EP-43's cap and its chain's end at 17:10 yesterday. Nothing odd.

## In scope

1. **S's cap of 75 minutes.** `autopilot::cap` gives S 75 minutes; M's 90 and L's three hours
   stay until EP-53 has measured. The words follow from `Cap::words` ("S's cap of 1 h 15 min"),
   so the chain log's lines, the run's tab, the phone's pages and the notifications say the new
   figure as they are; the tests and the canned lines that spell the old one follow, and so does
   the README's autopilot paragraph.
2. **The opener tells a capped run its cap.** A new part, `cap` (a template's `{cap}`, as
   `{build}` is), holds a paragraph only for a headless run of a project whose autopilot is on:
   the brief's cap in `Cap::words`'s words, counted in working time from the run's start at a
   local time (`HH:MM`), a request's wait for the owner left out; that the run's work stays in the
   tree when the window stops it there; and that when the session writes its brief it judges
   whether the row fits the cap with its checks and its close, and if it will not, splits it
   there: does the first part, proposes the rest as a row in its completion note, and closes. It
   follows the launch's paragraph. The time is the clock's when the prompt is made, stamped again
   at the dialog's Start headless (the editor taking it unless the owner has edited the prompt)
   and at each chained or phone start (`Draft::for_chain`). A visual launch, a project whose
   autopilot is off and a migration say nothing of it; this project's build paragraph is kept as
   it is.
3. **The dialog names the cap.** The launch dialog's Headless note says that the window stops the
   run at its cap when the project's autopilot is on.
4. **A D-22 addendum** (2026-11-23, EP-43a).
5. **The warning parked.** A warning sent into a run that nears its cap goes to `v2-notes.md`
   with what this brief found.

## Out of scope

- A warning sent into a run that nears its cap (parked, item 5).
- M's and L's caps, which EP-53 measures; the budget-estimate flag.
- A resumed run's cap: each resume has its own, and its message is the owner's word or an
  answer, never an opener.
- Any write into another project.

## Verification / acceptance

- Unit tests: `cap` gives S 75 minutes, M's and L's unchanged; the working clock's test at S's
  new figure; `Cap::words` and the chained line for an S.
- The opener: a headless draft of a project whose autopilot is on carries the paragraph with the
  cap's words and the stamped time, after the launch's paragraph; a visual one, or one whose
  autopilot is off, carries none; a new stamp changes it; `{cap}` renders in a template; the tool's
  own opener keeps its build paragraph.
- The dialog: Start headless stamps the time into an unedited prompt and leaves an edited one as
  written; the Headless note names the cap with the autopilot on, and not without it.
- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings`, `cargo
  fmt --check`; a screenshot run of the launch dialog on Headless with harbor's autopilot on;
  `crw prompt --headless` on this project's folder, which renders the paragraph from the real
  roadmap; the release build, and the install last.
- No tracked file names a managed project, the owner's login or the machine (rule 3); the word
  check prints nothing on the staged diff.

**Residual risk.** No session here runs the real CLI (rule 5), so whether a capped run heeds the
paragraph, judging its row at the brief and splitting one that will not fit, is unexercised until
the next headless run of a project whose autopilot is on; a run that misjudges its row still meets
the cap, now at 75 minutes for an S. After a split the gate sees a completed brief and the chain
goes on to the next row while the rest waits as a proposal in the completion note, which the next
session reads first; a next row that builds on the rest is that session's to notice. The time the
opener says is the clock's when the prompt is made, stamped again at Start headless and at each
chained or phone start, a few seconds (at most about 75 s, while an idle tab is confirmed) before
the cap's clock starts, so it errs early; a prompt the owner edited keeps the time it had. The
paragraph follows the autopilot's state when the dialog opened, as the commit rule does, so the
autopilot turned on or off while the dialog is open gives a run whose cap and opener disagree; a
project's template that does not name `{cap}` says no cap. The screenshot run and `crw prompt`
show the words; the next re-plan's walk reads how the first capped runs used them.

> **Completion note (2026-11-23; claude-opus-5-5/max, a visual session; 22 minutes from the launch
> to the work commit, the install a minute after it).** Launched from the window into VS Code at
> 17:23 (ledger row `57b5c79f`), the panel running the model and effort the launch suggested. The
> work commit (17:45) and the record commit, then the install, whose restart the window took at
> once: the rolling log's "Restarted into the installed build of 2026-11-23 17:44" at 17:45:37.
> This note's time was corrected in a docs commit after the install, the first figure having been
> written before the end.
>
> **Owner decisions.** None arose: nothing outward-facing, nothing in the owner's settings or
> another project, and no change of scope; the brief note settled the rule the opener states. The
> commits are the autopilot's standing yes (D-22); the push is the owner's.
>
> **Built.**
>
> 1. **S's cap of 75 minutes** (`autopilot::cap`), M's 90 and L's three hours kept;
>    `autopilot::cap_words`, the words `Cap::words` gives, from the size alone. The chain log's
>    lines, the run's tab, the phone's pages and the notifications say "S's cap of 1 h 15 min" as
>    they were.
> 2. **The opener's paragraph** (`Parts::cap`, a template's `{cap}`), after the launch's, for a
>    headless run of a project whose autopilot is on: "The autopilot is on for this project, so
>    the window stops this run at S's cap of 1 h 15 min of working time, counted from its start at
>    17:44 local time, the time a request waits for the owner's answer not counted; what it has
>    done then stays in the tree, uncommitted. When you write the brief, judge whether the row
>    fits that cap with its checks and its close. If it will not, split it there: do the first
>    part, propose the rest as a row in the completion note, and close." `Parts::sized` carries
>    the parsed size, `Parts::time` the start (`prompt::clock`, read when the prompt is made),
>    `Parts::started_at` stamps it again, and `Draft::for_chain` stamps each chained or phone
>    start.
> 3. **The dialog**: `Dialog::stamp_start` at Start headless (`Board::start_headless`), the editor
>    taking the stamp unless the owner has edited the prompt, a migration's left alone; the
>    Headless note says "and the window stops it at S's cap of 1 h 15 min of working time, the
>    autopilot being on" while it is.
> 4. **The records**: a D-22 addendum; the README's autopilot paragraph; the warning into a run
>    near its cap parked in `v2-notes.md` with a resumed run's cap; the canned lines of the
>    screenshot feature and of a test at the new figure.
>
> **Verification.** `cargo test`: 266 in `app`, 478 in `core`, the CLI's and the integration
> suites, all passing; `cargo test --features screenshot`: 274 in `app`; both clippy runs with
> `-D warnings`: clean; `cargo fmt --check`: clean once `cargo fmt` had wrapped three of the new
> assertions. An S brief, so no review subagent: the session read its own diff while the tests
> ran. A screenshot run (`autopilot=lantern,launch=lantern:headless`, an S over the fixtures,
> kept in the scratchpad) shows the paragraph under the launch's line and the note naming the
> cap. `crw prompt clauderoadmapwrangler --headless`, from the release build, renders the
> paragraph from the real roadmap with this project's install paragraph kept, and `--visual`
> renders none. The release build passed; the word check printed nothing.
>
> **Session's calls.** The paragraph after the launch's, where the run's facts are, rather than
> among the rules; the time read in `prompt::parts` rather than a new parameter for `draft`'s
> eight callers; the time as `HH:MM` local, the date being the launch's line's; the dialog's note
> (In scope 3), since the brief note's "the launch dialog where it names the cap" found a dialog
> that named none; `cap_words` beside `Cap::words` so that the opener and the log say one thing.
>
> **Found and logged, not changed.** (1) `crash::tests::the_hook_writes_a_panicking_threads_report`
> installs the process's panic hook while the other tests run, so another test failing at that
> moment fails it too: it did once, beside this brief's dialog test, before that test's "3 h 00
> min" was fixed; filtering the report by the probe thread's name would close it, at the next
> brief that touches `crash.rs`. (2) The ledger's `length` writes three hours as "3 h 00 min", so
> an L's cap reads "L's cap of 3 h 00 min", as it did before.
>
> No owner task and no message to another project.
