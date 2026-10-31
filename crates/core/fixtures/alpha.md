# alpha — Roadmap

Master roadmap for alpha, a synthetic project the parser's tests read. Planned 2026-10-01 (v1):
eleven session briefs in four phases, in the house style this repository uses.

## How to use this roadmap

Hand one EP to one session. The next EP is the first unchecked row in table order. A phase table
inside a code block is an example, not a phase:

```text
| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-99 | An example row | S | — | core | ☐ |
```

## Phase P0 — Planning (done 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-0 | [The planning interview: 10 rounds, 40 questions; D-1 … D-12; phases P1 … P3](EP-0-planning-interview.md) | L | — | core | ☑ `a1b2c3d` |

## Phase P1 — Foundations and the read-only view (planned 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | [Toolchain bootstrap: workspace (`core`, `app`, `cli`), pins, gate, CI](EP-1-toolchain-bootstrap.md) (D-2, D-5) | M | EP-0 | core | ☑ `b2c3d4e` |
| EP-2 | [Widget store: `widgets.toml` with backups, scan, adopt and ignore](EP-2-widget-store.md) (D-6 … D-8) | M | EP-1 | core | ☑ `c3d4e5f` |
| EP-3 | Gizmo parser: tables, rows, sizes, `☑` cells of every form; next EP in table order (D-9) | M | EP-1 | core | ☐ |
| EP-4 | Sprocket view: list, detail, filters, keys (D-10) | L | EP-2, EP-3 | core | ☐ |
| EP-5 | Re-plan P1: retro and timings, addenda, P2 confirmed with pickup notes | S | EP-1, EP-2, EP-3, EP-4 | core | ☐ |

Ordering rationale: the parser is the risk and comes before any view.

Standing decisions for phase P1: D-3 (synthetic names), D-9 (the next EP is the first unchecked row).

**Brief notes.**

- **EP-1.** Workspace with `crates/core`, `crates/app`, `crates/cli`; the gate green on an empty
  skeleton.
- **EP-2.** `core::widgets`: `widgets.toml` with a dated backup before any change.
- **EP-3.** `core::gizmo`: find the tables by their header row; rows are single lines; cells:
  id, title, size, depends-on, core, done.
  - a nested list item stays with its bullet
  - so does this one
- **EP-4.** The view: one row per gizmo; `S` starts, `A` toggles.
- **EP-5.** Retro, addenda, the P2 rows confirmed with pickup notes.

## Phase P2 — Sessions (planned 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-6 | Runner: spawn, stop, ledger rows (D-14) | L | EP-2 | core | ☐ |
| EP-7 | Prompt and launch dialog (D-16, D-17) | M | EP-5, EP-6 | core | ☐ |
| EP-8 | Re-plan P2 | S | EP-6, EP-7 | core | ☐ |

**Brief notes.**

- **EP-6.** `core::runner`: spawn the child in a job object; a graceful stop closes stdin first.
- **EP-7.** The opener is assembled from the parse.
- **EP-8.** Retro, addenda, the P3 charters upgraded.

## Phase P3 — Phone view (charter briefs; upgraded at EP-8)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-9 | Web pages: templates with live updates (D-20) | L | EP-6 | core | ☐ |
| EP-10 | Re-plan P3 and v1 close-out | S | EP-9 | core | ☐ |

**Charters.** Each is upgraded to a full brief by EP-8 or at pickup.

- **EP-9.** Routes under `/m/`: list, project, decisions.
- **EP-10.** Timings for every EP, the README's state of the app.

## Risks / open items

1. **The endpoint is undocumented.** A schema change blanks the meter.

> **Addendum (2026-10-02, EP-1).** A blockquote is never a table:
> | # | Brief | Size | Depends on | Core | Done |
> |---|---|---|---|---|---|
> | EP-98 | Quoted | S | — | core | ☐ |

<!--
| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-97 | Commented out | S | — | core | ☐ |
-->

## Decision record

Decisions live in `DECISIONS.md` (D-1 … D-20).
