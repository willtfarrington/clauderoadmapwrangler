# iota — Roadmap

## How to use this roadmap

**Minimum context load order for a cold session** (stop when the brief's needs are met):

| # | File | Why |
|---|---|---|
| 1 | `CLAUDE.md` | session rules |
| 2 | `iota/DECISIONS.md` — **index block only** | which decisions exist |
| 3 | `roadmap/EP-n-<slug>.md` | the work |

**Definition of Done.** Acceptance commands green; ☑ ticked with the short hash; a brief whose
acceptance names CI stays `◐` until that run exists.

## Phase P0 — Foundation and governance (full briefs)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-0 | [Baseline & safety hygiene](EP-0-baseline.md) | S | — | core | ☑ 3a3a3a3 |
| EP-1 | [Toolchain, skeleton, CI](EP-1-toolchain.md) | M | EP-0 | core | ☑ 4b4b4b4 + 5c5c5c5 |
| EP-2 | [Docs + badge scheme](EP-2-docs.md) | L | EP-1 | core | ☑ 6d6d6d6 |
| EP-3 | [Re-plan P0](EP-3-replan-p0.md) | L | EP-0 … EP-2 | core | ☑ 7e7e7e7 |

> **EP-1 was `◐` for a day.** The row was held at *done but not landed* until its CI run existed.

## Phase P1 — Content model (full briefs)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-4 | [Sensor registry](EP-4-sensor-registry.md) | M | EP-3 | core | ☑ 8f8f8f8 |
| EP-5 | [Data rights table](EP-5-rights.md) | L | EP-4 | core | ☑ 9a9a9a9 |
| EP-6 | [Leak checks in CI](EP-6-leak-checks.md) | L | EP-5 | core | ◐ |
| EP-7 | [Storage roots and the disk floor](EP-7-storage.md) | L | EP-6 | core | ☐ |
| EP-8 | [Re-plan P1](EP-8-replan-p1.md) | M | EP-4 … EP-7 | core | ☐ |

**EP-7 opens with a pickup gate.** Three items closed P0 unresolved.

## Phase P2 — Evaluation (charter briefs; upgraded by EP-8, the P1 re-plan)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-9 | [Evaluation harness](EP-9-eval-harness.md) | L | EP-8 | core | ☐ |
| EP-10 | [Calibration report](EP-10-calibration.md) | M | EP-9 | stretch | ☐ |

## Traceability

| Requirement | Briefs |
|---|---|
| R-1 | EP-0, EP-6 |
