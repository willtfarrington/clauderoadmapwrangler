//! The window's restart into the installed build (D-29; EP-37). The exes run from their installed
//! copy in the local folder's `bin` (`crw_core::install`). `crw install`, which the install script
//! runs after the build, copies a release build of the window's repository there and asks the
//! window over the loopback server to restart; File > Install and restart and the tray's item do
//! the same from the window, installing first a settled build of its repository newer than the
//! copy. The window then starts the installed window with its own process id (`--restarted
//! <pid>`), which waits for this one to end, so that the layout is saved and the port free, and
//! quits, unless the new process has ended already. A restart is refused, with every reason, while
//! something would be lost: a headless run of the window's that works, waits or is being stopped
//! (D-13), a session or a visual launch on its way, a revert, a gate being read or a chained
//! start, a held chain or a chain plan (which the handover of EP-37a will carry), a session's
//! start not yet in the ledger, a disposition being typed, an open dialog. A refused request is
//! not remembered: the status bar's chip stays, and File > Install and restart, or the tray's,
//! takes it once nothing stands in the way. A request that installed into another folder than the
//! window's (a trial's `CRW_HOME\bin`) is refused, and one whose caller stopped waiting is
//! dropped. Every 30 seconds, and every second while the File menu is open, the window looks
//! whether a newer build is built or installed, which the status bar then says. A screenshot run
//! restarts nothing.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use crw_core::install::{self, Running, Stamp};
use crw_core::paths;
use crw_core::signals::RestartAnswer;
use eframe::egui;

use crate::board::Board;
use crate::server::RestartRequest;

/// How often the window looks at the build and the installed copy.
const LOOK_EVERY: Duration = Duration::from_secs(30);
/// How often while the File menu is open.
pub const LOOK_OPEN: Duration = Duration::from_secs(1);
/// How long the window stays after a restart is decided, so that the loopback's answer goes out
/// before the process ends.
const ANSWER_GRACE: Duration = Duration::from_millis(400);

/// Where the window stands toward the newest build.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Standing {
    /// It runs the newest build, or nothing was read.
    #[default]
    Newest,
    /// A settled build of its repository is newer than the installed copy, or nothing is
    /// installed yet.
    Built,
    /// The installed copy is one to restart into.
    Installed,
}

/// What came of a restart asked for, each in words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The installed window started; this one closes in a moment.
    Restarting(String),
    /// Nothing newer to open, an install made or not: no warning.
    Newest(String),
    /// Refused, or an install or a start that failed.
    Refused(String),
}

/// The restart's state in the window.
#[derive(Debug, Default)]
pub struct Restart {
    /// The build the window runs, read at its start; none in a screenshot run or a test.
    pub running: Option<Running>,
    /// The install's folder; none without a local folder.
    pub bin: Option<PathBuf>,
    /// The release build of the window's repository.
    pub build: PathBuf,
    pub standing: Standing,
    looked: Option<Instant>,
    /// The close lets the process end at this moment: the restart is decided and the installed
    /// window started.
    pub closing_at: Option<Instant>,
    /// The installed window started for the restart, until this one closes.
    started: Option<Child>,
}

impl Restart {
    /// The restart of a window that runs `running`, its installed copy in `bin`, its repository's
    /// build in `build`.
    pub fn new(running: Option<Running>, bin: Option<PathBuf>, build: PathBuf) -> Self {
        Self {
            running,
            bin,
            build,
            ..Self::default()
        }
    }

    /// Looks at the files when `every` has passed since the last look.
    pub fn look(&mut self, every: Duration) {
        if self.running.is_none() || self.looked.is_some_and(|at| at.elapsed() < every) {
            return;
        }
        self.looked = Some(Instant::now());
        self.standing = self.standing_now();
    }

    fn standing_now(&self) -> Standing {
        let (Some(running), Some(bin)) = (&self.running, &self.bin) else {
            return Standing::Newest;
        };
        if install::restart_due(running, &bin.join(install::WINDOW_EXE)) {
            Standing::Installed
        } else if install::build_ready(&self.build, bin, SystemTime::now()) {
            Standing::Built
        } else {
            Standing::Newest
        }
    }

    /// The status bar's words while a newer build waits: the chip and its hover.
    pub fn chip(&self) -> Option<(&'static str, String)> {
        match self.standing {
            Standing::Newest => None,
            Standing::Built => Some((
                "newer build",
                format!(
                    "A build in {} is newer than the installed copy: File > Install and restart \
                     installs it and opens it in place of this window",
                    self.build.display()
                ),
            )),
            Standing::Installed => Some((
                "newer build installed",
                "A newer build is installed: File > Install and restart, or the tray's, opens it \
                 in place of this window once nothing stands in the way"
                    .to_owned(),
            )),
        }
    }

    /// The exit of the installed window started for the restart, when it has ended already.
    fn ended_at_once(&mut self) -> Option<String> {
        let child = self.started.as_mut()?;
        match child.try_wait() {
            Ok(Some(status)) => {
                self.started = None;
                Some(status.to_string())
            }
            _ => None,
        }
    }
}

/// The new window, started from `exe` with this process's id: it waits for this one to end.
fn start_installed(exe: &Path, folder: &Path) -> Result<Child, String> {
    Command::new(exe)
        .arg(install::RESTARTED_FLAG)
        .arg(std::process::id().to_string())
        .current_dir(folder)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| {
            format!(
                "the installed window did not start ({}): {e}",
                exe.display()
            )
        })
}

/// A previous copy's date, from the local time: `20261110-081500`.
fn stamp_now() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}

impl Board {
    /// What stands in the way of a restart now, each in words; none when it may go (D-13; EP-37).
    pub fn restart_refusals(&self) -> Vec<String> {
        let mut no = Vec::new();
        let runs = self.sessions.stoppable();
        if !runs.is_empty() {
            no.push(format!(
                "a headless run of the window's works or waits: {}",
                runs.join("; ")
            ));
        }
        let stopping = self.sessions.stopping();
        if !stopping.is_empty() {
            no.push(format!(
                "a headless run of the window's is being stopped, its end still to be recorded: {}",
                stopping.join("; ")
            ));
        }
        if self.sessions.is_starting() {
            no.push("a headless session is starting".to_owned());
        }
        if self.visual.is_opening() {
            no.push("a visual launch is on its way to VS Code".to_owned());
        }
        if self.reverting() {
            no.push("a revert is running".to_owned());
        }
        if self.autopilot.is_busy() {
            no.push("the autopilot reads a gate or starts a chained brief".to_owned());
        }
        for held in &self.autopilot.held {
            no.push(format!(
                "{}'s chain is held, and a held chain does not outlive the window yet",
                held.project
            ));
        }
        for plan in self.autopilot.plans.values() {
            no.push(format!(
                "{}'s chain plan is kept, and a plan does not outlive the window yet",
                plan.project
            ));
        }
        let starts = self.watch.unrecorded_starts();
        if !starts.is_empty() {
            no.push(format!(
                "a session's start waits for its first prompt to enter the ledger, and a restart \
                 would lose it, the revert going back to the last recorded brief: {}",
                starts.join(", ")
            ));
        }
        let mut typed: Vec<String> = self
            .drafts
            .iter()
            .filter(|(_, text)| !text.trim().is_empty())
            .map(|((project, id), _)| format!("{project} {id}"))
            .collect();
        if !typed.is_empty() {
            typed.sort();
            no.push(format!(
                "a disposition is being typed: {}",
                typed.join(", ")
            ));
        }
        for (open, what) in [
            (self.launch.is_some(), "the launch dialog is open"),
            (self.revert.is_some(), "the revert's preview is open"),
            (self.offer.is_some(), "the offer of auto is open"),
            (self.quit_asked, "the question before Quit is open"),
        ] {
            if open {
                no.push(what.to_owned());
            }
        }
        if self.fixtures {
            no.push("a screenshot run restarts nothing".to_owned());
        }
        no
    }

    /// Installs a settled build of the window's repository newer than the installed copy, then
    /// starts the installed window in place of this one, which closes a moment later (EP-37).
    pub fn install_and_restart(&mut self) -> Outcome {
        if self.restart.closing_at.is_some() {
            return Outcome::Restarting("the window is restarting already".to_owned());
        }
        // Whatever comes of it, the next pass looks at the files again.
        self.restart.looked = None;
        let refusals = self.restart_refusals();
        if !refusals.is_empty() {
            return Outcome::Refused(format!("not restarted: {}", refusals.join("; ")));
        }
        let (Some(running), Some(bin)) = (self.restart.running.clone(), self.restart.bin.clone())
        else {
            return Outcome::Refused(
                "not restarted: the window knows no installed copy to start".to_owned(),
            );
        };
        let mut installed = String::new();
        if install::build_ready(&self.restart.build, &bin, SystemTime::now()) {
            match install::install(&self.restart.build, &bin, &stamp_now()) {
                Ok(report) if !report.copied.is_empty() => {
                    installed = format!("installed {}; ", report.copied.join(" and "));
                }
                Ok(_) => {}
                Err(e) => return Outcome::Refused(format!("not installed: {e}")),
            }
        }
        let exe = bin.join(install::WINDOW_EXE);
        if !install::restart_due(&running, &exe) {
            return Outcome::Newest(format!(
                "{installed}not restarted: the window runs the newest build already"
            ));
        }
        match start_installed(&exe, &bin) {
            Ok(child) => self.restart.started = Some(child),
            Err(e) => return Outcome::Refused(format!("{installed}not restarted: {e}")),
        }
        self.restart.closing_at = Some(Instant::now() + ANSWER_GRACE);
        self.ctx().request_repaint_after(ANSWER_GRACE);
        let when = Stamp::of(&exe)
            .map(|stamp| stamp.when())
            .unwrap_or_default();
        Outcome::Restarting(format!(
            "{installed}restarting into the build of {when}: the window opens again in a moment"
        ))
    }

    /// File > Install and restart, and the tray's item: the restart, or the window shown with
    /// why not.
    pub fn restart_from_window(&mut self) {
        match self.install_and_restart() {
            Outcome::Restarting(words) | Outcome::Newest(words) => {
                self.notify(capitalized(&words));
            }
            Outcome::Refused(words) => {
                if self.hidden {
                    self.show();
                }
                self.warn(capitalized(&words));
            }
        }
    }

    /// Answers a restart request from the loopback (`crw install`'s): dropped when its caller
    /// stopped waiting, refused when it installed into another folder than the window's (a
    /// trial's), else the restart or why not, a refusal said in the status bar too.
    fn answer_restart(&mut self, request: RestartRequest) {
        // The server answered the caller that the window did not answer in time.
        if request.reply.is_closed() {
            return;
        }
        let foreign = match (&request.bin, &self.restart.bin) {
            (Some(theirs), Some(ours)) => !paths::same(theirs, ours),
            (Some(_), None) => true,
            (None, _) => false,
        };
        let outcome = match (&request.bin, foreign) {
            (Some(theirs), true) => Outcome::Refused(format!(
                "not restarted: the request installed into {}, which is not this window's \
                 install folder",
                theirs.display()
            )),
            _ => self.install_and_restart(),
        };
        let from = capitalized(&request.from);
        let answer = match outcome {
            Outcome::Restarting(words) => RestartAnswer {
                restarting: true,
                words,
            },
            Outcome::Newest(words) => {
                self.notify(format!("{from} asked for a restart: {words}"));
                RestartAnswer {
                    restarting: false,
                    words,
                }
            }
            Outcome::Refused(words) => {
                self.warn(format!("{from} asked for a restart: {words}"));
                RestartAnswer {
                    restarting: false,
                    words,
                }
            }
        };
        let _ = request.reply.send(answer);
    }

    /// The restart's share of each pass, the window shown or hidden: the files looked at, the
    /// loopback's requests answered, and the close let through once a decided restart's answer
    /// has had its moment, unless the installed window has ended already.
    pub fn restart_pass(&mut self, ctx: &egui::Context) {
        self.restart.look(LOOK_EVERY);
        for request in self.watch.take_restarts() {
            self.answer_restart(request);
        }
        if let Some(at) = self.restart.closing_at {
            let now = Instant::now();
            if now < at {
                ctx.request_repaint_after(at - now);
            } else if let Some(code) = self.restart.ended_at_once() {
                self.restart.closing_at = None;
                self.warn(format!(
                    "Not restarted: the installed window ended at once ({code}), so this window \
                     stays"
                ));
            } else {
                self.close_for_restart();
            }
        }
    }
}

/// `words` with a capital first letter, for a notice.
fn capitalized(words: &str) -> String {
    let mut chars = words.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Places, View};
    use crate::portfolio::tests::{Scratch, fake_store};
    use tokio::sync::oneshot;

    /// A board over the fake portfolio's registry in `scratch`, as a test has it: no installed
    /// copy, nothing running.
    fn board(scratch: &Scratch) -> Board {
        let places = Places {
            store: fake_store(scratch),
            ledger: None,
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        Board::new(places, &ctx, View::default(), egui::ThemePreference::Light)
    }

    #[test]
    fn a_screenshot_run_and_a_window_with_nothing_running_say_why() {
        let scratch = Scratch::new("restart-refusals");
        let mut board = board(&scratch);
        // Nothing stands in the way of a test's board; it knows no installed copy.
        assert!(board.restart_refusals().is_empty());
        let Outcome::Refused(error) = board.install_and_restart() else {
            panic!("a board with no installed copy restarts nothing");
        };
        assert!(error.contains("no installed copy"), "{error}");
        // A screenshot run, a held chain and a plan each refuse, every reason said.
        board.fixtures = true;
        board.autopilot.held.push(crate::autopilot::Held {
            project: "harbor".to_owned(),
            after: None,
            words: "held".to_owned(),
        });
        board.offer = Some(crate::autopilot::Offer {
            project: "harbor".to_owned(),
            mode: "acceptEdits".to_owned(),
        });
        let refusals = board.restart_refusals();
        assert_eq!(refusals.len(), 3, "{refusals:?}");
        assert!(refusals[0].starts_with("harbor's chain is held"));
        assert_eq!(refusals[1], "the offer of auto is open");
        assert_eq!(refusals[2], "a screenshot run restarts nothing");
        let Outcome::Refused(error) = board.install_and_restart() else {
            panic!("a refused restart");
        };
        assert!(
            error.starts_with("not restarted: harbor's chain is held"),
            "{error}"
        );
        assert!(board.restart.closing_at.is_none());
    }

    /// What lives only in the window's memory refuses a restart too: a session's start not yet in
    /// the ledger, a disposition being typed (the review of EP-37).
    #[test]
    fn a_start_not_yet_recorded_and_a_typed_disposition_refuse() {
        let scratch = Scratch::new("restart-memory");
        let mut board = board(&scratch);
        board.watch.reading_start_of("0e1d2c3b");
        board.drafts.insert(
            ("harbor".to_owned(), "IN-2".to_owned()),
            "rolled into EP-4".to_owned(),
        );
        // A field left empty is nothing typed.
        board
            .drafts
            .insert(("meadow".to_owned(), "IN-1".to_owned()), "  ".to_owned());
        let refusals = board.restart_refusals();
        assert_eq!(refusals.len(), 2, "{refusals:?}");
        assert!(
            refusals[0].starts_with("a session's start waits for its first prompt")
                && refusals[0].ends_with(": a session"),
            "{}",
            refusals[0]
        );
        assert_eq!(refusals[1], "a disposition is being typed: harbor IN-2");
    }

    /// A request whose caller stopped waiting is dropped unanswered, and one that installed into
    /// another folder (a trial's) is refused before anything is installed (the review of EP-37).
    #[test]
    fn a_late_or_foreign_request_restarts_nothing() {
        let scratch = Scratch::new("restart-requests");
        let mut board = board(&scratch);
        board.restart.bin = Some(scratch.0.join("bin"));
        let before = board.notice_text().map(str::to_owned);
        let (reply, answer) = oneshot::channel();
        drop(answer);
        board.answer_restart(RestartRequest {
            from: "crw install".to_owned(),
            bin: Some(scratch.0.join("bin")),
            reply,
        });
        assert_eq!(
            board.notice_text().map(str::to_owned),
            before,
            "a late request says nothing"
        );
        let (reply, mut answer) = oneshot::channel();
        board.answer_restart(RestartRequest {
            from: "crw install".to_owned(),
            bin: Some(scratch.0.join("trial").join("bin")),
            reply,
        });
        let answer = answer.try_recv().unwrap();
        assert!(!answer.restarting);
        assert!(
            answer.words.contains("not this window's install folder"),
            "{}",
            answer.words
        );
        assert!(
            board
                .notice_text()
                .is_some_and(|text| text.starts_with("Crw install asked for a restart")),
            "{:?}",
            board.notice_text()
        );
        assert!(!scratch.0.join("bin").exists(), "nothing installed");
        assert!(board.restart.closing_at.is_none());
    }

    #[test]
    fn the_standing_follows_the_build_and_the_installed_copy() {
        let scratch = Scratch::new("restart-standing");
        let (build, bin) = (scratch.0.join("release"), scratch.0.join("bin"));
        std::fs::create_dir_all(&build).unwrap();
        // A build left alone longer than it takes a linker to end.
        let settled = SystemTime::now() - install::BUILD_SETTLE - Duration::from_secs(1);
        for name in install::EXES {
            let path = build.join(name);
            std::fs::write(&path, name).unwrap();
            std::fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(settled)
                .unwrap();
        }
        let running = Running {
            exe: build.join(install::WINDOW_EXE),
            stamp: Stamp::of(&build.join(install::WINDOW_EXE)),
        };
        let mut restart = Restart::new(Some(running), Some(bin.clone()), build.clone());
        restart.look(Duration::ZERO);
        assert_eq!(restart.standing, Standing::Built);
        assert!(
            restart
                .chip()
                .unwrap()
                .1
                .contains("newer than the installed copy")
        );
        install::install(&build, &bin, "20261110-080000").unwrap();
        restart.look(Duration::ZERO);
        // The same build installed where the window does not run from: a restart moves it.
        assert_eq!(restart.standing, Standing::Installed);
        // Within the pace nothing is read again.
        restart.standing = Standing::Newest;
        restart.look(LOOK_EVERY);
        assert_eq!(restart.standing, Standing::Newest);
    }

    #[test]
    fn a_build_still_being_written_is_not_offered() {
        let scratch = Scratch::new("restart-unsettled");
        let (build, bin) = (scratch.0.join("release"), scratch.0.join("bin"));
        std::fs::create_dir_all(&build).unwrap();
        for name in install::EXES {
            std::fs::write(build.join(name), name).unwrap();
        }
        // Running elsewhere, an older build: only the fresh build could be offered.
        let running = Running {
            exe: scratch.0.join("old").join(install::WINDOW_EXE),
            stamp: None,
        };
        let mut restart = Restart::new(Some(running), Some(bin), build);
        restart.look(Duration::ZERO);
        assert_eq!(restart.standing, Standing::Newest);
        assert!(restart.chip().is_none());
    }

    #[test]
    fn a_notice_starts_with_a_capital() {
        assert_eq!(capitalized("not restarted: x"), "Not restarted: x");
        assert_eq!(capitalized(""), "");
    }
}
