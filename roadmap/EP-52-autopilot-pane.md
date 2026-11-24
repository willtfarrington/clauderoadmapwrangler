---
ep: 52
title: The Autopilot pane
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-52 — The Autopilot pane

**Size:** S · **Depends on:** EP-51 · **Decisions:** D-22, D-25 (and D-21, D-29 as standing)

## Context

Added at EP-45, the owner's request 6: the autopilot's section leaves the head of the Sessions
pane's All tab for a pane of its own, its chain log folding, as the owner tasks (EP-50) and the
mail (EP-51) got theirs (the D-25 addendum of 2026-11-22).

**What the code says** (read at pickup).

- **The section:** `Board::autopilot_section` (`crates/app/src/autopilot.rs`) draws, under a
  `CollapsingHeader` "Autopilot" open by default, the state chip (paused by you, held by usage, off
  everywhere, on) with the projects it is on in and Pause or Resume; the usage refusals in amber;
  the day's line with its picker (`day_total`); each autopilot project's entry (`autopilot_projects`:
  its permission mode, its limit pickers, the gate's standing verdict, the sessions open there); the
  held chains; each chain plan with its own five-hour mark picker; the log's problem; the chain log,
  `LOG_LINES` (8) lines through `log_line`, the newest first. `Board::all_tab`
  (`crates/app/src/sessions.rs`) calls it first.
- **The reads** are the pass's, shown or hidden (`read_pilot` from `autopilot_pass`): moving the
  drawing changes no read.
- **The chain's moment:** `toasts::Target::ChainLog`, whose Show (`Board::popup`) clears the Sessions
  pane's tab and brings `Pane::Sessions` forward; the popup's hover says "the Sessions pane's All
  tab, where the chain log is". Its ntfy link opens the phone's list page (`PAGES_ROOT`), where the
  autopilot block is; the tray's Pause and Resume navigate nowhere. Both stay.
- **The panes:** `layout::Pane` (`ALL`, `title`, `default_layout`, `complete`, which adds a pane a
  saved layout lacks as a tab beside Detail, `tab_title_for_pane`, `pane_ui`). EP-50 and EP-51 put
  their panes behind Detail in the default layout, where `complete` puts them in a saved one, so the
  two agree. `View::folds` keeps the Detail pane's headers' folds by name, saved with the view.
- **Other words:** the list's autopilot box hover ("the Sessions pane's All tab logs each step",
  `list.rs`), the module docs, the screenshot setup `autopilot` (the Sessions pane's All tab in
  front), the README's autopilot paragraph and pane list, `published.rs`' doc of the phone's block.

**Session start (2026-11-24, 12:54 local time).** A headless run the window chained after EP-51a
(the chain log, 12:54: "chained EP-52 after EP-51a: claude-opus-5-5 at max, the rules' choice, S's
cap of 1 h 15 min; it holds at 80 % of the session (five-hour) window, your mark for this chain");
`crw sessions clauderoadmapwrangler` lists the row (`5f17fcb7`, headless, EP-52, open), and the
prompt is the tool's opener. The field check read, since EP-51a's record commit (12:53:14):
`crash.log`, unchanged since the panic of 2026-11-23 04:00 that EP-43a read; the rolling log,
EP-51a's install restart refused while it worked (12:53:47), then the restart into the 12:50 build
at the gate's moment carrying the chain and its plan (12:54:08), the chained start (12:54:16) and
the owner's change of this chain's mark to 90 % (12:54:32); `crw notifications` and `crw autopilot`
say the same: nothing odd. No inbox message is open; OT-31 (the phone picture) is open and gates
nothing.

**Against the cap.** S's cap is 1 h 15 min of working time from 12:54, so about 14:09; the reading
took to about 12:58. One pane made of a section that exists, a fold, one line in the All tab, a
target moved, the words that name the old place, the tests, the screenshot runs over the fixtures
and one over a copy, the checks, the release build, the records and the install; an S brief asks
for no review. **Judged at about 12:59: it fits;** no split.

## In scope

1. **`Pane::Autopilot`** ("Autopilot"), in `Pane::ALL`: a tab behind Detail after Inbox in the
   default layout, where `complete` puts it in a saved one (EP-50's and EP-51's precedent, so the
   default and an upgraded layout agree). Its tab reads "Autopilot".
2. **The pane** (`Board::autopilot_pane`, in `autopilot.rs`, the section's body in a vertical scroll
   area, no header of its own): the state line with Pause or Resume, the usage refusals, the day's
   line with its picker, each autopilot project's entry, the held chains, the chain plans with their
   marks and the log's problem, as today; then **the chain log** under a header that folds,
   **shut by default**, its newest line beside the header while shut (one line, cut with an
   ellipsis at the pane's edge, whole on hover), every line under it when open. The fold is kept in
   `View::folds` by its name, saved with the view as the Detail pane's headers are; a click on the
   triangle, the title or the newest line toggles it. An empty log says so in its one weak line,
   as today.
3. **The All tab's line** (`all_tab`): the state chip, how many projects have it on (the name when
   one has), how many chains are held, and **Open in Autopilot**, which brings the pane forward
   (`Action::OpenAutopilot`).
4. **The chain's moment:** `Target::ChainLog`'s Show brings the Autopilot pane forward (the Sessions
   pane's tab left as it is); the popup's hover and the toasts' docs say so. The ntfy link, the tray
   and the phone's block point where they did.
5. **The words:** the list's autopilot box hover, the module docs, `published.rs`' doc, the
   screenshot setup (`autopilot` brings the Autopilot pane forward; `folds-open` opens the chain log
   too; `tile=autopilot` comes with the pane's name).
6. **Tests:** the layouts (the default's place, a saved layout gaining the pane beside Detail); the
   pane drawn with the log shut (its newest line beside the header, an older one not drawn), opened
   by a click on its header and kept in the view, drawn open from the view; the All tab's line and
   its button's action; the action and `Target::ChainLog`'s Show bringing the pane; the day's line
   test drawing the pane.
7. **Screenshot runs** (rule 5): `autopilot` at 1600 × 1000 in Dark and Light, the log shut and
   open (`folds-open`); the All tab's line (`autopilot,tile=sessions`); the pane narrow (1100 ×
   900); one run over a copy of the real portfolio's roadmaps with a copied project's autopilot on;
   each looked at. *(Corrected at the close: Light was pictured at 1100 × 900 alone, the narrow
   run; Dark at 1600 × 1000, shut and open.)*
8. **The records:** the README (the pane list, a sentence on the pane, the autopilot paragraph's
   places); a D-22 addendum (where the autopilot is set and read) and a D-25 addendum (the pane).

## Out of scope

- The phone's autopilot block and the tray's menu: they keep their places and words.
- The chain log's length (`LOG_LINES`, 8; `crw autopilot` prints more) and its reads.
- A count or state in the pane's tab title.
- The rest of the Sessions pane (its tabs, filters and cards).

## Verification / acceptance

- Unit tests as in scope (6).
- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings`, `cargo
  fmt --check`.
- Screenshot runs as in scope (7), kept in the git-ignored `target\ep52-shots`; the copy and its
  pictures never enter git, and this brief names what was checked, never the projects (rule 3).
- The release build, and the install last.
- No tracked file names a managed project, the owner's login or the machine (rule 3); the word
  check prints nothing on the staged diff.

**Residual risk.** The pictures draw the fake portfolio's autopilot and a canned chain log, and the
run over a copy turns one copied project's autopilot on in the run's own registry; the owner's six
projects with the autopilot on, their real verdicts and the real log's long lines are seen first in
the window. The owner's saved layout gains the pane through `complete`, which the unit tests run on
synthetic saved layouts, not on the owner's own; it should come up as a tab behind Detail, Detail
staying in front. A chain notification's Show is exercised through `Board::lead` on a bare context,
not by a click in a real popup. The fold's memory across a restart rides on the view's saving,
which the Detail pane's headers have used since EP-51 without a fault reported, and a second
screenshot run in the same home drew the log as the first had left it. The newest line beside the
header is cut by egui at the pane's edge and shows whole on hover; the hover was not pressed in a
picture, only its truncation looked at.

> **Completion note (2026-11-24; claude-opus-5-5/max, a headless run the window chained after
> EP-51a; about 40 minutes of working time from the start at 12:54 to the work commit, no question
> asked).**
> Chained by the window after EP-51a at 12:54 (ledger row `5f17fcb7`) under S's cap of 1 h 15
> min. Judged at the brief (about 12:59) to fit, so not split. The code compiled clean at about
> 13:03 with the touched tests passing; the full tests passed by 13:09, the screenshot-feature
> tests, both clippy runs and the format check by 13:12, the pictures over the fixtures and over a
> copy by 13:15. One line changed after them (the held chains' label, below), so every check ran
> again on the final tree, then the release build, then the records, the work commit, the record
> commit and the install.
>
> **Owner decisions.** None was asked: the brief's notes settle the pane's contents, the fold and
> where a chain's Show leads, and the rest are routine choices, below. The commits are the
> autopilot's standing yes (D-22); the push is the owner's.
>
> **Built.**
>
> 1. **The pane:** `Pane::Autopilot` ("Autopilot"), a tab behind Detail after Inbox in the
>    default layout, where `complete` puts it in a saved one. `Board::autopilot_pane` draws the
>    section's body in a scroll area (the state chip with Pause or Resume, the usage holds, the
>    day's line with its picker, each autopilot project's line, the held chains, the plans with
>    their marks, the log's problem), then `chain_log`: a folding header (egui's
>    `CollapsingState`) shut by default, its newest line beside the title cut with an ellipsis at
>    the pane's edge and whole on hover, an empty log saying so there, every line under it when
>    open; a click on the triangle, the title or that line toggles it, and the fold is kept in
>    `View::folds` under `CHAIN_LOG` ("Chain log"), saved with the view.
> 2. **The All tab's line:** `Board::autopilot_line` at the head of `all_tab`: "Autopilot", the
>    state chip, how many projects have it on (the name when one has), the chains held in amber,
>    and **Open in Autopilot** (`Action::OpenAutopilot`); `state_chip` and `pilot_now` are shared
>    with the pane.
> 3. **The chain's moment:** the popup's match moved into `Board::lead`, where `Target::ChainLog`
>    leads to the Autopilot pane, the Sessions pane's tab left as it is; the popup's hover and
>    the toasts' docs say so. The ntfy link (the phone's list page), the tray's Pause and Resume
>    and the phone's block are unchanged.
> 4. **The words:** the list's autopilot box hover ("the Autopilot pane logs each step"), the
>    docs of the actions, the modules and core that named the section, and the screenshot setup
>    (`autopilot` brings the pane forward, `folds-open` opens the log too, `tile=autopilot`).
> 5. **The records:** the README's pane list and autopilot paragraph; a D-22 addendum (where the
>    autopilot is set and read) and a D-25 addendum (the pane); under EP-59's brief note, an
>    addendum parking the status bar's finding (below).
>
> **Verification.** `cargo test`: 312 in `app`, 481 in `core`, the CLI's and the integration
> suites, 855 in all, passing; `cargo test --features screenshot`: 320 in `app`, 863 in all; both
> clippy runs with `-D warnings` and `cargo fmt --check` clean, all on the final tree. The new and
> changed tests: the default layout's place for the pane, and three older saved layouts gaining it
> beside Detail; the pane's log shut with its newest line and no older one, opened by a click on
> its header and kept in the view, drawn open from the view and shut again from it, an empty log's
> words; the All tab's line (its state, no Pause) and its button's action; the action and
> `Target::ChainLog`'s Show through `Board::lead` bringing the pane forward, the Sessions pane in
> front of its own group left as it was; the day's line test drawing the pane. Screenshot runs,
> kept in the git-ignored `target\ep52-shots`, each looked at: the pane at 1600 × 1000 in Dark
> with the log shut, its newest line beside the header; the log open, every line indented under
> it, with the Sessions pane's All tab beside it, its line wrapped in the narrow pane; Light at
> 1100 × 900, the newest line cut with its ellipsis. Over a copy of three of the real portfolio's
> non-data projects (`scripts\copy-roadmap.ps1`, one at a time, into a scratch folder outside the
> repository, removed after with its run's home and pictures), one copied project's autopilot on
> in the run's registry: its long name wraps its pickers onto a second row; in the narrow
> Sessions pane the All tab's line broke "1 chain held" after its count, so that label is now
> kept whole, and the retake shows it on a row of its own; the retake, in the same home, drew the
> log open without `folds-open`, the saved view carrying the fold across the restart. The
> release build passed; the word check printed nothing.
>
> **Seen in passing, parked.** At 1100 × 900 the status bar's long notice, cut by characters
> rather than by the width left, runs under the right-hand chips: not this pane's, an S at most,
> parked under EP-59's brief note for the re-plan to place.
>
> **Session's calls.**
>
> 1. The pane behind Detail after Inbox, as EP-50's and EP-51's panes are, so that the default
>    layout and an upgraded one agree; not beside Sessions, where a saved layout would not put it.
> 2. The fold kept with the view (`View::folds`), across restarts as the Detail pane's headers
>    are, rather than in egui's memory for the window's life.
> 3. The newest line cut beside the header with an ellipsis, whole on hover, since a folding
>    header's text never wraps (EP-13); a click on it opens the log too.
> 4. A chain's Show leaves the fold as the owner left it: the newest line, the one the
>    notification is about, shows beside the header either way.
> 5. The All tab's line says how many projects have the autopilot on, not their names (the pane
>    lists them), keeps the held chains' label whole, and carries no Pause (the pane's and the
>    tray's).
> 6. The popup's match moved into `Board::lead`, so that the chain's Show is tested without a
>    popup.
> 7. The tab reads "Autopilot", without a count.
> 8. The README's pane list gains Inbox, which EP-51 left out.
>
> No owner task and no message to another project.
