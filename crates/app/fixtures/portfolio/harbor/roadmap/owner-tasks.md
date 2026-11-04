# Owner tasks — what only the owner can do

One line per task (D-10): `- [ ] OT-n (EP-k, date) text`, optionally `before EP-m`. Sessions append
at completion; the owner ticks in the tool.

- [x] OT-1 (EP-1, 2026-09-20) Create the private remote.
- [ ] OT-2 (EP-2, 2026-10-02) Ask the harbour master for last season's berth list, as a CSV if
  the office can export one. Optional, takes an afternoon: walk the pontoons with the printed
  list and mark the berths whose boats changed. Put the file in `import\berths-2025.csv`; EP-4
  reads it with `cargo run -- import`. Notes go to `roadmap/inbox.md` [before EP-5]
- [ ] OT-3 (EP-3, 2026-10-28) Check a week of the tide cache against the printed almanac.
