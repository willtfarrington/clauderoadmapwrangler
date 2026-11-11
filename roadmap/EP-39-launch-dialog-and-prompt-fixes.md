---
ep: 39
title: Launch dialog and prompt fixes from the P3 walk
size: M
model: claude-fable-5-1
effort: xhigh
status: done
---

# EP-39 — Launch dialog and prompt fixes from the P3 walk

**Size:** M · **Depends on:** EP-37a, EP-10, EP-20b · **Blocks:** EP-24b, EP-29

## Context

The P3 walk (EP-23, 2026-11-11) settled three findings against the launch dialog and the
prompts, each an owner's choice recorded as an addendum of that date: the launch dialog says
nothing while a newer build is installed, so a run started by hand starts in the older window
(D-29); the opener quotes a project's read-first rule with the file's hard line breaks, which a
wide editor shows broken in mid-sentence (D-17); and the prompt hook's context cuts a long
`CLAUDE.md` and a long brief note each in the middle, so this project's own rules and a re-plan's
note both ended mid-sentence (D-20). This brief builds the three.

**Session start (2026-11-11, 18:07 local time).** A headless run chained by the autopilot after
EP-38, through the window's restart into EP-38's build; its prompt is the tool's opener, and
`crw sessions clauderoadmapwrangler` lists its row (`103fb7d1`, headless, EP-39, fable/xhigh,
open). The field check read, since EP-38's record commit (`c7c4fb2`, 18:05): `crash.log` (the
2026-11-03 report, nothing newer); `crw notifications` (EP-38's `waiting` popup at 17:56 and
its `finished` one at 18:06, then the `chain` popup "chained EP-39 after EP-38" at 18:07, each
with its ntfy line); `crw sessions` (this run; EP-38 completed in 39 min with one question,
answered in 35 s); `crw autopilot` (not paused; `held after EP-38 for the window's restart into
the installed build`, then `chained EP-39 after EP-38: fable at xhigh, the rules' choice, M's
cap of 1 h 30 min`, the restart taken at the gate's moment as EP-37a intended). Nothing odd to
triage. No open owner task, no open inbox message; the tree is clean.

**What exists.**

- `install::restart_due(&Running, installed)` says whether the installed copy is one to
  restart into; `restart::Restart::look` reads it every 30 s into `Standing::Installed`, whose
  `chip()` the status bar shows ("A newer build is installed: File > Install and restart …").
  `Board::restart_refusals` lists "the launch dialog is open" among the reasons, and
  `Board::install_and_restart` (`restart_window(Asked::Owner)`) writes the handover
  (`crw_core::handover::Handover`, version 1: `held`, `plans`, `hidden`, `urgent_seen` with
  `#[serde(default)]`) only when something is carried, then starts the installed window; the new
  window's `take_handover` reads it at its start. `Board::open_launch` builds `launch::Dialog`
  from `prompt::draft`; a screenshot run's `launch=<project>[:headless]` opens it through
  `open_when_read` once the files are read, the path a restarted window can reuse. The frozen
  version-1 file is `handover::tests::FROZEN`.
- `prompt::read_first(claude_md)` returns the rule as written (the list item or paragraph
  opening with **Read first**, without its marker, or a section's body); `prompt::parts` puts it
  into `Parts::read_first`, which `opener` writes under "Read first, as the project's CLAUDE.md
  says:" and a template may name as `{read_first}`; `crw prompt`, the dialog and a chained run's
  opener all come from `prompt::draft`. The app's `markup::reflow` joins continuation lines for
  the Detail pane; `core` has nothing of the kind.
- `signals::mentions::context(prompt, cwd, projects)` writes per mentioned project a framing
  line, the whole `CLAUDE.md`, then the next row and the brief notes verbatim, the two texts
  sharing `BUDGET` (9,000) through `shares` (six tenths to the rules when both are long) and
  `cut` (at a line's end where one lies in the second half, else mid-text, a marker naming the
  file). `crw hook` calls it on `UserPromptSubmit` for an interactive session
  (`signals::mentions_context`); EP-20b's trial ran the release exe against a scratch registry
  on another port, so no post reached the owner's window.

## In scope

1. **A newer build is installed, in the dialog** (D-29 addendum, 2026-11-11). `Restart` gains
   `installed_words()`: the words of an installed build to restart into, read now (not at the
   30 s pace), naming the build's date; a screenshot run cans them (`canned`). `Board::open_launch`
   puts them into `Dialog::newer_build`, and each pass keeps the open dialog's line current. The
   dialog draws the line above its footer in amber, with an Install and restart button beside it;
   Start headless and Open in VS Code stay enabled, and nothing restarts by itself. The button
   queues `Action::RestartFromLaunch`: the board takes the dialog out, remembers its project as
   `reopen_launch`, and asks `install_and_restart`; the handover gains `open_launch:
   Option<String>` (`#[serde(default)]`, version 1 kept), set from `reopen_launch` and counting
   as carried; the restarted window's `take_handover` puts it into `open_when_read`, so the
   dialog opens on that project once its files are read. A refused restart puts the dialog back
   with the reasons in its error line; "the window runs the newest build already" puts it back
   with the line gone. The frozen-form test is extended in both directions: the frozen file reads
   with `open_launch` none, and the same file with the field reads with it.
2. **The quote's lines joined** (D-17 addendum, 2026-11-11). `prompt::joined(text)`: each
   paragraph one line, a list item, a heading or a table row starting a line of its own with its
   indent kept, blank lines kept, a fenced block verbatim. `prompt::parts` stores the read-first
   rule joined, so `crw prompt`, the dialog's editor, the opener a session receives and the
   `{read_first}` placeholder all carry it; the house default is one line already, and the rule
   block's own item (three lines in `formalise::RULES`) joins to one.
3. **The hook's context** (D-20 addendum, 2026-11-11). `mentions::section` writes, in this
   order: the framing line; the project's next row and the start of its brief note, cut at a
   sentence's end where one is near; its rules in short, each numbered rule's first sentence on a
   line of its own (a `CLAUDE.md` without numbered rules: its first paragraphs and bullets, whole),
   never one cut in the middle, the rules left out counted and their file named; then the paths
   of its `CLAUDE.md` and its `roadmap/README.md` for the rest. `shares` still splits the room
   between the rules and the note, and several mentioned projects still share `BUDGET`. A trial of
   the release `crw.exe hook` runs with a scratch registry on another port, as EP-20b's did.
4. **Tests.** The dialog's line and button on a canned standing, the restart from the dialog
   (refused: the dialog stays with the reasons; taken: the handover carries the project; the new
   window opens the dialog once read); the handover's frozen form both ways; `joined` on the item,
   the section and a fenced shape, and the opener's quote on one line; the context's order, a long
   `CLAUDE.md` of numbered rules ending at a whole rule with the count, one without numbered rules
   ending at a paragraph, a long note cut at a sentence, the paths last, five projects under the
   cap.
5. **The records.** Addenda to D-29, D-17 and D-20 (as built); the README's sentences on the
   restart and the prompt hook; CLAUDE.md rules 6 and 11 naming the dialog's button; this brief.

## Out of scope

- **A restart by itself** when the dialog opens: the owner's choice at the walk (nothing restarts
  by itself; Start stays enabled).
- **Carrying the dialog's edits** (the prompt as edited, the mode, the chain plan's rows) across
  the restart: the dialog opens afresh from the registry and the files, which its line says.
- **The chat's own mentions** (EP-24b), which will call the same function.
- **`urgent_seen` counted as carried**: today a shown window with no chain writes no handover,
  so its seen set is not carried; noted, unchanged.
- **The Detail pane's `markup::reflow`**: it stays the app's; `core`'s `joined` is for the prompt.

## Verification / acceptance

- `cargo test --workspace`, `cargo test --features screenshot`, both clippy runs and `cargo fmt
  --check` clean.
- A review subagent given the diff alone (rule 6), launched as soon as the code compiles, its
  findings fixed or logged.
- **Screenshot runs** (`target\tmp\ep39\`, never in git): the launch dialog with the canned
  newer-build line and its button, both themes, on the fake portfolio; the dialog over a copy of
  one real project's roadmap (rule 5: this brief changes a pane's dialog), its quoted rule on one
  line per paragraph in the editor.
- `target\release\crw.exe prompt clauderoadmapwrangler` prints the read-first quote with each
  paragraph on one line; `crw hook` on a canned `UserPromptSubmit` with `@<project>` against a
  scratch registry on port 47999 prints the next row first, the rules in short and the paths
  last, under 9,000 characters.
- The release build and the install (`pwsh -File scripts\install-shortcut.ps1`) with the window
  up; rule 3's word check on each staged diff prints nothing.

**Residual risk.** The restart from the dialog was exercised with a stand-in for the installed
window's start and a scratch handover folder: the real round trip (the dialog closing, the new
window reading its files and opening the dialog again) runs first when the owner presses the
button in a window with a newer build installed, which EP-38's and this brief's installs leave
behind by design; the path it takes is `open_when_read`'s, which every screenshot run's `launch=`
flag takes. The hook's new context was tried on the release exe against a scratch registry, not in
a real session's `UserPromptSubmit`, whose relay and budget are unchanged since EP-20b. The joined
quote was checked on this project's own rule and the fixtures' shapes; a project whose rule holds
a form `joined` does not know (a nested fence inside a list item, say) would carry it as written.

> **Completion note (2026-11-11; claude-fable-5-1/xhigh; about 40 min of wall time, a headless
> run under the autopilot, no question).** Ledger row `103fb7d1`, chained after EP-38 at 18:07
> through the window's restart into EP-38's build (an M's cap of 90 min); the prompt was the
> opener.
>
> **Built.**
>
> 1. **A newer build is installed, in the dialog.** `Restart::look_now` reads the standing when
>    the dialog opens, `look` keeps the installed build's date, and `installed_words` gives the
>    line (`installed_words_of(when)`, canned by a screenshot run's `newer` flag, EP-39's only
>    new flag); `Board::launch_dialog` sets `Dialog::newer_build` from it each pass, for a brief's
>    dialog and not a migration's. The footer draws the line in amber with Install and restart
>    beside it (wrapped under it at the dialog's width), Start headless and Open in VS Code
>    untouched; the button queues `Action::RestartFromLaunch`, and `Board::restart_from_launch`
>    takes the dialog out, keeps its project in `reopen_launch`, asks `install_and_restart` and
>    puts the dialog back with the words on a refusal ("Not restarted: the offer of auto is open"),
>    on nothing newer (the line gone) or while a restart is under way already. The handover's
>    `open_launch` (`#[serde(default)]`, version 1 kept, `Handover::carries` counting it) is set
>    from `reopen_launch`, and the restarted window's `take_handover` puts it into
>    `open_when_read`, the path a screenshot run's `launch=` takes, so the dialog opens once the
>    files are read; the start's notice says "carrying the launch dialog of <project>". An
>    installed window that ends at once opens the dialog here again, since `reopen_launch` is kept
>    until then. Tests: the dialog's line and button on canned words (`launch::tests`), the
>    restart from the dialog in every outcome and the restarted window's `open_when_read`
>    (`restart::tests`), the frozen version 1 file with and without the field and this build's
>    own writing of it (`handover::tests`).
> 2. **The quote's lines joined.** `prompt::joined` (a paragraph one line; a list item, a
>    heading, a table row, a quote's line or a thematic break on a line of its own with its
>    indent; a run of blank lines one; a fenced block as written), applied in `prompt::parts`, so
>    `crw prompt`, the dialog, a chained run's opener and `{read_first}` carry it. On this folder,
>    `crw prompt clauderoadmapwrangler` prints rule 1 as one line of about 900 characters under
>    "Read first, as the project's CLAUDE.md says:", where the file's eleven lines stood.
> 3. **The hook's context.** `mentions::section` in the new order, `in_short` (numbered rules'
>    first sentences, else paragraphs and bullets whole), `first_sentence`, `cut_units` (whole
>    lines, the rest counted) and `cut_note` (a sentence's end, a line's end, a space); `shares`
>    and the three-project limit unchanged. A trial of the debug `crw.exe hook` with a scratch
>    `CRW_HOME` registry on port 47999 (no window heard it) over a `UserPromptSubmit` naming two
>    scratch projects, one holding a copy of this repository's `CLAUDE.md` and roadmap, printed
>    5,725 bytes: that project's row, the note's first paragraph cut at "…cans the state." with
>    the marker, seven of its thirteen rules as first sentences and `[and 6 more rules, in …]`,
>    then the two paths; the other, without numbered rules, its three paragraphs whole; the
>    relay's post went to the closed port.
>
> **The pictures** (`target\tmp\ep39\`, never git): the fake portfolio's harbor dialog (headless,
> light) and lantern's (visual, dark) with the line and the button above the footer's notes and
> Start enabled; EP-38's copy of the real roadmaps, this project's dialog over it, the quoted rule
> one wrapped paragraph in the editor, light with the line and dark without.
>
> **The review (rule 6).** A subagent given the diff alone, launched once the code compiled,
> reported after seven minutes: two confirmed and five plausible points. Fixed: (1) a migration's
> dialog showed the line, and its restart would have reopened a brief's dialog (confirmed): a
> migration's dialog shows none; (2) the dialog was lost when the installed window ended at once
> (confirmed): `reopen_launch` is kept until that branch, which opens the dialog again and says
> so; (3) a restart under way already took the dialog and reported it carried: the dialog is put
> back with "The window is restarting already."; (5) `joined` glued a thematic break to the line
> before it: a break stands alone. Logged, not changed: (6) `cut_units`' and `cut_note`'s
> fallbacks for a room smaller than their marker exceed it, unreachable from `section` since a
> project's share leaves about 2,200 characters at the least and `shares` gives the rules six
> tenths; (7) `first_sentence` returns the whole rule when a bracket is left open and ends early
> at an abbreviation such as "e.g.", both within the budget and whole; (4) the state, the serde
> form and the footer's UI were found sound.
>
> **Verification.** `cargo test --workspace`: 558 passed (358 in `core`, 150 in `app`, 50 in the
> other suites); with the feature 158 in `app`; both clippy runs (one `last` on a double-ended
> iterator fixed to `next_back`) and `cargo fmt --check` clean, the gate run again after the
> review's fixes. Rule 3's word check on the staged diff printed nothing. The release build and
> the install (`scripts\install-shortcut.ps1`) run after the record commit, the opener's last
> step; the restart is refused while this run works and taken at the gate's moment, or from File
> > Install and restart, the tray's, or now the launch dialog's.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> No question was put: nothing else in the brief is the owner's (no split, no scope change, no
> outward-facing text beyond the README's two generic sentences, no unresolved risk beyond the
> residual-risk paragraph).
>
> **Routine choices (the session's call, logged).**
>
> 1. **The dialog opens afresh** after the restart: the project alone is carried, not the
>    prompt's edits, the mode or the chain plan's rows, which the registry and the files give
>    again; the line says "afresh".
> 2. **The line above the footer's notes**, in the notes' amber, the button under it: the words
>    are long, so `horizontal_wrapped` puts the button on the next line at the dialog's width.
> 3. **The join in `parts`, not in `read_first`**: the formalise step's tests read the rule as
>    written, and `read_first` stays the parser.
> 4. **Only numbered items are rules** for the first-sentence form; a `CLAUDE.md` of bullets and
>    paragraphs gives them whole, as the brief note says.
> 5. **The paths last**, as a sentence ("The rest is in … and …"), even when nothing was cut.
> 6. **`open_launch` is a project name**, like `urgent_seen`'s; a kind (brief or migration) was
>    not added, since a migration's dialog shows no line.
> 7. **`Board::open_launch` is `pub(crate)`** for the restart test, as `open_migration` was.
> 8. **The scratchpad** is `target\tmp\ep39\` (ignored): the pictures, the logs, the hook trial's
>    registry and projects, and the diff the review read.
>
> No owner task; no message for another project. **Next: EP-24** (the chat's hosted session),
> chained by the autopilot when its gate allows, else from the window.
