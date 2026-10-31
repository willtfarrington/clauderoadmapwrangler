# eta Single-Yard Refit — Roadmap

## How to use this roadmap

Each `EP-N-*.md` brief is self-contained. Execute in order (below), verify the acceptance
criteria, commit, then check the box here.

## Phase 1 — single-yard refit (complete)

| # | Brief | Size | Depends on | Done |
|---|-------|------|-----------|------|
| EP-0 | [Baseline & hygiene](EP-0-baseline.md) | S | — | ☑ `aa11bb2` |
| EP-1 | [Archive out-of-scope data](EP-1-archive.md) | S | EP-0 | ☑ `bb22cc3` |
| EP-2 | [Map helper + importer](EP-2-importer.md) | L | EP-1 | ☑ `cc33dd4` |
| EP-3 | [Code scope cleanup](EP-3-cleanup.md) | S | EP-1 (cleanest after EP-2) | ☑ `dd44ee5` |

> **EP-6/EP-7 two-hash note (2026-08-12).** EP-6 hit its context limit uncommitted; EP-7
> completed on top of it, so EP-6's box lists both hashes.

## Phase 2 — dashboards & flow (planned 2026-08-12)

| # | Brief | Size | Depends on | Done |
|---|-------|------|-----------|------|
| EP-5 | [Platform refit + provenance hygiene](EP-5-platform-refit.md) | M | EP-3 | ☑ `ee55ff6` + `ff66aa7` |
| EP-6 | [Track grid + rolling-stats engine](EP-6-track-grid.md) | L | EP-5 | ☑ `1a2b3c4` + `2b3c4d5` |
| EP-7 | [Arrivals model: daily rhythm](EP-7-arrivals.md) | M | EP-6 | ☑ `2b3c4d5` |
| EP-4 | [Route-graph enrichment](EP-4-route-graph.md) | L | EP-2 (met) | ☑ `3c4d5e6` |
| EP-10 | [Category rulings](EP-10-category-rulings.md) | S→M (owner escalated retirements) | EP-5 (met) — run before EP-11/EP-9 (dwell regime) | ☑ `4d5e6f7` |
| EP-9 | [Load forecasting (fan)](EP-9-load-forecast.md) | M | EP-7 | ☑ `5e6f7a8` |
| EP-11 | [Seeded backfill](EP-11-seeded-backfill.md) | M | EP-7, EP-9 | ☑ `6f7a8b9` |
| EP-8 | [Worked examples + doc refresh](EP-8-worked-examples.md) | S/M | EP-4, EP-9 | ☐ |

Standing decisions for phase 2: hand-painted charts (no plotting crate).

## Decision record (settled with the project owner, 2026-08-11)

1. **Connections.** Three footbridges join the platforms.
2. **Phase order.** Foundation first.
