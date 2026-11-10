---
ep: 19c
title: Standing model and effort
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-19c — Standing model and effort

**Size:** S → M (re-scoped at pickup) · **Depends on:** EP-19b · **Blocks:** EP-23

## Context

The owner asked for a standing model and effort after the first real chain ran its second brief
at the rules' choice, where they had wanted another (D-22's EP-19a addendum). At EP-19b's pickup
the owner split this half off into its own row. They also chose where a kept value goes, which was
not the recommended option. "Keep for this project" writes a `[[project.rule]]` that matches every
brief, placed after any phase rule. It therefore applies after the global table, so a kept effort
holds for an S brief too. `[project.launch]` model and effort stay hand-written.

**What exists.**

- `launch::keep_menu` is the pin beside the launch dialog's Permissions picker. Its items write at
  once (`Action::KeepPermissionMode`). `Board::keep_permission_mode` then updates the open dialog
  through `Dialog::kept_permission_mode`.
- `registry::Changes::set_permission_mode` and `set_autopilot_limit` share `set_in_table`: the
  tool's inline form laid out afresh, a table the owner wrote kept with its comments, and a table
  taken out once nothing is left in it.
- `rules::resolve` resolves in layers: the tool's default, `[project.launch]`, the first global
  rule that matches (the built-in S rule while the registry lists none), the first project rule
  that matches, and then the Fable downshift (D-23). `Rule::check` refuses a rule that sets
  nothing.
- `autopilot::plan` resolves each chain-plan row through the rules. A row the owner touched keeps
  `Planned::picked`.
- `crw set` writes the permission mode and the autopilot's limits.
- The owner's registry has no `[[rule]]` and no `[[project.rule]]` today.

**What pickup found.** Each table applies only its first matching rule. A phase rule written
before the every-brief rule would therefore hide the every-brief rule from that phase's briefs. If
the owner keeps a model for the project and an effort for phase P3, a P3 brief would take the
phase rule's effort and lose the kept model, falling back to the tool's default. The owner chose to
combine rules key by key (decision 1 below).

**Session start (2026-11-09, 19:35 local time, 00:35 UTC on the 10th).** Launched from the window
as a visual launch (rule 11): ledger row `d2c579f5`, visual, EP-19c, open
(`crw sessions clauderoadmapwrangler`). The opener suggested Fable at high effort, and the session
runs at Opus 5.5 with max effort, chosen in the tab's panel. The tree is clean at `4be38a8`. No
owner task is open and the inbox is empty.

**Field check.** `crash.log` holds only the report of 2026-11-04 00:34 UTC, read before. I read
`crw notifications`, `crw sessions` and `crw autopilot` back to EP-19b's record commit (`4be38a8`).
Nothing new has happened since. The latest lines are a private project's chain up to its daily cap
and EP-19b's own popups, all read by EP-19b's field check. There is nothing to triage.

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-09)

1. **Rules combine key by key, in both tables** (the recommended option). Every rule that matches
   a brief applies. For each of model, effort and ultracode, the earliest matching rule that sets
   it wins, and an `effort_step` steps from what the later rules and the lower layers give. The
   global `[[rule]]` table and the project's `[[project.rule]]` table both work this way. This is a
   D-22 addendum, amending the EP-10 addendum's "first rule that matches".
2. **The size.** The row is kept whole and re-scoped S → M. Its work is close to EP-19b's, an M.

## In scope

1. **The resolver** (`crw_core::rules::resolve`). Within each table (the global or built-in one,
   then the project's), the matching rules apply from the last to the first. Each value keeps the
   reason of the rule that set it. The module's and the function's docs say so.
2. **The rules the window keeps** (`crw_core::rules`).
   - `Scope` is `Project` (every brief) or `Phase(id)` (that phase alone). `Key` is `Model` or
     `Effort`.
   - `kept_rule(project, scope)` finds the scope's rule by its match. For every brief, that is the
     first rule with no condition. For one phase, it is the first rule whose only condition is
     that phase. The tool finds its rules this way, so a rule the owner wrote in that form counts
     as the scope's.
   - `Kept::of(project, phase, key)` gives what the project keeps for a key, for every brief and
     for the brief's phase.
   - The name the tool writes is "kept for this project" or "kept for phase Pn", which is the
     reason the dialog then shows ("the project's rule 2: kept for this project").
3. **The writer** (`crw_core::registry::Changes::keep(name, scope, key, value)`).
   - A value goes into the scope's rule `set`, through `set_in_table`. An effort replaces an
     `effort_step` in the same `set`, since a rule sets one or the other.
   - Where the scope has no rule, the tool writes one. A new every-brief rule goes after every
     other project rule. A new phase rule goes just before the first every-brief rule, or last if
     there is none.
   - `None` (Back to the rules) takes the key out, and the rule with it once its `set` is empty.
   - The rule a key is found in keeps its name, comments and other keys. The next `[[project]]`
     stays as it was.
   - A model or an effort is one word. A project whose rules are not written as `[[project.rule]]`
     tables is refused, with nothing written.
4. **The draft's phase** (`crw_core::prompt::Draft::phase`): the phase heading's id, for the
   dialog's "Keep for phase Pn".
5. **The plan's choices** (`crw_core::autopilot::Plan::keep_choices`). A plan made again after a
   keep takes each row's Chain box and the owner's choice from the plan before it. A row left
   alone shows the new resolution.
6. **The Keep menus** (`app::launch`). A pin right of the Model and the Effort pickers opens:
   - **Keep {value} for this project**;
   - **Keep {value} for phase Pn**, when the brief's phase has an id;
   - **Back to the rules**, one item per scope that keeps the key, each naming what it drops.

   An item that would change nothing is greyed out, with its reason. The pin is lit while the
   project keeps the key for every brief or for this phase. The pin follows its picker, so it is
   greyed out in Visual (EP-19b's decision 2). The effort's "CLI default" cannot be kept.
7. **The dialog follows** (`Board::keep_rule`, `Dialog::kept_rule`).
   - The registry is written at once, and the dialog's brief is resolved again from it.
   - After Back to the rules, the picker takes the rules' value.
   - After a keep, the picker takes the new resolution when its value is the one kept, so the
     reason names the project's rule. When something else stands over the kept value for this
     brief (an earlier rule, or the downshift for an S brief on Fable), the picker keeps the
     owner's value, and the notice names what stands over it: "kept, but for EP-n the project's
     rule 1: kept for phase P3 stands over it".
   - The chain plan is made again (`autopilot::plan`, then `keep_choices`), and the prompt is
     regenerated unless the owner has edited it.
8. **`crw set <project> model <id|default>` and `effort <level|default>`.** These write the
   every-brief rule. An effort is one of the built-in levels in any case, and `default` takes the
   key out. The usage text is updated.
9. **The screenshot setup flag** `keeprule=<project>:<key>[@<phase>]=<value>`, which keeps a value
   as the menu does, before `launch=` opens the dialog.
10. **Records.** A D-22 addendum, the README's paragraphs on the rule table and the dialog, the
    roadmap's EP-19c link and size cell, a "From EP-19c" note wherever a later brief needs one,
    and v2 notes for what stays parked.

## Out of scope

- A Keep menu beside the chain plan rows' pickers. The main pickers' keep reaches the rows by
  re-resolving them.
- Keeping ultracode; `crw set` for a phase rule; a keep in Visual (greyed out, EP-19b's decision
  2). These go to the v2 notes.
- The downshift over a kept Fable model. It stays the resolver's last layer (D-23), and the
  notice names it when it stands over a keep.
- `[project.launch]` model and effort, which stay hand-written.
- Any real run of the CLI, or any press of Start headless outside a screenshot run (D-29, rule 5).
  The owner's registry is written only by the owner's own clicks in their window (rule 13).

## Verification / acceptance

- **Unit tests**:
  - the resolver: a phase rule's effort and an every-brief rule's model both hold for a brief in
    that phase; an earlier rule's step over a later rule's effort; two global rules combined;
  - `kept_rule` and `Kept::of` over the tool's rules and over rules the owner wrote in their
    form;
  - the writer:
    - with no rules;
    - with the tool's rules, a phase rule placed before the every-brief rule;
    - with an owner's every-brief rule, its name and comment kept and its `effort_step`
      replaced;
    - with a `set` the owner wrote as a table;
    - a key taken out, and the rule with it;
    - the refusals;
    - the next `[[project]]` left as it was;
  - `Plan::keep_choices`;
  - `crw set`'s reading of model and effort;
  - the menus pressed through the dialog, and `Dialog::kept_rule`'s three cases (taken, stood
    over, back).
- **Build checks**: `cargo test`, `cargo test --features screenshot`, both clippy runs and
  `cargo fmt --check`.
- **Review**: before the release build, a review subagent reads the diff without this session's
  conclusions (D-29 addendum). Each finding is fixed or logged in the completion note.
- **Screenshot runs**, each picture looked at: the dialog with a model kept for the project and an
  effort kept for the phase (lit pins, reasons naming the project's rules, the chain plan's row
  re-resolved), in both themes.
- **The release**: `cargo build --release` once the owner has quit the window from the tray, then
  `scripts\install-shortcut.ps1`.
- **Rule 3's word check** on the staged diff prints nothing.
- **Residual risk.** Four things this session could not exercise, and why I expect each to hold.
  1. **The pins on the owner's real registry.** The tests drive the writer on scratch registries,
     and the screenshot runs drove it through the window on the run's own copy. It is the same
     code either way. The owner's registry has no rule yet, so the first keep writes a
     `[[project.rule]]` after the project's own tables, which is the layout the tests assert byte
     for byte.
  2. **The resolver's new reading on real launches.** No launch changes until a rule exists. The
     owner's registry has none, the built-in table has a single rule, and the resolver's existing
     tests gave the same results under the new reading.
  3. **A chained brief taking a kept value.** The gate drafts each brief through `prompt::draft`,
     which resolves through the same `rules::resolve` as the dialog. A plan row left alone
     therefore runs with the kept value, as the plan shows before the chain starts (a test and
     the light picture). No real chain has run with a kept rule.
  4. **The visual suggestion.** A visual opener names the rules' suggestion, which now includes
     the kept values, while the tab's panel still chooses (D-16). The dark picture shows the
     wording.

  EP-23's walk takes these in turn. Keep an effort for a project from a headless launch dialog,
  then read the reason line and `crw prompt <project>`. Keep a model for the brief's phase and look
  at the chain plan's rows. Press Back to the rules and see the rule leave the registry. Run
  `crw set <project> effort default`.

> **Completion note (2026-11-09; claude-opus-5-5/max; about 1 h 15 min to the commit, with the
> owner's two rounds and one quit of the window).** One session in the VS Code extension from
> 19:35 local time, launched from the window (ledger row `d2c579f5`, visual, EP-19c). It covered
> the reading and the field check, the pickup round (two questions), this brief, the core and CLI
> half (by a subagent given the API), the window half, the records, a review subagent's pass over
> the diff and its fixes, the screenshot runs, the closing round and the release.
>
> **Built.**
>
> - `crw_core`:
>   - `rules::resolve` combines rules key by key in each table, from the last matching rule to
>     the first; a step that a rule applied after it makes moot leaves no note;
>   - `rules::Scope`, `Key`, `kept_rule`, `Kept` and `stands_over`;
>   - `registry::Changes::keep`, an effort taking the place and the comment of an owner's step;
>   - `prompt::Draft::phase` and `autopilot::Plan::keep_choices`.
> - `crw`: `crw set <project> model <id|default>` and `effort <level|default>`.
> - The window:
>   - the Keep menus beside Model and Effort (`launch::rule_keep_menu`, `Action::KeepRule`);
>   - `Board::keep_rule` and `Dialog::kept_rule`: the brief resolved again, the plan made again
>     with the owner's rows kept, and the notice naming what stands over a kept value;
>   - `open_launch` resolving against the registry as just read;
>   - the screenshot setup flag `keeprule=`, and `testing::texts_at` for the three pins.
> - The records: the D-22 addendum, the README's paragraph on the rule table, the roadmap's EP-19c
>   link and re-scoped size, its note's re-scope sentence, EP-37's "From EP-19c" note, and two v2
>   notes.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed 451 (113 + 3 + 1 + 1 + 2 + 1 + 293 + 2 + 7 + 6 + 11 + 11,
>   against EP-19b's 436), and 454 with `--features screenshot`. Both clippy runs and `cargo fmt
>   --check` are clean. The new tests cover:
>   - the resolver key by key, in the project's table and the global one, an earlier step over a
>     later effort, and a moot step's note;
>   - the kept rules found by their match, and what stands over a kept value;
>   - the writer: a rule written where the scope has none, an owner's rule written into, a new
>     rule's place, an effort in a step's place, and the refusals;
>   - the plan's choices kept, and `crw set`'s reading of a model and an effort;
>   - the menus pressed through the dialog, and the dialog after a keep: taken, stood over, back,
>     and numbered afresh;
>   - `Board::keep_rule`'s writes and notices, and `keeprule=`.
> - **Screenshot runs**, kept in the session's scratchpad:
>   - light `autopilot,keeprule=harbor:model=claude-opus-5-5,keeprule=harbor:effort@P1=high,
>     launch=harbor:headless`: both pins lit; the reasons "the project's rule 2: kept for this
>     project" and "the project's rule 1: kept for phase P1"; the opener and the chain plan's EP-5
>     row at Opus 5.5 and high;
>   - dark `keeprule=lantern:effort=max,launch=lantern`: an S brief in Visual at max, "suggested
>     by the project's rule 1: kept for this project", the pins greyed out.
> - **Review** (a subagent given the diff, D-29 addendum). Four findings, all fixed:
>   1. A keep that numbers the project's rules afresh (a phase rule placed first, an emptied rule
>      taken out) left the other picker's reason with the old number, and only the kept key's pin
>      was read again. `Dialog::kept_rule` now reads both pins from the project as written and
>      gives every value the rules chose its new reason, keeping the owner's choices. A test makes
>      two keeps through one dialog.
>   2. A step that could not be taken still left a note when a rule applied after it set the
>      effort.
>   3. An effort replacing an owner's step went to the end of their `[project.rule.set]` table,
>      not to the step's place.
>   4. `open_launch` resolved against the portfolio's last reading of the project. A dialog opened
>      in the same pass as a keep (a screenshot run's `keeprule=`, then `launch=`) showed the
>      values from before the keep.
> - **The release.** Built once the owner had quit the window; then `scripts\install-shortcut.ps1`
>   refreshed both shortcuts, and the new `crw` prints the two `set` lines. Rule 3's word check
>   found nothing.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-09).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | How matching rules combine | Key by key, in both tables | `rules::resolve`; D-22 addendum |
> | 2 | The row's size | Keep it whole, S → M | The row's size cell |
> | 3 | The release build | Quit the window, build and install | Built and installed |
> | 4 | Commit and push | Commit and push | Two commits, pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. The core and CLI half went to a subagent given the exact API; the window half stayed here,
>    to keep this context small with auto-compaction off.
> 2. Back to the rules is one item per scope that keeps the key, each naming what it drops, in
>    place of EP-19b's single item.
> 3. The tool's rule for every brief is the first with no condition, written with no `match` key.
>    Its phase rule is the first whose only condition is that phase. A rule the owner wrote in
>    either form is written into, with its name and comments kept.
> 4. An effort replaces a step in the same `set`, in its place and with its comment, since a rule
>    sets one or the other.
> 5. After a keep, the picker takes the new resolution when its value is the one kept. Otherwise it
>    keeps the value shown, and the notice names what stands over it.
> 6. `crw set` takes the built-in effort levels (low to max), and any one word for a model.
> 7. The pins follow their pickers and are greyed out in Visual (EP-19b's decision 2). The CLI's
>    own effort cannot be kept.
>
> **Notes for later sessions.**
>
> - EP-37: a standing model and effort for this project's chained briefs can be kept through the
>   pins or `crw set` (its "From EP-19c" note).
> - Neither the registry's format nor the ledger's schema changed. Builds since EP-10 read the kept
>   rules, under first match.
>
> No messages for other projects. No owner task: under the D-10 addendum nothing here needs the
> owner's hands, and the four Residual risk points go to EP-23's walk. **Next: EP-36** (screenshot
> runs over a portfolio copy), launched from the window.
