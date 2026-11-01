# orchard — Roadmap

Master roadmap for orchard, a fake home-inventory app for a family's tools and appliances: a
project of the dashboard's screenshot runs. Every row is checked; the project is in maintenance.

Open items for the owner, with the detail in the last completion note:

- Now: move the old photo folder into the archive.
- On or after 2026-12-01, once the warranty export lands: re-run the import.
- Optional: label the two drills found without a receipt.

## Phase P1 — Inventory (done 2026-07-19)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | Toolchain and the item format (D-1, D-2) | S | — | core | ☑ `c1e2a37` |
| EP-2 | Items with photos and receipts (D-3) | M | EP-1 | core | ☑ `4b8f90d` |
| EP-3 | Warranty reminders (D-4) | M | EP-2 | core | ☑ `ae03c6b` |
| EP-4 | Re-plan P1 and close-out | S | EP-1 … EP-3 | core | ☑ `7f6d1e2` |
