//! The hosting and backup actions (D-24; EP-34): Create private remote, Make public, Make
//! private, Remove remote, an orphan's Archive and Delete, and Back up now. Each is planned from
//! what the window knows ([`Facts`]) into a [`Plan`]: the command it runs, named whole, what it
//! does in words, the warnings it repeats (`gh`'s consequences paragraph, rule 3's word on
//! publishing, the scope a delete needs) and what refuses it. The window's dialog shows the plan,
//! reads it again at the owner's word and runs it only when it is the plan shown; [`run`] runs it
//! through a [`Runner`], each child in its turn (D-29), and [`log`] keeps an `event` row of kind
//! [`HOSTING`] or [`BACKUP`] with the command and its outcome. A failure is shown, never retried.
//! Make public's plan also holds the word check of the project's history ([`Check`], EP-42): one
//! read-only pass through the same runner against the owner's list, whose words are never shown,
//! logged or kept ([`crate::wordcheck`]). Tests run a fake runner: never `gh`, PowerShell or the
//! job, and git only over a scratch repository (D-29).

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::badges::backup;
use crate::badges::hosting::{self, Hosting, Listed, Visibility};
use crate::{git, process, wordcheck};

/// The `event` rows of the six actions on GitHub and the origin.
pub const HOSTING: &str = "hosting";
/// The `event` rows of Back up now.
pub const BACKUP: &str = "backup";

/// How long a `gh` or `git` change may take, holding the children's turn (EP-34's review: no
/// longer than it must, since every other child waits).
const GH_TIMEOUT: Duration = Duration::from_secs(60);
/// How long Create may take: it pushes the current branch's history.
const PUSH_TIMEOUT: Duration = Duration::from_secs(300);
/// How long the window waits for the backup job; a run took two minutes in November 2026, and one
/// still running then goes on by itself.
const BACKUP_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// What keeps `gh` and the `git` it runs from asking anything: the window's children have no
/// console, so a prompt would wait for ever.
const QUIET: [(&str, &str); 3] = [
    ("GH_PROMPT_DISABLED", "1"),
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GCM_INTERACTIVE", "never"),
];

/// `gh auth status` asked for the active account on github.com, as JSON: its login and the
/// token's scopes.
pub(crate) const AUTH: [&str; 7] = [
    "auth",
    "status",
    "--active",
    "--hostname",
    "github.com",
    "--json",
    "hosts",
];

/// The scope `gh repo delete` needs, and how the owner adds it (`gh`'s own words).
pub(crate) const DELETE_SCOPE: &str = "delete_repo";
pub const ADD_DELETE_SCOPE: &str = "gh auth refresh -s delete_repo";

/// `gh repo edit`'s paragraph on changing a repository's visibility (gh 2.104), repeated by the
/// dialogs of Make public and Make private.
pub const VISIBILITY_CONSEQUENCES: &str = "Changing repository visibility can have unexpected \
consequences including but not limited to: losing stars and watchers, affecting repository \
ranking; detaching public forks from the network; disabling push rulesets; allowing access to \
GitHub Actions history and logs. For all the potential consequences, see \
https://gh.io/setting-repository-visibility (gh repo edit --help).";

/// What this repository's rule 3 says of a push, repeated by Make public's dialog (D-3).
pub const PUBLISHES: &str = "Making it public publishes it at once: every commit and every file \
its history holds, to anyone, and copies may outlive a later Make private. As this tool's own \
rule 3 asks of each of its commits, check the history first for the names of private projects \
and for machine details (drive labels, backup paths, devices, user names in paths).";

/// One of D-24's actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Verb {
    CreatePrivate,
    MakePublic,
    MakePrivate,
    RemoveRemote,
    Archive,
    Delete,
    BackUpNow,
}

impl Verb {
    pub const ALL: [Verb; 7] = [
        Verb::CreatePrivate,
        Verb::MakePublic,
        Verb::MakePrivate,
        Verb::RemoveRemote,
        Verb::Archive,
        Verb::Delete,
        Verb::BackUpNow,
    ];

    /// The word `crw act` and the screenshot flag take.
    pub fn id(self) -> &'static str {
        match self {
            Verb::CreatePrivate => "create-private",
            Verb::MakePublic => "make-public",
            Verb::MakePrivate => "make-private",
            Verb::RemoveRemote => "remove-remote",
            Verb::Archive => "archive",
            Verb::Delete => "delete",
            Verb::BackUpNow => "backup",
        }
    }

    pub fn parse(word: &str) -> Option<Verb> {
        let word = word.trim().to_ascii_lowercase();
        Verb::ALL
            .into_iter()
            .find(|verb| verb.id() == word || (*verb == Verb::BackUpNow && word == "back-up-now"))
    }

    /// The menu's and the button's words; each opens the dialog.
    pub fn label(self) -> &'static str {
        match self {
            Verb::CreatePrivate => "Create a private remote…",
            Verb::MakePublic => "Make public…",
            Verb::MakePrivate => "Make private…",
            Verb::RemoveRemote => "Remove the remote…",
            Verb::Archive => "Archive the orphan…",
            Verb::Delete => "Delete the orphan…",
            Verb::BackUpNow => "Back up now…",
        }
    }

    /// The dialog's question about `target`.
    pub fn question(self, target: &str) -> String {
        match self {
            Verb::CreatePrivate => format!("Create a private remote for {target}?"),
            Verb::MakePublic => format!("Make {target} public?"),
            Verb::MakePrivate => format!("Make {target} private?"),
            Verb::RemoveRemote => format!("Remove {target}'s remote?"),
            Verb::Archive => format!("Archive {target}?"),
            Verb::Delete => format!("Delete {target}?"),
            Verb::BackUpNow => "Back up now?".to_owned(),
        }
    }

    /// What it did, after the target: `alpha: made public`.
    pub fn done(self) -> &'static str {
        match self {
            Verb::CreatePrivate => "a private remote created and pushed",
            Verb::MakePublic => "made public",
            Verb::MakePrivate => "made private",
            Verb::RemoveRemote => "the remote removed",
            Verb::Archive => "archived",
            Verb::Delete => "deleted",
            Verb::BackUpNow => "backed up",
        }
    }

    /// The `event` rows' kind.
    pub fn kind(self) -> &'static str {
        match self {
            Verb::BackUpNow => BACKUP,
            _ => HOSTING,
        }
    }

    /// The actions a project's hosting offers, in the order the row's menu lists them; a plan
    /// may still refuse one (the data mark, a session at work, the token's scope).
    pub fn offered(hosting: &Hosting) -> Vec<Verb> {
        match hosting {
            Hosting::Local => vec![Verb::CreatePrivate],
            Hosting::Orphan(listed) if listed.archived => vec![Verb::Delete],
            Hosting::Orphan(_) => vec![Verb::Archive, Verb::Delete],
            Hosting::Listed(listed) if listed.archived => vec![Verb::RemoveRemote],
            Hosting::Listed(listed) if listed.visibility == Visibility::Public => {
                vec![Verb::MakePrivate, Verb::RemoveRemote]
            }
            Hosting::Listed(_) => vec![Verb::MakePublic, Verb::RemoveRemote],
            Hosting::Missing(_) | Hosting::GitHub(_) | Hosting::Elsewhere(_) => {
                vec![Verb::RemoveRemote]
            }
        }
    }
}

/// The project an action is on, as the window knows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// Its name in the registry.
    pub name: String,
    pub folder: PathBuf,
    /// Marked a data repository in the registry (`data = true`).
    pub data: bool,
    /// Whether `HEAD` names a commit, as git last read it; `None` before git was asked.
    pub has_commit: Option<bool>,
    /// Whether the folder holds a repository (`.git`).
    pub repository: bool,
    /// The origin's URL as `.git/config` holds it.
    pub remote: Option<String>,
}

impl Target {
    /// A project's target, its repository and its origin read by path now (no child).
    pub fn read(name: &str, folder: &Path, data: bool, has_commit: Option<bool>) -> Self {
        Self {
            name: name.to_owned(),
            folder: folder.to_owned(),
            data,
            has_commit,
            repository: folder.join(".git").exists(),
            remote: git::remote_url(folder, "origin"),
        }
    }
}

/// What a plan is read from: the action, its project (none for Back up now), the account's list
/// as the window cached it, the window's own refusals (a session at work there), and the backup
/// job's settings file.
#[derive(Debug, Clone, PartialEq)]
pub struct Facts {
    pub verb: Verb,
    pub target: Option<Target>,
    /// `None` while no list is cached.
    pub account: Option<Vec<Listed>>,
    pub refusals: Vec<String>,
    /// The registry's `backup_settings`, in full; `None` when it sets none.
    pub backup: Option<PathBuf>,
    /// Make public's word list, read from the file the registry's `private_words` names: `None`
    /// when it names none, an error when the file did not read (EP-42). Its `Debug` shows the
    /// count of its words alone.
    pub words: Option<Result<wordcheck::List, String>>,
}

/// What Make public's word check of the history found (D-24, D-3; EP-42).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    /// No list to read it against (none named, its file unread, or no word in it), in words:
    /// the dialog says so and goes on as before.
    NoList(String),
    /// The history did not read, in git's words: the dialog asks for the second tick.
    Failed(String),
    /// The history read: its commits and those that hit.
    Read(wordcheck::Scan),
}

impl Check {
    /// Whether the owner must tick a second time before the action runs: a commit hit, or the
    /// history did not read.
    pub fn asks_tick(&self) -> bool {
        match self {
            Check::NoList(_) => false,
            Check::Failed(_) => true,
            Check::Read(scan) => !scan.hits.is_empty(),
        }
    }

    /// The check in one line, for the dialog and `crw act`.
    pub fn summary(&self) -> String {
        match self {
            Check::NoList(why) | Check::Failed(why) => why.clone(),
            Check::Read(scan) => format!("The history: {}.", scan.summary()),
        }
    }
}

/// A command an action runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub program: PathBuf,
    pub args: Vec<String>,
    /// The backup job: started in its turn, waited on outside it, never killed.
    pub long: bool,
    /// How long it may run (a long one: how long it is waited on).
    pub timeout: Duration,
    /// The folder it runs in, when not the window's: the backup job's script's.
    pub folder: Option<PathBuf>,
}

impl Command {
    /// `gh` with `args`, given `timeout`.
    fn gh(args: &[&str], timeout: Duration) -> Self {
        Self {
            program: hosting::gh(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            long: false,
            timeout,
            folder: None,
        }
    }

    /// The program's name: `gh`, `git`, `pwsh`.
    fn name(&self) -> String {
        self.program.file_stem().map_or_else(
            || self.program.display().to_string(),
            |stem| stem.to_string_lossy().into_owned(),
        )
    }

    /// What a failure is said of: `gh repo edit`, `git remote remove`, `pwsh`.
    fn short(&self) -> String {
        let name = self.name();
        let words: Vec<&str> = match name.as_str() {
            "gh" => self.args.iter().take(2).map(String::as_str).collect(),
            // After `-C <folder>`.
            "git" => self
                .args
                .iter()
                .skip(2)
                .take(2)
                .map(String::as_str)
                .collect(),
            _ => Vec::new(),
        };
        std::iter::once(name.as_str())
            .chain(words)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The command as the dialog names it: the program's name, then each argument, quoted when
    /// it holds a space.
    pub fn line(&self) -> String {
        let program = self.name();
        std::iter::once(program)
            .chain(self.args.iter().map(|arg| {
                if arg.contains(' ') {
                    format!("\"{arg}\"")
                } else {
                    arg.clone()
                }
            }))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// One action, planned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub verb: Verb,
    /// The project acted on; `None` for Back up now.
    pub project: Option<String>,
    /// What the dialog's question names: the project, the orphan's repository, the job.
    pub target: String,
    /// `None` when a refusal leaves nothing to run.
    pub command: Option<Command>,
    /// What it does, a sentence each.
    pub words: Vec<String>,
    pub warnings: Vec<String>,
    /// What refuses it; the owner can only close the dialog then.
    pub problems: Vec<String>,
    /// Make public's word check of the history, read when nothing else refuses the plan (EP-42).
    pub check: Option<Check>,
}

impl Plan {
    pub fn is_refused(&self) -> bool {
        !self.problems.is_empty() || self.command.is_none()
    }

    /// Whether the dialog asks for a second tick before it runs the plan (EP-42).
    pub fn asks_tick(&self) -> bool {
        self.check.as_ref().is_some_and(Check::asks_tick)
    }

    fn new(verb: Verb, project: Option<&str>, target: impl Into<String>) -> Self {
        Self {
            verb,
            project: project.map(str::to_owned),
            target: target.into(),
            command: None,
            words: Vec::new(),
            warnings: Vec::new(),
            problems: Vec::new(),
            check: None,
        }
    }
}

/// How a child ended, as a [`Runner`] reports it.
pub type Exit = process::Exit;

/// Runs a plan's commands: the real one, or a test's.
pub trait Runner: Send + Sync {
    /// Runs `command` to its end: how it ended, whatever its exit code; `Err` when it did not
    /// start or did not end in time.
    fn run(&self, command: &Command) -> Result<Exit, String>;
}

/// `gh` and `git` in their turn among the window's children, prompts off; the backup job
/// started in its turn and waited on outside it, so the window's git and GitHub passes go on.
pub struct Real;

impl Runner for Real {
    fn run(&self, command: &Command) -> Result<Exit, String> {
        if command.long {
            let mut child = {
                let _turn = process::turn();
                process::start_alone(&command.program, &command.args, command.folder.as_deref())?
            };
            let code = process::wait_alone(&mut child, command.timeout)?;
            return Ok(Exit {
                code,
                stdout: String::new(),
                stderr: String::new(),
            });
        }
        let _turn = process::turn();
        process::exit_of(&command.program, &command.args, &QUIET, command.timeout)
    }
}

/// The active github.com account `gh` is logged in as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub login: String,
    pub scopes: Vec<String>,
}

/// `gh auth status --active --hostname github.com --json hosts`, read.
pub fn parse_auth(json: &str) -> Result<Account, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("gh printed no status: {e}"))?;
    let entries = value["hosts"]["github.com"]
        .as_array()
        .ok_or("gh is not logged in to github.com")?;
    let entry = entries
        .iter()
        .find(|entry| entry["active"].as_bool() == Some(true))
        .or_else(|| entries.first())
        .ok_or("gh is not logged in to github.com")?;
    let state = entry["state"].as_str().unwrap_or("unknown");
    if state != "success" {
        return Err(format!("gh's login to github.com is not usable: {state}"));
    }
    let login = entry["login"]
        .as_str()
        .filter(|login| !login.is_empty())
        .ok_or("gh names no login")?
        .to_owned();
    let scopes = entry["scopes"]
        .as_str()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|scope| !scope.is_empty())
        .map(str::to_owned)
        .collect();
    Ok(Account { login, scopes })
}

/// Asks `gh` which account it is logged in as, through `runner`.
fn gh_account(runner: &dyn Runner) -> Result<Account, String> {
    // A read: the usual 20 s, so an unreachable GitHub holds the children's turn no longer.
    let exit = runner.run(&Command::gh(&AUTH, process::TIMEOUT))?;
    if exit.code != Some(0) {
        return Err(said("gh auth status", &exit));
    }
    parse_auth(&exit.stdout)
}

/// A failed child in words: its errors' words, else its output's, else its exit code.
fn said(what: &str, exit: &Exit) -> String {
    let words = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let text = [words(&exit.stderr), words(&exit.stdout)]
        .into_iter()
        .find(|text| !text.is_empty());
    let code = exit.code.map_or_else(
        || "no exit code".to_owned(),
        |code| format!("exit code {code}"),
    );
    match text {
        Some(text) => format!("{what} failed ({code}): {text}"),
        None => format!("{what} failed ({code})"),
    }
}

/// The hosting's words for a refusal.
fn hosting_words(hosting: &Hosting) -> String {
    match hosting {
        Hosting::Local => "it has no remote".to_owned(),
        Hosting::Orphan(listed) => format!(
            "it has no remote, and the account holds {} (an orphan)",
            listed.repo.slug()
        ),
        Hosting::Listed(listed) => format!("its origin is {}", listed.repo.slug()),
        Hosting::Missing(repo) => {
            format!("its origin {} is not in the account's list", repo.slug())
        }
        Hosting::GitHub(repo) => format!(
            "its origin {} is not matched in the account's list",
            repo.slug()
        ),
        Hosting::Elsewhere(host) => format!("its origin is on {host}, not GitHub"),
    }
}

/// The backup job's script: the `.ps1` of the settings file's name in its folder (the job's
/// settings are named after it), with the name as the folder holds it.
pub fn job_script(settings: &Path) -> Option<PathBuf> {
    let stem = settings.file_stem()?.to_string_lossy().into_owned();
    let folder = settings.parent()?;
    let wanted = format!("{stem}.ps1");
    std::fs::read_dir(folder)
        .ok()?
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .find(|name| name.eq_ignore_ascii_case(&wanted))
        .map(|name| folder.join(name))
}

/// PowerShell 7 from PATH, else where its installer puts it.
fn pwsh() -> PathBuf {
    let on_path = std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join("pwsh.exe"))
            .find(|candidate| candidate.is_file())
    });
    let installed = std::env::var_os("ProgramFiles")
        .map(|dir| {
            PathBuf::from(dir)
                .join("PowerShell")
                .join("7")
                .join("pwsh.exe")
        })
        .filter(|candidate| candidate.is_file());
    on_path
        .or(installed)
        .unwrap_or_else(|| PathBuf::from("pwsh"))
}

/// The plan of `facts`' action: the origin read again from `.git/config`, the hosting from it
/// and the cached list, and, for Create and Delete, `gh`'s account asked through `runner`. Make
/// public's plan that nothing refuses also reads the history against the word list through
/// `runner` (EP-42), so a refused plan, a data repository's among them, never reads it.
pub fn plan(facts: &Facts, runner: &dyn Runner) -> Plan {
    let mut plan = match (&facts.target, facts.verb) {
        (_, Verb::BackUpNow) => back_up_now(facts.backup.as_deref()),
        (Some(target), verb) => on_project(verb, target, facts.account.as_deref(), runner),
        (None, verb) => {
            let mut plan = Plan::new(verb, None, "?");
            plan.problems.push("no project is named".to_owned());
            plan
        }
    };
    plan.problems.extend(facts.refusals.iter().cloned());
    if plan.verb == Verb::MakePublic
        && !plan.is_refused()
        && let Some(target) = &facts.target
    {
        plan.check = Some(check(&target.folder, facts.words.as_ref(), runner));
    }
    plan
}

/// The word check of the history at `folder` against `words` (EP-42): one pass through `runner`
/// when there is a list with a word in it.
fn check(
    folder: &Path,
    words: Option<&Result<wordcheck::List, String>>,
    runner: &dyn Runner,
) -> Check {
    let unchecked = "so the history is not checked for private words before it is published";
    match words {
        None => Check::NoList(format!(
            "No word list is named (crw setting private_words <file>), {unchecked}."
        )),
        Some(Err(why)) => Check::NoList(format!("The word list: {why}, {unchecked}.")),
        Some(Ok(list)) if list.is_empty() => {
            Check::NoList(format!("The word list holds no word, {unchecked}."))
        }
        Some(Ok(list)) => match wordcheck::history(folder, list, runner) {
            Ok(scan) => Check::Read(scan),
            Err(why) => Check::Failed(format!(
                "The history did not read against the word list: {why}."
            )),
        },
    }
}

fn on_project(
    verb: Verb,
    target: &Target,
    account: Option<&[Listed]>,
    runner: &dyn Runner,
) -> Plan {
    let name = target.name.as_str();
    let folder_name = target
        .folder
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_owned());
    let remote = &target.remote;
    let hosting = hosting::classify(&[name, &folder_name], remote.as_deref(), account);
    let repository = target.repository;
    let mut plan = Plan::new(verb, Some(name), name);
    let data = || {
        format!(
            "{name} is marked a data repository (`data = true`): the window never puts one on \
             GitHub or makes one public"
        )
    };
    match verb {
        Verb::CreatePrivate => {
            if !target.folder.is_dir() {
                plan.problems.push(format!("{name}: its folder is missing"));
            } else if !repository {
                plan.problems
                    .push(format!("{name} is not a git repository: nothing to push"));
            } else if target.has_commit == Some(false) {
                plan.problems.push(format!(
                    "{name} has no commit yet: gh would create the repository and fail to push"
                ));
            }
            match (&hosting, account) {
                (Hosting::Local, Some(_)) => {}
                (Hosting::Local, None) => plan.problems.push(
                    "the account's list is not read yet, so whether the name is free is not \
                     known: F5 reads it"
                        .to_owned(),
                ),
                (Hosting::Orphan(listed), _) => plan.problems.push(format!(
                    "the account already holds {} (an orphan): Archive or Delete it first, or \
                     connect it by hand",
                    listed.repo.slug()
                )),
                (other, _) => plan.problems.push(format!(
                    "{name} has a remote already: {}",
                    hosting_words(other)
                )),
            }
            if target.data {
                plan.problems.push(data());
            }
            if !plan.problems.is_empty() {
                return plan;
            }
            match account_of(runner) {
                Ok(login) => {
                    let slug = format!("{login}/{folder_name}");
                    let source = target.folder.display().to_string();
                    plan.command = Some(Command::gh(
                        &[
                            "repo",
                            "create",
                            &slug,
                            "--private",
                            "--source",
                            &source,
                            "--push",
                        ],
                        PUSH_TIMEOUT,
                    ));
                    plan.words.push(format!(
                        "Creates the private repository {slug} on GitHub, adds it to {name} as \
                         its origin and pushes the current branch with its upstream (other \
                         branches and tags stay local); nothing is made public."
                    ));
                }
                Err(why) => plan.problems.push(why),
            }
        }
        Verb::MakePublic | Verb::MakePrivate => {
            let public = verb == Verb::MakePublic;
            let wanted = if public { "public" } else { "private" };
            match &hosting {
                Hosting::Listed(listed) if listed.archived => plan.problems.push(format!(
                    "{} is archived on GitHub: its visibility changes once it is unarchived there",
                    listed.repo.slug()
                )),
                Hosting::Listed(listed) if (listed.visibility == Visibility::Public) == public => {
                    plan.problems
                        .push(format!("{} is {wanted} already", listed.repo.slug()));
                }
                Hosting::Listed(listed) => {
                    let slug = listed.repo.slug();
                    plan.command = Some(Command::gh(
                        &[
                            "repo",
                            "edit",
                            &slug,
                            "--visibility",
                            wanted,
                            "--accept-visibility-change-consequences",
                        ],
                        GH_TIMEOUT,
                    ));
                    plan.words.push(format!(
                        "Changes {slug}'s visibility on GitHub to {wanted}; the clone and its \
                         origin stay as they are."
                    ));
                    plan.warnings.push(VISIBILITY_CONSEQUENCES.to_owned());
                    if public {
                        plan.warnings.push(PUBLISHES.to_owned());
                    }
                }
                other => plan.problems.push(format!(
                    "only a repository the account lists changes its visibility here: {}",
                    hosting_words(other)
                )),
            }
            if public && target.data {
                plan.problems.push(data());
            }
        }
        Verb::RemoveRemote => match remote {
            Some(url) => {
                let folder = target.folder.display().to_string();
                plan.command = Some(Command {
                    program: PathBuf::from("git"),
                    args: ["-C", &folder, "remote", "remove", "origin"]
                        .map(str::to_owned)
                        .to_vec(),
                    long: false,
                    timeout: GH_TIMEOUT,
                    folder: None,
                });
                plan.words.push(format!(
                    "Removes the origin {url} from {name}'s git configuration, with the \
                     remote-tracking branches it fetched; the commits stay in the clone."
                ));
                plan.words.push(match hosting.repo() {
                    Some(repo) => format!(
                        "The GitHub repository {} is left as it is; while the account lists it \
                         under the project's or its folder's name, the row shows it as an orphan.",
                        repo.slug()
                    ),
                    None => "Nothing elsewhere is touched.".to_owned(),
                });
            }
            None => plan
                .problems
                .push(format!("{name} has no origin to remove")),
        },
        Verb::Archive | Verb::Delete => match &hosting {
            Hosting::Orphan(listed) => {
                let slug = listed.repo.slug();
                plan.target = slug.clone();
                if verb == Verb::Archive {
                    if listed.archived {
                        plan.problems.push(format!("{slug} is archived already"));
                        return plan;
                    }
                    plan.command = Some(Command::gh(
                        &["repo", "archive", &slug, "--yes"],
                        GH_TIMEOUT,
                    ));
                    plan.words.push(format!(
                        "Archives {slug} on GitHub: read-only from then on, kept with its \
                         history, and unarchived there by hand; {name}'s folder is left alone."
                    ));
                } else {
                    plan.words.push(format!(
                        "Deletes {slug} on GitHub with its history, issues and settings; \
                         {name}'s folder is left alone."
                    ));
                    plan.warnings.push(
                        "Deleting a private repository that is not itself a fork also deletes \
                         its forks; gh refuses one with forks here, since the window never \
                         passes --accept-cascading-deletion-of-private-forks."
                            .to_owned(),
                    );
                    match gh_account(runner) {
                        Ok(account) if account.scopes.iter().any(|s| s == DELETE_SCOPE) => {
                            plan.command =
                                Some(Command::gh(&["repo", "delete", &slug, "--yes"], GH_TIMEOUT));
                        }
                        Ok(_) => plan.problems.push(format!(
                            "deletion requires the token's {DELETE_SCOPE} scope, which it \
                             lacks: run `{ADD_DELETE_SCOPE}` in a terminal, then open this again"
                        )),
                        Err(why) => plan.problems.push(why),
                    }
                }
            }
            other => plan.problems.push(format!(
                "only an orphan is archived or deleted from the window: {}",
                hosting_words(other)
            )),
        },
        Verb::BackUpNow => unreachable!("planned by back_up_now"),
    }
    plan
}

/// The login `gh` is logged in as, or why it is not known.
fn account_of(runner: &dyn Runner) -> Result<String, String> {
    gh_account(runner)
        .map(|account| account.login)
        .map_err(|why| format!("which account gh is logged in as is not known: {why}"))
}

fn back_up_now(settings: Option<&Path>) -> Plan {
    let mut plan = Plan::new(Verb::BackUpNow, None, "the backup job");
    let Some(settings) = settings else {
        plan.problems.push(
            "no backup job is set: crw setting backup_settings <its settings file>".to_owned(),
        );
        return plan;
    };
    let reading = backup::Reading::read(settings, None);
    if let Err(why) = &reading.job {
        plan.problems
            .push(format!("the job's settings could not be read: {why}"));
        return plan;
    }
    let mut plan = match job_script(settings) {
        Some(script) => backup_plan(&script),
        None => {
            plan.problems.push(format!(
                "no script beside the job's settings: the job runs {}",
                settings.with_extension("ps1").display()
            ));
            plan
        }
    };
    if !reading.mounted {
        plan.problems.push(
            "the job's target is not mounted, so the job would skip the run with a warning"
                .to_owned(),
        );
    }
    plan
}

/// Back up now's plan with the job's script at `script`: run as the job's daily run is, with
/// `-Force` (a screenshot run's canned plan too).
pub fn backup_plan(script: &Path) -> Plan {
    let mut plan = Plan::new(Verb::BackUpNow, None, "the backup job");
    plan.command = Some(Command {
        program: pwsh(),
        args: [
            "-NoProfile",
            "-NonInteractive",
            "-File",
            &script.display().to_string(),
            "-Mode",
            "Run",
            "-Force",
        ]
        .map(str::to_owned)
        .to_vec(),
        long: true,
        timeout: BACKUP_TIMEOUT,
        folder: script.parent().map(Path::to_owned),
    });
    plan.words.push(
        "Runs the backup job now as its daily run does, from its script's folder, with -Force \
         setting aside its minimum hours between runs: it writes each unit whose files changed \
         since its newest archive, and logs to its own run log."
            .to_owned(),
    );
    plan.words.push(
        "A run took about two minutes lately: the dialog may be closed meanwhile, and the status \
         bar says how it ended; the window then reads the job again."
            .to_owned(),
    );
    plan
}

/// How an action ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum End {
    Done,
    /// It ran, and said a warning (the backup job's exit code 2).
    Warned(String),
    Failed(String),
}

/// What running a plan did: how it ended, what it printed, and why the ledger did not take its
/// row, if it did not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    pub verb: Verb,
    pub project: Option<String>,
    pub target: String,
    pub command: String,
    pub end: End,
    /// Its standard output, trimmed.
    pub printed: String,
    pub unlogged: Option<String>,
}

impl Ran {
    /// The status bar's and the event row's words: `alpha: made public (gh repo edit …)`.
    pub fn words(&self) -> String {
        match &self.end {
            End::Done => format!("{}: {}", self.target, self.verb.done()),
            End::Warned(why) => format!(
                "{}: {}, with a warning: {why}",
                self.target,
                self.verb.done()
            ),
            End::Failed(why) => format!(
                "{}: {} did not go through: {why}",
                self.target,
                self.verb.label().trim_end_matches('…')
            ),
        }
    }

    /// The event row's detail: the verb, the outcome, the command.
    pub fn detail(&self) -> String {
        format!("{}: {} · {}", self.verb.id(), self.words(), self.command)
    }
}

/// Runs a plan the owner confirmed through `runner` and logs it in `ledger` (when there is one):
/// a refused plan runs nothing and is not logged.
pub fn run(plan: &Plan, runner: &dyn Runner, ledger: Option<&Path>) -> Option<Ran> {
    if plan.is_refused() {
        return None;
    }
    let command = plan.command.as_ref()?;
    let backup = plan.verb == Verb::BackUpNow;
    let (end, printed) = match runner.run(command) {
        Ok(exit) if exit.code == Some(0) => (End::Done, exit.stdout.trim().to_owned()),
        Ok(exit) if backup && exit.code == Some(2) => {
            let why = "the job's exit code 2: its run log says what (a target not ready, \
                       unreadable files)";
            (End::Warned(why.to_owned()), String::new())
        }
        Ok(exit) if backup => {
            let why = match exit.code {
                Some(code) => format!("the job's exit code {code}: its run log says why"),
                None => "the job ended with no exit code: its run log says why".to_owned(),
            };
            (End::Failed(why), String::new())
        }
        Ok(exit) => {
            let why = said(&command.short(), &exit);
            (End::Failed(why), exit.stdout.trim().to_owned())
        }
        Err(why) => (End::Failed(why), String::new()),
    };
    let mut ran = Ran {
        verb: plan.verb,
        project: plan.project.clone(),
        target: plan.target.clone(),
        command: command.line(),
        end,
        printed,
        unlogged: None,
    };
    if let Some(path) = ledger {
        ran.unlogged = log(path, &ran).err();
    }
    Some(ran)
}

/// Keeps `ran` as an `event` row of its verb's kind, under its project (none for the backup).
pub fn log(path: &Path, ran: &Ran) -> Result<(), String> {
    crate::ledger::Ledger::open(path)
        .and_then(|ledger| {
            ledger.add_event(
                chrono::Utc::now(),
                ran.verb.kind(),
                ran.project.as_deref(),
                &ran.detail(),
            )
        })
        .map(|_| ())
        .map_err(|e| format!("the ledger did not take the action's line: {e}"))
}

#[cfg(test)]
pub mod tests {
    use std::fs;
    use std::sync::Mutex;

    use super::*;
    use crate::badges::hosting::Repo;
    use crate::files::tests::Scratch;
    use crate::ledger::Ledger;

    /// A runner that answers from a script and keeps every command it was asked to run.
    pub struct Fake {
        pub asked: Mutex<Vec<String>>,
        pub auth: Result<String, String>,
        pub exit: Exit,
        /// What `git log` answers, when set (EP-42's word check); else `exit`.
        pub log: Option<Exit>,
    }

    impl Fake {
        pub fn new(scopes: &str) -> Self {
            Self {
                asked: Mutex::new(Vec::new()),
                auth: Ok(auth_json("example", scopes)),
                exit: Exit {
                    code: Some(0),
                    stdout: "done\n".to_owned(),
                    stderr: String::new(),
                },
                log: None,
            }
        }

        pub fn asked(&self) -> Vec<String> {
            self.asked.lock().unwrap().clone()
        }
    }

    impl Runner for Fake {
        fn run(&self, command: &Command) -> Result<Exit, String> {
            self.asked.lock().unwrap().push(command.line());
            if command.args.first().map(String::as_str) == Some("auth") {
                return self.auth.clone().map(|stdout| Exit {
                    code: Some(0),
                    stdout,
                    stderr: String::new(),
                });
            }
            if let Some(log) = &self.log
                && command.args.iter().any(|arg| arg == "log")
            {
                return Ok(log.clone());
            }
            Ok(self.exit.clone())
        }
    }

    pub fn auth_json(login: &str, scopes: &str) -> String {
        format!(
            r#"{{"hosts":{{"github.com":[{{"state":"success","active":true,"host":"github.com","login":"{login}","tokenSource":"keyring","scopes":"{scopes}","gitProtocol":"https"}}]}}}}"#
        )
    }

    fn listed(name: &str, visibility: Visibility, archived: bool) -> Listed {
        Listed {
            repo: Repo {
                owner: "example".to_owned(),
                name: name.to_owned(),
            },
            visibility,
            archived,
            pushed_at: None,
            default_branch: Some("main".to_owned()),
        }
    }

    /// The account's list: alpha public, beta private, gamma private and archived, delta an
    /// orphan's name (no project has it as its origin), epsilon an archived orphan.
    fn account() -> Vec<Listed> {
        vec![
            listed("alpha", Visibility::Public, false),
            listed("beta", Visibility::Private, false),
            listed("gamma", Visibility::Private, true),
            listed("delta", Visibility::Private, false),
            listed("epsilon", Visibility::Private, true),
        ]
    }

    /// A project folder under `scratch`, a repository with `origin` when given.
    fn project(scratch: &Scratch, name: &str, origin: Option<&str>) -> Target {
        let folder = scratch.0.join(name);
        fs::create_dir_all(folder.join(".git")).unwrap();
        let config = match origin {
            Some(url) => format!("[core]\n\tbare = false\n[remote \"origin\"]\n\turl = {url}\n"),
            None => "[core]\n\tbare = false\n".to_owned(),
        };
        fs::write(folder.join(".git").join("config"), config).unwrap();
        Target::read(name, &folder, false, Some(true))
    }

    fn facts(verb: Verb, target: Target) -> Facts {
        Facts {
            verb,
            target: Some(target),
            account: Some(account()),
            refusals: Vec::new(),
            backup: None,
            words: None,
        }
    }

    /// Make public reads the history against the list through the runner, once nothing else
    /// refuses it, and the words never reach the child (EP-42).
    #[test]
    fn make_public_reads_the_history_against_the_list_unless_refused() {
        let scratch = Scratch::new("actions-check");
        let beta = project(&scratch, "beta", Some("git@github.com:example/beta.git"));
        let log = |code: i32, stdout: &str, stderr: &str| Exit {
            code: Some(code),
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        };
        let history = "\0a1b2c3d\u{1f}Add notes\u{1f}Add notes\n\u{1e}\ndiff --git a/n.md b/n.md\n\
                       --- /dev/null\n+++ b/n.md\n@@ -0,0 +1 @@\n+the Harbour plan\n\
                       \0b2c3d4e\u{1f}Start\u{1f}Start\n\u{1e}\n";
        let mut runner = Fake::new("repo");
        runner.log = Some(log(0, history, ""));
        // No list: said, and no child.
        let plain = plan(&facts(Verb::MakePublic, beta.clone()), &runner);
        assert!(
            matches!(&plain.check, Some(Check::NoList(why)) if why.contains("crw setting private_words")),
            "{:?}",
            plain.check
        );
        assert!(!plain.asks_tick() && !plain.is_refused());
        assert!(runner.asked().is_empty());
        // A list: one git child, the hit named, the tick asked.
        let mut listed = facts(Verb::MakePublic, beta.clone());
        listed.words = Some(Ok(wordcheck::List::parse("harbour\n")));
        let hit = plan(&listed, &runner);
        let Some(Check::Read(scan)) = &hit.check else {
            panic!("{:?}", hit.check);
        };
        assert_eq!(scan.commits, 2);
        assert_eq!(scan.hits.len(), 1);
        assert_eq!(scan.hits[0].words(), "a1b2c3d Add notes (1 line)");
        assert!(hit.asks_tick() && !hit.is_refused());
        assert!(
            hit.check
                .as_ref()
                .unwrap()
                .summary()
                .contains("2 commits read against a list of 1 word: 1 hit")
        );
        let asked = runner.asked();
        assert_eq!(asked.len(), 1);
        assert!(
            asked[0].starts_with("git -C ")
                && asked[0].contains(" log --branches --tags --remotes"),
            "{asked:?}"
        );
        assert!(
            !asked[0].to_lowercase().contains("harbour"),
            "the words never reach a child"
        );
        assert!(!format!("{hit:?}").to_lowercase().contains("harbour"));
        // No hit: no tick.
        let mut other = listed.clone();
        other.words = Some(Ok(wordcheck::List::parse("lighthouse\n")));
        assert!(!plan(&other, &runner).asks_tick());
        // A list that did not read, or holds no word: said, and no child.
        let quiet = Fake::new("repo");
        let mut unread = listed.clone();
        unread.words = Some(Err(r"C:\Lists\words.txt did not read: gone".to_owned()));
        let unread = plan(&unread, &quiet);
        assert!(
            matches!(&unread.check, Some(Check::NoList(why)) if why.contains("did not read: gone"))
        );
        let mut empty = listed.clone();
        empty.words = Some(Ok(wordcheck::List::parse("\n\n")));
        assert!(matches!(
            plan(&empty, &quiet).check,
            Some(Check::NoList(why)) if why.contains("holds no word")
        ));
        assert!(quiet.asked().is_empty());
        // A pass that failed: in git's words, and the tick asked.
        let mut failing = Fake::new("repo");
        failing.log = Some(log(128, "", "fatal: not a git repository"));
        let failed = plan(&listed, &failing);
        assert!(
            matches!(&failed.check, Some(Check::Failed(why)) if why.contains("not a git repository")),
            "{:?}",
            failed.check
        );
        assert!(failed.asks_tick());
        // A refused plan never reads: a data repository, a session at work; nor Make private.
        let data = Facts {
            target: Some(Target {
                data: true,
                ..beta.clone()
            }),
            ..listed.clone()
        };
        assert!(plan(&data, &quiet).check.is_none());
        let mut busy = listed.clone();
        busy.refusals = vec!["a session works there".to_owned()];
        assert!(plan(&busy, &quiet).check.is_none());
        let alpha = project(
            &scratch,
            "alpha",
            Some("https://github.com/example/alpha.git"),
        );
        let private = Facts {
            verb: Verb::MakePrivate,
            target: Some(alpha),
            ..listed.clone()
        };
        assert!(plan(&private, &quiet).check.is_none());
        assert!(quiet.asked().is_empty());
    }

    fn args(plan: &Plan) -> Vec<String> {
        plan.command.as_ref().expect("a command").args.clone()
    }

    #[test]
    fn verbs_read_and_word_themselves() {
        for verb in Verb::ALL {
            assert_eq!(Verb::parse(verb.id()), Some(verb));
            assert!(verb.label().ends_with('…'));
        }
        assert_eq!(Verb::parse("Back-Up-Now"), Some(Verb::BackUpNow));
        assert_eq!(Verb::parse("publish"), None);
        assert_eq!(Verb::BackUpNow.kind(), BACKUP);
        assert_eq!(Verb::Delete.kind(), HOSTING);
        let local = hosting::classify(&["omega"], None, Some(&account()));
        assert_eq!(Verb::offered(&local), [Verb::CreatePrivate]);
        let orphan = hosting::classify(&["delta"], None, Some(&account()));
        assert_eq!(Verb::offered(&orphan), [Verb::Archive, Verb::Delete]);
        let archived_orphan = hosting::classify(&["epsilon"], None, Some(&account()));
        assert_eq!(Verb::offered(&archived_orphan), [Verb::Delete]);
        let public = hosting::classify(
            &["alpha"],
            Some("https://github.com/example/alpha.git"),
            Some(&account()),
        );
        assert_eq!(
            Verb::offered(&public),
            [Verb::MakePrivate, Verb::RemoveRemote]
        );
        let private = hosting::classify(
            &["beta"],
            Some("git@github.com:example/beta.git"),
            Some(&account()),
        );
        assert_eq!(
            Verb::offered(&private),
            [Verb::MakePublic, Verb::RemoveRemote]
        );
        let elsewhere = hosting::classify(&["x"], Some("https://gitlab.com/a/x.git"), None);
        assert_eq!(Verb::offered(&elsewhere), [Verb::RemoveRemote]);
    }

    #[test]
    fn reads_gh_s_account() {
        let account = parse_auth(&auth_json("example", "gist, read:org, repo, workflow")).unwrap();
        assert_eq!(account.login, "example");
        assert_eq!(account.scopes, ["gist", "read:org", "repo", "workflow"]);
        assert!(parse_auth("{}").is_err());
        assert!(parse_auth("not json").is_err());
        let failed =
            r#"{"hosts":{"github.com":[{"state":"error","active":true,"login":"example"}]}}"#;
        assert!(parse_auth(failed).unwrap_err().contains("error"));
    }

    #[test]
    fn create_names_the_login_and_the_folder_and_refuses_what_it_cannot_push() {
        let scratch = Scratch::new("actions-create");
        let runner = Fake::new("repo");
        let omega = project(&scratch, "omega", None);
        let plan = plan(&facts(Verb::CreatePrivate, omega.clone()), &runner);
        assert!(!plan.is_refused(), "{:?}", plan.problems);
        let source = omega.folder.display().to_string();
        assert_eq!(
            args(&plan),
            [
                "repo",
                "create",
                "example/omega",
                "--private",
                "--source",
                &source,
                "--push"
            ]
        );
        assert!(
            plan.command
                .as_ref()
                .unwrap()
                .line()
                .starts_with("gh repo create example/omega")
        );
        assert_eq!(runner.asked().len(), 1, "gh auth status alone, nothing run");
        // A remote already, an orphan's name, no list, no commit, a data repository.
        let beta = project(
            &scratch,
            "beta",
            Some("https://github.com/example/beta.git"),
        );
        let refused = |facts: Facts| super::plan(&facts, &Fake::new("repo")).problems;
        assert!(refused(facts(Verb::CreatePrivate, beta))[0].contains("has a remote already"));
        let delta = project(&scratch, "delta", None);
        assert!(
            refused(facts(Verb::CreatePrivate, delta))[0].contains("example/delta (an orphan)")
        );
        let mut unlisted = facts(Verb::CreatePrivate, omega.clone());
        unlisted.account = None;
        assert!(refused(unlisted)[0].contains("not read yet"));
        let empty = Target {
            has_commit: Some(false),
            ..omega.clone()
        };
        assert!(refused(facts(Verb::CreatePrivate, empty))[0].contains("no commit yet"));
        let data = Target {
            data: true,
            ..omega.clone()
        };
        assert!(refused(facts(Verb::CreatePrivate, data))[0].contains("data repository"));
        // gh's login unknown: refused with its words.
        let mut offline = Fake::new("repo");
        offline.auth = Err("no answer within 300s".to_owned());
        let plan = super::plan(&facts(Verb::CreatePrivate, omega), &offline);
        assert!(
            plan.problems[0].contains("no answer within"),
            "{:?}",
            plan.problems
        );
    }

    #[test]
    fn visibility_changes_repeat_the_warnings_and_refuse_what_does_not_apply() {
        let scratch = Scratch::new("actions-visibility");
        let runner = Fake::new("repo");
        let beta = project(&scratch, "beta", Some("git@github.com:example/beta.git"));
        let public = plan(&facts(Verb::MakePublic, beta.clone()), &runner);
        assert_eq!(
            args(&public),
            [
                "repo",
                "edit",
                "example/beta",
                "--visibility",
                "public",
                "--accept-visibility-change-consequences"
            ]
        );
        assert_eq!(public.warnings, [VISIBILITY_CONSEQUENCES, PUBLISHES]);
        assert!(
            runner.asked().is_empty(),
            "no child to plan a visibility change"
        );
        let private = plan(&facts(Verb::MakePrivate, beta.clone()), &runner);
        assert!(private.problems[0].contains("private already"));
        let alpha = project(
            &scratch,
            "alpha",
            Some("https://github.com/example/alpha.git"),
        );
        let private = plan(&facts(Verb::MakePrivate, alpha), &runner);
        assert_eq!(args(&private)[4], "private");
        assert_eq!(
            private.warnings,
            [VISIBILITY_CONSEQUENCES],
            "rule 3's words for public only"
        );
        let gamma = project(
            &scratch,
            "gamma",
            Some("https://github.com/example/gamma.git"),
        );
        assert!(plan(&facts(Verb::MakePublic, gamma), &runner).problems[0].contains("archived"));
        let local = project(&scratch, "omega", None);
        assert!(plan(&facts(Verb::MakePublic, local), &runner).problems[0].contains("no remote"));
        let data = Target { data: true, ..beta };
        assert!(
            plan(&facts(Verb::MakePublic, data), &runner)
                .problems
                .iter()
                .any(|problem| problem.contains("data repository"))
        );
    }

    #[test]
    fn remove_remote_names_the_origin_and_leaves_github_alone() {
        let scratch = Scratch::new("actions-remove");
        let runner = Fake::new("repo");
        let alpha = project(
            &scratch,
            "alpha",
            Some("https://github.com/example/alpha.git"),
        );
        let plan = plan(&facts(Verb::RemoveRemote, alpha.clone()), &runner);
        let folder = alpha.folder.display().to_string();
        assert_eq!(args(&plan), ["-C", &folder, "remote", "remove", "origin"]);
        assert!(plan.words[0].contains("https://github.com/example/alpha.git"));
        assert!(plan.words[1].contains("example/alpha is left as it is"));
        assert!(plan.command.as_ref().unwrap().line().starts_with("git -C "));
        let omega = project(&scratch, "omega", None);
        assert!(
            super::plan(&facts(Verb::RemoveRemote, omega), &runner).problems[0]
                .contains("no origin")
        );
    }

    #[test]
    fn archive_and_delete_are_for_an_orphan_and_delete_needs_its_scope() {
        let scratch = Scratch::new("actions-orphan");
        let delta = project(&scratch, "delta", None);
        let runner = Fake::new("gist, repo");
        let archive = plan(&facts(Verb::Archive, delta.clone()), &runner);
        assert_eq!(archive.target, "example/delta");
        assert_eq!(
            args(&archive),
            ["repo", "archive", "example/delta", "--yes"]
        );
        let delete = plan(&facts(Verb::Delete, delta.clone()), &runner);
        assert!(delete.is_refused());
        assert!(
            delete.problems[0].contains(ADD_DELETE_SCOPE),
            "{:?}",
            delete.problems
        );
        let scoped = plan(&facts(Verb::Delete, delta), &Fake::new("repo, delete_repo"));
        assert_eq!(args(&scoped), ["repo", "delete", "example/delta", "--yes"]);
        assert!(scoped.warnings[0].contains("--accept-cascading-deletion-of-private-forks"));
        let epsilon = project(&scratch, "epsilon", None);
        assert!(
            plan(&facts(Verb::Archive, epsilon), &runner).problems[0].contains("archived already")
        );
        let alpha = project(
            &scratch,
            "alpha",
            Some("https://github.com/example/alpha.git"),
        );
        assert!(plan(&facts(Verb::Delete, alpha), &runner).problems[0].contains("only an orphan"));
    }

    #[test]
    fn the_windows_refusals_join_a_projects_plan() {
        let scratch = Scratch::new("actions-refusals");
        let mut facts = facts(
            Verb::RemoveRemote,
            project(
                &scratch,
                "alpha",
                Some("https://github.com/example/alpha.git"),
            ),
        );
        facts.refusals = vec!["a visual session 3c9d2e71 is busy".to_owned()];
        let plan = plan(&facts, &Fake::new("repo"));
        assert!(plan.is_refused() && plan.command.is_some());
        assert_eq!(plan.problems, ["a visual session 3c9d2e71 is busy"]);
    }

    /// The job's settings in a scratch folder with its script beside them, under another case.
    fn job(scratch: &Scratch, mounted: bool) -> PathBuf {
        let folder = scratch.0.join("jobs");
        fs::create_dir_all(&folder).unwrap();
        let target = if mounted {
            scratch.0.join("target").display().to_string()
        } else {
            r"B:\Nowhere".to_owned()
        };
        let settings = folder.join("nightly-job.psd1");
        fs::write(&settings, format!("@{{\n    Target = '{target}'\n}}\n")).unwrap();
        fs::write(
            folder.join("Nightly-Job.ps1"),
            "param([string]$Mode, [switch]$Force)\n",
        )
        .unwrap();
        settings
    }

    #[test]
    fn back_up_now_runs_the_script_beside_the_settings() {
        let scratch = Scratch::new("actions-backup");
        let settings = job(&scratch, true);
        assert_eq!(
            job_script(&settings).unwrap().file_name().unwrap(),
            "Nightly-Job.ps1"
        );
        let facts = Facts {
            verb: Verb::BackUpNow,
            target: None,
            account: None,
            refusals: Vec::new(),
            backup: Some(settings.clone()),
            words: None,
        };
        let plan = plan(&facts, &Fake::new(""));
        assert!(!plan.is_refused(), "{:?}", plan.problems);
        let command = plan.command.as_ref().unwrap();
        assert!(command.long);
        // It runs from its script's folder, waited on for up to half an hour.
        assert_eq!(
            command.folder.as_deref(),
            Some(scratch.0.join("jobs").as_path())
        );
        assert_eq!(command.timeout, BACKUP_TIMEOUT);
        // The window's refusal (a run it started still going) refuses it.
        let twice = Facts {
            refusals: vec!["the backup job the window started is still running".to_owned()],
            ..facts.clone()
        };
        assert!(super::plan(&twice, &Fake::new("")).is_refused());
        let script = scratch
            .0
            .join("jobs")
            .join("Nightly-Job.ps1")
            .display()
            .to_string();
        assert_eq!(
            command.args,
            [
                "-NoProfile",
                "-NonInteractive",
                "-File",
                &script,
                "-Mode",
                "Run",
                "-Force"
            ]
        );
        // No job, an unreadable one, no script, a target not mounted.
        let refused = |backup: Option<PathBuf>| {
            super::plan(
                &Facts {
                    backup,
                    ..facts.clone()
                },
                &Fake::new(""),
            )
            .problems
        };
        assert!(refused(None)[0].contains("no backup job is set"));
        assert!(refused(Some(scratch.0.join("missing.psd1")))[0].contains("could not be read"));
        fs::remove_file(scratch.0.join("jobs").join("Nightly-Job.ps1")).unwrap();
        assert!(refused(Some(settings))[0].contains("no script beside"));
        let other = Scratch::new("actions-backup-unmounted");
        assert!(refused(Some(job(&other, false)))[0].contains("not mounted"));
    }

    #[test]
    fn a_run_says_how_it_ended_and_is_logged() {
        let scratch = Scratch::new("actions-run");
        let ledger = scratch.0.join("ledger.sqlite");
        let alpha = project(
            &scratch,
            "alpha",
            Some("https://github.com/example/alpha.git"),
        );
        let runner = Fake::new("repo");
        let planned = plan(&facts(Verb::MakePrivate, alpha.clone()), &runner);
        let ran = run(&planned, &runner, Some(&ledger)).expect("it ran");
        assert_eq!(ran.end, End::Done);
        assert_eq!(ran.printed, "done");
        assert!(ran.unlogged.is_none(), "{:?}", ran.unlogged);
        assert_eq!(ran.words(), "alpha: made private");
        assert_eq!(
            runner.asked().last().unwrap(),
            &planned.command.as_ref().unwrap().line()
        );
        // A failure in gh's words, never retried.
        let mut failing = Fake::new("repo");
        failing.exit = Exit {
            code: Some(1),
            stdout: String::new(),
            stderr: "HTTP 403: Must have admin rights to Repository.\n".to_owned(),
        };
        let ran = run(&planned, &failing, Some(&ledger)).unwrap();
        assert_eq!(
            ran.end,
            End::Failed(
                "gh repo edit failed (exit code 1): HTTP 403: Must have admin rights to Repository."
                    .to_owned()
            )
        );
        assert_eq!(failing.asked().len(), 1, "run once");
        // A refused plan runs nothing.
        let refused = Plan {
            problems: vec!["a session works there".to_owned()],
            ..planned.clone()
        };
        assert!(run(&refused, &runner, Some(&ledger)).is_none());
        let read = Ledger::open_read_only(&ledger).unwrap();
        let rows = read.events(HOSTING, None).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].project.as_deref(), Some("alpha"));
        assert!(
            rows[1].detail.as_deref().is_some_and(|detail| detail
                .starts_with("make-private: alpha: made private · gh repo edit example/alpha")),
            "{:?}",
            rows[1].detail
        );
        // The backup's exit code 2 is a warning, logged under no project.
        let settings = job(&scratch, true);
        let backup = plan(
            &Facts {
                verb: Verb::BackUpNow,
                target: None,
                account: None,
                refusals: Vec::new(),
                backup: Some(settings),
                words: None,
            },
            &runner,
        );
        let mut warned = Fake::new("");
        warned.exit.code = Some(2);
        let ran = run(&backup, &warned, Some(&ledger)).unwrap();
        assert!(matches!(ran.end, End::Warned(_)), "{:?}", ran.end);
        let rows = read.events(BACKUP, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].project, None);
    }
}
