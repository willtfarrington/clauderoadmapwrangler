---
ep: 36
title: Screenshot runs over a portfolio copy
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-36 — Screenshot runs over a portfolio copy

**Size:** S · **Depends on:** EP-6 · **Blocks:** EP-23

## Context

A screenshot run (D-29's EP-6 addendum, rule 5) reads only the tracked fake portfolio,
`crates/app/fixtures/portfolio`. A layout is therefore checked on invented content alone, and what
that hides surfaces later, at the owner's look or the phase walk (risk 16). The planning session of
2026-11-08 added this row (the D-29 addendum of that date, part 2), so that a session can point a
run at a scratch copy of the real roadmap files and look at the pictures itself. The same flag
replaces EP-8's procedure for the README's pictures: a `subst` drive over a scratch folder with a
junction to the fixtures, so that the Detail pane named `P:\Projects\harbor` and not a personal
path.

**What exists.**

- `screenshot::Run { home, flags }` comes from `CRW_SCREENSHOT` and `CRW_SCREENSHOT_SETUP`.
  `Run::store` writes `fixtures/registry.toml` into the run's home on the first run, with the data
  root `portfolio_root()` (the tracked folder) filled in. A later run from the same home starts
  from what the earlier one changed.
- Every guard keys on `Board::fixtures`, which `Run::arrange` sets. A run refuses ticks and every
  write into a project (`folder_for_write`), starts no session, opens nothing in VS Code, answers
  nothing, reverts nothing, and writes no workspace file of its own on Adopt (`OwnFiles::Skip`).
  `main` gives it its own ledger and `online: false`, so the GitHub pass reads only that ledger's
  empty cache. It binds no port, polls no live session, and never reads the credentials or asks the
  endpoint. The usage meter is off, or canned with `usage`.
- The fixtures are not repositories. The git pass skips a folder without `.git`, and
  `screenshot::readings` puts the canned git, GitHub and backup readings of `FAKES` (matched by
  name) into the portfolio at every pass.
- The files a project's row reads, all from the working tree with no child process:
  - `roadmap/` (the README, the briefs, `owner-tasks.md`, `inbox.md`, and the older forms
    `owner-decisions.md` and `open-questions.md`);
  - `DECISIONS.md` at the root or one folder down, and ADR folders;
  - `docs/handoffs/EP-n.md`;
  - the stack's manifests at the root and one folder down (`badges::stack`);
  - the license file;
  - `CLAUDE.md` or `.claude/CLAUDE.md`, whose read-first rule the opener quotes.

**Session start (2026-11-09, 21:26 local time, 02:26 UTC on the 10th).** Launched from the window
as a visual launch (rule 11): ledger row `157ba22d`, visual, EP-36, open
(`crw sessions clauderoadmapwrangler`). The opener suggested Fable at high effort, and the session
runs at Opus 5.5 with max effort, chosen in the tab's panel. The tree is clean at `4c7d348`. No
owner task is open and the inbox is empty.

**Field check.** I read the records back to EP-19c's record commit (`4c7d348`, 21:15):

- `crash.log` was last written on 2026-11-03, read before;
- `crw notifications`: the newest is from 20:45, EP-19c's own;
- `crw sessions`: this session's row, and EP-19c's row still open with no end recorded (its tab
  was left open, the known gap of risk 15);
- `crw autopilot`: not paused, one private project on, its last line at 16:59, read by EP-19c's
  field check.

Nothing is new, so there is nothing to triage.

## In scope

1. **The root.** `CRW_SCREENSHOT_ROOT=<folder>` or the setup flag `root=<folder>` names the data
   root a run reads, in place of `fixtures/portfolio`. The flag wins over the variable, and a path
   with a comma takes the variable. `Run` gains `root: Option<PathBuf>`. A root that is not a
   folder is refused with a line on stderr, as an unwritable home is.
2. **Its registry**, written into the home on the first run as before:
   - A root that holds every folder of the fake portfolio (the tracked folder, a copy of it, or a
     relative path to it) reads with the fake registry, the data root replaced.
   - Any other root reads with a registry in the fake registry's shape: one `[[project]]` per
     folder (`discovery::folders`, dot folders left out), named for it. A folder is active with a
     house-style roadmap and unformalised without, which is `Folder::first_state` with a copy
     counted as a repository. Every folder takes priority 3, no pin, no launch settings and an
     empty ignore list.
   - A home whose registry names another data root is refused ("start from a fresh home"). A
     reused home would otherwise read a portfolio the run did not name.
3. **The readings.** A copied portfolio, where no project is named after a fake, gives each project
   with a folder the fake at its place in the list, under its own name: its remote, its listing in
   the fake account, its backup unit and its archive. The badge columns then hold values as the
   real window's do. The fake portfolio and its copies keep their fakes by name, unchanged.
4. **The guards stay as they are.** `Board::fixtures` is set for every screenshot run, whatever its
   root. Their notices say "a fixture or a copy" where they said "tracked fixtures".
5. **The README pictures' neutral root.** A relative root is resolved against the run's working
   folder. *(Changed while building: a relative root kept as written would have shown a neutral
   `portfolio\harbor`, but the registry refuses a data root that is not a full path, which is
   D-8's rule and stays.)* The neutral root is therefore a copy of the fixtures in a folder whose
   path names nothing of the machine: `C:\Users\Public\Projects`, which any user may create and a
   session may remove. *(Changed while building: `C:\Projects` could be created, since
   authenticated users may create folders at the drive root, but Claude Code refused to remove a
   folder directly under it.)* The copy is removed after the run, and no `subst` drive or junction
   is needed.
6. **`scripts\copy-roadmap.ps1 -Project <folder> -Into <copy root>`** copies into
   `<copy root>\<name>` the files a row reads (the list above). Lock files are written empty,
   since only their presence counts. It runs once per project (rule 9), only on a non-data project.
   It is a developer aid and names no project.
7. **The records**:
   - rule 5 in present tense with the variable, the flag, the script and the relative root (the
     brief note's acceptance);
   - the module documentation of `screenshot.rs`;
   - a sentence in the README's paragraph on the screenshot feature;
   - a D-29 addendum;
   - the roadmap's EP-36 link, and EP-32's note on the pictures.

## Out of scope

- **Retaking the README's two pictures.** EP-32 refreshes them. This session takes candidate
  pictures from the neutral root and puts the replacement to the owner, since the README is
  public.
- **The canned sessions, transcripts, revert plans, chain plans and usage.** They stay those of the
  fake projects, under the tracked folder.
- **The owner's registry.** A copy's states are derived, not the owner's. A session that wants
  them can write the run's home registry by hand before the first run, reading the owner's file
  and never writing it (rule 13).
- **A run over the real data root.** It would list the data repositories and run git in real
  repositories. A data repository is never copied (rule 5 of the house style).

## Verification / acceptance

- **Unit tests**:
  - the fixtures copied to a temp folder and run through the flag give the readings of the tracked
    ones: the same projects with their states, next briefs, counts, owner items, inbox, stack chips
    and licenses, the same unadopted and ignored folders, and the same canned readings (the brief
    note's acceptance);
  - a foreign root's registry: one entry per folder with its state, and none of the fake projects;
  - a copied portfolio's readings by place, under each project's own name, and a missing folder
    left bare;
  - a relative root resolved against the working folder;
  - the flag over the variable, a root that is not a folder refused, and a home naming another root
    refused.
- **The checks**: `cargo test` with and without `--features screenshot`, both clippy runs, and
  `cargo fmt --check`.
- **Screenshot runs**, each picture looked at and kept in the scratchpad:
  - the neutral root, a copy of the fixtures in `C:\Users\Public\Projects` named by `root=`, in
    both themes, with the README's setups;
  - a copy of the non-data projects' files made with the script, one project at a time: the list,
    the Detail pane on long notes, the owner items and the inbox, and the launch dialog's opener.
    The brief names what was checked, never the projects.
- **The release**: `cargo build --release` once the owner has quit the window from the tray, then
  `scripts\install-shortcut.ps1`.
- **Rule 3's word check** on the staged diff prints nothing.
- **Residual risk.** The real window changes only in the notices a screenshot run shows, so the
  risk sits in what a copy can and cannot say. Four limits:
  1. **Another project's layout.** The script ran on this portfolio's twelve non-data projects.
     A project that keeps a manifest two folders down, or its decision log in a skipped folder,
     shows fewer chips or no log in its copy than in the real window. A copy is for layout, not
     for readings.
  2. **The canned readings by place.** A copy's activity, hosting, CI and backup marks, and the
     state proposals that follow from a commit's age, come from the fakes, not from the projects.
  3. **"Session in progress".** A copy is no repository, so a project whose next brief was
     written ahead shows the mark (the files are the only evidence, D-9) where the real window
     would not.
  4. **The neutral root elsewhere.** `C:\Users\Public` is on every Windows install and writable by
     every user. The run only reads it.

  EP-23's walk takes nothing from these: they bound what a session may conclude from a copy, and
  the real window is unchanged. The copy's findings are listed in the completion note.

> **Completion note (2026-11-09; claude-opus-5-5/max; about 1 h 15 min to the commit, with the
> owner's one round and one quit of the window).** One session in the VS Code extension from
> 21:26 local time, launched from the window (ledger row `157ba22d`, visual, EP-36). It covered
> the reading and the field check, this brief, the code and its tests, the copy script, the
> records, the screenshot runs over the neutral root and over a copy of the real roadmaps, the
> closing round and the release.
>
> **Built.**
>
> - `screenshot.rs`:
>   - the root from `CRW_SCREENSHOT_ROOT` or the setup flag `root=` (`Run::root`, `root_of`);
>   - the registry written for a root (`registry_for`, `holds_the_fakes`);
>   - the store's refusals: a root that is no folder, and a home written for another root;
>   - the canned readings by place for a copied portfolio (`readings`).
> - The guards' notices say "fixtures or a copy" (board, revert, autopilot), and `main`'s line
>   says why a run cannot start.
> - `scripts\copy-roadmap.ps1`.
> - The README's two pictures, retaken from the neutral root, with their alt text and caption.
> - The records:
>   - rule 5, the README's paragraph on the screenshot feature and the D-29 addendum;
>   - the roadmap's EP-36 link and EP-32's note on the pictures;
>   - EP-23's note on the copy's findings.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed 451, unchanged, since the new tests need the feature. With
>   `--features screenshot` it passed 459: EP-19c's 454 and five new tests. They cover:
>   - the fixtures copied to a temp folder and read through the flag as the tracked ones (the
>     acceptance);
>   - another root's registry, and its readings by place;
>   - a relative root resolved against the working folder;
>   - the flag over the variable;
>   - the refusals.
>
>   Both clippy runs and `cargo fmt --check` are clean.
> - **Screenshot runs**, kept in the session's scratchpad:
>   - **The neutral root**, `C:\Users\Public\Projects`, with EP-8's setups
>     (`light,select=harbor,size=2300x1000` and `dark,select=meadow,expand=meadow,legend`). The
>     Detail pane names `C:\Users\Public\Projects\harbor` and `…\meadow`, and the run shows the
>     tracked run's 9 projects, 1 unadopted folder, 6 owner items and 2 inbox messages. These
>     two became the README's pictures (decision 1).
>   - **A copy of the twelve non-data projects** made with the script: ten active with a roadmap,
>     two unformalised. Taken at 1600×1000 and 2300×1000, in both themes, and with the launch
>     dialog open. I checked:
>     - the groups, and the Next and Brief columns on long real titles;
>     - the Done and Waiting counts, and the stack chips from the copied manifests;
>     - the canned readings by place in the badge columns;
>     - the Detail pane's long notes, with their headings, code and links;
>     - an expanded row's phases and the legend;
>     - the opener quoting a copied `CLAUDE.md`.
>
>     The runs found three things the fake portfolio hides, parked in EP-23's notes
>     (decision 2):
>     1. at 1440 or 1600 points the long names leave no room for any badge column;
>     2. the roadmaps' numbered risks count as owner items (D-10), about 78 across the portfolio;
>     3. the opener's quoted rule keeps `CLAUDE.md`'s hard line breaks.
> - **The release.** Built once the owner had quit the window, then `scripts\install-shortcut.ps1`
>   refreshed both shortcuts.
> - **Rule 3's word check** found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-09).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The README's two pictures | Replace them now | `docs/dashboard-light.png` and `docs/dashboard-dark.png` from the neutral root |
> | 2 | The copy's three findings | A note in EP-23's brief notes | EP-23's "From EP-36" note |
> | 3 | `C:\Projects`, which the session could not remove | The owner deleted it | Checked gone |
> | 4 | The release build | Quit the window, build and install | Built and installed |
> | 5 | Commit and push | Commit and push | Two commits, pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **A root that holds every folder of the fake portfolio reads with the fake registry.** Its
>    copies therefore read as the tracked fixtures (the acceptance) and keep the unadopted folder
>    and the missing project. Any other root gets one entry per folder.
> 2. **A copy's entries.** Active with a house-style roadmap and unformalised without, which is
>    Adopt's first state with the copy counted as a repository. Priority 3, no pin, no launch
>    settings.
> 3. **The fakes by place apply only when no project is named after a fake.** A fixtures run's
>    adopted `sketchbook` therefore stays bare, as before.
> 4. **A home that names another root is refused**, not rewritten.
> 5. **The flag wins over the variable.** A path with a comma takes the variable.
> 6. **Two changes while building.** A relative root is resolved, not kept, since the registry
>    refuses a relative data root (D-8). The neutral root moved from `C:\Projects`, which could be
>    created but which Claude Code refused to remove, to `C:\Users\Public\Projects`.
> 7. **The copy script** writes lock files empty and copies nothing from `.git`, `private` or
>    `target`.
> 8. **No review subagent**, since this is an S brief (rule 6).
>
> **Notes for later sessions.**
>
> - EP-20 … EP-22 can check a pane on real content. Copy each non-data project with the script,
>   one call per project, then start a screenshot run with `CRW_SCREENSHOT_ROOT` and a fresh
>   `CRW_HOME`.
> - A copy's "session in progress" marks and its badges are artefacts (the Residual risk above).
> - EP-32 takes the pictures as its note says.
>
> No messages for other projects, and no owner task: the owner deleted `C:\Projects` during the
> round. **Next: EP-37** (the installed exe and the window's restart), launched from the window.
