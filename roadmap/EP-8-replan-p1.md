---
ep: 8
title: Re-plan P1
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-8 — Re-plan P1

**Size:** S · **Depends on:** EP-1 … EP-7a · **Blocks:** EP-9 … EP-16 (P2 confirmed with pickup
notes) and the public remote (D-2)

## Context

P1 closed with EP-7a on 2026-11-01: the read-only dashboard shows every registered project's next
brief with its progress, owner items, badges and session mark under a header row with a legend, and
`crw` reads roadmaps, briefs, decision logs, owner tasks and inboxes and ticks the tool's own files.
This re-plan closes the phase (D-4): the briefs against plan, the addenda P1 left to it (D-7, D-16
and D-18 from IN-1 and IN-2; D-24's actions from EP-7; the plan's totals after EP-7a), the README's
pictures of the fake portfolio, the pre-flip audit, the public flip decided at EP-0 (D-2), P2
confirmed with pickup notes, and `v2-notes.md` grown.

**Session start (2026-11-01, 19:23).** Started from `DATA\clauderoadmapwrangler.code-workspace`.
OT-3 (the owner's look at EP-7a's polish on the real portfolio, before EP-8) is ticked in the
working tree by the owner, and the inbox holds nothing open, so no adjustment reaches the pictures.
The owner's prompt carried a note from a session before the workspace split: "EP-8's pre-flip audit
should include roadmap/inbox.md. I kept IN-1 and IN-2 generic, but the audit is the place to
confirm it before the repository goes public." It is written into the inbox as IN-5 and rolled in
(scope 4): it fits this brief and is S-sized.

**Verified at pickup.**

- History: 25 commits on `main`, one branch, no tag or stash, and `origin` holds the same head.
  `git grep` for every name kept in `private/`, the environment repository's, the aggregate
  workspace file's and the machine's finds nothing in any commit's tree or message.
- The tracked planning record (`planning-2026-10-31.md` §1 and §2.4, domain N, round 16) states
  machine details in every commit since EP-0: the backup drive's label, path and schedule, the
  account's plan and rate-limit tier, the phone and its VPN, Tailscale Serve's port mappings, the
  antivirus products and the shell's rights, a start-up manager, and the usage figures at the
  probe with a time zone. EP-4's brief shows the Windows user name inside a transcript path, and
  EP-31's brief note names the VPN. None is a token, an address or a host name.
- All 25 commits carry the author address the owner's four public repositories already carry.
- GitHub: the repository is private, its MIT license detected, issues and projects on, wiki off,
  no description and no topics; the owner's public repositories carry a one-line description and
  no topics.
- `crw doctor`: claude 2.1.316 (auto-updated since EP-1's 2.1.261; the extension still bundles
  2.1.314), gh 2.103.3, tailscale 1.104.4, code 1.144.0. The 2.1.316 `--help` has no `--max-turns`
  and lists the effort levels low … max without `ultracode`; `--permission-prompts host|none`,
  `--add-dir`, `--settings`, `--plugin-dir`, `--session-id`, `--name` and `--max-budget-usd` are
  there.
- crates.io: eframe 0.37.0, egui_tiles 0.18.0 and claude-codes 2.1.316 are the newest releases and
  the pins (D-15); egui_plot's newest is 0.38.0, the version EP-26 names.
- Claude Code's permissions page (code.claude.com, read 2026-11-01): `acceptEdits` "automatically
  accepts file edits and common filesystem commands such as `mkdir`, `touch`, `mv`, and `cp` for
  paths in the working directory or `additionalDirectories`"; "`Edit` rules apply to all built-in
  tools that edit files"; Windows paths match in POSIX form (`//c/…`); a folder added with
  `--add-dir` loads its `CLAUDE.md`, `.claude/rules/` and `CLAUDE.local.md` only when
  `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1` is set, and `permissions.additionalDirectories`
  in a settings file grants file access only. The page does not say whether an `Edit` deny rule
  also stops the filesystem commands `acceptEdits` accepts.
- Code: the ledger is at schema 2 with `session`, `usage_sample`, `snapshot`, `event` and
  `badge_cache`; `process::turn()` and `process::output_with` are the runner's starting points;
  the minimum Claude Code version is 2.1.261 (`version::MIN_CLAUDE_CODE`); a fresh registry names a
  `workspace_file` only when the data root holds exactly one `.code-workspace`
  (`registry::only_workspace_file`), which no longer holds since the owner's per-project files.
- P1's out-of-scope lists name two items no later row holds: D-25's cards view (EP-6) and
  dismissing a proposal (EP-2); EP-7a left sorting by a click on a header unplaced.
- The transcripts of EP-0 … EP-7a are filed under this project's slug, one session per brief.

## In scope

1. **Retro.** A table *EP · size planned · actual · what bit* for EP-0 … EP-7a from the completion
   notes, the commit times and the session transcripts (timestamps only: from the session's first
   entry to the EP's last commit; active time leaves out gaps over five minutes, the waits on the
   owner); the phase total against plan; lessons.
2. **Addenda** dated 2026-11-01, EP-8, appended and never rewritten: D-2 (the flip), D-3 (the
   audit and the machine details), D-4 (the totals and the measured pace), D-7 (per-project
   workspace files, IN-1), D-12 (lettered ids in front matter), D-16 and D-18 (neighbour folders by
   `--add-dir`, IN-2), D-24 (where the actions go), D-25 (where the cards view goes); no new number
   unless the owner approves one.
3. **Parked items.** Every out-of-scope item of EP-1 … EP-7a gets a disposition: a later brief, a
   v2 note, done, or dropped with the reason (a table in the completion note).
4. **The pre-flip audit** (EP-1's addition and IN-5): `git grep` over every commit for the names in
   `private/`; a re-read of the tracked planning record, the briefs, `owner-tasks.md` and
   `inbox.md` for personal details; the findings to the owner before the flip.
5. **README**: two self-screenshots of the fake portfolio (`CRW_SCREENSHOT`, `--features
   screenshot`), each looked at for anything personal before it is committed; the status paragraph
   brought to P1's end.
6. **Roadmap**: the header's totals; the EP-8 row linked and, after the commit, ticked; P2
   confirmed with pickup notes (EP-9 … EP-16) and the later notes a P1 fact changes; rows added
   where the owner places them (IN-1 and IN-2's follow-ups, D-24's actions, D-25's cards view),
   each with the next free number (the house rule for an added brief) and a brief-notes bullet;
   the risks brought up to date; `v2-notes.md` grown.
7. **The public flip**, after the owner's yes and the commit: `gh repo edit
   willtfarrington/clauderoadmapwrangler --visibility public
   --accept-visibility-change-consequences` with the description the owner approves, then the
   CI run checked.
8. **Owner decisions** through `AskUserQuestion` (rule 7): the audit's findings; the sizes after
   the retro; the homes of the added work; sign-offs of the addenda, the roadmap edits, the v2
   notes, the README and any `CLAUDE.md` change; the commit, the push and the flip.

## Out of scope

- Any code, dependency, pin or build change: no crate moves (the pins are the newest releases), and
  the added rows are planned here, not built.
- Writing P2's briefs, or an added row's brief (each session writes its own, rule 2).
- Rewriting `DECISIONS.md` text or a completed brief's note (addenda and dated notes only).
- The owner's registry and ledger (rule 13); another project's files (rule 4).

## Verification / acceptance

- `cargo test` green at pickup; no code changes, so nothing else is built but the debug app with
  the screenshot feature for the pictures.
- The retro covers EP-0 … EP-7a with planned and measured times and the phase total; every figure
  is checked, not recalled.
- Every addendum is dated and cites EP-8; numbering stays D-1 … D-30 unless the owner approves a new
  decision; no decision text is rewritten.
- Every parked item of P1 has a disposition; no dangling D or EP reference in `roadmap/README.md`;
  every added row has six cells and a brief-notes bullet; `crw roadmap .` (with `CRW_HOME` on a
  scratch folder) reads the roadmap with no problem.
- The pictures show only fake projects and no personal path; `git grep` over the diff and the new
  files finds no name kept in `private/` and none of the machine details the owner keeps out.
- After the owner's yes: the commits pushed, CI green, the repository public with its description,
  and the flip recorded.
- The owner has reviewed every decision interactively.

> **Completion note (2026-11-01; opus/max; ≈ 31 min to the owner's last answer).** One session from
> `DATA\clauderoadmapwrangler.code-workspace`, from 19:23: the baseline `cargo test`, this brief, the
> transcripts' timings, the audit and the two pictures, then the owner's four rounds of questions,
> with the drafts written between the second and the third. OT-3 was ticked by the owner and the
> inbox held nothing open; the owner's note in the prompt became IN-5, rolled in. No wait passed five
> minutes, so the 31 minutes are all active, against S ≈ 30 min.
>
> **Item 1, retro.** From the completion notes, the commit times and the session transcripts
> (timestamps and the brief each session wrote, nothing else; each session's first entry to its EP's
> last commit; active time leaves out gaps over five minutes, the waits on the owner):
>
> | EP | size planned | actual | what bit |
> |---|---|---|---|
> | EP-0 | L ≈ 2 h | 2 h 14 min of session, 1 h 20 min active (the note's ≈ 2 h 45 min counts from the survey) | 72 questions in 18 rounds, five waits of 5–18 min; a "remote ahead" reading and one stack were wrong at first |
> | EP-1 | M ≈ 1 h | 28 min, all active | started from another project's window; `WaitForInputIdle` returns before eframe's window exists |
> | EP-2 | M ≈ 1 h | 38 min, all active | an empty `roadmap/README.md` is not a roadmap; the folder count corrected at pickup |
> | EP-3 | M ≈ 1 h | 51 min, 46 active | the first fixture drafts paraphrased real roadmaps and were rewritten (D-3); one call read the owner's registry without `CRW_HOME` (read only) |
> | EP-4 | M ≈ 1 h | 56 min, 41 active | the literal "brief without a note" rule would have shown two pre-written projects as in progress (D-9 addendum) |
> | EP-5 | M ≈ 1 h | 33 min, all active | `dedent` left the list marker on an item's first line; a test count misstated in a question |
> | EP-6 | L ≈ 2 h | 59 min, 45 active | code only the feature used was dead without it (rule 6's second clippy); the fixed columns too wide at 1600 points |
> | EP-7 | M ≈ 1 h | 59 min, all active; then 31 min (10 active) turning IN-4 into EP-7a | `crw status` ran once over the whole registry, data repositories included (the owner's rule since); a progress bar that read as a link's underline |
> | EP-7a | M ≈ 1 h | 53 min, 47 active | a scroll area inside a panel never grows; `Grid` centres every cell; `ep: 7a` read as no EP |
> | EP-8 | S ≈ 30 min | 31 min to the last answer, all active (the commits, CI and the flip after) | the first picture printed the repository's real path; the timing pass first counted the fixtures' own briefs |
>
> EP-1 … EP-7a took 6 h 17 min of session (5 h 37 min active) against 9 h planned, about 70 %; with
> EP-0 and this re-plan, 9 h 2 min (7 h 28 min active) against 11.5 h. **Three lessons.** (1) The M
> is generous: P1's took 28 to 59 minutes, 45 on average, and the one L 59; the sizes stay the
> planning unit (decision 3), and the autopilot's caps (D-22) keep a margin of two to one or more.
> (2) The owner's look at the real window finds what the fake portfolio cannot show: OT-2's notes
> became EP-7a, so an owner task after each brief that changes the window stays the practice. (3) A
> privacy check must read pictures and paths, not only names: the planning record's machine details
> sat in every commit since EP-0, and the first README picture printed a personal path; rule 3 now
> checks each commit's added lines against a private word list, and pictures name a neutral data
> root (decisions 1, 2).
>
> **Item 2, addenda.** Ten, dated 2026-11-01 and citing EP-8: D-2 (the flip), D-3 (the audit and
> the check before each commit), D-4 (the measured pace and the totals), D-6 (dismissing a
> proposal), D-7 (per-project workspace files), D-12 (lettered ids in front matter), D-16 and D-18
> (read-only neighbour folders), D-24 (EP-34), D-25 (the cards view in EP-26). No new number: every
> P1 fact refines an existing decision.
>
> **Item 3, parked items.**
>
> | Brief | Parked item | Disposition |
> |---|---|---|
> | EP-1 | App icon, single instance, start-at-login, `doctor`'s other checks | EP-28, as planned |
> | EP-1 | The public flip and the README's pictures | done here |
> | EP-2 | Dismissing a proposal | EP-33 (D-6 addendum) |
> | EP-2 | The registry's later keys, ledger rows, the daily export, integrity checks | EP-9 … EP-30, as planned |
> | EP-3 | A split set's name; eligibility from the dependency graph | v2 §6 (D-9 stands) |
> | EP-4 | A running session as an in-progress source | EP-12 (pickup note) |
> | EP-4 | Brief sections beyond the labelled blocks; YAML beyond flat lines | dropped: no brief needs them |
> | EP-4 | IN-1 and IN-2's own work | EP-33, EP-9, EP-10, EP-11, EP-12, EP-21; D-7, D-16, D-18 addenda |
> | EP-5 | Delivery and composing, the prompt's section, creating the files, the chat, the registry mirror | EP-20, EP-10, EP-21, EP-24, EP-30, as planned |
> | EP-6 | D-25's cards view | EP-26 (D-25 addendum) |
> | EP-6 | The other tiles, toasts and tray, the graphs | EP-10 … EP-28, as planned |
> | EP-7 | D-24's actions | EP-34 (D-24 addendum) |
> | EP-7 | The badge events' notifications, the usage share, the drive's label | EP-15, EP-18, EP-28, as planned |
> | EP-7 | Fetching before the head check | rejected at EP-7 (D-24 addendum) |
> | EP-7a | Drag-resizable columns | v2 §9, since EP-7 |
> | EP-7a | Sorting by a click on a header | v2 §9 |
>
> **Item 4, the audit.** No name kept in `private/` (nor the environment repository's, the
> aggregate workspace file's or the machine's) in any of the 25 commits' files or messages; IN-1 …
> IN-5 and the owner tasks generic; the author address is one the owner's public repositories
> already show. Found and put in general words on the owner's choice: the planning record's
> machine details (§1, §2.3, §2.4, domain N, round 16, with a dated note), the Windows user name in
> EP-4's transcript path and the VPN's name in EP-31's charter; the first wording is appended to
> the private survey, and `private/public-check.txt` (27 words) is the list rule 3 checks against.
> `DECISIONS.md` keeps its wording (D-28, D-29).
>
> **Item 5, roadmap.** A re-plan paragraph in the header (35 briefs, 9 S · 20 M · 6 L ≈ 36.5
> session-hours); the EP-8 row linked; EP-33 in P2 before EP-11, which now depends on it, and EP-34
> in P4 after EP-27; EP-9's and EP-26's rows widened; P2's and P4's rationale and standing
> decisions with dated lines; pickup notes under EP-9, EP-10, EP-11, EP-12, EP-13, EP-14, EP-16 and
> EP-26, bullets for EP-33 and EP-34, and the picture method under EP-32's charter; risks 3 and 5
> brought up to date and risk 11 (the public repository) added. `v2-notes.md` gains five items.
> The README: the status paragraph, the two pictures, the checks line as rule 6 has it. `CLAUDE.md`
> rule 3: the check before each commit.
>
> **Verification.** `cargo test` 164 passed at pickup (22 + 116 + 2 + 7 + 6 + 11); no code,
> dependency or build file changed, so the only build was the debug app with the screenshot feature,
> for the pictures. `crw roadmap .` with `CRW_HOME` on a scratch folder: 6 phases, 36 rows, 9 done,
> next EP-8, 30 decisions with 20 addenda, no problem; every EP reference resolves (EP-0 … EP-34 and
> EP-7a) and none to a decision passes D-30. The word list found nothing in the added lines of the
> working diff or in this brief; both pictures were looked at (fake names, `example/…` origins, the
> `P:\Projects\…` paths).
>
> **Owner decisions (`AskUserQuestion`, 2026-11-01).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The audit's machine details in the tracked files | Put them in general words | The record, EP-4's line and EP-31's charter changed; the first wording in the private survey; D-3 addendum |
> | 2 | A standing check before each commit | Add it to rule 3 | `CLAUDE.md` rule 3; `private/public-check.txt` |
> | 3 | Re-cut the sizes after the retro | Keep the sizes | D-4 addendum with the measured pace |
> | 4 | P2's order and the pins | Keep the order; no pin moves | EP-9 → EP-10 → EP-33 → EP-11 → … → EP-16 |
> | 5 | IN-1's workspace follow-ups and dismissing a proposal | A new S row before EP-11 | EP-33; D-6 and D-7 addenda |
> | 6 | Neighbour folders in headless runs | Read-only neighbours | EP-9 and EP-10 notes; D-16 and D-18 addenda |
> | 7 | D-24's actions | A new M row in P4 | EP-34 after EP-27; D-24 addendum |
> | 8 | D-25's cards view | Fold into EP-26 | EP-26's row and note; D-25 addendum |
> | 9 | The ten addenda | Accept as written | Written |
> | 10 | The roadmap edits and the v2 notes | Accept as written | Item 5 |
> | 11 | The README and its two pictures | Accept as written | `docs/dashboard-light.png`, `docs/dashboard-dark.png` |
> | 12 | The generalised text and rule 3's sentence | Accept as written | As drafted |
> | 13 | Commit and push | Commit and push | Two commits in the house pattern, pushed to the private origin |
> | 14 | The public flip | After a green CI run | Recorded after the commit |
> | 15 | The repository's description | The full sentence | "Windows desktop dashboard and session launcher for a portfolio of Claude Code projects run as roadmap briefs (Rust, egui). Built for one developer's workflow." |
> | 16 | Recording CI and the flip | A third commit | The line after the commit |
>
> **Routine choices (the session's call, logged).** (1) EP-33 and EP-34 take the next free numbers,
> the house rule for an added brief; EP-7a keeps its letter. (2) Session times come from the
> transcripts' timestamps, matched by the Write or Edit of this repository's own brief (a first pass
> counted the fixtures' briefs of the same names); EP-7's IN-4 tail is reported apart. (3) Dismissing
> a proposal, placed with EP-33 by decision 5, got its own D-6 addendum. (4) The planning record says
> in a dated note what was put in general words; EP-4's brief changed in its context, never its
> completion note. (5) The pictures' neutral root: a registry written into each run's home ahead of
> the run, naming `P:\Projects`, with `P:` a `subst` drive over a scratch folder holding a junction to
> the fake portfolio; both removed afterwards, the junction deleted as a link and never recursively;
> the pictures in `docs/`. (6) The README's checks line brought to rule 6. (7) IN-5 written from the
> owner's prompt and ticked as rolled in. (8) The cross-project memory's note on re-plan timing gains
> the fixtures' trap (the environment's memory folder, not this repository).
>
> No owner tasks; no messages for other projects. **Next: EP-9** (the session runner).
> **After the commit (≈ 40 min in all):** `5f3aa64` and the record commit `83f4a1f` pushed together
> at 00:57 UTC on 2026-11-02; CI run 41369921014 on windows-latest passed in 2 min 29 s (format,
> clippy with and without the screenshot feature, the tests without and with it); the repository
> turned public at 01:00 UTC (`gh repo view` reads PUBLIC, with the description as decided, issues
> on, the wiki off and no topics), and its page and both pictures load without a login. This line is
> a third commit, on the owner's yes, as at EP-5 … EP-7a, and the first push to the public
> repository.
