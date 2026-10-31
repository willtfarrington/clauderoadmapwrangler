# epsilon — Roadmap

**Re-plan EPs** close each phase: actual timings against planned, addenda, the next phase confirmed.

## How to use this roadmap

Hand one EP to one session. Every row is checked: the project runs as maintenance now. Rows stay
unlinked even when done, and EP-14 sits after EP-6 because a re-plan added it there.

## Phase P0 — Import and tagging charter (done 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-0 | Recipe import + manifest runner (D-1 … D-7) | M | — | core | ☑ `0d1e2f3` |
| EP-1 | Tagging charter: owner interview on cuisines and courses (D-8 … D-14) | M | EP-0 | core | ☑ `1e2f3a4` |

## Phase P1 — Protect, then sort (done 2026-10-02)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-2 | Nightly export: one file per recipe, a scheduled task with catch-up (D-15) | M | EP-1 | core | ☑ `2f3a4b5` |
| EP-3 | Duplicate check and the shelf skeleton | S | EP-2 | core | ☑ `3a4b5c6` |
| EP-4 | Re-plan P1 | S | EP-2, EP-3 | core | ☑ `4b5c6d7` |

## Phase P2 — Triage (session list; refined at EP-4; done 2026-10-03)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-5 | Root of the recipe box: runner fixes first (Undo returns a moved card to the inbox instead of deleting it; Plan and Apply refuse a manifest whose source changed since it was drafted; a dry run prints every move with its size), then the loose cards: sorted into the shelves by course and cuisine, the old scanned folder split into breads, soups and sauces, the duplicate photo exports compared by hash and the newer copy kept, the empty folders left by earlier passes removed, and the inbox emptied into the shelves or queued for the owner with a one-line reason each; every move recorded in the manifest, undoable for thirty days, and summarised in the completion note by shelf, count and size, with the queue handed to the next brief as a numbered list the owner can answer in one sitting, one shelf at a time (D-16, D-17) | L | EP-4 | core | ☑ `5c6d7e8` |
| EP-6 | The clippings inbox: about 120 cards queued in intake | M | EP-5 | core | ☑ `6d7e8f9` |
| EP-14 | Photos and scans: both collections reconciled | M | EP-6 | core | ☑ `7e8f9a0` |
| EP-7 | Re-plan P2 and the move to maintenance | S | EP-5, EP-6, EP-14 | core | ☑ `8f9a0b1` |

## Maintenance

| Routine | When | How | Decisions |
|---|---|---|---|
| Export | Nightly | The scheduled task writes one file per recipe. | D-15 |
| Sweep | When the owner asks | The sweep script drafts a manifest for the owner to approve. | D-3 |

**Open items for the owner**

- From 2027-01-01: empty the trash once each pass's undo window has passed.

> **Addendum (2026-10-04, maintenance).** The sweep runs on request only.
