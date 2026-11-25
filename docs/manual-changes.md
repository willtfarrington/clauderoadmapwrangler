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
