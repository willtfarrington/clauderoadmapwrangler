---
ep: 19a
title: Chain plan and the chain's moment
size: S
model: claude-fable-5-1
effort: max
status: done
---

# EP-19a — Chain plan and the chain's moment

**Size:** S · **Depends on:** EP-19 · **Blocks:** EP-23

## Context

EP-19 built the autopilot (D-22): after a headless run of the window's completes in a project
whose autopilot is on, the gate reads the project afresh and, if every condition holds, drafts
the next brief with the rule table's model and effort (the downshift included) and starts it
headless. The owner's word at EP-18b, placed in EP-19's note: until the tool can set each
session's model and effort to the owner's liking, chained or not, they choose them in a visual
tab's panel by their own judgement. The split at EP-19's pickup (the owner's choice, D-4) made
this brief: (1) **the chain plan**, the owner's model and effort for each brief a chain may run,
set in the launch dialog before the chain starts, the rules' choice the default; (2) **the chain's
moment**, a notification of its own when a chain stops.

**What the first real chain found (2026-11-06, one of the owner's private projects).** Two
headless runs went well: the owner set the first run's model and effort in the launch dialog
(Opus 5.5 at max), and the gate chained the second brief with the rules' choice, fable at xhigh,
where the owner had wanted the same Opus at max; the chain then ended at a re-plan, as the gate
should. The owner's note at this session's start: configure the number of runs a chain makes, or
set a project- or phase-wide model and effort. The second is the rule table's job
(`[project.launch]` and `[[project.rule]]`, EP-10), hand-written today; the first is the plan's
to give.

**What exists.** The dialog (`app::launch::Dialog`) holds a `prompt::Draft` of the launched
brief: its `rules::Resolution` (each value a `Choice` with its `Source`), its `Parts` and its
prompt, regenerated as the pickers change. The chain is `app::autopilot`: `read_gate` reads
`crw_core::autopilot::read` on a worker thread, which drafts the next brief with `prompt::draft`;
`gated` queues the start (`autopilot::launch` takes the draft's resolution); `start_queued` and
`chain_started` log the start or the refusal; each line goes through `Board::log_chain`, and a
status-bar notice says it (`notify`, `warn`). The notifications' rule table is
`crw_core::notify::Kind` with its `Channel`; a toast is `app::toasts::Toast` with a `Target` its
Show leads to, delivered by `Toasts::deliver`, which holds it back in the quiet hours or, in
front, hands it to the status bar. `Roadmap::rows()` gives the rows in table order and
`rules::resolve` the resolution of any row. A gap found while reading: `prompt::draft` words the
opener for the project's launch mode (Visual by default), so a chained run, headless whatever
the mode, was told it ran "into VS Code, whose Claude Code panel chooses its model and effort".

**Session start (2026-11-06, 18:58 UTC).** Launched from the window as a visual launch
(rule 11): the ledger row `4c6a2628`, visual, EP-19a, open (`crw sessions
clauderoadmapwrangler`). The opener suggested Fable at high; the owner runs it at Fable 5.1 with
max effort, chosen in the tab's panel. The tree is clean at `3762bbc` but for the owner's tick of
OT-18 and their inbox message IN-7, both uncommitted. No owner task is open; IN-7 is triaged below.

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-06)

1. **The plan lives in the window**, like a held chain: from Start headless until the chain
   ends, the project's autopilot is turned off, or a new headless launch of the project replaces
   it; shown in the Sessions pane's Autopilot section; gone with the window, as the chain is. No
   registry key, so the builds before read the owner's registry still.
2. **Every change of the chain's state notifies**, as a kind of its own: a chained start, a hold
   (once per change of its words), an end.
3. **A Chain box per brief** gives the number of runs: unticked, the chain ends before it.
4. **IN-7 becomes EP-19b (M)**, "Launch settings in the window", right after this brief: IN-7's
   four parts, the autopilot's limits in the window and `crw set`, and "keep for this project"
   (or "for this phase", a `[[project.rule]]`) beside the dialog's Model and Effort pickers, the
   standing model and effort the owner asked for. This brief stays S on the plan and the moment.

## In scope

1. **The plan** (`crw_core::autopilot::Plan`): the next unchecked rows in table order after the
   launched brief, up to the project's `daily`, each with `rules::resolve` over its row (the
   downshift included) as its default; the list stops at a row the gate would refuse on its own
   account (a re-plan, a size over `max_size`, no size), named with the reason; an owner task
   gated before a row is noted on it. The owner's choice for a row is the model, effort and
   ultracode as they left its pickers; a row they did not touch takes the rules' choice when the
   chain reaches it, the downshift judged then. Each row has a Chain box, ticked by default:
   unticked, the chain ends before it (the number of runs).
2. **The dialog** (`app::launch`): a Chain plan section under the prompt and the choices, shown
   for a project whose autopilot is on in Headless (a chain follows a headless run only, D-22),
   one row per planned brief: the box, the id and size, the title, the Model and Effort pickers
   and the Ultracode box as the main choices offer them, the reason on hover; a line below says
   how the chain takes them and the day's caps; in Visual, one line says to switch to Headless.
3. **The chain takes the plan** (`app::autopilot`): Start headless keeps the dialog's plan in
   the window (`Pilot`), keyed by project; `gated` looks the next brief up: a row the owner
   chose is drafted with their choice (`Draft::for_chain`, the prompt regenerated, headless
   wording), a row left alone with the rules', a row left out ends the chain, a brief the plan
   does not list (a row added since) takes the rules' choice and the log says so; the log line
   says whose choice it was. The plan goes when the chain ends, when the project's autopilot is
   turned off, or when a new headless launch of the project replaces it; a held chain keeps it.
   The Autopilot section of the Sessions pane shows each plan. The headless wording for every
   chained draft (the gap above).
4. **The chain's moment** (`crw_core::notify::Kind::Chain`, a toast): a notification of the
   chain's own kind at each change of its state, as the owner decides at pickup: a chained start,
   a hold (once per change of its words), an end (a refusal that ends it, a cap, a start the
   runner refused, a brief left out of the plan). Its Show brings the Sessions pane's All tab
   forward, where the chain log is (`Target::ChainLog`). Held back in the quiet hours and while
   the window is in front as every toast is, when the status bar says it instead; logged as
   `chain` in the notifications' log, which `crw notifications` prints.
5. **Records**: D-22 and D-21 addenda, the README's paragraph, the roadmap's EP-19a link, IN-7's
   disposition, a "From EP-19a" note where a later brief needs one, v2 notes, an owner task.

## Out of scope

- A plan kept in the registry or across a restart of the window (v2 notes): a chain does not
  outlive the window either.
- `crw autopilot` printing the plan: it lives in the window.
- IN-7's window controls for the permission mode and the autopilot's limits, and a standing
  model and effort set from the dialog: placed at pickup (below).
- Any real run of the CLI, or any press of Start headless outside a screenshot run (D-29, rule 5).

## Verification / acceptance

- **Unit tests**: the plan over a fixture roadmap (the rows after the launched one, the limit,
  the stop at a re-plan and at an L, the resolution per row, a due task noted); the plan's
  answer for a brief (chosen, the rules', left out, not listed); `Draft::for_chain` (the owner's
  choice with `Source::Owner`, the headless wording, the rules' choice kept); the rule table with
  `Kind::Chain`; the chain toast's title, text, target and tone; the dialog's section pressed
  through `testing::Screen` (a row's box, a picker's change recorded as the owner's choice).
- **Build checks**: `cargo test`, `cargo test --features screenshot`, `cargo clippy --all-targets
  --features screenshot -- -D warnings`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt
  --check`.
- **Screenshot runs**: the launch dialog in Headless with the plan (both themes), the Autopilot
  section with a plan, the popup with a chain's moment; each picture looked at.
- **The release**: `cargo build --release` once the owner has quit the window from the tray, then
  `scripts\install-shortcut.ps1`.
- **Rule 3's word check** on the staged diff prints nothing.

> **Completion note (2026-11-06; claude-fable-5-1/max; about 1 h 25 min to the commit, with the
> owner's two rounds and one quit of the window).** One session in the VS Code extension from
> 18:58 UTC, launched from the window (ledger row `4c6a2628`, visual, EP-19a). It covered the
> reading, the pickup round (four questions), this brief, IN-7's triage and the EP-19b row, the
> code and its tests, the screenshot runs, the records, the closing round and the release.
>
> **Built.**
>
> - `crw_core::autopilot`: the plan (`Plan`, `Planned`, `Stop`, `Answer`, `plan`,
>   `left_out_words`), and `capped_words` beginning "capped EP-n:" so a notification's title
>   names the brief; `crw_core::rules::Picked` and `Resolution::pick`; `prompt::Draft::for_chain`
>   (headless wording, the owner's choice in place of the rules'); `notify::Kind::Chain`.
> - The window: the dialog's Chain plan section (`launch::plan_section` and `plan_row`, the
>   pickers factored into `model_picker` and `effort_picker` for both, `Dialog::set_mode`);
>   `Pilot::plans`, kept at Start headless, read in `gated`, dropped when a chain ends, the
>   autopilot goes off, a start is refused or a new launch replaces it; `Board::chain_moment`
>   (a log line and a `Toast::chain` with `Target::ChainLog`, amber for a hold or an end, violet
>   in the popup); the Autopilot section's plan lines; the screenshot setup
>   `launch=<project>:headless` and a canned plan.
> - The records: D-22 and D-21 addenda, the README's paragraph, the roadmap's EP-19a link, the
>   EP-19b row and note, IN-7's disposition, three v2 items, OT-19.
>
> **Verification.** `cargo test` 417 passed (100 + 1 + 1 + 2 + 1 + 275 + 2 + 7 + 6 + 11 + 11,
> against EP-19's 413), 419 with `--features screenshot`; both clippy runs and `cargo fmt
> --check` are clean. The new tests: the plan over the house fixture and a scratch roadmap (the
> rows after the launched brief, the daily cap, the stop at a re-plan and at an L, the project's
> rule per row, a due task noted, the four answers, the words); `Draft::for_chain`; the rule
> table with `Kind::Chain`; the chain toast's title, text, target and tone; the dialog's plan
> section drawn through `testing::Screen` with its box pressed and a picked row's words; the
> efforts offered per model. Screenshot runs, kept in the session's scratchpad: light and dark
> `autopilot,launch=harbor:headless` (the dialog's Chain plan: EP-5's row with its box, pickers
> and due-task line, the end at EP-6), light `autopilot` (the Autopilot section's plan line),
> dark `popup` (the chain's notification). The parser reads the roadmap with the new row: 46
> rows, next EP-19a, no problem. The release was built once the owner had quit the window,
> then `scripts\install-shortcut.ps1` refreshed both shortcuts. Rule 3's word check found
> nothing.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-06).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Where the plan lives | In the window | `Pilot::plans`; no registry key |
> | 2 | The chain's moments | Every change of state | `Kind::Chain` for a start, a hold and an end |
> | 3 | The number of runs | A box per brief | `Planned::chain`; unticked, the chain ends before it |
> | 4 | IN-7's home | A new row EP-19b (M) | EP-19b after EP-19a; IN-7 ticked |
> | 5 | The release build | Quit the window, build and install | Built and installed |
> | 6 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. A row the owner leaves alone takes the rules' choice resolved when the chain reaches it,
>    the downshift judged then, not the plan-time one; a touched row keeps all three pickers as
>    shown.
> 2. The list stops at the first row the gate would refuse by the row's own facts (a re-plan, a
>    size over `max_size`, no size); owner tasks due before a row are noted, not a stop, since
>    they may be ticked by then.
> 3. A brief the plan does not list runs with the rules' choice, and the log says "not in your
>    plan".
> 4. Every chained draft is regenerated headless: before, a chained run of a project in the
>    default Visual mode was told that VS Code's panel chooses its model and effort.
> 5. The chain's moments reach the popup through the toasts' pass (the quiet hours, in front,
>    the log), so the status-bar notices the chain wrote before are the in-front form of the
>    same moments.
> 6. The plan's section sits under the two columns at full width and takes its room from the
>    prompt editor; in Visual it is one line.
> 7. The screenshot setup's canned plan holds the owner's choice while the dialog reads the
>    fixture roadmap afresh, so the picture's section and dialog differ on purpose.
>
> **Notes for later sessions.**
>
> - A plan lives in the window only: a restart loses it with the chain, as a held chain is lost.
> - EP-19b's "keep for this project" beside the pickers shares `launch::model_picker` and
>   `effort_picker` with the plan's rows.
> - No schema or registry key changed: the ledger and the registry need no migration.
>
> No messages for other projects. Owner task: OT-19, before EP-19b. **Next: EP-19b** (launch
> settings in the window), launched from the window.
