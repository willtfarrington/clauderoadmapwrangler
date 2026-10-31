# zeta — Roadmap

A mature roadmap: glossary tables, long phase statuses, rows inserted by re-plans with high
numbers, a Done cell with eight hashes, two-digit phases, no brief notes (every brief is
pre-written).

## How to use this roadmap

**Notation used in briefs** (added at EP-91). Briefs were written before some names existed:

| Brief says | It means |
|---|---|
| `zeta doctor` | `uv run zeta doctor` |
| "doctor green" | exit 0, no failing row |

**Glossary.**

| # | Term | Meaning |
|---|---|---|
| 1 | tier | the size of a data extract |
| 2 | brief | one session's work |

## Phase P0 — Charter, rules & toolchain (template step 1; full briefs; planned 2026-08-16)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-0 | [Baseline & hygiene](EP-0-baseline.md) | S | — | core | ☑ `a0a0a0a` + `b1b1b1b` |
| EP-1 | [Toolchain bootstrap (uv + pyproject)](EP-1-toolchain-bootstrap.md) | M | EP-0 | core | ☑ `c2c2c2c` |
| EP-2 | [Re-plan P0](EP-2-replan-p0.md) | S | EP-0, EP-1 | core | ☑ `d3d3d3d` + `e4e4e4e` + `f5f5f5f` + `0606060` + `1717171` + `2828282` + `3939393` + `4a4a4a4` |

## Phase P1 — Inventory & knowledge pack (template steps 2–3; full briefs; planned 2026-08-16)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-90 | [Toolchain repair (P1)](EP-90-toolchain-repair.md) | S | EP-1 | core | ☑ `5b5b5b5` |
| EP-3 | [File inventory manifest ⏱](EP-3-file-inventory.md) | M | EP-90 | core | ☑ `6c6c6c6` |
| EP-91 | [Retro A: docs and status page](EP-91-retro-docs.md) | M | EP-3 | core | ☑ `7d7d7d7` |
| EP-4 | [Re-plan P1](EP-4-replan-p1.md) | S | EP-3, EP-90, EP-91 | core | ☑ `8e8e8e8` |

## Phase P2 — Staging: lake, runner, catalog (template step 4a; full briefs; planned 2026-08-16)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-92 | [Retro B: brief reconciliation](EP-92-retro-reconciliation.md) | M | EP-4, EP-91 | core | ☑ `9f9f9f9` |
| EP-5 | [Lake layout & the step runner](EP-5-lake-runner.md) | L | EP-92 | core | ☐ |
| EP-6 | [Catalog & safe query](EP-6-catalog.md) | M | EP-5 | core | ☐ |

## Phase P10 — Free text (optional; gated at EP-60; stretch track) (template step 7d; charter briefs; planned 2026-08-16)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-60 | [Text gate: go or no-go](EP-60-text-gate.md) | S | EP-6 | stretch | ☐ |
| EP-61 | [Text embeddings](EP-61-text-embeddings.md) | L | EP-60 | stretch | ☐ |

## Phase P11 — Showcase & release (template step 9; charter briefs; planned 2026-08-16)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-62 | [Showcase notebook](EP-62-showcase.md) | M | EP-6 | core | ☐ |
| EP-63 | [Re-plan P11 and close-out](EP-63-closeout.md) | S | all UI EPs | core | ☐ |

## Capability coverage (4 categories → briefs)

| # | Capability category | Covering briefs |
|---|---|---|
| 1 | Inventory | EP-3, EP-90 |
| 2 | Staging | EP-5, EP-6 |

## Risks / open items

1. ~~The lake outgrows the disk.~~ **Resolved by EP-90 (2026-09-01).**
