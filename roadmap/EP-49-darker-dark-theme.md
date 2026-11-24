---
ep: 49
title: The dark theme on a darker base with more contrast, after the One Hunter extension's Vercel theme, the accents keeping their meanings
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-49 — The dark theme on a darker base with more contrast

**Size:** M · **Depends on:** EP-46 · **Decisions:** D-25 (and D-15, D-29, D-3 as standing)

## Context

The owner's request 3 at EP-45, and their choice there: **Dark is replaced** by a theme on a
near-black base with more contrast, after the One Hunter extension's Vercel theme they use in VS
Code; Light stays; no fourth choice (D-25 addendum of 2026-11-22).

**What the code says** (read at pickup).

- No palette exists. `Dashboard::new` (`crates/app/src/board.rs`) restores the `ThemePreference`
  under `THEME_KEY` and calls `set_theme`; the View menu's System / Light / Dark radio buttons call
  it again. egui keeps a dark and a light `Style` of its own (`Options::dark_style`,
  `light_style`), both egui 0.37's stock `Visuals::dark()` and `light()`, and never persists them
  (`serde(skip)`), so visuals set at the window's creation hold for its life and a theme switch
  picks between them. `Density::apply` changes only the text styles and the spacing of both.
- The accents live in `crates/app/src/style.rs`: `green`, `blue`, `red` and `reference` pick a
  shade by `dark_mode`; `amber` is the visuals' `warn_fg_color` (egui's orange, 255 143 0, in
  Dark); `state_colour` and `stack_colour` are one shade for both themes, drawn as chips (`chip`:
  a 32 % tint with its ink lifted toward white in Dark) and as the group header's bulb.
- The visuals the app reads beyond the text colours: `faint_bg_color` fills the cards (the Detail
  pane's notes, the decisions, the chat's exchanges and conversation rows, the ledger cards) and
  the progress pill's track; `widgets.hovered.weak_bg_fill` a hovered row and group header;
  `selection.bg_fill` at 55 % the selected row (`list::tint`); `extreme_bg_color` the meter's bars;
  `window_fill` and `window_stroke` the popup's own window (`popup.rs`); egui_tiles takes its tab
  bars and the gaps between panes from `extreme_bg_color` in Dark, the active tab from
  `panel_fill` with an outline of `widgets.active.bg_fill`; egui_commonmark (the chat's
  Markdown) its code blocks from `extreme_bg_color` and the noninteractive stroke, its text from
  the text colours, and the chat sets its links' colour to the violet (`chat.rs`). A text box at
  rest takes its frame from `widgets.inactive.bg_stroke`, which egui's Dark leaves empty.
- The phone's pages carry one style sheet (`crates/app/templates/app.css`) with a dark block under
  `prefers-color-scheme: dark`; no test pins its colours.

**The source.** The extension's `themes/OneHunter-Vercel-color-theme.json` ("One Hunter Theme
Vercel", version 1.4.1, under `%USERPROFILE%\.vscode\extensions\raillyhugo.one-hunter-1.4.1`),
its `colors` read. Its license, `LICENSE.txt`, is MIT (the copyright line left as the template's
`[year] [fullname]`; the publisher is Railly Hugo, `github.com/Railly/one-hunter-vscode`). MIT
asks for its notice in copies or substantial portions of the software; a dozen colour values
taken from a theme file are neither, so it does not ask for a credit, and one is the owner's
call at the close. The values taken, and what each becomes in Dark (an alpha value composed over
the black, as VS Code composes it):

| The theme's key(s) | Value | In the window's Dark |
|---|---|---|
| `editor.background`, `panel.background`, `terminal.background`, `input.background` | `#000000` | the panels; a text box's well |
| `list.hoverBackground`, `list.focusBackground`, `editorGroupHeader.tabsBackground` | `#0A0A0A` | dialogs, menus, tooltips and the popup (`window_fill`); a pressed or focused button, darker than at rest as in egui's Dark (after the review) |
| `editorWidget.background`, `editorInlayHint.background`, `editorRuler.foreground` | `#34393E` | separators; a checkbox's, a radio's and a slider's fill; at 35 % the cards (`#121416`), at 50 % a button at rest and the wells (`#1A1C1F`: egui's `extreme_bg_color`, so the tab bars, the tracks, a plot's and a sparkline's ground, a heatmap's empty day and a code block, after the review), at 60 % code's background and a progress pill's track (`#1F2225`), at 75 % a hovered button or row (`#272B2F`) |
| `panel.border`, `editorWidget.border`, `input.border` | `#454D54` | the frame of dialogs, buttons and text boxes; the active tab's outline |
| `foreground`, `editor.foreground` | `#DCE3EA` | the text (egui's Dark had a grey of 140) |
| `tab.inactiveForeground` | `#DCE3EA88` | the weak text, lifted from its `#75797D` over black to the foreground at 72 % (`#9EA3A8`), so it keeps 4.5 to 1 on the cards, a hovered row and the selection |
| `editor.selectionBackground` | `#43AAF955` | the selection (`#163953`) |
| `terminal.ansiBrightBlue` | `#A7D1F5` | a selected button's text and a focused text box's outline |

Read and not taken: `sideBar.background` `#010101` (no step from the black); the theme's pink,
`#DD4F7D` (`focusBorder`, `badge.background`, `list.activeSelectionBackground`,
`editorLink.activeForeground`), a colour the window does not use; `button.background` `#43AAF9`
(every button blue would read as busy); the accents `#5BD1B9`, `#FAC760`, `#E61F44`, `#B267E6`,
`#43AAF9`, since the window's accents keep their shades unless a contrast fails (below); the
token colours.

**Session start (2026-11-23, 22:50 local time).** A chained headless run: the window restarted
into EP-48's build at 22:49:59 and chained EP-49 at 22:50:08 (the rolling log and `crw
autopilot`: claude-opus-5-5 at max, the rules' choice, M's cap of 1 h 30 min, the chain's own
mark at 80 % of the session window); `crw sessions clauderoadmapwrangler` lists the row
(`4873ab98`, headless, EP-49, open), and the prompt is the tool's chained opener. The field
check read, since EP-48's record commit (22:49:02): `crash.log`, unchanged since the panic of
04:00 that EP-43a read; the rolling log, EP-48's install restart refused while it worked
(22:49:39), the run's end, the restart at the gate's moment carrying the held chain and its plan,
and this chain; `crw notifications` and `crw autopilot`, nothing odd. No owner task and no inbox
message is open.

**Against the cap.** The code is one function of visuals and its constants in `style.rs`, a line
at the window's creation, perhaps the gaps between panes in `layout.rs`, and the phone's dark
block; the rest is the contrast test, the screenshot runs and their looking, the review an M
asks for, the README's picture and the records: judged at 23:00, ten minutes in, to fit M's cap
(00:20) with room. Should `style::dark()` and its test not pass by 23:35, the phone's style sheet
goes to a v2 note and the runs over a copy are cut to one size.

## In scope

1. **`style::dark()`** builds egui's `Visuals` on the palette above, each value a named constant
   whose doc comment names its source key: the panel, window and extreme backgrounds, the text
   box's well, the faint (cards) and code backgrounds, the text, weak and strong colours, the
   widgets' fills, strokes and inks at rest, hovered, pressed and open, the selection, the links
   in the violet of `style::reference`, the separators, the window's frame; the shadows and the
   rest as egui's Dark has them. `warn_fg_color` stays egui's orange (it is `style::amber`), and
   `error_fg_color` becomes `style::red`'s dark shade, so egui's own error text matches.
2. **Applied once:** `Dashboard::new` sets it as egui's dark visuals before the theme is set, so
   System and Dark both draw it and the View menu's switch needs nothing new; the screenshot
   feature's `dark` flag draws it the same way.
3. **The accents keep their meanings and shades** (amber waiting, green done or healthy, violet
   references, blue busy, red failing): a test measures each accent's text contrast in Dark (and
   the text and weak colours') on the panel, a dialog, a card, code's background, a hovered row
   and the selected row, and each chip's ink on its tint; any pair under 4.5 to 1 has its accent
   tuned in Dark only. The arithmetic is WCAG 2's relative luminance.
4. **What takes its colours from the visuals**, checked in pictures: the popup, the dialogs, the
   transcript, the chat's Markdown (egui_commonmark), the tab bars and the gaps between panes
   (drawn in the separator colour in Dark if the black leaves them unseen: the egui_tiles
   behaviour's `resize_stroke`, Light untouched).
5. **The phone's dark style sheet** takes the palette's base, card, text, weak, line, chip and
   code values, its accents kept; checked for contrast as the window's are.
6. **Pictures:** screenshot runs (rule 5) of every pane and the dialogs in Dark at 1600 × 1000,
   2300 × 1000 and 1100 × 900, and over a copy of the real portfolio's roadmaps, each looked at;
   the README's dark picture retaken from the neutral root with EP-32's setup.
7. **The records:** a D-25 addendum; the README's picture and, on the owner's word, a credit for
   the theme.

## Out of scope

- Light, which stays egui's own (the owner's choice at EP-45); a fourth choice.
- The accents' hues, kept unless a contrast fails; the theme's pink and its token colours; a
  syntax colouring of code blocks.
- The egui pins (D-15).
- The phone's README picture (`docs/phone-list.png`), which the owner captured in a browser
  (rule 5: no session drives one); what becomes of it is put to the owner at the close.

## Verification / acceptance

- Unit tests: the contrast of each accent and of the text and weak colours on each surface of
  Dark at 4.5 to 1 or more, each chip's ink on its tint likewise; `style::dark()` is dark, its
  panels black, its weak colour set; the Light visuals untouched (egui's own).
- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings`, `cargo
  fmt --check`; a review subagent given the diff (an M brief), launched once the code compiles.
- Screenshot runs as in scope (6), kept in the git-ignored `target\ep49-shots`, each looked at;
  the copy and its pictures never enter git, and this brief names what was checked, never the
  projects (rule 3).
- The release build, and the install last.
- No tracked file names a managed project, the owner's login or the machine (rule 3); the word
  check prints nothing on the staged diff.

**Residual risk.** The pictures are the app's own frames of the fake portfolio and of a copy of
the real roadmaps, drawn by the same renderer at the screenshot's scale; the real window on the
owner's 4K display at 150 % draws the same colours, but how the near-black reads on that panel
and in a dim or bright room is the owner's to judge at P6's walk. The contrast test measures the
surfaces the code names; a colour the code draws on a fill it does not name (a hovered menu item
over a dialog, a tooltip over a card) is checked only in the pictures that happen to show it.
The popup is a window of its own; its picture is drawn inside the main window by the screenshot
feature, so its frame on the desktop is not pictured. A pressed or focused button and a hovered
one are measured, not pictured, since a screenshot run presses and hovers nothing. The phone's
dark sheet is checked by its values only, not in a browser (rule 5), until the owner's capture
for OT-31 shows it.

> **Completion note (2026-11-24; claude-opus-5-5/max, a chained headless run; about 35 minutes of
> working time from the start at 22:50 on 2026-11-23 to the work commit, 21 before the owner's
> question and 14 after its answer, the hour it waited not counted).**
> Chained at 22:50 after EP-48 (ledger row `4873ab98`) under M's cap of 1 h 30 min of working
> time. Judged at the brief to fit with room, so nothing was split: the code and its tests
> passed at 23:02, the screenshot series and the review ran by 23:11, when the owner's question
> went out; it was answered from the phone at 00:14, and the review's fixes, the final pictures,
> the checks and the records followed. The work commit `c879c7c` at 00:27, the record commit,
> then the install.
>
> **Owner decisions** (one `AskUserQuestion` at 23:11, both answered with the recommended
> option).
>
> 1. **The theme's credit:** one sentence in the README's Notes names the One Hunter Theme's
>    Vercel variant by Railly Hugo (MIT). Not chosen: no credit (MIT does not ask one for colour
>    values, and the decision log names the extension already).
> 2. **The README's phone picture**, which shows the old dark shades: OT-31, the owner's capture
>    of the standalone page this session wrote to `target\ep49-phone\phone-list.html` from a
>    neutral copy of the fixtures, committed by the owner. Not chosen: leaving it as is.
>
> The commits are the autopilot's standing yes (D-22); the push is the owner's.
>
> **Built.**
>
> 1. **`style::dark()`** (`crates/app/src/style.rs`): the palette as named constants, each doc
>    comment naming the theme's keys; `apply_themes` sets it as egui's dark visuals in
>    `Dashboard::new` (`crates/app/src/board.rs`), Light left egui's own; `RED_DARK` and
>    `VIOLET_DARK` shared with `red` and `reference`; `chip_colours` out of `chip`, so the test
>    measures what a chip draws; `track`, a progress pill's ground.
> 2. **The lines between panes** (`crates/app/src/layout.rs`): `resize_stroke` and
>    `resting_line`, the separators' colour in Dark, egui_tiles' own in Light.
> 3. **The chat's Markdown** (`crates/app/src/chat.rs`): the resting frame dropped in its scope,
>    one outline per code block; the list's progress pill on `style::track` (`list.rs`); the
>    sparkline's doc comment (`analytics.rs`).
> 4. **The phone's style sheet** (`crates/app/templates/app.css`): the dark block's base, card,
>    text, weak, line, chip and code values, and `--on-accent` for a button's words.
> 5. **The records:** the README's dark picture, retaken from the neutral root
>    `C:\Users\Public\Projects` with EP-32's setup and a fresh home, the copy removed after; its
>    alt text and the credit; a D-25 addendum; OT-31.
>
> **Verification.** `cargo test`: 290 in `app`, 479 in `core`, the CLI's and the integration
> suites, all passing; `cargo test --features screenshot`: 298 in `app`; both clippy runs with
> `-D warnings` and `cargo fmt --check` clean, all on the final tree. The new tests: the window's
> Dark is `dark()` and its Light egui's own; every accent, the text and the weak text at 4.5 to 1
> or more on each surface the code fills (the panel, a dialog, a card, code, a well, a button at
> rest, pressed and hovered, the selected row and the selection), a selected button's words on
> the selection, and each chip's ink on its tint over the panel, a card and a dialog; the wells
> and a pill's track off the black and the cards, a pressed button darker than one at rest, and
> Light's track unchanged; the lines between panes; WCAG's arithmetic on its known pairs. No
> accent needed tuning. Screenshot runs in Dark, kept in the git-ignored `target\ep49-shots`,
> each looked at: at 1600 × 1000 the list with an expanded row and the legend, the Detail pane's
> history and graphs and the Decisions pane, the Sessions pane with a transcript, the Chat pane
> with every Markdown form, the Usage pane, the Diagnostics pane, the Autopilot section, the
> launch, revert, compose, Make public, formalise, quit and Enable remote view dialogs, and the
> popup; at 2300 × 1000 the list with Decisions, the Chat with the usage strip and the
> transcript; at 1100 × 900 the list, the Diagnostics pane, the launch dialog with its chain
> plan, the Chat and the cards. Over a copy of the real portfolio's twelve non-data projects
> (`scripts\copy-roadmap.ps1`, one at a time, into a scratch folder outside the repository,
> removed after): every row expanded with the legend at 1600, the list at 1100 and the cards at
> 2300. Nothing unreadable; after the review's fixes the list, the Chat and the cards again. The
> release build passed; the word check printed nothing.
>
> **Review** (a subagent given the diff and the brief, about 15 minutes). Six points, all fixed:
>
> 1. **Weak text on the full selection at 3.93 to 1** (medium: the model picker's further models
>    are weak, and keep their colour on the selected entry): the weak text lifted from 65 % to
>    72 % of the foreground, and every ink is measured on the selection now.
> 2. **A pressed or focused button** filled in `#34393E` put the red buttons' words at 4.48 to 1
>    and weak text at 3.81, lighter than hover: now `#0A0A0A`, darker than at rest as in egui's
>    Dark, inside its white outline; measured.
> 3. **A Markdown code block outlined twice**: egui frames a read-only text box with the resting
>    stroke, and egui_commonmark adds an outline of its own; the resting stroke is dropped in the
>    Markdown's scope.
> 4. **The progress pill's track** was the cards' own fill, unseen on a card: a track of its own in
>    Dark (`style::track`, code's background), Light's as before; tested.
> 5. **egui's extreme background**, `#0A0A0A`, vanished on the black (the phase bars' tracks, a
>    heatmap's empty days, a sparkline's ground): the wells stand at `#1A1C1F`; tested; the
>    sparkline's doc comment no longer calls it the panel's darker fill.
> 6. **Doc comments and a clone**: `WIDGET` no longer claims the scroll handles, which egui
>    0.37's floating bars paint in the foreground colour; a needless clone gone.
>
> **Session's calls.**
>
> 1. The accents kept with their shades, the brief note's rule: none fell under 4.5 to 1 on the
>    new surfaces; the theme's own accents are recorded above, not taken.
> 2. Buttons and text boxes framed at rest in the border grey, as the theme frames its inputs:
>    on the black a text box's well is otherwise unseen, and buttons share egui's resting stroke.
> 3. The lines between panes in the separators' colour, beyond the brief note's list: the first
>    picture showed the panes running into one another.
> 4. In the phone's sheet, a button's words black on the light accent in Dark, where white read
>    2.4 to 1: found in the sheet's check and cheap.
> 5. The phone page and the README's picture rendered from the neutral copy: over the fixtures
>    the page names a gone fixture's folder by the machine's path.
>
> **Found and logged, not changed.** The chat's Markdown table cuts its last header at the pane's
> edge in the default layout ("Before" read "Befor" at 1600 × 1000): a layout matter of the
> narrow pane, not of colour, for the P6 walk.
>
> No message to another project.
