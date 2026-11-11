---
ep: 20b
title: The prompt hook
size: S
model: claude-opus-5-5
effort: max
status: done
---

# EP-20b — The prompt hook

**Size:** S · **Depends on:** EP-20 · **Blocks:** EP-23

## Context

IN-2 (2026-10-31) asked for a `UserPromptSubmit` hook that recognises `@<project>` in a prompt
and gives the session that project's rules and next-EP row: context from another project on
demand. EP-12 built the relay (`crw hook`, exec form; `--inbox` prints a start's open messages)
and the installer (`signals::install`, whose entries are known by the program they run); EP-13
left the prompt hook to EP-20 (D-20 addendum), and EP-20's split put it here. The state model
already reads a `UserPromptSubmit` hook as busy and as work, so the installed event also makes a
prompted tab busy in the window at once, without waiting for the poll.

**Session start (rule 1).** This session is the chained headless run of EP-20b after EP-20c
(ledger row `9315f5ad`, claude-opus-5-5 at max, the owner's choice, S's cap of 45 minutes of
working time), started by the window's gate at 20:17 after its restart into EP-20c's build; its
prompt is the tool's opener. The field check read: `crash.log` (the 2026-11-03 report of EP-15d's
popup panic, nothing newer); `crw notifications` (the chain's and EP-20c's notes, a manual
session's in another project, nothing odd); `crw sessions` (EP-20c completed at 20:17, this run
open); `crw autopilot` (held after EP-20c for the restart, then EP-20b chained); `crw mail` (the
first readings of 13:53, nothing delivered since). No open inbox message; three open owner tasks
(OT-21, OT-23, OT-24), none gated before this brief.

**Claude Code's contract** (the hooks reference, read 2026-11-10). The input carries the common
fields and `prompt`, the text as typed (a slash command as typed; pasted text expanded in place
between a `<pasted_content id="…">` line and a `</pasted_content id="…">` line). On exit 0 plain
stdout is added to Claude's context as a system reminder, not shown in the transcript. A hook's
plain stdout is capped at 10,000 characters: beyond it Claude Code saves the output to a file and
passes a 2,000-character preview, which Claude is not asked to read. No matcher applies; a
timed-out hook's output is dropped and the prompt goes on. The hook also runs on a `/loop`
iteration, a background subagent's report and a message another session sends.

## In scope

1. **`crw_core::signals::mentions`**: a prompt's `@<project>` mentions. An `@` at the start or
   after a space or an opening bracket or quote, then a name (letters, digits, `-`, `_`, `.`), its
   trailing punctuation dropped (so an e-mail address is none); resolved as the mailman resolves a
   receiver, by the registry's name in any case; each project once, in the prompt's order, at most
   three. Left out: mentions inside code (backtick spans, fenced blocks) and inside pasted content,
   and the session's own project (its folder holds the session's), whose `CLAUDE.md` is loaded
   already.
2. **The context** printed for them: a framing line (the rules are that project's, not this
   session's, whose own `CLAUDE.md` governs; that project is read-only from here, its plan
   changing through its `roadmap/inbox.md`), its `CLAUDE.md` as the launch finds it
   (`prompt::claude_md`), and its next row as the opener writes it (the row line, the title, the
   brief notes verbatim) from the working tree's README, or why there is none. The texts share a
   budget of 9,000 characters, so the whole stays under Claude Code's cap; a cut says where the
   rest is.
3. **`crw hook`**: on a `UserPromptSubmit` of an interactive session (an SDK's `-p` run, whose
   prompt a program wrote, is left out: the tool's own headless runs among them), the registry is
   read only when the prompt holds an `@` (`CRW_HOME`'s in a trial) and the context printed before
   the post. The prompt never leaves the relay: the window gets the kept fields only.
4. **`signals::install`**: `UserPromptSubmit` joins the four events (exec form, `crw.exe hook`,
   5 s); the status words say when only the prompt hook is missing; `crw signals`, `crw doctor`
   and the usage text name it.
5. **The records**: a D-20 addendum, the roadmap's row and notes (EP-23's walk item, EP-24's
   note), the owner task for the install (D-10 addendum), the completion note; the install last
   (rule 6).

## Out of scope

- Writing the owner's user settings: an owner task at the close (D-10 addendum).
- Mentions by folder path (`@..\beta`) or by a folder name that differs from the registry's
  name; a `sessionTitle` from the hook; blocking or changing a prompt.
- The chat tile's own mentions (EP-24), whose sessions are an SDK's.

## Verification / acceptance

- New tests: the mentions (the start of the prompt, after a space or a bracket, an e-mail address
  not, code spans and fenced blocks, pasted content, punctuation, case, a repeat, the limit, the
  session's own project, an unknown name); the context (the `CLAUDE.md` and the row, the cuts
  under the cap, a project without a `CLAUDE.md` or a README, every row checked); the relay (a
  prompt prints the context and posts without the prompt's text; an SDK's prints nothing; no `@`
  reads no registry); the install (five events, the status words, an install over the four
  entries adds one).
- `cargo test`, `cargo test --features screenshot`, both clippy runs and `cargo fmt --check`
  clean.
- A trial of the release `crw.exe hook` with a scratch `CRW_HOME` registry of synthetic projects,
  a `UserPromptSubmit` input piped in, its output read and its time taken.
- Rule 3's word check before each commit; the release built with the window up, then the
  install.

**Residual risk.** The session cannot write the hook into the owner's user settings (OT-25), type
a prompt into a real VS Code or terminal session, or see what Claude Code adds to a real
session's context (rules 5 and 13). It expects them to hold because the entry is the same
exec-form `crw.exe hook` the four installed events run, which the installer's tests write and
read back; the relay's input is the documented `UserPromptSubmit` form, driven by the tests and
by a trial of the release exe over a scratch registry; plain stdout is the channel `--inbox`
already uses for a start, and the reference names both events for it; and the context stays
under Claude Code's 10,000-character cap by construction, tested with five long projects. A real
session's answer to a prompt naming another project, the tab busy in the window at once, and a
headless run given nothing are looks at changed surfaces for the P3 walk (EP-23's note).

> **Completion note (2026-11-10; claude-opus-5-5/max; about 30 min of wall time, no question).**
> The chained headless run of EP-20b after EP-20c (ledger row `9315f5ad`, from 20:17), under S's
> cap of 45 minutes of working time; the window's gate started it after the restart into EP-20c's
> build.
>
> **Built.**
>
> - `crw_core::signals::mentions`: `names` (a prompt's `@` names in order, each once, outside
>   backtick spans, fenced blocks and pasted content, none before `/` or `\`), `context` (the
>   registered projects they name, the session's own left out, at most `LIMIT` = 3 and the rest
>   named; per project a framing line, its `CLAUDE.md` and its next row with the brief notes,
>   cut to share `BUDGET` = 9,000 characters, the `CLAUDE.md` six tenths when both are long, the
>   room one leaves going to the other, a cut ending at a line and naming its file).
> - `signals::prompt_context` and `relay_hook`'s lazy `projects` argument, `signals::registered`
>   (the registry's projects with their folders); the prompt's text read by the relay alone.
> - `install::EVENTS` gains `UserPromptSubmit` (`install::PROMPT`, the same exec-form entry);
>   `Status::lacks_prompt_hook` and the words `hooks installed but not the prompt hook (crw signals
>   install adds it)`, which the window's Signals item and `crw doctor` show.
> - `prompt::row_parts` and `row_lines`: the opener's row, shared with the hook.
> - `crw hook`'s registry closure, `crw signals`' column widened to the event's name, the usage
>   text; the README's hooks paragraph; the D-20 addendum; EP-23's walk item and EP-24's note
>   (the chat's runs are an SDK's); OT-25, the install.
>
> **Verification.**
>
> - **Tests.** `cargo test` passed (350 in `core`, 137 in `app`, the rest in the fixtures and the
>   CLI), and with `--features screenshot` 145 in `app`: EP-20c's counts plus 5 new in `core`
>   (the mentions' parse, the context with its rules, row and gaps, the cuts under the cap with
>   five long projects, the relay printing the context without posting the prompt and an SDK's
>   run or a prompt without `@` asking no registry, an install over the four entries adding one).
>   Both clippy runs and `cargo fmt --check` are clean.
> - **A trial of the release `crw.exe`** with a scratch `CRW_HOME` registry of synthetic projects
>   (its port 47999, so no window heard it): a `UserPromptSubmit` naming `@beta` and `@alpha`
>   (the session's own, once in code) printed beta's framing, `CLAUDE.md` and row; the same input
>   under `sdk-cli` printed nothing; over a copy of this repository's own README and `CLAUDE.md`
>   the context came to 8,946 bytes, cut as planned. Timed against the relay's other hooks with
>   no window listening: a prompt without `@` costs what a `Stop` with the same input costs
>   (100 to 180 ms here, the closed port's wait and the process; D-20 measured 75 ms), and a
>   mention over a real-size roadmap adds at most about 20 ms. `crw signals` over a scratch
>   settings file with the four entries listed `UserPromptSubmit  not installed` and the one
>   change, which `crw signals install` then made after a dated copy.
> - **The release** built with the window up; the install runs last, after the commits.
> - **Rule 3's word check** found nothing in the added lines or the brief.
>
> **Owner decisions.**
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | The install into the user settings | An owner task (D-10 addendum; EP-13's answer: this brief's, on the owner's yes) | OT-25, not gated: no later brief builds on it |
> | 2 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> No question was put: nothing else in the brief is the owner's (no split, no scope change, no
> outward-facing text, no unresolved risk beyond the residual-risk paragraph).
>
> **Routine choices (the session's call, logged).**
>
> 1. **An SDK's session is given nothing**: its prompt is a program's (the tool's own openers
>    quote brief notes, which may name projects), so mentions are read only where the owner types;
>    EP-24's note says how the chat may give its own mentions their context.
> 2. **The session's own project is left out**: its `CLAUDE.md` is loaded and its roadmap is its
>    own to read.
> 3. **No flag on the entry**: the prompt hook is the event's whole purpose, as busy at once is
>    its by-product; `crw signals remove` takes it out with the rest. `--inbox` had a flag because
>    it replaced the owner's own stopgap.
> 4. **The cut, not a pointer alone**: a long `CLAUDE.md` is cut to its share, the cut naming the
>    file, since Claude Code would hand on only a 2,000-character preview of a longer output; the
>    walk weighs the alternatives (EP-23's note).
> 5. **A name before `/` or `\` is a file mention**, which Claude Code reads itself; a folder path
>    or a folder name other than the registry's name is out of scope.
> 6. **The trial's registry named port 47999**, so the relay's posts went to a closed port and
>    never to the owner's window (rule 13).
> 7. **No review subagent**: rule 6 asks one for M and L briefs; the session read its own diff,
>    which found `crw signals`' column too narrow for the new event's name and two subtractions
>    in `context` that are now saturating.
> 8. **EP-20c's front matter** still said `status: in progress` after its completion note; set to
>    `done` in the record commit.
>
> No messages for other projects. **Next: EP-20d**, which a chain may run if the gate passes.
