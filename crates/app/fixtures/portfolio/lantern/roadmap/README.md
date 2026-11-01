# lantern — Roadmap

Master roadmap for lantern, a fake static site for the same invented sailing club (news, race
results, the members' page): a project of the dashboard's screenshot runs. Planned 2026-10-05: six
briefs in two phases.

## Phase P0 — Planning (done 2026-10-05)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-0 | The planning interview: 6 rounds, 24 questions; D-1 … D-8 | M | — | core | ☑ `aa01b7e` |

## Phase P1 — The site (planned 2026-10-05)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | Site skeleton: templates, a build script, the deploy check (D-2, D-3) | M | EP-0 | core | ☑ `3c9e0d2` |
| EP-2 | [Race results: one page per regatta from the timing sheet](EP-2-race-results.md) (D-4) | M | EP-1 | core | ☑ `7d41f8b` |
| EP-3 | Home page: news, the next high water, the season's calendar (D-5) | S | EP-2 | core | ☐ |
| EP-4 | Members' page behind the club login (D-6, D-7) | M | EP-3 | core | ☐ |
| EP-5 | Re-plan P1 | S | EP-1 … EP-4 | core | ☐ |

**Brief notes.**

- **EP-3.** The news list from `news/*.md`, the season's calendar from `calendar.toml`, and the
  next high water from harbor's tide cache when IN-1 is rolled in.
- **EP-4.** The members' page reads the consent flags; nothing personal is built into the site.
- **EP-5.** Retro and addenda.
