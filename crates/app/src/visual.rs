//! The visual launch from the window (D-16, EP-11): VS Code opened on the project's own workspace
//! file and the prompt filled into a Claude Code tab there, by `crw_core::visual` on a worker
//! thread, since a window that is not open yet takes seconds to start. The window never holds the
//! session: the owner sends the prompt in VS Code, and the ledger keeps the launch.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crw_core::discovery::OwnFile;
use crw_core::version::Version;
use crw_core::visual::{self, Launch, Opened, Readiness};
use eframe::egui;

/// How often the window looks for the launch's answer while it waits.
const POLL: Duration = Duration::from_millis(250);

/// A visual launch on its way.
struct Opening {
    receiver: Receiver<Result<Opened, String>>,
    cancel: Arc<AtomicBool>,
    project: String,
    ep: String,
}

/// What became of a visual launch.
pub enum Outcome {
    Opened(String),
    Refused(String),
}

/// The window's visual launches: at most one on its way.
#[derive(Default)]
pub struct Visual {
    opening: Option<Opening>,
    /// A screenshot run's stand-in for VS Code, its `Code.exe` and the extension's version, with
    /// no window open: no path of the machine shows, and nothing is opened.
    pub canned: Option<(PathBuf, Version)>,
}

impl Visual {
    /// What a visual launch of `folder` would meet; `setting` is the registry's `vscode`.
    pub fn readiness(&self, folder: &Path, setting: Option<&Path>) -> Readiness {
        match &self.canned {
            Some((code, extension)) => {
                Readiness::of(folder, Some(code.clone()), Some(*extension), &[])
            }
            None => visual::readiness(folder, setting),
        }
    }

    pub fn is_opening(&self) -> bool {
        self.opening.is_some()
    }

    /// Starts `launch` on a worker thread.
    pub fn start(&mut self, launch: Launch, ctx: &egui::Context) {
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let (project, ep) = (
            launch.project.clone(),
            launch.ep.clone().unwrap_or_default(),
        );
        let (flag, ctx) = (Arc::clone(&cancel), ctx.clone());
        let spawned = thread::Builder::new()
            .name("visual launch".to_owned())
            .spawn(move || {
                let _ = sender.send(visual::open(&launch, &flag));
                ctx.request_repaint();
            });
        if spawned.is_ok() {
            self.opening = Some(Opening {
                receiver,
                cancel,
                project,
                ep,
            });
        }
    }

    /// Stops a launch that waits for its window: nothing is handed to VS Code after this, and
    /// its answer is not waited for.
    pub fn cancel(&mut self) {
        if let Some(opening) = self.opening.take() {
            opening.cancel.store(true, Ordering::Relaxed);
        }
    }

    /// Takes in a launch's answer when it has come; asks for frames while one waits.
    pub fn receive(&mut self, ctx: &egui::Context) -> Option<Outcome> {
        let opening = self.opening.as_ref()?;
        let outcome = match opening.receiver.try_recv() {
            Ok(Ok(opened)) => Outcome::Opened(said(&opening.project, &opening.ep, &opened)),
            Ok(Err(why)) => Outcome::Refused(format!("VS Code did not get the prompt: {why}")),
            Err(TryRecvError::Disconnected) => {
                Outcome::Refused("the visual launch's thread ended without an answer".to_owned())
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(POLL);
                return None;
            }
        };
        self.opening = None;
        Some(outcome)
    }
}

/// The notice of a launch that went through.
fn said(project: &str, ep: &str, opened: &Opened) -> String {
    let file = opened
        .workspace
        .path()
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let window = match (&opened.workspace, opened.was_open) {
        (OwnFile::Written(_), _) => format!("{file} written beside the folder and opened"),
        (_, true) => format!("its window, {file}, brought to the front"),
        (_, false) => format!("{file} opened in {} s", opened.waited.as_secs().max(1)),
    };
    let mut text = format!(
        "{project} {ep}: the prompt waits in a new Claude Code tab in VS Code ({window}). Choose \
         the model and effort in its panel, then send it with Enter."
    );
    match &opened.ledger_problem {
        Some(problem) => text.push_str(&format!(" But {problem}.")),
        None if opened.ledger_row.is_some() => {
            text.push_str(" The ledger records the launch as visual.");
        }
        None => {}
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opened(workspace: OwnFile, was_open: bool) -> Opened {
        Opened {
            workspace,
            was_open,
            waited: Duration::from_millis(300),
            ledger_row: Some(4),
            ledger_problem: None,
        }
    }

    #[test]
    fn the_notice_says_where_the_prompt_waits() {
        let file = PathBuf::from(r"C:\Data\alpha.code-workspace");
        let open = said("alpha", "EP-3", &opened(OwnFile::Kept(file.clone()), true));
        assert_eq!(
            open,
            "alpha EP-3: the prompt waits in a new Claude Code tab in VS Code (its window, \
             alpha.code-workspace, brought to the front). Choose the model and effort in its \
             panel, then send it with Enter. The ledger records the launch as visual."
        );
        let started = said("alpha", "EP-3", &opened(OwnFile::Kept(file.clone()), false));
        assert!(
            started.contains("(alpha.code-workspace opened in 1 s)"),
            "{started}"
        );
        let mut written = opened(OwnFile::Written(file), false);
        written.ledger_row = None;
        written.ledger_problem = Some("the ledger did not record the launch: locked".to_owned());
        let text = said("alpha", "EP-3", &written);
        assert!(
            text.contains("written beside the folder and opened"),
            "{text}"
        );
        assert!(text.ends_with("But the ledger did not record the launch: locked."));
    }
}
