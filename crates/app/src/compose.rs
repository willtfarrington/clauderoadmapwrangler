//! The compose dialog (D-13; EP-20c): the owner's message to a project, written into its
//! `roadmap/inbox.md` as `- [ ] IN-n (from the owner, <date>) <text>` through the mailman's own
//! append (`mailman::send_from_owner`), on a worker thread in the process's turn, logged as the
//! tool's write (a revert of a session there redoes it, EP-15b) and in the mail log. Opened
//! from the Detail pane's Inbox section and from a row's menu. A session at work or waiting in
//! the project does not hold the owner's message, unlike a delivery (the owner's choice at
//! pickup): the dialog says so, and the next session there triages it. A screenshot run's Send
//! is refused as every write is (rule 5).

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use crw_core::files;
use crw_core::inbox::mailman::{self, Sent};
use eframe::egui::{self, RichText};

use crate::board::{Action, Board};
use crate::markup::{self, Look};
use crate::style;

/// The dialog's widest, in points.
const WIDTH: f32 = 640.0;

/// The hint in the empty text.
const HINT: &str = "What the project's next session should know; it triages the message first";

/// The warning for a message longer than the ledger keeps (EP-20c's review; EP-20d).
const TOO_LONG: &str = "Longer than the ledger keeps (1,000 characters with its id and date): a \
                        revert of a session there would name this message instead of putting it \
                        back.";

/// The compose dialog, while it is open.
pub struct Dialog {
    pub project: String,
    pub folder: PathBuf,
    pub text: String,
    pub urgent: bool,
    /// The next free id of the project's inbox when the dialog opened, for the preview; the
    /// write takes the next free id then.
    pub next_id: u32,
    /// A session works or waits in the project, in the mailman's words (the Detail pane's and
    /// the row's), which the dialog says and the write goes ahead of.
    pub busy: Option<String>,
    pub error: Option<String>,
    sending: Option<Receiver<Result<Sent, String>>>,
}

impl Dialog {
    pub fn new(project: &str, folder: PathBuf, next_id: u32, busy: Option<String>) -> Self {
        Self {
            project: project.to_owned(),
            folder,
            text: String::new(),
            urgent: false,
            next_id,
            busy,
            error: None,
            sending: None,
        }
    }

    pub fn is_sending(&self) -> bool {
        self.sending.is_some()
    }

    /// The message's words as the inbox line takes them, the flag at the end when urgent.
    pub fn words(&self) -> String {
        mailman::compose_text(&self.text, self.urgent)
    }

    /// The line as it will be written, with the preview's id.
    pub fn line(&self, today: &str) -> String {
        mailman::line(self.next_id, &mailman::owner_meta(today), &self.words())
    }

    /// A revert is open or running in the project (the review's finding): the message would be
    /// written under it and put back with the start's inbox, so Send waits for it.
    pub fn revert_open(&self) -> bool {
        self.busy
            .as_deref()
            .is_some_and(|busy| busy.starts_with("a revert"))
    }

    /// Whether there is a message to send: words beyond the flag, no revert open there.
    pub fn ready(&self) -> bool {
        let words = mailman::compose_text(&self.text, false);
        !mailman::is_empty_message(&words) && !self.is_sending() && !self.revert_open()
    }

    /// Whether a revert of a session there would put the message back: its line fits the
    /// ledger's cap (EP-20c's review; EP-20d).
    pub fn revert_keeps(&self, today: &str) -> bool {
        mailman::revert_keeps(
            &format!("IN-{}", self.next_id),
            &mailman::owner_meta(today),
            &self.words(),
        )
    }

    /// Starts the write on a worker thread, which wakes `ctx` when it is done.
    fn start(
        &mut self,
        backups: PathBuf,
        today: String,
        ledger: Option<PathBuf>,
        ctx: egui::Context,
    ) {
        let (tx, rx) = mpsc::channel();
        let (folder, project) = (self.folder.clone(), self.project.clone());
        let (text, urgent) = (self.text.clone(), self.urgent);
        let spawned = thread::Builder::new()
            .name("compose".to_owned())
            .spawn(move || {
                let sent = mailman::send_from_owner(
                    &folder,
                    &project,
                    &text,
                    urgent,
                    &backups,
                    &today,
                    ledger.as_deref(),
                );
                let _ = tx.send(sent);
                ctx.request_repaint();
            });
        match spawned {
            Ok(_) => {
                self.error = None;
                self.sending = Some(rx);
            }
            Err(e) => self.error = Some(format!("The message could not be sent: {e}")),
        }
    }

    /// The worker's answer, once it has come.
    fn answer(&mut self) -> Option<Result<Sent, String>> {
        let rx = self.sending.as_ref()?;
        match rx.try_recv() {
            Ok(answer) => {
                self.sending = None;
                Some(answer)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.sending = None;
                Some(Err("the write's thread ended without a word".to_owned()))
            }
        }
    }

    /// The dialog's contents: the text, the Urgent box, the preview, Send and Cancel. Returns
    /// what the owner pressed.
    pub fn ui(&mut self, ui: &mut egui::Ui, width: f32, today: &str) -> Option<Action> {
        ui.set_width(width);
        let mut action = None;
        ui.heading(format!("A message to {}", self.project));
        ui.label(
            RichText::new(format!(
                "Written into roadmap/inbox.md as IN-{} from the owner, dated {today}; the \
                 project's next session triages it first (D-13).",
                self.next_id
            ))
            .weak(),
        );
        if let Some(busy) = &self.busy {
            let words = if self.revert_open() {
                format!(
                    "{}: the message would be put back under it. Send once the revert is done.",
                    capital(busy)
                )
            } else {
                format!(
                    "{}: it has read its inbox already, so the session after it triages the \
                     message. The write is the tool's own, kept through a revert.",
                    capital(busy)
                )
            };
            ui.label(RichText::new(words).color(style::amber(ui)));
        }
        ui.add_space(4.0);
        ui.add_enabled(
            !self.is_sending(),
            egui::TextEdit::multiline(&mut self.text)
                .desired_rows(5)
                .desired_width(f32::INFINITY)
                .hint_text(HINT),
        );
        ui.add_enabled(
            !self.is_sending(),
            egui::Checkbox::new(&mut self.urgent, "Urgent"),
        )
        .on_hover_text(
            "Ends the message with [urgent]: a message so flagged notifies you when a \
                 session hands it on, never the receiving session (D-13). Written by you, it \
                 makes no popup and no push.",
        );
        ui.add_space(6.0);
        ui.separator();
        ui.label(RichText::new("Preview").strong());
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!("IN-{}", self.next_id))
                    .monospace()
                    .strong(),
            );
            if self.urgent {
                style::chip(ui, "urgent", style::red(ui));
            }
            ui.label(RichText::new(mailman::owner_meta(today)).weak());
        });
        let shown = mailman::compose_text(&self.text, false);
        if shown.is_empty() {
            ui.label(RichText::new("Nothing to send yet.").weak());
        } else {
            markup::note(
                ui,
                &shown,
                Look {
                    sentences: true,
                    ..Look::default()
                },
            );
            egui::CollapsingHeader::new("As written")
                .id_salt("compose-as-written")
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(RichText::new(self.line(today)).monospace().small())
                            .wrap(),
                    );
                });
            if !self.revert_keeps(today) {
                ui.label(RichText::new(TOO_LONG).color(style::amber(ui)));
            }
        }
        if let Some(error) = &self.error {
            ui.add_space(4.0);
            ui.label(RichText::new(error).color(style::red(ui)));
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let send = ui
                .add_enabled(self.ready(), egui::Button::new("Send"))
                .on_hover_text("Append the message to the project's roadmap/inbox.md")
                .on_disabled_hover_text(if self.is_sending() {
                    "Sending"
                } else {
                    "Type the message first"
                });
            if send.clicked() {
                action = Some(Action::Send);
            }
            if self.is_sending() {
                ui.spinner();
                ui.label(RichText::new("Sending…").weak());
            }
            if ui
                .add_enabled(!self.is_sending(), egui::Button::new("Cancel"))
                .clicked()
            {
                action = Some(Action::CloseCompose);
            }
        });
        action
    }
}

/// `words` with a capital first letter.
fn capital(words: &str) -> String {
    let mut chars = words.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

impl Board {
    /// Opens the compose dialog on `project` (D-13; EP-20c), with the next free id of its inbox
    /// for the preview and a word on a session at work there.
    pub fn open_compose(&mut self, project: &str) {
        let Some(row) = self.portfolio.project(project) else {
            self.fail(format!("{project} is not a registered project."));
            return;
        };
        let Some(folder) = row.folder.as_ref().map(|folder| folder.path.clone()) else {
            self.fail(format!("{project}: its folder is missing."));
            return;
        };
        let next_id = row
            .snapshot
            .inbox
            .as_ref()
            .map_or(1, |inbox| mailman::next_id(&inbox.messages));
        let name = row.name().to_owned();
        let busy = self.busy_projects().remove(&name.to_lowercase());
        self.compose = Some(Dialog::new(&name, folder, next_id, busy));
    }

    /// The dialog's Send: the write starts on its thread; a screenshot run refuses it.
    pub fn send_message(&mut self) {
        let fixtures = self.fixtures;
        let backups = self.store().backups.clone();
        let ledger = self.ledger().map(PathBuf::from);
        let ctx = self.ctx().clone();
        let Some(dialog) = self.compose.as_mut() else {
            return;
        };
        if dialog.is_sending() {
            return;
        }
        if fixtures {
            dialog.error = Some(format!(
                "A screenshot run writes nothing into its projects: {} is a fixture or a copy.",
                dialog.project
            ));
            return;
        }
        if dialog.revert_open() {
            dialog.error = Some("A revert is open there: send once it is done.".to_owned());
            return;
        }
        if !dialog.ready() {
            dialog.error = Some("The message is empty.".to_owned());
            return;
        }
        dialog.start(backups, files::today(), ledger, ctx);
    }

    /// The compose dialog's share of each pass: the write's answer taken in, the dialog closed
    /// once the message is written and the files read again, so the row shows it.
    pub fn compose_pass(&mut self) {
        let Some(dialog) = self.compose.as_mut() else {
            return;
        };
        let Some(answer) = dialog.answer() else {
            return;
        };
        match answer {
            Ok(sent) => {
                let project = dialog.project.clone();
                self.compose = None;
                let kept = match &sent.copy {
                    Some(copy) => format!("the file as it was is kept as {}", copy.display()),
                    None => "roadmap/inbox.md created with its header".to_owned(),
                };
                let unlogged = match &sent.unlogged {
                    Some(why) => format!(" {}; a revert would not redo it.", capital(why)),
                    None if !sent.revert_keeps => format!(" {TOO_LONG}"),
                    None => String::new(),
                };
                self.notify(format!("Sent {} to {project}; {kept}.{unlogged}", sent.id));
                self.loader.read_files();
            }
            Err(why) => dialog.error = Some(format!("Not sent: {why}")),
        }
    }

    /// The compose dialog over the window, while it is open. Escape and Cancel close it; a click
    /// beside it does not, so the text is not lost by a stray click; nothing closes it while the
    /// write runs.
    pub fn compose_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.compose.as_mut() else {
            return;
        };
        let width = WIDTH.min(ctx.content_rect().width() - 80.0);
        let today = files::today();
        let modal = egui::Modal::new(egui::Id::unique("compose-dialog"))
            .show(ctx, |ui| dialog.ui(ui, width, &today));
        let closable = !dialog.is_sending() && !modal.backdrop_response.clicked();
        match modal.inner {
            Some(action) => self.actions.push(action),
            None if closable && modal.should_close() => self.actions.push(Action::CloseCompose),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, Instant};

    use crw_core::inbox;
    use eframe::egui::PointerButton;

    use super::*;
    use crate::board::{Places, View};
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::testing::{self, Screen};

    fn board(scratch: &Scratch) -> Board {
        let places = Places {
            store: fake_store(scratch),
            ledger: Some(scratch.0.join("ledger.sqlite")),
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        Board::new(places, &ctx, View::default(), egui::ThemePreference::Light)
    }

    #[test]
    fn the_dialog_previews_the_line_and_sends_only_with_words() {
        let mut dialog = Dialog::new("harbor", PathBuf::from("harbor"), 4, None);
        assert!(!dialog.ready());
        dialog.urgent = true;
        assert!(!dialog.ready(), "the flag alone is no message");
        dialog.text = "Read the  export's\nnew column first.".to_owned();
        assert!(dialog.ready());
        assert_eq!(
            dialog.line("2026-11-10"),
            "- [ ] IN-4 (from the owner, 2026-11-10) Read the export's new column first. [urgent]"
        );
        // Drawn: Send answers a click, Cancel too.
        let mut screen = Screen::new();
        let mut pressed = Vec::new();
        let mut draw = |events: Vec<egui::Event>| {
            screen.pass(events, |ui| {
                if let Some(action) = dialog.ui(ui, 600.0, "2026-11-10") {
                    pressed.push(action);
                }
            })
        };
        let output = draw(Vec::new());
        let send = testing::text_at(&output, "Send").expect("a Send button");
        let cancel = testing::text_at(&output, "Cancel").expect("a Cancel button");
        assert!(testing::text_at(&output, "urgent").is_some(), "the chip");
        for at in [send, cancel] {
            draw(testing::moved(at));
            draw(testing::button(at, PointerButton::Primary, true));
            draw(testing::button(at, PointerButton::Primary, false));
        }
        assert_eq!(pressed, [Action::Send, Action::CloseCompose]);
    }

    #[test]
    fn a_message_longer_than_the_ledger_keeps_is_warned_of() {
        let mut dialog = Dialog::new("harbor", PathBuf::from("harbor"), 4, None);
        dialog.text = "[Urgent] !".to_owned();
        assert!(!dialog.ready(), "the flag and punctuation are no message");
        dialog.text = "word ".repeat(150);
        assert!(dialog.ready() && dialog.revert_keeps("2026-11-10"));
        let mut screen = Screen::new();
        let mut draw = |dialog: &mut Dialog| {
            screen.pass(Vec::new(), |ui| {
                dialog.ui(ui, 600.0, "2026-11-10");
            })
        };
        assert!(testing::text_at(&draw(&mut dialog), TOO_LONG).is_none());
        dialog.text = "word ".repeat(200);
        assert!(dialog.ready() && !dialog.revert_keeps("2026-11-10"));
        assert!(testing::text_at(&draw(&mut dialog), TOO_LONG).is_some());
    }

    #[test]
    fn the_window_writes_the_message_into_a_scratch_project_and_refuses_a_fixture() {
        let scratch = Scratch::new("compose-send");
        let mut board = board(&scratch);
        board.fixtures = false;
        board.portfolio = crate::portfolio::read_files(board.store());
        // Opened on a fake project: the next id from its inbox, nothing written.
        board.open_compose("harbor");
        let dialog = board.compose.as_ref().expect("the dialog");
        assert_eq!(dialog.project, "harbor");
        assert!(dialog.next_id > 1, "{}", dialog.next_id);
        assert_eq!(dialog.busy, None);
        board.open_compose("nowhere");
        assert!(board.error_text().is_some_and(|e| e.contains("nowhere")));
        // Sent into a scratch project of its own.
        let folder = scratch.0.join("meadow");
        fs::create_dir_all(folder.join("roadmap")).unwrap();
        let mut dialog = Dialog::new(
            "meadow",
            folder.clone(),
            1,
            Some("a session works there".into()),
        );
        dialog.text = "Note the  trap.".to_owned();
        dialog.urgent = true;
        board.compose = Some(dialog);
        board.send_message();
        assert!(board.compose.as_ref().unwrap().is_sending());
        let started = Instant::now();
        while board.compose.is_some() && started.elapsed() < Duration::from_secs(20) {
            thread::sleep(Duration::from_millis(20));
            board.compose_pass();
        }
        assert!(board.compose.is_none(), "closed once written");
        let notice = board.notice_text().unwrap_or_default();
        assert!(
            notice.starts_with("Sent IN-1 to meadow; roadmap/inbox.md created with its header."),
            "{notice}"
        );
        let text = fs::read_to_string(inbox::path(&folder)).unwrap();
        assert!(
            text.contains("\n- [ ] IN-1 (from the owner, ")
                && text.ends_with(") Note the trap. [urgent]\n"),
            "{text}"
        );
        // A screenshot run's Send is refused in the dialog, the file untouched.
        board.fixtures = true;
        let mut dialog = Dialog::new("meadow", folder.clone(), 2, None);
        dialog.text = "Again.".to_owned();
        board.compose = Some(dialog);
        board.send_message();
        let dialog = board.compose.as_ref().unwrap();
        assert!(!dialog.is_sending());
        assert!(
            dialog
                .error
                .as_deref()
                .is_some_and(|e| e.contains("A screenshot run writes nothing")),
            "{:?}",
            dialog.error
        );
        assert_eq!(fs::read_to_string(inbox::path(&folder)).unwrap(), text);
    }
}
