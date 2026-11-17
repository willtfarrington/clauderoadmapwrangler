//! The Diagnostics tile (D-29; EP-28): the doctor's report, which `crw doctor` prints too,
//! gathered on a worker thread when the tile is first shown and at its Check again button, and
//! above it the window's own lines, read at each frame: the build it runs, the signals, the
//! notifications and the tray, the usage meter, the mailman, the headless route, the defer
//! route's `crw.exe` and the rolling log. Only the real window's checks start children (`main`
//! starts them); a screenshot run shows a canned report, and a test's board starts none.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, TimeDelta, Utc};
use crw_core::badges::short_age;
use crw_core::doctor::{self, Group, Level, Report};
use crw_core::session::Route;
use crw_core::{install, paths, process};
use eframe::egui::{self, Color32, RichText, TextFormat, text::LayoutJob};

use crate::board::Board;
use crate::server::Listening;
use crate::style;

/// The tile's state: the last report and the run that gathers the next.
#[derive(Default)]
pub struct Diagnostics {
    /// Started by `main`: the checks may start children.
    live: bool,
    report: Option<(Report, DateTime<Utc>)>,
    running: Option<Receiver<Report>>,
    /// The first check was asked: the tile was shown, or a report was canned.
    asked: bool,
    /// The registry's `settings.decisions` as read for the last check: the headless route's line.
    setting: Option<Route>,
    /// The window's lines and when they were read: kept a moment, since the notifications' line
    /// reads the registry's file.
    window: Option<(Instant, Vec<WindowLine>)>,
}

/// How long the window's lines are kept before they are read again.
const WINDOW_FOR: Duration = Duration::from_secs(2);

impl Diagnostics {
    /// The real window's: its checks may start children.
    pub fn start(&mut self) {
        self.live = true;
    }

    /// A screenshot run's or a test's report, shown as it is; nothing is gathered.
    #[cfg(any(test, feature = "screenshot"))]
    pub fn can(&mut self, report: Report, at: DateTime<Utc>) {
        self.report = Some((report, at));
        self.asked = true;
    }

    pub fn checking(&self) -> bool {
        self.running.is_some()
    }

    /// The real window's: its checks may start children (the phone's doctor's page, EP-31).
    pub fn is_live(&self) -> bool {
        self.live
    }

    /// The last report and when it was gathered, when one was gathered or canned (EP-31).
    pub fn last(&self) -> Option<(&Report, DateTime<Utc>)> {
        self.report.as_ref().map(|(report, at)| (report, *at))
    }

    /// The last report, when one was gathered or canned.
    #[cfg(test)]
    pub fn report(&self) -> Option<&Report> {
        self.report.as_ref().map(|(report, _)| report)
    }

    /// Takes in a report gathered since: at the pane's drawing, and in the window's pass for the
    /// phone's page, which may ask while the window is hidden (EP-31).
    pub(crate) fn receive(&mut self) {
        if let Some(rx) = &self.running {
            match rx.try_recv() {
                Ok(report) => {
                    self.running = None;
                    self.report = Some((report, Utc::now()));
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => self.running = None,
            }
        }
    }

    /// Gathers the report on a worker thread, the children one at a time in their turn, unless
    /// one is under way or the window is not the real one.
    fn check(&mut self, ctx: &egui::Context) {
        self.asked = true;
        if !self.live || self.running.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let report = doctor::report(&doctor::probe(), &doctor::Places::real(), Utc::now());
            let _ = tx.send(report);
            ctx.request_repaint();
        });
        self.running = Some(rx);
    }
}

/// One of the window's own lines: how it stands, its name, its words.
pub type WindowLine = (Level, &'static str, String);

impl Board {
    /// The tile: the window's lines, then the report by group.
    pub fn diagnostics_pane(&mut self, ui: &mut egui::Ui) {
        self.diagnostics.receive();
        if !self.diagnostics.asked {
            self.check_diagnostics();
        }
        let stale = self
            .diagnostics
            .window
            .as_ref()
            .is_none_or(|(at, _)| at.elapsed() >= WINDOW_FOR);
        if stale {
            let lines = self.window_lines(Utc::now());
            self.diagnostics.window = Some((Instant::now(), lines));
            ui.ctx().request_repaint_after(WINDOW_FOR);
        }
        let window = self
            .diagnostics
            .window
            .as_ref()
            .map(|(_, lines)| lines.clone())
            .unwrap_or_default();
        let mut again = false;
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.label(RichText::new("This window").strong());
                for (level, name, text) in &window {
                    line_ui(ui, *level, name, text);
                }
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Checks").strong());
                    if self.diagnostics.checking() {
                        ui.spinner();
                        ui.label(RichText::new("Checking…").weak());
                    } else if self.diagnostics.live && ui.button("Check again").clicked() {
                        again = true;
                    }
                    if let Some((_, at)) = &self.diagnostics.report {
                        let at = at.with_timezone(&Local).format("%H:%M:%S");
                        ui.label(RichText::new(format!("as of {at}")).weak());
                    }
                });
                match &self.diagnostics.report {
                    None if self.diagnostics.checking() => {}
                    None => {
                        ui.label(RichText::new("No checks in this window.").weak());
                    }
                    Some((report, _)) => {
                        for group in Group::ALL {
                            let mut lines = report.group(group).peekable();
                            if lines.peek().is_none() {
                                continue;
                            }
                            ui.add_space(4.0);
                            ui.label(RichText::new(group.title()).weak());
                            for line in lines {
                                line_ui(ui, line.level, &line.name, &line.text);
                            }
                        }
                    }
                }
                ui.add_space(6.0);
                ui.label(
                    RichText::new(
                        "crw doctor prints the same checks in a terminal; nothing here is \
                         changed or deleted.",
                    )
                    .weak(),
                );
            });
        if again {
            self.check_diagnostics();
        }
    }

    /// The phone's Check again (EP-31, the owner's choice): the pane's own check on its worker
    /// thread, refused where the window gathers none (a screenshot run, a test), said when one is
    /// under way already. The report lands in a later pass, which the doctor's page then shows.
    pub(crate) fn check_from_page(&mut self) -> Result<String, String> {
        if !self.diagnostics.live {
            return Err("This window gathers no report: a screenshot run or a test.".to_owned());
        }
        if self.diagnostics.checking() {
            return Ok(
                "A check is under way already: the page shows its report when it lands.".to_owned(),
            );
        }
        self.check_diagnostics();
        Ok(
            "Checking: the tools answer one at a time in their turn, as at the pane's Check \
            again, and the page shows the report when it lands."
                .to_owned(),
        )
    }

    /// Reads the registry's route setting, then gathers the report.
    fn check_diagnostics(&mut self) {
        if self.diagnostics.live {
            self.diagnostics.setting = self.decisions_setting();
        }
        let ctx = self.ctx().clone();
        self.diagnostics.check(&ctx);
    }

    /// The window's own lines, from what it holds now.
    pub fn window_lines(&self, now: DateTime<Utc>) -> Vec<WindowLine> {
        let mut lines: Vec<WindowLine> = Vec::new();
        // The build this window runs, and the installed copy when it runs another.
        let installed = self
            .restart
            .bin
            .as_ref()
            .map(|bin| bin.join(install::WINDOW_EXE));
        let build = match &self.restart.running {
            Some(running) => {
                let runs = match running.stamp {
                    Some(stamp) => format!("{}, built {}", running.exe.display(), stamp.when()),
                    None => running.exe.display().to_string(),
                };
                match &installed {
                    Some(copy) if paths::same(copy, &running.exe) => {
                        format!("{runs} (the installed copy)")
                    }
                    Some(copy) => match install::Stamp::of(copy) {
                        Some(stamp) => {
                            format!("{runs}; the installed copy is built {}", stamp.when())
                        }
                        None => format!("{runs}; no installed copy"),
                    },
                    None => runs,
                }
            }
            None => "not read (a screenshot run or a test)".to_owned(),
        };
        lines.push(match self.restart.chip() {
            Some((chip, _)) => (Level::Warn, "build", format!("{build}; {chip}")),
            None => (Level::Ok, "build", build),
        });
        lines.push(match self.watch.line() {
            Some((line, true)) => (Level::Warn, "signals", line),
            Some((line, false)) => (Level::Ok, "signals", line),
            None => (
                Level::Note,
                "signals",
                "not started (a screenshot run or a test)".to_owned(),
            ),
        });
        // What the start-up reconciliation closed (EP-28a).
        lines.push(match self.watch.reconcile_line() {
            Some((line, true)) => (Level::Warn, "reconciliation", line),
            Some((line, false)) => (Level::Ok, "reconciliation", line),
            None => (
                Level::Note,
                "reconciliation",
                "not run (a screenshot run or a test)".to_owned(),
            ),
        });
        let (notifications, warns) = self.notifications_state();
        let level = if warns { Level::Warn } else { Level::Ok };
        lines.push((level, "notifications", notifications));
        let (state, warns) = self.meter.state(now);
        let level = if warns { Level::Warn } else { Level::Ok };
        lines.push((level, "usage meter", state.replace('\n', " ")));
        let serving = matches!(self.watch.listening, Some(Listening::On(_)));
        lines.push(match (serving, self.mail.last_round()) {
            (false, _) => (
                Level::Note,
                "mailman",
                "this window holds no loopback port, so it runs no round".to_owned(),
            ),
            (true, Some(at)) => (
                Level::Ok,
                "mailman",
                format!(
                    "the last round {}; one every 5 minutes and after a run",
                    ago(at)
                ),
            ),
            (true, None) => (
                Level::Note,
                "mailman",
                "no round yet: the first waits for the poll of live sessions and the files"
                    .to_owned(),
            ),
        });
        lines.push(self.headless_line());
        lines.push(self.push_guard_line());
        // A screenshot run names a neutral path: a picture never shows the machine's (rule 3).
        let beside = if self.fixtures {
            Some(PathBuf::from(r"C:\Tools\Roadmap Wrangler\crw.exe"))
        } else {
            crate::board::crw_exe()
        };
        lines.push(match beside {
            Some(exe) => (
                Level::Ok,
                "defer route",
                format!("its hooks run {}", exe.display()),
            ),
            None => (
                Level::Warn,
                "defer route",
                format!(
                    "no crw.exe beside the window, so the route is refused: {}",
                    process::GONE
                ),
            ),
        });
        lines.push(match crw_core::log::folder() {
            Some(dir) => {
                let today = now.with_timezone(&Local).date_naive();
                let file = dir.join(crw_core::log::file_name(today));
                (Level::Ok, "log", file.display().to_string())
            }
            None => (
                Level::Note,
                "log",
                "none in a screenshot run or a test".to_owned(),
            ),
        });
        lines
    }

    /// The push guard an autopilot run is passed (D-22; EP-42), read back from the launch the
    /// autopilot builds with the window's own preflight's flags; `crw doctor` prints the same.
    fn push_guard_line(&self) -> WindowLine {
        let Some(preflight) = self.sessions.preflight_ready() else {
            return (
                Level::Note,
                doctor::PUSH_GUARD,
                "not read yet in this window: the launch dialog reads the CLI's flags".to_owned(),
            );
        };
        match &preflight.flags {
            Ok(flags) => {
                let (level, text) = doctor::push_guard(flags);
                (level, doctor::PUSH_GUARD, text)
            }
            Err(e) => (
                Level::Warn,
                doctor::PUSH_GUARD,
                format!("the CLI's flags did not read: {e}"),
            ),
        }
    }

    /// The headless runs' readiness and route, from the window's own preflight.
    fn headless_line(&self) -> WindowLine {
        let Some(preflight) = self.sessions.preflight_ready() else {
            return (
                Level::Note,
                "headless",
                "not read yet in this window: the launch dialog reads it".to_owned(),
            );
        };
        let refusals = preflight.refusals();
        if !refusals.is_empty() {
            return (
                Level::Warn,
                "headless",
                format!("refused: {}", refusals.join("; ")),
            );
        }
        let route = match self
            .sessions
            .route_for(&preflight, self.diagnostics.setting)
        {
            (Route::Protocol, _) => {
                "questions wait in the Decisions tile (the control protocol)".to_owned()
            }
            (Route::Defer, why) => format!(
                "the defer route: a run ends at a question and the answer resumes it{}",
                why.map_or_else(String::new, |why| format!(" ({why})"))
            ),
        };
        match &preflight.flags {
            Ok(flags) if flags.prompt_tool() => (Level::Ok, "headless", format!("ready; {route}")),
            Ok(_) => (
                Level::Warn,
                "headless",
                "ready, but the CLI's help names no --permission-prompt-tool: a run's questions \
                 are disabled and its permission requests denied"
                    .to_owned(),
            ),
            Err(e) => (
                Level::Warn,
                "headless",
                format!("the CLI's flags did not read: {e}"),
            ),
        }
    }
}

/// How long ago `at` was, in words.
fn ago(at: Instant) -> String {
    let age = TimeDelta::from_std(at.elapsed()).unwrap_or_default();
    match short_age(age).as_str() {
        "now" => "just now".to_owned(),
        age => format!("{age} ago"),
    }
}

/// A level's colour: the window's own green, amber and red, a note in the weak text's.
fn colour(ui: &egui::Ui, level: Level) -> Color32 {
    match level {
        Level::Ok => style::green(ui),
        Level::Note => ui.visuals().weak_text_color(),
        Level::Warn => style::amber(ui),
        Level::Fail => style::red(ui),
    }
}

/// One line as one wrapped text: the level in its colour, the name, the words; a narrow pane
/// wraps it rather than widening (EP-13's trap). The parts are apart by real spaces, where a
/// line may break: a section's leading space is none, which left the level alone on its row.
fn line_ui(ui: &mut egui::Ui, level: Level, name: &str, text: &str) {
    let body = egui::TextStyle::Body.resolve(ui.style());
    let mono = egui::TextStyle::Monospace.resolve(ui.style());
    let mut job = LayoutJob::default();
    job.append(
        &format!("{:<4} ", level.word()),
        0.0,
        TextFormat {
            font_id: mono,
            color: colour(ui, level),
            ..Default::default()
        },
    );
    job.append(
        &format!("{name}  "),
        0.0,
        TextFormat {
            font_id: body.clone(),
            color: ui.visuals().strong_text_color(),
            ..Default::default()
        },
    );
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: body,
            color: ui.visuals().text_color(),
            ..Default::default()
        },
    );
    ui.add(egui::Label::new(job).wrap());
}

/// A screenshot run's and the tests' report: every group, each level, neutral paths.
#[cfg(any(test, feature = "screenshot"))]
pub fn canned() -> Report {
    use crw_core::doctor::Line;
    let line = |group, name: &str, level, text: &str| Line {
        group,
        name: name.to_owned(),
        level,
        text: text.to_owned(),
    };
    let tool = r"C:\Tools\Roadmap Wrangler";
    Report {
        lines: vec![
            line(
                Group::Tools,
                "claude",
                Level::Ok,
                r"2.1.316 at C:\Tools\claude.exe (the minimum is 2.1.261)",
            ),
            line(Group::Tools, "gh", Level::Ok, r"2.104.0 at C:\Tools\gh.exe"),
            line(
                Group::Tools,
                "extension",
                Level::Ok,
                "2.1.323 in VS Code's extensions.json; the standalone CLI is 2.1.316",
            ),
            line(
                Group::Tools,
                "protocol",
                Level::Ok,
                "claude-codes 2.1.316, pinned: it moves only in a re-plan (D-15)",
            ),
            line(
                Group::Exes,
                "installed",
                Level::Ok,
                &format!("{tool}, built 2026-11-13 22:58"),
            ),
            line(
                Group::Exes,
                "start at login",
                Level::Ok,
                "starts the installed window hidden in the tray",
            ),
            line(
                Group::Exes,
                "crw on PATH",
                Level::Note,
                r"not on PATH: crw path on puts a copy in C:\Tools\bin",
            ),
            line(
                Group::Exes,
                "SessionStart",
                Level::Ok,
                "runs the installed copy hook --inbox",
            ),
            line(
                Group::Exes,
                "Stop",
                Level::Fail,
                &format!(
                    r"runs {tool}\old\crw.exe hook, which is not there: {}; crw signals install points the entries at the installed copy",
                    process::GONE
                ),
            ),
            line(
                Group::Accounts,
                "gh",
                Level::Ok,
                "logged in to github.com as someone; the token lacks delete_repo, so an orphan's \
                 Delete is refused until gh auth refresh -s delete_repo",
            ),
            line(
                Group::Network,
                "tailscale",
                Level::Warn,
                "Stopped: the tailnet is not up",
            ),
            line(
                Group::Backup,
                "drive",
                Level::Ok,
                r"X:\ is mounted, labelled SPARE as the job expects",
            ),
            line(
                Group::Records,
                "ledger",
                Level::Ok,
                r"C:\Tools\ledger.sqlite: schema 7, sound by SQLite's quick check, 640 KB",
            ),
            line(Group::Records, "crash log", Level::Ok, "no report"),
            line(Group::Readings, "usage", Level::Ok, "read 2 min ago"),
            line(
                Group::Readings,
                "status line",
                Level::Note,
                "no status line's figures among the latest samples",
            ),
            line(
                Group::VsCode,
                "lock files",
                Level::Note,
                r"3 lock files in C:\Tools\ide: 1 live, 2 stale; the extension leaves a stale one when VS Code is killed, and the tool never deletes them",
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::board::{Places, View as BoardView};
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::testing::{Screen, text_containing};

    /// The tile draws a canned report's groups and lines, the window's own lines above, and a
    /// test's board gathers nothing itself (rule 5).
    #[test]
    fn the_tile_draws_the_report_and_the_windows_lines() {
        let scratch = Scratch::new("diagnostics");
        let places = Places {
            store: fake_store(&scratch),
            ledger: Some(scratch.0.join("ledger.sqlite")),
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        let mut board = Board::new(
            places,
            &ctx,
            BoardView::default(),
            egui::ThemePreference::Light,
        );
        // A first pass with no report asks for one, which a test's board never gathers.
        let mut screen = Screen::new();
        screen.pass(Vec::new(), |ui| board.diagnostics_pane(ui));
        assert!(!board.diagnostics.live, "a test's board starts no child");
        assert!(!board.diagnostics.checking());
        assert!(board.diagnostics.report().is_none());
        board.diagnostics.can(canned(), Utc::now());
        screen.pass(Vec::new(), |ui| board.diagnostics_pane(ui));
        let output = screen.pass(Vec::new(), |ui| board.diagnostics_pane(ui));
        for words in [
            "This window",
            "Checks",
            "Backup drive",
            "VS Code",
            "which is not there",
            "never deletes them",
        ] {
            assert!(text_containing(&output, words).is_some(), "{words}");
        }
        assert!(!board.diagnostics.checking());
        let names: Vec<&str> = board
            .window_lines(Utc::now())
            .iter()
            .map(|(_, name, _)| *name)
            .collect();
        assert_eq!(
            names,
            [
                "build",
                "signals",
                "reconciliation",
                "notifications",
                "usage meter",
                "mailman",
                "headless",
                "push guard",
                "defer route",
                "log"
            ]
        );
        // Before the window has read the CLI's flags, the push guard's line says so (EP-42).
        let lines = board.window_lines(Utc::now());
        let guard = lines
            .iter()
            .find(|(_, name, _)| *name == doctor::PUSH_GUARD)
            .unwrap();
        assert_eq!(guard.0, Level::Note);
        assert!(guard.2.contains("not read yet"), "{}", guard.2);
    }
}
