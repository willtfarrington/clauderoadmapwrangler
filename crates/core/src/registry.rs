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
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value};

use crate::{files, paths};

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
}

impl Settings {
    /// The keys `crw setting` reads and sets.
    pub const KEYS: [&str; 5] = [
        "data_root",
        "workspace_file",
        "backup_settings",
        "backup_log",
        "claude_cli",
    ];

    /// A setting's value as written, by its key.
    pub fn get(&self, key: &str) -> Option<&Path> {
        match key {
            "data_root" => self.data_root.as_deref(),
            "workspace_file" => self.workspace_file.as_deref(),
            "backup_settings" => self.backup_settings.as_deref(),
            "backup_log" => self.backup_log.as_deref(),
            "claude_cli" => self.claude_cli.as_deref(),
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
}

fn default_priority() -> u8 {
    DEFAULT_PRIORITY
}

/// The project's launch defaults; the launch dialog resolves them with the rule table (EP-10).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Launch {
    pub mode: Option<LaunchMode>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub permission_mode: Option<String>,
}

/// How a session is started (D-16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LaunchMode {
    Visual,
    Headless,
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
            projects: Vec::new(),
        }
    }

    pub fn data_root(&self) -> PathBuf {
        self.settings
            .data_root
            .clone()
            .unwrap_or_else(paths::default_data_root)
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
    fn validate(&self) -> Result<(), Error> {
        let invalid = |message: String| Err(Error::Invalid(message));
        for (key, path) in [
            ("data_root", &self.settings.data_root),
            ("claude_cli", &self.settings.claude_cli),
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
    /// The project's folder: its path resolved against the data root.
    pub fn folder(&self, data_root: &Path) -> PathBuf {
        data_root.join(&self.path)
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

/// The text of a registry that does not exist yet: the header, the data root, the workspace file
/// when the data root holds exactly one, and the repository as the first project (D-5).
fn default_text(defaults: &Defaults) -> String {
    let mut document: DocumentMut = format!("{HEADER}format = {FORMAT}\nignore = []\n")
        .parse()
        .expect("the header is valid TOML");
    let mut settings = Table::new();
    settings.decor_mut().set_prefix("\n");
    settings["data_root"] = toml_edit::value(defaults.data_root.display().to_string());
    if let Some(file) = only_workspace_file(&defaults.data_root) {
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

/// The name of the one `.code-workspace` file in `dir`, if there is exactly one.
fn only_workspace_file(dir: &Path) -> Option<String> {
    let mut found = fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.to_lowercase().ends_with(".code-workspace"));
    let first = found.next()?;
    found.next().is_none().then_some(first)
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

    /// Sets one of [`Settings::KEYS`] in the `[settings]` table, keeping its comment.
    pub fn set_setting(&mut self, key: &str, value: &str) -> Result<(), Error> {
        if !Settings::KEYS.contains(&key) {
            return Err(Error::Invalid(format!(
                "unknown setting {key:?}: {}",
                Settings::KEYS.join(", ")
            )));
        }
        if value.trim().is_empty() {
            return Err(Error::Invalid(format!("{key} needs a path")));
        }
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
        set_keeping_decor(settings, key, Value::from(value));
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

    fn set(&mut self, name: &str, key: &str, value: Value) -> Result<(), Error> {
        if self.registry.project(name).is_none() {
            return Err(Error::UnknownProject(name.to_owned()));
        }
        let table = self
            .document
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
            .ok_or_else(written_as_tables)?;
        set_keeping_decor(table, key, value);
        Ok(())
    }
}

fn written_as_tables() -> Error {
    Error::Invalid("the tool changes projects written as [[project]] tables only".to_owned())
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
        fs::create_dir_all(&store.defaults.data_root).unwrap();
        fs::write(store.defaults.data_root.join("main.code-workspace"), "{}").unwrap();
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
            registry.settings.get("backup_log"),
            Some(Path::new(r"D:\logs\backup.jsonl"))
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
            registry.settings.get("claude_cli"),
            Some(Path::new(r"C:\Tools\claude.exe"))
        );
        // `crw setting` sets it like the other paths.
        let (registry, ()) = store
            .change(TODAY, |c| c.set_setting("claude_cli", r"E:\cli\claude.exe"))
            .unwrap();
        assert_eq!(
            registry.settings.claude_cli,
            Some(PathBuf::from(r"E:\cli\claude.exe"))
        );
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
