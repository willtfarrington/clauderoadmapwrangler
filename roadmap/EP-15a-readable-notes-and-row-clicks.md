---
ep: 15a
title: Readable notes and whole-row clicks
size: M
model: claude-opus-5-5
effort: max
status: done
---

# EP-15a — Readable notes and whole-row clicks

**Size:** M · **Depends on:** EP-7a, EP-13 · **Blocks:** EP-16 (the re-plan, which makes the window
the way this project's sessions start)

## Context

Two of the window's faults, put in one M at the owner's request (2026-11-03), as EP-7a took IN-4's
notes. First, the Detail pane draws the next brief's notes, the items waiting on the owner and the
inbox through `markup`, whose `reflow` joins every continuation line into one paragraph and whose
`spans` mark only bold, code and struck text. A bullet such as EP-15's becomes one block of about
2,500 characters, and the owner loses their place in it. Second, a row of the Projects pane answers
hover and click only between its cells: over the name, a chip, the title, the counts, a badge or an
icon, a label takes the pointer, so the row is neither hovered nor clicked and a right click misses
its menu.

**Session start (2026-11-03, 22:28 UTC).** Started from `DATA\clauderoadmapwrangler.code-workspace`
in the VS Code extension. The tree is clean, no inbox message is open, and OT-10 (the owner's look at
the popup) is open, gated before EP-16. The model is Opus 5.5 and the environment gives the effort as
max.

**The owner's first answers (2026-11-03, the brief's first question).** All four parts of the
renderer: labelled paragraphs, numbered parts as a list, a reading measure and colours for the runs.
No folding: every segment stays open. A long owner item is split at its sentences, with its gate as
a chip beside its id; inbox messages and handed-on mail share the renderer.

**Verified at pickup (egui 0.37.0's source).**

- A label is selectable by default (`style.interaction.selectable_labels`). A selectable label
  senses click and drag (`widgets/label.rs`).
- In the hit test, the widget registered last wins a tie: "the one on top" (`hit_test.rs`,
  `find_closest_within`). A row allocates its rect with `Sense::click()` before its cells, so a
  selectable label drawn over it takes the click and the drag. An interactive widget that is not
  the hit is not hovered (`interaction.rs`). A label that senses hover only lets the row take the
  click, and it is hovered too, its hover text intact, being non-interactive and on top of the row.
- A `Frame` registers its rect after its content (`containers/frame.rs`, `Prepared::end`). A
  Sessions pane card that adds `Sense::click()` to its frame's response is therefore on top of its
  labels. Over a label, the card should win the click and the label the drag (text selection). A
  test settles it.
- A disabled widget is taken as sensing nothing, so the autopilot box (disabled until EP-19) leaves
  the click to the row and keeps its hover text.

## In scope

1. **The renderer** (`markup`, D-25). A note's text, reflowed as today, becomes blocks:
   - **Labelled paragraphs.** A segment that opens with a bold label ending in a colon
     (`**From EP-7:**`, `**From EP-10 (the owner's answer at its close):**`, `**From IN-1 (…):**`)
     starts a paragraph under its label. The label is on a line of its own, in the accent colour.
     No segment folds (the owner's choice).
   - **Numbered parts.** Parts written `(1) … (2) …` in sequence become an indented list, each
     part's text wrapping under itself beside its number.
   - Blank lines and list lines stay.
   - **A reading measure.** In a wide pane, lines wrap at about 100 characters of the body font.
   - **The runs' colours**, from `style`, readable in both themes:
     - inline code in the code face, its kind told apart where it is plain: a path, a command
       (`crw …`, `git …`), a name in the code (`list::cell`);
     - the ids `EP-n`, `D-n`, `OT-n` and `IN-n` in a reference colour;
     - bold labels in the accent;
     - a `[before EP-m]` gate in amber;
     - struck text weak;
     - a link as its text, in the link colour.

   Nothing is reworded or dropped. A form the rules do not know (another project's) draws as one
   paragraph, as today. The launch dialog's prompt stays source text, being what the session is
   sent.
2. **Owner items, inbox, mail** (the owner's choice). The same renderer. A long item (more than two
   sentences, or over about 240 characters) puts each sentence on a line of its own, so its steps
   (`Optional, spends usage:`, `Notes go to …`) read one at a time. A gate is a chip beside the
   item's id. A trailing `[before EP-m]` is not repeated in the text, nor the gate in the meta line.
   Open inbox messages get the renderer, and so does each handed-on block's text, which the Detail
   pane does not show today. The ticked ones are drawn weak.
3. **The whole row answers** (D-25). A row's labels are not selectable, so they sense hover only,
   keep their hover texts and leave the click to the row. The expander and Start keep their own
   clicks. Every cell is drawn through `list::cell`, one place for the rule, beside the progress
   pill's `ui.put`. The folder rows and the group headers are made alike.
   - The Sessions pane's cards are checked too: the clickable one (a window's session) gets
     labels that are not selectable.
   - A row's text can no longer be selected for copying; the Detail pane's stays selectable.
   - A test drives an `egui::Context` with the pointer over a row's name and expects the row
     hovered and clicked.
4. **EP-15's notes.**
   - `style::red` is egui's own `error_fg_color`, a pure red that reads dim on a button in the
     dark theme (the Decisions pane's Deny, the Quit question). It gets a shade of its own per
     theme, looked at beside the notes' colours.
   - A session the poll reports waiting reads "waits for input needed" on its card and row
     (`Seen::doing`). The words follow the popup's, "waits for you: input needed".
5. **The fake portfolio.** A fake project's next brief gains a long note in every form, and it
   gains a long owner task and inbox message, for the pictures.
6. **Records.** A D-25 addendum; the README where it describes the Detail pane; pickup notes for
   later briefs; the owner task; the completion note.

## Out of scope

- Folding segments (the owner's choice).
- References and links that open what they name (a brief, a decision, a URL); this is v2 matter.
- Rewording a note, or changing how the files are written.
- The launch dialog's prompt, the transcript, the Decisions pane's question text.
- A row's text selected for copying (gone by design; the Detail pane keeps it).

## Verification / acceptance

- `cargo test`, `cargo test --features screenshot`, both clippy runs, `cargo fmt --check`, `cargo
  build --release` with the window quit from the tray, then the install script.
- Unit tests:
  - The blocks: segments, numbered parts in sequence (and not out of it), blank lines, list lines,
    a form the rules do not know.
  - The sentences: abbreviations, code, parentheses, the short item left whole.
  - The runs: each code kind, the ids, the gate, links, struck text, nothing inside code.
  - The trailing gate left out once the chip shows it.
  - The reading measure.
  - The row hovered and clicked with the pointer over its name; the session card clicked over its
    text.
  - The waiting words.
- Screenshot runs at 1600 × 1000, 2300 × 1000 and 1100 wide, in both themes, on the fake project
  with the long note, each picture looked at. A dark picture of the Decisions pane and of the Quit
  question for the red.
- The real window can be judged only on the owner's screen (rule 5): the owner looks at it with the
  new build at the close, or later as an owner task.
- Rule 3's word check prints nothing on the staged diff.

> **Completion note (2026-11-03; claude-opus-5-5/max; about 1 h 50 min of wall time with the
> owner's two trials and answers).** The Detail pane sets a brief's notes, the items waiting on the
> owner, the inbox and the mail for reading, in three of the window's own colours. A row of the
> Projects pane takes a click, a right click and the hover anywhere on it. At the first trial the
> owner asked for more space between paragraphs, bullets for an owner task's sentences and fewer,
> on-theme colours in place of a teal accent and seven hues; the second build has them.
>
> **Built.**
>
> - `markup`:
>   - Links join the marks: a `[text](target)` shows its text.
>   - Runs are told by kind (`Kind`): an id, a gate, or code of a kind (`Code`: path, command,
>     name, other).
>   - `blocks` sets a text as `Block`s: a label, a paragraph, a numbered part, a step (a sentence
>     of a long item) and a blank line.
>   - `note` draws the blocks at a reading measure of a hundred characters (`measure`), spaced by
>     `gap`, a part beside its number and a step beside a bullet.
>   - `coloured` colours the runs; `job` keeps the plain marks for a row's title.
> - `style`:
>   - `reference` (violet) and `code_colour` (green for a path or a command).
>   - `red` gets a shade per theme.
>   - `unselectable` makes a `Ui`'s labels sense hover only.
> - The Detail pane:
>   - The next brief's title and notes go through the renderer, the notes' frame no wider than
>     the measure.
>   - An owner item shows its id with its gate as an amber chip, and its text a sentence to a
>     bullet when long. The gate is dropped from the text's end and from the meta line.
>   - Open and ticked inbox messages and their dispositions go through the renderer.
>   - Each handed-on block's text shows under its line, its label left out.
> - The whole row:
>   - `list::cell` makes a cell's labels hover-only, and the list's `Ui` does it once for every
>     cell. The progress pill's label is not selectable.
>   - The group headers and the folder rows draw through `cell`.
>   - A window session's card in the Sessions pane is made alike.
> - The waiting words: `Seen::doing` says "waits for you: …", and `Seen::waits_for_you(place)` puts
>   the place before what is awaited, on the Sessions and Decisions cards.
> - Tests and fixtures:
>   - `testing` drives a bare context pass by pass with a pointer.
>   - The portfolio tests' scratch folder and fake store are shared with the other modules'
>     tests.
>   - In the fake portfolio, harbor's EP-4 note has every form and its OT-2 is long; lantern's
>     IN-1 is long, with parts.
>
> **Verification.**
>
> - `cargo test`: 334 tests; with the screenshot feature, 336. Of these, 13 are new. Both clippy
>   runs and `cargo fmt --check` are clean, and `crw roadmap .` reports 0 problems.
> - The row test fails with the labels left selectable ("the row is drawn hovered") and passes
>   with the fix. The session card's test passes either way. Its frame registers after its labels
>   and lies on top, so a click over its text already reached it; the change stops a drag or a
>   double click from selecting the card's text.
> - Screenshot runs, each looked at, before and after the owner's trial notes:
>   - harbor at 1600 × 1000, 2300 × 1000 and 1100 × 1000, in both themes;
>   - tall ones of harbor's owner items and of lantern's inbox and mail;
>   - dark ones of the Decisions pane and the Quit question, for the red.
> - The release build and the install script ran twice. The owner tried both builds on the real
>   portfolio: the rows answered over their text at the first, and the notes read right at the
>   second.
>
> **Owner decisions.**
>
> | Decision | Outcome |
> | --- | --- |
> | The renderer's parts (the brief's first question) | All four: labelled paragraphs, numbered parts as a list, a reading measure, colours for the runs. |
> | Whether a segment folds | No folding (recommended). |
> | Owner items, inbox and mail | Split at sentences, the gate a chip beside the id (recommended). |
> | The first build's trial | The rows answer. The notes read well, with three changes asked: more padding, bullets for a task's sentences, fewer colours from the window's theme in place of the teal headings. Done in a second build. The notes' length was left for EP-24's chat. |
> | The records (the D-25 addendum, the README, the pickup notes, OT-11) | Kept as written (recommended). |
> | The README's pictures, which predate the renderer | Left to EP-16 (recommended). |
> | The second build's trial | Reads right (recommended). |
> | Commit and push | Committed and pushed (recommended); the row's hash is recorded in a commit of its own. |
>
> **Routine choices.**
>
> - The colours after the trial: a label and a gate in `style::amber`; a reference and a link in
>   `style::reference`; a path or a command in `style::green`; a name and other code in the code
>   face's own ink; struck text weak. A row's title keeps the plain marks.
> - A segment's label is a bold run ending in a colon that opens a sentence (at the line's start,
>   or after `.`, `!` or `?`). So a part's own label (`(2) **Where:**`) stays in its part.
> - Parts count only in sequence from (1): at least two, between spaces, outside code. The last
>   part runs to its segment's end.
> - Code kinds:
>   - a command opens with a program the notes name (`crw`, `git`, `gh`, `cargo`, `claude`, …),
>     an `.exe` given arguments, or a flag;
>   - a path has a separator, a home or environment folder, or a known extension;
>   - a name has `::`, a call, a field, an underscore or camel case.
> - A long item has more than two sentences or over 240 characters. A sentence ends at `.`, `?`
>   or `!` before a space and a capital, a digit, code, a bold mark or a quote. It never ends
>   inside code or brackets, nor after an abbreviation such as e.g. or etc. A single long sentence
>   stays a paragraph.
> - The measure is a fixed sentence of 100 characters laid out in the body font.
> - Spacing beyond the item spacing: 5 points between paragraphs, 10 before a label and none after
>   it, 2 between a list's items.
> - A trailing `[urgent]` stays in a message's text beside its chip (noted for EP-16).
> - Only a window session's card takes a click in the Sessions pane, so only its text stops being
>   selectable; the other cards keep theirs, for copying a session id.
>
> **For later briefs.** The "From EP-15a" notes in EP-15b, EP-16, EP-20, EP-24 and EP-26; OT-11,
> the owner's look at their other projects' notes, before EP-16.
>
> **Handed on.** Nothing. **Next:** EP-15b.
