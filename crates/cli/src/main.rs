//! `crw`, the console side of clauderoadmapwrangler (D-2, D-14): `doctor` from EP-1; `scan`,
//! `adopt`, `ignore` and `set` from EP-2, the console side of the registry until the dashboard
//! (EP-6); `roadmap` from EP-3 and EP-4, the parsers' (a project's snapshot, one README or one
//! brief); `tasks`, `tick` and `untick` from EP-5, the owner's items and the inbox; `status` and
//! `setting` from EP-7, the badges and the registry's settings; `prompt` from EP-10, what a launch
//! starts from, worded for either launch mode from EP-11; `dismiss` and `undismiss` from EP-33,
//! the proposals the owner leaves out; `inbox
//! send` and the `hook` and `statusline` relays arrive with their briefs.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chrono::{DateTime, Utc};
use crw_core::badges::activity::Activity;
use crw_core::badges::backup::{self, Backup, State as BackupState};
use crw_core::badges::health::{self, Run};
use crw_core::badges::hosting::{self, Check, Hosting, Listed};
use crw_core::badges::online::{self, Reading};
use crw_core::badges::short_age;
use crw_core::badges::stack::{self, Chip};
use crw_core::catalog::{self, Catalog};
use crw_core::discovery::{self, Folder, Listing, OwnFile, OwnFiles, View};
use crw_core::doctor::{self, Outcome};
use crw_core::inbox::{self, Recipient};
use crw_core::prompt::Chosen;
use crw_core::proposal::{Kind, Proposal};
use crw_core::registry::{LaunchMode, Project, Settings, State, Store};
use crw_core::roadmap::{Size, brief, snapshot, table};
use crw_core::session::{self, Preflight};
use crw_core::tasks::{self, OwnerItem};
use crw_core::version::MIN_CLAUDE_CODE;
use crw_core::{files, git, paths, process, prompt, proposal, workspace};

const USAGE: &str = "\
usage: crw <command>

commands:
  doctor                         print the versions of claude, gh, tailscale, code and the extension,
                                 and whether claude would start a headless session (no usage)
  scan                           print the registry: projects, unadopted folders, proposals
  adopt <folder>...              register folders of the data root, write each one's own
                                 <folder>.code-workspace beside it unless one is there, and add
                                 them to the workspace file
  ignore <folder>...             leave folders of the data root out of the unadopted list
  set <project> state <state>    active, paused, maintenance, done, archived or unformalised
  set <project> priority <1-5>   1 is the highest
  set <project> pinned <yes|no>  pinned projects sort first
  dismiss <project> [<proposal>] leave a proposed state out until the fact it rests on changes;
                                 every open proposal of the project, or the one named:
                                 all_rows_checked, pickup_gate or no_commit_since
  undismiss <project> [<proposal>]
                                 bring dismissed proposals back
  roadmap <project|folder>       print a project's snapshot: the README at HEAD and in the working
                                 tree, the next EP with its brief notes and its brief, the briefs
                                 and the decision log found, whether a session is in progress
  roadmap <README.md|EP-n-*.md>  print one roadmap README's tables, or one brief's parse
  tasks <project|folder>         print what waits on the owner: the owner tasks, the inbox, the
                                 handed-on blocks and the open items of the older forms
  tick <project|folder> OT-n     tick an owner task in roadmap/owner-tasks.md
  tick <project|folder> IN-n <disposition...>
                                 tick an inbox message, its disposition written on its line
  untick <project|folder> OT-n   clear an owner task's tick
  status [--offline] [<project|folder>]
                                 print the badges of every registered project, or one: stack,
                                 progress, activity, hosting with the head check, CI and license,
                                 backup; asks git, gh and the origins (--offline: git only)
  setting [<key> [<value>]]      print the registry's settings, or set one: data_root,
                                 workspace_file, backup_settings, backup_log, claude_cli, vscode
  prompt <project|folder> [--visual|--headless]
                                 print what a launch of the next brief starts from: the launch
                                 mode, the model, effort, ultracode and permission mode with
                                 their reasons (the rule table), the neighbour folders and the
                                 prompt, worded for the project's launch mode or the one asked;
                                 launches nothing
  help                           print this text
  --version                      print crw's version

A tick changes one box in the project's file and nothing else there; the file as it was is copied
to the backups folder first. status writes nothing: it never fetches and keeps no cache.

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
        ["dismiss", project] => dismiss(project, None),
        ["dismiss", project, which] => dismiss(project, Some(which)),
        ["undismiss", project] => undismiss(project, None),
        ["undismiss", project, which] => undismiss(project, Some(which)),
        ["roadmap", target] => roadmap(target),
        ["tasks", target] => waiting(target),
        ["tick", target, id, words @ ..] => tick(target, id, words, true),
        ["untick", target, id] => tick(target, id, &[], false),
        ["status", rest @ ..] => status(rest),
        ["prompt", target] => prompt(target, None),
        ["prompt", target, "--visual"] => prompt(target, Some(LaunchMode::Visual)),
        ["prompt", target, "--headless"] => prompt(target, Some(LaunchMode::Headless)),
        ["setting"] => setting(None, None),
        ["setting", key] => setting(Some(key), None),
        ["setting", key, value] => setting(Some(key), Some(value)),
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

/// Prints one line per tool, then the session runner's preflight (EP-9), and fails when Claude
/// Code is missing, older than the minimum, or would refuse a headless session.
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
    let meets = doctor::claude_meets_minimum(&checks);
    if meets {
        println!("claude meets the minimum version {MIN_CLAUDE_CODE}");
    } else {
        println!(
            "claude is missing or older than {MIN_CLAUDE_CODE}: headless sessions will be refused"
        );
    }
    // The CLI the runner would start: the registry's setting, else where the installer puts it.
    let setting = store()
        .ok()
        .and_then(|store| store.load().ok())
        .and_then(|loaded| loaded.registry.settings.claude_cli);
    let Some(program) = session::cli::locate(setting.as_deref()) else {
        println!("  headless   refused: no Claude Code CLI found");
        return ExitCode::FAILURE;
    };
    let preflight = Preflight::run(&program);
    match &preflight.auth {
        Ok(auth) if auth.logged_in => {
            let how: Vec<&str> = [&auth.method, &auth.subscription]
                .into_iter()
                .flatten()
                .map(String::as_str)
                .collect();
            println!("  login      logged in ({})", how.join(", "));
        }
        Ok(_) => println!("  login      not logged in"),
        Err(e) => println!("  login      ? {e}"),
    }
    let refusals = preflight.refusals();
    if !refusals.is_empty() {
        println!("  headless   refused: {}", refusals.join("; "));
        return ExitCode::FAILURE;
    }
    if let Ok(flags) = &preflight.flags {
        let efforts = match flags.efforts.as_slice() {
            [] => "effort levels not listed".to_owned(),
            levels => format!("effort {}", levels.join(", ")),
        };
        let budget = if flags.has("--max-budget-usd") {
            "--max-budget-usd"
        } else {
            "no --max-budget-usd"
        };
        let ultracode = if flags.ultracode() {
            "ultracode"
        } else {
            "no ultracode"
        };
        println!("  headless   ready: {efforts}; {ultracode}; {budget}");
    }
    if meets {
        ExitCode::SUCCESS
    } else {
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
        None => println!("  workspace  {}", unset_workspace(&view.data_root)),
        Some(file) => {
            // A default registry chose it (D-7 addendum).
            let chosen = if loaded.written {
                ""
            } else {
                "; the data root's one file that lists more than one folder"
            };
            match &file.entries {
                Ok(entries) => println!(
                    "  workspace  {} ({} folders{chosen})",
                    file.path.display(),
                    entries.len()
                ),
                Err(e) => println!("  workspace  {} (unreadable: {e})", file.path.display()),
            }
        }
    }
    let own = view
        .registered
        .iter()
        .filter(|r| r.folder.as_ref().is_some_and(|f| f.own_workspace))
        .count();
    println!(
        "  own files  {own} of {} registered projects have <folder>.code-workspace beside them",
        view.registered.len()
    );

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

    let (dismissed, open): (Vec<_>, Vec<_>) = view
        .registered
        .iter()
        .zip(&facts)
        .flat_map(|(registered, facts)| {
            let project = &registered.project;
            proposal::propose(project.state, facts, &project.dismissed, now)
                .into_iter()
                .map(move |p| (project, p))
        })
        .partition(|(_, p)| p.dismissed);
    let line = |(project, proposal): &(&Project, Proposal)| {
        let to: Vec<&str> = proposal.to.iter().map(|s| s.as_str()).collect();
        println!(
            "  {:<width$}  {} -> {}  ({})  [{}]",
            project.name,
            project.state,
            to.join(" or "),
            proposal.reason,
            proposal.reason.kind()
        );
    };
    println!("\nproposals ({})", open.len());
    open.iter().for_each(line);
    if !open.is_empty() {
        println!(
            "confirm one with: crw set <project> state <state>; or leave it out until its fact \
             changes: crw dismiss <project> [<proposal>]"
        );
    }
    if !dismissed.is_empty() {
        println!("\ndismissed ({})", dismissed.len());
        dismissed.iter().for_each(line);
        println!("bring one back with: crw undismiss <project> [<proposal>]");
    }
    ExitCode::SUCCESS
}

/// Why the registry names no workspace file, and the setting that would name one (D-7
/// addendum).
fn unset_workspace(data_root: &Path) -> String {
    match workspace::aggregates(data_root).as_slice() {
        [] => "not set: no workspace file of the data root lists more than one folder".to_owned(),
        [one] => format!(
            "not set: {one} is the data root's one file that lists more than one folder; \
             crw setting workspace_file {one}"
        ),
        several => format!(
            "not set: {} files of the data root list more than one folder ({}); \
             crw setting workspace_file <file>",
            several.len(),
            several.join(", ")
        ),
    }
}

/// `git  roadmap  aggregate  own-file` and the like: what a folder is, whether the aggregate
/// workspace file lists it, and whether its own workspace file sits beside it.
fn describe(view: &View, folder: &Folder) -> String {
    let kind = if folder.git { "git" } else { "folder" };
    let roadmap = if folder.roadmap { "roadmap" } else { "-" };
    let listed = match view.in_workspace(&folder.path) {
        Some(true) => "aggregate",
        Some(false) => "-",
        None => "?",
    };
    let own = if folder.own_workspace {
        "own-file"
    } else {
        "-"
    };
    format!("{kind:<6}  {roadmap:<7}  {listed:<9}  {own:<8}")
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
    match discovery::adopt(&store, folders, &files::today(), OwnFiles::Write) {
        Ok(adopted) => {
            for one in adopted {
                let own = match one.own_file {
                    OwnFile::Written(path) => format!("wrote {}", path.display()),
                    OwnFile::Kept(path) => format!("kept its own {}", path.display()),
                    OwnFile::Skipped => "wrote no workspace file of its own".to_owned(),
                };
                let listing = match one.workspace {
                    Listing::Appended { backup } => format!(
                        "added to the workspace file (the old one is kept as {})",
                        backup.display()
                    ),
                    Listing::AlreadyListed => "already in the workspace file".to_owned(),
                    Listing::NoWorkspaceFile => "no workspace file is set".to_owned(),
                };
                println!("adopted {} as {}; {own}; {listing}", one.name, one.state);
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

/// Dismisses a project's open proposals, or the one named, keeping each one's fact in the
/// registry so that it returns when the fact changes (D-6 addendum). Asks git for the project's
/// last commit, as `scan` does.
fn dismiss(project: &str, which: Option<&str>) -> ExitCode {
    let kind = match which.map(str::parse::<Kind>).transpose() {
        Ok(kind) => kind,
        Err(e) => return usage_error(&e),
    };
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    let registry = match store.load() {
        Ok(loaded) => loaded.registry,
        Err(e) => return failed(e),
    };
    let Some(entry) = registry.project(project) else {
        return failed(format!("no project named {project:?} in the registry"));
    };
    let folder = entry.folder(&registry.data_root());
    let Some(folder) = Folder::inspect(&folder) else {
        return failed(format!("{}: the folder is missing", folder.display()));
    };
    let facts = discovery::facts_of(&folder);
    let open: Vec<Proposal> = proposal::propose(entry.state, &facts, &entry.dismissed, Utc::now())
        .into_iter()
        .filter(|p| !p.dismissed && kind.is_none_or(|kind| p.reason.kind() == kind))
        .collect();
    let name = entry.name.clone();
    if open.is_empty() {
        let named = kind.map(|kind| format!(" {kind}")).unwrap_or_default();
        return failed(format!(
            "{name} has no open proposal{named} (crw scan lists them)"
        ));
    }
    let changed = store.change(&files::today(), |c| {
        open.iter().try_for_each(|p| c.dismiss(&name, &p.reason))
    });
    match changed {
        Ok(_) => {
            for p in &open {
                println!(
                    "{name}: dismissed ({}): it returns when {}",
                    p.reason,
                    p.reason.returns_when()
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => failed(e),
    }
}

/// Takes a project's dismissals back, or the one named.
fn undismiss(project: &str, which: Option<&str>) -> ExitCode {
    let kinds = match which {
        Some(which) => match which.parse::<Kind>() {
            Ok(kind) => vec![kind],
            Err(e) => return usage_error(&e),
        },
        None => Kind::ALL.to_vec(),
    };
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    let changed = store.change(&files::today(), |c| {
        let mut taken = Vec::new();
        for kind in kinds {
            if c.undismiss(project, kind)? {
                taken.push(kind);
            }
        }
        Ok(taken)
    });
    match changed {
        Ok((registry, taken)) => {
            let name = registry
                .project(project)
                .map_or(project, |p| p.name.as_str());
            if taken.is_empty() {
                println!("{name}: nothing was dismissed");
            }
            for kind in taken {
                println!("{name}: {kind} is no longer dismissed");
            }
            ExitCode::SUCCESS
        }
        Err(e) => failed(e),
    }
}

/// What `crw roadmap` was pointed at.
enum Target {
    /// A project folder (given, or a registered project's): the whole snapshot.
    Folder(PathBuf),
    /// A roadmap README: the table reading alone.
    Readme(PathBuf),
    /// A brief file: its parse.
    Brief(PathBuf),
}

/// Prints what the parser reads: a project's snapshot (both readings, the briefs, the decision
/// log, whether a session is in progress), one README's tables, or one brief. Reads, never
/// writes.
fn roadmap(target: &str) -> ExitCode {
    println!("crw {} roadmap", env!("CARGO_PKG_VERSION"));
    match roadmap_target(target) {
        Ok(Target::Folder(folder)) => print_snapshot(&folder),
        Ok(Target::Readme(path)) => match fs::read_to_string(&path) {
            Ok(text) => {
                println!("  file      {}", path.display());
                let parsed = table::parse(&text);
                print_summary(&parsed);
                print_rest(&parsed, &[]);
                ExitCode::SUCCESS
            }
            Err(e) => failed(format!("{}: {e}", path.display())),
        },
        Ok(Target::Brief(path)) => match brief::read(&path) {
            Ok(parsed) => {
                print_brief(&path, &parsed);
                ExitCode::SUCCESS
            }
            Err(e) => failed(format!("{}: {e}", path.display())),
        },
        Err(e) => failed(e),
    }
}

/// A file as given (a brief by its name, else a README), a folder, or a registered project's
/// folder.
fn roadmap_target(target: &str) -> Result<Target, String> {
    let path = Path::new(target);
    if path.is_file() {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        return Ok(if brief::file_id(&name).is_some() {
            Target::Brief(path.to_owned())
        } else {
            Target::Readme(path.to_owned())
        });
    }
    if path.is_dir() {
        return Ok(Target::Folder(path.to_owned()));
    }
    let registry = store()?.load().map_err(|e| e.to_string())?.registry;
    let project = registry
        .project(target)
        .ok_or_else(|| format!("{target} is not a file, a folder or a registered project"))?;
    Ok(Target::Folder(project.folder(&registry.data_root())))
}

fn print_snapshot(folder: &Path) -> ExitCode {
    let snapshot = snapshot::take(folder);
    println!("  folder    {}", folder.display());
    let reading = match (&snapshot.head, &snapshot.tree, &snapshot.git) {
        (None, None, _) => "no roadmap/README.md".to_owned(),
        (Some(_), None, _) => "HEAD only (the README is missing from the working tree)".to_owned(),
        (None, Some(_), None) => {
            "the working tree (not a repository, or git is missing)".to_owned()
        }
        (None, Some(_), Some(_)) => "the working tree (the README is not committed)".to_owned(),
        (Some(head), Some(_), _) if snapshot.differs() => {
            let counts = head.counts();
            let next = head
                .next()
                .map_or("none".to_owned(), |row| row.id.to_string());
            format!(
                "the working tree; HEAD differs: {} of {} done, next {next}",
                counts.done, counts.total
            )
        }
        (Some(_), Some(_), _) => "the working tree (HEAD reads the same)".to_owned(),
    };
    println!("  reading   {reading}");
    let Some(current) = snapshot.current() else {
        print_files(&snapshot);
        print_problems(&[], &snapshot.problems);
        return ExitCode::SUCCESS;
    };
    print_summary(current);
    print_files(&snapshot);
    print_rest(current, &snapshot.problems);
    ExitCode::SUCCESS
}

/// The lines about the briefs, the next brief, the decision log and the session, between the
/// summary and the phases.
fn print_files(snapshot: &snapshot::Snapshot) {
    if let Some(row) = snapshot.next() {
        match snapshot.entry(row.id) {
            Some(entry) => {
                let brief = &entry.brief;
                let front = brief
                    .front
                    .as_ref()
                    .map_or("no front matter".to_owned(), |f| {
                        let mut parts = vec![format!(
                            "front matter ep {}",
                            f.ep().map_or("?".to_owned(), |id| id.to_string())
                        )];
                        if let Some(size) = f.size() {
                            parts.push(size.to_string());
                        }
                        if let (Some(model), Some(effort)) = (f.model(), f.effort()) {
                            parts.push(format!("{model}/{effort}"));
                        }
                        if let Some(status) = f.status() {
                            parts.push(status.to_owned());
                        }
                        parts.join(", ")
                    });
                let completion = match brief.completion() {
                    Some(evidence) => format!("complete: {evidence}"),
                    None => "open (no completion marker)".to_owned(),
                };
                println!("  brief     {}: {front}; {completion}", entry.file.name);
                if let Some(gate) = brief.pickup_gate() {
                    println!("            pickup gate at line {}", gate.line);
                }
                if let Some(handoff) = &entry.handoff {
                    let state = if handoff.is_complete() {
                        "complete"
                    } else {
                        "not complete"
                    };
                    println!(
                        "            handoff file {}: {state}{}",
                        handoff.path.display(),
                        handoff
                            .date()
                            .map(|d| format!(" ({d})"))
                            .unwrap_or_default()
                    );
                }
            }
            None => println!("  brief     {}: no brief file", row.id),
        }
    }
    let files = &snapshot.files;
    let complete = snapshot
        .briefs
        .iter()
        .filter(|entry| entry.is_complete())
        .count();
    let with_front = snapshot
        .briefs
        .iter()
        .filter(|entry| entry.brief.front.is_some())
        .count();
    println!(
        "  briefs    {} ({complete} complete, {with_front} with front matter), {} companions, {} duplicates",
        files.briefs.len(),
        files.companions.len(),
        files.duplicates.len()
    );
    let decisions = &snapshot.decisions;
    let log = match &decisions.path {
        Some(path) => {
            let name = path
                .strip_prefix(&snapshot.folder)
                .unwrap_or(path)
                .display()
                .to_string();
            let last = decisions
                .log
                .last_addendum()
                .map(|date| format!(" (last {date})"))
                .unwrap_or_default();
            let index = match decisions.log.index_rows {
                0 => String::new(),
                n => format!(", an index of {n} rows"),
            };
            let domain = match decisions.log.domain_entries {
                0 => String::new(),
                n => format!(", {n} domain-id entries"),
            };
            let problem = decisions
                .problem
                .as_ref()
                .map(|p| format!(" (unreadable: {p})"))
                .unwrap_or_default();
            format!(
                "{name}: {} entries, {} addenda{last}{index}{domain}{problem}",
                decisions.log.entries.len(),
                decisions.log.addenda.len()
            )
        }
        None => "no DECISIONS.md".to_owned(),
    };
    let adr = match decisions.adr_files {
        0 => String::new(),
        n => format!("; {n} ADR files"),
    };
    println!("  decisions {log}{adr}");
    let reasons = snapshot.in_progress();
    let session = match (&snapshot.git, reasons.is_empty()) {
        (_, false) => {
            let texts: Vec<String> = reasons.iter().map(ToString::to_string).collect();
            format!("in progress: {}", texts.join("; "))
        }
        (Some(_), true) => "not in progress (roadmap/ is clean)".to_owned(),
        (None, true) => "not in progress by the files (git not asked)".to_owned(),
    };
    println!("  session   {session}");
}

fn print_brief(path: &Path, brief: &brief::Brief) {
    println!("  file      {}", path.display());
    println!("  title     {}", brief.title.as_deref().unwrap_or("-"));
    let front = match &brief.front {
        Some(front) => front
            .fields
            .iter()
            .map(|f| format!("{}: {}", f.label, f.value))
            .collect::<Vec<_>>()
            .join(" · "),
        None => "none".to_owned(),
    };
    println!("  front     {front}");
    let fields = brief
        .fields
        .iter()
        .map(|f| format!("{}: {}", f.label, f.value))
        .collect::<Vec<_>>()
        .join(" · ");
    println!(
        "  fields    {}",
        if fields.is_empty() { "none" } else { &fields }
    );
    match brief.completion() {
        Some(evidence) => println!("  complete  yes: {evidence}"),
        None => println!("  complete  no"),
    }
    if let Some(handoff) = &brief.handoff {
        let state = if handoff.filled {
            "filled"
        } else {
            "not filled"
        };
        println!(
            "  handoff   {state}{} (line {})",
            handoff
                .date
                .as_ref()
                .map(|d| format!(" {d}"))
                .unwrap_or_default(),
            handoff.line
        );
    }
    println!("\nblocks ({})", brief.quotes.len());
    for quote in &brief.quotes {
        let date = quote
            .date
            .as_ref()
            .map(|d| format!(" {d}"))
            .unwrap_or_default();
        let label = if quote.label.is_empty() {
            "(no label)".to_owned()
        } else {
            quote.label.clone()
        };
        println!("  line {:<4} {}{date}  {label}", quote.line, quote.kind);
    }
}

/// The table reading's summary: profile, progress and the next EP.
fn print_summary(parsed: &table::Roadmap) {
    let counts = parsed.counts();
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
}

/// The table reading's detail: the phases, the next EP's brief notes and the problems.
fn print_rest(parsed: &table::Roadmap, extra_problems: &[String]) {
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
    print_problems(&parsed.problems, extra_problems);
}

fn print_problems(problems: &[table::Problem], extra: &[String]) {
    println!("\nproblems ({})", problems.len() + extra.len());
    for problem in problems {
        println!("  {problem}");
    }
    for problem in extra {
        println!("  {problem}");
    }
}

/// A project folder as given, or a registered project's.
fn project_folder(target: &str) -> Result<PathBuf, String> {
    match roadmap_target(target)? {
        Target::Folder(folder) => Ok(folder),
        Target::Readme(path) | Target::Brief(path) => Err(format!(
            "{} is a file: give a project folder or a registered project",
            path.display()
        )),
    }
}

/// Prints what waits on the owner in one project (D-10, D-13): the owner tasks, the inbox, the
/// handed-on blocks with their recipients, and the older forms' open items. Reads, never writes.
fn waiting(target: &str) -> ExitCode {
    println!("crw {} tasks", env!("CARGO_PKG_VERSION"));
    let folder = match project_folder(target) {
        Ok(folder) => folder,
        Err(e) => return failed(e),
    };
    let snapshot = snapshot::read_tree(&folder);
    let registry = store()
        .ok()
        .and_then(|store| store.load().ok())
        .map(|loaded| loaded.registry);
    let outgoing = inbox::outgoing(&snapshot.briefs, |name| {
        registry
            .as_ref()
            .and_then(|registry| registry.project(name))
            .map(|project| project.name.clone())
    });
    let relative = |path: &Path| {
        path.strip_prefix(&folder)
            .unwrap_or(path)
            .display()
            .to_string()
    };

    println!("  folder    {}", folder.display());
    match &snapshot.tasks {
        Some(file) => {
            let gated = file.open().filter(|task| task.gate.is_some()).count();
            println!(
                "  tasks     {}: {}, {} open ({gated} gated)",
                relative(&file.path),
                file.tasks.len(),
                file.open().count()
            );
        }
        None => println!("  tasks     no roadmap/{}", tasks::FILE),
    }
    match &snapshot.inbox {
        Some(file) => {
            let urgent = file.open().filter(|message| message.urgent).count();
            println!(
                "  inbox     {}: {} messages, {} open ({urgent} urgent)",
                relative(&file.path),
                file.messages.len(),
                file.open().count()
            );
        }
        None => println!("  inbox     no roadmap/{}", inbox::FILE),
    }
    let legacy = &snapshot.legacy;
    let count = |state| legacy.iter().filter(|item| item.state == state).count();
    let mut forms: Vec<_> = legacy.iter().map(|item| item.form).collect();
    forms.sort();
    forms.dedup();
    println!(
        "  older     {} items in {} forms: {} open, {} partly resolved, {} resolved",
        legacy.len(),
        forms.len(),
        count(tasks::State::Open),
        count(tasks::State::Partly),
        count(tasks::State::Resolved)
    );
    let latest = snapshot
        .latest_complete()
        .map_or("none".to_owned(), |entry| {
            let date = entry
                .completed_on()
                .map(|date| format!(" ({date})"))
                .unwrap_or_default();
            format!("{}{date}", entry.file.id)
        });
    println!("            the latest completed brief's hand-offs: {latest}");

    if let Some(file) = &snapshot.tasks {
        println!("\nowner tasks ({})", file.tasks.len());
        let mut ordered: Vec<_> = file.tasks.iter().collect();
        ordered.sort_by_key(|task| task.done);
        for task in ordered {
            let mut meta: Vec<String> = Vec::new();
            meta.extend(task.raised.map(|id| id.to_string()));
            meta.extend(task.date.clone());
            meta.extend(task.gate.map(|id| format!("before {id}")));
            println!(
                "  {} {:<6} {:<32} {}",
                if task.done { "[x]" } else { "[ ]" },
                task.id,
                meta.join(", "),
                one_line(&task.text, 70)
            );
        }
    }
    if let Some(file) = &snapshot.inbox {
        println!("\ninbox ({})", file.messages.len());
        let mut ordered: Vec<_> = file.messages.iter().collect();
        ordered.sort_by_key(|message| message.done);
        for message in ordered {
            let mut meta: Vec<String> = Vec::new();
            meta.extend(message.from.clone());
            meta.extend(message.date.clone());
            if message.urgent {
                meta.push("urgent".to_owned());
            }
            println!(
                "  {} {:<6} {:<40} {}",
                if message.done { "[x]" } else { "[ ]" },
                message.id,
                one_line(&meta.join(", "), 40),
                one_line(&message.text, 60)
            );
            if let Some(disposition) = &message.disposition {
                println!("             {}", one_line(disposition, 100));
            }
        }
    }
    println!("\nhanded on ({})", outgoing.len());
    for block in &outgoing {
        let recipient = match &block.recipient {
            Recipient::Project(name) => format!("to {name}: registered"),
            Recipient::Unknown => {
                format!("to {}: not registered, shown and not delivered", block.to)
            }
        };
        println!(
            "  {} line {:<4} {recipient}",
            relative(&block.path),
            block.line
        );
    }
    let open: Vec<&OwnerItem> = legacy.iter().filter(|item| item.is_open()).collect();
    println!(
        "\nopen items in the older forms ({} of {})",
        open.len(),
        legacy.len()
    );
    for item in open {
        let what = match &item.id {
            Some(id) => format!("{} {id}", item.form),
            None => item.form.to_string(),
        };
        let mut meta: Vec<String> = vec![format!("{}:{}", relative(&item.path), item.line)];
        meta.extend(item.gate.as_ref().map(ToString::to_string));
        if item.state == tasks::State::Partly {
            meta.push("partly resolved".to_owned());
        }
        println!(
            "  {what:<18} {:<48} {}",
            one_line(&meta.join(", "), 48),
            one_line(&item.text, 70)
        );
    }
    let mut problems: Vec<String> = snapshot.problems.clone();
    if let Some(file) = &snapshot.tasks {
        let path = relative(&file.path);
        problems.extend(file.problems.iter().map(|p| format!("{path} {p}")));
    }
    if let Some(file) = &snapshot.inbox {
        let path = relative(&file.path);
        problems.extend(file.problems.iter().map(|p| format!("{path} {p}")));
    }
    println!("\nproblems ({})", problems.len());
    for problem in &problems {
        println!("  {problem}");
    }
    ExitCode::SUCCESS
}

/// The first line of `text`, its runs of spaces collapsed, cut at `width` characters.
fn one_line(text: &str, width: usize) -> String {
    let first = text.lines().next().unwrap_or_default();
    let words = first.split_whitespace().collect::<Vec<_>>().join(" ");
    let more = text.lines().nth(1).is_some();
    if words.chars().count() > width {
        let cut: String = words.chars().take(width.saturating_sub(1)).collect();
        format!("{}…", cut.trim_end())
    } else if more {
        format!("{words} …")
    } else {
        words
    }
}

/// Prints the badges (D-24, D-25) of every registered project, or of the one named: the files'
/// readings, git's, the backup job's and, unless `--offline`, GitHub's and the origins'. Reads
/// and asks, never writes: no fetch, no cache.
fn status(args: &[&str]) -> ExitCode {
    let offline = args.contains(&"--offline");
    let targets: Vec<&str> = args
        .iter()
        .copied()
        .filter(|arg| *arg != "--offline")
        .collect();
    if targets.len() > 1 || targets.iter().any(|arg| arg.starts_with("--")) {
        return usage_error(&format!(
            "status takes --offline and one project, not {args:?}"
        ));
    }
    println!("crw {} status", env!("CARGO_PKG_VERSION"));
    let registry = store()
        .ok()
        .and_then(|store| store.load().ok())
        .map(|loaded| loaded.registry);
    let projects: Vec<(String, Option<State>, PathBuf)> = match targets.first() {
        Some(target) => match project_folder(target) {
            Ok(folder) => {
                let registered = registry.as_ref().and_then(|r| r.project(target));
                let name = registered.map_or_else(|| folder_name(&folder), |p| p.name.clone());
                vec![(name, registered.map(|p| p.state), folder)]
            }
            Err(e) => return failed(e),
        },
        None => match &registry {
            Some(registry) => {
                let root = registry.data_root();
                registry
                    .projects
                    .iter()
                    .map(|p| (p.name.clone(), Some(p.state), p.folder(&root)))
                    .collect()
            }
            None => return failed("the registry could not be read"),
        },
    };
    let now = Utc::now();

    let job = registry.as_ref().and_then(|registry| {
        let (settings, log) = registry.backup_files();
        settings.map(|settings| backup::Reading::read(&settings, log.as_deref()))
    });
    match &job {
        None => {
            println!("  backup    no job set (crw setting backup_settings <its settings file>)")
        }
        Some(job) => {
            let target = match &job.job {
                Ok(settings) => format!(
                    "target {} ({})",
                    settings.target.display(),
                    if job.mounted {
                        "mounted"
                    } else {
                        "not mounted"
                    }
                ),
                Err(e) => format!("unreadable: {e}"),
            };
            let run = match (&job.log, job.last_run()) {
                (None, _) => "; no log set".to_owned(),
                (Some(Err(e)), _) => format!("; the log is unreadable: {e}"),
                (Some(Ok(_)), Some(run)) => format!(
                    "; the last run ended {} ago: {}",
                    short_age(now - run.at),
                    run.status
                ),
                (Some(Ok(_)), None) => "; the log names no run".to_owned(),
            };
            println!("  backup    {}: {target}{run}", job.settings.display());
        }
    }

    let mut account: Option<Result<Vec<Listed>, String>> = None;
    let mut heads: HashMap<String, Check> = HashMap::new();
    let mut runs: HashMap<String, Result<Option<Run>, String>> = HashMap::new();
    if !offline {
        let asked: Vec<online::Project> = projects
            .iter()
            .filter(|(_, _, folder)| folder.is_dir())
            .map(|(name, _, folder)| online::Project {
                name: name.clone(),
                folder: folder.clone(),
                remote: git::remote_url(folder, "origin"),
                workflows: health::workflows(folder) > 0,
            })
            .collect();
        online::fetch(&asked, |reading| {
            match reading {
                Reading::Account(list) => account = Some(list),
                Reading::Head { project, check } => {
                    heads.insert(project, check);
                }
                Reading::Run { project, run } => {
                    runs.insert(project, run);
                }
            }
            true
        });
    }
    match &account {
        None => println!("  github    not asked (--offline)"),
        Some(Ok(list)) => println!(
            "  github    {} repositories of {}",
            list.len(),
            list.first()
                .map_or("the account", |l| l.repo.owner.as_str())
        ),
        Some(Err(e)) => println!("  github    could not be asked: {e}"),
    }
    let list = account
        .as_ref()
        .and_then(|account| account.as_ref().ok())
        .map(Vec::as_slice);

    for (name, state, folder) in &projects {
        println!();
        println!(
            "{name}{}",
            state.map(|s| format!("  ({s})")).unwrap_or_default()
        );
        if !folder.is_dir() {
            println!("  missing   {}", folder.display());
            continue;
        }
        let chips = stack::read(folder);
        let stack = chips.iter().map(Chip::text).collect::<Vec<_>>().join(", ");
        println!(
            "  stack     {}",
            if stack.is_empty() { "-" } else { &stack }
        );
        let snapshot = snapshot::read_tree(folder);
        let progress = match snapshot.current().map(|roadmap| roadmap.counts()) {
            Some(counts) if counts.total > 0 => format!(
                "{} of {} done ({} %)",
                counts.done,
                counts.total,
                counts.done * 100 / counts.total
            ),
            Some(_) => "no phase table".to_owned(),
            None => "no roadmap/README.md".to_owned(),
        };
        println!("  progress  {progress}");
        let activity = folder.join(".git").exists().then(|| {
            let status = {
                let _turn = process::turn();
                git::status(folder)
            };
            let last = {
                let _turn = process::turn();
                proposal::last_commit(folder)
            };
            status.map(|status| Activity::read(&status, last))
        });
        println!("  activity  {}", activity_text(activity.as_ref(), now));
        let remote = git::remote_url(folder, "origin");
        let hosting = hosting::classify(&[name, &folder_name(folder)], remote.as_deref(), list);
        let head = heads
            .get(name)
            .map(|check| format!("; the origin: {}", check.text()));
        println!(
            "  hosting   {}{}",
            hosting_text(&hosting, remote.as_deref(), now),
            head.unwrap_or_default()
        );
        let ci = match (health::workflows(folder), runs.get(name)) {
            (0, _) => "no workflows".to_owned(),
            (_, None) => "workflows (GitHub not asked)".to_owned(),
            (_, Some(Ok(Some(run)))) => format!(
                "{}{}",
                run.text(),
                run.created_at
                    .map(|at| format!(", {} ago", short_age(now - at)))
                    .unwrap_or_default()
            ),
            (_, Some(Ok(None))) => "workflows, no run yet".to_owned(),
            (_, Some(Err(e))) => format!("GitHub could not be asked: {e}"),
        };
        let license = health::license(folder).unwrap_or_else(|| "no license file".to_owned());
        println!("  health    {ci}; {license}");
        if let Some(job) = &job {
            println!("  backup    {}", backup_text(&job.project(folder), now));
        }
    }
    ExitCode::SUCCESS
}

fn folder_name(folder: &Path) -> String {
    folder
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `last commit 3 d ago; 2 changed; main → origin/main, level`.
fn activity_text(activity: Option<&Result<Activity, String>>, now: DateTime<Utc>) -> String {
    match activity {
        None => "not a git repository".to_owned(),
        Some(Err(e)) => format!("git could not be asked: {e}"),
        Some(Ok(activity)) => {
            let last = activity
                .last_commit
                .map_or("no commit yet".to_owned(), |at| {
                    format!("last commit {} ago", short_age(now - at))
                });
            let changed = match activity.changed {
                0 => "clean".to_owned(),
                n => format!("{n} changed"),
            };
            format!("{last}; {changed}; {}", activity.branch_text())
        }
    }
}

fn hosting_text(hosting: &Hosting, remote: Option<&str>, now: DateTime<Utc>) -> String {
    let pushed = |listed: &Listed| {
        listed
            .pushed_at
            .map(|at| format!(", pushed {} ago", short_age(now - at)))
            .unwrap_or_default()
    };
    match hosting {
        Hosting::Listed(listed) => format!(
            "{} {}{}",
            hosting.label(),
            listed.repo.slug(),
            pushed(listed)
        ),
        Hosting::Orphan(listed) => format!(
            "orphan: no remote, but the account holds {}{}",
            listed.repo.slug(),
            pushed(listed)
        ),
        Hosting::Missing(repo) => format!("missing: the account does not list {}", repo.slug()),
        Hosting::GitHub(repo) => format!("GitHub {} (not in the account's list)", repo.slug()),
        Hosting::Elsewhere(host) => format!("a remote at {host}"),
        Hosting::Local => match remote {
            Some(url) => format!("local ({url})"),
            None => "local: no remote".to_owned(),
        },
    }
}

fn backup_text(backup: &Backup, now: DateTime<Utc>) -> String {
    let unit = backup.unit.as_deref().unwrap_or_default();
    let checked = backup
        .checked
        .map(|at| format!("checked {} ago", short_age(now - at)))
        .unwrap_or_default();
    let archive = backup
        .archive
        .as_ref()
        .map(|archive| format!(", newest archive {}", archive.name))
        .unwrap_or_default();
    let excluded = match backup.excluded.as_slice() {
        [] => String::new(),
        paths => format!("; leaves out {}", paths.join(", ")),
    };
    let text = match backup.state(now) {
        BackupState::NotCovered => return "not covered by the job".to_owned(),
        BackupState::Unknown => format!("{unit}: no archive and no log line yet"),
        BackupState::Fresh => format!("{unit}: {checked}{archive}"),
        BackupState::Stale => format!("{unit}: stale, {checked}{archive}"),
        BackupState::Failing => format!(
            "{unit}: the job's last word: {}",
            backup.problem.as_deref().unwrap_or_default()
        ),
    };
    format!("{text}{excluded}")
}

/// Prints the registry's settings, or one, or sets one through the store (a daily copy of the
/// file first, D-8).
fn setting(key: Option<&str>, value: Option<&str>) -> ExitCode {
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    if let Some(key) = key
        && !Settings::KEYS.contains(&key)
    {
        return usage_error(&format!(
            "unknown setting {key:?}: {}",
            Settings::KEYS.join(", ")
        ));
    }
    if let (Some(key), Some(value)) = (key, value) {
        return match store.change(&files::today(), |c| c.set_setting(key, value)) {
            Ok((registry, ())) => {
                println!("{key} = {value}");
                let full = registry.data_root().join(value);
                if key != "data_root" && !full.exists() {
                    println!("note: {} does not exist yet", full.display());
                }
                ExitCode::SUCCESS
            }
            Err(e) => failed(e),
        };
    }
    let registry = match store.load() {
        Ok(loaded) => loaded.registry,
        Err(e) => return failed(e),
    };
    println!("crw {} setting", env!("CARGO_PKG_VERSION"));
    for name in Settings::KEYS
        .iter()
        .filter(|name| key.is_none_or(|k| k == **name))
    {
        let value = match (registry.settings.get(name), *name) {
            (Some(path), _) => path.display().to_string(),
            (None, "data_root") => format!("{} (the default)", registry.data_root().display()),
            (None, _) => "not set".to_owned(),
        };
        println!("  {name:<16} {value}");
    }
    ExitCode::SUCCESS
}

/// Ticks one line of a project's `owner-tasks.md` or `inbox.md`, or clears an owner task's tick
/// (D-10, D-13): the box changes in the project's file and nothing else there; the file as it was
/// is copied to the backups folder first (D-11).
fn tick(target: &str, id: &str, words: &[&str], checked: bool) -> ExitCode {
    let folder = match project_folder(target) {
        Ok(folder) => folder,
        Err(e) => return failed(e),
    };
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    let today = files::today();
    let prefix = id
        .split('-')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    let (path, result) = match (prefix.as_str(), checked) {
        ("OT", _) if !words.is_empty() => {
            return usage_error("an owner task is ticked without a disposition");
        }
        ("OT", true) => (
            tasks::path(&folder),
            tasks::tick(&folder, id, &store.backups, &today),
        ),
        ("OT", false) => (
            tasks::path(&folder),
            tasks::untick(&folder, id, &store.backups, &today),
        ),
        ("IN", true) => (
            inbox::path(&folder),
            inbox::tick(&folder, id, &words.join(" "), &store.backups, &today),
        ),
        ("IN", false) => {
            return usage_error("an inbox message's tick stays: its disposition is on the line");
        }
        _ => return usage_error(&format!("{id} is not an OT-n or IN-n id")),
    };
    match result {
        Ok(copy) => {
            println!(
                "{} {id} in {}; the file as it was is kept as {}",
                if checked { "ticked" } else { "cleared" },
                path.display(),
                copy.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => failed(format!("{}: {e}", path.display())),
    }
}

/// Prints what a launch of a project's next brief starts from (D-17, D-22): the rule table's
/// resolution with its reasons, the neighbour folders and the prompt, worded for `mode` when one
/// is asked (EP-11). Reads, never launches, and asks nothing of the CLI: an effort step moves
/// along the built-in levels here.
fn prompt(target: &str, mode: Option<LaunchMode>) -> ExitCode {
    println!("crw {} prompt", env!("CARGO_PKG_VERSION"));
    let folder = match project_folder(target) {
        Ok(folder) => std::path::absolute(&folder).unwrap_or(folder),
        Err(e) => return failed(e),
    };
    let registry = match store().and_then(|store| store.load().map_err(|e| e.to_string())) {
        Ok(loaded) => loaded.registry,
        Err(e) => return failed(e),
    };
    let root = registry.data_root();
    let registered = registry
        .projects
        .iter()
        .find(|project| paths::same(&project.folder(&root), &folder));
    let project = registered
        .cloned()
        .unwrap_or_else(|| Project::new(&folder_name(&folder), &folder, State::Active));
    let snapshot = snapshot::take(&folder);
    let claude_md = prompt::claude_md(&folder);
    let today = files::today();
    let mut draft = match prompt::draft(
        &registry,
        &project,
        &snapshot,
        claude_md.as_deref(),
        &[],
        &today,
    ) {
        Ok(draft) => draft,
        Err(e) => return failed(format!("{}: nothing to launch: {e}", project.name)),
    };
    let mode_reason = match mode {
        Some(mode) => {
            draft.resolution.mode.value = mode;
            let resolution = &draft.resolution;
            draft.parts.choose(&Chosen {
                model: &resolution.model.value,
                effort: resolution.effort.value.as_deref(),
                ultracode: resolution.ultracode.value,
                date: &today,
                visual: mode == LaunchMode::Visual,
            });
            draft.prompt = prompt::generate(&draft.parts, project.launch.template.as_deref());
            format!("--{} on the command line", mode.as_str())
        }
        None => draft.resolution.mode.reason(),
    };
    let catalog = Catalog::folder().map_or_else(
        || Catalog::built_in("no profile folder".to_owned()),
        |folder| Catalog::read(&folder),
    );
    let note = if registered.is_some() {
        ""
    } else {
        " (not registered: the global rules and the tool's defaults apply)"
    };
    println!("  project    {}{note}", project.name);
    println!("  folder     {}", folder.display());
    let size = draft.size.map_or("no size", Size::as_str);
    println!(
        "  next       {} ({size}) {}",
        draft.ep,
        one_line(&draft.title, 72)
    );
    let resolution = &draft.resolution;
    println!(
        "  launch     {}: {mode_reason}",
        resolution.mode.value.as_str()
    );
    let named = catalog
        .model(&resolution.model.value)
        .filter(|model| !model.id.eq_ignore_ascii_case(&resolution.model.value))
        .map(|model| format!(" ({}, {})", model.name, model.id))
        .unwrap_or_default();
    println!(
        "  model      {}{named}: {}",
        resolution.model.value,
        resolution.model.reason()
    );
    println!(
        "  effort     {}: {}",
        resolution
            .effort
            .value
            .as_deref()
            .unwrap_or("the CLI's default"),
        resolution.effort.reason()
    );
    println!(
        "  ultracode  {}: {}",
        if resolution.ultracode.value {
            "yes"
        } else {
            "no"
        },
        resolution.ultracode.reason()
    );
    println!(
        "  mode       {}: {}",
        resolution.permission_mode.value,
        resolution.permission_mode.reason()
    );
    let neighbours: Vec<String> = project
        .neighbour_folders(&root)
        .iter()
        .map(|folder| folder.display().to_string())
        .collect();
    println!(
        "  neighbours {}",
        if neighbours.is_empty() {
            "none".to_owned()
        } else {
            neighbours.join(", ")
        }
    );
    for note in resolution.notes.iter().chain(&draft.prompt.problems) {
        println!("  note       {note}");
    }
    let source = match &catalog.source {
        catalog::Source::Cached { surface, fetched } => format!(
            "Claude Code's cache ({surface}{})",
            fetched.map_or(String::new(), |at| format!(
                ", fetched {}",
                at.format("%Y-%m-%d %H:%M UTC")
            ))
        ),
        catalog::Source::BuiltIn(why) => format!("the built-in list ({why})"),
    };
    println!("  catalog    {source}");
    println!(
        "\nThe prompt ({} characters; the launch dialog opens it in its editor):\n",
        draft.prompt.text.chars().count()
    );
    println!("{}", draft.prompt.text);
    ExitCode::SUCCESS
}
