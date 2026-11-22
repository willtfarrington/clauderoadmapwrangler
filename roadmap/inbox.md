# Inbox — messages from other projects

Read right after the roadmap at the start of a session (rule 1). Triage rule (D-13): for each open
message, roll it into this session if it fits the brief and is at most S-sized; otherwise add it to the
brief notes of a named later EP or propose a new row in the right phase, confirming the choice with the
owner through `AskUserQuestion`; or decline with a reason. Write the disposition on the message's line
and tick its box. From EP-20 the tool delivers the other projects' `Handed on (to …)` blocks here
(D-13); the owner may still write a message by hand.

- [x] IN-1 (from the owner, workspace split, 2026-10-31) The owner replaced the one multi-root VS Code
  workspace over the data root with **one single-folder `<name>.code-workspace` per project, beside the
  project folders** (the one non-project folder got a file too). Claude Code sessions start only from
  those files, so cwd, transcripts and auto-memory belong to the project they work on, which is what
  D-16 asked for; the aggregate file stays for manual browsing and is still what the registry's
  `workspace_file` names. Consequences for this project: (1) EP-11's Visual launch should open the
  existing `<data root>\<name>.code-workspace` instead of generating one (D-16 addendum). (2) Adopt (EP-2,
  D-7) still appends to the aggregate file; it should also create `<name>.code-workspace` from the same
  template, and the first-run default "exactly one `.code-workspace` in the data root" (`only_workspace_file`)
  no longer holds, so a fresh registry needs another rule for `workspace_file`. (3) `crw scan`'s workspace
  listing should read the per-project files as the listing, not only the aggregate. (4) Sessions of this
  project now reach the other roadmaps through `permissions.additionalDirectories` in the untracked
  `.claude\settings.local.json` (the data root; rule 4 keeps them read-only); the owner's user-level git
  ignore already excludes that file. (5) This project's own earlier transcripts (EP-0 … EP-3) were
  re-filed under its own slug on 2026-10-31, so a re-plan's timing pass finds them there. Suggested
  homes: EP-8 for the D-7 and D-16 addenda, EP-11 for the launch itself. **Triaged at EP-4
  (2026-10-31):** none of it S-sized for a parser brief; (1) parked in EP-11's brief notes; (2) and
  (3) parked in EP-8's brief notes with the D-7 and D-16 addenda, the re-plan deciding whether they
  are an S row in P2; (4) and (5) recorded in EP-4's brief as the session-start check (the four
  transcripts found under this project's slug), with rule 11's wording put to the owner at EP-4's
  close.
- [x] IN-2 (from the owner, environment repository EP-0, 2026-10-31) Follow-ups to IN-1 after the owner
  moved more of the environment to the machine level. (1) Neighbour folders: the owner now grants the
  non-data projects to every session through user-level `permissions.additionalDirectories` and keeps a
  data relationship in the depending project's own `.claude\settings.json`. Claude Code's documentation
  says a settings-file grant gives file access only, while a folder passed at launch with `--add-dir`
  also loads its `CLAUDE.md` and `.claude/rules` when `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1`
  is set. So the registry should hold each project's neighbour list and the headless launcher pass it
  as `--add-dir` with that variable (D-16 "no extra directories" and D-18 revisited, an addendum each).
  (2) Hooks: the owner runs a stopgap user-level `SessionStart` hook
  (`~/.claude/hooks/session-start-inbox.ps1`) that prints the project's open inbox lines into the
  session; EP-12's hook installation replaces it (remove it or chain it). A `UserPromptSubmit` hook that
  recognises `@<project>` in a prompt and injects that project's rules and next-EP row would give
  on-demand cross-project context (EP-12 or EP-20 scope; the owner's wish). (3) Every active project
  now has `roadmap/inbox.md` in the D-13 form, hand-carried, so EP-21's formalise step creates the file
  only where it is missing and adds the triage line to rule 1. (4) The owner's environment repository
  (local-only; environment decisions and the shared memory) exists under the data root; the owner
  adopts it with `crw adopt` (rule 13: the registry is theirs). Suggested homes: EP-8 for the addenda,
  EP-12 and EP-21 for the rest. **Triaged at EP-4 (2026-10-31):** (1) parked in EP-8's brief notes
  (the D-16 and D-18 addenda; the registry's neighbour list and the `--add-dir` flag land where the
  re-plan puts them in P2, with pickup notes for EP-9 and EP-10); (2) parked in EP-12's brief notes;
  (3) and (4) parked in EP-21's brief notes.
- [x] IN-3 (from the owner, crw on PATH, 2026-11-01) Put `crw` on PATH, which belongs to EP-28's
  install script (`install-shortcut.ps1` final). A shim in `%USERPROFILE%\.local\bin` next to
  `claude.exe` is the natural form: that folder is already on PATH, so `crw scan`, `crw tasks` and
  the rest would run from any shell without the `target\release\` prefix. **Triaged at EP-6
  (2026-11-01):** not dashboard work and not S-sized for EP-6; parked in EP-28's brief notes, where
  the script picks the shim's form (a `crw.cmd` forwarding to the release exe, or a copy refreshed at
  each run) and Diagnostics checks that it reaches the current exe.
- [x] IN-4 (from the owner, list and detail polish, 2026-11-01) Four notes after checking OT-2. (1)
  Even with the window very wide the badge spacing is cramped (the stack chip is cut off on the right
  for slightly longer abbreviations), and in general the way the Projects pane's columns share width
  is clunky. (2) The badges' semantic span is good, but there should be a button that toggles a
  key/legend explaining what everything means, even though hovering explains each badge. (3) The
  Detail pane's Status section spells license two ways (also licence), at least for this project.
  (4) The Status section's subsections (Stack, Activity, …) run from one to four lines each, so a
  subsection's name, centred against its lines, is hard to follow: at minimum theme-appropriate
  dividers, line breaks or partial borders between subsections, or else more padding and spacing that
  stays consistent across window sizes (as with the badges' widths). Anything already slated can be
  disregarded; the rest goes into still-pending parts of the roadmap. **Triaged at EP-7's close
  (2026-11-01):** none of it was slated; all four became a new M row, EP-7a, in phase P1 before EP-8
  (so the README's pictures and the public flip show the polish), with its brief notes from the four
  points; the owner chose columns sized to their content under a header row, a Legend toggle in the
  Projects pane's toolbar with the key in the pane, and the spelling license; drag-resizable columns
  went to the v2 notes.
- [x] IN-5 (from the owner, a session before the workspace split, 2026-10-31) EP-8's pre-flip audit
  should include `roadmap/inbox.md`: IN-1 and IN-2 were kept generic, but the audit is the place to
  confirm it before the repository goes public. (Carried in the owner's prompt at EP-8.) **Triaged
  at EP-8 (2026-11-01):** rolled in, as it fits the re-plan's audit and is S-sized; the audit read
  IN-1 … IN-4 and found no project's name, no path beyond the home-relative ones and no machine
  detail.
- [x] IN-6 (from the owner, environment repository EP-1, 2026-11-01) The untracked
  `.claude\settings.local.json` grants the whole data root, and a local settings file needs no
  workspace trust, so the grant is live: this project's sessions have the data repositories among
  their working directories, which the environment's rule for neighbour folders keeps out (a data
  repository is reached only as a session's own folder or through a relationship declared by the
  project that depends on it). Since IN-2 the user-level grant covers every other project, so the
  entry adds only the data repositories and the loose files at the data root: drop it or narrow it,
  and bring rule 11's sentence along. Auto mode admits reads anywhere, so the gap shows in Manual
  mode and for edits. Suggested home: EP-9, the neighbour-folder brief. **Triaged at EP-9
  (2026-11-02):** rolled in, as it fits the neighbour-folder brief and is S-sized; on the owner's
  choice the entry was dropped (the file held nothing else and is gone) and rule 11 now names the
  user-level grant, which leaves out the data repositories.
- [x] IN-7 (from the owner, OT-18, 2026-11-06) Opting a project into a chain takes a hand edit of
  the registry today: the gate refuses a project whose runs would wait at each shell command
  (`Waits`), and the only way past it without allow rules is `permission_mode = "auto"` written
  under the project's `[project.launch]`, which no window control and no `crw` command sets (so a
  session can make the change only by editing the owner's registry by hand, rule 13). Make it GUI
  friendly: (1) ticking the autopilot box (or A) on a project whose runs would wait says so and
  offers to set its permission mode to `auto` there, with a line on what `auto` means; (2) beside
  the launch dialog's Permissions picker, a "keep for this project" choice that writes `[project.launch]
  permission_mode`, or takes it out to return to the default; (3) the Autopilot section, or the
  Detail pane, shows each autopilot project's permission mode and whether the gate would refuse it
  for waiting; (4) `crw set <project> permission_mode <mode|default>` as the command-line twin.
  Perhaps the autopilot's limits too (`max_size`, `daily`, `max_budget_usd`), which are also
  hand-written today. Suggested home: EP-19a, whose launch dialog already serves autopilot
  projects; a row of its own if it does not fit an S. **Triaged at EP-19a (2026-11-06):** more
  than an S with the limits and the owner's wish of the same day for a standing model and effort
  per project or phase, so, on the owner's choice, a new M row EP-19b right after EP-19a takes
  all four parts, the three limits and "keep for this project" (or phase) beside the dialog's
  pickers; EP-19a stays on the chain plan and the chain's moment.
- [x] IN-8 (from clauderoadmapwrangler EP-20d, 2026-11-10) A test message from EP-20d, at the
  owner's request, for OT-21's check of the mailman's delivery between projects: nothing to build;
  decline it as a test and tick it. **Triaged at EP-21a (2026-11-10):** declined as the test it
  is: the window's mailman delivered it at 23:03, once EP-20d's brief was committed (`crw mail`:
  `delivered EP-20d:… to clauderoadmapwrangler as IN-8`), which was the check.
- [x] IN-9 (from the owner, 2026-11-10) OT-24 is done (the first real push reached the phone, and
  crw notifications listed ntfy · test). Two findings for the README's ntfy note and OT-24's
  wording: (1) when the ntfy server runs on the window's own PC, ntfy_host is best its loopback
  address (http://127.0.0.1:<port>), since a VPN on the PC can leave the server's tailnet name
  unreachable from the PC itself, while the phone uses the tailnet name; (2) ntfy's iOS app needs a
  self-hosted server with upstream-base-url set to ntfy.sh and a base-url equal, character for
  character, to the server URL typed into the app (both sides hash it), and it fetches each
  message's text over the tailnet, so while another VPN replaces the tailnet on the phone the banner
  says only "New message" and the text waits in the server's cache until the app is next opened.
  **Triaged at EP-21a (2026-11-10):** rolled in on the owner's choice, though outside the brief's
  subject (a few README sentences): the README's ntfy paragraph says both points in generic
  words; OT-24, ticked, keeps its wording.
- [x] IN-10 (from the owner, 2026-11-11) A test from the P3 walk: decline it. **Triaged at EP-23
  (2026-11-11):** declined as the test it is: the owner wrote it from the window's Compose…
  dialog while this session worked here, the first real compose (`crw mail`: `written IN-10 from
  the owner` at 12:40), which was the walk's check.
- [x] IN-11 (from clauderoadmapwrangler EP-23, 2026-11-11) A test letter from EP-23's P3 walk, at
  the owner's request, for the first real urgent popup and its push: nothing to build; decline it as
  a test and tick it, saying what `crw notifications` showed for it. [urgent] **Triaged at EP-38
  (2026-11-11):** declined as the test it is. The window's mailman delivered it once EP-23 was
  committed and its tab idle (`crw mail`: `delivered EP-23:3cee3099e762a81f to
  clauderoadmapwrangler as IN-11` at 13:48:01), and `crw notifications` showed at 13:48:02 the
  `badge · inbox` row, the `popup · urgent` row ("clauderoadmapwrangler: IN-11 is urgent", the
  letter's first words as its gist) and the `ntfy · urgent` row: the first real urgent letter
  reached the popup and the phone.
- [x] IN-12 (from the owner, OT-26, 2026-11-11) Three findings from the first real chat. OT-26 is
  done: the data marks, the digest, the confinement of Read and Glob to the given folders (both
  refused outside them, the data repositories absent from the list), the `crw` commands, the chips
  and a mention all held. (1) For a project without `roadmap/owner-tasks.md` the digest says "N
  open owner tasks" where it counts the older forms, a risks list among them, so the chat's first
  answer named the project with the most "tasks" when they were tracked risks; a later reply,
  having run `crw tasks`, said so itself. The digest should name them as items in older forms, as
  `crw tasks` does. (2) The digest does not say that Compose… creates a missing
  `roadmap/inbox.md`, so the chat told the owner to formalise a project before writing to it.
  (3) The chat is launched with `--permission-mode manual`, yet its transcript records
  `permissionMode: "default"` on every turn; most likely the CLI's internal name for the same
  mode, worth a line in D-27 once checked. **Triaged at EP-25 (2026-11-11):** rolled in on the
  owner's choice, though outside the brief's subject, the three together being S-sized: (1) the
  digest says "N open items (no owner-tasks.md: items in older forms, a risks list among them,
  as crw tasks lists them)" for such a project; (2) the digest says that Compose creates a
  missing `roadmap/inbox.md`, so a project need not be formalised first; (3) checked against
  `claude --help` (choices acceptEdits, auto, bypassPermissions, manual, dontAsk, plan): the
  transcript's `default` is the CLI's own name for `manual`, a line in D-27.
- [x] IN-13 (from the owner, 2026-11-11) I am deferring decision on position for placement within
  the pre-existing roadmap on this minor issue: quicker or automated refresh of individual project's
  status/state after completion of a session or session chain (to update next session, owner tasks,
  etc); currently it has to recheck every project via a manual hit of the refresh button
  **Triaged at EP-25 (2026-11-11):** outside a history brief and about an S of window work; on
  the owner's choice a new S row, EP-25a, right after EP-25 in P4: a project's files and git
  re-read on their own when its session or its chain's run ends, and from its row's menu.
- [x] IN-14 (from the owner, 2026-11-15) A registry key written during a run is unreadable to the
  window that started the run, and that window's gate reads the registry first. Seen after EP-42:
  the session set `private_words` once its install's restart had been refused (the run still
  worked), and when the run ended the window, on EP-41's build, logged "The autopilot could not
  read the registry: … unknown field `private_words`" from `weigh_run`
  (`crates/app/src/autopilot.rs`), which returns before `read_gate`. So nothing after it ran: no
  chain-log line (the chain stopped without one), the run-ended re-read said the project was "not
  read again: the registry does not list it", and, had the chain gone on, the restart a held chain
  takes at the gate's moment (EP-37a) would have been lost too, the one restart that brings in the
  build that knows the key. File > Install and restart by hand is the way out, as EP-42's
  completion note foresaw. The held chains' re-read (`let Ok(loaded) = … else { return; }`) and
  `start_queued` fail the same way, the first silently. Only a later build can soften this, for
  keys added after it: when the gate's registry read fails while a newer installed build waits,
  take the restart anyway and let the new window weigh the gate with the chain carried over; or at
  least write the chain log's line and a notice naming File > Install and restart. A convention
  would also do: a brief that adds a registry key leaves setting it to an owner task once the
  window runs the new build (EP-42's recommended option). Low priority, about an S. Suggested home:
  a brief that touches the gate or the install, else EP-32's v2 notes. **Triaged at EP-30
  (2026-11-16):** about an S and outside that brief, which does not touch the gate; on the owner's
  choice parked in EP-30b's brief notes, since EP-30b builds the phone's Start from the gate's two
  halves (`Board::gated`, `Board::start_queued`). EP-30's field check saw one more effect: the
  chain plan outlived its chain (the owner's restart at 23:15 carried it), since the gate that
  ends a chain never ran. EP-30 adds `owner_login` and leaves it unset in the owner's registry,
  so the trap does not recur there. **Done at EP-30b (2026-11-16):** the chain is held with
  words naming the failure and File > Install and restart, said once, and a newer installed
  build is taken at once with the chain carried (D-22 addendum).
- [x] IN-15 (from a private project EP-19, 2026-11-18) The `replan` skill's retro rule (step 4:
  "gaps over five minutes are waits on the owner") misread one gap at that project's EP-19
  (2026-11-18): EP-15's session had a 7.5-minute gap a minute in, between an `api-request` entry and
  an `assistant` entry carrying a thinking block, the model's own reasoning at max effort, while its
  one `AskUserQuestion` was answered within a minute (counted as a wait, 33 active minutes would
  have read 26). Suggest counting a gap as a wait only when the entry after it is a `user` entry
  whose `tool_result` carries an `AskUserQuestion` call's `tool_use_id`, or a new prompt from the
  owner, and reporting other long gaps as session time; entry types and tool ids are metadata, so
  the rule still reads no conversation text. The owner's shared memory note
  `replan-timing-from-transcripts.md` carries the same refinement since 2026-11-18.
  **Triaged at EP-45 (2026-11-22):** rolled in, as it fits a re-plan brief and is S-sized: the
  `replan` skill's step 4 now counts a gap as the owner's wait only when an answered
  `AskUserQuestion` or a new prompt ends it, any other long gap being the session's own time;
  EP-53, which times every project's briefs, uses the rule. The line named its sender and
  another of the owner's repositories, both on the private word list of this public
  repository: reworded on the owner's choice before the commit, and EP-51's Inbox pane flags
  such a message on arrival.
