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
//! start, a session's start not yet in the ledger, a disposition being typed, an open dialog. A
//! refused request is not remembered: the status bar's chip stays, and File > Install and restart,
//! or the tray's, takes it once nothing stands in the way. A request that installed into another
//! folder than the window's (a trial's `CRW_HOME\bin`) is refused, and one whose caller stopped
//! waiting is dropped. Every 30 seconds, and every second while the File menu is open, the window
//! looks whether a newer build is built or installed, which the status bar then says. A screenshot
//! run restarts nothing.
//!
//! **The handover** (EP-37a, `crw_core::handover`): the held chains and the chain plans cross the
//! restart in a file the window writes before it starts the installed window, which takes it at its
//! start; a carried chain's gate is read there once the first poll of live sessions has answered.
//! **The gate's moment**: a gate that passes or holds while a newer build is installed restarts
//! the window, a passing chain held for the restart, so that a chain of the tool's own briefs goes
//! on in each brief's build; a restart refused then lets the chain go on in this window. A chain's
//! end restarts nothing (the owner's choice), and the gate's moment installs nothing.
//! **The launch dialog** (EP-39): while a newer build is installed, the dialog says so above its
//! footer, since a run started from it would run in the older window, and offers Install and
//! restart, which closes the dialog (an open one refuses a restart), asks the owner's restart and
//! carries the dialog's project in the handover, so that the new window opens the dialog again
//! once its files are read; a refusal puts the dialog back with the reasons.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use crw_core::handover::{self, Handover, Taken, UrgentSeen};
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

/// Who asked for a restart (EP-37a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// The owner, from the File menu or the tray, or `crw install`: a settled build is installed
    /// first, and the new window opens shown.
    Owner,
    /// The autopilot's gate between two runs: only an installed build is taken, and a window
    /// hidden in the tray stays hidden.
    Gate,
}

/// Starts the installed window for a restart: the real start, or a test's stand-in.
type Spawn = fn(&Path, &Path) -> Result<Option<Child>, String>;

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
    /// The local folder, where the handover goes (EP-37a); none in a test or a screenshot run
    /// unless set.
    pub local: Option<PathBuf>,
    /// A test's stand-in for the installed window's start; none starts it.
    pub spawn: Option<Spawn>,
    /// The installed copy whose window ended at once: the gate's moment leaves it alone until the
    /// owner's own restart or a newer install.
    failed: Option<Stamp>,
    /// The installed build's date, read with the standing while it is one to restart into
    /// (EP-39): what the launch dialog's line names.
    installed_when: Option<String>,
    /// A screenshot run's canned words for the launch dialog's line (EP-39): a run has no
    /// installed copy to read.
    pub canned: Option<String>,
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
        self.installed_when = match (self.standing, &self.bin) {
            (Standing::Installed, Some(bin)) => {
                Stamp::of(&bin.join(install::WINDOW_EXE)).map(|stamp| stamp.when())
            }
            _ => None,
        };
    }

    /// Looks at the files now, whatever the pace: at the launch dialog's opening (EP-39).
    pub fn look_now(&mut self) {
        self.looked = None;
        self.look(LOOK_EVERY);
    }

    /// The launch dialog's line while a newer build is installed (EP-39): a run started from
    /// the dialog would run in this older window, and the dialog's Install and restart opens
    /// the new one with the dialog again. None while the window runs the newest build; a
    /// screenshot run's canned words otherwise.
    pub fn installed_words(&self) -> Option<String> {
        if let Some(words) = &self.canned {
            return Some(words.clone());
        }
        match (self.standing, &self.installed_when) {
            (Standing::Installed, Some(when)) => Some(installed_words_of(when)),
            _ => None,
        }
    }
}

/// The launch dialog's line for an installed build of `when` (EP-39).
pub fn installed_words_of(when: &str) -> String {
    format!(
        "A newer build is installed (of {when}): a session started from here runs in this older \
         window. Install and restart opens the new window and this dialog again, afresh."
    )
}

impl Restart {
    /// Whether the installed copy is one to restart into at the gate's moment (EP-37a), looked at
    /// now: another build than the window runs, so that a gate never restarts the window into
    /// its own build (the move out of `target\release` stays the owner's), and not one whose
    /// window ended at once before.
    pub fn pending_install(&self) -> bool {
        let (Some(running), Some(bin)) = (&self.running, &self.bin) else {
            return false;
        };
        let exe = bin.join(install::WINDOW_EXE);
        let stamp = Stamp::of(&exe);
        install::restart_due(running, &exe)
            && stamp.is_some()
            && stamp != running.stamp
            && (self.failed.is_none() || self.failed != stamp)
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
        // A held chain and a chain plan cross the restart in the handover (EP-37a).
        let starts = self.watch.unrecorded_starts();
        if !starts.is_empty() {
            no.push(format!(
                "a session open with no prompt yet would lose its ledger row and its start to a \
                 restart: {}",
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
            // A message being written would be lost (EP-21a's finding: EP-20c left it out).
            (self.compose.is_some(), "the compose dialog is open"),
            (self.formalise.is_some(), "the formalise preview is open"),
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
        self.restart_window(Asked::Owner)
    }

    /// The gate's moment (EP-37a): with a newer build installed and nothing else in the way, the
    /// window restarts into it, the held chains and the plans carried; else why not, in words.
    pub fn restart_at_gate(&mut self) -> Result<(), String> {
        if self.fixtures || !self.restart.pending_install() {
            return Err("no newer build is installed".to_owned());
        }
        match self.restart_window(Asked::Gate) {
            Outcome::Restarting(words) => {
                self.notify(capitalized(&words));
                Ok(())
            }
            Outcome::Newest(words) | Outcome::Refused(words) => {
                Err(words.trim_start_matches("not restarted: ").to_owned())
            }
        }
    }

    /// What stands in the way of a restart the gate asks for, beyond what refuses the owner's
    /// (EP-37a's review): an ended run whose tab offers Commit, which the owner has not taken and
    /// a restart nobody asked for would drop; and a finished migration's tab, whose report and
    /// commit are the owner's (EP-21a's review).
    fn gate_refusals(&self) -> Vec<String> {
        let tabs = |keep: fn(&crate::sessions::Live) -> bool| -> Vec<String> {
            self.sessions
                .live
                .iter()
                .filter(|live| keep(live))
                .map(|live| live.title())
                .collect()
        };
        let mut refusals = Vec::new();
        let offers = tabs(crate::sessions::Live::offers_commit);
        if !offers.is_empty() {
            refusals.push(format!(
                "an ended run's tab offers Commit: {}",
                offers.join(", ")
            ));
        }
        let migrated = tabs(crate::sessions::Live::finished_migration);
        if !migrated.is_empty() {
            refusals.push(format!(
                "a finished migration's tab waits for you: {}",
                migrated.join(", ")
            ));
        }
        refusals
    }

    /// What the window carries across a restart asked by `asked` (EP-37a): its held chains, its
    /// plans, whether the new window opens hidden, the urgent messages it has seen (EP-20c) and
    /// the project whose launch dialog asked for the restart, to open again (EP-39).
    pub fn handover(&self, asked: Asked) -> Handover {
        Handover::new(
            std::process::id(),
            chrono::Utc::now().to_rfc3339(),
            self.autopilot.held.clone(),
            self.autopilot.plans.values().cloned().collect(),
            asked == Asked::Gate && self.hidden,
            self.toasts.urgent_seen().map(|seen| {
                seen.into_iter()
                    .map(|(project, id)| UrgentSeen { project, id })
                    .collect()
            }),
            self.reopen_launch.clone(),
        )
    }

    /// The launch dialog's Install and restart (EP-39): the dialog closes, since an open one
    /// refuses a restart, and the handover carries its project, so that the new window opens it
    /// again once its files are read. A refused restart, none due, or one under way already
    /// puts the dialog back with the words; nothing restarts by itself. The project stays in
    /// `reopen_launch` while the window closes, so that an installed window that ends at once
    /// opens the dialog here again (the review's finding).
    pub fn restart_from_launch(&mut self) {
        let Some(mut dialog) = self.launch.take() else {
            return;
        };
        if self.restart.closing_at.is_some() {
            dialog.error = Some("The window is restarting already.".to_owned());
            self.launch = Some(dialog);
            return;
        }
        self.reopen_launch = Some(dialog.project.clone());
        match self.install_and_restart() {
            Outcome::Restarting(words) => self.notify(capitalized(&format!(
                "{words}, with the launch dialog of {} open again",
                dialog.project
            ))),
            Outcome::Newest(words) => {
                self.reopen_launch = None;
                dialog.newer_build = None;
                dialog.error = None;
                self.launch = Some(dialog);
                self.notify(capitalized(&words));
            }
            Outcome::Refused(words) => {
                self.reopen_launch = None;
                dialog.error = Some(capitalized(&words));
                self.launch = Some(dialog);
            }
        }
    }

    /// The restart, asked by the owner (a settled build installed first) or at the gate's moment
    /// (the installed copy only): the handover written, then the installed window started in place
    /// of this one, which closes a moment later.
    fn restart_window(&mut self, asked: Asked) -> Outcome {
        if self.restart.closing_at.is_some() {
            return Outcome::Restarting("the window is restarting already".to_owned());
        }
        // Whatever comes of it, the next pass looks at the files again.
        self.restart.looked = None;
        let mut refusals = self.restart_refusals();
        if asked == Asked::Gate {
            refusals.extend(self.gate_refusals());
        }
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
        if asked == Asked::Owner
            && install::build_ready(&self.restart.build, &bin, SystemTime::now())
        {
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
        // The handover first, which the new window takes at its start (EP-37a).
        let handover = self.handover(asked);
        let local = match (&self.restart.local, handover.carries()) {
            (_, false) => None,
            (Some(dir), true) => Some(dir.clone()),
            (None, true) => {
                return Outcome::Refused(format!(
                    "{installed}not restarted: the window knows no local folder for the handover \
                     of {}",
                    handover.words()
                ));
            }
        };
        if let Some(dir) = &local
            && let Err(e) = handover::write(dir, &handover)
        {
            return Outcome::Refused(format!("{installed}not restarted: {e}"));
        }
        let spawn = self
            .restart
            .spawn
            .unwrap_or(|exe, folder| start_installed(exe, folder).map(Some));
        match spawn(&exe, &bin) {
            Ok(child) => self.restart.started = child,
            Err(e) => {
                if let Some(dir) = &local {
                    handover::discard(dir);
                }
                return Outcome::Refused(format!("{installed}not restarted: {e}"));
            }
        }
        if asked == Asked::Owner {
            self.restart.failed = None;
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
                // The new window never took the handover: this one keeps what it holds, its held
                // chains read again at once, and the gate's moment leaves that build alone.
                if let Some(dir) = &self.restart.local {
                    handover::discard(dir);
                }
                self.restart.failed = self
                    .restart
                    .bin
                    .as_ref()
                    .and_then(|bin| Stamp::of(&bin.join(install::WINDOW_EXE)));
                self.autopilot.recheck = true;
                // The launch dialog that asked opens here again, the files being read (EP-39).
                let dialog = match self.reopen_launch.take() {
                    Some(project) => {
                        self.open_when_read = Some(project.clone());
                        format!(", and the launch dialog of {project} opens again")
                    }
                    None => String::new(),
                };
                self.warn(format!(
                    "Not restarted: the installed window ended at once ({code}), so this window \
                     stays{dialog}"
                ));
            } else {
                self.close_for_restart();
            }
        }
    }
}

impl Board {
    /// Takes the handover at the window's start (EP-37a), from the local folder `dir`, and says
    /// so in the start's notice (`when` names the build a restarted window runs): a restarted
    /// window's held chains go on, their gates read once the first poll of live sessions and the
    /// first files read have answered, its plans are kept, and it hides in the tray when the
    /// restart was the gate's from a hidden window. A file no restart took is dropped, and the
    /// notice and the chain log say what was lost.
    pub fn take_handover(&mut self, dir: Option<&Path>, restarted: Option<u32>, when: &str) {
        let taken = dir.map_or(Taken::Nothing, |dir| handover::take(dir, restarted));
        let restart = restarted.map(|_| format!("Restarted into the installed build{when}"));
        match taken {
            Taken::Nothing => {
                if let Some(restart) = restart {
                    self.notify(restart);
                }
            }
            Taken::Ours(found) => {
                let carried = (!found.held.is_empty()
                    || !found.plans.is_empty()
                    || found.open_launch.is_some())
                .then(|| found.words());
                // The launch dialog that asked for the restart opens again once the files are
                // read (EP-39), the way a screenshot run's `launch=` opens one.
                if let Some(project) = found.open_launch {
                    self.open_when_read = Some(project);
                    self.open_headless = false;
                }
                if !found.held.is_empty() {
                    self.autopilot.carried = Some(Instant::now());
                    self.autopilot.held.extend(found.held);
                }
                for plan in found.plans {
                    self.autopilot.keep_plan(plan);
                }
                if found.hidden {
                    self.hide_at_start();
                }
                // The old window's last look at the urgent messages: what arrived since is news
                // to this one (EP-20c).
                if let Some(seen) = found.urgent_seen {
                    self.toasts.carry_urgent_seen(
                        seen.into_iter()
                            .map(|seen| (seen.project, seen.id))
                            .collect(),
                    );
                }
                if let Some(restart) = restart {
                    self.notify(match carried {
                        Some(words) => format!("{restart}, carrying {words}"),
                        None => restart,
                    });
                }
            }
            Taken::Left(words) => {
                let lost =
                    format!("a handover no restart took was dropped, its chains lost: {words}");
                self.log_chain(None, &lost);
                self.warn(match restart {
                    Some(restart) => format!("{restart}, but {lost}"),
                    None => capitalized(&lost),
                });
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
pub(crate) mod tests {
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
        // A held chain no longer refuses: the handover carries it (EP-37a).
        board.autopilot.held.push(crate::autopilot::Held {
            project: "harbor".to_owned(),
            after: None,
            words: "held".to_owned(),
        });
        assert!(board.restart_refusals().is_empty());
        // A screenshot run and a dialog each refuse, every reason said.
        board.fixtures = true;
        board.offer = Some(crate::autopilot::Offer {
            project: "harbor".to_owned(),
            mode: "acceptEdits".to_owned(),
        });
        let refusals = board.restart_refusals();
        assert_eq!(refusals.len(), 2, "{refusals:?}");
        assert_eq!(refusals[0], "the offer of auto is open");
        assert_eq!(refusals[1], "a screenshot run restarts nothing");
        let Outcome::Refused(error) = board.install_and_restart() else {
            panic!("a refused restart");
        };
        assert!(
            error.starts_with("not restarted: the offer of auto is open"),
            "{error}"
        );
        assert!(board.restart.closing_at.is_none());
    }

    /// A window with an installed copy to restart into, in `scratch`: a build newer than the one
    /// it runs, the start a stand-in, the handover's folder `local`.
    pub(crate) fn installed(board: &mut Board, scratch: &Scratch) {
        let (build, bin) = (scratch.0.join("release"), scratch.0.join("bin"));
        std::fs::create_dir_all(&build).unwrap();
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
        install::install(&build, &bin, "20261110-080000").unwrap();
        // The window runs an older build of its own.
        let old = scratch.0.join("old").join(install::WINDOW_EXE);
        std::fs::create_dir_all(old.parent().unwrap()).unwrap();
        std::fs::write(&old, "older").unwrap();
        std::fs::File::options()
            .write(true)
            .open(&old)
            .unwrap()
            .set_modified(settled - Duration::from_secs(3600))
            .unwrap();
        let running = Running {
            stamp: Stamp::of(&old),
            exe: old,
        };
        board.restart = Restart::new(Some(running), Some(bin), build);
        board.restart.local = Some(scratch.0.join("local"));
        board.restart.spawn = Some(|_, _| Ok(None));
    }

    /// The gate's moment restarts only into another build than the window runs (EP-37a's review):
    /// the same build installed where the window does not run from is the owner's move, and a
    /// copy whose window ended at once is left alone.
    #[test]
    fn a_gate_never_restarts_into_the_build_it_runs() {
        let scratch = Scratch::new("restart-pending");
        let mut board = board(&scratch);
        installed(&mut board, &scratch);
        assert!(board.restart.pending_install());
        let exe = board
            .restart
            .bin
            .as_ref()
            .unwrap()
            .join(install::WINDOW_EXE);
        // The window runs the build that is installed, from `target\release`.
        let build = board.restart.build.join(install::WINDOW_EXE);
        board.restart.running = Some(Running {
            stamp: Stamp::of(&build),
            exe: build,
        });
        assert!(install::restart_due(
            board.restart.running.as_ref().unwrap(),
            &exe
        ));
        assert!(!board.restart.pending_install());
        // A newer copy whose window ended at once.
        installed(&mut board, &scratch);
        board.restart.failed = Stamp::of(&exe);
        assert!(!board.restart.pending_install());
    }

    /// The handover is written before the installed window starts and removed when the start
    /// fails; a window with chains to carry and no local folder does not restart (EP-37a).
    #[test]
    fn the_handover_is_written_at_a_restart_and_removed_when_the_start_fails() {
        let scratch = Scratch::new("restart-handover");
        let mut board = board(&scratch);
        installed(&mut board, &scratch);
        board.autopilot.held.push(crate::autopilot::Held {
            project: "harbor".to_owned(),
            after: Some("EP-3".parse().unwrap()),
            words: "held after EP-3 until the pause lifts".to_owned(),
        });
        let file = handover::path(&scratch.0.join("local"));
        // The start fails: no file is left, and the window keeps its chain.
        board.restart.spawn = Some(|_, _| Err("the installed window did not start".to_owned()));
        let Outcome::Refused(error) = board.install_and_restart() else {
            panic!("a start that failed");
        };
        assert!(
            error.ends_with("the installed window did not start"),
            "{error}"
        );
        assert!(!file.exists() && board.autopilot.held.len() == 1);
        // No local folder for what must be carried: refused before anything starts.
        board.restart.local = None;
        let Outcome::Refused(error) = board.install_and_restart() else {
            panic!("no folder for the handover");
        };
        assert!(error.contains("no local folder for the handover of 1 held chain (harbor)"));
        // The start goes: the file holds the chain, and the window closes in a moment.
        board.restart.local = Some(scratch.0.join("local"));
        board.restart.spawn = Some(|_, _| Ok(None));
        let Outcome::Restarting(words) = board.install_and_restart() else {
            panic!("a restart");
        };
        assert!(words.contains("restarting into the build of"), "{words}");
        assert!(board.restart.closing_at.is_some());
        let Taken::Ours(found) = handover::take(&scratch.0.join("local"), Some(std::process::id()))
        else {
            panic!("the handover is the window's");
        };
        assert_eq!(found.held, board.autopilot.held);
        // The owner's restart opens the new window shown, the gate's keeps a hidden one hidden.
        assert!(!found.hidden);
        board.hidden = true;
        assert!(board.handover(Asked::Gate).hidden && !board.handover(Asked::Owner).hidden);
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
            refusals[0].starts_with("a session open with no prompt yet would lose its ledger row")
                && refusals[0].ends_with(": a session"),
            "{}",
            refusals[0]
        );
        assert_eq!(refusals[1], "a disposition is being typed: harbor IN-2");
    }

    /// A session open with no prompt yet refuses a restart in any registered project, its
    /// revert's switch on or off: only the window that saw it open gives it its ledger row (the
    /// review of EP-37a). One that worked, a headless one and one outside the registry do not.
    #[test]
    fn a_session_open_with_no_prompt_refuses_whatever_its_revert() {
        use crw_core::sessions::{Mode, Seen, Status};
        let scratch = Scratch::new("restart-opened");
        let mut board = board(&scratch);
        let started = chrono::Utc::now();
        let seen = |id: &str, project: Option<&str>, mode: Mode, worked: bool| {
            let mut seen = Seen::canned(id, project, mode, Status::Idle, None, started);
            seen.worked = worked;
            seen
        };
        for one in [
            seen("w1", Some("meadow"), Mode::Manual, true),
            seen("h1", Some("meadow"), Mode::Headless, false),
            seen("x1", None, Mode::Manual, false),
        ] {
            board.watch.model.insert(one);
        }
        assert!(board.restart_refusals().is_empty());
        board
            .watch
            .model
            .insert(seen("n1", Some("quarry"), Mode::Manual, false));
        assert_eq!(
            board.restart_refusals(),
            [
                "a session open with no prompt yet would lose its ledger row and its start to a \
              restart: quarry"
            ]
        );
    }

    /// A restart nobody asked for also waits for an ended run's Commit (the review of EP-37a);
    /// the owner's own does not.
    #[test]
    fn a_gate_restart_waits_for_an_offered_commit() {
        let scratch = Scratch::new("restart-commit");
        let mut board = board(&scratch);
        installed(&mut board, &scratch);
        assert!(board.gate_refusals().is_empty());
        // A run of the window's completed under the rule that stops before committing.
        let mut done = crate::sessions::tests::live();
        done.take(crate::sessions::tests::exit(
            crw_core::session::Outcome::Completed,
            None,
        ));
        board.sessions.live.push(done);
        assert!(
            board.restart_refusals().is_empty(),
            "the owner's restart goes"
        );
        assert_eq!(
            board.gate_refusals(),
            ["an ended run's tab offers Commit: alpha EP-3"]
        );
        assert_eq!(
            board.restart_at_gate(),
            Err("an ended run's tab offers Commit: alpha EP-3".to_owned())
        );
        assert!(board.restart.closing_at.is_none());
        // A finished migration offers no Commit, and still holds the restart (EP-21a's review).
        let done = &mut board.sessions.live[0];
        done.migration = true;
        done.ep = crate::launch::MIGRATION.to_owned();
        assert!(!done.offers_commit());
        assert_eq!(
            board.gate_refusals(),
            ["a finished migration's tab waits for you: alpha migration"]
        );
        assert!(board.restart_refusals().is_empty());
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

    /// A board over the fake portfolio with its files read and the CLI and VS Code canned, so
    /// that the launch dialog opens without asking anything real (rule 5).
    fn board_with_files(scratch: &Scratch) -> Board {
        let mut board = board(scratch);
        board.portfolio = crate::portfolio::read_files(board.store());
        board.sessions.canned = Some(std::sync::Arc::new(crate::launch::canned_preflight()));
        board.visual.canned = Some((
            PathBuf::from(r"C:\Tools\VS Code\Code.exe"),
            crw_core::version::Version::new(2, 1, 314),
        ));
        board.catalog_folder = None;
        board
    }

    /// The launch dialog's Install and restart (EP-39): the dialog closes for the restart, the
    /// handover carries its project, and the restarted window opens it again once its files
    /// are read; a refusal puts the dialog back with the reasons, its own openness not among
    /// them; nothing newer puts it back with its line gone.
    #[test]
    fn the_launch_dialogs_restart_carries_its_project_and_a_refusal_keeps_the_dialog() {
        let scratch = Scratch::new("restart-launch");
        let mut board = board_with_files(&scratch);
        installed(&mut board, &scratch);
        // Opened, the dialog learns that a newer build is installed, read then and there.
        assert!(board.restart.installed_words().is_none(), "not looked yet");
        board.open_launch("harbor");
        assert!(board.launch.is_some());
        let words = board
            .restart
            .installed_words()
            .expect("a newer build installed");
        assert!(
            words.starts_with("A newer build is installed (of 20")
                && words.ends_with("opens the new window and this dialog again, afresh."),
            "{words}"
        );
        assert_eq!(board.restart_refusals(), ["the launch dialog is open"]);
        // Refused: the dialog stays, with the reason, and no handover is written.
        board.offer = Some(crate::autopilot::Offer {
            project: "harbor".to_owned(),
            mode: "acceptEdits".to_owned(),
        });
        board.restart_from_launch();
        let dialog = board.launch.as_ref().expect("the dialog is back");
        assert_eq!(
            dialog.error.as_deref(),
            Some("Not restarted: the offer of auto is open")
        );
        assert!(board.reopen_launch.is_none() && board.restart.closing_at.is_none());
        let local = scratch.0.join("local");
        assert!(!handover::path(&local).exists());
        // Taken: the dialog is gone, the handover names its project, and the window closes.
        board.offer = None;
        board.restart_from_launch();
        assert!(board.launch.is_none());
        assert!(board.restart.closing_at.is_some());
        assert!(
            board.notice_text().is_some_and(|text| {
                text.starts_with("Restarting into the build of")
                    && text.ends_with(", with the launch dialog of harbor open again")
            }),
            "{:?}",
            board.notice_text()
        );
        let Taken::Ours(found) = handover::take(&local, Some(std::process::id())) else {
            panic!("the handover is the window's");
        };
        assert_eq!(found.open_launch.as_deref(), Some("harbor"));
        assert!(found.held.is_empty() && !found.hidden);
        assert_eq!(found.words(), "the launch dialog of harbor");
        // The project is kept while the window closes, for an installed window that ends at
        // once; a dialog opened meanwhile is put back, the window restarting already.
        assert_eq!(board.reopen_launch.as_deref(), Some("harbor"));
        board.open_launch("harbor");
        board.restart_from_launch();
        let dialog = board.launch.as_ref().expect("the dialog is back");
        assert_eq!(
            dialog.error.as_deref(),
            Some("The window is restarting already.")
        );
        assert_eq!(board.reopen_launch.as_deref(), Some("harbor"));
        board.launch = None;
        // The restarted window: the dialog opens again once the files are read, the way a
        // screenshot run's `launch=` opens one.
        handover::write(&local, &found).unwrap();
        let mut restarted = board_with_files(&scratch);
        restarted.open_headless = true;
        restarted.take_handover(
            Some(&local),
            Some(std::process::id()),
            " of 2026-11-11 18:05",
        );
        assert_eq!(restarted.open_when_read.as_deref(), Some("harbor"));
        assert!(
            !restarted.open_headless,
            "the dialog opens in the project's mode"
        );
        assert_eq!(
            restarted.notice_text(),
            Some(
                "Restarted into the installed build of 2026-11-11 18:05, carrying the launch \
                 dialog of harbor"
            )
        );
        // Nothing newer after all: the dialog comes back with its line gone.
        let scratch = Scratch::new("restart-launch-newest");
        let mut board = board_with_files(&scratch);
        installed(&mut board, &scratch);
        board.open_launch("harbor");
        let exe = board
            .restart
            .bin
            .as_ref()
            .unwrap()
            .join(install::WINDOW_EXE);
        board.restart.running = Some(Running {
            stamp: Stamp::of(&exe),
            exe,
        });
        board.restart_from_launch();
        let dialog = board.launch.as_ref().expect("the dialog is back");
        assert!(dialog.error.is_none() && dialog.newer_build.is_none());
        assert!(board.restart.closing_at.is_none());
        assert!(
            board
                .notice_text()
                .is_some_and(|text| text.contains("runs the newest build already")),
            "{:?}",
            board.notice_text()
        );
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
