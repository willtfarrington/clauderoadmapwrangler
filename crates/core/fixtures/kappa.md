# kappa roadmap

## How to read this roadmap

- Packets are tracked in the milestone tables below; each packet file's header carries the same
  status.
- Status convention: `[ ]` planned · `[~]` in progress · `[x] <commit>` done
  (the commit that landed the work; the handoff commit follows it).

### Packet sizing and splitting

- **S** — fits one session.
- **Split convention.** A packet that must be split keeps its number and gains letter suffixes.

## Document index

| Doc | Contents |
|---|---|
| [charter.md](charter.md) | Charter, users, non-goals |
| [EP-1](EP-1-governance.md) … [EP-7](EP-7-slice.md) | Work packets, one file each |

## Milestones and work packets

One row per packet, grouped by milestone. A milestone is done when every packet in its table is
done; its heading records that with the same status convention.

### M0 — Governance bootstrap · `[x] 1f1f1f1`

Go/no-go: the repository is presentable at any commit. Met 2026-09-02 with EP-2.

| # | Packet | Size | Depends on | Status |
|---|---|---|---|---|
| EP-1 | [Repository governance bootstrap](EP-1-governance.md) | S | — | [x] 0e0e0e0 |
| EP-2 | [Scaffold + offline CI](EP-2-scaffold-ci.md) | M | EP-1 | [x] 1f1f1f1 |

### M1 — Pipeline skeleton + fixture · `[x] 4c4c4c4`

| # | Packet | Size | Depends on | Status |
|---|---|---|---|---|
| EP-3 | [Synthetic town fixture](EP-3-town-fixture.md) | M | EP-2 | [x] 2a2a2a2 |
| EP-4a | [Snapshot engine + download guards](EP-4a-snapshot-engine.md) | S | EP-3 | [x] 3b3b3b3 |
| EP-4b | [Stage runner: fingerprints, resume, cancel](EP-4b-stage-runner.md) | S | EP-4a | [x] 4c4c4c4 |

### Checkpoints · recurring

Recurring S-sized checkpoint packets; they belong to no milestone, and the packet after one
depends on it.

| # | Packet | Size | Depends on | Status |
|---|---|---|---|---|
| EP-8 | [Checkpoint 1: fixture re-run, docs sync](EP-8-checkpoint-1.md) | S | EP-4b | [x] 5d5d5d5 |
| EP-12 | [Checkpoint 2: fresh-clone re-run, budgets](EP-12-checkpoint-2.md) | S | EP-11 (M3 done) | [ ] |

### M2 — Base layer + first source · `[x] 9a9a9a9`

| # | Packet | Size | Depends on | Status |
|---|---|---|---|---|
| EP-5a | [Base-layer adapters: acquisition path](EP-5a-base-acquisition.md) | S | EP-8 (checkpoint), EP-4b | [x] 6e6e6e6 |
| EP-5b | [Curated base layer + invariants](EP-5b-base-curated.md) | S | EP-5a | [x] 7f7f7f7 |
| EP-6 | [Shop-list adapter](EP-6-shop-list.md) | M | EP-5 (= EP-5b) | [x] 8a8a8a8 |
| EP-7 | [Thin-slice metric + public output](EP-7-slice.md) | M | EP-6 | [x] 9a9a9a9 |

### M3 — Route spike · `[x] 2d2d2d2`

| # | Packet | Size | Depends on | Status |
|---|---|---|---|---|
| EP-9 | [M3 refinement gate](EP-9-m3-gate.md) | S | EP-8 (checkpoint) | [x] 0b0b0b0 |
| EP-10 | [Route sources](EP-10-route-sources.md) | S | EP-9 | [x] 1c1c1c1 |
| EP-11 | [The M3 verdict](EP-11-verdict.md) | S | EP-10 (its night finished) | [x] 2d2d2d2 |

### M4–M8 · `[ ]` refinement gates pending

No packet files exist yet; each gate authors its packets and adds their rows here.

## Phase overview

| Phase | Milestones | Outcome | Status |
|---|---|---|---|
| Build | M0–M3 | A thin slice | done |
| Grow | M4–M8 | The full set | planned |
