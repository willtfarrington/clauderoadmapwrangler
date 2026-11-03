# Owner tasks — what only the owner can do

One line per task (D-10): `- [ ] OT-n (EP-k, date) text`, optionally `before EP-m`. Sessions append
at completion; the owner ticks in the tool (the Detail pane, from EP-6) or here.

- [x] OT-1 (EP-6, 2026-11-01) Open Roadmap Wrangler from its shortcut once and set it beside
  `crw scan`: every registered project in its state group with the next EP `crw roadmap` names, and
  the status bar's "Asking git" counting through the repositories to "Up to date" (about 15 s). The
  session could check the window only on the fake portfolio (rule 5) [before EP-7]
- [x] OT-2 (EP-7, 2026-11-01) Open Roadmap Wrangler once and look at its badges: the first start
  creates the ledger (`%LOCALAPPDATA%\clauderoadmapwrangler\ledger.sqlite`, the GitHub cache) and
  the status bar counts "Asking GitHub: n of 9" after git; the row's badges and the Detail pane's
  Status section should agree with `target\release\crw.exe status`, the backup badge now reading
  the job through the two settings set at EP-7's close. Widen the window to see the full badge form.
  The session checked the window only on the fake portfolio (rule 5)
- [x] OT-3 (EP-7a, 2026-11-01) Open Roadmap Wrangler on the real portfolio and look at EP-7a's
  polish: each column as wide as its widest value under a header row (no chip clipped), the badges
  loosening as the window widens and the optional columns giving way as it narrows, the Legend
  button in the Projects toolbar opening the key at the foot of the pane, and the Detail pane's
  Status section with each name level with its first line and dividers between the subsections.
  Anything to adjust goes to `roadmap/inbox.md` before EP-8 takes the README's pictures; the gaps'
  share of a wide window is one constant (`list.rs`). The session checked only the fake portfolio
  (rule 5) [before EP-8]
- [x] OT-4 (EP-10, 2026-11-02) Open Roadmap Wrangler, select a project and press S (or its row's
  Start button): the launch dialog should show the model, effort, permission mode and prompt that
  `target\release\crw.exe prompt <project>` prints, and its Claude line the installed CLI, logged
  in. A first real headless run is optional and spends usage: Start headless, then watch the
  Sessions pane; a request waits in the Decisions pane until EP-14, Stop ends the session, and the
  ledger keeps its row. Notes for the dialog go to `roadmap/inbox.md`. The session checked the
  dialog only on the fake portfolio and started no real session (rule 5) [before EP-11]
- [x] OT-5 (EP-11, 2026-11-02) Try a Visual launch from Roadmap Wrangler on a project whose VS
  Code window is closed: select it and press S; the dialog opens on Visual, its VS Code line
  saying "no window has it open: one opens"; press Open in VS Code. The project's window should
  open from its own `<name>.code-workspace` and, once the Claude Code extension is up (within a
  minute), a new Claude Code tab should hold the prompt, unsent: choose the model and effort in
  its panel, then send it or close the tab. If VS Code asks "Allow Claude Code extension to open
  this URI?", tick "Do not ask me again" and Open. The session tried only a window already open,
  its own, with a test prompt (rule 5). Notes go to `roadmap/inbox.md` [before EP-12]
- [x] OT-6 (EP-12, 2026-11-02) Open Roadmap Wrangler from its shortcut and hover over "Up to date"
  in the status bar: it should end "Signals: listening on 127.0.0.1:47831; hooks installed, status
  line installed". Then open a new Claude Code tab in a registered project's VS Code window: within
  a few seconds the Sessions pane should list it under "Started elsewhere" as manual and idle, the
  project's row should show a session in progress, and a project with open inbox messages should
  have them printed into the new session as the stopgap hook did. Send a prompt (busy within 5 s,
  from the poll); a permission prompt should show it waiting, in amber; closing the tab ends it
  (done, gone from the pane after ten minutes). The session checked the window only on the fake
  portfolio and the relays on a test port (rule 5). The stopgap script
  `~\.claude\hooks\session-start-inbox.ps1` is no longer run (retiring it is your environment
  repository's next brief); `target\release\crw.exe signals remove` takes the tool's entries out.
  Notes go to `roadmap/inbox.md` [before EP-13]
- [ ] OT-7 (EP-13, 2026-11-02) Open Roadmap Wrangler from its shortcut: its first start brings the
  ledger to schema 3 and closes the two visual launches from before the hooks, which
  `target\release\crw.exe sessions` then lists as lapsed (before the window, it already reads them
  so, without writing). In the Sessions pane's All tab, try the project, mode and Running only
  filters over the ledger's rows. Open a new Claude Code tab in a registered project's window: it
  should show as new, with no row and no session in progress; send a prompt, and within 30 s (at
  once if you switch to the window) it turns busy, in blue, the project's row shows it in progress,
  and `crw sessions` shows its row from its start. A real headless run is optional and spends
  usage: Start headless from the launch dialog, watch its tab's transcript, Stop it, then type a
  message and Resume; the ledger keeps a row per run. The session checked the tile only on the fake
  portfolio and the ledger's rule on scratch ledgers (rule 5). Notes go to `roadmap/inbox.md`
  [before EP-14]
