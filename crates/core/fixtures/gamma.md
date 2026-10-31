# gamma — Roadmap

A re-cut roadmap: EP-8 runs before EP-7 (the numbers are kept), a placeholder phase has no table,
and the brief-notes labels come in their variant forms.

## Phase P0 — Inventory design (planned 2026-10-01; done 2026-10-03)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-0 | [Design interview](EP-0-design-interview.md) | L | — | core | ☑ `3c4d5e6` |
| EP-1 | [Hygiene pass](EP-1-hygiene.md) (D-2) | S | EP-0 | core | ☑ `4d5e6f7` |
| EP-2 | [Inventory design document](EP-2-design-document.md) (D-3 … D-9) | L | EP-0 | core | ☑ `5e6f7a8` |
| EP-3 | [Hooks and checks](EP-3-hooks.md) (D-10) | M | EP-2 | core | ☑ `6f7a8b9` |
| EP-4 | [Re-plan P0](EP-4-replan-p0.md) | S | EP-0 … EP-3 | core | ☑ `7a8b9c0` |

## Phase P1 — The inventory built: shelves, labels, reports (planned 2026-10-01; re-cut at EP-4, 2026-10-03)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-5 | [Shelves and bins](EP-5-shelves.md) (D-11) | M | EP-4 | core | ☑ `8b9c0d1` |
| EP-6 | Label schema and the audit command (D-12) | M | EP-5 | core | ☐ |
| EP-8 | Weekly template and the Monday report (D-13) | S | EP-6, EP-4 | core | ☐ |
| EP-7 | Item reader: split the header, keep descriptions byte for byte (D-12) | M | EP-8 | core | ☐ |
| EP-9 | Re-plan P1 | S | EP-5 … EP-8 | core | ☐ |

**Brief notes** (reconciled with the design document at EP-4, 2026-10-03: the open items of
its last section; D-9 addendum).

- **EP-6.** Write a schema note per kind of item; `gamma audit` reports items that break it.
- **EP-8** (runs right after EP-6, EP-4). The weekly template; the Monday report links the
  week's open items.
- **EP-7** (runs after EP-8, EP-4). `gamma.items`: read and split the header with the
  description kept byte for byte.
- **EP-9.** Retro and addenda.

## Phase P2 — Integrations (charter briefs; planned 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-10 | Barcode counts | M | EP-9 | stretch | ☐ |
| EP-11 | Spreadsheet export | M | EP-9 | core | ☐ |
| EP-12 | Re-plan P2 | S | EP-10, EP-11 | core | ☐ |

**Charters.** Each is upgraded to a full brief by EP-9 or at pickup. EP-11 was reconciled with
the design document at EP-4.

- **EP-10.** Read the scanner's counts into rows; only worth building if the owner labels every
  bin.
- **EP-11.** A fixed export path, written nightly.
- **EP-12.** Retro and addenda.

## Phase P3 — Garage ingestion (placeholder; not planned)

Nothing is planned here until the owner decides what the garage shelves hold.

## Risks / open items

1. A second writer on hand-kept item notes.
