---
ep: 17
title: Usage meter
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-17 — Usage meter

**Size:** M · **Depends on:** EP-12 · **Blocks:** EP-18, EP-19

## Context

D-23 gives the meter two sources. The first is the undocumented `GET
https://api.anthropic.com/api/oauth/usage`. It has the weekly limit, the Fable-scoped weekly limit,
the five-hour window while one is active and the week's use by surface. It is polled every five
minutes and after each session the tool launches. The token is read from
`~/.claude/.credentials.json` at poll time and never stored. The second is the status-line feed,
which EP-12 built. `crw statusline` posts each session's documented `rate_limits` to the window,
where `watch.model` keeps each session's latest (`Seen::line`). The VS Code extension most likely
never runs the status line (D-23 addendum), so the feed comes from terminal sessions, and the
endpoint is the meter's real source.

When the endpoint breaks, the meter falls back to the status-line figures and says so. It shows a
strip in the status bar and a tile with the reset countdowns, the breakdown by surface and seven
days of samples. It sends notifications at 80 % and 95 % of the weekly or Fable limit. The ledger
has had `usage_sample` since V1 and holds the cache rows of EP-7's GitHub pass, whose pattern the
poll follows: cache first, a failure shown and not stored. The usage notifications join
`crw_core::notify`'s rule table as a kind of their own, delivered through `Toasts::deliver`
(EP-15). The session row's `weekly_before`/`after` and `fable_before`/`after` wait for EP-18.

The user settings deny Claude's own tools a read of `~\.claude\.credentials.json`. The poller
reads that file in the tool's own code, at poll time. So this session never reads the real file.
Its tests use a synthetic credentials file in a temp folder and never reach the endpoint, and the
first real poll is the window's, after the owner's yes.

**Session start (2026-11-04, 21:57 UTC).** This is the first self-launched session (D-5 addendum).
The owner started it from this project's row in the window as a visual launch. Its prompt is the
generated opener ("Run the next brief of clauderoadmapwrangler: EP-17, …", launched from
clauderoadmapwrangler). `crw sessions clauderoadmapwrangler` lists its row: `66483ca0`, EP-17,
visual, open. The tree was clean at `3c25d9e`. No owner task and no inbox message is open. The
model is Opus 5.5 at max effort. The launch suggested max, but the tab opened at the panel's own
effort, xhigh, since the extension's link carries no effort. That was the owner's finding,
recorded by EP-16's third commit (`186a38b`) while this session ran. The owner set the panel to
max before sending the opener. The window runs, so the release build waits for the owner to quit
it from the tray.

**Decided at pickup (owner, through `AskUserQuestion`).** The HTTPS call goes through WinHTTP,
Windows' own HTTP stack. The `windows` crate the lock already pins at 0.62.2 gains one feature,
`Win32_Networking_WinHttp`, and no crate joins the lock. The alternatives were ureq with
native-tls, reqwest with rustls, and `curl.exe` per poll.

## In scope

1. **The token, per poll** (`core::usage`).
   - The file is `%CLAUDE_CONFIG_DIR%\.credentials.json`, else `%USERPROFILE%\.claude\.credentials.json`.
   - The poller reads `claudeAiOauth.accessToken` and `expiresAt` into a `Token`. Its `Debug`
     prints `[redacted]`, its bytes are overwritten when it drops, and it is never written,
     logged or shown.
   - An expired token is not sent; the meter says that Claude Code renews it.
2. **The request.** A WinHTTP GET with `Authorization: Bearer …`, `anthropic-beta:
   oauth-2025-04-20` and the tool's own `User-Agent`. Timeouts cap a poll at about 30 s; the body
   is capped at 1 MB. The poller reads the status and a 429's `Retry-After`.
3. **The reading.**
   - `limits[]`: `kind` weekly_all is the weekly limit; weekly_scoped is named by
     `scope.model.display_name`, and Fable is the scoped limit of that name. Each limit has
     `percent`, `resets_at` and `is_active`.
   - Also `five_hour` (while active), `seven_day` (the weekly figure when `limits[]` has no
     weekly_all), `seven_day_breakdown.rows[]` by surface, and `extra_usage`.
   - Fields the parser does not know are ignored. A `limits[]` kind it does not know is kept by
     its name and shown in the pane. A body it cannot read is a schema change, named with the
     body's shape (keys and types, never the values), which each reading also keeps for `crw
     usage` (added after the first real poll, decision 6).
4. **The schedule.**
   - The first poll comes a few seconds after the window starts, then one every 5 min.
   - Another comes after a session the tool launched ends (its own headless runs, and the visual
     launches the hooks report), at least a minute after the last poll.
   - After a failure the meter retries at 1, 2, 4, 8, 16, then every 30 min, and honours a 429's
     `Retry-After`. A success brings back the 5-minute pace.
   - A poll runs on a worker thread of its own and never blocks the frame loop.
5. **The samples.**
   - Each successful poll writes one `usage_sample` row of source `endpoint`. It also replaces
     the cached reading, a `badge_cache` row (`""`, `usage`), which the window shows at its
     start until its first poll answers.
   - `Watch::receive` writes a row of source `statusline` when a session's status-line figures
     change.
   - Each change of the meter's state (degraded and why, back) is an `event` row of kind
     `usage`. A failure is shown, logged as that change, and never stored as a reading.
6. **The merge and the degraded mode.**
   - While the endpoint's last poll succeeded, its figures stand.
   - Otherwise the strip and the tile show the newest status-line weekly and five-hour figures,
     marked "degraded" with the reason. Fable, which the status line lacks, is the cached
     reading's, with its age.
   - The status line's figures are the tile's second series.
7. **The strip** in the status bar: weekly, Fable and the five-hour window while active. Each is a
   small bar with its percent, amber from 80 % and red from 95 %, in the window's own colours. Its
   hover gives the reset times, the source and the reading's age.
8. **The tile** (the Usage pane, a placeholder since EP-6).
   - Each limit as a bar with its reset countdown and time.
   - The week's use by surface, and extra usage when the reading has it.
   - A 7-day sparkline of the samples, the endpoint's weekly and Fable lines with the status
     line's weekly points. It is drawn with egui's painter: egui_plot would move the egui family
     (D-15).
   - The meter's state: the last poll, the next, a problem.
9. **The notifications.**
   - `notify::Kind::Usage` joins the rule table as a toast, at 80 % and 95 % of the weekly or the
     Fable limit.
   - Each threshold of each limit toasts once per limit window, the window known by its reset
     time; when 95 is already reached, the 80 is not sent. What was sent is kept in the cache,
     so a restart does not repeat it.
   - The toast's Show brings the Usage pane forward.
10. **`crw usage`.** It reads the ledger only, never the credentials or the network: the cached
    reading, the latest samples and the meter's events.
11. **A switch.** `settings.usage_endpoint` (`crw setting usage_endpoint on|off`), on by default.
    Off keeps the meter on the status line alone: an undocumented endpoint gets an off switch
    that needs no rebuild (D-23: treated as breakable).
12. **The screenshot feature.** `usage` sets canned readings and seven days of samples,
    `usage=degraded` an endpoint failure over status-line figures. With `tile=usage` it shows
    the strip and the tile without a network or a credentials file.

## Out of scope

- The downshift rule, the before-and-after attribution and its columns, the usage-share badge:
  EP-18.
- The autopilot's pause at 90 %: EP-19.
- A poll from `crw`. The CLI never reads the credentials, and only the window polls.
- Renewing or writing the token, which is Claude Code's job; transcripts (D-23); ntfy (EP-20).
- Charts beyond the sparkline (EP-26) and egui_plot (D-15).

## Verification / acceptance

- **Unit tests** never touch the real credentials or the endpoint (D-29, the brief notes):
  - the token read from a synthetic credentials file in a temp folder: missing, unreadable,
    expired and redacted;
  - the parser on synthetic bodies shaped like the probe's, a schema change named by its shape;
  - WinHTTP against a plain-HTTP listener on 127.0.0.1 in the test: the headers sent, the status,
    the body and `Retry-After` read;
  - the schedule and its backoff, the alerts once per threshold per window, the merge and the
    degraded mode, the samples in a scratch ledger.
- **Build checks:** `cargo test`, `cargo test --features screenshot`, `cargo clippy --all-targets
  --features screenshot -- -D warnings`, `cargo clippy --all-targets -- -D warnings` and `cargo
  fmt --check`.
- **Screenshot runs** of the strip and the tile with canned readings: light, dark, degraded.
- **The release.** `cargo build --release` once the owner has quit the window from the tray, then
  `scripts\install-shortcut.ps1`.
- **The first real poll**, on the owner's yes: the window polls, and `crw usage` shows an
  `endpoint` sample or names the failure.
- **Rule 3's word check** on the staged diff prints nothing.

> **Completion note (2026-11-04; claude-opus-5-5/max; about 1 h 40 min to the commit, with the
> owner's answers and three restarts of the window among it).** One session in the VS Code
> extension from 21:57 UTC, the first launched from the window (D-5 addendum), its ledger row
> `66483ca0` (visual, EP-17). The reading, then the HTTP question (decision 1), this brief, the
> code and its tests, the screenshot runs, the docs, then the owner's four rounds and the three
> release builds with the real polls between them.
>
> **Built.**
>
> - `crw_core::usage`: the token read per poll into a `Secret` (redacted `Debug`, zeroed on
>   drop); the WinHTTP GET (`usage::http`); the reading, the schedule, the merge (`Gauge`), the
>   alerts, the samples.
> - The ledger: `Sample`, `add_sample`, `samples_since`, `latest_samples` and `cached_one`. No
>   migration: `usage_sample` is V1's.
> - `notify::Kind::Usage` in the rule table. The registry's `usage_endpoint` switch, with `crw
>   setting usage_endpoint`.
> - `app::meter`: the worker thread, the strip in the status bar, the Usage pane and a canned
>   meter for screenshot runs (`usage`, `usage=degraded`).
> - Usage toasts (`Toast::usage`, `Target::Usage`), never merged in a pass. The watch hands its
>   status lines to the meter and flags a visual or headless session's end; the board wires both
>   in.
> - `crw usage`. The README, two addenda (D-14, D-23) and "From EP-17" notes under EP-18, EP-19
>   and EP-26.
>
> **The first real poll** (decision 2), at the window's first start after the build. The reading
> parsed: the weekly and the Fable limits with their reset, the week by surface and extra usage.
> No five-hour figure came. The owner then said that no five-hour limit applies to the account at
> present, and the answer's shape, added for this (decision 6), shows `five_hour` as null and
> `limits[]` holding only `weekly_all` and `weekly_scoped`. A five-hour figure that comes back,
> in its old place or as a new kind, now shows. The first shape was cut by about thirty null
> promotional keys before it reached `limits[]`, so the kinds now come first and the null keys
> last, by name (decision 7). Four `endpoint` samples followed, one per poll, the 5-minute pace
> kept across two restarts. No five-hour or degraded state has yet been seen for real; the
> screenshot runs show both.
>
> **Verification.** `cargo test` 381 passed (89 + 1 + 1 + 1 + 1 + 251 + 2 + 7 + 6 + 11 + 11,
> against EP-16's 357), 383 with `--features screenshot`; clippy with and without the feature
> and `cargo fmt --check` are clean. The tests never read the real credentials or reach the
> endpoint: a synthetic credentials file in a temp folder, and WinHTTP against a plain-HTTP
> listener on 127.0.0.1. `crw usage` was tested on a scratch `CRW_HOME` ledger, which it leaves
> byte for byte as it was. Screenshot runs, dark with the canned reading and light degraded,
> stayed in the session's scratchpad. The release was built three times, each with the window
> quit from the tray, through `scripts\install-shortcut.ps1`. Rule 3's word check found nothing
> in the added lines.
>
> **Owner decisions (`AskUserQuestion`, 2026-11-04).**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The poller's HTTP client, at pickup | WinHTTP via `windows` | One feature of the pinned crate, no crate added; D-14 addendum |
> | 2 | The release build and the first real poll | Quit, build, then poll | The window polled at its start; read through `crw usage` |
> | 3 | The effort this tab ran at | Max, set in the panel | The front matter and this note say max |
> | 4 | `crw usage` and the `usage_endpoint` switch, beyond the brief notes | Keep both | As built |
> | 5 | The addenda, the README and the roadmap notes | Accept as written | Written; decision 6 later added to the D-23 addendum |
> | 6 | The missing five-hour figure | Quit again for the shape; no five-hour limit applies at present | Each reading keeps its shape; an unknown `limits[]` kind shows by its name |
> | 7 | A last rebuild for the shape's order | Quit, rebuild | The release matches the commit |
> | 8 | Commit and push | Commit and push | Two commits in the house pattern, pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. The parser accepts the synonyms the probe's list left open (`percent`, `utilization`,
>    `used_percentage`; `label`, `display_name`, `name`), since no real answer was read before
>    the first poll.
> 2. A failure's words keep an error answer's `error.type` and `message`, cut to 160
>    characters, and never a header. A token is taken as expired a minute before its time.
> 3. A problem on this machine is looked at again each minute, never backed off: no request
>    was sent.
> 4. A status line writes a sample only when its figures change, which keeps the 30-second
>    status lines from filling the ledger.
> 5. The strip's separator shows only while the meter is on, so the screenshots without
>    `usage` are unchanged.
> 6. The sparkline is drawn with egui's painter, with faint lines at 80 %, 95 % and each local
>    midnight. A line breaks at a gap of over 30 minutes.
> 7. `crw usage` has no poll of its own: an unknown argument is refused, and only the window
>    polls.
>
> **Notes for later sessions.**
>
> - The release window now polls the endpoint with the owner's real token, so a session never
>   starts the window outside a screenshot run. A `CRW_HOME` trial of the window polls too: the
>   credentials file follows `CLAUDE_CONFIG_DIR` or the profile, not `CRW_HOME`.
> - `crw usage` reads the owner's ledger safely.
> - EP-16's third commit (`186a38b`), by its own session, recorded this launch while this
>   session ran; this session left its files alone.
>
> No owner tasks; no messages for other projects. **Next: EP-18** (usage coupling).
