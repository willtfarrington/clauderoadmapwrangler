---
ep: 27
title: Exports
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-27 — Exports

**Size:** S · **Depends on:** EP-25 · **Blocks:** EP-29 · **Decisions:** D-8, D-24, D-3

## Context

D-8 keeps a daily JSON export of the registry and the ledger in this repository's git-ignored
`private/` folder, which the owner's Documents backup archives (the ledger under
`%LOCALAPPDATA%` is outside it), and a CSV export per project on demand. D-24 has the tool write
`projects.json` and an HTML snippet (name, badge, done and total, the next items, an "as of"
date) for the public projects into the site repository, which a session of the site wires in
once; the tool never edits the site's `index.html`. D-3 says the site export names public
projects only. The brief note adds: public is `Hosting::class() == Class::Public` from the
account's list cached in the ledger's `badge_cache` (EP-7); the site gets a `Handed on (to
<site>)` block naming it by its registry name, delivered by the mailman once this brief is
committed (EP-20, EP-23); the daily export reads the ledger at the schema it has (7 since
EP-25).

**Session start (2026-11-13).** The prompt is this project's opener; `crw sessions
clauderoadmapwrangler` lists this run (`6156c03c`, headless, EP-27, started 17:04), and `crw
autopilot`'s chain log says the autopilot chained it after EP-26a with claude-opus-5-5 at max
(the rules' choice) under S's cap of 45 min. Read: the P4 table and EP-27's brief note, D-3,
D-8, D-24 and D-26 with their addenda, EP-26a's completion note, `owner-tasks.md` (no open task)
and the inbox (no open message). The field check: `crash.log` is still EP-14a's of 2026-11-03,
seen; `crw notifications` since EP-26a's record commit (15:12) shows its finish, then the chain
held three times (15:13, 16:29, 16:34) at 57 % of the session (five-hour) window against the
50 % mark until the window's reset at about 17:00, then this chain at 17:04: the first real
five-hour hold, whose reading EP-40 parked for EP-29 (it held and let go as built). `crw
sessions` shows this run open; `crw autopilot` six projects on. Nothing odd to triage. The
working tree was clean at the start.

**The code as found.** No export exists. `crw status` reads what a card needs: the stack chips
(`stack::read`), the counts (`snapshot::read_tree(folder).current()` → `Roadmap::counts`), the
hosting (`hosting::classify(names, remote, account)`) with the account's list asked live; the
window keeps that list in the ledger's cache (`online::ACCOUNT`, `Reading::from_cache`). The
registry names no site: D-6 manages the site's repository as the export target, and it is
registered as an unformalised project with no roadmap. `paths::repository()` is the repository
the build comes from; `Ledger::sessions` and `Ledger::snapshots` read the rows a CSV needs; the
window's files pass writes the daily counts (`portfolio::record_counts`).

## In scope

1. **`core::export`.** The site: `Site` (`as_of`, the projects) and `SiteProject` (name, badge
   text from the stack chips, done, total, percent, the next one or two undone rows with id,
   title and size), `public(&Hosting)`, `site_json`, `site_html` (one card per project, every
   text escaped), `write_site(site_folder, &Site)` writing `assets/data/projects.json` and
   `assets/data/projects-cards.html` and nothing else. The daily export: `Ledger::dump` (every
   table at the schema the ledger has, each row an object keyed by its columns) and
   `export::daily(registry_file, ledger, folder)` writing `registry.toml` (the file as it is)
   and `ledger.json` into `<folder>/<date>/`; `export::folder()`, this repository's
   `private/export`, or `CRW_HOME\export` in a trial. The CSV: `sessions_csv` and
   `snapshots_csv`, quoted as RFC 4180 says.
2. **The registry's `site` setting**: the registered project that receives the site export
   (`crw setting site <project|none>`), refused for a name the registry does not hold.
3. **`crw export`**: `export site [--yes]` previews where the files go, each project's figures
   and why a project is left out (not public, no phase table), and writes with `--yes` (the
   formalise pattern), refused while no site is set or no account list is cached; `export
   daily` writes today's folder (again on asking, replacing it); `export csv <project>` writes
   `<project>-sessions.csv` and `<project>-snapshots.csv` into today's folder.
4. **The window's daily export**: once a day at the files pass that records the counts, when
   today's folder holds no `ledger.json`, on the loader's thread; never in a screenshot run.
5. **Tests**: the site's JSON (fields, percent, the two-row limit, checked rows skipped), the
   HTML's escaping, the public filter, the CSV's quoting, the dump's tables and rows, the daily
   folder's two files, the `site` setting refused for an unknown project.
6. **Docs**: addenda to D-8 and D-24, the README's words, the roadmap row linked, the
   completion note with a `Handed on (to <site>)` block and an owner task for the first write
   into the site.

## Out of scope

- An Export item in the window's menus and a CSV item in the row's menu: the CLI carries the
  owner's yes for now (parked for EP-29).
- Wiring the snippet into the site's page and any push (the site's own session, D-24).
- A retention rule for the daily folders, each a whole copy of the ledger (parked for EP-29).
- Restoring a registry or a ledger from an export.

## Verification / acceptance

1. `cargo test --workspace` and `cargo test --workspace --features screenshot` pass; both
   clippy runs and `cargo fmt --check` are clean; rule 3's word check on the staged diff prints
   nothing.
2. A trial under `CRW_HOME` with a scratch registry over a scratch data root of synthetic
   names: `crw setting site`, `crw export site` and `--yes`, `crw export daily`, `crw export csv
   <project>`, the files read back.
3. `crw export site` (the preview, a read) over the owner's registry and ledger, checking the
   public filter against the cached list; `crw export daily` once over them, into the ignored
   `private/export/`.
4. The release build and the install run last (`scripts\install-shortcut.ps1`).

**Residual risk.** The site's page was not exercised: the first real write into the site's
repository is the owner's (OT-27), and the snippet's look waits on the site's session, which
styles its classes. The public filter rests on the account's list the window cached, up to a
day old (a repository made public since reads as not public until the next GitHub pass, which
F5 asks), and was checked against the real cache in the preview only; the session expects it
to hold because the list is EP-7's, read by the same `hosting::classify` the row's hosting
badge uses. The window's daily write was exercised in a trial of `crw export daily` and in
tests, not by a running window: it runs at the first files pass of a day once the window
restarts into this build, and a failure there is silent until `crw export daily` is run by
hand. Each day's folder holds a whole copy of the ledger (1.0 MB of JSON today, growing with
the ledger), which the Documents backup zips every day; EP-29 weighs a retention rule.

> **Completion note (2026-11-13; claude-opus-5-5/max; about 25 min of wall time, a headless run
> chained by the autopilot, no question asked).** Ledger row `6156c03c`, started 17:04 under S's
> cap of 45 min; the prompt was the opener. Built as scoped.
>
> **Built.**
>
> 1. **`core::export`**: `Site`, `SiteProject`, `NextRow`, `site_project` (the stack chips
>    joined, done, total, the percent rounded down, the first two undone rows), `public`,
>    `site_json`, `site_html` (every text escaped), `site_data` and `write_site` (the two files
>    in `assets\data` and nothing else); `folder`, `day_folder`, `written` and `daily` (the
>    registry's copy and `ledger.json` with the schema and the time); `sessions_csv`,
>    `snapshots_csv` and `write_csv` (RFC 4180, CRLF). `Ledger::dump` reads every table at the
>    schema the ledger holds.
> 2. **The `site` setting** (`Settings::site`, the 18th key), kept in the registry's spelling and
>    refused for an unknown project; `crw setting` words it.
> 3. **`crw export site [--yes]`, `crw export daily`, `crw export csv <project>`**, with the
>    refusals the brief names and a plain word for a ledger not yet created (found in the trial).
> 4. **The window's daily export** (`portfolio::daily_export`) after the files pass's counts,
>    when today's folder holds no `ledger.json`; never offline (a screenshot run) or in tests.
> 5. **Docs**: addenda to D-8 and D-24, the README's words, the roadmap row linked, EP-29's
>    note given the parked items.
>
> **Checked.** A trial under `CRW_HOME` (a scratch registry over a scratch data root of two
> synthetic projects): `setting site` in another case kept the registry's spelling, an unknown
> name refused; the preview and `--yes` refused with no list cached; `export daily` wrote the
> registry's copy and all nine tables at schema 7; `export csv` wrote both files with their
> headers. The trial showed the site's path with mixed separators (`assets/data` joined whole),
> now joined a component at a time. Over the owner's registry and ledger: the preview (a read)
> listed four public projects with phase tables and left out eleven (three data repositories
> unread, four local, two private, one orphan, and the site itself for having no phase table),
> matching the cached list of 9 repositories (18 h old); one `export daily` wrote
> `private\export\2026-11-13\` (1.0 MB of JSON, ignored by git).
>
> **Verification.** `cargo test --workspace` (180 in `app`, 386 in `core`) and `cargo test
> --workspace --features screenshot` pass; both clippy runs (after one fix: a useless
> `into_iter`) and `cargo fmt --check` are clean; rule 3's word check on the staged diff printed
> nothing. An S brief, so no review subagent (rule 6). The release build and the install run
> after the record commit, the opener's last step.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> No other owner decision arose: the scope is the brief note's; the first write into the site's
> repository (a write into another project) is OT-27, the owner's hands; the message to the site
> is the brief note's plan, delivered by the mailman once committed.
>
> **Routine choices (the session's call, logged).**
>
> 1. **The CLI carries the owner's yes** (`--yes`, the formalise pattern); the window's menus
>    are parked for EP-29's walk.
> 2. **The site named by a setting**, not a folder name in code (rule 12, D-3).
> 3. **Public from the window's cached list**, never asked of GitHub at the export; the write
>    refused without a list or a card, so a stale state never blanks the site's cards.
> 4. **A data repository left out before any git child.**
> 5. **One JSON document per day** of every table, read whatever the schema; the CSV leaves the
>    run's text and its start's record to it.
> 6. **The daily write in the files pass's thread**, after the counts, a stat per pass once
>    written; silent on failure as the counts are, `crw export daily` saying why.
> 7. **The scratchpad** is `target\tmp\ep27\` (ignored): the logs, the trial's home and data
>    root, the real preview's output.
>
> **Owner task:** OT-27, the first site export. **Next: EP-34** (hosting and backup actions),
> chained by the autopilot when its gate allows, else from the window. The work commit is
> `53c474b`; the record commit follows.

> **Handed on (to prime).** From clauderoadmapwrangler's EP-27 (2026-11-13): Roadmap Wrangler's
> site export writes `assets/data/projects.json` and `assets/data/projects-cards.html` into this
> repository when the owner runs `crw export site --yes`, replacing both at each export, and
> nothing else here. `projects.json` is `{"as_of": "YYYY-MM-DD", "projects": [{"name", "badge",
> "done", "total", "percent", "next": [{"id", "title", "size"}]}]}` for the owner's public
> projects with a roadmap; the snippet is one `<article class="project-card">` per project in a
> `<div class="project-cards">`, with a `<progress>` bar and the next rows. Wire one of them
> into the page once (fetch the JSON and draw the cards, or include the snippet), style the
> classes, and drop the hand-typed figures they replace; the tool never edits the page and
> never pushes.
