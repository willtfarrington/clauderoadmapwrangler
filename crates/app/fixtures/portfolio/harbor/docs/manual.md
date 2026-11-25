# harbor

The mooring list as phase P1 leaves it: the berth register, the tide tables and the ledger view.
A fixture of the screenshot runs, with fake names alone.

## What it does

harbor keeps the berths of a small marina in `berths.toml`, reads the harbour office's tide tables
and shows the ledger of fees.

## Install and start

Build it with `cargo build --release` and start `harbor.exe`; the register lives beside it.

## Walkthrough

### The first season's fees

Open the ledger view, pick a berth and note what the season's fee reads.
