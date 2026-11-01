//! Discovery (D-6, D-7): the top-level folders of the data root, what each one is, and how they
//! stand against the registry; and the two actions on a folder the registry does not list, Adopt
//! and Ignore. It looks at names and at whether two paths exist, never at what files contain.

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

    /// The facts proposals are made from, one per registered project: the last commit of each
    /// repository, asked of git one child at a time (D-29), and the roadmap facts (every row
    /// checked, a pickup gate on the next brief) read from the working tree without a child
    /// process (`roadmap::snapshot::read_tree`).
    pub fn facts(&self) -> Vec<Facts> {
        let mut asked = false;
        self.registered
            .iter()
            .map(|registered| {
                let mut facts = Facts::default();
                let Some(folder) = &registered.folder else {
                    return facts;
                };
                if folder.git {
                    if asked {
                        thread::sleep(process::SPACING);
                    }
                    asked = true;
                    facts.last_commit = proposal::last_commit(&folder.path);
                }
                if folder.roadmap {
                    snapshot::read_tree(&folder.path).fill(&mut facts);
                }
                facts
            })
            .collect()
    }
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

/// Adopts folders of the data root by name (D-7): appends the ones the workspace file does not
/// list yet, after a dated copy of it in the backups folder, then registers them; an ignored
/// folder leaves the ignore list. The workspace file goes first, so a failure in between leaves a
/// folder listed but not registered, and adopting it again finishes the job.
pub fn adopt(store: &Store, names: &[&str], today: &str) -> Result<Vec<Adopted>, Error> {
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
    let listings = list_in_workspace(&registry, &picked, &store.backups, today)?;
    store.change(today, |changes| {
        for folder in &picked {
            changes.unignore(&folder.name);
            changes.add_project(&folder.name, &folder.name, folder.first_state())?;
        }
        Ok(())
    })?;
    Ok(picked
        .into_iter()
        .zip(listings)
        .map(|(folder, workspace)| Adopted {
            state: folder.first_state(),
            name: folder.name,
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

/// Appends the folders the workspace file does not list yet, in one write after one copy.
fn list_in_workspace(
    registry: &Registry,
    picked: &[Folder],
    backups: &Path,
    today: &str,
) -> Result<Vec<Listing>, Error> {
    let Some(path) = registry.workspace_file() else {
        return Ok(vec![Listing::NoWorkspaceFile; picked.len()]);
    };
    let unreadable = |e: String| Error::Invalid(format!("{}: {e}", path.display()));
    let original = fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
    let dir = path.parent().unwrap_or(Path::new("."));
    let listed = workspace::entries(&original).map_err(unreadable)?;
    let mut text = original.clone();
    let mut appended = Vec::new();
    for folder in picked {
        let new = !workspace::lists(&listed, dir, &folder.path);
        if new {
            text = workspace::append(&text, &workspace::entry_path(dir, &folder.path))
                .map_err(unreadable)?;
        }
        appended.push(new);
    }
    if text == original {
        return Ok(vec![Listing::AlreadyListed; picked.len()]);
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (stem, ext) = name.split_at(name.find('.').unwrap_or(name.len()));
    let backup = files::dated_copy(original.as_bytes(), backups, stem, today, ext)
        .map_err(|e| Error::io(backups, e))?;
    files::write_atomic(&path, text.as_bytes()).map_err(|e| Error::io(&path, e))?;
    Ok(appended
        .into_iter()
        .map(|new| match new {
            true => Listing::Appended {
                backup: backup.clone(),
            },
            false => Listing::AlreadyListed,
        })
        .collect())
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
        let found = folders(&store.defaults.data_root).unwrap();
        let summary: Vec<_> = found
            .iter()
            .map(|f| (f.name.as_str(), f.git, f.roadmap, f.first_state()))
            .collect();
        assert_eq!(
            summary,
            [
                ("alpha", true, true, State::Active),
                ("beta", true, false, State::Unformalised),
                ("notes", false, false, State::Unformalised),
                ("worktree", true, false, State::Unformalised),
                ("wrangler", true, true, State::Active),
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
        let workspace_file = store.defaults.data_root.join("main.code-workspace");
        ignore(&store, &["beta"], TODAY).unwrap();

        let adopted = adopt(&store, &["ALPHA", "beta"], TODAY).unwrap();
        let backup = store.backups.join("main.2026-10-31.code-workspace");
        assert_eq!(
            adopted,
            [
                Adopted {
                    name: "alpha".to_owned(),
                    state: State::Active,
                    workspace: Listing::AlreadyListed,
                },
                Adopted {
                    name: "beta".to_owned(),
                    state: State::Unformalised,
                    workspace: Listing::Appended {
                        backup: backup.clone()
                    },
                },
            ]
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
        assert!(adopt(&store, &["beta"], TODAY).is_err());
        assert!(ignore(&store, &["alpha"], TODAY).is_err());
        assert!(adopt(&store, &["missing"], TODAY).is_err());
        assert!(adopt(&store, &[".hidden"], TODAY).is_err());
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
        adopt(&store, &["alpha", "beta"], TODAY).unwrap();
        let view = view(&store.load().unwrap().registry).unwrap();
        let facts = view.facts();
        let summary: Vec<_> = view
            .registered
            .iter()
            .zip(&facts)
            .map(|(r, f)| (r.project.name.as_str(), f.all_rows_checked, f.pickup_gate))
            .collect();
        assert_eq!(
            summary,
            [
                ("wrangler", Some(false), Some(false)),
                ("alpha", Some(true), Some(false)),
                ("beta", Some(false), Some(true)),
            ]
        );
        assert!(facts.iter().all(|f| f.last_commit.is_none()));
    }

    #[test]
    fn adopt_without_a_workspace_file_only_registers() {
        let scratch = Scratch::new("discovery-no-workspace");
        let store = data_root(&scratch);
        fs::remove_file(store.defaults.data_root.join("main.code-workspace")).unwrap();
        let adopted = adopt(&store, &["notes"], TODAY).unwrap();
        assert_eq!(adopted[0].workspace, Listing::NoWorkspaceFile);
        assert!(store.load().unwrap().registry.project("notes").is_some());
    }

    #[test]
    fn a_failed_workspace_write_registers_nothing() {
        let scratch = Scratch::new("discovery-bad-workspace");
        let store = data_root(&scratch);
        let file = store.defaults.data_root.join("main.code-workspace");
        fs::write(&file, "{ \"folders\": [ ").unwrap();
        assert!(adopt(&store, &["beta"], TODAY).is_err());
        assert!(!store.path.exists(), "the registry was not written");
        assert_eq!(fs::read_to_string(&file).unwrap(), "{ \"folders\": [ ");
    }
}
