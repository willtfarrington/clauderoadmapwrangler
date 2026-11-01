---
ep: 6
title: Dashboard shell
size: L
model: claude-opus-5-5
effort: max
status: done
---

# EP-6 — Dashboard shell

**Size:** L · **Depends on:** EP-2, EP-4 · **Blocks:** EP-7, EP-8, EP-10, EP-13, EP-14, EP-17, EP-24

## Context

EP-2 … EP-5 built the readers (the registry and discovery, the roadmap parsers, the owner's items
and the inbox), and `crw` prints what they read. EP-6 is the first window: the tile shell that later
briefs fill, a dense project list and a detail tile over the same data, the writes the tool already
owns (Adopt, Ignore, a confirmed proposal, pin and priority, the ticks), and the screenshot feature
that lets every later session check the UI against fake projects (rule 5) and gives EP-8 its
README pictures. It is one of the six L engines (D-4).

**Session start (2026-11-01).** Started from `DATA\clauderoadmapwrangler.code-workspace`. The inbox
held nothing open (IN-1 and IN-2 were triaged at EP-4); at the start of this session the owner asked
for a line about putting `crw` on PATH, which the session wrote as IN-3 and triaged into EP-28's
brief notes (the install script final), to be confirmed at the close. No `roadmap/owner-tasks.md`.

**Verified at pickup.**

- eframe 0.37.0 draws through `App::ui(&mut self, ui, frame)`; panels are `egui::Panel::top` /
  `bottom` and `CentralPanel::show(ui, …)`; `Context::request_screenshot(callback)` hands back the
  frame's `ColorImage`; `NativeOptions::persistence_path` moves eframe's state file and
  `App::raw_input_hook` lets a run inject keys. The owner's other egui app uses all of them on the
  same pins, and its docking layout (a saved `egui_tiles` tree completed with panes a later build
  adds) is the pattern here.
- eframe already depends on `image` 0.25.10 with its `png` feature, so writing a PNG adds no crate
  to `Cargo.lock`.
- egui_material_icons 0.9.0 (MIT) depends on egui 0.37.0 alone (`include-flate` is optional); its
  default `filled` feature embeds a 1.8 MB font.
- egui_tiles 0.18.0, egui_extras 0.37.0 and notify 8.2.1 are pinned in `[workspace.dependencies]`
  but not yet in `Cargo.lock`, since no member used them; this brief adds egui_tiles and notify.
- The fixtures of EP-3 … EP-5 are test-shaped (edge cases, format names as project names); a
  screenshot needs a portfolio that reads as one, so the fake projects are new fixtures.

## In scope

1. **The shell** (`crates/app`): eframe with persistence; an `egui_tiles` tree of six panes:
   Projects (left, wide), Detail (centre), and Decisions, Sessions, Chat and Usage tabbed on the
   right as placeholders that name the brief filling each (EP-14, EP-13, EP-24, EP-17); the tree
   saved in eframe's state with the window's size and place, completed on restore with any pane a
   later build adds; View → Reset layout; a menu bar and a status bar for notices and errors.
   With `CRW_HOME` set, eframe's state lives there too, so a trial never moves the owner's layout.
2. **The data** (`portfolio`): the registry through `Store::locate()` → `discovery::view` → each
   registered project's `snapshot::read_tree` (file reads only, so the list shows at once); then a
   git pass on a worker thread, one child at a time with the spacing (D-29): `snapshot::take` for
   the `HEAD` reading and the status, `proposal::last_commit`, each project's row filled in as its
   turn ends; outgoing mail from `inbox::outgoing` resolved through `Registry::project`; a `notify`
   watcher on the data root's top level that re-reads (files only) when a folder appears, goes or is
   renamed (D-7); Refresh (F5) re-reads with the git pass; a write re-reads the files and keeps the
   git readings.
3. **The Projects pane**: collapsible groups by state (active, paused, maintenance, unformalised,
   unadopted, done, archived; done and archived collapsed at first), pinned first, then a sort key
   (priority, completion, remaining, last activity, name; hosting and usage share shown and disabled
   until EP-7 and EP-18); a name filter, and the stack and hosting filters shown and disabled until
   EP-7. One row per project: an expander, the pin, the name, a state chip, the next EP's id, size
   and one-line title, done over total, the open owner items and inbox counts, the badge slot EP-7
   fills, the session status (in progress, with the reasons as hover text), the autopilot toggle
   (disabled until EP-19) and Start (disabled until EP-10). A row expands inline to its per-phase
   counts with the graphs' placeholder (EP-26); a right-click menu pins, sets the priority or the
   state; an unadopted folder's row offers Adopt and Ignore.
4. **The Detail pane**: for the selected project its state, folder and decision log; its proposals,
   each proposed state with a Confirm button (`proposal::propose`, then `set_state`); the next EP
   (id, size, title, phase, depends on) with its brief-notes bullet verbatim and its brief's front
   matter, completion evidence and pickup gate; why a session is taken to be in progress; the
   `HEAD` reading when it differs; per-phase counts; the open owner items (a box on the tickable
   ones; the older forms read-only, with their form, gate and partly-resolved state), the resolved
   ones counted and the ticked tasks with Untick; the inbox (each open message with a disposition
   field and Tick, the ticked ones with their dispositions); the handed-on blocks with their
   recipients (registered, or unknown and not delivered, D-13); the problems (the roadmap's, the
   snapshot's, the checklist lines'). For an unadopted folder: what it holds, Adopt and Ignore.
5. **The writes**, all of them the tool's own (D-7, D-8, D-10, D-11, D-13): Adopt and Ignore
   through `discovery`, Confirm, pin, priority and state through `Store::change`, the ticks
   through `tasks::tick` / `untick` and `inbox::tick` with the store's backups folder; each outcome
   or error in the status bar, a tick's with the copy's path; nothing else written into a project.
6. **Look and keys**: a type scale for a 4K-class display in compact and comfortable densities;
   theme System / Light / Dark; the Material Symbols icon font (egui_material_icons 0.9.0); keys
   ↑ ↓ select, → ← expand and fold the row, Enter brings the Detail tile forward, S and A answer in
   the status bar that Start (EP-10) and the autopilot (EP-19) are not built yet, F5 refreshes.
   Saved with the layout: theme, density, sort key, filter text, folded groups, expanded rows and
   the selection.
7. **The screenshot feature** (`--features screenshot`, rule 5): `CRW_SCREENSHOT=<file.png>` runs
   the app on a fake portfolio, saves its own frame as a PNG and exits; nothing outside the window
   is captured. Its home is `CRW_HOME` when set, else a `crw-screenshot` folder beside the picture:
   the registry, written there from a tracked template on the first run (so a second run starts from
   the first one's changes), eframe's state and the backups. The fake projects are tracked fixtures
   in `crates/app/fixtures/portfolio/` with synthetic names (D-3), so a screenshot run writes
   nothing into them: ticks are refused there with a notice. `CRW_SCREENSHOT_SETUP`
   (comma-separated): `keys=<chord>;…` (one chord per frame through the input hook), `light`,
   `dark`, `compact`, `comfortable`, `select=<project>`, `tile=<pane>`, `expand=<project>`,
   `size=<w>x<h>`.
8. **Tests**: grouping, sorting and filtering on synthetic rows; the key order; the layout's
   default, its round trip through eframe's storage and its completion; the setup flags; the
   portfolio's two passes on the fake portfolio and on a scratch data root with one git repository
   (as EP-4's tests make one).

## Out of scope

- The badges and their data (stack chips, the progress bar, activity, hosting, health, backup age:
  EP-7); the filters they feed stay disabled until then.
- Start and the launch dialog (EP-10), the autopilot (EP-19), the sessions, decisions, chat and
  usage tiles (EP-13, EP-14, EP-24, EP-17), the graphs of the inline expansion (EP-26), the cards
  view of D-25, toasts and the tray (EP-15); single instance, the app icon, start-at-login and the
  rolling log (EP-28).
- Composing inbox messages and delivering handed-on blocks (EP-20); the IN-1 follow-ups on Adopt
  (per-project workspace files, a rule for `workspace_file`; EP-8's re-plan decides where).
- Any write into a project beyond the ticks; the owner's registry in tests or trials (rule 13).

## Verification / acceptance

- The gate passes with no warnings: `cargo test`, `cargo clippy --all-targets --features
  screenshot -D warnings`, `cargo fmt --check`, `cargo build --release`; `install-shortcut.ps1`
  last.
- Screenshot runs on the fake portfolio, each picture looked at: the default layout in both themes
  and both densities; selection by injected keys with the Detail tile showing the next EP's notes,
  the owner items, the inbox, the handed-on blocks, a proposal and the problems; the unadopted
  group; an expanded row; a second run that starts from the first run's registry and layout. No
  screen capture; `git status` shows nothing written into the fixtures.
- Unit and fixture tests as in scope 8; the git pass's children counted one at a time.
- No private project name in the tracked files (`git grep` for the names kept in `private/`).

> **Completion note (2026-11-01; opus/max; ≈ 40 min to the owner's questions).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`; at its start the owner asked for an inbox line about
> putting `crw` on PATH, written as IN-3 and triaged into EP-28's brief notes. Owner decisions
> (`AskUserQuestion`, every one the recommended option): **IN-3**: parked in EP-28's brief notes, the
> line ticked with that disposition; **row menu**: the right-click menu (pin, priority, state) kept,
> beyond the roadmap note's scope; **addenda**: the D-25 and D-29 addenda recorded as written;
> **gate**: rule 6 and CI gain `cargo test --features screenshot` and a clippy without the feature,
> since only that one caught the code the feature alone used; **commit**: both commits in the house
> pattern, pushed to the private remote. Routine choices made by the session: a new fake portfolio
> (`crates/app/fixtures/portfolio`, ten folders with invented names, a registry template) rather than
> the test-shaped fixtures of EP-3 … EP-5; `snapshot::read_git` split out of `take`, which behaves as
> before, so the list shows the files first and adds git's readings, one lock keeping every git child
> of every worker thread one at a time; rows drawn by hand rather than with an `egui_extras` table, so
> group headers and expansions span the row, the title taking what the fixed columns leave and the
> counts and the progress giving way below 160 points; a small inline renderer (`markup`) that draws
> `**bold**`, code and `~~struck~~` and joins continuation lines, for brief notes, owner items and
> messages; Enter and a double click bring the Detail pane forward, → ← expand and fold; the watcher
> counts only creations, removals and renames on the data root (a write inside a project only touches
> its folder's times); `CRW_HOME` also moves eframe's state (`paths::home` made public); a spinner
> beside Refresh while a read runs; setup flags `adopt=`, `ignore=`, `pin=` and `tick=` that queue the
> same actions a click does; `image` 0.25.10, already locked by eframe, only behind the feature;
> egui_material_icons 0.9.0 added to the pins; `Cargo.lock` gained egui_tiles, egui_material_icons,
> notify and notify's dependencies for other platforms, and no version moved. Verified: `cargo test`
> 125 passed (11 new in the app), 127 with the feature (13 new), clippy with and without the feature
> and `cargo fmt --check` clean, the release build in 17 s through `install-shortcut.ps1`, which
> refreshed both shortcuts; seven screenshot runs on the fake portfolio, each picture looked at. The
> first two found the row's columns too wide for the list (titles cut after a word, Start off the
> edge) and the brief notes shown with their source line breaks and markup, both fixed; then five runs
> from one home: dark with lantern selected (an urgent message, mail to a registered and to an unknown
> project); ↓ ↓ → Enter typed from the saved selection to quarry, light and comfortable (its
> pickup-gate proposal with Set paused, its row expanded); an Adopt of the unadopted folder, registered
> in the run's registry after its dated copy, with a refused tick (harbor's OT-2 still open in the
> fixture); dark and compact with the adopted folder; orchard's three older-form items with their `on
> or after` gate. Theme, density, selection and expanded rows carried from run to run; a clipped inbox
> count at the comfortable density was widened. `crw tasks` read the fake projects and this
> repository's new `owner-tasks.md`; no private project name in the tracked or new files. Owner task
> OT-1 (the window on the real portfolio, before EP-7). No messages for other projects. Next: EP-7
> (badges), with EP-6's pickup notes in its brief notes and in those of EP-8 and EP-10.
> **After the commit (≈ 50 min in all):** `1b5bfca` and the record commit `62d2204` pushed together at
> 21:20 UTC; CI run 41351508475 on windows-latest passed in 2 min 13 s (format, clippy with and
> without the screenshot feature, the tests without and with it). Because the record commit went out
> with the work, this line is a third commit, on the owner's yes.
