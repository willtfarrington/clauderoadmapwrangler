---
ep: 40
title: Field fixes from P4's first chains
size: M
model: claude-fable-5-1
effort: high
status: done
---

# EP-40 — Field fixes from P4's first chains

**Size:** M · **Depends on:** EP-17, EP-18, EP-19, EP-37a · **Blocks:** EP-26a, EP-29 ·
**Decisions:** D-22, D-23, D-29

## Context

The first week of P4's chains met three things the tool did not plan for. (1) Anthropic brought
the account's five-hour limit back: the usage endpoint now carries it as a `limits[]` entry of kind
`session` (the name Claude's own usage page uses) beside the older top-level `five_hour` object,
so the Usage pane drew the one limit twice, each at 0 % with "no reset time given". (2) The
autopilot gate weighs the weekly and the Fable limits only (D-22, 90 %): EP-26a's chained run
(fable at high, started 19:30 on 2026-11-12 after EP-25a and EP-26 in the same five-hour window)
ended failed after 19 minutes on "You've hit your 5-hour limit, resets 11:30pm", the owner reverted
it, and the three Fable briefs of that window had taken about 9 % of the weekly limit together,
so one M brief on Fable takes roughly 30 to 45 % of a five-hour window. The owner's standing
wish since then: Fable for L briefs only, Opus 5.5 at max for everything else. (3) Two installed
windows ran at once this morning: the one the chain's restart started at 19:30 (holding port
47831, so alone hearing the hooks and the status lines, hidden in the tray) and one the owner
started by hand at 11:52, whose status bar said "Signals: not listening (… os error 10048)";
both polled the endpoint, so the usage was attributed twice. EP-28a planned the single-instance
guard; the owner chose to have it now.

**Session start (2026-11-13).** A session by hand on Fable 5.1 at high effort, not from the
window: the owner opened it in this project's VS Code window with a non-roadmap request (the
status-bar line, the Usage pane, the model policy) and asked for questions. `crw sessions
clauderoadmapwrangler` lists its row (`b8fbc18f`, manual, open). Read: the P4 table and the
brief notes, EP-26's completion note, D-22, D-23 and D-29 with their addenda, `owner-tasks.md`
(no open task) and the inbox (no open message). The field check: `crash.log` unchanged since
2026-11-03; `crw notifications` shows EP-26 finished 19:30, EP-26a chained, then failed at
19:49 ("exit code 1") and the chain's end; `crw sessions` shows EP-26a failed on the five-hour
message, EP-26 and EP-25a completed, EP-25 the day before; `crw autopilot` shows six projects
on and the chain log ending with EP-26a's; the registry holds settings only (no `[[rule]]`, no
project rule); `crw usage` prints the answer's shape with the kinds `session, weekly_all,
weekly_scoped`; `crw prompt` resolves EP-26a to fable at xhigh, the tool's default; `netstat`
and `Get-Process` show the two windows. The working tree was clean at the start. Two rounds of
four questions settled the scope (the completion note records them); the owner quit the newer
window during the session.

**The code as found.** `usage::parse` keeps a `limits[]` kind it does not know under its name in
`Reading::other`, which the pane and `crw usage` list, and reads the top-level `five_hour`
into `Reading::five_hour`; `Gauge::figures` shows the five-hour figure only while
`Limit::is_current` (not called inactive, before its reset). `autopilot::gate` weighs
`Facts::weekly` and `Facts::fable` through `weigh` at `settings.autopilot_pause_percent`;
`usage_pause` gives the window's held state. `rules::Resolution::downshift` weighs S briefs
alone. `Server::start` reports a failed bind as `Listening::Failed(words)`, which
`Watch::state_words` turns into the status-bar line; `signals::ask_restart` already probes
`/health` through `relay::request`, and `Board::show` brings the window to the front from the
tray. A restarted window waits for the old one's end before it binds (`install::wait_for_end`).

## In scope

1. **The meter.** `usage::parse` merges a `limits[]` entry of kind `session` into the five-hour
   reading (its percent, reset, `is_active` and severity win over the top-level object's; the
   top-level alone still reads as before), so it no longer lands in `other`. The Usage pane draws
   one row, "Session (five-hour window)": while a window is open, the bar, the percent, the reset
   countdown and "5 h" in the status-bar strip; while none is open (the endpoint gives the limit
   inactive with no reset), one small weak line "Session (five-hour window): no window open", no
   bar, nothing in the strip. `crw usage` prints one line, `session (five-hour)`, with "no window
   open" where the reset was "no reset given". Tests on the new shape and the older ones.
2. **The gate.** A new setting `autopilot_five_hour_percent` (1 to 100, or `default`: 50; `crw
   setting`, validated as the pause percent is). `Facts` gains the five-hour figure and the mark;
   `gate` weighs it as the others, except that no figure passes (a closed window, or an account
   without the limit) and a figure under the mark is never stale by age alone (the window is at
   most five hours long, and its reset lifts the hold). The refusal's words name "the session
   (five-hour) figure"; `usage_pause`, the Autopilot section's "held by usage", `crw autopilot`'s
   first line and the chain log carry it. The launch dialog shows a notice under the model's
   reason while the session figure is at or over the mark. A run whose result says it hit a
   limit ("hit your 5-hour limit", "hit your weekly limit", "usage limit") is named so in the
   ended line (the failed toast, the Sessions pane, the ledger's result line) with the reset
   words the message gives, and the chain log's end line says it too.
3. **The downshift for every size** (D-23's named alternative, the owner's choice): any brief
   whose model so far is Fable is proposed `claude-opus-5-5` at the same effort while Fable's
   weekly share is above the mark; the words say "a brief on Fable" where they said "an S brief".
4. **The single-instance guard** (EP-28a's, pulled forward): a window whose bind fails with the
   address in use asks `GET /health` on the port; when a Roadmap Wrangler window answers, it
   sends `POST /show` (a new route, `signals::SHOW_ROUTE`, which that window takes as its tray's
   Show) and quits at once, before its meter polls or its mailman runs; when something else
   holds the port, or the bind failed otherwise, the status words stay and the bind is retried
   every 30 s. A restarted window is unaffected, since it binds only after the old one ended.
5. **The owner's registry** gains two `[[rule]]` entries on the owner's yes (D-22's hand-written
   table): size S or M → `claude-opus-5-5` at `max`; size L → `fable` at `high`. The built-in
   table (an S brief one effort step lower) then stands aside, as D-22 says.
6. **Records.** Addenda to D-22 (the five-hour gate, the policy as registry rules), D-23 (the
   `session` kind, the downshift for every size) and D-29 (the guard); the README's words on the
   meter, the gate and the start; the P4 row and this note, the totals recounted; EP-28a's note
   marked for the guard; the completion note; the project memory.

## Out of scope

- A predictive five-hour gate (the next brief's expected share from the ledger's attribution,
  which would need the five-hour rise split per session): parked in EP-29's note.
- Resuming a run that hit the limit after the reset: the owner prefers a fresh run (the context
  reloaded after a cut costs quality).
- Notifications at 80 % and 95 % of the five-hour window; a third line in the sparkline.
- The rest of EP-28a: start at login, the start-up reconciliation, `crw` on PATH, the icon.
- A change to the tool's built-in defaults (fable at xhigh): the policy lives in the registry.

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy forms, `cargo fmt --check`.
- A screenshot run of the Usage pane with the canned reading (`usage`, `tile=usage`) showing the
  one session row open, and `usage=closed` showing the quiet line; the strip in both.
- `crw usage` on the owner's ledger prints one `session (five-hour)` line; `crw setting
  autopilot_five_hour_percent` prints the default; `crw autopilot` names the three limits; `crw
  prompt clauderoadmapwrangler` resolves EP-26a (M) to `claude-opus-5-5` at max by rule 1 of the
  registry, and an L row to fable at high by rule 2.
- A review of the diff by a subagent given the diff alone (rule 6), its findings fixed or logged.
- `cargo build --release`, then `pwsh -File scripts\install-shortcut.ps1`, which asks the window
  to restart into the build (the owner's yes at the close).

**Residual risk.** The guard's `/show` path, the retry and the quit are exercised against the
tests' own server and by one real second start after the install (the owner's live step at the
close: start the shortcut twice, expect the first window in front and no second one). Two gaps
the review named stay: a hand-started window left retrying every 30 s (something else held the
port, or a build from before this brief did) could bind in the second or two between a restart's
old window ending and its new window binding, and the restarted window, which never yields,
would then retry forever while the hand-started one held the port without the handover (rare:
after this install every holder answers `/show`, so a hand-started window yields at once); and
the second window's quit comes before its meter's first poll by timing (5 s against a probe of
at most a second), not by a gate. A real five-hour hold is not exercised until a chain meets a
window over 50 %, which the next chains will; the merged session row is checked on the canned
reading and on the owner's live one (closed at the time of writing, since no window was open);
the policy's rules are checked through `crw prompt` only, a chained run under them being the
next chain's.

> **Completion note (2026-11-13; claude-fable-5-1/high; about 2 h 45 min).** Built as scoped,
> in a session by hand. The owner's decisions, each put through `AskUserQuestion`:
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The two windows running at once | Quit the newer one now (recommended) | Done during the session: one window ran at the close, the port holder |
> | 2 | The single-instance guard now or at EP-28a | Build it now (recommended) | Built: `/show`, `Watch::guard_port`, a restarted window never yields |
> | 3 | The Usage pane's session row | One row, a quiet line while closed (recommended) | Built and pictured open and closed (`usage=closed`) |
> | 4 | The gate and the five-hour window | Hold at 50 % (recommended) | `autopilot_five_hour_percent`, default 50; the dialog's notice; the limit-hit words |
> | 5 | Where the model policy lives | Rules in the owner's registry (recommended) | Two `[[rule]]`s written on their word; `crw prompt` resolves EP-40 (M) to Opus 5.5 at max by rule 1 |
> | 6 | Fable's effort for an L brief | high (not the recommended xhigh) | Rule 2 sets `fable` at `high` |
> | 7 | The downshift's reach | Every size (recommended) | Built; the tests changed with it |
> | 8 | How the session is recorded | A new row EP-40 before EP-26a (recommended) | This brief, the row, the note, the totals recounted |
> | 9 | Commit | Commit and push (not the recommended local pair) | Two commits in the house pattern, pushed |
> | 10 | The install | Install and restart now (recommended) | `scripts\install-shortcut.ps1` run last; the window restarted into the build |
> | 11 | The guard's live check | Yes, after the restart (recommended) | See the line under this note |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The merged limit's precedence**: the `session` entry's figures win over the top-level
>    object's, since the entry carries `is_active` and the severity; a reading cached by a
>    build before this one drops the `session` kind it kept under its own name
>    (`Reading::from_json`), so a restarted window's first frame shows one row.
> 2. **The five-hour figure has no age test** in the gate: the weekly figure of the same
>    reading already holds a chain when the meter is stale; a window that has ended passes.
> 3. **The dialog's notice** is set each frame from the meter, like the newer-build line.
> 4. **The limit-hit words** keep the message's own name for the limit and its reset words;
>    the name must be within 40 characters of "hit your" and name a window or the usage (the
>    review's point: "hit your head on the limit" is no limit), and the reset is looked for
>    after it.
> 5. **The guard retries the bind every 30 s** after any failed bind and probes the holder
>    afresh at each retry (the review's finding: the first version never asked again).
> 6. **The CLI's "no window open"** is the session line's alone (the review's point).
> 7. **The review subagent** (launched at the first clean compile, back in ten minutes) made
>    six points, all fixed; its two remaining notes (a hand-started window binding in a
>    restart's gap, the quit before the first poll by timing) are in the residual risk.
> 8. **The scratchpad** is `target\tmp\ep40\` (ignored): the diff, the logs, the pictures, the
>    commit messages.
>
> **Owner task:** none. No message for another project. **Parked for EP-29:** a predictive
> five-hour gate from the ledger's attribution, the first real hold's reading, five-hour
> notifications. **Next: EP-26a** (analytics in the list), chained by the autopilot when its
> gate allows (now under rule 1, Opus 5.5 at max), else from the window.
>
> **The guard's live check (2026-11-13, after the install's restart).** The owner started the
> shortcut a second time while the window ran: the running window came to the front and no
> second one opened (their report); the process list read after it showed the one window, the
> port holder started at the restart. The work commit is `1eac82a`; the record commit follows.
