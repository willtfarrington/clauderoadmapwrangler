//! The window's mailman (D-13; EP-20): a round of `crw_core::inbox::mailman` on a worker thread,
//! once the poll of live sessions has answered and the files are read, then after each of the
//! window's runs ends, when the watch records a session's start or end, and every five minutes.
//! A receiver where a session works or waits gets nothing until it is free. A screenshot run, a
//! test and a window without a ledger deliver nothing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use crw_core::inbox::mailman::{self, Mark, Report, Round, Sender};
use crw_core::inbox::{self, Outgoing};
use crw_core::ledger::Ledger;
use crw_core::registry::Registry;
use crw_core::roadmap::snapshot;
use crw_core::sessions::Status;
use crw_core::{files, git, process};

use crate::board::Board;
use crate::portfolio::Portfolio;
use crate::server::Listening;

/// A round at least this often while the window runs.
const EVERY: Duration = Duration::from_secs(300);

/// The mailman's state in the window.
#[derive(Default)]
pub struct Mail {
    /// A round is asked for.
    pub due: bool,
    last: Option<Instant>,
    running: Option<Receiver<Report>>,
    /// The window's runs seen ended, so that a new end asks for a round.
    ended: usize,
    /// The latest round's marks, each block's state.
    pub marks: Vec<Mark>,
}

impl Mail {
    /// Sets each row's marks from the latest round's, again after each files read (EP-18b's
    /// pattern for the week's usage).
    pub fn apply(&self, portfolio: &mut Portfolio) {
        for row in &mut portfolio.projects {
            let name = row.project.name.clone();
            row.mail = self
                .marks
                .iter()
                .filter(|mark| mark.sender.eq_ignore_ascii_case(&name))
                .cloned()
                .collect();
        }
    }
}

impl Board {
    /// The mailman's share of each pass: a finished round taken in, a new one started when due.
    pub fn mail_pass(&mut self) {
        if let Some(rx) = &self.mail.running {
            match rx.try_recv() {
                Ok(report) => {
                    self.mail.running = None;
                    self.mail_report(report);
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => self.mail.running = None,
            }
        }
        self.mail.apply(&mut self.portfolio);
        if self.fixtures {
            return;
        }
        let Some(ledger) = self.ledger().map(Path::to_path_buf) else {
            return;
        };
        let ended = self
            .sessions
            .live
            .iter()
            .filter(|live| live.ended.is_some())
            .count();
        if ended > self.mail.ended {
            self.mail.due = true;
        }
        self.mail.ended = ended;
        if self.mail.last.is_none_or(|at| at.elapsed() >= EVERY) {
            self.mail.due = true;
        }
        // The poll must have answered, so that a session at work in a receiver is known; and the
        // window must hold the loopback port, where the hooks arrive, so that a second window (no
        // single instance until EP-28) neither misses a session at work nor sends a letter twice.
        let polled = self.watch.model.polled.is_some();
        let serving = matches!(self.watch.listening, Some(Listening::On(_)));
        let read = !self.loader.reading_files && !self.portfolio.projects.is_empty();
        // The urgent messages' first look comes first, so that an urgent letter this window
        // delivers is news to it, not part of what it takes as seen (EP-20a's review).
        let looked = self.toasts.urgent_looked();
        if !self.mail.due || self.mail.running.is_some() || !polled || !serving || !read || !looked
        {
            return;
        }
        // Taken as done before the registry is read, so a registry that does not read is tried
        // again at the next due round, not at every frame.
        self.mail.due = false;
        self.mail.last = Some(Instant::now());
        let Ok(loaded) = self.store().load() else {
            return;
        };
        let busy = self.busy_projects();
        let backups = self.store().backups.clone();
        let (tx, rx) = mpsc::channel();
        let ctx = self.ctx().clone();
        thread::spawn(move || {
            let report = round(&loaded.registry, &ledger, &busy, &backups);
            let _ = tx.send(report);
            ctx.request_repaint();
        });
        self.mail.running = Some(rx);
    }

    /// The projects where a session works or waits, by lowercase name, with the words.
    fn busy_projects(&self) -> HashMap<String, String> {
        let mut busy = HashMap::new();
        for seen in self.watch.model.all() {
            let Some(project) = &seen.project else {
                continue;
            };
            let words = match seen.status {
                Status::Busy => "a session works there",
                Status::Waiting => "a session there waits for you",
                _ => continue,
            };
            busy.entry(project.to_lowercase())
                .or_insert_with(|| words.to_owned());
        }
        for live in self.sessions.live.iter().filter(|live| live.is_running()) {
            busy.entry(live.project.to_lowercase())
                .or_insert_with(|| "a headless run of the window's works there".to_owned());
        }
        // A revert open or running there would put the inbox back under the letter.
        if let Some(dialog) = &self.revert {
            busy.entry(dialog.project.to_lowercase())
                .or_insert_with(|| "a revert is open there".to_owned());
        }
        busy
    }

    fn mail_report(&mut self, report: Report) {
        self.mail.marks = report.marks;
        if !report.delivered.is_empty() {
            self.notify(format!("Delivered {}.", report.delivered.join("; ")));
            // The receiver's row shows its new message.
            self.loader.read_files();
        }
        if let Some(problem) = report.problems.first() {
            self.warn(format!("The mailman: {problem}"));
        }
    }
}

/// One round over every registered project whose folder is there (git asked only of a sender
/// with a letter to send, in its turn, D-29).
fn round(
    registry: &Registry,
    ledger: &Path,
    busy: &HashMap<String, String>,
    backups: &Path,
) -> Report {
    let ledger = match Ledger::open(ledger) {
        Ok(ledger) => ledger,
        Err(e) => {
            return Report {
                problems: vec![format!("the ledger did not open: {e}")],
                ..Report::default()
            };
        }
    };
    let root = registry.data_root();
    let projects: Vec<(String, PathBuf, Vec<Outgoing>)> = registry
        .projects
        .iter()
        .filter_map(|project| {
            let folder = project.folder(&root);
            folder.is_dir().then(|| {
                let snapshot = snapshot::read_tree(&folder);
                let outgoing = inbox::outgoing(&snapshot.briefs, |name| {
                    registry.project(name).map(|known| known.name.clone())
                });
                (project.name.clone(), folder, outgoing)
            })
        })
        .collect();
    let senders: Vec<Sender<'_>> = projects
        .iter()
        .map(|(name, folder, outgoing)| Sender {
            name,
            folder,
            outgoing,
        })
        .collect();
    let folder_of = |name: &str| {
        registry
            .project(name)
            .map(|project| project.folder(&root))
            .filter(|folder| folder.is_dir())
    };
    let mut changes = |folder: &Path| {
        // Its own repository only, as the git pass asks: in a folder without one, git would answer
        // for an enclosing repository, whose paths never name the brief, so all would seem
        // committed.
        if !folder.join(".git").exists() {
            return Err("the folder is not a git repository".to_owned());
        }
        let _turn = process::turn();
        let status = git::status(folder);
        thread::sleep(process::SPACING);
        status.map(|status| {
            status
                .changes
                .into_iter()
                .map(|change| change.path)
                .collect()
        })
    };
    let busy_of = |name: &str| busy.get(&name.to_lowercase()).cloned();
    let today = files::today();
    let mut cx = Round {
        folder_of: &folder_of,
        changes: &mut changes,
        busy: &busy_of,
        backups,
        today: &today,
        now: Utc::now(),
    };
    mailman::round(&ledger, &senders, &mut cx)
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use crw_core::inbox::mailman::State;
    use crw_core::sessions::{Mode, Seen};

    use super::*;
    use crate::board::{Places, View};
    use crate::portfolio::tests::{Scratch, fake_store};

    fn board(scratch: &Scratch) -> Board {
        let places = Places {
            store: fake_store(scratch),
            ledger: Some(scratch.0.join("ledger.sqlite")),
            online: false,
            crash_log: None,
        };
        let ctx = eframe::egui::Context::default();
        Board::new(
            places,
            &ctx,
            View::default(),
            eframe::egui::ThemePreference::Light,
        )
    }

    #[test]
    fn a_receiver_at_work_waits_and_a_screenshot_run_sends_nothing() {
        let scratch = Scratch::new("mail-busy");
        let mut board = board(&scratch);
        let now = Utc::now();
        let canned = |id, project, status| {
            let mut seen = Seen::canned(id, Some(project), Mode::Visual, status, None, now);
            seen.heard = now - TimeDelta::seconds(1);
            seen
        };
        board
            .watch
            .model
            .insert(canned("a1", "harbor", Status::Busy));
        board
            .watch
            .model
            .insert(canned("b2", "quarry", Status::Waiting));
        board
            .watch
            .model
            .insert(canned("c3", "meadow", Status::Idle));
        let busy = board.busy_projects();
        assert_eq!(
            busy.get("harbor").map(String::as_str),
            Some("a session works there")
        );
        assert_eq!(
            busy.get("quarry").map(String::as_str),
            Some("a session there waits for you")
        );
        assert!(
            !busy.contains_key("meadow"),
            "an idle tab does not hold the mail"
        );
        // Everything else ready: the files read, a poll answered, the port held.
        board.portfolio = crate::portfolio::read_files(board.store());
        board.loader.reading_files = false;
        board.watch.model.polled = Some(now);
        board.watch.listening = Some(Listening::On(1));
        // A screenshot run never starts a round, whatever is due.
        board.fixtures = true;
        board.mail.due = true;
        board.mail_pass();
        assert!(board.mail.running.is_none());
        // Without the port (a second window), no round either.
        board.fixtures = false;
        board.watch.listening = Some(Listening::Failed("taken".to_owned()));
        board.mail_pass();
        assert!(board.mail.running.is_none());
        // Without a poll, no round, so that a session at work is never missed.
        board.watch.listening = Some(Listening::On(1));
        board.watch.model.polled = None;
        board.mail_pass();
        assert!(board.mail.running.is_none());
        // With the notifications on, no round before the urgent messages' first look (EP-20a's
        // review); the look lets it go.
        board.watch.model.polled = Some(now);
        board.toasts.start(None);
        board.mail_pass();
        assert!(board.mail.running.is_none());
        assert!(
            board.toasts.urgent(&board.portfolio).is_empty(),
            "the first look"
        );
        assert!(board.toasts.urgent_looked());
        // With all of it, a round over the fake portfolio. Its folders hold no repository of
        // their own, so git is never asked for them and nothing is written into the tracked
        // fixtures: lantern's letter to harbor waits.
        let harbor = board.portfolio.project("harbor").unwrap().path.clone();
        let inbox_before = std::fs::read_to_string(inbox::path(&harbor)).unwrap();
        board.watch.model.polled = Some(now);
        board.mail_pass();
        assert!(board.mail.running.is_some());
        let started = Instant::now();
        while board.mail.running.is_some() && started.elapsed() < Duration::from_secs(20) {
            thread::sleep(Duration::from_millis(20));
            board.mail_pass();
        }
        let lantern = board.portfolio.project("lantern").unwrap();
        assert!(
            lantern.mail.iter().any(|mark| matches!(
                &mark.state,
                State::Waits(why) if why.contains("not a git repository")
            )),
            "{:?}",
            lantern.mail
        );
        assert_eq!(
            std::fs::read_to_string(inbox::path(&harbor)).unwrap(),
            inbox_before
        );
    }

    #[test]
    fn each_row_takes_its_own_marks() {
        let scratch = Scratch::new("mail-marks");
        let mut board = board(&scratch);
        board.portfolio = crate::portfolio::read_files(board.store());
        let mark = |sender: &str, state| Mark {
            sender: sender.to_owned(),
            path: PathBuf::from("EP-6-export.md"),
            line: 5,
            state,
        };
        board.mail.marks = vec![
            mark("Harbor", State::Counted),
            mark(
                "quarry",
                State::Waits("its brief is not committed yet".to_owned()),
            ),
        ];
        board.mail.apply(&mut board.portfolio);
        let marks = |name: &str| board.portfolio.project(name).map(|row| row.mail.len());
        assert_eq!(marks("harbor"), Some(1));
        assert_eq!(marks("quarry"), Some(1));
        assert_eq!(marks("meadow"), Some(0));
    }
}
