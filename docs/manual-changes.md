# Roadmap Wrangler — changes since the last phase's close

Since EP-58 (2026-11-25), the file's creation with the first manual. Each brief that changes what
a user sees or does adds its section; the next re-plan folds them into `docs/manual.md` and
empties this file.

## EP-58a — A project named by its registered name before a folder of the same name (2026-11-25)

The `crw` commands that take a project or a folder (`roadmap`, `tasks`, `tick`, `untick`, `send`,
`formalise`, `manual`, `status`, `prompt`, `history`, and `revert`, which needs a registered
project) read their argument in one order:

- **A word alone** (no `/` or `\`, not `.` or `..`, no drive) is a registered project's name
  first, from any folder. A file or a folder of that name in the working directory is read only
  when no project is registered under it. When both exist, as in a project laid out as
  `<name>\<name>`, the project is read and a `note:` line names the folder passed over: write
  `.\<name>` to read that folder instead.
- **A path** (`.`, `..`, a separator or a drive) reads that place. A folder that a project is
  registered in reads as that project, with its name and state, so `crw status .` from a
  project's root shows its registered name and state, and `crw send .` and `crw tick .` log the
  registered name.
- **A file** is still read as a brief (by its name) or a README by `crw roadmap`.

`crw help` says the order under the commands. The `formalise` skill now says that a session in a
project may name it by its registered name, by `.` or by its path, and that `crw path on` puts
`crw` on PATH.

## EP-58b — A data repository's site card from its own file (2026-11-25)

`crw export site` now carries a data repository's card too, from a file that the repository
writes itself, outside its folder, in the tool's local folder: `cards\<registered name>.json`
under `%LOCALAPPDATA%\clauderoadmapwrangler` (under `CRW_HOME` in a trial). The tool still reads
nothing inside a data repository and starts nothing there.

- **The file** is JSON in UTF-8 with `as_of` (the day it was written, `YYYY-MM-DD`) and the card's
  fields: `name` (the registered name), `badge` (the stack's text), `done`, `total`, `percent`
  (done of total, rounded down) and `next`, the first undone rows in table order, at most two,
  each with `id`, `title` and `size` (`S`, `M`, `L` or null). Other fields are ignored.
- **Public** is the account's list the window caches: the card is carried only when the list
  holds a public repository by the project's registered name or its folder's name; otherwise the
  file is not even opened.
- **The preview** prints the folder of the cards, and for a data repository either its card with
  "its own card of" and its day, or why it is left out: not public, no card file, a file that
  does not read, or one that is malformed (another name, a day badly written or after today,
  counts that do not add up, a wrong number of next rows, a size other than S, M, L or null). A
  card is never left out for its age: the repository rewrites it whenever its figures change.
- **The snippet** `projects-cards.html` is no longer written: `--yes` writes `projects.json` alone,
  which the site's page reads. A copy an older export wrote stays where it is.
