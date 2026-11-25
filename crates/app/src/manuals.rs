//! A project's manual from the window (D-12 addendum; EP-57a): a row's menu and the Detail pane's
//! head offer Manual and Changes since the last phase, which build the PDF through
//! `crw_core::manual` on a worker thread when its source, its pictures or the window's exe are
//! newer than the PDF, then open it with the system's viewer, the first file the window opens. One
//! build at a time; a screenshot run builds and opens nothing. The Status section's Manual line
//! dates the manual against the last phase's close.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crw_core::manual::{self, Built, Job, Runner, Which};
use eframe::egui;

use crate::portfolio::ProjectRow;

/// How often the window looks for the build's answer while it waits.
const POLL: Duration = Duration::from_millis(250);

/// The two entries, in the order the menu and the Detail pane show them.
pub const ENTRIES: [Which; 2] = [Which::Manual, Which::Changes];

/// What the entries weigh beyond the row: whether Typst was found, and whether a build is on its
/// way (one at a time).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct State {
    pub typst: bool,
    pub busy: bool,
}

/// An entry's words.
pub fn label(which: Which) -> &'static str {
    match which {
        Which::Manual => "Manual",
        Which::Changes => "Changes since the last phase",
    }
}

/// An entry's hover while it can act.
pub fn hint(which: Which) -> &'static str {
    match which {
        Which::Manual => {
            "Build the manual's PDF from docs/manual.md when that, a picture or this build of the \
             window is newer, then open it in your PDF viewer"
        }
        Which::Changes => {
            "Build the PDF of what changed since the last phase's close from \
             docs/manual-changes.md when it is stale, then open it in your PDF viewer"
        }
    }
}

/// Whether an entry can act, or why it is greyed, in words for its hover.
pub fn availability(row: &ProjectRow, which: Which, state: State) -> Result<(), String> {
    let name = row.name();
    if row.folder.is_none() {
        return Err("The project's folder is missing".to_owned());
    }
    let present = row.snapshot.manual;
    if present.manual.is_none() {
        return Err(format!(
            "{name} owes no manual: it has no {} (D-12)",
            Which::Manual.source()
        ));
    }
    if !present.has(which) {
        return Err(format!(
            "{name} has {} but no {}: the session that settled its manual creates both",
            Which::Manual.source(),
            which.source()
        ));
    }
    if !state.typst {
        return Err(format!(
            "Typst is not found on PATH or in winget's Links folder: install it with {}, then \
             press F5",
            manual::INSTALL
        ));
    }
    if state.busy {
        return Err("A manual is being built: a moment".to_owned());
    }
    Ok(())
}

/// What a build and its open did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    pub pdf: PathBuf,
    /// Built now: the date it carries and the warnings; `None` when it was up to date.
    pub built: Option<(String, Vec<String>)>,
}

/// The worker's work, apart for the tests' fakes: `job` built through `runner` (no child at all
/// when the PDF is fresh), then the PDF handed to `open`.
pub fn build_and_open(
    job: &Job,
    runner: &dyn Runner,
    open: &dyn Fn(&Path) -> Result<(), String>,
) -> Result<Opened, String> {
    let opened = match manual::build(job, runner)? {
        Built::Fresh { pdf } => Opened { pdf, built: None },
        Built::Wrote {
            pdf,
            date,
            warnings,
            ..
        } => Opened {
            pdf,
            built: Some((date, warnings)),
        },
    };
    open(&opened.pdf).map_err(|why| {
        format!(
            "{} is in place, but Explorer could not hand it to your viewer: {why}",
            opened.pdf.display()
        )
    })?;
    Ok(opened)
}

/// What became of a build, for the status bar: its first line there, the whole in the rolling
/// log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Opened(String),
    Refused(String),
}

/// The words of a build's answer.
pub fn words(project: &str, which: Which, answer: Result<Opened, String>) -> Outcome {
    match answer {
        Ok(Opened { pdf, built }) => {
            let file = pdf.file_name().map_or_else(
                || pdf.display().to_string(),
                |name| name.to_string_lossy().into_owned(),
            );
            match built {
                None => Outcome::Opened(format!(
                    "{project}: {file} is up to date and was handed to your PDF viewer"
                )),
                Some((date, warnings)) => {
                    let mut text = format!(
                        "{project}: {file} built from {}, dated {date}, and handed to your PDF \
                         viewer",
                        which.source()
                    );
                    if !warnings.is_empty() {
                        text.push_str(&format!(
                            "; {} warning{} in the rolling log",
                            warnings.len(),
                            if warnings.len() == 1 { "" } else { "s" }
                        ));
                        for warning in warnings {
                            text.push_str(&format!("\n  warning: {warning}"));
                        }
                    }
                    Outcome::Opened(text)
                }
            }
        }
        Err(why) => Outcome::Refused(refused(project, which, &why)),
    }
}

/// Why an entry did nothing, as the status bar says it.
pub fn refused(project: &str, which: Which, why: &str) -> String {
    format!("{project}, {}: {why}", label(which))
}

/// A build on its way.
struct Building {
    project: String,
    which: Which,
    receiver: Receiver<Result<Opened, String>>,
}

/// The window's manuals: at most one build on its way, and whether Typst was found.
pub struct Manuals {
    building: Option<Building>,
    /// Typst where the engine looks for it: at the window's start, after each build, on F5 and on
    /// a row's Refresh this project.
    typst: bool,
    /// A screenshot run or a test: Typst taken as found, nothing built or opened.
    pub canned: bool,
}

impl Manuals {
    pub fn new() -> Self {
        Self {
            building: None,
            typst: manual::program().is_some(),
            canned: cfg!(test),
        }
    }

    /// Looks for Typst again, a PATH scan with no child: after its install the entries act.
    pub fn look_for_typst(&mut self) {
        self.typst = manual::program().is_some();
    }

    /// What the entries weigh.
    pub fn state(&self) -> State {
        State {
            typst: self.typst || self.canned,
            busy: self.building.is_some(),
        }
    }

    /// Starts the build of `which` for `project` in `folder` on a worker thread, then its open;
    /// `Err` with the words when it cannot start.
    pub fn start(
        &mut self,
        project: &str,
        folder: &Path,
        which: Which,
        ctx: &egui::Context,
    ) -> Result<(), String> {
        if self.building.is_some() {
            return Err("A manual is being built: a moment".to_owned());
        }
        if self.canned {
            return Err("a screenshot run or a test builds and opens no manual".to_owned());
        }
        let out =
            manual::dir().ok_or_else(|| "no profile folder: LOCALAPPDATA is not set".to_owned())?;
        // A new build of the window may write the template otherwise: its PDFs are built again
        // once, as `crw manual` does with its own exe.
        let template = std::env::current_exe()
            .and_then(std::fs::metadata)
            .and_then(|meta| meta.modified())
            .ok();
        let (sender, receiver) = mpsc::channel();
        let (name, folder, ctx) = (project.to_owned(), folder.to_owned(), ctx.clone());
        thread::Builder::new()
            .name("manual".to_owned())
            .spawn(move || {
                let job = Job {
                    project: &name,
                    folder: &folder,
                    which,
                    out: &out,
                    force: false,
                    template,
                };
                let _ = sender.send(build_and_open(&job, &manual::Real::locate(), &manual::open));
                ctx.request_repaint();
            })
            .map_err(|e| format!("the manual's thread did not start: {e}"))?;
        self.building = Some(Building {
            project: project.to_owned(),
            which,
            receiver,
        });
        Ok(())
    }

    /// Takes in a build's answer when it has come; asks for frames while one waits.
    pub fn receive(&mut self, ctx: &egui::Context) -> Option<Outcome> {
        let building = self.building.as_ref()?;
        let answer = match building.receiver.try_recv() {
            Ok(answer) => answer,
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(POLL);
                return None;
            }
            Err(TryRecvError::Disconnected) => {
                Err("the manual's thread ended without an answer".to_owned())
            }
        };
        let building = self.building.take()?;
        // Typst may have come or gone meanwhile: the entries weigh it again.
        self.look_for_typst();
        Some(words(&building.project, building.which, answer))
    }

    /// A test's build on its way, its answer to come through `receiver`.
    #[cfg(test)]
    fn waiting(&mut self, project: &str, which: Which) -> mpsc::Sender<Result<Opened, String>> {
        let (sender, receiver) = mpsc::channel();
        self.building = Some(Building {
            project: project.to_owned(),
            which,
            receiver,
        });
        sender
    }
}

impl Default for Manuals {
    fn default() -> Self {
        Self::new()
    }
}

/// The Status section's Manual line (EP-57a): the manual's date beside the last phase's close,
/// and whether that phase closed after it (amber); `None` for a project that owes no manual.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dated {
    pub text: String,
    pub stale: bool,
}

/// The line for `row`: the manual dated by its last commit while the file is as the git pass saw
/// it, else by its own time (an edit, a file git does not track, a folder that is no repository);
/// a repository the git pass has not reached says so, as does a time that could not be read.
pub fn dated(row: &ProjectRow) -> Option<Dated> {
    let own = row.snapshot.manual.manual?;
    if row.awaits_git() {
        return Some(Dated {
            text: "docs/manual.md is there; git has not been asked its date yet".to_owned(),
            stale: false,
        });
    }
    let committed = row
        .git
        .as_ref()
        .and_then(|git| git.reading.manual)
        .filter(|commit| own <= commit.file);
    let stamp = committed.map_or(own, |commit| commit.at);
    if stamp <= 0 {
        return Some(Dated {
            text: "docs/manual.md is there; its time could not be read".to_owned(),
            stale: false,
        });
    }
    Some(dated_words(
        &manual::date_of(stamp),
        committed.is_some(),
        row.snapshot.last_close(),
    ))
}

/// The words of the Manual line from the manual's date (its last commit when `committed`) and
/// the last phase's close: amber when the phase closed after the manual's date, since the
/// re-plan that closed it folds the changes into the manual (D-12 addendum).
pub fn dated_words(date: &str, committed: bool, close: Option<(String, Option<String>)>) -> Dated {
    let by = if committed {
        "its last commit"
    } else {
        "the file's own time"
    };
    let manual = format!("docs/manual.md dated {date} ({by})");
    match close {
        None => Dated {
            text: format!("{manual}; no phase has closed yet"),
            stale: false,
        },
        Some((phase, None)) => Dated {
            text: format!("{manual}; {phase} closed last, undated"),
            stale: false,
        },
        Some((phase, Some(closed))) if closed.as_str() > date => Dated {
            text: format!(
                "{manual}, older than {phase}'s close on {closed}: the changes since are not \
                 folded in"
            ),
            stale: true,
        },
        Some((phase, Some(closed))) => Dated {
            text: format!("{manual}; {phase} closed on {closed}"),
            stale: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::ffi::OsString;
    use std::fs;

    use crw_core::manual::Present;
    use crw_core::process::Exit;

    use crate::portfolio::tests::{Scratch, fake_store};

    /// A Typst and a git that answer as told and count their calls; a compile writes its PDF.
    #[derive(Default)]
    struct Fake {
        calls: RefCell<Vec<String>>,
        version: Option<String>,
    }

    impl Runner for Fake {
        fn version(&self) -> Result<String, String> {
            self.calls.borrow_mut().push("version".to_owned());
            self.version
                .clone()
                .ok_or_else(|| "Typst is not found".to_owned())
        }

        fn compile(&self, args: &[OsString], _input: &str) -> Result<Exit, String> {
            self.calls.borrow_mut().push("compile".to_owned());
            fs::write(args.last().unwrap(), b"%PDF-1.7 fake").unwrap();
            Ok(Exit {
                code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            })
        }

        fn changed(&self, _folder: &Path, _path: &str) -> bool {
            false
        }

        fn committed(&self, _folder: &Path, _path: &str) -> Option<i64> {
            Some(1_764_000_000)
        }
    }

    /// A synthetic project `alpha` with `files` in its `docs`, and a manuals folder beside it.
    fn project(scratch: &Scratch, files: &[(&str, &str)]) -> (PathBuf, PathBuf) {
        let folder = scratch.0.join("alpha");
        fs::create_dir_all(folder.join("docs")).unwrap();
        for (name, text) in files {
            fs::write(folder.join("docs").join(name), text).unwrap();
        }
        (folder, scratch.0.join("manuals"))
    }

    fn job<'a>(folder: &'a Path, out: &'a Path, which: Which) -> Job<'a> {
        Job {
            project: "alpha",
            folder,
            which,
            out,
            force: false,
            template: None,
        }
    }

    /// The fake portfolio's harbor, its manual's files as `present` says.
    fn harbor(scratch: &Scratch, present: Present) -> ProjectRow {
        let portfolio = crate::portfolio::read_files(&fake_store(scratch));
        let mut row = portfolio.project("harbor").unwrap().clone();
        row.snapshot.manual = present;
        row
    }

    const READY: State = State {
        typst: true,
        busy: false,
    };

    #[test]
    fn an_entry_is_greyed_with_its_reason() {
        let scratch = Scratch::new("manuals-availability");
        let both = Present {
            manual: Some(1),
            changes: true,
        };
        let row = harbor(&scratch, both);
        assert_eq!(availability(&row, Which::Manual, READY), Ok(()));
        assert_eq!(availability(&row, Which::Changes, READY), Ok(()));

        // No docs/manual.md: it owes none, and neither entry acts.
        let none = harbor(&scratch, Present::default());
        for which in ENTRIES {
            let why = availability(&none, which, READY).unwrap_err();
            assert!(
                why.contains("owes no manual") && why.contains("docs/manual.md"),
                "{why}"
            );
        }
        // The manual without its changes file: Manual acts, Changes says what is missing.
        let manual_only = harbor(
            &scratch,
            Present {
                manual: Some(1),
                changes: false,
            },
        );
        assert_eq!(availability(&manual_only, Which::Manual, READY), Ok(()));
        let why = availability(&manual_only, Which::Changes, READY).unwrap_err();
        assert!(why.contains("no docs/manual-changes.md"), "{why}");

        // Typst missing: the winget line.
        let no_typst = State {
            typst: false,
            busy: false,
        };
        let why = availability(&row, Which::Manual, no_typst).unwrap_err();
        assert!(
            why.contains(manual::INSTALL) && why.ends_with("then press F5"),
            "{why}"
        );
        // A build on its way: one at a time.
        let busy = State {
            typst: true,
            busy: true,
        };
        assert!(availability(&row, Which::Manual, busy).is_err());

        // A missing folder comes first.
        let mut gone = row.clone();
        gone.folder = None;
        let why = availability(&gone, Which::Manual, READY).unwrap_err();
        assert!(why.contains("folder is missing"), "{why}");
    }

    #[test]
    fn a_fresh_pdf_opens_with_no_child_and_a_stale_one_is_built_first() {
        let scratch = Scratch::new("manuals-build-open");
        let (folder, out) = project(&scratch, &[("manual.md", "# Alpha\n\n## One\n\nText.\n")]);
        let opened = RefCell::new(Vec::new());
        let open = |pdf: &Path| {
            opened.borrow_mut().push(pdf.to_owned());
            Ok(())
        };
        let typst = Fake {
            version: Some("typst 0.15.1 (9dfd3a08)".to_owned()),
            ..Fake::default()
        };

        // Stale (no PDF yet): built, then opened.
        let first = build_and_open(&job(&folder, &out, Which::Manual), &typst, &open).unwrap();
        assert_eq!(first.pdf, out.join("alpha-manual.pdf"));
        assert!(first.built.is_some(), "{first:?}");
        assert_eq!(*typst.calls.borrow(), ["version", "compile"]);
        assert_eq!(*opened.borrow(), [out.join("alpha-manual.pdf")]);

        // Fresh: opened again, no child asked.
        typst.calls.borrow_mut().clear();
        let second = build_and_open(&job(&folder, &out, Which::Manual), &typst, &open).unwrap();
        assert_eq!(second.built, None);
        assert!(
            typst.calls.borrow().is_empty(),
            "{:?}",
            typst.calls.borrow()
        );
        assert_eq!(opened.borrow().len(), 2);
    }

    #[test]
    fn a_refused_build_opens_nothing_and_a_failed_open_is_said() {
        let scratch = Scratch::new("manuals-refused");
        let (folder, out) = project(&scratch, &[("manual.md", "# Alpha\n")]);
        let opened = RefCell::new(0);
        let open = |_: &Path| {
            *opened.borrow_mut() += 1;
            Ok(())
        };
        // Typst at another version: the engine's refusal, nothing opened.
        let other = Fake {
            version: Some("typst 0.16.0 (abc)".to_owned()),
            ..Fake::default()
        };
        let why = build_and_open(&job(&folder, &out, Which::Manual), &other, &open).unwrap_err();
        assert!(why.contains(manual::INSTALL), "{why}");
        // No changes file: refused before any child.
        let why = build_and_open(&job(&folder, &out, Which::Changes), &other, &open).unwrap_err();
        assert!(why.contains("docs/manual-changes.md"), "{why}");
        assert_eq!(*opened.borrow(), 0);

        // The viewer's start failing is said with the PDF's path.
        let typst = Fake {
            version: Some("typst 0.15.1 (9dfd3a08)".to_owned()),
            ..Fake::default()
        };
        let refuse = |_: &Path| Err("did not start: access was denied".to_owned());
        let why = build_and_open(&job(&folder, &out, Which::Manual), &typst, &refuse).unwrap_err();
        assert!(
            why.contains(
                "alpha-manual.pdf is in place, but Explorer could not hand it to your viewer: \
                 did not start"
            ),
            "{why}"
        );
    }

    /// One build at a time, none in a test or a screenshot run; an answer, or a thread that
    /// ended without one, frees the next.
    #[test]
    fn a_build_goes_one_at_a_time_and_its_answer_frees_the_next() {
        let ctx = egui::Context::default();
        let mut manuals = Manuals::new();
        assert!(manuals.canned, "a test builds nothing");
        let why = manuals
            .start("alpha", Path::new(r"C:\Data\alpha"), Which::Manual, &ctx)
            .unwrap_err();
        assert!(why.contains("builds and opens no manual"), "{why}");
        assert!(!manuals.state().busy);

        let sender = manuals.waiting("alpha", Which::Manual);
        assert!(manuals.state().busy);
        let why = manuals
            .start("alpha", Path::new(r"C:\Data\alpha"), Which::Changes, &ctx)
            .unwrap_err();
        assert!(why.contains("being built"), "{why}");
        assert_eq!(manuals.receive(&ctx), None, "no answer yet");
        sender
            .send(Ok(Opened {
                pdf: PathBuf::from(r"C:\Local\manuals\alpha-manual.pdf"),
                built: None,
            }))
            .unwrap();
        assert!(matches!(manuals.receive(&ctx), Some(Outcome::Opened(_))));
        assert!(!manuals.state().busy);

        // A thread that ended without an answer is a refusal, and frees the next.
        drop(manuals.waiting("alpha", Which::Changes));
        assert_eq!(
            manuals.receive(&ctx),
            Some(Outcome::Refused(
                "alpha, Changes since the last phase: the manual's thread ended without an answer"
                    .to_owned()
            ))
        );
        assert!(!manuals.state().busy);
        assert_eq!(manuals.receive(&ctx), None);
    }

    #[test]
    fn the_answer_is_said_in_one_line_with_the_warnings_after_it() {
        let pdf = PathBuf::from(r"C:\Local\manuals\alpha-manual.pdf");
        let fresh = words(
            "alpha",
            Which::Manual,
            Ok(Opened {
                pdf: pdf.clone(),
                built: None,
            }),
        );
        assert_eq!(
            fresh,
            Outcome::Opened(
                "alpha: alpha-manual.pdf is up to date and was handed to your PDF viewer"
                    .to_owned()
            )
        );
        let Outcome::Opened(built) = words(
            "alpha",
            Which::Manual,
            Ok(Opened {
                pdf,
                built: Some((
                    "2026-11-25".to_owned(),
                    vec!["raw HTML at line 4".to_owned()],
                )),
            }),
        ) else {
            panic!("opened");
        };
        let mut lines = built.lines();
        assert_eq!(
            lines.next(),
            Some(
                "alpha: alpha-manual.pdf built from docs/manual.md, dated 2026-11-25, and handed \
                 to your PDF viewer; 1 warning in the rolling log"
            )
        );
        assert_eq!(lines.next(), Some("  warning: raw HTML at line 4"));
        assert_eq!(
            words("alpha", Which::Changes, Err("Typst is missing".to_owned())),
            Outcome::Refused("alpha, Changes since the last phase: Typst is missing".to_owned())
        );
    }

    #[test]
    fn the_manual_line_is_amber_when_a_phase_closed_after_it() {
        let close = |date: Option<&str>| Some(("P5".to_owned(), date.map(str::to_owned)));
        let stale = dated_words("2026-11-10", true, close(Some("2026-11-17")));
        assert!(stale.stale, "{stale:?}");
        assert_eq!(
            stale.text,
            "docs/manual.md dated 2026-11-10 (its last commit), older than P5's close on \
             2026-11-17: the changes since are not folded in"
        );
        // The same day, or later: the re-plan folded the changes in.
        for date in ["2026-11-17", "2026-11-20"] {
            let held = dated_words(date, true, close(Some("2026-11-17")));
            assert!(!held.stale, "{held:?}");
            assert!(held.text.ends_with("P5 closed on 2026-11-17"), "{held:?}");
        }
        let undated = dated_words("2026-11-10", false, close(None));
        assert!(!undated.stale);
        assert_eq!(
            undated.text,
            "docs/manual.md dated 2026-11-10 (the file's own time); P5 closed last, undated"
        );
        let none = dated_words("2026-11-10", true, None);
        assert!(!none.stale && none.text.ends_with("no phase has closed yet"));
    }

    #[test]
    fn the_line_dates_the_manual_by_git_while_committed_else_by_the_file() {
        let scratch = Scratch::new("manuals-dated");
        let own = 1_763_000_000; // 2025-11-13
        let mut row = harbor(
            &scratch,
            Present {
                manual: Some(own),
                changes: false,
            },
        );
        assert_eq!(dated(&harbor(&scratch, Present::default())), None);
        // Not a repository: the file's own time.
        row.folder.as_mut().unwrap().git = false;
        row.git = None;
        let by_file = format!(
            "docs/manual.md dated {} (the file's own time)",
            manual::date_of(own)
        );
        let line = dated(&row).unwrap();
        assert!(line.text.starts_with(&by_file), "{line:?}");
        // A repository the git pass has not reached says so.
        row.folder.as_mut().unwrap().git = true;
        assert!(dated(&row).unwrap().text.contains("git has not been asked"));
        // Its commit once read, while the file is as git saw it.
        let mut git = crate::portfolio::Git::default();
        git.reading.manual = Some(crw_core::roadmap::snapshot::ManualCommit {
            at: 1_764_000_000,
            file: own,
        });
        row.git = Some(git);
        let line = dated(&row).unwrap();
        assert!(
            line.text.starts_with(&format!(
                "docs/manual.md dated {} (its last commit)",
                manual::date_of(1_764_000_000)
            )),
            "{line:?}"
        );
        // An edit since the git pass read it: the file's own time until git is asked again.
        row.snapshot.manual.manual = Some(own + 60);
        let line = dated(&row).unwrap();
        assert!(
            line.text.starts_with(&format!(
                "docs/manual.md dated {} (the file's own time)",
                manual::date_of(own + 60)
            )),
            "{line:?}"
        );
        // A time that could not be read says so, never amber.
        row.git = None;
        row.folder.as_mut().unwrap().git = false;
        row.snapshot.manual.manual = Some(0);
        let line = dated(&row).unwrap();
        assert!(
            !line.stale && line.text.contains("could not be read"),
            "{line:?}"
        );
    }
}
