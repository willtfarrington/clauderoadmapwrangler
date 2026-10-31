//! `crw`, the console side of clauderoadmapwrangler (D-2, D-14): `doctor` from EP-1; `scan`,
//! `adopt`, `ignore` and `set` from EP-2, the console side of the registry until the dashboard
//! (EP-6); `roadmap` from EP-3, the parser's; `inbox send` and the `hook` and `statusline` relays
//! arrive with their briefs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chrono::Utc;
use crw_core::discovery::{self, Folder, Listing, View};
use crw_core::doctor::{self, Outcome};
use crw_core::proposal;
use crw_core::registry::{State, Store};
use crw_core::roadmap::{self, Size, table};
use crw_core::version::MIN_CLAUDE_CODE;
use crw_core::{files, paths};

const USAGE: &str = "\
usage: crw <command>

commands:
  doctor                         print the versions of claude, gh, tailscale, code and the extension
  scan                           print the registry: projects, unadopted folders, proposals
  adopt <folder>...              register folders of the data root and add them to the workspace file
  ignore <folder>...             leave folders of the data root out of the unadopted list
  set <project> state <state>    active, paused, maintenance, done, archived or unformalised
  set <project> priority <1-5>   1 is the highest
  set <project> pinned <yes|no>  pinned projects sort first
  roadmap <project|folder|file>  print what the parser reads in a roadmap: phases, the next EP,
                                 its brief notes, counts and problems
  help                           print this text
  --version                      print crw's version

CRW_HOME=<folder> keeps the registry, the ledger and the backups in that folder instead of the
profile folders, for trying the commands.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["doctor"] => doctor(),
        ["scan"] => scan(),
        ["adopt", folders @ ..] if !folders.is_empty() => adopt(folders),
        ["ignore", folders @ ..] if !folders.is_empty() => ignore(folders),
        ["set", project, field, value] => set(project, field, value),
        ["roadmap", target] => roadmap(target),
        ["--version" | "-V"] => {
            println!("crw {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [] | ["help" | "--help" | "-h"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => usage_error(&format!("unknown command: {}", args.join(" "))),
    }
}

fn usage_error(message: &str) -> ExitCode {
    eprintln!("crw: {message}\n\n{USAGE}");
    ExitCode::from(2)
}

fn failed(error: impl std::fmt::Display) -> ExitCode {
    eprintln!("crw: {error}");
    ExitCode::FAILURE
}

/// Prints one line per tool and fails when Claude Code is missing or older than the minimum.
fn doctor() -> ExitCode {
    println!("crw {} doctor", env!("CARGO_PKG_VERSION"));
    let checks = doctor::run();
    for check in &checks {
        let (version, detail) = match &check.outcome {
            Outcome::Found { version, source } => {
                (version.to_string(), source.display().to_string())
            }
            Outcome::Missing => ("-".to_owned(), "not found".to_owned()),
            Outcome::Failed { source, reason } => {
                ("?".to_owned(), format!("{reason}: {}", source.display()))
            }
        };
        println!("  {:<10} {:<9} {detail}", check.name, version);
    }
    if doctor::claude_meets_minimum(&checks) {
        println!("claude meets the minimum version {MIN_CLAUDE_CODE}");
        ExitCode::SUCCESS
    } else {
        println!(
            "claude is missing or older than {MIN_CLAUDE_CODE}: headless sessions will be refused"
        );
        ExitCode::FAILURE
    }
}

/// The owner's registry, or `CRW_HOME`'s.
fn store() -> Result<Store, String> {
    Store::locate().ok_or_else(|| {
        format!(
            "no profile folder: neither APPDATA and LOCALAPPDATA nor {} is set",
            paths::HOME_VAR
        )
    })
}

/// Prints the registry view and the proposals; reads, never writes (EP-24's read-only chat runs
/// it).
fn scan() -> ExitCode {
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    let loaded = match store.load() {
        Ok(loaded) => loaded,
        Err(e) => return failed(e),
    };
    let view = match discovery::view(&loaded.registry) {
        Ok(view) => view,
        Err(e) => return failed(e),
    };
    let facts = view.facts();
    let now = Utc::now();

    println!("crw {} scan", env!("CARGO_PKG_VERSION"));
    let note = if loaded.written {
        ""
    } else {
        " (not written yet: the default)"
    };
    println!("  registry   {}{note}", store.path.display());
    println!("  data root  {}", view.data_root.display());
    match &view.workspace {
        None => println!("  workspace  not set"),
        Some(file) => match &file.entries {
            Ok(entries) => println!(
                "  workspace  {} ({} folders)",
                file.path.display(),
                entries.len()
            ),
            Err(e) => println!("  workspace  {} (unreadable: {e})", file.path.display()),
        },
    }

    let width = view
        .registered
        .iter()
        .map(|r| r.project.name.len())
        .chain(view.unadopted.iter().map(|f| f.name.len()))
        .max()
        .unwrap_or(0);
    println!("\nregistered ({})", view.registered.len());
    for (registered, facts) in view.registered.iter().zip(&facts) {
        let project = &registered.project;
        let pin = if project.pinned { "pinned" } else { "" };
        let place = match &registered.folder {
            Some(folder) => describe(&view, folder),
            None => format!("missing: {}", registered.path.display()),
        };
        let age = facts
            .last_commit
            .map(|at| format!("  last commit {}", days_ago((now - at).num_days())))
            .unwrap_or_default();
        let line = format!(
            "  {:<width$}  {:<12}  p{}  {pin:<6}  {place}{age}",
            project.name,
            project.state.as_str(),
            project.priority,
        );
        println!("{}", line.trim_end());
    }
    println!("\nunadopted ({})", view.unadopted.len());
    for folder in &view.unadopted {
        let line = format!("  {:<width$}  {}", folder.name, describe(&view, folder));
        println!("{}", line.trim_end());
    }
    println!("\nignored ({})", view.ignored.len());
    if !view.ignored.is_empty() {
        println!("  {}", view.ignored.join(", "));
    }

    let proposals: Vec<_> = view
        .registered
        .iter()
        .zip(&facts)
        .flat_map(|(registered, facts)| {
            let state = registered.project.state;
            proposal::propose(state, facts, now)
                .into_iter()
                .map(move |p| (registered, state, p))
        })
        .collect();
    println!("\nproposals ({})", proposals.len());
    for (registered, state, proposal) in &proposals {
        let to: Vec<&str> = proposal.to.iter().map(|s| s.as_str()).collect();
        println!(
            "  {:<width$}  {state} -> {}  ({})",
            registered.project.name,
            to.join(" or "),
            proposal.reason
        );
    }
    if !proposals.is_empty() {
        println!("confirm one with: crw set <project> state <state>");
    }
    ExitCode::SUCCESS
}

/// `git  roadmap  workspace` and the like: what a folder is and where it is listed.
fn describe(view: &View, folder: &Folder) -> String {
    let kind = if folder.git { "git" } else { "folder" };
    let roadmap = if folder.roadmap { "roadmap" } else { "-" };
    let listed = match view.in_workspace(&folder.path) {
        Some(true) => "workspace",
        Some(false) => "-",
        None => "?",
    };
    format!("{kind:<6}  {roadmap:<7}  {listed:<9}")
}

fn days_ago(days: i64) -> String {
    match days {
        ..=0 => "today".to_owned(),
        1 => "1 day ago".to_owned(),
        n => format!("{n} days ago"),
    }
}

fn adopt(folders: &[&str]) -> ExitCode {
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    match discovery::adopt(&store, folders, &files::today()) {
        Ok(adopted) => {
            for one in adopted {
                let listing = match one.workspace {
                    Listing::Appended { backup } => format!(
                        "added to the workspace file (the old one is kept as {})",
                        backup.display()
                    ),
                    Listing::AlreadyListed => "already in the workspace file".to_owned(),
                    Listing::NoWorkspaceFile => "no workspace file is set".to_owned(),
                };
                println!("adopted {} as {}; {listing}", one.name, one.state);
            }
            ExitCode::SUCCESS
        }
        Err(e) => failed(e),
    }
}

fn ignore(folders: &[&str]) -> ExitCode {
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    match discovery::ignore(&store, folders, &files::today()) {
        Ok(ignored) if ignored.is_empty() => {
            println!("already ignored");
            ExitCode::SUCCESS
        }
        Ok(ignored) => {
            println!("ignoring {}", ignored.join(", "));
            ExitCode::SUCCESS
        }
        Err(e) => failed(e),
    }
}

fn set(project: &str, field: &str, value: &str) -> ExitCode {
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    let today = files::today();
    let changed = match field {
        "state" => value
            .parse::<State>()
            .and_then(|state| store.change(&today, |c| c.set_state(project, state))),
        "priority" => match value.parse::<u8>() {
            Ok(priority) => store.change(&today, |c| c.set_priority(project, priority)),
            Err(_) => return usage_error(&format!("priority {value:?} is not 1 to 5")),
        },
        "pinned" => match value {
            "yes" | "true" => store.change(&today, |c| c.set_pinned(project, true)),
            "no" | "false" => store.change(&today, |c| c.set_pinned(project, false)),
            _ => return usage_error(&format!("pinned takes yes or no, not {value:?}")),
        },
        _ => {
            return usage_error(&format!(
                "set changes state, priority or pinned, not {field:?}"
            ));
        }
    };
    match changed {
        Ok((registry, ())) => {
            let name = registry
                .project(project)
                .map_or(project, |p| p.name.as_str());
            println!("{name}: {field} {value}");
            ExitCode::SUCCESS
        }
        Err(e) => failed(e),
    }
}

/// Prints what the parser reads in one roadmap: the phases, the next EP with its row and brief
/// notes, the counts and the problems. Reads, never writes.
fn roadmap(target: &str) -> ExitCode {
    let path = match roadmap_file(target) {
        Ok(path) => path,
        Err(e) => return failed(e),
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => return failed(format!("{}: {e}", path.display())),
    };
    let parsed = table::parse(&text);
    let counts = parsed.counts();

    println!("crw {} roadmap", env!("CARGO_PKG_VERSION"));
    println!("  file      {}", path.display());
    match parsed.profile() {
        Some(profile) => println!(
            "  profile   {profile}: {} phases, {} rows",
            parsed.phases.len(),
            counts.total
        ),
        None => println!("  profile   none: no phase table"),
    }
    let stretch = match counts.stretch {
        0 => String::new(),
        n => format!(", {n} stretch"),
    };
    println!(
        "  progress  {} of {} done{stretch}",
        counts.done, counts.total
    );
    match parsed.next() {
        Some(row) => {
            let size = row.size.map_or("?", Size::as_str);
            println!("  next      {}  {size}  {}", row.id, row.brief.title);
            let phase = parsed
                .phase_of(row.id)
                .map(|phase| {
                    let heading = &phase.heading;
                    format!("phase {}, ", heading.id.as_ref().unwrap_or(&heading.name))
                })
                .unwrap_or_default();
            let depends = match row.depends.text.as_str() {
                "" => "-",
                text => text,
            };
            println!("            {phase}depends on {depends}, {}", row.done);
        }
        None if counts.all_checked() => println!("  next      none: every row is checked"),
        None => println!("  next      none"),
    }

    println!("\nphases ({})", parsed.phases.len());
    for phase in &parsed.phases {
        let heading = &phase.heading;
        let phase_counts = phase.counts();
        let progress = match phase_counts.total {
            0 => "no rows".to_owned(),
            total => format!("{} of {total}", phase_counts.done),
        };
        let status = heading
            .status
            .as_deref()
            .map(|status| format!("({status})"))
            .unwrap_or_default();
        let words = format!("{} {status}", heading.name);
        let line = format!(
            "  {:<6} {progress:<9} {}",
            heading.id.as_deref().unwrap_or("-"),
            words.trim()
        );
        println!("{}", line.trim_end());
    }

    if let Some(row) = parsed.next() {
        match parsed.note(row.id) {
            Some(note) => {
                println!("\nbrief notes of {} (line {})", row.id, note.line);
                for line in note.text.lines() {
                    println!("  {line}");
                }
            }
            None => println!("\nbrief notes of {}: none", row.id),
        }
    }

    println!("\nproblems ({})", parsed.problems.len());
    for problem in &parsed.problems {
        println!("  {problem}");
    }
    ExitCode::SUCCESS
}

/// A roadmap file as given, a folder's `roadmap\README.md`, or a registered project's.
fn roadmap_file(target: &str) -> Result<PathBuf, String> {
    let path = Path::new(target);
    if path.is_file() {
        return Ok(path.to_owned());
    }
    if path.is_dir() {
        return Ok(roadmap::readme(path));
    }
    let registry = store()?.load().map_err(|e| e.to_string())?.registry;
    let project = registry
        .project(target)
        .ok_or_else(|| format!("{target} is not a file, a folder or a registered project"))?;
    Ok(roadmap::readme(&project.folder(&registry.data_root())))
}
