---
ep: 19b
title: Launch settings in the window
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-19b — Launch settings in the window

**Size:** M · **Depends on:** EP-19a · **Blocks:** EP-19c, EP-37, EP-23

## Context

Today a project joins a chain only through a hand edit of the registry (IN-7, the owner's message
after OT-18). The gate (D-22) refuses a project whose headless runs would wait for the owner at
each shell command (`Refusal::Waits`: a permission mode of `acceptEdits`, `manual`, `plan` or
`default` with no allow rule in the project's `.claude` settings or the user's). Without allow
rules, the way past it is `permission_mode = "auto"` under the project's `[project.launch]`, which
no window control and no `crw` command writes. The autopilot's limits (`max_size`, `daily`,
`max_budget_usd`) are hand-written too. EP-19a placed IN-7's four parts in this row, with the
limits and the owner's wish for a standing model and effort. At pickup the owner split the model
and effort off into EP-19c (below).

**What `auto` is** (Claude Code's permission-modes and permissions pages, read 2026-11-09). A
classifier model reviews each action that is neither a read nor an edit in the working folder. It
blocks an action that goes beyond the request, targets unrecognised infrastructure or looks driven
by content the session read: downloading and running code, sending data out, a force push,
`git reset --hard`, destroy commands. Explicit ask and deny rules still apply. A plain push to a
branch of the repository the session works in is among what it allows by default. The autopilot's
runs are told "Never push: the push stays the owner's" (the commit rule of `prompt`), and nothing
else keeps a chained commit local. A deny rule `Bash(git push *)` refuses `git push` as written,
inside compound commands too and in auto mode, but not a push written another way
(`git -C . push`). So it guards the usual case, not every case.

**What exists.**

- `registry::Changes::set_autopilot` is the pattern for the registry's writers: the tool's inline
  form (`autopilot = { enabled = true }`) laid out afresh at each change, a table the owner wrote
  kept with its comments (`set_keeping_decor`), and a default taken out, with the key itself once
  nothing is left, so that the builds before read the file.
- `[project.launch]` (`registry::Launch`) is the second layer of `rules::resolve`. The owner's
  registry holds one such table, hand-written as a table.
- `autopilot::would_wait(mode, &settings_files(folder, home))` is the gate's waiting condition.
- The list's box and the A key push `Action::SetAutopilot`, which `Board::set_autopilot` writes
  at once.
- The launch dialog's Permissions picker changes only the launch at hand.
- The Sessions pane's Autopilot section (`Board::autopilot_section`) shows the state, the held
  chains, the plans and the log, but not the projects' settings.
- `crw set` changes `state`, `priority`, `pinned`, `answer_timeout` and `revert`.
- The runner already passes deny rules through `--settings`, the neighbours' `Edit` rules
  (`session::launch`).

**Session start (2026-11-09, 22:38 UTC).** Launched from the window as a visual launch (rule 11):
ledger row `d2f91553`, visual, EP-19b, open (`crw sessions clauderoadmapwrangler`). The opener
suggested Fable at xhigh, and the session runs at Opus 5.5 with max effort, chosen in the tab's
panel. The tree is clean at `811d9a6`, apart from the owner's tick of OT-19, which is not
committed. No owner task is open and the inbox is empty.

**Field check.** `crash.log` holds only the report of 2026-11-04 00:34 UTC, which was read before
EP-19a's record commit (`cde0628`). I read `crw notifications`, `crw sessions` and
`crw autopilot` back to that commit. In one of the owner's private projects the real window ran:

- On 2026-11-06, a chained brief at the rules' choice, the chain ending at a re-plan.
- On 2026-11-09, the owner's chain plan ("your choice", Opus 5.5 at max) through two chained
  briefs to the project's daily cap. Every run completed, and each one's closing question was
  answered in the Decisions pane.

Three things were odd:

1. One chain ended because two VS Code tabs of the project were open but idle (their turns over),
   and the owner started the next brief by hand. Triaged at pickup (below) and rolled in.
2. A finished run's popup showed the last line of the run's result, a raw markdown link from a
   list of sources, instead of what the run did. Parked in EP-20's brief notes (the owner's
   choice), since EP-20 sends the same line through ntfy.
3. On 2026-11-08 every project's backup badge turned stale at once. The Documents backup job was
   overdue, and it ran today, so there is nothing for the tool to do.

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-09)

1. **Split.** The model and effort go to a new row, EP-19c (S, after EP-19b, before EP-36): "keep"
   beside Model and Effort, keep for this phase, the chain plan's rows re-resolved, and `crw set`
   model and effort. EP-19b keeps the permission mode, the limits, the section and the `crw set`
   twins, which is what EP-37 needs to chain this project.
2. **The Keep control** is a menu per picker, opened by a pin button right of the picker: Keep for
   this project, Keep for phase Pn (Model and Effort, in EP-19c) and, once something is kept,
   Back to the default. It writes the registry at once, and it follows the picker, so it is greyed
   out in Visual.
3. **For EP-19c** (not the recommended option): a kept model and effort hold for S briefs too.
   "Keep for this project" writes a `[[project.rule]]` that matches every brief, placed after any
   phase rule, so that it comes after the built-in S rule. `[project.launch]` model and effort
   stay hand-written.
4. **The push guard.** Every headless run of a project whose autopilot is on, chained or started
   from the dialog, gets the deny rule `Bash(git push *)` through `--settings`.
5. **Idle tabs** (field check 1). The gate's "another session is open" condition counts only a
   session that works or waits. An idle one is named in the chained line instead. A D-22
   addendum.
6. **The finished popup's line** (field check 2) goes to EP-20's brief notes.

## In scope

1. **The registry's writers** (`crw_core::registry::Changes`), after `set_autopilot`'s pattern.
   One change can make several edits, so the offer's mode and switch go in one write.
   - `set_permission_mode(name, Option<&str>)` writes `launch = { permission_mode = "auto" }`
     when the project has no `launch`, the tool's inline form laid out afresh. A `[project.launch]`
     table the owner wrote is kept with its comments. The tool's default (`acceptEdits`) or `None`
     takes the key out, and the `launch` key with it once nothing is left. A mode is one word.
   - `set_autopilot_limit(name, Limit)` covers `max_size` (S, M or L), `daily` (0 … 100 runs) and
     `max_budget_usd` (dollars above 0). It writes into the `autopilot` table in either form and
     takes a default (`None`) out. A limit set while the autopilot is off keeps the key
     (`autopilot = { max_size = "S" }`), which builds since EP-19 read.
2. **The box's offer** (`app::autopilot`, `app::board`). Turning a project's autopilot on (the
   box, or A) when its runs would wait opens a dialog. "Would wait" is `would_wait` with the
   project's permission mode, over the project's two settings files and the user's; a screenshot
   run reads no user settings. The dialog says:
   - why the project's runs would wait, and that the gate would therefore refuse every chain;
   - the line on `auto`: what the classifier lets through and what it blocks, the push it allows,
     the deny rule that refuses an ordinary push, and that chained commits stay local until the
     owner pushes them after reading the diff.

   It has three buttons: **Set auto and turn on** (one registry change), **Turn on as it is** (the
   gate will refuse the project's chains for waiting; its runs still commit themselves) and
   **Cancel**. When the mode was set, the chain log says "turned on by the owner, with permission
   mode auto".
3. **Keep beside Permissions** (`app::launch`). A pin button right of the Permissions picker opens
   a menu: **Keep for this project**, which writes the picker's mode (the default is taken out),
   and **Back to the default (acceptEdits)** when the project has a mode of its own. Each item
   writes at once (`Action::KeepPermissionMode`); the reason line then reads "the project's launch
   defaults". The button is greyed out with the picker in Visual, and EP-19c reuses it beside Model
   and Effort.
4. **The Autopilot section** (`Board::autopilot_section`). One line for each project whose
   autopilot is on:
   - its permission mode, and where the mode comes from;
   - the gate's verdict on the conditions that stand between runs: runs that would wait (amber,
     with the way out), the tool's own project (until EP-37), sessions open there that would end a
     chain now (busy or waiting), and idle ones, which would not;
   - its limits, as small pickers that write at once (`Action::SetAutopilotLimit`): largest size,
     runs a day, and budget (none or a preset; any amount through `crw set`).

   The waiting verdicts are read with the log, every 30 seconds, never each frame.
5. **The gate's idle tabs** (`app::autopilot`). The gate's sessions are the window's own runs, a
   chained start on its way, and the seen sessions that are busy or waiting. An idle session is
   named in the chained line instead. The check just before the chained start does the same.
6. **The push guard** (`session::launch`). `Launch::deny_push` is set for every headless run of a
   project whose autopilot is on (`Board::start_headless`, `autopilot::launch`). The runner then
   adds `Bash(git push *)` to the deny rules it passes through `--settings`. A CLI without
   `--settings` drops the rule and says so, as it does for other wishes it cannot take.
7. **`crw set` twins.** `crw set <project> permission_mode <mode|default>`,
   `max_size <S|M|L|default>`, `daily <runs|default>` and `max_budget_usd <dollars|none>`, with
   the usage text. `crw autopilot` prints whether each project's runs would wait, beside its mode.
8. **Records.** D-18 and D-22 addenda, the README's paragraphs, the roadmap's EP-19b link, the
   EP-19c row and its notes, EP-20's addendum (field check 2), EP-37's note on the push guard, a
   "From EP-19b" note wherever a later brief needs one, and v2 notes if any arise.

## Out of scope

- Keep beside Model and Effort, keep for this phase, the plan's rows re-resolved, and `crw set`
  model and effort: all EP-19c.
- A control for the permission mode in the Autopilot section. The section shows the mode; the
  offer, the dialog's Keep and `crw set` change it.
- The finished popup's line: EP-20.
- Any real run of the CLI, or any press of Start headless outside a screenshot run (D-29, rule
  5). The owner's registry is written only by the owner's own clicks in their window (rule 13).

## Verification / acceptance

- **Unit tests**:
  - each writer in its three forms (no key, the tool's inline form, a table the owner wrote with a
    comment), with a default taken out (and the key with it) and the result read back;
  - `crw set`'s reading of the four fields;
  - the offer's decision: a waiting mode with no allow rule, then an allow rule, then `auto`;
  - the runner's command line with the push rule beside a neighbour's rule and alone, and the rule
    dropped when the CLI has no `--settings`;
  - the gate's sessions: busy and waiting ones refuse, idle ones are named;
  - the offer's dialog and the Keep menu, pressed through `testing::Screen`.
- **Build checks**: `cargo test`, `cargo test --features screenshot`, both clippy runs and
  `cargo fmt --check`.
- **Review**: before the release build, a review subagent reads the diff, without this session's
  conclusions (D-29 addendum). Each finding is fixed or logged in the completion note.
- **Screenshot runs**, each picture looked at: the offer (`offer=<project>`), the dialog after a
  kept mode, and the Autopilot section with its project lines, in both themes.
- **The release**: `cargo build --release` once the owner has quit the window from the tray, then
  `scripts\install-shortcut.ps1`.
- **Rule 3's word check** on the staged diff prints nothing.
- **Residual risk.** Four things this session could not exercise, and why I expect each to hold.
  1. **The offer, the Keep menu and the section's pickers on the owner's real registry.** The
     tests and the screenshot runs drive them on scratch registries and the fake portfolio. The
     writers are the same code either way, and the registry's one hand-written `[project.launch]`
     table is the form the tests keep, comment and all.
  2. **The push rule in a real headless run.** The tests check the command line. That Claude Code
     refuses `git push` under the rule, `auto` included, comes from its permissions page, and
     EP-9's supervised probe saw an `Edit` deny rule passed the same way refuse without a prompt. A
     run that tries a push should show a refused tool use in its transcript.
  3. **The gate with real hooks.** The idle filter reads the same `Seen::status` that the Sessions
     pane shows, and a tab whose turn has ended reads idle there.
  4. **The section's verdict against the real user settings.** `would_wait` and `settings_files`
     are the gate's own, unchanged since EP-19, and the field check shows the gate reading them
     in the owner's chains.

  EP-23's walk takes these four in turn: tick the box of a project whose runs would wait and read
  the offer; keep a mode from the dialog and read the reason line under the picker; set a limit in
  the section and see it with `crw autopilot`; let a chain pass an idle tab.

> **Completion note (2026-11-09; claude-opus-5-5/max; about 1 h 55 min to the commit, with the
> owner's three rounds and one quit of the window).** One session in the VS Code extension from
> 22:38 UTC, launched from the window (ledger row `d2f91553`, visual, EP-19b). It covered the
> reading and the field check, the pickup rounds (six questions in two dialogs), this brief, the
> split and the EP-19c row, the core and CLI half (by a subagent given the API), the window half,
> a review subagent's pass over the diff and its fixes, the screenshot runs, the records, the
> closing round and the release.
>
> **Built.**
>
> - `crw_core`:
>   - `registry::Limit`, and `Changes::set_permission_mode` and `set_autopilot_limit`, over one
>     helper (`set_in_table`) that `set_autopilot` now shares;
>   - `rules::PERMISSION_MODES` and `rules::permission_mode`, and `autopilot::standing`;
>   - the gate's waiting refusal, which now names the window's way out;
>   - `session::PUSH_DENY_RULE` and `Launch::deny_push`, which `autopilot::launch` sets.
> - `crw`: `crw set <project> permission_mode|max_size|daily|max_budget_usd`, and `crw autopilot`
>   saying when a project's runs would wait.
> - The window:
>   - the offer of `auto` (`autopilot::Offer`, `Board::offer_dialog`, `Action::AutopilotOn` and
>     `CloseOffer`);
>   - the Keep menu beside Permissions (`launch::keep_menu`, `Action::KeepPermissionMode`,
>     `Dialog::kept_permission_mode`);
>   - the Autopilot section's project lines (`autopilot_projects`, `limit_pickers`,
>     `Pilot::standing` read with the log);
>   - the gate's sessions (`gate_sessions`, `open_elsewhere`), with idle sessions named in the
>     chained line once a poll confirms them;
>   - the push guard at Start headless;
>   - the screenshot setup flags `offer=` and `keep=`, with the canned autopilot turned on past the
>     offer and quarry beside it with `auto`.
> - The records: D-18 and D-22 addenda, the README's paragraphs, the roadmap's EP-19b link and
>   retitled row, the EP-19c row and its note, the split's sentence, and EP-20's and EP-37's notes.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed 436 (109 + 2 + 1 + 1 + 2 + 1 + 283 + 2 + 7 + 6 + 11 + 11,
>   against EP-19a's 417), and 438 with `--features screenshot`. Both clippy runs and `cargo fmt
>   --check` are clean. The new tests cover:
>   - the writers in each form, a default taken out, an owner's empty table kept, and one change
>     carrying both the mode and the switch;
>   - `standing`, and the push rule alone, beside a neighbour's rule, and dropped;
>   - `crw set`'s parsers;
>   - the offer's path on a scratch board, the offer pressed through `testing::Screen`, and
>     Escape;
>   - the Keep menu pressed through the dialog;
>   - the gate's sessions, the poll's confirmation, the chained line's idle names, and the
>     limits in words.
> - **Screenshot runs**, kept in the session's scratchpad:
>   - light `offer=harbor`: the offer;
>   - dark `keep=harbor:auto,launch=harbor:headless`: the lit pin and "the project's launch
>     defaults", the run's registry having kept harbor's own `[project.launch]` table with the key
>     added in it;
>   - `autopilot`, light at 2300 × 1000 and dark at 1600 × 1000 and 1600 × 700: the project lines
>     and the shortened verdict.
> - **Review** (a subagent given the diff, D-29 addendum). Six findings, all fixed:
>   1. A tab the owner has just prompted reads idle until the next poll, since the prompt hook
>      waits for EP-20. A chained start that would pass an idle session now waits for a poll newer
>      than its queueing (at most 75 s), then counts the session.
>   2. The keys reached the list behind the offer.
>   3. The section flagged the window's own run as a session that would end a chain.
>   4. Start headless read the autopilot switch from the last reading of the files; it now reads
>      the registry.
>   5. `crw set` wrote a default value out explicitly, and refused `default` for the budget.
>   6. A removal that found nothing dropped an empty table the owner wrote.
>
>   Its two test gaps (the chained line's idle names, and Escape on the offer) are now covered.
> - **The release.** Built once the owner had quit the window; then `scripts\install-shortcut.ps1`
>   refreshed both shortcuts, and the new `crw autopilot` read the owner's registry. Rule 3's word
>   check found nothing.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-09).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Split the M | Model and effort go to EP-19c | EP-19c (S) after EP-19b |
> | 2 | The Keep control's form | A menu per picker | `launch::keep_menu`, written at once |
> | 3 | A kept effort and S briefs (for EP-19c) | It holds for S too (not the recommended option) | EP-19c writes an every-brief `[[project.rule]]`; D-22 addendum |
> | 4 | The push guard | A deny rule for autopilot projects | `Launch::deny_push`, `Bash(git push *)`; D-18 addendum |
> | 5 | Idle tabs (field check) | Count only working or waiting sessions | The gate's sessions; D-22 addendum |
> | 6 | The finished popup's line (field check) | EP-20's brief notes | Parked there |
> | 7 | The release build | Quit the window, build and install | Built and installed |
> | 8 | Commit and push | Commit and push | Two commits, pushed |
> | 9 | The session's effort | max, as recorded | Front matter kept |
>
> **Routine choices (the session's call, logged).**
>
> 1. The core and CLI half went to a subagent given the exact API; the window half stayed here,
>    to keep this context small with auto-compaction off.
> 2. When a project has no `launch`, the tool writes `launch = { permission_mode = "auto" }`
>    inline; a table the owner wrote keeps its form.
> 3. Keeping the default mode takes the key out, as Back to the default does.
> 4. The limit pickers offer presets (0 to 8 runs a day, budgets of $5 to $50); `crw set` takes
>    any figure, and a value the pickers lack is offered as it stands.
> 5. The section reads the waiting condition with the log, every 30 seconds or after a write, never
>    each frame; screenshot runs and the tests read no user settings.
> 6. The section names the window's way out of a waiting project; the log keeps every way.
> 7. The box and the A key both reach the offer, and Escape or a click beside it closes it.
> 8. A CLI without `--settings` drops the push rule with a sentence and starts anyway.
> 9. A chained start waits at most 75 seconds (two poll turns while the window is hidden) for a
>    poll to confirm its idle sessions, then counts them.
>
> **Notes for later sessions.**
>
> - EP-19c builds Keep for Model and Effort on `launch::keep_menu` and the writers' helper; the
>   every-brief project rule is the owner's choice.
> - EP-37: the offer (this project's runs would wait), the pin or `crw set` sets this project's
>   mode, and its headless runs carry the push rule.
> - Neither the registry's format nor the ledger's schema changed, so nothing migrates.
>
> No messages for other projects. No owner task: under the D-10 addendum nothing here needs the
> owner's hands, and the four Residual risk points go to EP-23's walk. **Next: EP-19c** (standing
> model and effort), launched from the window.
