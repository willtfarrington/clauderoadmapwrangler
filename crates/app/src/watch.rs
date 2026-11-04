//! The sessions the window sees besides its own (D-20; EP-12): what the loopback server hears from
//! the hooks and the status lines, a poll of `claude agents --json` (every 5 s while the window has
//! the focus, every 30 s otherwise: one listing costs about 0.2 s of a core, and the hooks bring a
//! wait or an end at once), and the state model they feed (`crw_core::sessions`), whose working
//! sessions and ends the ledger records (EP-13: a row at a session's first sign of work). Once a
//! minute the visual launches no session took up within their 30 minutes lapse. A session's
//! start hook in a registered project has the project's start read for a revert (EP-15b), on a
//! worker thread, and kept until the session earns its row. A screenshot run starts neither the
//! server nor the poll: its sessions and its readings are canned (rule 5).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use crw_core::ledger::Ledger;
use crw_core::process;
use crw_core::revert::{self, Start};
use crw_core::sessions::agents::{self, Agent};
use crw_core::sessions::{self, Change, Registered, Seen, Sessions, VISUAL_CLAIM};
use crw_core::signals::{Hook, Signal, install};
use eframe::egui;

use crate::server::{Listening, Server};

/// How often the live sessions are asked for while the window has the focus, and while it has not.
const POLL_FOCUSED: Duration = Duration::from_secs(5);
const POLL_UNFOCUSED: Duration = Duration::from_secs(30);

/// How often the poll's thread looks at the clock and the focus.
const STEP: Duration = Duration::from_millis(250);

/// How often the visual launches past their window are closed (EP-13).
const LAPSE_EVERY: Duration = Duration::from_secs(60);

type Listing = Result<Vec<Agent>, String>;

/// The poll's thread: its listings, and the focus it paces itself by.
struct Poller {
    listings: Receiver<Listing>,
    focused: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct Watch {
    pub model: Sessions,
    server: Option<Server>,
    poller: Option<Poller>,
    /// Where the server stands; none before it starts (a screenshot run's is canned).
    pub listening: Option<Listening>,
    /// The tool's entries in Claude Code's user settings, in words, or why they did not read.
    pub entries: Option<Result<String, String>>,
    /// A ledger write that failed, until the next one succeeds.
    pub problem: Option<String>,
    /// The ledger changed since the Sessions pane last read it (EP-13).
    pub recorded: bool,
    lapsed_at: Option<Instant>,
    starts: Starts,
}

/// The starts of the sessions the hooks see begin (EP-15b): read on a worker thread, one git child
/// in its turn (D-29), and kept until the session earns its row (EP-13), then written into it.
#[derive(Default)]
struct Starts {
    /// The backups folder the copies go to; none until the watch starts (never in a test or a
    /// screenshot run, unless set).
    backups: Option<PathBuf>,
    ctx: Option<egui::Context>,
    worker: Option<Worker>,
    /// Being read, by session id.
    asked: HashSet<String>,
    /// Read, waiting for the session's row.
    pending: HashMap<String, Start>,
    /// The sessions that have their row: a start read after it goes into the row.
    rowed: HashSet<String>,
}

/// A start to read: the session, its folder, where its copies go and the ledger that may hold it
/// already (a session resumed in VS Code keeps its first start).
struct Capture {
    session_id: String,
    folder: PathBuf,
    copies: PathBuf,
    ledger: Option<PathBuf>,
    at: DateTime<Utc>,
}

/// What the worker read: none when the ledger holds the session's start already.
type Captured = (String, Option<Result<Start, String>>);

struct Worker {
    orders: Sender<Capture>,
    results: Receiver<Captured>,
}

impl Watch {
    /// Starts the server on `port` and the poll of `program`'s live sessions; reads the entries.
    /// The sessions' starts are copied into `backups` (EP-15b).
    pub fn start(&mut self, port: u16, program: PathBuf, backups: PathBuf, ctx: &egui::Context) {
        let wake = ctx.clone();
        let server = Server::start(port, move || wake.request_repaint());
        self.listening = Some(server.listening.clone());
        self.server = Some(server);
        self.poller = poll(program, ctx.clone());
        self.starts.backups = Some(backups);
        self.starts.ctx = Some(ctx.clone());
        self.read_entries();
    }

    /// Reads what Claude Code's user settings hold of the tool's (at the start and on F5).
    pub fn read_entries(&mut self) {
        let Some(settings) = install::settings_file() else {
            self.entries = Some(Err("no profile folder".to_owned()));
            return;
        };
        let exe = std::env::current_exe()
            .map(|exe| exe.with_file_name("crw.exe"))
            .unwrap_or_default();
        self.entries = Some(
            install::status(&settings)
                .map(|status| status.summary(&exe))
                .map_err(|e| e.to_string()),
        );
    }

    /// Takes in what the server heard and the poll listed since the last frame, records the
    /// starts and ends in `ledger`, and lets ended sessions go. `projects` is asked only when
    /// there is news.
    pub fn receive(
        &mut self,
        projects: impl FnOnce() -> Vec<Registered>,
        ledger: Option<&Path>,
        focused: bool,
    ) {
        let now = Utc::now();
        let mut signals = Vec::new();
        if let Some(server) = &mut self.server {
            signals = server.receive();
            self.listening = Some(server.listening.clone());
        }
        let mut listings = Vec::new();
        if let Some(poller) = &self.poller {
            poller.focused.store(focused, Ordering::Relaxed);
            listings.extend(poller.listings.try_iter());
        }
        if !signals.is_empty() || !listings.is_empty() {
            let projects = projects();
            for signal in signals {
                match signal {
                    Signal::Hook(hook) => self.take_hook(&hook, &projects, ledger, now),
                    Signal::StatusLine(line) => self.model.status_line(&line, &projects, now),
                }
            }
            for listing in listings {
                match listing {
                    Ok(agents) => {
                        for change in self.model.poll(&agents, &projects, now) {
                            self.record(&change, ledger);
                        }
                    }
                    Err(why) => self.model.poll_failed(why),
                }
            }
        }
        self.model.expire(now);
        self.take_starts(ledger);
        self.lapse(ledger);
    }

    /// Takes in one hook: the state model's change kept in the ledger, a start in a registered
    /// project with its switch on read for a revert (EP-15b), and the start of a session that
    /// ended without working dropped with its copies.
    fn take_hook(
        &mut self,
        hook: &Hook,
        projects: &[Registered],
        ledger: Option<&Path>,
        now: DateTime<Utc>,
    ) {
        let opens = hook.event == "SessionStart"
            && hook.source.as_deref() != Some("compact")
            && sessions::interactive(hook.entrypoint.as_deref());
        if opens
            && let Some(project) = sessions::registered_of(projects, &hook.cwd)
            && project.revert
        {
            self.ask_start(&hook.session_id, &project.folder, ledger, now);
        }
        let change = self.model.hook(hook, projects, now);
        self.record(&change, ledger);
        if hook.event == "SessionEnd"
            && self.model.get(&hook.session_id).is_none_or(Seen::is_new)
            && let Some(start) = self.starts.pending.remove(&hook.session_id)
        {
            start.discard();
        }
    }

    /// Asks the worker for the start of `session_id` in `folder`, once.
    fn ask_start(
        &mut self,
        session_id: &str,
        folder: &Path,
        ledger: Option<&Path>,
        at: DateTime<Utc>,
    ) {
        let starts = &mut self.starts;
        let Some(backups) = starts.backups.clone() else {
            return;
        };
        if session_id.is_empty()
            || starts.asked.contains(session_id)
            || starts.pending.contains_key(session_id)
            || starts.rowed.contains(session_id)
        {
            return;
        }
        if starts.worker.is_none() {
            starts.worker = start_worker(starts.ctx.clone());
        }
        let Some(worker) = &starts.worker else {
            return;
        };
        let capture = Capture {
            session_id: session_id.to_owned(),
            folder: folder.to_owned(),
            copies: revert::start::copies_dir(&backups, folder, session_id),
            ledger: ledger.map(Path::to_owned),
            at,
        };
        if worker.orders.send(capture).is_ok() {
            starts.asked.insert(session_id.to_owned());
        }
    }

    /// Takes in the starts the worker read: into the session's row when it has one, else kept
    /// until it earns it; the start of a session gone without working is dropped.
    fn take_starts(&mut self, ledger: Option<&Path>) {
        let results: Vec<Captured> = match &self.starts.worker {
            Some(worker) => worker.results.try_iter().collect(),
            None => Vec::new(),
        };
        for (session_id, result) in results {
            self.starts.asked.remove(&session_id);
            match result {
                Some(Ok(start)) if self.starts.rowed.contains(&session_id) => {
                    self.write_start(&session_id, &start, ledger);
                }
                // Kept for the row it may earn; gone, or ended without working, it keeps none.
                Some(Ok(start)) => {
                    let open = self
                        .model
                        .get(&session_id)
                        .is_some_and(|seen| !seen.status.is_over());
                    if open {
                        self.starts.pending.insert(session_id, start);
                    } else {
                        start.discard();
                    }
                }
                Some(Err(why)) => {
                    self.problem = Some(format!(
                        "A session's start was not recorded, so its revert would go back to the \
                         last recorded brief: {why}"
                    ));
                }
                None => {}
            }
        }
        // A session that left the window without a row keeps no start.
        let gone: Vec<String> = self
            .starts
            .pending
            .keys()
            .filter(|id| self.model.get(id).is_none())
            .cloned()
            .collect();
        for id in gone {
            if let Some(start) = self.starts.pending.remove(&id) {
                start.discard();
            }
        }
    }

    /// Writes a session's start into its first row, when that row has none.
    fn write_start(&mut self, session_id: &str, start: &Start, ledger: Option<&Path>) {
        let Some(path) = ledger else {
            return;
        };
        match Ledger::open(path)
            .and_then(|ledger| ledger.set_start_of(session_id, &start.to_json()))
        {
            Ok(_) => self.recorded = true,
            Err(e) => {
                self.problem = Some(format!("The ledger did not keep a session's start: {e}"))
            }
        }
    }

    /// Keeps a start or an end in the ledger, and the mode the ledger gives the session; a session
    /// that earns its row takes its start along (EP-15b).
    fn record(&mut self, change: &Change, ledger: Option<&Path>) {
        let (Some(path), false) = (ledger, *change == Change::Nothing) else {
            return;
        };
        match Ledger::open(path).and_then(|ledger| sessions::record(&ledger, change)) {
            Ok(mode) => {
                self.problem = None;
                self.recorded = true;
                if let (Some(mode), Change::Started { session_id, .. }) = (mode, change) {
                    self.model.set_mode(session_id, mode);
                    self.starts.rowed.insert(session_id.clone());
                    if let Some(start) = self.starts.pending.remove(session_id) {
                        self.write_start(session_id, &start, ledger);
                    }
                }
            }
            Err(e) => self.problem = Some(format!("The ledger did not take a session: {e}")),
        }
    }

    /// Closes the visual launches no session took up within their window, once a minute and at
    /// the window's first frame (EP-13).
    fn lapse(&mut self, ledger: Option<&Path>) {
        let Some(path) = ledger else {
            return;
        };
        if self.lapsed_at.is_some_and(|at| at.elapsed() < LAPSE_EVERY) {
            return;
        }
        self.lapsed_at = Some(Instant::now());
        match Ledger::open(path).and_then(|ledger| ledger.lapse_visual(Utc::now(), VISUAL_CLAIM)) {
            Ok(0) => {}
            Ok(_) => self.recorded = true,
            Err(e) => {
                self.problem = Some(format!("The ledger's visual launches did not lapse: {e}"));
            }
        }
    }

    /// The sessions that work in the project called `name` and are not the window's own: a
    /// session open with no prompt yet is none (EP-13).
    pub fn running<'a>(
        &'a self,
        name: &'a str,
        own: impl Fn(&str) -> bool + 'a,
    ) -> impl Iterator<Item = &'a Seen> + 'a {
        self.model
            .of_project(name)
            .into_iter()
            .filter(move |seen| !seen.status.is_over() && !seen.is_new() && !own(&seen.session_id))
    }

    /// The status bar's line on the signals, and whether it is a warning.
    pub fn line(&self) -> Option<(String, bool)> {
        match self.listening.as_ref()? {
            Listening::Starting(port) => Some((format!("Signals: starting on {port}"), false)),
            Listening::On(port) => {
                let entries = match &self.entries {
                    Some(Ok(words)) => words.clone(),
                    Some(Err(why)) => format!("the user settings did not read: {why}"),
                    None => "the user settings not read yet".to_owned(),
                };
                Some((
                    format!("Signals: listening on 127.0.0.1:{port}; {entries}"),
                    false,
                ))
            }
            Listening::Failed(why) => Some((
                format!(
                    "Signals: not listening ({why}): another window may hold the port; the \
                     sessions started elsewhere are seen through the poll only"
                ),
                true,
            )),
        }
    }
}

/// The thread that reads the sessions' starts in the order asked, each git child in its turn
/// (D-29), and passes over a session whose start the ledger holds already.
fn start_worker(ctx: Option<egui::Context>) -> Option<Worker> {
    let (orders, inbox) = mpsc::channel::<Capture>();
    let (sender, results) = mpsc::channel();
    thread::Builder::new()
        .name("session starts".to_owned())
        .spawn(move || {
            for capture in inbox {
                let known = capture
                    .ledger
                    .as_deref()
                    .filter(|path| path.exists())
                    .and_then(|path| Ledger::open_read_only(path).ok())
                    .and_then(|ledger| ledger.start_of(&capture.session_id).ok().flatten())
                    .is_some();
                let result = (!known).then(|| {
                    let _turn = process::turn();
                    revert::start::capture(&capture.folder, &capture.copies, capture.at)
                });
                if sender.send((capture.session_id, result)).is_err() {
                    return;
                }
                if let Some(ctx) = &ctx {
                    ctx.request_repaint();
                }
            }
        })
        .ok()?;
    Some(Worker { orders, results })
}

/// Asks `program` for the live sessions on a thread of its own, at once and then at the pace the
/// focus sets, until the window goes.
fn poll(program: PathBuf, ctx: egui::Context) -> Option<Poller> {
    let (sender, listings) = mpsc::channel();
    let focused = Arc::new(AtomicBool::new(true));
    let flag = Arc::clone(&focused);
    thread::Builder::new()
        .name("live sessions".to_owned())
        .spawn(move || {
            let mut last: Option<Instant> = None;
            loop {
                let pace = if flag.load(Ordering::Relaxed) {
                    POLL_FOCUSED
                } else {
                    POLL_UNFOCUSED
                };
                if last.is_some_and(|at| at.elapsed() < pace) {
                    thread::sleep(STEP);
                    continue;
                }
                last = Some(Instant::now());
                if sender.send(agents::ask(&program)).is_err() {
                    return;
                }
                ctx.request_repaint();
            }
        })
        .ok()?;
    Some(Poller { listings, focused })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crw_core::git;

    use super::*;
    use crate::portfolio::tests::Scratch;

    fn hook(event: &str, id: &str, cwd: &Path) -> Hook {
        Hook {
            event: event.to_owned(),
            session_id: id.to_owned(),
            cwd: cwd.to_owned(),
            source: (event == "SessionStart").then(|| "startup".to_owned()),
            entrypoint: Some("claude-vscode".to_owned()),
            ..Hook::default()
        }
    }

    /// Waits for the worker to read the start of `id`.
    fn read_start(watch: &mut Watch, id: &str, ledger: &Path) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while watch.starts.asked.contains(id) {
            assert!(Instant::now() < deadline, "no start came for {id}");
            thread::sleep(Duration::from_millis(50));
            watch.take_starts(Some(ledger));
        }
    }

    /// A VS Code session's start (EP-15b): read at its start hook, kept until the session works,
    /// then written into its row; one that ends without working drops it with its copies; a
    /// project switched off reads none. A scratch repository, never the owner's (rule 13).
    #[test]
    fn a_sessions_start_waits_for_its_row() {
        let scratch = Scratch::new("watch-starts");
        let folder = scratch.0.join("alpha");
        fs::create_dir_all(&folder).unwrap();
        {
            let _turn = process::turn();
            git::run(&folder, &["init", "--quiet", "-b", "main"]).unwrap();
        }
        fs::write(folder.join("draft.md"), "the owner's draft\n").unwrap();
        let ledger = scratch.0.join("ledger.sqlite");
        let backups = scratch.0.join("backups");
        let mut projects = vec![Registered {
            name: "alpha".to_owned(),
            folder: folder.clone(),
            revert: true,
        }];
        let mut watch = Watch::default();
        watch.starts.backups = Some(backups.clone());
        let now = Utc::now();

        watch.take_hook(
            &hook("SessionStart", "s-1", &folder),
            &projects,
            Some(&ledger),
            now,
        );
        read_start(&mut watch, "s-1", &ledger);
        assert!(watch.starts.pending.contains_key("s-1"), "kept: no row yet");
        // Its first finished turn earns its row, and the start goes in.
        watch.take_hook(&hook("Stop", "s-1", &folder), &projects, Some(&ledger), now);
        let read = Ledger::open_read_only(&ledger).unwrap();
        let start = Start::from_json(&read.start_of("s-1").unwrap().unwrap()).unwrap();
        assert_eq!(start.files.len(), 1);
        assert!(start.copy_of("draft.md").unwrap().is_file());
        assert!(watch.starts.pending.is_empty());

        // A tab that ends without working keeps no start, nor its copies.
        watch.take_hook(
            &hook("SessionStart", "s-2", &folder),
            &projects,
            Some(&ledger),
            now,
        );
        read_start(&mut watch, "s-2", &ledger);
        let copies = revert::start::copies_dir(&backups, &folder, "s-2");
        assert!(copies.is_dir());
        watch.take_hook(
            &hook("SessionEnd", "s-2", &folder),
            &projects,
            Some(&ledger),
            now,
        );
        assert!(!copies.exists());
        assert!(watch.starts.pending.is_empty());

        // Switched off, the project reads none.
        projects[0].revert = false;
        watch.take_hook(
            &hook("SessionStart", "s-3", &folder),
            &projects,
            Some(&ledger),
            now,
        );
        assert!(!watch.starts.asked.contains("s-3"));
    }
}
