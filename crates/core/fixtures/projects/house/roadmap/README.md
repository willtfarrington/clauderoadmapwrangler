# house — Roadmap

A synthetic project in the current house style: front matter in every brief, the new completion-note
form, two notes in one brief, handed-on and parked blocks, a companion file beside a brief, and a
decision log at the root. The next EP's brief exists without a completion note: a session is in
progress. Beside it: owner tasks, an inbox, owner actions and a risks list with ids.

**Owner actions outside any brief** (the roadmap assumes them): a hardware key on the account that
owns the remote (D-2); a second disk for the backups (D-3, before EP-4); ~~the licence text read
through~~ (D-1); a monthly look at the logs.

## Phase P0 — Planning (done 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-0 | [The plan](EP-0-plan.md) (D-1 … D-3) | L | — | core | ☑ `a1b2c3d` |

## Phase P1 — Build (planned 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | [Toolchain](EP-1-toolchain.md) (D-2) | M | EP-0 | core | ☑ `b2c3d4e` |
| EP-2 | [Registry](EP-2-registry.md) (D-3) | M | EP-1 | core | ☑ `c3d4e5f` + `d4e5f6a` |
| EP-3 | Parser | M | EP-1 | core | ☐ |
| EP-4 | Viewer | L | EP-2, EP-3 | core | ☐ |
| EP-5 | Re-plan P1 | S | EP-1 … EP-4 | core | ☐ |

**Brief notes.**

- **EP-3.** Read the tables.
- **EP-4.** Show them.
- **EP-5.** Retro and addenda.

## Risks (struck through when resolved)

- **R-1** The parser may meet a header it has never seen; the fixtures grow with each survey.
- ~~**R-2** The registry's backups could fill the disk.~~ **Resolved by EP-2 (2026-10-03):**
  fourteen daily copies, then the oldest goes.
- **R-3** Two sessions may edit one roadmap at once; ~~a lock file~~ **dropped at EP-2:** the
  snapshot reads HEAD and the tree.
