---
ep: 57
title: crw manual, the Markdown turned into Typst by the tool itself and built by the pinned Typst CLI
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-57 — `crw manual`: the Markdown turned into Typst by the tool itself, one template, the pinned Typst CLI building the PDFs into the local folder, the doctor's Typst line

**Size:** M · **Depends on:** EP-56 · **Blocks:** EP-57a (the window's part), EP-58 (this
project's own manual)

## Context

EP-56 settled the manual's convention (D-12 addendum of 2026-11-25, the plugin reference's
section "The manual"): a project that owes a manual keeps `docs/manual.md` and
`docs/manual-changes.md`; the PDFs are built on demand, never tracked, into
`%LOCALAPPDATA%\clauderoadmapwrangler\manuals\<project>-manual.pdf` and
`<project>-manual-changes.pdf`; the renderer is the Typst CLI pinned at 0.15.1 (installed by
EP-56 through winget, held by `winget pin add`), and `crw` turns the Markdown into Typst itself
with `pulldown-cmark`, every run of text a Typst string literal, inside one template. EP-56's trial
(`%TEMP%\crw-ep56\trial\trial.typ`, a hand-written document of the converter's shape) built
byte-identical three times and from stdin under `--root`. This brief builds the command, the
converter, the template, the build and the doctor's line; the window's entries are EP-57a's.

**Session start (2026-11-25).** A chained headless run: the window chained this brief at 09:15
after EP-56 (ledger row `9579a5be`, EP-57, headless, claude-opus-5-5/max; the chain's mark 70 % of
the session window); the prompt is the tool's opener, so no hand start to explain. M's cap of
1 h 30 min of working time from 09:15. **Judged to fit, so not split:** the converter is one
event loop over `pulldown-cmark`'s events with its text tests; the build is one function behind a
runner trait as `gh` and `tailscale` go; `--changes` is the same build over the other file, and
the doctor's line is one more tool in the existing Tools group. If time runs short after the
converter and the build, the doctor's line moves to EP-57a's notes. Read first: the roadmap's P6
table and EP-57's brief note, `DECISIONS.md` (D-12 and its addendum of EP-56, D-15, D-29),
EP-56's completion note, `roadmap/owner-tasks.md` (OT-32 … OT-43 open, none gated) and
`roadmap/inbox.md` (no open message). Field check since EP-56's record commit (`f34b90d`, 09:14):
`crash.log` last written 2026-11-23 04:00, nothing new; `crw notifications`: EP-56's finish and
this chain's start, nothing odd; `crw sessions`: this run and the owner's idle manual session of
last night; `crw autopilot`: not paused, six projects on, the chain log's last lines EP-56's held
restart and this chain. Nothing to triage.

## In scope

1. **The converter** (`crw_core::manual`, `pulldown-cmark` 0.13.6 as a direct dependency of
   `core`, `default-features = false`, tables and strikethrough on, smart punctuation off): the
   manual's first `#` heading is the title page's title and the paragraphs before the first `##`
   its text (where the convention puts the phase it describes); `##` a numbered level 1, `###` a
   level 2 and so on; paragraphs, emphasis, strong, strikethrough, inline code, links (a link
   with a scheme as `#link`, a relative one as its text alone), soft breaks as a space literal and
   hard breaks as `#linebreak()`, lists and numbered lists nested (a start number kept), pipe
   tables with their alignment, a picture as a figure with its alt text as the caption (a picture
   alone in its paragraph at block level; a URL or a path outside `docs` left out with a
   warning), block quotes, code blocks with their language, a rule; raw HTML left out with a
   warning naming its line (an HTML comment silently); a second `#` heading read as a level 1
   with a warning; no `#` heading at all a warning and a title from the project's name. Every run
   of text a string literal with only `\`, `"` and control characters escaped, followed only by
   `#`, `]` or a line's end. Straight quotes stay straight (the brief's call; smart punctuation
   would also turn a flag's `--` written outside code into a dash).
2. **The template:** EP-56's trial as the note gives it (US Letter, margins 2.2 × 2.4 cm,
   Libertinus Serif 11 pt justified, DejaVu Sans Mono 9 pt for code with a light fill under a
   block, numbered headings and a weak page break before each chapter, a title page with no
   header or footer, `#outline(depth: 2)`, the title and the date in the header, `n / N` in the
   footer, `set document(title: …, author: …)`), a table's first row bold by a show rule.
3. **The build** behind a `Runner` trait (the real one runs `typst` and `git`, a test's is a
   fake): `docs/manual.md` (or `docs/manual-changes.md` with `--changes`) read, a project without
   it refused as owing none (D-12); fresh when the PDF is newer than the source and every picture
   it names (a missing picture refused with its path), `--force` building anyway; Typst located
   on PATH or in winget's `Links` folder, its version exactly 0.15.1 (pinned in `core`), else a
   refusal naming `winget install --id Typst.Typst --version 0.15.1`; the creation timestamp from
   `git log -1 --format=%ct -- docs/<file>`, the file's own time when git has none, and the title
   page's date from it; the generated Typst piped to `typst compile --root <project>\docs
   --ignore-system-fonts --creation-timestamp <t> - <temp>.pdf`, the temp file renamed over the
   PDF, removed on a failure; a child that pipes stdin added to `process`.
4. **`crw manual <project|folder> [--changes] [--force]`:** what it wrote (the PDF's path, its
   size, the source and the date) or why it did not, the converter's warnings, the usage text;
   `CRW_HOME` puts the manuals folder in the scratch home as it does every other.
5. **The doctor's Typst line** in the Tools group: ok at the pinned version, a warning at another
   (`crw manual` refuses it), a note when missing, each naming the winget line where it helps.
6. **The words:** the README's paragraph on the manual and its Diagnostics paragraph; the CLI's
   description; no `docs/manual-changes.md` section, since this project's manual starts at EP-58
   (rule 14 holds once `docs/manual.md` exists).

## Out of scope

- The window's Manual entries, the Detail pane's links, the manual's date in the Status section
  and EP-54's parked words (EP-57a); this project's own manual (EP-58).
- Opening the PDF with a viewer (EP-57a's first open-a-file action).
- Footnotes, math, task lists, definition lists and heading attributes (left as the text they
  are); syntax highlighting beyond Typst's own for a code block's language.
- A newer Typst or a change to the winget pin (D-12 addendum: only a brief that needs it).

## Verification / acceptance

- Converter tests as text over synthetic Markdown: each construct above, the characters Typst's
  markup reads as syntax (`# $ * _ @ < > \ `` ` `` ~ = - + / [ ]` and quotes) inside literals, the
  literal-follower rule checked over every test's output, warnings with line numbers.
- Build tests on temp folders through a fake runner: built when the PDF is missing, fresh when it
  is newer, built again after the source or a picture changes and with `--force`; the refusals
  (no manual, a missing picture, Typst missing or at another version); a failed compile leaves no
  PDF and no temp file; the arguments and the piped input as the note gives them, the timestamp
  from git or the file. The stdin child tested with a system tool, never `typst` or `claude`
  (rule 5). The doctor's three Typst words.
- A trial outside the repository under `CRW_HOME`: a synthetic project's manual with a picture of
  the fake portfolio built by the real `crw manual` and the real Typst, built twice and compared
  byte for byte, looked at page by page; then `--changes`; then fresh.
- `cargo test`, `cargo test --features screenshot`, both clippy runs and `cargo fmt --check`; a
  review subagent given the diff (an M brief, rule 6); no screenshot run, since no list or pane
  changes (the Diagnostics pane lists the report's lines as they come).
- The word check of rule 3 on the staged diff prints nothing.
- The release build and the install run last, as the opener asks.
- **Residual risk.** The real Typst built one synthetic manual and one changes file of the
  convention's shape (the trial), not a real project's: EP-58 writes this project's, the first.
  The constructs the trial did not hold (a picture inside a table cell, a manual of hundreds of
  pages, a picture in a format Typst does not read) rest on the converter's text tests and on
  Typst reading valid calls; what Typst refuses fails the build in its own words and leaves the
  last good PDF, and what it leaves out it prints as a warning, which `crw manual` now shows, so a
  problem shows rather than misprints. The window changed only by the Diagnostics pane's Tools
  group, which gains the Typst line through the report it shares with `crw doctor` (one more
  line, no layout), so no screenshot run was owed; the window builds and opens nothing yet
  (EP-57a). Freshness reads file times: a picture replaced by a copy that keeps an older time is
  caught only by `--force`, and each install of a new build makes the next build of each PDF run
  once (the exe's own time), about a second. An edit not yet committed is dated by the file's own
  time, so a PDF built before its commit (as a re-plan builds) carries the edit's day until the
  file or the tool changes. Typst stays at 0.15.1 through winget's pin and the exact check;
  nothing else on the machine was changed.

> **Completion note (2026-11-25; claude-opus-5-5/max; about 55 minutes of working time from the
> start at 09:15 to the install at about 10:10, no wait on the owner).** A chained headless run
> (ledger row `9579a5be`), judged at the brief to fit M's cap, so not split. Built:
> `crw_core::manual` (the converter over `pulldown-cmark` 0.13.6, now a direct dependency of
> `core` without its default features, so the lock file gains only that edge; the template; the
> build behind a `Runner` trait whose real side runs `typst` and `git`), a child given its input on
> its standard input in `process`, the doctor's Typst line, `crw manual <project|folder>
> [--changes] [--force]` with its usage text, the README's words on the manual and on the
> Diagnostics checks, and pickup notes for EP-57a and EP-58. Tried in `%TEMP%\crw-ep57`, outside
> the repository, under a scratch `CRW_HOME`: a synthetic project's manual (a title page, the
> contents, chapters, nested lists, an aligned table with code in a cell, pictures of the fake
> portfolio in a subfolder of `docs`, a picture amid text, a code line wider than the page, a
> quote, a rule, raw HTML; after the review also a linked picture, two pictures in one paragraph,
> a `%20` path with a GitHub fragment, headings inside a quote and a list item, a table and code
> inside items) built by the real `crw manual` and Typst 0.15.1 in under a second, byte-identical
> across builds, every page looked at; the changes file built; a second call left the fresh PDF
> alone; in a scratch git repository the commit dated the PDF, and an edit not yet committed the
> file's own time.
>
> **Owner decisions.** None arose: the choices below are technical and reversible, logged as the
> session's calls. The commits are the autopilot's standing yes (crw's D-22); the push is the
> owner's.
>
> **Session's calls.**
>
> 1. The title page holds the first `#` heading and the paragraphs before the first `##`, where
>    the convention's outline names the phase, then the source and the date; no field of its own
>    for the phase.
> 2. Straight quotes stay straight: no smart punctuation, which would also turn a flag's `--`
>    written outside code into a dash; Typst's smart quotes act on markup alone, and every run of
>    text is a string literal.
> 3. A link with a scheme is a link; a relative one (a file of the repository, an anchor) keeps
>    its text alone, which a PDF can show.
> 4. Pictures alone in their paragraph (under a link or an emphasis too, a line break between them
>    aside) are figures, the alt text their caption; amid text a picture is a box one line high; a
>    URL, an absolute path or a path leaving `docs` is left out with a warning, its alt text kept;
>    a path is read as GitHub reads it (`%20` decoded, a query or a fragment dropped).
> 5. A heading inside a quote or a list is unnumbered, kept out of the contents and starts no page;
>    a `#` that is not the title says why (a second one, one after the first chapter, one inside a
>    block).
> 6. The changes PDF runs its sections on without page breaks (the first trial gave each brief's
>    section a page of its own), and a wrapped title on the title page is not justified (the first
>    trial spread its words).
> 7. Freshness also compares the exe's own time, beyond the note's source and pictures, so a new
>    build of the tool, which may write the template otherwise, builds each PDF again once; a fresh
>    PDF asks no child at all.
> 8. The date is the source's last commit while the file is as committed, else the file's own
>    time (an edit not yet committed, as a re-plan builds before it commits, or a file git does not
>    track); the title page and the header carry it as a local date.
> 9. Typst's environment loses the variables that would change what it reads (`TYPST_FONT_PATHS`,
>    `TYPST_ROOT`, `TYPST_IGNORE_EMBEDDED_FONTS`, the package paths) and `SOURCE_DATE_EPOCH`; its own
>    warnings on a good build follow the converter's.
> 10. The PDF's name is the registered project's name when the folder is a registered project's,
>     else the folder's, so EP-57a's window can key on the registry's name.
> 11. The doctor's line: ok at the pin, a warning at another version, a note when missing, since
>     the tool works without manuals.
> 12. `pulldown-cmark`'s workspace entry carries `default-features = false` itself: a member cannot
>     turn a workspace dependency's defaults off.
> 13. No `docs/manual-changes.md` section, since this project's manual starts at EP-58 (rule 14);
>     no screenshot run, since no list or pane changed; no owner task, since Typst is installed and
>     pinned since EP-56; no `Handed on` block.
> 14. The checks: `cargo test` (887 passed) and `cargo test --features screenshot` (895), both
>     clippy runs and `cargo fmt --check` clean. A review subagent given the diff and the row's
>     note found nine points, all fixed, and two gaps in the tests, both filled: (1) a picture
>     under a link or an emphasis became a figure inside a paragraph, which Typst leaves out, so
>     pictures are judged by their paragraph and Typst's warnings are shown; (2) an edit not yet
>     committed was dated by the commit before it; (3) a heading inside a quote or a list put a
>     page break inside a container, which Typst refuses; (4) two pictures alone in a paragraph
>     shrank to icons; (5) picture paths were not read as GitHub reads them; (6) an HTML block that
>     opens with a comment dropped what followed on its line unsaid; (7) the stray title's warning
>     misworded two of its cases; (8) the PDF took the folder's name over a registered name; (9) a
>     picture's path kept its `/` inside one name under a verbatim folder; the stdin test's failure
>     message could panic, and a child that reads none of its input was untested (now `cmd /c exit
>     3` with 4 MB of input). Then the second trial, the word check of rule 3 on the staged diff,
>     the release build and the install.
