---
ep: 56
title: The manual's convention, the renderer chosen from what the machine has
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-56 — The manual's convention, the renderer chosen from what the machine has

**Size:** M · **Depends on:** EP-45 · **Blocks:** EP-57 (`crw manual`), EP-57a (the window's
part), EP-58 (this project's own manual)

## Context

EP-45 (the owner's request 10 and their choice) planned a manual for each project that works
well enough to owe one: Markdown in the repository, the PDF built. This brief is the planning
part, with a short interview. It settles (1) the renderer, from what the machine has or can pin
without an installer run as administrator, with fonts, a table of contents, pictures, page
numbers, and builds that are reproducible and offline; (2) the files, `docs/manual.md`
(reference chapters, then a walkthrough chapter for a project whose full use is still wanting)
and `docs/manual-changes.md` (what changed since the last phase's close, folded into the manual
at each re-plan), the PDFs' names and where they live; (3) when each is written; (4) how a
project comes to owe one (the owner's word per project, asked in EP-55's sweep). Its outcome is
a D-12 addendum, the wording in the plugin's house-style reference and the `replan` skill's step
(EP-54 has run, so changed here), a message for the owner's environment repository, whose
user-level house style states the conventions for every project, and EP-57 and EP-58 confirmed
or re-cut from what the renderer needs.

**What the machine has (checked this session).**

- **Typst:** not installed (`typst` not on PATH). Winget offers `Typst.Typst` 0.15.1 (released
  2026-07-17, Typst GmbH, Apache-2.0) as a **portable zip** (the release's
  `typst-x86_64-pc-windows-msvc.zip`, SHA-256 `19ce3551…6736`, "Offline Distribution
  Supported"), which winget installs in the user's profile with a link in its `Links` folder,
  already on PATH (uv lives there): no administrator. What Typst offers by its
  documentation, not yet exercised here: fonts built into the CLI (`--ignore-system-fonts` uses
  them alone), `#outline()` for a table of contents with page numbers, page numbering by one
  rule, `image()` for PNG, JPEG and SVG, a fixed creation date from `SOURCE_DATE_EPOCH` or
  `--creation-timestamp`, and no network unless a document imports a `@preview` package.
- **Typst as a crate inside `crw`:** `typst`, `typst-pdf` and `typst-kit` (with its
  `embedded-fonts` feature) 0.15.1 resolve to 299 packages (a scratch `cargo generate-lockfile`,
  no build), 195 of them versions this repository's lock file (501 packages) lacks: a
  bibliography engine, ICU data, a code highlighter, a WebAssembly runtime for plugins among
  them. The build time and the exe's growth were not measured.
- **Python through uv:** uv 0.13.9 on PATH; the machine already renders and fills PDFs with
  PyMuPDF through `uv run --with` (the owner's shared note on PDF tooling). A Markdown-to-PDF
  package on PyMuPDF exists; what it gives for a printed table of contents and page numbers was
  not checked.
- **In the lock file already:** `pulldown-cmark` 0.13.6 (through egui_commonmark), `ttf-parser`,
  `image` and `png`; no PDF writer. A layout engine of the tool's own (line breaking, pages, a
  contents page with page numbers) is far more than an M.
- **Also present, weighed and not offered:** Edge, at its usual place and off PATH: it updates
  itself (no pin), prints no page numbers into a contents page, and printing through it drives a
  browser, which D-29 keeps out of sessions and tests. Pandoc is not installed; with Typst it
  would be two binaries to pin for what `pulldown-cmark` already parses.

**Session start (2026-11-25).** A chained headless run: the window chained this brief at 07:52
after EP-55 (ledger row `2f19aaf6`, EP-56, claude-opus-5-5/max, the chain's mark 70 % of the
session window); the prompt is the tool's opener. M's cap of 1 h 30 min from 07:52: judged to
fit with its checks and close (one interview; one addendum; the reference, a skill step and
perhaps the rule block's text with the test that quotes it; one message; a trial build of a
synthetic document if the owner has the renderer installed; the interview's waits not counted).
If the time runs short after the interview, the trial and the rule block's bullet move to EP-57's
pickup note. Read first: the roadmap's P6 table and EP-56's brief note, `DECISIONS.md` (D-1,
D-11, D-12 and their addenda), EP-55's completion note, `roadmap/owner-tasks.md` (OT-32 … OT-43
open, none gated) and `roadmap/inbox.md` (no open message). Field check since EP-55's record
commit (`bb8ae3e`, 07:51): `crash.log` last written 2026-11-23 04:00, nothing new; `crw
notifications`: EP-55's finish, this chain's start and the ten inbox badges of EP-55's messages,
nothing odd; `crw autopilot`: not paused, six projects on, the log's last line this chain; `crw
sessions`: this run and the owner's idle manual session of last night. Nothing to triage.

**Decided at pickup (round 1, the owner's answers, each the recommended option).** (1) The
renderer is the **Typst CLI, pinned** (0.15.1 through winget's portable zip), `crw` converting the
Markdown itself with `pulldown-cmark` and keeping one template. (2) This session **installs it
and runs the trial**. (3) The PDFs are **built on demand and never tracked**, into the tool's
local folder, `%LOCALAPPDATA%\clauderoadmapwrangler\manuals\<project>-manual.pdf` and
`<project>-manual-changes.pdf`. (4) The rule block **gains a Manual bullet**.

**Decided after the trial (round 2, each the recommended option).** (5) A project owes a manual
when **`docs/manual.md` exists**: the session that settles it with the owner creates it and an
empty `docs/manual-changes.md`. (6) **EP-57 is split** into EP-57 (`crw manual`) and EP-57a (the
window's part, with EP-54's parked words in `list.rs`); EP-58 stays one M, split at its pickup
if it proves larger. (7) The message to the environment repository is **sent now** by `crw
send`. (8) The drafted texts are **signed off as drafted**: the walkthrough loses each route
once real use has met it and is gone when every route has been used.

## In scope

1. **The interview** (`AskUserQuestion`, rule 7), a few rounds: the renderer; whether this
   session installs it and proves it; where the PDFs live; where the convention is written (the
   rule block or not); how a project comes to owe a manual and when the files are written; the
   message to the environment repository; EP-57 and EP-58 confirmed or re-cut.
2. **The trial**, if the owner has the renderer installed now: a synthetic document in a scratch
   folder outside the repository, with a title, a contents page, headings, a list, a table, a
   code block, a picture and page numbers, built twice with a fixed date and compared byte for
   byte, then looked at.
3. **The convention written:** a D-12 addendum; the plugin's `reference/house-style.md` (the
   files table and a section on the manual); the `replan` skill's step that folds the changes
   into the manual; the rule block's bullet if the owner chooses it (`RULES` in
   `crates/core/src/formalise.rs`, its quoted copy in the reference, which
   `crates/core/tests/plugin.rs` compares byte for byte); a new rule 14 in this project's
   `CLAUDE.md`.
4. **The roadmap:** EP-57's and EP-58's brief notes confirmed or re-cut with pickup notes, the
   table and the ordering rationale following.
5. **The message** to the owner's environment repository, through `crw send` on the owner's
   word (a `Handed on` block in a tracked brief would name its receiver, rule 3).

## Out of scope

- `crw manual`, the Markdown-to-Typst conversion and the window's entries (EP-57); this
  project's own manual (EP-58).
- Which other projects owe a manual: each project's session settles it with the owner (EP-55's
  sweep, OT-34 … OT-43); this brief writes no file into another project.
- Any install without the owner's word; the egui pins (D-15).

## Verification / acceptance

- The convention reads the same in the D-12 addendum, the reference, the `replan` skill, the
  rule block (if changed) and the message.
- If the rule block changed: `cargo test` (the plugin's structural test compares the reference's
  quote with `RULES`), `cargo test --features screenshot`, both clippy runs and `cargo fmt
  --check`; a review subagent given the diff (an M brief, rule 6).
- If the trial ran: two builds of the synthetic document byte-identical, its pages looked at.
- The word check of rule 3 on the staged diff prints nothing; the message names no project and
  no machine detail.
- The release build and the install run last, as the opener asks.
- **Residual risk.** `crw manual` does not exist yet: the rule block and the reference name a
  command EP-57 builds, so a session elsewhere that reads them before EP-57 lands finds no such
  command; it should hold, since EP-57 is the next row and the plugin reaches the owner's
  sessions only after the push (OT-32). The trial ran Typst on a hand-written document of the
  shape the converter will emit, not on the converter: nested lists, a table cell holding code,
  a picture in a subfolder of `docs` and a code line wider than the page are EP-57's to test.
  Offline was not proven by cutting the network: by Typst's documentation the CLI fetches only
  `@preview` packages, which the template does not import. Winget's pin holds Typst at 0.15.1
  against `winget upgrade --all`, and EP-57's exact version check refuses another, so the bytes
  stay those of one Typst; an explicit install of another version past the pin is caught by
  that check alone. The new bullet
  makes every formalised project's row offer Formalise… again (`House::wants` compares the
  block), checked here only by the formalise tests, since no list or pane changed and so no
  screenshot run was owed; it should hold as EP-54's Sizes bullet did. The message's triage is
  the environment repository's own session's.

> **Completion note (2026-11-25; claude-opus-5-5/max; about 35 minutes of working time from the
> start at 07:52 to the install at about 09:17, the owner's two rounds of answers, about 48
> minutes, not counted).** A chained headless run (ledger row `2f19aaf6`), judged at the brief to fit M's
> cap, so not split. Written: the convention, in a D-12 addendum; the rule block's Manual bullet
> (`RULES`, `crates/core/src/formalise.rs`, its test now checking the size rule and then the
> manual last) and its byte-for-byte quote in the plugin's reference, which also gains both
> files in its table and a section "The manual" (the two files' outlines, the walkthrough's rule,
> the build); the `replan` skill's new step 8, "The manual" (the interview and the close now 9
> and 10, its description and step 1 naming the changes file); this project's `CLAUDE.md` rule
> 14; the top-level README's house-style bullet; the roadmap: EP-57 re-cut to `crw manual` with
> pickup notes (the converter's rules, the template, the build through stdin), EP-57a added for
> the window's part with EP-54's parked words, EP-58 after EP-57a, segments for EP-58 and EP-59,
> the P6 counts. Installed: Typst 0.15.1 by winget (portable, hash verified, `typst.exe` linked
> in `%LOCALAPPDATA%\Microsoft\WinGet\Links`), held there by a gating `winget pin add`. Tried, in `%TEMP%\crw-ep56` outside the
> repository: a synthetic six-page document built in about 0.1 s, byte-identical across three
> builds and from stdin, looked at page by page. Sent: the convention to the environment
> repository's inbox by `crw send` at 08:57 (IN-9 there, 913 characters, ASCII only).
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The renderer | **Typst CLI, pinned** (recommended; the alternatives were Typst compiled into `crw` and a Python renderer through uv) | Typst 0.15.1, held by a winget pin and checked exactly; `crw` converts the Markdown itself |
> | 2 | Install it now and try it | **Install now and trial** (recommended; the alternative was an owner task before EP-57) | Installed by winget; the trial passed |
> | 3 | Where the PDFs live | **Built on demand, untracked** (recommended; the alternatives tracked the manual's PDF or both) | `%LOCALAPPDATA%\clauderoadmapwrangler\manuals\<project>-manual.pdf` and `-manual-changes.pdf`; D-11 unchanged |
> | 4 | A Manual bullet in the rule block | **Add it** (recommended; the alternative left the block as it was) | The bullet; formalised rows offer Formalise… again |
> | 5 | What marks a project as owing a manual | **`docs/manual.md` exists** (recommended; the alternatives were a registry key and a `CLAUDE.md` line) | The file's presence; created with an empty changes file |
> | 6 | EP-57 and EP-58 | **Split EP-57 only** (recommended; the alternatives split EP-58 too or kept both) | EP-57 (`crw manual`) and EP-57a (the window's part); EP-58 split at its pickup if it proves larger |
> | 7 | The message to the environment repository | **Send it now** (recommended; the alternative held it for the owner) | Sent by `crw send`, IN-9 there |
> | 8 | The drafted texts and the walkthrough | **As drafted** (recommended; the alternative kept the walkthrough whole) | The walkthrough loses each route real use has met |
>
> A correction: question 6 said P6 would go from 17 rows to 18; since EP-51a the table held 18,
> so the split makes 19 (6 S, 13 M). The choice was the split, not the count, so it stands. The
> commits are the autopilot's standing yes (crw's D-22); the push is the owner's.
>
> **Session's calls.**
>
> 1. The trial's template choices, which EP-57 inherits and may revise: US Letter (the owner's
>    locale), Libertinus Serif 11 pt for the text and DejaVu Sans Mono for code (both built into
>    Typst), numbered chapters with a page break before each, `n / N` page numbers.
> 2. The PDFs carry the project's name (`<project>-manual.pdf`), so a viewer's tab says which.
> 3. A changes section is headed `## EP-n — <title> (date)`; the changes file's header carries a
>    `Since` line naming the last re-plan.
> 4. The generated Typst goes to the CLI's stdin with `--root` at the project's `docs`: checked,
>    a main file outside `--root` is refused, so nothing need be written into the project.
> 5. EP-54's parked block on the window's old size words moved from EP-59's note to EP-57a's, as
>    question 6's recommended option named; EP-59's note says so.
> 6. No owner task: Typst is installed, and OT-32 (the push, then the plugin's update) still
>    open, carries this brief's reference and skill changes to the owner's sessions as well.
> 7. No `Handed on` block: the message went by `crw send` on the owner's word, since a block in a
>    tracked brief would name its receiver (rule 3).
> 8. "Pinned" is exact: the winget pin keeps `winget upgrade --all` from moving Typst, and
>    EP-57's check refuses another version, since a newer Typst may read the template otherwise
>    and changes the bytes; the pin moves only in a brief that needs a newer Typst.
> 9. The checks: `cargo test` (856 passed) and `cargo test --features screenshot` (864), both
>    clippy runs and `cargo fmt --check` clean; a review subagent given the diff found twelve
>    points, all fixed: who creates `docs/manual.md` made one answer everywhere (a stub by the
>    session that settles it, the first manual by a brief of its own, EP-58 the exception since
>    this project's word came at EP-45); "pinned" made exact, as call 8 says, where EP-57's
>    note had read 0.15.1 as a minimum; the re-plan builds the manual's PDF, where the draft
>    built "the PDFs" after emptying the changes; `pulldown-cmark` taken with
>    `default-features = false`, without which it adds two packages; the sweep checklist's
>    manual step; "a user" in rule 14 as everywhere else; a missing verb in the roadmap, the
>    changes file's `Since` line for a new file, the brief's rule number, a stale test comment,
>    the manual's fold in the reference's and the README's re-plan summaries, a walk item said
>    twice, and an installed tool's name dropped from the brief. Then `cargo test -p crw-core`
>    and fmt again, the word check of rule 3 on the staged diff and on the message, no
>    screenshot run (no list or pane changed), the release build and the install.
