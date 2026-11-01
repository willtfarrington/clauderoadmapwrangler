# harbor — Roadmap

Master roadmap for harbor, a fake desktop app that keeps a sailing club's mooring list: one of the
invented projects the dashboard's screenshot runs show. Planned 2026-09-14: nine briefs in three
phases.

## Phase P0 — Planning (done 2026-09-14)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-0 | The planning interview: 12 rounds, 48 questions; D-1 … D-14; phases P1 … P2 | L | — | core | ☑ `4f1a9c2` |

## Phase P1 — The mooring list (planned 2026-09-14)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | Toolchain: Cargo workspace, pins, gate, CI (D-2, D-3) | M | EP-0 | core | ☑ `8be3d01` |
| EP-2 | Berth register: `berths.toml` with backups and a format migration (D-4, D-5) | M | EP-1 | core | ☑ `c07f5e4` |
| EP-3 | [Tide tables: the harbour office's CSV read and cached](EP-3-tide-tables.md) (D-6) | S | EP-1 | core | ☑ `19d2b7a` |
| EP-4 | [Ledger view: berth list, detail, filters, keys](EP-4-ledger-view.md) (D-7, D-8) | L | EP-2, EP-3 | core | ☐ |
| EP-5 | Season fees: rates by length band, invoices as PDF (D-9) | M | EP-4 | core | ☐ |
| EP-6 | Re-plan P1 | S | EP-1 … EP-5 | core | ☐ |

**Brief notes.**

- **EP-4.** The ledger view: one row per berth (number, boat, owner, length, a paid-up chip), a
  detail pane with the season's history, filters by pontoon and by arrears, keys ↑ ↓ and Enter;
  the layout saved with the window; a screenshot run over a fake club. **From IN-1:** the colours
  come from the shared `palette.toml`.
- **EP-5.** Rates by length band from `rates.toml`, one invoice per member as a PDF, a run log.
- **EP-6.** Retro, addenda, P2 confirmed with pickup notes.

## Phase P2 — Members (planned 2026-09-14)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-7 | Member directory with consent flags (D-10) | M | EP-6 | core | ☐ |
| EP-8 | Waiting list and berth offers (D-11) | S | EP-7 | stretch | ☐ |

**Brief notes.**

- **EP-7.** The directory from the club's export; consent flags decide what the site may show.
- **EP-8.** Offers in waiting-list order; a stretch brief.
