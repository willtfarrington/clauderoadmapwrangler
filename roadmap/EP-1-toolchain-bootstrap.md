---
ep: 1
title: Toolchain bootstrap
size: M
model: claude-opus-5-5
effort: xhigh
status: done
---

# EP-1 — Toolchain bootstrap

**Size:** M · **Depends on:** EP-0 · **Blocks:** EP-2, EP-3 (and through them every later brief)

## Context

EP-0 left a planning-only repository on `main` (two commits: the plan and its hash record): `DECISIONS.md`
D-1 … D-30, the roadmap, `CLAUDE.md`, a one-paragraph `README.md`, no code and no remote. This EP lays down
the Cargo workspace every later brief builds in (D-14), with the gate of rule 6 green on an empty
skeleton, the identity items of D-2 (licence, README, the short alias, the private remote), the install
script skeleton (D-29), CI (D-29) and a first useful command, `doctor`, which prints the versions of the
tools the later briefs drive.

**Verified at pickup (2026-10-31).** rustc and cargo 1.98.0 (stable, MSVC) with clippy and rustfmt; git
2.56.0; `gh` 2.103.3 logged in as the owner with scopes gist, read:org, repo, workflow; `claude` 2.1.261 at
`%USERPROFILE%\.local\bin\claude.exe`; VS Code's `code.cmd` on PATH; VS Code's own `extensions.json`
lists `anthropic.claude-code` 2.1.314; `tailscale.exe` under `C:\Program Files\Tailscale`, on the
PowerShell PATH; no repository named `clauderoadmapwrangler` on the owner's GitHub account. Every pin in
the brief notes exists on crates.io (eframe 0.37.0, egui_tiles 0.18.0, egui_extras 0.37.0, tokio 1.54.1,
process-wrap 10.0.1, claude-codes 2.1.316, pulldown-cmark 0.13.6, notify 8.2.1, rusqlite 0.41.1, toml
1.1.8, axum 0.8.9, askama 0.16.1); notify's newest is 9.0.0-rc.7, a release candidate, so 8.2.1 stands.
The unversioned names resolve today to serde 1.0.229, serde_json 1.0.154, chrono 0.4.45, anyhow 1.0.104,
thiserror 2.0.21 and tracing 0.1.44. The highest `rust-version` among them is 1.95 (the egui family). The
owner's public CI pins first-party actions by commit SHA (`actions/checkout` v7.0.1 = `3d3c42e…`;
`actions/cache` v6.1.0 = `55cc834…`, released 2026-06-26). An audit of both existing commits found only
already-public project names and no tokens, hostnames or addresses.

**Inbox and owner tasks.** Neither `roadmap/inbox.md` nor `roadmap/owner-tasks.md` exists yet: nothing to
triage (rule 1).

**Session note.** This session was started from another project's VS Code window with an
`@clauderoadmapwrangler` request, so its transcript is filed under that project's folder slug, not this
one's (rule 11 and D-16 ask for a window on this folder alone; EP-0 ran the same way).

## In scope

1. **Workspace.** A virtual root `Cargo.toml` (`resolver = "3"`, `[workspace.package]` with edition 2024,
   `rust-version = "1.98"`, MIT, `publish = false`) and three members: `crates/core` (library, no UI
   types), `crates/app` (the desktop binary `clauderoadmapwrangler`, an empty eframe window; the
   `screenshot` feature declared for the gate, its frame capture is EP-6's) and `crates/cli` (the console
   binary named by the alias).
2. **Pins.** `[workspace.dependencies]` lists every crate of the brief notes at its exact version (`=`),
   with the features the notes give (`persistence` for eframe, `bundled` for rusqlite); members take only
   what the skeleton uses; `Cargo.lock` committed.
3. **`core`.** `version`: the minimum Claude Code version 2.1.261 (D-15) and a version type that reads the
   first `x.y.z` in a tool's output; `doctor`: probes of `claude`, `gh`, `tailscale` and `code` (`--version`
   or `version`, one child at a time with a short spacing and a timeout, no console window, no
   `ANTHROPIC_API_KEY` in the child environment) and of the VS Code extension from `extensions.json`
   (folder names as the fallback), with no process.
4. **`cli doctor`** prints the five versions with their source and the `claude` version against the
   minimum, exits 1 when `claude` is missing or below it; `--version` and `help`.
5. **The gate** (rule 6): `cargo test`, `cargo clippy --all-targets --features screenshot` with no
   warnings, `cargo fmt --check`, `cargo build --release`.
6. **`LICENSE`** (MIT, the owner's form) and **`README.md`**: the one-paragraph description, the house-style
   section the parser expects (moved out of the roadmap's "How to use", which keeps a pointer), building
   and `doctor`, the AI-assistance and "drives your own CLI and login" notes.
7. **`scripts/install-shortcut.ps1`** skeleton: release build, desktop shortcut, Start-menu entry carrying
   an AppUserModelID (the id EP-15's toasts are attributed to); the two folders are parameters, so the
   script is first verified against scratch folders.
8. **`.github/workflows/ci.yml`**: windows-latest, Rust 1.98.0 installed by rustup, the gate with
   `--locked` and warnings denied, first-party actions pinned by SHA, read-only token, no secrets.
9. **The short alias** decided with the owner and recorded as a D-2 addendum.
10. **The private remote**: `gh repo create willtfarrington/clauderoadmapwrangler --private --source .
    --push` after the owner's yes, then the first CI run checked.

## Out of scope

- The registry, discovery and the ledger (EP-2); every parser (EP-3, EP-4, EP-5); the tile shell, fake
  projects and the screenshot capture (EP-6); badges (EP-7).
- An app icon, single instance, start-at-login and the rest of `doctor`'s checks (auth status, hooks,
  Tailscale state, `D:`, `gh` auth, integrity: EP-28).
- The public flip and the README screenshots (EP-8).
- Any crate beyond the brief notes' list.

## Verification / acceptance

- The gate passes locally with no warnings, and the same commands pass in CI on windows-latest.
- `crw doctor` (or the chosen alias) prints the five versions on this machine and reports `claude` against
  the minimum.
- The install script, run against scratch folders, writes both shortcuts, and the Start-menu entry's
  `System.AppUserModel.ID` reads back as the chosen id through the shell's own property system.
- `git status` shows `private/` ignored; `git grep` over the tracked files and the history finds no
  private project name.
- After the owner's yes: the private repository exists, `main` is pushed and tracks `origin/main`, and the
  first CI run is green.

> **Completion note (2026-10-31; opus/xhigh; ≈ 25 min to the commit).** Executed in one session from 14:07,
> started from another project's window (see the session note). Owner decisions (`AskUserQuestion`, every
> one the recommended option): **alias**: `crw` for the console binary, the core crate and the `CRW_*`
> variables, "Roadmap Wrangler" for the desktop shortcut and the Start-menu entry (D-2 addendum);
> **commit**: both commits in the house pattern; **remote**: `gh repo create
> willtfarrington/clauderoadmapwrangler --private --source . --push`, with no description or topics until
> the public flip; **shortcuts**: the install script run for real, so `Roadmap Wrangler.lnk` sits on the
> Desktop and in the Start menu's Programs folder, both carrying the AppUserModelID; **EP-8**: a pre-flip
> audit added to its brief notes. Routine choices made by the session: a virtual root with `resolver = "3"`
> and shared package keys; every planned crate pinned with `=` though only eframe and serde_json are used
> yet; packages `crw-core`, `clauderoadmapwrangler` and `crw-cli` (binary `crw`); hand-matched arguments in
> `crw` (no argument-parsing crate until a brief needs options); `doctor` tries `~\.local\bin\claude.exe`
> before PATH, as EP-9 will, and the other tools on PATH before their install folders, reads the extension
> from VS Code's `extensions.json` with folder names as the fallback, spaces probes 250 ms, times out at
> 20 s and exits 1 only when `claude` is missing or below the minimum; the AppUserModelID
> `willtfarrington.clauderoadmapwrangler`, written through the shell's COM link object (`Add-Type`), since
> WScript.Shell cannot set it; CI installs Rust 1.98.0 with rustup rather than through a
> `rust-toolchain.toml` (which would make rustup download a second copy of the same toolchain here), caches
> the registry and `target/` with `actions/cache`, denies clippy warnings and leaves the release build local.
> Verified: `cargo test` 10 passed; clippy with `-D warnings` and `cargo fmt --check` clean; a cold release
> build in 1 min 28 s; `crw doctor` in 1.4 s found claude 2.1.261 (meets the minimum), gh 2.103.3, tailscale
> 1.104.4, code 1.144.0 and the extension 2.1.314; the install script against scratch folders and then for
> real, the id read back through `Shell.Application`; a smoke run of the window (started, responding, closed
> by `WM_CLOSE`), which wrote eframe's `%APPDATA%\clauderoadmapwrangler\data\app.ron` beside the future
> registry; `git grep` over the new files and both earlier commits found no private project name. Found on
> the way: `WaitForInputIdle` returns before eframe's window exists (the first top-level window had no
> title), so a smoke check waits for the titled window. No owner tasks; no messages for other projects.
> Next: EP-2 (registry and discovery), started by hand in a VS Code window opened on this folder alone
> (D-16).
