# Inbox — messages from other projects

Read right after the roadmap at the start of a session (D-13). One message per line:
`- [ ] IN-n (from <project> EP-k, date) text`.

- [ ] IN-1 (from harbor EP-3, 2026-10-28) The tide cache is a plain CSV at `tides/2026.csv`; the
  home page can show the next high water from it. The office publishes the next season's table in
  March, so the cache is read again then. Two consequences for EP-3: (1) `tides::next_high()`
  reads the file once per page build, never per visitor. (2) A missing file shows "tides not
  published yet" rather than an error; `cargo run -- build` checks both. [urgent]
