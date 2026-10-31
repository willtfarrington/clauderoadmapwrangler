---
ep: 3
title: Parser A — phase tables, brief notes and the next EP
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-3 — Parser A: phase tables, brief notes and the next EP

**Size:** M · **Depends on:** EP-1 · **Blocks:** EP-4 (and through it EP-5 to EP-8)

## Context

EP-2 left the registry, discovery and `crw scan`, which knows where each project's
`roadmap/README.md` is but never reads it. This EP is the first half of the parser (D-9): it reads a
roadmap README's phase headings, phase tables and brief notes, finds the next EP (the first unchecked
row in table order, not the lowest number) and counts done, total and stretch rows. EP-4 adds briefs,
completion notes, decision logs, the git snapshot and the proposal facts; EP-6 shows the result.

**Verified at pickup (2026-10-31, 16:10).** A read-only survey of the twelve `roadmap/README.md` files
under the data root (one is empty) found what the parser must cope with, all of it already named in the
planning record §2.6:

- **61 phase tables** behind three headers: `| # | Brief | Size | Depends on | Core | Done |` (50),
  `| # | Brief | Size | Depends on | Done |` (6) and `| # | Packet | Size | Depends on | Status |` (5).
  Two other tables also start with `| # |` (a file list and a coverage table), and others start
  with words (glossaries, routines, document indexes, a phase overview).
- **475 rows**, all single lines, the longest about 870 characters. Done cells: `☐` 302, `☑ \`hash\``
  129, bare `☑ hash` 19, `[x] hash` 18, two hashes joined by ` + ` 5 (backticked or bare), eight
  hashes 1, `[ ]` 1. `◐` and `[~]` are defined in two roadmaps' prose but are not in any cell today.
- **Sizes:** M 300, S 100, L 72, and three re-scopes (`S → L (re-scoped …)`, `S→M (…)`, `S/M`).
  **Core:** 378 core, 15 stretch.
- **Depends on:** comma lists, ranges with `…` (16) and one with an en dash and a bare number
  (`EP-26–34`), parenthesised annotations (`(met)`, `(checkpoint)`, `(= EP-5b)`, `(cleanest after
  EP-4)`), one ordering clause (`— run before EP-21/EP-17`) and prose (`everything`, `all UI EPs`).
- **Brief cells:** a link around the whole title, part of it or none; decision references inside the
  link text, after it as `(D-2, D-14)` or as a range `(D-1 … D-8)`; backticks, quotes, emoji and
  parentheses inside titles.
- **Phase headings:** `## Phase P1 — Name (status)`, `## Phase 1 — …`, `## R1 — …`,
  `### M0 — Name · \`[x] hash\``, `### Checkpoints · recurring` (a packet table under a heading with no
  id), `### M4–M8 · \`[ ]\` …` (no table), two-digit phases with two parenthesised groups, a placeholder
  phase without a table, phases with no status.
- **Brief notes:** `**Brief notes.**` then `- **EP-n.** …` bullets (116); `- **EP-n** (runs after
  …).` (2); the labels `**Brief notes** (…)` over two lines and `**Charters.** …` with text on the
  label's line; prose `**Scope sketch (refine at EP-n).** EP-n: … EP-m: …` paragraphs (4); and three
  roadmaps with no notes because every brief is pre-written.

**Inbox and owner tasks.** Neither `roadmap/inbox.md` nor `roadmap/owner-tasks.md` exists yet: nothing to
triage (rule 1).

**Session note.** Started again from another project's VS Code window with an `@clauderoadmapwrangler`
request, so the transcript files under that project's folder slug (rule 11 and D-16 ask for a window on
this folder alone; EP-0 to EP-2 ran the same way).

## In scope

1. **Shared roadmap types** (`core::roadmap`). `EpId` (`EP-12`, `EP-4a`), `Size` (S, M, L), the
   reference scanners for EP ids and decision numbers with ranges expanded (`EP-6 … EP-12`, `EP-26–34`,
   `D-1 … D-8`), and the README path of a project folder; EP-4 and EP-5 reuse them.
2. **The README parse** (`core::roadmap::table`), read-only, from text:
   - lines with a BOM, CRLF or LF; fenced code blocks and HTML comments skipped; blockquotes are never
     tables, headings or notes;
   - sections by heading; a section is a **phase** when it holds a phase table or its heading carries a
     phase id; a subsection without either belongs to the phase above it;
   - **phase tables found by their header row**, never by the `| #` prefix: columns are located by
     name (`#`, `Brief` or `Packet`, `Size`, `Depends on`, `Core`, `Done` or `Status`), so the three
     surveyed headers and a later added column all read; the **profile** (house, house5, packet)
     follows from the columns; a delimiter row is required; a table that starts like a phase table but
     lacks a required column is reported;
   - **cells:** the id; the Brief cell as written, its title without link markup or a trailing
     group of decision references, its link and the decisions it cites; the Size cell as written and its
     final size (`S → L (…)` is L); Depends on as written and the ids it names (parenthesised
     annotations and `before` clauses are not dependencies); core or stretch; the Done cell as one of
     open (`☐`, `[ ]`), in progress (`[~]`), done but not landed (`◐`), checked with its hashes (`☑`
     backticked or bare, ` + ` joined, `[x]`), or anything else kept as written and reported;
   - **phase headings:** id (`P1`, `1`, `R0`, `M0`, `M4–M8`), name, status (the last parenthesised
     group, or what follows ` · `) and a milestone's own mark (`[x] hash`, `[ ]`);
   - **brief notes:** `- **EP-n.** …` and `- **EP-n** …` bullets with their continuation lines, under
     any label or none, and prose segments `EP-n: …` in a paragraph that opens with a bold label, split
     at the ids of the phase's rows; kept verbatim for the launch prompt (EP-10);
   - **problems** with line numbers (a row without an id, too few cells, an unknown Done cell, an id
     twice, a phase-like table that cannot be read), never a panic;
   - **the next EP** (the first row in document order that is not checked), **counts** (done, total,
     stretch) and lookups by id (row, phase, note).
3. **Fixtures** (`crates/core/fixtures/`): one synthetic roadmap per surveyed variant, with invented
   project names (`alpha`, `beta`, …) and titles, including the long row, a two-digit phase, a phase
   without a table and a glossary-like table that starts with `| #`; integration tests assert each
   one's phases, rows, next EP, counts, notes and problems, with LF and with CRLF.
4. **`crw roadmap <project|folder|file>`**: prints what the parser finds in one roadmap (profile,
   phases with their progress, the next EP with its row and brief notes, counts, problems); reads only.
   It is the console side of the parser until the dashboard (EP-6), as `adopt`, `ignore` and `set` are
   for the registry.

## Out of scope

- Briefs and their front matter, completion notes, decision logs, the packet profile's handoff files,
  the HEAD-plus-status snapshot, "session in progress", and filling `proposal::Facts` so `crw scan`
  proposes from the parse (EP-4, its "From EP-2" note).
- `owner-tasks.md`, `inbox.md`, `Handed on` blocks and the legacy owner-item forms (EP-5).
- The dashboard (EP-6); badges (EP-7).
- Writing any roadmap file: the tool never edits `roadmap/README.md` (D-11).
- Resolving a split set's name (`EP-5` naming `EP-5a` and `EP-5b`) or computing eligibility from the
  dependency graph; the next EP stays the first unchecked row (D-9).

## Verification / acceptance

- The gate passes with no warnings: `cargo test`, `cargo clippy --all-targets --features screenshot`,
  `cargo fmt --check`, `cargo build --release`.
- Unit tests cover every Done form, size form, depends-on form, Brief-cell form, heading form and
  note form found in the survey; fences, HTML comments, blockquotes, BOM and CRLF; tables that are not
  phase tables; each problem; the next EP and the counts.
- Every fixture parses to its expected phases, rows, next EP, counts and notes, with LF and CRLF line
  endings; this repository's own roadmap parses with no problems.
- `crw roadmap` on every roadmap under the real data root, run with `CRW_HOME` on a scratch folder and
  folder paths (nothing written anywhere): the next EP equals the first unchecked row an independent
  `grep` finds, the done and total counts equal `grep`'s, the house-style roadmaps' next EP comes with
  its brief notes, and every reported problem is a real irregularity of that file.
- No private project name in the tracked files (`git grep` for the names kept in `private/`).

> **Completion note (2026-10-31; opus/max; ≈ 50 min to the commit).** Executed in one session from 16:04,
> started from another project's window (see the session note). Owner decisions (`AskUserQuestion`, every
> one the recommended option): **commit**: both commits in the house pattern, pushed to the private
> remote; **`crw roadmap`**: kept as the parser's console side until the dashboard (EP-6); **`◐` and
> `[~]`**: not done, so only `☑` and `[x]` count, the next EP stops at a held or in-progress row and the
> counts leave it out, recorded with the by-name column rule as a D-9 addendum; **hand-offs**: "From
> EP-3" sentences in the brief notes of EP-4 (the snapshot parses HEAD and the working tree each,
> `Counts::all_checked`, `brief.link`, a blockquote block for completion notes), EP-5 (line offsets from
> `roadmap::markdown`, `ep_ids` for gates) and EP-6 (the API the tiles read). Routine choices made by the
> session: a line scanner over Markdown blocks rather than pulldown-cmark, since rows are single lines
> and the parser wants line numbers and byte offsets (pulldown-cmark stays pinned for EP-5); the scanner
> (ATX headings, pipe tables with a delimiter row, list items with indented and lazy continuations,
> paragraphs; fences, HTML comments, blockquotes and thematic breaks dropped; BOM and CRLF) split into
> `roadmap::markdown` once one file passed 1,800 lines; columns located by name, `#`, Brief or Packet,
> Size and Done or Status required, the profile derived from them; problems with their line for a table
> that starts like a phase table but lacks a column, a phase header without a delimiter row, a row
> without an id or with too few cells, an unknown Done cell and a repeated id; extra cells joined back
> into the Brief cell (an unescaped pipe); a phase is a section with a phase table or a phase id (`P1`,
> `1`, `R0`, `M0`, `M4–M8` after an optional `Phase `, then ` — `, ` – `, ` - `, `: ` or nothing), and a
> deeper section with neither belongs to the phase above; a heading's status is its last parenthesised
> group, else what follows ` · `, whose milestone mark reads like a Done cell; a Brief title without link
> markup or a trailing group that only cites decisions, and the first `EP-….md` link target as the
> brief, else the first link; Depends-on ids without parenthesised annotations or `before` clauses
> (`run before EP-21/EP-17` and `(cleanest after EP-4)` are not dependencies), ranges with `…`, `...` or
> `–` expanded up to 500 ids; hashes of 7 to 40 lowercase hex digits; brief notes as `**EP-n.**`,
> `**EP-n:**` or `**EP-n**` bullets under any label (one outside every phase only as a fallback) and
> prose `EP-n:` segments only inside a phase, split at its own row ids; `EpId` (with `EP-4a`), `Size`,
> `ep_ids`, `decisions` and `readme` in `core::roadmap` for EP-4 and EP-5, and `discovery` using
> `readme`; ten fixtures `alpha` … `kappa` with invented names and titles (the long row 868 bytes), each
> read with LF and CRLF; a test that this repository's roadmap reads without a problem and that every open
> brief has notes; the README's status, a `roadmap` line and a sentence in the house-style section.
> Verified: `cargo test` 70 passed (30 new: 19 unit, 11 integration); clippy with `-D warnings` and
> `cargo fmt --check` clean; the release build in 10 s through `install-shortcut.ps1`, which refreshed both
> shortcuts; `crw roadmap` on all twelve `roadmap/README.md` files under the data root, with folder paths
> and `CRW_HOME` on an empty scratch folder that stayed empty: no problem in any file, and the next EP and
> the done and total counts (176 of 476 rows) equal to `grep`'s in all twelve (two roadmaps all checked,
> one empty); the five roadmaps that keep brief notes gave their next EP's notes, the four with pre-written
> briefs none, as the survey said. One call (an unknown name, to see the error) ran without `CRW_HOME` and
> read the owner's registry, which `Store::load` only reads: the registry and its copies kept their 15:57
> times. `git grep` over this EP's files found no private project name. Found on the way: the roadmaps
> moved during the session (one more row and four more ticks between the survey at 16:10 and the check at
> 16:40), which is what EP-4's snapshot is for. No owner tasks; no messages for other projects. Next: EP-4
> (Parser B), in a VS Code window opened on this folder alone (D-16).
> **After the commit (≈ 52 min in all):** `ecca039` pushed at 16:53; CI run 41240973060 on windows-latest
> passed in 1 min 5 s (format, clippy, the same 70 tests), its cache whole since `Cargo.lock` did not
> change; with the tick, `crw roadmap .` reads this repository's next EP as EP-4.
