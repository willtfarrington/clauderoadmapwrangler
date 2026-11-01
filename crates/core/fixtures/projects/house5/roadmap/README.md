# house5 — Roadmap

A synthetic project in the older house style: five columns, bare hashes, rows out of numeric order,
every row checked, completion recorded three different ways and once not at all, decisions kept
inline below, and no `DECISIONS.md`.

## Phase 1 — Refit

| # | Brief | Size | Depends on | Done |
|---|---|---|---|---|
| EP-0 | [Old code](EP-0-old.md) | S | — | ☑ 1a2b3c4 |
| EP-2 | [Podium refit](EP-2-refit.md) | M | EP-0 | ☑ 2b3c4d5 |
| EP-1 | [Room matrix](EP-1-engine.md) | L | EP-0 | ☑ 3c4d5e6 |
| EP-3 | [Report](EP-3-report.md) | S | EP-1, EP-2 | ☑ 4d5e6f7 |

## Decisions

**D-1 Engine.** Keep the engine. *Why.* It works.

**D-2 Report.** One page. *Why.* Enough.

## Maintenance

Every row is checked; the project is in maintenance.

Open items for the owner, with the detail in the last completion note:

- Now: delete the scratch folder the refit left behind.
- On or after 2026-11-02, once the clocks change: the nightly job starts at 01:00, not 00:00.
- Optional: confirm the two rooms filed by inference.

Notes from other projects are parked below this line; none is waiting.

## Risks / open items

1. ~~The podium overhangs the tower.~~ **Resolved by EP-2** (2026-08-12): the plate follows the
   wings. New open items spawned by the refit:
   - **Bed count**: the directory models twelve beds; revisit when a source appears.
   - ~~**Floor numbers**: two sources disagree.~~ **Resolved by EP-3** (2026-08-13).
2. Bridge levels are inferred pending a visit.
3. Speeds: ~~the pitch shifts~~ **resolved at EP-3**; the long takes still drift.
