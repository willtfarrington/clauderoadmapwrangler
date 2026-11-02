//! The sessions the window sees besides its own (D-20; EP-12): what the loopback server hears from
//! the hooks and the status lines, a poll of `claude agents --json` (every 5 s while the window has
//! the focus, every 30 s otherwise: one listing costs about 0.2 s of a core, and the hooks bring a
//! wait or an end at once), and the state model they feed (`crw_core::sessions`), whose starts and
//! ends the ledger records. A screenshot run starts neither the server nor the poll: its sessions
//! and its readings are canned (rule 5).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use crw_core::ledger::Ledger;
use crw_core::sessions::agents::{self, Agent};
use crw_core::sessions::{self, Change, Registered, Seen, Sessions};
use crw_core::signals::{Signal, install};
use eframe::egui;

use crate::server::{Listening, Server};

/// How often the live sessions are asked for while the window has the focus, and while it has not.
const POLL_FOCUSED: Duration = Duration::from_secs(5);
const POLL_UNFOCUSED: Duration = Duration::from_secs(30);

/// How often the poll's thread looks at the clock and the focus.
const STEP: Duration = Duration::from_millis(250);

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
}

impl Watch {
    /// Starts the server on `port` and the poll of `program`'s live sessions; reads the entries.
    pub fn start(&mut self, port: u16, program: PathBuf, ctx: &egui::Context) {
        let wake = ctx.clone();
        let server = Server::start(port, move || wake.request_repaint());
        self.listening = Some(server.listening.clone());
        self.server = Some(server);
        self.poller = poll(program, ctx.clone());
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
                    Signal::Hook(hook) => {
                        let change = self.model.hook(&hook, &projects, now);
                        self.record(&change, ledger);
                    }
                    Signal::StatusLine(line) => self.model.status_line(&line, &projects, now),
                }
            }
            for listing in listings {
                match listing {
                    Ok(agents) => self.model.poll(&agents, &projects, now),
                    Err(why) => self.model.poll_failed(why),
                }
            }
        }
        self.model.expire(now);
    }

    /// Keeps a start or an end in the ledger, and the mode the ledger gives the session.
    fn record(&mut self, change: &Change, ledger: Option<&Path>) {
        let (Some(path), false) = (ledger, *change == Change::Nothing) else {
            return;
        };
        match Ledger::open(path).and_then(|ledger| sessions::record(&ledger, change)) {
            Ok(mode) => {
                self.problem = None;
                if let (Some(mode), Change::Started { session_id, .. }) = (mode, change) {
                    self.model.set_mode(session_id, mode);
                }
            }
            Err(e) => self.problem = Some(format!("The ledger did not take a session: {e}")),
        }
    }

    /// The sessions that run in the project called `name` and are not the window's own.
    pub fn running<'a>(
        &'a self,
        name: &'a str,
        own: impl Fn(&str) -> bool + 'a,
    ) -> impl Iterator<Item = &'a Seen> + 'a {
        self.model
            .of_project(name)
            .into_iter()
            .filter(move |seen| !seen.status.is_over() && !own(&seen.session_id))
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
