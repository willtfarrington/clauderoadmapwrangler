# Inbox — messages from other projects

Read right after the roadmap at the start of a session (rule 1). Triage rule (D-13): for each open
message, roll it into this session if it fits the brief and is at most S-sized; otherwise add it to the
brief notes of a named later EP or propose a new row in the right phase, confirming the choice with the
owner through `AskUserQuestion`; or decline with a reason. Write the disposition on the message's line
and tick its box. Until EP-20 delivers messages, the owner carries them here by hand.

- [ ] IN-1 (from the owner, workspace split, 2026-10-31) The owner replaced the one multi-root VS Code
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
  homes: EP-8 for the D-7 and D-16 addenda, EP-11 for the launch itself.
- [ ] IN-2 (from the owner, environment repository EP-0, 2026-10-31) Follow-ups to IN-1 after the owner
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
  EP-12 and EP-21 for the rest.
