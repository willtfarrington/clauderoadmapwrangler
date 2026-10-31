# beta — Roadmap

**Re-planned 2026-10-05 (EP-4, end of P1).** P1 took about 2 h of active session time against 3 h
planned; P2 is confirmed below with pickup notes.

## How to use this roadmap

Each session writes its brief first and links its row only when it completes.

## Phase P0 — The v1 plan (planned 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-0 | [Planning interview: 12 rounds, 48 questions; D-1 … D-14; phases P1 … P3](EP-0-planning-interview.md) | L | — | core | ☑ `d4e5f6a` |

## Phase P1 — Foundation: client, checklist, scope (planned 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | [Toolchain bootstrap: uv project, pinned dependencies, `poe check` (D-3, D-15)](EP-1-toolchain-bootstrap.md) | M | EP-0 | core | ☑ `e5f6a7b` |
| EP-2 | [Sightings client: a rate-limited client, typed records, a run log (D-5)](EP-2-sightings-client.md) | S → L (re-scoped 2026-10-02) | EP-1 | core | ☑ `f6a7b8c` + `0a1b2c3` |
| EP-3 | [Species list: a pinned checklist and the scope rule (D-6, D-7)](EP-3-species-list.md) | M | EP-2 | core | ☑ `1b2c3d4` |
| EP-4 | [Re-plan P1](EP-4-replan-p1.md) | S | EP-1 … EP-3 | core | ☑ `2c3d4e5` |

## Phase P2 — The review loop (planned 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-5 | Report engine: templates and photos (D-8) | M | EP-3 | core | ☐ |
| EP-6 | Review queue: approve, reject, defer (D-9) | M | EP-5 | core | ☐ |
| EP-7 | Bulk mode for reports (D-10) | M | EP-6 | stretch | ☐ |
| EP-8 | Maps for reports | S | EP-5 | stretch | ☐ |
| EP-9 | Re-plan P2 | S | EP-5 … EP-8 | core | ☐ |

**Brief notes.**

- **EP-5.** `beta.reports`: render a template with its photos; a dry run first, then a report
  saved under `work/reports/`. **From EP-4:** the client's run log names each request.
- **EP-6.** The review queue reads `work/queue.jsonl`.
- **EP-7.** Only worth building if the queue grows past fifty reports a week.
- **EP-8.** Maps as SVG.
- **EP-9.** Retro, addenda, `v2-notes.md` grown.

> **Owner decisions (AskUserQuestion, 2026-10-05).**
>
> | # | Question | Answer |
> |---|---|---|
> | 1 | Commit? | Yes |
