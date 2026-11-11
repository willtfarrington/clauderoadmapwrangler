//! `crw`, the console side of clauderoadmapwrangler (D-2, D-14): `doctor` from EP-1; `scan`,
//! `adopt`, `ignore` and `set` from EP-2, the console side of the registry until the dashboard
//! (EP-6); `roadmap` from EP-3 and EP-4, the parsers' (a project's snapshot, one README or one
//! brief); `tasks`, `tick` and `untick` from EP-5, the owner's items and the inbox; `status` and
//! `setting` from EP-7, the badges and the registry's settings; `prompt` from EP-10, what a launch
//! starts from, worded for either launch mode from EP-11; `dismiss` and `undismiss` from EP-33,
//! the proposals the owner leaves out; the `hook` and `statusline` relays and `signals`, their
//! entries in Claude Code's user settings, from EP-12; `sessions`, the ledger's rows read without
//! a write, from EP-13; `hook --defer` and `hook --answer`, the defer route's decisions, and a
//! project's `answer_timeout`, from EP-14; `notifications`, the window's log read without a
//! write, and the `quiet_hours` setting from EP-15; `revert`, a session undone back to its start,
//! and a project's `revert` switch from EP-15b; `usage`, the meter's readings in the ledger read
//! without a write, and the `usage_endpoint` setting from EP-17; `autopilot`, its state and its
//! chain log, a project's switch and the owner's Pause, with its three settings, from EP-19; the
//! permission mode and the autopilot's limits through `set`, and whether a project's runs would
//! wait in `autopilot`, from EP-19b; a project's standing model and effort through `set`, kept in
//! its rule for every brief, from EP-19c; `install`, the release build copied into the install
//! folder and the window asked to restart into it, with `signals` and `doctor` naming the
//! installed copy, from EP-37; `mail`, the mailman's log read without a write, from EP-20;
//! `formalise`, a project's house-style files previewed and written and the migration prompt of
//! an older roadmap, from EP-21; `send`, the owner's message into a project's inbox, from
//! EP-20c; the prompt hook, `hook` on a `UserPromptSubmit` giving a prompt's `@<project>`
//! mentions their context and `signals install` adding the event, from EP-20b.

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
use crw_core::inbox::{self, Recipient, mailman};
use crw_core::install;
use crw_core::ledger::{self, Ledger, ProjectShare, RowState, SessionRow};
use crw_core::prompt::Chosen;
use crw_core::proposal::{Kind, Proposal};
use crw_core::registry::{LaunchMode, Limit, MAX_DAILY, Project, Settings, State, Store};
use crw_core::revert;
use crw_core::roadmap::{Size, brief, snapshot, table};
use crw_core::session::{self, Preflight};
use crw_core::sessions::agents;
use crw_core::sessions::{Registered, Status, VISUAL_CLAIM, project_of};
use crw_core::signals::install::{self as signal_entries, Hooked, Line, Options};
use crw_core::signals::{self, relay};
use crw_core::tasks::{self, OwnerItem};
use crw_core::version::MIN_CLAUDE_CODE;
use crw_core::{
    files, formalise, git, notify, paths, plugin, process, prompt, proposal, rules, usage,
    workspace,
};

const USAGE: &str = "\
usage: crw <command>

commands:
  doctor                         print the versions of claude, gh, tailscale, code and the extension,
                                 where this exe and the installed copy are and which one the hooks
                                 run, and whether claude would start a headless session (no usage)
  install [--no-restart]         copy the release build of the repository crw was built in (its
                                 target\\release: the window's exe and crw.exe) into the install
                                 folder, %LOCALAPPDATA%\\clauderoadmapwrangler\\bin, each exe there
                                 renamed to a dated previous copy first (a running one included)
                                 and an earlier install's previous copies removed; then ask the
                                 window to restart into it, which it does unless something would
                                 be lost (a headless run of its own that works or waits, a
                                 dialog...): then its File > Install and restart, or the tray's,
                                 does it later; a trial (CRW_HOME) asks no window
  scan                           print the registry: projects, unadopted folders, proposals
  adopt <folder>...              register folders of the data root, write each one's own
                                 <folder>.code-workspace beside it unless one is there, and add
                                 them to the workspace file
  ignore <folder>...             leave folders of the data root out of the unadopted list
  set <project> state <state>    active, paused, maintenance, done, archived or unformalised
  set <project> priority <1-5>   1 is the highest
  set <project> pinned <yes|no>  pinned projects sort first
  set <project> answer_timeout <minutes|none>
                                 how long a headless session's question or permission request
                                 waits for an answer before the run is deferred; none for no limit
  set <project> revert <yes|no>  whether each session's start is recorded there (its HEAD and a
                                 copy of its uncommitted files, in the backups folder) and a
                                 session may be reverted; yes by default
  set <project> permission_mode <mode|default>
                                 the permission mode its sessions start in: acceptEdits by
                                 default, which asks at each shell command without an allow rule,
                                 so the autopilot's runs would wait; auto lets them through
                                 without allow rules, a classifier blocking what goes beyond the
                                 request; plan, dontAsk, bypassPermissions or manual; default
                                 takes the project's own mode out
  set <project> max_size <S|M|L|default>
                                 the largest brief the autopilot chains there; M by default
  set <project> daily <runs|default>
                                 the chained runs a day there, 0 to 100; 2 by default
  set <project> max_budget_usd <dollars|none>
                                 --max-budget-usd for each headless run there while its autopilot
                                 is on, in dollars of the CLI's own estimate; none by default
  set <project> model <id|default>
                                 the model each brief there starts with, kept in the project's
                                 rule for every brief (the launch dialog's Keep for this
                                 project), over the rule table; default takes it out
  set <project> effort <level|default>
                                 the effort each brief there starts with: low, medium, high,
                                 xhigh or max, kept in the same rule; default takes it out
  dismiss <project> [<proposal>] leave a proposed state out until the fact it rests on changes;
                                 every open proposal of the project, or the one named:
                                 all_rows_checked, pickup_gate or no_commit_since
  undismiss <project> [<proposal>]
                                 bring dismissed proposals back
  formalise <project|folder> [--yes]
                                 preview the house-style files a project is given: the rule
                                 block in its CLAUDE.md between the roadmap-wrangler markers,
                                 roadmap/owner-tasks.md and roadmap/inbox.md with their headers,
                                 the brief template roadmap/_TEMPLATE.md; each one created, its
                                 block added or replaced, or kept; with --yes, write them: a file
                                 there is never replaced, and CLAUDE.md changes only between the
                                 markers, after a copy to the backups folder; refused while a
                                 session works or waits there
  formalise <project|folder> --migration
                                 print the prompt of a session that migrates a five-column or
                                 work-packet roadmap to the house style; launches nothing
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
  send <project|folder> <words...> [--urgent]
                                 write a message from you into a project's roadmap/inbox.md, as
                                 `IN-n (from the owner, <date>)`, for its next session to triage;
                                 --urgent flags it; the file as it was is copied first, and the
                                 write is logged for a revert to redo (D-13)
  status [--offline] [<project|folder>]
                                 print the badges of every registered project, or one: stack,
                                 progress, activity, hosting with the head check, CI and license,
                                 backup; asks git, gh and the origins (--offline: git only)
  setting [<key> [<value>]]      print the registry's settings, or set one: data_root,
                                 workspace_file, backup_settings, backup_log, claude_cli, vscode,
                                 port (the window's loopback port for the signals), decisions
                                 (protocol, or defer: a headless run ends at a question and the
                                 answer resumes it), quiet_hours (HH:MM-HH:MM in local time, such
                                 as 22:00-07:00, when the window holds its notifications back;
                                 none for notifications at any hour), usage_endpoint (on, the
                                 default, or off: the window's meter keeps to the status lines),
                                 fable_downshift_percent (1 to 100, or default: 60; above it an S
                                 brief that would run on Fable is proposed Opus 5.5),
                                 autopilot_pause_percent (1 to 100, or default: 90; at it the
                                 weekly or Fable figure holds every chain until its reset),
                                 autopilot_daily_total (chained runs a day in all projects, or
                                 default: 4), autopilot_paused (on or off: your Pause),
                                 ntfy_host and ntfy_topic (your ntfy server, https:// unless
                                 written http://, and its topic: each notification the popup
                                 shows is pushed there too; none takes one out)
  autopilot                      print the autopilot's state: your Pause, the mark, the day's caps,
                                 the projects with it on, their limits and permission modes and
                                 whether their runs would wait at each shell command, and the
                                 chain log's latest lines; reads only
  autopilot <project> <on|off>   turn a project's autopilot on (its runs commit themselves, your
                                 standing yes, and a headless run's completion may chain its next
                                 brief) or off (its runs stop before committing)
  autopilot pause|resume         hold every chain until you resume, or let them go on; the runs
                                 that work go on either way
  prompt <project|folder> [--visual|--headless]
                                 print what a launch of the next brief starts from: the launch
                                 mode, the model, effort, ultracode and permission mode with
                                 their reasons (the rule table), the downshift's judgement on the
                                 window's last usage reading, the neighbour folders and the
                                 prompt, worded for the project's launch mode or the one asked;
                                 launches nothing
  sessions [<project>] [--all]   print the ledger's sessions, the newest first: the last 20, or
                                 every one with --all, of every project or one: when, the session's
                                 short id, the project and EP, the mode, model and effort, how
                                 long, how it ended, the cost estimate and the result's last line;
                                 reads only
  revert <project> [<session id>] [--yes]
                                 preview putting the project back as it was when its latest
                                 session, or the one named (its id or the id's first 8 characters),
                                 started: the target, each commit and file undone, what stays,
                                 the refusals; with --yes, copy the work to the backups folder
                                 first, then reset to the start, remove the session's new files and
                                 put the start's files back. Never pushes; refused while a session
                                 works there or when the remote has one of its commits
  mail [--all]                   print the mailman's log, the newest first: the last 20, or every
                                 row with --all: each project's first reading, the blocks counted
                                 then, each delivery with the message's id (EP-20); reads only
  notifications [--all]          print the window's notifications, the newest first: the last 20,
                                 or every one with --all: each one and what became of it (in the
                                 popup, held back by the quiet hours or the window in front, a
                                 repeat), each one opened with Show, and each badge; reads only
  usage                          print the usage meter's readings from the ledger: the endpoint's
                                 last reading (each limit, its reset, the week by surface), the
                                 marks notified, the latest samples of the endpoint and the
                                 status lines, the meter's last changes of state; reads only, and
                                 never the credentials or the network (only the window polls)
  signals [status]               print the tool's hooks and status line in Claude Code's user
                                 settings, what an install would change, and whether a window
                                 listens for them
  signals install [--inbox|--no-inbox] [--no-status-line]
                                 write the SessionStart, Notification, Stop, SessionEnd and
                                 UserPromptSubmit hooks and the status line into the user
                                 settings, running the
                                 installed crw.exe once there is one (else this crw.exe), after a
                                 dated copy of the file; --inbox has the start
                                 hook print the project's open inbox messages and takes the
                                 stopgap inbox hook out (kept at a later install unless
                                 --no-inbox)
  signals remove                 take the tool's hooks and status line out of the user settings
  hook [--inbox]                 the hooks' relay: posts the hook's input to the window and
                                 prints nothing (with --inbox, a start's open inbox messages);
                                 a prompt that names a registered project as @<project> in a
                                 session you work in is given that project's CLAUDE.md and next
                                 row, which Claude Code adds to the session's context
  hook --defer | --answer <file> the defer route's PreToolUse decisions, which a headless run's
                                 settings install: defer a question, or answer the deferred call
                                 the file names
  statusline                     the status line's relay: posts its input to the window and
                                 prints nothing
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
        ["formalise", rest @ ..] => formalise_project(rest),
        ["roadmap", target] => roadmap(target),
        ["tasks", target] => waiting(target),
        ["tick", target, id, words @ ..] => tick(target, id, words, true),
        ["untick", target, id] => tick(target, id, &[], false),
        ["send", rest @ ..] => send_message(rest),
        ["status", rest @ ..] => status(rest),
        ["prompt", target] => prompt(target, None),
        ["prompt", target, "--visual"] => prompt(target, Some(LaunchMode::Visual)),
        ["prompt", target, "--headless"] => prompt(target, Some(LaunchMode::Headless)),
        ["setting"] => setting(None, None),
        ["setting", key] => setting(Some(key), None),
        ["setting", key, value] => setting(Some(key), Some(value)),
        // The defer route's decisions (D-19; EP-14): printed for Claude Code, never an error, so
        // that the tool never stops a run by failing.
        ["hook", "--defer"] => {
            let _ = hook_input();
            println!("{}", session::defer::defer());
            ExitCode::SUCCESS
        }
        ["hook", "--answer", file] => {
            if let Some(output) = session::defer::answer(&hook_input(), Path::new(file)) {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        // The relays never fail and never print an error: a session must not wait on the tool
        // or show its errors (D-20).
        ["hook", rest @ ..] => {
            let mut out = std::io::stdout().lock();
            let inbox = rest.contains(&"--inbox");
            let _ = signals::relay_hook(
                std::io::stdin().lock(),
                signals::entrypoint(),
                inbox,
                signals::registered,
                &mut out,
                signals::port(),
            );
            ExitCode::SUCCESS
        }
        ["statusline", ..] => {
            let _ = signals::relay_status_line(
                std::io::stdin().lock(),
                signals::entrypoint(),
                signals::port(),
            );
            ExitCode::SUCCESS
        }
        ["autopilot"] => autopilot_state(),
        ["autopilot", "pause"] => autopilot_pause(true),
        ["autopilot", "resume"] => autopilot_pause(false),
        ["autopilot", project, "on"] => autopilot_switch(project, true),
        ["autopilot", project, "off"] => autopilot_switch(project, false),
        ["sessions", rest @ ..] => sessions(rest),
        ["revert", rest @ ..] => revert_session(rest),
        ["notifications", rest @ ..] => notifications(rest),
        ["mail", rest @ ..] => mail(rest),
        ["usage"] => usage(),
        ["signals"] | ["signals", "status"] => signals_status(),
        ["signals", "install", flags @ ..] => signals_install(flags),
        ["signals", "remove"] => signals_remove(),
        ["install", flags @ ..] => install_build(flags),
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

/// A hook's input from stdin, at most 1 MB of it; whatever reads.
fn hook_input() -> String {
    use std::io::Read as _;
    let mut input = String::new();
    let _ = std::io::stdin()
        .lock()
        .take(1 << 20)
        .read_to_string(&mut input);
    input
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
    print_exes();
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
        let plugin_flag = if flags.has(plugin::FLAG) {
            plugin::FLAG.to_owned()
        } else {
            format!("no {}", plugin::FLAG)
        };
        println!("  headless   ready: {efforts}; {ultracode}; {budget}; {plugin_flag}");
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
        "answer_timeout" => match value {
            "none" => store.change(&today, |c| c.set_answer_timeout(project, None)),
            minutes => match minutes.parse::<u32>() {
                Ok(minutes) => {
                    store.change(&today, |c| c.set_answer_timeout(project, Some(minutes)))
                }
                Err(_) => {
                    return usage_error(&format!(
                        "answer_timeout takes minutes or none, not {value:?}"
                    ));
                }
            },
        },
        "revert" => match value {
            "yes" | "true" => store.change(&today, |c| c.set_revert(project, true)),
            "no" | "false" => store.change(&today, |c| c.set_revert(project, false)),
            _ => return usage_error(&format!("revert takes yes or no, not {value:?}")),
        },
        "permission_mode" => match permission_mode_word(value) {
            Ok(mode) => store.change(&today, |c| c.set_permission_mode(project, mode)),
            Err(why) => return usage_error(&why),
        },
        "model" => match model_word(value) {
            Ok(model) => store.change(&today, |c| {
                c.keep(project, &rules::Scope::Project, rules::Key::Model, model)
            }),
            Err(why) => return usage_error(&why),
        },
        "effort" => match effort_word(value) {
            Ok(effort) => store.change(&today, |c| {
                c.keep(project, &rules::Scope::Project, rules::Key::Effort, effort)
            }),
            Err(why) => return usage_error(&why),
        },
        _ => match limit_of(field, value) {
            Some(Ok(limit)) => store.change(&today, |c| c.set_autopilot_limit(project, &limit)),
            Some(Err(why)) => return usage_error(&why),
            None => {
                return usage_error(&format!(
                    "set changes state, priority, pinned, answer_timeout, revert, \
                     permission_mode, max_size, daily, max_budget_usd, model or effort, not \
                     {field:?}"
                ));
            }
        },
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

/// The permission mode `crw set` writes for `value` (D-18; EP-19b): one of the modes the tool
/// knows, in any case, in the tool's spelling; `None` for `default`, which takes the project's
/// own mode out.
fn permission_mode_word(value: &str) -> Result<Option<&'static str>, String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("default") {
        return Ok(None);
    }
    rules::PERMISSION_MODES
        .into_iter()
        .find(|mode| mode.eq_ignore_ascii_case(value))
        .map(Some)
        .ok_or_else(|| {
            format!(
                "permission_mode takes {} or default, not {value:?}",
                rules::PERMISSION_MODES.join(", ")
            )
        })
}

/// The model `crw set` keeps in the project's rule for every brief (D-22; EP-19c): one word, as
/// written, trimmed (an alias such as `fable` or an id); `None` for `default`, in any case, which
/// takes the kept model out.
fn model_word(value: &str) -> Result<Option<&str>, String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("default") {
        return Ok(None);
    }
    if value.is_empty() || value.contains(char::is_whitespace) {
        return Err(format!(
            "model takes one word, a model's alias or id, or default, not {value:?}"
        ));
    }
    Ok(Some(value))
}

/// The effort `crw set` keeps in the project's rule for every brief (D-22; EP-19c): one of the
/// built-in levels, in any case, in its lowercase spelling; `None` for `default`, which takes the
/// kept effort out.
fn effort_word(value: &str) -> Result<Option<&'static str>, String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("default") {
        return Ok(None);
    }
    rules::EFFORTS
        .into_iter()
        .find(|level| level.eq_ignore_ascii_case(value))
        .map(Some)
        .ok_or_else(|| {
            format!(
                "effort takes {} or default, not {value:?}",
                rules::EFFORTS.join(", ")
            )
        })
}

/// The autopilot's limit `crw set` writes for `field` and `value` (D-22; EP-19b): `max_size` S,
/// M or L in any case, `daily` runs from 0 to [`MAX_DAILY`], `max_budget_usd` dollars above 0
/// with or without a `$`, each back to its default with `default` (the budget with `none` too).
/// A value that is the default is taken out of the registry as `default` is, as the window's
/// pickers do; `None` when `field` is not one of the limits.
fn limit_of(field: &str, value: &str) -> Option<Result<Limit, String>> {
    use crw_core::autopilot::{DEFAULT_DAILY, DEFAULT_MAX_SIZE};

    let value = value.trim();
    let default = value.eq_ignore_ascii_case("default");
    let size = |size: Size| Limit::MaxSize((size != DEFAULT_MAX_SIZE).then_some(size));
    Some(match field {
        "max_size" => match value.to_ascii_uppercase().as_str() {
            _ if default => Ok(Limit::MaxSize(None)),
            "S" => Ok(size(Size::S)),
            "M" => Ok(size(Size::M)),
            "L" => Ok(size(Size::L)),
            _ => Err(format!(
                "max_size takes S, M or L, or default ({DEFAULT_MAX_SIZE}), not {value:?}"
            )),
        },
        "daily" => match value.parse::<u32>() {
            _ if default => Ok(Limit::Daily(None)),
            Ok(runs) if runs <= MAX_DAILY => {
                Ok(Limit::Daily((runs != DEFAULT_DAILY).then_some(runs)))
            }
            _ => Err(format!(
                "daily takes a number of runs from 0 to {MAX_DAILY}, or default \
                 ({DEFAULT_DAILY}), not {value:?}"
            )),
        },
        "max_budget_usd" if default || value.eq_ignore_ascii_case("none") => {
            Ok(Limit::Budget(None))
        }
        "max_budget_usd" => {
            let dollars = value.strip_prefix('$').unwrap_or(value).trim();
            match dollars.parse::<f64>() {
                Ok(dollars) if dollars.is_finite() && dollars > 0.0 => {
                    Ok(Limit::Budget(Some(dollars)))
                }
                _ => Err(format!(
                    "max_budget_usd takes dollars above 0, or none (the default), not {value:?}"
                )),
            }
        }
        _ => return None,
    })
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

/// `crw autopilot` (D-22; EP-19): the owner's Pause, the mark, the day's caps, the projects with
/// the autopilot on, their limits and permission modes and whether their runs would wait at each
/// shell command (EP-19b), and the chain log's latest lines, read without a write.
fn autopilot_state() -> ExitCode {
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    let registry = match store.load() {
        Ok(loaded) => loaded.registry,
        Err(e) => return failed(e),
    };
    let settings = &registry.settings;
    println!(
        "autopilot: {}; it holds every chain at {} % of the weekly or the Fable limit until the \
         reset; {} chained runs a day in all projects",
        if settings.autopilot_paused() {
            "paused by you (crw autopilot resume)"
        } else {
            "not paused"
        },
        settings.autopilot_pause_percent(),
        settings.autopilot_daily_total()
    );
    let on: Vec<_> = registry
        .projects
        .iter()
        .filter(|project| project.autopilot.enabled)
        .collect();
    if on.is_empty() {
        println!("  no project has it on (crw autopilot <project> on)");
    }
    // The gate's waiting condition reads the project's settings files and the user's (EP-19b).
    let home = std::env::var_os("USERPROFILE").map(PathBuf::from);
    let data_root = registry.data_root();
    for project in on {
        let autopilot = &project.autopilot;
        let mode = rules::permission_mode(project).value;
        let folder = project.folder(&data_root);
        let waits = crw_core::autopilot::standing(project, &folder, home.as_deref())
            .iter()
            .any(|refusal| matches!(refusal, crw_core::autopilot::Refusal::Waits(_)));
        let waits = if waits {
            format!(
                " (its runs would wait at each shell command, so the gate refuses its chains: \
                 crw set {} permission_mode auto, or allow rules)",
                project.name
            )
        } else {
            String::new()
        };
        let budget = autopilot
            .max_budget_usd
            .map_or(String::new(), |usd| format!(", --max-budget-usd {usd}"));
        println!(
            "  {}: on, briefs up to {}, {} chained runs a day, permission mode \
             {mode}{waits}{budget}",
            project.name,
            autopilot.max_size(),
            autopilot.daily()
        );
    }
    let Some(path) = paths::ledger_file() else {
        return ExitCode::SUCCESS;
    };
    match crw_core::autopilot::recent(&path, 20) {
        Ok(rows) if rows.is_empty() => println!("the chain log is empty"),
        Ok(rows) => {
            println!("the chain log, the newest first:");
            for row in rows {
                let at = row
                    .at()
                    .map(|at| {
                        at.with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                    })
                    .unwrap_or_default();
                let project = row
                    .project
                    .as_deref()
                    .map_or(String::new(), |name| format!("{name}: "));
                println!("  {at}  {project}{}", row.detail.unwrap_or_default());
            }
        }
        Err(e) => return failed(format!("the chain log did not read: {e}")),
    }
    ExitCode::SUCCESS
}

/// `crw autopilot <project> on|off`: the project's switch, logged in the chain log.
fn autopilot_switch(project: &str, on: bool) -> ExitCode {
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    match store.change(&files::today(), |c| c.set_autopilot(project, on)) {
        Ok((registry, ())) => {
            let name = registry
                .project(project)
                .map_or(project.to_owned(), |p| p.name.clone());
            if on {
                println!(
                    "{name}: autopilot on: its runs commit themselves (your standing yes), and \
                     after a headless run completes the gate may chain its next brief"
                );
            } else {
                println!("{name}: autopilot off: its runs stop before committing");
            }
            println!("note: a window that runs reads it at the next run's end");
            let words = if on {
                "turned on by the owner"
            } else {
                "turned off by the owner"
            };
            log_autopilot(Some(&name), words);
            ExitCode::SUCCESS
        }
        Err(e) => failed(e),
    }
}

/// `crw autopilot pause|resume`: the owner's Pause of every chain, logged in the chain log.
fn autopilot_pause(paused: bool) -> ExitCode {
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    let value = if paused { "on" } else { "off" };
    match store.change(&files::today(), |c| {
        c.set_setting("autopilot_paused", value)
    }) {
        Ok(_) => {
            if paused {
                println!("autopilot paused: no brief is chained until crw autopilot resume");
                log_autopilot(None, crw_core::autopilot::PAUSED_BY_OWNER);
            } else {
                println!("autopilot resumed: a window that runs reads its held chains again");
                log_autopilot(None, crw_core::autopilot::RESUMED_BY_OWNER);
            }
            ExitCode::SUCCESS
        }
        Err(e) => failed(e),
    }
}

/// A line of the chain log, when the ledger exists; a failure is said and changes nothing else.
fn log_autopilot(project: Option<&str>, words: &str) {
    if let Some(path) = paths::ledger_file().filter(|path| path.exists())
        && let Err(e) = crw_core::autopilot::log(&path, project, words, chrono::Utc::now())
    {
        eprintln!("note: the chain log did not keep it: {e}");
    }
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
            Ok((registry, ())) if key == "quiet_hours" => {
                match registry.settings.get(key) {
                    Some(hours) => println!("quiet_hours = {hours}"),
                    None => println!("quiet_hours: none, notifications at any hour"),
                }
                println!("note: a window that runs reads it before its next notification");
                ExitCode::SUCCESS
            }
            Ok((registry, ())) if key == "ntfy_host" || key == "ntfy_topic" => {
                match registry.settings.get(key) {
                    Some(set) => println!("{key} = {set}"),
                    None => println!("{key}: none"),
                }
                match registry.settings.ntfy() {
                    Some(Ok(ntfy)) => println!(
                        "note: a window that runs pushes each notification its popup shows to {} \
                         from the next one; File > Send a test notification checks it",
                        ntfy.server()
                    ),
                    Some(Err(why)) => println!("note: ntfy stays off: {why}"),
                    None => {
                        println!("note: ntfy stays off until ntfy_host and ntfy_topic are both set")
                    }
                }
                ExitCode::SUCCESS
            }
            Ok((registry, ())) if key == "usage_endpoint" => {
                if registry.settings.usage_endpoint_on() {
                    println!("usage_endpoint = on (the default)");
                } else {
                    println!("usage_endpoint = off: the meter shows the status lines' figures");
                }
                println!("note: a window that runs reads it at its next poll");
                ExitCode::SUCCESS
            }
            Ok((registry, ())) if key == "fable_downshift_percent" => {
                let mark = registry.settings.fable_downshift_percent();
                let default = if registry.settings.fable_downshift_percent.is_none() {
                    " (the default)"
                } else {
                    ""
                };
                println!("fable_downshift_percent = {mark}{default}");
                println!("note: the window weighs it at the next launch dialog it opens");
                ExitCode::SUCCESS
            }
            Ok((registry, ())) => {
                println!("{key} = {value}");
                let full = registry.data_root().join(value);
                if Settings::is_path(key) && key != "data_root" && !full.exists() {
                    println!("note: {} does not exist yet", full.display());
                }
                if key == "port" {
                    println!(
                        "note: a window that runs listens on its port until it is started again"
                    );
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
            (Some(value), _) => value,
            (None, "data_root") => format!("{} (the default)", registry.data_root().display()),
            (None, "port") => format!("{} (the default)", registry.port()),
            (None, "decisions") => format!("{} (the default)", session::Route::default().as_str()),
            (None, "quiet_hours") => "none (the default: notifications at any hour)".to_owned(),
            (None, "usage_endpoint") => "on (the default: the window polls it)".to_owned(),
            (None, "ntfy_host" | "ntfy_topic") => {
                "none (the default: nothing is pushed to a phone)".to_owned()
            }
            (None, "fable_downshift_percent") => format!(
                "{} (the default: above it an S brief on Fable is proposed Opus 5.5)",
                rules::DEFAULT_DOWNSHIFT_PERCENT
            ),
            (None, _) => "not set".to_owned(),
        };
        println!("  {name:<23} {value}");
    }
    ExitCode::SUCCESS
}

/// How many of the latest notifications `crw notifications` prints without `--all`.
const NOTIFICATIONS_SHOWN: usize = 20;

/// Prints the window's notifications from the ledger (D-21; EP-15), the newest first, read
/// without migrating or writing it: each one and what became of it, each Show, each badge.
fn notifications(args: &[&str]) -> ExitCode {
    events(
        "notifications",
        notify::EVENT_KIND,
        "the window logs each notification and what became of it, each Show and each badge",
        args,
    )
}

/// Prints the mailman's log from the ledger (D-13; EP-20), the newest first, read without
/// migrating or writing it: each project's first reading, the blocks counted then, each delivery.
fn mail(args: &[&str]) -> ExitCode {
    events(
        "mail",
        mailman::EVENT_KIND,
        "the window's mailman logs each project's first reading and each message it delivers",
        args,
    )
}

/// Prints the ledger's `event` rows of `kind` as `crw <command>`, the latest
/// [`NOTIFICATIONS_SHOWN`] or every one with `--all`.
fn events(command: &str, kind: &str, none_yet: &str, args: &[&str]) -> ExitCode {
    let all = match args {
        [] => false,
        ["--all"] => true,
        _ => {
            return usage_error(&format!("{command} takes --all, not {}", args.join(" ")));
        }
    };
    let Some(path) = paths::ledger_file() else {
        return failed("no profile folder: LOCALAPPDATA is not set");
    };
    if !path.exists() {
        println!(
            "No ledger yet at {}: the window makes it at its first start.",
            path.display()
        );
        return ExitCode::SUCCESS;
    }
    let limit = (!all).then_some(NOTIFICATIONS_SHOWN);
    let rows = match Ledger::open_read_only(&path).and_then(|ledger| ledger.events(kind, limit)) {
        Ok(rows) => rows,
        Err(e) => return failed(format!("{}: {e}", path.display())),
    };
    let which = match (rows.len(), all) {
        (0, _) => "none yet".to_owned(),
        (n, true) => format!("all {n}, the newest first"),
        (n, false) => format!("the latest {n}, the newest first"),
    };
    println!(
        "crw {} {command}: {which} ({})",
        env!("CARGO_PKG_VERSION"),
        path.display()
    );
    if rows.is_empty() {
        println!("  {none_yet}");
    }
    let width = rows
        .iter()
        .map(|row| row.project.as_deref().unwrap_or("-").chars().count())
        .max()
        .unwrap_or(0);
    for row in &rows {
        let when = row.at().map_or(row.at.clone(), |at| {
            at.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        });
        let project = row.project.as_deref().unwrap_or("-");
        let detail = row.detail.as_deref().unwrap_or_default();
        println!("  {when}  {project:<width$}  {detail}");
    }
    ExitCode::SUCCESS
}

/// How many of the latest usage samples `crw usage` prints, and of the meter's changes of state.
const SAMPLES_SHOWN: usize = 12;
const SHIFTS_SHOWN: usize = 5;

/// Prints the usage meter's readings from the ledger (D-23; EP-17), read without migrating or
/// writing it, and without the credentials or the network: only the window polls.
fn usage() -> ExitCode {
    let Some(path) = paths::ledger_file() else {
        return failed("no profile folder: LOCALAPPDATA is not set");
    };
    if !path.exists() {
        println!(
            "No ledger yet at {}: the window makes it at its first start.",
            path.display()
        );
        return ExitCode::SUCCESS;
    }
    let read = Ledger::open_read_only(&path).and_then(|ledger| {
        Ok((
            ledger.cached_one("", usage::CACHE_SOURCE)?,
            ledger.cached_one("", usage::ALERTS_SOURCE)?,
            ledger.latest_samples(SAMPLES_SHOWN)?,
            ledger.events(usage::EVENT_KIND, Some(SHIFTS_SHOWN))?,
        ))
    });
    let (reading, alerts, samples, shifts) = match read {
        Ok(read) => read,
        Err(e) => return failed(format!("{}: {e}", path.display())),
    };
    let now = Utc::now();
    let local = |at: DateTime<Utc>| {
        at.with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    };
    println!(
        "crw {} usage ({})",
        env!("CARGO_PKG_VERSION"),
        path.display()
    );
    match reading
        .as_ref()
        .and_then(|row| Some((row, usage::Reading::from_json(&row.data)?)))
    {
        None => println!("  no reading of the endpoint yet: the window polls it every 5 minutes"),
        Some((row, reading)) => {
            println!(
                "  the endpoint's last reading, {} ({}):",
                local(row.fetched_at),
                match short_age(now - row.fetched_at).as_str() {
                    "now" => "just now".to_owned(),
                    age => format!("{age} ago"),
                }
            );
            let mut limits = Vec::new();
            if let Some(weekly) = &reading.weekly {
                limits.push(("weekly".to_owned(), weekly.clone()));
            }
            for scoped in &reading.scoped {
                limits.push((scoped.name.clone(), scoped.limit.clone()));
            }
            // A `limits[]` kind the parser does not know, by its name.
            for other in &reading.other {
                limits.push((other.name.clone(), other.limit.clone()));
            }
            if let Some(five_hour) = &reading.five_hour {
                limits.push(("five-hour".to_owned(), five_hour.clone()));
            }
            let width = limits
                .iter()
                .map(|(name, _)| name.chars().count())
                .max()
                .unwrap_or(0);
            for (name, limit) in &limits {
                let resets = limit.resets_at.map_or_else(
                    || "no reset given".to_owned(),
                    |at| format!("resets {} (in {})", local(at), usage::countdown(at, now)),
                );
                println!(
                    "    {name:<width$}  {:>6}  {resets}",
                    usage::percent(limit.percent)
                );
            }
            if !reading.surfaces.is_empty() {
                let parts: Vec<String> = reading
                    .surfaces
                    .iter()
                    .map(|surface| format!("{} {}", surface.name, usage::percent(surface.percent)))
                    .collect();
                println!("    by surface: {}", parts.join(", "));
            }
            if let Some(extra) = &reading.extra {
                let state = match extra.enabled {
                    Some(true) => "on",
                    Some(false) => "off",
                    None => "not given",
                };
                println!("    extra usage: {state}");
            }
            if !reading.shape.is_empty() {
                println!("    the answer's shape: {}", reading.shape);
            }
        }
    }
    let alerts = alerts
        .and_then(|row| usage::Alerts::from_json(&row.data))
        .unwrap_or_default();
    let alerted = |alerted: &Option<usage::Alerted>| {
        alerted
            .as_ref()
            .map_or_else(|| "none".to_owned(), |alerted| usage::percent(alerted.mark))
    };
    println!(
        "  notified in this window: weekly {}, Fable {}",
        alerted(&alerts.weekly),
        alerted(&alerts.fable)
    );
    match samples.len() {
        0 => println!("  no samples yet"),
        1 => println!("  the latest sample:"),
        n => println!("  the latest {n} samples, the newest first:"),
    }
    for sample in &samples {
        let figure = |name: &str, percent: Option<f64>| {
            percent.map(|percent| format!("{name} {}", usage::percent(percent)))
        };
        let figures: Vec<String> = [
            figure("weekly", sample.weekly_percent),
            figure("Fable", sample.fable_percent),
            figure("5 h", sample.five_hour_percent),
        ]
        .into_iter()
        .flatten()
        .collect();
        let session = sample
            .session_id
            .as_deref()
            .map(|id| format!("  (session {})", id.chars().take(8).collect::<String>()))
            .unwrap_or_default();
        println!(
            "    {}  {:<10}  {}{session}",
            local(sample.taken_at),
            sample.source,
            figures.join("  ")
        );
    }
    if !shifts.is_empty() {
        println!("  the meter's last changes of state, the newest first:");
    }
    for row in &shifts {
        let when = row.at().map_or(row.at.clone(), local);
        println!("    {when}  {}", row.detail.as_deref().unwrap_or_default());
    }
    ExitCode::SUCCESS
}

/// How many of the latest sessions `crw sessions` prints without `--all`.
const SESSIONS_SHOWN: usize = 20;

/// Prints the ledger's sessions (D-26; EP-13), the newest first, read without migrating or
/// writing it: a visual launch past its 30 minutes reads as lapsed before a window closes it.
fn sessions(args: &[&str]) -> ExitCode {
    let all = args.contains(&"--all");
    if let Some(flag) = args
        .iter()
        .find(|arg| arg.starts_with("--") && **arg != "--all")
    {
        return usage_error(&format!("sessions takes --all, not {flag}"));
    }
    let names: Vec<&str> = args
        .iter()
        .copied()
        .filter(|arg| !arg.starts_with("--"))
        .collect();
    let project = match names.as_slice() {
        [] => None,
        [one] => Some(*one),
        _ => return usage_error("sessions takes one project at most"),
    };
    let Some(path) = paths::ledger_file() else {
        return failed("no profile folder: LOCALAPPDATA is not set");
    };
    if !path.exists() {
        println!(
            "No ledger yet at {}: the window makes it at its first start.",
            path.display()
        );
        return ExitCode::SUCCESS;
    }
    let now = Utc::now();
    let read = Ledger::open_read_only(&path).and_then(|ledger| {
        // The weekly window the shares are summed over (EP-18a): its reset is the window's
        // cached reading's, read without the credentials or the network.
        let since = ledger
            .cached_one("", usage::CACHE_SOURCE)?
            .and_then(|row| usage::Reading::from_json(&row.data))
            .and_then(|reading| reading.weekly?.resets_at)
            .map(|resets_at| usage::share::week_start(resets_at, now));
        let week = match since {
            Some(since) => Some((since, ledger.shares_since(since)?)),
            None => None,
        };
        Ok((ledger.sessions()?, week))
    });
    let (rows, week) = match read {
        Ok(read) => read,
        Err(e) => return failed(format!("{}: {e}", path.display())),
    };
    let mut rows: Vec<SessionRow> = rows
        .into_iter()
        .rev()
        .filter(|row| project.is_none_or(|name| row.project.eq_ignore_ascii_case(name)))
        .collect();
    let total = rows.len();
    if !all {
        rows.truncate(SESSIONS_SHOWN);
    }
    let whose = project.map_or("every project".to_owned(), str::to_owned);
    println!(
        "crw {} sessions: {} of {total} for {whose}, the newest first ({})",
        env!("CARGO_PKG_VERSION"),
        rows.len(),
        path.display()
    );
    print_week(week, project);
    let lines: Vec<[String; 10]> = rows
        .iter()
        .map(|row| {
            let when = row.started().map_or(row.started_at.clone(), |at| {
                at.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            });
            // What `crw revert <project> <id>` takes (EP-15b).
            let id = row
                .session_id
                .as_deref()
                .map_or("-".to_owned(), |id| revert::short(id).to_owned());
            let title = match &row.ep {
                Some(ep) => format!("{} {ep}", row.project),
                None => row.project.clone(),
            };
            let model = match (&row.model, &row.effort) {
                (Some(model), Some(effort)) => format!("{model}/{effort}"),
                (Some(model), None) => model.clone(),
                _ => "-".to_owned(),
            };
            let state = match row.state(now, VISUAL_CLAIM) {
                RowState::Ended(outcome) => outcome.to_owned(),
                RowState::Lapsed => "lapsed: no session took it up".to_owned(),
                RowState::Launched => "handed to VS Code, not taken up yet".to_owned(),
                RowState::Open => "open: no end recorded".to_owned(),
            };
            let cost = row
                .cost_usd
                .map_or(String::new(), |cost| format!("${cost:.2}"));
            [
                when,
                id,
                title,
                row.mode.clone(),
                model,
                row.took().map_or("-".to_owned(), ledger::length),
                state,
                cost,
                ledger::asked(row).unwrap_or_default(),
                ledger::shares(row).unwrap_or_default(),
            ]
        })
        .collect();
    let mut widths = [0usize; 10];
    for line in &lines {
        for (width, cell) in widths.iter_mut().zip(line) {
            *width = (*width).max(cell.chars().count());
        }
    }
    for (row, line) in rows.iter().zip(&lines) {
        let cells: Vec<String> = line
            .iter()
            .zip(widths)
            .map(|(cell, width)| format!("{cell:<width$}"))
            .collect();
        println!("  {}", cells.join("  ").trim_end());
        if let Some(result) = &row.result {
            println!("      {}", one_line(result, 100));
        }
    }
    if !all && total > rows.len() {
        println!("  … {} older: crw sessions --all", total - rows.len());
    }
    ExitCode::SUCCESS
}

/// Each project's share of the usage over the weekly window (D-23; EP-18a), or `project`'s alone:
/// the sums of its sessions started since the window opened.
fn print_week(week: Option<(DateTime<Utc>, Vec<ProjectShare>)>, project: Option<&str>) {
    let Some((since, sums)) = week else {
        println!("  no reading of the endpoint yet, so no week to sum the shares over");
        return;
    };
    let since = since.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M");
    let sums: Vec<ProjectShare> = sums
        .into_iter()
        .filter(|sum| project.is_none_or(|name| sum.project.eq_ignore_ascii_case(name)))
        .collect();
    if sums.is_empty() {
        println!("  the week since {since}: no session has a share of the usage yet");
        return;
    }
    println!("  the week since {since}, each project's share of the usage:");
    let width = sums
        .iter()
        .map(|sum| sum.project.chars().count())
        .max()
        .unwrap_or(0);
    for sum in &sums {
        let figures: Vec<String> = [("week", sum.weekly), ("Fable", sum.fable)]
            .into_iter()
            .filter_map(|(name, share)| Some(format!("{name} +{} %", ledger::points(share?))))
            .collect();
        let sessions = match sum.sessions {
            1 => "1 session".to_owned(),
            n => format!("{n} sessions"),
        };
        println!(
            "    {:<width$}  {}, over {sessions}",
            sum.project,
            figures.join(" · ")
        );
    }
}

/// Previews the revert of a project's latest session, or of the one `args` names, and reverts it
/// with `--yes` (EP-15b): the work first copied to the backups folder, then the project reset to
/// the session's start. Refused while `claude agents --json` lists a session at work there, or
/// when it does not answer.
fn revert_session(args: &[&str]) -> ExitCode {
    let yes = args.contains(&"--yes");
    if let Some(flag) = args
        .iter()
        .find(|arg| arg.starts_with("--") && **arg != "--yes")
    {
        return usage_error(&format!("revert takes --yes, not {flag}"));
    }
    let words: Vec<&str> = args
        .iter()
        .copied()
        .filter(|arg| !arg.starts_with("--"))
        .collect();
    let (target, id) = match words.as_slice() {
        [target] => (*target, None),
        [target, id] => (*target, Some(*id)),
        _ => return usage_error("revert takes a project and, at most, a session id"),
    };
    println!("crw {} revert", env!("CARGO_PKG_VERSION"));
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    let registry = match store.load() {
        Ok(loaded) => loaded.registry,
        Err(e) => return failed(e),
    };
    let root = registry.data_root();
    let project = match registry.project(target) {
        Some(project) => project.clone(),
        None => {
            let found = project_folder(target).ok().and_then(|folder| {
                registry
                    .projects
                    .iter()
                    .find(|project| paths::same(&project.folder(&root), &folder))
                    .cloned()
            });
            match found {
                Some(project) => project,
                None => return failed(format!("{target} is not a registered project")),
            }
        }
    };
    let folder = project.folder(&root);
    let Some(path) = paths::ledger_file() else {
        return failed("no profile folder: LOCALAPPDATA is not set");
    };
    if !path.exists() {
        return failed(format!(
            "no ledger yet at {}: no session of {} is recorded",
            path.display(),
            project.name
        ));
    }
    let read = match Ledger::open_read_only(&path) {
        Ok(read) => read,
        Err(e) => return failed(format!("{}: {e}", path.display())),
    };
    let rows = match read.sessions() {
        Ok(rows) => rows,
        Err(e) => return failed(format!("{}: {e}", path.display())),
    };
    let chosen = match revert::choose(&rows, &project.name, id) {
        Ok(chosen) => chosen,
        Err(why) => return failed(why),
    };
    let ticks = match chosen.first.started() {
        Some(since) => match revert::ticks_since(&read, &project.name, since) {
            Ok(ticks) => ticks,
            Err(why) => return failed(why),
        },
        None => Vec::new(),
    };
    drop(read);
    let (working, idle) = live_sessions(&registry, &folder);
    let cx = revert::Context {
        project: &project.name,
        folder: &folder,
        enabled: project.revert,
        working,
        idle,
        ticks,
        backups: &store.backups,
        now: Utc::now(),
    };
    let plan = revert::plan(&chosen, &cx);
    for line in plan.lines() {
        println!("{line}");
    }
    if plan.is_refused() {
        return ExitCode::FAILURE;
    }
    if !yes {
        let session = plan.session_id.as_deref().map_or("", revert::short);
        println!(
            "Nothing changed. `crw revert {} {session} --yes` reverts.",
            project.name
        );
        return ExitCode::SUCCESS;
    }
    match revert::revert(&plan, Some(&path), &store.backups, &files::today()) {
        Ok(done) => {
            for line in done.lines() {
                println!("{line}");
            }
            if done.problems.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(why) => failed(why),
    }
}

/// The variable Claude Code gives the commands its tools run: the session's id (seen in 2.1.316,
/// EP-22).
const CALLER: &str = "CLAUDE_CODE_SESSION_ID";

/// The session that runs this command through its tools, if one does.
fn caller() -> Option<String> {
    std::env::var(CALLER)
        .ok()
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty())
}

/// The sessions `claude agents --json` lists in `folder`, in words (EP-15b): those that work or
/// wait there, each of which refuses a revert, and those open with nothing to do. A listing
/// that does not answer refuses it too: the revert must know that nothing works there.
fn live_sessions(
    registry: &crw_core::registry::Registry,
    folder: &Path,
) -> (Vec<String>, Vec<String>) {
    live_sessions_but(registry, folder, None)
}

/// [`live_sessions`] with the session `caller` left out: the formalise step's, run by a session
/// in the project it writes into, whose own turn waits for it (EP-22; the plugin's formalise
/// skill), while any other session at work there still refuses the write.
fn live_sessions_but(
    registry: &crw_core::registry::Registry,
    folder: &Path,
    caller: Option<&str>,
) -> (Vec<String>, Vec<String>) {
    let Some(program) = session::cli::locate(registry.settings.claude_cli.as_deref()) else {
        return (
            vec!["no Claude Code CLI was found to ask whether a session works there".to_owned()],
            Vec::new(),
        );
    };
    let agents = match agents::ask(&program) {
        Ok(agents) => agents,
        Err(why) => {
            return (
                vec![format!(
                    "{why}, so whether a session works there is not known"
                )],
                Vec::new(),
            );
        }
    };
    sessions_in(&agents, folder, caller)
}

/// The listed sessions in `folder`, in words: those that work or wait there, and those open with
/// nothing to do; the session `caller` is left out.
fn sessions_in(
    agents: &[agents::Agent],
    folder: &Path,
    caller: Option<&str>,
) -> (Vec<String>, Vec<String>) {
    let here = [Registered {
        name: String::new(),
        folder: folder.to_owned(),
        revert: true,
    }];
    let (mut working, mut idle) = (Vec::new(), Vec::new());
    for agent in agents {
        if project_of(&here, agent.folder()).is_none() {
            continue;
        }
        if caller.is_some() && agent.session_id.as_deref() == caller {
            continue;
        }
        let id = agent.session_id.as_deref().map_or("-", revert::short);
        match agent.status_now() {
            Some((status @ (Status::Busy | Status::Waiting), waiting_for)) => {
                let what = waiting_for
                    .map(|what| format!(": {what}"))
                    .unwrap_or_default();
                working.push(format!("session {id} is {}{what}", status.as_str()));
            }
            Some((Status::Idle, _)) => idle.push(format!("session {id}")),
            _ => {}
        }
    }
    (working, idle)
}

/// The user settings and the `crw.exe` the hooks and the status line are to run: the installed
/// copy once there is one (EP-37), else this exe.
fn signals_places() -> Result<(PathBuf, PathBuf), String> {
    let settings = signal_entries::settings_file()
        .ok_or("no profile folder: neither CLAUDE_CONFIG_DIR nor USERPROFILE is set")?;
    let exe = match installed_cli() {
        Some(installed) => installed,
        None => this_exe()?,
    };
    Ok((settings, exe))
}

fn this_exe() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("this exe's path did not read: {e}"))
}

/// The installed `crw.exe`, when there is one (EP-37).
fn installed_cli() -> Option<PathBuf> {
    install::bin_dir()
        .map(|bin| bin.join(install::CLI_EXE))
        .filter(|path| path.is_file())
}

/// A `crw.exe` the entries run, in words: the installed copy, this exe, or its path.
fn exe_words(theirs: &Path, this: Option<&Path>) -> String {
    if installed_cli().is_some_and(|installed| paths::same(theirs, &installed)) {
        "the installed copy".to_owned()
    } else if this.is_some_and(|this| paths::same(theirs, this)) {
        "this exe".to_owned()
    } else {
        theirs.display().to_string()
    }
}

/// Prints the tool's entries in Claude Code's user settings (D-20, D-23), whether a window listens
/// for them, and what an install would change: the entries run the installed copy once there is
/// one (EP-37), else this exe; reads, never writes.
fn signals_status() -> ExitCode {
    println!("crw {} signals", env!("CARGO_PKG_VERSION"));
    let (path, exe) = match signals_places() {
        Ok(places) => places,
        Err(e) => return failed(e),
    };
    let this = this_exe().ok();
    // One column wide enough for the longest event's name, UserPromptSubmit (EP-20b).
    println!("  {:<18}{}", "settings", path.display());
    println!(
        "  {:<18}{}",
        "this exe",
        this.as_ref()
            .map_or_else(|| "?".to_owned(), |this| this.display().to_string())
    );
    match installed_cli() {
        Some(installed) => println!("  {:<18}{}", "installed", installed.display()),
        None => println!(
            "  {:<18}none: crw install copies the release build there",
            "installed"
        ),
    }
    let status = match signal_entries::status(&path) {
        Ok(status) => status,
        Err(e) => return failed(e),
    };
    let which = |theirs: &Path| exe_words(theirs, this.as_deref());
    for (event, hooked) in &status.hooks {
        let text = match hooked {
            Hooked::Missing => "not installed".to_owned(),
            Hooked::Ours { exe, inbox } => format!(
                "runs {} hook{}",
                which(exe),
                if *inbox { " --inbox" } else { "" }
            ),
            Hooked::Several => {
                "more than one entry of the tool's: an install puts one back".to_owned()
            }
        };
        println!("  {event:<18}{text}");
    }
    let line = match &status.line {
        Line::Missing => "none".to_owned(),
        Line::Ours(theirs) => format!("runs {} statusline", which(theirs)),
        Line::Other(command) => format!("another command, which an install leaves: {command}"),
    };
    println!("  {:<18}{line}", "status line");
    if status.stopgap {
        println!(
            "  {:<18}the stopgap SessionStart hook prints the inbox (install --inbox replaces it)",
            "stopgap"
        );
    }
    let port = signals::port();
    let window = match relay::request(
        port,
        "GET",
        signals::HEALTH_ROUTE,
        None,
        std::time::Duration::from_millis(300),
    ) {
        Ok(answer) if answer.status == 200 => format!("listening on 127.0.0.1:{port}"),
        Ok(answer) => format!("127.0.0.1:{port} answers {}: not the window", answer.status),
        Err(_) => format!("no window listens on 127.0.0.1:{port}"),
    };
    println!("  {:<18}{window}", "window");
    let options = Options {
        status_line: true,
        inbox: status.stopgap || status.prints_inbox(),
    };
    match signal_entries::preview(&path, &exe, options) {
        Ok((changes, _)) if changes.is_empty() => {
            println!(
                "\nThe entries are installed and run {}.",
                exe_words(&exe, this.as_deref())
            );
        }
        Ok((changes, kept)) => {
            let flag = if options.inbox { " --inbox" } else { "" };
            println!(
                "\nWhat crw signals install{flag} would change, after a dated copy of the file:"
            );
            for change in &changes {
                println!("  - {change}");
            }
            println!(
                "{}",
                if kept {
                    "The file's other keys, their order and its layout stay as written."
                } else {
                    "The file's other keys stay, but its layout is not the two-space JSON Claude \
                     Code writes: an install rewrites it so."
                }
            );
        }
        Err(e) => println!(
            "\nAn install of {} is refused: {e}",
            exe_words(&exe, this.as_deref())
        ),
    }
    ExitCode::SUCCESS
}

/// Writes the tool's hooks and status line into Claude Code's user settings, after a dated copy of
/// the file in the backups folder (D-20, D-23; the owner's yes, rule 7).
fn signals_install(flags: &[&str]) -> ExitCode {
    let (path, exe) = match signals_places() {
        Ok(places) => places,
        Err(e) => return failed(e),
    };
    // The start hook keeps printing the inbox once it does, unless told otherwise: the stopgap
    // that did it before is gone by then.
    let prints_inbox = signal_entries::status(&path).is_ok_and(|status| status.prints_inbox());
    let mut options = Options {
        status_line: true,
        inbox: prints_inbox,
    };
    for flag in flags {
        match *flag {
            "--inbox" => options.inbox = true,
            "--no-inbox" => options.inbox = false,
            "--no-status-line" => options.status_line = false,
            other => return usage_error(&format!("signals install takes no {other}")),
        }
    }
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    match signal_entries::install(&path, &exe, options, &store.backups, &files::today()) {
        Ok(report) => {
            print_report(
                &report,
                "The entries were installed already; nothing changed.",
            );
            if options.status_line
                && let Ok(status) = signal_entries::status(&path)
                && let Line::Other(command) = status.line
            {
                println!("The status line runs another command and stays: {command}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => failed(e),
    }
}

/// Takes the tool's hooks and status line out of Claude Code's user settings, after a dated copy.
fn signals_remove() -> ExitCode {
    let (path, _) = match signals_places() {
        Ok(places) => places,
        Err(e) => return failed(e),
    };
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    match signal_entries::remove(&path, &store.backups, &files::today()) {
        Ok(report) => {
            print_report(
                &report,
                "No entry of the tool's was there; nothing changed.",
            );
            ExitCode::SUCCESS
        }
        Err(e) => failed(e),
    }
}

/// Copies the release build of the repository this crw was compiled in into the install folder
/// (D-29; EP-37), the installed exes renamed to previous copies first, then asks the window to
/// restart into it, unless `--no-restart`.
fn install_build(flags: &[&str]) -> ExitCode {
    let mut restart = true;
    for flag in flags {
        match *flag {
            "--no-restart" => restart = false,
            other => return usage_error(&format!("install takes no {other}")),
        }
    }
    let Some(bin) = install::bin_dir() else {
        return failed(format!(
            "no local folder: neither LOCALAPPDATA nor {} is set",
            paths::HOME_VAR
        ));
    };
    let from = install::build_dir();
    println!("crw {} install", env!("CARGO_PKG_VERSION"));
    println!("  from  {}", from.display());
    println!("  to    {}", bin.display());
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let report = match install::install(&from, &bin, &stamp) {
        Ok(report) => report,
        Err(e) => return failed(e),
    };
    for line in report.lines() {
        println!("  - {line}");
    }
    // The build's copy of the plugin, so that the installed build and the copy agree from here
    // on (D-27; EP-22a); the window writes it again before each headless start.
    if let Some(dir) = plugin::dir() {
        match plugin::write(&dir) {
            Ok(written) => println!("  - plugin {}: {}", dir.display(), written.words()),
            Err(e) => println!("  - plugin: {e}"),
        }
    }
    if !restart {
        println!("The window was not asked to restart (--no-restart).");
        return ExitCode::SUCCESS;
    }
    // A trial installs into its own folder and leaves the owner's window alone (rule 13).
    if let Some(home) = paths::home() {
        println!(
            "The window was not asked to restart: {}={} is a trial.",
            paths::HOME_VAR,
            home.display()
        );
        return ExitCode::SUCCESS;
    }
    let port = signals::port();
    match signals::ask_restart(port, "crw install", Some(bin.as_path())) {
        signals::RestartReply::Answered(answer) if answer.restarting => {
            println!("The window restarts: {}.", answer.words);
        }
        signals::RestartReply::Answered(answer) => println!(
            "The window keeps running: {}. It does not ask again: its File > Install and \
             restart, or the tray's, opens the installed build once nothing stands in the way.",
            answer.words
        ),
        signals::RestartReply::NoWindow => println!(
            "No window listens on 127.0.0.1:{port}: the shortcut opens the installed build."
        ),
        signals::RestartReply::NoRoute => println!(
            "The window runs a build from before the restart: quit it from the tray and start it \
             from the shortcut, which opens the installed build."
        ),
        signals::RestartReply::Failed(why) => println!("The window did not answer: {why}."),
    }
    ExitCode::SUCCESS
}

/// Where this exe and the installed copy are, and which `crw.exe` the hooks and the status line
/// run (D-29; EP-37).
fn print_exes() {
    let this = this_exe().ok();
    println!(
        "  this exe   {}",
        this.as_ref()
            .map_or_else(|| "?".to_owned(), |this| this.display().to_string())
    );
    match install::bin_dir() {
        Some(bin) => match install::Stamp::of(&bin.join(install::CLI_EXE)) {
            Some(stamp) => println!("  installed  {}, built {}", bin.display(), stamp.when()),
            None => println!(
                "  installed  none in {}: crw install copies the release build there",
                bin.display()
            ),
        },
        None => println!("  installed  ? no local folder"),
    }
    // The build's copy of the plugin a headless run loads (D-27; EP-22a).
    match plugin::dir() {
        Some(dir) => println!(
            "  plugin     {}: {}",
            dir.display(),
            plugin::Status::of(&dir).words()
        ),
        None => println!("  plugin     ? no local folder"),
    }
    let Some(status) =
        signal_entries::settings_file().and_then(|path| signal_entries::status(&path).ok())
    else {
        println!("  hooks      ? the user settings did not read");
        return;
    };
    let all = status
        .hooks
        .iter()
        .filter_map(|(_, hooked)| match hooked {
            Hooked::Ours { exe, .. } => Some(exe.clone()),
            _ => None,
        })
        .chain(match &status.line {
            Line::Ours(exe) => Some(exe.clone()),
            _ => None,
        });
    let mut run: Vec<PathBuf> = Vec::new();
    for exe in all {
        if !run.iter().any(|seen| paths::same(seen, &exe)) {
            run.push(exe);
        }
    }
    let words: Vec<String> = run
        .iter()
        .map(|exe| exe_words(exe, this.as_deref()))
        .collect();
    match words.as_slice() {
        [] => println!("  hooks      not installed (crw signals install)"),
        [one] if one == "the installed copy" => println!("  hooks      run the installed copy"),
        _ => println!(
            "  hooks      run {}{}",
            words.join(" and "),
            if installed_cli().is_some() {
                ": crw signals install points them at the installed copy"
            } else {
                ""
            }
        ),
    }
}

fn print_report(report: &signal_entries::Report, unchanged: &str) {
    if report.changes.is_empty() {
        println!("{unchanged}");
        return;
    }
    println!("{} changed:", report.path.display());
    for change in &report.changes {
        println!("  - {change}");
    }
    if let Some(copy) = &report.copy {
        println!("The file as it was is kept as {}", copy.display());
    }
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
            // Logged for a revert to redo (EP-15b); the tick stands whatever the ledger says.
            let disposition = (prefix == "IN").then(|| words.join(" "));
            let name = registered_name(&store, &folder).unwrap_or_else(|| folder_name(&folder));
            if let Some(ledger) = paths::ledger_file()
                && let Err(why) = revert::log_tick(
                    &ledger,
                    &name,
                    &id.to_ascii_uppercase(),
                    checked,
                    disposition.as_deref(),
                )
            {
                eprintln!("crw: {why}; a revert would not redo the tick");
            }
            ExitCode::SUCCESS
        }
        Err(e) => failed(format!("{}: {e}", path.display())),
    }
}

/// Writes a message from the owner into a project's inbox (D-13; EP-20c): `crw send
/// <project|folder> <words…> [--urgent]`, the mailman's own append after a copy, logged for a
/// revert to redo. A session at work there does not hold it (the owner's choice): it is named,
/// and the next session triages the message.
fn send_message(args: &[&str]) -> ExitCode {
    let Some((target, text, urgent)) = send_args(args) else {
        return usage_error(
            "send takes a project or folder and the message's words, and --urgent to flag it",
        );
    };
    let folder = match project_folder(target) {
        Ok(folder) => folder,
        Err(e) => return failed(e),
    };
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    let registry = store.load().ok().map(|loaded| loaded.registry);
    let name = registered_name(&store, &folder).unwrap_or_else(|| folder_name(&folder));
    let today = files::today();
    let ledger = paths::ledger_file();
    let sent = mailman::send_from_owner(
        &folder,
        &name,
        &text,
        urgent,
        &store.backups,
        &today,
        ledger.as_deref(),
    );
    match sent {
        Ok(sent) => {
            let kept = match &sent.copy {
                Some(copy) => format!("the file as it was is kept as {}", copy.display()),
                None => "the file was created with its header".to_owned(),
            };
            println!(
                "sent {} to {name} in {}; {kept}",
                sent.id,
                inbox::path(&folder).display()
            );
            println!("  {}", sent.written.replace('\n', "\n  "));
            if let Some(why) = sent.unlogged {
                eprintln!("crw: {why}; a revert would not redo the message");
            } else if !sent.revert_keeps {
                println!(
                    "note: the message is longer than the ledger keeps (1,000 characters with \
                     its id and date): a revert of a session there would name it instead of \
                     putting it back"
                );
            }
            if let Some(registry) = registry {
                let (working, _) = live_sessions(&registry, &folder);
                if !working.is_empty() {
                    println!(
                        "note: {}; a session that has read its inbox leaves the message to the \
                         next one",
                        working.join("; ")
                    );
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => failed(e),
    }
}

/// `crw send`'s arguments: `--urgent` anywhere (EP-20c's review; EP-20d), the first other one the
/// project or folder, the rest the message's words; `None` without a target or words.
fn send_args<'a>(args: &[&'a str]) -> Option<(&'a str, String, bool)> {
    let urgent = args.contains(&"--urgent");
    let mut rest = args.iter().copied().filter(|arg| *arg != "--urgent");
    let target = rest.next()?;
    let text = rest.collect::<Vec<_>>().join(" ");
    (!text.trim().is_empty()).then_some((target, text, urgent))
}

/// The registry's name for the project in `folder`, when one is registered there.
fn registered_name(store: &Store, folder: &Path) -> Option<String> {
    let registry = store.load().ok()?.registry;
    let root = registry.data_root();
    registry
        .projects
        .iter()
        .find(|project| paths::same(&project.folder(&root), folder))
        .map(|project| project.name.clone())
}

/// `crw formalise` (D-10 … D-13; EP-21): previews the house-style files a project is given, or
/// writes them with `--yes` once no session works or waits there (as `crw revert` asks), logging
/// the write in the ledger; with `--migration`, prints the prompt of the session that migrates an
/// older roadmap to the house style (D-9).
fn formalise_project(args: &[&str]) -> ExitCode {
    let flag = |name: &str| args.contains(&name);
    let (yes, migration) = (flag("--yes"), flag("--migration"));
    if let Some(other) = args
        .iter()
        .find(|arg| arg.starts_with("--") && !matches!(**arg, "--yes" | "--migration"))
    {
        return usage_error(&format!(
            "formalise takes --yes or --migration, not {other}"
        ));
    }
    if yes && migration {
        return usage_error("formalise takes --yes or --migration, not both");
    }
    let words: Vec<&str> = args
        .iter()
        .copied()
        .filter(|arg| !arg.starts_with("--"))
        .collect();
    let [target] = words.as_slice() else {
        return usage_error("formalise takes one project or folder");
    };
    println!("crw {} formalise", env!("CARGO_PKG_VERSION"));
    let folder = match project_folder(target) {
        Ok(folder) => std::path::absolute(&folder).unwrap_or(folder),
        Err(e) => return failed(e),
    };
    let store = match store() {
        Ok(store) => store,
        Err(e) => return failed(e),
    };
    let registry = match store.load() {
        Ok(loaded) => loaded.registry,
        Err(e) => return failed(e),
    };
    let root = registry.data_root();
    let registered = registry
        .projects
        .iter()
        .find(|project| paths::same(&project.folder(&root), &folder));
    let name = registered.map_or_else(|| folder_name(&folder), |project| project.name.clone());
    // How the commands printed below name the project: a folder that is not registered by its
    // path, which works from any folder (EP-21's review).
    let target = match registered {
        Some(_) => name.clone(),
        None => format!("\"{}\"", folder.display()),
    };
    let plan = formalise::plan(&folder, &name);
    let today = files::today();
    if migration {
        let Some(profile) = plan.migrates() else {
            return failed(format!(
                "{name}: no migration to prompt for: {}",
                match &plan.readme {
                    formalise::Readme::Missing => "it has no roadmap/README.md".to_owned(),
                    formalise::Readme::Unreadable(why) => format!("its roadmap README: {why}"),
                    formalise::Readme::Read(None) => {
                        "its roadmap README has no phase table the parser reads".to_owned()
                    }
                    formalise::Readme::Read(Some(_)) => "its tables are the house's".to_owned(),
                }
            ));
        };
        let crw = installed_cli()
            .or_else(|| std::env::current_exe().ok())
            .unwrap_or_else(|| PathBuf::from(install::CLI_EXE));
        let text = formalise::migration_prompt(&name, &folder, profile, &crw, &today)
            .expect("an older form has a migration prompt");
        println!(
            "The prompt of a headless session in {name} that migrates its tables to the house \
             style ({} characters; launches nothing):\n",
            text.chars().count()
        );
        println!("{text}");
        return ExitCode::SUCCESS;
    }
    for line in plan.lines(&target) {
        println!("{line}");
    }
    if plan.is_refused() {
        println!("Nothing changed.");
        return ExitCode::FAILURE;
    }
    if registered.is_some_and(|project| project.state == State::Unformalised) {
        println!(
            "  its state stays unformalised: `crw set {name} state active` once its roadmap is \
             written"
        );
    }
    if plan.writes() == 0 {
        println!("Nothing to write: the project has every file.");
        return ExitCode::SUCCESS;
    }
    for step in &plan.steps {
        if let Some(text) = step.shown() {
            let what = match step.change {
                formalise::Change::Create => "the new file",
                _ => "the block",
            };
            println!("\n{} ({}), {what}:\n", step.name, step.file.words());
            for line in text.lines() {
                println!("    {line}");
            }
        }
    }
    if !yes {
        println!("\nNothing changed. `crw formalise {target} --yes` writes.");
        return ExitCode::SUCCESS;
    }
    let caller = caller();
    let (working, _) = live_sessions_but(&registry, &folder, caller.as_deref());
    if !working.is_empty() {
        println!(
            "\nrefused while a session works or waits there: {}. Nothing changed.",
            working.join("; ")
        );
        return ExitCode::FAILURE;
    }
    if let Some(id) = &caller {
        println!(
            "\nthe session running this command ({}) is not counted",
            revert::short(id)
        );
    }
    let applied = match formalise::apply(&plan, &store.backups, &today) {
        Ok(applied) => applied,
        Err(e) => return failed(e),
    };
    println!();
    for written in &applied.written {
        let copy = written
            .copy
            .as_ref()
            .map(|copy| format!("; the file as it was is kept as {}", copy.display()))
            .unwrap_or_default();
        println!("wrote {}{copy}", written.path.display());
    }
    if let Some(path) = paths::ledger_file().filter(|path| path.exists())
        && !applied.written.is_empty()
    {
        let logged = Ledger::open(&path)
            .map_err(|e| e.to_string())
            .and_then(|ledger| {
                ledger
                    .add_event(
                        Utc::now(),
                        formalise::EVENT_KIND,
                        Some(&name),
                        &applied.detail(),
                    )
                    .map_err(|e| e.to_string())
            });
        if let Err(why) = logged {
            eprintln!("crw: the ledger did not take the write's line: {why}");
        }
    }
    match &applied.failed {
        Some(why) => failed(format!(
            "stopped: {why}; `crw formalise {target}` again finishes the job"
        )),
        None => ExitCode::SUCCESS,
    }
}

/// What the downshift weighs for `crw prompt` (D-23; EP-18): the endpoint's last reading, which
/// the window keeps in the ledger's cache, read without migrating or writing it; nothing read
/// when there is no ledger or no reading. The CLI never reads the credentials or polls.
fn cached_usage(now: DateTime<Utc>) -> rules::Usage {
    let cached = paths::ledger_file()
        .filter(|path| path.exists())
        .and_then(|path| Ledger::open_read_only(&path).ok())
        .and_then(|ledger| ledger.cached_one("", usage::CACHE_SOURCE).ok().flatten())
        .and_then(|row| Some((usage::Reading::from_json(&row.data)?, row.fetched_at)));
    match cached {
        Some((reading, at)) => rules::Usage::of_reading(&reading, at, now),
        None => rules::Usage::unread(now),
    }
}

/// Prints what a launch of a project's next brief starts from (D-17, D-22): the rule table's
/// resolution with its reasons, the downshift's judgement on the window's last usage reading
/// (EP-18), the neighbour folders and the prompt, worded for `mode` when one is asked (EP-11).
/// Reads, never launches, and asks nothing of the CLI: an effort step moves along the built-in
/// levels here.
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
    let usage = cached_usage(Utc::now());
    let mut draft = match prompt::draft(
        &registry,
        &project,
        &snapshot,
        claude_md.as_deref(),
        &[],
        &usage,
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
    if let Some(judged) = &resolution.downshift {
        let applied = if judged.applied { "applied: " } else { "" };
        println!("  downshift  {applied}{}", judged.words);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_takes_the_flag_anywhere() {
        let words = "Read the export first.";
        let expect = Some(("alpha", words.to_owned(), true));
        assert_eq!(
            send_args(&["--urgent", "alpha", "Read", "the", "export", "first."]),
            expect
        );
        assert_eq!(
            send_args(&["alpha", "Read", "the", "--urgent", "export", "first."]),
            expect
        );
        assert_eq!(
            send_args(&["alpha", "Read", "the", "export", "first."]),
            Some(("alpha", words.to_owned(), false))
        );
        assert_eq!(send_args(&["--urgent", "alpha"]), None, "no words");
        assert_eq!(send_args(&[]), None);
    }

    /// The formalise step's session check (EP-22): the session that runs `crw formalise --yes`
    /// in its own project is left out, another one at work there still refuses the write, and
    /// the revert's check, given no caller, counts every session.
    #[test]
    fn the_session_running_the_command_is_left_out_of_its_check() {
        let agents = agents::parse(
            r#"[
              {"cwd": "C:\\Data\\alpha", "sessionId": "caller-1", "status": "busy"},
              {"cwd": "C:\\Data\\alpha\\crates", "sessionId": "other-2", "status": "waiting"},
              {"cwd": "C:\\Data\\alpha", "sessionId": "tab-3", "status": "idle"},
              {"cwd": "C:\\Data\\beta", "sessionId": "elsewhere", "status": "busy"}
            ]"#,
        )
        .unwrap();
        let alpha = Path::new(r"C:\Data\alpha");
        let (working, idle) = sessions_in(&agents, alpha, Some("caller-1"));
        assert_eq!(working.len(), 1, "{working:?}");
        assert!(working[0].contains("is waiting"), "{working:?}");
        assert_eq!(idle.len(), 1);
        let (alone, _) = sessions_in(&agents[..1], alpha, Some("caller-1"));
        assert!(alone.is_empty(), "{alone:?}");
        let (all, _) = sessions_in(&agents, alpha, None);
        assert_eq!(all.len(), 2, "{all:?}");
    }

    /// `crw set`'s permission mode (EP-19b): a mode the tool knows, in its spelling; `default`
    /// for taking the project's out; anything else refused with the modes named.
    #[test]
    fn set_reads_a_permission_mode_in_the_tools_spelling() {
        assert_eq!(permission_mode_word("auto"), Ok(Some("auto")));
        assert_eq!(permission_mode_word(" DONTASK "), Ok(Some("dontAsk")));
        assert_eq!(permission_mode_word("acceptedits"), Ok(Some("acceptEdits")));
        assert_eq!(
            permission_mode_word("bypasspermissions"),
            Ok(Some("bypassPermissions"))
        );
        assert_eq!(permission_mode_word("Manual"), Ok(Some("manual")));
        assert_eq!(permission_mode_word("Default"), Ok(None));
        for wrong in ["yolo", "accept edits", ""] {
            let error = permission_mode_word(wrong).unwrap_err();
            assert!(
                error.starts_with("permission_mode takes acceptEdits, auto, plan")
                    && error.ends_with(&format!("or default, not {wrong:?}")),
                "{wrong:?}: {error}"
            );
        }
    }

    /// `crw set`'s model and effort (EP-19c): a model is one word as written, an effort one of
    /// the built-in levels in its spelling; `default` takes either out; anything else is refused
    /// with what the field takes.
    #[test]
    fn set_reads_a_model_and_an_effort() {
        assert_eq!(model_word(" claude-opus-5-5 "), Ok(Some("claude-opus-5-5")));
        assert_eq!(model_word("fable"), Ok(Some("fable")));
        assert_eq!(model_word("Claude-Fable-5-1"), Ok(Some("Claude-Fable-5-1")));
        assert_eq!(model_word("DEFAULT"), Ok(None));
        for wrong in ["claude opus", "", "  ", "opus\tfable"] {
            let error = model_word(wrong).unwrap_err();
            assert!(
                error.starts_with("model takes one word")
                    && error.ends_with(&format!("{:?}", wrong.trim())),
                "{wrong:?}: {error}"
            );
        }
        assert_eq!(effort_word("max"), Ok(Some("max")));
        assert_eq!(effort_word(" XHigh "), Ok(Some("xhigh")));
        assert_eq!(effort_word("Low"), Ok(Some("low")));
        assert_eq!(effort_word("default"), Ok(None));
        for wrong in ["ultra", "", "very high", "-1"] {
            let error = effort_word(wrong).unwrap_err();
            assert!(
                error.starts_with("effort takes low, medium, high, xhigh, max or default")
                    && error.ends_with(&format!("{:?}", wrong.trim())),
                "{wrong:?}: {error}"
            );
        }
        // Neither is one of the autopilot's limits.
        assert_eq!(limit_of("model", "fable"), None);
        assert_eq!(limit_of("effort", "max"), None);
    }

    /// `crw set`'s limits (EP-19b): each in its words, back to its default (a value that is the
    /// default taken out as `default` is), and refused out of range; a field that is no limit is
    /// left to the caller.
    #[test]
    fn set_reads_the_autopilots_limits() {
        for (field, value, limit) in [
            ("max_size", "s", Limit::MaxSize(Some(Size::S))),
            ("max_size", " M ", Limit::MaxSize(None)),
            ("max_size", "L", Limit::MaxSize(Some(Size::L))),
            ("max_size", "Default", Limit::MaxSize(None)),
            ("daily", "0", Limit::Daily(Some(0))),
            ("daily", "2", Limit::Daily(None)),
            ("daily", "100", Limit::Daily(Some(MAX_DAILY))),
            ("daily", "default", Limit::Daily(None)),
            ("max_budget_usd", "2.5", Limit::Budget(Some(2.5))),
            ("max_budget_usd", "$3", Limit::Budget(Some(3.0))),
            ("max_budget_usd", " $ 0.75 ", Limit::Budget(Some(0.75))),
            ("max_budget_usd", "None", Limit::Budget(None)),
            ("max_budget_usd", "default", Limit::Budget(None)),
        ] {
            assert_eq!(limit_of(field, value), Some(Ok(limit)), "{field} {value:?}");
        }
        for (field, wrong) in [
            ("max_size", "XL"),
            ("max_size", ""),
            ("daily", "101"),
            ("daily", "-1"),
            ("daily", "two"),
            ("max_budget_usd", "0"),
            ("max_budget_usd", "-1"),
            ("max_budget_usd", "nan"),
            ("max_budget_usd", "inf"),
            ("max_budget_usd", "$"),
        ] {
            let error = limit_of(field, wrong).unwrap().unwrap_err();
            assert!(
                error.starts_with(&format!("{field} takes"))
                    && error.ends_with(&format!("{wrong:?}")),
                "{field} {wrong:?}: {error}"
            );
        }
        assert_eq!(limit_of("permission_mode", "auto"), None);
        assert_eq!(limit_of("state", "active"), None);
    }
}
