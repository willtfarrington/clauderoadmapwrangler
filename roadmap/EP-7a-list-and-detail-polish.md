---
ep: 7a
title: List and detail polish
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-7a — List and detail polish

**Size:** M · **Depends on:** EP-7 · **Blocks:** EP-8

## Context

The owner looked at the window on the real portfolio after EP-7 (OT-2) and sent four notes (IN-4):
the Projects pane's columns share width clumsily, so even a very wide window packs the badges
tight and clips the stack chip; the badges need a key besides their hover texts; the Status section
spells license two ways; and the Status section's subsection names float in the middle of their
lines. EP-7's close turned the notes into this row, before EP-8 takes the README's pictures and
flips the repository public, with the owner's choices: columns sized to their content under a
header row, a Legend toggle in the Projects pane's toolbar, and the spelling license (D-25).

**Session start (2026-11-01).** Started from `DATA\clauderoadmapwrangler.code-workspace`. The inbox
holds nothing open (IN-1 … IN-4 are triaged; IN-4 is this brief); OT-1 and OT-2 are ticked.

**Verified at pickup.**

- egui 0.37's scroll bars float by default (`ScrollStyle::floating()`, no width allocated), so the
  rows inside the list's scroll area are as wide as a header row drawn above it: one column plan
  per frame serves the header and every row.
- `egui::Panel::bottom(id).show(ui, …)` shows a panel inside any `Ui` (0.37 renamed `show_inside`
  to `show`), so the legend can sit at the foot of the pane while the list's scroll area takes the
  rest.
- `WidgetText::into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, …)` measures a
  `RichText` as it is drawn (its family and size, monospace, the icon font) through egui's galley
  cache, cheap enough for the portfolio's rows every frame.
- `egui_material_icons` 0.9 has `ICON_LEGEND_TOGGLE`; `MaterialIcon` is `Copy` with a `const fn
  new`, so the badges' icons can sit in constant tables that the row and the legend both read.
- Today's widths are fixed in `list.rs`: `NAME` 160, `STATE` 104, `NEXT` 70, `PROGRESS` 44,
  `ITEMS` 72, the badges `FULL` [36, 18, 36, 46, 38, 18] and `COMPACT` [18, 36, 38, 18] three points
  apart; the stack chip (small text inside 7-point margins) outgrows its 36 points with `RS+1`.
- egui's `Grid` aligns every cell left and centre (a fixed alignment in its source), which is why
  the Status section's one-line names sit in the middle of their multi-line blocks.
- "licence" appears in the window's Status section (`Licence: <file>`, `No licence file at the
  root`), in `crw`'s usage text and `crw status`'s `no licence file`, in the identifiers `licence`,
  `licence_warning` and `health::licence` with their comments, and twice in the README; the hover
  text already says `LICENSE file`.

## In scope

1. **Columns sized to their content** (`list.rs`): each frame, the widest value the portfolio holds
   is measured per column with the window's own fonts — the name (the projects' and the unadopted
   folders'), the state chip with the proposal's bulb, the next EP with its size, the progress pill,
   the counts, the session mark, and each of the badge column's own columns (the stack chip, the
   hosting, the sync, the age with the dirty dot, the CI run with the license warning, the backup)
   — over every project and folder, folded or filtered out or not, so the columns stay put; each
   column is at least as wide as its header's label; padding between columns.
2. **A header row** above the list, outside its scroll area so it stays while the list scrolls: a
   slim row of small, weak column names over the columns the rows show (Project, State, Next,
   Brief, Done, Waiting, Stack, Hosting, Sync, Age, CI, Backup, Session; the autopilot and Start
   controls unnamed), with a line below it.
3. **Sharing the width**: the title takes what is left after the columns, at least its minimum; on
   a wide window part of the spare width widens the gaps between the columns (up to a cap), so the
   badges are not packed tight, and the title takes the rest; a narrow window keeps D-25's give-way
   order (badges compact, then the counts go, then the badges, then the progress; EP-7 addendum).
   The plan is a pure function of the measured widths and the row's width, unit-tested.
4. **The legend** (`badges.rs`, `list.rs`): every mark the row draws — the badges' icons and texts
   with their tones, the stack chips, the pin, the proposal's bulb, a session in progress, git not
   asked yet, the counts' icons — is defined once in constant tables of signs (glyph, tone,
   meaning) grouped as the badges are (stack, hosting, sync, activity, CI and license, backup, the
   row's other marks); the badge functions build their marks from those signs, so the key cannot
   drift from the row. A Legend toggle in the toolbar (`ICON_LEGEND_TOGGLE`) shows or hides the key
   in a panel at the foot of the pane, groups side by side as the width allows; its open state is
   saved with the view (`View::legend`, `#[serde(default)]`, so a saved view without it reads).
5. **One spelling**: license in the window (the Status section), in `crw`'s output and usage text,
   in the identifiers and comments of the three crates (`health::license`, `Badges::license`,
   `license_warning`) and in the README; DECISIONS.md keeps its wording, and so do the fixtures'
   synthetic project texts and the file names the reader accepts (`LICENCE*`).
6. **The Status section by subsection** (`detail.rs`): a fixed name column, each name level with
   its subsection's first line; a mark column, so every sentence starts at the same place and
   wraps under itself; a divider in the theme's separator colour between subsections; spacing in
   fixed points, the same at every window size and in both themes; no `Grid`.
7. **Screenshots**: a `legend` setup flag opens the key in a screenshot run.
8. **Tests**: the column plan (give-way order, no column narrower than its widest value, the gaps'
   share capped, the row filled to its right edge), the legend's tables (every sign a badge
   function returns is in its group's table; each group has distinct glyph-and-tone pairs), the
   view without `legend` reading as closed.

## Out of scope

- Drag-resizable columns with saved widths (v2 notes, the alternative the owner did not pick).
- Sorting by a click on a header (the Sort menu stays the one place).
- The README's pictures and the public flip (EP-8); D-24's actions (placed by EP-8).
- DECISIONS.md's own spelling; any change to what a badge reads or how often.

## Verification / acceptance

- The gate of rule 6 passes with no warnings, `install-shortcut.ps1` last.
- Unit tests as in scope 8; no test runs the `claude` CLI or the network (D-29).
- Screenshot runs on the fake portfolio at 1600 × 1000, 2300 × 1000 and 1100 × 1000 points, in
  both themes, with the legend open and closed, each picture looked at: no chip clipped, the header
  over its columns, the badges loosened on the wide window, the give-way order on the narrow one,
  the Status section's names level with their first lines and divided.
- No `licence` left in the window's text or `crw`'s output (`git grep -i licence` over `crates/`
  finds only fixtures' project texts and the file-name variants).
- No private project name in the tracked or new files (`git grep` for the names kept in
  `private/`).

> **Completion note (2026-11-01; opus/max; ≈ 40 min to the owner's questions).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`; the inbox held nothing open (IN-4 is this brief) and
> OT-1 and OT-2 were ticked. Owner decisions (`AskUserQuestion`, every one the recommended option):
> **commit**: both commits in the house pattern, pushed to the private remote, and a third
> recording the CI run once it finishes; **addendum**: D-25's EP-7a addendum recorded as written;
> **README**: the README says license too (its status line and its Notes), while DECISIONS.md and
> the older briefs keep licence; **parser fix** (asked after the others, when `crw roadmap .` read
> this brief's `ep: 7a` as no EP): `FrontMatter::ep` also tries the value as `EP-<value>`, so a
> lettered row's brief reads in the house form, included in this commit with an assertion in the
> house brief's test. Routine choices made by the session: every cell measured by the
> code that draws it (`style::width_of`, an invisible sizing-pass child that takes no room), over
> every project and folder each frame, so no column clips its widest value or moves as rows fold
> or filter; each column at least as wide as its header's name, which costs width in the icon
> columns (Hosting, Backup, Session) but keeps the header whole; one pure `Plan` per frame from the
> measured widths and the pane's width, shared by the header row and every row, with five unit
> tests; the gaps' share settled at three tenths, at most 12 points a gap, after the first
> 2300-point picture showed 0.4 and 14 taking too much from the titles; the badges' marks rebuilt
> from constant `Sign` tables (glyph, tone, a few words) in `badges.rs`, the row's other marks
> (pin, bulb, session, git not asked, the counts' icons, the folder) moved into the same tables,
> and the legend drawn from them, with tests that every mark a badge function returns is in its
> legend group and that no two signs of a group look alike; the legend as a bottom `Panel` sized to
> the measured key (a scroll area inside a panel fills only the height the panel already has, so
> the first picture showed two lines), its groups in as many aligned columns as fit, its lines as
> tall as their text, its meanings cut to a few words so the key takes two lines at 1600 points;
> the header labels with hover texts; the Status section as three columns through `style::padded`
> (name, mark, sentence) with egui separators between subsections and no `Grid`; `licence` renamed
> to `license` in the identifiers (`health::license`, `Badges::license`, `license_warning`) and the
> comments of the three crates, the fixtures' synthetic texts and the accepted file names
> (`LICENCE*`) left alone; `layout::tests::MemoryStorage` shared with `board.rs`'s new saved-view
> test; OT-3 (the owner's look at the real portfolio, before EP-8) and a pickup note in EP-8's
> brief notes. Verified: `cargo test` 164 passed (9 new), 166 with the feature (9 new), clippy with
> and without the feature and `cargo fmt --check` clean, the release build through
> `install-shortcut.ps1`, which refreshed both shortcuts; `crw help` and `crw status --offline` on
> this project print license; `git grep -i licence` over `crates/` finds only the fixtures'
> project texts, the file-name variants and the parsers' test texts; ten screenshot runs on the
> fake portfolio, each picture looked at: the baseline at 2300 points light, then 2300 light, 1600
> dark with the legend twice (the second after the panel's fix), 1600 light with the legend, 2300
> dark with the legend (and a full-resolution crop of the badge columns: no chip clipped), 2300
> light, 1100 light, 1100 dark with the legend, and 2300 dark comfortable with an expanded row. No
> private project name in the diff or the brief. No messages for other projects. Next: EP-8 (the
> P1 re-plan), after OT-3.
