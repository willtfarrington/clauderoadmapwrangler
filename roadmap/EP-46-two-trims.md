---
ep: 46
title: Two trims — the Projects pane without its Brief column, the Usage pane without "The last seven days"
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-46 — Two trims

**Size:** S · **Depends on:** EP-45 · **Decisions:** D-25, D-23 (and D-3, D-10, D-29 as standing)

## Context

The owner's requests 1 and 4 at EP-45: the Detail pane shows the next brief already, so the
Projects pane's Brief column goes; the Usage pane's spark line of the last seven days goes, usage
over time being v2's (v2 §2).

**What the code says** (read at pickup).

- `crates/app/src/list.rs`: `Column::Title` sits in `Column::ALL` and `Column::LEFT`, its header
  "Brief" in `Column::header`. `Plan::new` picks the first of `Column::GIVE_WAY`'s forms whose
  least leaves the title its least (`title_least`: `TITLE_AT_LEAST` in the full form,
  `TITLE_FLOOR` in the compact ones, EP-38), then the gaps after the name take `SPREAD_SHARE` of
  the room beyond `TITLE_AT_LEAST`, each at most `SPREAD`, and the title the rest (at least
  `TITLE_MIN`), so the row fills to its right edge. `project_row` draws the title from
  `next.brief.title` through `markup::job`; a row with no next brief draws `no_next`'s words
  across the Next and Title cells, and a registered project whose folder is gone draws
  `missing_folder`'s words in the Title cell. `next_cell` draws the id and the size, the size's
  hover saying what it means. The cards view (`project_card`, `TITLE_ROWS`) draws the title on
  two lines. The legend (`badges::legend`, `KEY`) keys the marks and names no title.
- `crates/app/src/meter.rs`: `Meter::tile` ends with the heading "The last seven days" and
  `sparkline()`, which draws `draw_run` and reads `Meter::samples` through `Pick`, `GAP` and
  `DAYS`. `Meter::samples` is read by nothing else: `restore` fills it from the ledger at start
  (`samples_since`), `receive` and `status_line` push to it, `receive` trims it, and the canned
  meter fills it with a week of hourly samples. The ledger keeps every sample (`keep`,
  `status_line`'s `add_sample`), whatever the pane draws.
- The README's two pictures (`docs/dashboard-light.png`, `-dark.png`) show the list with the
  Brief column, taken at EP-36 from the neutral root (`C:\Users\Public\Projects`, a copy of the
  fixtures) with `light,select=harbor,size=2300x1000` and `dark,select=meadow,expand=meadow,legend`;
  the README's usage paragraph names "a sparkline of the last seven days".

**Session start (2026-11-23, 20:37 local time).** A headless run started from the window: the
rolling log's "Started clauderoadmapwrangler EP-46 headless with claude-opus-5-5 at max effort" at
20:37:10, the prompt the tool's opener, and `crw sessions clauderoadmapwrangler` lists the row
(`46493945`, headless, EP-46, open). The autopilot is on for this project, so the run has S's cap
of 1 h 15 min. OT-30 is open, gated before EP-54, not this brief's; no inbox message is open. The
field check read, since EP-43a's record commit (2026-11-23 17:46): `crash.log`, unchanged since
the panic of 04:00 that EP-43a read; the rolling log, the restart into EP-43a's build at 17:45:36,
another project's visual session waiting, this start, and the project read again; `crw
notifications`, another project's visual sessions waiting and finishing, and this project's
EP-43a turn ending at 17:38; `crw autopilot`, not paused, the latest chain lines EP-43's of
yesterday. Nothing odd.

**Against the cap.** Two removals in two files of a known shape, with their tests, the screenshot
runs and one picture pair; the row fits S's cap with its checks and its close, so it is not split.

## In scope

1. **The Brief column goes.** `Column::Title` leaves the enum, `ALL`, `LEFT` and the header.
   `Plan::new` takes the first of `GIVE_WAY`'s forms that fits the row, D-25's give-way order
   kept, and shares what is left evenly among the gaps after the name, as EP-7a's gaps widen, so
   the row still fills to its right edge; `TITLE_AT_LEAST`, `TITLE_FLOOR`, `TITLE_MIN`,
   `title_least`, `SPREAD_SHARE` and `SPREAD` go. The next EP's id and size stay, and the id's
   hover carries the one-liner, its markup drawn as the title's was. A row with no next brief says
   `no_next`'s words in the Next cell, which is measured to hold them (a column as wide as the
   widest value the portfolio holds, EP-7a); a registered project whose folder is gone says
   `missing_folder`'s words across the cells it leaves empty, from the Next cell to the Usage
   column (the Session column when Usage has given way). The Next header's hover names the id's
   hover. The module's doc follows; the Plan tests are rewritten for the new rule.
2. **The cards keep their title.** A card has no Detail pane beside it by necessity, so
   `project_card` and `TITLE_ROWS` stay; the owner may strike them at the close.
3. **"The last seven days" goes.** The heading and `sparkline()` leave `Meter::tile`, with
   `draw_run`, `Pick`, `GAP`, `DAYS` and `Meter::samples` (read by the spark line alone): its
   start-up read, its pushes and its trim, and the canned meter's week of samples. The limits
   (`limits()`, `limit_block`), "This week by surface" (`surfaces()`) and extra usage stay. The
   samples stay in the ledger; the status-line tests check the ledger for the sample kept.
4. **The records.** The README's usage paragraph and its two pictures, retaken from the neutral
   root with EP-36's setups (the legend keys no title, so it is unchanged); a D-25 addendum and a
   D-23 addendum.

## Out of scope

- The cards' title (item 2), unless the owner strikes it.
- The phone's list and pages, which draw their own rows (EP-30).
- The samples in the ledger and any view of usage over time (v2 §2).
- Any write into another project.

## Verification / acceptance

- Unit tests: a plan of every form fills the row to its right edge with the gaps widened evenly,
  and none is narrower than its least; the give-way order (the full form while it fits, then the
  compact badges, then without the counts, then the badges, then the progress); a row draws no
  title, and the id's hover carries it; a row with no next brief shows its words in the Next cell
  whole; the meter's status-line tests count the ledger's samples.
- `cargo test`, `cargo test --features screenshot`, both clippy runs with `-D warnings`, `cargo
  fmt --check`.
- Screenshot runs at 1600 × 1000, 2300 × 1000 and 1100 points wide over the fixtures, the Usage
  pane on its tab, and one over a copy of the real portfolio (rule 5), kept in the scratchpad.
- The release build, and the install last.
- No tracked file names a managed project, the owner's login or the machine (rule 3); the word
  check prints nothing on the staged diff.

**Residual risk.** No screenshot run hovers, so the id's hover with the one-liner is checked by a
unit test's passes (the text drawn once the pointer has rested) and not seen in the real window;
a long one-liner wraps at egui's tooltip width, and the short words' hover is not tested at all.
The copy of the real portfolio has no ledger, so its run drew no usage shares; with them the
Usage column widens a little, as the fixtures' usage picture shows (the compact form at 1600
there), so the real window at the default layout may keep the compact badges with the counts,
as the copy showed, or drop the counts; the plan follows the widths either way and clips
nothing. The short words for a project with no next brief ("all checked", "no roadmap", "no
table") are new in the row, while the phone's list and the cards keep the long ones, so two
surfaces word one state two ways. A registered project whose folder is gone shows only in the
fixtures. The meter no longer reads the week's samples at its start, so nothing in the window
reads them until v2's view of usage over time; the ledger keeps writing them. The next re-plan's
walk reads how the trimmed list serves in daily use.

> **Completion note (2026-11-23; claude-opus-5-5/max, a headless run; 32 minutes from the start
> to the work commit, the owner's answer included).** Started headless from the window at 20:37
> (ledger row `46493945`) under S's cap of 1 h 15 min; the row was judged to fit at the brief and
> was not split. The work commit `8fd04f0` at 21:08, the record commit, then the install.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The cards' title, which the brief note left to the owner at the close | Keep it on cards (Recommended) | `project_card` and `TITLE_ROWS` unchanged; only the list loses the one-liner |
>
> The commits are the autopilot's standing yes (D-22); the push is the owner's, and with it the
> README's two retaken pictures, which the brief note asked for, go public.
>
> **Built.**
>
> 1. **The Brief column out** (`crates/app/src/list.rs`): `Column::Title` gone from the enum,
>    `ALL`, `LEFT` and the header; `Plan::new` takes the first of `GIVE_WAY`'s forms that fits,
>    the gaps after the name share the spare width up to `SPREAD` and the rest goes before the
>    session column; `TITLE_AT_LEAST`, `TITLE_FLOOR`, `TITLE_MIN`, the const assertion,
>    `title_least` and `SPREAD_SHARE` gone. `next_cell`'s id carries the one-liner on hover
>    through `markup::job`; the Next header's hover says so. `why_no_next` gives the Next cell
>    short words, measured, with `no_next`'s long words on hover; `Plan::reach` gives a gone
>    folder's words the cells from Next to Usage (Session's when Usage has given way), its
>    progress and counts no longer drawn.
> 2. **"The last seven days" out** (`crates/app/src/meter.rs`): the heading, `sparkline`,
>    `draw_run`, `Pick`, `GAP`, `DAYS` and `Meter::samples` with its start-up read
>    (`samples_since`), its pushes and its trim; the canned meter's week of hourly samples. The
>    status-line tests count the ledger's samples.
> 3. **The records**: D-25 and D-23 addenda; the README's usage paragraph, the light picture's alt
>    text and both pictures; the screenshot feature's note on `usage`.
>
> **Verification.** `cargo test`: 266 in `app`, 478 in `core`, the CLI's and the integration
> suites, all passing; `cargo test --features screenshot`: 274 in `app`; both clippy runs with
> `-D warnings` and `cargo fmt --check` clean (after `cargo fmt` wrapped two test lines), run
> again on the final tree. An S brief, so no review subagent: the session read its own diff while
> the checks ran and rewrapped one doc comment. Screenshot runs, kept in the git-ignored
> `target\ep46-shots`: over the fixtures at 1600 × 1000 (the full form, Stack and Age shown, the
> Next column as wide as an id), at 2300 × 1000 (the gaps at their cap, the rest before Session,
> the switches at the right edge) and at 1100 × 900 in Dark (the narrowest form, Done alone,
> nothing clipped), and the Usage pane (`usage,tile=usage`: the limits, "This week by surface"
> and extra usage, no spark line); over a copy of the real portfolio's twelve non-data projects
> made with `scripts\copy-roadmap.ps1` one project at a time (rule 5): at 1600 the compact badges
> with the counts, which EP-38's layout had dropped at that width, the short words on the
> projects with no next brief, nothing clipped. The README's pictures from the neutral root
> `C:\Users\Public\Projects` with EP-32's setups (`light,select=harbor,size=2300x1000,usage,
> analytics,chat` and `dark,select=meadow,expand=meadow,legend,analytics=all,decisions`), a fresh
> home each, the copy removed after. The release build passed; the word check printed nothing.
>
> **Session's calls.**
>
> 1. The spare width up to EP-7a's cap per gap, the rest before Session, so that the switches keep
>    the row's right edge; not spread without a cap (a wide pane would scatter the columns), nor
>    left at the row's end (the switches would float mid-row).
> 2. Short words in the Next cell: the first picture at 1600, with the long words measured, had a
>    Next column of about 170 points that cost the full form; with the short words it is about as
>    wide as an EP's id and size. The phone's list and the cards keep the long words.
> 3. A gone folder's words across the cells it leaves empty, its progress and counts not drawn: a
>    gone folder has no reading.
> 4. `Meter::samples` removed with the spark line, which alone read it (a public field, so clippy
>    could not call it dead); the samples stay in the ledger.
>
> **Found and logged, not changed.** `crash::tests::the_hook_writes_a_panicking_threads_report`
> failed once more beside this brief's own failing test at its first run, the shared panic hook
> taking the other test's panic (EP-43a's finding); its fix waits for a brief that touches
> `crash.rs`.
>
> No owner task and no message to another project.
