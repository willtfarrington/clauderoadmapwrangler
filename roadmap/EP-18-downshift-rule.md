---
ep: 18
title: Downshift rule
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-18 — Downshift rule

**Size:** S · **Depends on:** EP-10, EP-17 · **Blocks:** EP-18a, EP-19

## Context

D-23's downshift: above a configurable share of the Fable-scoped weekly limit (60 % by default),
the rule resolver proposes Opus 5.5 for S briefs. The launch dialog shows it, and the autopilot
applies it; M and L keep Fable until the pause threshold. The resolver is `rules::resolve`, in
layers (EP-10), each value a `Choice` with its `Source`. The Fable figure is the meter's
(`usage::Gauge::figures(now)`, EP-17): it comes from the endpoint alone, is absent before the first
reading, and stays the last reading's while the endpoint fails.

EP-16's finding: in Visual mode the dialog's model, effort and ultracode pickers change nothing in
the new tab, since the extension's link carries only the prompt. The constant
`rules::DEFAULT_LAUNCH_MODE` still gives EP-14's reason for Visual; D-16's EP-16 addendum gives
today's.

**Session start (2026-11-04, 23:34 UTC).** Started by hand, not from the window (rule 11): the
owner opened a Claude Code tab in this project's VS Code window and typed "run the next EP- brief
for this project". The ledger has the session's row: `b4040dda`, manual, open (`crw sessions
clauderoadmapwrangler`). The owner's reason is habit, and they asked for an owner task that walks
them through the window's launch (OT-14, before EP-18a). The tree was clean at `7818cf3`. No owner
task and no inbox message is open. The model is Opus 5.5, at the panel's effort. The window runs,
so the release build waits for the owner to quit it from the tray.

**Decided at pickup (owner, through `AskUserQuestion`).**

1. **Split** (D-4). EP-18 keeps the downshift, the dialog's Visual wording and the constant's
   comment. The attribution and the usage-share badge become EP-18a, a new S row after it in P3.
   Together they outgrew an S, and with auto-compaction off a long session risks its context.
2. **The Visual pickers** are greyed out and name the panel, like the Permissions row. The
   alternative was pickers left enabled with a plainer line.
3. **The manual start** is recorded as habit, with OT-14 to break it.

## In scope

1. **The setting.** `settings.fable_downshift_percent` is a whole percent from 1 to 100, 60 by
   default (`rules::DEFAULT_DOWNSHIFT_PERCENT`). `crw setting fable_downshift_percent <n>` writes
   it; `default` takes the key out, since a build before EP-18 refuses it.
2. **The layer.** `rules::resolve` takes what the downshift weighs (`rules::Usage`: the Fable
   figure the meter last read, with its time and reset, and the clock). It comes after the
   project's own rules.
   - For an S brief whose model so far is a Fable model, a figure above the setting proposes
     `claude-opus-5-5` (`rules::DOWNSHIFT_MODEL`) at the same effort. Its `Source::Downshift` names
     the figure and its age.
   - A figure above the mark downshifts whatever its age: within a window, the share only grows.
   - For the same briefs the judgement is kept in words when the rule does not apply: no reading
     yet, the window reset since the reading, or under the mark (and older than 15 minutes, so it
     may be higher now).
   - M and L briefs keep Fable (D-23). The owner's choice in the dialog stands over every layer.
3. **The callers.** `prompt::draft` passes the figure on. The window gives its meter's figures;
   `crw prompt` reads the ledger's cached reading, read only, and never the credentials or the
   network; the tests give none or a synthetic one.
4. **The dialog.** In Visual mode, Model, Effort and Ultracode are greyed out with "headless only:
   choose it in the new tab's panel", the rules' reason kept as the suggestion the opener names.
   Headless enables them. Under the model, the downshift's judgement shows in a weak line when the
   rule weighed the brief and did not apply.
5. **`crw prompt`** prints a `downshift` line.
6. **The constant's comment** (`rules::DEFAULT_LAUNCH_MODE`) gives the reasons of D-16's EP-16
   addendum.
7. **The split recorded.** EP-18's row is retitled, and EP-18a gets a row and brief notes: the
   attribution half of EP-18's note and what this session found. EP-23's depends-on and the totals
   follow. "From EP-18" notes go under EP-18a and EP-19, with D-23 and D-16 addenda.
8. **OT-14**: the walkthrough of the window's launch, for EP-18a (the owner's ask).

## Out of scope

- The attribution, the usage-share badge, its list column, its Detail line and its sort key:
  EP-18a.
- The autopilot's use of the downshift (EP-19 passes the gauge's figure to `prompt::draft`) and its
  pause at 90 %.
- A visual tab's model and effort: the extension's link takes only `prompt` and `session`.
- A downshift for M or L briefs, to a model other than Opus 5.5, or a rule's opt-out.

## Verification / acceptance

- **Unit tests**:
  - the layer: above, at and under the mark, stale, reset, missing, an M brief, a non-Fable model,
    a project rule's Fable, the setting's value;
  - the setting's parse and write;
  - the dialog's words in Visual mode, and `draft` with a figure.
- **Build checks:** `cargo test`, `cargo test --features screenshot`, `cargo clippy --all-targets
  --features screenshot -- -D warnings`, `cargo clippy --all-targets -- -D warnings` and `cargo
  fmt --check`.
- **Screenshot runs** in both themes:
  - `launch=lantern,usage`: an S brief, Visual, Fable at 82 %, so the downshift and the greyed
    pickers;
  - `launch=harbor,usage`: an L brief, Headless, the pickers enabled and no downshift.
- **`crw prompt clauderoadmapwrangler`** on the real registry prints the downshift line from the
  cached reading (read only).
- **The release.** `cargo build --release` once the owner has quit the window from the tray, then
  `scripts\install-shortcut.ps1`.
- **Rule 3's word check** on the staged diff prints nothing.

> **Completion note (2026-11-05; claude-opus-5-5/max; about 50 min to the commit, with the
> owner's two rounds and one quit of the window).** One session in the VS Code extension from
> 23:34 UTC on 2026-11-04, started by hand (ledger row `b4040dda`, manual; the owner's reason is
> habit, and OT-14 is there to break it). The reading, the pickup round, this brief, the code and
> its tests, the screenshot runs, then a check of `crw prompt` on the real registry that found the
> endpoint's `is_active` flaw. Then the fix, the docs, the closing round and the release.
>
> **Built.**
>
> - `rules`: the Fable downshift as the resolver's last layer (`Usage`, `Fable`, `Downshift`,
>   `Source::Downshift`, `DOWNSHIFT_MODEL`, `DEFAULT_DOWNSHIFT_PERCENT`, `is_fable`).
>   `Resolution::downshift` keeps the judgement in words. `DEFAULT_LAUNCH_MODE`'s comment gives
>   the reasons of D-16's EP-16 addendum.
> - The registry's `fable_downshift_percent`: a whole percent from 1 to 100, checked at load, set
>   with `crw setting`, and taken out by `default`.
> - `prompt::draft` takes the usage. The window passes its meter's figures. `crw prompt` reads the
>   cached reading from the ledger, read only, and prints a `downshift` line.
> - The launch dialog: in Visual, Model, Effort and Ultracode are greyed out with "headless only:
>   choose it in the new tab's panel; suggested by …", and the downshift's judgement shows under
>   the model.
> - `usage::Limit::before_reset`: the alerts and the downshift count a figure until its reset.
>   `is_current`, which also reads `is_active`, is kept for the five-hour figure.
> - The split: EP-18a's row and brief notes, EP-23's depends-on and the totals, From-EP-18 notes
>   under EP-19 and EP-23, the D-23 and D-16 addenda, the README, and OT-14.
>
> **Found at verification.** `crw prompt clauderoadmapwrangler` on the real registry first said
> that Fable's window "has ended", though its reset was three days away. The endpoint's
> `is_active` is `true` on the weekly limit and `false` on the Fable-scoped one while both windows
> run. That was read from the cache row's `active`, `severity` and reset fields alone, at
> 2026-11-05 00:01 UTC. `Limit::is_current` took `false` as ended, so EP-17's notifications would
> never have sent Fable's 80 % and 95 % marks. Both now go by the reset alone (decision 5). After
> the fix the line read "Fable at 19 %, read 3 min ago: under the downshift's 60 %".
>
> **Verification.** `cargo test` 385 passed (90 + 1 + 1 + 1 + 1 + 254 + 2 + 7 + 6 + 11 + 11,
> against EP-17's 381), 387 with `--features screenshot`; both clippy runs and `cargo fmt
> --check` are clean. Screenshot runs, kept in the session's scratchpad:
>
> - lantern's S brief in Visual, dark, with the canned meter's Fable at 82 %: Opus 5.5 suggested,
>   the pickers greyed out, the opener naming `claude-opus-5-5 at high effort`;
> - harbor's L brief, Headless, light: the pickers enabled, no downshift line;
> - meadow's S brief in Visual, light, with no meter: "no downshift: the meter has no Fable figure
>   yet".
>
> The release was built with the window quit from the tray, through
> `scripts\install-shortcut.ps1`. Rule 3's word check found nothing in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-04 and 2026-11-05).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | EP-18's scope, at pickup | Split: the downshift now, the attribution as EP-18a | EP-18a's row and notes; 42 briefs |
> | 2 | The Visual pickers, at pickup | Grey them out, name the panel | As built; D-16 addendum |
> | 3 | Why this session started by hand | Habit; an OT walkthrough to break it | The session-start paragraph; OT-14 before EP-18a |
> | 4 | The release build | Quit the window, build and install | Built and installed |
> | 5 | The notifications' `is_active` fix, beyond the brief | Keep it in EP-18 | `Limit::before_reset`; D-23 addendum |
> | 6 | The addenda, the README, the roadmap notes and OT-14 | Accept as written | Written |
> | 7 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. The downshift is the last layer and weighs only a Fable model (the alias, or an id that holds
>    "fable"), so a rule that names Fable is downshifted too. A figure strictly above the mark
>    applies.
> 2. A figure above the mark counts whatever its age, since a window's share only grows. One under
>    it and older than 15 minutes (`usage::FRESH_FOR`) is said to be possibly higher now.
> 3. The mark is a whole percent (`u8`). `crw setting` takes `75` or `75 %`, and `default` takes
>    the key out, as `usage_endpoint on` does.
> 4. `crw setting`'s name column is 23 characters wide, for the new key.
> 5. In Visual the model's reason keeps the downshift's judgement on a second line, and a choice
>    made in Headless reads "suggested by your choice in Headless". The reasons were shortened
>    after the first picture ran them to four lines.
> 6. The `is_active` check opened the ledger read only, from a script in the scratchpad, and read
>    the cache row's `active`, `severity` and reset fields, never its figures.
>
> **Notes for later sessions.**
>
> - EP-18a's first question rests on the four facts in its brief notes: a visual row's late end,
>   parallel sessions, coarse figures, and manual sessions under D-23.
> - The endpoint calls the Fable-scoped limit inactive while its window runs. Nothing should read
>   `is_active` as a window's state; the five-hour figure still does, unseen on this account.
> - EP-17's visual row was still open hours after its commit, most likely because its tab was.
>
> No messages for other projects. Owner task: OT-14, before EP-18a. **Next: EP-18a** (usage
> attribution), launched from the window.
