//! The registry (D-6, D-8): `registry.toml` in the roaming profile, the projects the tool manages
//! and its settings, edited by hand and by the tool. It is read strictly through serde, so a typo
//! is an error that names its line, and changed in place through `toml_edit`, so the owner's
//! comments, order and layout survive. Each change reads the file again first, checks that the
//! result reads back, and writes it atomically, after a copy of the day's first state; a format
//! migration first keeps the file as it was beside it (D-29). A missing file reads as the default
//! registry, with this repository as its first project (D-5).

use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value};

use crate::notify::QuietHours;
use crate::proposal::{Dismissed, Kind, Reason};
use crate::rules::Rule;
use crate::session::Route;
use crate::{files, paths, signals, workspace};

/// The longest time limit a request may have, in minutes: a day (EP-14).
pub const MAX_ANSWER_TIMEOUT: u32 = 24 * 60;

/// The registry format this build reads and writes.
pub const FORMAT: u32 = 1;

/// Daily copies of the registry kept in the backups folder.
pub const KEEP_DAILY: usize = 14;

/// A project's priority when none is written: 1 is the highest, 5 the lowest.
pub const DEFAULT_PRIORITY: u8 = 3;

/// The first lines of a registry the tool writes.
const HEADER: &str = "\
# clauderoadmapwrangler's registry (D-8): the projects it manages and its settings. Edit it by
# hand at any time: the tool reads it again before each change, changes only the values it sets
# and keeps the rest, comments included.
";

/// A managed project's state (D-6), set by hand or by confirming a proposal. *Unadopted* is not
/// one of them: it is what discovery calls a folder the registry does not list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Active,
    Paused,
    Maintenance,
    Done,
    Archived,
    Unformalised,
}

impl State {
    pub const ALL: [State; 6] = [
        State::Active,
        State::Paused,
        State::Maintenance,
        State::Done,
        State::Archived,
        State::Unformalised,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            State::Active => "active",
            State::Paused => "paused",
            State::Maintenance => "maintenance",
            State::Done => "done",
            State::Archived => "archived",
            State::Unformalised => "unformalised",
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for State {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        State::ALL
            .into_iter()
            .find(|state| state.as_str() == s)
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "unknown state {s:?}: active, paused, maintenance, done, archived or unformalised"
                ))
            })
    }
}

/// What can go wrong with the registry; nothing is written when any of these is returned.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
    // The parsers' errors are boxed: they carry the offending line and would make every
    // `Result` here large.
    #[error("{}: {source}", path.display())]
    Syntax {
        path: PathBuf,
        source: Box<toml_edit::TomlError>,
    },
    #[error("{}: {source}", path.display())]
    Read {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
    #[error(
        "{} has format {found}; this build reads format {supported} and leaves the file as it is",
        path.display()
    )]
    Newer {
        path: PathBuf,
        found: u32,
        supported: u32,
    },
    #[error("{0}")]
    Invalid(String),
    #[error("no project named {0:?} in the registry")]
    UnknownProject(String),
}

impl Error {
    pub(crate) fn io(path: &Path, source: io::Error) -> Self {
        Error::Io {
            path: path.to_owned(),
            source,
        }
    }
}

/// The registry as the tool reads it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub format: u32,
    /// Folder names of the data root left out of the unadopted list (D-7).
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub settings: Settings,
    /// The global rule table (D-22), `[[rule]]`: the first that matches a brief applies; while
    /// it is empty, the built-in table stands in (`rules::built_in`).
    #[serde(default, rename = "rule")]
    pub rules: Vec<Rule>,
    #[serde(default, rename = "project")]
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// The folder whose top-level folders are the projects (D-6); by default the folder that holds
    /// this repository.
    pub data_root: Option<PathBuf>,
    /// The owner's VS Code workspace file, which Adopt appends to (D-7); relative to the data root.
    pub workspace_file: Option<PathBuf>,
    /// The Documents backup job's settings file, which the backup badge reads (D-24); relative to
    /// the data root or a full path.
    pub backup_settings: Option<PathBuf>,
    /// The backup job's run log; relative to the data root or a full path.
    pub backup_log: Option<PathBuf>,
    /// The Claude Code CLI the session runner starts (D-15), a full path; by default the launcher
    /// in `~\.local\bin`, else `claude.exe` on PATH.
    pub claude_cli: Option<PathBuf>,
    /// The VS Code a visual launch opens (D-16), a full path to its `Code.exe`; by default the
    /// user install, the system install, else the install whose `bin\code.cmd` is on PATH.
    pub vscode: Option<PathBuf>,
    /// The loopback port the window listens on for the hooks and the status line (D-20, D-28;
    /// EP-12); by default `signals::DEFAULT_PORT`.
    pub port: Option<u16>,
    /// How headless runs' questions reach the owner (D-19; EP-14): `protocol`, the default, or
    /// `defer`, the documented fallback for a CLI that breaks the protocol.
    pub decisions: Option<Route>,
    /// The hours the window's notifications are held back, in local time, such as `22:00-07:00`
    /// (D-21; EP-15); none, the default, for notifications at any hour.
    pub quiet_hours: Option<QuietHours>,
    /// Whether the window polls the usage endpoint (D-23; EP-17): on by default, written only
    /// as `false`; off leaves the meter on the status lines alone.
    pub usage_endpoint: Option<bool>,
    /// The Fable-scoped weekly share, a whole percent, above which an S brief is proposed the
    /// downshift's model (D-23; EP-18); by default `rules::DEFAULT_DOWNSHIFT_PERCENT`.
    pub fable_downshift_percent: Option<u8>,
}

impl Settings {
    /// The keys `crw setting` reads and sets.
    pub const KEYS: [&str; 11] = [
        "data_root",
        "workspace_file",
        "backup_settings",
        "backup_log",
        "claude_cli",
        "vscode",
        "port",
        "decisions",
        "quiet_hours",
        "usage_endpoint",
        "fable_downshift_percent",
    ];

    /// The keys whose value is a path; `port` is a number, `decisions` a word, `quiet_hours`
    /// two times, `usage_endpoint` on or off and `fable_downshift_percent` a percent.
    pub fn is_path(key: &str) -> bool {
        !matches!(
            key,
            "port" | "decisions" | "quiet_hours" | "usage_endpoint" | "fable_downshift_percent"
        )
    }

    /// Whether the window may poll the usage endpoint.
    pub fn usage_endpoint_on(&self) -> bool {
        self.usage_endpoint != Some(false)
    }

    /// The Fable share above which the downshift proposes its model for an S brief (D-23).
    pub fn fable_downshift_percent(&self) -> u8 {
        self.fable_downshift_percent
            .unwrap_or(crate::rules::DEFAULT_DOWNSHIFT_PERCENT)
    }

    /// A setting's value as written, by its key.
    pub fn get(&self, key: &str) -> Option<String> {
        let path = |path: &Option<PathBuf>| path.as_ref().map(|p| p.display().to_string());
        match key {
            "data_root" => path(&self.data_root),
            "workspace_file" => path(&self.workspace_file),
            "backup_settings" => path(&self.backup_settings),
            "backup_log" => path(&self.backup_log),
            "claude_cli" => path(&self.claude_cli),
            "vscode" => path(&self.vscode),
            "port" => self.port.map(|port| port.to_string()),
            "decisions" => self.decisions.map(|route| route.as_str().to_owned()),
            "quiet_hours" => self.quiet_hours.map(|hours| hours.to_string()),
            "usage_endpoint" => self
                .usage_endpoint
                .map(|on| if on { "on" } else { "off" }.to_owned()),
            "fable_downshift_percent" => self.fable_downshift_percent.map(|p| p.to_string()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: String,
    /// A folder of the data root, or an absolute path.
    pub path: PathBuf,
    pub state: State,
    #[serde(default = "default_priority")]
    pub priority: u8,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub launch: Launch,
    #[serde(default)]
    pub autopilot: Autopilot,
    /// Folders a headless session of this project may read and never edit (D-16, D-18 addenda):
    /// folders of the data root, or full paths.
    #[serde(default)]
    pub neighbours: Vec<PathBuf>,
    /// The project's own rules, `[[project.rule]]`, applied over the global table (D-22).
    #[serde(default, rename = "rule")]
    pub rules: Vec<Rule>,
    /// The proposals the owner dismissed, each with the fact it rested on (D-6 addendum).
    #[serde(default)]
    pub dismissed: Dismissed,
    /// The minutes a headless session's request may wait for the owner before the tool defers
    /// the run (D-19; EP-14); none, the default, for no limit.
    #[serde(default)]
    pub answer_timeout: Option<u32>,
    /// Whether the tool records the start of each session here and may revert one (EP-15b): on,
    /// the default, unless `revert = false` turns both off (the owner's choice for the data
    /// repositories).
    #[serde(default = "default_revert")]
    pub revert: bool,
}

fn default_priority() -> u8 {
    DEFAULT_PRIORITY
}

fn default_revert() -> bool {
    true
}

/// The project's launch defaults, which the rule table resolves over the tool's own (EP-10).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Launch {
    pub mode: Option<LaunchMode>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub permission_mode: Option<String>,
    /// The launch prompt in the project's own words, with placeholders (D-17; `prompt`).
    pub template: Option<String>,
}

/// How a session is started (D-16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LaunchMode {
    Visual,
    Headless,
}

impl LaunchMode {
    pub fn as_str(self) -> &'static str {
        match self {
            LaunchMode::Visual => "visual",
            LaunchMode::Headless => "headless",
        }
    }
}

/// The autopilot's switch; its rules and caps arrive with EP-19.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Autopilot {
    #[serde(default)]
    pub enabled: bool,
}

impl Registry {
    fn empty() -> Self {
        Self {
            format: FORMAT,
            ignore: Vec::new(),
            settings: Settings::default(),
            rules: Vec::new(),
            projects: Vec::new(),
        }
    }

    pub fn data_root(&self) -> PathBuf {
        self.settings
            .data_root
            .clone()
            .unwrap_or_else(paths::default_data_root)
    }

    /// The loopback port of the signals (EP-12).
    pub fn port(&self) -> u16 {
        self.settings.port.unwrap_or(signals::DEFAULT_PORT)
    }

    /// The workspace file's full path, when one is set.
    pub fn workspace_file(&self) -> Option<PathBuf> {
        let file = self.settings.workspace_file.as_ref()?;
        Some(self.data_root().join(file))
    }

    /// The backup job's settings file and run log, as full paths, when they are set (D-24).
    pub fn backup_files(&self) -> (Option<PathBuf>, Option<PathBuf>) {
        let full = |path: &Option<PathBuf>| path.as_ref().map(|path| self.data_root().join(path));
        (
            full(&self.settings.backup_settings),
            full(&self.settings.backup_log),
        )
    }

    /// The project called `name`, in any case.
    pub fn project(&self, name: &str) -> Option<&Project> {
        self.projects
            .iter()
            .find(|project| project.name.eq_ignore_ascii_case(name))
    }

    /// The checks serde cannot make.
    pub(crate) fn validate(&self) -> Result<(), Error> {
        let invalid = |message: String| Err(Error::Invalid(message));
        for (n, rule) in self.rules.iter().enumerate() {
            if let Err(why) = rule.check() {
                return invalid(format!("rule {}: {why}", n + 1));
            }
        }
        if self.settings.port == Some(0) {
            return invalid("settings.port must be a port number, not 0".to_owned());
        }
        if let Some(percent) = self.settings.fable_downshift_percent
            && !(1..=100).contains(&percent)
        {
            return invalid(format!(
                "settings.fable_downshift_percent must be a percent from 1 to 100, not {percent}"
            ));
        }
        for (key, path) in [
            ("data_root", &self.settings.data_root),
            ("claude_cli", &self.settings.claude_cli),
            ("vscode", &self.settings.vscode),
        ] {
            if let Some(path) = path
                && !path.is_absolute()
            {
                return invalid(format!(
                    "settings.{key} must be a full path, not {}",
                    path.display()
                ));
            }
        }
        let root = self.data_root();
        let mut names = HashSet::new();
        let mut folders = HashSet::new();
        for project in &self.projects {
            let name = project.name.trim();
            if name.is_empty() || project.path.as_os_str().is_empty() {
                return invalid("every project needs a name and a path".to_owned());
            }
            if !(1..=5).contains(&project.priority) {
                return invalid(format!(
                    "{name}: priority {} is not between 1 and 5",
                    project.priority
                ));
            }
            if !names.insert(name.to_lowercase()) {
                return invalid(format!("{name} is registered twice"));
            }
            if let Some(minutes) = project.answer_timeout
                && !(1..=MAX_ANSWER_TIMEOUT).contains(&minutes)
            {
                return invalid(format!(
                    "{name}: answer_timeout is minutes from 1 to {MAX_ANSWER_TIMEOUT}, not {minutes}"
                ));
            }
            let folder = paths::normalize(&project.folder(&root));
            if !folders.insert(folder.to_string_lossy().to_lowercase()) {
                return invalid(format!("{name}: another project has the same folder"));
            }
            for neighbour in &project.neighbours {
                let text = neighbour.to_string_lossy();
                if text.trim().is_empty() || text.starts_with(r"\\") || text.starts_with("//") {
                    return invalid(format!(
                        "{name}: a neighbour is a folder on a drive, not {text:?}"
                    ));
                }
                if paths::same(&root.join(neighbour), &folder) {
                    return invalid(format!("{name}: its own folder is not a neighbour"));
                }
            }
            for (n, rule) in project.rules.iter().enumerate() {
                if let Err(why) = rule.check() {
                    return invalid(format!("{name}: rule {}: {why}", n + 1));
                }
            }
            let launch = &project.launch;
            for (key, value) in [
                ("model", &launch.model),
                ("effort", &launch.effort),
                ("permission_mode", &launch.permission_mode),
                ("template", &launch.template),
            ] {
                if value.as_ref().is_some_and(|value| value.trim().is_empty()) {
                    return invalid(format!("{name}: launch.{key} is empty"));
                }
            }
        }
        for folder in &self.ignore {
            if folder.trim().is_empty() || folder.contains(['/', '\\']) {
                return invalid(format!(
                    "ignore lists folder names of the data root, not {folder:?}"
                ));
            }
        }
        Ok(())
    }
}

impl Project {
    /// A project as a fresh `[[project]]` table reads: the default priority, not pinned, no
    /// launch defaults, neighbours, rules or dismissals.
    pub fn new(name: &str, path: &Path, state: State) -> Self {
        Self {
            name: name.to_owned(),
            path: path.to_owned(),
            state,
            priority: DEFAULT_PRIORITY,
            pinned: false,
            launch: Launch::default(),
            autopilot: Autopilot::default(),
            neighbours: Vec::new(),
            rules: Vec::new(),
            dismissed: Dismissed::default(),
            answer_timeout: None,
            revert: true,
        }
    }

    /// The project's folder: its path resolved against the data root.
    pub fn folder(&self, data_root: &Path) -> PathBuf {
        data_root.join(&self.path)
    }

    /// How long a headless session's request may wait for the owner (D-19); none for no limit.
    pub fn answer_timeout(&self) -> Option<std::time::Duration> {
        self.answer_timeout
            .map(|minutes| std::time::Duration::from_secs(u64::from(minutes) * 60))
    }

    /// The neighbour folders as full paths, resolved against the data root.
    pub fn neighbour_folders(&self, data_root: &Path) -> Vec<PathBuf> {
        self.neighbours
            .iter()
            .map(|neighbour| paths::normalize(&data_root.join(neighbour)))
            .collect()
    }
}

/// What a missing registry starts from (D-5, D-6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Defaults {
    pub data_root: PathBuf,
    /// The first registered project: the repository this build comes from.
    pub repository: PathBuf,
}

impl Defaults {
    pub fn of_this_build() -> Self {
        Self {
            data_root: paths::default_data_root(),
            repository: paths::repository(),
        }
    }
}

/// One format step: `steps[i]` brings a document from format `i + 1` to `i + 2`.
type Step = fn(&mut DocumentMut);

#[derive(Clone, Copy)]
struct Migrations {
    current: u32,
    steps: &'static [Step],
}

/// None yet: format 1 is the first.
const MIGRATIONS: Migrations = Migrations {
    current: FORMAT,
    steps: &[],
};

/// The registry file, the folder for its daily copies, and what it starts from when missing.
pub struct Store {
    pub path: PathBuf,
    pub backups: PathBuf,
    pub defaults: Defaults,
    migrations: Migrations,
}

/// The registry as [`Store::load`] found it.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    pub registry: Registry,
    /// Whether the file exists; when it does not, `registry` is the default.
    pub written: bool,
}

impl Store {
    pub fn new(path: PathBuf, backups: PathBuf, defaults: Defaults) -> Self {
        Self {
            path,
            backups,
            defaults,
            migrations: MIGRATIONS,
        }
    }

    /// The owner's registry, or the one in `CRW_HOME`, with this build's defaults; `None` without
    /// a profile folder.
    pub fn locate() -> Option<Self> {
        Some(Self::new(
            paths::registry_file()?,
            paths::backups_dir()?,
            Defaults::of_this_build(),
        ))
    }

    /// Reads the registry without writing anything; an older format is brought up to date in
    /// memory only.
    pub fn load(&self) -> Result<Loaded, Error> {
        let bytes = self.read()?;
        let (document, _) = self.document(bytes.as_deref())?;
        Ok(Loaded {
            registry: self.typed(&document.to_string())?,
            written: bytes.is_some(),
        })
    }

    /// Reads the registry again, lets `edit` change it, and writes the result if it reads back
    /// and differs from the file: after a copy of the file as it was before the day's first
    /// change, and after a copy beside it when this change migrates its format.
    pub fn change<T>(
        &self,
        today: &str,
        edit: impl FnOnce(&mut Changes<'_>) -> Result<T, Error>,
    ) -> Result<(Registry, T), Error> {
        let bytes = self.read()?;
        let (mut document, migrated_from) = self.document(bytes.as_deref())?;
        let before = self.typed(&document.to_string())?;
        let out = edit(&mut Changes {
            document: &mut document,
            registry: &before,
        })?;
        let text = document.to_string();
        // Never write a file the tool could not read back.
        let after = self.typed(&text)?;
        if bytes.as_deref() == Some(text.as_bytes()) {
            return Ok((after, out));
        }
        if let Some(old) = &bytes {
            if let Some(from) = migrated_from {
                let dir = self.path.parent().unwrap_or(Path::new("."));
                files::dated_copy(old, dir, &format!("registry.v{from}"), today, ".toml")
                    .map_err(|e| Error::io(dir, e))?;
            }
            files::daily_copy(&self.path, &self.backups, today, KEEP_DAILY)
                .map_err(|e| Error::io(&self.backups, e))?;
        } else if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
        }
        files::write_atomic(&self.path, text.as_bytes()).map_err(|e| Error::io(&self.path, e))?;
        Ok((after, out))
    }

    /// The file's bytes, or `None` when there is no file yet.
    fn read(&self) -> Result<Option<Vec<u8>>, Error> {
        match fs::read(&self.path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::io(&self.path, e)),
        }
    }

    /// The document to work on, brought up to the current format, and the format it was
    /// migrated from; the default registry when there is no file.
    fn document(&self, bytes: Option<&[u8]>) -> Result<(DocumentMut, Option<u32>), Error> {
        let text = match bytes {
            Some(bytes) => std::str::from_utf8(bytes)
                .map_err(|_| Error::Invalid(format!("{} is not UTF-8", self.path.display())))?
                .to_owned(),
            None => default_text(&self.defaults),
        };
        let mut document: DocumentMut = text.parse().map_err(|source| Error::Syntax {
            path: self.path.clone(),
            source: Box::new(source),
        })?;
        let format = document
            .get("format")
            .and_then(Item::as_integer)
            .and_then(|n| u32::try_from(n).ok())
            .filter(|&n| n >= 1)
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "{} needs a `format = {FORMAT}` line",
                    self.path.display()
                ))
            })?;
        let Migrations { current, steps } = self.migrations;
        if format > current {
            return Err(Error::Newer {
                path: self.path.clone(),
                found: format,
                supported: current,
            });
        }
        if format == current {
            return Ok((document, None));
        }
        for step in &steps[format as usize - 1..current as usize - 1] {
            step(&mut document);
        }
        set_keeping_decor(&mut document, "format", Value::from(i64::from(current)));
        Ok((document, Some(format)))
    }

    fn typed(&self, text: &str) -> Result<Registry, Error> {
        let registry: Registry = toml::from_str(text).map_err(|source| Error::Read {
            path: self.path.clone(),
            source: Box::new(source),
        })?;
        registry.validate()?;
        Ok(registry)
    }
}

/// The text of a registry that does not exist yet: the header, the data root, the aggregate
/// workspace file when the data root holds exactly one file that lists more than one folder (D-7
/// addendum), and the repository as the first project (D-5).
fn default_text(defaults: &Defaults) -> String {
    let mut document: DocumentMut = format!("{HEADER}format = {FORMAT}\nignore = []\n")
        .parse()
        .expect("the header is valid TOML");
    let mut settings = Table::new();
    settings.decor_mut().set_prefix("\n");
    settings["data_root"] = toml_edit::value(defaults.data_root.display().to_string());
    if let [file] = workspace::aggregates(&defaults.data_root).as_slice() {
        settings["workspace_file"] = toml_edit::value(file);
    }
    document["settings"] = Item::Table(settings);
    let repository = &defaults.repository;
    let name = repository
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let beside_the_others = repository
        .parent()
        .is_some_and(|parent| paths::same(parent, &defaults.data_root));
    let path = if beside_the_others {
        name.clone()
    } else {
        repository.display().to_string()
    };
    let empty = Registry::empty();
    Changes {
        document: &mut document,
        registry: &empty,
    }
    .add_project(&name, &path, State::Active)
    .expect("an empty registry takes any project");
    document.to_string()
}

/// Sets `key` in `table` to `new`, keeping the spacing and the comment of the value it replaces.
fn set_keeping_decor(table: &mut Table, key: &str, mut new: Value) {
    if let Some(old) = table.get(key).and_then(Item::as_value) {
        *new.decor_mut() = old.decor().clone();
    }
    table[key] = Item::Value(new);
}

/// The changes one [`Store::change`] makes, applied to the document in place.
pub struct Changes<'a> {
    document: &'a mut DocumentMut,
    registry: &'a Registry,
}

impl Changes<'_> {
    /// The registry as it stood before this change.
    pub fn registry(&self) -> &Registry {
        self.registry
    }

    /// Appends a `[[project]]` table with the default priority, not pinned.
    pub fn add_project(&mut self, name: &str, path: &str, state: State) -> Result<(), Error> {
        if self.registry.project(name).is_some() {
            return Err(Error::Invalid(format!("{name} is registered already")));
        }
        let projects = self
            .document
            .entry("project")
            .or_insert(Item::ArrayOfTables(ArrayOfTables::new()))
            .as_array_of_tables_mut()
            .ok_or_else(written_as_tables)?;
        let mut table = Table::new();
        table.decor_mut().set_prefix("\n");
        table["name"] = toml_edit::value(name);
        table["path"] = toml_edit::value(path);
        table["state"] = toml_edit::value(state.as_str());
        table["priority"] = toml_edit::value(i64::from(DEFAULT_PRIORITY));
        table["pinned"] = toml_edit::value(false);
        projects.push(table);
        Ok(())
    }

    pub fn set_state(&mut self, name: &str, state: State) -> Result<(), Error> {
        self.set(name, "state", Value::from(state.as_str()))
    }

    pub fn set_priority(&mut self, name: &str, priority: u8) -> Result<(), Error> {
        if !(1..=5).contains(&priority) {
            return Err(Error::Invalid(format!(
                "priority {priority} is not between 1 and 5"
            )));
        }
        self.set(name, "priority", Value::from(i64::from(priority)))
    }

    pub fn set_pinned(&mut self, name: &str, pinned: bool) -> Result<(), Error> {
        self.set(name, "pinned", Value::from(pinned))
    }

    /// Sets the minutes a project's requests may wait before the run is deferred, or takes the
    /// limit away (D-19; EP-14).
    pub fn set_answer_timeout(&mut self, name: &str, minutes: Option<u32>) -> Result<(), Error> {
        match minutes {
            Some(minutes) if !(1..=MAX_ANSWER_TIMEOUT).contains(&minutes) => Err(Error::Invalid(
                format!("answer_timeout is minutes from 1 to {MAX_ANSWER_TIMEOUT}, not {minutes}"),
            )),
            Some(minutes) => self.set(name, "answer_timeout", Value::from(i64::from(minutes))),
            None => {
                self.project_table(name)?.remove("answer_timeout");
                Ok(())
            }
        }
    }

    /// Turns the recording of a project's session starts and its revert on or off (EP-15b). On is
    /// the default and takes the key out, so that a registry with no project turned off reads
    /// in the builds before it.
    pub fn set_revert(&mut self, name: &str, on: bool) -> Result<(), Error> {
        if on {
            self.project_table(name)?.remove("revert");
            Ok(())
        } else {
            self.set(name, "revert", Value::from(false))
        }
    }

    /// Sets one of [`Settings::KEYS`] in the `[settings]` table, keeping its comment;
    /// `quiet_hours` set to `none`, `usage_endpoint` set to `on` and `fable_downshift_percent`
    /// set to `default` (the defaults, which the builds before EP-17 and EP-18 would refuse to
    /// read) are taken out.
    pub fn set_setting(&mut self, key: &str, value: &str) -> Result<(), Error> {
        if !Settings::KEYS.contains(&key) {
            return Err(Error::Invalid(format!(
                "unknown setting {key:?}: {}",
                Settings::KEYS.join(", ")
            )));
        }
        let default = match key {
            "quiet_hours" => value.trim() == "none",
            "usage_endpoint" => match value.trim() {
                "on" => true,
                "off" => false,
                _ => {
                    return Err(Error::Invalid(format!(
                        "usage_endpoint takes on or off, not {value:?}"
                    )));
                }
            },
            "fable_downshift_percent" => value.trim() == "default",
            _ => false,
        };
        if default {
            if let Some(settings) = self
                .document
                .get_mut("settings")
                .and_then(Item::as_table_mut)
            {
                settings.remove(key);
            }
            return Ok(());
        }
        let value = match key {
            "port" => match value.trim().parse::<u16>() {
                Ok(port) if port > 0 => Value::from(i64::from(port)),
                _ => {
                    return Err(Error::Invalid(format!(
                        "{key} needs a port number from 1 to 65535, not {value:?}"
                    )));
                }
            },
            "decisions" => match value.trim() {
                word @ ("protocol" | "defer") => Value::from(word),
                _ => {
                    return Err(Error::Invalid(format!(
                        "decisions takes protocol or defer, not {value:?}"
                    )));
                }
            },
            "quiet_hours" => match QuietHours::parse(value) {
                Ok(hours) => Value::from(hours.to_string()),
                Err(why) => return Err(Error::Invalid(format!("{why}, or none"))),
            },
            "usage_endpoint" => Value::from(false),
            "fable_downshift_percent" => match value.trim().trim_end_matches('%').trim().parse() {
                Ok(percent @ 1..=100u8) => Value::from(i64::from(percent)),
                _ => {
                    return Err(Error::Invalid(format!(
                        "{key} takes a whole percent from 1 to 100, or default, not {value:?}"
                    )));
                }
            },
            _ if value.trim().is_empty() => {
                return Err(Error::Invalid(format!("{key} needs a path")));
            }
            _ => Value::from(value),
        };
        let settings = self
            .document
            .entry("settings")
            .or_insert_with(|| {
                let mut table = Table::new();
                table.decor_mut().set_prefix("\n");
                Item::Table(table)
            })
            .as_table_mut()
            .ok_or_else(|| Error::Invalid("settings must be a [settings] table".to_owned()))?;
        set_keeping_decor(settings, key, value);
        Ok(())
    }

    /// Adds `folder` to the ignore list; `false` when it is on it already.
    pub fn ignore(&mut self, folder: &str) -> Result<bool, Error> {
        if self
            .registry
            .ignore
            .iter()
            .any(|f| f.eq_ignore_ascii_case(folder))
        {
            return Ok(false);
        }
        self.document
            .entry("ignore")
            .or_insert(toml_edit::value(Array::new()))
            .as_array_mut()
            .ok_or_else(|| Error::Invalid("ignore must be a list of folder names".to_owned()))?
            .push(folder);
        Ok(true)
    }

    /// Takes `folder` off the ignore list; `false` when it was not on it.
    pub fn unignore(&mut self, folder: &str) -> bool {
        let Some(list) = self.document.get_mut("ignore").and_then(Item::as_array_mut) else {
            return false;
        };
        let before = list.len();
        list.retain(|value| {
            !value
                .as_str()
                .is_some_and(|f| f.eq_ignore_ascii_case(folder))
        });
        list.len() != before
    }

    /// Keeps the dismissal of a proposal in the project's `dismissed` key with the fact it rests
    /// on (D-6 addendum), replacing an earlier dismissal of the same kind.
    pub fn dismiss(&mut self, name: &str, reason: &Reason) -> Result<(), Error> {
        let fact = match *reason {
            Reason::AllRowsChecked { rows } => Value::from(i64::try_from(rows).unwrap_or(i64::MAX)),
            Reason::PickupGate { ep } => Value::from(ep.to_string()),
            Reason::NoCommitFor { last, .. } => Value::from(
                last.format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string()
                    .parse::<toml_edit::Datetime>()
                    .expect("a UTC time in RFC 3339 is a TOML datetime"),
            ),
        };
        let key = reason.kind().key();
        let item = self
            .project_table(name)?
            .entry("dismissed")
            .or_insert_with(|| {
                let mut value = Value::InlineTable(InlineTable::new());
                value
                    .decor_mut()
                    .set_suffix("  # proposals left out until their facts change (D-6)");
                Item::Value(value)
            });
        match item {
            // The tool's own form, laid out afresh at each change.
            Item::Value(Value::InlineTable(dismissed)) => {
                dismissed.insert(key, fact);
                dismissed.fmt();
            }
            // A table the owner wrote: its layout and comments stay.
            Item::Table(dismissed) => set_keeping_decor(dismissed, key, fact),
            _ => return Err(not_a_table(name)),
        }
        Ok(())
    }

    /// Takes a dismissal of `kind` back, so its proposal shows again; `false` when there was
    /// none. The `dismissed` key goes when it is left empty.
    pub fn undismiss(&mut self, name: &str, kind: Kind) -> Result<bool, Error> {
        let table = self.project_table(name)?;
        let (removed, empty) = match table.get_mut("dismissed") {
            None => return Ok(false),
            Some(Item::Value(Value::InlineTable(dismissed))) => {
                let removed = dismissed.remove(kind.key()).is_some();
                dismissed.fmt();
                (removed, dismissed.is_empty())
            }
            Some(Item::Table(dismissed)) => {
                (dismissed.remove(kind.key()).is_some(), dismissed.is_empty())
            }
            Some(_) => return Err(not_a_table(name)),
        };
        if empty {
            table.remove("dismissed");
        }
        Ok(removed)
    }

    fn set(&mut self, name: &str, key: &str, value: Value) -> Result<(), Error> {
        set_keeping_decor(self.project_table(name)?, key, value);
        Ok(())
    }

    /// The `[[project]]` table of the project called `name`, in any case.
    fn project_table(&mut self, name: &str) -> Result<&mut Table, Error> {
        if self.registry.project(name).is_none() {
            return Err(Error::UnknownProject(name.to_owned()));
        }
        self.document
            .get_mut("project")
            .and_then(Item::as_array_of_tables_mut)
            .and_then(|projects| {
                projects.iter_mut().find(|table| {
                    table
                        .get("name")
                        .and_then(Item::as_str)
                        .is_some_and(|n| n.eq_ignore_ascii_case(name))
                })
            })
            .ok_or_else(written_as_tables)
    }
}

fn written_as_tables() -> Error {
    Error::Invalid("the tool changes projects written as [[project]] tables only".to_owned())
}

fn not_a_table(name: &str) -> Error {
    Error::Invalid(format!("{name}: dismissed must be a table"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    const TODAY: &str = "2026-10-31";

    fn store(scratch: &Scratch) -> Store {
        let data = scratch.0.join("data");
        Store::new(
            scratch.0.join("config").join("registry.toml"),
            scratch.0.join("local").join("backups"),
            Defaults {
                repository: data.join("wrangler"),
                data_root: data,
            },
        )
    }

    const HAND_WRITTEN: &str = r#"# The owner's notes stay.
format = 1
ignore = ["scratch"]  # never a project

[settings]
data_root = 'C:\Data'

# The first one.
[[project]]
name = "alpha"
path = "alpha"
state = "active"   # since the start
priority = 2

[project.launch]
model = "fable"

[[project]]
name = "beta"
path = "beta"
state = "paused"
"#;

    #[test]
    fn a_missing_file_reads_as_the_default_with_the_repository_first() {
        let scratch = Scratch::new("registry-default");
        let store = store(&scratch);
        let data = &store.defaults.data_root;
        fs::create_dir_all(data).unwrap();
        // The projects' own single-folder files beside the one aggregate (D-7 addendum).
        for name in ["wrangler", "alpha"] {
            fs::write(
                data.join(format!("{name}.code-workspace")),
                workspace::single_folder(name),
            )
            .unwrap();
        }
        fs::write(
            data.join("main.code-workspace"),
            r#"{"folders": [{"path": "wrangler"}, {"path": "alpha"}]}"#,
        )
        .unwrap();
        let loaded = store.load().unwrap();
        assert!(!loaded.written);
        let registry = loaded.registry;
        assert_eq!(registry.format, FORMAT);
        assert_eq!(registry.data_root(), store.defaults.data_root);
        assert_eq!(
            registry.workspace_file(),
            Some(store.defaults.data_root.join("main.code-workspace"))
        );
        assert_eq!(registry.projects.len(), 1);
        let first = &registry.projects[0];
        assert_eq!(
            (first.name.as_str(), first.path.as_path(), first.state),
            ("wrangler", Path::new("wrangler"), State::Active)
        );
        assert_eq!((first.priority, first.pinned), (DEFAULT_PRIORITY, false));
        // Reading wrote nothing.
        assert!(!scratch.0.join("config").exists());
        assert!(!scratch.0.join("local").exists());

        // Two files listing several folders: neither is chosen; none at all: none either.
        fs::write(
            data.join("other.code-workspace"),
            r#"{"folders": [{"path": "a"}, {"path": "b"}]}"#,
        )
        .unwrap();
        assert_eq!(store.load().unwrap().registry.workspace_file(), None);
        fs::remove_file(data.join("other.code-workspace")).unwrap();
        fs::remove_file(data.join("main.code-workspace")).unwrap();
        assert_eq!(store.load().unwrap().registry.workspace_file(), None);
    }

    #[test]
    fn dismissals_are_kept_in_place_with_their_facts() {
        let scratch = Scratch::new("registry-dismissed");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        let last = chrono::DateTime::parse_from_rfc3339("2026-08-15T10:00:00Z")
            .unwrap()
            .to_utc();
        let gate = Reason::PickupGate {
            ep: "EP-5".parse().unwrap(),
        };
        let (registry, ()) = store
            .change(TODAY, |c| {
                c.dismiss("alpha", &gate)?;
                c.dismiss("alpha", &Reason::NoCommitFor { days: 75, last })?;
                c.dismiss("beta", &Reason::AllRowsChecked { rows: 30 })
            })
            .unwrap();
        let text = fs::read_to_string(&store.path).unwrap();
        let comment = "  # proposals left out until their facts change (D-6)";
        assert!(
            text.contains(&format!(
                "priority = 2\ndismissed = {{ pickup_gate = \"EP-5\", no_commit_since = \
                 2026-08-15T10:00:00Z }}{comment}\n\n[project.launch]\n"
            )),
            "{text}"
        );
        assert!(
            text.ends_with(&format!(
                "state = \"paused\"\ndismissed = {{ all_rows_checked = 30 }}{comment}\n"
            )),
            "{text}"
        );
        let alpha = &registry.project("alpha").unwrap().dismissed;
        assert!(alpha.covers(&gate));
        assert!(alpha.covers(&Reason::NoCommitFor { days: 99, last }));
        assert_eq!(alpha.all_rows_checked, None);
        assert_eq!(
            registry.project("beta").unwrap().dismissed.all_rows_checked,
            Some(30)
        );

        // A newer dismissal of a kind replaces the older; taking the last one back drops the key.
        let (registry, removed) = store
            .change(TODAY, |c| {
                c.dismiss(
                    "alpha",
                    &Reason::PickupGate {
                        ep: "EP-6".parse().unwrap(),
                    },
                )?;
                c.undismiss("alpha", Kind::NoCommit)?;
                Ok([
                    c.undismiss("beta", Kind::AllRowsChecked)?,
                    c.undismiss("beta", Kind::PickupGate)?,
                ])
            })
            .unwrap();
        assert_eq!(removed, [true, false]);
        let text = fs::read_to_string(&store.path).unwrap();
        assert!(
            text.contains(&format!(
                "dismissed = {{ pickup_gate = \"EP-6\" }}{comment}\n"
            )),
            "{text}"
        );
        assert_eq!(text.matches("dismissed").count(), 1, "{text}");
        assert_eq!(
            registry.project("alpha").unwrap().dismissed.pickup_gate,
            Some("EP-6".parse().unwrap())
        );
        assert!(store.change(TODAY, |c| c.dismiss("delta", &gate)).is_err());

        // Written by hand, as a table of its own, and wrong.
        for (dismissed, message) in [
            ("dismissed = { pickup_gate = \"5\" }", "is not an EP id"),
            ("dismissed = { all_rows_checked = -1 }", "all_rows_checked"),
            (
                "dismissed = { no_commit_since = 2026-08-15 }",
                "not a date and a time with its offset",
            ),
            ("dismissed = { stale = true }", "stale"),
        ] {
            let text =
                HAND_WRITTEN.replace("priority = 2\n", &format!("priority = 2\n{dismissed}\n"));
            fs::write(&store.path, &text).unwrap();
            let error = store.load().unwrap_err().to_string();
            assert!(error.contains(message), "{message}: {error}");
        }
        let table = HAND_WRITTEN.replace(
            "[project.launch]",
            "[project.dismissed]\nno_commit_since = 2026-08-15T10:00:00Z\n\n[project.launch]",
        );
        fs::write(&store.path, &table).unwrap();
        let (registry, ()) = store.change(TODAY, |c| c.dismiss("alpha", &gate)).unwrap();
        let alpha = &registry.project("alpha").unwrap().dismissed;
        assert!(alpha.covers(&gate) && alpha.no_commit_since == Some(last));
        assert!(fs::read_to_string(&store.path).unwrap().contains(
            "[project.dismissed]\nno_commit_since = 2026-08-15T10:00:00Z\npickup_gate = \"EP-5\"\n"
        ));
    }

    #[test]
    fn the_first_change_writes_the_default_file() {
        let scratch = Scratch::new("registry-first-write");
        let store = store(&scratch);
        let (registry, ()) = store
            .change(TODAY, |changes| changes.set_pinned("wrangler", true))
            .unwrap();
        assert!(registry.projects[0].pinned);
        let text = fs::read_to_string(&store.path).unwrap();
        let root = store.defaults.data_root.display();
        let expected = format!(
            "{HEADER}format = 1\nignore = []\n\n[settings]\ndata_root = '{root}'\n\n\
             [[project]]\nname = \"wrangler\"\npath = \"wrangler\"\nstate = \"active\"\n\
             priority = 3\npinned = true\n"
        );
        assert_eq!(text, expected);
        // Nothing existed to copy.
        assert!(!store.backups.exists());
    }

    #[test]
    fn changes_keep_the_owners_comments_and_layout() {
        let scratch = Scratch::new("registry-layout");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        let (registry, ()) = store
            .change(TODAY, |changes| {
                changes.set_state("ALPHA", State::Paused)?;
                changes.set_pinned("beta", true)?;
                assert!(changes.ignore("junk")?);
                assert!(!changes.ignore("Scratch")?);
                changes.add_project("gamma", "gamma", State::Unformalised)
            })
            .unwrap();
        let expected = HAND_WRITTEN
            .replace(r#"ignore = ["scratch"]"#, r#"ignore = ["scratch", "junk"]"#)
            .replace(r#"state = "active"   #"#, r#"state = "paused"   #"#)
            .replace(
                "state = \"paused\"\n",
                "state = \"paused\"\npinned = true\n\n[[project]]\nname = \"gamma\"\n\
                 path = \"gamma\"\nstate = \"unformalised\"\npriority = 3\npinned = false\n",
            );
        assert_eq!(fs::read_to_string(&store.path).unwrap(), expected);
        assert_eq!(registry.projects.len(), 3);
        assert_eq!(registry.projects[0].launch.model.as_deref(), Some("fable"));
        // The day's first change kept the file as it was.
        let copy = store.backups.join("registry.2026-10-31.toml");
        assert_eq!(fs::read_to_string(copy).unwrap(), HAND_WRITTEN);
    }

    #[test]
    fn an_unchanged_registry_is_not_written() {
        let scratch = Scratch::new("registry-unchanged");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        store
            .change(TODAY, |changes| changes.ignore("scratch").map(drop))
            .unwrap();
        assert!(!store.backups.exists());
    }

    #[test]
    fn strict_reading_names_the_problem_and_writes_nothing() {
        let scratch = Scratch::new("registry-strict");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        let edit = |from: &str, to: &str| HAND_WRITTEN.replacen(from, to, 1);
        let cases = [
            (
                edit("state = \"paused\"\n", "state = \"paused\"\nprioirty = 2\n"),
                "prioirty",
            ),
            (edit(r#""paused""#, r#""resting""#), "resting"),
            (edit("priority = 2", "priority = 7"), "between 1 and 5"),
            (
                edit(r#"name = "beta""#, r#"name = "Alpha""#),
                "registered twice",
            ),
            (
                edit(r#"path = "beta""#, r#"path = "ALPHA/""#),
                "same folder",
            ),
            (edit("format = 1\n", ""), "format = 1"),
            (edit(r#"["scratch"]"#, r#"["a\\b"]"#), "folder names"),
            (
                edit(
                    "priority = 2\n",
                    "priority = 2\nneighbours = ['\\\\server\\share']\n",
                ),
                "on a drive",
            ),
            (
                edit("priority = 2\n", "priority = 2\nneighbours = ['ALPHA']\n"),
                "own folder",
            ),
            (
                edit(
                    "data_root = 'C:\\Data'\n",
                    "data_root = 'C:\\Data'\nclaude_cli = 'claude.exe'\n",
                ),
                "settings.claude_cli must be a full path",
            ),
            (
                edit(
                    "data_root = 'C:\\Data'\n",
                    "data_root = 'C:\\Data'\nvscode = 'Code.exe'\n",
                ),
                "settings.vscode must be a full path",
            ),
        ];
        for (text, message) in cases {
            fs::write(&store.path, &text).unwrap();
            let error = store.load().unwrap_err().to_string();
            assert!(error.contains(message), "{message}: {error}");
            assert!(
                store
                    .change(TODAY, |c| c.set_pinned("alpha", true))
                    .is_err()
            );
            assert_eq!(fs::read_to_string(&store.path).unwrap(), text);
        }
        assert!(!store.backups.exists());
    }

    #[test]
    fn a_newer_format_is_refused_and_left_as_it_is() {
        let scratch = Scratch::new("registry-newer");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        let newer = HAND_WRITTEN.replace("format = 1", "format = 2\nsomething_new = true");
        fs::write(&store.path, &newer).unwrap();
        assert!(matches!(
            store.load(),
            Err(Error::Newer {
                found: 2,
                supported: 1,
                ..
            })
        ));
        assert!(
            store
                .change(TODAY, |c| c.set_pinned("alpha", true))
                .is_err()
        );
        assert_eq!(fs::read_to_string(&store.path).unwrap(), newer);
    }

    /// A pretend format 2 that writes every project's `pinned` explicitly.
    fn pin_everything_explicitly(document: &mut DocumentMut) {
        if let Some(projects) = document
            .get_mut("project")
            .and_then(Item::as_array_of_tables_mut)
        {
            for table in projects.iter_mut() {
                if !table.contains_key("pinned") {
                    table["pinned"] = toml_edit::value(false);
                }
            }
        }
    }

    #[test]
    fn an_older_format_is_kept_beside_the_file_before_its_migration() {
        let scratch = Scratch::new("registry-migration");
        let mut store = store(&scratch);
        store.migrations = Migrations {
            current: 2,
            steps: &[pin_everything_explicitly],
        };
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        // Reading migrates in memory only.
        assert_eq!(store.load().unwrap().registry.format, 2);
        assert_eq!(fs::read_to_string(&store.path).unwrap(), HAND_WRITTEN);

        store
            .change(TODAY, |c| c.set_state("beta", State::Active))
            .unwrap();
        let config = store.path.parent().unwrap();
        let kept = config.join("registry.v1.2026-10-31.toml");
        assert_eq!(fs::read_to_string(kept).unwrap(), HAND_WRITTEN);
        let text = fs::read_to_string(&store.path).unwrap();
        assert!(text.contains("format = 2\n"), "{text}");
        assert_eq!(text.matches("pinned = false").count(), 2, "{text}");
        assert!(text.starts_with("# The owner's notes stay.\n"));
    }

    #[test]
    fn settings_are_set_in_place_and_read_back() {
        let scratch = Scratch::new("registry-settings");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        let (registry, ()) = store
            .change(TODAY, |c| {
                c.set_setting("backup_settings", r"jobs\backup.psd1")?;
                c.set_setting("backup_log", r"D:\logs\backup.jsonl")
            })
            .unwrap();
        let text = fs::read_to_string(&store.path).unwrap();
        assert!(
            text.contains(
                "data_root = 'C:\\Data'\nbackup_settings = 'jobs\\backup.psd1'\n\
                 backup_log = 'D:\\logs\\backup.jsonl'\n"
            ),
            "{text}"
        );
        assert_eq!(
            registry.backup_files(),
            (
                Some(PathBuf::from(r"C:\Data\jobs\backup.psd1")),
                Some(PathBuf::from(r"D:\logs\backup.jsonl"))
            )
        );
        assert_eq!(
            registry.settings.get("backup_log").as_deref(),
            Some(r"D:\logs\backup.jsonl")
        );
        assert!(
            store
                .change(TODAY, |c| c.set_setting("colour", "red"))
                .is_err()
        );
        assert!(
            store
                .change(TODAY, |c| c.set_setting("backup_log", " "))
                .is_err()
        );
        // A relative data root is refused when it reads back, and nothing is written.
        assert!(
            store
                .change(TODAY, |c| c.set_setting("data_root", "rel"))
                .is_err()
        );
        assert_eq!(fs::read_to_string(&store.path).unwrap(), text);
        // A registry without a [settings] table gains one.
        let bare = Scratch::new("registry-settings-bare");
        let store = self::store(&bare);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, "format = 1\n").unwrap();
        let (registry, ()) = store
            .change(TODAY, |c| c.set_setting("backup_settings", "job.psd1"))
            .unwrap();
        assert_eq!(
            registry.settings.backup_settings,
            Some(PathBuf::from("job.psd1"))
        );
    }

    #[test]
    fn the_decisions_route_and_a_projects_time_limit_read_back() {
        let scratch = Scratch::new("registry-decisions");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        let name = store.load().unwrap().registry.projects[0].name.clone();
        assert_eq!(store.load().unwrap().registry.settings.decisions, None);
        let (registry, ()) = store
            .change(TODAY, |c| {
                c.set_setting("decisions", "defer")?;
                c.set_answer_timeout(&name, Some(30))
            })
            .unwrap();
        assert_eq!(registry.settings.decisions, Some(Route::Defer));
        assert_eq!(registry.settings.get("decisions").as_deref(), Some("defer"));
        let project = registry.project(&name).unwrap();
        assert_eq!(project.answer_timeout, Some(30));
        assert_eq!(
            project.answer_timeout(),
            Some(std::time::Duration::from_secs(1800))
        );
        let text = fs::read_to_string(&store.path).unwrap();
        assert!(
            text.contains("decisions = \"defer\"") && text.contains("answer_timeout = 30"),
            "{text}"
        );
        // A word the route does not know, or a limit out of range, writes nothing.
        assert!(
            store
                .change(TODAY, |c| c.set_setting("decisions", "maybe"))
                .is_err()
        );
        assert!(
            store
                .change(TODAY, |c| c.set_answer_timeout(&name, Some(0)))
                .is_err()
        );
        assert_eq!(fs::read_to_string(&store.path).unwrap(), text);
        // Taking the limit away removes the key; a hand-written limit out of range is refused.
        let (registry, ()) = store
            .change(TODAY, |c| c.set_answer_timeout(&name, None))
            .unwrap();
        assert_eq!(registry.project(&name).unwrap().answer_timeout, None);
        assert!(
            !fs::read_to_string(&store.path)
                .unwrap()
                .contains("answer_timeout")
        );
        let wrong = HAND_WRITTEN.replace("priority = 2\n", "priority = 2\nanswer_timeout = 5000\n");
        fs::write(&store.path, wrong).unwrap();
        assert!(store.load().is_err());
    }

    /// The revert's switch (EP-15b): on unless turned off, and on again without a key.
    #[test]
    fn a_projects_revert_is_on_until_turned_off() {
        let scratch = Scratch::new("registry-revert");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        let name = store.load().unwrap().registry.projects[0].name.clone();
        assert!(
            store
                .load()
                .unwrap()
                .registry
                .project(&name)
                .unwrap()
                .revert
        );
        let (registry, ()) = store.change(TODAY, |c| c.set_revert(&name, false)).unwrap();
        assert!(!registry.project(&name).unwrap().revert);
        assert!(
            fs::read_to_string(&store.path)
                .unwrap()
                .contains("revert = false")
        );
        let (registry, ()) = store.change(TODAY, |c| c.set_revert(&name, true)).unwrap();
        assert!(registry.project(&name).unwrap().revert);
        assert!(
            !fs::read_to_string(&store.path)
                .unwrap()
                .contains("revert =")
        );
    }

    #[test]
    fn quiet_hours_are_set_read_back_and_taken_away() {
        let scratch = Scratch::new("registry-quiet");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        assert_eq!(store.load().unwrap().registry.settings.quiet_hours, None);
        let (registry, ()) = store
            .change(TODAY, |c| c.set_setting("quiet_hours", " 22:00 - 7:30 "))
            .unwrap();
        assert_eq!(
            registry.settings.get("quiet_hours").as_deref(),
            Some("22:00-07:30")
        );
        let text = fs::read_to_string(&store.path).unwrap();
        assert!(text.contains("quiet_hours = \"22:00-07:30\""), "{text}");
        // Two times it cannot read, or the same time twice, write nothing.
        for wrong in ["late", "22:00-22:00"] {
            assert!(
                store
                    .change(TODAY, |c| c.set_setting("quiet_hours", wrong))
                    .is_err(),
                "{wrong}"
            );
        }
        assert_eq!(fs::read_to_string(&store.path).unwrap(), text);
        // None takes the key out, and taking it out twice is no error.
        for _ in 0..2 {
            let (registry, ()) = store
                .change(TODAY, |c| c.set_setting("quiet_hours", "none"))
                .unwrap();
            assert_eq!(registry.settings.quiet_hours, None);
        }
        assert!(
            !fs::read_to_string(&store.path)
                .unwrap()
                .contains("quiet_hours")
        );
        // A hand-written value it cannot read is refused with the file.
        let wrong = text.replace("22:00-07:30", "evenings");
        fs::write(&store.path, wrong).unwrap();
        assert!(store.load().is_err());
    }

    #[test]
    fn the_usage_endpoint_switch_is_written_only_when_off() {
        let scratch = Scratch::new("registry-usage");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        assert!(store.load().unwrap().registry.settings.usage_endpoint_on());
        let (registry, ()) = store
            .change(TODAY, |c| c.set_setting("usage_endpoint", "off"))
            .unwrap();
        assert!(!registry.settings.usage_endpoint_on());
        assert_eq!(
            registry.settings.get("usage_endpoint").as_deref(),
            Some("off")
        );
        let text = fs::read_to_string(&store.path).unwrap();
        assert!(text.contains("usage_endpoint = false"), "{text}");
        assert!(
            store
                .change(TODAY, |c| c.set_setting("usage_endpoint", "maybe"))
                .is_err()
        );
        let (registry, ()) = store
            .change(TODAY, |c| c.set_setting("usage_endpoint", "on"))
            .unwrap();
        assert!(registry.settings.usage_endpoint_on());
        assert_eq!(registry.settings.usage_endpoint, None);
        assert!(
            !fs::read_to_string(&store.path)
                .unwrap()
                .contains("usage_endpoint")
        );
    }

    #[test]
    fn the_downshift_percent_is_a_whole_percent_and_default_takes_it_out() {
        let scratch = Scratch::new("registry-downshift");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        let settings = store.load().unwrap().registry.settings;
        assert_eq!(
            settings.fable_downshift_percent(),
            crate::rules::DEFAULT_DOWNSHIFT_PERCENT
        );
        let (registry, ()) = store
            .change(TODAY, |c| {
                c.set_setting("fable_downshift_percent", " 75 % ")
            })
            .unwrap();
        assert_eq!(registry.settings.fable_downshift_percent(), 75);
        assert_eq!(
            registry.settings.get("fable_downshift_percent").as_deref(),
            Some("75")
        );
        let text = fs::read_to_string(&store.path).unwrap();
        assert!(text.contains("fable_downshift_percent = 75"), "{text}");
        for wrong in ["0", "101", "60.5", "high", ""] {
            assert!(
                store
                    .change(TODAY, |c| c.set_setting("fable_downshift_percent", wrong))
                    .is_err(),
                "{wrong:?}"
            );
        }
        let (registry, ()) = store
            .change(TODAY, |c| {
                c.set_setting("fable_downshift_percent", "default")
            })
            .unwrap();
        assert_eq!(registry.settings.fable_downshift_percent, None);
        assert!(
            !fs::read_to_string(&store.path)
                .unwrap()
                .contains("fable_downshift_percent")
        );
        // A hand-written percent out of range is refused with the file.
        fs::write(
            &store.path,
            HAND_WRITTEN.replacen(
                "[settings]\n",
                "[settings]\nfable_downshift_percent = 0\n",
                1,
            ),
        )
        .unwrap();
        let error = store.load().unwrap_err().to_string();
        assert!(error.contains("from 1 to 100"), "{error}");
    }

    #[test]
    fn neighbours_and_the_cli_read_back() {
        let scratch = Scratch::new("registry-neighbours");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        let text = HAND_WRITTEN
            .replace(
                "priority = 2\n",
                "priority = 2\nneighbours = [\"beta\", 'D:\\Shared\\gamma', \"beta/../delta\"]\n",
            )
            .replace(
                "data_root = 'C:\\Data'\n",
                "data_root = 'C:\\Data'\nclaude_cli = 'C:\\Tools\\claude.exe'\n",
            );
        fs::write(&store.path, &text).unwrap();
        let registry = store.load().unwrap().registry;
        assert_eq!(
            registry
                .project("alpha")
                .unwrap()
                .neighbour_folders(&registry.data_root()),
            [
                PathBuf::from(r"C:\Data\beta"),
                PathBuf::from(r"D:\Shared\gamma"),
                PathBuf::from(r"C:\Data\delta")
            ]
        );
        assert!(registry.project("beta").unwrap().neighbours.is_empty());
        assert_eq!(
            registry.settings.get("claude_cli").as_deref(),
            Some(r"C:\Tools\claude.exe")
        );
        // `crw setting` sets it like the other paths.
        let (registry, ()) = store
            .change(TODAY, |c| c.set_setting("claude_cli", r"E:\cli\claude.exe"))
            .unwrap();
        assert_eq!(
            registry.settings.claude_cli,
            Some(PathBuf::from(r"E:\cli\claude.exe"))
        );
        // The VS Code a visual launch opens, likewise (EP-11).
        let (registry, ()) = store
            .change(TODAY, |c| {
                c.set_setting("vscode", r"E:\VS Code\Code - Insiders.exe")
            })
            .unwrap();
        assert_eq!(
            registry.settings.get("vscode").as_deref(),
            Some(r"E:\VS Code\Code - Insiders.exe")
        );
        // The signals' port is a number (EP-12): the default until it is set, 0 refused.
        assert_eq!(registry.port(), crate::signals::DEFAULT_PORT);
        let (registry, ()) = store
            .change(TODAY, |c| c.set_setting("port", "47900"))
            .unwrap();
        assert_eq!(
            (registry.port(), registry.settings.get("port").as_deref()),
            (47900, Some("47900"))
        );
        assert!(
            fs::read_to_string(&store.path)
                .unwrap()
                .contains("port = 47900\n")
        );
        for bad in ["0", "70000", "port"] {
            assert!(
                store.change(TODAY, |c| c.set_setting("port", bad)).is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn rules_and_templates_read_back_and_bad_ones_are_named() {
        let scratch = Scratch::new("registry-rules");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        let text = HAND_WRITTEN
            .replace(
                "ignore = [\"scratch\"]  # never a project\n",
                "ignore = [\"scratch\"]  # never a project\n\n[[rule]]\nname = \"small\"\n\
                 match = { size = \"S\" }\nset = { effort_step = -1 }\n",
            )
            .replace(
                "[project.launch]\nmodel = \"fable\"\n",
                "[project.launch]\nmodel = \"fable\"\ntemplate = \"\"\"\nRun {ep} of {project}.\n\
                 {opener}\"\"\"\n\n[[project.rule]]\nmatch = { title = [\"re-plan\", \"retro\"] }\n\
                 set = { model = \"claude-opus-5-5\", ultracode = true }\n",
            );
        fs::write(&store.path, &text).unwrap();
        let registry = store.load().unwrap().registry;
        assert_eq!(registry.rules.len(), 1);
        assert_eq!(registry.rules[0].set.effort_step, Some(-1));
        let alpha = registry.project("alpha").unwrap();
        assert_eq!(
            alpha.launch.template.as_deref(),
            Some("Run {ep} of {project}.\n{opener}")
        );
        assert_eq!(alpha.rules.len(), 1);
        assert_eq!(alpha.rules[0].when.title.0, ["re-plan", "retro"]);
        assert!(registry.project("beta").unwrap().rules.is_empty());
        // A change elsewhere keeps the rules as written.
        store.change(TODAY, |c| c.set_pinned("beta", true)).unwrap();
        assert!(
            fs::read_to_string(&store.path)
                .unwrap()
                .contains("[[project.rule]]\nmatch = { title = [\"re-plan\", \"retro\"] }\n")
        );

        for (from, to, message) in [
            (
                "effort_step = -1",
                "effort_step = 0",
                "rule 1: its effort_step 0",
            ),
            (
                "size = \"S\"",
                "size = \"XL\"",
                "rule 1: its size is S, M or L",
            ),
            ("ultracode = true", "ultracode = \"yes\"", "ultracode"),
            (
                "model = \"fable\"\n",
                "model = \"\"\n",
                "alpha: launch.model is empty",
            ),
            (
                "set = { model = \"claude-opus-5-5\", ultracode = true }",
                "set = {}",
                "alpha: rule 1: it sets nothing",
            ),
        ] {
            fs::write(&store.path, text.replacen(from, to, 1)).unwrap();
            let error = store.load().unwrap_err().to_string();
            assert!(error.contains(message), "{message}: {error}");
        }
    }

    #[test]
    fn set_checks_the_project_and_the_value() {
        let scratch = Scratch::new("registry-set");
        let store = store(&scratch);
        fs::create_dir_all(store.path.parent().unwrap()).unwrap();
        fs::write(&store.path, HAND_WRITTEN).unwrap();
        assert!(matches!(
            store.change(TODAY, |c| c.set_state("delta", State::Done)),
            Err(Error::UnknownProject(name)) if name == "delta"
        ));
        assert!(store.change(TODAY, |c| c.set_priority("alpha", 0)).is_err());
        assert!(
            store
                .change(TODAY, |c| c.add_project("Beta", "elsewhere", State::Active))
                .is_err()
        );
        let (registry, ()) = store.change(TODAY, |c| c.set_priority("beta", 1)).unwrap();
        assert_eq!(registry.project("BETA").unwrap().priority, 1);
        assert_eq!("maintenance".parse::<State>().unwrap(), State::Maintenance);
        assert!("unadopted".parse::<State>().is_err());
    }
}
