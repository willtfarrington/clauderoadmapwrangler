# theta roadmap

**Repository audit at planning time (2026-08-23):** public skeleton only.

## How to use this roadmap

Every EP has a pre-written brief; a session reads its brief and the appendices it cites. Several
rows can be eligible at once; the next one is still the first unchecked row in table order.

## R0 — Foundation & governance

| # | Brief | Size | Depends on | Done |
|---|---|---|---|---|
| EP-1 | [Repository governance](EP-1-governance.md) | S | — | ☑ 0a1b2c3 |
| EP-2 | [Monorepo scaffold + CI](EP-2-scaffold.md) | M | EP-1 | ☑ 1b2c3d4 |
| EP-3 | [Rules glossary](EP-3-glossary.md) | S | EP-1 | ☑ 2c3d4e5 + 3d4e5f6 |

## R1 — Vertical slice → v0.1

| # | Brief | Size | Depends on | Done |
|---|---|---|---|---|
| EP-4 | [Board schema v0](EP-4-board-schema.md) | M | EP-2, EP-3 | ☑ 4e5f6a7 |
| EP-5 | [Golden harness](EP-5-golden-harness.md) | M | EP-4 | ☐ |
| EP-6 | [Exemplar game](EP-6-exemplar.md) | L | EP-4 | ☐ |
| EP-7 | [Viewer shell](EP-7-viewer.md) | M | EP-2 | ☐ |

## R2 — Depth & breadth

| # | Brief | Size | Depends on | Done |
|---|---|---|---|---|
| EP-8 | [Game library](EP-8-game-library.md) | L | EP-5, EP-6 | ☐ |
| EP-9 | [Scoring rubric](EP-9-scoring.md) | M | EP-20, ≥3 game EPs | ☐ |

## R3 — Hardening → v1.0

| # | Brief | Size | Depends on | Done |
|---|---|---|---|---|
| EP-10 | [Release checklist](EP-10-release.md) | S | everything | ☐ |

## Critical path & parallelism

EP-5, EP-6 and EP-7 are all eligible once EP-4 lands.

## Appendices (integrated specialist outputs)

- [Appendix A — board schema](appendices/A-board-schema.md)
