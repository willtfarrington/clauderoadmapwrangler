---
ep: 58b
title: A data repository's site card from a file it writes itself outside its folder; the file defined, `crw export site` carrying the card with nothing inside the repository read, the definition handed back
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-58b — A data repository's site card from a file it writes itself outside its folder

**Size:** S · **Depends on:** EP-58 · **Blocks:** EP-59 (the P6 re-plan, which takes it into its
retro) · **Decisions:** D-24, D-3 (and D-11, D-29 as standing)

## Context

IN-19 and IN-20 (2026-11-25, one request in two messages, placed here at the triage of that day
on the owner's choice): the site (the registry's `site`, whose page fetches
`assets/data/projects.json` into cards of its own since its IN-1, the snippet
`projects-cards.html` unused and git-ignored there) should show the data repository's card too.
`export_site` (`crates/cli/src/main.rs`) leaves a `data = true` project out before any git child
(D-24 addendum of EP-27), so its card on the site is hand-typed. The data repository's side is
settled (its D-48): a local post-commit hook runs its own roadmap parser over its committed
`roadmap/README.md` and rewrites a published-card file whenever the figures change, so nothing of
this tool reads inside it. This brief defines that file with the owner, builds the export's reader
and hands the definition back; the data repository then builds the writer (its EP-174), and the
site's IN-3 wires the card once the export carries it.

**Session start (2026-11-25).** A chained headless run that the window started at 18:34 after
EP-58a (ledger row `2a5bfa3c`, EP-58b, headless, claude-opus-5-5/max; the chain log's 18:34 line,
the chain's mark at 80 % of the session window, the owner's choice in its plan); the prompt is the
tool's opener, so no hand start to explain. S's cap of 1 h 15 min of working time from 18:34. Read
first: the roadmap's P6 table and EP-58b's brief note, `DECISIONS.md` (D-24 and its addenda, D-2,
which names the data repository among the owner's public repositories), EP-58a's completion note,
`roadmap/owner-tasks.md` (OT-40 … OT-43 and OT-45 open, none gated before EP-58b) and
`roadmap/inbox.md` (no open message; IN-19 and IN-20 triaged here). The site's own inbox was read
too (read-only): its IN-1's disposition says that its `site.js` fills each card that names its
project (`data-project`) from `projects.json`, and its IN-3 waits for this export. Field check
since EP-58a's record commit (`fda8de1`, 18:33): `crash.log` last written 2026-11-23 04:00,
nothing new; `crw notifications`: EP-58a's finish and this chain's start, nothing odd; `crw
sessions`: this run open, three manual sessions of the day with no end recorded (VS Code tabs
left open); `crw autopilot`: not paused, six projects on, and at 18:34 "the restart into the
installed build waits: a session open with no prompt yet would lose its ledger row": so the
window still runs the build from before EP-58a, which matters only if this brief adds a registry
key (IN-14's lesson). Nothing to triage. Neither the data repository's name nor the site's is on
rule 3's word list (checked by count), so both may be named here.

**Judged to fit S's cap, so not split.** A reader of about a hundred lines and its tests in
`crates/core/src/export.rs`, one branch of `export_site`, the help's line, a D-24 addendum, the
README's site paragraph, a section of `docs/manual-changes.md`, the Handed on block; no window code,
no list or pane, so no screenshot run (rule 5). If time runs short, the snippet's removal (if the
owner drops it) is proposed as a row and the snippet stays.

## In scope

1. **The definition**, settled with the owner at the start through `AskUserQuestion` and recorded
   in a D-24 addendum: the file's place (outside every project folder, never this repository nor
   its `private/`; IN-20 suggests the tool's own local folder), its form (JSON with the fields of
   `export::SiteProject`, name, the stack badge's text, done, total, percent and the next rows' id,
   title and size, at most `NEXT_ROWS`, and an `as_of` day), where the card's public standing
   comes from (never a git child), and what a stale or malformed file does (left out with its
   reason in the preview, never a blank card).
2. **The reader** in `crw_core::export`: the file read and checked (the name the registry's, a
   day written `YYYY-MM-DD` and not after today, counts that add up: a total above nought, done at
   most the total, the percent done of total rounded down, as many next rows as the undone rows
   allow up to `NEXT_ROWS`, each with an id and a size of S, M, L or none); a public standing
   decided as the owner chooses. `crw export site` carries a data repository's card from it, the
   preview saying that it is the repository's own card and of which day, or why it is left out (no
   file, unreadable, malformed, not public); the cards sorted by name as today; nothing inside the
   repository read, its folder not even looked at, no child started there.
3. **The snippet**: still written or dropped, the owner's call (D-24 named it; the site reads the
   JSON alone since its IN-1).
4. **The words**: the help's `export site` line, the README's site paragraph, `docs/manual-changes.md`
   (rule 14).
5. **The hand-back**: the definition in a `Handed on (to mimicwarehouse)` block after the
   completion note, which the mailman delivers once the brief is committed; the first export that
   carries the card is an owner task once the file exists.

## Out of scope

- The writer (the data repository's EP-174) and the site's wiring (its IN-3).
- The window: the site export has no window action (D-24 addendum of EP-27, `crw export site`
  serves); the manual's own text (EP-59's fold).
- Any read of the data repository: its git, its roadmap, its folder.

## Verification / acceptance

- Unit tests in `crw_core::export` on temp folders with synthetic names and synthetic card files:
  a good card read; no file; a file that is no JSON; each check failing (another name, a total of
  nought, done over the total, a wrong percent, too few or too many next rows, a size other than
  S, M, L or none, a day badly written or after today); the public standing's cases.
- `cargo test`, `cargo test --features screenshot`, both clippy runs and `cargo fmt --check`.
- A try under `CRW_HOME` with a scratch registry (synthetic names, one project marked `data =
  true` whose folder does not exist), a cached account list and a synthetic card file: `crw export
  site` previews the card with its day, and `--yes` writes into a scratch site; the file removed or
  broken, the preview says why and the card is left out.
- The word check of rule 3 on the staged diff prints nothing.
- The release build and the install run last, as the opener asks.
- **Residual risk.** No real card file exists yet: the data repository's writer comes with its
  EP-174, so the first real read is the owner's export (an owner task). The reader is exercised by
  the tests and the `CRW_HOME` try on synthetic files written to the definition; the writer is
  built from the Handed on block's words, so a mismatch shows in the preview's reason, never as a
  blank card.

> **Completion note (2026-11-25; claude-opus-5-5/max headless; about 20 minutes of working time:
> 18:34 to the question at 18:38, then from the owner's answers at 19:38 to the commits at about
> 19:53, the hour's wait not counted; the install after them).**
>
> **What was done.** Judged at the brief to fit S's cap, so not split. `crw_core::export` reads a
> data repository's own card: `cards_dir` (the local folder's `cards`, `CRW_HOME\cards` in a
> trial), `card_file`, `listed_public` (the cached account list by the registered name or the
> folder's, with no git child and no remote read), `check_card` (the checks of D-24's new
> addendum, each failure worded), `read_card` (no file, unreadable, over 64 KiB, malformed) and
> `data_card`, which opens the file only once the list names the repository public. `crw export
> site` takes a data repository's card from it, its folder never looked at: the preview prints
> the cards folder, the card's line with "its own card of <day>", or "left out: a data
> repository, <why>"; the cards are sorted by name as before, and the site's `as_of` stays the
> export's day. The snippet is no longer written: `write_site` writes `projects.json` alone, and
> `site_html` and its test are gone (`escape` stays, the phone's pages use it). The help's line,
> the README's site paragraph, D-24's addendum of 2026-11-25 and `docs/manual-changes.md`'s
> section say so. OT-46 is the owner's first export carrying the card, once the file exists.
>
> **Owner decisions** (one `AskUserQuestion` of four questions, asked at 18:38 and answered in the
> window at 19:38; each answer the recommended option):
>
> 1. **The file's place and form**: the local folder, `%LOCALAPPDATA%\clauderoadmapwrangler\cards\<registered name>.json`,
>    JSON with `as_of` and the site card's fields, other fields ignored. *Outcome:* built so.
> 2. **Its public standing**: the account's list the window caches, by the registered name or the
>    folder's; no registry key, nothing in the file. *Outcome:* built so; no key, so IN-14's lesson
>    does not arise and the window's older build reads the registry as before.
> 3. **A stale card**: never left out for its age; the preview names its day. *Outcome:* only a
>    missing, unreadable, too heavy or malformed file leaves the card out.
> 4. **The snippet**: no longer written; the copy in the site's folder stays untouched.
>    *Outcome:* removed from the export.
>
> The commits are the autopilot's standing yes (crw's D-22), on in this project; the push is the
> owner's.
>
> **Session's calls.**
>
> 1. **The checks' details**, within the owner's definition: a byte-order mark allowed (a
>    PowerShell writer's habit); 64 KiB at most, a heavier file left out unread; a day after today refused
>    as malformed; exactly as many next rows as the undone rows allow up to two; an id and a title
>    not blank; the name matched exactly, since the site's page matches cards by it; the percent's
>    arithmetic checked, so a file's counts of any size never overflow (found reading the diff).
> 2. **The order**: the standing first, then the file, so a private or local-only data
>    repository's file is never opened, and the preview says "not public (…)" as it does for the
>    other projects.
> 3. **An end-to-end test** (`crates/cli/tests/export.rs`) in place of a hand try under
>    `CRW_HOME`: the binary over a scratch home with a synthetic registry (a site and a data
>    repository whose folder does not exist), the account's list seeded into the scratch ledger as
>    the window's pass caches it, and a synthetic card file: no list, no file, the card carried
>    with its day, `--yes` writing `projects.json` alone with the card in it, a malformed file and
>    a private listing each saying why.
> 4. **No screenshot run**: no list or pane changed (rule 5). **No review subagent**: an S brief
>    (rule 6 asks one of an M); the diff was read through before the last checks.
> 5. **The checks**: `cargo test` (908 passed, the six new tests among them, over the final code)
>    and `cargo test --features screenshot` (916), both clippy runs and `cargo fmt --check` clean;
>    the release build in the install. A read-only `crw export site` (the preview, which writes
>    nothing) over the owner's registry and the window's cached list, as EP-27 ran it, read with
>    its project lines filtered out: the `cards` line names the local folder; mimicwarehouse passes
>    the public standing and reads "left out: a data repository, no card file (…\cards\mimicwarehouse.json)",
>    so its card is carried once its writer runs; the two other data repositories read "not
>    public (not on the account's list)", their files never opened. The word check of rule 3 on
>    the staged diff printed nothing.
>
> **Not handed on to the site.** Its IN-3 already waits for this export to carry the card, which
> it does once the data repository writes its file (OT-46 then runs the export); the snippet's
> copy there is git-ignored and unread, so no letter goes to it.

> **Handed on (to mimicwarehouse).** For your IN-3 and D-48: the published-card file, defined
> with the owner in clauderoadmapwrangler's EP-58b (2026-11-25, its D-24 addendum of that day),
> for your EP-174 (S) to build the writer. (1) Place:
> `%LOCALAPPDATA%\clauderoadmapwrangler\cards\mimicwarehouse.json`, named by your registered name;
> create the `cards` folder when it is missing; the hook writes nothing else there and edits no
> project's files. (2) Form: one JSON object in UTF-8 (a byte-order mark allowed, at most 64 KiB)
> with `as_of`, the local day it is written as `YYYY-MM-DD`, never a later day; `name`, exactly
> `mimicwarehouse`; `badge`, your stack badge's text, its chips joined with ", " (it may be empty);
> `done` and `total`, the checked rows and all the rows of your committed roadmap's phase tables,
> the total above nought; `percent`, done × 100 / total rounded down; `next`, the first undone rows
> in table order, exactly as many as the undone rows allow up to two, each `{"id": "EP-n",
> "title": the Brief cell without its link markup and its decisions, "size": "S", "M", "L" or
> null}`, the id and the title not blank. Other fields are ignored; a missing one makes the file
> malformed. (3) Write it whole (a temporary file beside it, then a rename) whenever the figures
> change; there is no age limit, since the card carries its own day. (4) Its public standing is
> not the file's: crw carries the card only when the GitHub list its window caches names a public
> repository called mimicwarehouse (by the registered name or the folder's), and otherwise does
> not open the file. (5) To check a file: `crw export site` (a preview, it writes nothing) prints
> mimicwarehouse's line with "its own card of <day>", or why it is left out ("no card file", "not
> public (…)", or "its card file is malformed: …" naming what failed). The first export that
> carries the card is the owner's (crw's OT-46); prime's IN-3 then wires it into the page.
