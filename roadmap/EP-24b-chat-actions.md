---
ep: EP-24b
title: "Chat, the delegated actions: chips for Adopt, Formalise…, Migrate… and Compose… opening the window's own dialogs, the crw commands the session may run, the chat's own @<project> mentions"
size: M
model: fable
effort: xhigh
status: done
launched_from: clauderoadmapwrangler
launched_on: 2026-11-11
---

# EP-24b — Chat, the delegated actions

**Session start.** Launched from clauderoadmapwrangler on 2026-11-11 by the autopilot, chained
after EP-24a through the window's restart into EP-24a's build (the chain log: "chained EP-24b
after EP-24a: fable at xhigh, the rules' choice, M's cap of 1 h 30 min"); the prompt was the
opener, and `crw sessions clauderoadmapwrangler` lists this run as `e6fbf514`, EP-24b, headless,
fable/xhigh, open. Read first: the roadmap's P4 table (EP-24b the first unchecked row) and the
EP-24 family's brief notes, D-27 with its four addenda, D-20's EP-39 addendum, D-11 and D-13,
EP-24a's completion note, `roadmap/owner-tasks.md` (OT-26 open, ungated) and `roadmap/inbox.md`
(no open message). The field check: `crash.log` is the one of 2026-11-03 (EP-14a), nothing new;
`crw notifications` holds only the chain's own rows (EP-24a finished at 20:30, EP-24b chained at
20:31, the EP-24a question answered); `crw autopilot` shows the chain held after EP-24a for the
restart and this run chained in the new window. Nothing odd to triage.

## Context

EP-24 built the chat's hosted session (one read-only Claude Code session per conversation, run in
the tool's own `chat` folder with every registered project not marked `data = true` joined by
`--add-dir` under `--restricted`, the window refusing every permission request itself) and EP-24a
its replies as Markdown, the conversation list, the pickers and the session's questions answered
in the tile. D-27 delegates every write to the window: the chat may propose an action, and the
owner takes it through a surface of the window's own. This brief adds the three pieces the
family's notes left for last:

1. **The delegated actions.** A reply may propose an action; the tile draws it as a chip under
   the reply, and the chip opens the window's own dialog with the project filled in:
   `Board::open_formalise(project)` (the preview, EP-21a), `Board::open_migration(name)` (the
   launch dialog with the migration prompt, EP-21a), `Board::open_compose(project)` (EP-20c),
   and Adopt for an unadopted folder of the data root (D-7), which here offers the data mark
   first (EP-24's note: a newly adopted data repository must never be readable by mistake).
   Nothing is written before the owner's click in the dialog.
2. **The commands the session may run.** The session gets `Bash` among its tools, narrowed by
   `--allowedTools` to the tool's own `crw` reading commands (`crw roadmap <project>`, `crw tasks
   <project>`, `crw sessions [<project>]`, `crw mail`, `crw notifications`, `crw usage`, `crw
   autopilot`), exact rules per readable project so that a data repository's name or a path
   never matches; any other command's permission request is refused by the window as EP-24
   refuses every request. **The first question at pickup**, which of the writing commands
   (`crw send`, `crw formalise --yes`, `crw adopt`) the session may run itself and which stay
   chips, goes to the owner through `AskUserQuestion`; the recommended answer is none, every
   write a chip (D-27: writes delegated; D-11).
3. **The chat's mentions.** An `@<project>` in the owner's message gets the prompt hook's
   context (`signals::mentions::context`, EP-39's order: the next row and the note's start, the
   rules in short, the paths), since the hook leaves an SDK's session out (EP-20b); the context
   is appended to the digest for that turn, for readable projects only, so a data repository's
   files are never read into the chat.

A chat run that works refuses a restart as a headless run does: `Sessions::stoppable` already
lists a running chat live, which `Board::restart_refusals` names; this brief adds the test.

## In scope

- **`core::chat`.** `TOOLS` gains `Bash`. `chat::Proposed` (Adopt, Formalise, Migrate, Compose
  with the message's words) and `chat::proposals(text)`, which reads a reply's fenced block
  tagged `actions` out of the text (one action per line: `adopt <folder>`, `formalise
  <project>`, `migrate <project>`, `compose <project>: <words>`; an unknown line kept as
  such) and returns the text without it. `chat::allowed_rules(readable)`: the `--allowedTools`
  rules, exact per readable project for the commands that take one, bare for the rest, plus
  whatever writing commands the owner admits (`WRITING`, a constant from the answer).
  `chat::appended(digest, mentions)`: the digest text with a mentioned project's context after
  it. The digest's wording: what the session can run and how, the actions' form, the unadopted
  folders (for Adopt), what stays the owner's. `REFUSAL` says to offer a write as an action.
- **`core::session::Launch`.** `allowed_tools: Vec<String>` → `--allowedTools`, one argument per
  rule, when the help lists the flag, else dropped and said; `path_prepend: Option<PathBuf>`,
  the folder put at the front of the child's `PATH` so that `crw` is found by name (the
  installed copy's folder, the one the window runs beside; the rules name `crw`, never a path).
- **`app::chat`.** Chips under a reply (`horizontal_wrapped`, the window's own buttons):
  Formalise… and Migrate… for a registered project (a chip for an unknown name is drawn disabled
  and says so), Compose… with the words prefilled in the compose dialog (`open_compose_with`),
  Adopt… for an unadopted folder of the data root, which opens an inline confirmation in the
  tile with the data box ("a data repository: the chat never reads it", `data = true` written
  with the adoption) and Adopt / Not now. `Action::ChatAct(Proposed)` and `Action::ChatAdopt {
  folder, data }`; `Board::adopt_folder` shared with `Action::Adopt`. The mentions: `chat_send`
  builds the readable projects as `Registered` and appends `mentions::context` to the digest;
  the header's hover names the mentioned projects. The exchange's Markdown is drawn without the
  actions block. A refused Bash call shows in red as every refusal does.
- **Screenshot flag** `chat=actions`: the second turn's reply ending in an actions block (all
  four forms, one unknown), a `Bash` call of `crw tasks` in the exchange, `Bash` in the canned
  init's tools; pictures at 1600 light and 1200 dark; one run over a scratch copy of a real
  roadmap (rule 5), since the chips open the formalise preview and the compose dialog, which
  draw roadmap content.
- **Tests.** Core: the proposals' parsing (each form, words holding a colon, no block, a block
  in the middle, an unknown line), the rules, the appended text, the launch's `--tools` with
  Bash, its `--allowedTools` arguments and the `PATH` entry, a help without the flag dropping
  the rules and saying so. App: chips drawn from a canned reply and a click queuing the action,
  the disabled chip for an unknown project, the Adopt confirmation's box and its action, the
  compose dialog opened with the words, the mentions' context for a readable project and none
  for a data repository, a running chat among the restart's refusals.
- **Docs.** D-27 addendum (the actions as built, the chip form, the commands, the owner's
  answer), D-20 addendum (the chat's mentions), the README's chat paragraph, OT-26's wording
  (the reply may offer buttons; `crw` reading commands run in the session), the roadmap row's
  hash, the completion note.

## Out of scope

- An owner task added from the chat (no command adds one; `crw tasks` prints them, D-27 addendum
  of EP-23).
- A tool call the host answers as the proposal's channel (an MCP server of the tool's own): the
  session has `--strict-mcp-config` and the fenced block needs no protocol; parked as a v2 note
  only if the line form proves fragile in real use.
- `crw scan` and `crw chats` for the session: the digest lists the unadopted folders itself,
  and the conversations are the pane's.
- A chip from the streaming text: chips are read from a reply once it is whole.
- Any change to the prompt hook, the formalise engine, the compose dialog's write or Adopt's
  own engine beyond the data mark.

## Verification / acceptance

1. `cargo test --workspace` and `cargo test --features screenshot` pass; both clippy runs and
   `cargo fmt --check` are clean; rule 3's word check on the staged diff prints nothing.
2. The core's launch test shows the chat's command line with `--tools
   Read,Glob,Grep,Skill,AskUserQuestion,Bash`, one `--allowedTools` argument per rule (`Bash(crw
   roadmap alpha)`, `Bash(crw tasks alpha)`, `Bash(crw sessions alpha)`, `Bash(crw sessions)`,
   `Bash(crw mail)`, `Bash(crw notifications)`, `Bash(crw usage)`, `Bash(crw autopilot)`), the
   child's `PATH` starting with the bin folder, and no rule for a data repository's name.
3. A canned reply ending in an actions block draws its chips and not the block; clicking
   Formalise… opens the preview on the fixture, Compose… the compose dialog with the words,
   Adopt… the confirmation with the data box; an unknown project's chip is disabled.
4. A message `@alpha what is next?` gives the digest alpha's context (its next row, its rules in
   short); `@beta` (data) gives nothing.
5. `CRW_SCREENSHOT` with `chat=actions` at 1600 and 1200 shows the chips under the reply; a run
   over a copy of a real roadmap (`CRW_SCREENSHOT_ROOT`) shows the formalise preview and the
   compose dialog opened from the chips.
6. The release build and the install run last (`scripts\install-shortcut.ps1`); the restart is
   refused while this run works and taken at the gate's moment.

**Residual risk.** No real chat session runs in this brief (D-29): whether a real session writes
the actions block as the digest asks, whether Claude Code's `--allowedTools` takes each rule as
its own argument and matches `Bash(crw roadmap alpha)` exactly as the session types it (a `2>&1`
or a pipe would miss the rule and be refused, which the pane shows in red), and whether Git
Bash finds `crw` through the child's `PATH`, are OT-26's to see; the help's wording for
`--allowedTools` ("comma or space-separated … e.g. "Bash(git *) Edit"") and the
permission-rules page's exact-match form were read on 2026-11-11, and the digest spells the
commands as the rules match them. The appended system prompt grows by a mention's context
(at most 9,000 characters beside the digest's 12,000) on a command line whose Windows limit is
32,767 characters: with the folders and the rules of a 14-project registry it stays under
25,000, and the launch names the digest's file for the record either way.

> **Completion note (2026-11-11; claude-fable-5-1/xhigh; about 70 min of wall time, a
> headless run under the autopilot, one question answered in the Decisions tile within
> minutes).** Ledger row `e6fbf514`, chained after EP-24a at 20:31 through the window's
> restart into EP-24a's build (an M's cap of 90 min); the prompt was the opener.
>
> **Built.**
>
> 1. **The commands** (D-27 addendum): `chat::TOOLS` gains `Bash`; `chat::allowed_rules`
>    gives `--allowedTools` one exact rule per readable project for `crw roadmap`, `crw
>    tasks` and `crw sessions` (`Bash(crw roadmap alpha)`, a name with a space in double
>    quotes) and bare rules for `crw sessions`, `crw mail`, `crw notifications`, `crw usage`
>    and `crw autopilot`; `WRITING_COMMANDS` is empty on the owner's answer (question 1).
>    `Launch::allowed_tools` goes out as one argument per rule after the flag (the CLI
>    reference's own form, so a rule's spaces are never a separator; the next argument is a
>    flag in every branch), dropped and said by a claude without the flag;
>    `Launch::path_prepend` puts the folder the window runs beside at the front of the
>    child's `PATH`, so the rules and the digest say `crw` and never a path with the owner's
>    user name in it. `REFUSAL` says to offer a write as an action.
> 2. **The actions** (D-27 addendum): `chat::Proposed` and `chat::proposals`, which read a
>    reply's fenced block tagged `actions` (a run of three or more backticks or tildes, closed
>    as CommonMark closes a fence) out of the text, one action per line, a list marker before
>    the verb let go, an unknown line kept as such; the digest asks for the form and lists
>    the unadopted folders. The tile draws each as a chip under the reply (`chips`), enabled
>    when the window has its target (`Known`), its hover saying what the click opens, an
>    unknown line's label cut at 40 characters: Formalise… opens the preview, Migrate… the
>    launch dialog with the migration prompt, Compose… the compose dialog with the words typed
>    (`Board::open_compose_with`), Adopt… an inline confirmation at the exchange's end with the
>    data box, Adopt and Not now (`Adopting`, `adopt_confirmation`); `Action::ChatAct` and
>    `Action::ChatAdopt`, `Board::chat_act` and `chat_adopt`, the list's Adopt handler shared
>    as `Board::adopt_folder`, the data mark written with `set_data` after the adoption and a
>    failure said with the command to run by hand.
> 3. **The mentions** (D-20 addendum): `Board::chat_mentions` builds the readable projects as
>    `Registered` and calls `mentions::context` with the chat folder as cwd; `chat::appended`
>    puts the context after the digest with a line saying whose it is and that an inbox
>    message is a compose action; the names go into `Pending` and reach the header's hover
>    once the start came back.
> 4. **Screenshot flags** `chat=actions` and `chat=adopt`; tests: eleven in the core's chat
>    module (the rules, the proposals in every form, the appended text, the command line's
>    `--tools`, rules and `PATH`, a claude without the flag) and four new in the app's (the
>    chips and the dialogs they open, the Adopt confirmation through to `data = true` in the
>    scratch registry, a mention's context for a readable project and none for a data
>    repository, a running chat among the restart's refusals).
> 5. **Docs**: D-27 and D-20 addenda, the README's chat paragraph, OT-26's wording.
>
> **The pictures** (`target\tmp\ep24b\`, never git): `chat=actions` at 1600 light and 1200
> dark (the `crw tasks quarry` line with the shell's icon, the four chips under the reply
> and the unknown line greyed, the block itself not drawn, the narrow pane not widened);
> `chat=adopt` at both (the confirmation with its box at the exchange's end); and, over a
> copy of this project's own roadmap files (`scripts\copy-roadmap.ps1`, rule 5), the Chat
> pane beside the real row with every chip greyed, since the canned names are not registered
> there, and the compose dialog over it with the copy's next inbox id. Nothing of the copy
> entered git.
>
> **The review (rule 6).** A subagent given the diff alone, launched once the code compiled
> and before the test gate, reported after thirteen minutes: three confirmed findings, four
> plausible and five nits. Fixed: (1, confirmed) an exact rule on a name holding a space
> could never match a command the shell would pass whole: the rule and the digest now spell
> such a name in double quotes (`chat::spelled`); (2, confirmed) a fence of four or more
> backticks never closed, so an actions block after one gave no chips: `fence_of` reads the
> run and closes it as CommonMark does; (3, confirmed) the mentioned names were written into
> the header before the start was known: they travel in `Pending` now; (4, plausible) an
> unknown line drawn whole as a button could widen the pane past its edge (EP-13's trap):
> the label is cut at 40 characters, the line in the hover; (5, plausible) the confirmation's
> fixed room under the exchange would push the footer off a narrow pane once its text
> wrapped: it is drawn at the exchange's end inside the scroll area, the fixed room gone.
> Refuted: a clippy `const_is_empty` warning on the empty writing list, which both `-D
> warnings` runs passed without. Nits taken: a Formalise chip while the preview is open, and
> a Compose chip while a message is being written, now say so instead of doing nothing or
> replacing the text; a list marker before an action's verb is let go; the mention framing
> says an inbox message is a compose action; the D-27 addendum says the bare commands print
> ledger metadata in which a data repository's name and titles appear. Logged, not changed:
> `proposals` re-parses each reply every frame (short strings); `WRITING_COMMANDS`' two
> spellings must move together if the owner ever admits one.
>
> **Verification.** `cargo test --workspace`: 161 passed in `app`, 368 in `core`, the other
> suites as before; with the feature 169 in `app`; both clippy runs and `cargo fmt --check`
> clean, run again after the review's fixes; rule 3's word check on the staged diff printed
> nothing. The release build and the install (`scripts\install-shortcut.ps1`) ran after the
> record commit, the opener's last step; the restart is refused while this run works and
> taken at the gate's moment, or from File > Install and restart, the tray's, or the launch
> dialog's.
>
> **Owner decisions** (asked once the code compiled, answered within minutes while the test
> gate and the review ran).
>
> | # | Decision | Owner's answer | Outcome |
> |---|---|---|---|
> | 1 | Which writing commands the chat's session may run itself: none (every write a chip), `crw send` too, send and `formalise --yes`, or all three | None: every write a chip (recommended) | `WRITING_COMMANDS` empty; D-27 addendum |
> | 2 | Commit | The autopilot's standing yes (D-22) | Two commits in the house pattern, not pushed |
>
> **Routine choices (the session's call, logged).**
>
> 1. **The proposal's channel is a fenced block in the reply**, not a tool call the host
>    answers: the session has no MCP server, and a line form needs no protocol.
> 2. **Exact rules per project name**, not prefix rules, so that a path or a pipe matches
>    none; `crw scan` and `crw chats` are not admitted (the digest lists the unadopted
>    folders itself).
> 3. **`crw` found through the child's `PATH`** rather than named by its full path in the
>    rules, which would carry the owner's user name into the session's command line.
> 4. **The Adopt confirmation lives in the tile**, inline at the exchange's end, rather than
>    a dialog of its own; the data mark is a second registry write after the adoption, its
>    failure said with the command to run by hand.
> 5. **The mention's context goes with the digest**, not into the message, so the exchange
>    shows the owner's words alone; readable projects only.
> 6. **The scratchpad** is `target\tmp\ep24b\` (ignored): the pictures, the logs, the copy
>    and the diff the review read.
>
> **Owner task:** none new; OT-26's wording now covers the commands, the chips and the
> mentions. No message for another project. **Next: EP-25** (history backfill and
> snapshots), chained by the autopilot when its gate allows, else from the window.
