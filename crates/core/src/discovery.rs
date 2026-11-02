//! Discovery (D-6, D-7): the top-level folders of the data root, what each one is, and how they
//! stand against the registry; and the two actions on a folder the registry does not list, Adopt
//! and Ignore. It looks at names and at whether a few paths exist, never at what a project's files
//! contain. Adopt writes the folder's own workspace file beside it, appends it to the aggregate
//! workspace file and registers it (the D-7 addendum, EP-33).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;

use crate::proposal::{self, Facts};
use crate::registry::{Error, Project, Registry, State, Store};
use crate::roadmap::snapshot;
use crate::{files, paths, process, workspace};

/// A folder and what it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub name: String,
    pub path: PathBuf,
    /// A `.git` folder, or a `.git` file (a worktree or a submodule).
    pub git: bool,
    /// The house-style roadmap, `roadmap/README.md`, with something in it; whether it parses is
    /// the parser's question (`roadmap::table`).
    pub roadmap: bool,
    /// Its own workspace file, `<name>.code-workspace` beside it (D-7 addendum).
    pub own_workspace: bool,
}

impl Folder {
    /// What is at `path`, or `None` when it is not a folder.
    pub fn inspect(path: &Path) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }
        let readme = crate::roadmap::readme(path);
        Some(Self {
            name: path.file_name()?.to_string_lossy().into_owned(),
            path: path.to_owned(),
            git: path.join(".git").exists(),
            roadmap: fs::metadata(readme).is_ok_and(|m| m.is_file() && m.len() > 0),
            own_workspace: workspace::own_file(path).is_some_and(|file| file.is_file()),
        })
    }

    /// The state an adopted folder starts in: active for a repository with a roadmap,
    /// unformalised for anything else (D-6).
    pub fn first_state(&self) -> State {
        if self.git && self.roadmap {
            State::Active
        } else {
            State::Unformalised
        }
    }
}

/// The top-level folders of `root`, by name; files and dot folders are left out.
pub fn folders(root: &Path) -> io::Result<Vec<Folder>> {
    let mut found: Vec<Folder> = fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        .filter_map(|entry| Folder::inspect(&entry.path()))
        .collect();
    found.sort_by_key(|folder| folder.name.to_lowercase());
    Ok(found)
}

/// The registry set against the data root.
#[derive(Debug, Clone)]
pub struct View {
    pub data_root: PathBuf,
    /// The workspace file and its entries, when the registry names one.
    pub workspace: Option<WorkspaceFile>,
    pub registered: Vec<Registered>,
    /// Folders of the data root that are neither registered nor ignored.
    pub unadopted: Vec<Folder>,
    pub ignored: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceFile {
    pub path: PathBuf,
    /// Its entries, or why it could not be read.
    pub entries: Result<Vec<workspace::Entry>, String>,
}

#[derive(Debug, Clone)]
pub struct Registered {
    pub project: Project,
    pub path: PathBuf,
    /// `None` when the folder is missing.
    pub folder: Option<Folder>,
}

impl View {
    /// Whether the workspace file lists `folder`; `None` without a readable workspace file.
    pub fn in_workspace(&self, folder: &Path) -> Option<bool> {
        let file = self.workspace.as_ref()?;
        let dir = file.path.parent()?;
        Some(workspace::lists(file.entries.as_ref().ok()?, dir, folder))
    }

    /// The facts proposals are made from, one per registered project ([`facts_of`]), the
    /// repositories asked of git one child at a time (D-29).
    pub fn facts(&self) -> Vec<Facts> {
        let mut asked = false;
        self.registered
            .iter()
            .map(|registered| {
                let Some(folder) = &registered.folder else {
                    return Facts::default();
                };
                if folder.git {
                    if asked {
                        thread::sleep(process::SPACING);
                    }
                    asked = true;
                }
                facts_of(folder)
            })
            .collect()
    }
}

/// The facts one folder's proposals are made from: the last commit of a repository, asked of
/// git, and the roadmap facts (every row checked, a pickup gate on the next brief) read from the
/// working tree without a child process (`roadmap::snapshot::read_tree`).
pub fn facts_of(folder: &Folder) -> Facts {
    let mut facts = Facts::default();
    if folder.git {
        facts.last_commit = proposal::last_commit(&folder.path);
    }
    if folder.roadmap {
        snapshot::read_tree(&folder.path).fill(&mut facts);
    }
    facts
}

/// Reads the data root and sets the registry against it; writes nothing.
pub fn view(registry: &Registry) -> Result<View, Error> {
    let data_root = registry.data_root();
    let listed = folders(&data_root).map_err(|e| Error::io(&data_root, e))?;
    let registered: Vec<Registered> = registry
        .projects
        .iter()
        .map(|project| {
            let path = project.folder(&data_root);
            Registered {
                folder: Folder::inspect(&path),
                project: project.clone(),
                path,
            }
        })
        .collect();
    let unadopted = listed
        .into_iter()
        .filter(|folder| {
            !registered
                .iter()
                .any(|r| paths::same(&r.path, &folder.path))
                && !registry
                    .ignore
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&folder.name))
        })
        .collect();
    let workspace = registry.workspace_file().map(|path| WorkspaceFile {
        entries: fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|text| workspace::entries(&text)),
        path,
    });
    Ok(View {
        data_root,
        workspace,
        registered,
        unadopted,
        ignored: registry.ignore.clone(),
    })
}

/// What adopting one folder did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adopted {
    pub name: String,
    pub state: State,
    /// Its own workspace file.
    pub own_file: OwnFile,
    /// The aggregate workspace file.
    pub workspace: Listing,
}

/// What Adopt did with the workspace file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Listing {
    /// Appended to it, after a copy of the file as it was.
    Appended {
        backup: PathBuf,
    },
    AlreadyListed,
    /// The registry names no workspace file.
    NoWorkspaceFile,
}

/// What became of a folder's own workspace file (D-7 addendum).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnFile {
    /// Written beside the folder.
    Written(PathBuf),
    /// One was there already, and is left as it was.
    Kept(PathBuf),
    /// Not asked for ([`OwnFiles::Skip`]).
    Skipped,
}

/// Whether Adopt writes each folder's own workspace file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnFiles {
    Write,
    /// A screenshot run's: its data root is the tracked fixtures, which it never writes into.
    Skip,
}

/// Writes `folder`'s own workspace file in the owner's form (`workspace::single_folder`) unless
/// a file of that name is there, which is never overwritten (D-7 addendum). EP-11's Visual
/// launch writes a missing one through it.
pub fn write_own_file(folder: &Path) -> Result<OwnFile, Error> {
    let (Some(name), Some(path)) = (folder.file_name(), workspace::own_file(folder)) else {
        return Err(Error::Invalid(format!(
            "{} has no name to give a workspace file",
            folder.display()
        )));
    };
    let text = workspace::single_folder(&name.to_string_lossy());
    match files::create_new(&path, text.as_bytes()) {
        Ok(()) => Ok(OwnFile::Written(path)),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(OwnFile::Kept(path)),
        Err(e) => Err(Error::io(&path, e)),
    }
}

/// Adopts folders of the data root by name (D-7): writes each one's own workspace file beside it
/// when none is there (unless `own_files` says skip), appends the ones the aggregate workspace
/// file does not list yet, after a dated copy of it in the backups folder, then registers them;
/// an ignored folder leaves the ignore list. The aggregate's new text is worked out before
/// anything is written, so a file Adopt cannot read stops it at once; the files go before the
/// registry, so a failure in between leaves a folder with its files but not registered, and
/// adopting it again finishes the job.
pub fn adopt(
    store: &Store,
    names: &[&str],
    today: &str,
    own_files: OwnFiles,
) -> Result<Vec<Adopted>, Error> {
    let registry = store.load()?.registry;
    let picked = pick(&registry, names)?;
    if let Some(folder) = picked.iter().find(|folder| {
        registry
            .projects
            .iter()
            .any(|p| paths::same(&p.folder(&registry.data_root()), &folder.path))
    }) {
        return Err(Error::Invalid(format!(
            "{} is registered already",
            folder.name
        )));
    }
    let appending = Appending::plan(&registry, &picked)?;
    let own: Vec<OwnFile> = picked
        .iter()
        .map(|folder| match own_files {
            OwnFiles::Write => write_own_file(&folder.path),
            OwnFiles::Skip => Ok(OwnFile::Skipped),
        })
        .collect::<Result<_, _>>()?;
    let listings = match appending {
        Some(appending) => appending.write(&store.backups, today)?,
        None => vec![Listing::NoWorkspaceFile; picked.len()],
    };
    store.change(today, |changes| {
        for folder in &picked {
            changes.unignore(&folder.name);
            changes.add_project(&folder.name, &folder.name, folder.first_state())?;
        }
        Ok(())
    })?;
    Ok(picked
        .into_iter()
        .zip(own)
        .zip(listings)
        .map(|((folder, own_file), workspace)| Adopted {
            state: folder.first_state(),
            name: folder.name,
            own_file,
            workspace,
        })
        .collect())
}

/// Remembers folders of the data root as ignored (D-7) and returns the names newly ignored; a
/// registered project cannot be ignored (its state can be archived instead).
pub fn ignore(store: &Store, names: &[&str], today: &str) -> Result<Vec<String>, Error> {
    let registry = store.load()?.registry;
    let picked = pick(&registry, names)?;
    let data_root = registry.data_root();
    if let Some(project) = registry.projects.iter().find(|p| {
        picked
            .iter()
            .any(|folder| paths::same(&p.folder(&data_root), &folder.path))
    }) {
        return Err(Error::Invalid(format!(
            "{} is registered: set its state to archived instead",
            project.name
        )));
    }
    let (_, ignored) = store.change(today, |changes| {
        let mut ignored = Vec::new();
        for folder in &picked {
            if changes.ignore(&folder.name)? {
                ignored.push(folder.name.clone());
            }
        }
        Ok(ignored)
    })?;
    Ok(ignored)
}

/// The top-level folders of the data root called `names`, spelled as on disk, each once.
fn pick(registry: &Registry, names: &[&str]) -> Result<Vec<Folder>, Error> {
    let data_root = registry.data_root();
    let listed = folders(&data_root).map_err(|e| Error::io(&data_root, e))?;
    let mut picked: Vec<Folder> = Vec::new();
    for name in names {
        let folder = listed
            .iter()
            .find(|folder| folder.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| {
                Error::Invalid(format!("{name} is not a folder of {}", data_root.display()))
            })?;
        if !picked.contains(folder) {
            picked.push(folder.clone());
        }
    }
    Ok(picked)
}

/// The aggregate workspace file with the folders it does not list yet appended, worked out in
/// memory before Adopt writes anything.
struct Appending {
    path: PathBuf,
    original: String,
    text: String,
    /// Per folder, whether it was appended.
    new: Vec<bool>,
}

impl Appending {
    /// `None` when the registry names no workspace file.
    fn plan(registry: &Registry, picked: &[Folder]) -> Result<Option<Self>, Error> {
        let Some(path) = registry.workspace_file() else {
            return Ok(None);
        };
        let unreadable = |e: String| Error::Invalid(format!("{}: {e}", path.display()));
        let original = fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
        let dir = path.parent().unwrap_or(Path::new("."));
        let listed = workspace::entries(&original).map_err(unreadable)?;
        let mut text = original.clone();
        let mut new = Vec::new();
        for folder in picked {
            let missing = !workspace::lists(&listed, dir, &folder.path);
            if missing {
                text = workspace::append(&text, &workspace::entry_path(dir, &folder.path))
                    .map_err(unreadable)?;
            }
            new.push(missing);
        }
        Ok(Some(Self {
            path,
            original,
            text,
            new,
        }))
    }

    /// Writes the file, in one write after one copy, when anything was appended.
    fn write(self, backups: &Path, today: &str) -> Result<Vec<Listing>, Error> {
        if self.text == self.original {
            return Ok(vec![Listing::AlreadyListed; self.new.len()]);
        }
        let name = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let (stem, ext) = name.split_at(name.find('.').unwrap_or(name.len()));
        let backup = files::dated_copy(self.original.as_bytes(), backups, stem, today, ext)
            .map_err(|e| Error::io(backups, e))?;
        files::write_atomic(&self.path, self.text.as_bytes())
            .map_err(|e| Error::io(&self.path, e))?;
        Ok(self
            .new
            .into_iter()
            .map(|new| match new {
                true => Listing::Appended {
                    backup: backup.clone(),
                },
                false => Listing::AlreadyListed,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;
    use crate::registry::Defaults;

    const TODAY: &str = "2026-10-31";

    /// The layout of the owner's workspace file, with synthetic names.
    const WORKSPACE: &str = "{\n\t\"folders\": [\n\t\t{\n\t\t\t\"name\": \"alpha\",\n\t\t\t\"path\": \"alpha\"\n\t\t},\n\t\t{\n\t\t\t\"path\": \"wrangler\"\n\t\t}\n\t]\n}";

    /// A data root with the tool's repository, a repository with a roadmap, one whose roadmap is
    /// empty, a plain folder, a worktree, a file and a dot folder; the registry not written yet.
    fn data_root(scratch: &Scratch) -> Store {
        let data = scratch.0.join("data");
        for dir in [
            "wrangler/.git",
            "wrangler/roadmap",
            "alpha/.git",
            "alpha/roadmap",
            "beta/.git",
            "beta/roadmap",
            "notes",
            "worktree",
            ".hidden",
        ] {
            fs::create_dir_all(data.join(dir)).unwrap();
        }
        fs::write(data.join("wrangler/roadmap/README.md"), "# roadmap").unwrap();
        fs::write(data.join("alpha/roadmap/README.md"), "# roadmap").unwrap();
        fs::write(data.join("beta/roadmap/README.md"), "").unwrap();
        fs::write(data.join("worktree/.git"), "gitdir: elsewhere").unwrap();
        fs::write(data.join("main.code-workspace"), WORKSPACE).unwrap();
        Store::new(
            scratch.0.join("config").join("registry.toml"),
            scratch.0.join("local").join("backups"),
            Defaults {
                repository: data.join("wrangler"),
                data_root: data,
            },
        )
    }

    #[test]
    fn lists_and_classifies_the_top_level_folders() {
        let scratch = Scratch::new("discovery-folders");
        let store = data_root(&scratch);
        let data = &store.defaults.data_root;
        fs::write(data.join("wrangler.code-workspace"), "{}").unwrap();
        fs::create_dir(data.join("notes.code-workspace")).unwrap();
        let found = folders(data).unwrap();
        let summary: Vec<_> = found
            .iter()
            .map(|f| (f.name.as_str(), f.git, f.roadmap, f.own_workspace))
            .collect();
        assert_eq!(
            summary,
            [
                ("alpha", true, true, false),
                ("beta", true, false, false),
                ("notes", false, false, false),
                ("notes.code-workspace", false, false, false),
                ("worktree", true, false, false),
                ("wrangler", true, true, true),
            ]
        );
        let states: Vec<_> = found.iter().map(Folder::first_state).collect();
        assert_eq!(
            states,
            [
                State::Active,
                State::Unformalised,
                State::Unformalised,
                State::Unformalised,
                State::Unformalised,
                State::Active
            ]
        );
    }

    #[test]
    fn sets_the_registry_against_the_data_root() {
        let scratch = Scratch::new("discovery-view");
        let store = data_root(&scratch);
        let view = view(&store.load().unwrap().registry).unwrap();
        let registered: Vec<_> = view
            .registered
            .iter()
            .map(|r| (r.project.name.as_str(), r.folder.is_some()))
            .collect();
        assert_eq!(registered, [("wrangler", true)]);
        let unadopted: Vec<_> = view.unadopted.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(unadopted, ["alpha", "beta", "notes", "worktree"]);
        let data = &store.defaults.data_root;
        assert_eq!(view.in_workspace(&data.join("Alpha")), Some(true));
        assert_eq!(view.in_workspace(&data.join("beta")), Some(false));

        ignore(&store, &["notes", "NOTES"], TODAY).unwrap();
        fs::remove_dir_all(data.join("wrangler")).unwrap();
        let view = super::view(&store.load().unwrap().registry).unwrap();
        assert!(
            view.registered[0].folder.is_none(),
            "the repository is missing"
        );
        assert_eq!(view.ignored, ["notes"]);
        let unadopted: Vec<_> = view.unadopted.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(unadopted, ["alpha", "beta", "worktree"]);
    }

    #[test]
    fn adopt_registers_and_appends_to_the_workspace_file_once() {
        let scratch = Scratch::new("discovery-adopt");
        let store = data_root(&scratch);
        let data = &store.defaults.data_root;
        let workspace_file = data.join("main.code-workspace");
        // The owner's own file for alpha, in a layout of their own: kept byte for byte.
        let alpha_file = data.join("alpha.code-workspace");
        let theirs = "{ \"folders\": [ { \"path\": \"alpha\" } ] } // mine\r\n";
        fs::write(&alpha_file, theirs).unwrap();
        ignore(&store, &["beta"], TODAY).unwrap();

        let adopted = adopt(&store, &["ALPHA", "beta"], TODAY, OwnFiles::Write).unwrap();
        let backup = store.backups.join("main.2026-10-31.code-workspace");
        let beta_file = data.join("beta.code-workspace");
        assert_eq!(
            adopted,
            [
                Adopted {
                    name: "alpha".to_owned(),
                    state: State::Active,
                    own_file: OwnFile::Kept(alpha_file.clone()),
                    workspace: Listing::AlreadyListed,
                },
                Adopted {
                    name: "beta".to_owned(),
                    state: State::Unformalised,
                    own_file: OwnFile::Written(beta_file.clone()),
                    workspace: Listing::Appended {
                        backup: backup.clone()
                    },
                },
            ]
        );
        assert_eq!(fs::read_to_string(&alpha_file).unwrap(), theirs);
        assert_eq!(
            fs::read_to_string(&beta_file).unwrap(),
            workspace::single_folder("beta")
        );
        assert_eq!(fs::read_to_string(&backup).unwrap(), WORKSPACE);
        assert_eq!(
            fs::read_to_string(&workspace_file).unwrap(),
            WORKSPACE.replace(
                "\"wrangler\"\n\t\t}\n",
                "\"wrangler\"\n\t\t},\n\t\t{\n\t\t\t\"path\": \"beta\"\n\t\t}\n"
            )
        );
        let registry = store.load().unwrap().registry;
        let names: Vec<_> = registry.projects.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["wrangler", "alpha", "beta"]);
        assert!(registry.ignore.is_empty(), "beta left the ignore list");

        // Nothing is adopted twice, ignored once registered, or taken from outside the root.
        let write = OwnFiles::Write;
        assert!(adopt(&store, &["beta"], TODAY, write).is_err());
        assert!(ignore(&store, &["alpha"], TODAY).is_err());
        assert!(adopt(&store, &["missing"], TODAY, write).is_err());
        assert!(adopt(&store, &[".hidden"], TODAY, write).is_err());
    }

    #[test]
    fn a_screenshot_run_writes_no_file_beside_the_folder() {
        let scratch = Scratch::new("discovery-adopt-skip");
        let store = data_root(&scratch);
        let adopted = adopt(&store, &["notes"], TODAY, OwnFiles::Skip).unwrap();
        assert_eq!(adopted[0].own_file, OwnFile::Skipped);
        assert!(
            !store
                .defaults
                .data_root
                .join("notes.code-workspace")
                .exists()
        );
        assert!(store.load().unwrap().registry.project("notes").is_some());
    }

    #[test]
    fn an_own_file_needs_a_folder_with_a_name() {
        assert!(write_own_file(Path::new(r"C:\")).is_err());
    }

    /// Runs git on folders that are not repositories, never the `claude` CLI (D-29).
    #[test]
    fn facts_come_from_git_and_the_parsers() {
        let scratch = Scratch::new("discovery-facts");
        let store = data_root(&scratch);
        let data = &store.defaults.data_root;
        const HEADER: &str = "## Phase P1 — Build\n\n| # | Brief | Size | Depends on | Core | Done |\n|---|---|---|---|---|---|\n";
        fs::write(
            data.join("alpha/roadmap/README.md"),
            format!("{HEADER}| EP-1 | One | S | — | core | ☑ `0123abc` |\n"),
        )
        .unwrap();
        fs::write(
            data.join("beta/roadmap/README.md"),
            format!("{HEADER}| EP-1 | One | S | — | core | ☐ |\n"),
        )
        .unwrap();
        fs::write(
            data.join("beta/roadmap/EP-1-one.md"),
            "# EP-1 — One\n\n> **PICKUP GATE — owner input required (EP-0, 2026-10-01).** Two items.\n",
        )
        .unwrap();
        adopt(&store, &["alpha", "beta"], TODAY, OwnFiles::Skip).unwrap();
        let view = view(&store.load().unwrap().registry).unwrap();
        let facts = view.facts();
        let summary: Vec<_> = view
            .registered
            .iter()
            .zip(&facts)
            .map(|(r, f)| {
                (
                    r.project.name.as_str(),
                    f.all_rows_checked,
                    f.rows,
                    f.pickup_gate,
                    f.next.map(|id| id.to_string()),
                )
            })
            .collect();
        let one = Some("EP-1".to_owned());
        assert_eq!(
            summary,
            [
                ("wrangler", Some(false), 0, Some(false), None),
                ("alpha", Some(true), 1, Some(false), None),
                ("beta", Some(false), 1, Some(true), one),
            ]
        );
        assert!(facts.iter().all(|f| f.last_commit.is_none()));
        assert_eq!(
            facts_of(view.registered[2].folder.as_ref().unwrap()),
            facts[2]
        );
    }

    #[test]
    fn adopt_without_a_workspace_file_writes_the_own_file_and_registers() {
        let scratch = Scratch::new("discovery-no-workspace");
        let store = data_root(&scratch);
        let data = &store.defaults.data_root;
        fs::remove_file(data.join("main.code-workspace")).unwrap();
        let adopted = adopt(&store, &["notes"], TODAY, OwnFiles::Write).unwrap();
        assert_eq!(adopted[0].workspace, Listing::NoWorkspaceFile);
        assert_eq!(
            adopted[0].own_file,
            OwnFile::Written(data.join("notes.code-workspace"))
        );
        assert!(store.load().unwrap().registry.project("notes").is_some());
    }

    #[test]
    fn a_workspace_file_adopt_cannot_read_stops_it_before_any_write() {
        let scratch = Scratch::new("discovery-bad-workspace");
        let store = data_root(&scratch);
        // Named in the registry: a file that does not read is never a fresh registry's choice.
        store
            .change(TODAY, |c| {
                c.set_setting("workspace_file", "main.code-workspace")
            })
            .unwrap();
        let registry = fs::read_to_string(&store.path).unwrap();
        let file = store.defaults.data_root.join("main.code-workspace");
        fs::write(&file, "{ \"folders\": [ ").unwrap();
        assert!(adopt(&store, &["beta"], TODAY, OwnFiles::Write).is_err());
        assert_eq!(
            fs::read_to_string(&store.path).unwrap(),
            registry,
            "the registry was not written"
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), "{ \"folders\": [ ");
        assert!(
            !store
                .defaults
                .data_root
                .join("beta.code-workspace")
                .exists(),
            "no own file either"
        );
    }
}
