---
ep: 22
title: Roadmap-wrangler plugin
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-22 — Roadmap-wrangler plugin

**Size:** M · **Depends on:** EP-21 · **Blocks:** EP-22a, EP-24

## Context

D-27 ships the owner's roadmap method as a Claude Code plugin in this repository,
`plugins/roadmap-wrangler/`: skills plan (the multi-round interview), replan, formalise and status,
and the house-style reference, loaded by the tool's sessions with `--plugin-dir` and installed by the
owner into their own sessions from the repository. The brief note adds the method as practised at
EP-0 (a research pass, a domain map, rounds of four with the recommended option first, the planning
file written first and filled per round, a close-out with the unpicked items), the re-plan's parts
(retro, addenda, re-cut, pickup notes, v2 notes), the formalise step through `crw formalise` (EP-21)
so that the texts stay the engine's, and a structural test in place of a CLI smoke in CI. From EP-16:
a visual launch cannot pass `--plugin-dir`, so the owner's own install is what reaches most of their
sessions. It was split at pickup (below).

**What exists.**

- `crw_core::formalise` holds the house-style texts: the rule block (`block`, between `BEGIN` and
  `END`), `OWNER_TASKS_HEADER`, `TEMPLATE`; `inbox::mailman::HEADER` is the inbox's header (EP-20,
  EP-21). `crw formalise <project> [--yes | --migration]` previews, writes and prints the migration
  prompt; `crw roadmap <folder>` and `crw tasks <folder>` print the parser's reading.
- The README's house-style section describes the schema the parser expects in short.
- Claude Code 2.1.316 (`claude --help`, `claude plugin --help`, read 2026-11-10): `--plugin-dir
  <path>` loads a plugin for one session, in place; `claude plugin marketplace add <source>` takes
  a GitHub `owner/repo`, a URL or a path; `claude plugin install <name>@<marketplace>`; `claude
  plugin validate <path>` checks a plugin or a marketplace without a session. Its documentation
  (code.claude.com, plugins reference, loading reference, marketplace reference, skills; read
  2026-11-10): `.claude-plugin/plugin.json` needs only `name`; skills are `skills/<name>/SKILL.md`
  with YAML front matter (`description` recommended, 1,536 characters with `when_to_use`), invoked
  as `/<plugin>:<skill>`; `${CLAUDE_PLUGIN_ROOT}` is substituted in a plugin skill's text; a
  marketplace is `.claude-plugin/marketplace.json` (`name`, `owner`, `plugins` with a `./` relative
  `source`); a plugin from a git-hosted marketplace without a `version` is versioned by its commit,
  one from a local-path marketplace loads in place from the working tree; a `--plugin-dir` copy
  replaces an installed plugin of the same name for its session.

**Session start (2026-11-10, 14:55 local time).** Started by the owner from the window with Start
headless after the chain ended (rule 11): the chain log says at 14:37 `the chain ends after EP-21:
EP-22 is a re-plan, the owner's to run`. Ledger row `0b44aede`, headless, EP-22,
`claude-opus-5-5/max`, open (`crw sessions clauderoadmapwrangler`). The prompt is the window's
opener in its headless wording. The tree holds one change, the owner's tick of OT-22 in
`roadmap/owner-tasks.md` (its copy in the backups folder at 14:42); the inbox has no open message;
OT-21 is open and not gated.

**Field check.** I read the records back to EP-21's record commit (`193d869`):

- `crash.log` was last written on 2026-11-03, read before;
- `crw notifications`: EP-21's finished popup and the chain's end (14:37), then a manual session of
  another project waiting and finishing (14:49, 14:51);
- `crw autopilot`: the chain ended after EP-21 because the gate took **EP-22 for a re-plan**: its
  test (`autopilot::is_replan`) matches `re-plan` or `replan` anywhere in a title, and EP-22's
  names the plugin's replan skill. Rolled in (decision 2);
- `crw mail`: nothing since the first reading (13:53); no committed brief holds a block to another
  project yet;
- `crw sessions`: this run's row and EP-21's completed one;
- the backups folder: a `CLAUDE.<date>.md` copy of another project's rule file at 14:42, beside
  the OT-22 tick: the first real formalise, which OT-22 asked of the owner.

## Decided at pickup (owner, through `AskUserQuestion`, 2026-11-10)

1. **Split in two** (D-4), since the note holds more than one M under M's cap and a run stopped at
   its cap leaves its work uncommitted:
   - **EP-22** (this brief): the plugin itself, its marketplace file, the README's install notes,
     the structural test;
   - **EP-22a** (S), right after EP-22 and before EP-20a: the tool loading the plugin into its own
     headless runs with `--plugin-dir` (the plugin's files carried in the build and written under
     the tool's local folder, the flag passed only when `claude --help` lists it, the launch dialog
     and `crw doctor` saying so).
2. **The gate's re-plan misreading is rolled in** (D-13's triage rule: S-sized, found in this
   brief's own chain).
3. **The owner installs the plugin from GitHub**, after a push: the README leads with that route
   (the local folder second), and the owner task starts with the push of this brief's commits.

## In scope

1. **The plugin**, `plugins/roadmap-wrangler/`:
   - `.claude-plugin/plugin.json`: the name `roadmap-wrangler`, a display name, a description, the
     author, homepage, repository, license and keywords; no `version`, so a push is a release;
   - `skills/plan/SKILL.md`: the planning interview as practised at EP-0;
   - `skills/replan/SKILL.md`: the phase walk, the retro timed from transcripts, addenda, the
     re-cut, pickup notes and v2 notes;
   - `skills/formalise/SKILL.md`: the house-style files through `crw formalise` (its preview, then
     `--yes` on the owner's word), the migration prompt handed to a session in that project, a first
     roadmap from a brief (the plan skill for a project that needs the interview);
   - `skills/status/SKILL.md`: a project's roadmap read the way the parser reads it (`crw roadmap`
     and `crw tasks` when the tool is there), reporting the next EP, the counts and the open items;
   - `reference/house-style.md`: the schema the parser expects, the forms of D-12 and D-13, and the
     engine's texts quoted verbatim.
2. **The marketplace**, `.claude-plugin/marketplace.json` at the repository root: the name
   `clauderoadmapwrangler`, its owner and description, one entry `roadmap-wrangler` with the source
   `./plugins/roadmap-wrangler`.
3. **A structural test** (`crates/core/tests/plugin.rs`): the manifest and the marketplace parse
   with their required fields; each skill's front matter names its folder and has a description
   within Claude Code's limit; each `${CLAUDE_PLUGIN_ROOT}/…` path the skills name exists; the
   reference quotes the engine's texts byte for byte, so it cannot drift from them.
4. **The README**: install notes (GitHub first, the local folder second, the `/plugin` dialog in a
   session), and the house-style section pointing at the reference.
5. **The gate's re-plan test** (rolled in): still failing safe, the word counts wherever it
   stands, but for an item inside a comma list (the skill named among others) and a mention of
   another re-plan after an article or a preposition; it always counts as the first word and next
   to a phase's id (`Re-plan P3`, `…, re-plan P5`, `P3 re-plan: …`), as every re-plan row read in
   the portfolio is written. Tests.
6. **The records**: a D-27 addendum (the plugin as built, the marketplace, the install route,
   `--plugin-dir` in EP-22a), a D-22 addendum (the re-plan test); the roadmap's EP-22 row, the EP-22a
   row and note, the ordering paragraph, EP-23's note; an owner task for the push and the install
   (a write into the owner's user settings, D-10 addendum).

## Out of scope

- **EP-22a:** `--plugin-dir` for the tool's headless runs, the plugin's copy under the local
  folder, the dialog's and `crw doctor`'s lines.
- The chat's use of the plugin (EP-24, D-27) and the window's Formalise… and Migrate… (EP-21a).
- Installing the plugin into the owner's Claude Code (the owner's task) and the extension's
  `/install-plugin` link, whose parameters are unverified (the v2 notes, if the walk wants it).
- A `claude --plugin-dir … --help` smoke in CI (the note leaves it out; tests never run the CLI).

## Verification / acceptance

- **Tests:** the structural test above; the gate's re-plan test on synthetic titles of every form
  (first word, a phase id after a comma, a list naming the skill, a mention of an earlier re-plan,
  the link-text form).
- **The checks:** `cargo test` with and without `--features screenshot`, both clippy runs, `cargo
  fmt --check`, and a review of the diff by a subagent given only the diff (rule 6).
- **Claude Code's own check:** `claude plugin validate` on the plugin folder and on the repository
  root (the marketplace), by hand; it loads no session and spends nothing.
- **The release:** `cargo build --release` with the window up, then `pwsh -File
  scripts\install-shortcut.ps1` (rule 6); its restart request is refused while this run works.
- **Rule 3's word check** on the staged diff prints nothing; the plugin's texts name no private
  project and no machine detail, being public once pushed.
- **Residual risk.** No session has loaded the plugin: tests never run the CLI (rule 5), and
  installing it writes the owner's user settings (rule 13). Four things are not exercised:
  1. **The skills in a real session** (OT-23 and the P3 walk). `claude plugin validate` loaded the
     manifest, the marketplace and the four skills' front matter and passed them (two warnings: no
     `version`, no listing icon), and the structural test holds their names, descriptions and
     paths; how well a session follows each skill's steps shows only in use, and the walk weighs
     their wording.
  2. **The GitHub install** (OT-23). It needs the push, which is the owner's; the commands are the
     ones `claude plugin --help` lists in 2.1.316 (`install … --marketplace <owner/repo>`, and the
     two-step form for older ones) and the marketplace's validation advises.
  3. **The gate's new reading of a re-plan** on the portfolio's real rows. Every re-plan row read
     across the non-data projects' roadmaps still reads as one (the word first or next to its
     phase's id), and the two misreads found (this brief's title, a fix named after an earlier
     re-plan) now read as none. The test still fails safe elsewhere, so a stray mention of the
     word ends a chain as before, which the first week's chain log would show.
  4. **`crw formalise --yes` from a session in the project** (the review's fix). The session is
     known by the `CLAUDE_CODE_SESSION_ID` its commands get, read from this session's own
     environment (2.1.316), and the filter is tested on a synthetic listing; no real session has
     run the write (OT-23's walk may, on a project the owner wants formalised).

> **Completion note (2026-11-10; claude-opus-5-5/max; about 2 h 15 min of wall time, 47 min of
> it work, the rest the answers to three questions at pickup and three in the closing round).**
> One headless run (ledger row
> `0b44aede`, headless, EP-22), started by the owner from the window at 14:55 local time after the
> gate had ended the chain at EP-21, taking EP-22 for a re-plan.
>
> **Built.**
>
> - `plugins/roadmap-wrangler/`: `.claude-plugin/plugin.json`; the skills `plan` (the interview
>   as practised at EP-0: the survey, the research pass, the planning file written first with the
>   domain map, rounds of four recorded as they come, the shape and close-out rounds, the documents
>   from the record), `replan` (the walk first, the data check, the retro timed from transcripts,
>   addenda, the re-cut with pickup notes, the v2 notes), `formalise` (`crw formalise`'s preview
>   and `--yes` from a session in the project, the migration prompt handed to a session there, a
>   first roadmap from a brief) and `status` (`crw roadmap` and `crw tasks`, or the parser's rules
>   by hand); `reference/house-style.md`, the schema in full with the engine's four texts quoted.
> - `.claude-plugin/marketplace.json` at the repository root.
> - `crates/core/tests/plugin.rs`: the manifest and the marketplace, each skill's front matter and
>   listing length, every file a skill names, the engine's texts byte for byte.
> - The gate's re-plan test (`autopilot::is_replan`) and its test of 25 titles.
> - `crw formalise --yes` leaves out the session that runs it (`CLAUDE_CODE_SESSION_ID`), with a
>   test of the filter (`sessions_in`); `crw revert`'s check is unchanged.
> - The README's plugin section and install notes; the D-11, D-22 and D-27 addenda; the roadmap's
>   EP-22 row, the EP-22a row and note, the ordering paragraph and EP-23's note; OT-23.
>
> **The review (rule 6).** A subagent given only the diff found thirteen issues; all are fixed:
>
> 1. **A session could never finish the formalise write** in its own project: `crw formalise
>    --yes` counted the calling session, busy with that very command, and refused (OT-22 worked
>    only from a terminal). The CLI now leaves out the session named by `CLAUDE_CODE_SESSION_ID`,
>    which Claude Code gives its commands (seen in this session's environment), and still refuses
>    while any other session works or waits there (a D-11 addendum).
> 2. **The re-plan test missed re-plans**, the unsafe direction: a trailing note in brackets, markup
>    around a middle word, a symbol at either end, `P3 re-plan: …`. It now fails safe: the word
>    counts but for an inner comma-list item and a mention after an article or a preposition, with
>    the review's titles in its test.
> 3. **The status skill ran a plain `git status`**, which lists untracked folders anywhere, data
>    folders included: now `git status --porcelain -- roadmap/`.
> 4. … 13. Smaller: the README's local-clone route lacked its install step and the note that one
>    name holds one marketplace; the one-step install's CLI version; the test did not assert
>    `when_to_use` and would read a block scalar as text, and allowed `..` in a named path; the
>    plan and formalise skills both claimed a first roadmap from a brief (now formalise's alone);
>    the plan skill wrote its EP-0 brief last (now first, as the house rule says); the formalise
>    skill promised a re-check the CLI's `--yes` does not make, and named a Migrate… and a chat
>    not built yet (now: the prompt pasted into the launch dialog, or sent in a session there); the
>    D-27 addendum said "never by hand" where the skill allows the reference's texts on the
>    owner's word without the tool; the reference left out the revert among the tool's writes;
>    the transcript folder's rule replaces every character other than a letter or a digit; the
>    status skill misstated which phase-table columns the parser needs.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed 516, and with `--features screenshot` 524: EP-21's 510 and 518
>   plus six new in each run (four structural tests of the plugin, the re-plan titles, the
>   formalise check's filter). Both clippy runs and `cargo fmt --check` are clean, run again after
>   the review's fixes.
> - **Claude Code's check.** `claude plugin validate plugins/roadmap-wrangler` and `claude plugin
>   validate .` passed with warnings, before and after the fixes: no `version` (routine choice 1)
>   and no listing icon, which only Anthropic's plugin directory reads.
> - **The release.** Built with the window up; the install runs last, after the two commits (rule
>   6), its request to restart the window refused while this run works, so the window restarts
>   into the new build at the gate's moment or from File > Install and restart.
> - **Rule 3's word check** found nothing in the added lines, the plugin's texts included.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-10).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Splitting EP-22 at pickup (a headless run under M's cap) | Split, EP-22a next | EP-22 the plugin, its marketplace, the install notes and the test; EP-22a (S), right after it, `--plugin-dir` for the tool's runs |
> | 2 | The field check's finding: the gate took EP-22 for a re-plan | Roll it into EP-22 | `is_replan` reads the word's place in the title; a D-22 addendum |
> | 3 | How the owner's own sessions get the plugin | GitHub, after a push | The README leads with `claude plugin install … --marketplace willtfarrington/clauderoadmapwrangler`; OT-23 starts with the push |
> | 4 | A correction: the re-plan rule put at pickup (the title's first clause) missed the combined re-plan rows, and a stricter rule missed others (closing round) | Keep the fail-safe rule | The word counts but for an inner comma-list item and a mention of another re-plan; the D-22 addendum says so |
> | 5 | The review's fix outside the plugin: `crw formalise --yes` leaving out the session that runs it (closing round) | Keep the change | In this brief, with a D-11 addendum and a test |
> | 6 | The plugin's texts, public once pushed (closing round) | Signed off as written | Committed as written |
> | 7 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **No `version` in `plugin.json`**: Claude Code versions a plugin of a git-hosted marketplace
>    by its commit, so each push that changes it reaches `claude plugin update`; a version bumped
>    by hand would hold the owner's install until the next bump. `claude plugin validate` warns
>    about it, so its `--strict` would fail; nothing runs it strictly.
> 2. **The marketplace is the repository's root**, where `claude plugin marketplace add` looks by
>    default, named `clauderoadmapwrangler`, so the plugin's id reads
>    `roadmap-wrangler@clauderoadmapwrangler`.
> 3. **The skills may be invoked by the model** as well as by `/roadmap-wrangler:<skill>`: the chat
>    (EP-24) and a session asked to formalise a project should find them; each description says
>    when, within Claude Code's listing cap, which the test checks.
> 4. **The skills point at the reference** through `${CLAUDE_PLUGIN_ROOT}`, which Claude Code
>    substitutes in a plugin skill's text, so the reference is read only when a skill needs it.
> 5. **The structural test lives in `crw_core`'s tests**, since it compares the reference with the
>    engine's constants; it reads the repository's own files, so CI runs it as committed.
> 6. **The formalise skill without the tool**: the reference's texts may be written by hand on the
>    owner's word only, under the engine's rules (no file replaced, `CLAUDE.md` changed only between
>    the markers); with the tool, never by hand.
> 7. **The status skill reads by path only** and lists no data folder, the house rule for data
>    repositories, wherever it runs.
> 8. **The owner's tick of OT-22**, made in the window before this run (its copy in the backups
>    folder), is committed with this brief's records.
> 9. **OT-23 is not gated**: EP-22a gives the tool's runs the build's copy and does not build on the
>    owner's install.
>
> **Notes for later sessions.**
>
> - EP-22a: the plugin is seven files (`plugin.json`, four `SKILL.md`, the reference); a test
>   beside `crates/core/tests/plugin.rs` can hold the build's table to the folder.
> - `claude plugin validate plugins/roadmap-wrangler` and `claude plugin validate .` are the checks
>   by hand; the marketplace's validation advises `/plugin install roadmap-wrangler --marketplace
>   willtfarrington/clauderoadmapwrangler` as the README's one-step line, which it carries.
>
> No messages for other projects. **Next: EP-22a**, which a chain may run if the gate passes.
