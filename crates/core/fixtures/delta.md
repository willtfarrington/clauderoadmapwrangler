# delta — Roadmap

**Owner actions outside any brief** (the roadmap assumes them): a second device enrolled before
EP-4; the soil test booked before EP-7.

## How to use this roadmap

Full briefs for P0; P1 is a charter phase whose scope is sketched in prose.

## Phase P0 — Foundations: hygiene, toolchain, test instance

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-0 | [Hygiene: ignore rules, secrets scan](EP-0-hygiene.md) | S | — | core | ☑ `9c0d1e2` |
| EP-1 | Toolchain: compose file, pinned images, the check script | M | EP-0 | core | ☐ |
| EP-2 | Test instance: a throwaway stack with synthetic beds | M | EP-1 | core | ☐ |
| EP-3 | Encrypted backups to the external volume | M | EP-1 | core | ☐ |

**Brief notes.**

- **EP-1.** The compose file pins every image by digest; `scripts/check.ps1` runs the gate.
- **EP-2.** `scripts/test-instance.ps1` starts and stops a stack with synthetic beds only.
- **EP-3.** A nightly archive, encrypted, kept thirty days.

## Phase P1 — Garden: beds, plantings, harvest log (charter)

| # | Brief | Size | Depends on | Core | Done |
|---|-------|------|-----------|------|------|
| EP-4 | Bed layout form | M | EP-3 | core | ☐ |
| EP-5 | Planting calendar | M | EP-4 | core | ☐ |
| EP-6 | Re-plan P1 | S | EP-4 … EP-5 | core | ☐ |

**Scope sketch (refine at EP-3).** EP-4: a bed per row typed from the sketch, filed under
`beds/<name>/`, as in EP-3: the archive keeps it; frost dates show on the calendar panel (EP-2).
EP-5: plantings carry a date, a variety and a spacing; the tool computes harvest windows.
EP-6: retro, addenda, the P2 charters upgraded.

Standing decisions for P1: D-18, D-19.

## Risks (struck through when resolved)

- **R-1.** ~~The image registry rate-limits pulls.~~ **Resolved by EP-0 (2026-10-02).**
