//! What the dashboard shows (D-6, D-9, D-24, D-25): the registry set against the data root, each
//! registered project's snapshot, the facts its proposals come from, the mail its briefs hand on
//! and its badges. It is read in passes on worker threads: the files first, with no child process,
//! so the list shows at once; then the backup job and git, one child at a time with the usual
//! spacing (D-29), each project's row filled in as its turn ends; and the GitHub pass, from the
//! ledger's cache at once and from the network when the cache is a day old or on F5, its children
//! taking turns with git's. A watcher on the data root's top level asks for a new read of the files
//! when a folder appears, goes or is renamed (D-7). The groups, the sort keys and the filters are
//! here too, as plain functions the list pane and its keys share.

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering as Atomic};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use crw_core::badges::activity::Activity;
use crw_core::badges::backup::{self, Backup};
use crw_core::badges::health::{self, Run};
use crw_core::badges::hosting::{self, Check, Class, Hosting, Listed};
use crw_core::badges::online::{self, Reading};
use crw_core::badges::stack::{self, Chip, Kind};
use crw_core::discovery::{self, Folder, Registered};
use crw_core::git;
use crw_core::inbox::mailman::Mark;
use crw_core::inbox::{self, Outgoing};
use crw_core::ledger::Ledger;
use crw_core::process;
use crw_core::proposal::{self, Facts, Proposal};
use crw_core::registry::{Project, Registry, State, Store};
use crw_core::roadmap::snapshot::{self, GitReading, InProgress, Snapshot};
use crw_core::roadmap::table::{Counts, Row};
use eframe::egui;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};
use serde::{Deserialize, Serialize};

/// While the window is open, the GitHub pass looks at its cache's age this often; it asks the
/// network only when the cache is older than [`online::FRESH_FOR`].
const ONLINE_CHECK_EVERY: Duration = Duration::from_secs(3600);

/// The portfolio as one read of the files found it, with the git and GitHub readings added as
/// they come.
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
    /// The account's repositories (D-24), from the cache or the GitHub pass.
    pub account: Option<Fetched<Result<Vec<Listed>, String>>>,
    /// The backup job as the git pass last read it; `None` when the registry sets none.
    pub backup: Option<backup::Reading>,
}

/// A networked reading and the time it was fetched, this session or from the cache.
#[derive(Debug, Clone)]
pub struct Fetched<T> {
    pub value: T,
    pub at: DateTime<Utc>,
}

/// What a row's badges show (D-24, D-25): the files' readings at once, the backup and git's with
/// the git pass, GitHub's from the cache or the GitHub pass, and the week's usage from the
/// ledger at each pass (EP-18b).
#[derive(Debug, Clone)]
pub struct Badges {
    pub stack: Vec<Chip>,
    pub license: Option<String>,
    /// Workflow files in `.github/workflows`.
    pub workflows: usize,
    /// The origin's URL, from `.git/config`.
    pub remote: Option<String>,
    /// From the remote and the account's list, once it is read.
    pub hosting: Hosting,
    pub activity: Option<Activity>,
    pub backup: Option<Backup>,
    pub head: Option<Fetched<Check>>,
    pub run: Option<Fetched<Result<Option<Run>, String>>>,
    /// None while no session of the project has a share this week, or no week is known.
    pub usage: Option<Usage>,
}

/// A project's share of the week's usage (D-23, D-25; EP-18b): its sessions' parts since the
/// weekly window opened, in points of each limit, with the week they cover.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Usage {
    pub weekly: Option<f64>,
    pub fable: Option<f64>,
    /// Its sessions with a share.
    pub sessions: u32,
    /// The weekly window's start.
    pub since: DateTime<Utc>,
    /// Every project's weekly sum: all that the window attributed this week.
    pub attributed: f64,
}

impl Default for Badges {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            license: None,
            workflows: 0,
            remote: None,
            hosting: Hosting::Local,
            activity: None,
            backup: None,
            head: None,
            run: None,
            usage: None,
        }
    }
}

impl Badges {
    /// The readings of the project's files: no child.
    fn read(folder: &Path) -> Self {
        Self {
            stack: stack::read(folder),
            license: health::license(folder),
            workflows: health::workflows(folder),
            remote: git::remote_url(folder, "origin"),
            ..Self::default()
        }
    }

    /// Whether the head check still describes the clone: its local commit is the one git last
    /// read (before the git pass, the check is all there is).
    pub fn head_is_current(&self) -> bool {
        match (&self.head, &self.activity) {
            (Some(head), Some(activity)) => {
                head.value.local.is_some() && head.value.local == activity.oid
            }
            (Some(_), None) => true,
            (None, _) => false,
        }
    }
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
    /// What the mailman made of each block (EP-20), set into the row at each pass.
    pub mail: Vec<Mark>,
    /// The git pass's reading; `None` until it reaches the project, and for a folder that is
    /// not a repository.
    pub git: Option<Git>,
    pub badges: Badges,
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

    /// The proposals, each saying whether the owner dismissed it (D-6 addendum).
    pub fn proposals(&self, now: DateTime<Utc>) -> Vec<Proposal> {
        proposal::propose(
            self.project.state,
            &self.facts,
            &self.project.dismissed,
            now,
        )
    }

    /// Whether a proposal the owner has not dismissed waits: the row's bulb.
    pub fn proposes(&self, now: DateTime<Utc>) -> bool {
        self.proposals(now).iter().any(|p| !p.dismissed)
    }

    /// The folder's own name, which an orphan is matched by as well as the project's.
    pub fn folder_name(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// The hosting class the sort key and the filter use; `None` when the folder is missing.
    pub fn hosting_class(&self) -> Option<Class> {
        self.folder.as_ref().map(|_| self.badges.hosting.class())
    }

    /// This week's share in tenths of a point of the weekly limit, as the badge writes it
    /// (`ledger::points`), for the sort key; `None` without one (EP-18b).
    pub fn usage_tenths(&self) -> Option<i64> {
        let weekly = self.badges.usage?.weekly?;
        Some((weekly * 10.0).round() as i64)
    }

    /// Adds the git pass's reading.
    fn add_git(&mut self, git: Git) {
        git.reading.clone().fill(&mut self.snapshot);
        self.facts = Facts::default();
        self.snapshot.fill(&mut self.facts);
        self.facts.last_commit = git.last_commit;
        self.badges.activity = git
            .reading
            .worktree
            .as_ref()
            .map(|status| Activity::read(status, git.last_commit));
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
    let (snapshot, badges) = match &registered.folder {
        Some(folder) => (
            snapshot::read_tree(&folder.path),
            Badges::read(&folder.path),
        ),
        None => (Snapshot::default(), Badges::default()),
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
        mail: Vec::new(),
        git: None,
        badges,
    }
}

/// The git pass for one repository: `HEAD`'s README and the status, then the last commit, in
/// one turn among the threads that start children (D-29), each child after the spacing.
pub fn read_git(folder: &Path) -> Git {
    let _turn = process::turn();
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
    /// The backup job, read at the start of a git pass; `reading` is `None` when none is set.
    Backups {
        generation: u64,
        reading: Option<backup::Reading>,
        projects: HashMap<String, Backup>,
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
    /// A networked reading, from the cache or the network; `progress` counts the projects asked
    /// about, and is `None` for the cache's readings.
    Online {
        generation: u64,
        reading: Reading,
        at: DateTime<Utc>,
        progress: Option<(usize, usize)>,
    },
    OnlineDone {
        generation: u64,
        problem: Option<String>,
    },
    /// A folder of the data root appeared, went or was renamed.
    Folders,
}

/// The networked readings so far, by project name.
#[derive(Debug, Default)]
struct Online {
    account: Option<Fetched<Result<Vec<Listed>, String>>>,
    heads: HashMap<String, Fetched<Check>>,
    runs: HashMap<String, Fetched<Result<Option<Run>, String>>>,
}

impl Online {
    /// Keeps `reading` unless a newer one of its kind is kept already: the cache, read again,
    /// never hides what this session fetched.
    fn put(&mut self, reading: Reading, at: DateTime<Utc>) {
        fn by_project<T>(
            map: &mut HashMap<String, Fetched<T>>,
            project: &str,
            value: T,
            at: DateTime<Utc>,
        ) {
            if map.get(&key(project)).is_none_or(|old| at >= old.at) {
                map.insert(key(project), Fetched { value, at });
            }
        }
        match reading {
            Reading::Account(value) => {
                if self.account.as_ref().is_none_or(|old| at >= old.at) {
                    self.account = Some(Fetched { value, at });
                }
            }
            Reading::Head { project, check } => by_project(&mut self.heads, &project, check, at),
            Reading::Run { project, run } => by_project(&mut self.runs, &project, run, at),
        }
    }
}

/// Starts the reads, receives their results and keeps the git, backup and GitHub readings for
/// the reads to come: a read of the files after a write keeps what the other passes said.
pub struct Loader {
    store: Arc<Store>,
    /// The ledger the GitHub readings are cached in; `None` without a profile folder.
    ledger: Option<PathBuf>,
    /// Whether the GitHub pass may ask the network; a screenshot run's only reads its cache.
    online_allowed: bool,
    ctx: egui::Context,
    tx: Sender<Message>,
    rx: Receiver<Message>,
    files: Arc<AtomicU64>,
    git: Arc<AtomicU64>,
    online: Arc<AtomicU64>,
    readings: HashMap<String, Git>,
    backups: Option<(backup::Reading, HashMap<String, Backup>)>,
    fetched: Online,
    /// A read of the files is under way.
    pub reading_files: bool,
    /// The git pass: repositories read, and how many there are.
    pub git_progress: Option<(usize, usize)>,
    /// The GitHub pass: projects asked about, and how many there are.
    pub online_progress: Option<(usize, usize)>,
    /// Why the last GitHub pass could not use the cache.
    pub online_problem: Option<String>,
    online_checked: Option<Instant>,
    watcher: Option<(PathBuf, RecommendedWatcher)>,
    pub watch_problem: Option<String>,
}

impl Loader {
    pub fn new(
        store: Arc<Store>,
        ledger: Option<PathBuf>,
        online_allowed: bool,
        ctx: egui::Context,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            store,
            ledger,
            online_allowed,
            ctx,
            tx,
            rx,
            files: Arc::new(AtomicU64::new(0)),
            git: Arc::new(AtomicU64::new(0)),
            online: Arc::new(AtomicU64::new(0)),
            readings: HashMap::new(),
            backups: None,
            fetched: Online::default(),
            reading_files: false,
            git_progress: None,
            online_progress: None,
            online_problem: None,
            online_checked: None,
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

    /// Runs the git pass on a worker thread, after superseding any pass still running: the
    /// backup job first (file reads), then git, one repository after another.
    pub fn read_git(&mut self) {
        let generation = self.git.fetch_add(1, Atomic::SeqCst) + 1;
        let (store, tx, ctx) = (self.store.clone(), self.tx.clone(), self.ctx.clone());
        let latest = self.git.clone();
        self.git_progress = Some((0, 0));
        thread::spawn(move || {
            let registry = store.load().map(|loaded| loaded.registry).ok();
            let projects: Vec<(String, PathBuf)> = registry
                .as_ref()
                .map(|registry| {
                    let root = registry.data_root();
                    registry
                        .projects
                        .iter()
                        .map(|project| (project.name.clone(), project.folder(&root)))
                        .collect()
                })
                .unwrap_or_default();
            let (settings, log) = registry
                .as_ref()
                .map(Registry::backup_files)
                .unwrap_or_default();
            let reading = settings.map(|settings| backup::Reading::read(&settings, log.as_deref()));
            let backups = reading
                .as_ref()
                .map(|reading| {
                    projects
                        .iter()
                        .map(|(name, folder)| (key(name), reading.project(folder)))
                        .collect()
                })
                .unwrap_or_default();
            let _ = tx.send(Message::Backups {
                generation,
                reading,
                projects: backups,
            });
            ctx.request_repaint();
            let repositories: Vec<(String, PathBuf)> = projects
                .into_iter()
                .filter(|(_, folder)| folder.join(".git").exists())
                .collect();
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

    /// Runs the GitHub pass on a worker thread, after superseding any pass still running: the
    /// cached readings at once, then the network when the account's reading is older than a
    /// day, or always when `force` (F5). Every reading fetched is cached with its time.
    pub fn read_online(&mut self, force: bool) {
        let generation = self.online.fetch_add(1, Atomic::SeqCst) + 1;
        let (store, tx, ctx) = (self.store.clone(), self.tx.clone(), self.ctx.clone());
        let (latest, ledger, allowed) = (
            self.online.clone(),
            self.ledger.clone(),
            self.online_allowed,
        );
        self.online_progress = Some((0, 0));
        self.online_checked = Some(Instant::now());
        thread::spawn(move || {
            let mut problem = None;
            let ledger = ledger.and_then(|path| match Ledger::open(&path) {
                Ok(ledger) => Some(ledger),
                Err(e) => {
                    problem = Some(format!(
                        "The ledger could not be opened, so nothing is cached: {e}"
                    ));
                    None
                }
            });
            let mut current = false;
            for row in ledger
                .as_ref()
                .and_then(|ledger| ledger.cached().ok())
                .unwrap_or_default()
            {
                if let Some(reading) = Reading::from_cache(&row.project, &row.source, &row.data) {
                    current |= row.source == online::ACCOUNT
                        && Utc::now() - row.fetched_at < online::FRESH_FOR;
                    let _ = tx.send(Message::Online {
                        generation,
                        reading,
                        at: row.fetched_at,
                        progress: None,
                    });
                }
            }
            ctx.request_repaint();
            if allowed && (force || !current) {
                let projects = online_projects(&store);
                let of = projects.iter().filter(|p| p.remote.is_some()).count();
                let mut done = 0;
                online::fetch(&projects, |reading| {
                    if latest.load(Atomic::SeqCst) != generation {
                        return false;
                    }
                    let at = Utc::now();
                    if let (Some(ledger), Some((project, source, json))) =
                        (&ledger, reading.to_cache())
                        && let Err(e) = ledger.cache(&project, source, at, &json)
                    {
                        problem = Some(format!("A GitHub reading could not be cached: {e}"));
                    }
                    if matches!(reading, Reading::Head { .. }) {
                        done += 1;
                    }
                    let _ = tx.send(Message::Online {
                        generation,
                        reading,
                        at,
                        progress: Some((done, of)),
                    });
                    ctx.request_repaint();
                    true
                });
            }
            let _ = tx.send(Message::OnlineDone {
                generation,
                problem,
            });
            ctx.request_repaint();
        });
    }

    /// Whether the hourly look at the GitHub cache is due (D-24's daily refresh).
    pub fn online_due(&self) -> bool {
        self.online_progress.is_none()
            && self
                .online_checked
                .is_none_or(|at| at.elapsed() >= ONLINE_CHECK_EVERY)
    }

    /// Whether a read is under way.
    pub fn busy(&self) -> bool {
        self.reading_files || self.git_progress.is_some() || self.online_progress.is_some()
    }

    /// Takes in what the threads sent: a newer portfolio (with the other passes' readings so
    /// far), one repository's reading, the backup job's, or a GitHub reading. Returns whether the
    /// data root's folders changed.
    pub fn receive(&mut self, portfolio: &mut Portfolio) -> bool {
        let mut folders = false;
        let mut changed = false;
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
                    changed = true;
                }
                Message::Backups {
                    generation,
                    reading,
                    projects,
                } if generation == self.git.load(Atomic::SeqCst) => {
                    self.backups = reading.map(|reading| (reading, projects));
                    changed = true;
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
                Message::Online {
                    generation,
                    reading,
                    at,
                    progress,
                } if generation == self.online.load(Atomic::SeqCst) => {
                    self.fetched.put(reading, at);
                    if progress.is_some() {
                        self.online_progress = progress;
                    }
                    changed = true;
                }
                Message::OnlineDone {
                    generation,
                    problem,
                } if generation == self.online.load(Atomic::SeqCst) => {
                    self.online_progress = None;
                    self.online_problem = problem;
                }
                Message::Folders => folders = true,
                // A superseded read's result.
                Message::Files { .. }
                | Message::Backups { .. }
                | Message::Git { .. }
                | Message::GitDone { .. }
                | Message::Online { .. }
                | Message::OnlineDone { .. } => {}
            }
        }
        if changed {
            self.apply(portfolio);
        }
        folders
    }

    /// Puts the backup and GitHub readings into the portfolio, and the hosting each row's remote
    /// and the account's list give.
    fn apply(&self, portfolio: &mut Portfolio) {
        portfolio.account = self.fetched.account.clone();
        portfolio.backup = self.backups.as_ref().map(|(reading, _)| reading.clone());
        let account = portfolio
            .account
            .as_ref()
            .and_then(|fetched| fetched.value.as_ref().ok())
            .map(Vec::as_slice);
        for row in &mut portfolio.projects {
            let name = key(row.name());
            row.badges.backup = self
                .backups
                .as_ref()
                .and_then(|(_, projects)| projects.get(&name).cloned());
            row.badges.head = self.fetched.heads.get(&name).cloned();
            row.badges.run = self.fetched.runs.get(&name).cloned();
            let folder = row.folder_name();
            row.badges.hosting = hosting::classify(
                &[row.name(), &folder],
                row.badges.remote.as_deref(),
                account,
            );
        }
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

/// The projects the GitHub pass asks about: every registered folder, with its origin and whether
/// it has workflows.
fn online_projects(store: &Store) -> Vec<online::Project> {
    let Ok(loaded) = store.load() else {
        return Vec::new();
    };
    let root = loaded.registry.data_root();
    loaded
        .registry
        .projects
        .iter()
        .map(|project| {
            let folder = project.folder(&root);
            online::Project {
                name: project.name.clone(),
                remote: git::remote_url(&folder, "origin"),
                workflows: health::workflows(&folder) > 0,
                folder,
            }
        })
        .collect()
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

    pub fn hint(self) -> &'static str {
        match self {
            SortKey::Priority => "1 first",
            SortKey::Completion => "the most complete first",
            SortKey::Remaining => "the fewest open rows first",
            SortKey::LastActivity => "the latest commit first",
            SortKey::Hosting => "public, private, other remotes, then local only",
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

/// What the list keeps (D-25): the names that contain `name`, case aside, and when stacks or
/// hosting classes are chosen, the projects with one of the chosen stacks and one of the chosen
/// classes; unadopted folders, which have no badges, only while none are chosen.
#[derive(Clone, Copy, Debug)]
pub struct Filter<'a> {
    pub name: &'a str,
    pub stacks: &'a BTreeSet<Kind>,
    pub hosting: &'a BTreeSet<Class>,
}

impl Filter<'_> {
    fn by_badges(&self) -> bool {
        !self.stacks.is_empty() || !self.hosting.is_empty()
    }

    fn named(&self, name: &str) -> bool {
        let wanted = self.name.trim().to_lowercase();
        wanted.is_empty() || name.to_lowercase().contains(&wanted)
    }

    fn keeps(&self, row: &ProjectRow) -> bool {
        self.named(row.name())
            && (self.stacks.is_empty()
                || row
                    .badges
                    .stack
                    .iter()
                    .any(|chip| self.stacks.contains(&chip.kind)))
            && (self.hosting.is_empty()
                || row
                    .hosting_class()
                    .is_some_and(|class| self.hosting.contains(&class)))
    }
}

impl Portfolio {
    /// The list's groups in [`GroupKind::ORDER`], empty ones left out; inside each, pinned
    /// projects first, then `sort`, then the name; only what `filter` keeps.
    pub fn groups(&self, sort: SortKey, filter: Filter<'_>) -> Vec<Group> {
        GroupKind::ORDER
            .into_iter()
            .filter_map(|kind| {
                let members: Vec<Member> = match kind {
                    GroupKind::State(state) => {
                        let mut rows: Vec<usize> = (0..self.projects.len())
                            .filter(|&i| {
                                let row = &self.projects[i];
                                row.project.state == state && filter.keeps(row)
                            })
                            .collect();
                        rows.sort_by(|&a, &b| compare(&self.projects[a], &self.projects[b], sort));
                        rows.into_iter().map(Member::Project).collect()
                    }
                    GroupKind::Unadopted if filter.by_badges() => Vec::new(),
                    GroupKind::Unadopted => (0..self.unadopted.len())
                        .filter(|&i| filter.named(&self.unadopted[i].name))
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
        filter: Filter<'_>,
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
    fn last_if_none<T: Ord>(a: Option<T>, b: Option<T>) -> Ordering {
        match (a, b) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
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
            SortKey::Hosting => last_if_none(
                a.hosting_class().map(|c| c as usize),
                b.hosting_class().map(|c| c as usize),
            ),
            // The largest share first (EP-18b): the negated tenths keep "last if none".
            SortKey::UsageShare => {
                last_if_none(a.usage_tenths().map(|t| -t), b.usage_tenths().map(|t| -t))
            }
            SortKey::Name => Ordering::Equal,
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
pub(crate) mod tests {
    use std::fs;

    use chrono::TimeDelta;
    use crw_core::registry::Defaults;
    use crw_core::roadmap::EpId;

    use super::*;

    static NO_STACKS: BTreeSet<Kind> = BTreeSet::new();
    static NO_CLASSES: BTreeSet<Class> = BTreeSet::new();

    impl<'a> Filter<'a> {
        /// The name filter alone.
        fn name(name: &'a str) -> Self {
            Self {
                name,
                stacks: &NO_STACKS,
                hosting: &NO_CLASSES,
            }
        }
    }

    /// A folder under the system temp folder, removed when dropped.
    pub(crate) struct Scratch(pub(crate) PathBuf);

    impl Scratch {
        pub(crate) fn new(name: &str) -> Self {
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
    pub(crate) fn fake_store(scratch: &Scratch) -> Store {
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
        let groups = portfolio.groups(SortKey::Priority, Filter::name(""));
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
        assert!(quarry.proposes(now));
        assert!(quarry.in_progress().is_empty());
        // Dismissed in the run's registry, the gate stays out of the bulb until its brief
        // changes (D-6 addendum).
        let reason = quarry.proposals(now)[0].reason;
        store
            .change("2026-11-02", |c| c.dismiss("quarry", &reason))
            .unwrap();
        let again = read_files(&store);
        let quarry = again.project("quarry").unwrap();
        assert!(!quarry.proposes(now));
        assert!(quarry.proposals(now)[0].dismissed);
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
            let groups = portfolio.groups(sort, Filter::name(filter));
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
        let groups = portfolio.groups(SortKey::Name, Filter::name("AR"));
        let kept: Vec<Vec<String>> = groups.iter().map(|g| names(&portfolio, g)).collect();
        assert_eq!(
            kept,
            [
                vec!["harbor".to_owned(), "quarry".to_owned()],
                vec!["orchard".to_owned()]
            ]
        );
        assert!(
            portfolio
                .groups(SortKey::Name, Filter::name("nothing"))
                .is_empty()
        );
    }

    #[test]
    fn the_usage_share_sort_puts_the_largest_share_first() {
        let scratch = Scratch::new("portfolio-usage");
        let store = fake_store(&scratch);
        let mut portfolio = read_files(&store);
        let since = Utc::now() - TimeDelta::days(2);
        // lantern's raw figure is the larger, but both write `6`: the name decides. quarry's
        // `0` is a share all the same, so meadow, which has none, comes last.
        for (name, weekly) in [("harbor", 6.0), ("lantern", 6.04), ("quarry", 0.0)] {
            let row = portfolio
                .projects
                .iter_mut()
                .find(|row| row.name() == name)
                .unwrap();
            row.badges.usage = Some(Usage {
                weekly: Some(weekly),
                fable: None,
                sessions: 1,
                since,
                attributed: 12.04,
            });
        }
        let active = |portfolio: &Portfolio| {
            let groups = portfolio.groups(SortKey::UsageShare, Filter::name(""));
            names(portfolio, &groups[0])
        };
        // harbor is pinned in the fixture, so it leads whatever its share.
        portfolio.projects[0].project.pinned = false;
        assert_eq!(
            active(&portfolio),
            ["harbor", "lantern", "quarry", "meadow"]
        );
        let harbor = &mut portfolio.projects[0];
        harbor.badges.usage = harbor.badges.usage.map(|usage| Usage {
            weekly: Some(0.5),
            ..usage
        });
        assert_eq!(
            active(&portfolio),
            ["lantern", "harbor", "quarry", "meadow"]
        );
        // A share of Fable alone is no weekly share.
        let harbor = &mut portfolio.projects[0];
        harbor.badges.usage = harbor.badges.usage.map(|usage| Usage {
            weekly: None,
            fable: Some(3.0),
            ..usage
        });
        assert_eq!(
            active(&portfolio),
            ["lantern", "quarry", "harbor", "meadow"]
        );
    }

    fn listed(name: &str, visibility: hosting::Visibility) -> Hosting {
        Hosting::Listed(Listed {
            repo: hosting::Repo {
                owner: "example".to_owned(),
                name: name.to_owned(),
            },
            visibility,
            archived: false,
            pushed_at: None,
            default_branch: Some("main".to_owned()),
        })
    }

    #[test]
    fn the_hosting_sort_and_the_badge_filters() {
        let scratch = Scratch::new("portfolio-badges");
        let store = fake_store(&scratch);
        let mut portfolio = read_files(&store);
        for (name, hosting) in [
            ("harbor", listed("harbor", hosting::Visibility::Public)),
            ("lantern", listed("lantern", hosting::Visibility::Private)),
            ("meadow", listed("meadow", hosting::Visibility::Public)),
        ] {
            let row = portfolio
                .projects
                .iter_mut()
                .find(|row| row.name() == name)
                .unwrap();
            row.badges.hosting = hosting;
        }
        // The fixtures' manifests give the stacks.
        let harbor = portfolio.project("harbor").unwrap();
        let kinds: Vec<Kind> = harbor.badges.stack.iter().map(|chip| chip.kind).collect();
        assert_eq!(kinds, [Kind::Rust, Kind::PowerShell]);
        assert_eq!(harbor.badges.license.as_deref(), Some("LICENSE"));

        let shown = |portfolio: &Portfolio, sort, filter: Filter<'_>| -> Vec<Vec<String>> {
            portfolio
                .groups(sort, filter)
                .iter()
                .map(|group| names(portfolio, group))
                .collect()
        };
        // Public, private, then local; pinned harbor still first.
        assert_eq!(
            shown(&portfolio, SortKey::Hosting, Filter::name(""))[0],
            ["harbor", "meadow", "lantern", "quarry"]
        );
        let rust = BTreeSet::from([Kind::Rust]);
        let public = BTreeSet::from([Class::Public]);
        let by_stack = Filter {
            stacks: &rust,
            ..Filter::name("")
        };
        // Rust only: no unadopted folder, which has no badges.
        assert_eq!(
            shown(&portfolio, SortKey::Name, by_stack),
            [vec!["harbor", "quarry"], vec!["beacon"]]
        );
        let both = Filter {
            hosting: &public,
            ..by_stack
        };
        assert_eq!(shown(&portfolio, SortKey::Name, both), [vec!["harbor"]]);
        let local = BTreeSet::from([Class::Local]);
        let local_only = Filter {
            hosting: &local,
            ..Filter::name("")
        };
        let kept = shown(&portfolio, SortKey::Name, local_only);
        assert!(
            kept.iter()
                .flatten()
                .all(|name| !["harbor", "lantern", "meadow"].contains(&name.as_str()))
        );
        // The missing folder has no hosting at all.
        assert!(!kept.iter().flatten().any(|name| name == "tundra"));
    }

    #[test]
    fn the_newer_github_reading_wins() {
        let mut online = Online::default();
        let now = Utc::now();
        let run = |conclusion: &str| {
            Ok(Some(Run {
                workflow: "ci".to_owned(),
                status: "completed".to_owned(),
                conclusion: Some(conclusion.to_owned()),
                created_at: None,
                head_sha: None,
            }))
        };
        online.put(
            Reading::Run {
                project: "Harbor".to_owned(),
                run: run("failure"),
            },
            now,
        );
        // The cache, read again, holds an older reading: it does not replace this session's.
        online.put(
            Reading::Run {
                project: "harbor".to_owned(),
                run: run("success"),
            },
            now - chrono::TimeDelta::hours(5),
        );
        let kept = &online.runs["harbor"];
        assert_eq!(kept.at, now);
        assert_eq!(
            kept.value
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .conclusion
                .as_deref(),
            Some("failure")
        );
        online.put(Reading::Account(Err("offline".to_owned())), now);
        assert!(online.account.unwrap().value.is_err());
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
        let order = portfolio.order(SortKey::Priority, Filter::name(""), folded);
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
        // The same status child gives the activity badge: the changed README, no upstream.
        let activity = row.badges.activity.as_ref().unwrap();
        assert_eq!(activity.changed, 1);
        assert_eq!(activity.upstream, None);
        assert_eq!(activity.oid.as_ref().map(String::len), Some(40));
        assert_eq!(row.badges.remote, None);
        assert_eq!(row.badges.hosting, Hosting::Local);
    }
}
