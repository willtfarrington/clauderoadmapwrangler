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
