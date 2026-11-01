# quarry — Roadmap

Master roadmap for quarry, a fake command-line tool that renames scanned photos by the date written
on their backs: a project of the dashboard's screenshot runs. Planned 2026-09-01.

## Phase P1 — Rename (planned 2026-09-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | Toolchain and the sample scans (D-1, D-2) | S | — | core | ☑ `61f0a3b` |
| EP-2 | Date reader: handwriting to a date, with the confidence kept (D-3) | L | EP-1 | core | ☑ `0e9d47c` |
| EP-3 | Dry run: the plan of renames as a table (D-4) | M | EP-2 | core | ☑ `a5c3e18` |
| EP-4 | Re-plan P1 | S | EP-1 … EP-3 | core | ☑ `3d2b6f0` |

## Phase P2 — The archive (planned 2026-10-20)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-5 | [Archive layout: folders by decade, the shoebox's labels kept](EP-5-archive-layout.md) (D-5) | M | EP-4 | core | ☐ |
| EP-6 | [Undo log: every rename reversible](EP-6-undo-log.md) (D-6) | S | EP-5 | core | ☐ |

**Brief notes.**

- **EP-5.** Folders by decade, then by the shoebox label; the owner names the labels first.
- **EP-6.** One log line per rename; `quarry undo <run>` replays it backwards.
