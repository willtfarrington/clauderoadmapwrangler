# packet — Roadmap

A synthetic project in the work-packet style: `[ ]`, `[~]` and `[x] hash` cells, milestone
headings with their own mark, a packet split into `EP-4a` and `EP-4b` with a parent file that has no
row, handoff payloads inside the briefs and one `docs/handoffs/EP-N.md`, ADR files and no
`DECISIONS.md`.

## Milestones

### M0 — Bootstrap · `[x] 102af00`

| # | Packet | Size | Depends on | Status |
|---|---|---|---|---|
| EP-1 | [Governance bootstrap](EP-1-governance.md) | S | — | [x] 102af00 |
| EP-2 | [Scaffold](EP-2-scaffold.md) | M | EP-1 | [x] 9bcb7b2 |

### M1 — Data · `[ ]`

| # | Packet | Size | Depends on | Status |
|---|---|---|---|---|
| EP-3 | [Ingest](EP-3-ingest.md) | M | EP-2 | [~] |
| EP-4a | [Spine A](EP-4a-spine-a.md) | S | EP-3 | [ ] |
| EP-4b | [Spine B](EP-4b-spine-b.md) | S | EP-4a | [ ] |
