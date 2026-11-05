---
ep: 18b
title: Usage-share badge
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-18b — Usage-share badge

**Size:** S · **Depends on:** EP-18a · **Blocks:** EP-23

## Context

D-25's usage share: each project's share of the week as a badge of its row, a line of the Detail
pane's Status section, the legend's key and the sort key `SortKey::UsageShare`, which has been in
the sort menu since EP-6, greyed out until its data arrived (`arrives_at`). EP-18a, which split
this brief off, writes the data. Each poll's rise is split among the sessions that worked since
the poll before, and each session's ledger row keeps its part of the weekly and Fable figures in
points of the limit (`SessionRow::usage`, schema 5). `Ledger::shares_since(since)` sums each
project's parts over the sessions started since `since` (a `ProjectShare`: weekly, Fable, the
sessions counted), and `usage::share::week_start(resets_at, now)` gives the weekly window's start.
`crw sessions` prints both. In the window, the weekly reset is that of `meter.gauge.figures(now)
.weekly`. The board's `split_usage` writes the shares and marks the Sessions pane's history stale,
the moment to read the sums again. The endpoint gives whole percents, so the sums run in halves and
thirds of a point (`ledger::points` writes one whole or to a tenth).

The list's badges are columns of their own (`list::Column`) that give way as a row narrows
(`Column::GIVE_WAY`: the badges in full, compact, then compact without the counts, then the
progress alone). Each mark is made from a `badges::Sign` of the legend's tables (`badges::KEY`),
its sentence the hover text and a line of the Detail pane's Status section (EP-7, EP-7a). A files
read replaces the portfolio's rows (`Loader::receive`), and the loader puts its own readings back
afterwards.

**Session start (2026-11-05, 21:31 UTC).** Launched from the window as a visual launch (rule 11):
the ledger row `6fa87918`, visual, EP-18b, open (`crw sessions clauderoadmapwrangler`). The opener
suggested Fable at high effort. The owner chose Opus 5.5 at max in the tab's panel, by their own
judgement, and will keep choosing there until the tool can set a session's model and effort to
their liking, whether sessions run one by one or chained. The tree was clean at `0f5dca9` but for
the owner's tick of OT-15. No owner task and no inbox message is open.

**What the pickup checked** (`crw sessions`, read only). The week since 2026-10-31 18:00 local
has two projects with shares: another project at 1 point (28 → 29 % while it worked) and this one
at 0 so far, over this session. The account stands at 29 % of the weekly limit and 20 % of
Fable's. So the window splits on the real ledger at schema 5, as OT-15 asked.

**Decided at pickup (owner, through `AskUserQuestion`).**

1. **The badge's figure: points of the weekly limit.** It shows the project's weekly sum this
   week, `+6 %`, in the unit of the meter's own "Week 29 %". The project's part of all the points
   attributed this week goes into the hover text and the Detail line. The alternative was that
   part as the badge's figure, which sums to 100 % over the projects but moves whenever any
   project works.
2. **Fable in the hover text and the Detail line only.** The cell stays one figure wide, and the
   sort ranks by the weekly sum. The alternatives were both figures in the cell (`+1 % · F +0 %`)
   and the weekly figure alone, with Fable left to `crw sessions`.
3. **The session's model and effort**: Opus 5.5 at max, chosen in the tab's panel.

## In scope

1. **The week in the window** (`meter::Week`, held by the meter). The sums come from
   `Ledger::shares_since(week_start(resets_at, now))`, read without a write. The reset is the
   meter's weekly figure's. The sums are read when the week is first known or moves on, and again
   after a split wrote shares. A failed read is shown and tried again after a while. At each pass
   the board puts each registered project's sums into its row (`Badges::usage`, a
   `portfolio::Usage`), matching names case aside, because a files read replaces the rows. A
   `Usage` holds the weekly and Fable sums, the sessions counted, the week's start, and the
   points attributed to every project this week.
2. **The badge** (`badges::usage`). It shows the weekly sum in points of the limit (`+6 %`) in
   the text's own colour, or `+0 %` weak when the project's sessions worked through no rise. It
   shows nothing when no session of the project has a share this week, or the meter has no
   weekly reading. The hover sentence gives the points and the sessions, the week's start in
   local time, Fable's sum and the project's part of all the points attributed. No new colour.
3. **The column** (`Column::Usage`, "Usage" in the header row with what it holds as hover text).
   It sits after Backup, in the three forms of `GIVE_WAY` with the badges (full, compact, compact
   without the counts). It is measured and drawn for every project row, a missing folder's
   included, since the ledger's figures do not need the folder.
4. **The legend's key.** A Usage group in `badges::KEY`, between Backup and Row: `+2 %` (points of
   the weekly limit this week) and `+0 %` weak (worked, no rise yet).
5. **The Detail line.** A sixth subsection, Usage, closes the Status section after Backup. It
   holds the mark and its sentence, then the method in a weak line. Without a share, a weak line
   says why: no session with a share since the week's start, no weekly reading yet, or the
   ledger's error.
6. **The sort key.** `SortKey::UsageShare` is enabled: the largest weekly sum first, compared to
   a tenth as `ledger::points` writes it, so that equal figures fall back to the name. Projects
   without a share go last. `arrives_at` goes, since no key waits for data any more.
7. **The screenshot run's week.** With `usage`, the canned meter also carries the week's sums of
   four fake projects (harbor, lantern, quarry, orchard, one of them `+0 %`). The flag's doc says
   so.
8. **Recorded.** D-23 and D-25 addenda, the README's status paragraph and the window's paragraph,
   and "From EP-18b" notes where later briefs need them.

## Out of scope

- The Fable sum in the cell, and a sort by Fable (the owner's choice).
- The unattributed remainder shown anywhere, and a split weighted by busy time (as in EP-18a).
- The usage share in `crw status`. `crw sessions` already prints the week's sums.
- The chart of D-23 and D-26 (P4's analytics), and the five-hour figure.
- The cards view's badges (EP-26).
- Setting a visual tab's model and effort: the extension's link takes only `prompt` and
  `session` (EP-16), so the owner keeps choosing in the panel.

## Verification / acceptance

- **Unit tests**:
  - the badge: every usage mark made from a sign of the legend's Usage group, `+0 %` weak, none
    without a share, the hover's figures and part;
  - the sort: the largest share first, equal figures by name, none last;
  - the week: read from a scratch ledger at schema 5, not read again until a split marks it
    stale or the week moves on, none without a reading or a ledger;
  - the rows' sums: put into the rows by name, case aside;
  - the columns: the existing give-way and naming tests, with the new column in every form with
    the badges.
- **Build checks:** `cargo test`, `cargo test --features screenshot`, `cargo clippy --all-targets
  --features screenshot -- -D warnings`, `cargo clippy --all-targets -- -D warnings` and `cargo
  fmt --check`.
- **Screenshot runs** with `usage`, in both themes: the Projects pane with its Usage column and
  the legend open, the Detail pane's Usage line for a project with a share and one without, and
  `sort=usage-share`.
- **The release.** `cargo build --release` once the owner has quit the window from the tray, then
  `scripts\install-shortcut.ps1`.
- **Rule 3's word check** on the staged diff prints nothing.

> **Completion note (2026-11-05; claude-opus-5-5/max; about 45 min to the commit, with the
> owner's two rounds and one quit of the window).** One session in the VS Code extension from
> 21:31 UTC, launched from the window (ledger row `6fa87918`, visual, EP-18b). It covered the
> reading, the pickup round, this brief, the code and its tests, the screenshot runs, the closing
> rounds and the release.
>
> **Built.**
>
> - `portfolio`: `Usage` on `Badges::usage` (the weekly and Fable sums, the sessions counted, the
>   week's start, the points attributed to every project) and `ProjectRow::usage_tenths`.
>   `SortKey::UsageShare` ranks by it, and `arrives_at` is gone.
> - `meter::Week`: the week's sums, read from the ledger without a write.
>   - It is read when the week is first known or moves on, after a split (`stale`), and 30 s
>     after a failed read (`Meter::read_week`).
>   - `Week::apply` puts the sums into the rows at each pass.
>   - The canned meter carries four fake projects' sums.
> - `badges`: `usage` (the mark and its sentence), the signs `+2 %` and `+0 %` and the legend's
>   Usage group.
> - `list`: `Column::Usage` after Backup, in the three forms with the badges, measured and drawn
>   for every project. The sort menu greys no key out any more.
> - `detail`: the Status section's sixth subsection, Usage.
> - `board`: the week read and put into the rows at each pass; a split marks it stale.
> - The records: D-23 and D-25 addenda, three README sentences, the EP-18b row, "From EP-18b"
>   notes for EP-19, EP-23, EP-24 and EP-26, and OT-16.
>
> **Verification.** `cargo test` 395 passed (94 + 1 + 1 + 2 + 1 + 259 + 2 + 7 + 6 + 11 + 11,
> against EP-18a's 392), 397 with `--features screenshot`; both clippy runs and `cargo fmt
> --check` are clean. Screenshot runs with `usage`, kept in the session's scratchpad:
>
> - light, the legend open, lantern selected: harbor `+9.5 %`, lantern `+6 %`, quarry `+0.5 %`,
>   orchard's weak `+0 %`; lantern's Usage line reads "6 points of the weekly limit and 4.5 of
>   Fable's over 3 sessions this week, since the reset on …: 38 % of the 16 points the window
>   attributed";
> - dark, sorted by usage share, meadow selected: harbor (pinned), lantern, quarry, then meadow
>   without a share, whose line says that no session of the project has taken one.
>
> The same light view at HEAD, taken from a temporary worktree, showed the same compact form
> without the counts at the default 1600 points. So the new column takes about 65 points from the
> Brief title and pushes out no other column at that width. The release was built with the window
> quit from the tray, then `scripts\install-shortcut.ps1` refreshed both shortcuts. `crw sessions
> clauderoadmapwrangler` on the real ledger, read only, gave this session `week +1 % (29 → 30 %) ·
> Fable +0 % (20 %)` and the project `week +1 %` over the week, which OT-16's look should find in
> the column. Rule 3's word check found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-05).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The badge's figure, at pickup | Points of the weekly limit | As built; D-23 and D-25 addenda |
> | 2 | Fable, at pickup | In the hover text and the Detail line only | As built |
> | 3 | The session's model and effort | Opus 5.5 at max, chosen in the tab's panel by the owner's own judgement | The front matter |
> | 4 | The release build | Quit the window, build and install | Built and installed |
> | 5 | The addenda, the README and the roadmap notes | Accept as written | Written |
> | 6 | Where the owner's wish to set each session's model and effort is kept | A brief note for EP-19 | "From EP-18b" in EP-19's notes |
> | 7 | OT-16 | Add it | OT-16, before EP-35 |
> | 8 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. The week lives in the meter, so a screenshot run's canned meter brings its sums. It reaches
>    the rows at each pass because a files read replaces them.
> 2. The week is read when it moves on and after a split, not on a timer, since only the window
>    writes shares. A failed read keeps the shares while the week stays the same.
> 3. The sort compares the figures to a tenth, as the cell writes them, so that equal figures
>    fall back to the name.
> 4. The column is drawn for a row whose folder is missing, since the ledger's figures do not
>    need it.
> 5. A sum under a twentieth of a point is the weak `+0 %`. A project with a share of Fable
>    alone shows no badge and sorts with those without one.
> 6. The legend has no sign for a missing reading: every row would show it at once, so the
>    Detail line says why instead.
>
> **Notes for later sessions.**
>
> - A worktree built with a shared `CARGO_TARGET_DIR` leaves its exe in `target\debug` until the
>   main tree's package builds again, and `cargo run` then starts the worktree's exe: touch a
>   source file first. This session's first dark screenshot was the worktree's, and was retaken.
> - The week's start is shown in local time, an hour off the reset's own hour across a change of
>   daylight time (EP-18a's note).
> - Still unchecked from EP-18a: whether `claude agents --json` lists subagents or the
>   extension's helpers as sessions, whose parts would stay unattributed. OT-16's real figures
>   may show it.
>
> No messages for other projects. Owner task: OT-16, before EP-35. **Next: EP-35** (Resume in VS
> Code), launched from the window.
