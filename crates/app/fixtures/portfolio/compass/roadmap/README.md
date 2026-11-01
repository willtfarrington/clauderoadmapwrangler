# compass — Roadmap

Master roadmap for compass, a fake renderer that draws printable trail maps from GPS tracks: a
project of the dashboard's screenshot runs. Paused while the map tiles' licence is checked.

## Phase P1 — Maps (planned 2026-07-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | Toolchain and three sample tracks (D-1) | S | — | core | ☑ `9a0b4c1` |
| EP-2 | Track cleaning: jitter and gaps (D-2) | M | EP-1 | core | ☑ `d5e6f72` |
| EP-3 | Contours from the elevation tiles (D-3, D-4) | L | EP-2 | core | ☐ |
| EP-4 | A4 and A3 layouts with a scale bar (D-5) | M | EP-3 | core | ☐ |
| EP-5 | Re-plan P1 | S | EP-1 … EP-4 | core | ☐ |

**Brief notes.**

- **EP-3.** Contours every 10 m from the elevation tiles, labelled every 50 m; the tiles' licence
  decides whether they may be cached.

## Risks / open items

1. ~~The tracks' timestamps drift.~~ **Resolved by EP-2** (2026-07-20): the cleaner re-times them.
2. The elevation tiles' licence may forbid printing; asked on 2026-08-01, no answer yet.
