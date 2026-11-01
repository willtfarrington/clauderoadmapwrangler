# meadow — Roadmap

Master roadmap for meadow, a fake pipeline that turns a farm's soil-sensor logs into weekly charts:
a project of the dashboard's screenshot runs. Planned 2026-08-02; re-planned 2026-09-10.

## Phase P1 — Ingest (done 2026-08-30)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | Toolchain: uv project, pins, the gate (D-1, D-2) | S | — | core | ☑ `0b3e1c4` |
| EP-2 | Logger files read, units checked (D-3) | M | EP-1 | core | ☑ `5a6d2f1` |
| EP-3 | Gaps and spikes flagged, never filled (D-4) | M | EP-2 | core | ☑ `e81c7b0` |
| EP-4 | Re-plan P1 | S | EP-1 … EP-3 | core | ☑ `2f90d6a` |

## Phase P2 — Charts (done 2026-09-28)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-5 | Weekly charts per field (D-5) | M | EP-4 | core | ☑ `7c4a5e3` |
| EP-6 | Moisture against rainfall (D-6) | M | EP-5 | core | ☑ `91be0f8` |
| EP-7 | Re-plan P2 | S | EP-5, EP-6 | core | ☑ `d3a8c25` |

## Phase P3 — Sharing (planned 2026-09-28)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-8 | Export with ISO dates for the other tools (D-7) | S | EP-7 | core | ☑ `46e0b9d` |
| EP-9 | A printable season report (D-8) | M | EP-8 | core | ☑ `bb17f22` |
| EP-10 | Re-plan P3 and close-out | S | EP-8, EP-9 | core | ☐ |

**Brief notes.**

- **EP-10.** Timings for every EP, the README's state of the project, v2 notes compiled.
