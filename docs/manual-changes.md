# Roadmap Wrangler — changes since the last phase's close

Since EP-59a (2026-11-27), the first fold at P6's close, which took EP-58a's and EP-58b's sections
into `docs/manual.md`. Each brief that changes what a user sees or does adds its section; the next
re-plan folds them into `docs/manual.md` and empties this file.

## EP-60 — The backup job read again on its own; the gate holding a planning interview (2026-11-27)

**The Backup badge in a window left up** ("Hosting and backup"). The window reads the backup
job again by itself whenever its run log (the `backup_log` setting) changes: the projects'
watcher looks at the log every 3 seconds, as it looks at each roadmap, and once the log has gone
10 seconds without a change, so after a run's last line, the job's settings, log and archives are
read again, with no program started. It reads the job at the day's turn too, beside every
project's files. A window left up for days therefore shows each daily run's check on the rows'
Backup badges and in the tray's marks, and no longer turns every badge "stale" at the job's usual
time two days after the window started; F5 is no longer needed to clear them. A log the window
cannot watch (none set, or on a drive not mounted when the job was read) is still read at the
day's turn, so its reading is never more than a day old. The rolling log's line at the day's turn
now ends "…, and the backup job".

**The gate and a planning interview** ("The autopilot", the gate's second condition). The next
brief must not be a planning interview either: a title naming a "planning interview", an "owner
interview" (or "owner's interview"), an "interview with the owner", a "planning session" or a
"planning round" is the owner's to run, as a re-plan is, unless it names another session's
interview after a word such as "at", "from", "after" or "before" ("fixes from the planning
interview"). An interview named without those words, such as a project's own interview feature,
does not count. The chain log says "the chain ends after EP-n: EP-m is a planning interview, the
owner's to run", and a chain plan stops at such a row as it stops at a re-plan. A title that
names both reads as a re-plan.

## EP-60a — The panes' "This project only" switch not kept; the template's placeholder note (2026-11-27)

**This project only in the Owner tasks and Inbox panes** ("The window", Owner tasks and Inbox).
The switch is no longer kept across restarts: each start shows every project, whatever it was
when the window closed. **Open in Owner tasks** and **Open in Inbox**, on the Detail pane's
lines, no longer turn it on: they select the project and bring the pane forward scrolled to the
project's group, every project still shown, and an urgent message's Show does the same in the
Inbox pane. A selected project with nothing open is not in the pane's list, and a line under the
summary says so, for example "<project>, selected in the list, has no open owner task: This
project only shows it alone." While the switch is on, a line under the summary says plainly what
the pane shows and how many open tasks (or messages, with how many of them urgent) of other
projects it hides, in amber while it hides any, for example "Shown: <project> alone, hiding 2
open owner tasks in 1 other project.", with **Show all**, which turns the switch off.

**The completion note's placeholder** ("Formalise and migrate", the brief template; the briefs as
"The autopilot and its gate" and the Detail pane's History read them). A completion note whose
label still reads `YYYY-MM-DD`, the brief template's placeholder, is no note: a brief that copied
the template and kept its closing placeholder reads as open, in `crw roadmap`, in the window and
at the autopilot's gate, until its real note is written, and the History's model, effort and
duration come from the real note. The brief template that `crw formalise` writes now gives the
note as an instruction in a comment, to be written at the close; the templates already in your
projects are left as they are.
