//! What the dashboard shows (D-6, D-9, D-25): the registry set against the data root, each
//! registered project's snapshot, the facts its proposals come from and the mail its briefs hand
//! on. It is read in two passes on worker threads: the files first, with no child process, so the
//! list shows at once; then git, one child at a time with the usual spacing (D-29), each
//! project's row filled in as its turn ends. A watcher on the data root's top level asks for a new
//! read of the files when a folder appears, goes or is renamed (D-7). The groups, the sort keys
//! and the filter are here too, as plain functions the list pane and its keys share.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering as Atomic};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use chrono::{DateTime, Utc};
use crw_core::discovery::{self, Folder, Registered};
use crw_core::inbox::{self, Outgoing};
use crw_core::process;
use crw_core::proposal::{self, Facts, Proposal};
use crw_core::registry::{Project, Registry, State, Store};
use crw_core::roadmap::snapshot::{self, GitReading, InProgress, Snapshot};
use crw_core::roadmap::table::{Counts, Row};
use eframe::egui;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};
use serde::{Deserialize, Serialize};

/// The portfolio as one read of the files found it, with the git readings added as they come.
#[derive(Debug, Clone, Default)]
pub struct Portfolio {
    pub registry: PathBuf,
    /// Whether the registry file exists; when it does not, the default registry is shown.
    pub written: bool,
    pub data_root: PathBuf,
    pub projects: Vec<ProjectRow>,
    /// Folders of the data root that are neither registered nor ignored.
    pub unadopted: Vec<Folder>,
    /// Why the registry or the data root could not be read; the list is empty then.
    pub problem: Option<String>,
}

/// A registered project and what its files say.
#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub project: Project,
    pub path: PathBuf,
    /// `None` when the folder is missing.
    pub folder: Option<Folder>,
    pub snapshot: Snapshot,
    pub facts: Facts,
    pub outgoing: Vec<Outgoing>,
    /// The git pass's reading; `None` until it reaches the project, and for a folder that is
    /// not a repository.
    pub git: Option<Git>,
}

/// What the git pass reads of one repository.
#[derive(Debug, Clone, Default)]
pub struct Git {
    pub reading: GitReading,
    pub last_commit: Option<DateTime<Utc>>,
}

impl ProjectRow {
    pub fn name(&self) -> &str {
        &self.project.name
    }

    pub fn is_repository(&self) -> bool {
        self.folder.as_ref().is_some_and(|folder| folder.git)
    }

    /// A repository the git pass has not reached yet.
    pub fn awaits_git(&self) -> bool {
        self.is_repository() && self.git.is_none()
    }

    pub fn next(&self) -> Option<&Row> {
        self.snapshot.next()
    }

    /// The counts of the current reading; zero rows without a roadmap.
    pub fn counts(&self) -> Counts {
        self.snapshot
            .current()
            .map(|roadmap| roadmap.counts())
            .unwrap_or_default()
    }

    /// Done over total in thousandths; `None` without rows.
    pub fn completion(&self) -> Option<usize> {
        let counts = self.counts();
        (counts.total > 0).then(|| counts.done * 1000 / counts.total)
    }

    /// Rows still open; `None` without rows.
    pub fn remaining(&self) -> Option<usize> {
        let counts = self.counts();
        (counts.total > 0).then(|| counts.total - counts.done)
    }

    /// The owner's open items: owner tasks and the older forms (D-10).
    pub fn open_items(&self) -> usize {
        let tasks = self.snapshot.tasks.as_ref().map_or(0, |t| t.open().count());
        tasks
            + self
                .snapshot
                .legacy
                .iter()
                .filter(|item| item.is_open())
                .count()
    }

    /// Open inbox messages, and how many of them are urgent (D-13).
    pub fn open_messages(&self) -> (usize, usize) {
        self.snapshot.inbox.as_ref().map_or((0, 0), |inbox| {
            let open: Vec<_> = inbox.open().collect();
            (open.len(), open.iter().filter(|m| m.urgent).count())
        })
    }

    /// Why a session is taken to be in progress (D-9); the files' reasons until git answers.
    pub fn in_progress(&self) -> Vec<InProgress> {
        self.snapshot.in_progress()
    }

    pub fn proposals(&self, now: DateTime<Utc>) -> Vec<Proposal> {
        proposal::propose(self.project.state, &self.facts, now)
    }

    /// Adds the git pass's reading.
    fn add_git(&mut self, git: Git) {
        git.reading.clone().fill(&mut self.snapshot);
        self.facts = Facts::default();
        self.snapshot.fill(&mut self.facts);
        self.facts.last_commit = git.last_commit;
        self.git = Some(git);
    }
}

/// The first pass: the registry, the data root and every registered project's files. No child
/// process runs.
pub fn read_files(store: &Store) -> Portfolio {
    let mut portfolio = Portfolio {
        registry: store.path.clone(),
        ..Portfolio::default()
    };
    let loaded = match store.load() {
        Ok(loaded) => loaded,
        Err(e) => {
            portfolio.problem = Some(e.to_string());
            return portfolio;
        }
    };
    portfolio.written = loaded.written;
    let registry = loaded.registry;
    portfolio.data_root = registry.data_root();
    let view = match discovery::view(&registry) {
        Ok(view) => view,
        Err(e) => {
            portfolio.problem = Some(e.to_string());
            return portfolio;
        }
    };
    portfolio.unadopted = view.unadopted;
    portfolio.projects = view
        .registered
        .into_iter()
        .map(|registered| row(registered, &registry))
        .collect();
    portfolio
}

fn row(registered: Registered, registry: &Registry) -> ProjectRow {
    let snapshot = match &registered.folder {
        Some(folder) => snapshot::read_tree(&folder.path),
        None => Snapshot::default(),
    };
    let mut facts = Facts::default();
    snapshot.fill(&mut facts);
    let outgoing = inbox::outgoing(&snapshot.briefs, |name| {
        registry.project(name).map(|project| project.name.clone())
    });
    ProjectRow {
        project: registered.project,
        path: registered.path,
        folder: registered.folder,
        snapshot,
        facts,
        outgoing,
        git: None,
    }
}

/// Git children of every thread run one at a time (D-29): a superseded pass may still be
/// finishing its child when the next one starts.
static GIT: Mutex<()> = Mutex::new(());

/// The git pass for one repository: `HEAD`'s README and the status, then the last commit,
/// each child after the spacing.
pub fn read_git(folder: &Path) -> Git {
    let _turn = GIT.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    thread::sleep(process::SPACING);
    let reading = snapshot::read_git(folder).unwrap_or_default();
    thread::sleep(process::SPACING);
    Git {
        reading,
        last_commit: proposal::last_commit(folder),
    }
}

/// What the worker threads and the watcher send the window.
pub enum Message {
    Files {
        generation: u64,
        portfolio: Portfolio,
    },
    Git {
        generation: u64,
        project: String,
        git: Git,
        done: usize,
        of: usize,
    },
    GitDone {
        generation: u64,
    },
    /// A folder of the data root appeared, went or was renamed.
    Folders,
}

/// Starts the reads, receives their results and keeps the git readings for the reads to come:
/// a read of the files after a write keeps what git said before it.
pub struct Loader {
    store: Arc<Store>,
    ctx: egui::Context,
    tx: Sender<Message>,
    rx: Receiver<Message>,
    files: Arc<AtomicU64>,
    git: Arc<AtomicU64>,
    readings: HashMap<String, Git>,
    /// A read of the files is under way.
    pub reading_files: bool,
    /// The git pass: repositories read, and how many there are.
    pub git_progress: Option<(usize, usize)>,
    watcher: Option<(PathBuf, RecommendedWatcher)>,
    pub watch_problem: Option<String>,
}

impl Loader {
    pub fn new(store: Arc<Store>, ctx: egui::Context) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            store,
            ctx,
            tx,
            rx,
            files: Arc::new(AtomicU64::new(0)),
            git: Arc::new(AtomicU64::new(0)),
            readings: HashMap::new(),
            reading_files: false,
            git_progress: None,
            watcher: None,
            watch_problem: None,
        }
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Reads the files again on a worker thread; an older read still running is dropped.
    pub fn read_files(&mut self) {
        let generation = self.files.fetch_add(1, Atomic::SeqCst) + 1;
        self.reading_files = true;
        let (store, tx, ctx) = (self.store.clone(), self.tx.clone(), self.ctx.clone());
        thread::spawn(move || {
            let portfolio = read_files(&store);
            let _ = tx.send(Message::Files {
                generation,
                portfolio,
            });
            ctx.request_repaint();
        });
    }

    /// Runs the git pass on a worker thread, after superseding any pass still running.
    pub fn read_git(&mut self) {
        let generation = self.git.fetch_add(1, Atomic::SeqCst) + 1;
        let (store, tx, ctx) = (self.store.clone(), self.tx.clone(), self.ctx.clone());
        let latest = self.git.clone();
        self.git_progress = Some((0, 0));
        thread::spawn(move || {
            let repositories: Vec<(String, PathBuf)> = store
                .load()
                .map(|loaded| {
                    let root = loaded.registry.data_root();
                    loaded
                        .registry
                        .projects
                        .iter()
                        .map(|project| (project.name.clone(), project.folder(&root)))
                        .filter(|(_, folder)| folder.join(".git").exists())
                        .collect()
                })
                .unwrap_or_default();
            let of = repositories.len();
            for (done, (project, folder)) in repositories.into_iter().enumerate() {
                if latest.load(Atomic::SeqCst) != generation {
                    return;
                }
                let git = read_git(&folder);
                let _ = tx.send(Message::Git {
                    generation,
                    project,
                    git,
                    done: done + 1,
                    of,
                });
                ctx.request_repaint();
            }
            let _ = tx.send(Message::GitDone { generation });
            ctx.request_repaint();
        });
    }

    /// Whether a read is under way.
    pub fn busy(&self) -> bool {
        self.reading_files || self.git_progress.is_some()
    }

    /// Takes in what the threads sent: a newer portfolio (with the git readings so far), or
    /// one repository's reading. Returns whether the data root's folders changed.
    pub fn receive(&mut self, portfolio: &mut Portfolio) -> bool {
        let mut folders = false;
        while let Ok(message) = self.rx.try_recv() {
            match message {
                Message::Files {
                    generation,
                    portfolio: mut read,
                } if generation == self.files.load(Atomic::SeqCst) => {
                    for row in &mut read.projects {
                        if let Some(git) = self.readings.get(&key(row.name()))
                            && row.is_repository()
                        {
                            row.add_git(git.clone());
                        }
                    }
                    *portfolio = read;
                    self.reading_files = false;
                    self.watch(&portfolio.data_root);
                }
                Message::Git {
                    generation,
                    project,
                    git,
                    done,
                    of,
                } if generation == self.git.load(Atomic::SeqCst) => {
                    if let Some(row) = portfolio
                        .projects
                        .iter_mut()
                        .find(|row| key(row.name()) == key(&project))
                    {
                        row.add_git(git.clone());
                    }
                    self.readings.insert(key(&project), git);
                    self.git_progress = Some((done, of));
                }
                Message::GitDone { generation } if generation == self.git.load(Atomic::SeqCst) => {
                    self.git_progress = None;
                }
                Message::Folders => folders = true,
                // A superseded read's result.
                Message::Files { .. } | Message::Git { .. } | Message::GitDone { .. } => {}
            }
        }
        folders
    }

    /// Watches the data root's top level (D-7), once per data root.
    fn watch(&mut self, root: &Path) {
        if root.as_os_str().is_empty()
            || self
                .watcher
                .as_ref()
                .is_some_and(|(watched, _)| watched == root)
        {
            return;
        }
        let (tx, ctx) = (self.tx.clone(), self.ctx.clone());
        let started = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            if event.is_ok_and(|event| changes_folders(&event.kind)) {
                let _ = tx.send(Message::Folders);
                ctx.request_repaint();
            }
        })
        .and_then(|mut watcher| {
            watcher.watch(root, RecursiveMode::NonRecursive)?;
            Ok(watcher)
        });
        match started {
            Ok(watcher) => {
                self.watcher = Some((root.to_owned(), watcher));
                self.watch_problem = None;
            }
            Err(e) => {
                self.watcher = None;
                self.watch_problem = Some(format!(
                    "Not watching {} for new folders: {e}",
                    root.display()
                ));
            }
        }
    }
}

/// An entry of the data root created, removed or renamed; a write inside a project folder only
/// touches the folder's times, which is not a change of folders.
fn changes_folders(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(_))
    )
}

/// A project's name as the registry compares names: case aside.
fn key(name: &str) -> String {
    name.to_lowercase()
}

/// How the list orders the projects inside a group, after the pinned ones (D-25).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortKey {
    #[default]
    Priority,
    Completion,
    Remaining,
    LastActivity,
    Hosting,
    UsageShare,
    Name,
}

impl SortKey {
    pub const ALL: [SortKey; 7] = [
        SortKey::Priority,
        SortKey::Completion,
        SortKey::Remaining,
        SortKey::LastActivity,
        SortKey::Hosting,
        SortKey::UsageShare,
        SortKey::Name,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SortKey::Priority => "Priority",
            SortKey::Completion => "Completion",
            SortKey::Remaining => "Remaining",
            SortKey::LastActivity => "Last activity",
            SortKey::Hosting => "Hosting",
            SortKey::UsageShare => "Usage share",
            SortKey::Name => "Name",
        }
    }

    /// The brief that brings a key's data, for the keys that have none yet.
    pub fn arrives_at(self) -> Option<&'static str> {
        match self {
            SortKey::Hosting => Some("EP-7"),
            SortKey::UsageShare => Some("EP-18"),
            _ => None,
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            SortKey::Priority => "1 first",
            SortKey::Completion => "the most complete first",
            SortKey::Remaining => "the fewest open rows first",
            SortKey::LastActivity => "the latest commit first",
            SortKey::Hosting => "by remote",
            SortKey::UsageShare => "the largest share of this week's usage first",
            SortKey::Name => "A to Z",
        }
    }
}

/// A group of the list: a state, or the folders not adopted yet (D-6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupKind {
    State(State),
    Unadopted,
}

impl GroupKind {
    /// Active work first, then what waits, what is new, and what is finished.
    pub const ORDER: [GroupKind; 7] = [
        GroupKind::State(State::Active),
        GroupKind::State(State::Paused),
        GroupKind::State(State::Maintenance),
        GroupKind::State(State::Unformalised),
        GroupKind::Unadopted,
        GroupKind::State(State::Done),
        GroupKind::State(State::Archived),
    ];

    /// The name a folded group is saved under.
    pub fn name(self) -> &'static str {
        match self {
            GroupKind::State(state) => state.as_str(),
            GroupKind::Unadopted => "unadopted",
        }
    }
}

/// One line the list can select: a registered project or an unadopted folder, by name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selection {
    Project(String),
    Folder(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Member {
    /// An index into [`Portfolio::projects`].
    Project(usize),
    /// An index into [`Portfolio::unadopted`].
    Folder(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub kind: GroupKind,
    pub members: Vec<Member>,
}

impl Portfolio {
    /// The list's groups in [`GroupKind::ORDER`], empty ones left out; inside each, pinned
    /// projects first, then `sort`, then the name. `filter` keeps the names that contain it,
    /// case aside.
    pub fn groups(&self, sort: SortKey, filter: &str) -> Vec<Group> {
        let filter = filter.trim().to_lowercase();
        let shown = |name: &str| filter.is_empty() || name.to_lowercase().contains(&filter);
        GroupKind::ORDER
            .into_iter()
            .filter_map(|kind| {
                let members: Vec<Member> = match kind {
                    GroupKind::State(state) => {
                        let mut rows: Vec<usize> = (0..self.projects.len())
                            .filter(|&i| {
                                let row = &self.projects[i];
                                row.project.state == state && shown(row.name())
                            })
                            .collect();
                        rows.sort_by(|&a, &b| compare(&self.projects[a], &self.projects[b], sort));
                        rows.into_iter().map(Member::Project).collect()
                    }
                    GroupKind::Unadopted => (0..self.unadopted.len())
                        .filter(|&i| shown(&self.unadopted[i].name))
                        .map(Member::Folder)
                        .collect(),
                };
                (!members.is_empty()).then_some(Group { kind, members })
            })
            .collect()
    }

    pub fn selection(&self, member: Member) -> Selection {
        match member {
            Member::Project(i) => Selection::Project(self.projects[i].name().to_owned()),
            Member::Folder(i) => Selection::Folder(self.unadopted[i].name.clone()),
        }
    }

    pub fn project(&self, name: &str) -> Option<&ProjectRow> {
        self.projects
            .iter()
            .find(|row| row.name().eq_ignore_ascii_case(name))
    }

    pub fn folder(&self, name: &str) -> Option<&Folder> {
        self.unadopted
            .iter()
            .find(|folder| folder.name.eq_ignore_ascii_case(name))
    }

    /// The lines the arrow keys move through: every member of the groups not folded.
    pub fn order(
        &self,
        sort: SortKey,
        filter: &str,
        folded: impl Fn(GroupKind) -> bool,
    ) -> Vec<Selection> {
        self.groups(sort, filter)
            .into_iter()
            .filter(|group| !folded(group.kind))
            .flat_map(|group| group.members)
            .map(|member| self.selection(member))
            .collect()
    }
}

/// Pinned first, then the key, then the name; projects without the key's data last.
fn compare(a: &ProjectRow, b: &ProjectRow, sort: SortKey) -> Ordering {
    let last_if_none = |a: Option<usize>, b: Option<usize>| match (a, b) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    };
    b.project
        .pinned
        .cmp(&a.project.pinned)
        .then_with(|| match sort {
            SortKey::Priority => a.project.priority.cmp(&b.project.priority),
            SortKey::Completion => last_if_none(
                a.completion().map(|c| 1000 - c),
                b.completion().map(|c| 1000 - c),
            ),
            SortKey::Remaining => last_if_none(a.remaining(), b.remaining()),
            SortKey::LastActivity => b.facts.last_commit.cmp(&a.facts.last_commit),
            SortKey::Hosting | SortKey::UsageShare | SortKey::Name => Ordering::Equal,
        })
        .then_with(|| key(a.name()).cmp(&key(b.name())))
}

/// The next line after (or before) `current` in `order`; the first (or the last) when nothing
/// in `order` is selected.
pub fn step(order: &[Selection], current: Option<&Selection>, down: bool) -> Option<Selection> {
    let at = current.and_then(|current| order.iter().position(|s| s == current));
    let next = match (at, down) {
        (None, true) => 0,
        (None, false) => order.len().checked_sub(1)?,
        (Some(at), true) => (at + 1).min(order.len() - 1),
        (Some(at), false) => at.saturating_sub(1),
    };
    order.get(next).cloned()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::TimeDelta;
    use crw_core::registry::Defaults;
    use crw_core::roadmap::EpId;

    use super::*;

    /// A folder under the system temp folder, removed when dropped.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("crw-app-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
    }

    /// The screenshot runs' registry over the fake portfolio, in a scratch folder.
    fn fake_store(scratch: &Scratch) -> Store {
        let root = fixtures().join("portfolio");
        let text = fs::read_to_string(fixtures().join("registry.toml"))
            .unwrap()
            .replace("@DATA_ROOT@", &format!("'{}'", root.display()));
        let path = scratch.0.join("registry.toml");
        fs::write(&path, text).unwrap();
        Store::new(
            path,
            scratch.0.join("backups"),
            Defaults {
                data_root: root.clone(),
                repository: root.join("harbor"),
            },
        )
    }

    fn names(portfolio: &Portfolio, group: &Group) -> Vec<String> {
        group
            .members
            .iter()
            .map(|&member| match portfolio.selection(member) {
                Selection::Project(name) | Selection::Folder(name) => name,
            })
            .collect()
    }

    #[test]
    fn the_fake_portfolio_reads_into_its_groups() {
        let scratch = Scratch::new("portfolio-groups");
        let store = fake_store(&scratch);
        let portfolio = read_files(&store);
        assert_eq!(portfolio.problem, None);
        assert!(portfolio.written);
        let groups = portfolio.groups(SortKey::Priority, "");
        let summary: Vec<(&str, Vec<String>)> = groups
            .iter()
            .map(|group| (group.kind.name(), names(&portfolio, group)))
            .collect();
        let owned = |names: &[&str]| names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
        assert_eq!(
            summary,
            [
                ("active", owned(&["harbor", "lantern", "meadow", "quarry"])),
                ("paused", owned(&["compass"])),
                ("maintenance", owned(&["orchard"])),
                ("unformalised", owned(&["atlas"])),
                ("unadopted", owned(&["sketchbook"])),
                ("done", owned(&["beacon"])),
                ("archived", owned(&["tundra"])),
            ]
        );
        // Nothing here is a repository: the git pass has nothing to wait for.
        assert!(portfolio.projects.iter().all(|row| !row.awaits_git()));

        let harbor = portfolio.project("Harbor").unwrap();
        assert_eq!(harbor.next().map(|row| row.id), Some(EpId::new(4)));
        assert_eq!((harbor.counts().done, harbor.counts().total), (4, 9));
        assert_eq!(harbor.open_items(), 2);
        assert_eq!(harbor.open_messages(), (1, 0));
        assert_eq!(
            harbor.in_progress(),
            [InProgress::BriefWithoutCompletion(EpId::new(4))]
        );
        // Mail between registered projects resolves; a name the registry lacks does not.
        let lantern = portfolio.project("lantern").unwrap();
        let recipients: Vec<_> = lantern
            .outgoing
            .iter()
            .map(|block| block.recipient.clone())
            .collect();
        assert_eq!(
            recipients,
            [
                inbox::Recipient::Project("harbor".to_owned()),
                inbox::Recipient::Unknown
            ]
        );
        assert_eq!(lantern.open_messages(), (1, 1));
        // The pickup gate proposes a pause; the finished project in maintenance is left alone.
        let now = Utc::now();
        let quarry = portfolio.project("quarry").unwrap();
        assert_eq!(quarry.proposals(now).len(), 1);
        assert!(quarry.in_progress().is_empty());
        assert!(
            portfolio
                .project("orchard")
                .unwrap()
                .proposals(now)
                .is_empty()
        );
        assert_eq!(portfolio.project("orchard").unwrap().open_items(), 3);
        assert!(portfolio.project("tundra").unwrap().folder.is_none());
        assert_eq!(portfolio.project("atlas").unwrap().counts().total, 0);
    }

    #[test]
    fn sort_keys_and_the_filter() {
        let scratch = Scratch::new("portfolio-sort");
        let store = fake_store(&scratch);
        let mut portfolio = read_files(&store);
        let active = |portfolio: &Portfolio, sort, filter| {
            let groups = portfolio.groups(sort, filter);
            groups
                .first()
                .map(|group| names(portfolio, group))
                .unwrap_or_default()
        };
        // harbor is pinned, so it leads whatever the key.
        assert_eq!(
            active(&portfolio, SortKey::Completion, ""),
            ["harbor", "meadow", "quarry", "lantern"]
        );
        assert_eq!(
            active(&portfolio, SortKey::Remaining, ""),
            ["harbor", "meadow", "quarry", "lantern"]
        );
        assert_eq!(
            active(&portfolio, SortKey::Name, ""),
            ["harbor", "lantern", "meadow", "quarry"]
        );
        // Last activity: the latest commit first, unknown ones last.
        let now = Utc::now();
        for (name, days) in [("quarry", 1), ("lantern", 9)] {
            let row = portfolio
                .projects
                .iter_mut()
                .find(|row| row.name() == name)
                .unwrap();
            row.facts.last_commit = Some(now - TimeDelta::days(days));
        }
        assert_eq!(
            active(&portfolio, SortKey::LastActivity, ""),
            ["harbor", "quarry", "lantern", "meadow"]
        );
        // Unpinned, harbor sorts by its key like the rest.
        portfolio.projects[0].project.pinned = false;
        assert_eq!(
            active(&portfolio, SortKey::Name, ""),
            ["harbor", "lantern", "meadow", "quarry"]
        );
        assert_eq!(
            active(&portfolio, SortKey::Remaining, ""),
            ["meadow", "quarry", "lantern", "harbor"]
        );
        // The filter keeps names that contain it, case aside, across groups.
        let groups = portfolio.groups(SortKey::Name, "AR");
        let kept: Vec<Vec<String>> = groups.iter().map(|g| names(&portfolio, g)).collect();
        assert_eq!(
            kept,
            [
                vec!["harbor".to_owned(), "quarry".to_owned()],
                vec!["orchard".to_owned()]
            ]
        );
        assert!(portfolio.groups(SortKey::Name, "nothing").is_empty());
    }

    #[test]
    fn the_arrow_keys_walk_the_groups_not_folded() {
        let scratch = Scratch::new("portfolio-keys");
        let store = fake_store(&scratch);
        let portfolio = read_files(&store);
        let folded = |kind: GroupKind| {
            matches!(
                kind,
                GroupKind::State(State::Done | State::Archived | State::Paused)
            )
        };
        let order = portfolio.order(SortKey::Priority, "", folded);
        let project = |name: &str| Selection::Project(name.to_owned());
        assert_eq!(
            order,
            [
                project("harbor"),
                project("lantern"),
                project("meadow"),
                project("quarry"),
                project("orchard"),
                project("atlas"),
                Selection::Folder("sketchbook".to_owned()),
            ]
        );
        assert_eq!(step(&order, None, true), Some(project("harbor")));
        assert_eq!(
            step(&order, None, false),
            Some(Selection::Folder("sketchbook".to_owned()))
        );
        assert_eq!(
            step(&order, Some(&project("harbor")), true),
            Some(project("lantern"))
        );
        // The ends hold; a selection in a folded group starts over from the top.
        assert_eq!(
            step(&order, Some(&project("harbor")), false),
            Some(project("harbor"))
        );
        assert_eq!(
            step(&order, order.last(), true),
            Some(Selection::Folder("sketchbook".to_owned()))
        );
        assert_eq!(
            step(&order, Some(&project("beacon")), true),
            Some(project("harbor"))
        );
        assert_eq!(step(&[], None, true), None);
    }

    /// Runs git on a scratch repository (never the `claude` CLI, D-29), one child at a time.
    #[test]
    fn the_git_pass_adds_the_head_reading_and_the_last_commit() {
        let scratch = Scratch::new("portfolio-git");
        let data = scratch.0.join("data");
        let repo = data.join("delta");
        fs::create_dir_all(repo.join("roadmap")).unwrap();
        const ROADMAP: &str = "## Phase P1 — Build\n\n| # | Brief | Size | Depends on | Core | Done |\n|---|---|---|---|---|---|\n| EP-1 | One | S | — | core | ☐ |\n";
        fs::write(repo.join("roadmap").join("README.md"), ROADMAP).unwrap();
        // A neutral identity, no signing and no line-ending conversion, as EP-4's tests run it.
        let git = |args: &[&str]| {
            let mut all = vec![
                "-C",
                repo.to_str().unwrap(),
                "-c",
                "user.name=crw-test",
                "-c",
                "user.email=crw-test@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.autocrlf=false",
            ];
            all.extend(args);
            thread::sleep(process::SPACING);
            process::output_of(Path::new("git"), all, process::TIMEOUT)
                .unwrap_or_else(|e| panic!("git {args:?}: {e}"))
        };
        git(&["init", "-q"]);
        git(&["add", "roadmap"]);
        git(&["commit", "-q", "-m", "plan"]);
        // The working tree moves on: EP-1 checked, not committed.
        fs::write(
            repo.join("roadmap").join("README.md"),
            ROADMAP.replace("☐", "☑ `abc1234`"),
        )
        .unwrap();
        let store = Store::new(
            scratch.0.join("registry.toml"),
            scratch.0.join("backups"),
            Defaults {
                data_root: data.clone(),
                repository: repo.clone(),
            },
        );
        let portfolio = read_files(&store);
        let mut row = portfolio.projects[0].clone();
        assert!(row.awaits_git());
        assert_eq!(row.facts.all_rows_checked, Some(true));
        assert!(row.in_progress().is_empty(), "git not asked yet");

        row.add_git(read_git(&repo));
        assert!(!row.awaits_git());
        assert!(row.facts.last_commit.is_some());
        assert_eq!(row.snapshot.head.as_ref().unwrap().counts().done, 0);
        assert_eq!(
            row.counts().done,
            1,
            "the working tree is the reading shown"
        );
        assert!(row.snapshot.differs());
        assert_eq!(row.in_progress(), [InProgress::ReadmeDirty]);
    }
}
